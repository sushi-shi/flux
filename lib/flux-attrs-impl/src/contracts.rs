//! Rust-shaped contracts lowered to the existing refinement signature language.
//! Unsupported expressions are errors, never omitted proof obligations.

use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Expr, FnArg, ItemFn, Pat, ReturnType, Type, spanned::Spanned};

#[derive(Clone, Copy)]
enum ValueKind {
    Scalar,
    Str,
    Slice,
}

struct Value {
    index: Ident,
    kind: ValueKind,
}

pub fn expand(name: &str, attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let mut item: ItemFn = syn::parse2(item)?;
    if item.sig.asyncness.is_some() || !item.sig.generics.params.is_empty() {
        return Err(syn::Error::new(
            item.sig.span(),
            "readable contracts do not yet support async or generic functions",
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
    let mut values = HashMap::new();
    let mut inputs = Vec::new();
    for input in &item.sig.inputs {
        let FnArg::Typed(input) = input else {
            return Err(syn::Error::new(
                input.span(),
                "readable contracts do not yet support self receivers",
            ));
        };
        let Pat::Ident(pat) = &*input.pat else {
            return Err(syn::Error::new(input.pat.span(), "contract parameters must be named"));
        };
        if pat.by_ref.is_some() || pat.subpat.is_some() || pat.ident == "result" {
            return Err(syn::Error::new(
                pat.span(),
                "unsupported parameter pattern or reserved name `result`",
            ));
        }
        let index = format_ident!("__flux_contract_{}", pat.ident, span = pat.ident.span());
        let (ty, kind) = indexed_type(&input.ty, &index, true)?;
        values.insert(pat.ident.to_string(), Value { index, kind });
        inputs.push(ty);
    }
    let pre = conjunction(&requires, &values)?;
    let output = match &item.sig.output {
        ReturnType::Default => quote!(),
        ReturnType::Type(_, ty) if matches!(&**ty, Type::Tuple(t) if t.elems.is_empty()) => {
            quote!()
        }
        ReturnType::Type(_, ty) => {
            let index = format_ident!("__flux_contract_result", span = ty.span());
            let (ty, kind) = indexed_type(ty, &index, false)?;
            values.insert("result".into(), Value { index, kind });
            quote!(-> #ty)
        }
    };
    let post = conjunction(&ensures, &values)?;
    Ok(quote! {
        #[flux_tool::sig(fn(#(#inputs),*) #output requires #pre ensures #post)]
        #item
    })
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
        Type::Path(p)
            if p.qself.is_none() && p.path.segments.iter().all(|s| s.arguments.is_empty()) =>
        {
            let kind = if p.path.is_ident("str") { ValueKind::Str } else { ValueKind::Scalar };
            Ok((quote!(#ty[#binder]), kind))
        }
        _ => {
            Err(syn::Error::new(
                ty.span(),
                "unsupported contract type; use scalars, reflected enums, or immutable strings/slices",
            ))
        }
    }
}

fn conjunction(exprs: &[Expr], values: &HashMap<String, Value>) -> syn::Result<TokenStream> {
    let exprs = exprs
        .iter()
        .map(|e| lower(e, values))
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(quote!(true #(&& (#exprs))*))
}

fn lower(expr: &Expr, values: &HashMap<String, Value>) -> syn::Result<TokenStream> {
    match expr {
        Expr::Path(p) => {
            if let Some(name) = p.path.get_ident() {
                if let Some(value) = values.get(&name.to_string()) {
                    if matches!(value.kind, ValueKind::Slice) {
                        return Err(syn::Error::new(
                            p.span(),
                            "slice contents are not modeled; use .len() or .is_empty() explicitly",
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
        Expr::Lit(l)
            if matches!(
                l.lit,
                syn::Lit::Int(_) | syn::Lit::Bool(_) | syn::Lit::Str(_) | syn::Lit::Char(_)
            ) =>
        {
            Ok(quote!(#l))
        }
        Expr::Paren(p) => {
            let e = lower(&p.expr, values)?;
            Ok(quote!((#e)))
        }
        Expr::Unary(u) if matches!(u.op, syn::UnOp::Not(_) | syn::UnOp::Neg(_)) => {
            let op = &u.op;
            let e = lower(&u.expr, values)?;
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
            let left = lower(&b.left, values)?;
            let right = lower(&b.right, values)?;
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
            let text = lower(&m.receiver, values)?;
            let part = lower(&m.args[0], values)?;
            if m.method == "starts_with" {
                Ok(quote!(str_prefix_of(#part, #text)))
            } else {
                Ok(quote!(str_suffix_of(#part, #text)))
            }
        }
        Expr::MethodCall(m) if m.args.is_empty() && m.turbofish.is_none() => {
            let Expr::Path(p) = &*m.receiver else { return unsupported(expr) };
            let Some(value) = p.path.get_ident().and_then(|i| values.get(&i.to_string())) else {
                return unsupported(expr);
            };
            let index = &value.index;
            let len = match value.kind {
                ValueKind::Str => quote!(flux_core::str::byte_len(#index)),
                ValueKind::Slice => quote!(#index),
                ValueKind::Scalar => return unsupported(expr),
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
            let scrutinee = lower(&scrutinee, values)?;
            lower_pattern(&pat, &scrutinee)
        }
        _ => unsupported(expr),
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
}
