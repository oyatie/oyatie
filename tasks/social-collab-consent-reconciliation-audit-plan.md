# Plan: social-collab-consent-reconciliation-audit

## Objective

Add a pure collaborative-consent reconciliation/audit kernel to `oya-community-social-domain`.
Post-construction (after `SocialPost::new` already gate-checks consent count >= owner count),
this kernel computes the _named_ per-owner gap: who is satisfied, who is missing, and which
consents are extraneous (granted for non-owners).

## Requirements Analysis

### Core semantics
- `collab_consent_audit(owner_refs, consent_refs) -> Result<CollabConsentAudit, SocialError>`
- Input validation: any empty/blank ref in either slice → `Err(SocialError::Invalid)`
- Set algebra (BTreeSet for determinism):
  - `satisfied`          = owners ∩ consents  (ref appears in both)
  - `missing_consent`    = owners − consents   (owner has no matching consent)
  - `extraneous_consent` = consents − owners   (consent names a non-owner)
- `is_fully_consented(audit) -> bool` = `audit.missing_consent.is_empty()`

### Edge cases
- Both slices empty → satisfied=∅, missing=∅, extraneous=∅, is_fully_consented=true
- Owners empty, some consents → all consents are extraneous; is_fully_consented=true (vacuously)
- Single blank ref triggers Invalid immediately (early return on first blank)
- Duplicate refs in input: BTreeSet deduplication is automatic — semantics are correct
- Order independence: BTreeSet ensures deterministic ordering regardless of input order

### Acceptance tests (≥7 required)
1. Full consent: every owner has a consent → missing=∅, is_fully_consented=true
2. Partial gap: one owner missing consent → appears in missing_consent
3. Extraneous consent: consent for non-owner → appears in extraneous_consent
4. Blank owner ref → Err(SocialError::Invalid)
5. Blank consent ref → Err(SocialError::Invalid)
6. Empty owners + non-empty consents → all consents extraneous, is_fully_consented=true
7. Ordering deterministic across runs (same result twice)
8. Mixed scenario: some satisfied, some missing, some extraneous

## Subtasks (ordered)

1. [x] Write plan file (this file)
2. [x] Write spec file `docs/specs/task-social-collab-consent-reconciliation-audit.md`
3. [x] Add `CollabConsentAudit` struct and `collab_consent_audit` + `is_fully_consented` to `src/lib.rs` (red → green)
4. [x] Write ≥7 unit tests in `src/lib.rs` cfg(test) block; confirm cargo check passes
5. [x] Run `cargo nextest run -p oya-community-social-domain` → all green
6. [x] Self-review (correctness/security/perf/arch); simplify; re-run nextest
7. [x] Commit + push + open PR
