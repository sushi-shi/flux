//! Optional return payload predicates, without executing the user's closure.
//!
//! Each tuple component retains its own refinement type. Cross-component
//! predicates require dependent tuple types and are rejected explicitly.

use super::*;

pub(super) fn is_option_path(path: &syn::Path) -> bool {
    let names: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    (names == ["Option"]
        || names == ["core", "option", "Option"]
        || names == ["std", "option", "Option"])
        && matches!(
            &path.segments.last().unwrap().arguments,
            syn::PathArguments::AngleBracketed(args)
                if args.args.len() == 1 && matches!(args.args[0], syn::GenericArgument::Type(_))
        )
}

struct Payload<'a> {
    ty: &'a Type,
    children: Option<Vec<Payload<'a>>>,
    index: Ident,
    predicates: Vec<TokenStream>,
}

impl<'a> Payload<'a> {
    fn new(ty: &'a Type, next: &mut usize) -> Self {
        let index = format_ident!("__flux_payload_{}", *next, span = ty.span());
        *next += 1;
        let children = if let Type::Tuple(tuple) = ty {
            Some(tuple.elems.iter().map(|ty| Self::new(ty, next)).collect())
        } else {
            None
        };
        Self { ty, children, index, predicates: vec![] }
    }

    fn bind(&self, pat: &Pat, bindings: &mut HashMap<String, Value>) -> syn::Result<()> {
        match pat {
            Pat::Wild(_) => Ok(()),
            Pat::Ident(p)
                if p.by_ref.is_none() && p.subpat.is_none() && self.children.is_none() =>
            {
                let (_, kind) = indexed_type(self.ty, &self.index, false)?;
                let value = Value { index: self.index.clone(), kind };
                if bindings.insert(p.ident.to_string(), value).is_some() {
                    return Err(syn::Error::new(p.span(), "duplicate payload binding"));
                }
                Ok(())
            }
            Pat::Tuple(p)
                if self
                    .children
                    .as_ref()
                    .is_some_and(|c| c.len() == p.elems.len()) =>
            {
                for (child, pat) in self.children.as_ref().unwrap().iter().zip(&p.elems) {
                    child.bind(pat, bindings)?;
                }
                Ok(())
            }
            _ => {
                Err(syn::Error::new(
                    pat.span(),
                    "use a payload name, _, or a matching tuple pattern in is_none_or",
                ))
            }
        }
    }

    fn constrain(&mut self, index: &Ident, predicate: TokenStream) {
        if &self.index == index {
            self.predicates.push(predicate);
        } else if let Some(children) = &mut self.children {
            for child in children {
                child.constrain(index, predicate.clone());
            }
        }
    }

    fn tokens(&self) -> TokenStream {
        let ty = if let Some(children) = &self.children {
            let children = children.iter().map(Self::tokens);
            quote!((#(#children,)*))
        } else {
            self.ty.to_token_stream()
        };
        let predicates = &self.predicates;
        if predicates.is_empty() {
            ty
        } else {
            let index = &self.index;
            // Match indexed_type's canonical standard-container spelling.
            // A user type named Vec/Result/Option must not inherit that model.
            let ty = canonical_type(self.ty);
            quote!(#ty{#index: true #(&& (#predicates))*})
        }
    }
}

fn canonical_type(ty: &Type) -> Type {
    match ty {
        Type::Reference(reference) => {
            let mut reference = reference.clone();
            reference.elem = Box::new(canonical_type(&reference.elem));
            Type::Reference(reference)
        }
        Type::Path(path) if path.qself.is_none() => {
            let args = &path.path.segments.last().unwrap().arguments;
            if is_vec_path(&path.path) {
                syn::parse_quote!(std::vec::Vec #args)
            } else if is_result_path(&path.path) {
                syn::parse_quote!(std::result::Result #args)
            } else if is_option_path(&path.path) {
                syn::parse_quote!(core::option::Option #args)
            } else {
                ty.clone()
            }
        }
        _ => ty.clone(),
    }
}

pub(super) fn output(
    ty: &Type,
    ensures: &mut Vec<Expr>,
    values: &HashMap<String, Value>,
    old_values: &HashMap<String, Value>,
) -> syn::Result<Option<TokenStream>> {
    let mut conditions = vec![];
    let mut retained = vec![];
    for condition in std::mem::take(ensures) {
        // Splitting a conjunction is valid; lifting a payload predicate out of
        // a disjunction or conditional would strengthen the user's contract.
        split_conjunction(condition, &mut conditions);
    }
    let mut payload_conditions = vec![];
    for condition in conditions {
        if let Expr::MethodCall(call) = &condition
            && call.method == "is_none_or"
            && matches!(&*call.receiver, Expr::Path(p) if p.path.is_ident("result"))
        {
            payload_conditions.push(call.clone());
        } else {
            retained.push(condition);
        }
    }
    *ensures = retained;
    if payload_conditions.is_empty() {
        return Ok(None);
    }
    let Type::Path(path) = ty else { return unsupported_type(ty) };
    if path.qself.is_some() || !is_option_path(&path.path) {
        return unsupported_type(ty);
    }
    let syn::PathArguments::AngleBracketed(args) = &path.path.segments.last().unwrap().arguments
    else {
        unreachable!()
    };
    let syn::GenericArgument::Type(inner) = &args.args[0] else { unreachable!() };
    let mut payload = Payload::new(inner, &mut 0);
    let mut outer = vec![];
    for call in payload_conditions {
        if call.turbofish.is_some() || call.args.len() != 1 {
            return Err(syn::Error::new(call.span(), "is_none_or takes one predicate closure"));
        }
        let Expr::Closure(closure) = &call.args[0] else {
            return Err(syn::Error::new(call.span(), "is_none_or requires a predicate closure"));
        };
        if closure.inputs.len() != 1
            || closure.asyncness.is_some()
            || closure.constness.is_some()
            || closure.movability.is_some()
            || closure.lifetimes.is_some()
            || !matches!(closure.output, ReturnType::Default)
        {
            return Err(syn::Error::new(
                closure.span(),
                "use a pure one-parameter payload closure",
            ));
        }
        let mut bindings = HashMap::new();
        payload.bind(&closure.inputs[0], &mut bindings)?;
        // old(x) always refers to a function input, never a closure binding
        // that happens to shadow it. Reject that ambiguous spelling.
        let mut entry = old_values.clone();
        for name in bindings.keys() {
            entry.remove(name);
        }
        let mut scoped = values.clone();
        scoped.extend(bindings.clone());
        let mut clauses = vec![];
        split_conjunction((*closure.body).clone(), &mut clauses);
        for clause in clauses {
            let used = bindings
                .iter()
                .filter(|(name, _)| {
                    mentions(clause.to_token_stream(), &Ident::new(name, clause.span()))
                })
                .map(|(_, value)| &value.index)
                .collect::<Vec<_>>();
            if used.len() > 1 {
                return Err(syn::Error::new(
                    clause.span(),
                    "correlated tuple payload conditions are not yet supported; each conjunct may reference one component",
                ));
            }
            let predicate = lower(&clause, &scoped, Some(&entry))?;
            if let Some(index) = used.first() {
                payload.constrain(index, predicate);
            } else {
                outer.push(predicate);
            }
        }
    }
    let payload = payload.tokens();
    Ok(Some(if outer.is_empty() { payload } else { quote!({#payload | true #(&& (#outer))*}) }))
}

fn split_conjunction(expr: Expr, out: &mut Vec<Expr>) {
    match expr {
        Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
            split_conjunction(*binary.left, out);
            split_conjunction(*binary.right, out);
        }
        Expr::Paren(p) => split_conjunction(*p.expr, out),
        Expr::Block(block)
            if block.label.is_none()
                && matches!(block.block.stmts.as_slice(), [syn::Stmt::Expr(_, None)]) =>
        {
            let syn::Stmt::Expr(expr, _) = block.block.stmts.into_iter().next().unwrap() else {
                unreachable!()
            };
            split_conjunction(expr, out);
        }
        expr => out.push(expr),
    }
}

fn unsupported_type<T>(ty: &Type) -> syn::Result<T> {
    Err(syn::Error::new(ty.span(), "result.is_none_or requires a standard Option return type"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_payload_forms_are_errors() {
        for condition in [
            quote!(result.is_none_or(|(x, y)| x < y)),
            quote!(result.is_none_or(|(x, _)| old(x) == 0)),
            quote!(result.is_none_or(|(x, x)| x == 0)),
            quote!(result.is_none_or(|pair| pair.0 == 0)),
            quote!(result.is_none_or(|(x, _)| {
                panic!();
                x == 0
            })),
            quote!(result.is_none_or(|(x, _)| x == 0) || flag),
            quote!(result.is_none_or(predicate)),
        ] {
            assert!(
                expand(
                    "ensures",
                    condition,
                    quote!(
                        fn f(flag: bool) -> Option<(usize, usize)> {
                            None
                        }
                    )
                )
                .is_err()
            );
        }
    }

    #[test]
    fn payload_contract_requires_option_return_type() {
        assert!(
            expand(
                "ensures",
                quote!(result.is_none_or(|x| x > 0)),
                quote!(
                    fn f() -> usize {
                        1
                    }
                )
            )
            .is_err()
        );
    }
}
