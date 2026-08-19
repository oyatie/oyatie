//! What the source's FAILURE CONVENTION becomes in a body.
//!
//! Two translations, and both are about the same disagreement: the source carries failure as an
//! extra value that a caller may drop, and the target carries it as the whole return, which a
//! caller cannot. A return has to pick a constructor, and a bind-and-check pair becomes an operator
//! that cannot be forgotten.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustStmt, TupleBind};

use crate::body::Body;
use crate::body_operand::returned_operand;
use crate::failure_proof::{discards_nothing, is_certainly_a_failure};
use crate::body_expr::expression;
use crate::body_ops::own_returned_string;
use crate::error::TransformError;
use crate::naming::{to_screaming_snake, to_snake_case};
use crate::vocabulary::{
    ATTR_CALLEE, ATTR_OP, ATTR_VALUE, KIND_CALL, KIND_COMPOSITE, KIND_IDENT, KIND_UNARY, OPERATOR_ADDRESS_OF,
};

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

/// The unchecked propagation as the target spells it: the operator, then the success.
///
/// Two statements out for two statements in, because the source's two do two things — run the
/// fallible call, and return the values with whatever it produced. The target's operator carries
/// the failure out and the success carries the values, which is the same split.
///
/// # Errors
/// [`TransformError`] from translating the call or the returned values.
pub(crate) fn propagate_into_success(
    found: &crate::failure::TailPropagation<'_>,
    cx: &Body<'_>,
) -> Result<Vec<RustStmt>, TransformError> {
    let operator = RustStmt::Semi(RustExpr::Try(Box::new(expression(found.source, cx)?)));
    let mut values = Vec::with_capacity(found.values.len());
    for value in found.values {
        values.push(expression(value, cx)?);
    }
    let ok = match values.len() {
        0 => RustExpr::Tuple(Vec::new()),
        1 => values.into_iter().next().unwrap_or(RustExpr::Todo),
        _ => RustExpr::Tuple(values),
    };
    let success = RustStmt::Tail(RustExpr::Call {
        callee: Box::new(RustExpr::Path("Ok".to_owned())),
        args: vec![ok],
    });
    Ok(vec![operator, success])
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
            .enumerate()
            .map(|(index, child)| returned_operand(index, child, cx))
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
            .enumerate()
            .map(|(index, child)| returned_operand(index, child, cx))
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

    if !is_certainly_a_failure(failure, cx) {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "a failing return's operand must be PROVABLY a failure, and this one is a `{}` \
                 whose value the engine cannot show is non-absent. The source's operand may be \
                 absent, in which case its caller sees SUCCESS; the target's `Err(..)` reports \
                 failure unconditionally, so emitting it here would be a different program — and \
                 one that reports failure where the source reported success. The two proofs the \
                 engine has are a call to a declared failure constructor and the address of a \
                 fresh composite; the pack's `failure_convention.constructor_reason` says which \
                 callees those are and what admits another",
                failure.kind
            ),
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
    let built = match sentinel_failure(failure, cx) {
        Some(built) => built,
        None => expression(failure, cx)?,
    };
    let built = inferred(built, cx);
    Ok(RustExpr::Call {
        callee: Box::new(RustExpr::Path("Err".to_owned())),
        args: vec![built],
    })
}

/// A built failure, in the shorter form the DESTINATION allows.
///
/// Inside `Err(..)` of a function whose return type names the failure, the type is already known,
/// so the explicit conversion the general mapping uses says something the signature has said. The
/// pack decides whether there is a shorter form and what it is; where it declares none, or where
/// the built expression is not a plain conversion this can rewrite, the general form stands.
fn inferred(built: RustExpr, cx: &Body<'_>) -> RustExpr {
    let Some(convention) = cx.resolver.failure else {
        return built;
    };
    if convention.inferred_construction.is_empty() {
        return built;
    }
    // Only a rendered CONSTRUCTION is rewritten, and only by matching the general form the pack
    // declares: the operand inside it is what the shorter form takes, and reading it from the form
    // rather than from the text means a pack that changes one changes both.
    let RustExpr::Literal(rendered) = &built else {
        return built;
    };
    let Some(mapping) = convention
        .constructors
        .iter()
        .find_map(|identity| cx.resolver.function_map.get(identity))
    else {
        return built;
    };
    let Some((prefix, suffix)) = mapping.form.split_once("{0}") else {
        return built;
    };
    match rendered
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(suffix))
    {
        Some(operand) => RustExpr::Literal(
            convention
                .inferred_construction
                .replace("{0}", operand),
        ),
        None => built,
    }
}


/// The failure a SENTINEL operand builds, if the operand is one.
///
/// The one place a sentinel may be read: here the engine knows the operand IS the failure, which an
/// identifier standing on its own does not. Built through the mapping the pack declares for the
/// sentinel's own constructor, so a return of the sentinel and a direct call to that constructor
/// spell the same thing and a pack that changes one changes both.
fn sentinel_failure(operand: &Declaration, cx: &Body<'_>) -> Option<RustExpr> {
    if operand.kind != KIND_IDENT {
        return None;
    }
    cx.resolver.scope.sentinels.get(&operand.name)?;
    let convention = cx.resolver.failure?;
    let mapping = convention
        .sentinel_constructors
        .iter()
        .find_map(|identity| cx.resolver.function_map.get(identity))?;
    Some(RustExpr::Literal(
        mapping
            .form
            .replace("{0}", &to_screaming_snake(&operand.name)),
    ))
}
