//! Operator and identifier tables.
//!
//! Every table here is TOTAL over what it admits and returns `None` otherwise, so an operator with
//! no faithful target form reaches a refusal instead of a plausible substitute.

use port_engine_api::Declaration;
use port_engine_rust_ir::{BinaryOp, UnaryOp};

use crate::body::Body;
use crate::error::TransformError;
use crate::naming::{to_pascal_case, to_screaming_snake, to_snake_case};
use crate::vocabulary::{ATTR_OP, ATTR_REF};

/// Case an identifier by what it REFERS to.
///
/// A reference to a constant must render in the target's constant casing or it names nothing at
/// all, so the front end's classification is used rather than one default applied to everything.
pub(crate) fn reference(node: &Declaration) -> String {
    match node.attr(ATTR_REF) {
        Some("const") => to_screaming_snake(&node.name),
        Some("type") => to_pascal_case(&node.name),
        _ => to_snake_case(&node.name),
    }
}

/// `true` when this identifier is the enclosing method's receiver, whose target spelling is not
/// its name.
pub(crate) fn is_receiver(node: &Declaration) -> bool {
    node.attr(ATTR_REF) == Some("receiver")
}

pub(crate) fn binary_operator(spelling: &str) -> Option<BinaryOp> {
    Some(match spelling {
        "+" => BinaryOp::Add,
        "-" => BinaryOp::Sub,
        "*" => BinaryOp::Mul,
        "/" => BinaryOp::Div,
        "%" => BinaryOp::Rem,
        "==" => BinaryOp::Eq,
        "!=" => BinaryOp::Ne,
        "<" => BinaryOp::Lt,
        "<=" => BinaryOp::Le,
        ">" => BinaryOp::Gt,
        ">=" => BinaryOp::Ge,
        "&&" => BinaryOp::And,
        "||" => BinaryOp::Or,
        "&" => BinaryOp::BitAnd,
        "|" => BinaryOp::BitOr,
        "^" => BinaryOp::BitXor,
        "<<" => BinaryOp::Shl,
        ">>" => BinaryOp::Shr,
        // `&^` (AND NOT) has no single-operator target form. It is spellable as `& !`, but the
        // operand widths differ between the languages and a silent rewrite of a bit operation is
        // exactly the class of change nobody reviews.
        _ => return None,
    })
}

pub(crate) fn unary_operator(spelling: &str) -> Option<UnaryOp> {
    Some(match spelling {
        "-" => UnaryOp::Neg,
        // Logical NOT and bitwise NOT are both `!` in the target, distinguished by operand type
        // rather than by spelling.
        "!" | "^" => UnaryOp::Not,
        // `&` and `*` are references and dereferences. Both are aliasing decisions, which
        // docs/programs/k8s-port/census/ownership-escape.md exists to work out.
        _ => return None,
    })
}

pub(crate) fn operator_of<'a>(
    node: &'a Declaration,
    cx: &Body<'_>,
) -> Result<&'a str, TransformError> {
    node.attr(ATTR_OP)
        .ok_or_else(|| TransformError::MissingDatum {
            construction: node.kind.clone(),
            name: cx.owner.to_owned(),
            datum: ATTR_OP,
        })
}
