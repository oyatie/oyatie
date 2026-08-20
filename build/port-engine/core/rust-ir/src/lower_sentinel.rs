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
            is_failure,
        } => {
            let docs = lower_docs(docs);
            let self_ty = crate::lower_parts::parse_type(self_ty)?;
            // The body already WRITES at every exit — see `RustExpr::FormatterWrite`, which the
            // transform wraps each `return` and the tail in. This used to decide how to write here,
            // looking only at the last statement, which is why a body with an early return could
            // not become a display impl at all. One node, one decision, every exit.
            let written = crate::lower_body::lower_block(body)?;
            // THE ERROR TRAIT ONLY FOR A FAILURE. See `RustItem::MessageImpl::is_failure`: the
            // display construction is shared between the source's error interface and its stringer,
            // and only one of the two is an error.
            let error_impl = match is_failure {
                true => quote! { impl StdError for #self_ty {} },
                false => quote! {},
            };
            Ok(quote! {
                #docs
                impl fmt::Display for #self_ty {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        #written
                    }
                }

                #error_impl
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
