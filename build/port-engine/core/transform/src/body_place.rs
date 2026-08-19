//! The PLACES an expression reads from, and what reading one costs.
//!
//! Split from `body_expr.rs` because the dispatch and the reads answer different questions. A
//! field read, an address-of, a sentinel: each is a place the source names freely and the target
//! charges for — a clone, a box, or a refusal — and deciding which is what these are.

use port_engine_api::{Declaration, TypeRef};
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::body_argument::constructed;
use crate::body_expr::{Position, expression};
use crate::body_parts::one_child;
use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::vocabulary::{DISPOSITION_OWNED_POINTER, KIND_COMPOSITE, KIND_UNARY};

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
pub(crate) fn refuse_sentinel_out_of_place(
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
pub(crate) fn address_of_fresh(
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

pub(crate) fn selector(
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
pub(crate) fn convert(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
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
