---
id: ADR-0592
title: "Tenant-scoped, body-fingerprinted accounting idempotency keys (cross-tenant collision fix)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-23
door: two-way
owner: axis-billing-accounting
supersedes: []
superseded_by: [ADR-702]
amends: []
depends_on: [ADR-0083, ADR-0131]
related: [ADR-0510, ADR-0515, ADR-0581]
related_specs:
  - /specs/capability-registry.json
milestone: W2
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0592: Tenant-scoped, body-fingerprinted accounting idempotency keys

## Status

**Proposed - 2026-06-23 (authored for founder sign-off; BLOCKED pending adversarial security
review. Door: two-way — the change adds a leading tenant scope and a body-fingerprint component to
an internally-constructed idempotency key plus one additive storage-error variant and one additive
record field, all behind the existing clean-architecture seam; it is reversible by reverting the key
format without unwinding any SSOT. On approval the founder flips this to Accepted and admits the
born-unpropagated decision into
`ci/facade/baseline-ratchet/gate-baseline.signoff.json` per the established
new-Accepted-ADR door precedent, then propagates it into the masterplan/roadmap faces.)**

## Context

The accounting journal capability records app-layer audit and Workflow-dispatch envelopes keyed by
an idempotency key. The key is constructed in the app layer
(`billing/core/accounting-app/src/lib.rs`) and consumed by the storage port
(`billing/core/accounting-journal`, `AccountingJournalStoragePort`) and its in-memory adapter
(`billing/adapters/accounting-storage-inmemory`). The HTTP runtime adapter
(`billing/adapters/accounting-http`) accepts caller-supplied JSON whose `journal_id` and `tenant_id`
are raw strings (`JournalPostRequest::into_domain`), then surfaces the resulting key in its `202`
response.

Codex flagged this surface under AUTH-005 Wave-2b (framed as "billing"; the real capability is
accounting). Verifying the live `origin/dev` code confirmed a real cross-tenant money-integrity
defect:

- `journal_audit_envelope` built the key as `format!("{}:{}:posted", journal_id, 1)` — **with no
  tenant component**. Because `journal_id` is caller-chosen and the store keys records by the raw
  string, two distinct tenants posting a journal under the *same* `journal_id` produced the *same*
  key. The first write wins; the second tenant's audit record is refused as a "duplicate", so tenant
  A can suppress (and observe the existence of) tenant B's journal-post audit record — a
  cross-tenant collision and audit-suppression defect on financial data.
- The VAT-dispatch and payroll-posting keys already embedded the tenant id, so they were not subject
  to the cross-tenant collision. They were, however, **not body-fingerprinted**: a key replayed with
  a *changed* request body was indistinguishable from a genuine replay, so the store could silently
  treat a mutated command as an already-completed one.

This matches the AUTH-005 doctrine that audit/idempotency fields must be derived such that one
tenant's input can never collide with or suppress another's, and that a reused idempotency key
carrying a different body must be rejected, not silently de-duplicated.

## Decision

1. **Tenant-scope every accounting idempotency key, tenant-id first.** Introduce a single-sourced
   builder `scoped_idempotency_key(tenant_id, scope, primary_ref)` in the core crate that emits the
   *logical* key `idem-v2:<tenant_id>:<scope>:<primary_ref>`. The tenant id is the leading keyed
   component, so two tenants can never collide on a shared caller-chosen `primary_ref`. All three
   builders (journal-post, VAT-dispatch, payroll-posting) now route through it; the journal-post
   builder gains the previously-missing tenant scope. The key encodes the LOGICAL identity of a
   command and deliberately does NOT embed the body fingerprint (see Decision #3 for why).

2. **Body-fingerprint every command as a SEPARATE field.** Add a dependency-free, deterministic
   `idempotency_body_fingerprint(&[&str])` in core (FNV-1a/64 over a length-prefixed canonical
   encoding so field boundaries are unambiguous). The fingerprint covers every caller-mutable,
   money-material field of the command (journal: source_documents + per-line detail; VAT:
   evidence_paths; payroll: approval_evidence_ref + per-line detail). Each envelope carries the
   fingerprint as the SEPARATE `body_fingerprint` field, and the stored record persists it
   independently of the key.

3. **Distinguish a changed body from a replay at the store.** Add
   `AccountingStorageError::IdempotencyKeyBodyMismatch { key, stored, candidate }`. The store keys on
   the LOGICAL key only and persists the fingerprint as a separate record field. On `put_record`, a
   logical key that already exists with a *different* fingerprint is rejected as a body mismatch; an
   identical fingerprint is the prior `DuplicateIdempotencyKey` replay behaviour. The
   `reserve_idempotency_key` reservation path also reserves by the logical key so reserve and commit
   stay consistent. (The originally-authored shape embedded the fingerprint *inside* the key —
   `...:<primary_ref>#<fingerprint>` — and keyed the store by that full string. Adversarial review
   rejected it: a changed body produced a DIFFERENT map key, so `put_record` missed the prior record
   and silently inserted a SECOND one — the body-mismatch branch was unreachable dead code, defeating
   the silent-substitution / record-proliferation objective. Separating the logical key from the
   fingerprint makes the check live.)

The fingerprint is a **change-detection** mechanism for replay-vs-changed-body within the trusted
app-layer construction path, not a collision-resistant MAC against an adversary who can choose both
bodies. Cross-tenant isolation is enforced by the tenant scope (component 1), not by the
fingerprint. The future durable Postgres/RLS adapter additionally enforces isolation via row-level
security and can upgrade the fingerprint to a cryptographic digest from the owned crypto substrate.

## Consequences

- Idempotency keys change format (`idem-v2:` scheme). Because the in-memory adapter is volatile
  (NOT-FOR-PRODUCTION) and no durable store exists yet, there is no persisted-key migration. The
  scheme version (`IDEMPOTENCY_KEY_SCHEME = "idem-v2"`) makes a future durable migration explicit.
- The change is additive at the type level (one new error variant, one new record/envelope field)
  behind the existing port; downstream consumers that only read `idempotency_key` (the HTTP adapter)
  are unaffected except for the exact key string they surface.

## Born-accounting rows

This change adds **no new crate** and therefore creates **no born-accounting register rows**. It
modifies three existing crates (`billing-accounting-journal`, `billing-accounting-app`,
`billing-accounting-storage-inmemory-adapter`) and one downstream test
(`billing-accounting-http-adapter`). The generated cloud-ci faces are re-materialized via
`buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin` so any derived totals/registry views stay
in-sync, but no new accounting identity is born.

## RED tests

- `accounting-storage-inmemory/tests/storage.rs::accounting_storage_same_journal_id_across_tenants_does_not_collide`
  — two tenants posting the same `journal_id` produce distinct keys and both records persist (RED
  before fix: keys collided, second persist was refused).
- `accounting-storage-inmemory/tests/storage.rs::accounting_storage_same_logical_key_changed_body_is_rejected`
  — drives the REAL app builder (`post_journal_with_audit`) twice with the same `journal_id` but a
  changed line amount/total; the two envelopes share a logical key but differ in fingerprint, so the
  second persist is rejected as `IdempotencyKeyBodyMismatch` and `store.len() == 1` (RED before fix:
  the fingerprinted key landed the changed body in a different slot, growing `store.len()` to 2).
- `accounting-storage-inmemory/tests/storage.rs::accounting_storage_same_logical_key_identical_body_is_idempotent`
  — identical replay is refused as `DuplicateIdempotencyKey` with `store.len() == 1`.
- `accounting-journal/tests/journal.rs` — fingerprint determinism, change-detection, length-prefix
  field-boundary collision-resistance, and tenant-first LOGICAL key ordering.
- `accounting-app/tests/app_envelopes.rs`, `accounting-api/tests/contracts.rs`, and
  `accounting-http/tests/runtime.rs` updated to assert the tenant-scoped LOGICAL key shape (no
  embedded fingerprint) plus a non-empty separate `body_fingerprint` field.
