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
    OPERATOR_ADDRESS_OF, SOURCE_INT, TYPE_POINTER,
};

/// Every parameter the signature made the target's index type.
///
/// Read by the BODY, so a read of one is not converted a second time: the signature and the body
/// must agree, and they agree by asking this rather than each deciding.
pub(crate) fn index_parameters(
    declaration: &Declaration,
    resolver: &crate::resolve::Resolver<'_>,
) -> BTreeSet<String> {
    declaration
        .children_of_kind(crate::vocabulary::CHILD_PARAM)
        .into_iter()
        .filter(|param| indexes_only_parameter(declaration, param, resolver))
        .map(|param| crate::naming::to_snake_case(&param.name))
        .collect()
}

/// Whether this parameter is used for NOTHING BUT indexing, and so is a `usize`.
///
/// The same question the loop counter asks, one scope out: a name whose every read is an index
/// operand never has its signed value observed, so it does not need one. The difference is that a
/// parameter's value comes from a CALLER, which is why the call site converts — see `body_call`.
///
/// Requires the source's own integer type, because a parameter of any other type is not a candidate
/// for the target's index type at all; and a BODY, because a signature-only declaration proves
/// nothing about how its parameters are used.
pub(crate) fn indexes_only_parameter(
    declaration: &Declaration,
    param: &Declaration,
    resolver: &crate::resolve::Resolver<'_>,
) -> bool {
    if param.type_ref.name != SOURCE_INT || resolver.idiom_index_counter().is_none() {
        return false;
    }
    let Some(body) = declaration.children_of_kind(CHILD_BODY).first().copied() else {
        return false;
    };
    crate::counters::indexes_only(body, &param.name)
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
    /// Whether the single result IS a length, so a returned length keeps its `usize`.
    pub(crate) is_a_length: bool,
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
            is_a_length: own && yields_a_length(declaration, resolver.length_functions),
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
