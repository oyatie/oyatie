//! The `if`, and the block its init clause needs.
//!
//! The source scopes `if x := f(); cond` so that `x` is visible in the condition and in both
//! branches, and nowhere after. The target has exactly one construct with that shape — a block —
//! so the init statement becomes the block's first statement and the conditional its last.
//!
//! Hoisting the binding into the ENCLOSING scope would compile and would be a different program:
//! the name would outlive the branch, shadow differently, and drop later. The front end used to
//! refuse the whole construct on the claim that the target had no form for it; it has one, and the
//! thing that is unfaithful is the hoist rather than the `if`.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustStmt};

use crate::body::{Body, TailPosition, statement, translate};
use crate::body_parts::{named_child, one_child};
use crate::body_expr::expression;
use crate::error::TransformError;

/// An `if`, and the block its init clause needs.
///
/// The source scopes `if x := f(); cond` so that `x` is visible in the condition and in both
/// branches, and nowhere after. The target has exactly one construct with that shape -- a block --
/// so the init statement becomes the block's first statement and the conditional its last.
///
/// Hoisting the binding into the ENCLOSING scope would compile and would be a different program:
/// the name would outlive the branch, shadow differently, and drop later. That is the refusal the
/// front end used to make, and it refused the whole construct rather than the hoist.
pub(crate) fn conditional(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let inner = plain_conditional(node, cx)?;
    let Some(init) = node.children_of_kind("init").first().copied() else {
        return Ok(inner);
    };

    let bound = statement(one_child(init, cx, "init")?, cx, false)?;
    Ok(RustExpr::Block(vec![bound, RustStmt::Semi(inner)]))
}

fn plain_conditional(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let condition = named_child(node, "cond", cx, "if")?;
    let then = named_child(node, "then", cx, "if")?;

    let otherwise = match node.children_of_kind("else").first() {
        None => None,
        Some(branch) => {
            let inner = one_child(branch, cx, "else")?;
            Some(Box::new(match statement(inner, cx, false)? {
                RustStmt::Semi(expr) => expr,
                other => RustExpr::Block(vec![other]),
            }))
        }
    };

    Ok(RustExpr::If {
        cond: Box::new(expression(one_child(condition, cx, "cond")?, cx)?),
        // An `if` in statement position yields unit, so its branches keep their `return`s. Making
        // a branch yield a value here is what produced `if id == "" { fallback }` — which parses,
        // does not type-check, and is exactly the class of defect the compile proof exists for.
        then: translate(&then.children, cx, TailPosition::No)?,
        otherwise,
    })
}
