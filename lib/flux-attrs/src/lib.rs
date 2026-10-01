#[cfg(not(flux_sysroot))]
use attr_dummy as attr_impl;
#[cfg(flux_sysroot)]
use attr_sysroot as attr_impl;
use proc_macro::TokenStream;

/// Permit panics while continuing to check functional contracts and invariants
/// on normal return. This does not suppress overflow checks or trust the body.
/// Use for APIs whose callbacks or allocation sizes can legitimately panic.
/// On trait method declarations this changes only the panic effect; an existing
/// explicit refinement signature is preserved.
#[proc_macro_attribute]
pub fn may_panic(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return "compile_error!(\"may_panic takes no arguments\");"
            .parse()
            .unwrap();
    }
    #[cfg(flux_sysroot)]
    {
        let checked = flux_attrs_impl::may_panic(tokens.clone().into()).into();
        when_checking(tokens, checked)
    }
    #[cfg(not(flux_sysroot))]
    {
        erase_when_not_checking(tokens)
    }
}

/// Infer a named struct's refinement record and field connections from its Rust types.
///
/// Supports named structs with scalar fields, named refined types, and
/// standard Option, default-allocator Vec and BTreeMap models (containers require std
/// except Option).
/// Invariants use the field names. Container models describe only their modeled
/// properties, not full contents; equality of models is not content equality.
/// Unsupported field types are errors during verification. Native builds erase
/// the attribute and preserve the original struct and its layout.
/// Generic structs select modeled fields explicitly: `#[refined(buffer, count)]`.
/// Other fields remain opaque and are not silently assumed preserved.
#[proc_macro_attribute]
pub fn refined(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    #[cfg(flux_sysroot)]
    {
        let checked = flux_attrs_impl::refined(attr.into(), tokens.clone().into()).into();
        when_checking(tokens, checked)
    }
    #[cfg(not(flux_sysroot))]
    {
        let _ = attr;
        erase_when_not_checking(tokens)
    }
}

/// A precondition written with parameter names and Rust-shaped expressions.
///
/// Stack with [`ensures`] in either order. The current frontend supports scalar
/// parameters, refined structs, reflected payload-free enums, strings, slices,
/// and default-allocator `Vec<T>` lengths;
/// `len`, `is_empty`, string `starts_with`/`ends_with`, and enum `matches!` patterns.
/// Shared and mutable receivers/parameters are supported, as are pure `if/else`
/// expressions. Standard `Result<T, E>` supports `is_ok()` and `is_err()`;
/// its payload is not modeled. Result and Vec lowering currently require `std`.
/// Arithmetic denotes mathematical integers, as in Flux refinement signatures;
/// it does not execute Rust arithmetic or wrap at machine bounds. Division and
/// remainder currently require a positive integer literal divisor.
/// Named fields project an aggregate's refinement record. Its field refinements
/// must connect those same names to the corresponding Rust fields.
///
/// Unsupported operations (including indexing, payload patterns, mutation,
/// arbitrary calls, and async signatures) are rejected during checking.
/// Generic payloads absent from the conditions remain opaque. Vec-valued field
/// len/is_empty calls use a typed standard Vec model; other field methods fail.
/// Option fields support is_some/is_none with a Rust type witness. String-slice
/// parameters and results support is_char_boundary(offset).
/// Outside Flux these attributes are erased without executing their expressions.
#[proc_macro_attribute]
pub fn requires(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::contract("requires", attr, tokens)
}

/// A postcondition; `result` refers to the function's returned value.
///
/// Mutable-reference parameters refer to their state on return; `old(expr)`
/// refers to entry state. Other parameters retain their entry values. State
/// preservation must be specified explicitly: omitted fields are not assumed
/// unchanged. `old` cannot be nested or used in a precondition, and cannot refer
/// to `result`. Unit-returning lemmas may establish
/// facts about their parameters without mentioning `result`. The lemma body is
/// checked normally; this attribute never makes the function trusted.
/// `concat(left, right)` denotes concatenation of two string models, for
/// example `ensures(buffer.text == concat(old(buffer.text), appended))`.
///
/// Standard Option results support `result.is_none_or(|value| predicate)`.
/// Tuple payloads use destructuring, for example
/// `result.is_none_or(|(_, index)| index < limit)`. Each conjunct may refer to
/// one tuple component and function parameters; correlated component predicates
/// are rejected. None satisfies the payload condition; add `result.is_some()`
/// when presence is required. Payload predicates must be top-level conjuncts,
/// not nested inside an alternative or conditional. Their closures are logical
/// expressions and never execute in a native build.
#[proc_macro_attribute]
pub fn ensures(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::contract("ensures", attr, tokens)
}

#[proc_macro_attribute]
pub fn alias(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::alias(attr, tokens)
}

#[proc_macro_attribute]
pub fn sig(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::sig(attr, tokens)
}

#[proc_macro_attribute]
pub fn spec(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::spec(attr, tokens)
}

#[proc_macro_attribute]
pub fn specs(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::specs(attr, tokens)
}

#[proc_macro_attribute]
pub fn qualifiers(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::qualifiers(attr, tokens)
}

#[proc_macro_attribute]
pub fn reveal(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::reveal(attr, tokens)
}

#[proc_macro_attribute]
pub fn refined_by(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::refined_by(attr, tokens)
}

#[proc_macro_attribute]
pub fn invariant(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::invariant(attr, tokens)
}

#[proc_macro_attribute]
pub fn constant(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::constant(attr, tokens)
}

#[proc_macro_attribute]
pub fn opaque(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::opaque(attr, tokens)
}

#[proc_macro_attribute]
pub fn reflect(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::reflect(attr, tokens)
}

#[proc_macro_attribute]
pub fn opts(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::opts(attr, tokens)
}

#[proc_macro_attribute]
pub fn trusted(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::trusted(attr, tokens)
}

#[proc_macro_attribute]
pub fn trusted_impl(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::trusted_impl(attr, tokens)
}

#[proc_macro_attribute]
pub fn trusted_derive(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::trusted_derive(attr, tokens)
}

#[proc_macro_attribute]
pub fn proven_externally(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::proven_externally(attr, tokens)
}

#[proc_macro_attribute]
pub fn generics(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::generics(attr, tokens)
}

#[proc_macro_attribute]
pub fn assoc(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::assoc(attr, tokens)
}

#[proc_macro]
pub fn flux(tokens: TokenStream) -> TokenStream {
    flux_attrs_impl::flux(tokens.into()).into()
}

#[proc_macro]
pub fn defs(tokens: TokenStream) -> TokenStream {
    attr_impl::defs(tokens)
}

#[proc_macro_attribute]
pub fn extern_spec(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::extern_spec(attrs, tokens)
}

#[proc_macro_attribute]
pub fn ignore(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::ignore(attrs, tokens)
}

#[proc_macro_attribute]
pub fn should_fail(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::should_fail(attrs, tokens)
}

#[proc_macro_attribute]
pub fn no_panic(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::no_panic(attrs, tokens)
}

#[proc_macro_attribute]
pub fn no_panic_if(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::no_panic_if(attrs, tokens)
}

#[proc_macro_attribute]
pub fn no_suggestions(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::no_suggestions(attrs, tokens)
}

#[proc_macro_attribute]
pub fn reft(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::reft(attrs, tokens)
}

#[proc_macro_attribute]
pub fn assume_parametric(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    attr_impl::assume_parametric(attrs, tokens)
}

#[cfg(not(flux_sysroot))]
fn erase_when_not_checking(item: TokenStream) -> TokenStream {
    // A manually invoked driver must not accept contracts erased by an ordinary
    // Cargo-built macro. cargo-flux rebuilds this crate with FLUX_BUILD_SYSROOT.
    let mut output: TokenStream = r#"
        #[cfg(flux)]
        compile_error!("native Flux attributes cannot be used for verification; use cargo flux or rebuild flux-attrs with FLUX_BUILD_SYSROOT=1");
    "#
    .parse()
    .unwrap();
    output.extend(item);
    output
}

#[cfg(flux_sysroot)]
fn when_checking(original: TokenStream, checked: TokenStream) -> TokenStream {
    // Cargo shares this host proc-macro between checked and unchecked target
    // crates. Select the expansion using the caller's cfg, not the macro's cfg.
    // Keep the original item intact: generated type witnesses and tool attributes
    // must not leak into dependencies that are not being verified.
    let mut output: TokenStream = "#[cfg(flux)]".parse().unwrap();
    output.extend(checked);
    output.extend("#[cfg(not(flux))]".parse::<TokenStream>().unwrap());
    output.extend(original);
    output
}

#[cfg(flux_sysroot)]
mod attr_sysroot {
    use super::*;

    pub fn contract(name: &str, attr: TokenStream, item: TokenStream) -> TokenStream {
        let checked = flux_attrs_impl::contract(name, attr.into(), item.clone().into()).into();
        when_checking(item, checked)
    }

    pub fn extern_spec(attr: TokenStream, tokens: TokenStream) -> TokenStream {
        flux_attrs_impl::extern_spec(attr.into(), tokens.into()).into()
    }

    pub fn refined_by(attr: TokenStream, item: TokenStream) -> TokenStream {
        flux_attrs_impl::refined_by(attr.into(), item.into()).into()
    }

    pub fn defs(tokens: TokenStream) -> TokenStream {
        flux_attrs_impl::defs(tokens.into()).into()
    }

    macro_rules! flux_tool_attrs {
        ($($name:ident),+ $(,)?) => {
            $(
            pub fn $name(attr: TokenStream, item: TokenStream) -> TokenStream {
                flux_attrs_impl::flux_tool_item_attr(stringify!($name), attr.into(), item.into()).into()
            }
            )*
        };
    }

    flux_tool_attrs!(
        alias,
        spec,
        specs,
        sig,
        qualifiers,
        reveal,
        constant,
        invariant,
        opaque,
        reflect,
        opts,
        trusted,
        trusted_impl,
        trusted_derive,
        proven_externally,
        generics,
        assoc,
        ignore,
        should_fail,
        reft,
        no_panic,
        no_panic_if,
        assume_parametric,
        no_suggestions,
    );
}

#[cfg(not(flux_sysroot))]
mod attr_dummy {
    use super::*;

    pub fn contract(_name: &str, _attr: TokenStream, item: TokenStream) -> TokenStream {
        erase_when_not_checking(item)
    }

    pub fn refined_by(attr: TokenStream, item: TokenStream) -> TokenStream {
        flux_attrs_impl::refined_by(attr.into(), item.into()).into()
    }

    pub fn defs(_tokens: TokenStream) -> TokenStream {
        TokenStream::new()
    }

    pub fn extern_spec(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
        flux_attrs_impl::extern_spec_doc(attrs.into(), tokens.into()).into()
    }

    macro_rules! no_op {
        ($($name:ident),+ $(,)?) => {
            $(
            pub fn $name(_attr: TokenStream, item: TokenStream) -> TokenStream {
                item
            }
            )+
        };
    }

    no_op!(
        alias,
        spec,
        specs,
        sig,
        qualifiers,
        reveal,
        invariant,
        constant,
        opaque,
        reflect,
        opts,
        trusted,
        trusted_impl,
        trusted_derive,
        proven_externally,
        generics,
        assoc,
        ignore,
        should_fail,
        no_panic,
        no_panic_if,
        no_suggestions,
        reft,
        assume_parametric,
    );
}
