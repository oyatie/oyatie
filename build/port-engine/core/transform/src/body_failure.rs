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
        1 => values
            .into_iter()
            .next()
            .unwrap_or_else(|| unreachable!("the arm matched exactly one value")),
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
    // `(T, bool)` ANSWERED AS AN OPTION. The signature already decided this; the flag is carried
    // rather than re-derived so the two cannot disagree about whether a return is a pair.
    //
    // `return v, true` is `Some(v)` and `return <literal>, false` is `None` — and the literal is
    // DROPPED, which is the whole point: it was never an answer, only what the source had to put
    // in a slot that could not be empty.
    if cx.returns_option {
        let built = option_return(node, cx)?;
        return Ok(match is_last {
            true => RustStmt::Tail(built),
            false => RustStmt::Return(Some(built)),
        });
    }
    let value = if cx.fallible {
        Some(fallible_return(node, cx)?)
    } else {
        // The trailing ABSENT failure comes off, because the signature no longer has that result.
        // The operand is the source stating it did not fail, and the target states that by the
        // absence of a `Result` — restating it as a returned value would be a second result nobody
        // declared.
        let operands = match cx.drops_absent_failure {
            true => &node.children[..node.children.len().saturating_sub(1)],
            false => &node.children[..],
        };
        // A BARE return in a function with NAMED results returns those results. The source binds
        // them at entry and `return` hands back whatever they hold — so an empty return is not a
        // return of nothing, and emitting one produced `return;` from a function with a result type.
        if operands.is_empty()
            && let Some(named) = crate::body_wider::named_results(cx)
        {
            return Ok(match is_last {
                true => RustStmt::Tail(named),
                false => RustStmt::Return(Some(named)),
            });
        }
        let values = operands
            .iter()
            .enumerate()
            .map(|(index, child)| returned_operand(index, child, cx))
            .collect::<Result<Vec<_>, _>>()?;
        match values.len() {
            0 => None,
            1 => values.into_iter().next().map(|expr| crate::body_ops::own_returned_sequence(own_returned_string(expr, cx), 0, cx)),
            // Several results leave as a tuple, matching how the signature renders them.
            // Each ELEMENT owns on its own terms: a tuple of results is several results, and only
            // some of them are sequences the target owns.
            _ => Some(RustExpr::Tuple(
                values
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| crate::body_ops::own_returned_sequence(value, index, cx))
                    .collect(),
            )),
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

    // FORWARDING. `return f(x)` where `f` fails the same way this function does is not a failing
    // return at all — the source hands its caller whatever `f` answered, success or failure, and
    // the target's `Result` is that same answer. There is nothing to prove about the operand
    // because nothing is being wrapped: `Err(..)` never appears.
    //
    // This is why 16 declarations across seven packages asked to prove a `call` was "certainly a
    // failure" and could not. They were never claiming one. `func (i *KSUID) Set(s string) error {
    // return i.UnmarshalText([]byte(s)) }` is the shape, and it is common — a method that exists to
    // delegate.
    if values.is_empty()
        && let Some(forwarded) = forwards_the_same_failure(failure, cx)
    {
        return expression(forwarded, cx);
    }

    if crate::failure::is_absent(failure, cx.resolver.failure) {
        let values = values
            .iter()
            .enumerate()
            .map(|(index, child)| returned_operand(index, child, cx))
            .collect::<Result<Vec<_>, _>>()?;
        let ok = match values.len() {
            0 => RustExpr::Tuple(Vec::new()),
            // OWNED here too. A successful return through the failure channel is still a return,
            // and the result type it has to fit is the one inside `Ok`.
            1 => crate::body_ops::own_returned_sequence(
                values
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| unreachable!("the arm matched exactly one value")),
                0,
                cx,
            ),
            _ => RustExpr::Tuple(values),
        };
        return Ok(RustExpr::Call {
            callee: Box::new(RustExpr::Path("Ok".to_owned())),
            args: vec![ok],
        });
    }

    if !is_certainly_a_failure(failure, cx.resolver) {
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
    // A SENTINEL is already the failure's own type, so the destination's conversion is all it
    // needs — there is no construction to rewrite because there was none.
    if let RustExpr::Path(name) = &built
        && cx.resolver.scope.sentinels.keys().any(|source| {
            cx.resolver.sentinel_path(source) == *name
        })
    {
        return RustExpr::Literal(
            convention.inferred_construction.replace("{0}", name),
        );
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
    cx.resolver.failure?;
    // The sentinel VALUE, which is the unit struct itself. What turns it into the function's
    // failure type is the destination's own conversion, applied by `inferred` beside every other
    // built failure — so a sentinel and a constructed failure reach `Err` the same way.
    Some(RustExpr::Path(cx.resolver.sentinel_path(&operand.name)))
}

/// The call this return FORWARDS, when it forwards one.
///
/// Two things have to hold, and each is read off the model rather than assumed. The operand is a
/// CALL — a name that merely holds a failure is a value being wrapped, not an answer being passed
/// on. And the CALLEE's own results are the failure alone, which the model carries on the call's
/// selector: the front end records a callee's whole signature there, so no table has to be asked.
///
/// The third thing the caller has already proved: this is reached only where the return's operands
/// are the failure and nothing else, so the enclosing function results in exactly what the callee
/// does. Equal shapes are the whole requirement — a callee answering with a value beside the
/// failure would need that value carried, and this returns the call unchanged.
fn forwards_the_same_failure<'a>(
    operand: &'a Declaration,
    cx: &Body<'_>,
) -> Option<&'a Declaration> {
    if operand.kind != crate::vocabulary::KIND_CALL {
        return None;
    }
    // A CONSTRUCTOR is not a forward. `return errors.New("key size must be 16, 24 or 32 bytes")`
    // is a call whose result is the failure type, and it is the source saying FAIL WITH THIS —
    // there is no answer being passed on, because the callee had no chance to succeed. The pack
    // already names which callees those are, and reading its answer keeps the two rules from
    // disagreeing about one call. Without this, `validate_key` returned a boxed error where its
    // signature promised a `Result`.
    if is_certainly_a_failure(operand, cx.resolver) {
        return None;
    }
    let callee = operand.children.first()?;
    if callee.type_ref.kind != "func" {
        return None;
    }
    let produced = callee.type_ref.args.get(1)?;
    let [answered] = produced.args.as_slice() else {
        return None;
    };
    crate::failure::is_failure_type(answered, cx.resolver.failure).then_some(operand)
}

/// A return from a function whose `(T, bool)` results the signature answered as `Option<T>`.
///
/// # Errors
/// [`TransformError::Unsupported`] when the return does not have the shape the signature was
/// promised — which `returns::spells_an_option` checks for the whole declaration before the
/// signature commits, so reaching this is a disagreement between the two rather than a source the
/// engine cannot handle.
fn option_return(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let disagreed = || TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: "this function's results were answered as an option, and this return is not one \
                 of the two shapes that decides — a value with `true`, or a literal with `false`"
            .to_owned(),
    };
    let [value, decided] = node.children.as_slice() else {
        return Err(disagreed());
    };
    match decided.name.as_str() {
        "true" => Ok(RustExpr::Call {
            callee: Box::new(RustExpr::Path("Some".to_owned())),
            args: vec![expression(value, cx)?],
        }),
        "false" => Ok(RustExpr::Path("None".to_owned())),
        _ => Err(disagreed()),
    }
}
