//! Expressions.
//!
//! Two decisions live here that a naive translator gets silently wrong, and both are about the
//! difference between what the source's syntax MEANS and what the same syntax means in the target:
//! reading a field is a copy in Go and a move in Rust, and a struct literal zero-fills in Go and
//! must name every field in Rust.

use port_engine_api::{Declaration, TypeRef};
use port_engine_api::PointerConstruction;
use port_engine_rust_ir::RustExpr;

use crate::body::{Body, one_child, two_children, unsupported_source};
use crate::body_index::slice;
use crate::body_call::call;
use crate::body_idiom::emptiness_test;
use crate::body_ops::{
    binary_operator, is_receiver, operator_of, own_string_for, reference,
    refuse_deferred_reference, unary_operator, unary_refusal,
};
use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::vocabulary::{
    ATTR_CALLEE, ATTR_CALLEE_KIND, ATTR_LIT_KIND, ATTR_VALUE, CALLEE_KIND_METHOD, IDIOM_EMPTY_STRING, KIND_LITERAL, KIND_UNARY, LIT_KIND_STRING, OPERATOR_ADDRESS_OF,
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
            Ok(RustExpr::Path(reference(node)))
        }
        // A source-level parenthesis carries no information the tree does not already have, and
        // re-emitting it would fight the precedence the IR computes.
        "paren" => expression(one_child(node, cx, "paren")?, cx),
        "binary" => binary(node, cx),
        "unary" => {
            let spelling = operator_of(node, cx)?;
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

/// A field access, cloned when reading it would MOVE.
///
/// The source copied; `.clone()` is what that costs in the target. In PLACE position there is no
/// read at all, so there is nothing to clone — and cloning there would emit an assignment to a
/// temporary, which parses and silently does nothing.
fn selector(
    node: &Declaration,
    cx: &Body<'_>,
    position: Position,
) -> Result<RustExpr, TransformError> {
    let field = RustExpr::Field {
        base: Box::new(expression(one_child(node, cx, "selector")?, cx)?),
        name: to_snake_case(&node.name),
    };
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
fn moves_on_read(type_ref: &TypeRef, cx: &Body<'_>) -> bool {
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

/// A struct literal, with every field named.
///
/// Go zero-fills the fields a literal omits; the target has no such rule and rejects an incomplete
/// literal outright. The front end therefore emits one entry per DECLARED field, and the omitted
/// ones arrive as `zero` nodes carrying the field's type — so the target's zero is a pack answer
/// rather than something inferred from a spelling here.
fn composite(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let path = cx.resolver.resolve(&node.type_ref, cx.owner).map_err(|_| {
        TransformError::MissingDatum {
            construction: "composite".to_owned(),
            name: cx.owner.to_owned(),
            datum: "type",
        }
    })?;

    let keyed = node.children_of_kind("keyed");
    let mut fields = Vec::with_capacity(keyed.len());
    for entry in keyed {
        fields.push((
            to_snake_case(&entry.name),
            expression(one_child(entry, cx, "keyed")?, cx)?,
        ));
    }
    Ok(RustExpr::StructLiteral {
        path: path.spelling(),
        fields,
    })
}

/// The target's zero for a source type the literal left out.
///
/// Refuses BY NAME rather than reaching for `Default::default()`. That would compile for the types
/// the corpus has and would silently be a different program for the ones it does not: a type whose
/// `Default` impl is not its zero value, or one that has no `Default` at all and only fails much
/// later, in the emitted crate, with the source declaration nowhere in the message.
fn zero_value(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    cx.resolver
        .zero_value(&node.type_ref)
        .map(RustExpr::Literal)
        .ok_or_else(|| TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "a struct literal omits field `{}` of type `{}`, and the pack declares no zero \
                 value for it — Go fills the field with that type's zero and the target must \
                 spell it out",
                node.name,
                node.type_ref.describe()
            ),
        })
}
