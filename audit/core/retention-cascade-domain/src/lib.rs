//! Audit-chain retention-cascade domain: pure retention-window, DSR-cascade
//! and redaction-token rules.
//!
//! This crate enforces three families of rule, all sourced from
//! `audit/policy/retention-matrix.yaml` (the "matrix"):
//!
//! 1. **Retention floors** ([`validate_retention_policy`]) — a
//!    [`RetentionPolicy`]'s `retention_seconds` must meet or exceed the
//!    floor for its `(pack, data_class)` pair. The floor starts from the
//!    pack's `minimum_years`, is raised to the pack's
//!    `financial_services_years` when the caller asserts the policy governs
//!    financial-services records (today only `pack-kr`, 3 years → 5 years),
//!    and is then raised again to the data class's `minimum_years_override`
//!    if that is stricter still (only `PHI` carries one, at 6 years). The
//!    floor is always the GREATEST of whichever of those apply.
//! 2. **DSR cascade confinement** ([`plan_dsr_cascade`]) — a data-subject-rights
//!    cascade may only touch one pack, and may not run before
//!    `dsr_redaction_grace_days` have elapsed since the request.
//! 3. **Redaction token rules** ([`redact_payload`]) — a redaction token must
//!    carry a known reason and a lawful basis, and:
//!    - a redaction is refused outright for any data class whose matrix
//!      entry does not set `preserve_merkle_proof: true` — every class in
//!      today's matrix does, but a caller-supplied matrix that disagrees is
//!      refused rather than silently honoured, because this is the one
//!      invariant [`RedactionEffect::merkle_proof_preserved`] exists to
//!      guarantee;
//!    - a DSR-triggered redaction is refused for any data class whose
//!      matrix entry does not set `dsr_redaction_supported: true` (only
//!      `PII_IDENTIFYING` does);
//!    - a retention-expiry redaction erases the payload only when the data
//!      class's `delete_payload_after_retention` is `true` (only `AUDIT`
//!      does today) — for every other class the token is still accepted,
//!      but [`RedactionEffect::payload_erased`] comes back `false`.
//!
//! ## Identifier normalization (L2 / L3)
//!
//! [`DsrCascade`]'s `tenant_id`, `subject_id` and `source_microservice`, and
//! [`RedactionToken`]'s `audit_id` and `lawful_basis`, are all validated with
//! [`is_normalized_identifier`]: every character must be ASCII alphanumeric
//! or one of `- _ . : @`. This is a decision, not an oversight: it rejects
//! whitespace padding, embedded newlines, and every Unicode invisible/format
//! character (zero-width space, BOM, word joiner, soft hyphen, tag
//! characters, control characters, …) in the SAME pass, rather than trying
//! to enumerate them. The alternative — trim-then-store — was rejected
//! because it makes `"tenant-1"`, `" tenant-1"` and `"tenant-1\n"` three
//! distinct stored partitions for what a human reading a ticket would call
//! one tenant (the exact aliasing bug this crate must not reproduce); this
//! crate refuses all but the single normalized spelling instead of guessing
//! which one to keep.
//!
//! ## No I/O (L8 / pure domain)
//!
//! This crate performs no file or network I/O and depends on nothing beyond
//! `audit-retention-cascade-api`. It cannot read `retention-matrix.yaml`
//! itself. [`RetentionMatrix`] is the ALREADY-PARSED shape the matrix enters
//! the domain through: [`RetentionMatrix::audit_chain_canonical`] hardcodes
//! the values this crate's checks actually use (each pinned by a test in
//! this crate, so drift between the two is a failing test, not a silent
//! gap), and [`RetentionMatrix::from_parsed_rules`] lets an adapter supply a
//! matrix it parsed itself — checked at construction against the two values
//! whose being `0` would make this crate's floor and grace-period checks
//! no-ops, so a degenerate adapter-parsed matrix cannot be built at all; see
//! its own doc for exactly what is and is not checked. The adapter that
//! reads and parses the YAML file
//! — for example a loader living under `audit/adapters/file` alongside the
//! [`RetentionPolicySource` port declared in
//! `audit-retention-cascade-kernel`](../../ports/retention-cascade-kernel/src/lib.rs)
//! — is the layer responsible for keeping that parsed shape in sync with the
//! file; this crate only checks values it is handed.
//!
//! Two matrix keys are deliberately NOT represented, because no rule in this
//! crate's brief attaches a check to them: `pack-eu`'s `purpose_limited` and
//! `pack-us-healthcare`'s `legal_basis` are informational citations this
//! crate carries no behaviour for and does not track. `worm_required`, by
//! contrast, IS represented ([`PackRetentionRule::worm_required`], pinned
//! for every pack by `matrix_pack_minimums_match_policy_file` so an edit to
//! it in the YAML is a failing test) even though no function in this crate
//! enforces it: write-once-read-many is a storage-layer property this pure
//! domain has no way to check, so the field is tracked for drift but not
//! gated on. `SECRET`'s `payload_export_forbidden` governs payload EXPORT, a
//! capability this crate does not implement (this crate only plans DSR
//! cascades and applies redaction); whichever capability implements payload
//! export — none exists in this repo yet — is the layer that must enforce
//! it, not this retention/redaction-cascade domain.
#![allow(dead_code)]

use std::collections::BTreeMap;

pub use audit_retention_cascade_api::{DsrCascade, RedactionToken, RetentionPolicy, RetentionRun};

/// Domain-level retention error.
///
/// The first three variants (`PolicyShortenedBelowMinimum`, `CrossPackCascade`,
/// `InvalidRedactionReason`) are the original scaffold surface and are kept
/// unit variants on purpose: callers that already match on them keep
/// compiling. Every other variant was added while giving this crate a real
/// implementation and carries the detail its check needs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetentionDomainError {
    /// A [`RetentionPolicy`]'s `retention_seconds` is below the applicable
    /// floor (the greater of the pack minimum and any class override).
    PolicyShortenedBelowMinimum,
    /// A DSR cascade's target packs are not all the same pack.
    CrossPackCascade,
    /// A [`RedactionToken`]'s `reason` is empty or not one of the closed set
    /// of accepted reasons, or its `lawful_basis` is empty/invisible-only.
    InvalidRedactionReason,

    /// A pack label is not one of the closed set the matrix declares.
    UnknownPack { pack: String },
    /// A data-class label is not one of the closed set the matrix declares.
    UnknownDataClass { data_class: String },
    /// A [`Pack`] parsed successfully but the [`RetentionMatrix`] in use has
    /// no rule for it (only reachable with a caller-supplied matrix from
    /// [`RetentionMatrix::from_parsed_rules`] that omits an entry).
    PackNotInMatrix { pack: String },
    /// A [`DataClass`] parsed successfully but the [`RetentionMatrix`] in use
    /// has no rule for it (only reachable with a caller-supplied matrix from
    /// [`RetentionMatrix::from_parsed_rules`] that omits an entry).
    DataClassNotInMatrix { data_class: String },

    /// A [`DsrCascade`]'s `tenant_id` is empty or is not a normalized
    /// identifier per [`is_normalized_identifier`] (contains whitespace, an
    /// invisible/format character, or any character outside
    /// `[A-Za-z0-9\-_.:@]`).
    EmptyDsrTenantId,
    /// A [`DsrCascade`]'s `subject_id` is not a normalized identifier; see
    /// [`RetentionDomainError::EmptyDsrTenantId`].
    EmptyDsrSubjectId,
    /// A [`DsrCascade`]'s `source_microservice` is not a normalized
    /// identifier; see [`RetentionDomainError::EmptyDsrTenantId`].
    EmptyDsrSourceMicroservice,
    /// A DSR cascade named zero target packs — there is nothing to confine.
    EmptyCascadeTargets,
    /// A DSR cascade was planned before `dsr_redaction_grace_days` had
    /// elapsed since the request.
    RedactionGracePeriodNotElapsed {
        required_days: u32,
        elapsed_days: u32,
    },

    /// A [`RedactionToken`]'s `audit_id` is not a normalized identifier; see
    /// [`RetentionDomainError::EmptyDsrTenantId`].
    EmptyRedactionAuditId,
    /// A DSR-triggered redaction was requested for a data class whose matrix
    /// entry does not set `dsr_redaction_supported: true`.
    DsrRedactionUnsupportedForClass { data_class: String },
    /// A redaction was requested for a data class whose matrix entry sets
    /// `preserve_merkle_proof: false`. This crate refuses to perform ANY
    /// redaction under such a matrix rather than construct a
    /// [`RedactionEffect`] whose proof-preservation guarantee would not
    /// actually hold — see the module doc's "No I/O" section.
    MerkleProofPreservationNotConfigured { data_class: String },

    /// [`RetentionMatrix::from_parsed_rules`] was handed a rule that would
    /// silently defeat a check this crate exists to enforce: a pack's
    /// `minimum_years` of `0` (the retention floor [`validate_retention_policy`]
    /// checks against), or a matrix-wide `dsr_redaction_grace_days` of `0`
    /// (the grace period [`plan_dsr_cascade`] checks against). Refused at
    /// construction rather than accepted and silently rendered toothless.
    DegenerateMatrixRule { detail: String },
}

/// A tenancy pack, as declared under `packs:` in
/// `audit/policy/retention-matrix.yaml`. Closed set — a label that is not one
/// of these is rejected by [`Pack::parse`] rather than silently accepted.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Pack {
    Eu,
    Kr,
    Us,
    UsHealthcare,
    Jp,
    Sg,
    Au,
    In,
    Br,
    Ae,
    Ksa,
}

impl Pack {
    /// Parse the matrix's literal `pack-*` label. Unknown labels are
    /// rejected rather than defaulted, so a typo or a not-yet-onboarded pack
    /// never silently falls through to some other pack's floor.
    pub fn parse(label: &str) -> Result<Self, RetentionDomainError> {
        match label {
            "pack-eu" => Ok(Self::Eu),
            "pack-kr" => Ok(Self::Kr),
            "pack-us" => Ok(Self::Us),
            "pack-us-healthcare" => Ok(Self::UsHealthcare),
            "pack-jp" => Ok(Self::Jp),
            "pack-sg" => Ok(Self::Sg),
            "pack-au" => Ok(Self::Au),
            "pack-in" => Ok(Self::In),
            "pack-br" => Ok(Self::Br),
            "pack-ae" => Ok(Self::Ae),
            "pack-ksa" => Ok(Self::Ksa),
            _ => Err(RetentionDomainError::UnknownPack {
                pack: label.to_string(),
            }),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Eu => "pack-eu",
            Self::Kr => "pack-kr",
            Self::Us => "pack-us",
            Self::UsHealthcare => "pack-us-healthcare",
            Self::Jp => "pack-jp",
            Self::Sg => "pack-sg",
            Self::Au => "pack-au",
            Self::In => "pack-in",
            Self::Br => "pack-br",
            Self::Ae => "pack-ae",
            Self::Ksa => "pack-ksa",
        }
    }
}

/// A data class, as declared under `classes:` in
/// `audit/policy/retention-matrix.yaml`. Closed set for the same reason as
/// [`Pack`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum DataClass {
    Audit,
    Phi,
    PiiIdentifying,
    Secret,
}

impl DataClass {
    pub fn parse(label: &str) -> Result<Self, RetentionDomainError> {
        match label {
            "AUDIT" => Ok(Self::Audit),
            "PHI" => Ok(Self::Phi),
            "PII_IDENTIFYING" => Ok(Self::PiiIdentifying),
            "SECRET" => Ok(Self::Secret),
            _ => Err(RetentionDomainError::UnknownDataClass {
                data_class: label.to_string(),
            }),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Audit => "AUDIT",
            Self::Phi => "PHI",
            Self::PiiIdentifying => "PII_IDENTIFYING",
            Self::Secret => "SECRET",
        }
    }
}

/// Parsed per-pack retention rule — one entry of the matrix's `packs:` map.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackRetentionRule {
    pub minimum_years: u32,
    pub worm_required: bool,
    /// The stricter floor that applies when the policy governs
    /// financial-services records, e.g. `pack-kr`'s `financial_services_years: 5`
    /// (vs. its plain `minimum_years: 3`). `None` for packs the matrix does
    /// not raise for financial-services data. See
    /// [`validate_retention_policy`]'s `is_financial_services` parameter.
    pub financial_services_years: Option<u32>,
}

/// Parsed per-class retention rule — one entry of the matrix's `classes:` map.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DataClassRetentionRule {
    pub preserve_merkle_proof: bool,
    pub minimum_years_override: Option<u32>,
    pub dsr_redaction_supported: bool,
    pub delete_payload_after_retention: bool,
}

/// Parsed matrix-wide defaults — the matrix's `defaults:` map.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionMatrixDefaults {
    pub hot_tier_days: u32,
    pub cold_tier_years: u32,
    pub dsr_redaction_grace_days: u32,
}

/// The already-parsed retention matrix this crate checks values against.
///
/// This struct — not a file path, not a YAML string — is how the matrix
/// enters the domain (L8: a pure domain crate does no I/O).
/// [`Self::audit_chain_canonical`] is the hardcoded mirror of
/// `audit/policy/retention-matrix.yaml` as it reads today; every number in
/// it is pinned by a test in this crate's `tests` module.
/// [`Self::from_parsed_rules`] exists for an adapter that has parsed a
/// (possibly different, e.g. a future revision of the same file) matrix
/// itself and wants this crate's checks applied to it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionMatrix {
    packs: BTreeMap<Pack, PackRetentionRule>,
    classes: BTreeMap<DataClass, DataClassRetentionRule>,
    defaults: RetentionMatrixDefaults,
}

impl RetentionMatrix {
    /// The hardcoded mirror of `audit/policy/retention-matrix.yaml`.
    pub fn audit_chain_canonical() -> Self {
        let mut packs = BTreeMap::new();
        packs.insert(
            Pack::Eu,
            PackRetentionRule {
                minimum_years: 2,
                worm_required: true,
                financial_services_years: None,
            },
        );
        packs.insert(
            Pack::Kr,
            PackRetentionRule {
                minimum_years: 3,
                worm_required: true,
                financial_services_years: Some(5),
            },
        );
        packs.insert(
            Pack::UsHealthcare,
            PackRetentionRule {
                minimum_years: 6,
                worm_required: true,
                financial_services_years: None,
            },
        );
        packs.insert(
            Pack::Jp,
            PackRetentionRule {
                minimum_years: 5,
                worm_required: true,
                financial_services_years: None,
            },
        );
        packs.insert(
            Pack::Sg,
            PackRetentionRule {
                minimum_years: 5,
                worm_required: true,
                financial_services_years: None,
            },
        );
        packs.insert(
            Pack::In,
            PackRetentionRule {
                minimum_years: 5,
                worm_required: true,
                financial_services_years: None,
            },
        );
        packs.insert(
            Pack::Br,
            PackRetentionRule {
                minimum_years: 5,
                worm_required: true,
                financial_services_years: None,
            },
        );
        packs.insert(
            Pack::Us,
            PackRetentionRule {
                minimum_years: 7,
                worm_required: true,
                financial_services_years: None,
            },
        );
        packs.insert(
            Pack::Au,
            PackRetentionRule {
                minimum_years: 7,
                worm_required: true,
                financial_services_years: None,
            },
        );
        packs.insert(
            Pack::Ae,
            PackRetentionRule {
                minimum_years: 7,
                worm_required: true,
                financial_services_years: None,
            },
        );
        packs.insert(
            Pack::Ksa,
            PackRetentionRule {
                minimum_years: 7,
                worm_required: true,
                financial_services_years: None,
            },
        );

        let mut classes = BTreeMap::new();
        classes.insert(
            DataClass::Audit,
            DataClassRetentionRule {
                preserve_merkle_proof: true,
                minimum_years_override: None,
                dsr_redaction_supported: false,
                delete_payload_after_retention: true,
            },
        );
        classes.insert(
            DataClass::Phi,
            DataClassRetentionRule {
                preserve_merkle_proof: true,
                minimum_years_override: Some(6),
                dsr_redaction_supported: false,
                delete_payload_after_retention: false,
            },
        );
        classes.insert(
            DataClass::PiiIdentifying,
            DataClassRetentionRule {
                preserve_merkle_proof: true,
                minimum_years_override: None,
                dsr_redaction_supported: true,
                delete_payload_after_retention: false,
            },
        );
        classes.insert(
            DataClass::Secret,
            DataClassRetentionRule {
                preserve_merkle_proof: true,
                minimum_years_override: None,
                dsr_redaction_supported: false,
                delete_payload_after_retention: false,
            },
        );

        Self {
            packs,
            classes,
            defaults: RetentionMatrixDefaults {
                hot_tier_days: 365,
                cold_tier_years: 7,
                dsr_redaction_grace_days: 30,
            },
        }
    }

    /// Build a matrix from adapter-supplied, already-parsed rules. Later
    /// entries for the same key win, matching how a YAML map would parse.
    ///
    /// This is a CHECKING constructor, not a bare struct assembly: an
    /// adapter can get most rule values wrong (a wrong `preserve_merkle_proof`,
    /// a wrong `minimum_years_override`, …) and the worst that happens is the
    /// matrix disagrees with reality — [`redact_payload`] and
    /// [`validate_retention_policy`] still apply whatever it says
    /// consistently. But two values are different in kind: a pack
    /// `minimum_years` of `0` or a `dsr_redaction_grace_days` of `0` do not
    /// merely disagree with reality, they make the corresponding check a
    /// no-op — EVERY policy clears a zero floor, EVERY cascade clears a
    /// zero-day grace period — which silently defeats the exact thing this
    /// crate exists to enforce. Those two are rejected here so a degenerate
    /// adapter-parsed matrix cannot construct at all, rather than construct
    /// successfully and enforce nothing.
    ///
    /// Every other rule field (class flags, `worm_required`,
    /// `financial_services_years`, `minimum_years_override`, …) is trusted
    /// verbatim: this crate has no independent source of truth to check them
    /// against, so the adapter that parsed `retention-matrix.yaml` remains
    /// responsible for THEIR fidelity to the file.
    ///
    /// # Errors
    /// - [`RetentionDomainError::DegenerateMatrixRule`] — some pack's
    ///   `minimum_years` is `0`, or `defaults.dsr_redaction_grace_days` is `0`.
    pub fn from_parsed_rules(
        packs: impl IntoIterator<Item = (Pack, PackRetentionRule)>,
        classes: impl IntoIterator<Item = (DataClass, DataClassRetentionRule)>,
        defaults: RetentionMatrixDefaults,
    ) -> Result<Self, RetentionDomainError> {
        let packs: BTreeMap<Pack, PackRetentionRule> = packs.into_iter().collect();
        for (pack, rule) in &packs {
            if rule.minimum_years == 0 {
                return Err(RetentionDomainError::DegenerateMatrixRule {
                    detail: format!(
                        "{}: minimum_years is 0, which would defeat the retention floor \
                         validate_retention_policy exists to enforce",
                        pack.as_str()
                    ),
                });
            }
        }
        if defaults.dsr_redaction_grace_days == 0 {
            return Err(RetentionDomainError::DegenerateMatrixRule {
                detail: "dsr_redaction_grace_days is 0, which would defeat the grace period \
                         plan_dsr_cascade exists to enforce"
                    .to_string(),
            });
        }

        Ok(Self {
            packs,
            classes: classes.into_iter().collect(),
            defaults,
        })
    }

    pub fn defaults(&self) -> RetentionMatrixDefaults {
        self.defaults
    }

    pub fn pack_rule(&self, pack: Pack) -> Option<PackRetentionRule> {
        self.packs.get(&pack).copied()
    }

    pub fn class_rule(&self, data_class: DataClass) -> Option<DataClassRetentionRule> {
        self.classes.get(&data_class).copied()
    }
}

/// Seconds in one 365.25-day Julian year (`365 * 86_400 + 21_600`). The
/// matrix expresses minima in whole years; this crate converts to seconds
/// using the 365.25-day average rather than a bare 365-day year so the
/// floor never UNDER-counts leap time — fail-closed per L4 doctrine, never
/// the more permissive of two plausible conversions.
pub const SECONDS_PER_YEAR: u64 = 365 * 86_400 + 21_600;

fn minimum_years_to_seconds(years: u32) -> u64 {
    u64::from(years) * SECONDS_PER_YEAR
}

/// A [`RetentionPolicy`] that has been checked against a [`RetentionMatrix`]
/// and found to meet or exceed its floor.
///
/// Fields are private and the only constructor is [`validate_retention_policy`],
/// so holding a `ValidatedRetentionPolicy` is proof the check ran — there is
/// no struct-literal path that bypasses it (L1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedRetentionPolicy {
    pack: Pack,
    data_class: DataClass,
    retention_seconds: u64,
    floor_seconds: u64,
    is_financial_services: bool,
}

impl ValidatedRetentionPolicy {
    pub fn pack(&self) -> Pack {
        self.pack
    }

    pub fn data_class(&self) -> DataClass {
        self.data_class
    }

    pub fn retention_seconds(&self) -> u64 {
        self.retention_seconds
    }

    pub fn floor_seconds(&self) -> u64 {
        self.floor_seconds
    }

    /// The `is_financial_services` flag [`validate_retention_policy`] was
    /// called with, i.e. whether the pack's stricter
    /// `financial_services_years` floor (where the pack has one) was applied.
    pub fn is_financial_services(&self) -> bool {
        self.is_financial_services
    }
}

/// Validate a [`RetentionPolicy`] against `matrix`, making
/// [`RetentionDomainError::PolicyShortenedBelowMinimum`] reachable.
///
/// The floor for `(policy.pack, policy.data_class)` starts at the pack's
/// `minimum_years`; if `is_financial_services` is `true` and the pack
/// declares a `financial_services_years` (only `pack-kr` does today, 3 → 5),
/// that value is used instead when it is stricter; the data class's
/// `minimum_years_override` (today only `PHI`, at 6 years) then raises the
/// floor again if IT is stricter still. The floor used is always the
/// GREATEST of whichever of those apply — `policy.retention_seconds` below
/// it, converted via [`SECONDS_PER_YEAR`], is rejected.
///
/// `is_financial_services` is caller-supplied because classifying a record
/// as financial-services business is not something `pack` or `data_class`
/// alone determine — the same shape of caller-supplied-context pattern
/// [`plan_dsr_cascade`] uses for `target_packs`.
///
/// # Errors
/// - [`RetentionDomainError::UnknownPack`] / [`RetentionDomainError::UnknownDataClass`] —
///   `policy.pack` / `policy.data_class` is not one of the matrix's closed labels.
/// - [`RetentionDomainError::PackNotInMatrix`] / [`RetentionDomainError::DataClassNotInMatrix`] —
///   the label parsed but `matrix` (if caller-supplied) has no rule for it.
/// - [`RetentionDomainError::PolicyShortenedBelowMinimum`] — `retention_seconds`
///   is below the applicable floor.
pub fn validate_retention_policy(
    policy: &RetentionPolicy,
    is_financial_services: bool,
    matrix: &RetentionMatrix,
) -> Result<ValidatedRetentionPolicy, RetentionDomainError> {
    let pack = Pack::parse(&policy.pack)?;
    let data_class = DataClass::parse(&policy.data_class)?;
    let pack_rule =
        matrix
            .pack_rule(pack)
            .ok_or_else(|| RetentionDomainError::PackNotInMatrix {
                pack: policy.pack.clone(),
            })?;
    let class_rule = matrix.class_rule(data_class).ok_or_else(|| {
        RetentionDomainError::DataClassNotInMatrix {
            data_class: policy.data_class.clone(),
        }
    })?;

    let pack_floor_years = if is_financial_services {
        pack_rule
            .financial_services_years
            .unwrap_or(pack_rule.minimum_years)
            .max(pack_rule.minimum_years)
    } else {
        pack_rule.minimum_years
    };
    let floor_years = match class_rule.minimum_years_override {
        Some(override_years) => pack_floor_years.max(override_years),
        None => pack_floor_years,
    };
    let floor_seconds = minimum_years_to_seconds(floor_years);

    if policy.retention_seconds < floor_seconds {
        return Err(RetentionDomainError::PolicyShortenedBelowMinimum);
    }

    Ok(ValidatedRetentionPolicy {
        pack,
        data_class,
        retention_seconds: policy.retention_seconds,
        floor_seconds,
        is_financial_services,
    })
}

/// Returns `true` if every character of `value` is ASCII alphanumeric or one
/// of `- _ . : @`, and `value` is non-empty.
///
/// This is a POSITIVE character class, not a denylist (L3): `str::trim`
/// strips only Unicode `White_Space`, so an identifier made purely of
/// invisible/format characters outside that property — U+200D ZWJ, U+FEFF
/// BOM, U+00AD SOFT HYPHEN, U+2062 INVISIBLE TIMES, a U+E00xx TAG character,
/// or an ASCII control character such as BEL/ESC — would survive
/// `.trim().is_empty()` unchanged, and no finite denylist of such characters
/// is exhaustive. Requiring every character to be in the small allowed set
/// rejects all of them in the same pass, along with any other Unicode letter
/// or digit outside ASCII, whitespace, and embedded newlines — see the
/// module doc's "Identifier normalization" section for why non-ASCII input
/// is rejected rather than merely allowed to pass through unnormalized.
fn is_normalized_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':' | '@'))
}

/// A [`DsrCascade`] that has been confined to one pack and cleared the
/// redaction grace period.
///
/// Fields are private; the only constructor is [`plan_dsr_cascade`] (L1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrCascadePlan {
    tenant_id: String,
    subject_id: String,
    source_microservice: String,
    pack: Pack,
    elapsed_days_since_request: u32,
}

impl DsrCascadePlan {
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn subject_id(&self) -> &str {
        &self.subject_id
    }

    pub fn source_microservice(&self) -> &str {
        &self.source_microservice
    }

    pub fn pack(&self) -> Pack {
        self.pack
    }

    pub fn elapsed_days_since_request(&self) -> u32 {
        self.elapsed_days_since_request
    }
}

/// Plan a [`DsrCascade`] against the packs it would touch, making
/// [`RetentionDomainError::CrossPackCascade`] reachable.
///
/// `target_packs` are the packs of the audit records the cascade's
/// `(tenant_id, subject_id)` matches — supplied by the caller because
/// discovering them is a query, not a domain computation. All of them must
/// be the same pack. `elapsed_days_since_request` is the number of days
/// since the DSR request was received, ALREADY COMPUTED by the caller — this
/// crate has no clock (no I/O) and never asks for one.
///
/// Every leg of the identity tuple is validated (L7): `tenant_id`,
/// `subject_id`, and `source_microservice` must each be a normalized
/// identifier per [`is_normalized_identifier`] (L2 / L3) — not merely
/// non-empty by `.trim()`.
///
/// # Errors
/// - [`RetentionDomainError::EmptyDsrTenantId`] / [`EmptyDsrSubjectId`](RetentionDomainError::EmptyDsrSubjectId) /
///   [`EmptyDsrSourceMicroservice`](RetentionDomainError::EmptyDsrSourceMicroservice) —
///   the corresponding [`DsrCascade`] field is not a normalized identifier.
/// - [`RetentionDomainError::EmptyCascadeTargets`] — `target_packs` is empty.
/// - [`RetentionDomainError::UnknownPack`] — a target pack label is not in the closed set.
/// - [`RetentionDomainError::CrossPackCascade`] — `target_packs` names more than one pack.
/// - [`RetentionDomainError::PackNotInMatrix`] — the confined pack parsed but
///   `matrix` (if caller-supplied) has no rule for it. Checked unconditionally,
///   the same way [`redact_payload`] looks up its class rule unconditionally
///   and [`validate_retention_policy`] looks up its pack rule unconditionally
///   — this is the destructive entry point of the three, and a pack the
///   matrix has never heard of (offboarded, or not yet onboarded) must not
///   yield an authorized cascade plan.
/// - [`RetentionDomainError::RedactionGracePeriodNotElapsed`] — fewer than
///   `matrix`'s `dsr_redaction_grace_days` have elapsed.
pub fn plan_dsr_cascade(
    cascade: &DsrCascade,
    target_packs: &[String],
    elapsed_days_since_request: u32,
    matrix: &RetentionMatrix,
) -> Result<DsrCascadePlan, RetentionDomainError> {
    if !is_normalized_identifier(&cascade.tenant_id) {
        return Err(RetentionDomainError::EmptyDsrTenantId);
    }
    if !is_normalized_identifier(&cascade.subject_id) {
        return Err(RetentionDomainError::EmptyDsrSubjectId);
    }
    if !is_normalized_identifier(&cascade.source_microservice) {
        return Err(RetentionDomainError::EmptyDsrSourceMicroservice);
    }
    if target_packs.is_empty() {
        return Err(RetentionDomainError::EmptyCascadeTargets);
    }

    let mut parsed_packs = Vec::with_capacity(target_packs.len());
    for label in target_packs {
        parsed_packs.push(Pack::parse(label)?);
    }
    let confined_pack = parsed_packs[0];
    if parsed_packs.iter().any(|&pack| pack != confined_pack) {
        return Err(RetentionDomainError::CrossPackCascade);
    }
    if matrix.pack_rule(confined_pack).is_none() {
        return Err(RetentionDomainError::PackNotInMatrix {
            pack: confined_pack.as_str().to_string(),
        });
    }

    let grace_days = matrix.defaults().dsr_redaction_grace_days;
    if elapsed_days_since_request < grace_days {
        return Err(RetentionDomainError::RedactionGracePeriodNotElapsed {
            required_days: grace_days,
            elapsed_days: elapsed_days_since_request,
        });
    }

    Ok(DsrCascadePlan {
        tenant_id: cascade.tenant_id.clone(),
        subject_id: cascade.subject_id.clone(),
        source_microservice: cascade.source_microservice.clone(),
        pack: confined_pack,
        elapsed_days_since_request,
    })
}

/// Closed set of accepted [`RedactionToken`] reasons. Free text is never
/// accepted — a reason outside this set (including one built entirely from
/// invisible characters, which cannot match either literal) is
/// [`RetentionDomainError::InvalidRedactionReason`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedactionReason {
    /// The subject exercised a data-subject-rights erasure request; gated
    /// by [`DataClassRetentionRule::dsr_redaction_supported`].
    DsrErasureRequest,
    /// The retention window elapsed and the payload is eligible for
    /// deletion, independent of any DSR request.
    RetentionExpired,
}

impl RedactionReason {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "dsr_erasure_request" => Some(Self::DsrErasureRequest),
            "retention_expired" => Some(Self::RetentionExpired),
            _ => None,
        }
    }
}

/// The effect of applying one [`RedactionToken`]: the payload is erased (if
/// the class's `delete_payload_after_retention` / DSR support authorizes it)
/// but the Merkle proof of the original event's existence is always
/// retained.
///
/// [`redact_payload`] is the only constructor, and it looks up and checks
/// `class_rule.preserve_merkle_proof` BEFORE constructing a value of this
/// type — see its body. So [`Self::merkle_proof_preserved`] returning `true`
/// unconditionally is not an unchecked assumption: by the time any
/// `RedactionEffect` exists, `redact_payload` has already refused to build
/// one under a matrix that set the flag to `false`
/// ([`RetentionDomainError::MerkleProofPreservationNotConfigured`]). Fields
/// are private; there is no other constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedactionEffect {
    audit_id: String,
    payload_erased: bool,
}

impl RedactionEffect {
    pub fn audit_id(&self) -> &str {
        &self.audit_id
    }

    /// Whether the payload was actually erased. `false` for a
    /// [`RedactionReason::RetentionExpired`] redaction against a class whose
    /// `delete_payload_after_retention` is `false` — the token is still
    /// accepted (nothing about it was invalid), but the matrix says this
    /// class is not deleted on retention expiry, so nothing was erased.
    pub fn payload_erased(&self) -> bool {
        self.payload_erased
    }

    /// Always `true` — see the type-level doc: `redact_payload` refuses to
    /// construct this type at all under a matrix that would make it `false`.
    pub fn merkle_proof_preserved(&self) -> bool {
        true
    }
}

/// Apply a [`RedactionToken`] to an event of `data_class`, making
/// [`RetentionDomainError::InvalidRedactionReason`],
/// [`RetentionDomainError::DsrRedactionUnsupportedForClass`], and
/// [`RetentionDomainError::MerkleProofPreservationNotConfigured`] reachable.
///
/// The class rule is looked up UNCONDITIONALLY for every reason (not only
/// [`RedactionReason::DsrErasureRequest`]), because [`RedactionEffect`]'s
/// proof-preservation guarantee applies to every redaction regardless of
/// reason, and because a [`RedactionReason::RetentionExpired`] redaction
/// must consult `delete_payload_after_retention` to decide whether it
/// actually erases anything.
///
/// # Errors
/// - [`RetentionDomainError::EmptyRedactionAuditId`] — `token.audit_id` is
///   not a normalized identifier.
/// - [`RetentionDomainError::InvalidRedactionReason`] — `token.reason` is not
///   one of [`RedactionReason`]'s closed set, or `token.lawful_basis` is not
///   a normalized identifier.
/// - [`RetentionDomainError::DataClassNotInMatrix`] — `data_class` parsed but
///   `matrix` (if caller-supplied) has no rule for it.
/// - [`RetentionDomainError::MerkleProofPreservationNotConfigured`] —
///   `data_class`'s matrix entry does not set `preserve_merkle_proof: true`.
/// - [`RetentionDomainError::DsrRedactionUnsupportedForClass`] — the reason is
///   [`RedactionReason::DsrErasureRequest`] but `data_class`'s matrix entry
///   does not set `dsr_redaction_supported: true` (today only
///   `PII_IDENTIFYING` does — `AUDIT`, `PHI`, and `SECRET` all reject).
pub fn redact_payload(
    token: &RedactionToken,
    data_class: DataClass,
    matrix: &RetentionMatrix,
) -> Result<RedactionEffect, RetentionDomainError> {
    if !is_normalized_identifier(&token.audit_id) {
        return Err(RetentionDomainError::EmptyRedactionAuditId);
    }
    let reason = RedactionReason::parse(&token.reason)
        .ok_or(RetentionDomainError::InvalidRedactionReason)?;
    if !is_normalized_identifier(&token.lawful_basis) {
        return Err(RetentionDomainError::InvalidRedactionReason);
    }

    let class_rule = matrix.class_rule(data_class).ok_or_else(|| {
        RetentionDomainError::DataClassNotInMatrix {
            data_class: data_class.as_str().to_string(),
        }
    })?;

    if !class_rule.preserve_merkle_proof {
        return Err(RetentionDomainError::MerkleProofPreservationNotConfigured {
            data_class: data_class.as_str().to_string(),
        });
    }

    let payload_erased = match reason {
        RedactionReason::DsrErasureRequest => {
            if !class_rule.dsr_redaction_supported {
                return Err(RetentionDomainError::DsrRedactionUnsupportedForClass {
                    data_class: data_class.as_str().to_string(),
                });
            }
            true
        }
        RedactionReason::RetentionExpired => class_rule.delete_payload_after_retention,
    };

    Ok(RedactionEffect {
        audit_id: token.audit_id.clone(),
        payload_erased,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(pack: &str, data_class: &str, retention_seconds: u64) -> RetentionPolicy {
        RetentionPolicy {
            pack: pack.to_string(),
            data_class: data_class.to_string(),
            retention_seconds,
        }
    }

    fn cascade(tenant_id: &str, subject_id: &str, source_microservice: &str) -> DsrCascade {
        DsrCascade {
            tenant_id: tenant_id.to_string(),
            subject_id: subject_id.to_string(),
            source_microservice: source_microservice.to_string(),
        }
    }

    fn token(audit_id: &str, reason: &str, lawful_basis: &str) -> RedactionToken {
        RedactionToken {
            audit_id: audit_id.to_string(),
            reason: reason.to_string(),
            lawful_basis: lawful_basis.to_string(),
        }
    }

    // --- matrix numbers pinned against audit/policy/retention-matrix.yaml ---

    #[test]
    fn matrix_defaults_match_policy_file() {
        let defaults = RetentionMatrix::audit_chain_canonical().defaults();
        assert_eq!(defaults.hot_tier_days, 365);
        assert_eq!(defaults.cold_tier_years, 7);
        assert_eq!(defaults.dsr_redaction_grace_days, 30);
    }

    #[test]
    fn matrix_pack_minimums_match_policy_file() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let expected = [
            (Pack::Eu, 2),
            (Pack::Kr, 3),
            (Pack::UsHealthcare, 6),
            (Pack::Jp, 5),
            (Pack::Sg, 5),
            (Pack::In, 5),
            (Pack::Br, 5),
            (Pack::Us, 7),
            (Pack::Au, 7),
            (Pack::Ae, 7),
            (Pack::Ksa, 7),
        ];
        for (pack, minimum_years) in expected {
            assert_eq!(
                matrix.pack_rule(pack).unwrap().minimum_years,
                minimum_years,
                "{pack:?} minimum_years"
            );
            assert!(matrix.pack_rule(pack).unwrap().worm_required);
        }
    }

    #[test]
    fn matrix_pack_kr_financial_services_years_matches_policy_file() {
        // pack-kr is the ONLY pack with a financial_services_years override
        // in audit/policy/retention-matrix.yaml (5, vs. its plain
        // minimum_years of 3). Every other pack must have None.
        let matrix = RetentionMatrix::audit_chain_canonical();
        assert_eq!(
            matrix.pack_rule(Pack::Kr).unwrap().financial_services_years,
            Some(5)
        );
        for pack in [
            Pack::Eu,
            Pack::Us,
            Pack::UsHealthcare,
            Pack::Jp,
            Pack::Sg,
            Pack::Au,
            Pack::In,
            Pack::Br,
            Pack::Ae,
            Pack::Ksa,
        ] {
            assert_eq!(
                matrix.pack_rule(pack).unwrap().financial_services_years,
                None,
                "{pack:?} financial_services_years"
            );
        }
    }

    #[test]
    fn matrix_class_rules_match_policy_file() {
        let matrix = RetentionMatrix::audit_chain_canonical();

        let audit = matrix.class_rule(DataClass::Audit).unwrap();
        assert!(audit.preserve_merkle_proof);
        assert_eq!(audit.minimum_years_override, None);
        assert!(!audit.dsr_redaction_supported);
        assert!(audit.delete_payload_after_retention);

        let phi = matrix.class_rule(DataClass::Phi).unwrap();
        assert!(phi.preserve_merkle_proof);
        assert_eq!(phi.minimum_years_override, Some(6));
        assert!(!phi.dsr_redaction_supported);
        assert!(!phi.delete_payload_after_retention);

        let pii = matrix.class_rule(DataClass::PiiIdentifying).unwrap();
        assert!(pii.preserve_merkle_proof);
        assert_eq!(pii.minimum_years_override, None);
        assert!(pii.dsr_redaction_supported);
        assert!(!pii.delete_payload_after_retention);

        let secret = matrix.class_rule(DataClass::Secret).unwrap();
        assert!(secret.preserve_merkle_proof);
        assert_eq!(secret.minimum_years_override, None);
        assert!(!secret.dsr_redaction_supported);
        assert!(!secret.delete_payload_after_retention);
    }

    #[test]
    fn every_matrix_class_preserves_merkle_proof() {
        // The critical invariant, asserted directly over the matrix data:
        // there is no data class anywhere in the policy for which redaction
        // is allowed to drop proof material.
        let matrix = RetentionMatrix::audit_chain_canonical();
        for class in [
            DataClass::Audit,
            DataClass::Phi,
            DataClass::PiiIdentifying,
            DataClass::Secret,
        ] {
            assert!(matrix.class_rule(class).unwrap().preserve_merkle_proof);
        }
    }

    #[test]
    fn seconds_per_year_is_365_25_days() {
        assert_eq!(SECONDS_PER_YEAR, 31_557_600);
    }

    // --- (a) retention policy validation ---

    #[test]
    fn policy_at_pack_floor_is_accepted() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let floor = minimum_years_to_seconds(5); // pack-jp, no class override
        let validated =
            validate_retention_policy(&policy("pack-jp", "AUDIT", floor), false, &matrix).unwrap();
        assert_eq!(validated.pack(), Pack::Jp);
        assert_eq!(validated.data_class(), DataClass::Audit);
        assert_eq!(validated.floor_seconds(), floor);
        assert!(!validated.is_financial_services());
    }

    #[test]
    fn policy_one_second_under_pack_floor_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let floor = minimum_years_to_seconds(5); // pack-jp, no class override
        let result =
            validate_retention_policy(&policy("pack-jp", "AUDIT", floor - 1), false, &matrix);
        assert_eq!(
            result,
            Err(RetentionDomainError::PolicyShortenedBelowMinimum)
        );
    }

    #[test]
    fn class_override_wins_when_stricter_than_pack_minimum() {
        // pack-eu minimum_years=2, PHI minimum_years_override=6: floor is 6.
        let matrix = RetentionMatrix::audit_chain_canonical();
        let floor = minimum_years_to_seconds(6);
        assert!(
            validate_retention_policy(&policy("pack-eu", "PHI", floor), false, &matrix).is_ok()
        );
        let result =
            validate_retention_policy(&policy("pack-eu", "PHI", floor - 1), false, &matrix);
        assert_eq!(
            result,
            Err(RetentionDomainError::PolicyShortenedBelowMinimum)
        );
    }

    #[test]
    fn pack_minimum_wins_when_stricter_than_class_override() {
        // pack-us minimum_years=7, PHI minimum_years_override=6: floor is 7,
        // not 6 — proves the rule is the GREATER of the two, not "override
        // always wins".
        let matrix = RetentionMatrix::audit_chain_canonical();
        let floor = minimum_years_to_seconds(7);
        assert!(
            validate_retention_policy(&policy("pack-us", "PHI", floor), false, &matrix).is_ok()
        );
        let result =
            validate_retention_policy(&policy("pack-us", "PHI", floor - 1), false, &matrix);
        assert_eq!(
            result,
            Err(RetentionDomainError::PolicyShortenedBelowMinimum)
        );
    }

    #[test]
    fn financial_services_flag_raises_pack_kr_floor_from_3_years_to_5() {
        // pack-kr minimum_years=3, financial_services_years=5. A policy at
        // exactly 3 years passes when is_financial_services=false but is
        // rejected when is_financial_services=true — the flag must WIDEN the
        // effective floor, never narrow it (L4: fail closed).
        let matrix = RetentionMatrix::audit_chain_canonical();
        let three_years = minimum_years_to_seconds(3);
        assert!(
            validate_retention_policy(&policy("pack-kr", "AUDIT", three_years), false, &matrix)
                .is_ok()
        );
        assert_eq!(
            validate_retention_policy(&policy("pack-kr", "AUDIT", three_years), true, &matrix),
            Err(RetentionDomainError::PolicyShortenedBelowMinimum)
        );

        let five_years = minimum_years_to_seconds(5);
        let validated =
            validate_retention_policy(&policy("pack-kr", "AUDIT", five_years), true, &matrix)
                .unwrap();
        assert_eq!(validated.floor_seconds(), five_years);
        assert!(validated.is_financial_services());
        let result =
            validate_retention_policy(&policy("pack-kr", "AUDIT", five_years - 1), true, &matrix);
        assert_eq!(
            result,
            Err(RetentionDomainError::PolicyShortenedBelowMinimum)
        );
    }

    #[test]
    fn financial_services_flag_has_no_effect_on_a_pack_without_an_override() {
        // pack-eu has no financial_services_years, so is_financial_services
        // must fall back to the plain minimum_years (2), not raise it.
        let matrix = RetentionMatrix::audit_chain_canonical();
        let two_years = minimum_years_to_seconds(2);
        let validated =
            validate_retention_policy(&policy("pack-eu", "AUDIT", two_years), true, &matrix)
                .unwrap();
        assert_eq!(validated.floor_seconds(), two_years);
    }

    #[test]
    fn unknown_pack_label_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result =
            validate_retention_policy(&policy("pack-atlantis", "AUDIT", u64::MAX), false, &matrix);
        assert_eq!(
            result,
            Err(RetentionDomainError::UnknownPack {
                pack: "pack-atlantis".to_string()
            })
        );
    }

    #[test]
    fn unknown_data_class_label_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result =
            validate_retention_policy(&policy("pack-eu", "TOP_SECRET", u64::MAX), false, &matrix);
        assert_eq!(
            result,
            Err(RetentionDomainError::UnknownDataClass {
                data_class: "TOP_SECRET".to_string()
            })
        );
    }

    #[test]
    fn pack_not_in_caller_supplied_matrix_is_rejected() {
        let matrix = RetentionMatrix::from_parsed_rules(
            [],
            [(
                DataClass::Audit,
                DataClassRetentionRule {
                    preserve_merkle_proof: true,
                    minimum_years_override: None,
                    dsr_redaction_supported: false,
                    delete_payload_after_retention: true,
                },
            )],
            RetentionMatrixDefaults {
                hot_tier_days: 365,
                cold_tier_years: 7,
                dsr_redaction_grace_days: 30,
            },
        )
        .unwrap();
        let result =
            validate_retention_policy(&policy("pack-eu", "AUDIT", u64::MAX), false, &matrix);
        assert_eq!(
            result,
            Err(RetentionDomainError::PackNotInMatrix {
                pack: "pack-eu".to_string()
            })
        );
    }

    #[test]
    fn from_parsed_rules_rejects_a_zero_minimum_years_pack() {
        // A zero-year floor would make every policy against that pack clear
        // validate_retention_policy's check for free — exactly the defeat
        // from_parsed_rules must refuse to construct.
        let result = RetentionMatrix::from_parsed_rules(
            [(
                Pack::Us,
                PackRetentionRule {
                    minimum_years: 0,
                    worm_required: false,
                    financial_services_years: None,
                },
            )],
            [],
            RetentionMatrixDefaults {
                hot_tier_days: 365,
                cold_tier_years: 7,
                dsr_redaction_grace_days: 30,
            },
        );
        assert_eq!(
            result,
            Err(RetentionDomainError::DegenerateMatrixRule {
                detail: "pack-us: minimum_years is 0, which would defeat the retention floor \
                         validate_retention_policy exists to enforce"
                    .to_string()
            })
        );
    }

    #[test]
    fn from_parsed_rules_rejects_a_zero_dsr_redaction_grace_days_default() {
        // A zero-day grace period would let plan_dsr_cascade accept a
        // same-day cascade unconditionally — the grace-period check would
        // never fire.
        let result = RetentionMatrix::from_parsed_rules(
            [],
            [],
            RetentionMatrixDefaults {
                hot_tier_days: 365,
                cold_tier_years: 7,
                dsr_redaction_grace_days: 0,
            },
        );
        assert_eq!(
            result,
            Err(RetentionDomainError::DegenerateMatrixRule {
                detail: "dsr_redaction_grace_days is 0, which would defeat the grace period \
                         plan_dsr_cascade exists to enforce"
                    .to_string()
            })
        );
    }

    #[test]
    fn from_parsed_rules_accepts_a_well_formed_matrix() {
        // Sanity companion to the two rejection tests above: a matrix with a
        // real floor and a real grace period still constructs.
        let matrix = RetentionMatrix::from_parsed_rules(
            [(
                Pack::Us,
                PackRetentionRule {
                    minimum_years: 7,
                    worm_required: true,
                    financial_services_years: None,
                },
            )],
            [],
            RetentionMatrixDefaults {
                hot_tier_days: 365,
                cold_tier_years: 7,
                dsr_redaction_grace_days: 30,
            },
        );
        assert!(matrix.is_ok());
    }

    // --- (b) DSR cascade rules ---

    #[test]
    fn single_pack_cascade_after_grace_period_is_accepted() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let plan = plan_dsr_cascade(
            &cascade("tenant-1", "subject-1", "billing-service"),
            &["pack-eu".to_string(), "pack-eu".to_string()],
            30,
            &matrix,
        )
        .unwrap();
        assert_eq!(plan.pack(), Pack::Eu);
        assert_eq!(plan.tenant_id(), "tenant-1");
        assert_eq!(plan.subject_id(), "subject-1");
        assert_eq!(plan.source_microservice(), "billing-service");
        assert_eq!(plan.elapsed_days_since_request(), 30);
    }

    #[test]
    fn cascade_crossing_packs_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = plan_dsr_cascade(
            &cascade("tenant-1", "subject-1", "billing-service"),
            &["pack-eu".to_string(), "pack-jp".to_string()],
            30,
            &matrix,
        );
        assert_eq!(result, Err(RetentionDomainError::CrossPackCascade));
    }

    #[test]
    fn cascade_with_unknown_pack_label_is_rejected() {
        // UnknownPack is reachable through plan_dsr_cascade's own target-pack
        // parsing, not only through validate_retention_policy — a bad label
        // here must not be swallowed as CrossPackCascade or silently ignored.
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = plan_dsr_cascade(
            &cascade("tenant-1", "subject-1", "billing-service"),
            &["pack-atlantis".to_string()],
            30,
            &matrix,
        );
        assert_eq!(
            result,
            Err(RetentionDomainError::UnknownPack {
                pack: "pack-atlantis".to_string()
            })
        );
    }

    #[test]
    fn cascade_against_a_pack_absent_from_the_matrix_is_rejected() {
        // Same fail-open shape as the retention_expired / DataClassNotInMatrix
        // finding, on the OTHER destructive entry point: a caller-supplied
        // matrix that has never heard of a pack (offboarded, or not yet
        // onboarded) must not yield an authorized DSR cascade plan for it,
        // even though plan_dsr_cascade otherwise only reads
        // `matrix.defaults()`. Confirm the matrix genuinely lacks the pack
        // first, so this test cannot pass for the wrong reason.
        let matrix = RetentionMatrix::from_parsed_rules(
            [],
            [],
            RetentionMatrixDefaults {
                hot_tier_days: 365,
                cold_tier_years: 7,
                dsr_redaction_grace_days: 30,
            },
        )
        .unwrap();
        assert!(matrix.pack_rule(Pack::Eu).is_none());

        let result = plan_dsr_cascade(
            &cascade("tenant-1", "subject-1", "billing-service"),
            &["pack-eu".to_string()],
            30,
            &matrix,
        );
        assert_eq!(
            result,
            Err(RetentionDomainError::PackNotInMatrix {
                pack: "pack-eu".to_string()
            })
        );
    }

    #[test]
    fn cascade_before_grace_period_elapses_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = plan_dsr_cascade(
            &cascade("tenant-1", "subject-1", "billing-service"),
            &["pack-eu".to_string()],
            29,
            &matrix,
        );
        assert_eq!(
            result,
            Err(RetentionDomainError::RedactionGracePeriodNotElapsed {
                required_days: 30,
                elapsed_days: 29,
            })
        );
    }

    #[test]
    fn cascade_with_no_target_packs_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = plan_dsr_cascade(
            &cascade("tenant-1", "subject-1", "billing-service"),
            &[],
            30,
            &matrix,
        );
        assert_eq!(result, Err(RetentionDomainError::EmptyCascadeTargets));
    }

    #[test]
    fn cascade_with_empty_tenant_id_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = plan_dsr_cascade(
            &cascade("", "subject-1", "billing-service"),
            &["pack-eu".to_string()],
            30,
            &matrix,
        );
        assert_eq!(result, Err(RetentionDomainError::EmptyDsrTenantId));
    }

    #[test]
    fn cascade_with_invisible_only_tenant_id_is_rejected() {
        // L3: a tenant_id made purely of invisible/format characters must be
        // rejected even though none of them is Unicode `White_Space`, so
        // `.trim().is_empty()` would say it's non-empty. Covers ZWSP, BOM,
        // word-joiner and NUL (the original denylist) PLUS the characters an
        // enumerated denylist misses: ZWJ, ZWNJ, soft hyphen, invisible
        // times, and a plane-14 tag character.
        let matrix = RetentionMatrix::audit_chain_canonical();
        let invisible_only = [
            "\u{200B}\u{FEFF}\u{2060}\0", // ZWSP + BOM + word joiner + NUL
            "\u{200D}",                   // ZERO WIDTH JOINER
            "\u{200C}",                   // ZERO WIDTH NON-JOINER
            "\u{00AD}",                   // SOFT HYPHEN
            "\u{2062}",                   // INVISIBLE TIMES
            "\u{E0041}",                  // TAG LATIN CAPITAL LETTER A
            "\u{0007}",                   // BEL
            "\u{001B}",                   // ESC
        ];
        for tenant_id in invisible_only {
            let result = plan_dsr_cascade(
                &cascade(tenant_id, "subject-1", "billing-service"),
                &["pack-eu".to_string()],
                30,
                &matrix,
            );
            assert_eq!(
                result,
                Err(RetentionDomainError::EmptyDsrTenantId),
                "tenant_id {tenant_id:?}"
            );
        }
    }

    #[test]
    fn cascade_with_non_normalized_tenant_id_is_rejected_not_silently_aliased() {
        // L2: every one of these spellings must be REJECTED, not silently
        // accepted as some distinct partition — a redaction domain that
        // accepted all six would key one tenant's data under six different
        // stored identifiers.
        let matrix = RetentionMatrix::audit_chain_canonical();
        let non_normalized = [
            " tenant-1",
            "tenant-1 ",
            "tenant-1\n",
            "tenant-1\u{200B}",
            "\u{FEFF}tenant-1",
            "tenant 1",
        ];
        for tenant_id in non_normalized {
            let result = plan_dsr_cascade(
                &cascade(tenant_id, "subject-1", "billing-service"),
                &["pack-eu".to_string()],
                30,
                &matrix,
            );
            assert_eq!(
                result,
                Err(RetentionDomainError::EmptyDsrTenantId),
                "tenant_id {tenant_id:?}"
            );
        }
        // The one normalized spelling is still accepted, and is the only
        // stored value for it.
        let plan = plan_dsr_cascade(
            &cascade("tenant-1", "subject-1", "billing-service"),
            &["pack-eu".to_string()],
            30,
            &matrix,
        )
        .unwrap();
        assert_eq!(plan.tenant_id(), "tenant-1");
    }

    #[test]
    fn cascade_with_empty_subject_id_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = plan_dsr_cascade(
            &cascade("tenant-1", "  ", "billing-service"),
            &["pack-eu".to_string()],
            30,
            &matrix,
        );
        assert_eq!(result, Err(RetentionDomainError::EmptyDsrSubjectId));
    }

    #[test]
    fn cascade_with_empty_source_microservice_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = plan_dsr_cascade(
            &cascade("tenant-1", "subject-1", ""),
            &["pack-eu".to_string()],
            30,
            &matrix,
        );
        assert_eq!(
            result,
            Err(RetentionDomainError::EmptyDsrSourceMicroservice)
        );
    }

    // --- (c) redaction token rules ---

    #[test]
    fn redaction_with_empty_reason_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = redact_payload(
            &token("audit-1", "", "gdpr_article_17"),
            DataClass::PiiIdentifying,
            &matrix,
        );
        assert_eq!(result, Err(RetentionDomainError::InvalidRedactionReason));
    }

    #[test]
    fn redaction_with_unknown_reason_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = redact_payload(
            &token("audit-1", "because_i_felt_like_it", "gdpr_article_17"),
            DataClass::PiiIdentifying,
            &matrix,
        );
        assert_eq!(result, Err(RetentionDomainError::InvalidRedactionReason));
    }

    #[test]
    fn redaction_with_invisible_only_reason_is_rejected() {
        // L3: an invisible-character reason cannot match either literal in
        // the closed set, so it falls through to the unknown-reason branch.
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = redact_payload(
            &token("audit-1", "\u{200B}\u{FEFF}", "gdpr_article_17"),
            DataClass::PiiIdentifying,
            &matrix,
        );
        assert_eq!(result, Err(RetentionDomainError::InvalidRedactionReason));
    }

    #[test]
    fn redaction_with_missing_lawful_basis_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = redact_payload(
            &token("audit-1", "dsr_erasure_request", ""),
            DataClass::PiiIdentifying,
            &matrix,
        );
        assert_eq!(result, Err(RetentionDomainError::InvalidRedactionReason));
    }

    #[test]
    fn redaction_with_invisible_only_lawful_basis_is_rejected() {
        // L3/L1 parity with the DSR cascade identifiers: every invisible or
        // format character that an enumerated denylist would miss.
        let matrix = RetentionMatrix::audit_chain_canonical();
        for lawful_basis in ["\u{2060}", "\u{200D}", "\u{00AD}", "\u{E0041}", "\u{001B}"] {
            let result = redact_payload(
                &token("audit-1", "dsr_erasure_request", lawful_basis),
                DataClass::PiiIdentifying,
                &matrix,
            );
            assert_eq!(
                result,
                Err(RetentionDomainError::InvalidRedactionReason),
                "lawful_basis {lawful_basis:?}"
            );
        }
    }

    #[test]
    fn redaction_with_non_normalized_lawful_basis_is_rejected() {
        // L2: whitespace-padded lawful_basis values must not be silently
        // accepted as distinct from the normalized spelling.
        let matrix = RetentionMatrix::audit_chain_canonical();
        for lawful_basis in [" gdpr_article_17", "gdpr_article_17 ", "gdpr article 17"] {
            let result = redact_payload(
                &token("audit-1", "dsr_erasure_request", lawful_basis),
                DataClass::PiiIdentifying,
                &matrix,
            );
            assert_eq!(
                result,
                Err(RetentionDomainError::InvalidRedactionReason),
                "lawful_basis {lawful_basis:?}"
            );
        }
    }

    #[test]
    fn redaction_with_empty_audit_id_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = redact_payload(
            &token("", "dsr_erasure_request", "gdpr_article_17"),
            DataClass::PiiIdentifying,
            &matrix,
        );
        assert_eq!(result, Err(RetentionDomainError::EmptyRedactionAuditId));
    }

    #[test]
    fn redaction_with_non_normalized_audit_id_is_rejected() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        for audit_id in [" audit-1", "audit-1\u{200B}", "audit 1"] {
            let result = redact_payload(
                &token(audit_id, "dsr_erasure_request", "gdpr_article_17"),
                DataClass::PiiIdentifying,
                &matrix,
            );
            assert_eq!(
                result,
                Err(RetentionDomainError::EmptyRedactionAuditId),
                "audit_id {audit_id:?}"
            );
        }
    }

    #[test]
    fn valid_dsr_redaction_preserves_merkle_proof() {
        // The critical invariant, proven against the actually-computed
        // effect rather than a fabricated shape (L5).
        let matrix = RetentionMatrix::audit_chain_canonical();
        let effect = redact_payload(
            &token("audit-1", "dsr_erasure_request", "gdpr_article_17"),
            DataClass::PiiIdentifying,
            &matrix,
        )
        .unwrap();
        assert!(effect.merkle_proof_preserved());
        assert!(effect.payload_erased());
        assert_eq!(effect.audit_id(), "audit-1");
    }

    #[test]
    fn retention_expired_redaction_preserves_merkle_proof_too() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let effect = redact_payload(
            &token("audit-2", "retention_expired", "retention_window_elapsed"),
            DataClass::Audit,
            &matrix,
        )
        .unwrap();
        assert!(effect.merkle_proof_preserved());
        // AUDIT is the ONE class with delete_payload_after_retention: true.
        assert!(effect.payload_erased());
    }

    #[test]
    fn retention_expired_redaction_does_not_erase_payload_for_classes_that_forbid_it() {
        // PHI, PII_IDENTIFYING and SECRET all set
        // delete_payload_after_retention: false in the matrix — a
        // retention_expired token against them must be ACCEPTED (there is
        // nothing wrong with the token) but must NOT erase the payload.
        let matrix = RetentionMatrix::audit_chain_canonical();
        for class in [DataClass::Phi, DataClass::PiiIdentifying, DataClass::Secret] {
            let effect = redact_payload(
                &token("audit-3", "retention_expired", "retention_window_elapsed"),
                class,
                &matrix,
            )
            .unwrap();
            assert!(!effect.payload_erased(), "{class:?} payload_erased");
            assert!(
                effect.merkle_proof_preserved(),
                "{class:?} merkle_proof_preserved"
            );
        }
    }

    #[test]
    fn retention_expired_redaction_for_class_absent_from_matrix_is_rejected() {
        // The matrix lookup must run unconditionally, even for
        // RetentionExpired — a class the supplied matrix never heard of must
        // not silently authorize erasure.
        let matrix = RetentionMatrix::from_parsed_rules(
            [],
            [],
            RetentionMatrixDefaults {
                hot_tier_days: 365,
                cold_tier_years: 7,
                dsr_redaction_grace_days: 30,
            },
        )
        .unwrap();
        let result = redact_payload(
            &token("audit-4", "retention_expired", "retention_window_elapsed"),
            DataClass::Audit,
            &matrix,
        );
        assert_eq!(
            result,
            Err(RetentionDomainError::DataClassNotInMatrix {
                data_class: "AUDIT".to_string()
            })
        );
    }

    #[test]
    fn redaction_is_refused_when_matrix_declares_preserve_merkle_proof_false() {
        // The critical invariant, enforced rather than assumed: a
        // caller-supplied matrix that contradicts
        // "preserve_merkle_proof: true" must make redact_payload refuse to
        // build a RedactionEffect at all — for EITHER reason.
        let matrix = RetentionMatrix::from_parsed_rules(
            [],
            [(
                DataClass::PiiIdentifying,
                DataClassRetentionRule {
                    preserve_merkle_proof: false,
                    minimum_years_override: None,
                    dsr_redaction_supported: true,
                    delete_payload_after_retention: false,
                },
            )],
            RetentionMatrixDefaults {
                hot_tier_days: 365,
                cold_tier_years: 7,
                dsr_redaction_grace_days: 30,
            },
        )
        .unwrap();
        let expected = Err(RetentionDomainError::MerkleProofPreservationNotConfigured {
            data_class: "PII_IDENTIFYING".to_string(),
        });
        assert_eq!(
            redact_payload(
                &token("audit-5", "dsr_erasure_request", "gdpr_article_17"),
                DataClass::PiiIdentifying,
                &matrix,
            ),
            expected
        );
        assert_eq!(
            redact_payload(
                &token("audit-5", "retention_expired", "retention_window_elapsed"),
                DataClass::PiiIdentifying,
                &matrix,
            ),
            expected
        );
    }

    // --- (d) dsr_redaction_supported gating ---

    #[test]
    fn dsr_redaction_is_rejected_for_audit_class() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = redact_payload(
            &token("audit-1", "dsr_erasure_request", "gdpr_article_17"),
            DataClass::Audit,
            &matrix,
        );
        assert_eq!(
            result,
            Err(RetentionDomainError::DsrRedactionUnsupportedForClass {
                data_class: "AUDIT".to_string()
            })
        );
    }

    #[test]
    fn dsr_redaction_is_rejected_for_phi_class() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = redact_payload(
            &token("audit-1", "dsr_erasure_request", "gdpr_article_17"),
            DataClass::Phi,
            &matrix,
        );
        assert_eq!(
            result,
            Err(RetentionDomainError::DsrRedactionUnsupportedForClass {
                data_class: "PHI".to_string()
            })
        );
    }

    #[test]
    fn dsr_redaction_is_rejected_for_secret_class() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = redact_payload(
            &token("audit-1", "dsr_erasure_request", "gdpr_article_17"),
            DataClass::Secret,
            &matrix,
        );
        assert_eq!(
            result,
            Err(RetentionDomainError::DsrRedactionUnsupportedForClass {
                data_class: "SECRET".to_string()
            })
        );
    }

    #[test]
    fn dsr_redaction_is_accepted_for_pii_identifying_class() {
        let matrix = RetentionMatrix::audit_chain_canonical();
        let result = redact_payload(
            &token("audit-1", "dsr_erasure_request", "gdpr_article_17"),
            DataClass::PiiIdentifying,
            &matrix,
        );
        assert!(result.is_ok());
    }

    // --- every error variant is constructible ---

    #[test]
    fn every_error_variant_constructs() {
        // NOTE: the `variants.len()` assert below is a compile-time-fixed
        // completeness count, not proof any variant is reachable — an array
        // literal typechecks whether or not the variant is ever produced by
        // real code. Reachability through the actual function that returns
        // each variant is proved elsewhere, one dedicated test per variant:
        // PolicyShortenedBelowMinimum      -> policy_one_second_under_pack_floor_is_rejected
        // CrossPackCascade                 -> cascade_crossing_packs_is_rejected
        // InvalidRedactionReason           -> redaction_with_empty_reason_is_rejected (+ siblings)
        // UnknownPack                      -> unknown_pack_label_is_rejected,
        //                                      cascade_with_unknown_pack_label_is_rejected
        // UnknownDataClass                 -> unknown_data_class_label_is_rejected
        // PackNotInMatrix                  -> pack_not_in_caller_supplied_matrix_is_rejected
        // DataClassNotInMatrix             -> retention_expired_redaction_for_class_absent_from_matrix_is_rejected
        // EmptyDsrTenantId                 -> cascade_with_empty_tenant_id_is_rejected (+ siblings)
        // EmptyDsrSubjectId                -> cascade_with_empty_subject_id_is_rejected
        // EmptyDsrSourceMicroservice       -> cascade_with_empty_source_microservice_is_rejected
        // EmptyCascadeTargets              -> cascade_with_no_target_packs_is_rejected
        // RedactionGracePeriodNotElapsed   -> cascade_before_grace_period_elapses_is_rejected
        // EmptyRedactionAuditId            -> redaction_with_empty_audit_id_is_rejected (+ siblings)
        // DsrRedactionUnsupportedForClass  -> dsr_redaction_is_rejected_for_audit_class (+ siblings)
        // MerkleProofPreservationNotConfigured -> redaction_is_refused_when_matrix_declares_preserve_merkle_proof_false
        // DegenerateMatrixRule             -> from_parsed_rules_rejects_a_zero_minimum_years_pack,
        //                                      from_parsed_rules_rejects_a_zero_dsr_redaction_grace_days_default
        let variants = [
            RetentionDomainError::PolicyShortenedBelowMinimum,
            RetentionDomainError::CrossPackCascade,
            RetentionDomainError::InvalidRedactionReason,
            RetentionDomainError::UnknownPack {
                pack: "pack-atlantis".to_string(),
            },
            RetentionDomainError::UnknownDataClass {
                data_class: "TOP_SECRET".to_string(),
            },
            RetentionDomainError::PackNotInMatrix {
                pack: "pack-eu".to_string(),
            },
            RetentionDomainError::DataClassNotInMatrix {
                data_class: "AUDIT".to_string(),
            },
            RetentionDomainError::EmptyDsrTenantId,
            RetentionDomainError::EmptyDsrSubjectId,
            RetentionDomainError::EmptyDsrSourceMicroservice,
            RetentionDomainError::EmptyCascadeTargets,
            RetentionDomainError::RedactionGracePeriodNotElapsed {
                required_days: 30,
                elapsed_days: 29,
            },
            RetentionDomainError::EmptyRedactionAuditId,
            RetentionDomainError::DsrRedactionUnsupportedForClass {
                data_class: "AUDIT".to_string(),
            },
            RetentionDomainError::MerkleProofPreservationNotConfigured {
                data_class: "AUDIT".to_string(),
            },
            RetentionDomainError::DegenerateMatrixRule {
                detail: "pack-us: minimum_years is 0".to_string(),
            },
        ];
        assert_eq!(variants.len(), 16);
    }

    // --- is_normalized_identifier: unit-level coverage of the helper ---

    #[test]
    fn is_normalized_identifier_accepts_ascii_alphanumeric_and_the_allowed_punctuation() {
        for value in ["tenant-1", "a", "A1", "svc.name:1@ns", "___"] {
            assert!(is_normalized_identifier(value), "{value:?}");
        }
    }

    #[test]
    fn is_normalized_identifier_rejects_empty_and_disallowed_characters() {
        for value in [
            "",
            " ",
            "tenant 1",
            "tenant-1\n",
            "\u{200B}",
            "\u{FEFF}",
            "café", // non-ASCII letter
            "a/b",
        ] {
            assert!(!is_normalized_identifier(value), "{value:?}");
        }
    }
}
