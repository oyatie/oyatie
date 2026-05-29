# IP-013 — OpenFeature SDK Provider Contract

**microservice**: feature-flags
**bc**: flag
**layer**: adapter
**qualifier**: openfeature
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0245, ADR-0248, ADR-0253, ADR-0258
**companion_ips**: IP-014, IP-015, IP-016
**references**: OpenFeature specification v0.8.0 (CNCF); contracts/openfeature-sdk-contract.md

## Scope

Canonical OpenFeature provider contract shared by all language SDKs. Defines the provider interface, resolution reasons, error codes, caching contract (30s TTL + SSE invalidation + 30-min LKG disk cache), and versioning policy.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | Provider interface | `initialize()`, `shutdown()`, `resolve_boolean_value()`, `resolve_string_value()`, `resolve_number_value()`, `resolve_object_value()`; matches OpenFeature spec §2.1 |
| 2 | `OyatieEvaluationContext` | Extension fields: `tenant_id`, `audience_type`, `session_id`, `device_fingerprint_hash`, `pack_id`; carries through to server |
| 3 | Resolution reasons | `STATIC`, `DEFAULT`, `TARGETING_MATCH`, `SPLIT`, `CACHED`, `UNKNOWN`, `DISABLED`, `ERROR` |
| 4 | Error codes | `PROVIDER_NOT_READY`, `FLAG_NOT_FOUND`, `PARSE_ERROR`, `TYPE_MISMATCH`, `GENERAL` |
| 5 | Caching contract | Hit: return from DashMap; Miss: fetch API; TTL 30s; SSE stream invalidates immediately; LKG disk fallback 30min |
| 6 | Stale-while-revalidate | Return stale value + trigger background refresh when TTL expired but SSE not yet delivered |
| 7 | Versioning | Provider version `1.x` tracks openapi-v1.yaml SemVer; breaking changes require major bump + sunset per ADR-0258 |
| 8 | Conformance tests | OpenFeature spec conformance set passes for all 4 resolution types |

## Caching Architecture

```
Client request
     │
     ▼
DashMap<(tenant_id, flag_key), CachedEntry>
     │ hit (age < 30s)         │ miss or stale
     ▼                         ▼
Return cached value       HTTP/3 + gRPC evaluate()
     │                         │
     │          SSE stream ◄───┘
     │          (invalidates on FlagStateChanged)
     ▼
LKG disk cache (30min, survives process restart)
```

## Definition of Done

- OpenFeature spec conformance set green
- Stale-while-revalidate: stale value returned within 1ms; refresh completes within 2s
- SSE invalidation: flag change propagates to client within 5s (SLO: flag-state-propagation)
- All 4 language SDKs (IP-014, IP-015, IP-016, + Go in Phase 2) implement this contract
