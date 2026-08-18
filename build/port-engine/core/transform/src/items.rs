//! The declaration-level construction builders: what each construction emits.

use port_engine_api::Declaration;

use crate::body::render_statements;
use crate::error::TransformError;
use crate::naming::{to_pascal_case, to_screaming_snake, to_snake_case, visibility};
use crate::resolve::Resolver;
use crate::signature::{
    Visibility, refuse_variadic, render_inherent_impl, render_method_signature, render_params,
    render_results,
};
use crate::vocabulary::{
    ATTR_VALUE, CHILD_BODY, CHILD_FIELD, CHILD_METHOD, CONSTRUCTION_RUST_CONST,
    CONSTRUCTION_RUST_FN, CONSTRUCTION_RUST_FN_BODY, CONSTRUCTION_RUST_NEWTYPE,
    CONSTRUCTION_RUST_STRUCT, CONSTRUCTION_RUST_TRAIT, CONSTRUCTION_RUST_TYPE_ALIAS,
};

pub(crate) fn declaration_source(
    construction: &str,
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    match construction {
        CONSTRUCTION_RUST_CONST => build_const(declaration, resolver),
        CONSTRUCTION_RUST_TYPE_ALIAS => build_type_alias(declaration, resolver),
        CONSTRUCTION_RUST_NEWTYPE => build_newtype(declaration, resolver),
        CONSTRUCTION_RUST_STRUCT => build_struct(declaration, resolver),
        CONSTRUCTION_RUST_TRAIT => build_trait(declaration, resolver),
        CONSTRUCTION_RUST_FN => build_fn(declaration, resolver, BodyMode::Stub),
        CONSTRUCTION_RUST_FN_BODY => build_fn(declaration, resolver, BodyMode::Translate),
        other => Err(TransformError::UnknownConstruction {
            rule: String::new(),
            construction: other.to_owned(),
        }),
    }
}

fn build_const(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    let value = declaration
        .attr(ATTR_VALUE)
        .ok_or_else(|| TransformError::MissingDatum {
            construction: CONSTRUCTION_RUST_CONST.to_owned(),
            name: declaration.name.clone(),
            datum: ATTR_VALUE,
        })?;
    let ty = resolver.resolve(&declaration.type_ref, &declaration.name)?;
    Ok(format!(
        "{}const {}: {} = {};",
        visibility(declaration),
        to_screaming_snake(&declaration.name),
        ty,
        value
    ))
}

fn build_type_alias(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    let ty = resolver.resolve(&declaration.type_ref, &declaration.name)?;
    Ok(format!(
        "{}type {} = {};",
        visibility(declaration),
        to_pascal_case(&declaration.name),
        ty
    ))
}

/// A defined type over an underlying type becomes a newtype, never an alias.
///
/// The distinction is the whole point of the source construct: a defined type is a DISTINCT type
/// that does not interchange with its underlying one, and rendering it as an alias would erase
/// exactly the property it was declared for. A newtype keeps the distinction in the target's own
/// type system.
fn build_newtype(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    let ty = resolver.resolve(&declaration.type_ref, &declaration.name)?;
    let name = to_pascal_case(&declaration.name);
    let vis = visibility(declaration);
    let mut out = format!("{vis}struct {name}({vis}{ty});");
    if let Some(methods) = render_inherent_impl(&name, declaration, resolver)? {
        out.push('\n');
        out.push_str(&methods);
    }
    Ok(out)
}

fn build_struct(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    let name = to_pascal_case(&declaration.name);
    let vis = visibility(declaration);

    let fields = declaration.children_of_kind(CHILD_FIELD);
    let mut body = String::new();
    for field in fields {
        let ty = resolver.resolve(&field.type_ref, &field.name)?;
        body.push_str(&format!(
            "    {}{}: {},\n",
            visibility(field),
            to_snake_case(&field.name),
            ty
        ));
    }

    let mut out = if body.is_empty() {
        format!("{vis}struct {name};")
    } else {
        format!("{vis}struct {name} {{\n{body}}}")
    };
    if let Some(methods) = render_inherent_impl(&name, declaration, resolver)? {
        out.push('\n');
        out.push_str(&methods);
    }
    Ok(out)
}

fn build_trait(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    let name = to_pascal_case(&declaration.name);
    let mut body = String::new();
    for method in declaration.children_of_kind(CHILD_METHOD) {
        // A trait item carries NO visibility: it is as public as the trait itself, and `pub` on
        // one is not valid Rust. syn parses it anyway, which is exactly why the emitted tree is
        // compiled rather than only parsed.
        let signature = render_method_signature(method, resolver, Visibility::Inherited)?;
        body.push_str(&format!("    {signature};\n"));
    }
    Ok(format!(
        "{}trait {} {{\n{}}}",
        visibility(declaration),
        name,
        body
    ))
}

/// Whether a function's body is translated or stubbed.
#[derive(Clone, Copy, Eq, PartialEq)]
enum BodyMode {
    /// Emit `todo!()`; the model's body, if any, is not read.
    Stub,
    /// Translate the model's body, refusing anything outside the supported subset.
    Translate,
}

fn build_fn(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    mode: BodyMode,
) -> Result<String, TransformError> {
    if !declaration.children_of_kind(CHILD_FIELD).is_empty() {
        return Err(TransformError::ConstructionKindMismatch {
            construction: CONSTRUCTION_RUST_FN.to_owned(),
            kind: declaration.kind.clone(),
            name: declaration.name.clone(),
        });
    }
    refuse_variadic(declaration)?;
    let params = render_params(declaration, resolver, None)?;
    let results = render_results(declaration, resolver)?;
    let body = match mode {
        BodyMode::Stub => " todo!() ".to_owned(),
        BodyMode::Translate => {
            let body = declaration
                .children_of_kind(CHILD_BODY)
                .first()
                .copied()
                .ok_or_else(|| TransformError::MissingDatum {
                    construction: CONSTRUCTION_RUST_FN_BODY.to_owned(),
                    name: declaration.name.clone(),
                    datum: "body",
                })?;
            format!(
                " {} ",
                render_statements(&body.children, &declaration.name)?
            )
        }
    };
    Ok(format!(
        "{}fn {}({}){} {{{}}}",
        visibility(declaration),
        to_snake_case(&declaration.name),
        params,
        results,
        body
    ))
}
