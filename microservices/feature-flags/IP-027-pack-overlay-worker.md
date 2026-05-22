# IP-027 — Pack Overlay Worker

**microservice**: feature-flags
**bc**: flag
**layer**: worker
**crate**: oya-feature-flags-pack-overlay-worker
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0243, ADR-0247, ADR-0248, ADR-0251, ADR-0263, ADR-0293
**companion_ips**: IP-003, IP-018

## Scope

Dedicated worker for applying compliance pack flag overrides: subscribes to pack activation events, applies Cedar-validated pack overlay rules to flag definitions, emits `PackFlagOverrideApplied` audit events, enforces Foundry cosign attestation on all overlay operations.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `PackOverlayWorker` | Kafka consumer for `oya.platform.pack.activated`; processes pack overlay rules per tenant |
| 2 | Cedar pre-check | Every `PackOverrideApply` evaluated against `pack-overlay-authorization.cedar`; cosign attestation verified |
| 3 | Overlay application | Calls `FlagMutationService::apply_pack_override`; tenant-scoped; EMERGENCY_SERVICES flags NEVER overridden to disable |
| 4 | Idempotency | Pack overlay application is idempotent: same pack applied twice → same result, no duplicate audit events |
| 5 | Audit events | `PackFlagOverrideApplied` per flag modified; sealed in audit chain per ADR-0263 |
| 6 | FORBID guard | Cedar FORBID: pack override that sets `enabled=false` on any flag where `is_emergency_services_flag=true` → rejected with `PolicyViolation` error |
| 7 | Tests | Cosign attestation missing → `PolicyViolation`; EMERGENCY_SERVICES FORBID test; idempotency test (2× apply → 1× audit event) |

## Pack Overlay Roster (12 rules)

| Pack | Affected Flags | Override Type |
|------|---------------|---------------|
| HIPAA | PHI-related feature flags | Disable client-side export |
| PCI-DSS | Payment UI flags | Disable on non-PCI cells |
| FedRAMP-High | All govcloud flags | Force-enable audit logging |
| KR-FSS | KR-cell flags | Enable KR ISMS-P audit trail |
| EU-AI-Act | ML decision flags | Force-enable explainability |
| GDPR-EU | Data processing flags | Enable consent-gate |
| COPPA | MINOR_TARGETED flags | Force-disable without parental consent |
| EU-Child-Safety | MINOR_TARGETED experiments | Disable A/B tests for minors |
| SOC2-Type2 | All flags | Enable continuous monitoring |
| ISO-27001 | All flags | Enable ISO audit trail |
| CSAP | China-cell flags | Enable MLPS audit |
| CCPA | CA-resident flags | Enable opt-out respect |

## Definition of Done

- `cargo test -p oya-feature-flags-pack-overlay-worker` green
- All 12 pack overlay rules applied correctly in integration test
- EMERGENCY_SERVICES FORBID: attempt to disable `NENA_I3_ROUTING` via pack override → rejected
- Cosign verification: unsigned overlay → `PolicyViolation`
