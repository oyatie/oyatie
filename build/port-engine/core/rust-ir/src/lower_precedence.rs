//! Where a lowered expression needs BRACKETING, and where it does not.
//!
//! Split from `lower_expr.rs` because the two answer different questions. That file says what one
//! IR node becomes; this one says what the target's grammar does to it in the position it lands in
//! — which operands a cast is hostile to, which forms bind tighter than their parent, and which are
//! block-like and cannot sit bare where a value is expected.
//!
//! Getting this wrong does not fail to compile. It reassociates, and the emitted program means
//! something else — which is why it is worth its own face rather than being folded into the node
//! that happens to need it.

use proc_macro2::TokenStream;
use quote::quote;

use port_engine_api::PortError;

use crate::expr::RustExpr;
use crate::lower_expr::lower_expr;
use crate::ops::BinaryOp;
use crate::ty::RustType;

/// The base of a postfix form needs bracketing when it is not itself atomic.
///
/// `(a + b).x` and `a + b.x` are different expressions, and the tree already knows which one it
/// holds — so the brackets come from the structure rather than from emitting them everywhere.
pub(crate) fn lower_postfix_base(expr: &RustExpr) -> Result<TokenStream, PortError> {
    // A SLICE in postfix position is a PLACE, not a borrowed value. See `lower_slice_place`.
    if matches!(expr, RustExpr::Slice { .. }) {
        return crate::lower_expr::lower_slice_place(expr);
    }
    let tokens = lower_expr(expr)?;
    match expr {
        RustExpr::Binary { .. }
        | RustExpr::Unary { .. }
        | RustExpr::Range { .. }
        | RustExpr::Reference { .. }
        // A CAST is postfix-hostile: `x as i64.wrapping_add(y)` is not `(x as i64).wrapping_add(y)`
        // — the target rejects it outright rather than parsing it the other way, which is the good
        // failure mode and is how this was found. A whole real package failed to render on it, and
        // the hermetic corpus never had a cast with a method on it.
        | RustExpr::Cast { .. }
        | RustExpr::If { .. } => Ok(quote! { (#tokens) }),
        _ => Ok(tokens),
    }
}

pub(crate) fn lower_operand(
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

/// Whether this expression binds tighter than a cast, so bracketing it changes nothing.
///
/// A literal, a path, `self`, a field, an index, a call and a method call are all primary
/// expressions: `as` cannot pull them apart, so the brackets a compound operand needs are noise
/// here. Anything else — a binary operation, a unary one, another cast — is bracketed, because
/// reassociating a cast is a silent change of meaning rather than a formatting one.
pub(crate) fn binds_tighter_than_cast(expr: &RustExpr) -> bool {
    matches!(
        expr,
        RustExpr::Literal(_)
            | RustExpr::Path(_)
            | RustExpr::SelfValue
            | RustExpr::Field { .. }
            | RustExpr::Index { .. }
            | RustExpr::Call { .. }
            | RustExpr::MethodCall { .. }
            | RustExpr::MacroCall { .. }
            // Delimited by brackets, so a cast cannot reach inside it.
            | RustExpr::VecRepeat { .. }
            | RustExpr::ArrayLiteral(_)
    )
}

/// Whether this expression is already a statement in the target's grammar.
///
/// The target's block-like forms end in a brace and stand alone; a semicolon after one discards a
/// value it does not have and changes what the enclosing block means.
pub(crate) fn is_block_like(expr: &RustExpr) -> bool {
    matches!(
        expr,
        RustExpr::If { .. } | RustExpr::Block(_) | RustExpr::Match { .. }
    )
}

/// The typed-literal spelling of a conversion, when that is what the conversion IS.
///
/// Only for an integer literal reaching an integer type: `int64(1)` in the source is the value one
/// at that width, and `1i64` says so where `1 as i64` says a conversion happened. A float target,
/// a non-literal operand, or anything with a sign or a point is left as a cast, because those are
/// conversions that can change the value and must keep saying so.
pub(crate) fn typed_literal(expr: &RustExpr, ty: &RustType) -> Option<String> {
    const INTEGER_TARGETS: &[&str] = &[
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
    ];
    let RustExpr::Literal(text) = expr else {
        return None;
    };
    let spelling = ty.spelling();
    if !INTEGER_TARGETS.contains(&spelling.as_str()) {
        return None;
    }
    if text.is_empty() || !text.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("{text}{spelling}"))
}
