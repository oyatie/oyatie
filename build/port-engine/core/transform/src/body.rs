//! Function bodies: source statements and expressions into IR nodes.
//!
//! The supported subset is small ON PURPOSE and everything outside it refuses BY NAME. A
//! translator that guesses at a construct it does not understand emits code that compiles and is
//! wrong, which the receipt then certifies as reproducible.
//!
//! Nothing here builds text. Operator precedence is the IR's problem now, which is why the
//! operator tables below map to typed operators rather than to spellings: a spelling has to be
//! parenthesised defensively, a typed operator carries its own binding power.

use port_engine_api::Declaration;
use port_engine_rust_ir::{BinaryOp, RustExpr, RustStmt, UnaryOp};

use crate::error::TransformError;
use crate::naming::{to_pascal_case, to_screaming_snake, to_snake_case};
use crate::vocabulary::{ATTR_OP, ATTR_REF, ATTR_SOURCE_NODE, ATTR_VALUE};

/// Translate a function body's statements.
///
/// A trailing `return` becomes a TAIL EXPRESSION. That is a target-language idiom rather than a
/// change of meaning — `return x;` as the last statement of a function and `x` are the same
/// program — and it is owned here for the same reason identifier casing is: this face renders
/// Rust, so Rust's conventions are its business.
pub(crate) fn statements(
    nodes: &[Declaration],
    owner: &str,
) -> Result<Vec<RustStmt>, TransformError> {
    translate(nodes, owner, TailPosition::Yes)
}

/// Whether the last statement of this sequence is in TAIL position — the position whose value is
/// the enclosing block's value.
#[derive(Clone, Copy, Eq, PartialEq)]
enum TailPosition {
    Yes,
    No,
}

fn translate(
    nodes: &[Declaration],
    owner: &str,
    tail: TailPosition,
) -> Result<Vec<RustStmt>, TransformError> {
    let mut out = Vec::with_capacity(nodes.len());
    for (index, node) in nodes.iter().enumerate() {
        let is_tail = tail == TailPosition::Yes && index + 1 == nodes.len();
        out.push(statement(node, owner, is_tail)?);
    }
    Ok(out)
}

fn statement(node: &Declaration, owner: &str, is_last: bool) -> Result<RustStmt, TransformError> {
    match node.kind.as_str() {
        "return" => {
            let values = node
                .children
                .iter()
                .map(|child| expression(child, owner))
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
        "block" => Ok(RustStmt::Semi(RustExpr::Block(translate(
            &node.children,
            owner,
            TailPosition::No,
        )?))),
        "if" => Ok(RustStmt::Semi(conditional(node, owner)?)),
        "let" => Ok(RustStmt::Let {
            name: to_snake_case(&node.name),
            value: expression(one_child(node, owner, "let")?, owner)?,
        }),
        "expr_stmt" => Ok(RustStmt::Semi(expression(
            one_child(node, owner, "expr_stmt")?,
            owner,
        )?)),
        "unsupported" => Err(unsupported_source(node, owner)),
        other => Err(TransformError::Unsupported {
            name: owner.to_owned(),
            detail: format!("statement kind `{other}` has no translation"),
        }),
    }
}

fn conditional(node: &Declaration, owner: &str) -> Result<RustExpr, TransformError> {
    let condition = node
        .children_of_kind("cond")
        .first()
        .copied()
        .ok_or_else(|| TransformError::MissingDatum {
            construction: "if".to_owned(),
            name: owner.to_owned(),
            datum: "cond",
        })?;
    let then = node
        .children_of_kind("then")
        .first()
        .copied()
        .ok_or_else(|| TransformError::MissingDatum {
            construction: "if".to_owned(),
            name: owner.to_owned(),
            datum: "then",
        })?;

    let otherwise = match node.children_of_kind("else").first() {
        None => None,
        Some(branch) => {
            let inner = one_child(branch, owner, "else")?;
            Some(Box::new(match statement(inner, owner, false)? {
                RustStmt::Semi(expr) => expr,
                other => RustExpr::Block(vec![other]),
            }))
        }
    };

    Ok(RustExpr::If {
        cond: Box::new(expression(one_child(condition, owner, "cond")?, owner)?),
        // An `if` in statement position yields unit, so its branches keep their `return`s. Making
        // a branch yield a value here is what produced `if id == "" { fallback }` — which parses,
        // does not type-check, and is exactly the class of defect the compile proof exists for.
        then: translate(&then.children, owner, TailPosition::No)?,
        otherwise,
    })
}

fn expression(node: &Declaration, owner: &str) -> Result<RustExpr, TransformError> {
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
                name: owner.to_owned(),
                datum: ATTR_VALUE,
            }),
        "ident" => Ok(RustExpr::Path(reference(node))),
        // A source-level parenthesis carries no information the tree does not already have, and
        // re-emitting it would fight the precedence the IR computes.
        "paren" => expression(one_child(node, owner, "paren")?, owner),
        "binary" => {
            let spelling = operator_of(node, owner)?;
            let op = binary_operator(spelling).ok_or_else(|| TransformError::Unsupported {
                name: owner.to_owned(),
                detail: format!("binary operator `{spelling}` has no direct translation"),
            })?;
            let (lhs, rhs) = two_children(node, owner, "binary")?;
            Ok(RustExpr::Binary {
                op,
                lhs: Box::new(expression(lhs, owner)?),
                rhs: Box::new(expression(rhs, owner)?),
            })
        }
        "unary" => {
            let spelling = operator_of(node, owner)?;
            let op = unary_operator(spelling).ok_or_else(|| TransformError::Unsupported {
                name: owner.to_owned(),
                detail: format!("unary operator `{spelling}` has no direct translation"),
            })?;
            Ok(RustExpr::Unary {
                op,
                operand: Box::new(expression(one_child(node, owner, "unary")?, owner)?),
            })
        }
        "unsupported" => Err(unsupported_source(node, owner)),
        other => Err(TransformError::Unsupported {
            name: owner.to_owned(),
            detail: format!("expression kind `{other}` has no translation"),
        }),
    }
}

/// Case an identifier by what it REFERS to.
///
/// A reference to a constant must render in the target's constant casing or it names nothing at
/// all, so the front end's classification is used rather than one default applied to everything.
fn reference(node: &Declaration) -> String {
    match node.attr(ATTR_REF) {
        Some("const") => to_screaming_snake(&node.name),
        Some("type") => to_pascal_case(&node.name),
        _ => to_snake_case(&node.name),
    }
}

fn binary_operator(spelling: &str) -> Option<BinaryOp> {
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

fn unary_operator(spelling: &str) -> Option<UnaryOp> {
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

fn operator_of<'a>(node: &'a Declaration, owner: &str) -> Result<&'a str, TransformError> {
    node.attr(ATTR_OP)
        .ok_or_else(|| TransformError::MissingDatum {
            construction: node.kind.clone(),
            name: owner.to_owned(),
            datum: ATTR_OP,
        })
}

fn unsupported_source(node: &Declaration, owner: &str) -> TransformError {
    let source_node = node
        .attr(ATTR_SOURCE_NODE)
        .unwrap_or("an unnamed construct");
    TransformError::Unsupported {
        name: owner.to_owned(),
        detail: format!(
            "source construct `{source_node}` has no translation yet — a rule for it belongs in \
             the pack, and the analysis in docs/programs/k8s-port/census/"
        ),
    }
}

fn one_child<'a>(
    node: &'a Declaration,
    owner: &str,
    what: &str,
) -> Result<&'a Declaration, TransformError> {
    node.children
        .first()
        .ok_or_else(|| TransformError::Unsupported {
            name: owner.to_owned(),
            detail: format!("`{what}` node carries no operand"),
        })
}

fn two_children<'a>(
    node: &'a Declaration,
    owner: &str,
    what: &str,
) -> Result<(&'a Declaration, &'a Declaration), TransformError> {
    match node.children.as_slice() {
        [lhs, rhs] => Ok((lhs, rhs)),
        other => Err(TransformError::Unsupported {
            name: owner.to_owned(),
            detail: format!("`{what}` node needs two operands, got {}", other.len()),
        }),
    }
}
