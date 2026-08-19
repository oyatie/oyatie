//! IDIOM rules: spellings the target prefers for something the source says another way.
//!
//! Separated from the expression walk because an idiom is a different KIND of rule from everything
//! around it. Every other decision here changes what the emitted program does or refuses to; an
//! idiom changes only how it reads, and an idiom that alters meaning is not an idiom but a bug.
//!
//! Each rule is pack data carrying its seed provenance, because
//! `specs/k8s-port/licensing.json` rejects a rust-skills-derived rule without `seed_source`,
//! `seed_license` and `seed_commit` — a rule whose derivation cannot be re-checked is a rule
//! nobody can audit.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, UnaryOp};

use crate::body::Body;
use crate::body_expr::expression;
use crate::error::TransformError;
use crate::vocabulary::{ATTR_LIT_KIND, ATTR_VALUE, IDIOM_EMPTY_STRING, KIND_LITERAL, LIT_KIND_STRING};

/// `x == ""` and `x != ""`, which the target spells as a method.
///
/// Exactly equivalent — both are true precisely when the value has length zero — so this changes
/// the spelling and not the program, which is what makes it an idiom rather than a translation.
/// `clippy::style` flags the comparison form, and a comment a reviewer would make on hand-written
/// code is a defect in code held to that bar.
///
/// Either operand may be the literal, because the source permits `"" == x` too.
///
/// # Errors
/// [`TransformError`] from translating the non-literal operand.
pub(crate) fn emptiness_test(
    spelling: &str,
    lhs: &Declaration,
    rhs: &Declaration,
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let Some(method) = cx.resolver.idiom_method(IDIOM_EMPTY_STRING) else {
        return Ok(None);
    };
    let negated = match spelling {
        "==" => false,
        "!=" => true,
        _ => return Ok(None),
    };
    let subject = match (is_empty_string(lhs), is_empty_string(rhs)) {
        (true, false) => rhs,
        (false, true) => lhs,
        // Both literals is a comparison of two constants, and neither is a subject to ask about
        // emptiness; neither is not this shape at all.
        _ => return Ok(None),
    };

    let call = RustExpr::MethodCall {
        receiver: Box::new(expression(subject, cx)?),
        method: method.to_owned(),
        args: Vec::new(),
    };
    Ok(Some(match negated {
        false => call,
        true => RustExpr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(call),
        },
    }))
}

/// Whether this node is the empty string literal the source compares against.
fn is_empty_string(node: &Declaration) -> bool {
    node.kind == KIND_LITERAL
        && node.attr(ATTR_LIT_KIND) == Some(LIT_KIND_STRING)
        && node.attr(ATTR_VALUE) == Some("\"\"")
}
