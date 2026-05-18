---
id: ADR-SITES-0004
status: Accepted
date: 2026-05-17
microservice: sites
deciders: axis-sites, council-architecture, ops-security
owner: axis-sites + ops-security
supersedes: []
superseded_by: []
related:
  - ADR-0117
  - ADR-0131
  - ADR-0133
  - ADR-SITES-0003
related_artifacts:
  - microservices/sites/PRD.md §FR-06, AC-03
  - microservices/sites/iac/helm/values.yaml (domainBinding section)
  - microservices/sites/runbooks/acme-cert-renewal-failure.md
  - microservices/sites/runbooks/custom-domain-dns-drift.md
  - microservices/sites/threat-model.md (subdomain-takeover, spoofing)
purpose: |
  Define the custom-domain flow: DNS verification, ACME RFC 8555 cert
  issuance + renewal, and cert distribution to CDN edges. Address
  Let's Encrypt rate limits and subdomain-takeover risks.
---

# ADR-SITES-0004: ACME RFC 8555 + custom-domain flow — Let's Encrypt DNS-01 primary; HTTP-01 fallback; multi-account pool; cert-manager 1.16 reconciliation

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Custom-domain binding is a Must-have FR per PRD-sites §FR-06. The flow:
1. Tenant adds a custom domain (apex, subdomain, or wildcard).
2. Tenant adds DNS records pointing the domain to our CDN.
3. Sites verifies DNS resolution + control via signed challenge.
4. Sites issues an ACME-issued TLS cert.
5. Cert is loaded to CDN edges for TLS termination.
6. Cert auto-renews 30-day-pre-expiry.

ACME (RFC 8555) is the industry-standard automation protocol;
Let's Encrypt is the dominant free issuer. The three challenge types:
- **HTTP-01**: tenant places file at `/.well-known/acme-challenge/<token>`. Works only for root domain (not wildcards).
- **DNS-01**: tenant adds TXT record at `_acme-challenge.<domain>`. Works for apex + subdomain + wildcard.
- **TLS-ALPN-01**: TLS handshake-based; requires control of port 443 of the domain.

Let's Encrypt rate limits (2026-05):
- 50 certs per registered domain per week.
- 5 duplicate certs per week.
- 300 new orders per account per 3h.

At scale (50k tenants binding domains in the launch window), we WILL
hit per-account rate limits if we use a single ACME account.

Per `threat-model.md` STRIDE matrix, subdomain takeover is a Sev-1
risk: if a tenant binds `customer.example`, points DNS to our CDN,
later changes DNS away (without unbinding), an attacker could
re-claim the DNS pointer and we'd still serve the tenant's cert. The
ACME cert must auto-revoke on unbind to mitigate.

## Decision

The sites µservice ships an **ACME RFC 8555 client with DNS-01
primary, HTTP-01 fallback, multi-account pool, and cert-manager 1.16
LTS reconciliation**:

- **Adapter**: `oya-sites-domain-binding-adapter-acme` (backend-
  qualified per ADR-0105 Amendment 3).
- **Issuer**: Let's Encrypt (production directory
  `acme-v02.api.letsencrypt.org`). Staging directory used in CI
  per `policy/ci-scope.cedar`.
- **Challenge type**: DNS-01 preferred (supports wildcard; less edge
  surface). HTTP-01 alternative for tenant-controlled root domains
  without DNS API access. TLS-ALPN-01 not currently used.
- **Multi-account pool**: per-pack ACME account pool with size 3
  baseline (scale to 10 at 50k tenant launch); rotation triggered at
  > 30 certs/wk per account.
- **cert-manager 1.16 LTS reconciliation**: cert-manager CRDs
  (`Certificate`, `Issuer`, `ClusterIssuer`) reconcile cert
  lifecycle. The `oya-sites-domain-binding-adapter-cert-manager`
  bridges the kernel `CertificateStore` port to cert-manager.
- **Cert distribution**: issued cert → CDN edge load via signed-
  upload contract (per ADR-SITES-0003).
- **Auto-renewal**: 30-day-pre-expiry; CronJob (`acme-renewal-worker`)
  every 6h scans expiring certs.
- **Auto-revocation on unbind**: when a domain is unbound, the cert
  is revoked at Let's Encrypt before the binding record is deleted.
  Prevents subdomain takeover.
- **DNS-drift watchdog**: scheduled job verifies DNS state for every
  bound domain weekly; drift → runbook `custom-domain-dns-drift.md`.

## Alternatives Considered

### A. ZeroSSL or Buypass instead of Let's Encrypt

- **Pros**:
  - Different rate-limit envelopes (ZeroSSL: 90 days, similar; Buypass: 180 days but lower volume).
- **Cons**:
  - Less battle-tested at scale.
  - ZeroSSL has a commercial tier we'd want to avoid.
- **Rejected** as primary; could be added as a second issuer if Let's Encrypt cools.

### B. Internal CA (Step CA / Smallstep)

- **Pros**:
  - Substrate-portability; no rate limits.
  - Full control.
- **Cons**:
  - Internal CA not trusted by browsers; would require tenant manual
    trust install — unacceptable UX.
  - Only useful for internal mTLS, not public TLS.
- **Rejected** for public-facing certs; used internally for mesh mTLS
  per cloud-secrets µservice.

### C. CDN-provider-issued certs (e.g., Cloudflare Universal SSL)

- **Pros**:
  - Auto-managed at CDN; we don't run ACME.
- **Cons**:
  - Tied to CDN provider; violates substrate-portability per
    ADR-SITES-0003.
  - Per-tenant cert provisioning depends on CDN provider's quota.
- **Rejected** in favor of own ACME flow.

### D. Single ACME account

- **Pros**:
  - Simplest ops.
- **Cons**:
  - Hits 50-cert/wk rate limit at ~50 tenants/wk; blocks onboarding.
- **Rejected** — multi-account pool needed.

### E. DNS-01 primary + HTTP-01 fallback + multi-account pool + cert-manager  ← **CHOSEN**

- **Pros**:
  - DNS-01 supports wildcard (per-tenant `*.acme.tenant.example`).
  - Multi-account pool handles rate limit.
  - cert-manager 1.16 LTS is the industry-standard reconciliation
    pattern; well-understood ops.
  - Auto-revoke on unbind prevents subdomain takeover.
- **Cons**:
  - DNS-01 requires tenant DNS API access OR manual TXT record
    update — friction in tenant onboarding.
  - Multi-account pool adds operational surface.
- **Accepted**.

## Consequences

### Positive

- **Wildcard cert support** (`*.tenant.example`) via DNS-01.
- **Rate-limit-resilient** via multi-account pool.
- **Substrate-portable**: we don't depend on CDN-provider-issued certs.
- **Subdomain-takeover-resistant**: auto-revoke on unbind.
- **Industry-standard reconciliation**: cert-manager is the
  Kubernetes standard.

### Negative

- **DNS-01 tenant friction**: tenants must have DNS API access OR
  manually update TXT records. Mitigated by clear docs + DNS provider
  compatibility matrix at `microservices/sites/specs/dns-provider-compatibility.json`.
- **Multi-account pool ops surface**: 3+ ACME accounts to manage per
  pack. Mitigated by automated account rotation worker.
- **Cert revocation logic must be correct**: revoke-before-delete
  contract on unbind.

### Operational

- **CronJob `acme-renewal-worker`**: every 6h scans expiring certs.
- **Watchdog `dns-drift-watchdog`**: weekly DNS-state verify per
  bound domain.
- **2-person rule for cert revocation** (Cedar policy).
- **Runbooks**: `acme-cert-renewal-failure.md`,
  `custom-domain-dns-drift.md`.
- **PrometheusRule alerts**: cert expiry < 24h (page); < 7d (ticket).

### Regulatory

- **GDPR Art. 32**: TLS 1.3 in transit; cert auto-renewal as
  appropriate technical measure.
- **HIPAA 45 CFR §164.312(e)**: transmission security via TLS satisfied.
- **eIDAS**: ACME-issued certs are domain-validated; for tenants
  needing extended-validation, manual cert upload remains possible
  per a successor-IP ADR-SITES-XXXX.

## Verification

- [ ] **ACME DNS-01 wildcard** —
  `cargo nextest run -p oya-sites-domain-binding-adapter-acme -- dns01_wildcard`.
- [ ] **ACME HTTP-01 root** —
  `cargo nextest run -p oya-sites-domain-binding-adapter-acme -- http01_root`.
- [ ] **Multi-account rotation** —
  `cargo nextest run -p oya-sites-domain-binding-adapter-acme -- rate_limit_account_rotate`.
- [ ] **Auto-revoke on unbind** —
  `cargo nextest run -p oya-sites-domain-binding-usecase -- revoke_on_unbind`.
- [ ] **DNS-drift watchdog** —
  `cargo nextest run -p oya-sites-domain-binding-worker -- dns_drift_detect`.

## References

- RFC 8555 — ACME.
- RFC 8737 — TLS-ALPN-01.
- Let's Encrypt — `letsencrypt.org/docs/`.
- Let's Encrypt rate limits — `letsencrypt.org/docs/rate-limits/`.
- cert-manager — `cert-manager.io/docs`.
- ADR-0117, ADR-0131, ADR-0133, ADR-SITES-0003.
- `microservices/sites/PRD.md` §FR-06, AC-03.
- `microservices/sites/threat-model.md` STRIDE Spoofing matrix.
- `microservices/sites/runbooks/acme-cert-renewal-failure.md`.
- `microservices/sites/runbooks/custom-domain-dns-drift.md`.
- OWASP Subdomain Takeover guidance.
