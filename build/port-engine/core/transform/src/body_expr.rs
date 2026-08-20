//! Expressions.
//!
//! Two decisions live here that a naive translator gets silently wrong, and both are about the
//! difference between what the source's syntax MEANS and what the same syntax means in the target:
//! reading a field is a copy in Go and a move in Rust, and a struct literal zero-fills in Go and
//! must name every field in Rust.

use port_engine_api::{Declaration, TypeRef};
use port_engine_api::PointerConstruction;
use port_engine_rust_ir::RustExpr;

use crate::body::{Body};
use crate::body_place::{
    address_of_fresh, convert, refuse_sentinel_out_of_place, selector,
};
use crate::body_parts::{one_child, two_children, unsupported_source};
use crate::body_index::slice;
use crate::body_argument::constructed;
use crate::body_call::call;
use crate::body_literal::{composite, zero_value};
use crate::body_idiom::emptiness_test;
use crate::body_ops::{binary_operator, compares_lengths, is_receiver, operator_of, own_string_for, reference, refuse_deferred_reference, unary_operator, unary_refusal};
use crate::error::TransformError;
use crate::naming::{to_snake_case, to_screaming_snake};
use crate::vocabulary::{
    ATTR_CALLEE, ATTR_CALLEE_KIND, ATTR_LIT_KIND, ATTR_VALUE, CALLEE_KIND_METHOD, DISPOSITION_OWNED_POINTER, FLAG_REREAD, IDIOM_EMPTY_STRING, KIND_COMPOSITE, KIND_IDENT, KIND_LITERAL, KIND_UNARY, LIT_KIND_STRING, OPERATOR_ADDRESS_OF,
};

/// Where an expression appears: a value is READ, a place is WRITTEN TO.
///
/// The distinction is what keeps the clone rule from producing `self.total.clone() = x`.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Position {
    Value,
    Place,
}

pub(crate) fn expression(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    in_position(node, cx, Position::Value)
}

pub(crate) fn in_position(
    node: &Declaration,
    cx: &Body<'_>,
    position: Position,
) -> Result<RustExpr, TransformError> {
    match node.kind.as_str() {
        // A literal passes through as SOURCE TEXT, which is safe only because the emitted tree is
        // parsed and compiled. Where the two languages' lexical forms diverge — a rune literal, an
        // imaginary literal — the pass-through fails the parse, which is the correct outcome and
        // is why no attempt is made to normalise numbers here.
        // A LITERAL keeps its value and may not keep its spelling: a long decimal one is grouped,
        // because the target groups digits and the source does not. Applied here as well as at a
        // constant's declaration, since the same literal reaches the output both ways — a constant
        // whose value is an expression carries its numbers through this path and was left ungrouped
        // beside neighbours that were not.
        "literal" => {
            let Some(value) = node.attr(ATTR_VALUE) else {
                return Err(TransformError::MissingDatum {
                    construction: "literal".to_owned(),
                    name: cx.owner.to_owned(),
                    datum: ATTR_VALUE,
                });
            };
            // A RUNE resolves to a TYPE before it resolves to a spelling, because the target spells
            // a byte and a character differently and has no untyped constant to defer the choice to.
            if let Some(spelled) = crate::body_literal::typed_literal(node, cx, value) {
                return spelled;
            }
            Ok(RustExpr::Literal(
                crate::items_value::readable_literal(value, cx.resolver)
                    .unwrap_or_else(|| value.to_owned()),
            ))
        }
        "zero" => zero_value(node, cx),
        "ident" if is_receiver(node) => Ok(RustExpr::SelfValue),
        "ident" => {
            refuse_deferred_reference(node, cx)?;
            refuse_sentinel_out_of_place(node, cx)?;
            Ok(RustExpr::Path(reference(node, cx.resolver)))
        }
        // A source-level parenthesis carries no information the tree does not already have, and
        // re-emitting it would fight the precedence the IR computes.
        "paren" => expression(one_child(node, cx, "paren")?, cx),
        "binary" => binary(node, cx),
        "unary" => {
            let spelling = operator_of(node, cx)?;
            // `&T{..}` — the address of a value this expression just created. No caller owns it,
            // nothing else can alias it, and no binding is moved out of, so the owned form is the
            // only one available and needs no destination to choose it.
            if spelling == OPERATOR_ADDRESS_OF
                && let Some(rendered) = address_of_fresh(node, cx)?
            {
                return Ok(rendered);
            }
            let op = unary_operator(spelling).ok_or_else(|| TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: unary_refusal(spelling),
            })?;
            Ok(RustExpr::Unary {
                op,
                operand: Box::new(expression(one_child(node, cx, "unary")?, cx)?),
            })
        }
        "selector" => selector(node, cx, position),
        "call" => call(node, cx),
        "index" => {
            let (base, index) = two_children(node, cx, "index")?;
            // Inside a loop that WALKS this sequence, an index by the counter it no longer has is
            // the element itself — the whole point of the rewrite.
            if let Some(walked) = &cx.walked
                && base.kind == KIND_IDENT
                && base.name == walked.sequence
                && index.kind == KIND_IDENT
                && index.name == walked.counter
            {
                return Ok(RustExpr::Path(walked.element.clone()));
            }
            Ok(RustExpr::Index {
                base: Box::new(crate::body_index::byte_indexed_base(base, cx)?),
                index: Box::new(crate::body_index::index_operand(index, cx)?),
            })
        }
        "closure" => crate::body_closure::closure(node, cx),
        "composite" => composite(node, cx),
        "convert" => convert(node, cx),
        "slice" => slice(node, cx),
        "unsupported" => Err(unsupported_source(node, cx)),
        other => Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!("expression kind `{other}` has no translation"),
        }),
    }
}

/// A binary operation, spelled so that OVERFLOW keeps its meaning.
///
/// The source defines integer overflow as wrapping; the target panics on it in a debug build and
/// wraps in a release one. Emitting the plain operator therefore turns one source program into two
/// target programs, neither of which is it — and it compiles, which is why nothing caught it until
/// a reviewer who did not know the code was generated read a mixing loop and asked what happens at
/// forty elements.
///
/// The result TYPE decides, and it is not recoverable from the operator: `+` on floats, on strings
/// and on integers are three different rules. The front end records it for exactly this.
fn binary(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let spelling = operator_of(node, cx)?;
    let (lhs, rhs) = two_children(node, cx, "binary")?;

    // An IDIOM first: it changes the spelling and never the program, so it applies wherever the
    // shape matches regardless of what the operands turn out to be.
    if let Some(rendered) = emptiness_test(spelling, lhs, rhs, cx)? {
        return Ok(rendered);
    }

    // IS THIS FAILURE THAT SENTINEL? The source compares identity; the target asks the trait object
    // what concrete type it holds. Available only because the sentinel became a type — while it was
    // its message there was nothing to compare, and this refused.
    if let Some(rendered) = crate::body_swap::identity_test(spelling, lhs, rhs, cx)? {
        return Ok(rendered);
    }

    crate::body_wider::refuse_opaque_newtype(lhs, rhs, spelling, cx)?;

    // A guard comparing a LENGTH CONSTANT against a length: both sides are the target's index type,
    // so the conversion the length call's mapping adds is what is wrong. The constant's declaration
    // read the same proof, so the two sides cannot end up different types.
    let (left, right) = match compares_lengths(node, cx) {
        true => (
            crate::counters::unsigned_bound(lhs, cx)?,
            crate::counters::unsigned_bound(rhs, cx)?,
        ),
        false => (
            newtype_operand(lhs, spelling, cx)?,
            newtype_operand(rhs, spelling, cx)?,
        ),
    };

    if let Some(method) = cx.resolver.wrapping_method(node, spelling) {
        return Ok(RustExpr::MethodCall {
            // A BARE INTEGER LITERAL has no type of its own in the target until something gives it
            // one, and a method call is not something that does: `2.wrapping_add(n)` is ambiguous
            // and does not compile. Every other position infers from its neighbours; this one has
            // to say. The type comes from the OPERATION, which is the type the source gave the
            // whole expression -- so the literal is spelled at the type it already had.
            receiver: Box::new(typed_receiver(left, node, cx)),
            method: method.to_owned(),
            args: vec![right],
        });
    }

    // CONCATENATION ONTO A LITERAL. The target's `+` on strings takes an OWNED left operand and
    // reuses its allocation; a literal is a borrowed `&'static str` and owns nothing, so
    // `"a" + &b` is not an operation the target has at all. The source has one string type and
    // cannot express the difference.
    //
    // Built as the target's formatting macro rather than by making the literal owned. `format!`
    // allocates once for the result, which is what the source's concatenation does;
    // `String::from("a") + &b` allocates for the literal and then again whenever the result
    // outgrows it, and reads like a workaround because it is one.
    if spelling == "+"
        && node.type_ref.name == crate::vocabulary::SOURCE_STRING
        && lhs.kind == KIND_LITERAL
    {
        return Ok(RustExpr::MacroCall {
            name: "format".to_owned(),
            template: "{}{}".to_owned(),
            args: vec![left, right],
        });
    }

    let op = binary_operator(spelling).ok_or_else(|| TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!("binary operator `{spelling}` has no direct translation"),
    })?;
    let built = RustExpr::Binary {
        op,
        lhs: Box::new(left),
        // CONCATENATION is not symmetric in the target. The source adds two strings and gets a
        // third; the target's `+` on an owned string takes a BORROW on the right and reuses the
        // left's allocation, so two owned operands do not typecheck. The source cannot express the
        // difference because it has only one string type — which is exactly why the asymmetry has to
        // be added here rather than recovered from the operand.
        rhs: Box::new(borrowed_concat_operand(node, right, spelling, cx)),
    };

    // A BOUNDED COMPARISON is a range test, which the target has and the source does not. Read from
    // what was BUILT rather than from the source shape, so it cannot fire on a conjunction that
    // merely looks like one — the same way the membership test is recognised.
    //
    // MECHANISM, not pack data, and the distinction is worth stating because most of this file's
    // decisions go the other way. The pack answers questions with more than one defensible answer;
    // this one has exactly one — `(A..=B).contains(&x)` and `x >= A && x <= B` are the same
    // predicate over the same values in the target, and a pack that said otherwise would be wrong
    // rather than different. It is also not derived from a seed: it comes from the target's own
    // lint, `clippy::manual_range_contains`, and the pack's idiom table requires a seed commit that
    // could only be invented here. The licensing policy that requires it is fail-closed on purpose,
    // so the rule goes where it needs no such claim.
    if let Some(ranged) = crate::body_swap::bounded_range(&built) {
        return Ok(ranged);
    }
    Ok(built)
}

/// A method-call receiver that is a bare integer literal, given the type the operation has.
///
/// Only a LITERAL, and only when the operation's type resolves to something the target spells as an
/// integer suffix. Anything else is returned untouched: a receiver that already has a type does not
/// need one, and suffixing an expression that is not a literal is not a thing the target allows.
fn typed_receiver(receiver: RustExpr, node: &Declaration, cx: &Body<'_>) -> RustExpr {
    let RustExpr::Literal(spelled) = &receiver else {
        return receiver;
    };
    if !spelled.bytes().all(|byte| byte.is_ascii_digit() || byte == b'_') {
        return receiver;
    }
    let Ok(resolved) = cx.resolver.resolve(&node.type_ref, cx.owner) else {
        return receiver;
    };
    let suffix = resolved.spelling();
    match matches!(
        suffix.as_str(),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
    ) {
        true => RustExpr::Literal(format!("{spelled}{suffix}")),
        false => receiver,
    }
}

/// An operand of an ARITHMETIC or ORDERING operator, reaching through a newtype wrapper.
///
/// The source's defined type and its underlying are one thing, so `v > 15` on a `type Version byte`
/// is a comparison of bytes. The target's newtype is a struct and carries NONE of its underlying
/// type's operators, so the same expression has to reach the field.
///
/// EQUALITY is excluded, and that is the whole reason this is keyed on the operator rather than
/// applied to every operand. The target's newtype derives `PartialEq`, so `a == b` between two of
/// them already means what the source meant; reaching through there would churn every existing
/// comparison to say the same thing in more characters.
fn newtype_operand(
    operand: &Declaration,
    spelling: &str,
    cx: &Body<'_>,
) -> Result<RustExpr, TransformError> {
    match matches!(spelling, "==" | "!=") {
        true => expression(operand, cx),
        false => crate::body_index::unwrapped_base(operand, cx),
    }
}

/// The right operand of a target CONCATENATION, borrowed.
///
/// Only for `+` on the source's string type, read from the type the front end recorded on the
/// operation rather than from the operands: the same operator on integers is arithmetic and must be
/// left exactly alone.
fn borrowed_concat_operand(
    node: &Declaration,
    right: RustExpr,
    spelling: &str,
    cx: &Body<'_>,
) -> RustExpr {
    if spelling != "+" || node.type_ref.name != crate::vocabulary::SOURCE_STRING {
        return right;
    }
    let _ = cx;
    RustExpr::Reference {
        mutable: false,
        inner: Box::new(right),
    }
}
