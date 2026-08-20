//! Parameters and results.
//!
//! Both are POSITIONS, which is what makes them one module: a type's target form depends on where
//! it appears, and a trait in a parameter is a different decision from a trait in a result. The
//! failure convention is the same observation one level up — a trailing result of the failure type
//! is not a result at all, it is the shape of the whole return.

use std::collections::BTreeSet;

use port_engine_api::{Declaration, TypeRef};
use port_engine_rust_ir::{RustStmt, Visibility, RustParam, RustType};

use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::ownership::{binds_by_pointer, parameter_target, reference_target};
use crate::resolve::Resolver;
use crate::vocabulary::{
    CHILD_PARAM, CHILD_RESULT, FLAG_REBOUND, FLAG_UNREAD, FLAG_VARIADIC, IDIOM_BORROWED_SLICE, IDIOM_SINGLE_EXPRESSION_INLINE, POSITION_PARAM, POSITION_RESULT, SOURCE_STRING, TARGET_STR,
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
    // Parameters the body FOLDED away, which therefore need no `mut`. Empty where there is no body
    // to fold — a signature alone answers a different question and must not guess this one.
    consumed: &BTreeSet<String>,
) -> Result<Vec<RustParam>, TransformError> {
    params_at(declaration, resolver, owner, consumed, POSITION_PARAM)
}

/// The same, at a stated POSITION.
///
/// The position decides what an interface parameter becomes, and a trait method's answer differs
/// from a free function's — not by preference but by necessity: `impl Trait` in a trait method's
/// argument makes the trait not dyn-compatible, and the source's interface values live in slices
/// that need a boxed trait object to exist at all.
pub(crate) fn params_at(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    owner: &str,
    consumed: &BTreeSet<String>,
    position: &str,
) -> Result<Vec<RustParam>, TransformError> {
    // WRITTEN THROUGH A CALL. The source spells a write into a value as a call — `PutUint16(id[..],
    // n)` fills the array it is given — and the target spells it as a mutation of the receiver. A
    // parameter that only ever appears as such an argument is never SEEN assigned, so it came out
    // immutable and the emitted body did not compile.
    //
    // The same walk the named results already use, and which callees write is the pack's to say:
    // it names them, because they are the same ones it maps.
    let mut written = BTreeSet::new();
    for child in &declaration.children {
        crate::body_wider::collect_assigned(child, resolver, &mut written);
    }
    declaration
        .children_of_kind(CHILD_PARAM)
        .into_iter()
        .map(|param| {
            // A POINTER parameter is an ownership question and gets a decision; anything else is
            // just a type. The split is deliberate: a pointer inside a field or a result has no
            // call site to borrow across, so it stays a plain type-map answer.
            let site = format!("{owner}::{}({})", declaration.name, param.name);
            // A REFERENCE parameter — a map or a slice — is shared with the caller in the source
            // exactly as a pointer is, so it gets the same decision on the same observed facts.
            // Emitting it owned would consume the caller's value and lose the sharing, which is
            // what a reviewer probing the emitted crate found: `size(my_table)` lost `my_table`.
            // A parameter used for NOTHING BUT indexing is a `usize`, exactly as a loop counter
            // is. Its every value reaches the target's index, which is unsigned — so the signed
            // type exists only to be converted, and a NEGATIVE argument the source would reject
            // with a bounds check becomes one the target rejects at the call. Stricter, never
            // different: no value the source accepts here is one the target does not.
            let ty = if crate::index_params::indexes_only_parameter(declaration, param, resolver) {
                RustType::path("usize")
            } else if is_reference_kind(&param.type_ref) {
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
                resolver.resolve_in(&param.type_ref, &declaration.name, position)?
            };
            // AN UNNAMED PARAMETER STAYS UNNAMED. The source writes `pkcs7decode(buf []byte, _ int)`
            // and `NotifyMsg([]byte)` — in both the name is absent on purpose, and the body cannot
            // refer to the value because there is nothing to refer to. The target spells that `_`.
            //
            // It used to become `arg{index}`, argued as inventing nothing because the position was
            // already its identity. But a NAME is not a position: `arg0` appears in the emitted
            // documentation, and every downstream implementor of a trait method has to write it
            // out. Both review gates named it as a placeholder that should not ship.
            let name = if param.name.is_empty() || param.name == "_" {
                "_".to_owned()
            } else {
                to_snake_case(&param.name)
            };
            Ok(RustParam {
                name,
                // NOT rebound where the body FOLDS it: an accumulator becomes one expression, so
                // the binding it was assigned through is gone and a `mut` on it would be a
                // mutability nothing uses. One fact, read by the signature and the body alike.
                // NOT rebound where the body FOLDED it away: the binding it was assigned through
                // is gone, and a `mut` on it would be a mutability nothing uses. Read from what the
                // fold DID rather than from what the recogniser predicted, because the two differ
                // whenever a value arrives as opaque target text.
                rebound: (param.has_flag(FLAG_REBOUND) || written.contains(&param.name))
                    && !consumed.contains(&param.name),
                unread: param.has_flag(FLAG_UNREAD),
                ty,
            })
        })
        .collect()
}

/// Whether this source type kind is a REFERENCE the caller keeps.
///
/// A map is shared outright; a slice shares its backing array, so writing an element is visible to
/// the caller while re-slicing is not. Both are the ownership question a pointer is, which is why
/// they are answered by the same dispositions.
fn is_reference_kind(type_ref: &TypeRef) -> bool {
    // A STRING is a reference too, and it is the one that is not a composite kind. The source's
    // string is immutable and shares its backing, so passing it costs nothing and the caller keeps
    // it — emitting an owned `String` consumes a value the source never consumed.
    matches!(type_ref.kind.as_str(), "map" | "slice") || type_ref.name == SOURCE_STRING
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
    if param.type_ref.name == SOURCE_STRING
        && resolver.idiom_method(IDIOM_BORROWED_SLICE).is_some()
    {
        // `str` is the string's unsized view, exactly as `[T]` is a sequence's: `&str` takes every
        // `&String` and also a literal and a subslice, where `&String` takes only the container.
        return Ok(TARGET_STR.to_owned());
    }
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

/// The parameter names this signature BORROWS.
///
/// Answered by asking the same question the signature answered, rather than by re-deriving it: a
/// parameter is borrowed exactly when it is a reference type whose disposition chose a borrow. A
/// value reaching a position that OWNS — a struct literal's field — has to be owned there, and the
/// source did not have to say so because its string and its slice were already shared.
pub(crate) fn borrowed_parameters(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> BTreeSet<String> {
    declaration
        .children_of_kind(CHILD_PARAM)
        .into_iter()
        .filter(|param| is_reference_kind(&param.type_ref))
        .filter(|param| {
            let site = format!("{}({})", declaration.name, param.name);
            borrowed_spelling(param, resolver, &declaration.name)
                .ok()
                .and_then(|spelling| {
                    reference_target(param, &spelling, &site, resolver.ownership).ok()
                })
                .is_some_and(|target| target.starts_with('&'))
        })
        .map(|param| to_snake_case(&param.name))
        .collect()
}

/// The attributes a function carries, which today is at most the target's inline hint.
///
/// A private function whose body is ONE expression gets it, and that narrowness is the argument.
/// This is the rare case where NOT emitting something changes the ported program relative to the
/// source: the source's compiler inlines a small function by a cost heuristic with no annotation,
/// and the target's does not across codegen units for a non-generic private one. So the source's
/// helper is inlined and the port's is a call — a performance difference the translation introduced
/// rather than one the author chose.
///
/// One expression is the shape the source's own heuristic would certainly have inlined, so the
/// attribute RESTORES a decision the source already made rather than inventing one. A public
/// function is left alone: whether to promise inlining across a crate boundary is a decision about
/// the ported library's contract, and that belongs to whoever ports it.
pub(crate) fn inline_attrs(
    body: Option<&[RustStmt]>,
    vis: Visibility,
    resolver: &Resolver<'_>,
) -> Vec<String> {
    let mut attrs = Vec::new();
    // A JUSTIFIED ALLOW, which is the only kind this engine emits. `clippy::manual_range_contains`
    // asks for a rewrite that is WRONG for a partially-ordered type, and the deny-warnings policy
    // would otherwise force the engine to choose between a lint and the program's meaning. It
    // chooses meaning and says why, in the attribute, where the next reader finds it.
    if body.is_some_and(|statements| {
        statements
            .iter()
            .any(crate::body_swap::compares_float_bounds)
    }) {
        attrs.push(
            "#[allow(clippy::manual_range_contains, reason = \"the range form answers the \
             opposite for NaN: every comparison against NaN is false, so the source's disjunction \
             is false and the negated contains is true\")]"
                .to_owned(),
        );
    }
    let Some(method) = resolver.idiom_method(IDIOM_SINGLE_EXPRESSION_INLINE) else {
        return attrs;
    };
    if vis != Visibility::Inherited {
        return attrs;
    }
    if let Some([RustStmt::Tail(_)]) = body {
        attrs.push(method.to_owned());
    }
    attrs
}
