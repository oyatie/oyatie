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

use crate::body::{Body, TailPosition, translate};
use crate::body_expr::expression;
use crate::body_parts::{named_child, one_child};
use crate::body_stmt::statement;
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

/// The same `if`, as an EXPRESSION whose branches yield the two given values.
///
/// Built here rather than in `body_choice` so one place decides what an `if` becomes: the condition,
/// the init clause and the block that scopes it are all this module's, and a second builder would be
/// free to scope the init differently.
///
/// The init clause keeps its block, which is what makes this faithful. The binding stays scoped to
/// the conditional exactly as the source scopes it — the whole block simply becomes the value now
/// rather than a statement.
///
/// # Errors
/// [`TransformError`] from translating the condition, the init clause, or either value.
pub(crate) fn branch_value(
    node: &Declaration,
    cx: &Body<'_>,
    then_value: &Declaration,
    else_value: &Declaration,
) -> Result<RustExpr, TransformError> {
    let condition = named_child(node, "cond", cx, "if")?;
    let chosen = RustExpr::If {
        cond: Box::new(expression(one_child(condition, cx, "cond")?, cx)?),
        then: vec![RustStmt::Tail(expression(then_value, cx)?)],
        // The target's `else` takes a block or another `if`, never a bare expression, so the
        // value is wrapped exactly as a `then` branch's statement list already is.
        otherwise: Some(Box::new(RustExpr::Block(vec![RustStmt::Tail(expression(
            else_value, cx,
        )?)]))),
    };
    let Some(init) = node.children_of_kind("init").first().copied() else {
        return Ok(chosen);
    };
    let bound = statement(one_child(init, cx, "init")?, cx, false)?;
    Ok(RustExpr::Block(vec![bound, RustStmt::Tail(chosen)]))
}

/// An `if` whose init clause is hoisted out of the block that scoped it.
///
/// The source scopes `if size := len(s); cond` so `size` dies with the branch, and the block this
/// module emits is what reproduces that. The block is only NECESSARY where the scope can be
/// observed, and for a binding whose type COPIES there is nothing to observe: a copy type has no
/// drop to delay, so the only remaining difference is shadowing.
///
/// `None` unless all of:
///
/// - the statement is an `if` with an init clause binding one name;
/// - that binding's type COPIES, so the scope has no drop in it;
/// - the name is not already bound in the enclosing body, so hoisting cannot shadow one;
/// - the name is not bound again after this statement, which would shadow the hoisted one.
///
/// Where any fails, the block stays — it is faithful, and it is what this module emitted before any
/// of this. Two reviewers read the bare block as a Go statement form transliterated, which it was;
/// what they could not see is that it was also the only faithful shape until the copy test existed.
///
/// # Errors
/// [`TransformError`] from translating the init clause or the conditional.
pub(crate) fn hoisted_init(
    statements: &[Declaration],
    index: usize,
    cx: &Body<'_>,
) -> Result<Option<Vec<RustStmt>>, TransformError> {
    let Some(node) = statements.get(index).filter(|node| node.kind == "if") else {
        return Ok(None);
    };
    let Some(init) = node.children_of_kind("init").first().copied() else {
        return Ok(None);
    };
    let Some(bound) = init.children.first().filter(|node| node.kind == "let") else {
        return Ok(None);
    };
    if !cx.resolver.copy_types.contains(&bound.type_ref.name) {
        return Ok(None);
    }
    if binds_elsewhere(statements, index, &bound.name) {
        return Ok(None);
    }
    Ok(Some(vec![
        statement(bound, cx, false)?,
        RustStmt::Semi(plain_conditional(node, cx)?),
    ]))
}

/// Whether any statement but this one binds the same name.
///
/// Hoisting a name into the enclosing scope is safe only where nothing else there has it: a name
/// bound before would be shadowed by the hoist, and one bound after would shadow it. Either is a
/// different program from the one the source scoped.
fn binds_elsewhere(statements: &[Declaration], skip: usize, name: &str) -> bool {
    statements
        .iter()
        .enumerate()
        .filter(|(position, _)| *position != skip)
        .any(|(_, node)| binds(node, name))
}

/// Whether this subtree binds the given name anywhere.
fn binds(node: &Declaration, name: &str) -> bool {
    (matches!(node.kind.as_str(), "let" | "bind") && node.name == name)
        || node.children.iter().any(|child| binds(child, name))
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
