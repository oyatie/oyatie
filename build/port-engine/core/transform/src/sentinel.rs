//! A package-level failure value the source declares once and returns from many places.
//!
//! `var ErrSize = errors.New("size")` is the commonest error-typed package variable in real code,
//! and it blocked twice over: the declaration is not a constant expression, so it could not be a
//! `static`, and every `return ErrSize` is an operand nothing could prove was a failure.
//!
//! A sentinel becomes its MESSAGE. `static ERR_SIZE: &str = "size"`, and each return builds a
//! failure from it through the same mapping the pack already declares for the constructor — one
//! rule doing the work rather than a second one free to disagree with it. The message is a constant
//! expression, so there is no lazy initialisation and no when-does-the-work-happen question.
//!
//! What it costs is stated in the pack and repeated here because a reader of this file is who needs
//! it: the source's sentinel has IDENTITY. `errors.New` returns a pointer and a source caller
//! writes `err == ErrSize`. The target's boxed trait object has no equality, so that comparison has
//! no translation — and it refuses at the comparison site rather than silently comparing something
//! else. Returning a sentinel ports; comparing against one does not.

use std::collections::BTreeMap;

use port_engine_api::{Declaration, FailureConvention};

use crate::vocabulary::{ATTR_CALLEE, ATTR_VALUE, FLAG_REBOUND, KIND_CALL, KIND_LITERAL, KIND_VAR};

/// Every sentinel the unit declares, by source name, with the message it carries.
///
/// Recognised by SHAPE and by the pack's own table, never by name: a variable named `ErrFoo` that
/// is built some other way is not a sentinel, and one named anything at all that is built by a
/// declared sentinel constructor is.
///
/// Four facts required, and each because dropping it would admit something else:
///
/// - a package-level `var`, because that is what a sentinel is;
/// - NOTHING WRITES IT, because a variable something rebinds is not one value returned from many
///   places, and its form is the decision the pack still declines;
/// - its initialiser is a CALL to a callee the pack names a sentinel constructor, which is what
///   makes the value a failure and its sole argument the message;
/// - that argument is a LITERAL, because a message computed from anything else is not a constant
///   expression, and `fmt.Errorf` is exactly the case that fails here.
pub(crate) fn sentinels(
    declarations: &[Declaration],
    failure: Option<&FailureConvention>,
) -> BTreeMap<String, String> {
    sentinels_with(declarations, failure, &BTreeMap::new())
        .into_iter()
        .map(|(name, (message, _))| (name, message))
        .collect()
}

/// The same, keeping each sentinel's format ARGUMENTS.
///
/// A source sentinel is sometimes built by a FORMATTING constructor over constants —
/// `fmt.Errorf("Valid KSUIDs are %v bytes", byteLength)`. Its message is fixed at compile time, so
/// it is a sentinel as surely as one built from a bare literal; it is simply not spelled as one. Ten
/// of the corpus's seventeen sentinels are built this way, and while they were unrecognised every
/// `return Nil, errStrSize` refused for want of a proof the sentinel would have supplied.
///
/// A NON-CONSTANT argument disqualifies it. A message that depends on a runtime value is not one
/// value the program has — it is a different string per call, which is a formatted error rather than
/// a sentinel, and the difference is exactly what a caller comparing against it relies on.
pub(crate) fn sentinels_with(
    declarations: &[Declaration],
    failure: Option<&FailureConvention>,
    verbs: &BTreeMap<String, String>,
) -> BTreeMap<String, (String, Vec<String>)> {
    let Some(convention) = failure else {
        return BTreeMap::new();
    };
    declarations
        .iter()
        .filter_map(|declaration| {
            let carried = message_of(declaration, convention, verbs)?;
            Some((declaration.name.clone(), carried))
        })
        .collect()
}

/// The message a sentinel declaration carries, if this declaration is one.
fn message_of(
    declaration: &Declaration,
    convention: &FailureConvention,
    verbs: &BTreeMap<String, String>,
) -> Option<(String, Vec<String>)> {
    if declaration.kind != KIND_VAR || declaration.flags.iter().any(|flag| flag == FLAG_REBOUND) {
        return None;
    }
    let call = declaration.children.first()?;
    if call.kind != KIND_CALL
        || !convention
            .sentinel_constructors
            .contains(call.attr(ATTR_CALLEE)?)
    {
        return None;
    }
    // The callee's own selector is a child too, so the message is the first LITERAL rather than
    // the first child. A constructor called with anything but one literal is not a sentinel this
    // rule answers for, and falls through to the ordinary refusal.
    let literals: Vec<&Declaration> = call
        .children
        .iter()
        .filter(|child| child.kind == KIND_LITERAL)
        .collect();
    let [only] = literals.as_slice() else {
        return None;
    };
    let message = only.attr(ATTR_VALUE)?.to_owned();
    // Every remaining operand must be a package CONSTANT, or the message is not fixed.
    let mut arguments = Vec::new();
    for operand in call.children.iter().skip(1) {
        if operand.kind == KIND_LITERAL {
            continue;
        }
        if operand.kind != crate::vocabulary::KIND_IDENT
            || operand.attr(crate::vocabulary::ATTR_REF) != Some(crate::vocabulary::REF_CONST)
        {
            return None;
        }
        arguments.push(operand.name.clone());
    }
    if arguments.is_empty() {
        return Some((message, arguments));
    }
    // The source's VERBS become the target's placeholders, through the same table the formatting
    // calls use — so a sentinel's message and an inline formatted message cannot disagree.
    let mut rendered = message;
    for (verb, placeholder) in verbs {
        rendered = rendered.replace(verb, placeholder);
    }
    Some((rendered, arguments))
}
