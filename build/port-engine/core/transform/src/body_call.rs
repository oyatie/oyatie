//! A CALL, and the arguments it hands over.
//!
//! Separated from the expression walk because an argument is the one expression translated for a
//! DESTINATION rather than on its own terms. The signature table holds every parameter the engine
//! has translated, so `f(&x)` can take the construction that parameter's disposition declares and
//! a bare string literal can be owned when the parameter holds an owned string — both the same
//! decision the parameter position already made, applied at the other end.
//!
//! What a call resolves to was already the trickiest part of the front end: the source spells
//! `f(x)`, `value.Method()` and `package.Function()` with one production, and deciding by shape is
//! how a cross-package call became a method call on a binding that did not exist.

use port_engine_api::{Declaration, FunctionMapping, PointerConstruction};
use port_engine_rust_ir::RustExpr;

use crate::body::{Body};
use crate::body_parts::{one_child};
use crate::body_expr::{Position, expression, in_position};
use crate::body_ops::{operator_of, own_string_for, reference};
use crate::error::TransformError;
use crate::naming::{module_path, to_snake_case};
use crate::vocabulary::{
    ARGUMENT_STRING_LITERAL, ATTR_CALLEE, ATTR_CALLEE_KIND, ATTR_LIT_KIND, CALLEE_KIND_METHOD,
    KIND_LITERAL, KIND_UNARY, LIT_KIND_STRING, OPERATOR_ADDRESS_OF,
};

/// One argument, translated for the parameter it reaches.
///
/// Two conversions become possible that an isolated expression cannot make, and both are the SAME
/// decision the parameter position already made, applied at the other end:
///
///   * A pointer operand takes the construction its disposition declares — found by the id the
///     parameter recorded, never by matching the spelling that decision produced.
///   * A bare string literal is owned when the parameter holds the owned string target.
///
/// A destination the table cannot give is NOT "no conversion needed": the argument is translated
/// as it stands, and a pointer operand refuses by name saying what was missing.
fn argument(
    node: &Declaration,
    callee: &str,
    index: usize,
    cx: &Body<'_>,
) -> Result<RustExpr, TransformError> {
    let target = cx.resolver.signatures.param(callee, index);

    if node.kind == KIND_UNARY && operator_of(node, cx)? == OPERATOR_ADDRESS_OF {
        let operand = expression(one_child(node, cx, KIND_UNARY)?, cx)?;
        let construction = target
            .and_then(|target| target.disposition.as_deref())
            .and_then(|id| cx.resolver.ownership.construction_for(id));
        let Some(construction) = construction else {
            return Err(TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: address_of_refusal(callee, index, target.is_some()),
            });
        };
        return Ok(constructed(construction, operand));
    }

    let expr = expression(node, cx)?;
    let Some(target) = target else {
        return Ok(expr);
    };
    Ok(own_string_for(expr, &target.ty.spelling(), cx))
}

/// Build the argument the construction describes.
///
/// `Wrap` applies its paths INNERMOST FIRST, which is the order the value passes through them and
/// the order they are declared.
pub(crate) fn constructed(construction: &PointerConstruction, operand: RustExpr) -> RustExpr {
    match construction {
        PointerConstruction::Borrow { mutable, .. } => RustExpr::Reference {
            mutable: *mutable,
            inner: Box::new(operand),
        },
        PointerConstruction::Wrap { paths, .. } => paths.iter().fold(operand, |inner, path| {
            RustExpr::Call {
                callee: Box::new(RustExpr::Path(path.clone())),
                args: vec![inner],
            }
        }),
    }
}

/// Why a `&x` argument could not be given a form, in terms of what was missing.
fn address_of_refusal(callee: &str, index: usize, has_target: bool) -> String {
    if callee.is_empty() {
        return format!(
            "unary `&` is argument {index} of a METHOD call, whose signature the table does not \
             hold: a method's key is its receiver type rather than a path"
        );
    }
    if !has_target {
        return format!(
            "unary `&` is argument {index} of `{callee}`, whose signature is not in the snapshot — \
             it is foreign, or the engine could not translate it"
        );
    }
    format!(
        "unary `&` is argument {index} of `{callee}`, whose parameter is not a pointer, so no \
         disposition decided how an argument reaches it"
    )
}

/// A call, which is a method call when its callee is a field access.
///
/// The source spells both as one form and distinguishes them by what the callee resolves to; the
/// target spells them differently. A field access in callee position is a method call — a plain
/// field holding a function is a shape the corpus does not have and that refuses rather than being
/// silently rewritten into a method.
pub(crate) fn call(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let callee = node
        .children
        .first()
        .ok_or_else(|| TransformError::MissingDatum {
            construction: "call".to_owned(),
            name: cx.owner.to_owned(),
            datum: "callee",
        })?;
    // Each argument is translated FOR ITS DESTINATION. The callee identity is what the signature
    // table is keyed by, and a call carrying none — a method — gets no destinations at all, which
    // the argument path reads as "cannot say" rather than as "no conversion needed".
    let callee_id = node.attr(ATTR_CALLEE).unwrap_or_default();
    // A VARIADIC callee needs its trailing arguments collected into a sequence, and which of the
    // target's sequence forms that is has not been decided — nor what `f(xs...)` does when the
    // caller forwards a slice it already has. Refused here rather than at the declaration: the
    // signature is an ordinary slice and ports fine, and only a CALL needs the answer.
    if cx.resolver.signatures.is_variadic(callee_id) {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "`{callee_id}` is variadic, and the target has no variadic call: the trailing \
                 arguments need collecting into a sequence, and the pack declares neither which \
                 sequence form nor what a forwarded slice becomes"
            ),
        });
    }
    let args = node.children[1..]
        .iter()
        .enumerate()
        .map(|(index, arg)| argument(arg, callee_id, index, cx))
        .collect::<Result<Vec<_>, _>>()?;

    // The pack answers for the callee FIRST, by identity. A call it answers for is one the target
    // has no name of its own for — a builtin, or something from a standard library that does not
    // come along — and emitting the source's spelling would name nothing.
    if let Some(rendered) = mapped_call(node, &args, cx)? {
        return Ok(rendered);
    }

    // A call through a RECEIVER, as the type-checker saw it — not as the syntax looked. The
    // source spells `value.Method()` and `package.Function()` the same way, and deciding by shape
    // emitted a method call on a package name.
    if node.attr(ATTR_CALLEE_KIND) == Some(CALLEE_KIND_METHOD) {
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

    // A free function, named by the path its identity resolves to. A local one keeps its bare
    // name; one from another unit is reached through that unit's emitted module, the same way a
    // type from another unit is.
    let path = cx
        .resolver
        .function_path(node.attr(ATTR_CALLEE), cx.owner)?;
    Ok(RustExpr::Call {
        callee: Box::new(RustExpr::Path(path)),
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
    let Some(mapping) = cx.resolver.function_map.get(identity) else {
        return Ok(None);
    };
    refuse_wrong_argument_shape(node, identity, mapping, cx)?;

    let mut rendered = mapping.form.clone();
    for (index, arg) in args.iter().enumerate() {
        let operand = render_operand(arg).ok_or_else(|| TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "an argument to `{identity}` is a compound expression, and the pack answers for \
                 that call with a TEXT template — substituting one would need parentheses the \
                 template cannot ask for"
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
    Ok(Some(structured(&rendered, &mapping.form)))
}

/// A rendered mapping as an EXPRESSION, with a trailing conversion read as one.
///
/// The pack's forms are text templates, and one of them ends in a conversion: `{0}.len() as i64`.
/// Handed to the IR as a flat literal, nothing downstream can see that the outermost thing is a
/// cast — and `xs.len() as i64` with a method on it renders as `xs.len() as i64.wrapping_sub(1)`,
/// which the target REJECTS outright. Two whole packages failed to render on exactly that, and the
/// hermetic corpus never had a cast with a method on it.
///
/// Read from the FORM the pack declares rather than by sniffing the rendered text, so a template
/// with no trailing conversion is left exactly alone and a pack that changes one changes this.
fn structured(rendered: &str, form: &str) -> RustExpr {
    let Some((_, target)) = form.rsplit_once(" as ") else {
        return RustExpr::Literal(rendered.to_owned());
    };
    match rendered
        .strip_suffix(&format!(" as {target}"))
        .filter(|inner| !inner.is_empty())
    {
        Some(inner) => RustExpr::Cast {
            expr: Box::new(RustExpr::Literal(inner.to_owned())),
            ty: port_engine_rust_ir::RustType::path(target),
        },
        None => RustExpr::Literal(rendered.to_owned()),
    }
}

/// Refuse a mapped call whose argument is not the shape the mapping declares.
///
/// Some mappings hold for any argument and some do not. `panic` is the case this exists for: Go's
/// `panic(v)` aborts carrying `v` and Rust's `panic!` aborts carrying a formatted string, so where
/// `v` is a STRING LITERAL the two are the same abort with the same message and the same payload
/// type — and where it is an error or an arbitrary value the payload TYPE is lost, which a caller
/// that recovers and type-asserts on it would see as a different program.
///
/// The condition is pack data and its vocabulary is CLOSED: a shape the engine has never heard of
/// refuses rather than being read as "no condition", because a condition nobody checks is a
/// condition that is not there.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the call, the shape required, and the shape found.
fn refuse_wrong_argument_shape(
    node: &Declaration,
    identity: &str,
    mapping: &FunctionMapping,
    cx: &Body<'_>,
) -> Result<(), TransformError> {
    let Some(required) = mapping.requires_argument.as_deref() else {
        return Ok(());
    };
    let argument = node.children.get(1);
    let holds = match required {
        ARGUMENT_STRING_LITERAL => argument.is_some_and(|arg| {
            arg.kind == KIND_LITERAL && arg.attr(ATTR_LIT_KIND) == Some(LIT_KIND_STRING)
        }),
        unknown => {
            return Err(TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: format!(
                    "the pack requires argument shape `{unknown}` for `{identity}`, which is not \
                     a shape this engine knows how to check"
                ),
            });
        }
    };
    if holds {
        return Ok(());
    }
    Err(TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!(
            "`{identity}` is answered by the pack only for a `{required}` argument, and this call \
             passes `{}` — {}",
            argument.map_or("nothing", |arg| arg.kind.as_str()),
            mapping.reason
        ),
    })
}

/// An argument, as target text for a template to interpolate.
///
/// Only the forms whose text is unambiguous are admitted. A template is textual substitution, and
/// substituting a compound expression into one would need parentheses this cannot see the need for —
/// so anything else refuses rather than producing text that reassociates.
pub(crate) fn render_operand(arg: &RustExpr) -> Option<String> {
    match arg {
        RustExpr::Literal(text) | RustExpr::Path(text) => Some(text.clone()),
        _ => None,
    }
}
