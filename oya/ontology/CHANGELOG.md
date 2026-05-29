---
doc_class: Reference
shape: Reference
status: Accepted
date: 2026-05-20
---

# Ontology µservice — Changelog

## Unreleased

- Added `ARCHITECTURE.md`, `README.md`, `CHANGELOG.md` per documentation-rigor §2 ArchitectureWalkthrough rigor.
- Added `policy/abuse-defence.cedar` per ADR-0297 abuse-defence baseline.
- Added `policy/cross-tenant-refusal.cedar` enforcing ADR-0257 amendment cross-tenant projection refusal.
- Added `policy/ontology-write-quota.cedar` enforcing per-tenant write quota gates.
- Added `runbooks/ontology-read-library-fallback.md`, `runbooks/share-token-revocation.md`, `runbooks/ontology-bot-score-recalibration.md`, `runbooks/postgres-citus-rebalance.md`.
- Added `IP-016-read-path-library-rollout.md`, `IP-017-share-token-surface.md`, `IP-018-abuse-defence-edge-wiring.md`.
- Added IaC: `iac/<env>-edge-waf.yaml`, `iac/<env>-ech-config.yaml`, `iac/<env>-pqc-cert.yaml`, `iac/openbao-policy.yaml`, `iac/network-policy.yaml`, `iac/secret-bindings.yaml`.
- Added catalog records: `oya-ontology-read-path-library.yaml`, `oya-ontology-share-token-kernel.yaml`, `oya-ontology-write-quota-adapter-postgres.yaml`.
- Added dashboards: `read-path-library-freshness.json`, `abuse-defence-outcomes.json`.
- Added SLO `ontology-read-path-library-freshness.openslo.yaml`.

## 0.1.0 — 2026-05-18

- Initial scaffolding per PR-143 with 88 artifacts.
- 15 IPs covering type-registry through agent-gateway.
- ADR-0257 amendment landed (read-path library-first dispatch).
