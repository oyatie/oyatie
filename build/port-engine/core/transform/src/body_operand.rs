//! One operand of a return, in the shape its RESULT POSITION wants.
//!
//! Split from `body_failure.rs` because the two answer different questions: that file decides
//! whether a return is a failure or a success, and this decides what the values it carries have to
//! BE. Every result idiom lands here — a pointer that is a value, a field read that is a view, a
//! length that keeps its `usize`, a comparison literal that names an ordering — and each is the
//! body's half of a decision the signature already made.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::body_expr::expression;
use crate::error::TransformError;
use crate::vocabulary::{ATTR_OP, ATTR_VALUE, KIND_UNARY, OPERATOR_ADDRESS_OF};

/// One operand of a return, in the shape its RESULT POSITION wants.
///
/// A `&T{..}` is the owned pointer everywhere except a position the signature proved is never
/// absent, where it is the value itself. The signature and this must agree or the function does not
/// compile, which is why both read the same proof rather than each deciding.
///
/// # Errors
/// [`TransformError`] from translating the operand.
pub(crate) fn returned_operand(
    index: usize,
    operand: &Declaration,
    cx: &Body<'_>,
) -> Result<RustExpr, TransformError> {
    // An ORDERING result: the source's three literals ARE the ordering, and each names a variant.
    if cx.results.is_an_ordering && index == 0 {
        return Ok(RustExpr::Path(ordering_variant(operand)));
    }
    // A LENGTH result keeps the length, so the call's own conversion comes off — through the one
    // function that knows how, which strips only the form the pack declares.
    if cx.results.is_a_length && index == 0 {
        return crate::counters::unsigned_bound(operand, cx);
    }
    // A getter's result is a BORROW of the receiver, so the field read is not a copy and needs no
    // clone. Same proof the signature read, so the two cannot disagree.
    // A stored FAILURE is lent, not copied: the field holds an owned optional box and the result
    // is an optional borrow. The target spells that conversion with one method, and which method
    // is the pack's to say.
    if cx.results.borrows_failure
        && index == 0
        && let Some(method) = cx.resolver.idiom_method(crate::vocabulary::IDIOM_FAILURE_GETTER)
    {
        return Ok(RustExpr::MethodCall {
            receiver: Box::new(crate::body_place::field_place(operand, cx)?),
            method: method.to_owned(),
            args: Vec::new(),
        });
    }
    if cx.results.borrows_receiver && index == 0 {
        return Ok(RustExpr::Reference {
            mutable: false,
            inner: Box::new(crate::body_place::field_place(operand, cx)?),
        });
    }
    if cx.results.bare_pointers.contains(&index)
        && operand.kind == KIND_UNARY
        && operand.attr(ATTR_OP) == Some(OPERATOR_ADDRESS_OF)
        && let Some(inner) = operand.children.first()
    {
        return expression(inner, cx);
    }
    expression(operand, cx)
}

/// The ordering variant one of the source's three comparison literals names.
///
/// The mapping is the target's own definition of what those integers mean: negative is less,
/// positive is greater, zero is equal. Reached only for an operand the signature already proved is
/// one of the three, so the fallback cannot be taken.
fn ordering_variant(operand: &Declaration) -> String {
    let equal = operand.attr(ATTR_VALUE) == Some("0");
    let greater = operand.attr(ATTR_VALUE) == Some("1");
    match (equal, greater) {
        (true, _) => "std::cmp::Ordering::Equal".to_owned(),
        (_, true) => "std::cmp::Ordering::Greater".to_owned(),
        _ => "std::cmp::Ordering::Less".to_owned(),
    }
}
