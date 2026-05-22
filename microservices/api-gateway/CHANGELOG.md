# api-gateway — CHANGELOG

All notable changes per SemVer 2.0 + ADR-0258 versioning policy.

## [0.2.0] — 2026-05-20 — Wave-3-A buildout (PHASE-01)

### Added

- `ARCHITECTURE.md` — full layer-by-layer trace; binding to ADR-0157, ADR-0182, ADR-0253, ADR-0297 (in flight).
- `PHASE-01-EDGE-SUBSTRATE-BUILDOUT.md` — phase scope + sequencing.
- `threat-model.md` — STRIDE + LINDDUN; 9 adversary classes; control-roster cross-ref.
- `dpia.md` — GDPR Art. 35 DPIA; pack overlay table.
- `README.md`, `compliance.md`, `capacity-model.md`, `cost-budget.md`, `failure-modes.md`, `multi-region.md`, `incident-response.md`, `backfill-replay.md`, `competitor-parity-matrix.md`, `sdk-plan.md`.
- Cedar fragments: `route-authorization.cedar`, `rate-limit.cedar`, `abuse-defence.cedar`, `tls-policy.cedar`, `sov-cloud-overlay.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`, `public-read.cedar`.
- Runbooks: `ddos-mitigation.md`, `rate-limit-saturation.md`, `tls-cert-rotation.md`, `bot-storm.md`, `edge-cache-poisoning.md`, `blue-green-rollback.md`, `h3-fallback-verification.md`, `ech-key-rotation.md`, `pqc-cert-rotation.md`, `circuit-breaker-engaged.md`, `cell-evac.md`.
- Contracts: extended OpenAPI 3.2.0 routing-control-plane; AsyncAPI 3.1.0 routing-events; proto3 gRPC management plane; `metric-naming-convention.md`.
- Capabilities: `edge-cedar-eval.yaml`, `canary-route-shift.yaml`.
- Dashboards: `edge-overview.json`+`.md`, `rate-limit-hits.json`+`.md`, `tls-health.json`+`.md`, `bot-score-distribution.json`+`.md`.
- SLOs: `edge-availability`, `edge-latency-p50/p95/p99`, `tls-handshake-success`, `h3-negotiation-rate`.
- IPs: IP-002..IP-016 covering routing/rate-limit/auth-handoff/abuse-defence × {domain,kernel,usecase,adapter,rest,grpc,worker}.
- Catalog records: 11+ entries.
- IaC: `k8s-deployment.yaml`, `envoy-config.yaml`, `cloudflare-config.yaml`, `cert-manager.yaml`, `ech-config.yaml`, `pqc-cert.yaml`, `edge-waf.yaml`, `spire-trust-bundle.yaml`.
- `scorecards/overrides.json`.
- `AUDIT-FINDINGS-2026-05-20.json`.

### Changed

- `manifest.json` extended to reference all new artifacts.
- `PRD.md` extended from 117 to ≥1500 lines per documentation-rigor.md §2.

### Compatibility

- No breaking changes to public contracts. OpenAPI version stays at v1; AsyncAPI version stays at v1.
- New audit-event classes added (per ADR-0263 §D registry); consumers SHOULD subscribe.

## [0.1.0] — 2026-05-18

- Initial scaffold: 16 artifacts (PRD stub, manifest, threat-model stub, IP-001).
- Baseline audit findings (`AUDIT-FINDINGS-2026-05-18.json`).
