//! Indexing and slicing.
//!
//! One module because they are the same question asked twice: what a subscript MEANS in the target.
//! The source indexes and slices the same backing array; the target distinguishes an element from a
//! borrowed view of several, and getting that wrong is a copy where the source had none.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::body_expr::expression;
use crate::error::TransformError;

/// An index operand, converted to what the target insists on.
///
/// The source indexes with its `int`; the target indexes with `usize`, and that mismatch has to
/// land somewhere. It used to land on `len`, which was mapped so its value would type as the
/// source's `int` — right for `return len(s)` and wrong for the counter of every indexed loop,
/// whose body then would not compile.
///
/// Here it lands at the only place the target actually insists: a LITERAL needs nothing, because
/// the target infers it, and every other operand converts.
///
/// The trade, recorded rather than discovered: a negative index panics in both languages for
/// different reasons — the source bounds-checks a negative, the target wraps it to an enormous
/// `usize` and bounds-checks that. Same outcome, different message.
pub(crate) fn index_operand(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let operand = expression(node, cx)?;
    if already_index_typed(node, cx) {
        return Ok(operand);
    }
    Ok(RustExpr::Cast {
        expr: Box::new(operand),
        ty: port_engine_rust_ir::RustType::path("usize"),
    })
}

/// `s[lo:hi]` — a BORROWED subrange, with either bound optional.
///
/// The bounds arrive positionally with an explicit `absent` node for the ones the source left out,
/// so `s[:hi]` and `s[lo:]` stay distinguishable. Reconstructing that from arity would be guessing
/// which end was missing.
pub(crate) fn slice(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let [base, low, high] = node.children.as_slice() else {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "a slice expression needs a base and both bounds, got {} operands",
                node.children.len()
            ),
        });
    };
    // A slice bound is an index like any other, and needs the same conversion for the same reason.
    let bound = |operand: &Declaration| -> Result<Option<Box<RustExpr>>, TransformError> {
        if operand.kind == "absent" {
            return Ok(None);
        }
        Ok(Some(Box::new(index_operand(operand, cx)?)))
    };
    // SLICING A SOURCE STRING has no faithful target form, and the one that looks right is the
    // dangerous one. The source's string is bytes and `s[a:b]` takes bytes; the target's is
    // guaranteed UTF-8 and `&s[a..b]` PANICS when either bound falls inside a multi-byte character.
    // The source cannot panic there at all. Emitting it anyway is a program that agrees with the
    // source on every ASCII input and aborts on the first one that is not — which is exactly the
    // defect that is invisible until it is in production.
    //
    // `&s.as_bytes()[a..b]` is faithful and is NOT substituted here, because it yields the target's
    // byte slice where the source yielded a string: every destination that wanted a string would
    // then be wrong, and that is a decision about the string type of the whole ported program
    // rather than about this expression.
    if is_source_string(base) {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: "slicing the source's string has no faithful target form: the source slices \
                     BYTES and cannot fail, and the target's string slice panics when a bound \
                     falls inside a multi-byte character — which needs the ported program's \
                     string type decided rather than this expression rewritten"
                .to_owned(),
        });
    }

    Ok(RustExpr::Slice {
        base: Box::new(unwrapped_base(base, cx)?),
        low: bound(low)?,
        high: bound(high)?,
    })
}

/// The base of an index or slice, reaching through a NEWTYPE where the target wraps one.
///
/// The source's named array IS the array — `type ID [12]byte` admits `id[:]` because the name and
/// the array are the same thing there. The target's newtype wraps it, so the same expression has to
/// reach the field first, and emitting the source's spelling produces `cannot index into a value of
/// type &Id`.
///
/// Only for the RECEIVER, because that is the case the body can answer. The front end records a type
/// on an expression only where one is needed and a receiver carries none, so what the body knows is
/// which declaration it is inside — and the scope maps that to whether the target shape wraps. An
/// index through any other binding of a newtype is a shape the corpus does not have, and it arrives
/// here unchanged rather than being guessed at.
pub(crate) fn unwrapped_base(
    base: &Declaration,
    cx: &Body<'_>,
) -> Result<RustExpr, TransformError> {
    unwrapped_in(base, cx, crate::body_expr::Position::Value)
}

/// The same, at a stated position.
///
/// A BORROW position does not need the copy a value position asks for. A field read of a
/// non-copying type clones, because reading one moves in the target — and a receiver, an index base
/// and a formatting operand are all borrowed rather than consumed, so the clone there is an
/// allocation nothing keeps.
pub(crate) fn unwrapped_in(
    base: &Declaration,
    cx: &Body<'_>,
    position: crate::body_expr::Position,
) -> Result<RustExpr, TransformError> {
    let translated = crate::body_expr::in_position(base, cx, position)?;
    let wraps = unwraps_newtype(base, cx);
    if !wraps {
        return Ok(translated);
    }
    Ok(RustExpr::TupleIndex {
        base: Box::new(translated),
        index: 0,
    })
}

/// Whether this occurrence is a value of one of this unit's NEWTYPES, which the target wraps.
pub(crate) fn unwraps_newtype(base: &Declaration, cx: &Body<'_>) -> bool {
    match crate::body_ops::is_receiver(base) {
        true => cx
            .receiver_type
            .is_some_and(|owner| cx.resolver.scope.newtypes.contains_key(owner)),
        // A PARAMETER of newtype type, which the signature stated and the body was told.
        false => {
            base.kind == crate::vocabulary::KIND_IDENT && cx.newtype_parameters.contains(&base.name)
        }
    }
}

/// Whether this operand's source type is the source's STRING.
///
/// Read from the type the front end recorded on the expression. A string it could not type is not
/// treated as one: guessing here would put `.as_bytes()` on a sequence that already is one.
pub(crate) fn is_source_string(operand: &Declaration) -> bool {
    operand.type_ref.kind == "basic" && operand.type_ref.name == crate::vocabulary::SOURCE_STRING
}

/// The base of an index, reaching through the source's string to the BYTES it is.
///
/// A source string is a sequence of BYTES that may hold anything, and `s[i]` yields one of them.
/// The target's string is guaranteed UTF-8 and is not indexable at all, so the index has to go
/// through its bytes -- which is the same read of the same byte, and cannot fail where the source's
/// could not.
///
/// Applied after the newtype reach-through, because a newtype OVER a string is still a string.
pub(crate) fn byte_indexed_base(
    base: &Declaration,
    cx: &Body<'_>,
) -> Result<RustExpr, TransformError> {
    let translated = unwrapped_base(base, cx)?;
    match is_source_string(base) {
        false => Ok(translated),
        true => Ok(RustExpr::MethodCall {
            receiver: Box::new(translated),
            method: "as_bytes".to_owned(),
            args: Vec::new(),
        }),
    }
}

/// Whether this operand is ALREADY the target's index type, so converting it would say something
/// about the value that is not true.
///
/// Three ways an operand gets there, and a fourth that is the composition of them:
///
/// - a LITERAL, which takes the type of the position it stands in;
/// - a proven index-only COUNTER, because the range that built it dropped its own conversion;
/// - a proven LENGTH CONSTANT, by the same proof the declaration read.
///
/// And ARITHMETIC over those. `buf[i + 1]` where `i` is proven is an index computed from index
/// values, and casting it casts a `usize` to a `usize` -- which the target's own lint rejects, and
/// which is what this end used to do the moment the operand stopped being a bare name. Both ends
/// have to read one proof or they disagree.
fn already_index_typed(node: &Declaration, cx: &Body<'_>) -> bool {
    match node.kind.as_str() {
        "literal" => true,
        "ident" => {
            cx.usize_counters.contains(&node.name)
                || (node.attr(crate::vocabulary::ATTR_REF) == Some(crate::vocabulary::REF_CONST)
                    && cx.resolver.scope.length_constants.contains(&node.name))
        }
        // EVERY operand, because one side of the wrong type would not compile whichever way this
        // answered -- so the only safe answer is the one that holds for all of them.
        "binary" | "paren" => node
            .children
            .iter()
            .all(|child| already_index_typed(child, cx)),
        _ => false,
    }
}
