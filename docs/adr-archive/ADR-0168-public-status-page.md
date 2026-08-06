---
id: ADR-0168
status: Superseded
deciders: council-architecture, ops-sre-reliability, axis-observability, axis-tenant-trust
date: 2026-05-18
owner: ops-sre-reliability
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0042, ADR-0121, ADR-0139, ADR-0148, ADR-0167]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/per-microservice-flat-layout.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0168 — Public status page derived from SLO state

## Status

Accepted (2026-05-18). Authorizes a public status page (status.oya.dev) automatically derived from per-µservice SLO state per ADR-0139, with incident updates pushed from the incident-response µservice. Tier C "nice-to-have" hyperscaler pattern per `/specs/hyperscaler-architecture-invariants.json` audit Row C2.

## Context

Every hyperscaler tenant expects a public status page showing real-time service health. The canonical references:

- **status.stripe.com** — per-API-product SLO state; live incident timeline; subscribe via email/RSS/webhook.
- **status.aws.amazon.com** — per-region per-service health; multi-month history.
- **status.cloudflare.com** — per-product status with detailed incident reports.
- **www.githubstatus.com** — per-product status; postmortems linked from each incident.
- **status.anthropic.com** — per-model availability; API + console status separately.

Today Oyatie publishes SLO state internally (ADR-0139 agentic SLO-gated promotion) but has no tenant-facing surface that exposes "is the platform up?" without logging into the tenant console. Tenants writing runbooks, status integrations, on-call playbooks, and incident-response procedures need a public, scrapeable, subscribe-able status surface.

Three signals must feed the status page:

1. **Per-µservice SLO health** — derived automatically from the SLO engine (ADR-0139 Layer-A Grafana / Mimir / Loki stack, IP-003 SLO engine kernel).
2. **Active incidents** — pushed from the incident-response µservice's runbook execution timeline.
3. **Scheduled maintenance** — pushed from the deployment-pipeline µservice when a window is registered.

The status page must NOT require manual operator updates for routine SLO degradations — those derive automatically. Manual updates are reserved for incident narrative + scheduled-maintenance announcements.

## Decision

Oyatie deploys a public status page at `status.oya.dev` (and per-pack subdomains: `status.kr.oya.dev`, `status.eu.oya.dev`, etc. per ADR-0010 regional packs) automatically derived from SLO state per ADR-0139, with incident narrative pushed from the incident-response µservice.

### Architecture

```
┌─────────────────────┐    ┌──────────────────────┐
│ ADR-0139 SLO engine │───▶│ statuspage projector │
│  (per-µservice)     │    │   (this ADR)         │
└─────────────────────┘    └──────────┬───────────┘
                                      │
┌─────────────────────┐                │
│ incident-response   │────────────────┤
│   µservice          │                │
└─────────────────────┘                │
                                       ▼
┌─────────────────────┐    ┌──────────────────────┐
│ deployment-pipeline │───▶│  status.oya.dev SPA  │
│  (maint windows)    │    │  + JSON/RSS/webhook  │
└─────────────────────┘    └──────────────────────┘
```

The "statuspage projector" is a thin read-side component deployed under `microservices/observability/iac/helm/statuspage/`. It:

1. Subscribes to the `slo.health.changed` event stream (ADR-0005 outbox pattern).
2. Subscribes to the `incident.timeline.updated` event stream from incident-response.
3. Subscribes to the `maintenance.window.scheduled` event stream from deployment-pipeline.
4. Projects into a Postgres read-side table `status_page_state` (per-product, per-pack).
5. Serves a static SPA + a JSON API at `https://status.oya.dev/api/v2/` (Statuspage.io-compatible shape for tooling parity).
6. Emits per-product RSS feeds + webhook subscriptions for tenant integrations.

### Public-facing surface

| Surface | Path | Format | Subscribe? |
|---|---|---|---|
| Web SPA | `https://status.oya.dev/` | HTML | — |
| Summary JSON | `https://status.oya.dev/api/v2/summary.json` | JSON (Statuspage.io schema) | — |
| Component JSON | `https://status.oya.dev/api/v2/components.json` | JSON | — |
| Incident history | `https://status.oya.dev/api/v2/incidents.json` | JSON | — |
| RSS — all incidents | `https://status.oya.dev/history.rss` | RSS 2.0 | yes |
| RSS — per-product | `https://status.oya.dev/products/<product>/history.rss` | RSS 2.0 | yes |
| Webhook subscription | `POST /api/v2/subscribers/webhook` | tenant supplies URL | yes |
| Email subscription | `POST /api/v2/subscribers/email` | tenant supplies email | yes |

### Per-product surface

Each public-facing product (per ADR-0001 flat catalog) gets its own component on the status page: workflow, messenger, tasks, social, drive, mail, calendar, ontology, foundry, etc. Each component rolls up its constituent µservices' SLOs.

### Status enum (Statuspage.io parity)

| Status | Trigger | Color |
|---|---|---|
| `operational` | all SLOs healthy | green |
| `degraded_performance` | latency SLO breached, availability SLO healthy | yellow |
| `partial_outage` | availability SLO breached for a subset of cells/regions | orange |
| `major_outage` | availability SLO breached fleet-wide | red |
| `under_maintenance` | scheduled maintenance window active | blue |

### Tenant-CLI integration

`oya status` command (ADR-0167) calls `status.oya.dev/api/v2/summary.json` and renders the table. Power-tenants can subscribe to webhooks from their own incident-response tooling.

## Alternatives considered

### A. Outsource to Statuspage.io (Atlassian)
- Pros: zero-code; mature SPA; well-known tenant UX.
- Cons: ~$1.5k/mo at our scale; tenant data leaves the EU/KR residency boundary (violates ADR-0008 data-use boundary for incident narrative containing tenant names); manual operator updates required (does not auto-derive from our SLO state).
- **Rejected**: residency violation + manual-update toil + recurring cost above an estimated 18-month payback for in-house build.

### B. Static HTML manually updated by SRE on-call
- Pros: zero infrastructure; minimal complexity.
- Cons: lag between SLO breach and page update (15-60min observed at peer companies); no subscribe surface; no JSON API for tenants to scrape; not the hyperscaler shape.
- **Rejected**: manual-update toil; missing tenant subscribe surface; violates "automate everything" memory directive.

### C. Internal-only Grafana dashboard, no public surface
- Pros: already exists under ADR-0042 Grafana stack; zero new code.
- Cons: tenants need to log in to see status — useless during an auth-µservice outage; no subscribe surface; not the hyperscaler shape.
- **Rejected**: useless during auth outage (the precise moment a tenant needs the status page); does not serve the tenant-runbook integration use case.

### D. Per-product status pages, no unified roll-up
- Pros: each product owns its own status page; clear ownership.
- Cons: tenants must check N pages to assess overall health; fragments the brand; not the hyperscaler shape (status.stripe.com is one page per company, not per product).
- **Rejected**: fragments the tenant surface; one status page per company is the industry pattern.

## Consequences

### Positive

1. **Hyperscaler-parity** — tenants get a `status.oya.dev` page on par with status.stripe.com and status.aws.amazon.com. Audit Row C2 closed.
2. **Auto-derived from SLO state** — routine degradations surface automatically per ADR-0139; no SRE toil for non-incident health changes.
3. **Survives auth outages** — `status.oya.dev` runs on a dedicated cell separate from the main tenant-auth path (cell-isolation per ADR-0009).
4. **Subscribe surface** — tenants integrate via RSS / webhook / email without polling.
5. **Tenant-CLI integration** — `oya status` provides a single-command health check (ADR-0167).

### Negative

1. **Statuspage.io schema commitment** — we adopt their public API schema for tooling parity. Any breaking change in their schema requires an Oyatie response within 30 days (semver-style).
2. **Incident-narrative authoring tool needed** — SRE on-call needs a forms surface to author incident updates. Built into the incident-response µservice (existing).
3. **Dedicated cell footprint** — `status.oya.dev` runs on a tenant-isolated cell to survive main-platform outages; ~3 small nodes per region adds ~$200/mo per region.
4. **Subscriber list is PII** — email subscribers handled per ADR-0008 data-use boundary; encrypted at rest, deletable per ADR-0038 DSR cascade.

### Operational

1. `microservices/observability/iac/helm/statuspage/` ships the helm chart per this ADR.
2. SLO state subscription via Mimir's remote-read API (per ADR-0139 IP-003).
3. Incident narrative authored via `oya incident publish ...` (internal-CLI subcommand; not tenant-facing).
4. Subscriber notification: webhook + RSS + email. SMTP delivery via the mail µservice's outbound queue.
5. SLO for the status page itself: 99.99% availability (one nine higher than the platform — the status page must outlive the platform during outages).

### Cell-isolation contract

The status page runs in a dedicated cell `cell-statuspage-<pack>` per ADR-0009. Hard constraints:

- No dependency on the main tenancy µservice for the public read surface (page renders without authenticated calls).
- Subscriber notification calls (webhook delivery) DO depend on mail + webhook-delivery kernel per ADR-0169; degraded if those µservices outage. Page itself stays up.
- Static SPA assets served from a CDN edge per pack; origin only serves the JSON API.
- Origin hosted on the federation-tier cluster per ADR-0171 (NOT on the main tenant-workload cluster) — preserves outage independence.

### Incident-state machine

| State | Triggered by | Renderable |
|---|---|---|
| `investigating` | SRE manual or SLO breach alert | yes |
| `identified` | SRE update via `oya incident publish` | yes |
| `monitoring` | SRE update | yes |
| `resolved` | SRE update or SLO returns to green for ≥10min | yes |
| `postmortem` | postmortem-link attached after resolution | yes |

Each transition emits an event into the public RSS / webhook feed. State-machine enforcement at the incident-response µservice (existing); statuspage projector renders the current state.

### Subscriber data retention

- Email subscribers: stored encrypted-at-rest in Postgres; deletable per ADR-0038 DSR cascade.
- Webhook endpoints: stored encrypted-at-rest; tenant-rotatable via API.
- Subscriber list retained until tenant unsubscribe; bulk-purged on tenant account closure per ADR-0038.
- Subscriber notifications logged 90 days for delivery-debugging; auto-purged thereafter.

### Performance budgets

- Page load p99 ≤500ms (CDN edge, no backend call).
- Summary JSON p99 ≤200ms.
- SLO-state-change → page reflects change ≤30s p99 (Mimir remote-read pull cadence).
- Webhook delivery from incident-state-change → subscriber URL ≤5s p99 (via webhook-delivery kernel ADR-0169).

### Migration / rollout plan

1. M01 slice: helm chart skeleton + read-side Postgres projector (this ADR's companion).
2. M01.5: SLO subscription wired; per-product status surfaces live (read-only; manual incidents only).
3. M02: incident-response µservice push integration; RSS + email subscribe live.
4. M02.5: webhook subscriber surface; tenant-CLI `oya status` command live.
5. M03: per-pack subdomain rollout (`status.kr.oya.dev`, `status.eu.oya.dev`).

## References

- status.stripe.com — https://status.stripe.com — canonical reference; OSS Statuspage.io schema-compatible.
- status.aws.amazon.com — https://health.aws.amazon.com/health/status — per-region per-service rollup; multi-month history.
- status.cloudflare.com — https://www.cloudflarestatus.com — per-product status + detailed incident reports.
- www.githubstatus.com — https://www.githubstatus.com — postmortems linked from each incident.
- status.anthropic.com — https://status.anthropic.com — per-model availability decomposition.
- Statuspage.io public API schema — https://developer.statuspage.io/ — JSON shape we adopt for tooling parity.
- Google SRE Book Ch. 4 — Service Level Objectives — SLO-driven status surfaces.
- ADR-0005 — eventing backbone outbox pattern (event-driven projection into the status page).
- ADR-0008 — data-use boundary (subscriber email handling).
- ADR-0009 — cell architecture per-tenant per-region (status page runs on a dedicated cell).
- ADR-0042 — observability stack OTel + in-house UI (underlying telemetry).
- ADR-0139 — agentic SLO-gated promotion (source of SLO health signal).
- ADR-0148 — service-mesh Istio (status-page µservice integrates the canonical mesh trait).
- ADR-0167 — tenant-facing CLI (`oya status` command consumes this status page).
- `/specs/hyperscaler-architecture-invariants.json` — audit Row C2 closes here.
