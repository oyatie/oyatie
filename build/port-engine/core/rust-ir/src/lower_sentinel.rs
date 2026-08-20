//! The source's sentinel failures, as the target spells them.
//!
//! Split from `lower.rs` because both forms of one decision live here — several types, or one enum
//! with a variant each. Which of them the pack chooses is the pack's business; that both are spelled
//! in one place is this file's, so the `Display` impl and the `Error` impl cannot diverge between
//! them.

use proc_macro2::TokenStream;
use quote::quote;

use port_engine_api::PortError;

use crate::item::RustItem;
use crate::lower_parts::{lower_docs, lower_vis, parse_expr, parse_ident};

/// One sentinel item — grouped or not — as target tokens.
///
/// # Errors
/// [`PortError::Render`] when a name or a message is not valid target syntax.
pub(crate) fn lower(item: &RustItem) -> Result<TokenStream, PortError> {
    match item {
        RustItem::SentinelEnum {
            docs,
            vis,
            name,
            exhaustive,
            variants,
        } => {
            let (docs, vis) = (lower_docs(docs), lower_vis(*vis));
            let name = parse_ident(name)?;
            let mut arms = Vec::with_capacity(variants.len());
            let mut cases = Vec::with_capacity(variants.len());
            for variant in variants {
                let ident = parse_ident(&variant.name)?;
                let message = parse_expr(&variant.message, "sentinel message")?;
                let variant_docs = lower_docs(&variant.docs);
                cases.push(quote! { #variant_docs #ident });
                // Each arm WRITES rather than yielding a string, so a plain message and a formatted
                // one can sit in the same match. Binding one value first only works while every
                // message is a literal, and a sentinel built by a formatting constructor over
                // constants is not — its message is still fixed, just not spelled as one literal.
                arms.push(match variant.arguments.is_empty() {
                    true => quote! { Self::#ident => f.write_str(#message) },
                    false => {
                        let args = variant
                            .arguments
                            .iter()
                            .map(crate::lower_expr::lower_expr)
                            .collect::<Result<Vec<_>, _>>()?;
                        quote! { Self::#ident => write!(f, #message #(, #args)*) }
                    }
                });
            }
            // NON_EXHAUSTIVE is the pack's answer, not this face's: whether a caller may match the
            // whole set is a question about the ported library's compatibility promise.
            let openness = match exhaustive {
                true => quote! {},
                false => quote! { #[non_exhaustive] },
            };
            // COPY and EQ, because a sentinel carries no data: comparing two is comparing which
            // sentinel they are, which is the whole point of the type existing.
            Ok(quote! {
                #docs
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                #openness
                #vis enum #name { #(#cases),* }

                impl fmt::Display for #name {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        // BOUND FIRST, rather than nested in the call. The two say the same thing,
                        // and this one survives formatting: the hermetic formatter wraps a `match`
                        // used as an argument across ten lines with a trailing comma after the
                        // block, where the formatter most authors run collapses it — a difference
                        // a reviewer read, correctly, as output nobody had formatted.
                        match self { #(#arms),* }
                    }
                }

                impl StdError for #name {}
            })
        }

        RustItem::MessageImpl {
            docs,
            self_ty,
            body,
        } => {
            let docs = lower_docs(docs);
            let self_ty = crate::lower_parts::parse_type(self_ty)?;
            let (head, tail) = body.split_at(body.len().saturating_sub(1));
            let leading = crate::lower_body::lower_block(head)?;
            // The TAIL is the message, and how it is written depends on what it is. A formatting
            // call is handed to the formatter directly — writing its result would allocate a string
            // only to copy it, which the target's own lint objects to. Anything else is written as
            // the string it is.
            let write = match tail.first() {
                Some(crate::stmt::RustStmt::Tail(crate::expr::RustExpr::MacroCall {
                    name,
                    template,
                    args,
                })) if name == "format" => {
                    let template: proc_macro2::TokenStream = format!("{template:?}").parse().map_err(
                        |err| PortError::Render {
                            detail: format!("a message template is not a target literal: {err}"),
                        },
                    )?;
                    let args = args
                        .iter()
                        .map(crate::lower_expr::lower_expr)
                        .collect::<Result<Vec<_>, _>>()?;
                    quote! { write!(f, #template #(, #args)*) }
                }
                // ALREADY A REFERENCE. `write_str` takes one, so a tail that is already a borrow
                // needs no second `&` — and adding one is `clippy::needless_borrow`, which the
                // deny-warnings policy makes a build failure. The borrow is added only where the
                // tail is a value.
                Some(crate::stmt::RustStmt::Tail(
                    expr @ crate::expr::RustExpr::Reference { mutable: false, .. },
                )) => {
                    let expr = crate::lower_expr::lower_expr(expr)?;
                    quote! { f.write_str(#expr) }
                }
                Some(crate::stmt::RustStmt::Tail(expr)) => {
                    let expr = crate::lower_expr::lower_expr(expr)?;
                    quote! { f.write_str(&#expr) }
                }
                _ => {
                    return Err(PortError::Render {
                        detail: "a message method's body does not end in the message".to_owned(),
                    });
                }
            };
            Ok(quote! {
                #docs
                impl fmt::Display for #self_ty {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        #leading
                        #write
                    }
                }

                impl StdError for #self_ty {}
            })
        }

        RustItem::SentinelError {
            docs,
            vis,
            name,
            message,
        } => {
            let (docs, vis) = (lower_docs(docs), lower_vis(*vis));
            let name = parse_ident(name)?;
            let message = parse_expr(message, "sentinel message")?;
            // COPY and EQ, because a sentinel is a value with no data: comparing two is comparing
            // which sentinel they are, which is the whole point of it having a type at all.
            Ok(quote! {
                #docs
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                #vis struct #name;

                impl fmt::Display for #name {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str(#message)
                    }
                }

                impl StdError for #name {}
            })
        }

        _ => unreachable!("the caller matched a sentinel item"),
    }
}
