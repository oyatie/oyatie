//! A parameter that is only ever an ACCUMULATOR, which is one expression rather than a sequence.
//!
//! The source is statement-oriented: it takes a parameter, assigns to it a few times, and returns
//! it. `acc = acc.wrapping_add(..); acc = rol31(acc); acc = acc.wrapping_mul(..); return acc` is
//! four statements holding one computation, and the target spells that computation as itself.
//!
//! Two reviewers reading real ported packages named the leftover shape — a `mut` parameter reassigned
//! statement by statement — as a transliteration of the source's statement style. They were right:
//! nothing in the target wants the intermediate binding, and the `mut` in the signature exists only
//! to allow the rewriting.
//!
//! Recognised on the SOURCE, before either the signature or the body is built, because both have to
//! agree: the body folds and the signature drops the `mut`, and a disagreement is either a mutable
//! binding nothing writes or a write to an immutable one.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustStmt};

use crate::vocabulary::{ATTR_OP, CHILD_BODY, CHILD_PARAM, KIND_IDENT};

/// The parameter this body only ever accumulates into, if there is exactly one.
///
/// Every condition here is load-bearing:
///
/// - every statement but the last assigns to ONE name, and that name is a parameter — so there is a
///   single chain and nothing else in the body to reorder around;
/// - the assignment is plain `=`, never a read-modify-write, because `acc ^= v` reads the place
///   twice once expanded and the fold would duplicate it;
/// - each assigned value mentions the name EXACTLY once, so substituting forward neither drops an
///   earlier step's effect nor evaluates one twice;
/// - the last statement returns that name and nothing else.
///
/// Anything else is a body doing more than accumulating, and it keeps its statements.
#[must_use]
pub(crate) fn folded_parameter(declaration: &Declaration) -> Option<String> {
    let body = declaration.children_of_kind(CHILD_BODY).first().copied()?;
    let parameters: Vec<&str> = declaration
        .children_of_kind(CHILD_PARAM)
        .iter()
        .map(|param| param.name.as_str())
        .collect();
    let (last, leading) = body.children.split_last()?;
    if leading.is_empty() {
        return None;
    }

    let name = accumulated_name(leading.first()?, &parameters)?;
    if !leading
        .iter()
        .all(|statement| accumulated_name(statement, &parameters).as_deref() == Some(name.as_str()))
    {
        return None;
    }
    returns_only(last, &name).then_some(name)
}

/// The name this statement assigns to, when it is a plain assignment to a parameter that reads it
/// back exactly once.
fn accumulated_name(statement: &Declaration, parameters: &[&str]) -> Option<String> {
    if statement.kind != "assign" {
        return None;
    }
    let [target, value] = statement.children.as_slice() else {
        return None;
    };
    if target.kind != KIND_IDENT || !parameters.contains(&target.name.as_str()) {
        return None;
    }
    // A READ-MODIFY-WRITE reads the place implicitly, and that read is the chain's link: the source
    // writes `acc += x` where it means `acc = acc + x`, and the engine's own expansion produces
    // exactly that. So the implicit read counts as the one mention, and the written value must have
    // none of its own — `acc += acc` would read it twice and is not a chain.
    let implicit = usize::from(statement.attr(ATTR_OP).is_some());
    (implicit + mentions(value, &target.name) == 1).then(|| target.name.clone())
}

/// Whether this statement is a bare return of the accumulated name, and nothing else.
fn returns_only(statement: &Declaration, name: &str) -> bool {
    statement.kind == "return"
        && match statement.children.as_slice() {
            [only] => only.kind == KIND_IDENT && only.name == name,
            _ => false,
        }
}

/// How many times this subtree reads the name.
fn mentions(node: &Declaration, name: &str) -> usize {
    let here = usize::from(node.kind == KIND_IDENT && node.name == name);
    here + node
        .children
        .iter()
        .map(|child| mentions(child, name))
        .sum::<usize>()
}

/// The chain FOLDED into the single expression it computes.
///
/// Substitutes each step forward into the next, which is exactly what the sequence means: every
/// value depends on the one before it and on nothing else that the body changes. The conditions
/// above are what make the substitution sound — one mention per step, so nothing is dropped and
/// nothing is evaluated twice.
///
/// Returns `None` where the translated statements are not the shape the source promised, which
/// cannot happen for a body the recogniser accepted and is checked anyway rather than assumed.
pub(crate) fn fold(statements: Vec<RustStmt>, name: &str) -> Option<Vec<RustStmt>> {
    let (last, leading) = statements.split_last()?;
    let mut accumulated: Option<RustExpr> = None;
    for statement in leading {
        let RustStmt::Assign {
            target: RustExpr::Path(target),
            op: None,
            value,
        } = statement
        else {
            return None;
        };
        if target != name {
            return None;
        }
        accumulated = Some(match accumulated {
            None => value.clone(),
            Some(previous) => substituted(value, name, &previous)?,
        });
    }
    let returns_name = matches!(last, RustStmt::Return(Some(RustExpr::Path(returned))) if returned == name)
        || matches!(last, RustStmt::Tail(RustExpr::Path(returned)) if returned == name);
    if !returns_name {
        return None;
    }
    Some(vec![RustStmt::Tail(accumulated?)])
}

/// One expression with the accumulated name replaced by what it holds.
///
/// The name appears exactly once — the recogniser proved it on the source — so this replaces the
/// first occurrence it finds and reports failure if there was none, rather than replacing blindly.
fn substituted(within: &RustExpr, name: &str, value: &RustExpr) -> Option<RustExpr> {
    if let RustExpr::Path(path) = within
        && path == name
    {
        return Some(value.clone());
    }
    let mut replaced = false;
    let mut once = |inner: &RustExpr| -> RustExpr {
        if replaced {
            return inner.clone();
        }
        match substituted(inner, name, value) {
            Some(new) => {
                replaced = true;
                new
            }
            None => inner.clone(),
        }
    };
    let rebuilt = match within {
        RustExpr::MethodCall {
            receiver,
            method,
            args,
        } => RustExpr::MethodCall {
            receiver: Box::new(once(receiver)),
            method: method.clone(),
            args: args.iter().map(&mut once).collect(),
        },
        RustExpr::Call { callee, args } => RustExpr::Call {
            callee: callee.clone(),
            args: args.iter().map(&mut once).collect(),
        },
        RustExpr::Binary { op, lhs, rhs } => RustExpr::Binary {
            op: *op,
            lhs: Box::new(once(lhs)),
            rhs: Box::new(once(rhs)),
        },
        RustExpr::Cast { expr, ty } => RustExpr::Cast {
            expr: Box::new(once(expr)),
            ty: ty.clone(),
        },
        RustExpr::Unary { op, operand } => RustExpr::Unary {
            op: *op,
            operand: Box::new(once(operand)),
        },
        // Any other shape is one the recogniser did not promise, and guessing at it would move an
        // expression somewhere its evaluation order is not the source's.
        _ => return None,
    };
    replaced.then_some(rebuilt)
}
