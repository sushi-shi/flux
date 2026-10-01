//! Rust-shaped contracts lowered to the existing refinement signature language.
//! Unsupported expressions are errors, never omitted proof obligations.

mod option_payload;

use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{Expr, FnArg, ItemFn, Pat, ReturnType, Type, spanned::Spanned};

#[derive(Clone, Copy)]
enum ValueKind {
    Opaque,
    Scalar,
    Str,
    Slice,
    Result,
    Option,
}

#[derive(Clone)]
struct Value {
    index: Ident,
    kind: ValueKind,
}

fn mentions(tokens: TokenStream, name: &Ident) -> bool {
    tokens.into_iter().any(|token| {
        match token {
            TokenTree::Ident(ident) => ident == *name,
            TokenTree::Group(group) => mentions(group.stream(), name),
            _ => false,
        }
    })
}

pub fn expand(name: &str, attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let mut item: ItemFn = syn::parse2(item)?;
    if item.sig.asyncness.is_some() {
        return Err(syn::Error::new(
            item.sig.span(),
            "readable contracts do not yet support async functions",
        ));
    }
    let mut requires = Vec::new();
    let mut ensures = Vec::new();
    let first: Expr = syn::parse2(attr)?;
    if name == "requires" {
        requires.push(first);
    } else {
        ensures.push(first);
    }
    let mut kept = Vec::new();
    for attr in std::mem::take(&mut item.attrs) {
        let path = attr.path();
        let last = path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        let own = path.segments.len() == 1
            || path.segments.first().is_some_and(|s| {
                ["flux_attrs", "flux_rs", "flux_core"]
                    .iter()
                    .any(|p| s.ident == p)
            });
        if own && (last == "requires" || last == "ensures") {
            let expr = attr.parse_args()?;
            if last == "requires" {
                requires.push(expr);
            } else {
                ensures.push(expr);
            }
        } else if ["sig", "spec"].contains(&last.as_str()) {
            return Err(syn::Error::new(
                attr.span(),
                "do not combine readable contracts with a refinement signature",
            ));
        } else {
            kept.push(attr);
        }
    }
    item.attrs = kept;
    let field_conditions = requires.iter().chain(&ensures).cloned().collect::<Vec<_>>();
    let mut values = HashMap::new();
    let mut after_values = HashMap::new();
    let mut inputs = Vec::new();
    let mut frames = Vec::new();
    for (position, input) in item.sig.inputs.iter().enumerate() {
        let (name, ty) = match input {
            FnArg::Typed(input) => {
                let Pat::Ident(pat) = &*input.pat else {
                    return Err(syn::Error::new(
                        input.pat.span(),
                        "contract parameters must be named",
                    ));
                };
                if pat.by_ref.is_some() || pat.subpat.is_some() || pat.ident == "result" {
                    return Err(syn::Error::new(
                        pat.span(),
                        "unsupported parameter pattern or reserved name `result`",
                    ));
                }
                (pat.ident.clone(), &*input.ty)
            }
            FnArg::Receiver(receiver) => {
                if receiver.colon_token.is_some() {
                    return Err(syn::Error::new(receiver.span(), "use self, &self, or &mut self"));
                }
                (Ident::new("self", receiver.self_token.span), &*receiver.ty)
            }
        };
        let index = format_ident!("__flux_contract_input_{}", position, span = name.span());
        let (ty, kind) = if let Type::Reference(r) = ty
            && r.mutability.is_some()
        {
            let (before, kind) = indexed_type(&r.elem, &index, true)?;
            let after_index =
                format_ident!("__flux_contract_after_{}", position, span = name.span());
            let (after, _) = indexed_type(&r.elem, &after_index, false)?;
            after_values.insert(name.to_string(), Value { index: after_index, kind });
            frames.push(quote!(#name: #after));
            (quote!(#name: &strg #before), kind)
        } else if !requires
            .iter()
            .chain(&ensures)
            .any(|expr| mentions(expr.to_token_stream(), &name))
        {
            // A parameter absent from the contract needs no refinement binder.
            // In particular, generic payloads can stay opaque instead of being
            // incorrectly treated as base-sort parameters.
            (quote!(#name: #ty), ValueKind::Opaque)
        } else {
            let (ty, kind) = indexed_type(ty, &index, true)?;
            (quote!(#name: #ty), kind)
        };
        values.insert(name.to_string(), Value { index, kind });
        inputs.push(ty);
    }
    let pre = conjunction(&requires, &values, None)?;
    let old_values = values.clone();
    values.extend(after_values);
    let output = match &item.sig.output {
        ReturnType::Default => quote!(),
        ReturnType::Type(_, ty) if matches!(&**ty, Type::Tuple(t) if t.elems.is_empty()) => {
            quote!()
        }
        ReturnType::Type(_, ty)
            if !ensures
                .iter()
                .any(|expr| mentions(expr.to_token_stream(), &Ident::new("result", ty.span()))) =>
        {
            // Unmentioned generic return values need no refinement binder.
            quote!(-> #ty)
        }
        ReturnType::Type(_, ty) => {
            let index = format_ident!("__flux_contract_result", span = ty.span());
            let (ty, kind) = if let Some(payload) =
                option_payload::output(ty, &mut ensures, &values, &old_values)?
            {
                (quote!(core::option::Option<#payload>[# #index]), ValueKind::Option)
            } else {
                indexed_type(ty, &index, false)?
            };
            values.insert("result".into(), Value { index, kind });
            quote!(-> #ty)
        }
    };
    let post = conjunction(&ensures, &values, Some(&old_values))?;
    check_field_types(&mut item, field_conditions.iter())?;
    Ok(quote! {
        #[flux_tool::sig(fn(#(#inputs),*) #output requires #pre ensures #(#frames,)* #post)]
        #item
    })
}

// Logical single-field records can coerce through their scalar index. A typed
// logical Vec projection alone therefore cannot establish that a Rust receiver
// is actually Vec. Ask rustc to check that fact in a dead block; the sealed local
// marker also prevents Deref coercions from accepting a custom len method.
fn check_field_types<'a>(
    item: &mut ItemFn,
    exprs: impl Iterator<Item = &'a Expr>,
) -> syn::Result<()> {
    use syn::visit_mut::{self, VisitMut};
    #[derive(Default)]
    struct Fields(Vec<(Expr, bool)>);
    impl VisitMut for Fields {
        fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
            if call.args.is_empty()
                && call.turbofish.is_none()
                && ["len", "is_empty", "is_some", "is_none"]
                    .iter()
                    .any(|m| call.method == *m)
                && matches!(&*call.receiver, Expr::Field(_))
            {
                self.0.push((
                    (*call.receiver).clone(),
                    call.method == "is_some" || call.method == "is_none",
                ));
            }
            visit_mut::visit_expr_method_call_mut(self, call);
        }
    }
    struct StripOld;
    impl VisitMut for StripOld {
        fn visit_expr_mut(&mut self, expr: &mut Expr) {
            if let Expr::Call(call) = expr
                && matches!(&*call.func, Expr::Path(p) if p.path.is_ident("old"))
                && call.args.len() == 1
            {
                *expr = call.args[0].clone();
            }
            visit_mut::visit_expr_mut(self, expr);
        }
    }
    let mut fields = Fields::default();
    for expr in exprs {
        fields.visit_expr_mut(&mut expr.clone());
    }
    if fields.0.is_empty() {
        return Ok(());
    }
    let mut params = Vec::new();
    let mut results = Vec::new();
    let has_vec = fields.0.iter().any(|(_, option)| !option);
    let has_option = fields.0.iter().any(|(_, option)| *option);
    for (mut receiver, option) in fields.0 {
        StripOld.visit_expr_mut(&mut receiver);
        let check = if option {
            quote!(__flux_check_option_field_type(&#receiver);)
        } else {
            quote!(__flux_check_vec_field_type(&#receiver);)
        };
        if mentions(receiver.to_token_stream(), &Ident::new("result", receiver.span())) {
            results.push(check);
        } else {
            params.push(check);
        }
    }
    let result_check = if results.is_empty() {
        quote!()
    } else {
        let ReturnType::Type(_, ty) = &item.sig.output else { unreachable!() };
        quote!(let _ = |result: #ty| { #(#results)* };)
    };
    let vec_witness = has_vec.then(|| {
        quote! {
                trait __FluxVecFieldType {}
                impl<T> __FluxVecFieldType for std::vec::Vec<T> {}
                const fn __flux_check_vec_field_type<T: __FluxVecFieldType>(_: &T) {}
        }
    });
    let option_witness = has_option.then(|| {
        quote! {
                trait __FluxOptionFieldType {}
                impl<T> __FluxOptionFieldType for core::option::Option<T> {}
                const fn __flux_check_option_field_type<T: __FluxOptionFieldType>(_: &T) {}
        }
    });
    let check = syn::parse2::<syn::Stmt>(quote! {
        #[allow(dead_code, unused_variables, unreachable_code)]
        if false {
            #vec_witness
            #option_witness
            #(#params)*
            #result_check
        }
    })?;
    item.block.stmts.insert(0, check);
    Ok(())
}

fn indexed_type(ty: &Type, index: &Ident, input: bool) -> syn::Result<(TokenStream, ValueKind)> {
    let binder = if input { quote!(@#index) } else { quote!(# #index) };
    match ty {
        Type::Reference(r) if r.mutability.is_none() => {
            let (inner, kind) = indexed_type(&r.elem, index, input)?;
            Ok((quote!(&#inner), kind))
        }
        Type::Slice(s) => {
            let elem = &s.elem;
            Ok((quote!([#elem][#binder]), ValueKind::Slice))
        }
        Type::Path(p) if p.qself.is_none() && is_result_path(&p.path) => {
            let args = &p.path.segments.last().unwrap().arguments;
            // Canonical spelling prevents a user-defined Result from acquiring
            // the standard discriminant model merely by its name.
            Ok((quote!(std::result::Result #args [#binder]), ValueKind::Result))
        }
        Type::Path(p) if p.qself.is_none() && is_vec_path(&p.path) => {
            let args = &p.path.segments.last().unwrap().arguments;
            Ok((quote!(std::vec::Vec #args [#binder]), ValueKind::Slice))
        }
        Type::Path(p) if p.qself.is_none() && option_payload::is_option_path(&p.path) => {
            let args = &p.path.segments.last().unwrap().arguments;
            Ok((quote!(core::option::Option #args [#binder]), ValueKind::Option))
        }
        Type::Path(p) if p.qself.is_none() => {
            let kind = if p.path.is_ident("str") { ValueKind::Str } else { ValueKind::Scalar };
            Ok((quote!(#ty[#binder]), kind))
        }
        _ => {
            Err(syn::Error::new(
                ty.span(),
                "unsupported contract type; use scalars, refined structs/enums, Result, Vec, or strings/slices",
            ))
        }
    }
}

fn is_result_path(path: &syn::Path) -> bool {
    let names: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    let standard = names == ["Result"]
        || names == ["core", "result", "Result"]
        || names == ["std", "result", "Result"];
    standard
        && matches!(
            &path.segments.last().unwrap().arguments,
            syn::PathArguments::AngleBracketed(args)
                if args.args.len() == 2 && args.args.iter().all(|a| matches!(a, syn::GenericArgument::Type(_)))
        )
}

fn is_vec_path(path: &syn::Path) -> bool {
    let names: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    let standard =
        names == ["Vec"] || names == ["alloc", "vec", "Vec"] || names == ["std", "vec", "Vec"];
    standard
        && matches!(
            &path.segments.last().unwrap().arguments,
            syn::PathArguments::AngleBracketed(args)
                if args.args.len() == 1 && args.args.iter().all(|a| matches!(a, syn::GenericArgument::Type(_)))
        )
}

fn conjunction(
    exprs: &[Expr],
    values: &HashMap<String, Value>,
    old_values: Option<&HashMap<String, Value>>,
) -> syn::Result<TokenStream> {
    let exprs = exprs
        .iter()
        .map(|e| lower(e, values, old_values))
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(quote!(true #(&& (#exprs))*))
}

fn lower(
    expr: &Expr,
    values: &HashMap<String, Value>,
    old_values: Option<&HashMap<String, Value>>,
) -> syn::Result<TokenStream> {
    match expr {
        Expr::Path(p) => {
            if let Some(name) = p.path.get_ident() {
                if let Some(value) = values.get(&name.to_string()) {
                    if matches!(
                        value.kind,
                        ValueKind::Slice
                            | ValueKind::Result
                            | ValueKind::Option
                            | ValueKind::Opaque
                    ) {
                        return Err(syn::Error::new(
                            p.span(),
                            "contents are not modeled; use length or discriminant methods explicitly",
                        ));
                    }
                    let index = &value.index;
                    return Ok(quote!(#index));
                }
                return Err(syn::Error::new(
                    p.span(),
                    "unknown contract parameter (or `result` used in a precondition)",
                ));
            }
            Ok(quote!(#p))
        }
        Expr::Call(call) if matches!(&*call.func, Expr::Path(p) if p.path.is_ident("old")) => {
            let Some(entry_values) = old_values else {
                return Err(syn::Error::new(
                    call.span(),
                    "old(...) is only allowed in postconditions and cannot be nested",
                ));
            };
            if call.args.len() != 1 {
                return Err(syn::Error::new(call.span(), "old(...) takes exactly one expression"));
            }
            // Entry bindings contain no result; clearing old_values rejects nesting.
            lower(&call.args[0], entry_values, None)
        }
        Expr::Call(call) if matches!(&*call.func, Expr::Path(p) if p.path.is_ident("concat")) => {
            if call.args.len() != 2 {
                return Err(syn::Error::new(call.span(), "concat(...) takes two string models"));
            }
            let left = lower(&call.args[0], values, old_values)?;
            let right = lower(&call.args[1], values, old_values)?;
            Ok(quote!(str_concat(#left, #right)))
        }
        Expr::If(branch) => {
            let Some((_, otherwise)) = &branch.else_branch else { return unsupported(expr) };
            let condition = lower(&branch.cond, values, old_values)?;
            let then = lower_block(&branch.then_branch, values, old_values)?;
            let otherwise = match &**otherwise {
                Expr::Block(block) if block.label.is_none() => {
                    lower_block(&block.block, values, old_values)?
                }
                Expr::If(_) => lower(otherwise, values, old_values)?,
                _ => return unsupported(expr),
            };
            Ok(quote!(if #condition { #then } else { #otherwise }))
        }
        Expr::Lit(l)
            if matches!(
                l.lit,
                syn::Lit::Int(_) | syn::Lit::Bool(_) | syn::Lit::Str(_) | syn::Lit::Char(_)
            ) =>
        {
            Ok(quote!(#l))
        }
        Expr::Paren(p) => {
            let e = lower(&p.expr, values, old_values)?;
            Ok(quote!((#e)))
        }
        Expr::Field(field) => {
            let syn::Member::Named(member) = &field.member else {
                return unsupported(expr);
            };
            let base = lower(&field.base, values, old_values)?;
            Ok(quote!((#base).#member))
        }
        Expr::Unary(u) if matches!(u.op, syn::UnOp::Not(_) | syn::UnOp::Neg(_)) => {
            let op = &u.op;
            let e = lower(&u.expr, values, old_values)?;
            Ok(quote!(#op (#e)))
        }
        Expr::Binary(b)
            if !matches!(
                b.op,
                syn::BinOp::AddAssign(_)
                    | syn::BinOp::SubAssign(_)
                    | syn::BinOp::MulAssign(_)
                    | syn::BinOp::DivAssign(_)
                    | syn::BinOp::RemAssign(_)
                    | syn::BinOp::BitXorAssign(_)
                    | syn::BinOp::BitAndAssign(_)
                    | syn::BinOp::BitOrAssign(_)
                    | syn::BinOp::ShlAssign(_)
                    | syn::BinOp::ShrAssign(_)
            ) =>
        {
            let left = lower(&b.left, values, old_values)?;
            let right = lower(&b.right, values, old_values)?;
            let op = &b.op;
            // Partial arithmetic needs definedness checks, not an implicit assumption.
            if matches!(op, syn::BinOp::Div(_) | syn::BinOp::Rem(_))
                && !matches!(&*b.right, Expr::Lit(l) if matches!(&l.lit, syn::Lit::Int(i) if i.base10_parse::<u128>().is_ok_and(|n| n > 0)))
            {
                return Err(syn::Error::new(
                    b.right.span(),
                    "contract division currently requires a positive integer literal divisor",
                ));
            }
            if matches!(
                op,
                syn::BinOp::BitAnd(_)
                    | syn::BinOp::BitOr(_)
                    | syn::BinOp::BitXor(_)
                    | syn::BinOp::Shl(_)
                    | syn::BinOp::Shr(_)
            ) {
                return Err(syn::Error::new(
                    op.span(),
                    "bitwise contract expressions are not supported",
                ));
            }
            Ok(quote!((#left) #op (#right)))
        }
        Expr::MethodCall(m)
            if m.method == "is_char_boundary" && m.args.len() == 1 && m.turbofish.is_none() =>
        {
            let Expr::Path(receiver) = &*m.receiver else { return unsupported(expr) };
            if !receiver
                .path
                .get_ident()
                .and_then(|i| values.get(&i.to_string()))
                .is_some_and(|v| matches!(v.kind, ValueKind::Str))
            {
                return unsupported(expr);
            }
            let text = lower(&m.receiver, values, old_values)?;
            let offset = lower(&m.args[0], values, old_values)?;
            Ok(quote!(flux_core::str::boundary(#text, #offset)))
        }
        Expr::MethodCall(m)
            if m.args.len() == 1
                && m.turbofish.is_none()
                && (m.method == "starts_with" || m.method == "ends_with") =>
        {
            let Expr::Path(receiver) = &*m.receiver else { return unsupported(expr) };
            let Expr::Path(argument) = &m.args[0] else { return unsupported(expr) };
            let is_string = |p: &syn::ExprPath| {
                p.path
                    .get_ident()
                    .and_then(|i| values.get(&i.to_string()))
                    .is_some_and(|v| matches!(v.kind, ValueKind::Str))
            };
            if !is_string(receiver) || !is_string(argument) {
                return unsupported(expr);
            }
            let text = lower(&m.receiver, values, old_values)?;
            let part = lower(&m.args[0], values, old_values)?;
            if m.method == "starts_with" {
                Ok(quote!(str_prefix_of(#part, #text)))
            } else {
                Ok(quote!(str_suffix_of(#part, #text)))
            }
        }
        Expr::MethodCall(m) if m.args.is_empty() && m.turbofish.is_none() => {
            if matches!(&*m.receiver, Expr::Field(_))
                && (m.method == "is_none" || m.method == "is_some")
            {
                let receiver = lower(&m.receiver, values, old_values)?;
                let some = quote!(flux_core::option::model_is_some(#receiver));
                return if m.method == "is_some" { Ok(some) } else { Ok(quote!(!(#some))) };
            }
            if matches!(&*m.receiver, Expr::Field(_))
                && (m.method == "len" || m.method == "is_empty")
            {
                let receiver = lower(&m.receiver, values, old_values)?;
                let len = quote!(flux_alloc::vec::model_len(#receiver));
                return if m.method == "len" { Ok(len) } else { Ok(quote!((#len) == 0)) };
            }
            let Expr::Path(p) = &*m.receiver else { return unsupported(expr) };
            let Some(value) = p.path.get_ident().and_then(|i| values.get(&i.to_string())) else {
                return unsupported(expr);
            };
            let index = &value.index;
            if matches!(value.kind, ValueKind::Result) {
                return if m.method == "is_ok" {
                    Ok(quote!((#index).is_ok))
                } else if m.method == "is_err" {
                    Ok(quote!(!(#index).is_ok))
                } else {
                    unsupported(expr)
                };
            }
            if matches!(value.kind, ValueKind::Option) {
                return if m.method == "is_some" {
                    Ok(quote!((#index).is_some))
                } else if m.method == "is_none" {
                    Ok(quote!(!(#index).is_some))
                } else {
                    unsupported(expr)
                };
            }
            let len = match value.kind {
                ValueKind::Str => quote!(flux_core::str::byte_len(#index)),
                ValueKind::Slice => quote!(#index),
                ValueKind::Scalar | ValueKind::Result | ValueKind::Option | ValueKind::Opaque => {
                    return unsupported(expr);
                }
            };
            if m.method == "len" {
                Ok(len)
            } else if m.method == "is_empty" {
                Ok(quote!((#len) == 0))
            } else {
                unsupported(expr)
            }
        }
        Expr::Macro(m) if m.mac.path.is_ident("matches") => {
            use syn::parse::Parser;
            let parser = |input: syn::parse::ParseStream| {
                let scrutinee: Expr = input.parse()?;
                input.parse::<syn::Token![,]>()?;
                let pat = Pat::parse_multi_with_leading_vert(input)?;
                if !input.is_empty() {
                    return Err(
                        input.error("payload guards are not yet supported in contract patterns")
                    );
                }
                Ok((scrutinee, pat))
            };
            let (scrutinee, pat) = parser.parse2(m.mac.tokens.clone())?;
            let scrutinee = lower(&scrutinee, values, old_values)?;
            lower_pattern(&pat, &scrutinee)
        }
        _ => unsupported(expr),
    }
}

fn lower_block(
    block: &syn::Block,
    values: &HashMap<String, Value>,
    old_values: Option<&HashMap<String, Value>>,
) -> syn::Result<TokenStream> {
    if let [syn::Stmt::Expr(expr, None)] = block.stmts.as_slice() {
        lower(expr, values, old_values)
    } else {
        Err(syn::Error::new(
            block.span(),
            "contract branches must contain a single pure expression",
        ))
    }
}

fn lower_pattern(pat: &Pat, value: &TokenStream) -> syn::Result<TokenStream> {
    match pat {
        Pat::Path(path) => Ok(quote!((#value) == #path)),
        Pat::Or(or) => {
            let cases = or
                .cases
                .iter()
                .map(|p| lower_pattern(p, value))
                .collect::<syn::Result<Vec<_>>>()?;
            Ok(quote!(false #(|| (#cases))*))
        }
        Pat::Paren(p) => lower_pattern(&p.pat, value),
        _ => {
            Err(syn::Error::new(
                pat.span(),
                "contract patterns currently require qualified, payload-free reflected enum variants",
            ))
        }
    }
}

fn unsupported<T>(expr: &Expr) -> syn::Result<T> {
    Err(syn::Error::new(
        expr.span(),
        "unsupported contract expression; no obligation was generated",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_expressions_that_need_unimplemented_models() {
        for expr in [
            quote!(xs[0] == 5),
            quote!(xs.iter().all(|x| *x == 5)),
            quote!(n / n == 1),
            quote!(matches!(n, Some(x) if x > 0)),
        ] {
            assert!(
                expand(
                    "requires",
                    expr,
                    quote!(
                        fn f(xs: &[i32], n: u32) {}
                    )
                )
                .is_err()
            );
        }
    }

    #[test]
    fn result_cannot_name_a_parameter_or_appear_in_requires() {
        assert!(
            expand(
                "requires",
                quote!(result > 0),
                quote!(
                    fn f() -> u32 {
                        1
                    }
                )
            )
            .is_err()
        );
        assert!(
            expand(
                "ensures",
                quote!(result > 0),
                quote!(
                    fn f(result: u32) -> u32 {
                        result
                    }
                )
            )
            .is_err()
        );
    }

    #[test]
    fn forbids_conflicting_signatures() {
        assert!(
            expand(
                "ensures",
                quote!(result > 0),
                quote!(
                    #[flux::sig(fn() -> u32)]
                    fn f() -> u32 {
                        1
                    }
                )
            )
            .is_err()
        );
    }

    #[test]
    fn slice_equality_must_not_turn_into_length_equality() {
        assert!(
            expand(
                "requires",
                quote!(xs == ys),
                quote!(
                    fn f(xs: &[u8], ys: &[u8]) {}
                )
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_invalid_old_and_effectful_branches() {
        for expr in [
            quote!(old(result) == 0),
            quote!(old(old(value)) == value),
            quote!(old() == 0),
            quote!(old(value, value) == 0),
            quote!(
                if value > 0 {
                    let x = value;
                    x
                } else {
                    0
                } == value
            ),
        ] {
            assert!(
                expand(
                    "ensures",
                    expr,
                    quote!(
                        fn f(value: u32) -> u32 {
                            value
                        }
                    )
                )
                .is_err()
            );
        }
        assert!(
            expand(
                "requires",
                quote!(old(value) > 0),
                quote!(
                    fn f(value: u32) {}
                )
            )
            .is_err()
        );
    }

    #[test]
    fn result_equality_must_not_turn_into_discriminant_equality() {
        assert!(
            expand(
                "ensures",
                quote!(result == input),
                quote!(
                    fn f(input: Result<u32, u32>) -> Result<u32, u32> {
                        input
                    }
                )
            )
            .is_err()
        );
        assert!(
            expand(
                "ensures",
                quote!(result == input),
                quote!(
                    fn f(input: Vec<u8>) -> Vec<u8> {
                        input
                    }
                )
            )
            .is_err()
        );
    }
}
