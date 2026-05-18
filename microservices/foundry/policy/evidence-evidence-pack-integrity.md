---
doc_class: PolicyDocument
title: foundry-evidence — evidence-pack integrity
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: axis-foundry-evidence + ops-security
related_adrs: [ADR-0028, ADR-0024, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/foundry/PRD.md
  - microservices/foundry/threat-model.md
  - microservices/foundry/failure-modes.md
  - microservices/audit-chain/policy/seal-integrity.md  (substrate; this doc refers)
doc_status: published
---

# foundry-evidence — evidence-pack integrity policy

## Scope

This policy specifies the integrity invariants for the **frontend** layer: per-invocation evidence-pack assembly, the audit-chain bridge, and the Postgres evidence index. Cryptographic-substrate integrity (Merkle, Ed25519, HSM, WORM Object Lock) is owned by `audit-chain` and governed by `microservices/audit-chain/policy/seal-integrity.md`; this document references those invariants where the frontend depends on them.

## EPI-01 — Pack identity is invocation-scoped

Each evidence pack is uniquely identified by `(tenant_id, invocation_id, attempt_no)`. Re-emission with the same key is idempotent: the first successful pack is canonical; later duplicates are rejected with a structured error and emit `oya_foundry_evidence_duplicate_attempt_total`.

## EPI-02 — Signal aggregation is **single-bind**

A pack is assembled exactly once per `(invocation_id, attempt_no)`. Late-arriving signals after assembly are not merged; they emit `oya.foundry_evidence.late_signal.v1` with `lateness_seconds` and are surfaced as `audit-chain` events linked to the canonical pack via `superseded_by_late_signal_ref`. This is honest (no silent re-write) and supports retrospective audit.

## EPI-03 — Pack payload SHA is end-to-end

- Pack-builder computes `pack_payload_sha = sha256(canonical_cbor(pack_envelope))` at assembly time.
- `pack_payload_sha` is passed into `audit-chain` emit as the `payload_sha` field per Bominal ADR-0003.
- audit-chain seal binds `pack_payload_sha` into the Merkle leaf; tampering with the pack content between assembly and seal is detectable on substrate verify.
- foundry-evidence stores `pack_payload_sha` in Postgres alongside the audit-chain `event_id`.

## EPI-04 — Postgres index is append-only

- Role `foundry_evidence_writer` is INSERT-only at the SQL level.
- UPDATE / DELETE permitted only via the retention-cascade RPC, which is Cedar-gated, 2-person-rule on the RPC envelope, and audit-emitted.
- LEAN lane `evidence-index-append-only` blocks any Helm change that grants UPDATE/DELETE to `foundry_evidence_writer`.

## EPI-05 — Pack-builder process integrity

- Pack-builder workers run with `readOnlyRootFilesystem=true` per `iac/helm/evidence-builder/values.yaml`.
- Code-signing image attestation (Sigstore) verified at pod admission.
- SPIFFE attestation required for outbound audit-chain emit.
- Workload Identity rotated per `cloud-secrets` rotation policy; no static credentials.

## EPI-06 — Schema evolution is no-silent-regression

- The pack schema (`oya-foundry-evidence-evidence-pack` types) is contract-tested against `/specs/foundry-evidence.json` reference vectors.
- Schema change requires ADR + version bump + sunset window + LEAN `no-silent-regression` lane green.
- Multiple schema versions live concurrently during the sunset window; readers index by `schema_version` field.

## EPI-07 — Eval-verdict join correctness

- The eval-verdict joined into the pack is the verdict that was **current at the invocation timestamp** per `foundry-eval`'s eval-verdict-history table.
- The join is computed at pack-assembly time and frozen into the pack; subsequent verdict changes do NOT mutate already-sealed packs.
- Property-based test asserts `eval_verdict.recorded_at <= pack.invocation_ts < eval_verdict.next_change_ts`.

## EPI-08 — Autonomy-tier rationale is single-author

- The `autonomy_tier_decision` field is supplied by `foundry-supervisor` and is the single authoritative source.
- pack-builder must not derive autonomy-tier from any other signal; LEAN lane `authority-cohesion` enforces single-source-of-truth.

## EPI-09 — Guardrail-decision attribution

- Each guardrail decision in the pack carries `(guardrail_id, guardrail_version, decision, rationale_hash)`.
- The `rationale_hash` content-addresses the rationale text in `audit-chain` WORM; the rationale text is gated by `payload_data_class` per `policy/tenant-scope.cedar`.

## EPI-10 — Late-bound substrate emission

- If `audit-chain` substrate is unavailable at pack-assembly time, the pack is enqueued in a durable dead-letter store with `audit_chain_emit_pending=true`.
- `record_invocation` still returns receipt to the caller, with `sealed=false` and `audit_event_id=null`.
- The bridge-retry worker drains the dead-letter store under bounded back-off; every retry attempt is itself audit-emitted (after substrate is available).
- The pack is NOT considered final until `audit_chain_emit_pending=false`; queries clearly indicate `pending_seal=true` for such packs.
- SLO: dead-letter drain ≤ 1h p99; see `slos/`.

## EPI-11 — Backfill posture

- Backfill / replay never writes to historical audit-chain periods (substrate-locked per `audit-chain/policy/seal-integrity.md` §SI-07).
- A pack that needs to be re-emitted (e.g., because a late signal was discovered after the original assembly) is written as a NEW pack at the current period, with `supersedes_pack_ref` pointing to the original.
- `runbooks/evidence-pack-rebuild.md` enforces 2-person rule + on-chain reason; backfill is observable, never invisible.

## EPI-12 — Cross-microservice import discipline

- foundry-evidence consumes `audit-chain` strictly via `oya-audit-chain-emission-sdk` re-exports.
- Direct import of `oya-audit-chain-*-domain` or any internal substrate crate is forbidden.
- LEAN lane `cross-microservice-import-forbidden` blocks at compile time.

## EPI-13 — Honest-claim posture per ADR-0133

- Every integrity invariant above is CI-asserted via a named LEAN lane or a `cargo nextest` drill.
- The `hyperscaler-maturity-claims` lane refuses commit-claims of integrity that are not asserted.
- See `competitor-parity-matrix.md` §"Integrity" for honest gaps vs CloudTrail Audit Lake + Splunk integrity controls.

## Verification (CI-level)

```bash
oya gate validate lean-a4 --microservice foundry-evidence  # cross-microservice-import-forbidden
oya gate validate evidence-index-append-only --microservice foundry-evidence
oya gate validate no-silent-regression --microservice foundry-evidence
oya gate validate authority-cohesion
oya gate validate hyperscaler-maturity-claims
cargo nextest run -p oya-foundry-evidence-evidence-pack-builder-domain --test pack_sha_end_to_end
cargo nextest run -p oya-foundry-evidence-eval-evidence-aggregator --test eval_join_temporal_correctness
cargo nextest run -p oya-foundry-evidence-capability-invocation-recorder-rest --test late_substrate_dead_letter
```

## Review cadence

- Per release.
- Out-of-cycle on any change to pack schema, audit-chain bridge, or retention-cascade RPC.
- Sign-off: ops-security + axis-foundry-evidence + council-privacy (for any change that affects DSR cascade).
