---
doc_class: Runbook
title: Adapter version pin (per-tenant)
microservice: foundry-providers
severity: "Sev-3 (planned pin) / Sev-2 (defensive pin during vendor breaking change)"
status: Accepted
owner_team: axis-foundry + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/threat-model.md (T-05 adapter substitution; T-06 subscription channel substitution)
  - microservices/intelligence/PRD.md §"FR-13 adapter version pin"
  - microservices/intelligence/runbooks/provider-outage-failover.md
doc_status: published
---

# Runbook: Adapter version pin (per-tenant)

## Trigger

ONE of:

1. **Vendor breaking change announced** — vendor announces upcoming model deprecation, API format change, or subscription-channel restructuring.
2. **Vendor breaking change observed** — `oya_foundry_providers_response_shape_anomaly_total{vendor="<v>"}` rises; adapter is auto-quarantined per T-03 mitigations.
3. **Tenant request** — tenant has a stability requirement and asks to pin to a known-good adapter version.
4. **Audit / compliance window** — tenant in a regulatory audit period asks to freeze adapter behavior for the duration.

## Severity

- Planned pin ahead of vendor change: **Sev-3** (document; execute).
- Defensive pin during observed vendor breaking change: **Sev-2**.

## Pre-checks

1. Identify the known-good adapter version: `oya_foundry_providers_adapter_version{vendor="<v>",status="known-good"}` returns the latest tagged version.
2. Confirm the tenant's current adapter version: `oya_foundry_providers_tenant_adapter_pin{tenant="<t>",vendor="<v>"}`.
3. Confirm the tenant's capability requirements are met by the known-good version (some new features may be on newer versions; pinning trades stability for capability).
4. Confirm 2-person approval for the pin change (per `policy/credential-isolation.md` CI-INV-09 process discipline).

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | If Sev-2 (observed vendor change): open `#inc-<id>`; IC + axis-foundry SME | ≤ 5 min |
| 2 | Confirm pre-checks above | ≤ 5 min |
| 3 | Apply the pin: `cargo run -p oya-dev-cli -- providers pin-adapter --tenant <t> --vendor <v> --version <known-good> --reason "<id>" --approver <person-1> --approver <person-2>` (2-person signed; audit-emitted) | ≤ 3 min |
| 4 | Verify pin: `oya_foundry_providers_tenant_adapter_pin{tenant="<t>",vendor="<v>"}` returns `<known-good>` | ≤ 1 min |
| 5 | Verify tenant workload continues normally on pinned version | ≤ 10 min |
| 6 | Notify tenant operator: pin applied; new-version features unavailable until unpin; recommended renew/unpin cadence (default: review after 30 d) | ≤ 30 min |
| 7 | For Sev-2: continue investigation of the vendor breaking change; tracker filed | – |

## Unpinning

When the vendor change has been absorbed by a new adapter version (response-shape conformance + baseline-set parity verified):

| Step | Action |
|---|---|
| 1 | Confirm new adapter version is `known-good` (tagged after parity tests pass) |
| 2 | Confirm tenant capability requirements unchanged or improved |
| 3 | Update pin: `cargo run -p oya-dev-cli -- providers unpin-adapter --tenant <t> --vendor <v> --reason "<id>" --approver <p1> --approver <p2>` |
| 4 | Confirm tenant workload on new version (canary-style ramp if cautious) |

## Rollback (of the pin)

If the pin itself causes issues (e.g., known-good version has its own bug):
1. Identify a different known-good version (older or newer).
2. Apply that pin.
3. Postmortem: how did the known-good designation slip?

## Verification

- `oya_foundry_providers_tenant_adapter_pin{tenant="<t>",vendor="<v>"}` returns the intended version.
- Tenant 5xx rate baseline for the workload.
- `evidence/adapter-pins/<tenant>-<vendor>-<unix_ts>.json` audit record present.
- Pin review reminder scheduled (default 30 d).

## Post-incident updates

- If the vendor breaking change was unexpected: add to `policy/data-residency.md` notes for that vendor.
- If a new adapter version absorbed the change: tag as `known-good` after parity tests pass.
- Adapter publish protocol (per CI-INV-09) emphasises 2-person review for any new tag.

## References

- `microservices/intelligence/threat-model.md` T-05 + T-06.
- `microservices/intelligence/PRD.md` FR-13.
- ADR-0133 — industry-best-practice conformance program.
