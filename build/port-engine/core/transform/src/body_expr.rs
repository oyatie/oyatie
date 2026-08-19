//! Expressions.
//!
//! Two decisions live here that a naive translator gets silently wrong, and both are about the
//! difference between what the source's syntax MEANS and what the same syntax means in the target:
//! reading a field is a copy in Go and a move in Rust, and a struct literal zero-fills in Go and
//! must name every field in Rust.

use port_engine_api::{Declaration, TypeRef};
use port_engine_rust_ir::RustExpr;

use crate::body::{Body, one_child, two_children, unsupported_source};
use crate::body_ops::{binary_operator, is_receiver, operator_of, reference, unary_operator};
use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::vocabulary::{ATTR_CALLEE, ATTR_VALUE};

/// Where an expression appears: a value is READ, a place is WRITTEN TO.
///
/// The distinction is what keeps the clone rule from producing `self.total.clone() = x`.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Position {
    Value,
    Place,
}

pub(crate) fn expression(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    in_position(node, cx, Position::Value)
}

pub(crate) fn in_position(
    node: &Declaration,
    cx: &Body<'_>,
    position: Position,
) -> Result<RustExpr, TransformError> {
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
                name: cx.owner.to_owned(),
                datum: ATTR_VALUE,
            }),
        "zero" => zero_value(node, cx),
        "ident" if is_receiver(node) => Ok(RustExpr::SelfValue),
        "ident" => Ok(RustExpr::Path(reference(node))),
        // A source-level parenthesis carries no information the tree does not already have, and
        // re-emitting it would fight the precedence the IR computes.
        "paren" => expression(one_child(node, cx, "paren")?, cx),
        "binary" => binary(node, cx),
        "unary" => {
            let spelling = operator_of(node, cx)?;
            let op = unary_operator(spelling).ok_or_else(|| TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: format!("unary operator `{spelling}` has no direct translation"),
            })?;
            Ok(RustExpr::Unary {
                op,
                operand: Box::new(expression(one_child(node, cx, "unary")?, cx)?),
            })
        }
        "selector" => selector(node, cx, position),
        "call" => call(node, cx),
        "index" => {
            let (base, index) = two_children(node, cx, "index")?;
            Ok(RustExpr::Index {
                base: Box::new(expression(base, cx)?),
                index: Box::new(expression(index, cx)?),
            })
        }
        "composite" => composite(node, cx),
        "unsupported" => Err(unsupported_source(node, cx)),
        other => Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!("expression kind `{other}` has no translation"),
        }),
    }
}

fn binary(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let spelling = operator_of(node, cx)?;
    let op = binary_operator(spelling).ok_or_else(|| TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!("binary operator `{spelling}` has no direct translation"),
    })?;
    let (lhs, rhs) = two_children(node, cx, "binary")?;
    Ok(RustExpr::Binary {
        op,
        lhs: Box::new(expression(lhs, cx)?),
        rhs: Box::new(expression(rhs, cx)?),
    })
}

/// A field access, cloned when reading it would MOVE.
///
/// The source copied; `.clone()` is what that costs in the target. In PLACE position there is no
/// read at all, so there is nothing to clone — and cloning there would emit an assignment to a
/// temporary, which parses and silently does nothing.
fn selector(
    node: &Declaration,
    cx: &Body<'_>,
    position: Position,
) -> Result<RustExpr, TransformError> {
    let field = RustExpr::Field {
        base: Box::new(expression(one_child(node, cx, "selector")?, cx)?),
        name: to_snake_case(&node.name),
    };
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
fn moves_on_read(type_ref: &TypeRef, cx: &Body<'_>) -> bool {
    if type_ref.is_empty() {
        return false;
    }
    !cx.resolver.copies(type_ref)
}

/// A struct literal, with every field named.
///
/// Go zero-fills the fields a literal omits; the target has no such rule and rejects an incomplete
/// literal outright. The front end therefore emits one entry per DECLARED field, and the omitted
/// ones arrive as `zero` nodes carrying the field's type — so the target's zero is a pack answer
/// rather than something inferred from a spelling here.
fn composite(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
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

/// The target's zero for a source type the literal left out.
///
/// Refuses BY NAME rather than reaching for `Default::default()`. That would compile for the types
/// the corpus has and would silently be a different program for the ones it does not: a type whose
/// `Default` impl is not its zero value, or one that has no `Default` at all and only fails much
/// later, in the emitted crate, with the source declaration nowhere in the message.
fn zero_value(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
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

/// A call, which is a method call when its callee is a field access.
///
/// The source spells both as one form and distinguishes them by what the callee resolves to; the
/// target spells them differently. A field access in callee position is a method call — a plain
/// field holding a function is a shape the corpus does not have and that refuses rather than being
/// silently rewritten into a method.
fn call(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let callee = node
        .children
        .first()
        .ok_or_else(|| TransformError::MissingDatum {
            construction: "call".to_owned(),
            name: cx.owner.to_owned(),
            datum: "callee",
        })?;
    let args = node.children[1..]
        .iter()
        .map(|arg| expression(arg, cx))
        .collect::<Result<Vec<_>, _>>()?;

    // The pack answers for the callee FIRST, by identity. A call it answers for is one the target
    // has no name of its own for — a builtin, or something from a standard library that does not
    // come along — and emitting the source's spelling would name nothing.
    if let Some(rendered) = mapped_call(node, &args, cx)? {
        return Ok(rendered);
    }

    if callee.kind == "selector" {
        return Ok(RustExpr::MethodCall {
            // The receiver of a method call is a PLACE, not a value: `x.m()` borrows `x` rather
            // than reading it, so cloning here would call the method on a temporary.
            receiver: Box::new(in_position(
                one_child(callee, cx, "selector")?,
                cx,
                Position::Place,
            )?),
            method: to_snake_case(&callee.name),
            args,
        });
    }
    Ok(RustExpr::Call {
        callee: Box::new(expression(callee, cx)?),
        args,
    })
}

/// A call the pack answers for by the callee's IDENTITY, rendered from its declared template.
///
/// Arity is checked rather than assumed: a template that expects an argument the call does not have
/// would leave its own placeholder in the output, which parses as nothing and would be discovered
/// far from its cause.
fn mapped_call(
    node: &Declaration,
    args: &[RustExpr],
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let Some(identity) = node.attr(ATTR_CALLEE) else {
        return Ok(None);
    };
    let Some(template) = cx.resolver.function_map.get(identity) else {
        return Ok(None);
    };

    let mut rendered = template.clone();
    for (index, arg) in args.iter().enumerate() {
        let operand = render_operand(arg).ok_or_else(|| TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "an argument to `{identity}` is a compound expression, and the pack answers for                  that call with a TEXT template — substituting one would need parentheses the                  template cannot ask for"
            ),
        })?;
        rendered = rendered.replace(&format!("{{{index}}}"), &operand);
    }
    if rendered.contains('{') {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "the pack's template for `{identity}` expects more arguments than the call has"
            ),
        });
    }
    Ok(Some(RustExpr::Literal(rendered)))
}

/// An argument, as target text for a template to interpolate.
///
/// Only the forms whose text is unambiguous are admitted. A template is textual substitution, and
/// substituting a compound expression into one would need parentheses this cannot see the need for —
/// so anything else refuses rather than producing text that reassociates.
fn render_operand(arg: &RustExpr) -> Option<String> {
    match arg {
        RustExpr::Literal(text) | RustExpr::Path(text) => Some(text.clone()),
        _ => None,
    }
}
