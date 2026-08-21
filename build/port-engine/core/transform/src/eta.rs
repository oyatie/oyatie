//! A private wrapper that exists only because the SOURCE spells a call verbosely.
//!
//! `func rol31(x uint64) uint64 { return bits.RotateLeft64(x, 31) }` is not an abstraction. It is a
//! shorthand, written because the source spells the rotation as a free function with the distance as
//! an argument and eight call sites reading `bits.RotateLeft64(acc, 31)` would be unreadable. The
//! target spells the same rotation as a method on the value, so the shorthand has nothing left to
//! shorten — and eight one-line wrappers around an intrinsic are, in three reviewers' words, the
//! clearest sign the source's structure was carried rather than the source's meaning.
//!
//! Same argument as dropping the source's `Err` prefix from a sentinel: both are conventions
//! answering a problem the target does not have, and carrying one across is carrying the problem.
//!
//! NARROW, because the reasoning only holds where it holds:
//!
//! - UNEXPORTED, so the wrapper is not API and removing it is not a change anyone outside can see;
//! - the body is one call and nothing else, so there is no logic to lose;
//! - that call is FOREIGN and the pack MAPS it, which is what makes it a spelling the target already
//!   has — a wrapper around a local function is somebody's abstraction and stays;
//! - every parameter is read exactly once in it, so substituting the arguments neither drops one nor
//!   evaluates one twice.
//!
//! A wrapper meeting all four is inlined at its call sites and not emitted. Anything else is a
//! function the author wrote for a reason the engine cannot see, and it is emitted as written.

use std::collections::BTreeSet;

use port_engine_api::Declaration;

use crate::vocabulary::{
    ATTR_CALLEE, CHILD_BODY, CHILD_PARAM, FLAG_EXPORTED, KIND_CALL, KIND_IDENT,
};

/// A wrapper whose call sites can spell what it wraps directly.
#[derive(Clone, Debug)]
pub(crate) struct EtaWrapper {
    /// The wrapper's parameter names, in the order the signature declares them.
    pub(crate) params: Vec<String>,
    /// The single call its body is, with the parameters still in it.
    pub(crate) wrapped: Declaration,
}

/// Recognise a wrapper, given the callees the pack maps and the units this model has.
pub(crate) fn wrapper(
    declaration: &Declaration,
    mapped: &BTreeSet<String>,
    units: &BTreeSet<String>,
) -> Option<EtaWrapper> {
    if declaration.kind != "func" || declaration.flags.iter().any(|flag| flag == FLAG_EXPORTED) {
        return None;
    }
    let body = declaration.children_of_kind(CHILD_BODY).first().copied()?;
    let [statement] = body.children.as_slice() else {
        return None;
    };
    if statement.kind != "return" {
        return None;
    }
    let [wrapped] = statement.children.as_slice() else {
        return None;
    };
    if wrapped.kind != KIND_CALL {
        return None;
    }
    // FOREIGN and MAPPED: the target already spells this, which is the whole reason the wrapper has
    // nothing to do. A call into this model is somebody's own function and the wrapper around it is
    // somebody's own abstraction.
    let callee = wrapped.attr(ATTR_CALLEE)?;
    let package = callee.rsplit_once('.').map(|(package, _)| package)?;
    if units.contains(package) || !mapped.contains(callee) {
        return None;
    }

    let params: Vec<String> = declaration
        .children_of_kind(CHILD_PARAM)
        .iter()
        .map(|param| param.name.clone())
        .collect();
    if params.is_empty() || params.iter().any(|name| mentions(wrapped, name) != 1) {
        return None;
    }
    Some(EtaWrapper {
        params,
        wrapped: wrapped.clone(),
    })
}

/// The wrapped call with the caller's ARGUMENTS in place of the wrapper's parameters.
///
/// One substitution per parameter, which the recogniser proved is all there is. An argument count
/// that does not match the parameters is not this wrapper's call and yields nothing.
pub(crate) fn inlined(wrapper: &EtaWrapper, arguments: &[Declaration]) -> Option<Declaration> {
    if arguments.len() != wrapper.params.len() {
        return None;
    }
    let mut built = wrapper.wrapped.clone();
    for (name, argument) in wrapper.params.iter().zip(arguments) {
        built = substituted(&built, name, argument);
    }
    Some(built)
}

/// One tree with every read of the name replaced by the argument.
fn substituted(node: &Declaration, name: &str, argument: &Declaration) -> Declaration {
    if node.kind == KIND_IDENT && node.name == name {
        return argument.clone();
    }
    let mut built = node.clone();
    built.children = node
        .children
        .iter()
        .map(|child| substituted(child, name, argument))
        .collect();
    built
}

/// How many times this subtree reads the name.
fn mentions(node: &Declaration, name: &str) -> usize {
    usize::from(node.kind == KIND_IDENT && node.name == name)
        + node
            .children
            .iter()
            .map(|child| mentions(child, name))
            .sum::<usize>()
}

/// Whether anything in this unit uses the name as a VALUE rather than calling it.
///
/// A wrapper is only droppable if every use is a call, because every call becomes the call it wraps
/// and nothing is left pointing at it. The source can also take a function as a value — `f := rol31`
/// — and that use has nowhere to go once the declaration is gone.
///
/// The callee child of a call is not such a use: it names the function being called, which is
/// exactly the case that inlines.
pub(crate) fn used_as_value(declarations: &[Declaration], name: &str) -> bool {
    declarations
        .iter()
        .any(|declaration| value_use(declaration, name))
}

fn value_use(node: &Declaration, name: &str) -> bool {
    if node.kind == KIND_CALL {
        // Skip the callee, walk the arguments. A call OF the name is the case that inlines; a call
        // that passes the name is a value use like any other.
        return node.children[1..]
            .iter()
            .any(|argument| value_use(argument, name));
    }
    if node.kind == KIND_IDENT && node.name == name {
        return true;
    }
    node.children.iter().any(|child| value_use(child, name))
}
