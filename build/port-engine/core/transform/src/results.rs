//! What a declaration RETURNS, in the target's terms.
//!
//! Split from `params.rs` because a parameter and a result are answered from opposite directions.
//! A parameter's shape is decided by what the CALLER keeps; a result's by what the body can prove
//! about what it produces — a pointer that is never absent, a view of the receiver, a length, an
//! ordering. Each of those is a fact the signature and the body must agree on.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustType;

use crate::error::TransformError;
use crate::resolve::Resolver;
use crate::vocabulary::{CHILD_RESULT, POSITION_RESULT, TARGET_STR};

/// The results a method has when its SIGNATURE came from elsewhere.
///
/// A trait fixes the shape, so no result idiom applies: the borrow a getter would earn is not this
/// method's to take, because its caller is written against the trait's spelling.
///
/// # Errors
/// [`TransformError`] from resolving a result type.
pub(crate) fn results_owned(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Option<RustType>, TransformError> {
    results_in(declaration, resolver, false)
}

pub(crate) fn results(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Option<RustType>, TransformError> {
    results_in(declaration, resolver, true)
}

/// Every result, with `idioms` deciding whether a result-shape idiom may apply.
///
/// # Errors
/// [`TransformError`] from resolving a result type, or a failure type in a non-trailing position.
pub(crate) fn results_in(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    idioms: bool,
) -> Result<Option<RustType>, TransformError> {
    let mut results = declaration.children_of_kind(CHILD_RESULT);

    // A trailing result of the FAILURE type is the source's whole signal that a function can fail,
    // and the target says it in the return type instead. Splitting it off here — rather than
    // resolving it as an ordinary result — is what turns a value the caller may drop into one the
    // caller must handle.
    // A SOLE failure result may not be the failure channel at all. Where the body never returns
    // the absent value it is handing back an error as a VALUE, and popping it into `Result` says
    // the function reports success — which it has no way to do. See `returns::sole_failure_role`.
    // ONE decision, not two. Deriving "this is a value" separately from "this is the form for it"
    // let them disagree: where the pack declares no nullable form the role still said value, so
    // nothing was popped and the loop below then refused the result for being a failure type in a
    // position it does not allow. A pack that has not made this decision must land exactly on the
    // previous behaviour, and it only does if the same answer drives both.
    // A GETTER lends what it holds. Asked before the owned forms, because the owned nullable one
    // would be a promise the receiver cannot keep: the target's boxed failure is not clonable, so
    // handing one out from behind a shared receiver has no spelling at all. The engine's own
    // compile proof caught exactly that.
    if idioms
        && crate::returns::borrows_failure_from_receiver(declaration, resolver)
        && let Some(convention) = resolver.failure
        && !convention.nullable_borrowed_type.is_empty()
    {
        return Ok(Some(RustType::path(convention.nullable_borrowed_type.clone())));
    }
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
        // A THREE-WAY COMPARISON returns the target's ordering type, which is the type its own
        // sorting and its own `Ord` are defined in terms of.
        if idioms && crate::returns::is_three_way_comparison(declaration, resolver) {
            types.push(RustType::path(
                resolver
                    .idiom_method(crate::vocabulary::IDIOM_ORDERING)
                    .unwrap_or("std::cmp::Ordering"),
            ));
            continue;
        }
        // A LENGTH is a `usize`, not the source's own integer. The conversion the call's mapping
        // adds exists to make the value type as the source's `int`, and where the value never is
        // one the conversion is what is wrong.
        if idioms && crate::returns::yields_a_length(declaration, resolver.length_functions) {
            types.push(RustType::path("usize"));
            continue;
        }
        // A GETTER's result is a VIEW of the receiver, not a copy of it. The source's string
        // shares its backing, so handing one back copies nothing; an owned `String` would clone on
        // every call, which is work the source never does.
        if idioms && crate::returns::borrows_from_receiver(declaration) {
            types.push(RustType::Reference {
                mutable: false,
                inner: Box::new(RustType::path(TARGET_STR)),
            });
            continue;
        }
        // A `*T` result that is NEVER ABSENT is a `T`. The pointer type carries `Option` because
        // the source's pointer admits nil, and the `Box` because a pointer owns — and a function
        // whose every return is the address of a value it just created can produce neither an
        // absent result nor an alias anyone else holds. So the caller gets ownership of a value,
        // which is exactly what the source hands them, without an `Option` that has one inhabited
        // case and an allocation nobody asked for. See `returns.rs` for what counts as proof.
        if crate::returns::never_absent_pointer(declaration, result) {
            let Some(pointee) = result.type_ref.args.first() else {
                return Err(TransformError::Unsupported {
                    name: declaration.name.clone(),
                    detail: "a pointer result carries no pointee type".to_owned(),
                });
            };
            types.push(resolver.resolve_in(pointee, &declaration.name, POSITION_RESULT)?);
            continue;
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
    // The unit's own name for the failure type, where the pack gives one. An alias is transparent,
    // so this changes what the signature READS as and not what it means.
    if let Some(alias) = resolver.failure_alias() {
        return Ok(Some(RustType::path(format!("{alias}<{}>", ok.spelling()))));
    }
    let error = resolver.failure_target(&declaration.name)?;
    Ok(Some(RustType::path(format!(
        "Result<{}, {}>",
        ok.spelling(),
        error
    ))))
}


/// Whether this declaration hands a failure back as a VALUE rather than through the channel.
///
/// The one place that question is answered, so the signature and the body cannot disagree about it.
/// They are built by different code and were asked separately once; a signature that says `Option<E>`
/// while the body still emits `Err(..)` is two spellings of one decision, which is how the `mut`
/// on a folded parameter went wrong three times.
pub(crate) fn returns_failure_as_value(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> bool {
    crate::returns::borrows_failure_from_receiver(declaration, resolver)
        && resolver
            .failure
            .is_some_and(|convention| !convention.nullable_borrowed_type.is_empty())
}
