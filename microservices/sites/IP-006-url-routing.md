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

## References

- RFC 3986 (URI generic syntax).
- `migration-from-connect.md` Hyrum surface #1.
- ADR-0105, ADR-0131.
