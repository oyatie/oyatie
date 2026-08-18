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

use crate::item::{RustField, RustFn, RustItem, RustParam, StructShape};
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

        RustItem::TypeAlias {
            docs,
            vis,
            name,
            ty,
        } => {
            let (docs, vis) = (lower_docs(docs), lower_vis(*vis));
            let (name, ty) = (parse_ident(name)?, parse_type(ty)?);
            Ok(quote! { #docs #vis type #name = #ty; })
        }

        RustItem::Struct {
            docs,
            vis,
            name,
            shape,
            methods,
        } => {
            let (doc_tokens, vis_tokens) = (lower_docs(docs), lower_vis(*vis));
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
            let mut tokens = quote! { #doc_tokens #vis_tokens struct #ident #body };
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
            methods,
        } => {
            let (doc_tokens, vis_tokens) = (lower_docs(docs), lower_vis(*vis));
            let ident = parse_ident(name)?;
            let rendered = methods
                .iter()
                .map(lower_fn)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! { #doc_tokens #vis_tokens trait #ident { #(#rendered)* } })
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
    let vis = lower_vis(function.vis);
    let name = parse_ident(&function.name)?;

    let mut inputs: Vec<TokenStream> = Vec::with_capacity(function.params.len() + 1);
    if let Some(receiver) = function.receiver {
        inputs.push(parse_expr(receiver.spelling(), "receiver")?.into_token_stream());
    }
    for RustParam { name, ty } in &function.params {
        let (name, ty) = (parse_ident(name)?, parse_type(ty)?);
        inputs.push(quote! { #name: #ty });
    }

    let ret = match &function.ret {
        Some(ty) if !ty.is_unit() => {
            let ty = parse_type(ty)?;
            quote! { -> #ty }
        }
        _ => TokenStream::new(),
    };

    match &function.body {
        None => Ok(quote! { #docs #vis fn #name(#(#inputs),*) #ret ; }),
        Some(body) => {
            let statements = lower_block(body)?;
            Ok(quote! { #docs #vis fn #name(#(#inputs),*) #ret { #statements } })
        }
    }
}
