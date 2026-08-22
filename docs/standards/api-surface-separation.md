---
contract: api-surface-separation
authored: 2026-05-18
canonical_authority: ADR-0177
related_specs:
  - /specs/api-surface-separation.json
related_adrs:
  - ADR-0011
  - ADR-0037
  - ADR-0044
  - ADR-0148
  - ADR-0157
  - ADR-0177
status: canonical-base
authorities_cited:
  - Stripe Engineering — API separation handbook excerpt
  - AWS — control plane vs data plane service design
  - Google Cloud — public APIs vs internal endpoints
---

# Internal vs external API surface separation standards

## Two surfaces, two hostnames

| Surface | Hostname | Audience | Auth | Rate-limit tier | Semver |
| --- | --- | --- | --- | --- | --- |
| Public | `api.oyatie.com` | External customers, partners | OAuth 2.0 + per-key signature | per-public-key + per-IP (ADR-0178) | Mandatory per ADR-0037 |
| Internal | `internal-api.oyatie.com` | Other µservices only | Mesh mTLS + SPIFFE id | per-microservice, 10× public budget | Optional |

## Route classification

Every OpenAPI route declares in its spec metadata:

```yaml
api_surface: public  # or: internal
```

The gateway tier reads the classification and routes accordingly.
Mis-routed requests (external client hitting an internal route) get
HTTP 404; the route is *not visible* outside the internal mesh.

## Semver discipline

| Change kind | Public surface | Internal surface |
| --- | --- | --- |
| Add optional field | semver minor; changelog entry | next deploy; ChangeSet entry |
| Add required field | semver major; deprecation cycle + customer migration runbook | next deploy + 7 days notice in ChangeSet |
| Remove field | semver major; 90-day deprecation | next deploy + 7 days notice |
| Change behavior | semver major; customer migration runbook | next deploy |
| Add route | semver minor; documented | next deploy |
| Remove route | semver major; 90-day deprecation | next deploy + 7 days notice |

## Promoting internal → public

Requires an ADR amendment plus the full ADR-0037 surface:

- Public-Stable stability tier annotation.
- Customer-facing docs in `docs/products/*/`.
- SDK regeneration through public SDK pipeline.
- Public changelog entry.
- 90-day deprecation cycle for any previously-Internal-only behavior
  the public consumers will rely on.

Demoting public → internal: same requirements PLUS a 180-day notice
window (vs the standard 90-day) because public consumers must migrate.

## Ingress isolation

The mesh ingress (Cilium L7 per ADR-0148):

| Surface | Ingress accepts | DNS record |
| --- | --- | --- |
| Public | any source IP; OAuth + key signature mandatory | public DNS |
| Internal | only intra-mesh SPIFFE ids | public DNS → internal LB → drop non-mesh at L4 |

## Observability split

Two dashboards:

- `microservices/observability/dashboards/api-public.md`
- `microservices/observability/dashboards/api-internal.md`

The public dashboard is the customer-impact triage surface; the
internal dashboard is the cross-µservice flow surface.

## SDK split

| SDK | Source | Audience |
| --- | --- | --- |
| Public | `platform-api-sdk-*` family per ADR-0036 (generated from public routes only) | External customers, partners |
| Internal | `oya-*-internal-sdk-*` family (generated from internal routes; Rust workspace members) | Other µservices |

## Documentation split

- `docs/products/*/prd.md` references public routes only.
- Internal routes live in `microservices/<ms>/contracts/internal/`.
- Doc-portal (ADR-0066) renders both with explicit "Public" vs
  "Internal" badges.

## Coverage tracker

Per-route classification rollout in
`registry/api-surface-classification/coverage-tracker.tsv`. Validator
lane `api-surface-classification` is advisory until coverage reaches
100% of openapi routes.
