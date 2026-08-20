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

use std::collections::BTreeSet;

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
pub(crate) fn folded_parameters(declaration: &Declaration) -> Option<(String, BTreeSet<String>)> {
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

    // The name the chain ENDS in is the one returned, and it is the one that folds. Statements
    // before it may assign OTHER parameters, and those fold into it — `val = round(0, val)` feeds
    // the `acc` chain below it, which is a chain with a longer neck rather than a different shape.
    let name = returned_name(last)?;
    if !parameters.contains(&name.as_str()) {
        return None;
    }
    // Every statement is a link. What makes the sequence foldable is a property of each ASSIGNMENT
    // rather than of each name: the value a statement produces must be read exactly ONCE before that
    // name is assigned again, or by the return if it never is. Read twice and the fold would
    // evaluate it twice; read never and the fold would drop what it did.
    //
    // `acc = f(acc); acc = g(acc); return acc` satisfies this three times over on one name, and
    // `val = f(val); acc ^= val; acc = g(acc); return acc` satisfies it on two — which is the same
    // chain with a longer neck, not a different shape.
    let mut assigned: Vec<String> = Vec::with_capacity(leading.len());
    for statement in leading {
        assigned.push(accumulated_name(statement, &parameters)?);
    }
    for (index, target) in assigned.iter().enumerate() {
        let next_write = assigned[index + 1..]
            .iter()
            .position(|other| other == target)
            .map_or(leading.len(), |at| index + 1 + at);
        let reads: usize = leading[index + 1..=next_write.min(leading.len() - 1)]
            .iter()
            .take(next_write - index)
            .map(|statement| mentions_read(statement, target))
            .sum::<usize>()
            + usize::from(next_write == leading.len() && target == &name);
        if reads != 1 {
            return None;
        }
    }
    // EVERY name the chain consumes, not only the one returned. The fold substitutes them all
    // away, so a `mut` left on any of them is a mutability nothing uses — which the target warns
    // about and the compile proof denies.
    Some((name, assigned.into_iter().collect()))
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

/// The name this statement returns, when it returns a bare name and nothing else.
fn returned_name(statement: &Declaration) -> Option<String> {
    if statement.kind != "return" {
        return None;
    }
    match statement.children.as_slice() {
        [only] if only.kind == KIND_IDENT => Some(only.name.clone()),
        _ => None,
    }
}

/// How many times this statement READS the name, not counting the target it assigns to.
fn mentions_read(statement: &Declaration, name: &str) -> usize {
    let implicit = usize::from(
        statement.kind == "assign"
            && statement.attr(ATTR_OP).is_some()
            && statement.children.first().is_some_and(|target| {
                target.kind == KIND_IDENT && target.name == name
            }),
    );
    let written = match statement.kind == "assign" {
        true => statement.children.iter().skip(1).map(|value| mentions(value, name)).sum(),
        false => mentions(statement, name),
    };
    implicit + written
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
