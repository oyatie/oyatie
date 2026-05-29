# Wave 15J-batch-4 tier scrub remediation notes: connector

## Scope

- Service: `connector`
- Doctrine: ADR-0329, ADR-0330, ADR-0331
- Deleted `capability-tiers/` directory: Y

## Files modified with line counts

- `microservices/connector/README.md` - 66 lines
- `microservices/connector/manifest.json` - 134 lines
- `microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md` - 104 lines
- `microservices/connector/tutorials/establish-cross-tenant-channel-with-mls-and-cedar.md` - 286 lines
- `microservices/connector/faqs/federation-engineer-faq.md` - 122 lines
- `microservices/connector/ARCHITECTURE.md` - 1066 lines
- `microservices/connector/incident-response.md` - 72 lines
- `microservices/connector/coherence-audit-2026-05-20.md` - 655 lines
- `microservices/connector/RETIREMENT-PLAN.md` - 238 lines
- `microservices/connector/reference-implementations/cross-tenant-message-rust-sdk.md` - 243 lines
- `microservices/connector/capabilities/connector-invoke.yaml` - 107 lines

## Replacement count

Rough vocabulary replacements: ~30 lines across the active and untracked connector service tree, plus the directory deletion.

## Design decisions

- Replaced customer-facing Bronze/Silver/Gold/Platinum copy with `tenant_class` language.
- Mapped PHI/regulated examples to compliance-pack gating instead of paid ladder gating.
- Treated performance/topology distinctions as `cell_topology` or operational deployment shape, not commercial class.
- Preserved non-customer infrastructure criticality references when they were outside the retired vocabulary targeted by ADR-0329.
- Updated README to cite ADR-0330 and state `billing_components` as the paid commercial model.

## Outstanding follow-ups

None for the assigned zero-residue vocabulary gate.

## Wave 15-IP-substance scrub (2026-05-21)

- Assigned bucket: IP-BUCKET-J / Wave 15-IP-substance.
- Rewritten in place: 5 stamped or short-shell IPs.
- Preserved as already-substantive with counterpart evidence pointer where needed: 46 IPs.
- Deleted as duplicative: 0. No pair was merged because the apparent duplicate journey names carry different journey IDs or regulatory overlays.
- Source grounding used: `microservices/connector/PRD.md`, `microservices/connector/ARCHITECTURE.md`, `microservices/connector/competitor-parity-matrix.md`, service `manifest.json`, contracts, policy, SLO, catalog, runbook, and IaC artifacts. No nonexistent `src/` paths were invented; these three assigned services have no `microservices/<ms>/src` tree in this checkout.
- Rewritten files:
  - `microservices/connector/IP-007-retry-dlq-domain.md`
  - `microservices/connector/IP-008-rest-surfaces.md`
  - `microservices/connector/IP-010-iac-postgres-openbao.md`
  - `microservices/connector/IP-011-slos-dashboards-observability.md`
  - `microservices/connector/IP-015-connector-adapter-trait-doc.md`
- Follow-up: implementation PRs must create the declared crates/types before claiming cargo-test evidence; this scrub only converts IP documentation from stamp to service-grounded plan content.
### Wave 15-IP-substance validation correction

- Expanded remaining 30-79 line `connector` IPs after the first validation pass still showed stamp-shell line-count signatures.
- Additional rewritten files:
  - `microservices/connector/IP-001-connect-retirement-design-readiness.md`
  - `microservices/connector/IP-003-oauth-broker-domain-kernel.md`
  - `microservices/connector/IP-004-webhook-receiver-domain.md`
  - `microservices/connector/IP-006-data-mapping-domain.md`
  - `microservices/connector/IP-009-connector-catalog-seed.md`
  - `microservices/connector/IP-012-wave2-connectors.md`
  - `microservices/connector/IP-013-connector-adapter-trait.md`
  - `microservices/connector/IP-014-compliance-critical-path.md`

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:

- `microservices/connector/PRD.md`
- `microservices/connector/backfill-replay.md`
- `microservices/connector/capacity-model.md`
- `microservices/connector/multi-region.md`
- `microservices/connector/policy/tenant-isolation.md`
- `microservices/connector/runbooks/connector-rate-limit-saturation.md`
- `microservices/connector/threat-model.md`

Counterpart-fact preservations:

- `microservices/connector/IP-012-wave2-connectors.md` preserves `Redis` as a counterpart-fact external database connector.
- `microservices/connector/IP-009-connector-catalog-seed.md` preserves `Redis` as a counterpart-fact external database connector.

Files renamed (git mv):

None.
