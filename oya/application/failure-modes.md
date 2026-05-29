---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-application
deciders: ops-sre-reliability, axis-application, ops-security, council-architecture
related_adrs: [ADR-0117, ADR-0123, ADR-0131]
related_artifacts:
  - microservices/application/threat-model.md
  - microservices/application/dpia.md
  - microservices/application/incident-response.md
  - microservices/application/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting application
doc_status: published
---

# Failure-Mode Catalog (application µservice)

## Purpose

Enumerate every failure scenario on-call must handle for the Application
Shell, the detection signal, immediate mitigation, RTO, and the runbook
that owns recovery. Cross-referenced from `incident-response.md` for
severity classification.

## Failure-Mode Index

Each entry carries: FM-ID, Trigger, Detection, Tenant impact, Severity,
Immediate mitigation, RTO, Recovery runbook, Postmortem owner.

## FM-01: CDN cache poisoning (WASM bundle)

| Field | Value |
|---|---|
| Trigger | Adversary causes CDN to cache attacker-controlled bytes under legitimate URL |
| Detection | `oya_application_bundle_sri_mismatch_total > 0` for ≥1 min OR Lighthouse synthetic alarm on TTI degradation with SRI failure |
| Tenant impact | Sign-in attempts may load malicious WASM; potential exfiltration |
| Severity | Sev-1 (security breach risk) |
| Immediate mitigation | Trigger global CDN purge; serve from origin-only with mTLS until purge completes; raise OnCall page |
| RTO | ≤ 60 s for global purge; ≤ 5 min for origin-only mode |
| Recovery runbook | `runbooks/cdn-purge.md` |
| Postmortem owner | ops-security + axis-application |

## FM-02: Module-loader integrity failure (signed manifest mismatch)

| Field | Value |
|---|---|
| Trigger | Tampered manifest OR revoked publisher key OR signature drift |
| Detection | `oya_application_module_signature_invalid_total > 0` OR `oya_application_module_key_revoked_total > 0` |
| Tenant impact | Product surface fails to load; user sees retry / fallback page |
| Severity | Sev-1 (security event) |
| Immediate mitigation | Refuse load (already automatic via fail-closed); page on-call; investigate signer key origin |
| RTO | ≤ 5 min triage; ≤ 30 min decision on rollback to prior bundle version |
| Recovery runbook | `runbooks/module-rollback.md` |
| Postmortem owner | ops-security + axis-application |

## FM-03: Tenant-context loss (downstream cannot resolve tenant_id)

| Field | Value |
|---|---|
| Trigger | tenant-context middleware fails to inject claim; OpenBao outage; tenancy µservice resolver unreachable |
| Detection | `oya_application_tenant_context_resolve_fail_total > 0` for ≥30s OR downstream µservice rejection rate climbs |
| Tenant impact | All requests fail closed; users see service-unavailable banner |
| Severity | Sev-1 (availability breach) |
| Immediate mitigation | Fail closed (already automatic); engage tenancy on-call; check OpenBao |
| RTO | ≤ 5 min |
| Recovery runbook | `runbooks/tenant-context-recovery.md` |
| Postmortem owner | axis-application + tenancy axis |

## FM-04: WASM bundle corruption (build artifact corrupt)

| Field | Value |
|---|---|
| Trigger | Build pipeline produces broken WASM (LLVM bug, dependency mismatch, hash mismatch on PR merge) |
| Detection | Synthetic Lighthouse + canary cohort 1% fails on instantiate; `oya_application_wasm_instantiate_fail_total > 0` |
| Tenant impact | Newly-promoted bundle won't run; users see retry then fallback to prior version (handled by frontend-bundle-serve worker) |
| Severity | Sev-2 (functional regression) |
| Immediate mitigation | Pin CDN to prior bundle version (revert pointer in frontend-bundle-serve); auto-rollback if SLO breaches |
| RTO | ≤ 60 s for pointer revert |
| Recovery runbook | `runbooks/wasm-bundle-rebuild.md` |
| Postmortem owner | axis-application |

## FM-05: Auth-gateway IdP outage (OIDC IdP unreachable)

| Field | Value |
|---|---|
| Trigger | OIDC IdP (Okta / Azure AD) returns 5xx or timeout |
| Detection | `oya_application_oidc_idp_error_rate > 5%` for ≥1 min |
| Tenant impact | New sign-ins fail; existing sessions unaffected |
| Severity | Sev-2 (partial availability) |
| Immediate mitigation | Engage IdP vendor; if SAML fallback configured for tenant, redirect; status-page update |
| RTO | dependent on IdP; oyatie SLO: 10 min before exec escalation |
| Recovery runbook | `runbooks/auth-gateway-restart.md` §"IdP outage fallback" |
| Postmortem owner | ops-security + axis-application |

## FM-06: Session storm (token-store memory exhaustion)

| Field | Value |
|---|---|
| Trigger | Credential-stuffing wave; viral sign-in surge; cookie-replay attack |
| Detection | Valkey memory > 80 % for ≥3 min OR `oya_application_session_create_rate` spikes 10× baseline |
| Tenant impact | New sessions may fail to insert; existing sessions evicted prematurely |
| Severity | Sev-2 |
| Immediate mitigation | Raise per-IP rate limit; engage WAF challenge; ensure auto-scale of session store |
| RTO | ≤ 10 min |
| Recovery runbook | `runbooks/session-storm.md` |
| Postmortem owner | ops-security + axis-application |

## FM-07: Cross-tenant route confusion (Cedar bypass attempt)

| Field | Value |
|---|---|
| Trigger | URL fuzzing / Cedar policy regression / IDOR attempt |
| Detection | `oya_application_route_denied_total{reason="cross-tenant"} > N` per minute |
| Tenant impact | Potential information disclosure if Cedar regresses |
| Severity | Sev-1 (security event if breach confirmed) |
| Immediate mitigation | Verify default-deny posture; if breach: revoke offending tokens; CDN-purge; audit-export |
| RTO | ≤ 15 min |
| Recovery runbook | `runbooks/cdn-purge.md` (asset purge) + audit |
| Postmortem owner | ops-security + axis-application |

## FM-08: Postgres + Citus shard imbalance

| Field | Value |
|---|---|
| Trigger | One tenant dominates shard; hot-tenant pattern; missing rebalance |
| Detection | per-shard QPS / size variance > 3× baseline |
| Tenant impact | Tail latency spike for tenants on hot shard |
| Severity | Sev-3 (degraded but functional) |
| Immediate mitigation | Citus rebalance; consider tenant-specific shard pin |
| RTO | ≤ 30 min |
| Recovery runbook | (referenced from tenancy runbook) |
| Postmortem owner | axis-application + ops-sre-reliability |

## FM-09: Audit chain seal latency breach

| Field | Value |
|---|---|
| Trigger | Audit-chain µservice degraded; Ed25519 seal queue backlog |
| Detection | `oya_application_audit_seal_latency_seconds{quantile="0.99"} > 5` for ≥5 min |
| Tenant impact | Audit records not yet sealed but emitted; eventual consistency degrade |
| Severity | Sev-2 (compliance impact) |
| Immediate mitigation | Engage audit-chain on-call; scale seal workers; queue persisted |
| RTO | ≤ 15 min |
| Recovery runbook | (audit-chain owned runbook; cross-link) |
| Postmortem owner | axis-application + audit-chain owner |

## FM-10: Leptos hydration regression (frontend version mismatch)

| Field | Value |
|---|---|
| Trigger | Build/deploy pushes a bundle whose hydration mismatches SSR HTML |
| Detection | `oya_application_hydration_error_rate > 1%` for ≥1 min; user-facing white-flash |
| Tenant impact | TTI breach for affected users |
| Severity | Sev-2 |
| Immediate mitigation | Revert bundle pointer (frontend-bundle-serve); auto-rollback triggers if SLO breaches |
| RTO | ≤ 60 s |
| Recovery runbook | `runbooks/wasm-bundle-rebuild.md` |
| Postmortem owner | axis-application |

## FM-11: CDN purge backlog (purge job queue depth grows)

| Field | Value |
|---|---|
| Trigger | CDN vendor rate-limit; purge worker outage |
| Detection | `oya_application_cdn_purge_queue_depth > 100` for ≥3 min |
| Tenant impact | Stale assets persist for some users (typically up to TTL) |
| Severity | Sev-3 |
| Immediate mitigation | Scale purge workers; engage CDN vendor |
| RTO | ≤ 30 min |
| Recovery runbook | `runbooks/cdn-purge.md` |
| Postmortem owner | axis-application |

## FM-12: Cookie scope misconfiguration (subdomain leak)

| Field | Value |
|---|---|
| Trigger | Helm value drift sets cookie `Domain=.oyatie.dev` instead of `.app.oyatie.dev` |
| Detection | `oya-application-cookie-scope-lint` lane on PR + runtime probe |
| Tenant impact | Session cookie reachable from sibling subdomain |
| Severity | Sev-1 (potential cross-product cookie leak) |
| Immediate mitigation | Force re-sign-in; CDN-purge; rotate session HMAC key |
| RTO | ≤ 15 min |
| Recovery runbook | `runbooks/auth-gateway-restart.md` §"Cookie scope rotate" |
| Postmortem owner | ops-security + axis-application |

## FM-13: Pack-residency misroute (tenant served from wrong pack)

| Field | Value |
|---|---|
| Trigger | DNS misconfiguration; cloud-iac drift |
| Detection | `oya_application_residency_pin_violation_total > 0` |
| Tenant impact | Tenant data may transit to non-residency region |
| Severity | Sev-1 (regulatory breach) |
| Immediate mitigation | Fail closed; redirect to correct pack via signed redirect; tenant-comm |
| RTO | ≤ 15 min |
| Recovery runbook | (cloud-iac residency runbook; cross-link) |
| Postmortem owner | council-privacy + axis-application |

## FM-14: SAML XSW attack attempted (signature wrap)

| Field | Value |
|---|---|
| Trigger | Malicious SAML response with wrapped signature |
| Detection | `oya_application_saml_xsw_block_total > 0` (lane catches XSW-1..8) |
| Tenant impact | Attempt blocked; potential targeted attack |
| Severity | Sev-2 (security event) |
| Immediate mitigation | Block source IP; alert tenant security |
| RTO | n/a (already blocked) |
| Recovery runbook | (auth-gateway runbook) |
| Postmortem owner | ops-security |

## FM-15: Cedar policy compile regression

| Field | Value |
|---|---|
| Trigger | Cedar fragment syntax error reaches deploy |
| Detection | `oya-application-cedar-policy-compiles` lane (PR-time); runtime: handler fails to start |
| Tenant impact | Application Shell fails closed; service-unavailable |
| Severity | Sev-1 (availability) |
| Immediate mitigation | Roll back deploy; revert PR |
| RTO | ≤ 5 min |
| Recovery runbook | `runbooks/auth-gateway-restart.md` §"Policy revert" |
| Postmortem owner | axis-application |

## FM-16: Lighthouse / TTI synthetic alarm sustained

| Field | Value |
|---|---|
| Trigger | TTI p99 > 2 s for ≥5 min (multi-window burn-rate) |
| Detection | observability burn-rate fast-burn (1h window) |
| Tenant impact | Slow-loading shell affects employee productivity |
| Severity | Sev-2 |
| Immediate mitigation | Auto-rollback to prior bundle if SLO threshold breached; investigate bundle size + CDN hit ratio |
| RTO | ≤ 60 s (auto-rollback); ≤ 30 min triage |
| Recovery runbook | `runbooks/wasm-bundle-rebuild.md` |
| Postmortem owner | axis-application |

## Cross-reference: incident severity classes

Severity definitions live in `incident-response.md`. Brief mapping:

| Severity | Examples (FM-IDs) |
|---|---|
| Sev-1 | FM-01, FM-02, FM-03, FM-07, FM-12, FM-13, FM-15 |
| Sev-2 | FM-04, FM-05, FM-06, FM-09, FM-10, FM-14, FM-16 |
| Sev-3 | FM-08, FM-11 |
