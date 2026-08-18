//! Lowering statements and expressions, where precedence is applied.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use port_engine_api::PortError;

use crate::expr::{BinaryOp, RustExpr, RustStmt};
use crate::lower_parts::{parse_expr, parse_ident};

pub(crate) fn lower_block(statements: &[RustStmt]) -> Result<TokenStream, PortError> {
    let mut tokens = TokenStream::new();
    for statement in statements {
        tokens.extend(lower_stmt(statement)?);
    }
    Ok(tokens)
}

fn lower_stmt(statement: &RustStmt) -> Result<TokenStream, PortError> {
    match statement {
        RustStmt::Let { name, value } => {
            let name = parse_ident(name)?;
            let value = lower_expr(value)?;
            Ok(quote! { let #name = #value; })
        }
        RustStmt::Semi(expr) => {
            let expr = lower_expr(expr)?;
            Ok(quote! { #expr; })
        }
        RustStmt::Tail(expr) => lower_expr(expr),
        RustStmt::Return(None) => Ok(quote! { return; }),
        RustStmt::Return(Some(expr)) => {
            let expr = lower_expr(expr)?;
            Ok(quote! { return #expr; })
        }
    }
}

/// Lower an expression, parenthesising an operand only where the tree says it needs it.
fn lower_expr(expr: &RustExpr) -> Result<TokenStream, PortError> {
    match expr {
        RustExpr::Literal(spelling) => {
            let literal = parse_expr(spelling, "literal")?;
            Ok(literal.into_token_stream())
        }
        RustExpr::Path(path) => {
            let path = parse_expr(path, "path")?;
            Ok(path.into_token_stream())
        }
        RustExpr::Binary { op, lhs, rhs } => {
            let left = lower_operand(lhs, *op, false)?;
            let right = lower_operand(rhs, *op, true)?;
            let operator: TokenStream = op.spelling().parse().map_err(|_| PortError::Render {
                detail: format!("`{}` is not a valid operator", op.spelling()),
            })?;
            Ok(quote! { #left #operator #right })
        }
        RustExpr::Unary { op, operand } => {
            let inner = lower_expr(operand)?;
            let operator: TokenStream = op.spelling().parse().map_err(|_| PortError::Render {
                detail: format!("`{}` is not a valid operator", op.spelling()),
            })?;
            // A prefix operator binds tighter than every binary one, so any binary operand under
            // it needs bracketing.
            if matches!(**operand, RustExpr::Binary { .. }) {
                Ok(quote! { #operator (#inner) })
            } else {
                Ok(quote! { #operator #inner })
            }
        }
        RustExpr::If {
            cond,
            then,
            otherwise,
        } => {
            let cond = lower_expr(cond)?;
            let then = lower_block(then)?;
            match otherwise {
                None => Ok(quote! { if #cond { #then } }),
                Some(branch) => {
                    let branch = lower_expr(branch)?;
                    Ok(quote! { if #cond { #then } else #branch })
                }
            }
        }
        RustExpr::Block(statements) => {
            let statements = lower_block(statements)?;
            Ok(quote! { { #statements } })
        }
        RustExpr::Tuple(elements) => {
            let rendered = elements
                .iter()
                .map(lower_expr)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! { ( #(#rendered),* ) })
        }
        RustExpr::Todo => Ok(quote! { todo!() }),
    }
}

fn lower_operand(
    operand: &RustExpr,
    parent: BinaryOp,
    is_right: bool,
) -> Result<TokenStream, PortError> {
    let tokens = lower_expr(operand)?;
    if operand.needs_parens_under(parent, is_right) {
        Ok(quote! { (#tokens) })
    } else {
        Ok(tokens)
    }
}
