//! Parameters and results.
//!
//! Both are POSITIONS, which is what makes them one module: a type's target form depends on where
//! it appears, and a trait in a parameter is a different decision from a trait in a result. The
//! failure convention is the same observation one level up — a trailing result of the failure type
//! is not a result at all, it is the shape of the whole return.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustParam, RustType};

use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::ownership::{binds_by_pointer, parameter_target, reference_target};
use crate::resolve::Resolver;
use crate::vocabulary::{
    CHILD_PARAM, CHILD_RESULT, FLAG_REBOUND, FLAG_UNREAD, FLAG_VARIADIC,
    IDIOM_BORROWED_SLICE, POSITION_PARAM, POSITION_RESULT,
};

/// A variadic parameter is a SLICE, which is what it already is.
///
/// The source records `func f(args ...T)` with its last parameter typed `[]T`, because that is
/// what `args` IS inside the function — go/types says so and the snapshot has carried it all
/// along. So the signature translates through the ordinary slice rule with nothing new to decide,
/// and the refusal that used to sit here refused a question nobody was asking.
///
/// What DOES need a decision is the call, and it is refused where it happens: see
/// [`crate::body_call`]. A package that declares a variadic function ports it; only one that calls
/// a variadic function is held back.
pub(crate) fn variadic_is_a_slice(declaration: &Declaration) -> bool {
    declaration.flags.contains(FLAG_VARIADIC)
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
            let site = format!("{owner}::{}({})", declaration.name, param.name);
            // A REFERENCE parameter — a map or a slice — is shared with the caller in the source
            // exactly as a pointer is, so it gets the same decision on the same observed facts.
            // Emitting it owned would consume the caller's value and lose the sharing, which is
            // what a reviewer probing the emitted crate found: `size(my_table)` lost `my_table`.
            let ty = if is_reference_kind(&param.type_ref.kind) {
                RustType::path(reference_target(
                    param,
                    &borrowed_spelling(param, resolver, &declaration.name)?,
                    &site,
                    resolver.ownership,
                )?)
            } else if param.type_ref.kind == "pointer" {
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
                RustType::path(parameter_target(
                    param,
                    &resolved.spelling(),
                    &site,
                    resolver.ownership,
                )?)
            } else {
                resolver.resolve_in(&param.type_ref, &declaration.name, POSITION_PARAM)?
            };
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
                rebound: param.has_flag(FLAG_REBOUND),
                unread: param.has_flag(FLAG_UNREAD),
                ty,
            })
        })
        .collect()
}

pub(crate) fn results(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Option<RustType>, TransformError> {
    let mut results = declaration.children_of_kind(CHILD_RESULT);

    // A trailing result of the FAILURE type is the source's whole signal that a function can fail,
    // and the target says it in the return type instead. Splitting it off here — rather than
    // resolving it as an ordinary result — is what turns a value the caller may drop into one the
    // caller must handle.
    let fallible = crate::failure::is_fallible(declaration, resolver.failure);
    if fallible {
        results.pop();
    }

    let mut types = Vec::with_capacity(results.len());
    for result in results {
        // A failure type anywhere but last is not the convention. It is a legitimate program and it
        // has no `Result` shape, so it refuses rather than being reordered into one.
        if crate::failure::is_failure_type(&result.type_ref, resolver.failure) {
            return Err(TransformError::Unsupported {
                name: declaration.name.clone(),
                detail: "a failure value in a non-trailing result is not the source's convention, \
                         and the target's fallible return carries exactly one"
                    .to_owned(),
            });
        }
        types.push(resolver.resolve_in(&result.type_ref, &declaration.name, POSITION_RESULT)?);
    }

    let value = match types.len() {
        0 => None,
        // Several results become a tuple. That is the target's own shape for "more than one value
        // out", and it keeps arity and order visible instead of inventing a struct nobody declared.
        1 => types.pop(),
        _ => Some(RustType::Tuple(types)),
    };
    if !fallible {
        return Ok(value);
    }
    let ok = value.unwrap_or_else(|| RustType::path("()"));
    let error = resolver.failure_target(&declaration.name)?;
    Ok(Some(RustType::path(format!(
        "Result<{}, {}>",
        ok.spelling(),
        error
    ))))
}

/// Whether this source type kind is a REFERENCE the caller keeps.
///
/// A map is shared outright; a slice shares its backing array, so writing an element is visible to
/// the caller while re-slicing is not. Both are the ownership question a pointer is, which is why
/// they are answered by the same dispositions.
fn is_reference_kind(kind: &str) -> bool {
    matches!(kind, "map" | "slice")
}

/// What a borrow of this reference type borrows.
///
/// A SLICE borrows as `[T]`, not as the owned container: `&[T]` accepts every `&Vec<T>` and also
/// an array, a boxed slice and a subrange, so it takes strictly more callers while promising
/// strictly less — which is why `clippy::style` flags the container form. Nothing about the
/// program changes, and the source's slice was never an owned container in the first place.
///
/// Composed STRUCTURALLY, from the element type, rather than by rewriting the container's
/// spelling: a re-spelled container cannot silently stop matching something nobody is matching on.
/// A map has no such unsized view, so it borrows as itself.
///
/// # Errors
/// [`TransformError`] when the element or the container does not resolve.
fn borrowed_spelling(
    param: &Declaration,
    resolver: &Resolver<'_>,
    owner: &str,
) -> Result<String, TransformError> {
    if param.type_ref.kind == "slice"
        && resolver.idiom_method(IDIOM_BORROWED_SLICE).is_some()
        && let Some(element) = param.type_ref.args.first()
    {
        return Ok(format!("[{}]", resolver.resolve(element, owner)?.spelling()));
    }
    Ok(resolver
        .resolve_in(&param.type_ref, owner, POSITION_PARAM)?
        .spelling())
}
