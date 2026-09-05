//! The migration surfaces and the wire vocabulary they share.
//!
//! `POST /v1/migrations/attest` reports what a plan still owes; it is a READ
//! dressed as a POST and mutates nothing. `POST /v1/migrations/run` executes
//! one, and is a write. Each lives in its own module; what they SHARE is the
//! plan on the wire, and it is shared rather than copied so that a rule
//! landing on one cannot silently miss the other.
//!
//! THE TENANT IS THE CREDENTIAL'S. `MigrationPlan` carries its own
//! `tenant_id`, and the write path settled what that means: nothing in the
//! body may move the tenant. A plan whose tenant disagrees with the
//! credential is refused rather than silently rewritten — a caller who names
//! the wrong tenant has asked a question this process should not answer.
//!
//! That check is DEFENCE IN DEPTH on both surfaces and is not what stops a
//! cross-tenant access; claiming otherwise would misdirect the next reader to
//! the wrong control. The PDP refuses a caller whose credential does not
//! carry the tenant, and `tenant_of` resolves the tenant by
//! `caller.tenant_id` unconditionally, so the body cannot select one. Delete
//! the check and a foreign plan still refuses, with a worse diagnostic — which
//! is the reason to keep it.

pub(crate) mod attest;
pub(crate) mod run;

use serde::Deserialize;

use foundry_edits::WireDouble;
use foundry_spine::{
    DefaultValue, MigrationPlan, UpcastTransform, ValueConversion, migration_attestation,
};

/// UNKNOWN FIELDS ARE REFUSED, and the omission was nearly catastrophic.
/// With `transforms` defaulting and an unknown key discarded, a one-character
/// typo — `"transform"` — yielded an empty transform list, a `validate` whose
/// transform loop passes vacuously, and `{"fixpoint":true,"pending":[]}`: a
/// green light to skip a migration that is owed. The write path has held this
/// law since it shipped, four lines above the tenancy rule this module took
/// from the same file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PlanRequest {
    pub(crate) tenant_id: String,              // data_class: INTERNAL_ONLY
    pub(crate) entity_type: String,            // data_class: INTERNAL_ONLY
    pub(crate) from_revision: u32,             // data_class: INTERNAL_ONLY
    pub(crate) to_revision: u32,               // data_class: INTERNAL_ONLY
    pub(crate) action_type: String,            // data_class: INTERNAL_ONLY
    pub(crate) audit_event_type: String,       // data_class: INTERNAL_ONLY
    pub(crate) declared_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    #[serde(default)]
    pub(crate) transforms: Vec<WireTransform>, // data_class: INTERNAL_ONLY
}

/// The transform vocabulary, mirrored on the wire.
///
/// The spine's `UpcastTransform` carries no serde, and giving it some would
/// make a kernel type's field names a public wire contract that could not
/// then be refactored. The mirror is the seam: it is a facade concern that
/// this vocabulary is expressible in JSON at all, and the mapping below is
/// where a wire value becomes a domain one — including the refusals, which
/// a derive would have had nowhere to put.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireTransform {
    CopyAs {
        from: String, // data_class: INTERNAL_ONLY
        to: String,   // data_class: INTERNAL_ONLY
    },
    ConvertAs {
        from: String,       // data_class: INTERNAL_ONLY
        to: String,         // data_class: INTERNAL_ONLY
        conversion: String, // data_class: INTERNAL_ONLY
    },
    DefaultTo {
        to: String,         // data_class: INTERNAL_ONLY
        value: WireDefault, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireDefault {
    String { value: String },        // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    Integer { value: i64 },          // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    Boolean { value: bool },         // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    Double { value: f64 },           // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    Timestamp { epoch_millis: i64 }, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
}

impl WireDefault {
    /// A non-finite double is refused here rather than canonicalised. NaN
    /// has no total order and no canonical bytes, so a plan carrying one
    /// could not produce a stable digest — `WireDouble::new` is the law and
    /// this surface reports it rather than routing around it.
    ///
    /// No JSON request can reach that refusal, and that is measured, not
    /// assumed: the grammar has no `NaN` or `Infinity` literal, and
    /// `serde_json` fails an out-of-range exponent (`1e400`) with "number
    /// out of range" rather than yielding an infinity. The arm stays because
    /// the law belongs to `WireDouble`, not to the codec that happens to
    /// front it today; a second codec would arrive to find it already held.
    fn into_domain(self) -> Result<DefaultValue, &'static str> {
        Ok(match self {
            WireDefault::String { value } => DefaultValue::String(value),
            WireDefault::Integer { value } => DefaultValue::Integer(value),
            WireDefault::Boolean { value } => DefaultValue::Boolean(value),
            WireDefault::Double { value } => DefaultValue::Double(
                WireDouble::new(value).map_err(|_| "a default double must be finite")?,
            ),
            WireDefault::Timestamp { epoch_millis } => DefaultValue::Timestamp { epoch_millis },
        })
    }
}

impl WireTransform {
    /// The VARIANT is observable here; the VALUE it carries is not. The
    /// distinction is exact and was got wrong once, so it is written down.
    ///
    /// The variant is checked: `check_transform` compares the target's
    /// declared scalar against `DefaultValue::scalar_type()`, which is a
    /// function of the variant alone. That check is invisible against an
    /// UNTYPED target, which carries the legacy String contract under which
    /// every non-string default is incompatible and they all refuse alike —
    /// so the suite declares a typed property for the arms to be told apart.
    ///
    /// The value is not: `computed_target` returns early when the target
    /// property is present — a default fills an absence and never overwrites
    /// — and when it is absent, any value at all yields a computed target. So
    /// `pending` cannot depend on which constant the arm produced.
    /// `POST /v1/migrations/run` is where that becomes observable, because
    /// there the value is written; it is pinned there, not pretended here.
    fn into_domain(self) -> Result<UpcastTransform, &'static str> {
        Ok(match self {
            WireTransform::CopyAs { from, to } => UpcastTransform::CopyAs { from, to },
            WireTransform::ConvertAs {
                from,
                to,
                conversion,
            } => UpcastTransform::ConvertAs {
                from,
                to,
                conversion: match conversion.as_str() {
                    "integer_to_string" => ValueConversion::IntegerToString,
                    "boolean_to_integer" => ValueConversion::BooleanToInteger,
                    _ => return Err("that conversion is not one this process performs"),
                },
            },
            WireTransform::DefaultTo { to, value } => UpcastTransform::DefaultTo {
                to,
                value: value.into_domain()?,
            },
        })
    }
}
