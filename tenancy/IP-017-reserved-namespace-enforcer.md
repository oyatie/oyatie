---
ip_id: IP-017
microservice: tenancy
bounded_context: reserved-namespace
layer: usecase
status: planned
related_adrs: [ADR-0242, ADR-0244, ADR-0284, ADR-0263]
---

# IP-017 — reserved-namespace enforcer

## A. Problem

Tenant creation currently has lifecycle, jurisdiction, and cell assignment plans, but no bespoke plan for refusing tenant slugs that impersonate Oyatie substrate owners, internal services, or privileged principals. `microservices/tenancy/ARCHITECTURE.md` names principals such as `oyatie.tenancy.lifecycle-controller` and `tenant.<id>.admin`; without a reserved-namespace gate, a tenant could request a human-visible slug or sub-scope alias that creates audit, support, or phishing ambiguity.

This is not a cosmetic naming check. It protects the tenancy authority boundary from homograph attacks, platform-owner-name drift, and reserved principal confusion.

## B. Approach

Implement a usecase-layer guard crate, `tenancy-reserved-namespace-usecase`, that resolves the platform-owner binding from `/specs/platform-owner-binding.json`, normalizes candidate names with Unicode confusable handling, and evaluates the action through `microservices/tenancy/policy/action-authorization.cedar` before returning allow/deny. The guard is called by tenant creation, tenant rename, sub-scope creation, and future operator portal flows.

## C. Deliverables

| Artifact | Action | Purpose |
|---|---|---|
| `microservices/tenancy/src/crates/tenancy-reserved-namespace-usecase/Cargo.toml` | create | Usecase crate. |
| `src/lib.rs` | create | Public `ReservedNamespaceEnforcer`. |
| `src/normalization.rs` | create | Unicode normalization and confusable skeleton generation. |
| `src/reserved_set.rs` | create | Builds reserved tokens from platform-owner binding plus tenancy principal roster. |
| `src/enforce.rs` | create | `enforce(candidate, actor, tenant_context)` command handler. |
| `microservices/tenancy/catalog/tenancy-reserved-namespace-usecase.yaml` | update/create | Catalog evidence already referenced by service inventory. |
| `microservices/tenancy/policy/action-authorization.cedar` | update | Add action names for reserved namespace decisions if absent. |

## D. Implementation

1. Read `/specs/platform-owner-binding.json` through a typed port so tests can substitute `oyatie` with another owner name.
2. Build reserved prefixes from the owner binding: owner name, owner-foundry, owner-internal, owner-platform-owner, and principal prefixes in `ARCHITECTURE.md`.
3. Normalize candidate names with case-folding, width folding, punctuation folding, and Unicode confusable skeletons so `oyatie`, `0yatie`, and Cyrillic-lookalike forms converge for comparison.
4. Implement `ReservedNamespaceEnforcer::enforce_create_slug` and `enforce_create_sub_scope_alias`; both require `TenantContext`, `Actor`, and `AuditCorrelationId`.
5. Emit `oya.tenancy.reserved-namespace-create-refused` with the normalized skeleton, matched reserved class, actor, tenant id, and candidate hash only.
6. Add tests for exact owner prefix, internal prefix, Cyrillic `a`, Greek `o`, full-width Latin, mixed-case, and benign strings such as `oyatier-customer`.
7. Wire the usecase into `IP-004-tenant-lifecycle-usecase.md` and `IP-016-sub-scope-registry-kernel.md` as a required pre-persistence check.

## E. Acceptance

- `cargo nextest run -p tenancy-reserved-namespace-usecase --all-features`.
- Tests include at least 20 reserved-name and confusable cases.
- Deny path emits `oya.tenancy.reserved-namespace-create-refused` and does not persist the requested slug.
- Allow path proves a non-reserved tenant slug continues to `CreateTenantUseCase`.
- `cargo run -p dev-cli -- gate validate cedar-coverage --microservice tenancy` includes the reserved namespace action.

## F. Evidence

- `microservices/tenancy/ARCHITECTURE.md` principal roster: `oyatie.tenancy.lifecycle-controller`, `oyatie.tenancy.isolation-policy-emitter`, and tenant principals.
- `microservices/tenancy/contracts/openapi/tenancy.yaml` defines `Tenant` and `CreateTenantRequest`; the REST layer will call this guard before creation.
- `microservices/tenancy/policy/action-authorization.cedar` is the existing action policy surface.
- ADR-0284 requires platform-owner-name indirection, so the implementation cannot hard-code `oyatie`.

## G. Counterparts

| Counterpart | Relevant capability | Gap this IP closes |
|---|---|---|
| GitHub | Reserved organization/user names and anti-impersonation controls | Prevents tenant slugs that look like platform or service owners. |
| Slack Enterprise Grid | Workspace URL/name reservation for enterprise domains | Protects user-facing workspace aliases below the tenant. |
| Stripe | Platform-owned account namespace separation | Keeps customer-controlled tenant names distinct from substrate principals and audit actors. |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/tenancy/IP-017-reserved-namespace-enforcer.md` matched `openapi`; contract files `microservices/tenancy/contracts/openapi/tenancy.yaml, microservices/tenancy/contracts/asyncapi/tenant-events.yaml, microservices/tenancy/contracts/proto/tenancy.proto`; type anchor `crates/tenancy-api/src/lib.rs::TenantCreateApiRequest`.
