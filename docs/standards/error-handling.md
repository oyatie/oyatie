# Oyatie — Error Handling Standard

> **Owner:** `council-architecture`.

## 1. Error type policy

- Per-crate error enum derived from `thiserror`
- Never panic at API / capability / runtime boundaries
- Always `Result<T, E>` for fallible operations
- `?` propagation as default; `match` only when arm-specific recovery
- Public API adapters expose typed error-envelope DTOs and map closed error variants into stable public error codes.

## 2. Error taxonomy (closed enumeration per crate)

```
GENERIC_*       — unrecoverable / unexpected
TENANT_*        — tenant-scoped
CAPABILITY_*    — Foundry capability
PRIVACY_*       — Data Use Boundary violation
COMPLIANCE_*    — regulatory denial
CONTRACT_*      — cross-axis contract violation
AUTH_*          — authentication / authorization
RATE_LIMIT      — rate limit hit
QUOTA_*         — budget / quota exhaustion
NETWORK_*       — transient network
DATA_*          — data integrity / consistency
INTERNAL        — bug / unexpected state
```

## 3. Retryable vs terminal

Every error variant declares `is_retryable()`:

- Terminal: AUTH_*, PRIVACY_*, COMPLIANCE_*, CONTRACT_*, DATA_*-most, INTERNAL
- Retryable: NETWORK_*, RATE_LIMIT (with backoff), QUOTA_* (after window), some TENANT_* (after onboarding)

## 4. Public-API error shape

Per [api-design.md §5](api-design.md):

```jsonc
{
  "error": {
    "code": "PRIVACY_DATA_CLASS_VIOLATION",
    "message": "Cannot send PHI-class field to ad-targeting purpose",
    "message_localized": "...",
    "request_id": "uuid",
    "details": [ { "field": "patient.diagnosis", "issue": "data_class=PHI; purpose=ad_targeting" } ],
    "retry_after_seconds": null
  }
}
```

## 5. Audit-chain emission for errors

Per ADR-0003 + ADR-0008:
- Privacy / Compliance / Contract violations emit `EVT-VIOLATION` + reason
- Foundry capability errors emit per-capability evidence
- Cross-axis contract errors emit cohesion-fitness drift signal

## 6. Anti-patterns

- Generic `Box<dyn Error>` in public API — never (must be typed per crate)
- Stringly-typed error codes — use enum
- OpenAPI runtime-bound error responses without concrete schema-bound DTOs — never
- Swallowing errors silently — never; log + audit-emit + propagate
- Re-throwing without context — wrap with `.context(...)` or per-error wrap

## 7. Sources
`thiserror`, `anyhow` (CLI tools only), Rust Error Handling Project Group, ADR-0003.
