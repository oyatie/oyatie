---
runbook: connector-onboarding
microservice: connector
owner_team: axis-integration + product
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0249, ADR-0297]
doc_status: published
---

# Runbook — Connector Onboarding

## A. Trigger

- New vendor connector requested (top-N demand) OR MPO publishes new connector

## B. Pre-checks

1. Vendor's OAuth specification documented (auth-code / client-credentials / JWT-bearer)
2. Vendor's webhook scheme documented (HMAC alg, header names, replay window)
3. Vendor's rate-limit profile documented
4. Compliance posture: which packs is this connector eligible for?

## C. Procedure

1. **Catalog entry** — author `catalog/connectors/<name>.yaml` per `connector-catalog-schema.json`.
2. **Adapter implementation** — Rust crate at `src/crates/connector-adapters/<name>/` implementing the canonical adapter trait.
3. **Tests** — unit + property + real-vendor-sandbox integration (gated by Cedar `ci-scope.cedar`).
4. **Cosign sign** — adapter binary signed via sigstore keyless OIDC.
5. **Catalog publish** — submit to marketplace ingest per `marketplace/PRD.md`.
6. **Security review** — automated CI lane + manual review for high-risk (PII / payments / auth providers).
7. **Soft launch** — staging cell; 7d soak; sample tenants invited.
8. **GA** — listed in production catalog.

## D. Verification

- Adapter passes `governance-connector-conformance` lane (action shape, error mapping, observability emission).
- Real-vendor-sandbox integration tests pass.
- Catalog entry appears in tenant-facing search.

## E. Rollback

Mark connector as `status: yanked` in catalog; existing wirings auto-pause with 90d sunset notice.

## F. Post-incident

- Track adoption metrics.
- Schema-drift monitoring starts at GA.

## G. References

- ADR-0249 multi-category marketplace
- `microservices/marketplace/PRD.md`
- documentation-rigor.md §1.1 (named precedent + failure-mode tree required)
