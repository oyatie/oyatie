//! The source's ALLOCATING builtin, which is not shaped like a call.
//!
//! `make([]byte, 0, n)` takes a TYPE and then numbers whose meaning changes with how many there
//! are. Neither fits a form keyed by callee identity, which is why this is its own path and its own
//! pack table.
//!
//! Two shapes are exact and everything else refuses by name. The distinction that matters is the
//! one between them: `make([]T, 0, n)` has NO elements and room for n, and `make([]T, n)` has n
//! ZERO elements. The same number in two different roles — exactly the shape of mistake that
//! compiles and means something else.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::body_expr::expression;
use crate::error::TransformError;
use crate::vocabulary::{ATTR_VALUE, KIND_LITERAL, KIND_TYPE};

/// Translate an allocating builtin, or say this is not one.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the declaration and the shape the pack does not answer.
pub(crate) fn allocation(
    node: &Declaration,
    callee: &str,
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    if callee != "make" {
        return Ok(None);
    }
    let rule = cx.resolver.allocation;
    let [_, allocated, sizes @ ..] = node.children.as_slice() else {
        return Ok(None);
    };
    // The first argument is a TYPE, which is what makes this not a call. Anything else here is a
    // shape the front end did not record the way this expects.
    if allocated.kind != KIND_TYPE {
        return Ok(None);
    }
    let refuse = |what: &str, why: &str| TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!("`make` {what}: {why}"),
    };

    // A SEQUENCE only. A map's target form is a decision about which map, and a channel is a
    // concurrency primitive whose counterpart is a decision about the ported program.
    if allocated.type_ref.kind != "slice" {
        return Err(refuse(
            &format!("allocates a `{}`", allocated.type_ref.kind),
            &rule.reason,
        ));
    }

    match sizes {
        // `make([]T, 0, n)` — no elements, room for n.
        [length, capacity] if is_zero(length) && !rule.empty_with_capacity.is_empty() => {
            let room = crate::body_call::render_operand(&expression(capacity, cx)?)
                .ok_or_else(|| {
                    refuse(
                        "is given a capacity that is a compound expression",
                        "the pack answers this with a text template, and substituting one would \
                         need parentheses the template cannot ask for",
                    )
                })?;
            Ok(Some(RustExpr::Literal(
                rule.empty_with_capacity.replace("{0}", &room),
            )))
        }
        // `make([]T, n)` — n elements, each the element type's zero.
        [length] if !rule.filled.is_empty() => {
            let element = allocated.type_ref.args.first().ok_or_else(|| {
                refuse(
                    "allocates a sequence whose element type the front end did not record",
                    "the fill value comes from that type, and there is nothing to ask",
                )
            })?;
            let zero = cx.resolver.zero_value(element).ok_or_else(|| {
                refuse(
                    &format!("fills with the zero of `{}`", element.describe()),
                    &rule.filled_reason,
                )
            })?;
            let count = crate::body_call::render_operand(&expression(length, cx)?)
                .ok_or_else(|| {
                    refuse(
                        "is given a length that is a compound expression",
                        "the pack answers this with a text template",
                    )
                })?;
            Ok(Some(RustExpr::Literal(
                rule.filled.replace("{0}", &count).replace("{1}", &zero),
            )))
        }
        // `make([]T, n, m)` with a non-zero length is BOTH shapes at once, and the target spells
        // them separately. Which of the two a call site wants is a decision about that call site.
        _ => Err(refuse(
            &format!("is given {} sizes in a shape the pack does not answer", sizes.len()),
            &rule.empty_with_capacity_reason,
        )),
    }
}

/// Whether this argument is the literal zero, which is what separates the two sequence shapes.
fn is_zero(node: &Declaration) -> bool {
    node.kind == KIND_LITERAL && node.attr(ATTR_VALUE) == Some("0")
}
