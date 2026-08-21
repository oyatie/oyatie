# Spec: social-collab-consent-reconciliation-audit

## Objective

Extend `community-social-domain` with a pure, deterministic collaborative-consent
reconciliation/audit kernel. The kernel operates after `SocialPost` construction and
answers the question: given a set of required owner refs and a set of granted consent refs,
which owners are fully consented, which are missing consent, and which consents name
non-owners?

## Crate boundary

`community-social-domain` (`crates/community-social-domain/src/lib.rs`).
No workspace changes, no new dependencies, no new crates.

## Mod layout (flat-clean-arch per ADR-0509)

All code lives in `src/lib.rs`. No new modules needed for this slice size.
If the file grows beyond ~600 lines a `consent_audit` sub-mod can be introduced later.

## Public API

```rust
/// Result of reconciling collab owner refs against granted consent refs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollabConsentAudit {
    /// Owners that have a matching consent ref.
    pub satisfied: BTreeSet<String>,
    /// Owners that have no matching consent ref.
    pub missing_consent: BTreeSet<String>,
    /// Consent refs that name no known owner.
    pub extraneous_consent: BTreeSet<String>,
}

/// Reconcile `owner_refs` against `consent_refs`.
///
/// Returns `Err(SocialError::Invalid)` if any ref (in either slice) is blank or
/// whitespace-only.  Otherwise computes the three disjoint sets deterministically.
pub fn collab_consent_audit(
    owner_refs: &[String],
    consent_refs: &[String],
) -> Result<CollabConsentAudit, SocialError>;

/// Returns `true` iff every owner has a matching consent (missing_consent is empty).
pub fn is_fully_consented(audit: &CollabConsentAudit) -> bool;
```

## Set semantics

| Set                 | Formula                          |
|---------------------|----------------------------------|
| `satisfied`         | owners ∩ consents                |
| `missing_consent`   | owners − consents                |
| `extraneous_consent`| consents − owners                |

All three sets are `BTreeSet<String>` — ordering is deterministic and stable.

## Validation rules

1. Any blank/whitespace-only ref in `owner_refs` → `Err(SocialError::Invalid)`.
2. Any blank/whitespace-only ref in `consent_refs` → `Err(SocialError::Invalid)`.
3. Empty `owner_refs` + empty `consent_refs` → `Ok` with all-empty sets.
4. Empty `owner_refs` + non-empty valid `consent_refs` → `Ok`; all consents land in `extraneous_consent`.

## Testing strategy

Hermetic unit tests inside `src/lib.rs` `#[cfg(test)]` block. Minimum 7 tests:

| Test name                          | Coverage area                        |
|------------------------------------|--------------------------------------|
| `audit_full_consent`               | All owners consented, is_fully_consented true |
| `audit_partial_gap`                | Some owners lack consent             |
| `audit_extraneous_consent`         | Consent refs naming non-owners       |
| `audit_blank_owner_rejected`       | Blank owner ref → Err(Invalid)       |
| `audit_blank_consent_rejected`     | Blank consent ref → Err(Invalid)     |
| `audit_empty_both_ok`              | Both empty → Ok, fully consented     |
| `audit_deterministic_ordering`     | Repeated calls produce identical sets|
| `audit_gap_and_extraneous`         | Overlap, gap, and extraneous coexist |

## Observability / SLO

Domain layer has no direct OTel instrumentation; spans/metrics are emitted by the
use-case adapter layer (`community-social-post-composition-usecase`). No OpenSLO
changes needed for this slice — the audit is a pure in-process computation with no
network or storage I/O.

## Contracts

No new OpenAPI / AsyncAPI / proto3 surface for this slice. The struct and functions are
consumed by use-case and API layers in subsequent IPs.

## Cloud-native readiness

- No I/O → trivially hermetic, embeds in any container/WASM boundary.
- Deterministic ordering → safe for audit-log append and idempotent reconciliation loops.
- `BTreeSet` → O(n log n) insert; acceptable for the expected owner-set cardinality (≤100).
