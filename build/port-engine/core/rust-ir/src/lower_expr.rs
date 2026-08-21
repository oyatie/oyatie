//! Lowering an EXPRESSION to target tokens.
//!
//! Split from statement lowering because the two answer different questions and only one of them
//! is about precedence. An expression has to be rendered so that the tokens reassociate the way
//! the tree says — a cast whose operand is a binary operation needs brackets the tree does not
//! carry — and that reasoning is what fills this file.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use port_engine_api::PortError;

use crate::expr::{MatchArm, RustExpr};
use crate::stmt::{RustStmt};
use crate::lower_body::lower_block;
use crate::lower_parts::{parse_expr, parse_ident, parse_type};
use crate::ops::BinaryOp;
use crate::lower_precedence::{
    binds_tighter_than_cast, lower_operand, lower_postfix_base, typed_literal,
};
use crate::ty::RustType;

/// Lower an expression, parenthesising an operand only where the tree says it needs it.
pub(crate) fn lower_expr(expr: &RustExpr) -> Result<TokenStream, PortError> {
    match expr {
        RustExpr::Slice { .. } => {
            let inner = lower_slice_place(expr)?;
            Ok(quote! { &#inner })
        }
        RustExpr::Cast { expr, ty } => {
            // A COMPOUND operand is bracketed. `as` binds tighter than every binary operator, so
            // `a + b as u8` casts `b` alone — a silently different program from the one the source
            // wrote, and one that compiles. An operand that already binds tighter than `as` cannot
            // reassociate, and bracketing it buys nothing while reading as though the writer was
            // unsure which way it would go.
            // A conversion of an integer LITERAL is a typed literal, not a cast. `1 as i64` and
            // `1i64` are the same value, and only the second is what someone writing the target
            // would put — the first is what a translator emits when it treats every conversion the
            // same way.
            if let Some(typed) = typed_literal(expr, ty) {
                return typed.parse().map_err(|_| PortError::Render {
                    detail: format!("`{typed}` is not a valid literal"),
                });
            }
            let inner = lower_expr(expr)?;
            let ty = parse_type(ty)?;
            match binds_tighter_than_cast(expr) {
                true => Ok(quote! { #inner as #ty }),
                false => Ok(quote! { (#inner) as #ty }),
            }
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
            // A CAST ON THE LEFT OF A COMPARISON IS BRACKETED, and this is a GRAMMAR rule rather
            // than a precedence one — which is why the precedence table, where a cast binds
            // tightest and therefore never needs brackets, is right and still not enough.
            //
            // `a.len() as i64 < b.len() as i64` does not parse. The target reads `i64 <` as the
            // start of generic arguments and says so:
            //
            //     `<` is interpreted as a start of generic arguments for `i64`, not a comparison
            //
            // It cost `gjson::string_less_insensitive`, which refused with a renderer error naming
            // a missing comma — a message that points nowhere near the cause.
            let left = match (matches!(**lhs, RustExpr::Cast { .. }), op.is_comparison()) {
                (true, true) => {
                    let inner = lower_expr(lhs)?;
                    quote! { (#inner) }
                }
                _ => lower_operand(lhs, *op, false)?,
            };
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
        RustExpr::TupleIndex { base, index } => {
            let base = lower_postfix_base(base)?;
            let index = syn::Index::from(*index);
            Ok(quote! { #base.#index })
        }
        RustExpr::Field { base, name } => {
            let base = lower_postfix_base(base)?;
            let name = parse_ident(name)?;
            Ok(quote! { #base.#name })
        }
        RustExpr::MacroCall {
            name,
            template,
            args,
        } => {
            let name = crate::lower_parts::parse_ident(name)?;
            let args = lower_each(args)?;
            // The template is a STRING LITERAL in the emitted macro, so it is lowered as a literal
            // rather than parsed — a template is data, and parsing it here would give its braces a
            // second meaning.
            let template = proc_macro2::Literal::string(template);
            match args.is_empty() {
                true => Ok(quote! { #name!(#template) }),
                false => Ok(quote! { #name!(#template, #(#args),*) }),
            }
        }
        RustExpr::Closure {
            moves,
            params,
            ret,
            body,
        } => {
            let mut bound = Vec::with_capacity(params.len());
            for param in params {
                let name = crate::lower_parts::parse_ident(&param.name)?;
                bound.push(match &param.ty {
                    None => quote! { #name },
                    Some(ty) => {
                        let ty = crate::lower_parts::parse_type(ty)?;
                        quote! { #name: #ty }
                    }
                });
            }
            let block = crate::lower_body::lower_block(body)?;
            let head = match moves {
                true => quote! { move },
                false => quote! {},
            };
            match ret {
                None => Ok(quote! { #head |#(#bound),*| { #block } }),
                Some(ty) => {
                    let ty = crate::lower_parts::parse_type(ty)?;
                    Ok(quote! { #head |#(#bound),*| -> #ty { #block } })
                }
            }
        }
        // HOW a value reaches the formatter, decided by what it IS. See `RustExpr::FormatterWrite`.
        // The formatter is named `f` because the impl this appears in binds it so — the two are
        // written once, in `lower_sentinel`, and this is the other half.
        RustExpr::FormatterWrite(value) => match value.as_ref() {
            // A FORMATTING CALL goes to the formatter directly. Writing its result would allocate a
            // string only to copy it, which the target's own lint objects to.
            RustExpr::MacroCall {
                name,
                template,
                args,
            } if name == "format" => {
                let template: proc_macro2::TokenStream = format!("{template:?}")
                    .parse()
                    .map_err(|err| PortError::Render {
                        detail: format!("a message template is not a target literal: {err}"),
                    })?;
                let args = lower_each(args)?;
                Ok(quote! { write!(f, #template #(, #args)*) })
            }
            // EVERY ARM A LITERAL: borrowed arms make the match a `&'static str`, which the
            // formatter takes directly, instead of one allocation per call for static text.
            expr @ RustExpr::Match { .. } if static_str_match(expr).is_some() => {
                let borrowed = static_str_match(expr).expect("checked by the guard");
                let matched = lower_expr(&borrowed)?;
                Ok(quote! { f.write_str(#matched) })
            }
            // A VALUE THAT CAN WRITE ITSELF does, rather than rendering to a string to be copied.
            RustExpr::MethodCall {
                receiver,
                method,
                args,
            } if method == "to_string" && args.is_empty() => {
                let inner = lower_expr(receiver)?;
                Ok(quote! { fmt::Display::fmt(&#inner, f) })
            }
            // ALREADY A REFERENCE, so no second borrow — which would be `needless_borrow`.
            expr @ RustExpr::Reference { mutable: false, .. } => {
                let expr = lower_expr(expr)?;
                Ok(quote! { f.write_str(#expr) })
            }
            other => {
                let expr = lower_expr(other)?;
                Ok(quote! { f.write_str(&#expr) })
            }
        },
        RustExpr::Deref(inner) => {
            let inner = crate::lower_precedence::lower_postfix_base(inner)?;
            Ok(quote! { *#inner })
        }
        RustExpr::ArrayLiteral(elements) => {
            let elements = lower_each(elements)?;
            Ok(quote! { [#(#elements),*] })
        }
        RustExpr::VecRepeat { value, count } => {
            let (value, count) = (lower_expr(value)?, lower_expr(count)?);
            Ok(quote! { vec![#value; #count] })
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
                    // FIELD-INIT SHORTHAND. A field whose value is a binding of the same name is
                    // written without the colon, which is defined as the long form and so changes
                    // the spelling and not the program. The source has no shorthand and always
                    // writes the long form, so a constructor passing a parameter straight into the
                    // field it names would otherwise emit what `clippy::style` calls a redundant
                    // field name on every one.
                    let ident = parse_ident(name)?;
                    if matches!(value, RustExpr::Path(path) if path == name) {
                        return Ok(quote! { #ident });
                    }
                    let value = lower_expr(value)?;
                    Ok(quote! { #ident: #value })
                })
                .collect::<Result<Vec<_>, PortError>>()?;
            Ok(quote! { #path { #(#rendered),* } })
        }
        RustExpr::Range {
            start,
            end,
            inclusive,
        } => {
            let end = lower_expr(end)?;
            let start = start.as_ref().map(|bound| lower_expr(bound)).transpose()?;
            match inclusive {
                true => Ok(quote! { #start..=#end }),
                false => Ok(quote! { #start..#end }),
            }
        }
        RustExpr::Await(inner) => {
            let inner = lower_postfix_base(inner)?;
            Ok(quote! { #inner.await })
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
    }
}

fn lower_each(exprs: &[RustExpr]) -> Result<Vec<TokenStream>, PortError> {
    exprs.iter().map(lower_expr).collect()
}

/// An arm's patterns are ORs of literal values; an arm with none is the wildcard.
fn lower_arm(arm: &MatchArm) -> Result<TokenStream, PortError> {
    let body = lower_block(&arm.body)?;
    let guard = arm
        .guard
        .as_ref()
        .map(lower_expr)
        .transpose()?
        .map(|guard| quote! { if #guard });
    if arm.patterns.is_empty() {
        return Ok(quote! { _ #guard => { #body } });
    }
    let patterns = lower_each(&arm.patterns)?;
    Ok(quote! { #(#patterns)|* #guard => { #body } })
}

/// A slice as a PLACE — `x[a..b]` — without the borrow that reading it as a value needs.
///
/// The borrow and the place are different things and only one of them belongs in postfix position.
/// `&x[..].to_vec()` is a reference to a vector, because the method binds to what is borrowed; and
/// bracketing it into `(&x[..]).to_vec()` compiles but borrows a value the compiler would borrow
/// anyway, which the target's own lint rejects and this engine is held to. A place takes the method
/// directly and autoref does the rest.
pub(crate) fn lower_slice_place(expr: &RustExpr) -> Result<TokenStream, PortError> {
    let RustExpr::Slice { base, low, high } = expr else {
        return lower_expr(expr);
    };
    // THE BASE OF A SLICE IS A PLACE, and lowering it as a value borrows it. That is invisible
    // until a slice's base is ITSELF a slice, at which point `&x[a..b]` gains `[..n]` and the
    // borrow — which binds looser than everything — swallows the whole method chain that follows:
    // `&x[a..b][..n].try_into()` is a reference to the conversion rather than a conversion of the
    // slice, and the type error it produces names neither.
    let base = crate::lower_precedence::lower_postfix_base(base)?;
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
    Ok(quote! { #base[#range] })
}

/// A `match` whose every arm yields an OWNED string literal, with the arms borrowed instead.
///
/// `"Null".to_owned()` in every arm makes the match a `String`, which costs an allocation per call
/// for text that never changes. Borrowing the arms makes it a `&'static str`, which is the same
/// text and no allocation.
///
/// `None` unless EVERY arm qualifies. A match with one computed arm has no single borrowed type,
/// and rewriting the rest would leave arms that do not agree.
pub(crate) fn static_str_match(expr: &RustExpr) -> Option<RustExpr> {
    let RustExpr::Match { scrutinee, arms } = expr else {
        return None;
    };
    let mut borrowed = Vec::with_capacity(arms.len());
    for arm in arms {
        let [crate::stmt::RustStmt::Tail(RustExpr::MethodCall {
            receiver,
            method,
            args,
        })] = arm.body.as_slice()
        else {
            return None;
        };
        if method != "to_owned" || !args.is_empty() {
            return None;
        }
        let RustExpr::Literal(text) = receiver.as_ref() else {
            return None;
        };
        if !text.starts_with('"') {
            return None;
        }
        borrowed.push(crate::expr::MatchArm {
            patterns: arm.patterns.clone(),
            guard: arm.guard.clone(),
            body: vec![crate::stmt::RustStmt::Tail(RustExpr::Literal(text.clone()))],
        });
    }
    Some(RustExpr::Match {
        scrutinee: scrutinee.clone(),
        arms: borrowed,
    })
}
