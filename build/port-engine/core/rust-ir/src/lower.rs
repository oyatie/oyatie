//! IR → tokens → `syn::File`.
//!
//! `quote!` builds a token stream and `syn` parses it. That is still a parse, and it is a
//! materially different one from the `format!`-a-string-and-reparse it replaces: tokens cannot be
//! malformed by whitespace, an identifier cannot accidentally merge with its neighbour, and
//! nothing in the item structure is decided by how the text happened to read. What syn does here
//! is check the assembly, not recover it.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use port_engine_api::PortError;

use crate::item::RustItem;
use crate::item_parts::{RustField, RustFn, RustParam, StructShape};
use crate::lower_body::lower_block;
use crate::lower_parts::{lower_docs, lower_vis, parse_expr, parse_ident, parse_type};

/// Lower a whole region's items into a parsed `syn::File`.
///
/// # Errors
/// [`PortError::Render`] when the assembled tokens are not a valid Rust file — which is how a bad
/// type spelling or a bad literal from the source surfaces, rather than being emitted and failing
/// far from its cause.
pub fn lower_file(items: &[RustItem]) -> Result<syn::File, PortError> {
    let mut tokens = TokenStream::new();
    for item in items {
        tokens.extend(lower_item(item)?);
    }
    syn::parse2(tokens.clone()).map_err(|err| PortError::Render {
        detail: format!("assembled tokens are not a valid Rust file: {err} — in `{tokens}`"),
    })
}

fn lower_item(item: &RustItem) -> Result<TokenStream, PortError> {
    match item {
        RustItem::Const {
            docs,
            vis,
            name,
            ty,
            value,
        } => {
            let (docs, vis) = (lower_docs(docs), lower_vis(*vis));
            let (name, ty) = (parse_ident(name)?, parse_type(ty)?);
            let value = parse_expr(value, "constant value")?;
            Ok(quote! { #docs #vis const #name: #ty = #value; })
        }

        RustItem::Use { path } => {
            let item = crate::lower_parts::parse_use(path)?;
            Ok(quote! { #item })
        }

        RustItem::Nothing => Ok(quote! {}),
        item @ (RustItem::SentinelEnum { .. }
        | RustItem::SentinelError { .. }
        | RustItem::MessageImpl { .. }) => crate::lower_sentinel::lower(item),

        RustItem::PackageValue {
            docs,
            vis,
            name,
            ty,
            value,
        } => {
            let (docs, vis) = (lower_docs(docs), lower_vis(*vis));
            let (name, ty) = (parse_ident(name)?, parse_type(ty)?);
            let value = crate::lower_expr::lower_expr(value)?;
            Ok(quote! { #docs #vis const #name: #ty = #value; })
        }

        RustItem::BlanketImpl { name, bounds } => {
            let name = parse_ident(name)?;
            let bounds = bounds
                .iter()
                .map(parse_type)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! { impl<T: #(#bounds)+*> #name for T {} })
        }

        RustItem::TypeAlias {
            docs,
            vis,
            name,
            generics,
            ty,
        } => {
            let (docs, vis) = (lower_docs(docs), lower_vis(*vis));
            let (name, ty) = (parse_ident(name)?, parse_type(ty)?);
            let parameters = generics
                .iter()
                .map(|generic| crate::lower_parts::parse_generic_param(generic))
                .collect::<Result<Vec<_>, _>>()?;
            match parameters.is_empty() {
                true => Ok(quote! { #docs #vis type #name = #ty; }),
                false => Ok(quote! { #docs #vis type #name<#(#parameters),*> = #ty; }),
            }
        }

        RustItem::Struct {
            docs,
            vis,
            name,
            shape,
            derives,
            methods,
        } => {
            let (doc_tokens, vis_tokens) = (lower_docs(docs), lower_vis(*vis));
            let derive_tokens = match derives.is_empty() {
                true => TokenStream::new(),
                false => {
                    let names = derives
                        .iter()
                        .map(|name| parse_ident(name))
                        .collect::<Result<Vec<_>, _>>()?;
                    quote! { #[derive(#(#names),*)] }
                }
            };
            let ident = parse_ident(name)?;
            let body = match shape {
                StructShape::Unit => quote! { ; },
                StructShape::Tuple(fields) => {
                    let fields = lower_tuple_fields(fields)?;
                    quote! { ( #(#fields),* ); }
                }
                StructShape::Named(fields) => {
                    let fields = lower_named_fields(fields)?;
                    quote! { { #(#fields),* } }
                }
            };
            let mut tokens = quote! { #doc_tokens #derive_tokens #vis_tokens struct #ident #body };
            if !methods.is_empty() {
                let rendered = methods
                    .iter()
                    .map(lower_fn)
                    .collect::<Result<Vec<_>, _>>()?;
                tokens.extend(quote! { impl #ident { #(#rendered)* } });
            }
            Ok(tokens)
        }

        RustItem::Trait {
            docs,
            vis,
            name,
            supertraits,
            methods,
        } => {
            let (doc_tokens, vis_tokens) = (lower_docs(docs), lower_vis(*vis));
            let ident = parse_ident(name)?;
            let rendered = methods
                .iter()
                .map(lower_fn)
                .collect::<Result<Vec<_>, _>>()?;
            let bounds = supertraits
                .iter()
                .map(parse_type)
                .collect::<Result<Vec<_>, _>>()?;
            let requires = if bounds.is_empty() {
                TokenStream::new()
            } else {
                quote! { : #(#bounds)+* }
            };
            Ok(quote! { #doc_tokens #vis_tokens trait #ident #requires { #(#rendered)* } })
        }

        RustItem::InherentImpl {
            docs,
            self_ty,
            methods,
        } => {
            let doc_tokens = lower_docs(docs);
            let self_tokens = parse_type(self_ty)?;
            let rendered = methods
                .iter()
                .map(lower_fn)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! { #doc_tokens impl #self_tokens { #(#rendered)* } })
        }
        RustItem::TraitImpl {
            docs,
            trait_path,
            self_ty,
            methods,
        } => {
            let doc_tokens = lower_docs(docs);
            let (trait_tokens, self_tokens) = (parse_type(trait_path)?, parse_type(self_ty)?);
            let rendered = methods
                .iter()
                .map(lower_fn)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! {
                #doc_tokens impl #trait_tokens for #self_tokens { #(#rendered)* }
            })
        }

        RustItem::Function(function) => lower_fn(function),
    }
}

fn lower_tuple_fields(fields: &[RustField]) -> Result<Vec<TokenStream>, PortError> {
    fields
        .iter()
        .map(|field| {
            let vis = lower_vis(field.vis);
            let ty = parse_type(&field.ty)?;
            Ok(quote! { #vis #ty })
        })
        .collect()
}

fn lower_named_fields(fields: &[RustField]) -> Result<Vec<TokenStream>, PortError> {
    fields
        .iter()
        .map(|field| {
            let docs = lower_docs(&field.docs);
            let vis = lower_vis(field.vis);
            let name = parse_ident(&field.name)?;
            let ty = parse_type(&field.ty)?;
            Ok(quote! { #docs #vis #name: #ty })
        })
        .collect()
}

fn lower_fn(function: &RustFn) -> Result<TokenStream, PortError> {
    let docs = lower_docs(&function.docs);
    // Parsed rather than pasted, so an attribute the pack cannot spell refuses here instead of in
    // the emitted file.
    let attrs = function
        .attrs
        .iter()
        .map(|attr| {
            // Parsed as an item that CARRIES the attribute, because an attribute alone is not a
            // parseable unit — and what comes back is the attribute, so an unspellable one refuses
            // here rather than in the emitted file.
            syn::parse_str::<syn::ItemStruct>(&format!("{attr} struct A;"))
                .map_err(|err| PortError::Render {
                    detail: format!("`{attr}` is not a valid target attribute: {err}"),
                })
                .and_then(|item| {
                    item.attrs
                        .into_iter()
                        .next()
                        .ok_or_else(|| PortError::Render {
                            detail: format!("`{attr}` parsed as no attribute at all"),
                        })
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let vis = lower_vis(function.vis);
    let name = parse_ident(&function.name)?;

    let mut inputs: Vec<TokenStream> = Vec::with_capacity(function.params.len() + 1);
    if let Some(receiver) = function.receiver {
        inputs.push(parse_expr(receiver.spelling(), "receiver")?.into_token_stream());
    }
    for RustParam {
        name,
        rebound,
        unread,
        ty,
    } in &function.params
    {
        let spelling = match unread {
            true => format!("_{name}"),
            false => name.clone(),
        };
        // A PARAMETER IS A PATTERN, and the blank is one. `parse_ident` refuses `_` because the
        // target reserves it — correctly, since it is not an identifier — so the blank is spelled
        // directly rather than parsed as a name. Without this a source parameter the author
        // deliberately left unnamed refused the whole declaration it belongs to.
        let bound = match spelling.as_str() {
            "_" => quote! { _ },
            other => parse_ident(other)?.into_token_stream(),
        };
        let ty = parse_type(ty)?;
        // `mut` on a parameter binds the callee's own copy and is invisible in the function's
        // type, so it never changes what a caller may pass.
        let mutability = match rebound {
            true => quote! { mut },
            false => quote! {},
        };
        inputs.push(quote! { #mutability #bound: #ty });
    }

    let ret = match &function.ret {
        Some(ty) if !ty.is_unit() => {
            let ty = parse_type(ty)?;
            quote! { -> #ty }
        }
        _ => TokenStream::new(),
    };

    //  sits between the visibility and the , which is the one place the target accepts
    // it -- so it is built here rather than folded into the attribute list.
    let asyncness = match function.is_async {
        true => quote! { async },
        false => TokenStream::new(),
    };

    match &function.body {
        None => Ok(quote! { #docs #(#attrs)* #vis #asyncness fn #name(#(#inputs),*) #ret ; }),
        Some(body) => {
            let statements = lower_block(body)?;
            Ok(
                quote! { #docs #(#attrs)* #vis #asyncness fn #name(#(#inputs),*) #ret { #statements } },
            )
        }
    }
}
