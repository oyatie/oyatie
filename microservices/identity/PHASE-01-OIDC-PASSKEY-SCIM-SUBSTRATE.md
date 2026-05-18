---
doc_class: PHASE-SPEC
template_id: TPL-PHASE-SPEC
phase_id: PHASE-01-identity-oidc-passkey-scim-substrate
microservice: identity
status: Accepted
milestone: M01-foundation
related_adrs: [ADR-0117, ADR-0131, ADR-0145, ADR-0148, ADR-0156, ADR-0157, ADR-0162, ADR-0175, ADR-0179, ADR-0182, ADR-0183, ADR-0187, ADR-0188, ADR-0189, ADR-0190, ADR-0191]
date: 2026-05-18
owner_team: axis-identity
---

# PHASE-01 — identity: OIDC + Passkey + SCIM + Step-up Substrate

This phase delivers the full Zitadel-backed identity substrate per ADR-0187 — OIDC issuer, WebAuthn Level-3 relying party, SCIM 2.0 inbound, step-up ACR enforcement, edge-vs-origin authz tier — as a deployable per-pack µservice ready for every other oyatie µservice to consume.

## Exit criteria

The phase is Complete when every IP-001 through IP-015 carries `acceptance_status: ga` in `manifest.json#ips`, all 9 OpenSLO targets are green over 30 rolling days in the pack-eu pack (the bellwether), `lean-a18-identity-vendor-isolation` advisory lane is clean, `lean-a15-step-up-acr-coverage` advisory lane shows ≥80% sensitive-path coverage, every Cedar policy under `microservices/identity/policy/` passes `oya-check-authz-tier-discipline`, and the audit-chain Merkle seal includes ≥1 of every event type in `manifest.json#audit_chain.seal_events` from production traffic.

## Implementation Plan inventory

| IP | Title | Acceptance |
|---|---|---|
| IP-001 | Zitadel Helm deployment per pack | ga |
| IP-002 | OIDC issuer kernel + JWKS verify | ga |
| IP-003 | OIDC issuer Zitadel adapter | ga |
| IP-004 | WebAuthn relying-party kernel | ga |
| IP-005 | WebAuthn register/authenticate REST | ga |
| IP-006 | FIDO-MDS3 AAGUID refresh worker | ga |
| IP-007 | SCIM 2.0 server kernel + RFC 7644 conformance | ga |
| IP-008 | SCIM Zitadel adapter (lifecycle propagation) | ga |
| IP-009 | HRIS adapter contract + Workday/BambooHR/Rippling impls | ga |
| IP-010 | Step-up ACR orchestrator + Cedar gate | ga |
| IP-011 | External IdP federation (Workspace/Okta/Entra) | ga |
| IP-012 | Audit emitter to audit-chain | ga |
| IP-013 | Edge authz rules (Coraza + rate-limit + geo) | ga |
| IP-014 | Continuous risk-scoring adjunct (CAEP) | ga |
| IP-015 | oya-shared-* kernel crate exports + reference impls | ga |
| IP-016 | Zitadel scale validation load test (GATE before pack-eu bellwether) | ga |

## Dependency graph

```
IP-015 → IP-002, IP-004, IP-007 (kernel crates first)
IP-001 → IP-003, IP-008 (Zitadel must be running before adapters)
IP-002 + IP-003 → IP-010 + IP-011 + IP-012 (OIDC must verify before step-up / federation / audit)
IP-004 + IP-005 → IP-006 (RP must work before AAGUID refresh hardens)
IP-007 → IP-008 + IP-009 (kernel before adapter / HRIS impls)
IP-010 → IP-014 (ACR before risk-scoring adjunct)
IP-013 is parallel to the OIDC path (edge-tier work)
IP-012 is the final wire; depends on every other emitter
```

## Promotion lane

Per ADR-0130 agentic SLO-gated promotion:

- **dev → staging**: IP-001 + IP-002 + IP-015 green; `cargo nextest` passes.
- **staging → pack-eu (bellwether)**: SLOs `oidc-token-issue-latency`, `jwks-availability`, `webauthn-authenticate-latency`, `scim-availability` green for 7 days; ACR step-up integration test passes against Zitadel real instance.
- **pack-eu → all packs**: bellwether green for 30 days; per-pack residency policy authored; Helm overlays applied; pack-kr / pack-us-healthcare / pack-ksa pass regulated-pack AAGUID-allowlist conformance.

## Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Zitadel upstream stalls or licensing changes | low | high | ADR-0187 §In-house roadmap Phase 2 swap blueprint; adapter boundary already in place |
| FIDO-MDS3 metadata blob fetch failure | medium | medium | local cache + fallback AAGUID allowlist per pack; alert if stale > 48h |
| SCIM dialect divergence (Okta vs Entra vs Workspace) | high | medium | dialect-quirks doc + conformance test against each vendor's SCIM sandbox |
| Postgres event-store migration during minor upgrade | medium | high | runbook `identity-zitadel-upgrade`; staging dry-run mandatory |
| Hardware-key cost vs UX friction for `acr=critical` | medium | medium | T&M tracker on critical-op success rate; YubiKey 5C provisioning included in onboarding budget |
| Cross-pack cookie / session bleed | low | critical | per-pack subdomain; SameSite=Strict; no `*.oyatie.dev` cookie scope |
