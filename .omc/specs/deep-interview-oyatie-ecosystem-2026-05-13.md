---
doc_class: DeepInterviewSpec
interview_id: oyatie-ecosystem-2026-05-13
rounds: "~7 (Round 0 topology + Rounds 1–5 + clarifications)"
final_ambiguity: "~12% (active components; M04+ deferred per user)"
type: brownfield
generated: 2026-05-13
threshold: 10%
status: PASSED_WITH_DEFERRAL
consensus: near-term M01–M03 locked; M04+ explicitly deferred
milestone: M-CC (cross-cutting)
phase: n/a (spec-level artifact)
---

# Deep Interview Spec: Oyatie Ecosystem Bigger Picture

## Metadata

- **Interview ID**: oyatie-ecosystem-2026-05-13
- **Rounds**: ~7 (Round 0 topology + Rounds 1–5 + clarifications)
- **Final Ambiguity**: ~12% (active near-term components resolved; M04+ deferred per user instruction)
- **Type**: brownfield (oyatie is a parallel codebase of Bominal; architecture inherits with named overrides)
- **Generated**: 2026-05-13
- **Threshold**: 10%
- **Status**: PASSED_WITH_DEFERRAL — near-term M01–M03 consensus reached; M04+ explicitly out of scope for this interview

---

## Clarity Breakdown

| Area | Clarity | Notes |
|---|---:|---|
| BNF v4.1 naming convention | 95% | User-confirmed; ADR-0056 amendment pending |
| Flat microservice catalog (no verticals/arms/groups) | 98% | User-confirmed; all 5 clarification rounds consistent |
| Workflow + Ontology = adapter layer | 97% | User-confirmed; THE load-bearing architectural rule |
| Bominal inheritance precedence | 95% | Default = inherit; session overrides listed |
| Glossary (shared/Ontology/Application/connect) | 98% | User-confirmed; all 4 terms locked |
| M01 scope (v4 cutover) | 90% | In-flight; Shard 1 dispatched |
| M02 substrate scope | 85% | Substrate components listed; sequencing TBD per Bominal evidence |
| M03 first-paying-tenant scope | 88% | KR group payroll + mail confirmed; broader bundle per Bominal M3 |
| M04+ µservice catalog sequencing | 12% (DEFERRED) | Explicitly deferred by user |
| Operating model + Bominal-inheritance rules | 92% | Explicit override list captured |

---

## Topology (5 confirmed components)

1. **M01 cutover (in flight)** — BNF v4.1 atomic rename of 140 crates; Shard 1 dispatched with grit claim coordination; 26 PROTOCOL-UNKNOWN rows deferred to Shard 1.5; 4 LEAN check crates scaffolded.
2. **M02 Substrate ready** — Foundry engine + Cloud + Ontology + Workflow + Application B2B shell + supporting µservices (tenancy / identity / audit-chain / eventing / secrets / observability / KMS / policy-cedar / search / vector / data-boundary / finance-library / capability-registry / records / ads / analytics). Exit gate: sibling team can scaffold + ship any µservice via grit claim/work/done with zero build-team help; 9 architecture planes (Bominal ADR-0224..0231) green; all `--report-only` lanes flipped to BLOCKER.
3. **M03 First-paying-tenant GA** — 1 KR group paying tenant live. Minimum bundle: HR + Payroll + Accounting + Connect Professional (Mail + Messenger) + Cloud-Tenancy substrate. Compliance: 4대보험 EDI + 연말정산; legal hold + eDiscovery for corporate mail; Merkle/Ed25519 audit chain segmented per (tenant_id, period).
4. **M04+ µservice catalog sequencing** (DEFERRED) — Healthcare, FinTech, Connect Personal, Industrial Suite, international (US/EU). To be planned in follow-up session.
5. **Operating model + Bominal-inheritance precedence** (active) — Flat catalog; 10 named overrides from Bominal; session decisions override Bominal; ADRs 0055/0056/0057/0058/0059/0060/0061 to be authored.

---

## Goal

Lock the bigger-picture Oyatie ecosystem plan; align M01–M03 near-term execution; defer M04+ Healthcare/FinTech/Connect-Personal/Industrial-Suite/international to follow-up planning sessions.

---

## Constraints

- Workflow + Ontology = ecosystem adapter layer (load-bearing; products never call each other directly)
- Flat microservice catalog (no Product Groups, no Verticals, no Arms, no grouping)
- Glossary: `shared` (not platform), `Ontology` (not Object Graph), `Application` (not Shell), `microservice` (not vertical/arm/group), `connect` (not workspace), `flat catalog` (not Arms)
- BNF v4.1: `oya-<microservice>[-<bc>]-<layer>` (BC optional); `oya-check-<rule>` BNF-exempt
- Bominal-inheritance precedence: adopt Bominal ADRs 1:1 with glossary translation; session decisions override
- Sales segmentation labels (Healthcare/Enterprise/FinTech/Social) for GTM only — NOT architecture
- B2C Connect Personal = separate entry path from B2B Application (person-pillar; does NOT go through B2B shell)
- Markets: KR-first, US parallel, EU after; jurisdiction-pluggable per ADR-0140
- Runtime: OCI A1 → OKE stages per ADR-0117
- Sanctioned primitives: `{grit, icm, oya-codeview-cli}` (post-M01-cutover name for oya-tooling-agent-read)
- **Quality bar = industry leaders** (Stripe/Palantir/Linear/Superhuman) competitive-benchmarked before any µservice graduates L4→L5
- **Performance bar = hyperscaler-grade**: p99 ≤50ms read / ≤200ms write; 10k+ req/s per cell baseline; 100M+ users architecture
- **Horizontal scalability mandatory from day one**: stateless services, sharded state, event-driven, cell architecture, active-active capable
- Stale information removed (not marked retired); no alias migration; no dead code
- Plans must be implementation-ready for autonomous "Implement the masterplan" execution

---

## Non-Goals

- M04+ specific sequencing (deferred)
- Payment regulatory hurdles in near-term (전자금융업 deferred to M05)
- Healthcare regulatory hurdles in near-term (의료법/HIRA/KFDA deferred to M04)
- Workspace branding (renamed to Connect; branding decisions outside scope)
- Product Group taxonomy (retired; flat catalog only)
- Alias/compatibility shims for renamed crates (atomic rename = old name GONE)

---

## Acceptance Criteria

All criteria are concrete and testable by an autonomous agent invoking `Implement the masterplan` referencing this spec + MASTERPLAN.md + phase SPECs.

### M01 — v4.1 Cutover

- [ ] `cargo check --workspace --all-features` exits 0 on post-Shard-1 `main`
- [ ] `rg -F -f /tmp/old-crate-names.txt . -g '!docs/CHANGELOG.md' -g '!docs/plans/rename-plan-*.md'` returns exit 1 (no hits — all old names gone)
- [ ] `rg "oya-platform-\|oya-workspace-\|oya-foundation-\|oya-tooling-\|oya-shared-" crates/` returns exit 1 (no old-prefix dirs)
- [ ] `rg "oya-object-graph" crates/ docs/ .github/` returns exit 1 (Ontology rename complete; old name gone)
- [ ] Cargo.lock contains zero entries matching old crate names (lockfile-parity gate exits 0)
- [ ] 4 LEAN check crates (`oya-check-architecture`, `oya-check-bounded-contexts`, `oya-check-supply-chain`, `oya-check-semver`) all scaffold-present and green on `cargo build`
- [ ] `oya-check-architecture -- report --format json` exits 0 (all 7 subcommands pass)
- [ ] 26 PROTOCOL-UNKNOWN rows resolved and renamed in Shard 1.5 (separate PR; gate: `rg "PROTOCOL-UNKNOWN" crates/*/Cargo.toml` exits 1)
- [ ] ADR-0056 status = Accepted and reflects BNF v4.1 (microservice slot; BC optional)
- [ ] ADR-0057 status = Accepted
- [ ] ADR-0054 amendment present (`rg "Amendment 2026-05-13" docs/decisions/ADR-0054*.md` exits 0)
- [ ] `[workspace.metadata.oya.microservices]` registered for every µservice referenced in rename map
- [ ] 4 partition sign-offs collected (PR comment heuristic returns 4)
- [ ] No alias crates; no re-export shims; `grep -r "pub use oya_platform\|pub use oya_workspace\|pub use oya_foundation\|pub use oya_tooling" crates/` exits 1

### M02 — Substrate Ready

- [ ] `cargo run -p oya-check-architecture -- report --format json` exits 0 (all architecture planes green)
- [ ] All `--report-only` CI lanes flipped to BLOCKER (verify: `rg "severity: BLOCKER" .github/workflows/checks.yml | wc -l` returns expected count)
- [ ] Sibling team onboarding gate: a fresh agent with no prior context can scaffold + ship a new µservice via `grit claim/work/done` in under 60 minutes with zero build-team intervention (recorded runbook demo passes)
- [ ] 9 architecture planes (Bominal ADR-0224..0231) all green on `main`
- [ ] `oya-application-app` (B2B shell) deployable to OCI A1 staging (smoke test exits 0)
- [ ] Foundry engine: capability registry online with ≥50 capabilities; autonomy ceiling enforcement live; evidence chain emission per invocation
- [ ] Ontology µservice: `oya-ontology-*` crates build and pass tests; typed entity/link/action/function contracts published
- [ ] Workflow µservice: `oya-workflow-*` crates build; state machine + approval + escalation BCs operational
- [ ] Every PRD for M02 µservices includes: Competitive Benchmark section + Performance Targets section + Horizontal Scalability section
- [ ] Every M02 impl plan includes `## Load test` section with results meeting p99 ≤50ms read / ≤200ms write targets
- [ ] 4 new CI lanes operational: `oya-check-statelessness-cli` + `oya-check-shardability-cli` + `oya-check-perf-budget-cli` + `oya-check-benchmark-cli`
- [ ] `oya-check-statelessness-cli` green on all `application` + `rest` + `grpc` + `worker` layer crates
- [ ] `oya-check-shardability-cli` green (all DB designs declare `tenant_id` partition key + row-level isolation)

### M03 — First-Paying-Tenant GA

- [ ] 1 KR group paying tenant live (verifiable: tenant record in prod DB with `status = active` and at least 1 payroll cycle completed)
- [ ] 4대보험 EDI green (국민연금 + 건강보험 + 고용보험 + 산재보험 electronic submissions passing validation)
- [ ] 연말정산 (year-end tax settlement) workflow complete for test tenant
- [ ] Connect Pro Mail live: legal hold + eDiscovery operational (gate: `cargo test -p oya-connect-ediscovery-*` exits 0; hold/release cycle recorded in audit chain)
- [ ] Audit chain segmented per (tenant_id, period) with Merkle/Ed25519 (gate: `cargo test -p oya-audit-chain-*` exits 0; cross-tenant proof verified)
- [ ] KR jurisdiction Cedar policy overlays active (gate: `cargo run -p oya-policy-cedar-* -- verify --jurisdiction KR` exits 0)
- [ ] All M03 ADRs authored and Accepted: ADR-0058 (KR-group launch bundle), ADR-0059 (Connect Pro legal-hold), ADR-0060 (4대보험 EDI), ADR-0061 (연말정산)
- [ ] Competitive benchmark completed for every shipped µservice (HR vs 더존 WEHAGO; Payroll vs ADP; Connect vs Slack/Gmail; Ontology vs Palantir Foundry)
- [ ] Load test results on record for all M03 µservices (p99 read ≤50ms, write ≤200ms at 10k req/s per cell)

### Cross-cutting (all milestones)

- [ ] All session-decided overrides authored as ADRs (0055/0056/0057/0058/0059/0060/0061)
- [ ] BNF v4.1 amendment landed (ADR-0056 v4.1)
- [ ] MASTERPLAN refreshed to canonical glossary (no `platform`, `Object Graph`, `workspace` crate names, `Product Group`, `Arm`, `vertical/shared binary slot`)
- [ ] All stale ADRs/plans/specs/docs DELETED (not marked retired) — `rtk grep -rl "Product Group\|Object Graph\|<shared|vertical>\|Modular Product Shell\|oya-platform-\|oya-workspace-" docs/ .omc/plans/ .omc/specs/` returns no hits outside of this spec's historical-transcript section
- [ ] Bominal-inheritance override list in `feedback_bominal_inheritance_precedence.md` reflected in oyatie ADRs (10 overrides, each with an ADR cite)
- [ ] `oya-check-benchmark-cli` green (every µservice that has reached Proof-Ladder L4 has a competitive-benchmark section in its PRD)

---

## Assumptions Exposed and Resolved

| # | Assumption | Resolution |
|---|---|---|
| 1 | Oyatie = separate codebase from Bominal | Confirmed: "oyatie is just working in parallel. same product as bominal." Two parallel codebases of the same product. |
| 2 | Products are grouped by industry (Healthcare Arm, Corporate SaaS Arm, etc.) | RESOLVED WRONG: flat microservice catalog; no grouping; segmentation is GTM-only |
| 3 | Workflow is Corporate-owned | OVERRIDE: Workflow is shared µservice; diverges from Bominal per [[feedback-workflow-is-shared]] |
| 4 | Object Graph is the data layer name | OVERRIDE: renamed to Ontology (Palantir term); [[feedback-glossary-ontology-not-object-graph]] |
| 5 | "Platform" = substrate layer name | OVERRIDE: `shared` is canonical; `platform` retired; [[feedback-glossary-shared-not-platform]] |
| 6 | Workspace = Connect product name | OVERRIDE: renamed to Connect; dual-context Personal + Professional per ADR-0208 model |
| 7 | BNF slot2 = shared|vertical binary | RESOLVED: slot2 retired; BNF v4.1 = `oya-<microservice>[-<bc>]-<layer>` |
| 8 | M01 needs 48h freeze window | RESOLVED: user directed "no need to wait 48 hours. start the review now." Expedited. |
| 9 | MVP-quality releases acceptable | RESOLVED: quality bar = industry leaders; no MVP releases; compete with Stripe/Palantir/Linear from day one |
| 10 | Horizontal scalability can be added later | RESOLVED: mandatory from day one; CI lanes enforce it |

---

## Technical Context

### Architecture Diagram

```
Foundry (internal-only µservice):
  grit, icm, oya-codeview-cli, LEAN check binaries,
  xtask-metadata-augment, Cedar engine, Wasmtime/Firecracker,
  Proof Ladder L0–L7, 9 architecture planes, Wave integration framework.

Application (oya-application-app — B2B unified shell):
  Tenants sign in; enable µservices à-la-carte (AWS-console model).

Flat catalog of µservices (any tenant can enable any subset):

  Customer-facing:
    medical, pharmacy, healthcare-portal, emergency, clinical,
    hr, payroll, accounting, ats, grc, performance,
    manufacturing, logistics, facility-ops, procurement, security,
    payments, insurance, finance, banking,
    connect (dual-context: messenger + mail + community),
    dining, cellar, ... (roadmap M04+)

  Cross-product adapter/glue layer (all inter-product integration):
    workflow (state machines, DAGs, approvals, escalations, SLA, handoffs)

  Information layer (Palantir-Ontology equivalent):
    ontology (typed entities + links + actions + functions +
              audit-chain + pillars [org/person] + property tiers +
              DUB + RLS + jurisdiction overlays)

  Substrate µservices (always-on; underpin every product):
    tenancy, identity, audit-chain, eventing, secrets,
    observability, kms, policy-cedar, search, vector,
    data-boundary, finance-library, capability-registry,
    records (FHIR-canonical), application (B2B shell),
    ads, analytics

  Runtime substrate µservices:
    cloud-tenancy, cloud-iam, cloud-kms, cloud-compute, cloud-storage,
    cloud-network, cloud-billing, cloud-cell, cloud-region,
    cloud-observability

B2C entry path:
  connect-personal (individuals; person-pillar)
  — does NOT go through B2B Application shell
```

### Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (workspace; all crates) |
| Web framework | Axum (REST) + Tonic (gRPC) + async-graphql (GraphQL) |
| DB | PostgreSQL + pgvector + pgroonga; Citus for sharding (stage 2); ClickHouse (analytics) |
| Search | pgroonga (full-text, KR/EN/ZH) + Tantivy (embedded) |
| Event streaming | Kafka KRaft (outbox → Kafka at scale per ADR-0116) |
| Cache | Valkey/Redis cluster |
| Policy engine | Cedar (AWS Cedar OSS; ADR-0140) |
| Auth | OIDC + PKCE + nonce (ADR-0123); own-rails identity |
| Runtime | OCI A1 Always Free → OKE → multi-region (ADR-0117 stages) |
| IaC | Pulumi (Rust SDK) |
| Observability | OpenTelemetry + Grafana stack |
| CI primitives | cargo-nextest, cargo-deny, cargo-semver-checks, cargo-llvm-cov, sccache |
| Client | Leptos (web, Rust/WASM) + 5 native platforms + SvelteKit (prototype lane per ADR-0209) |
| Document generation | Typst |
| Agent runtime | Foundry (internal); sanctioned primitives: grit + icm + oya-codeview-cli |

### Competitive Map

| Domain | Competitors (benchmark targets) |
|---|---|
| HR/Payroll (KR) | 더존 WEHAGO, 영림원 |
| HR/Payroll (global) | ADP, Workday, Rippling |
| Healthcare | 유비케어, 이지케어텍, Epic, Cerner |
| FinTech/Payments | Stripe, 토스, Kakao Pay |
| Connect (messaging + mail) | Slack, Gmail, Signal, Notion |
| Ontology / data layer | Palantir Foundry |
| Tenancy / identity | Auth0, Okta, AWS Cognito |
| Workflow | Camunda, Temporal, Linear |
| Accounting/ERP (KR) | 더존 iCUBE, SAP |

---

## Ontology (Key Entities)

22 tracked entities across the ecosystem:

| Entity | Owner µservice | ADR cite |
|---|---|---|
| Tenant | tenancy | ADR-0018, ADR-0125 |
| Organization | tenancy | ADR-0125 |
| User | identity | ADR-0125 |
| Person | ontology (person-pillar) | ADR-0132 |
| Employee | hr | ADR-0126 |
| Employment | hr | ADR-0126 (8 classes) |
| Cell | cloud-cell | ADR-0009 |
| Region | cloud-region | ADR-0019 |
| CapabilityRecord | foundry | ADR-0019 |
| WorkflowItem | workflow | ADR-0232 (overridden: shared) |
| WorkflowTransition | workflow | ADR-0232 |
| OntologyObjectType | ontology | ADR-0106 |
| OntologyLinkType | ontology | ADR-0106 |
| OntologyActionType | ontology | ADR-0106 |
| OntologyFunction | ontology | ADR-0107 |
| AuditEvent | audit-chain | ADR-0028 |
| PolicyPack | policy-cedar | ADR-0140 |
| JurisdictionOverlay | regulatory-pack | ADR-0127 |
| Message | connect (messenger) | ADR-0208 |
| Mail | connect (mail) | ADR-0208, ADR-0215 |
| PayrollCycle | payroll | ADR-0210 |
| LegalHold | connect (ediscovery) | ADR-0215 |

---

## Ontology Convergence

Stability 95% by Round 5 (final round). Remaining 5% ambiguity: M04+ entity additions (Healthcare patient, FinTech transaction, Connect Personal profile) deferred with user acknowledgment. No M01–M03 entity definitions contested.

---

## Interview Transcript Summary

**Round 0 — Topology**: Established oyatie ≡ Bominal parallel codebases. Confirmed brownfield status: codebase exists, naming convention migration in progress (Shard 1). Identified 5 top-level components.

**Round 1 — Architecture bisection**: Probed shared-vs-vertical binary. User clarified: "they are individual and modularized products that can be integrated with each other similar to how microservices integrate with each other in clean architecture." Established flat catalog principle. Identified Workflow + Ontology = adapter layer.

**Round 2 — Naming + glossary**: Probed Object Graph vs Ontology naming. User: "object graph = ontology at palantir" → rename to Ontology confirmed. Probed platform vs shared. User confirmed `shared` canonical; `platform` retired.

**Round 3 — BNF resolution**: Probed shared|vertical binary slot. User: "this means we can retire shared or vertical distinction" + "flat microservice structure." BNF v4.1 locked: `oya-<microservice>[-<bc>]-<layer>`.

**Round 4 — Connect rename + Bominal inheritance**: Probed workspace → connect rename. User confirmed. Established Bominal inheritance rule: default = inherit Bominal ADRs 1:1; session decisions override. 10 overrides catalogued.

**Round 5 — Milestones + quality bar**: Locked M01–M03 scope. User: "Our quality bar is industry leaders (with existing research benchmarked against competitors) and hyperscalers. Our quality and performance bar is high and must be horizontally scalable." Quality/perf/scalability constraints crystallized.

**Clarifications**: Shard 1 freeze window expedited ("no need to wait 48 hours"); M04+ explicitly deferred ("solidified in a follow-up planning session"); Corporate scope broader than HR/Payroll (per Bominal master-plan).

---

## ADRs Required (not yet authored)

| ADR | Title | Blocking |
|---|---|---|
| ADR-0055 | BNF v4.1 amendment (microservice slot; BC optional; flat catalog) | M01 Shard 1 |
| ADR-0056 | Cross-microservice isolation rule (Workflow + Ontology adapter plane) | M01 Shard 1 |
| ADR-0057 | Cutover mechanics v4.1 (Hybrid C; no freeze window; grit claim coordination) | M01 Shard 1 |
| ADR-0058 | KR-group launch bundle scope (M03) | M03 |
| ADR-0059 | Connect Professional legal-hold + eDiscovery contract | M03 |
| ADR-0060 | 4대보험 EDI integration | M03 |
| ADR-0061 | 연말정산 workflow | M03 |
