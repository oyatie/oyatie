---
id: ADR-0003
status: Superseded
superseded_by: [ADR-709]
doc_status: published
amended_by: [ADR-0350]
---

# ADR-0003: Audit chain and evidence emission as the single tamper-evident record-keeping substrate

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `platform-audit-chain`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0002, ADR-0004, ADR-0005, ADR-0007, ADR-0008, ADR-0011, ADR-0019

---

## Context

Oyatie hosts regulated tenant data across PHI, PII, PCI, KR `신용정보`, KR `영상정보`, and a 12-class data-use boundary (ADR-0008). The PRD's hard-zero metric is "tenant data egress without consent receipt = 0 events ever." That metric is only enforceable if every cross-microservice flow, every consent decision, every capability invocation, and every regulatory-pack evidence emission lands in an *append-only, hash-chained, replayable* surface that no axis can write around. A policy-only (memo-style) audit posture has already failed in the legacy ADR corpus — the contradiction ledger LEDG-007 is partially the consequence.

A second pressure is the cohesion claim itself (ADR-0001). The single-audit-chain substrate is what makes "one product, one audit chain, one consent surface" mechanically true. If the search axis writes audit elsewhere, or the ads axis maintains its own attribution log without chain anchoring, the claim collapses. A third pressure is regulator interaction: KR PIPC, KISA, MFDS, FSC, KCC; EU regulators under GDPR/DORA/AI Act; US HIPAA/PCI auditors all require evidence packs regenerable in hours, not weeks (PRD §4.1 metric: ≤ 4 hours from request).

---

## Decision

We adopt a single **append-only, hash-chained audit-event log** as the tamper-evident record-keeping surface for every regulated event in every axis. The kernel is `crates/oya-audit-chain-kernel`; the application layer is `crates/oya-audit-chain-app`; per-tenant shards live behind `crates/oya-audit-chain-adapter-postgres-*` with optional cold-tier mirror.

### Chain structure

```rust
// crates/oya-audit-chain-kernel
pub struct AuditEvent {
    pub event_id: EventId,                    // ULID, monotonic per tenant shard
    pub tenant_shard: TenantId,
    pub prior_block_hash: BlockHash,          // hash chain link
    pub event_class: EventClass,              // see EmissionContract below
    pub principal: PrincipalRef,
    pub capability: Option<CapabilityId>,
    pub data_classes_touched: BTreeSet<DataClass>,
    pub regulatory_packs_consumed: BTreeSet<RegulatoryPackId>,
    pub autonomy_tier_at_decision: Option<AutonomyTier>,
    pub payload: AuditPayload,                // event-class-specific, schema-validated
    pub timestamp_utc: chrono::DateTime<chrono::Utc>,
    pub anchor_pointer: Option<AnchorPointer>, // populated after periodic root anchoring
}

pub struct BlockHash([u8; 32]);  // BLAKE3 of canonicalized event bytes + prior_block_hash
```

### Per-tenant shard + cross-tenant root anchor

- **Per-tenant shard.** Each tenant has its own hash chain. This scopes regulator queries cleanly and prevents cross-tenant correlation through chain ordering.
- **Periodic root anchoring.** Every shard's tip hash is rolled into a global Merkle root every `T_anchor` (default 60 s; tunable per regulator). The root is published to the customer-facing trust portal (`trust.oyatie.com`) and Rekor-anchored per ADR-0013 supply-chain references.
- **Replayability.** Chain replay reconstructs the regulatory state at any prior `t`. Live mutations never break replay because the chain is append-only.

### Per-capability emission contract

Every capability declared under `registry/capability-templates/<id>.yaml` carries an explicit `evidence_emission_topic` field. The Foundry runtime (ADR-0007) refuses to invoke a capability whose declared emissions are not satisfied by the chain at run time. The contract is enforced at three points:

1. **Catalog validation** (`oya-governance-evidence-contract`) — fails any catalog record whose declared `data_classes_touched` ∪ `regulatory_packs_consumed` does not match the emission contract.
2. **Runtime guard** — every capability invocation that completes without emitting the contracted events is rolled back and re-tried under a `EVT-EVIDENCE-EMISSION-MISSING` alert.
3. **Daily integrity check** — `oya-audit-chain-app` walks each tenant shard, verifies hash continuity, recomputes per-day root, compares to the trust-portal anchor, and emits `EVT-AUDIT-INTEGRITY-OK` or pages a P1 bridge on mismatch.

### Required emitters by axis (non-exhaustive; full table in DESIGN §7)

| Axis | Event classes (must emit) |
|---|---|
| SaaS | tenant-onboarding, plugin-install, tenant-data-export, DSR fulfillment |
| Workspace | doc-share, doc-export, mail-forward-cross-tenant, meet-recording-publish |
| Vertical | regulatory-pack-adoption, control-evidence-collection, break-glass-invocation |
| Foundry — runtime | capability invocation, autonomy-ceiling decision, model invocation, RAG retrieval |
| Foundry — engineering platform | catalog mutation, gate ratchet WARN→BLOCK, foundation-bypass create/expire |
| Cloud | IAM mutation, region/AZ register, capacity grant, resource provision |
| Search | index lifecycle, DSR-driven cascade delete, public-corpus rights record |
| Ads | campaign create, audience create, ad-targeting decision, conversion attribution |

### Erasure semantics

The chain is append-only. DSR-driven deletes are recorded as a `deletion-evidence` record + a cryptographic invalidation pointer at the original record. The original event remains in the chain (so downstream replays still succeed); decryption keys for the encrypted payload are KMS-shredded per ADR-0009 cell architecture, satisfying GDPR Art 17 and PIPA Art 39-7 without violating chain integrity.

### Boundary

- Applies to: every regulated capability invocation, every cross-microservice data flow, every consent decision, every break-glass invocation, every regulatory-pack adoption, every model training-data ingestion under `model_training_oya` purpose.
- Does not apply to: ephemeral debug logs, per-microservice local telemetry that touches no regulated data, build-time stdout (which lives in CI logs).

---

## Consequences

### Positive

- The PRD hard-zero metric ("tenant data egress without consent receipt") becomes mechanically enforceable.
- Regulator evidence pack regeneration ≤ 4 hours becomes feasible because the chain is queryable per tenant.
- Cohesion thesis (ADR-0001) is mechanically true at the audit substrate — every axis writes here.
- Daily integrity checks turn audit-chain failure into a known-class incident with a runbook, not a surprise.

### Negative

- Per-event emission cost (~1–2 ms hot-path) on every regulated capability call. Acceptable for control-plane and most data-plane calls; high-frequency data-plane events (e.g. ad serving) batch-emit per-window aggregates with per-decision sampling.
- Chain storage grows monotonically; cold-tier mirroring policy is required per regulatory pack (KR PIPA = 5 yr; HIPAA = 6 yr; PCI-DSS = 1 yr; GDPR = lawful-basis-bound).
- Per-tenant shard boundary makes some cross-tenant analytics queries (e.g. "all capability invocations system-wide last hour") more expensive; analytics-plane projections (ADR-0004) carry the read load.

### Operational

- On-call: `EVT-AUDIT-CHAIN-INTEGRITY-MISMATCH` and `EVT-EVIDENCE-EMISSION-MISSING` page the audit on-call with a 5-minute SLA.
- Runbooks: `runbooks/audit-chain-integrity-recovery.md`, `runbooks/dsr-cascade-with-evidence.md`, `runbooks/regulator-evidence-pack-regen.md`.
- CI: `oya-governance-evidence-contract` runs on every PR touching `registry/capability-templates/` or `crates/oya-*-app/` with a capability dispatch.
- Auditor access: trust portal exposes per-pack evidence pack download with cryptographic chain proof.

---

## Alternatives considered

### Alternative A — Per-axis audit log + nightly rollup

- **Pros:** lower per-event hot-path cost; per-microservice ownership is clearer.
- **Cons:** rollup window is a regulator-visible gap; cross-microservice correlation requires log-stitch joins; cohesion claim fails at the substrate level.
- **Rejected because:** ADR-0001 forbids substrate forking; the chain is the cohesion proof.

### Alternative B — Vendor SIEM as authoritative log (Splunk, Datadog, etc.)

- **Pros:** turnkey querying.
- **Cons:** sovereignty (KR `망분리`, EU GAIA-X), license posture (ADR-0013), and cost-at-scale all push against vendor lock; tamper-evident hash chains require kernel control.
- **Rejected because:** sovereignty + the ADR-0013 license bar.

### Alternative C — Event sourcing across the entire codebase

- **Pros:** strong story for replayability everywhere.
- **Cons:** event sourcing has a steep operability cost when applied uniformly; the PRD selectively event-sources audit + chain only.
- **Rejected because:** scope; we event-source audit/state-of-record, not all aggregates.

---

## Open questions

1. **Q1.** Anchor cadence — 60 s default is conservative. Some packs (KR-FSC) may demand sub-second freshness. Per-pack override? Default: yes, with min 1 s. → owner: `regional-packs/oya-pack-kr` + `council-privacy`.
2. **Q2.** Cold-tier mirror — Iceberg/Parquet on object store, or a dedicated cold-chain store? Default: Iceberg + per-tenant key-shred. → owner: `cloud`.
3. **Q3.** High-frequency data-plane sampling — per-decision sampling rate vs aggregate-only emission for ad serving. Default: 1% per-decision + 100% aggregate per minute. → ADR-0008 + ads-axis ADR.
4. **Q4.** Cross-tenant root anchor publication cadence — minute? 5-minute? Trust portal UX implication. Default: 1 min. → owner: `foundry` (trust portal).

---

## References

- `docs/DESIGN.md` §7 (audit chain), §11 (cross-microservice contradiction audit)
- `docs/PRD.md` §6 constraint 2 (audit-chain immutability), §4 (success metrics: zero egress without consent receipt; ≤ 4 hours evidence regen)
- `docs/PRIVACY-PROGRAM.md` §2.2.4 layer 5 (audit-chain emission per decision), §2.2.9 (DSR cascade with proof-of-erasure)
- `docs/COMPLIANCE-MATRIX.md` §3.1 (KR PIPA Art 29/34/39-7), §3.2 (GDPR Art 17/30/33), §3.3 (HIPAA §164.404), §3.4 (PCI Req 10), §3.7 (EU AI Act Art 12 record-keeping)
- ADR-0001 (cohesion thesis), ADR-0007 (Cedar + persona tier; runtime enforcement), ADR-0008 (Data Use Boundary; emission requirement), ADR-0019 (doc catalog + EVT-AUDIT-FINDING)
