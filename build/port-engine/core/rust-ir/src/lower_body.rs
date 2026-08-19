//! Lowering statements and expressions, where precedence is applied.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use port_engine_api::PortError;

use crate::expr::{MatchArm, RustExpr};
use crate::stmt::{RustStmt};
use crate::lower_parts::{parse_expr, parse_ident, parse_type};
use crate::lower_expr::lower_expr;
use crate::lower_precedence::is_block_like;

pub(crate) fn lower_block(statements: &[RustStmt]) -> Result<TokenStream, PortError> {
    let mut tokens = TokenStream::new();
    for statement in statements {
        tokens.extend(lower_stmt(statement)?);
    }
    Ok(tokens)
}

fn lower_stmt(statement: &RustStmt) -> Result<TokenStream, PortError> {
    match statement {
        RustStmt::Let {
            name,
            mutable,
            ty,
            value,
        } => {
            let name = parse_ident(name)?;
            let mutability = if *mutable {
                quote! { mut }
            } else {
                TokenStream::new()
            };
            let annotation = match ty {
                Some(ty) => {
                    let ty = parse_type(ty)?;
                    quote! { : #ty }
                }
                None => TokenStream::new(),
            };
            match value {
                Some(value) => {
                    let value = lower_expr(value)?;
                    Ok(quote! { let #mutability #name #annotation = #value; })
                }
                None => Ok(quote! { let #mutability #name #annotation; }),
            }
        }
        RustStmt::LetTuple { names, value } => {
            // `mut` binds to the NAME inside a tuple pattern, not to the pattern, so each element
            // carries its own — which is also what lets one name be written again while its
            // neighbour stays immutable.
            let bound = names
                .iter()
                .map(|bind| {
                    let name = parse_ident(&bind.name)?;
                    let mutability = match bind.mutable {
                        true => quote! { mut },
                        false => quote! {},
                    };
                    Ok(quote! { #mutability #name })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let value = lower_expr(value)?;
            Ok(quote! { let ( #(#bound),* ) = #value; })
        }
        RustStmt::Semi(expr) => {
            let lowered = lower_expr(expr)?;
            // A BLOCK-LIKE expression is already a statement, and the semicolon after it is not
            // merely noise: it makes the block an expression statement whose value is discarded,
            // which is how an emitted `if` came to sit under a binding rustc then reported as
            // never read.
            match is_block_like(expr) {
                true => Ok(quote! { #lowered }),
                false => Ok(quote! { #lowered; }),
            }
        }
        RustStmt::Tail(expr) => lower_expr(expr),
        RustStmt::Return(None) => Ok(quote! { return; }),
        RustStmt::Return(Some(expr)) => {
            let expr = lower_expr(expr)?;
            Ok(quote! { return #expr; })
        }
        RustStmt::Assign { target, op, value } => {
            let (target, value) = (lower_expr(target)?, lower_expr(value)?);
            let operator: TokenStream = match op {
                None => quote! { = },
                Some(op) => {
                    let spelling = format!("{}=", op.spelling());
                    spelling.parse().map_err(|_| PortError::Render {
                        detail: format!("`{spelling}` is not a valid assignment operator"),
                    })?
                }
            };
            Ok(quote! { #target #operator #value; })
        }
        RustStmt::AssignTuple { places, values } => {
            let places = places.iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?;
            let values = values.iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?;
            // A SINGLE value is one expression that already yields the whole tuple — the source's
            // `x, err = f()`. Several are the source's `a, b = b, a`, and they become a tuple here
            // so both spell one destructuring assignment rather than two shapes.
            let right = match values.as_slice() {
                [only] => quote! { #only },
                several => quote! { ( #(#several),* ) },
            };
            Ok(quote! { ( #(#places),* ) = #right; })
        }
        RustStmt::While { cond, body } => {
            let (cond, body) = (lower_expr(cond)?, lower_block(body)?);
            Ok(quote! { while #cond { #body } })
        }
        RustStmt::Loop(body) => {
            let body = lower_block(body)?;
            Ok(quote! { loop { #body } })
        }
        RustStmt::ForIn {
            binding,
            iter,
            body,
        } => {
            let binding = parse_ident(binding)?;
            let (iter, body) = (lower_expr(iter)?, lower_block(body)?);
            Ok(quote! { for #binding in #iter { #body } })
        }
        RustStmt::Break => Ok(quote! { break; }),
    }
}
