//! Parameter lists, result types, method signatures, and the inherent `impl` block.

use port_engine_api::Declaration;

use crate::error::TransformError;
use crate::naming::{to_snake_case, visibility};
use crate::resolve::Resolver;
use crate::vocabulary::{
    CHILD_METHOD, CHILD_PARAM, CHILD_RESULT, FLAG_POINTER_RECEIVER, FLAG_VARIADIC,
};

/// Render the `impl` block for a declaration's methods, or `None` when it has none.
pub(crate) fn render_inherent_impl(
    type_name: &str,
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Option<String>, TransformError> {
    let methods = declaration.children_of_kind(CHILD_METHOD);
    if methods.is_empty() {
        return Ok(None);
    }
    let mut body = String::new();
    for method in methods {
        let signature = render_method_signature(method, resolver, Visibility::FromSource)?;
        body.push_str(&format!("    {signature} {{ todo!() }}\n"));
    }
    Ok(Some(format!("impl {type_name} {{\n{body}}}")))
}

/// Whether an item may carry a visibility keyword at all.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Visibility {
    /// Take the visibility the source declared.
    FromSource,
    /// Emit none — the enclosing item already decides it.
    Inherited,
}

pub(crate) fn render_method_signature(
    method: &Declaration,
    resolver: &Resolver<'_>,
    vis: Visibility,
) -> Result<String, TransformError> {
    refuse_variadic(method)?;
    // A pointer receiver is refused rather than rendered. `&self` would drop the mutation the
    // receiver exists to permit, and `&mut self` would claim a mutation the source may never
    // perform; both are a guess about aliasing, which is precisely what
    // docs/programs/k8s-port/census/ownership-escape.md is the analysis for.
    if method.flags.contains(FLAG_POINTER_RECEIVER) {
        return Err(TransformError::Unsupported {
            name: method.name.clone(),
            detail: "pointer receiver: `&self` drops the mutation it permits and `&mut self` \
                     claims one the source may not perform — see \
                     docs/programs/k8s-port/census/ownership-escape.md"
                .to_owned(),
        });
    }
    let params = render_params(method, resolver, Some("&self"))?;
    let results = render_results(method, resolver)?;
    let vis = match vis {
        Visibility::FromSource => visibility(method),
        Visibility::Inherited => "",
    };
    Ok(format!(
        "{vis}fn {}({params}){results}",
        to_snake_case(&method.name)
    ))
}

pub(crate) fn refuse_variadic(declaration: &Declaration) -> Result<(), TransformError> {
    if declaration.flags.contains(FLAG_VARIADIC) {
        return Err(TransformError::Unsupported {
            name: declaration.name.clone(),
            detail: "variadic signature: the target has no variadic parameter, so this needs a \
                     rule that chooses a slice or a builder rather than a default"
                .to_owned(),
        });
    }
    Ok(())
}

pub(crate) fn render_params(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    receiver: Option<&str>,
) -> Result<String, TransformError> {
    let mut rendered: Vec<String> = Vec::new();
    if let Some(receiver) = receiver {
        rendered.push(receiver.to_owned());
    }
    for (index, param) in declaration.children_of_kind(CHILD_PARAM).iter().enumerate() {
        let ty = resolver.resolve(&param.type_ref, &declaration.name)?;
        // An unnamed parameter is legal in the source and illegal in the target, so it is given a
        // positional name. The position is already the parameter's identity here, so nothing is
        // invented that was not already true.
        let name = if param.name.is_empty() || param.name == "_" {
            format!("arg{index}")
        } else {
            to_snake_case(&param.name)
        };
        rendered.push(format!("{name}: {ty}"));
    }
    Ok(rendered.join(", "))
}

pub(crate) fn render_results(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    let results = declaration.children_of_kind(CHILD_RESULT);
    let mut types = Vec::with_capacity(results.len());
    for result in results {
        types.push(resolver.resolve(&result.type_ref, &declaration.name)?);
    }
    match types.len() {
        0 => Ok(String::new()),
        1 => Ok(format!(" -> {}", types[0])),
        // Several results become a tuple. That is the target's own shape for "more than one value
        // out", and it keeps arity and order visible instead of inventing a struct nobody declared.
        _ => Ok(format!(" -> ({})", types.join(", "))),
    }
}
