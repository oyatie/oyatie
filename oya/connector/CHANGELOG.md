---
microservice: connector
doc_class: CHANGELOG
date: 2026-05-20
owner_team: axis-integration
status: Accepted
related_adrs: [ADR-0258]
doc_status: published
---

# Changelog — connector

All notable changes per [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) + SemVer per ADR-0258.

## [0.2.0] — 2026-05-20

### Added
- Integration-substrate scope: connector-catalog, oauth-broker, webhook-receiver, signature-verification, payload-canonicalization, connector-adapter, data-mapping, retry-and-DLQ BCs.
- 500+ connector adapter catalog seed (30 in this PR; remainder over M01–M03).
- OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3 contracts.
- 6+ Cedar policy fragments (default-deny + 7 permits + UX-floor abuse-defence).
- 6+ runbooks (cascade-failure, OAuth revocation, replay attack, rate-limit saturation, signature cascade, DLQ overflow).
- 15+ implementation plans across kernel/domain/usecase/adapter/rest/grpc/worker layers.
- 8+ IaC modules (Helm chart, Terraform module, OpenBao policy, network policy).
- 7 SLOs (catalog query latency, OAuth grant success, webhook ingest availability, action dispatch availability, signature verify p99, DLQ replay success, audit completeness).
- Abuse-defence baseline per ADR-0297 + documentation-rigor §3.2.3.

### Changed
- µservice tier: `retiring` → `substrate` (per 2026-05-20 wave-3-B buildout).

### Retained (umbrella-retirement coordination)
- `RETIREMENT-PLAN.md` + `IP-001-connect-retirement-design-readiness.md` + retirement contracts: continue to track umbrella dissolution per ADR-0237.

## [0.1.0] — 2026-05-18

### Added
- Umbrella-retirement coordination surface (platform dissolution).
- Retirement plan + IP-001 + retirement contracts/capabilities/SLO/policy.
