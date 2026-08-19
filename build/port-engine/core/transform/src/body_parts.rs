//! The small shared accessors every body file reaches for.
//!
//! Split from `body.rs` because they are a different thing from the statement translator: each
//! one asks the source node for a part it must have, and NAMES the owner and the part in the
//! refusal when it does not. That naming is the whole point — a translator that unwraps a missing
//! child produces a panic with no site in it, and one that skips it produces a body that is
//! quietly not the source's.

use port_engine_api::Declaration;

use crate::body::Body;
use crate::error::TransformError;
use crate::vocabulary::ATTR_SOURCE_NODE;

/// A named child holding a statement list.
pub(crate) fn branch<'a>(
    node: &'a Declaration,
    kind: &str,
    cx: &Body<'_>,
) -> Result<&'a Declaration, TransformError> {
    node.children_of_kind(kind)
        .first()
        .copied()
        .ok_or_else(|| TransformError::MissingDatum {
            construction: node.kind.clone(),
            name: cx.owner.to_owned(),
            datum: "body",
        })
}

/// The one child of a given kind, named in the refusal when it is absent.
pub(crate) fn named_child<'a>(
    node: &'a Declaration,
    kind: &'static str,
    cx: &Body<'_>,
    construction: &str,
) -> Result<&'a Declaration, TransformError> {
    node.children_of_kind(kind)
        .first()
        .copied()
        .ok_or_else(|| TransformError::MissingDatum {
            construction: construction.to_owned(),
            name: cx.owner.to_owned(),
            datum: kind,
        })
}

pub(crate) fn unsupported_source(node: &Declaration, cx: &Body<'_>) -> TransformError {
    let source_node = node
        .attr(ATTR_SOURCE_NODE)
        .unwrap_or("an unnamed construct");
    TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!(
            "source construct `{source_node}` has no translation yet — a rule for it belongs in \
             the pack, and the analysis in docs/programs/k8s-port/census/"
        ),
    }
}

pub(crate) fn one_child<'a>(
    node: &'a Declaration,
    cx: &Body<'_>,
    what: &str,
) -> Result<&'a Declaration, TransformError> {
    node.children
        .first()
        .ok_or_else(|| TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!("`{what}` node carries no operand"),
        })
}

pub(crate) fn two_children<'a>(
    node: &'a Declaration,
    cx: &Body<'_>,
    what: &str,
) -> Result<(&'a Declaration, &'a Declaration), TransformError> {
    match node.children.as_slice() {
        [lhs, rhs] => Ok((lhs, rhs)),
        other => Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!("`{what}` node needs two operands, got {}", other.len()),
        }),
    }
}
