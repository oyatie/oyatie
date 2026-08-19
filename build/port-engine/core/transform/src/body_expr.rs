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
use crate::body_parts::{one_child, two_children, unsupported_source};
use crate::body_index::slice;
use crate::body_call::{call, constructed};
use crate::body_literal::{composite, zero_value};
use crate::body_idiom::emptiness_test;
use crate::body_ops::{
    binary_operator, is_receiver, operator_of, own_string_for, reference,
    refuse_deferred_reference, unary_operator, unary_refusal,
};
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
        "literal" => node
            .attr(ATTR_VALUE)
            .map(|value| RustExpr::Literal(value.to_owned()))
            .ok_or_else(|| TransformError::MissingDatum {
                construction: "literal".to_owned(),
                name: cx.owner.to_owned(),
                datum: ATTR_VALUE,
            }),
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
                base: Box::new(expression(base, cx)?),
                index: Box::new(crate::body_index::index_operand(index, cx)?),
            })
        }
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

    let (left, right) = (expression(lhs, cx)?, expression(rhs, cx)?);

    if let Some(method) = cx.resolver.wrapping_method(node, spelling) {
        return Ok(RustExpr::MethodCall {
            receiver: Box::new(left),
            method: method.to_owned(),
            args: vec![right],
        });
    }

    let op = binary_operator(spelling).ok_or_else(|| TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!("binary operator `{spelling}` has no direct translation"),
    })?;
    Ok(RustExpr::Binary {
        op,
        lhs: Box::new(left),
        rhs: Box::new(right),
    })
}

/// A SENTINEL read anywhere but a failing return, which has no target expression.
///
/// The sentinel is emitted as its MESSAGE, and a failure is built from it at the one place a
/// failure is wanted — a failing return, where `fallible_return` knows that is what the operand is.
/// An identifier does not know where it stands, so building one here would build a failure in
/// places that want something else.
///
/// The comparison is the case that matters and the cost the decision names: the source's
/// `errors.New` returns a POINTER, so `err == ErrGone` compares identity and is a line real code
/// writes. The target's boxed trait object has no equality, so nothing means what that line means —
/// and emitting a comparison against a freshly built value would be FALSE at every call. It refuses
/// here rather than emitting that.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the sentinel and where it can be used.
fn refuse_sentinel_out_of_place(
    node: &Declaration,
    cx: &Body<'_>,
) -> Result<(), TransformError> {
    if !cx.resolver.scope.sentinels.contains_key(&node.name) {
        return Ok(());
    }
    Err(TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!(
            "`{}` is a SENTINEL failure, which is emitted as its message — so it can be built into \
             a failure where one is returned, and read nowhere else. The source's sentinel has \
             POINTER identity and the target's failure is a boxed trait object with no equality, so \
             a comparison against it has no target expression at all: emitting one against a \
             freshly built value would be false at every call. What is missing is a decision about \
             sentinel identity, not a spelling.",
            node.name
        ),
    })
}

/// The address of a freshly built composite, which is an owned pointer.
///
/// `None` when the operand is not a composite literal — `&x` of an existing binding borrows or
/// moves something that already has an owner, which is the ownership question the signature table
/// exists to answer, and a fresh value simply does not have it.
///
/// Built from the disposition that already describes the owned pointer rather than a second rule
/// saying the same thing: one place says what an owned pointer is and how one is constructed, and
/// a second would be free to disagree with it.
///
/// # Errors
/// [`TransformError`] from translating the composite.
fn address_of_fresh(
    node: &Declaration,
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let operand = one_child(node, cx, KIND_UNARY)?;
    if operand.kind != KIND_COMPOSITE {
        return Ok(None);
    }
    let Some(construction) = cx.resolver.ownership.construction_for(DISPOSITION_OWNED_POINTER)
    else {
        return Ok(None);
    };
    Ok(Some(constructed(construction, expression(operand, cx)?)))
}

/// A field access, cloned when reading it would MOVE.
///
/// The source copied; `.clone()` is what that costs in the target. In PLACE position there is no
/// read at all, so there is nothing to clone — and cloning there would emit an assignment to a
/// temporary, which parses and silently does nothing.
/// The PLACE a field selector names, with no decision about copying it.
///
/// Split out so a caller that has already decided the destination borrows — a getter's return — gets
/// the field itself rather than the clone the value position would add.
///
/// # Errors
/// [`TransformError`] from translating the base.
pub(crate) fn field_place(
    node: &Declaration,
    cx: &Body<'_>,
) -> Result<RustExpr, TransformError> {
    Ok(RustExpr::Field {
        base: Box::new(expression(one_child(node, cx, "selector")?, cx)?),
        name: to_snake_case(&node.name),
    })
}

fn selector(
    node: &Declaration,
    cx: &Body<'_>,
    position: Position,
) -> Result<RustExpr, TransformError> {
    let field = field_place(node, cx)?;
    if position == Position::Value && moves_on_read(&node.type_ref, cx) {
        return Ok(RustExpr::MethodCall {
            receiver: Box::new(field),
            method: "clone".to_owned(),
            args: Vec::new(),
        });
    }
    Ok(field)
}

/// Whether a plain read of this type moves in the target, and therefore needs a clone.
///
/// An absent type is NOT treated as moving. The front end records a type on every selector it can
/// resolve, so an absent one means the expression is not a field read at all — and cloning
/// something the engine could not identify would be a guess.
pub(crate) fn moves_on_read(type_ref: &TypeRef, cx: &Body<'_>) -> bool {
    if type_ref.is_empty() {
        return false;
    }
    !cx.resolver.copies(type_ref)
}

/// A type CONVERSION, which the source spells exactly like a call.
///
/// Three forms, because they are three operations. To a type the corpus declares, the target's
/// newtype CONSTRUCTS from the value. Between numeric types the source is defined to truncate, and
/// the target spells that as a cast — faithful, and the place `num-cast-try-from` will want
/// revisiting once the pack can say which conversions are meant to be checked. Anything else
/// refuses: converting between a string and a byte slice is infallible and lossy in the source and
/// FALLIBLE in the target, which is a decision about invalid input rather than a spelling.
fn convert(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let operand = expression(one_child(node, cx, "convert")?, cx)?;
    let target = &node.type_ref;

    // A named type the corpus declares emits as a newtype, so converting to it is construction.
    if target.kind == "named" {
        let path = cx.resolver.resolve(target, cx.owner)?;
        return Ok(RustExpr::Call {
            callee: Box::new(RustExpr::Path(path.spelling())),
            args: vec![operand],
        });
    }

    if target.kind == "basic" && cx.resolver.converts_by_cast(target) {
        let rendered = cx.resolver.resolve(target, cx.owner)?;
        return Ok(RustExpr::Cast {
            expr: Box::new(operand),
            ty: rendered,
        });
    }

    Err(TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!(
            "converting to `{}` has no declared target form — the source's conversion is \
             infallible and the target's is not, so what happens to input the target rejects is a \
             decision the pack has to make rather than a spelling",
            target.describe()
        ),
    })
}
