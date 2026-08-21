//! One argument, in the shape its DESTINATION wants.
//!
//! Split from `body_call.rs` because the two answer different questions. That file says what a
//! call becomes; this says what a value has to be to cross into a parameter — which is where every
//! ownership decision the pack makes is actually spent, and where a value entering the target's
//! index type converts.
//!
//! An argument knows what it IS and not where it is going, which is the whole reason the signature
//! table exists: `&x` and a bare string literal both need the destination, and both destinations
//! are signatures the engine has already translated.

use port_engine_api::{Declaration, PointerConstruction};
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::body_expr::expression;
use crate::body_ops::{operator_of, own_string_for};
use crate::body_parts::one_child;
use crate::error::TransformError;
use crate::vocabulary::{KIND_UNARY, OPERATOR_ADDRESS_OF};

/// One argument, translated for the parameter it reaches.
///
/// Two conversions become possible that an isolated expression cannot make, and both are the SAME
/// decision the parameter position already made, applied at the other end:
///
///   * A pointer operand takes the construction its disposition declares — found by the id the
///     parameter recorded, never by matching the spelling that decision produced.
///   * A bare string literal is owned when the parameter holds the owned string target.
///
/// A destination the table cannot give is NOT "no conversion needed": the argument is translated
/// as it stands, and a pointer operand refuses by name saying what was missing.
pub(crate) fn argument(
    node: &Declaration,
    callee: &str,
    index: usize,
    cx: &Body<'_>,
) -> Result<RustExpr, TransformError> {
    let target = cx.resolver.signatures.param(callee, index);

    // A parameter the callee's signature made the target's INDEX TYPE takes an index. The caller's
    // value is the source's own integer, so it converts here — the same conversion an index
    // operand makes, at the one other place a value crosses into that type. Without it the callee
    // would have a signature its own callers could not satisfy, which is a worse defect than the
    // conversion this removes from inside the callee.
    if target.is_some_and(|target| target.ty.spelling() == "usize") {
        return crate::body_index::index_operand(node, cx);
    }

    if node.kind == KIND_UNARY && operator_of(node, cx)? == OPERATOR_ADDRESS_OF {
        let operand = expression(one_child(node, cx, KIND_UNARY)?, cx)?;
        let construction = target
            .and_then(|target| target.disposition.as_deref())
            .and_then(|id| cx.resolver.ownership.construction_for(id));
        let Some(construction) = construction else {
            return Err(TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: address_of_refusal(callee, index, target.is_some()),
            });
        };
        return Ok(constructed(construction, operand));
    }

    let expr = expression(node, cx)?;
    let Some(target) = target else {
        return Ok(expr);
    };
    // A parameter the callee BORROWS takes a borrow. The argument was built for a value position —
    // a field read of a non-copying type CLONES, because reading one moves in the target — and a
    // clone handed to a borrowing parameter is both the wrong type and an allocation the source
    // never performed. `parse_int(self.str.clone())` where the callee takes `&str`.
    //
    // This is the call-site half of a decision the signature already made, applied at the other
    // end — the same shape as the pointer disposition above, for the case where the parameter is a
    // borrow the pack chose rather than one a disposition did.
    // Read from the SPELLING, because a borrowed parameter reaches the table both ways: the pack's
    // slice and string idioms produce a path that already carries the `&`, and a pointer
    // disposition produces a structured reference. One test that covers both is one answer; two
    // tests are how the signature and the call site come to disagree.
    if target.ty.spelling().starts_with('&') {
        return Ok(borrowed_for(expr, cx));
    }
    Ok(own_string_for(expr, &target.ty.spelling(), cx))
}

/// An argument for a parameter the callee BORROWS.
///
/// A `.clone()` is UNDONE rather than borrowed: `&x.clone()` borrows a temporary that dies at the
/// end of the statement, and the clone itself exists only because a value position needed an owned
/// copy. This position does not.
///
/// An expression that is already a reference is left exactly alone — borrowing it again yields a
/// double reference, which is the defect the range loop already had to learn.
fn borrowed_for(expr: RustExpr, cx: &Body<'_>) -> RustExpr {
    if let RustExpr::Reference { .. } = expr {
        return expr;
    }
    // ALREADY BORROWED because the enclosing signature borrows it. `u64(&b)` where `b` is already
    // `&[u8]` is `clippy::needless_borrow`, and under the deny-warnings policy that is a build
    // failure. The same answer the signature gave, read rather than derived a second time — which
    // is what the range loop reads for exactly this question.
    if let RustExpr::Path(name) = &expr
        && cx.borrowed.contains(name)
    {
        return expr;
    }
    let inner = match expr {
        RustExpr::MethodCall {
            receiver,
            ref method,
            ref args,
        } if method == "clone" && args.is_empty() => *receiver,
        other => other,
    };
    RustExpr::Reference {
        mutable: false,
        inner: Box::new(inner),
    }
}

/// Build the argument the construction describes.
///
/// `Wrap` applies its paths INNERMOST FIRST, which is the order the value passes through them and
/// the order they are declared.
pub(crate) fn constructed(construction: &PointerConstruction, operand: RustExpr) -> RustExpr {
    match construction {
        PointerConstruction::Borrow { mutable, .. } => RustExpr::Reference {
            mutable: *mutable,
            inner: Box::new(operand),
        },
        PointerConstruction::Wrap { paths, .. } => {
            paths.iter().fold(operand, |inner, path| RustExpr::Call {
                callee: Box::new(RustExpr::Path(path.clone())),
                args: vec![inner],
            })
        }
    }
}

/// Why a `&x` argument could not be given a form, in terms of what was missing.
fn address_of_refusal(callee: &str, index: usize, has_target: bool) -> String {
    if callee.is_empty() {
        return format!(
            "unary `&` is argument {index} of a METHOD call, whose signature the table does not \
             hold: a method's key is its receiver type rather than a path"
        );
    }
    if !has_target {
        return format!(
            "unary `&` is argument {index} of `{callee}`, whose signature is not in the snapshot — \
             it is foreign, or the engine could not translate it"
        );
    }
    format!(
        "unary `&` is argument {index} of `{callee}`, whose parameter is not a pointer, so no \
         disposition decided how an argument reaches it"
    )
}
