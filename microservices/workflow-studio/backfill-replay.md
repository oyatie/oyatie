---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: workflow-studio
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-workflow + ops-sre-reliability
deciders: axis-workflow, council-architecture, ops-sre-reliability
related_adrs: [ADR-0028, ADR-0110, ADR-0131, ADR-0164]
related_artifacts:
  - microservices/workflow-studio/PRD.md
  - microservices/workflow-studio/capacity-model.md
  - microservices/workflow-studio/threat-model.md §"T-T-01" + §"T-T-02"
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (workflow-studio µservice)

## Purpose

Specify how workflow-studio handles two scenarios:
1. **Backfill** — historical editor sessions / CRDT op streams need reconstruction (e.g., post-incident analysis, audit-chain integrity verification, regulatory disclosure request).
2. **Replay** — an existing definition version needs re-rendering OR re-saving with different parameters (e.g., bug fix in DSL emitter, jurisdiction overlay version change, schema migration).

## Backfill

### Contract

When historical editor session state needs reconstruction:

1. **Read source**: Postgres `editor_session_seals` table (Ed25519-sealed per-save) + Valkey ephemeral state (if still in memory) + audit-chain seal log (cross-µservice fallback).
2. **Reconstruct CRDT op stream**: from per-save seal-deltas, replay merged ops in sequence; validate sequence-num monotonic; verify HMAC on every op.
3. **Verify audit-chain integrity**: every reconstructed save event must have a corresponding seal in audit-chain µservice; chain must be reconstructable.
4. **Emit `EditorSessionReconstructed` event**: consumed by observability for forensic dashboards; audit-chain seals the reconstruction itself.

### Constraints

- Backfill does NOT mutate the original session record; reconstruction emits new events with `kind=reconstructed` tag.
- Retention bound: Postgres `editor_session_seals` retained 30d hot; cold-tier object storage retains seals 7y per Bominal ADR-0028.
- Cost: backfill is bounded by `O(save_count × session_count)`; sessions older than 30d require cold-tier fetch (slower; cost-budget bounded per `cost-budget.md`).
- Per-tenant rate-limiting: tenants cannot trigger more than 1 backfill per definition per hour (anti-abuse).
- 2-person rule + ops-security approval for any backfill that touches cross-tenant data (e.g., audit response).

### Verification

- Integration test: seed Postgres + Valkey with synthetic session; reconstruct via backfill; verify reconstructed canvas == original canvas (CRDT state isomorphism).
- Audit-chain integrity: every backfilled event has seal lineage to original.
- Determinism: re-running backfill emits identical reconstructed state.

## Replay

### Contract

Replay re-emits / re-renders a definition version with the current emitter / loader / overlay code:

| Trigger | Procedure | Output |
|---|---|---|
| Bug fix in dsl-emitter | Replay all saved definitions through new emitter; assert byte-equal output | If non-equal: flag for tenant review; do NOT auto-overwrite |
| Bug fix in dsl-loader | Replay load operations; assert no new validation errors | Surface errors per-tenant |
| Overlay version change | Replay jurisdiction overlay resolution; surface visual diff | Tenant decides accept/reject overlay update |
| Schema migration (workflow_spec.v1 → v2) | Replay via migration adapter; emit v2 spec; preserve v1 lineage | Two-version coexistence per Bominal ADR-0164 versioning policy |

### Procedure

1. Operator invokes: `cargo run -p oya-dev-cli -- workflow-studio replay --definition <id> --reason "<rfc>"`.
2. CLI requires 2-person rule + ops-security approval (replay touches tenant data).
3. Engine re-emits canonical spec via current dsl-emitter code; compares against stored version_sha.
4. Emits `DefinitionReplayed` event with `prior_version_sha`, `replay_version_sha`, `byte_equal`, `differences_summary`, `reason`.
5. Audit-chain seal: replay itself is sealed.

### Constraints

- Replay does NOT mutate the original definition; new version SHA created with explicit `kind=replayed` label.
- AC-02 byte-equality invariant: replay of a clean definition MUST produce byte-equal output. If it doesn't, that's a regression in dsl-emitter — file a bug.
- Replay cannot overwrite production-tier release pointers; new replay version stays in `draft` state pending tenant promotion.

### Verification

- Integration test: replay 100 golden specs through current emitter; expect 100% byte-equal.
- Migration test: replay v1 spec through v2 adapter; expect lossless conversion.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Backfill on regulatory request | per-tenant-disclosure | ~$1.00 (1 definition, 30d history, Postgres + cold-storage fetch) |
| Replay on dsl-emitter bug fix | per-emitter-release | ~$50 (full replay across all definitions; 100K active definitions) |
| Replay on overlay version change | per-overlay-publish | ~$5 (single overlay × all affected definitions) |
| Replay on schema migration (v1→v2) | one-time per migration | ~$500 (full corpus; bounded by versioning ADR) |

Costs surfaced in `cost-budget.md` § "Cost-Optimisation Levers" — backfill / replay budgeted as part of Studio's operational envelope.

## Limitations

- Backfill quality is bounded by Postgres + audit-chain retention windows. Sessions older than 7y are forensically lost (intentional per ADR-0028 retention).
- Replay assumes deterministic dsl-emitter; non-determinism is a bug (caught by `oya-governance-workflow-spec-roundtrip` lane).
- Schema migrations (v1 → v2) require explicit migration adapters; not auto-replayable without operator sign-off.
- Cross-tenant backfill (e.g., for breach forensics) requires legal + privacy approval per pack regulation (GDPR Art. 15 DSR; KR PIPA Art. 35; HIPAA §164.524).

## References

- `microservices/workflow-studio/PRD.md`.
- `microservices/workflow-studio/capacity-model.md`.
- `microservices/workflow-studio/cost-budget.md`.
- `microservices/workflow-studio/contracts/asyncapi.yaml`.
- ADR-0028 Audit-chain (Bominal inherited).
- ADR-0110 ChangeSet state machine.
- ADR-0164 Workflow canonical spec format (Bominal).
- yrs / loro CRDT replay semantics — vendor docs.
- Google SRE Workbook ch. 9 (Simplicity in design — limits replay scope).
