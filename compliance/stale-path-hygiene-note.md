---
doc_class: JudgmentNote
title: Stale microservices/compliance path hygiene (Seat A wave-1)
status: Accepted
owner_team: axis-compliance
date: 2026-08-10
related_artifacts:
  - compliance/manifest.json
  - compliance/runbooks/
ssot_todo: free-capability-compliance-prep
---

# Stale path hygiene note (`compliance/**`)

## Chesterton challenge

`microservices/compliance/...` citations were left as strangler compatibility after the capability tree landed under `compliance/`. On this tip the legacy directory is **gone** (`test ! -d microservices/compliance`). Keeping dead absolute citations fails day-2 operability; inventing missing trees (PRD/ARCHITECTURE/MIGRATION/policy `.md`) to satisfy greps would violate YAGNI and the wave envelope.

## Wave-1 scope (retargeted)

| Surface | Change |
|---|---|
| `compliance/manifest.json` | Rewritten to ADR-0562 capability-root format (`schema_version` + `capability`); tree-read crates/contracts/OpenSLO; all `microservices/compliance/**` path claims removed |
| `compliance/runbooks/*.md` (13) | `microservices/compliance/` → `compliance/`; `…/slos/` → `compliance/observability/slos/`; policy check simplified to verified `.cedar` |
| `compliance/iac/helm/evidence-collector/*` | Home retargeted where counterpart exists |
| `compliance/scorecards/*`, `AUDIT-FINDINGS-*`, IP docs | `microservices/compliance/` cites retargeted when destination exists |

### Citation inventory (tip after wave-1)

| Metric | Count |
|---|---|
| `microservices/compliance` cites closed (verified dest + cedar-only policy checks) | 284 |
| Remaining `microservices/compliance` cites (missing dest) | 2 (`README.md` PRD + ARCHITECTURE) |
| Remaining other `microservices/*` cites (cross-capability / cloud-iac / governance / specs schema `$id`) | deferred below — not invented; `specs/**` out of envelope |

## Left as intentional legacy (destination missing — do not invent)

| Missing / foreign cite | Example cite homes |
|---|---|
| `compliance/PRD.md` | `compliance/README.md` |
| `compliance/ARCHITECTURE.md` | `compliance/README.md` |
| `policy/pack-overlay-authorization.md` | removed from runbook `test -f` branches; `.cedar` retained |
| `MIGRATION-2026-05-21.md` | dropped with capability-root rewrite (was only on pre-rewrite tenant pinning) |
| Cross-capability `microservices/{identity,payments,mail,…}/PRD.md` | `compliance/IP-journey-j*.md` historical journey overlays |
| `microservices/cloud-iac/modules/` | `compliance/IPs/IP-ADR-0339-Shared-IaC-Modules.md`, `IP-WAVE-15-*` |
| `microservices/governance/iac/helm/_oya-helpers` | `compliance/iac/helm/evidence-collector/{Chart,values}.yaml` |
| `specs/microservices/*-schema.json` `$schema` refs | `AUDIT-FINDINGS-*.json`, `scorecards/overrides.json` (specs envelope; not rewritten here) |

## Non-claims

- No hubs (`specs/**`), no `Cargo.lock`, no merge in this wave.
- OpenSLO files under `compliance/observability/slos/` are declared in `compliance/manifest.json` from verified paths; runtime SLI readiness is not claimed beyond the OpenSLO authoring already in-tree.
