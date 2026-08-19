//! `x := 0; if c { x = a } else { x = b }` — a CHOICE the source has to spell as a mutation.
//!
//! The source's `if` is a statement, so a value chosen by a condition has to be written into a name
//! declared beforehand. The target's `if` is an EXPRESSION, and the same choice is spelled by
//! initialising the name from it. That is not a rewrite: the two run the same condition, evaluate
//! the same branch, and leave the same value in the same name.
//!
//! Three things follow from taking it, and each is a defect removed rather than a preference:
//!
//! - the declared name stops being `mut`, because nothing writes it after it is bound. The source
//!   had to make it assignable and the target does not;
//! - the initial value stops being emitted at all. It is dead — every path overwrites it before any
//!   read — and the target's flow analysis says so, which is the one warning a faithful port used to
//!   have no answer for;
//! - a reviewer stops reading `let mut result = 0;` followed by a bare block, which is the shape you
//!   write when your language does not have `if` as an expression, and which two reviewers named as
//!   a translation artifact.
//!
//! STRICT, in the same way the propagation matchers are. The `if` must be the VERY NEXT statement,
//! both branches must be present, and each must consist of exactly one plain assignment to the
//! declared name. Anything else — a branch that does more, a branch that is missing, a compound
//! assignment that reads the old value — is a program this does not express, and emitting the
//! expression form for it would drop whatever else the branch did.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustStmt};

use crate::body::Body;
use crate::body_cond::branch_value;
use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::vocabulary::{ATTR_OP, KIND_IDENT};

/// The binding an `if` at `index + 1` initialises, if the pair is one choice.
///
/// # Errors
/// [`TransformError`] from translating the condition or either branch's value.
pub(crate) fn choice(
    statements: &[Declaration],
    index: usize,
    cx: &Body<'_>,
) -> Result<Option<RustStmt>, TransformError> {
    let Some(bound) = statements.get(index).filter(|node| node.kind == "let") else {
        return Ok(None);
    };
    let Some(conditional) = statements.get(index + 1).filter(|node| node.kind == "if") else {
        return Ok(None);
    };
    // BOTH branches. Without an else the source leaves the initial value standing on one path, and
    // that value is then live rather than dead — the very thing this rule relies on being absent.
    let (Some(then), Some(otherwise)) = (
        conditional.children_of_kind("then").first().copied(),
        conditional.children_of_kind("else").first().copied(),
    ) else {
        return Ok(None);
    };
    let (Some(a), Some(b)) = (
        assigned_value(then, &bound.name),
        assigned_value(otherwise, &bound.name),
    ) else {
        return Ok(None);
    };

    Ok(Some(RustStmt::Let {
        name: to_snake_case(&bound.name),
        // Nothing writes it after it is bound, which is the whole point: the source needed it
        // assignable only because its `if` could not produce a value.
        mutable: false,
        ty: None,
        value: Some(branch_value(conditional, cx, a, b)?),
    }))
}

/// The expression a branch assigns to `name`, if that is all the branch does.
///
/// `None` for a branch that does anything else, that assigns to a different name, or that uses a
/// COMPOUND assignment — the last because `x += e` reads the old value, and the old value here is
/// the initialiser this rule is about to stop emitting.
fn assigned_value<'a>(branch: &'a Declaration, name: &str) -> Option<&'a Declaration> {
    let [only] = branch.children.as_slice() else {
        return None;
    };
    // An `else` arrives wrapped in a block where a `then` does not — the source's `else` holds a
    // statement and its `then` holds a statement LIST. One level, not a walk: a block inside a
    // block is a scope the source wrote deliberately, and flattening it would be a rewrite.
    let only = match only.kind == "block" {
        true => match only.children.as_slice() {
            [inner] => inner,
            _ => return None,
        },
        false => only,
    };
    if only.kind != "assign" || only.attr(ATTR_OP).is_some() {
        return None;
    }
    let [target, value] = only.children.as_slice() else {
        return None;
    };
    match target.kind == KIND_IDENT && target.name == name {
        true => Some(value),
        false => None,
    }
}
