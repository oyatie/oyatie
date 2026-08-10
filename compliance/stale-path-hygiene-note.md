---
doc_class: JudgmentNote
title: Stale microservices/compliance path hygiene (Seat A wave-5)
status: Accepted
owner_team: axis-compliance
date: 2026-08-10
related_artifacts:
  - compliance/manifest.json
  - compliance/IP-journey-j43-hipaa-cell-overlay.md
  - compliance/runbooks/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`compliance/**` — Seat A wave-5)

## Chesterton challenge

Wave-1 closed most `microservices/compliance/**` cites. Residual `microservices/*` cites remain as strangler leftovers or cross-capability historical journey overlays. Retarget only **verified** in-tree destinations. Do not invent missing PRD/ARCHITECTURE/homes. No hubs, no `Cargo.lock`, no merge.

## Wave-5 retargeted (verified)

| Surface | Change |
|---|---|
| `compliance/IP-journey-j43..j48` | `microservices/identity/PRD.md` → `iam/identity/PRD.md` (exists on tip) |

## Deferred (destination missing — do not invent)

| Missing / foreign cite | Example cite homes |
|---|---|
| `compliance/PRD.md` / `compliance/ARCHITECTURE.md` | `compliance/README.md` |
| Cross-capability `microservices/{payments,workflow-engine,ontology,messenger,mail,community}/PRD.md` | `compliance/IP-journey-j*.md` historical overlays |
| `microservices/cloud-iac/modules/` | `compliance/IPs/IP-ADR-0339-*`, `IP-WAVE-15-*` (peer tips also defer) |
| `microservices/governance/iac/helm/_oya-helpers` | `compliance/iac/helm/evidence-collector/{Chart,values}.yaml` |
| `specs/microservices/*-schema.json` `$schema` refs | `AUDIT-FINDINGS-*.json`, `scorecards/overrides.json` (specs envelope; out of tip) |

## Non-claims

- No hubs (`specs/**`), no `Cargo.lock`, no merge in this wave.
- Observation≠APPROVE. **STOP #1661.** Healthcare FLAT retained.
