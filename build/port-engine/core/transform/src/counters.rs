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

use std::collections::BTreeSet;

use port_engine_api::Declaration;
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::body_expr::expression;
use crate::error::TransformError;
use crate::vocabulary::{ATTR_CALLEE, ATTR_OP, KIND_CALL, KIND_IDENT, KIND_INDEX};

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

/// The sequence a counted loop walks, if it walks exactly one and nothing else.
///
/// `Some(name)` when the bound is `len(xs)` and every read of the counter is `xs[i]` for that same
/// `xs`. Two different sequences indexed by one counter is a walk of neither, and a bound that is
/// not a length says nothing about the range.
pub(crate) fn walked_sequence(
    bound: &Declaration,
    body: &Declaration,
    counter: &str,
    lengths: &BTreeSet<String>,
) -> Option<String> {
    if bound.kind != KIND_CALL || !lengths.contains(bound.attr(ATTR_CALLEE)?) {
        return None;
    }
    // The callee's own selector is a child too, so the sequence is the first IDENT argument.
    let sequence = bound
        .children
        .iter()
        .skip(1)
        .find(|child| child.kind == KIND_IDENT)?;
    let mut indexed = BTreeSet::new();
    collect_indexed(body, counter, &mut indexed);
    match indexed.len() == 1 && indexed.contains(&sequence.name) {
        true => Some(sequence.name.clone()),
        false => None,
    }
}

/// Whether every element this loop reads COPIES in the target.
///
/// Required, because the walk hands the body each element by value exactly as the source's index
/// read does. An element that MOVES would be moved out of the sequence by the first read, which the
/// source never does — so a sequence of those keeps its counter and its indexed reads.
pub(crate) fn elements_copy(body: &Declaration, counter: &str, cx: &Body<'_>) -> bool {
    let mut reads = Vec::new();
    collect_indexed_nodes(body, counter, &mut reads);
    !reads.is_empty()
        && reads.iter().all(|node| {
            node.children
                .first()
                .and_then(|base| base.type_ref.args.first())
                .is_some_and(|element| cx.resolver.copy_types.contains(&element.name))
        })
}

/// Every index node this counter appears in, so the element type can be read off the base.
fn collect_indexed_nodes<'a>(node: &'a Declaration, counter: &str, out: &mut Vec<&'a Declaration>) {
    if node.kind == KIND_INDEX
        && node
            .children
            .get(1)
            .is_some_and(|operand| operand.kind == KIND_IDENT && operand.name == counter)
    {
        out.push(node);
    }
    for child in &node.children {
        collect_indexed_nodes(child, counter, out);
    }
}

/// Every sequence name this counter indexes into, anywhere in the body.
fn collect_indexed(node: &Declaration, counter: &str, out: &mut BTreeSet<String>) {
    if node.kind == KIND_INDEX
        && let [base, operand] = node.children.as_slice()
        && operand.kind == KIND_IDENT
        && operand.name == counter
        && base.kind == KIND_IDENT
    {
        out.insert(base.name.clone());
    }
    for child in &node.children {
        collect_indexed(child, counter, out);
    }
}

/// The name a loop element takes, when the source gives none.
///
/// The sequence's own name with a trailing plural removed. `None` where that yields nothing usable
/// or collides with a name the body already binds — an invented name that shadows a real one would
/// be a different program, and keeping the counter is the honest fallback.
pub(crate) fn element_name(sequence: &str, body: &Declaration) -> Option<String> {
    let singular = sequence.strip_suffix('s').filter(|rest| !rest.is_empty())?;
    let mut bound = BTreeSet::new();
    collect_bound_names(body, &mut bound);
    match bound.contains(singular) || singular == sequence {
        true => None,
        false => Some(singular.to_owned()),
    }
}

/// Every name the body binds, so an invented one cannot shadow a real one.
fn collect_bound_names(node: &Declaration, out: &mut BTreeSet<String>) {
    if matches!(node.kind.as_str(), "let" | "bind" | "ident" | "param") && !node.name.is_empty() {
        out.insert(node.name.clone());
    }
    for child in &node.children {
        collect_bound_names(child, out);
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
    // THROUGH an arithmetic expression, because a bound is often computed: `len(magic) + 8*5 + 32`
    // is a length built from one, and the conversion has to come off the part that has it rather
    // than off the whole. Only the operators that keep a length a length — the source's own
    // arithmetic — and each side answered by the same question this one is.
    if bound.kind == "binary"
        && let [left, right] = bound.children.as_slice()
        && let Some(op) = bound.attr(ATTR_OP).and_then(crate::body_ops::binary_operator)
    {
        return Ok(RustExpr::Binary {
            op,
            lhs: Box::new(unsigned_bound(left, cx)?),
            rhs: Box::new(unsigned_bound(right, cx)?),
        });
    }
    let translated = expression(bound, cx)?;
    if bound.kind != KIND_CALL {
        return Ok(translated);
    }
    // A mapped call whose form ends in a conversion arrives as a CAST NODE, so the conversion comes
    // off by unwrapping it. This used to strip the rendered text against the pack's declared form,
    // which worked and depended on how the call had printed; reading the node instead means a
    // change to the rendering cannot silently stop it working.
    match translated {
        RustExpr::Cast { expr, .. } => Ok(*expr),
        other => Ok(other),
    }
}
