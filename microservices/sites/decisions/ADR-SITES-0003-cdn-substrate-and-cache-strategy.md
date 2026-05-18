---
id: ADR-SITES-0003
status: Accepted
date: 2026-05-17
microservice: sites
deciders: axis-sites, council-architecture, ops-sre-reliability, ops-finops
owner: axis-sites + ops-sre-reliability
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0117
  - ADR-0131
  - ADR-0133
  - ADR-SITES-0002
related_artifacts:
  - microservices/sites/PRD.md §"Performance"
  - microservices/sites/iac/helm/values.yaml (cdnDelivery section)
  - microservices/sites/runbooks/cdn-cache-purge-cascade.md
  - microservices/sites/migration-from-connect.md Hyrum #5
purpose: |
  Choose the CDN substrate for serving published sites and define the
  cache-key + purge model.
---

# ADR-SITES-0003: CDN substrate + cache strategy — Cloudflare-class primary + self-managed Varnish/Caddy alternative; cache-key includes version-hash

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Per ADR-SITES-0002, sites uses SSG/ISR hybrid rendering. The CDN edge
is the primary delivery surface. Choice of CDN substrate affects:
- Per-pack residency (CDN edges must respect data-residency: pack-eu
  visitors served from EU edges only).
- Pricing (bandwidth + invalidation cost).
- Purge propagation speed (cache invalidation p95 ≤ 2s target).
- Vendor lock-in posture (per `feedback_autonomous_decision_principles.md`
  — substrate-portability preferred).
- Custom-domain TLS coupling (CDN may also terminate TLS for the
  tenant's custom domain).

Three substrate categories:
1. **Cloudflare-class hosted CDN** (Cloudflare, Fastly, Akamai, AWS
   CloudFront): managed; high cache hit ratio; rich edge logic;
   vendor-bound.
2. **Self-managed Varnish/Caddy/Apache Traffic Server**: own edge
   nodes per-pack; substrate-portable; ops-heavy.
3. **Bunny.net / KeyCDN tier**: cheap; less-feature-rich; managed.

Per ADR-0117 data-residency, edges MUST be per-pack-geofenced for
non-public content. Per `feedback_autonomous_decision_principles.md`,
"long-term right > short-term cost" → substrate-portability matters
more than absolute lowest cost.

The legacy `oya-connect-sites-*` used a tenant-id-only cache-key,
which caused version-blind cache serving (Hyrum #5 surface in migration
guide). The new µservice MUST fix this.

## Decision

The sites µservice ships with **a CDN substrate abstraction layer**
that primarily integrates with Cloudflare-class (for pack-kr/eu/us/jp/
sg/au/in/br/ae/ksa), with a **self-managed Varnish/Caddy alternative**
for pack-us-healthcare (where HIPAA BAA-bound edge providers are
constrained). The abstraction is `CacheInvalidator` port trait
implemented by `oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub`;
adapter-qualified backends per ADR-0105 Amendment 3.

Concrete bindings:
- **Primary CDN per pack**: Cloudflare (or per-pack contract); pack-eu
  uses EU edges only; pack-kr uses KR edges only; etc.
- **Alternative for pack-us-healthcare**: self-managed Varnish edge
  nodes in BAA-eligible OCI regions (or BAA-on-file Cloudflare account
  if available).
- **Cache-key format**: `tenant-id|site-id|version-hash|route-path`.
  This fixes the legacy version-blind issue (Hyrum #5).
- **Cache TTL**: 24h with `stale-while-revalidate=86400` (per RFC 5861).
- **Purge model**: signed Ed25519 purge event from publish-pipeline →
  CDN provider API; per-pack edge purge propagation p95 ≤ 2s.
- **TLS termination**: at the CDN edge for custom-domains (per
  ADR-SITES-0004 ACME certs loaded to CDN); origin-protect via mTLS.
- **Origin-shield**: CDN edge fetches origin via a single
  shielded path (CDN → shield → origin), to reduce origin RPS.

## Alternatives Considered

### A. Cloudflare-only (single-provider)

- **Pros**:
  - Largest global edge footprint (300+ POPs).
  - Fastest purge (~250ms global).
  - Built-in WAF + DDoS protection.
- **Cons**:
  - Single-provider lock-in violates substrate-portability.
  - pack-us-healthcare BAA availability historically limited.
  - Compliance with pack-cn (future) constraints unclear.
- **Rejected** as the sole choice; selected as primary for most packs.

### B. Self-managed Varnish/Caddy only

- **Pros**:
  - Full substrate-portability.
  - No third-party pricing surprises.
  - All edges directly observable.
- **Cons**:
  - Edge POP count limited (we'd need 30+ edges globally; ops cost
    very high).
  - DDoS protection harder; we'd need separate scrubbing layer.
  - p95 latency higher (fewer edges = farther from visitor).
- **Rejected** as the primary; selected as alternative for
  pack-us-healthcare and as the substrate-portability escape hatch.

### C. AWS CloudFront

- **Pros**:
  - Integrated with AWS S3 origins.
  - WAF + ACM integration.
- **Cons**:
  - We're on OCI per ADR-0117 (not AWS); cross-cloud CDN choice adds
    network egress cost.
  - Same single-provider lock-in concerns.
- **Rejected** because of cloud-architecture mismatch.

### D. Multi-CDN with smart routing (Akamai + Cloudflare with active-active)

- **Pros**:
  - No single-provider failure.
- **Cons**:
  - Cost ~2x; ops complexity 3x.
  - Cache coherence across two CDNs is hard.
- **Rejected** until scale justifies (post-Q2 2027 projection).

### E. Cloudflare-class primary + self-managed Varnish/Caddy alternative  ← **CHOSEN**

- **Pros**:
  - Best of both: managed performance for most packs; substrate-
    portability via the alternative.
  - Per-pack flexibility (BAA-eligible alternative for healthcare).
  - Adapter-qualified per ADR-0105 Amendment 3 — clean
    `CacheInvalidator` port; hot-swap supported.
- **Cons**:
  - Two operational surfaces.
  - Per-pack contracts to negotiate.
- **Accepted** — operational complexity bounded; substrate-portability
  preserved.

## Consequences

### Positive

- **Substrate-portability** preserved via the `CacheInvalidator` port
  + alternative adapter.
- **Cache-key version-hash** fixes the legacy version-blind bug
  (Hyrum #5).
- **Per-pack residency** preserved: pack-eu serves from EU edges; etc.
- **Signed purge p95 ≤ 2s** achievable via Cloudflare API (Cloudflare
  global purge ~250ms typical).
- **TLS termination at edge** simplifies certificate distribution per
  ADR-SITES-0004.

### Negative

- **Two operational surfaces** (Cloudflare API for primary + Varnish
  ops for alternative). Mitigation: ops-sre-reliability covers both;
  runbook `cdn-cache-purge-cascade.md` covers both.
- **Cost ~$0.05/GB egress** at Cloudflare list price; scale to 50M
  visitors → ~$5k/mo per cell. Already in `cost-budget.md`.
- **Cache-key change is breaking vs legacy** (Hyrum #5). Mitigation:
  migration guide documents; one-time cache warm-up via runbook.

### Operational

- **New CI lane `oya-governance-cdn-purge-contract-conformance`**
  validates signed-purge Ed25519 + cache-key format.
- **Cloudflare API token per pack**, stored in OpenBao + LEAN refused
  if pack-specific token cross-used.
- **Origin-shield mandatory**: CDN edges fetch origin via shield only;
  raw S3 access from CDN refused at IAM layer.

### Regulatory

- **GDPR Art. 32**: TLS in transit + signed purge satisfies.
- **HIPAA**: pack-us-healthcare uses self-managed Varnish edges in
  BAA-eligible regions OR BAA-on-file Cloudflare.
- **ePrivacy Art. 5(3)**: CDN edge logs IP-hashed (no raw IP storage
  at edge); ops-security audits quarterly.

## Verification

- [ ] **Signed purge contract** —
  `cargo nextest run -p oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub -- signed_purge_ed25519`.
- [ ] **Cache-key version-hash format** —
  `cargo nextest run -p oya-sites-cdn-delivery-domain -- cache_key_version_hash`.
- [ ] **Purge p95 ≤ 2s** —
  `cargo bench -p oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub -- invalidate`.
- [ ] **Per-pack residency** —
  `cargo nextest run -p oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub -- per_pack_edge_selection`.

## References

- Cloudflare API — `developers.cloudflare.com/api`.
- Varnish — `varnish-cache.org`.
- Caddy — `caddyserver.com`.
- HTTP `Cache-Control` per RFC 9111.
- HTTP `stale-while-revalidate` per RFC 5861.
- ADR-0056, ADR-0117, ADR-0131, ADR-0133, ADR-SITES-0002, ADR-SITES-0004.
- `microservices/sites/PRD.md` §"Performance".
- `migration-from-connect.md` Hyrum #5.
- `runbooks/cdn-cache-purge-cascade.md`.
