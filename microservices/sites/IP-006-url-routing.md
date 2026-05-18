---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-006-url-routing
status: pending
execution_unit: ChangeSet
owner: axis-sites
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-layer-correctness]
---

# IP-006: url-routing BC

## Intent

Author the `url-routing` BC. Implements `Route`, `Redirect`, `RouteMatch` with RFC 3986 percent-encoding preservation. Redirect statuses 301/302/410. Route-precedence rules (specific > wildcard). Redirect-loop detection (refuses chains > 5).

## ChangeSet boundary

8 crates: `oya-sites-url-routing-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,app}`. Hyrum's-Law-bound URL signature stability test corpus per migration guide.

## Acceptance Gates

```bash
cargo nextest run -p oya-sites-url-routing-domain -- redirect_signature_stability
cargo nextest run -p oya-sites-url-routing-domain -- redirect_loop_refused
cargo nextest run -p oya-sites-url-routing-domain -- percent_encoding_preserved
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice sites
```

## ChangeSet metadata

```yaml
changeset_id: CS-SITES-IP-006-url-routing
depends_on_changesets: [CS-SITES-IP-003-site-and-page-bcs]
parallel_safe_with_changesets: [CS-SITES-IP-007-block-and-theme, CS-SITES-IP-010-search-meilisearch]
enables: [CS-SITES-IP-011-cdn-delivery]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Percent-encoding preserved across redirect chain (RFC 3986 §2.1) | `cargo nextest run -p oya-sites-url-routing-domain -- percent_encoding_preserved` |
| AC-02 | Redirect loop > 5 hops refused with `508 Loop Detected` (RFC 5842 §7.2 analogue) | `cargo nextest run -p oya-sites-url-routing-domain -- redirect_loop_refused` |
| AC-03 | Route-precedence rule: specific > wildcard; deterministic on collision | `cargo nextest run -p oya-sites-url-routing-domain -- precedence_specific_over_wildcard` |
| AC-04 | URL signature stability fixture corpus unchanged across releases | `cargo nextest run -p oya-sites-url-routing-domain -- redirect_signature_stability` |
| AC-05 | `oya gate validate layer-correctness --microservice sites` exits 0 | ADR-0105 / ADR-0131 |

## Build Sequence

1. Kernel: `RouteRepository`, `RedirectResolver`, `RoutePrecedence` ports.
2. Domain: `Route`, `Redirect`, `RouteMatch`; status-code enum (301/302/308/410).
3. Usecase: `ResolveRoute`, `MintRedirect`, `RetireRoute`.
4. Postgres adapter; REST handler.
5. `cargo nextest run -p oya-sites-url-routing-*`.
6. `cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice sites`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-sites FR | FR-07 (publish to CDN), FR-08 (301/302/410) |
| PRD-sites NFR | NFR perf — page-render p95 ≤ 200ms cached |
| PRD-sites AC | AC-02 (URL routing correctness) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Hyrum's-Law URL signature drift on migration | Stability corpus + `redirect_signature_stability` test |
| Redirect chain blows up traversal time | Cap chain at 5; refuse beyond |
| Trailing-slash policy ambiguity | Canonical form codified; refuse mixed-mode routes |

## References

- RFC 3986 (URI Generic Syntax).
- RFC 7538 (308 Permanent Redirect).
- `migration-from-connect.md` Hyrum surface #1.
- Next.js routing reference (`nextjs.org/docs/pages/building-your-application/routing`).
- Cloudflare Pages redirect rules (Cloudflare Docs).
- ADR-0105, ADR-0131.
