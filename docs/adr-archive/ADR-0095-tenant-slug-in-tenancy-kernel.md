---
id: ADR-0095
status: Superseded
superseded_by: [ADR-702]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0095: `TenantSlug` centralized in `oya-tenancy-kernel`

> **Status:** Accepted
> **Date:** 2026-05-14
> **Owner:** `council-architecture`
> **Related:** ADR-0092, ADR-0056

## Status

Accepted (2026-05-14).

## Context

`oya-http-tenant-middleware-domain` defined the tenant-id-from-header
grammar inline:

- ASCII alphanumeric + `-` + `_`,
- 1..=128 bytes,
- non-empty.

This put a domain decision (what is a valid customer-facing tenant
identifier shape?) in middleware extraction code. Risks:

1. Defense in depth violated: anyone bypassing the middleware (a test, a
   debug route) gets to construct an unvalidated tenant id string.
2. Grammar drift: a second middleware or a CLI tool that also accepts a
   tenant id from input has to re-implement the same rule.
3. Coupling: tenant id semantics are owned by the tenancy domain, not by
   HTTP infrastructure.

`oya-tenancy-kernel` already defined a `TenantId(String)` newtype with
the INTERNAL canonical form `ten_xxx`. That grammar is intentionally
stricter than what customers see on the wire (e.g., `acme-co`) — the
internal id is a directory entry, the customer-facing slug is the lookup
key.

## Decision

Add a SECOND newtype to `oya-tenancy-kernel`:

```rust
pub const TENANT_SLUG_MAX_LEN: usize = 128;

pub struct TenantSlug(String);

impl TenantSlug {
    pub fn try_new(value: impl Into<String>) -> Result<Self, TenantKernelError> { /* … */ }
    pub fn as_str(&self) -> &str { /* … */ }
    pub fn into_inner(self) -> String { /* … */ }
}

impl TryFrom<&str> for TenantSlug { /* delegates to try_new */ }
impl FromStr for TenantSlug { /* delegates to try_new */ }
impl Display for TenantSlug { /* … */ }
```

Grammar: ASCII alphanumeric + `-` + `_`, 1..=128 bytes (matches the
previous middleware grammar exactly).

`oya-http-tenant-middleware-infrastructure` adds an `oya-tenancy-kernel`
dependency and refactors `extract_tenant` to call `TenantSlug::try_new(value)`.
The `TenantError` enum is renamed to `TenantHeaderError` and now carries
the kernel-domain `TenantKernelError` for slug validation failures:

```rust
pub enum TenantHeaderError {
    Missing,
    InvalidSlug(TenantKernelError),
}
```

Two new error variants in `TenantKernelError`:

- `TenantSlugEmpty`
- `TenantSlugTooLong { actual: usize }`
- `TenantSlugInvalidChar`

## Why two types (`TenantId` AND `TenantSlug`)?

| Type | Semantic | Example | Grammar |
|---|---|---|---|
| `TenantId` | Internal canonical id (directory entry, audit-chain key) | `ten_kr` | `ten_` prefix + alnum/_/- |
| `TenantSlug` | Customer-facing slug (HTTP header, URL path) | `acme-co` | alnum/_/- only, 1..=128 |

The boundary at the API gateway maps a `TenantSlug` from the request to a
`TenantId` via a directory lookup. The two types are intentionally
distinct so the compiler refuses a tenant-id-coming-from-the-wire from
being passed into a function expecting an internal canonical id.

`F-TENANTID-FORMAL`: formalize this mapping (lookup path, caching policy,
audit emission) in PRD-tenancy when the auth/identity slice lands.

## Adversarial fixtures (F3)

13 new tests in `oya-tenancy-kernel`, including:

- `tenant_slug_accepts_alphanumeric_dash_underscore_at_short_lengths`
- `tenant_slug_rejects_empty`
- `tenant_slug_rejects_too_long`
- `tenant_slug_accepts_max_length` (boundary)
- `tenant_slug_rejects_invalid_char_slash`
- `tenant_slug_rejects_invalid_char_space`
- `tenant_slug_rejects_invalid_char_unicode` (homoglyph defense)
- `tenant_slug_rejects_dot_path_traversal_shape` (S5-related)
- `tenant_slug_try_from_str_works`
- `tenant_slug_from_str_parse_works`
- `tenant_slug_invariants_documented_in_max_len_constant`
- `tenant_slug_is_distinct_from_tenant_id`

Plus in tenant-middleware: `middleware_grammar_matches_kernel_tenant_slug_grammar`
which exercises both kernel and middleware against an identical input set
to prove they agree (single source of truth).

## Consequences

### Positive

- Defense in depth: invalid slug values cannot exist as `TenantSlug` values
  regardless of caller context.
- Single source of truth for tenant slug grammar.
- Type-distinct internal id and customer-facing slug — compile-time
  prevention of confused-deputy errors.

### Negative

- Two newtypes where one previously existed; small API surface increase.
- Middleware now depends on `oya-tenancy-kernel`. That's a clean inward
  dependency (infrastructure → kernel) per ADR-0056.

## References

- ADR-0092 (D10 specifies this within the seam policy)
- ADR-0056 (v4.1 12-layer enum + clean architecture)
- `oya-tenancy-kernel` (TenantId predecessor, ResidencyClass, RegionBinding)
- FixupTask F-TENANTID-FORMAL
