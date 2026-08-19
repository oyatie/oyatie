//! A body that only hands another call's answer back.
//!
//! `check(s)?; Ok(())` runs a call, propagates its failure, and reports success — every one of
//! which `check(s)` already does. The extra shape exists because the SOURCE cannot say it in one
//! statement: its `if err != nil { return err }; return nil` is two statements and a convention,
//! where the target's return type is the whole statement.
//!
//! Split from `body_failure.rs` because that file says what a failing return BECOMES, and this
//! says when a whole body is one call.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustStmt;

use crate::body::Body;
use crate::body_expr::expression;
use crate::error::TransformError;

/// A call whose failure is propagated and whose success is the function's, which IS the call.
///
/// `check(s)?; Ok(())` at the END of a fallible body runs the call, hands its failure out, and
/// reports success — every one of which `check(s)` already does. The extra shape exists because the
/// SOURCE cannot say it in one statement: its `if err != nil { return err }; return nil` is two
/// statements and a convention, and the target's return type is the whole statement.
///
/// STRICT: the pair must be the LAST two statements, the success must carry no value, and the call
/// must produce nothing bound. A success that carries a value is a different function from the one
/// it called; a bound value means the body used it; and a pair that is not last leaves code after
/// a return this would delete.
///
/// # Errors
/// [`TransformError`] from translating the call.
pub(crate) fn forwarded_call(
    statements: &[Declaration],
    index: usize,
    cx: &Body<'_>,
    tail: crate::body::TailPosition,
) -> Result<Option<RustStmt>, TransformError> {
    // The propagation pair is two statements and the success is the third, so the three must be
    // all that is left: a pair that is not last leaves code after a return this would delete.
    if tail != crate::body::TailPosition::Yes || index + 3 != statements.len() || !cx.fallible {
        return Ok(None);
    }
    let Some(found) = crate::failure::propagation(statements, index, cx.resolver.failure) else {
        return Ok(None);
    };
    // Nothing bound: `v, err := f()` gives the body a value, and a body that has one is not simply
    // forwarding the call.
    if !found.values.is_empty() {
        return Ok(None);
    }
    let returning = statements.get(index + 2);
    if !returning.is_some_and(|node| is_bare_success(node, cx)) {
        return Ok(None);
    }
    Ok(Some(RustStmt::Tail(expression(found.source, cx)?)))
}

/// Whether this statement is a return of success carrying no value.
fn is_bare_success(node: &Declaration, cx: &Body<'_>) -> bool {
    node.kind == "return"
        && matches!(node.children.as_slice(),
            [only] if crate::failure::is_absent(only, cx.resolver.failure))
}
