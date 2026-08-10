---
doc_class: JudgmentNote
title: Stale microservices/feature-flags path hygiene (Seat A wave-5)
status: Accepted
owner_team: axis-flags
date: 2026-08-10
related_artifacts:
  - flags/manifest.json
  - flags/IPs/IP-ADR-0339-Shared-IaC-Modules.md
  - flags/runbooks/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`flags/**` — feature-flags / Seat A wave-5)

## Chesterton challenge

`feature-flags` forever home is `flags/**` (`integ/flags` / `#1650`). Residual `microservices/feature-flags/**` cites are strangler leftovers. Retarget only **verified** in-tree destinations under `flags/**`. Do not invent missing PRD/ARCHITECTURE/IP-00N bodies. No hubs, no `Cargo.lock`, no merge.

## Wave-5 retargeted (verified)

| Surface | Change |
|---|---|
| `flags/IPs/IP-ADR-0339-Shared-IaC-Modules.md` SCOPE-002 | `microservices/feature-flags/iac/<context>/main.tf` → `flags/iac/<context>/main.tf` (`flags/iac/` exists) |

## Deferred (destination missing — do not invent)

| Missing / foreign cite | Example cite homes |
|---|---|
| `flags/PRD.md` / `flags/ARCHITECTURE.md` | `flags/README.md`, `flags/manifest.json`, `rust-mock-provider.rs` |
| `flags/IP-001`…`IP-020` bodies (not in tree; only ADR/WAVE IPs present) | `flags/manifest.json` `ips[]` |
| `flags/contracts/openfeature-sdk-contract.md` | `flags/manifest.json` contracts |
| `flags/{incident-response,backfill-replay,compliance}.md` | `flags/runbooks/*` related refs |
| `microservices/cloud-iac/modules/` | `flags/IPs/IP-ADR-0339-*`, `IP-WAVE-15-*` (peer tips also defer) |
| `VERIFY-028` `microservices/#{ms}/IPs/` template | `flags/IPs/IP-WAVE-15-*` (peer tips also defer) |

## Non-claims

- No hubs (`specs/**`), no `Cargo.lock`, no merge in this wave.
- Observation≠APPROVE. **STOP #1661.**
