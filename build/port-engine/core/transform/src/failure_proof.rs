//! What the engine will accept as PROOF that a value is a failure.
//!
//! Split from `body_failure.rs` because the two answer different questions: that file says what a
//! failing return BECOMES, and this one says whether it may become that at all.
//!
//! The target's failing return is `Err(..)`; the source's is a value that MAY be absent. So `Err`
//! is faithful only where the operand cannot be — and where it can, the emitted program reports
//! failure at exactly the points the source reported success, which compiles and means something
//! else. Everything here exists to keep that from happening.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::naming::to_screaming_snake;
use crate::vocabulary::{
    ATTR_CALLEE, ATTR_OP, KIND_CALL, KIND_COMPOSITE, KIND_IDENT, KIND_UNARY, OPERATOR_ADDRESS_OF,
};

/// Whether this operand cannot be the ABSENT failure value.
///
/// The target's failing return is `Err(..)`; the source's failing return is a value that may be
/// absent. So `Err` is faithful only where the operand cannot be — and where it can, the emitted
/// program reports failure at exactly the points the source reported success, which compiles and
/// means something else.
///
/// TWO proofs, and nothing else:
///
/// - a CALL to a callee the pack names a failure constructor. A constructor has no absent result to
///   return, and which callees those are is the pack's to say — a source function that merely
///   RETURNS an error, like `Check(s) error`, is not one of them and is exactly the case this
///   distinguishes;
/// - the ADDRESS OF A FRESH COMPOSITE, which is never absent because the expression creates the
///   value. This needs no table: it is a property of the construct.
///
/// A field read, a package variable, a parameter, a plain binding: none is proven, and the case
/// that proves the point is a getter — `func (r *Report) Cause() error` returning a stored field,
/// whose source caller compares the result against the absent value.
///
/// The TESTED binding — `if err != nil { return 0, err }` — is not listed because it never reaches
/// here: the propagation rule recognises that whole shape and rewrites it to the target's operator,
/// which is the translation that makes the check impossible to forget.
pub(crate) fn is_certainly_a_failure(operand: &Declaration, cx: &Body<'_>) -> bool {
    match operand.kind.as_str() {
        KIND_CALL => cx.resolver.failure.is_some_and(|convention| {
            operand
                .attr(ATTR_CALLEE)
                .is_some_and(|callee| convention.constructors.contains(callee))
        }),
        KIND_UNARY => {
            operand.attr(ATTR_OP) == Some(OPERATOR_ADDRESS_OF)
                && operand
                    .children
                    .first()
                    .is_some_and(|inner| inner.kind == KIND_COMPOSITE)
        }
        // A SENTINEL, which the unit declares as a failure built by a declared constructor. Proven
        // for the same reason the direct call is: the constructor has no absent result to return,
        // and the sentinel is that call's value under a name.
        KIND_IDENT => cx.resolver.scope.sentinels.contains_key(&operand.name),
        _ => false,
    }
}

/// Whether an operand alongside a failure carries no information.
///
/// The source's convention is that a failing return's other operands are zero values. A literal or
/// the absent value is one; anything else is a computed value, and this is deliberately narrow —
/// admitting more would mean deciding that some expression is "obviously" zero, which is exactly
/// the guess this engine does not make.
pub(crate) fn discards_nothing(node: &Declaration, cx: &Body<'_>) -> bool {
    // The pack decides HOW FAR to trust the source's failure convention. Where it says the
    // companion may be discarded, every value is discardable — the source documents that a result
    // beside a non-nil error is not guaranteed to be meaningful, so a reader of a conforming
    // program cannot observe the difference. Where it does not, only a value the engine can SEE is
    // inert may go, which is faithful to the cases inspection can confirm and refuses the rest.
    if cx
        .resolver
        .failure
        .is_some_and(|convention| convention.discards_companion)
    {
        return true;
    }
    node.kind == "literal" || crate::failure::is_absent(node, cx.resolver.failure)
}
