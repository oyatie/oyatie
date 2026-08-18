//! Parameters, results, and method signatures — as IR nodes.

use port_engine_api::Declaration;
use port_engine_rust_ir::{Receiver, RustFn, RustParam, RustType, Visibility};

use crate::error::TransformError;
use crate::naming::{to_snake_case, visibility};
use crate::resolve::Resolver;
use crate::vocabulary::{
    CHILD_METHOD, CHILD_PARAM, CHILD_RESULT, FLAG_POINTER_RECEIVER, FLAG_VARIADIC,
};

/// Whether a method's body is translated or stubbed.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Body {
    /// Emit `todo!()`.
    Stub,
    /// Emit no body at all — a trait item is a signature.
    None,
}

/// The methods declared on a type, as inherent `impl` items.
pub(crate) fn inherent_methods(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Vec<RustFn>, TransformError> {
    declaration
        .children_of_kind(CHILD_METHOD)
        .into_iter()
        .map(|method| method_signature(method, resolver, Visibility::Public, Body::Stub))
        .collect()
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
            let mut rendered =
                method_signature(method, resolver, Visibility::Inherited, Body::None)?;
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
) -> Result<RustFn, TransformError> {
    refuse_variadic(method)?;
    // A pointer receiver is refused rather than rendered. The IR can now SPELL `&mut self`, which
    // it could not before, and that does not make emitting one correct: whether the source mutates
    // through the receiver is a fact about aliasing that the front end does not yet report, so
    // choosing either mode here would still be a guess. What changed is that the guess is now
    // visible as a missing input rather than hidden in a format string.
    // See docs/programs/k8s-port/census/ownership-escape.md.
    if method.flags.contains(FLAG_POINTER_RECEIVER) {
        return Err(TransformError::Unsupported {
            name: method.name.clone(),
            detail: "pointer receiver: `&self` drops the mutation it permits and `&mut self` \
                     claims one the source may not perform — the front end does not yet report \
                     which, see docs/programs/k8s-port/census/ownership-escape.md"
                .to_owned(),
        });
    }

    Ok(RustFn {
        docs: Vec::new(),
        vis,
        name: to_snake_case(&method.name),
        receiver: Some(Receiver::Shared),
        params: params(method, resolver)?,
        ret: results(method, resolver)?,
        body: match body {
            Body::Stub => Some(vec![port_engine_rust_ir::RustStmt::Tail(
                port_engine_rust_ir::RustExpr::Todo,
            )]),
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
) -> Result<Vec<RustParam>, TransformError> {
    declaration
        .children_of_kind(CHILD_PARAM)
        .into_iter()
        .enumerate()
        .map(|(index, param)| {
            let ty = resolver.resolve(&param.type_ref, &declaration.name)?;
            // An unnamed parameter is legal in the source and illegal in the target, so it is
            // given a positional name. The position is already its identity, so nothing is
            // invented that was not already true.
            let name = if param.name.is_empty() || param.name == "_" {
                format!("arg{index}")
            } else {
                to_snake_case(&param.name)
            };
            Ok(RustParam {
                name,
                ty: RustType::path(ty),
            })
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
        types.push(RustType::path(
            resolver.resolve(&result.type_ref, &declaration.name)?,
        ));
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
