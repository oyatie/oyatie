---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-007-domain-binding-acme
status: pending
execution_unit: ChangeSet
owner: axis-sites + ops-security
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-layer-correctness, oya-governance-rfc-8555-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: domain-binding BC + ACME + cert-manager adapters

## Intent

Author the `domain-binding` BC per ADR-SITES-0004. Implements `Domain`, `DnsVerification`, `Certificate`, `AcmeChallenge`. ACME RFC 8555 DNS-01 client (preferred for wildcard); HTTP-01 fallback for tenant-controlled root domains. Multi-account pool to manage Let's Encrypt 50-cert/wk rate limit. cert-manager CRD adapter for cert lifecycle.

## ChangeSet boundary

9 crates: `oya-sites-domain-binding-{kernel,domain,usecase,api,adapter,adapter-acme,adapter-cert-manager,rest,worker,app}`. AC-03 covered.

## Acceptance Gates

```bash
cargo nextest run -p oya-sites-domain-binding-adapter-acme -- dns01_wildcard
cargo nextest run -p oya-sites-domain-binding-adapter-acme -- http01_root
cargo nextest run -p oya-sites-domain-binding-adapter-acme -- rate_limit_account_rotate
buck2 build //:quality-lane-registry-authority-check # lane=rfc-8555-conformance --microservice sites
```

## Test Plan

- Unit: ACME directory discovery; challenge round-trip.
- Integration: DNS-01 challenge against Let's Encrypt staging.
- Integration: wildcard cert issuance for `*.test.oyatie.dev`.
- Integration: account-pool rotation under rate-limit.
- Integration: cert-manager CRD reconcile loop.
- E2E: subdomain takeover refusal (Domain.tenant_id binding).

## References

- ADR-SITES-0004.
- RFC 8555 — ACME.
- RFC 8737 — TLS-ALPN-01.
- cert-manager — `cert-manager.io/docs`.
- Let's Encrypt — `letsencrypt.org/docs/rate-limits/`.
