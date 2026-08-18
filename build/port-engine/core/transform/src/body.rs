//! Function bodies: statements, expressions, and the operator tables.
//!
//! The supported subset is small ON PURPOSE and everything outside it refuses BY NAME. A
//! translator that guesses at a construct it does not understand emits code that compiles and is
//! wrong, which the receipt then certifies as reproducible.

use port_engine_api::Declaration;

use crate::error::TransformError;
use crate::naming::{to_pascal_case, to_screaming_snake, to_snake_case};
use crate::vocabulary::{ATTR_OP, ATTR_REF, ATTR_SOURCE_NODE, ATTR_VALUE};

// understand produces code that compiles and is wrong, which the receipt then certifies as
// reproducible. A refusal is a finding — it says which function, and which construct, and it
// points at the analysis. The census under docs/programs/k8s-port/census/ is where the hard cases
// are worked out before a rule for them is written.

pub(crate) fn render_statements(
    statements: &[Declaration],
    owner: &str,
) -> Result<String, TransformError> {
    let mut out = String::new();
    for statement in statements {
        out.push_str(&render_statement(statement, owner)?);
        out.push(' ');
    }
    Ok(out.trim_end().to_owned())
}

fn render_statement(statement: &Declaration, owner: &str) -> Result<String, TransformError> {
    match statement.kind.as_str() {
        "return" => {
            let values = render_expression_list(&statement.children, owner)?;
            match statement.children.len() {
                0 => Ok("return;".to_owned()),
                1 => Ok(format!("return {values};")),
                // Several results are a tuple on the way out, matching how the signature renders
                // them. Arity and order stay visible rather than becoming an invented struct.
                _ => Ok(format!("return ({values});")),
            }
        }
        "block" => Ok(format!(
            "{{ {} }}",
            render_statements(&statement.children, owner)?
        )),
        "if" => render_if(statement, owner),
        "let" => {
            let value = one_child(statement, owner, "let")?;
            Ok(format!(
                "let {} = {};",
                to_snake_case(&statement.name),
                render_expression(value, owner)?
            ))
        }
        "expr_stmt" => {
            let value = one_child(statement, owner, "expr_stmt")?;
            Ok(format!("{};", render_expression(value, owner)?))
        }
        "unsupported" => Err(unsupported_source(statement, owner)),
        other => Err(TransformError::Unsupported {
            name: owner.to_owned(),
            detail: format!("statement kind `{other}` has no translation"),
        }),
    }
}

fn render_if(statement: &Declaration, owner: &str) -> Result<String, TransformError> {
    let condition = statement
        .children_of_kind("cond")
        .first()
        .copied()
        .ok_or_else(|| TransformError::MissingDatum {
            construction: "if".to_owned(),
            name: owner.to_owned(),
            datum: "cond",
        })?;
    let condition = one_child(condition, owner, "cond")?;

    let then = statement
        .children_of_kind("then")
        .first()
        .copied()
        .ok_or_else(|| TransformError::MissingDatum {
            construction: "if".to_owned(),
            name: owner.to_owned(),
            datum: "then",
        })?;

    let mut out = format!(
        "if {} {{ {} }}",
        render_expression(condition, owner)?,
        render_statements(&then.children, owner)?
    );
    if let Some(otherwise) = statement.children_of_kind("else").first() {
        let branch = one_child(otherwise, owner, "else")?;
        out.push_str(&format!(" else {}", render_statement(branch, owner)?));
    }
    Ok(out)
}

fn render_expression_list(
    expressions: &[Declaration],
    owner: &str,
) -> Result<String, TransformError> {
    let mut rendered = Vec::with_capacity(expressions.len());
    for expression in expressions {
        rendered.push(render_expression(expression, owner)?);
    }
    Ok(rendered.join(", "))
}

fn render_expression(expression: &Declaration, owner: &str) -> Result<String, TransformError> {
    match expression.kind.as_str() {
        "literal" => render_literal(expression, owner),
        "ident" => Ok(render_reference(expression)),
        "paren" => Ok(format!(
            "({})",
            render_expression(one_child(expression, owner, "paren")?, owner)?
        )),
        "binary" => {
            let operator = operator_of(expression, owner)?;
            let rust_operator =
                translate_binary_operator(operator).ok_or_else(|| TransformError::Unsupported {
                    name: owner.to_owned(),
                    detail: format!("binary operator `{operator}` has no direct translation"),
                })?;
            let (left, right) = two_children(expression, owner, "binary")?;
            // Parenthesised unconditionally. The alternative is a precedence table for two
            // languages that do not share one, and a table that is subtly wrong produces
            // arithmetic that compiles and computes something else.
            Ok(format!(
                "({} {} {})",
                render_expression(left, owner)?,
                rust_operator,
                render_expression(right, owner)?
            ))
        }
        "unary" => {
            let operator = operator_of(expression, owner)?;
            let rust_operator =
                translate_unary_operator(operator).ok_or_else(|| TransformError::Unsupported {
                    name: owner.to_owned(),
                    detail: format!("unary operator `{operator}` has no direct translation"),
                })?;
            Ok(format!(
                "({}{})",
                rust_operator,
                render_expression(one_child(expression, owner, "unary")?, owner)?
            ))
        }
        "unsupported" => Err(unsupported_source(expression, owner)),
        other => Err(TransformError::Unsupported {
            name: owner.to_owned(),
            detail: format!("expression kind `{other}` has no translation"),
        }),
    }
}

/// A literal is passed through as SOURCE TEXT, and that is safe only because the emitted tree is
/// parsed by `syn` and compiled. Where the two languages' lexical forms diverge — a rune literal,
/// an imaginary literal — the pass-through produces something the parse rejects, which is the
/// correct outcome and is why no attempt is made to normalise numbers here.
fn render_literal(expression: &Declaration, owner: &str) -> Result<String, TransformError> {
    expression
        .attr(ATTR_VALUE)
        .map(ToOwned::to_owned)
        .ok_or_else(|| TransformError::MissingDatum {
            construction: "literal".to_owned(),
            name: owner.to_owned(),
            datum: ATTR_VALUE,
        })
}

/// Case an identifier by what it refers to. A reference to a constant must render in the target's
/// constant casing or it names nothing at all, so the front end's classification is used rather
/// than a single default applied to every identifier.
fn render_reference(expression: &Declaration) -> String {
    match expression.attr(ATTR_REF) {
        Some("const") => to_screaming_snake(&expression.name),
        Some("type") => to_pascal_case(&expression.name),
        _ => to_snake_case(&expression.name),
    }
}

fn translate_binary_operator(operator: &str) -> Option<&'static str> {
    match operator {
        "+" => Some("+"),
        "-" => Some("-"),
        "*" => Some("*"),
        "/" => Some("/"),
        "%" => Some("%"),
        "==" => Some("=="),
        "!=" => Some("!="),
        "<" => Some("<"),
        "<=" => Some("<="),
        ">" => Some(">"),
        ">=" => Some(">="),
        "&&" => Some("&&"),
        "||" => Some("||"),
        "&" => Some("&"),
        "|" => Some("|"),
        "^" => Some("^"),
        "<<" => Some("<<"),
        ">>" => Some(">>"),
        // `&^` (AND NOT) has no single-operator target form. It is spellable as `& !`, but the
        // operand widths differ between the languages and a silent rewrite of a bit operation is
        // exactly the class of change nobody reviews.
        _ => None,
    }
}

fn translate_unary_operator(operator: &str) -> Option<&'static str> {
    match operator {
        "-" => Some("-"),
        // Logical NOT and bitwise NOT are both `!` in the target, and they are distinguished by
        // operand type rather than by spelling.
        "!" | "^" => Some("!"),
        // `&` and `*` are references and dereferences. Both are aliasing decisions, which
        // docs/programs/k8s-port/census/ownership-escape.md exists to work out.
        _ => None,
    }
}

fn operator_of<'a>(expression: &'a Declaration, owner: &str) -> Result<&'a str, TransformError> {
    expression
        .attr(ATTR_OP)
        .ok_or_else(|| TransformError::MissingDatum {
            construction: expression.kind.clone(),
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
        [left, right] => Ok((left, right)),
        other => Err(TransformError::Unsupported {
            name: owner.to_owned(),
            detail: format!("`{what}` node needs two operands, got {}", other.len()),
        }),
    }
}
