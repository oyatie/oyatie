//! What the source's FAILURE CONVENTION becomes in a body.
//!
//! Two translations, and both are about the same disagreement: the source carries failure as an
//! extra value that a caller may drop, and the target carries it as the whole return, which a
//! caller cannot. A return has to pick a constructor, and a bind-and-check pair becomes an operator
//! that cannot be forgotten.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustStmt, TupleBind};

use crate::body::Body;
use crate::body_expr::expression;
use crate::body_ops::own_returned_string;
use crate::error::TransformError;
use crate::naming::to_snake_case;

/// The source's bind-and-check pair as one operator.
///
/// This is where the translation stops being a rename. The source's version is a CONVENTION — the
/// check is two statements a caller may simply not write, and nothing in the type system notices.
/// The target's is an operator on a type that cannot be used without addressing the failure, so the
/// same program becomes one the compiler enforces.
pub(crate) fn propagate(
    found: &crate::failure::Propagation<'_>,
    cx: &Body<'_>,
) -> Result<RustStmt, TransformError> {
    let value = RustExpr::Try(Box::new(expression(found.source, cx)?));
    match found.values.as_slice() {
        // `err := f()` with no value bound: the call is run for its effect and its failure
        // propagates. There is nothing to name.
        [] => Ok(RustStmt::Semi(value)),
        [only] => Ok(RustStmt::Let {
            name: to_snake_case(only),
            // The operator's own binding is never written again: it exists to name the value the
            // call produced, and anything that writes it does so under a name of its own.
            mutable: false,
            ty: None,
            value: Some(value),
        }),
        // The propagation operator has already consumed the failure, so these are the VALUE
        // names alone. They are the operator's own bindings and are never written again — anything
        // that writes one does so under a name of its own, exactly as in the single-value arm.
        several => Ok(RustStmt::LetTuple {
            names: several
                .iter()
                .map(|name| TupleBind {
                    name: to_snake_case(name),
                    mutable: false,
                })
                .collect(),
            value,
        }),
    }
}

pub(crate) fn translated_return(
    node: &Declaration,
    cx: &Body<'_>,
    is_last: bool,
) -> Result<RustStmt, TransformError> {
    let value = if cx.fallible {
        Some(fallible_return(node, cx)?)
    } else {
        let values = node
            .children
            .iter()
            .map(|child| expression(child, cx))
            .collect::<Result<Vec<_>, _>>()?;
        match values.len() {
            0 => None,
            1 => values.into_iter().next().map(|expr| own_returned_string(expr, cx)),
            // Several results leave as a tuple, matching how the signature renders them.
            _ => Some(RustExpr::Tuple(values)),
        }
    };
    match (is_last, value) {
        (true, Some(expr)) => Ok(RustStmt::Tail(expr)),
        (_, value) => Ok(RustStmt::Return(value)),
    }
}

/// A return from a function that can fail: the TRAILING operand decides the whole construction.
///
/// The source returns the failure alongside the value; the target returns one or the other. So a
/// failing return DISCARDS its companion operands, and that is only sound because the source's
/// convention is that they are the zero value — a caller that reads them after a non-nil failure is
/// reading something the source promised nothing about. Anything else refuses, because discarding a
/// computed value is a silent loss of work the reader would never see.
pub(crate) fn fallible_return(
    node: &Declaration,
    cx: &Body<'_>,
) -> Result<RustExpr, TransformError> {
    let Some((failure, values)) = node.children.split_last() else {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: "a bare return from a function that can fail says nothing about whether it \
                     succeeded"
                .to_owned(),
        });
    };

    if crate::failure::is_absent(failure, cx.resolver.failure) {
        let values = values
            .iter()
            .map(|child| expression(child, cx))
            .collect::<Result<Vec<_>, _>>()?;
        let ok = match values.len() {
            0 => RustExpr::Tuple(Vec::new()),
            1 => values.into_iter().next().unwrap_or(RustExpr::Todo),
            _ => RustExpr::Tuple(values),
        };
        return Ok(RustExpr::Call {
            callee: Box::new(RustExpr::Path("Ok".to_owned())),
            args: vec![ok],
        });
    }

    for discarded in values {
        if !discards_nothing(discarded, cx) {
            return Err(TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: "a failing return carries a computed value beside the failure, and the \
                         target's failing return carries only the failure — dropping the value \
                         here would lose work the reader cannot see was lost"
                    .to_owned(),
            });
        }
    }
    Ok(RustExpr::Call {
        callee: Box::new(RustExpr::Path("Err".to_owned())),
        args: vec![expression(failure, cx)?],
    })
}

/// Whether an operand alongside a failure carries no information.
///
/// The source's convention is that a failing return's other operands are zero values. A literal or
/// the absent value is one; anything else is a computed value, and this is deliberately narrow —
/// admitting more would mean deciding that some expression is "obviously" zero, which is exactly
/// the guess this engine does not make.
fn discards_nothing(node: &Declaration, cx: &Body<'_>) -> bool {
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
