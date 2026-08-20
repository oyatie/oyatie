//! Indexing and slicing.
//!
//! One module because they are the same question asked twice: what a subscript MEANS in the target.
//! The source indexes and slices the same backing array; the target distinguishes an element from a
//! borrowed view of several, and getting that wrong is a copy where the source had none.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::body_expr::expression;
use crate::error::TransformError;

/// An index operand, converted to what the target insists on.
///
/// The source indexes with its `int`; the target indexes with `usize`, and that mismatch has to
/// land somewhere. It used to land on `len`, which was mapped so its value would type as the
/// source's `int` — right for `return len(s)` and wrong for the counter of every indexed loop,
/// whose body then would not compile.
///
/// Here it lands at the only place the target actually insists: a LITERAL needs nothing, because
/// the target infers it, and every other operand converts.
///
/// The trade, recorded rather than discovered: a negative index panics in both languages for
/// different reasons — the source bounds-checks a negative, the target wraps it to an enormous
/// `usize` and bounds-checks that. Same outcome, different message.
pub(crate) fn index_operand(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let operand = expression(node, cx)?;
    if node.kind == "literal" {
        return Ok(operand);
    }
    // A proven index-only COUNTER is already a `usize`, because the range that built it dropped its
    // own conversion. Both ends read the same proof; converting here would convert a `usize` to a
    // `usize` and say something about the value that is not true.
    if node.kind == "ident" && cx.usize_counters.contains(&node.name) {
        return Ok(operand);
    }
    Ok(RustExpr::Cast {
        expr: Box::new(operand),
        ty: port_engine_rust_ir::RustType::path("usize"),
    })
}

/// `s[lo:hi]` — a BORROWED subrange, with either bound optional.
///
/// The bounds arrive positionally with an explicit `absent` node for the ones the source left out,
/// so `s[:hi]` and `s[lo:]` stay distinguishable. Reconstructing that from arity would be guessing
/// which end was missing.
pub(crate) fn slice(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let [base, low, high] = node.children.as_slice() else {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "a slice expression needs a base and both bounds, got {} operands",
                node.children.len()
            ),
        });
    };
    // A slice bound is an index like any other, and needs the same conversion for the same reason.
    let bound = |operand: &Declaration| -> Result<Option<Box<RustExpr>>, TransformError> {
        if operand.kind == "absent" {
            return Ok(None);
        }
        Ok(Some(Box::new(index_operand(operand, cx)?)))
    };
    Ok(RustExpr::Slice {
        base: Box::new(unwrapped_base(base, cx)?),
        low: bound(low)?,
        high: bound(high)?,
    })
}

/// The base of an index or slice, reaching through a NEWTYPE where the target wraps one.
///
/// The source's named array IS the array — `type ID [12]byte` admits `id[:]` because the name and
/// the array are the same thing there. The target's newtype wraps it, so the same expression has to
/// reach the field first, and emitting the source's spelling produces `cannot index into a value of
/// type &Id`.
///
/// Only for the RECEIVER, because that is the case the body can answer. The front end records a type
/// on an expression only where one is needed and a receiver carries none, so what the body knows is
/// which declaration it is inside — and the scope maps that to whether the target shape wraps. An
/// index through any other binding of a newtype is a shape the corpus does not have, and it arrives
/// here unchanged rather than being guessed at.
pub(crate) fn unwrapped_base(base: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let translated = expression(base, cx)?;
    let wraps = match crate::body_ops::is_receiver(base) {
        true => cx
            .receiver_type
            .is_some_and(|owner| cx.resolver.scope.newtypes.contains(owner)),
        // A PARAMETER of newtype type, which the signature stated and the body was told.
        false => base.kind == crate::vocabulary::KIND_IDENT
            && cx.newtype_parameters.contains(&base.name),
    };
    if !wraps {
        return Ok(translated);
    }
    Ok(RustExpr::Field {
        base: Box::new(translated),
        name: "0".to_owned(),
    })
}
