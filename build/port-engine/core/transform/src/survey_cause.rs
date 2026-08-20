//! A refusal reduced to what identifies its CAUSE.
//!
//! Split from `survey.rs` because ranking is a different question from measuring. What ranks the
//! work is how many PACKAGES a cause blocks, and two refusals share a cause when they are the same
//! missing rule — not when they happen to mention the same declaration.
//!
//! So the reason is carried verbatim and the declaration's own name is not part of it: a cause that
//! carried the site would count once per site and rank a rule nobody needs above one that blocks
//! six packages.

use crate::error::TransformError;

/// A refusal, reduced to what identifies its CAUSE.
///
/// Two competing pressures, and getting either wrong makes the ranking useless. A reason that keeps
/// the DECLARATION's name counts once per site and ranks nothing — two hundred functions blocked by
/// one missing rule must read as one row of two hundred. A reason that drops the SUBJECT ranks
/// everything into one row and names nothing to add: "the type map does not carry it", twenty
/// times, for twenty different types.
///
/// So the subject is kept and the site is dropped, per variant, rather than by cutting the rendered
/// string at a delimiter it was never designed to have.
pub(crate) fn refusal_of(error: &TransformError) -> String {
    match error {
        TransformError::UnmappedType { type_ref, .. } => {
            format!("unmapped type `{type_ref}`")
        }
        TransformError::MissingDatum {
            construction,
            datum,
            ..
        } => format!("`{construction}` needs `{datum}`, which the front end did not record"),
        TransformError::ConstructionKindMismatch {
            construction, kind, ..
        } => format!("construction `{construction}` does not fit a `{kind}`"),
        TransformError::Unsupported { detail, .. } => detail.clone(),
        // The FORM, and not the reason and not the name. The reason interpolates the declaration's
        // name, so rendering it here made one undecided form read as eighteen causes of one site
        // each — which is exactly what this module exists to prevent, and it hid the largest
        // structural cause in `google/uuid` for as long as the histogram has been used to choose
        // work. The form is the missing decision; every declaration waiting on it is one row.
        TransformError::UndecidedForm { form, .. } => {
            format!("`{form}` is a form the pack has not decided")
        }
        other => other.to_string(),
    }
}
