//! Parameters, results, and method signatures — as IR nodes.

use port_engine_api::Declaration;
use port_engine_rust_ir::{Receiver, RustFn, RustParam, RustType, Visibility};

use crate::error::TransformError;
use crate::naming::{to_snake_case, visibility};
use crate::ownership::{binds_by_pointer, facts_of, parameter_target, receiver_for};
use crate::resolve::Resolver;
use crate::vocabulary::{CHILD_BODY, CHILD_METHOD, CHILD_PARAM, CHILD_RESULT, FLAG_VARIADIC};

/// Whether a method's body is translated, stubbed, or absent.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Body {
    /// Emit `todo!()`.
    Stub,
    /// Translate the source body, refusing anything outside the supported subset.
    Translate,
    /// Emit no body at all — a trait item is a signature.
    None,
}

/// The methods declared on a type, as inherent `impl` items.
pub(crate) fn inherent_methods(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    body: Body,
) -> Result<Vec<RustFn>, TransformError> {
    declaration
        .children_of_kind(CHILD_METHOD)
        .into_iter()
        .map(|method| {
            method_signature(
                method,
                resolver,
                Visibility::Public,
                body,
                &declaration.name,
            )
        })
        .collect()
}

/// Parse a pack-declared receiver form into the IR's receiver.
fn parse_receiver(form: &str, site: &str) -> Result<Receiver, TransformError> {
    match form {
        "&self" => Ok(Receiver::Shared),
        "&mut self" => Ok(Receiver::Exclusive),
        "self" => Ok(Receiver::Owned),
        other => Err(TransformError::Ownership {
            detail: format!("`{other}` is not a receiver form the target has, at `{site}`"),
        }),
    }
}

/// The methods a trait requires, as bodiless signatures.
///
/// Visibility is [`Visibility::Inherited`] and not a choice: a trait item is as public as its
/// trait and may not say so. The previous renderer concatenated the source's `pub ` prefix into
/// the trait body, which `syn` parses and `rustc` rejects — and which a golden over the parsed
/// output would have frozen in place.
///
/// The RECEIVER is a choice, and the pack makes it. It cannot be recovered from the source: an
/// interface says nothing about how an implementation binds its receiver, and the implementations
/// are not all in view. `&self` was being emitted for every method, which is why the fixture's
/// `Rename(next string)` — a mutator — became a signature no mutating implementation can satisfy.
pub(crate) fn trait_methods(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    receiver: Receiver,
) -> Result<Vec<RustFn>, TransformError> {
    declaration
        .children_of_kind(CHILD_METHOD)
        .into_iter()
        .map(|method| {
            let mut rendered = method_signature(
                method,
                resolver,
                Visibility::Inherited,
                Body::None,
                &declaration.name,
            )?;
            // A trait method's receiver is the pack's DECLARED mode, not an inference: an
            // interface says nothing about how an implementation binds its receiver, and the
            // implementations are not all in view.
            rendered.receiver = Some(receiver);
            Ok(rendered)
        })
        .collect()
}

/// Map the pack's declared mode onto the IR's receiver.
pub(crate) fn declared_receiver(mode: &str, owner: &str) -> Result<Receiver, TransformError> {
    match mode {
        "shared" => Ok(Receiver::Shared),
        "exclusive" => Ok(Receiver::Exclusive),
        "owned" => Ok(Receiver::Owned),
        other => Err(TransformError::Unsupported {
            name: owner.to_owned(),
            detail: format!("`{other}` is not a receiver mode the target has"),
        }),
    }
}

fn method_signature(
    method: &Declaration,
    resolver: &Resolver<'_>,
    vis: Visibility,
    body: Body,
    owner: &str,
) -> Result<RustFn, TransformError> {
    refuse_variadic(method)?;

    // A pointer receiver used to be refused outright, because `&self` drops the mutation it
    // permits and `&mut self` claims one the source may not perform, and nothing reported which.
    // The front end now reports it, so the guess became a decision — made by the pack over
    // observed facts, and recorded per site.
    //
    // A VALUE receiver is not an aliasing question — the source already copied, so there are no
    // aliases to reason about — but it is not `self` either. Go's value receiver COPIES and the
    // caller's value survives the call; Rust's `self` CONSUMES, so calling the method twice would
    // stop compiling. `&self` is the form with the source's permissions.
    //
    // A value receiver the body mutates is a different shape again: the mutation is on the copy
    // and needs a local binding, which is body-translation work rather than a receiver form. It
    // refuses instead of silently picking one of the two wrong answers.
    let receiver = if binds_by_pointer(method) {
        let site = format!("{owner}::{}", method.name);
        parse_receiver(&receiver_for(method, &site, resolver.ownership)?, &site)?
    } else if facts_of(method).mutated {
        return Err(TransformError::Unsupported {
            name: method.name.clone(),
            detail: "value receiver mutated in the body: the source mutates its own COPY, which \
                     needs a local binding rather than a receiver form — `self` would consume the \
                     caller's value and `&mut self` would claim a mutation the caller can see"
                .to_owned(),
        });
    } else {
        Receiver::Shared
    };

    Ok(RustFn {
        docs: Vec::new(),
        vis,
        name: to_snake_case(&method.name),
        receiver: Some(receiver),
        params: params(method, resolver, owner)?,
        ret: results(method, resolver)?,
        body: match body {
            Body::Stub => Some(vec![port_engine_rust_ir::RustStmt::Tail(
                port_engine_rust_ir::RustExpr::Todo,
            )]),
            Body::Translate => {
                let source = method
                    .children_of_kind(CHILD_BODY)
                    .first()
                    .copied()
                    .ok_or_else(|| TransformError::MissingDatum {
                        construction: "rust_struct_body".to_owned(),
                        name: method.name.clone(),
                        datum: "body",
                    })?;
                Some(crate::body::statements(
                    &source.children,
                    &method.name,
                    resolver,
                )?)
            }
            Body::None => None,
        },
    })
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

pub(crate) fn params(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    owner: &str,
) -> Result<Vec<RustParam>, TransformError> {
    declaration
        .children_of_kind(CHILD_PARAM)
        .into_iter()
        .enumerate()
        .map(|(index, param)| {
            // A POINTER parameter is an ownership question and gets a decision; anything else is
            // just a type. The split is deliberate: a pointer inside a field or a result has no
            // call site to borrow across, so it stays a plain type-map answer.
            let ty = if param.type_ref.kind == "pointer" {
                let pointee =
                    param
                        .type_ref
                        .args
                        .first()
                        .ok_or_else(|| TransformError::Ownership {
                            detail: format!(
                                "pointer parameter `{}` of `{owner}::{}` has no pointee",
                                param.name, declaration.name
                            ),
                        })?;
                let resolved = resolver.resolve(pointee, &declaration.name)?;
                let site = format!("{owner}::{}({})", declaration.name, param.name);
                RustType::path(parameter_target(
                    param,
                    &resolved.spelling(),
                    &site,
                    resolver.ownership,
                )?)
            } else {
                resolver.resolve(&param.type_ref, &declaration.name)?
            };
            // An unnamed parameter is legal in the source and illegal in the target, so it is
            // given a positional name. The position is already its identity, so nothing is
            // invented that was not already true.
            let name = if param.name.is_empty() || param.name == "_" {
                format!("arg{index}")
            } else {
                to_snake_case(&param.name)
            };
            Ok(RustParam { name, ty })
        })
        .collect()
}

pub(crate) fn results(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Option<RustType>, TransformError> {
    let results = declaration.children_of_kind(CHILD_RESULT);
    let mut types = Vec::with_capacity(results.len());
    for result in results {
        types.push(resolver.resolve(&result.type_ref, &declaration.name)?);
    }
    match types.len() {
        0 => Ok(None),
        // Several results become a tuple. That is the target's own shape for "more than one value
        // out", and it keeps arity and order visible instead of inventing a struct nobody declared.
        1 => Ok(types.pop()),
        _ => Ok(Some(RustType::Tuple(types))),
    }
}

/// Visibility for a declaration, as a value the IR places rather than a prefix a string carries.
pub(crate) fn declared_visibility(declaration: &Declaration) -> Visibility {
    visibility(declaration)
}
