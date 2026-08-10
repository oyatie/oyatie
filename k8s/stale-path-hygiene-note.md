---
doc_class: JudgmentNote
title: Stale microservices/cloud-k8s path hygiene (wave-1+2)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - k8s/reorg-unit-judgments.v1.json
  - k8s/manifest.json
  - k8s/runbooks/
ssot_todo: free-capability-k8s-prep
---

# Stale path hygiene note (`k8s/**`)

## Chesterton challenge

`microservices/cloud-k8s/...` citations were left as strangler compatibility after the capability tree landed under `k8s/`. On this tip the legacy directory is **gone** (`test ! -d microservices/cloud-k8s`). Keeping dead absolute citations fails day-2 operability; inventing missing trees to satisfy greps would violate YAGNI and the wave envelope.

## Wave-1 scope (retargeted)

| Surface | Change |
|---|---|
| `k8s/manifest.json` | `service_local_contract_scaffolds` → `k8s/contracts/{openapi,asyncapi,proto}/...`; prose no longer claims home is `microservices/cloud-k8s` |
| `k8s/runbooks/*.md` (8 files) | All `microservices/cloud-k8s/` related_artifacts / References → `k8s/` where counterparts exist |

Verified destinations for every retargeted runbook cite:

- `k8s/failure-modes.md`
- `k8s/threat-model.md`
- `k8s/multi-region.md`
- `k8s/capacity-model.md`
- `k8s/incident-response.md`
- `k8s/policy/cluster-isolation.md`
- `k8s/runbooks/control-plane-restore.md`
- `k8s/runbooks/etcd-quorum-recovery.md`
- `k8s/contracts/openapi/cloud-k8s.yaml`
- `k8s/contracts/asyncapi/cloud-k8s-events.yaml`
- `k8s/contracts/proto/cloud-k8s.proto`

## Wave-2 scope (retargeted)

Core day-2 cross-ref ring: `related_artifacts` + `References` retargeted to `k8s/` where counterparts exist (86 cite lines closed on this batch).

| Surface | Change |
|---|---|
| `k8s/failure-modes.md` | Full outbound retarget (11 cites) |
| `k8s/multi-region.md` | Full outbound retarget (9 cites) |
| `k8s/cost-budget.md` | Full outbound retarget (5 cites) |
| `k8s/capacity-model.md` | Full outbound retarget (6 cites) |
| `k8s/competitor-parity-matrix.md` | Full outbound retarget (2 cites) |
| `k8s/backfill-replay.md` | Full outbound retarget (9 cites) |
| `k8s/policy/cluster-isolation.md` | Full outbound retarget (9 cites) |
| `k8s/contracts/openapi/cloud-k8s.yaml` | Policy cedar glob retarget (1 cite) |
| `k8s/incident-response.md` | Partial — 12 cites; `legal/notification-templates/gdpr-art-33.md` deferred |
| `k8s/dpia.md` | Partial — 8 cites; `legal/*` bundle deferred |
| `k8s/policy/data-residency.md` | Partial — 8 cites; `pack-routing.cedar` + `legal/*` deferred |
| `k8s/sdk-plan.md` | Partial — 6 cites; `sdk-generation/` deferred |

### Citation inventory (tip after wave-2)

| Metric | Count |
|---|---|
| `microservices/cloud-k8s` cite lines closed (wave-2 only) | 86 |
| Remaining cite lines in `k8s/**` (intentional legacy + not yet swept) | 508 |
| Files still containing at least one stale cite | 59 |

## Left as intentional legacy (destination missing — do not invent)

Interior docs outside wave-1/2 priority still cite paths with **no** in-tree counterpart. Leave until a later judged land creates the artifact or deletes the cite:

| Missing under `k8s/` | Example cite homes (not rewritten this wave) |
|---|---|
| `legal/transfer-register.md` (+ other `legal/*`) | `dpia.md`, `policy/data-residency.md`, `incident-response.md` |
| `evidence/cis-k8s-benchmark/`, `evidence/nsa-k8s-hardening/`, `evidence/multispectrum/` | `compliance.md` |
| `sdk-generation/` | `sdk-plan.md` |
| `runbooks/attestation-failure.md` | `reference-implementations/bootstrap-cluster-rust-sdk.md` |
| `CODEOWNERS` | `compliance.md` |
| `policy/pack-routing.cedar`, `policy/schema.cedarschema` | `policy/data-residency.md`, `policy/tenant-scope.cedar` |

## Explicit non-goals

- No `specs/k8s-port/**` edits; no fight with k8s-port programme.
- No mass rewrite of `IP-*.md` / full compliance corpus.
- No hubs, `Cargo.lock`, `specs/reachability*`, restack onto `dev`.

## Wave-4 Seat A follow-through (2026-08-10)

Retargeted verified `microservices/cloud-k8s/**` cites → `k8s/**` for iac/helm/terraform/kustomize, contracts, policy, slos, runbooks, capabilities, PRD/ARCHITECTURE/manifest where destinations exist.

### Deferred (still missing / brace-globs / legal)

- `legal/**`, `sdk-generation/**`, `evidence/**`, brace-expanded catalog/crate globs, and any `src/crates/**` cites without in-tree homes.
- No hubs, no `Cargo.lock`, no merge.


## Wave-5 Seat A follow-through (2026-08-10)

Retargeted verified remaps:

- Bare microservices/cloud-k8s -> k8s (manifest/reorg-unit/coherence/ADR-CK-001)

### Deferred

- Still missing legal/sdk-generation/evidence/CODEOWNERS/eval fixtures/brace catalog globs and create-only SLO/lane paths from IP-014/IP-015
- k8s/slos/** retained where files live (not dual-homed under observability/)
- No hubs, no Cargo.lock, no merge.
