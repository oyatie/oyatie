---
doc_class: JudgmentNote
title: Stale microservices/{identity,cloud-iam,consent-graph} path hygiene (wave-5 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - iam/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`iam/**` — Seat A wave-5)

## Scope

Retarget only **verified** in-tree destinations under `iam/**`. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- `microservices/identity/**` → `iam/identity/**` (IPs, iac/helm, policy, contracts, PRD/manifest where present)
- `microservices/identity/slos/**` → `iam/observability/slos/identity/**` when OpenSLO files exist
- `microservices/cloud-iam/**` → `iam/cloud-iam/**`
- `microservices/consent-graph/**` → `iam/consent-graph/**` (contracts/IPs/iac)

## Deferred

- Missing competitor-parity / testing-strategy / FIDO schema / some historical IP homes
- Cross-cap cites (payments, ontology, messenger, …) stay deferred

## Seat A wave-6 dep-ordered (2026-08-10)

- Verified remaps applied: **4** cite(s) across **1** file(s).
- Scope: path/manifest/SLO/contract/capability/catalog high-value only; missing homes deferred.
- Product unblock: forever cites for nested faces + observability prometheusrule alias.
- No hubs / Cargo.lock / merge / #1661 / cloud-os absorb.
