---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-003-page-bc-kernel
status: pending
execution_unit: ChangeSet
owner: axis-sites
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-layer-correctness, oya-governance-port-location]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: page BC — kernel + domain + usecase + api + adapter-postgres + rest + worker + sdk + app

## Intent

Author the `page` bounded-context's full crate stack including the Postgres adapter (per-tenant RLS) and the REST + worker layers. Implements `Page`, `PageVersion`, `PageDraftState`, `PageBindings`. Usecases: `create_page`, `update_page`, `publish_page`, `revert_page`, `delete_page`, `apply_legal_hold`, `release_legal_hold`, `ai_page_build` (T2 gated). URL-routing precedence; hreflang reciprocity; portable-text block-list version monotonicity.

## ChangeSet boundary

10 crates: `oya-sites-page-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}`. ~6000 LOC Rust + Postgres migration. Per-tenant RLS policy `page_tenant_isolation`. AC-01 + AC-13 + AC-15 covered by tests.

## Acceptance Gates

```bash
cargo build -p oya-sites-page-kernel -p oya-sites-page-domain -p oya-sites-page-usecase -p oya-sites-page-api
cargo build -p oya-sites-page-adapter -p oya-sites-page-adapter-postgres -p oya-sites-page-rest -p oya-sites-page-worker -p oya-sites-page-sdk -p oya-sites-page-app
cargo nextest run -p oya-sites-page-domain -- redirect_signature_stability
cargo nextest run -p oya-sites-page-usecase -- ai_page_build_refusal_hr
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice sites
```

## Test Plan

- Unit: page-version monotonicity; draft↔published transitions; revert.
- Unit: URL-routing precedence (specific > wildcard); hreflang reciprocity.
- Integration: Postgres RLS coverage; legal-hold preserves history.
- Integration: AI-page-build refusal for HR/legal/medical overlays.
- reference corpus: redirect-signature-stability (Hyrum #1 from migration guide).

## References

- ADR-0105, ADR-0106, ADR-0117, ADR-0131, ADR-0140 (retired per ADR-0145).
- ADR-SITES-0002 (rendering), ADR-SITES-0006 (AI-page-build bounds).
- PRD §"Bounded Contexts" + AC-01/AC-13/AC-15.
- RFC 3986 (URI generic syntax).
- WCAG 2.2 SC 3.1.2 (language of parts).
