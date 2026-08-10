---
doc_class: JudgmentNote
title: Stale microservices/{cloud-storage,drive,recordings,imaging} path hygiene (wave-5 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - storage/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`storage/**` — Seat A wave-5)

## Scope

Retarget only **verified** in-tree destinations under `storage/**` (incl. nested drive/recordings/imaging).
Do not invent missing homes. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- `microservices/cloud-storage/**` → `storage/**` (PRD/README/manifest/faqs/benchmarks/runbooks/migration-playbooks/iac/coherence artifacts when present)
- Nested faces already advanced in wave-3 remain; additional verified nested remaps when destinations exist

## Deferred

- Missing `src/` / `tests/` / `ARCHITECTURE.md` / historical IP homes under drive/recordings/imaging
- Cross-cap observability/governance/cloud-iac cites

## Seat A media tranche (2026-08-10)

Nearest envelope for noun **media** (no `integ/media` rail) = `integ/storage` nested faces `storage/recordings/**` + `storage/imaging/**` (drive is adjacent file substrate).

### Retargeted (verified)

- `microservices/recordings/iac/kustomize/overlays/**` cites → `storage/recordings/iac/kustomize/overlays/**` when overlays tree present

### Deferred (missing homes — do not invent)

- `microservices/recordings/IP-*.md`, `PRD.md`, `capacity-model.md`, `compliance.md`, historical ADR-RECORDINGS-* filenames, OpenSLO under recordings
- `microservices/imaging/PRD.md`, `ARCHITECTURE.md`

## Seat A wave-6 dep-ordered (2026-08-10)

- Verified remaps applied: **8** cite(s) across **2** file(s).
- Scope: path/manifest/SLO/contract/capability/catalog high-value only; missing homes deferred.
- Product unblock: forever cites for nested faces + observability prometheusrule alias.
- No hubs / Cargo.lock / merge / #1661 / cloud-os absorb.

## Seat A keep_forever interior (wave-6) — 2026-08-10

PREP only (no merge). Tip base `17afaecea`. Envelope `storage/**`.

### Retargeted (verified)

- Chart/home owner URLs (dest trees exist):
  - `storage/drive/iac/helm/Chart.yaml` home `…/microservices/drive` → `…/storage/drive`
  - `storage/recordings/iac/helm/recordings/Chart.yaml` maintainer url `…/microservices/recordings` → `…/storage/recordings`
  - `storage/recordings/contracts/openapi/recordings.yaml` contact url `…/microservices/recordings` → `…/storage/recordings`
- OpenSLO cites (dest `storage/observability/slos/<NAME>` exists):
  - `storage/recordings/runbooks/legal-hold-court-order-receipt.md`:
    - `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml` → `storage/observability/slos/legal-hold-engagement-latency.openslo.yaml`
    - `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml` → `storage/observability/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`

### Deferred (do not invent / out of this slice)

- Missing IP-001..015 / PRD / ARCHITECTURE / compliance / historical ADR-RECORDINGS-* under drive/recordings/imaging (leave manifest + runbook legacy cites)
- AUDIT-FINDINGS historical MISSING lists (incl. `microservices/drive/slos/file-list-latency.openslo.yaml`)
- Cross-cap observability prometheusrule remaps
- Hubs / Cargo.lock / merge / #1661
