//! Where the SOURCE's type system is wider than the target's, and the difference is not a spelling.
//!
//! Two constructs so far, and they have the same shape: the source admits something the target does
//! not, and translating the parts without the whole emits a crate that does not build. Both refuse
//! by name, and both name the rule that would replace the refusal.
//!
//! Kept together rather than in the files that call them because that is what they are about — not
//! statements, not operators, but the places where a faithful translation needs a construct the
//! target lacks and the engine must say so rather than approximate.

use port_engine_api::Declaration;

use crate::body::Body;
use crate::error::TransformError;

/// Refuse a signature whose RESULTS are named, which makes them pre-bound variables.
///
/// The source lets a signature name its results, and those names are bindings the body may assign
/// to before returning — `func (t Time) UnixTime() (sec, nsec int64)` writes `sec` and `nsec` and
/// then returns them. The target has no such thing: a result is a value the body produces, and a
/// name in the signature binds nothing.
///
/// Translating the assignments without the bindings emitted a body naming variables that do not
/// exist — the same dangling-name defect self-containment refuses everywhere else, arrived at from
/// the signature rather than from a call. Eleven of them in one package, and they only became
/// visible once the rung that stubbed those method bodies started translating them.
///
/// The rule this wants is small and is not written yet: each named result is a `let mut` of its
/// zero value at the top of the body, and a bare `return` returns them as a tuple. Refused by name
/// until it is, because a body that assigns to nothing is worse than one the engine declined.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the declaration and the results it cannot bind.
pub(crate) fn refuse_named_results(declaration: &Declaration) -> Result<(), TransformError> {
    let named: Vec<&str> = declaration
        .children_of_kind(crate::vocabulary::CHILD_RESULT)
        .iter()
        .filter(|result| !result.name.is_empty())
        .map(|result| result.name.as_str())
        .collect();
    if named.is_empty() {
        return Ok(());
    }
    Err(TransformError::Unsupported {
        name: declaration.name.clone(),
        detail: format!(
            "its results are NAMED ({}), which makes them bindings the body may assign to before \
             returning; the target has no such thing, and translating those assignments without \
             binding the names emits a body that reads variables which do not exist",
            named.join(", ")
        ),
    })
}

/// Refuse an operation the target's NEWTYPE does not support and the source's defined type did.
///
/// The source's `type Version byte` is transparent: it compares against an untyped constant, it
/// formats as a number, it does arithmetic. The target's newtype is opaque — a distinct type with
/// none of its underlying type's operators — and that opacity is the whole reason the newtype is
/// the faithful shape, because it keeps the distinction the source declared.
///
/// So the two disagree exactly here, and the disagreement is invisible until the operand is used:
/// `if v > 15` in the source became `if self > 15` on a `&Version`, which does not compile, and
/// `%d` of one became `format!("{}", self)` on a type with no `Display`.
///
/// The rule this wants is `self.0` — a defined type over a numeric underlying is its wrapped value
/// wherever the underlying type is what the operation needs. Refused by name until it is written,
/// because the alternative is not a wrong program but a crate that does not build, and because
/// unwrapping without proving the underlying type is numeric would silently reach inside a newtype
/// declared to stop exactly that.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the declaration and the type whose opacity blocks it.
pub(crate) fn refuse_opaque_newtype(
    left: &Declaration,
    right: &Declaration,
    spelling: &str,
    cx: &Body<'_>,
) -> Result<(), TransformError> {
    for (operand, other) in [(left, right), (right, left)] {
        let Some(name) = local_named_type(operand, cx) else {
            continue;
        };
        // The SAME defined type on both sides is what the newtype supports through its derives —
        // two `Version`s compare because the type derives the comparison. Anything else asks for an
        // operator the newtype does not have.
        if local_named_type(other, cx).as_deref() == Some(name.as_str()) {
            continue;
        }
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "`{spelling}` is applied to `{name}`, a type this unit DEFINES, and to something \
                 that is not one — the source's defined type carries its underlying type's \
                 operators and the target's newtype carries none, which is what keeps the \
                 distinction the source declared"
            ),
        });
    }
    Ok(())
}

/// The name of a locally declared defined type this operand has, if it has one.
fn local_named_type(operand: &Declaration, cx: &Body<'_>) -> Option<String> {
    let type_ref = &operand.type_ref;
    if type_ref.kind != "named" {
        return None;
    }
    // NOT filtered by what is emitted. The fixpoint that decides emittability only ever SHRINKS,
    // and that is what makes it terminate; a refusal that consults it would stop firing as the set
    // lost members, un-refusing a declaration and growing the set again. It hung on the first
    // package that had one. Whether a type is a newtype is a fact about the source, so ask the
    // source.
    cx.resolver.scope.types.get(&type_ref.name).cloned()
}
