//! Infer a named struct's refinement record from its Rust fields.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Fields, Ident, ItemStruct, Token, Type, parse::Parser, parse_quote, punctuated::Punctuated,
    spanned::Spanned,
};

pub fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let selected = Punctuated::<Ident, Token![,]>::parse_terminated.parse2(attr)?;
    let mut item: ItemStruct = syn::parse2(item)?;
    if !item.generics.params.is_empty() && selected.is_empty() {
        return Err(syn::Error::new(
            item.generics.span(),
            "generic structs require an explicit refined(field, ...) selection",
        ));
    }
    if item.attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|s| s.ident == "refined_by")
    }) {
        return Err(syn::Error::new(item.span(), "do not combine refined with refined_by"));
    }
    let Fields::Named(fields) = &mut item.fields else {
        return Err(syn::Error::new(item.span(), "refined requires named fields"));
    };
    let mut indices = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for name in &selected {
        if !seen.insert(name.to_string())
            || !fields
                .named
                .iter()
                .any(|field| field.ident.as_ref() == Some(name))
        {
            return Err(syn::Error::new(name.span(), "duplicate or unknown refined field"));
        }
    }
    for field in &mut fields.named {
        if field.attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .is_some_and(|s| s.ident == "field")
        }) {
            return Err(syn::Error::new(
                field.span(),
                "refined infers field annotations; remove the explicit field annotation",
            ));
        }
        let name = field.ident.as_ref().unwrap();
        if !selected.is_empty() && !seen.contains(&name.to_string()) {
            continue;
        }
        let (sort, ty) = field_model(&field.ty)?;
        indices.push(quote!(#name: #sort));
        field
            .attrs
            .push(parse_quote!(#[flux_tool::field(#ty[#name])]));
    }
    Ok(quote! {
        #[flux_tool::refined_by(#(#indices),*)]
        #item
    })
}

fn field_model(ty: &Type) -> syn::Result<(TokenStream, TokenStream)> {
    if let Type::Path(p) = ty
        && p.qself.is_none()
    {
        let names: Vec<_> = p
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        let last = p.path.segments.last().unwrap();
        if last.arguments.is_empty() {
            if names.len() == 1 && last.ident == "bool" {
                return Ok((quote!(bool), quote!(bool)));
            }
            if names.len() == 1
                && [
                    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128",
                    "isize",
                ]
                .contains(&names[0].as_str())
            {
                return Ok((quote!(int), quote!(#ty)));
            }
            // Named refined structs and reflected enums retain their own sort.
            return Ok((quote!(#ty), quote!(#ty)));
        }
        let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
            return Err(syn::Error::new(ty.span(), "unsupported refined field type"));
        };
        if args
            .args
            .iter()
            .all(|arg| matches!(arg, syn::GenericArgument::Type(_)))
        {
            if args.args.len() == 1
                && (names == ["Option"]
                    || names == ["std", "option", "Option"]
                    || names == ["core", "option", "Option"])
            {
                return Ok((quote!(core::option::Option), quote!(core::option::Option #args)));
            }
            if args.args.len() == 1
                && (names == ["Vec"]
                    || names == ["std", "vec", "Vec"]
                    || names == ["alloc", "vec", "Vec"])
            {
                return Ok((quote!(std::vec::Vec), quote!(std::vec::Vec #args)));
            }
            if args.args.len() == 2
                && (names == ["BTreeMap"]
                    || names == ["std", "collections", "BTreeMap"]
                    || names == ["alloc", "collections", "BTreeMap"])
            {
                return Ok((
                    quote!(std::collections::BTreeMap),
                    quote!(std::collections::BTreeMap #args),
                ));
            }
        }
    }
    Err(syn::Error::new(
        ty.span(),
        "unsupported refined field type; use scalars, named refined types, Vec<T>, or BTreeMap<K, V>",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_fields_fail_instead_of_becoming_opaque() {
        for item in [
            quote!(
                struct S {
                    p: *const u8,
                }
            ),
            quote!(
                struct S<T> {
                    x: T,
                }
            ),
            quote!(struct S(u32)),
            quote!(
                struct S {
                    x: Vec<u8, Allocator>,
                }
            ),
        ] {
            assert!(expand(quote!(), item).is_err());
        }
    }

    #[test]
    fn explicit_models_cannot_be_silently_replaced() {
        for item in [
            quote!(
                #[flux::refined_by(n: int)]
                struct S {
                    n: u32,
                }
            ),
            quote!(
                struct S {
                    #[flux::field(u32[0])]
                    n: u32,
                }
            ),
        ] {
            assert!(expand(quote!(), item).is_err());
        }
    }
}
