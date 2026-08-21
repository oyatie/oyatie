//! A parallel assignment that EXCHANGES two elements of one sequence.
//!
//! `a[i], a[j] = a[j], a[i]` is the shape, and the target's sequence has a method for exactly it.
//! The parallel assignment the engine already emits is faithful — both sides are evaluated before
//! either is written, which is what makes it an exchange rather than two writes — so this changes
//! the spelling and not the program.
//!
//! Recognised from the SOURCE nodes rather than from the rendered places, so the match cannot
//! depend on how they printed: a change to how an index renders would otherwise silently stop the
//! idiom firing, and nothing would say so.

use port_engine_api::Declaration;
use port_engine_rust_ir::{BinaryOp, MatchArm, RustExpr, RustStmt};

use crate::body::Body;
use crate::body_call::render_operand;
use crate::body_expr::expression;
use crate::error::TransformError;
use crate::naming::{to_pascal_case, to_snake_case};
use crate::vocabulary::{CHILD_PLACE, CHILD_VALUE, IDIOM_SWAP, KIND_IDENT, KIND_INDEX};

/// The exchange this parallel assignment is, if it is one.
///
/// Requires exactly two places and two values, every one an index into the SAME sequence by a plain
/// name, and the values naming the two indices in the opposite order. Anything else is a parallel
/// assignment that happens to have two of each and keeps its own form.
///
/// # Errors
/// [`TransformError`] from translating the sequence or either index.
pub(crate) fn exchange(
    node: &Declaration,
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let places = node.children_of_kind(CHILD_PLACE);
    let values = node.children_of_kind(CHILD_VALUE);
    let ([first, second], [third, fourth]) = (places.as_slice(), values.as_slice()) else {
        return Ok(None);
    };
    let (Some(a), Some(b), Some(c), Some(d)) = (
        indexed(first),
        indexed(second),
        indexed(third),
        indexed(fourth),
    ) else {
        return Ok(None);
    };
    // One sequence, and the values are its two indices the other way round. Comparing the index
    // NODES by name is what makes this an exchange rather than two coincidentally shaped writes.
    if a.0 != b.0 || a.0 != c.0 || a.0 != d.0 || a.1 != d.1 || b.1 != c.1 || a.1 == b.1 {
        return Ok(None);
    }
    let method = cx
        .resolver
        .idiom_method(IDIOM_SWAP)
        .unwrap_or("swap")
        .to_owned();
    Ok(Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Path(to_snake_case(a.0))),
        method,
        args: vec![
            place_index(first, cx)?,
            place_index(second, cx)?,
        ],
    }))
}

/// The `(sequence, index)` names this place indexes, when both are plain names.
///
/// A base or an index that is anything else — a field, a call, an expression — is not a name this
/// can compare, and comparing rendered text instead would make the match depend on spelling.
fn indexed(place: &Declaration) -> Option<(&str, &str)> {
    let node = index_node(place);
    if node.kind != KIND_INDEX {
        return None;
    }
    let [base, operand] = node.children.as_slice() else {
        return None;
    };
    match base.kind == KIND_IDENT && operand.kind == KIND_IDENT {
        true => Some((base.name.as_str(), operand.name.as_str())),
        false => None,
    }
}

/// The expression inside a `place` or `value` wrapper.
fn index_node(wrapper: &Declaration) -> &Declaration {
    wrapper.children.first().unwrap_or(wrapper)
}

/// The index operand of a place, translated as the target's own index wants it.
///
/// # Errors
/// [`TransformError`] from translating the operand.
fn place_index(place: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let node = index_node(place);
    match node.children.get(1) {
        Some(operand) => crate::body_index::index_operand(operand, cx),
        None => expression(node, cx),
    }
}

/// A match that only answers YES or NO, as the membership test it is.
///
/// `match x { "x" | "*" | "X" => true, _ => false }` is a test of whether `x` is one of three
/// things, and the target has a macro that is exactly that. The source has to spell it as a switch
/// because its switch is the only multi-pattern form it has.
///
/// Recognised from the ARMS AS BUILT rather than from the source shape, so it cannot fire on a match
/// that merely looks like one — what matters is what the arms yield, and that is known only after
/// they are translated.
///
/// `None` unless the default arm yields `false` and every other yields `true`. The reverse is a
/// NEGATED membership test and needs its own form; inverting it silently would be a different
/// expression wearing this one's shape.
pub(crate) fn membership(arms: &[MatchArm], scrutinee: &RustExpr) -> Option<RustExpr> {
    let (default, tested) = arms.split_last()?;
    if !default.patterns.is_empty() || !yields(&default.body, "false") {
        return None;
    }
    let mut patterns = Vec::new();
    for arm in tested {
        if arm.patterns.is_empty() || !yields(&arm.body, "true") {
            return None;
        }
        patterns.extend(arm.patterns.iter().map(render_operand).collect::<Option<Vec<_>>>()?);
    }
    if patterns.is_empty() {
        return None;
    }
    Some(RustExpr::Literal(format!(
        "matches!({}, {})",
        render_operand(scrutinee)?,
        patterns.join(" | ")
    )))
}

/// Whether this arm body is exactly the given boolean literal.
fn yields(body: &[RustStmt], literal: &str) -> bool {
    matches!(body, [RustStmt::Tail(RustExpr::Path(value) | RustExpr::Literal(value))]
        if value == literal)
}

/// `err == ErrSize` — is this failure that SENTINEL?
///
/// The source compares identity, because its sentinel is a pointer and every `return ErrSize` hands
/// back the same one. The target asks the trait object what concrete type it holds, which is true
/// in exactly the same cases.
///
/// Available only because the sentinel became a TYPE. While it was its message there was nothing to
/// compare — a fresh failure built from a shared string is equal to nothing — and this refused by
/// name, with the loss recorded as the cost of that decision.
///
/// `None` unless one side names a sentinel of this unit and the operator is an equality. A `!=` is
/// the same test negated, which is what the source means by it.
///
/// # Errors
/// [`TransformError`] from translating the other operand.
pub(crate) fn identity_test(
    spelling: &str,
    lhs: &Declaration,
    rhs: &Declaration,
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let negated = match spelling {
        "==" => false,
        "!=" => true,
        _ => return Ok(None),
    };
    let Some(convention) = cx.resolver.failure else {
        return Ok(None);
    };
    if convention.identity_test.is_empty() {
        return Ok(None);
    }
    // Through the RESOLVER, which is where the sentinel's type name is decided once. Casing it
    // here instead is how the declaration came out `Gone` and the test asking about it came out
    // `ErrGone` — the exact disagreement that decision exists to prevent, reintroduced by the one
    // site that did not ask.
    let sentinel_of = |node: &Declaration| {
        (node.kind == KIND_IDENT && cx.resolver.scope.sentinels.contains_key(&node.name))
            .then(|| cx.resolver.sentinel_path(&node.name))
    };
    let (sentinel, subject) = match (sentinel_of(lhs), sentinel_of(rhs)) {
        // BOTH sides a sentinel is a comparison of two known types, which the source can write and
        // which this form does not answer: it asks what a failure holds, and neither side is one.
        (Some(_), Some(_)) | (None, None) => return Ok(None),
        (Some(name), None) => (name, rhs),
        (None, Some(name)) => (name, lhs),
    };
    let Some(rendered) = render_operand(&expression(subject, cx)?) else {
        return Ok(None);
    };
    // GROUPED, and the question splits in two: the failure has to BE the shared type, and then be
    // that variant of it. Ungrouped, the type alone answers both.
    let test = match cx.resolver.sentinel_enum_name() {
        Some(group) if !convention.identity_test_grouped.is_empty() => convention
            .identity_test_grouped
            .replace("{0}", &rendered)
            .replace("{1}", group)
            .replace("{2}", &sentinel),
        _ => convention
            .identity_test
            .replace("{0}", &rendered)
            .replace("{1}", &sentinel),
    };
    Ok(Some(RustExpr::Literal(match negated {
        true => format!("!{test}"),
        false => test,
    })))
}

/// `a >= LOW && a <= HIGH` as the target's RANGE test.
///
/// Recognised from the built operands rather than from the source shape, so it cannot fire on a
/// conjunction that merely looks like one: both sides must test the same subject, the low bound
/// must be `>=` and the high `<=`, and both bounds must be literals.
///
/// The subject must be a value that READING TWICE IS THE SAME AS READING ONCE. The source evaluates
/// it on both sides of the `&&`; the range evaluates it once. For a name or a field that is the
/// same program, and for a call or an index it is not — so anything else is left exactly alone.
pub(crate) fn bounded_range(rendered: &RustExpr) -> Option<RustExpr> {
    // OUTSIDE the range is the same test negated, and the source spells it with `||` because it has
    // no range to be outside of. Recognised here rather than in a second function so the two forms
    // cannot disagree about what counts as a subject or a bound.
    if let RustExpr::Binary {
        op: BinaryOp::Or,
        lhs,
        rhs,
    } = rendered
        && let Some((subject, low)) = bound(lhs, BinaryOp::Lt)
        && let Some((again, high)) = bound(rhs, BinaryOp::Gt)
        && subject == again
        && reads_once(subject)
        // NOT FOR A PARTIAL ORDER, and this is a correctness condition rather than a preference.
        //
        // `f < A || f > B` and `!(A..=B).contains(&f)` are the same test for every value that
        // compares — and OPPOSITE for one that does not. Every comparison against NaN is false, so
        // the source's `||` yields false and the range's `contains` also yields false, which the
        // `!` then turns into true. `gjson.safe_int(NaN)` returns the value in the source and
        // `(0, false)` in the target: a different program, on an input a JSON parser certainly
        // receives, produced by a rewrite that was applied for tidiness.
        //
        // The POSITIVE form has no such hazard: `x >= A && x <= B` and `contains` are both false
        // for NaN, and agree everywhere else. Only the negation inverts the disagreement into an
        // answer.
        && !is_float_bound(low)
        && !is_float_bound(high)
    {
        return Some(RustExpr::Unary {
            op: port_engine_rust_ir::UnaryOp::Not,
            operand: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Range {
                    start: Some(Box::new(low.clone())),
                    end: Box::new(high.clone()),
                    inclusive: true,
                }),
                method: "contains".to_owned(),
                args: vec![RustExpr::Reference {
                    mutable: false,
                    inner: Box::new(subject.clone()),
                }],
            }),
        });
    }
    let RustExpr::Binary {
        op: BinaryOp::And,
        lhs,
        rhs,
    } = rendered
    else {
        return None;
    };
    let (subject, low) = bound(lhs, BinaryOp::Ge)?;
    let (again, high) = bound(rhs, BinaryOp::Le)?;
    if subject != again || !reads_once(subject) {
        return None;
    }
    // A NAMED CHARACTER CLASS where the bounds are one. The target has a predicate for each of
    // these and every reader knows what it means; the range says the same thing and makes the
    // reader check the endpoints. Recognised from the rendered BOUNDS, so a class is claimed only
    // when the two bytes actually delimit it.
    if let Some(class) = ascii_class(low, high) {
        return Some(RustExpr::MethodCall {
            receiver: Box::new(subject.clone()),
            method: class.to_owned(),
            args: Vec::new(),
        });
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Range {
            start: Some(Box::new(low.clone())),
            end: Box::new(high.clone()),
            // INCLUSIVE. `<=` includes its bound and `..` does not, so the exclusive range would
            // reject the highest value the source accepts — off by one, on the boundary, which is
            // where a character classification is most often wrong.
            inclusive: true,
        }),
        method: "contains".to_owned(),
        args: vec![RustExpr::Reference {
            mutable: false,
            inner: Box::new(subject.clone()),
        }],
    })
}

/// One side of the conjunction, as the subject it tests and the bound it tests against.
fn bound(side: &RustExpr, wanted: BinaryOp) -> Option<(&RustExpr, &RustExpr)> {
    let RustExpr::Binary { op, lhs, rhs } = side else {
        return None;
    };
    if *op != wanted {
        return None;
    }
    match is_constant_bound(rhs) {
        true => Some((lhs.as_ref(), rhs.as_ref())),
        false => None,
    }
}

/// Whether this operand is a CONSTANT bound — a literal, or a negated one.
///
/// A negative literal is a unary negation of a literal in the IR, not a literal, and requiring a
/// bare literal here silently declined every range with a negative lower bound. `gjson` has exactly
/// one and it is the widest test in the file.
fn is_constant_bound(expr: &RustExpr) -> bool {
    match expr {
        RustExpr::Literal(_) => true,
        RustExpr::Unary {
            op: port_engine_rust_ir::UnaryOp::Neg,
            operand,
        } => matches!(operand.as_ref(), RustExpr::Literal(_)),
        _ => false,
    }
}

/// Whether evaluating this expression twice is the same as evaluating it once.
fn reads_once(expr: &RustExpr) -> bool {
    match expr {
        RustExpr::Path(_) | RustExpr::SelfValue | RustExpr::Literal(_) => true,
        RustExpr::Field { base, .. } | RustExpr::TupleIndex { base, .. } => reads_once(base),
        _ => false,
    }
}

/// The target's predicate for a byte range that IS a standard character class.
///
/// Closed, and matched on the exact byte literals that delimit each class. A range that merely
/// overlaps one is not it — `b'0'..=b'8'` is not `is_ascii_digit`, and answering as though it were
/// would accept one fewer character than the source does.
fn ascii_class(low: &RustExpr, high: &RustExpr) -> Option<&'static str> {
    let (RustExpr::Literal(low), RustExpr::Literal(high)) = (low, high) else {
        return None;
    };
    match (low.as_str(), high.as_str()) {
        ("b'0'", "b'9'") => Some("is_ascii_digit"),
        ("b'a'", "b'z'") => Some("is_ascii_lowercase"),
        ("b'A'", "b'Z'") => Some("is_ascii_uppercase"),
        _ => None,
    }
}

/// Whether this bound is a FLOATING-POINT literal, and so orders only partially.
///
/// Read from the spelling, which is where the engine puts the distinction: a whole number resolved
/// to a float is spelled with a point precisely so the target reads it as one. An integer bound
/// carries no point and orders totally, which is what the negated range rewrite requires.
fn is_float_bound(expr: &RustExpr) -> bool {
    let literal = match expr {
        RustExpr::Literal(text) => text,
        RustExpr::Unary {
            op: port_engine_rust_ir::UnaryOp::Neg,
            operand,
        } => match operand.as_ref() {
            RustExpr::Literal(text) => text,
            _ => return false,
        },
        _ => return false,
    };
    literal.contains('.')
}

/// Whether this statement contains a two-sided FLOAT bounds test the range rewrite had to decline.
///
/// Mirrors what [`bounded_range`] refuses, so the attribute that silences the lint appears exactly
/// where the rewrite did not happen and nowhere else.
pub(crate) fn compares_float_bounds(statement: &RustStmt) -> bool {
    fn in_expr(expr: &RustExpr) -> bool {
        match expr {
            RustExpr::Binary {
                op: BinaryOp::Or,
                lhs,
                rhs,
            } => {
                let declined = bound(lhs, BinaryOp::Lt)
                    .zip(bound(rhs, BinaryOp::Gt))
                    .is_some_and(|((left, low), (right, high))| {
                        left == right && (is_float_bound(low) || is_float_bound(high))
                    });
                declined || in_expr(lhs) || in_expr(rhs)
            }
            RustExpr::Binary { lhs, rhs, .. } => in_expr(lhs) || in_expr(rhs),
            RustExpr::Unary { operand, .. } => in_expr(operand),
            RustExpr::If {
                cond,
                then,
                otherwise,
            } => {
                in_expr(cond)
                    || then.iter().any(compares_float_bounds)
                    || otherwise.as_deref().is_some_and(in_expr)
            }
            RustExpr::Block(body) => body.iter().any(compares_float_bounds),
            RustExpr::Match { arms, .. } => {
                arms.iter().any(|arm| arm.body.iter().any(compares_float_bounds))
            }
            _ => false,
        }
    }
    match statement {
        RustStmt::Semi(expr) | RustStmt::Tail(expr) | RustStmt::Discard(expr) => in_expr(expr),
        RustStmt::Return(Some(expr)) => in_expr(expr),
        RustStmt::Let { value, .. } => value.as_ref().is_some_and(in_expr),
        RustStmt::While { cond, body, .. } => in_expr(cond) || body.iter().any(compares_float_bounds),
        RustStmt::Loop(body) | RustStmt::Block(body) | RustStmt::ForIn { body, .. } => {
            body.iter().any(compares_float_bounds)
        }
        _ => false,
    }
}
