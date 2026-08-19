//! Whether a loop counter is ever anything but an INDEX, and what follows if it is not.
//!
//! The source counts with its `int` and the target indexes with `usize`, so the engine converts at
//! both ends: `for i in 0..values.len() as i64` and then `values[i as usize]`. Where the counter is
//! used for nothing else, the signed value is never observed and both conversions come off — the
//! counter is a `usize` from the start.
//!
//! Equivalent rather than merely tidier, and the argument is the loop's own bound: the range's
//! upper bound IS a length, so no value the loop produces can be negative or exceed `usize`, and
//! the round trip is the identity for every one of them. That is what makes this an idiom — it
//! changes the spelling and not the program.
//!
//! ONE read that is not an index and the conversions stay. Passed to a function, compared against
//! something the source typed `int`, stored in a field: in each of those the signed value IS
//! observed, and dropping the conversion would change what the program computes with.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::body_expr::expression;
use crate::error::TransformError;
use crate::vocabulary::{ATTR_CALLEE, KIND_CALL, KIND_IDENT, KIND_INDEX};

/// Whether every read of `counter` inside this body is a sequence index.
///
/// Walks the whole subtree rather than the top level, because a counter read inside a nested `if`
/// is read just as much as one at the top — and a rule that only looked at the top would call a
/// loop proven while its branch passed the counter somewhere.
///
/// A body with NO read of the counter is proven trivially, and correctly: a counter nothing reads
/// cannot have its signed value observed.
pub(crate) fn indexes_only(body: &Declaration, counter: &str) -> bool {
    let mut reads = 0usize;
    let mut indexed = 0usize;
    count_reads(body, counter, &mut reads, &mut indexed);
    reads == indexed
}

/// Total reads of the name, and how many of them stand in an index position.
fn count_reads(node: &Declaration, counter: &str, reads: &mut usize, indexed: &mut usize) {
    if node.kind == KIND_IDENT && node.name == counter {
        *reads += 1;
    }
    // An INDEX node's second child is the index operand. A counter standing there is read as an
    // index; a counter anywhere else in the same expression — inside the base, or inside a
    // compound index expression — is not, and is counted by the ordinary walk below.
    if node.kind == KIND_INDEX
        && let Some(operand) = node.children.get(1)
        && operand.kind == KIND_IDENT
        && operand.name == counter
    {
        *indexed += 1;
    }
    for child in &node.children {
        count_reads(child, counter, reads, indexed);
    }
}

/// The loop's upper bound with its conversion dropped, when the bound is a length.
///
/// Only a `len` call qualifies, and that is the argument rather than a convenience: a length is
/// already a `usize` in the target and the conversion the pack's mapping adds exists to make the
/// value type as the SOURCE's `int`. Where the value is never observed as one, the conversion is
/// what is wrong. Any other bound keeps its own translation, because nothing says its value fits.
///
/// # Errors
/// [`TransformError`] from translating the bound.
pub(crate) fn unsigned_bound(
    bound: &Declaration,
    cx: &Body<'_>,
) -> Result<RustExpr, TransformError> {
    let translated = expression(bound, cx)?;
    if bound.kind != KIND_CALL {
        return Ok(translated);
    }
    let Some(identity) = bound.attr(ATTR_CALLEE) else {
        return Ok(translated);
    };
    let Some(mapping) = cx.resolver.function_map.get(identity) else {
        return Ok(translated);
    };
    // The pack's own form, with the target's own conversion syntax stripped from its END. Done by
    // comparing against the form the pack declares rather than by editing the rendered text, so a
    // pack whose mapping has no trailing conversion is left exactly alone.
    let Some(prefix) = mapping.form.split_once(" as ") else {
        return Ok(translated);
    };
    let RustExpr::Literal(rendered) = &translated else {
        return Ok(translated);
    };
    let suffix = &mapping.form[prefix.0.len()..];
    match rendered.strip_suffix(suffix) {
        Some(trimmed) => Ok(RustExpr::Literal(trimmed.to_owned())),
        None => Ok(translated),
    }
}
