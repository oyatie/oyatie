//! COMPOSITE literals: a struct's fields, or a sequence's elements.
//!
//! One module because the source spells both with the same production and tells them apart by the
//! type behind it — a struct names its fields, a sequence gives its elements in order. Separated
//! from the expression walk because each carries a question the walk does not: which fields a
//! literal LEFT OUT, and what the target spells for a sequence of a given kind.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustExpr;

use crate::body::{Body, one_child};
use crate::body_call::render_operand;
use crate::body_expr::expression;
use crate::error::TransformError;
use crate::naming::to_snake_case;
/// rather than something inferred from a spelling here.
/// A struct literal, with every field named.
///
/// Go zero-fills the fields a literal omits; the target has no such rule and rejects an incomplete
/// literal outright. The front end therefore emits one entry per DECLARED field, and the omitted
/// ones arrive as `zero` nodes carrying the field's type — so the target's zero is a pack answer
pub(crate) fn composite(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    // A SEQUENCE literal is a different construction from a struct one, and the type's kind is
    // what says which. A struct names its fields; a sequence gives its elements in order.
    if let Some(rendered) = sequence_literal(node, cx)? {
        return Ok(rendered);
    }

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

/// A slice, array or map literal, when the pack says what its kind becomes.
///
/// EMPTY is answered by the type's ZERO rather than by an empty construction. `[20]byte{}` is not
/// an empty array — it is twenty zero bytes, and the target spells that `[0u8; 20]`, which is
/// exactly the type's zero value. Answering it any other way would need the engine to invent a
/// length it already has.
///
/// A kind the pack declares no constructor for refuses through the ordinary path: a map literal is
/// the case, because the source's map has no order and the target's ordered map imposes one, which
/// makes the entry order observable where it was not.
///
/// # Errors
/// [`TransformError`] from translating an element.
fn sequence_literal(
    node: &Declaration,
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let kind = node.type_ref.kind.as_str();
    if !cx.resolver.is_sequence_literal(kind) {
        return Ok(None);
    }
    if node.children.is_empty() {
        return zero_value(node, cx).map(Some);
    }

    let mut elements = Vec::with_capacity(node.children.len());
    for child in &node.children {
        elements.push(render_element(&expression(child, cx)?, cx)?);
    }
    let Some(rendered) = cx.resolver.sequence_form(kind, &elements) else {
        return Ok(None);
    };
    Ok(Some(RustExpr::Literal(rendered)))
}

/// One element of a sequence literal, as target text for the constructor to join.
///
/// Only forms whose text is unambiguous are admitted, for the same reason the mapped-call template
/// admits only those: a constructor is textual joining, and an element that reassociates would
/// need brackets this cannot see the need for.
fn render_element(expr: &RustExpr, cx: &Body<'_>) -> Result<String, TransformError> {
    render_operand(expr).ok_or_else(|| TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: "an element of a sequence literal is a compound expression, and the pack builds \
                 that literal from a TEXT template — substituting one would need parentheses the \
                 template cannot ask for"
            .to_owned(),
    })
}

/// The target's zero for a source type the literal left out.
///
/// Refuses BY NAME rather than reaching for `Default::default()`. That would compile for the types
/// the corpus has and would silently be a different program for the ones it does not: a type whose
/// `Default` impl is not its zero value, or one that has no `Default` at all and only fails much
/// later, in the emitted crate, with the source declaration nowhere in the message.
pub(crate) fn zero_value(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
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
