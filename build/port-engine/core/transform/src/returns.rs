//! What a body PROVES about the value a signature hands back.
//!
//! One question so far, and it is the one a reviewer named as the single most visible defect in the
//! emitted crate: `pub fn new(label: &str) -> Option<Box<Tally>>` for a constructor that cannot
//! fail. Two things are wrong with that signature and both come from the same place — the pack maps
//! the source's `*T` to the nil-representable owned form, which is right wherever a pointer may be
//! absent and wrong wherever it may not.
//!
//! The pointer type earns its `Option` from nil and its `Box` from ownership. A function whose
//! every return is the address of a value it JUST CREATED can produce neither: nothing can be
//! absent, and nothing else can hold an alias. So the caller gets ownership of a value, which is
//! exactly what the source hands them.
//!
//! The proof is the same one a failing return uses — the address of a fresh composite is never the
//! absent value — reused rather than restated, so a change to what counts as fresh changes both.

use std::collections::BTreeSet;

use port_engine_api::Declaration;

use crate::vocabulary::{
    ATTR_CALLEE, ATTR_OP, CHILD_BODY, KIND_CALL, KIND_COMPOSITE, KIND_RETURN, KIND_UNARY,
    ATTR_VALUE, KIND_IDENT, KIND_LITERAL, OPERATOR_ADDRESS_OF, SOURCE_INT, TYPE_POINTER,
};

/// Whether this declaration is a THREE-WAY COMPARISON, and so returns the target's ordering.
///
/// Every return is one of the three literals the source uses to spell a comparison, and that is the
/// whole range — a function that can return anything else is not this shape. A returned VARIABLE
/// could hold anything, and proving otherwise is a range analysis this does not have, so it does
/// not qualify.
pub(crate) fn is_three_way_comparison(
    declaration: &Declaration,
    resolver: &crate::resolve::Resolver<'_>,
) -> bool {
    if resolver.idiom_method(crate::vocabulary::IDIOM_ORDERING).is_none() {
        return false;
    }
    let results = declaration.children_of_kind(crate::vocabulary::CHILD_RESULT);
    let [result] = results.as_slice() else {
        return false;
    };
    if result.type_ref.name != SOURCE_INT {
        return false;
    }
    let Some(body) = declaration.children_of_kind(CHILD_BODY).first().copied() else {
        return false;
    };
    let mut returns = Vec::new();
    collect_returns(body, &mut returns);
    !returns.is_empty()
        && returns
            .iter()
            .all(|node| matches!(node.children.as_slice(), [only] if is_ordering_literal(only)))
}

/// Whether this operand is one of the three literals a source comparison returns.
///
/// `-1` reaches the model as a UNARY minus over `1`, which is what the source wrote; the other two
/// are plain literals.
fn is_ordering_literal(operand: &Declaration) -> bool {
    match operand.kind.as_str() {
        KIND_LITERAL => matches!(operand.attr(ATTR_VALUE), Some("0" | "1")),
        KIND_UNARY => {
            operand.attr(ATTR_OP) == Some("-")
                && operand
                    .children
                    .first()
                    .is_some_and(|inner| inner.attr(ATTR_VALUE) == Some("1"))
        }
        _ => false,
    }
}
/// Whether this declaration is a GETTER whose result borrows from the receiver.
///
/// The source's string is immutable and shares its backing, so `func (c Counter) Label() string`
/// hands the caller a view of the field and copies nothing. An owned `String` result CLONES on
/// every call, which is work the source never does — and a reviewer reading the emitted crate named
/// five separate accessors doing it.
///
/// The borrowed form is the same one a string PARAMETER takes, one position further on, and for the
/// same reason: the value is shared read-only data and the target's `&str` is exactly that.
///
/// Requires ALL of:
///
/// - exactly one result, of the source's string type;
/// - a body whose every return is a field read of the RECEIVER. One return that is anything else —
///   a literal, a computed value, a call — and the result is not a view of the receiver at all.
///
/// The receiver is not checked separately, because the return shape is what proves it: the front
/// end marks the one identifier that IS the receiver, so a free function reading a local's field
/// fails this test on the identifier rather than on a signature attribute.
///
/// Safe against a lifetime it cannot supply because the emitted receiver is always a borrow: a
/// pointer receiver that ESCAPES declares no receiver form and refuses, and a value receiver
/// becomes `&self`. An owned `self` would make this reference dangle, and the engine emits none.
///
/// The signature and the body must agree, so both read this rather than each deciding.
pub(crate) fn borrows_from_receiver(declaration: &Declaration) -> bool {
    let results = declaration.children_of_kind(crate::vocabulary::CHILD_RESULT);
    let [result] = results.as_slice() else {
        return false;
    };
    if result.type_ref.name != crate::vocabulary::SOURCE_STRING {
        return false;
    }
    let Some(body) = declaration.children_of_kind(CHILD_BODY).first().copied() else {
        return false;
    };
    let mut returns = Vec::new();
    collect_returns(body, &mut returns);
    !returns.is_empty()
        && returns.iter().all(|node| {
            matches!(node.children.as_slice(), [only] if is_receiver_field(only))
        })
}

/// Whether this operand reads a field of the enclosing method's receiver.
///
/// The receiver is the one identifier whose target spelling is not its name, and the front end marks
/// it — so this asks the model rather than comparing text.
fn is_receiver_field(operand: &Declaration) -> bool {
    operand.kind == crate::vocabulary::KIND_SELECTOR
        && operand
            .children
            .first()
            .is_some_and(crate::body_ops::is_receiver)
}

/// Whether this declaration's result IS a length, and so is a `usize`.
///
/// The source's `len` yields its own `int`, which the pack maps to `i64` — right for a value the
/// source typed `int`, and wrong for a LENGTH, which the target types `usize`. A function that
/// returns nothing but a length is returning a length, and the conversion the mapping adds exists
/// only to make the value type as the source's `int`: where the value never is one, the conversion
/// is what is wrong. `pub fn length(s: &str) -> i64 { Ok(s.len() as i64) }` becomes `-> usize`,
/// and the cast at the return goes with it.
///
/// Equivalent because a length is the same set of values in both: the source's `len` cannot be
/// negative and cannot exceed what the target's `usize` holds, so no value the function can produce
/// changes. A CALLER that wanted a signed value is a call site that now has to say so, which is a
/// refusal where the assumption was, not a silent narrowing.
///
/// Requires exactly one result of the source's integer type and a body whose EVERY return is a
/// length. One return that is a computed value and the result is not a length at all.
pub(crate) fn yields_a_length(declaration: &Declaration, lengths: &BTreeSet<String>) -> bool {
    let results = declaration.children_of_kind(crate::vocabulary::CHILD_RESULT);
    let [result] = results.as_slice() else {
        return false;
    };
    if result.type_ref.name != SOURCE_INT {
        return false;
    }
    let Some(body) = declaration.children_of_kind(CHILD_BODY).first().copied() else {
        return false;
    };
    let mut returns = Vec::new();
    collect_returns(body, &mut returns);
    !returns.is_empty()
        && returns
            .iter()
            .all(|node| matches!(node.children.as_slice(), [only] if is_length(only, lengths)))
}

/// Whether this operand is a call to a callee the pack declares yields a LENGTH.
///
/// By the pack's table rather than by the name `len`, so a pack for another source language names
/// its own and this code names none.
fn is_length(operand: &Declaration, lengths: &BTreeSet<String>) -> bool {
    operand.kind == KIND_CALL
        && operand
            .attr(ATTR_CALLEE)
            .is_some_and(|callee| lengths.contains(callee))
}

/// Every result position whose pointer this declaration's body proves is never absent.
///
/// The signature and the body must AGREE — one renders `T` and the other must produce a `T` rather
/// than the pointer's owned form — so both ask this one function rather than each deciding.
pub(crate) fn bare_pointer_results(declaration: &Declaration) -> BTreeSet<usize> {
    declaration
        .children_of_kind(crate::vocabulary::CHILD_RESULT)
        .iter()
        .enumerate()
        .filter(|(_, result)| never_absent_pointer(declaration, result))
        .map(|(index, _)| index)
        .collect()
}

/// What a signature decided about a declaration's results, for the body to honour.
///
/// Gathered once and carried, because the two must AGREE: a signature that renders `T` needs a body
/// that produces a `T`, and each deriving the answer separately is a disagreement waiting for a
/// corpus that exercises it. Every result idiom adds a field here rather than a parameter.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ResultFacts {
    /// Result positions whose `*T` the body proves is never absent, so they render as `T`.
    pub(crate) bare_pointers: BTreeSet<usize>,
    /// Whether the single result BORROWS from the receiver, so a returned field read is a view.
    pub(crate) borrows_receiver: bool,
    /// Whether the single result is a stored FAILURE the receiver lends rather than gives.
    pub(crate) borrows_failure: bool,
    /// Whether the single result IS a length, so a returned length keeps its `usize`.
    pub(crate) is_a_length: bool,
    /// Whether the single result is an ORDERING, so a returned -1/0/1 becomes one.
    pub(crate) is_an_ordering: bool,
}

impl ResultFacts {
    /// Everything the signature proved, or nothing where a trait fixed the shape.
    pub(crate) fn of(
        declaration: &Declaration,
        resolver: &crate::resolve::Resolver<'_>,
        shape: crate::body::ResultShape,
    ) -> Self {
        let own = shape == crate::body::ResultShape::Own;
        Self {
            bare_pointers: bare_pointer_results(declaration),
            borrows_receiver: own && borrows_from_receiver(declaration),
            borrows_failure: own && borrows_failure_from_receiver(declaration, resolver),
            is_a_length: own && yields_a_length(declaration, resolver.length_functions),
            is_an_ordering: own && is_three_way_comparison(declaration, resolver),
        }
    }

    /// No result idiom at all, for a body built outside a signature.
    pub(crate) fn none() -> Self {
        Self::default()
    }
}

/// Whether this result is a pointer the declaration's body proves is never absent.
///
/// Requires ALL of:
///
/// - the result is a pointer with a pointee — otherwise there is nothing to unwrap to;
/// - the declaration has a BODY. Without one there is nothing to prove and the nil-representable
///   form is the honest answer, because a caller of a signature-only declaration has no way to know
///   what its returns look like;
/// - the body has at least one return. A body that falls off the end returns the zero value, which
///   for a pointer IS the absent one;
/// - every return's operand in this result's position is the address of a fresh composite.
///
/// Deliberately not proven by "the pointee is not recursive" or "no other function stores it":
/// those are properties of the whole program, and this reads one declaration.
pub(crate) fn never_absent_pointer(declaration: &Declaration, result: &Declaration) -> bool {
    if result.type_ref.kind != TYPE_POINTER || result.type_ref.args.is_empty() {
        return false;
    }
    let Some(position) = position_of(declaration, result) else {
        return false;
    };
    let Some(body) = declaration.children_of_kind(CHILD_BODY).first().copied() else {
        return false;
    };
    let mut returns = Vec::new();
    collect_returns(body, &mut returns);
    !returns.is_empty()
        && returns.iter().all(|node| {
            node.children
                .get(position)
                .is_some_and(is_fresh_address)
        })
}

/// Which result this is, by position among the declaration's results.
///
/// By POINTER identity rather than by name or by type: a signature may declare two results of the
/// same type, and answering for the wrong one would unwrap a pointer the body never proved.
fn position_of(declaration: &Declaration, result: &Declaration) -> Option<usize> {
    declaration
        .children_of_kind(crate::vocabulary::CHILD_RESULT)
        .iter()
        .position(|candidate| std::ptr::eq(*candidate, result))
}

/// Every `return` anywhere in this subtree, including inside branches and loops.
///
/// A body's returns are not all at its top level, and a rule that only looked there would call a
/// body proven because the one return it could see was fresh while another was `nil`.
fn collect_returns<'a>(node: &'a Declaration, out: &mut Vec<&'a Declaration>) {
    if node.kind == KIND_RETURN {
        out.push(node);
    }
    for child in &node.children {
        collect_returns(child, out);
    }
}

/// Whether this operand is the address of a value the expression itself creates.
///
/// The same proof a failing return uses, and it needs no table: the expression creates the value,
/// so nothing can have made it absent and nothing else can hold an alias to it.
fn is_fresh_address(operand: &Declaration) -> bool {
    operand.kind == KIND_UNARY
        && operand.attr(ATTR_OP) == Some(OPERATOR_ADDRESS_OF)
        && operand
            .children
            .first()
            .is_some_and(|inner| inner.kind == KIND_COMPOSITE)
}

/// What a SOLE result of the source's failure type actually IS.
///
/// The source spells two different things identically. `func Validate(s string) error` reports
/// whether an operation succeeded and says so by returning the absent value; `func (w *withMessage)
/// Cause() error` hands back an error it is holding, and there is no success to contrast with. Both
/// have one result of the failure type, and reading the second as the first is what made
/// `Cause` translate to `Result<(), E>` — where `Ok(())` would mean "there is no cause" and
/// `Err(e)` would mean "the cause is e", the failure channel carrying data.
///
/// The discriminator is whether any return hands back the ABSENT value, which is exactly the
/// success path of a channel and is what a getter never does. Measured across the corpus: of 45
/// functions with a sole failure result, 32 return absent somewhere and 13 never do, and the split
/// is every `Unmarshal*`, `Scan` and `validate*` on one side against every `Cause`, `Unwrap` and
/// `StackTrace` on the other.
///
/// The same shape as [`never_absent_pointer`], and for the same reason: a source type that admits
/// nothing needs the body consulted before the target can promise a value.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum SoleFailure {
    /// The source's failure channel. Unchanged: this is what `Result` is for.
    Channel,
    /// A failure VALUE the body proves is always present.
    Present,
    /// A failure VALUE that may be absent, because the body hands back something it was given.
    Optional,
}

/// Classify a declaration's sole failure result, or `None` when it does not have one.
///
/// A declaration with NO BODY is a `Channel`, which is the conservative answer and the one that
/// preserves existing behaviour: there is nothing to prove, and a caller of a signature-only
/// declaration cannot know what its returns look like. A body that falls off the end is a
/// `Channel` too — reaching the end returns the zero value, and for the failure type that IS the
/// absent one.
pub(crate) fn sole_failure_role(
    declaration: &Declaration,
    resolver: &crate::resolve::Resolver<'_>,
) -> Option<SoleFailure> {
    let results = declaration.children_of_kind(crate::vocabulary::CHILD_RESULT);
    let [result] = results.as_slice() else {
        return None;
    };
    if !crate::failure::is_failure_type(&result.type_ref, resolver.failure) {
        return None;
    }
    let Some(body) = declaration.children_of_kind(CHILD_BODY).first().copied() else {
        return Some(SoleFailure::Channel);
    };
    let mut returns = Vec::new();
    collect_returns(body, &mut returns);
    if returns.is_empty() {
        return Some(SoleFailure::Channel);
    }
    let operands: Vec<&Declaration> = returns.iter().filter_map(|node| node.children.first()).collect();
    if operands.len() != returns.len() {
        return Some(SoleFailure::Channel);
    }
    if operands
        .iter()
        .any(|operand| crate::failure::is_absent(operand, resolver.failure))
    {
        return Some(SoleFailure::Channel);
    }
    match operands
        .iter()
        .all(|operand| crate::failure_proof::is_certainly_a_failure(operand, resolver))
    {
        true => Some(SoleFailure::Present),
        false => Some(SoleFailure::Optional),
    }
}

/// Whether this declaration is a GETTER handing back a failure the receiver STORES.
///
/// The same shape as [`borrows_from_receiver`] one type over: every return reads a field of the
/// receiver, and the result is the source's failure type rather than its string. Both hand back a
/// view of something the receiver owns, and neither copies it.
///
/// Only for the OPTIONAL role. A sole failure result the body proves is always present is a
/// constructor handing out a value it just made, which the receiver does not own and cannot lend.
pub(crate) fn borrows_failure_from_receiver(
    declaration: &Declaration,
    resolver: &crate::resolve::Resolver<'_>,
) -> bool {
    if sole_failure_role(declaration, resolver) != Some(SoleFailure::Optional) {
        return false;
    }
    let Some(body) = declaration.children_of_kind(CHILD_BODY).first().copied() else {
        return false;
    };
    let mut returns = Vec::new();
    collect_returns(body, &mut returns);
    !returns.is_empty()
        && returns
            .iter()
            .all(|node| matches!(node.children.as_slice(), [only] if is_receiver_field(only)))
}

/// Whether a FALLIBLE signature is one the body proves cannot fail.
///
/// The source's convention puts an error last whether or not the function can produce one, and an
/// interface it satisfies may require the result even when every implementation returns the absent
/// value — `MarshalBinary() ([]byte, error)` is required to have that shape, and eighteen functions
/// across all seven corpus packages never give it anything but nil.
///
/// Carried over literally, the target gets a `Result` with no failure case: every caller writes `?`
/// or an unwrap on something that cannot fail, and the crate's own error type appears in a signature
/// it can never be constructed for. That is not a faithful port of "this cannot fail" — it is the
/// source's interface obligation restated in a language that does not have it.
///
/// Requires a BODY and at least one return, for the reason [`never_absent_pointer`] does: a
/// signature-only declaration proves nothing, and a body that falls off the end returns the zero
/// value, which for the failure type IS the absent one but says nothing about the other results.
pub(crate) fn never_fails(
    declaration: &Declaration,
    resolver: &crate::resolve::Resolver<'_>,
) -> bool {
    if !crate::failure::is_fallible(declaration, resolver.failure) {
        return false;
    }
    // A SOLE failure result is a different question, answered by `sole_failure_role`: there is no
    // other value to hand back, so dropping the result would leave the function returning nothing.
    if declaration.children_of_kind(crate::vocabulary::CHILD_RESULT).len() < 2 {
        return false;
    }
    let Some(body) = declaration.children_of_kind(CHILD_BODY).first().copied() else {
        return false;
    };
    let mut returns = Vec::new();
    collect_returns(body, &mut returns);
    !returns.is_empty()
        && returns.iter().all(|node| {
            node.children
                .last()
                .is_some_and(|operand| crate::failure::is_absent(operand, resolver.failure))
        })
}
