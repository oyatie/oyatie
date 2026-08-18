//! Function bodies: source statements into IR nodes.
//!
//! The supported subset is small ON PURPOSE and everything outside it refuses BY NAME. A
//! translator that guesses at a construct it does not understand emits code that compiles and is
//! wrong, which the receipt then certifies as reproducible.
//!
//! Nothing here builds text. Operator precedence is the IR's problem now, which is why the
//! operator tables in [`crate::body_ops`] map to typed operators rather than to spellings: a
//! spelling has to be parenthesised defensively, a typed operator carries its own binding power.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustStmt};

use crate::body_expr::{Position, expression, in_position};
use crate::body_loops::{counted_loop, range_loop, switch};
use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::resolve::Resolver;
use crate::vocabulary::ATTR_SOURCE_NODE;

/// What one body translation needs in order to answer a question about the TARGET.
///
/// Threaded rather than ambient. An earlier version kept the copy-type set in a thread-local to
/// avoid the plumbing, and the moment a second pack table arrived the shortcut stopped paying for
/// itself — these tables are not properties of the process, they are properties of the rule pack,
/// and a body translated under a different pack must see different answers.
pub(crate) struct Body<'a> {
    /// The declaration being translated. Every refusal names it, which is the whole reason it is
    /// carried down rather than reconstructed at the top.
    pub(crate) owner: &'a str,
    /// The pack's answers: type mapping, copy types, zero values, ownership.
    pub(crate) resolver: &'a Resolver<'a>,
}

impl<'a> Body<'a> {
    pub(crate) fn new(owner: &'a str, resolver: &'a Resolver<'a>) -> Self {
        Self { owner, resolver }
    }
}

/// Translate a function body's statements.
///
/// A trailing `return` becomes a TAIL EXPRESSION. That is a target-language idiom rather than a
/// change of meaning — `return x;` as the last statement of a function and `x` are the same
/// program — and it is owned here for the same reason identifier casing is: this face renders
/// Rust, so Rust's conventions are its business.
///
/// # Errors
/// [`TransformError::Unsupported`] for any construct outside the translated subset.
pub(crate) fn statements(
    nodes: &[Declaration],
    owner: &str,
    resolver: &Resolver<'_>,
) -> Result<Vec<RustStmt>, TransformError> {
    translate(nodes, &Body::new(owner, resolver), TailPosition::Yes)
}

/// Whether the last statement of this sequence is in TAIL position — the position whose value is
/// the enclosing block's value.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum TailPosition {
    Yes,
    No,
}

pub(crate) fn translate(
    nodes: &[Declaration],
    cx: &Body<'_>,
    tail: TailPosition,
) -> Result<Vec<RustStmt>, TransformError> {
    let mut out = Vec::with_capacity(nodes.len());
    for (index, node) in nodes.iter().enumerate() {
        let is_tail = tail == TailPosition::Yes && index + 1 == nodes.len();
        out.push(statement(node, cx, is_tail)?);
    }
    Ok(out)
}

pub(crate) fn statement(
    node: &Declaration,
    cx: &Body<'_>,
    is_last: bool,
) -> Result<RustStmt, TransformError> {
    match node.kind.as_str() {
        "return" => translated_return(node, cx, is_last),
        "block" => Ok(RustStmt::Semi(RustExpr::Block(translate(
            &node.children,
            cx,
            TailPosition::No,
        )?))),
        "if" => Ok(RustStmt::Semi(conditional(node, cx)?)),
        "let" => Ok(RustStmt::Let {
            name: to_snake_case(&node.name),
            value: expression(one_child(node, cx, "let")?, cx)?,
        }),
        "expr_stmt" => Ok(RustStmt::Semi(expression(
            one_child(node, cx, "expr_stmt")?,
            cx,
        )?)),
        "assign" => {
            let (target, value) = two_children(node, cx, "assign")?;
            Ok(RustStmt::Assign {
                target: in_position(target, cx, Position::Place)?,
                value: expression(value, cx)?,
            })
        }
        "break" => Ok(RustStmt::Break),
        "for" => counted_loop(node, cx),
        "range" => range_loop(node, cx),
        "switch" => Ok(RustStmt::Semi(switch(node, cx)?)),
        "unsupported" => Err(unsupported_source(node, cx)),
        other => Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!("statement kind `{other}` has no translation"),
        }),
    }
}

fn translated_return(
    node: &Declaration,
    cx: &Body<'_>,
    is_last: bool,
) -> Result<RustStmt, TransformError> {
    let values = node
        .children
        .iter()
        .map(|child| expression(child, cx))
        .collect::<Result<Vec<_>, _>>()?;
    let value = match values.len() {
        0 => None,
        1 => values.into_iter().next(),
        // Several results leave as a tuple, matching how the signature renders them.
        _ => Some(RustExpr::Tuple(values)),
    };
    match (is_last, value) {
        (true, Some(expr)) => Ok(RustStmt::Tail(expr)),
        (_, value) => Ok(RustStmt::Return(value)),
    }
}

fn conditional(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let condition = named_child(node, "cond", cx, "if")?;
    let then = named_child(node, "then", cx, "if")?;

    let otherwise = match node.children_of_kind("else").first() {
        None => None,
        Some(branch) => {
            let inner = one_child(branch, cx, "else")?;
            Some(Box::new(match statement(inner, cx, false)? {
                RustStmt::Semi(expr) => expr,
                other => RustExpr::Block(vec![other]),
            }))
        }
    };

    Ok(RustExpr::If {
        cond: Box::new(expression(one_child(condition, cx, "cond")?, cx)?),
        // An `if` in statement position yields unit, so its branches keep their `return`s. Making
        // a branch yield a value here is what produced `if id == "" { fallback }` — which parses,
        // does not type-check, and is exactly the class of defect the compile proof exists for.
        then: translate(&then.children, cx, TailPosition::No)?,
        otherwise,
    })
}

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
