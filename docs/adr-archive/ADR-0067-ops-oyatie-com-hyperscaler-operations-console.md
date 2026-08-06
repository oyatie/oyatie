---
id: ADR-0067
status: Superseded
superseded_by: [ADR-709]
doc_status: published
sunset_topic: adr-0067-ops-console-protected-contracts
sunset_milestone: doctrine-not-time-bounded
---

# ADR-0067: `ops.oyatie.com` — the canonical hyperscaler-grade operations console for the entire oyatie platform

> **Status:** Accepted
> **Owner:** `council-architecture` + `axis-foundry` + `ops-sre-reliability`
> **Date:** 2026-05-13
> **Related:** ADR-0061 (Application B2B shell), ADR-0063, ADR-0064, ADR-0065 (docs Leptos), ADR-0066 (live introspection), Bominal ADR-0020 (observability), Bominal ADR-0117 (cell architecture), Bominal ADR-0132 (Cedar pillars)

---

## Context

User directive 2026-05-13: "This is essentially `ops.oyatie.com` — where everything ops will live. So dashboard, docs, database, schema, tech stack, architecture, health, tenant management, user management, observability, etc. Everything you would expect from hyperscaler ops."

ADR-0065 + ADR-0066 framed the work as a "docs portal." That framing was too narrow. The actual surface is a **unified operations console** — the oyatie equivalent of AWS Console + Datadog + Grafana + PagerDuty + Linear + Workday admin + GitHub Insights + Palantir Foundry's ops view — accessible at `ops.oyatie.com` with two audience tiers:

- **B2B tenant ops** — paying tenants see their own tenant scope: their µservice enablements, their health/SLO, their tenant users, their data residency, their billing.
- **Internal ops** — oyatie council/founder/SRE/Foundry teams see fleet-wide state: every tenant, every cell, every µservice, every fitness lane, every incident, every audit segment.

The "docs portal" framing of ADR-0065/0066 is correct but partial — docs is **one of ~12 surfaces** the ops console must serve.

---

## Decision

### 1. µservice rename: `docs` → `ops` (catalog entry)

Replace the `docs` µservice declared in ADR-0065 §2 with the parent µservice `ops`. The `docs` surface becomes one BC within `ops` (alongside dashboard / database / schema / tech-stack / architecture / health / tenant-mgmt / user-mgmt / observability / deployments / capacity / finops / on-call / incident / audit-view / ICM-browser / grit-status / CI-runs).

Naming per BNF v4.1: `oya-ops-*` namespace. Subdomain: `ops.oyatie.com` (subdomain scheme per Bominal ADR-0123 inheritance).

### 2. Bounded contexts (the ~12 surfaces)

Each is a BC inside `ops` µservice. All share the same Leptos SSR + SPA islands stack (per Bominal ADR-0209). Each has its own kernel/domain/application/adapter/rest layer crates per BNF v4.1.

| BC | Surface | Primary content sources |
|---|---|---|
| `overview` | Landing page; project-graph; high-level fleet state | `manifest.json` from generator; aggregated KPIs |
| `docs` | The doc surface from ADR-0065/0066 (ADRs / PRDs / microservices / phase-specs / impl-plans / packs / evidence bundles) | markdown frontmatter + body + cross-ref graph |
| `dashboards` | Hyperscaler-style customizable dashboards (per-tenant, per-fleet) | extractor manifest sections + VictoriaMetrics time-series |
| `database` | Per-µservice schema browser; live SQL DDL inventory; migration status; sample-data viewer (gated) | SQL-migrations extractor + Postgres live introspection (per Bominal ADR-0117 Citus posture) |
| `schema` | Ontology Object Type / Link Type / Action Type / Function Type browser; BC registry; entity-graph viewer | Ontology µservice (`oya-ontology-*`) + workspace metadata + `[package.metadata.oya]` per crate |
| `tech-stack` | Live cargo dep graph at crate level; per-crate versions; per-crate license; per-crate supply-chain attestation | cargo_metadata + cargo-deny + Cosign attestations |
| `architecture` | Product-graph (M01-M12+); 9 architecture planes (per Bominal ADR-0224..0231 inheritance); LEAN check lane state | product-graph extractor + 14 LEAN lane results |
| `health` | SLO / SLI / error-budget dashboards per µservice per cell; alert state; on-call schedule | OTel + VictoriaMetrics + PagerDuty-equivalent (Oyatie-native or external adapter) |
| `tenant-mgmt` | Active-tenant inventory; per-tenant µservice enablement; cell-binding; billing state; data-residency posture | Tenancy µservice + cloud-billing µservice + Application B2B shell |
| `user-mgmt` | Org users; roles (Cedar); passkey state; SSO config; session inventory | Identity µservice + Policy µservice |
| `observability` | Trace / log / metric / event explorers (Foundry-grade); cross-µservice trace stitching | OTel + VictoriaMetrics + audit-chain segment viewer |
| `deployments` | Per-cell deployment state; rollout status; canary state; rollback button (admin only) | Cloud-cell µservice + GH Actions runs + Kubernetes/OKE state |
| `capacity` | Per-cell capacity envelope; auto-scale state; pre-warmed pool health | Cloud-compute µservice + cell metadata |
| `finops` | Cost-per-tenant; cost-per-µservice; cost-per-cell; budget alerts | Cloud-billing µservice + tag-attribution |
| `on-call` | Active on-call schedule; alert routing; recent escalations; runbook search | PagerDuty-equivalent adapter + `docs/runbooks/*` |
| `incident` | Active + recent incidents; postmortems (Working Backwards / 5-whys); regression-detection signals | Bominal incident-management posture inherited |
| `audit-view` | Per-(tenant, period) Merkle-sealed Ed25519 audit-chain segment browser; tamper-evidence drill | Audit-chain µservice + KMS µservice |
| `icm-browser` | ICM rows browser (oyatie internal); filter by topic, importance, agent-session | ICM database (canonical agent-coordination ledger) |
| `grit-status` | Active grit claims; recent grit sessions; grit-done event log | grit CLI JSON output (canonical agent-coordination CLI) |
| `ci-runs` | GH Actions workflow runs; per-PR fitness lane state; lane history; failure-pattern analytics | GH Actions API + lane-state from manifest |

Total: 20 BCs. Each is a Leptos route + corresponding kernel/domain/application/adapter/rest layer crates. Naming: `oya-ops-<bc>-{kernel,domain,application,adapter,rest,worker,sdk}`.

`oya-ops-app` is the composition-root binary.

### 3. Subscope: docs portal (the work currently under consensus loop)

The ralplan-docs-portal-2026-05-13 plan is **the docs BC** of ops portal. Its consensus loop continues; the v3 plan ships the `oya-ops-docs-*` crates plus the generator + extractors + the 4 CI lanes. When that consensus accepts, the docs BC lands first.

Remaining ~19 BCs are authored in **subsequent ralplan cycles** (each gets its own consensus loop). Suggested order:

1. **docs** (in-flight; M03-P04/P05/P06)
2. **overview** + **dashboards** (M03-P06 IP extension)
3. **tech-stack** + **architecture** (M03-P06 IP extension)
4. **database** + **schema** (M03-P06 IP extension)
5. **observability** + **health** (M03-P07 IP — tied to Workflow Studio observability)
6. **tenant-mgmt** + **user-mgmt** + **deployments** (M04-onward — tied to multi-tenant operational maturity)
7. **capacity** + **finops** + **on-call** + **incident** (M04-onward)
8. **audit-view** + **icm-browser** + **grit-status** + **ci-runs** (M04-onward — Foundry-internal first, then exposed to tenant ops view)

Subdomain `ops.oyatie.com` resolves to whichever cell serves the requesting tenant (per Bominal ADR-0123 subdomain + ADR-0117 cell routing).

### 4. Audience tiers (Cedar-policy-gated)

Cedar policy fragments (per Bominal ADR-0132 inheritance):

| Tier | Audience | Cedar role | Surface visibility |
|---|---|---|---|
| **Public** | Anonymous browsers | (none required) | `/`, `/docs/decisions`, `/docs/microservices` (oyatie open ADRs / open µservice records only) |
| **Authenticated tenant** | Org members of paying tenants | `tenant-member` | Per-tenant scope: their µservice enablements, their health, their billing, their data; tenant-filtered manifest |
| **Tenant admin** | Org admins of paying tenants | `tenant-admin` | Tenant-member scope + tenant-mgmt (their tenant) + user-mgmt (their tenant) + finops (their tenant) |
| **Internal SRE** | oyatie ops-sre-reliability team | `internal-sre` | Fleet-wide health + observability + incident + on-call + deployments |
| **Internal Foundry** | oyatie axis-foundry team | `internal-foundry` | Fleet-wide CI runs + ICM + grit + audit-view (read-only) |
| **Internal admin** | Founder, council-architecture | `internal-admin` | Everything (read); deployment rollback (write); incident commander mode |

Cedar policy authoring + red-team validation per ADR-0066 §6.5 (pre-mortem §4) is a hard prerequisite before any non-public surface ships to a tenant.

### 5. Performance bar (per `feedback_quality_performance_scalability_bar`)

| Dimension | Target |
|---|---|
| Landing page first-paint (SSR) | ≤500ms p99 |
| SPA route navigation | ≤200ms p99 |
| SSE delta arrival (hot manifest section change → subscribed client) | ≤2s p99 |
| Live data refresh (warm extractor) | ≤30s p99 |
| Cross-tenant scope leak rate | **0** (Cedar policy hard-gate; CI-validated; pre-mortem §4) |
| Per-cell concurrent ops sessions | 10k+ baseline (matches `feedback_quality_performance_scalability_bar`) |
| Audit-chain segment-view latency | ≤1s per (tenant, period) per Bominal ADR-0028 |

### 5.5 Linus-style "no silent regression" policy (workspace-wide; surfaced in ops portal)

Per user directive 2026-05-13 ("Linus Torvalds style no silent regression principle"), the ops portal is the **first-class regression-detection surface**. Every regression in a public-facing contract is loud, immediate, CI-detectable. See `feedback_no_silent_regression.md` for the workspace-wide rule.

In `ops.oyatie.com` terms:

- `/ci-runs` surface displays `lean-a10-regression` results per-PR; failed lane shows the specific protected contract + the proposed delta + the required ADR template inline.
- `/audit-view` shows every regression-detection event (Ed25519-signed segment per Bominal ADR-0028) including: who attempted what change, when, against which contract, and the ADR (or supersession ADR) that authorized or rejected it.
- `/architecture` surface highlights any cross-product / cross-pack / cross-µservice contract widening with a red banner — never silent.
- `/tenant-mgmt` surface shows a per-tenant "contract version" — when oyatie rolls a new contract version, tenants see the sunset countdown for the prior version.
- Cedar policy widening (Bominal ADR-0132) NEVER expands silently — every policy bump emits an audit-chain event visible in `/audit-view`, and `/user-mgmt` displays the policy diff to tenant admins before activation.

CI lane `lean-a10-regression` is **BLOCKER day 1** (no report-only ramp). The whole point of the lane is catching what humans would otherwise miss; report-only mode would defeat the principle.

The four primary protected contracts on `ops.oyatie.com` itself:

| Contract | Versioning | Sunset window |
|---|---|---|
| `manifest.json` schema | `schema_version` field (top-level + per-section); semver-major rules | ≥1 milestone (e.g., M02-P22 → M03-P22) |
| Cedar policy fragments (per §4) | `policy_version` field; widening requires audit emit + tenant notification | ≥30 days notice for tenant-impacting changes |
| SSE delta contract | `sse_protocol_version` in handshake; backwards-compat parser in `oya-docs-portal-rest` | ≥1 milestone |
| ops.oyatie.com REST API (`/api/v1/...`) | URL-versioned path; `Deprecation` + `Sunset` HTTP headers (RFC 8594) on old version | ≥1 milestone after `Deprecation` header set |

### 6. Implications for current planning

- **ADR-0065 + ADR-0066 are subsumed.** They remain valid for the `docs` BC scope, but the parent µservice is `ops`, not `docs`. Naming convention: `oya-ops-docs-{kernel,domain,application,adapter,rest,worker}`.
- **The `docs` µservice declared in ADR-0065 §2 is retired** — renamed to `ops.docs` BC of the `ops` µservice. ADR-0065 + ADR-0066 references stand; their content semantics carry forward unchanged.
- **MASTERPLAN §2.1 catalog update**: `docs` line replaced with `ops` (with note that ops covers docs/dashboards/db/schema/etc.).
- **Workspace metadata**: `[workspace.metadata.oya.microservices.ops]` (planned status until M03-P04 crate scaffold lands).
- **In-flight ralplan-docs-portal-2026-05-13 plan**: continues; renames crate prefix from `oya-docs-*` to `oya-ops-docs-*` in implementation; otherwise the consensus loop continues unchanged.
- **Remaining 19 BCs** = subsequent ralplan cycles. Each gets its own ADR + consensus + impl-plans.

---

## Consequences

**Positive:**

- One canonical operations surface; tenants don't need 5 different tools (docs / dashboard / alerts / billing / admin).
- Internal team gets fleet-wide visibility from the same surface tenants use (with elevated Cedar role).
- Single auth + audit + observability path through `ops.oyatie.com`.
- Composable BC model means each surface evolves independently; rollout phased.

**Negative:**

- Large total scope (~20 BCs × 5-7 layer crates each = ~100-140 crates over time). Mitigated by per-BC ralplan cycles + per-phase IP dispatch.
- Concentration risk: ops.oyatie.com outage takes down the operations surface for both tenants and internal ops. Mitigated by per-cell isolation (Bominal ADR-0009) + active-active capability (ADR-0049 high-consequence µservice posture) + audit-chain segment recovery.

**Neutral:**

- Composes with ADR-0061 (Application B2B shell is the entry point; ops portal is hosted as one of the products tenants can enable, alongside Workflow Studio).
- Inherits Bominal ADR-0020 (observability), ADR-0107 (capability registry / agent gateway), ADR-0117 (OCI A1 → OKE), ADR-0132 (Cedar pillars), ADR-0224..0231 (9 architecture planes).

---

## Alternatives considered

| Alternative | Why rejected |
|---|---|
| **Multiple distinct portals** (docs.oyatie.com + dashboards.oyatie.com + admin.oyatie.com + observability.oyatie.com + ...) | Fragments the tenant experience; multiplies auth flows; cross-portal navigation breaks. Rejected per `feedback_quality_performance_scalability_bar` UX bar (Stripe/Linear single-portal pattern). |
| **Docs portal only, expand later** | User's "everything ops will live here" directive is explicit; deferring ops surfaces just means tenants and internal team rely on external tools (Datadog, PagerDuty, etc.) longer. Rejected. |
| **Use Application B2B shell as the ops portal directly (no separate µservice)** | Application is the product-enablement console (per ADR-0061); its scope is "which products is this tenant subscribed to." Ops portal scope is "what's the state of everything." Different abstraction; rejected. |
| **Per-µservice admin UIs** (e.g., each µservice ships its own admin surface) | Hidden ops state per-µservice; no cross-µservice view; agents can't query a single manifest endpoint. Rejected per ADR-0066 single-pane requirement. |
| **External tools (Datadog, Grafana, PagerDuty)** | oyatie quality bar is industry-leader parity, not "pay the industry leaders for the surface" (per `feedback_quality_performance_scalability_bar`). External tools also can't enforce Cedar policy on oyatie's per-tenant scope, can't surface Oyatie-specific audit-chain or ICM/grit state. Rejected for primary ops; may integrate as data sources via adapters. |

## Governed surfaces

The following repo paths are governed by this ADR. The accounting gate validates that each is
justified (this ADR is the justification reference):

`contracts/ops-docs-v1.openapi.meta.yaml`
`contracts/ops-workspace-shell-v1.openapi.meta.yaml`
`contracts/OWNERS`
---

## Compliance

CI lanes (M02-P22 BLOCKER):

- `lean-a5-documentation` + `lean-a6-docs-generated-consistency` + `lean-a7-endpoint-coverage` + `lean-a8-dead-code-zero-tolerance` (per ADR-0063/0066)
- NEW `lean-a9-ops-policy-coverage` (M03-P06 scope) — every non-public ops surface has a Cedar policy fragment + red-team probe in the test set (per pre-mortem §4 of ralplan-docs-portal). BLOCKER before any non-public surface goes live.

Owner: `axis-foundry` (substrate + extractors) + `ops-sre-reliability` (dashboards / health / on-call / incident / capacity / finops) + `council-privacy` (Cedar policy fragments) + `gtm-customer-success` (tenant-facing surface design).

First green window: M03-P06 (Application B2B live) ships docs BC. Remaining BCs land in subsequent M03-onward / M04-onward phases.

---

## References

- ADR-0061 (Application B2B shell)
- ADR-0063 (doc-coverage CI lanes)
- ADR-0064 (canonical base + localization packs)
- ADR-0065 (Leptos SSR + machine-readable docs)
- ADR-0066 (live code-introspection)
- Bominal ADR-0009 (cell architecture)
- Bominal ADR-0020 (observability)
- Bominal ADR-0028 (audit chain Ed25519)
- Bominal ADR-0049 (cross-region replication / high-consequence µservices)
- Bominal ADR-0107 (capability registry / agent gateway)
- Bominal ADR-0117 (OCI A1 → OKE staged scaling)
- Bominal ADR-0123 (OIDC + passkey + subdomain scheme)
- Bominal ADR-0132 (Cedar policy + pillars)
- Bominal ADR-0209 (Leptos client stack)
- Bominal ADR-0224..0231 (9 architecture planes)
- `feedback_quality_performance_scalability_bar`
- `feedback_workflow_studio_scope` (Workflow Studio is first product; Ops Portal is N+1 product)
- `.omc/plans/ralplan-docs-portal-2026-05-13.md` (in-flight; docs BC is the first ops surface to ship)
