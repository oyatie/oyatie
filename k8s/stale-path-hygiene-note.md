---
doc_class: JudgmentNote
title: Stale microservices/cloud-k8s path hygiene (wave-1 prep)
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

## Left as intentional legacy (destination missing — do not invent)

Interior docs outside wave-1 priority still cite paths with **no** in-tree counterpart. Leave until a later judged land creates the artifact or deletes the cite:

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
