---
doc_class: IP
ip_id: IP-016
microservice: identity
status: ga
related_adrs: [ADR-0187]
date: 2026-05-18
owner_team: axis-identity + ops-sre-reliability + axis-perf
---

# IP-016 — Zitadel scale validation load test (GATE BEFORE pack-eu BELLWETHER)

## Goal

Verify the projected per-pack capacity ceilings BEFORE pack-eu becomes the production bellwether. The capacity-model.md numbers are PROJECTIONS until this IP reports green. If any projection fails, the result drives the Phase-2 timeline per ADR-0187 §"In-house roadmap" trigger criterion (d).

**This is a HARD gate. pack-eu cannot enter prod-bellwether status until this IP returns green for year-1 numbers and amber/green for year-3.**

## Test matrix

### Year-1 validation (mandatory PASS before pack-eu bellwether)

| Scenario | Steady load | Burst load | Pass criterion |
|---|---|---|---|
| OIDC token issuance | 50 rps | 500 rps for 5min | p99 ≤ 80ms throughout |
| OIDC token verification | 1,000 rps | 5,000 rps for 5min | p99 ≤ 2ms; cache-hit ≥ 99% |
| WebAuthn authenticate | 30 rps | 300 rps for 5min | p99 ≤ 100ms |
| WebAuthn register | 5 rps | 50 rps for 5min | p99 ≤ 250ms |
| SCIM POST Users | 2 rps | 200 rps for 5min | p99 ≤ 500ms |
| SCIM PATCH (active=false) | 5 rps | 100 rps for 5min | p99 ≤ 500ms |
| Step-up grant | 1 rps | 30 rps for 5min | p99 ≤ 8s (UX-bound) |
| Audit emit | 100 eps | 1,000 eps for 5min | completeness = 1.0; p99 emit ≤ 300ms |

### Year-3 amber-green validation (informational; drives Phase-2 timeline)

| Scenario | Steady load | Burst load | Tracking metric |
|---|---|---|---|
| OIDC token issuance | 500 rps | 5,000 rps for 5min | p99 ≤ 100ms = GREEN; 100-200ms = AMBER; > 200ms = RED (Phase-2 trigger) |
| Tenants in single Zitadel Instance | 10,000 | n/a | sustained over 1h; p99 latency budget honoured = GREEN; > 200ms = AMBER; refuse-to-create = RED |
| Postgres event-store write IOPS | 5,000 wps | 20,000 wps for 5min | sustained without pgcat saturation > 80% |

### Year-5 stress validation (Phase-2 trigger probe)

| Scenario | Steady load | Pass = continue Phase 0; Fail = Phase 2 now |
|---|---|---|
| Tenants per Instance | 50,000 | sustained sub-200ms p99 token-issue |
| Cross-region active-active failover | RTO ≤ 30s | Zitadel does NOT support split-brain-safe multi-region writes natively → AMBER; failover with degraded RTO = AMBER; refuse-to-failover = RED |

## Tooling

- **k6** (Go-based load gen; Apache-2.0; OSS) for synthetic load.
- **vegeta** (Go-based; Apache-2.0; OSS) for parallel burst load.
- Per-µservice mTLS client cert from cloud-secrets OpenBao path.
- Prometheus + Grafana dashboards for live observation.
- Audit-chain replay verifies completeness post-run.

## Files

| File | Purpose |
|---|---|
| `microservices/identity/iac/loadtest/k6-token-issuance.js` | k6 scenario for token issuance |
| `microservices/identity/iac/loadtest/k6-token-verification.js` | k6 scenario for verify |
| `microservices/identity/iac/loadtest/k6-webauthn-authenticate.js` | WebAuthn assertion load |
| `microservices/identity/iac/loadtest/k6-scim-bulk-provision.js` | SCIM bulk-import |
| `microservices/identity/iac/loadtest/k6-step-up-grant.js` | step-up flow |
| `microservices/identity/iac/loadtest/k6-audit-completeness.js` | audit-emit at 1000 eps |
| `microservices/identity/iac/loadtest/scenarios.yaml` | scenario configs (load, duration, env) |
| `microservices/identity/iac/loadtest/run.sh` | orchestrator with prometheus snapshot |

## Procedure

1. Spin up dedicated load-test pack (`pack-loadtest`) on dev cluster.
2. Pre-populate Postgres with synthetic-tenant set:
   - 1,000 tenants (year-1 scenario)
   - 10,000 tenants (year-3 scenario)
   - 50,000 tenants (year-5 scenario)
3. Run k6 scenarios in matrix above; collect prometheus snapshots.
4. Verify SLO budgets honoured: no 4-window burn alert during the run.
5. Verify audit-chain completeness = 1.0 post-run.
6. Verify Postgres event-store consistency check passes.
7. Emit evidence report.

## Evidence to emit

| Artefact | Path |
|---|---|
| Per-scenario k6 JSON output | `evidence/identity/loadtest/k6-<scenario>-<date>.json` |
| Prometheus snapshot | `evidence/identity/loadtest/prom-snapshot-<date>.tar.gz` |
| Grafana dashboard PNG (core signals) | `evidence/identity/loadtest/grafana-<date>.png` |
| Postgres consistency check log | `evidence/identity/loadtest/pg-consistency-<date>.log` |
| Audit-completeness post-run | `evidence/identity/loadtest/audit-completeness-<date>.json` |
| **GO/NO-GO REPORT** | `evidence/identity/loadtest/go-no-go-<date>.json` |

## Go/no-go decision matrix

| Year-1 result | Year-3 result | Year-5 result | Decision |
|---|---|---|---|
| GREEN | GREEN | GREEN | Phase 0 continues; Phase 2 deferred indefinitely. |
| GREEN | GREEN | AMBER | Phase 0 continues; Phase 2 prep IPs (017-020) authored for Q1 2028 start. |
| GREEN | AMBER | RED | Phase 2 IPs accelerated; target start Q3 2027. |
| GREEN | RED | RED | **Phase 2 begins now**; Zitadel remains adapter for 12 months only. |
| AMBER | * | * | DO NOT promote pack-eu to bellwether. Diagnose. Retry. |
| RED | * | * | **STOP**. Phase 2 begins immediately. |

## Acceptance — DONE when

- All year-1 scenarios PASS.
- Year-3 scenarios report (any outcome — informational).
- Year-5 scenarios report (any outcome — informational).
- Go/no-go report committed to evidence/.
- Decision documented in ADR-0187 amendment or council-architecture meeting minutes.

## Cross-references

- ADR-0187 §"In-house roadmap" trigger criterion (d)
- capacity-model.md §"Scale validation status"
- ADR-0130 SLO-gated promotion

## Counterpart references - 016-zitadel-scale-validation-load-test

- Counterpart class: identity substrate.
- Palantir Foundry and GitHub Enterprise are the counterpart baseline for governed multi-tenant identity surfaces; this IP ties the slice to Oyatie identity contracts, Cedar, and audit-chain evidence rather than leaving the behavior as generic application authentication.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `iam/identity/PRD.md`, `iam/identity/manifest.json`, and the contract/policy files cited above.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `iam/identity/IP-016-zitadel-scale-validation-load-test.md` matched `SLO, multi-region, p99`.
- Numeric target: `rto_p99_seconds=30`, `rpo_p99_seconds=0` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), PCI-DSS-L1-v4(86400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `iam/identity/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `iam/observability/slos/identity/oidc-token-issue-latency.openslo.yaml`, `iam/observability/slos/identity/oidc-token-verify-latency.openslo.yaml`, `iam/observability/slos/identity/webauthn-authenticate-latency.openslo.yaml`, `iam/observability/slos/identity/scim-availability.openslo.yaml`, `iam/identity/policy/cedar-acr-predicates.cedar`.
