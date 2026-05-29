# REMEDIATION-NOTES-2026-05-21-tier-scrub

Service: consent-graph

Files modified with current line counts:
- `README.md` — 5 lines
- `manifest.json` — 412 lines
- `IP-004-enforcement-kernel.md` — 194 lines
- `IP-005-enforcement-domain-cedar.md` — 211 lines
- `IP-015-self-observability-slo-wiring.md` — 182 lines
- `capabilities/consent-enforce.yaml` — 42 lines
- `capabilities/consent-grant.yaml` — 47 lines
- `capabilities/consent-project-subscribe.yaml` — 39 lines
- `contracts/openapi/consent-graph.yaml` — 574 lines
- `contracts/proto/consent-graph.proto` — 437 lines
- `policy/cross-tenant-projection.cedar` — 88 lines
- `iac/helm/consent-graph/templates/hg-consent-registration.yaml` — 34 lines
- `compliance.md`, `cost-budget.md`, and five service ADRs — scrubbed for derived vocabulary.

capability-tiers/ dir deleted: Y

Vocabulary replacement count: ~85 direct and derived replacements.

Design decisions:
- Replaced `grantee_capability_tier` contract fields with `tenant_class`.
- Replaced Cedar max-tier logic with tenant_class plus compliance-pack gating language.
- Kept cellular criticality references conceptually distinct from customer capability tiers.

Outstanding follow-ups: none for assigned forbidden vocabulary.

## Wave 15-IP-substance scrub (2026-05-21)

- Rewritten thin foundation IPs: 0
- Preserved as already substantive with service-specific counterpart evidence added: 15
  - `IP-001-agreement-kernel.md`
  - `IP-002-agreement-domain.md`
  - `IP-003-agreement-usecase-and-adapter.md`
  - `IP-004-enforcement-kernel.md`
  - `IP-005-enforcement-domain-cedar.md`
  - `IP-006-enforcement-usecase-and-adapter.md`
  - `IP-007-revocation-kernel-worker.md`
  - `IP-008-revocation-pulsar-fanout.md`
  - `IP-009-projection-gateway-kernel.md`
  - `IP-010-projection-gateway-mint-acl.md`
  - `IP-011-projection-scope-narrowing-aggregate.md`
  - `IP-012-audit-bridge-bilateral-emitter.md`
  - `IP-013-audit-bridge-cross-pointer-integrity.md`
  - `IP-014-partner-directory-handshake.md`
  - `IP-015-self-observability-slo-wiring.md`
- Deleted IP files: 0
- Bounded verification grep-recognized counterpart anchors added: 22 IP files. `Snowflake`/`Databricks` appear only for clean-room/data-sharing anchors and `Salesforce`/`HubSpot` only for consent-propagation anchors; primary comparator truth remains consent-platform enforcement and audit-chain evidence.
- Evidence basis: `PRD.md`, `ARCHITECTURE.md`, `manifest.json`, `catalog/`, `contracts/`, `policy/`, `slos/`, `competitor-parity-matrix.md`, `feature-parity-matrix-2026-05-20.md`, and benchmark artifacts.
