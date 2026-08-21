//! Function literals, and the ownership question that decides which ones translate.
//!
//! A Go closure and a target closure look alike and differ in the one thing that matters: who owns
//! what the body reaches out of its own scope to use. Go's closure SHARES the enclosing function's
//! variables — the same storage, for as long as the closure is reachable — and the target has four
//! separate answers for that (borrow, borrow mutably, take ownership, share a handle), each with a
//! different meaning and a different proof.
//!
//! The target infers the first two for a closure that does not outlive its scope, which is why the
//! capture-free and non-escaping cases need no analysis at all. What needs analysis is the closure
//! that ESCAPES, and the proof it needs is one this engine cannot discharge yet: whether a callee
//! retains the value it is given. For a callee outside the corpus that is unknowable, and guessing
//! it produces either a borrow checker error or — worse — a `move` that silently stops the source's
//! sharing.
//!
//! So this file translates what is provable and refuses the rest BY NAME, listing the captures, so
//! the refusal says which decision is missing rather than that closures are unsupported.

use port_engine_api::Declaration;
use port_engine_rust_ir::{ClosureParam, RustExpr, RustStmt};

use crate::body::{Body, TailPosition, translate};
use crate::body_parts::branch;
use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::vocabulary::{CHILD_PARAM, FLAG_MUTATED};

/// The destination the front end records for a literal among a `return`'s operands.
const DESTINATION_RETURN: &str = "return";

/// A function literal.
///
/// # Errors
/// [`TransformError::Unsupported`] when the literal captures anything, or when its body does not
/// translate.
pub(crate) fn closure(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let captures = node.children_of_kind("capture");
    // OWNED, when the source says where the literal goes and owning is faithful there.
    //
    // Ownership is a property of the DESTINATION, not of the literal, and for most destinations the
    // engine cannot see it. For one it can: a literal among a `return`'s operands outlives the
    // frame it is written in, so its captures cannot be borrowed from that frame and must be owned.
    // The front end records the destination because it is a source fact.
    //
    // Owning is not automatically faithful. Go's closure shares the variable's STORAGE, so where
    // anything reassigns a capture -- the enclosing body, or a second literal over the same
    // variable -- the source has one value and `move` would make several. A variable nothing
    // reassigns has one value for its whole life, and there a copy is indistinguishable from the
    // original. So the reassigned case keeps refusing, and says which capture forced it.
    if !captures.is_empty()
        && node.attr(crate::vocabulary::ATTR_DESTINATION) == Some(DESTINATION_RETURN)
    {
        if let Some(shared) = captures
            .iter()
            .find(|capture| capture.has_flag(crate::vocabulary::FLAG_REASSIGNED))
        {
            return Err(TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: format!(
                    "a function literal is RETURNED and captures `{}`, which the enclosing body                      reassigns. The source shares one variable with the returned literal, so a                      later write is visible through it; the target's returned closure must own                      what it captures, and an owned copy stops agreeing at that write. Sharing a                      variable that outlives its frame needs a decision about which handle the two                      hold — which the source does not record",
                    shared.name
                ),
            });
        }
        return Ok(RustExpr::Closure {
            moves: true,
            params: closure_params(node),
            ret: None,
            body: translate_tail(&branch(node, "body", cx)?.children, cx)?,
        });
    }
    if !captures.is_empty() {
        // The captured NAMES are deliberately not in this text. They are the site, and a cause that
        // carries its site counts once per site — which is how one undecided form read as eighteen
        // rows in R3j and hid the largest blocker in the corpus. What a reader needs here is the
        // missing DECISION, and that is the same decision at every one of these sites.
        let written = captures
            .iter()
            .any(|capture| capture.has_flag(FLAG_MUTATED));
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "a function literal CAPTURES{}, and each capture needs an owner — which is a \
                 property of where the literal GOES rather than of the literal. A closure that \
                 does not outlive its scope borrows, and the target infers that; one that escapes \
                 must take ownership. Whether a callee retains the value it is given is not \
                 something this engine can prove for a callee outside the corpus, so the escaping \
                 case has no answer yet",
                match written {
                    true => ", and writes at least one capture",
                    false => "",
                }
            ),
        });
    }

    let params = closure_params(node);

    let body = branch(node, "body", cx)?;
    Ok(RustExpr::Closure {
        // NOTHING TO MOVE. A literal that captures nothing owns nothing, so `move` would be a
        // keyword with no operand — and adding it unconditionally is how a translator ends up
        // moving values the source shared.
        moves: false,
        params,
        ret: None,
        body: translate_tail(&body.children, cx)?,
    })
}

/// The literal's parameters, cased for the target.
fn closure_params(node: &Declaration) -> Vec<ClosureParam> {
    node.children_of_kind(CHILD_PARAM)
        .iter()
        .map(|param| ClosureParam {
            name: match param.name.is_empty() || param.name == "_" {
                true => "_".to_owned(),
                false => to_snake_case(&param.name),
            },
            // NO DECLARED TYPE. A literal with no captures is only reachable in a position that
            // supplies one — an argument, whose callee states the parameter — and stating it again
            // is noise a reviewer notices. Where nothing supplies one the target says so, and that
            // is a compile error rather than a wrong program.
            ty: None,
        })
        .collect()
}

/// The literal's body, whose last statement is its VALUE.
///
/// A closure body is an expression in the target, so its final statement yields rather than
/// returning — the same rule the top level of a function body already follows, applied here because
/// a literal is a body the enclosing translation does not own.
fn translate_tail(
    statements: &[Declaration],
    cx: &Body<'_>,
) -> Result<Vec<RustStmt>, TransformError> {
    translate(statements, cx, TailPosition::Yes)
}
