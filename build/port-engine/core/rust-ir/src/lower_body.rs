//! Lowering statements and expressions, where precedence is applied.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use port_engine_api::PortError;

use crate::expr::{MatchArm, RustExpr, RustStmt};
use crate::lower_parts::{parse_expr, parse_ident, parse_type};
use crate::ops::BinaryOp;

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
            let names = names
                .iter()
                .map(String::as_str)
                .map(parse_ident)
                .collect::<Result<Vec<_>, _>>()?;
            let value = lower_expr(value)?;
            Ok(quote! { let ( #(#names),* ) = #value; })
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
        RustStmt::Assign { target, value } => {
            let (target, value) = (lower_expr(target)?, lower_expr(value)?);
            Ok(quote! { #target = #value; })
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

/// Lower an expression, parenthesising an operand only where the tree says it needs it.
fn lower_expr(expr: &RustExpr) -> Result<TokenStream, PortError> {
    match expr {
        RustExpr::Slice { base, low, high } => {
            let base = lower_expr(base)?;
            let range = match (low, high) {
                (Some(low), Some(high)) => {
                    let (low, high) = (lower_expr(low)?, lower_expr(high)?);
                    quote! { #low..#high }
                }
                (Some(low), None) => {
                    let low = lower_expr(low)?;
                    quote! { #low.. }
                }
                (None, Some(high)) => {
                    let high = lower_expr(high)?;
                    quote! { ..#high }
                }
                (None, None) => quote! { .. },
            };
            Ok(quote! { &#base[#range] })
        }
        RustExpr::Cast { expr, ty } => {
            // The operand is BRACKETED. `as` binds tighter than every binary operator, so
            // `a + b as u8` casts `b` alone — a silently different program from the one the source
            // wrote, and one that compiles.
            let inner = lower_expr(expr)?;
            let ty = parse_type(ty)?;
            Ok(quote! { (#inner) as #ty })
        }
        RustExpr::Try(inner) => {
            let inner = lower_expr(inner)?;
            Ok(quote! { #inner? })
        }
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
        RustExpr::Field { base, name } => {
            let base = lower_postfix_base(base)?;
            let name = parse_ident(name)?;
            Ok(quote! { #base.#name })
        }
        RustExpr::Call { callee, args } => {
            let callee = lower_postfix_base(callee)?;
            let args = lower_each(args)?;
            Ok(quote! { #callee(#(#args),*) })
        }
        RustExpr::MethodCall {
            receiver,
            method,
            args,
        } => {
            let receiver = lower_postfix_base(receiver)?;
            let method = parse_ident(method)?;
            let args = lower_each(args)?;
            Ok(quote! { #receiver.#method(#(#args),*) })
        }
        RustExpr::Index { base, index } => {
            let base = lower_postfix_base(base)?;
            let index = lower_expr(index)?;
            Ok(quote! { #base[#index] })
        }
        RustExpr::StructLiteral { path, fields } => {
            let path = parse_expr(path, "struct path")?;
            let rendered = fields
                .iter()
                .map(|(name, value)| {
                    let name = parse_ident(name)?;
                    let value = lower_expr(value)?;
                    Ok(quote! { #name: #value })
                })
                .collect::<Result<Vec<_>, PortError>>()?;
            Ok(quote! { #path { #(#rendered),* } })
        }
        RustExpr::Range { start, end } => {
            let (start, end) = (lower_expr(start)?, lower_expr(end)?);
            Ok(quote! { #start..#end })
        }
        RustExpr::Reference { mutable, inner } => {
            let inner = lower_postfix_base(inner)?;
            if *mutable {
                Ok(quote! { &mut #inner })
            } else {
                Ok(quote! { &#inner })
            }
        }
        RustExpr::SelfValue => Ok(quote! { self }),
        RustExpr::Match { scrutinee, arms } => {
            let scrutinee = lower_expr(scrutinee)?;
            let arms = arms
                .iter()
                .map(lower_arm)
                .collect::<Result<Vec<_>, PortError>>()?;
            Ok(quote! { match #scrutinee { #(#arms)* } })
        }
        RustExpr::Todo => Ok(quote! { todo!() }),
    }
}

fn lower_each(exprs: &[RustExpr]) -> Result<Vec<TokenStream>, PortError> {
    exprs.iter().map(lower_expr).collect()
}

/// An arm's patterns are ORs of literal values; an arm with none is the wildcard.
fn lower_arm(arm: &MatchArm) -> Result<TokenStream, PortError> {
    let body = lower_block(&arm.body)?;
    if arm.patterns.is_empty() {
        return Ok(quote! { _ => { #body } });
    }
    let patterns = lower_each(&arm.patterns)?;
    Ok(quote! { #(#patterns)|* => { #body } })
}

/// The base of a postfix form needs bracketing when it is not itself atomic.
///
/// `(a + b).x` and `a + b.x` are different expressions, and the tree already knows which one it
/// holds — so the brackets come from the structure rather than from emitting them everywhere.
fn lower_postfix_base(expr: &RustExpr) -> Result<TokenStream, PortError> {
    let tokens = lower_expr(expr)?;
    match expr {
        RustExpr::Binary { .. }
        | RustExpr::Unary { .. }
        | RustExpr::Range { .. }
        | RustExpr::Reference { .. }
        | RustExpr::If { .. } => Ok(quote! { (#tokens) }),
        _ => Ok(tokens),
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
