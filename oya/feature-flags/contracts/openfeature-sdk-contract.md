---
doc_class: SDKContract
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0159
  - ADR-0258
companion_docs:
  - microservices/feature-flags/sdk-plan.md
  - microservices/feature-flags/contracts/openapi-v1.yaml
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# OpenFeature SDK Contract — Feature Flags

## Overview

All oyatie feature-flags SDK implementations MUST conform to the [OpenFeature Specification](https://openfeature.dev/specification/) as **server-side providers**. The OpenFeature spec is the CNCF standard for feature-flag SDK interoperability (LaunchDarkly, Split.io, Statsig, Unleash all publish OpenFeature providers).

## Provider interface

Each SDK implements `OpenFeatureProvider`:

```typescript
interface OpenFeatureProvider {
  readonly metadata: ProviderMetadata;
  initialize(context?: EvaluationContext): Promise<void>;
  onClose(): Promise<void>;
  resolveBooleanEvaluation(flagKey: string, defaultValue: boolean, context: EvaluationContext): Promise<ResolutionDetails<boolean>>;
  resolveStringEvaluation(flagKey: string, defaultValue: string, context: EvaluationContext): Promise<ResolutionDetails<string>>;
  resolveNumberEvaluation(flagKey: string, defaultValue: number, context: EvaluationContext): Promise<ResolutionDetails<number>>;
  resolveObjectEvaluation<T>(flagKey: string, defaultValue: T, context: EvaluationContext): Promise<ResolutionDetails<T>>;
}
```

## Oyatie EvaluationContext extension

Oyatie extends the OpenFeature `EvaluationContext` with required fields:

```typescript
interface OyatieEvaluationContext extends EvaluationContext {
  tenant_id: string;           // REQUIRED: per ADR-0244 tenant scoping
  principal_id?: string;       // For rollout hashing (one-way HMAC; not stored)
  persona_tier?: PersonaTier;  // B2C | B2B | INTERNAL_AGENT | EMERGENCY_SERVICES
  cohort_ids?: string[];       // Cohort membership for targeting
  consent_purposes?: string[]; // Active consent per ADR-0272
  audience_type?: AudienceType; // Per ADR-0244 audience_type
}
```

## Resolution reasons

Oyatie returns these `ResolutionReason` values (OpenFeature standard extended):

| Reason | Meaning |
|---|---|
| `TARGETING_MATCH` | Targeting rule matched evaluation context |
| `PERCENTAGE_ROLLOUT` | Percentage rollout bucket assigned |
| `DEFAULT` | Default variant returned (no rules matched) |
| `KILL_SWITCH` | Kill-switch active; default variant returned |
| `PACK_OVERRIDE` | Pack-mandated override active |
| `DISABLED` | Flag is archived or disabled |
| `ERROR` | Evaluation error; default returned |

## Error codes

| Error code | Meaning |
|---|---|
| `FLAG_NOT_FOUND` | Flag key does not exist for tenant; default returned |
| `PARSE_ERROR` | Flag definition malformed |
| `TYPE_MISMATCH` | Requested type does not match flag type |
| `TARGETING_KEY_MISSING` | `tenant_id` not provided |
| `GENERAL` | Unexpected error; default returned |
| `PROVIDER_NOT_READY` | SDK not initialized; default returned |

## Caching contract

- Local in-process cache: 30s TTL per flag per (tenant_id, flag_key).
- Cache invalidation: SSE stream events (`flag-state-changed`, `kill-switch-activated`) invalidate immediately.
- Cache miss: synchronous gRPC fetch; ≤1ms p99 on warm server.
- Stale-while-revalidate: on cache expiry, return stale value while fetching fresh; update cache on receipt.
- LKG (last-known-good) disk cache: 30-minute TTL; used during server unavailability.

## Versioning contract

- SDK major version tracks OpenFeature spec major version.
- Server API version: `X-API-Version` response header on all evaluation responses.
- SDK ↔ server backwards compatibility: server supports 2 major SDK versions.
- Deprecation: SDK fields marked deprecated via `@deprecated` annotation + CHANGELOG entry.
