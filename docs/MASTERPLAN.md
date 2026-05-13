---
doc_class: MasterPlan
shape: anchor
length_cap: 800
authority_tier: 0
status: Accepted
date: 2026-05-13
owners: ["council-architecture"]
canonical_authority: docs/CONSTITUTION.md
companion_docs:
  - docs/PRD.md
  - docs/DESIGN.md
  - docs/ROADMAP.md
  - docs/RACI-OWNERSHIP.md
  - docs/RISK-REGISTER.md
  - docs/CHANGELOG.md
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class files > working drafts
foundation_adrs:
  - ADR-0052
  - ADR-0053
  - ADR-0054
  - ADR-0056
---

# Oyatie — MASTERPLAN

## §Authority-anchor

This is the canonical Master Plan for oyatie. All milestone INDEXes / phase INDEXes / Implementation Plans under `.omc/plans/milestones/M*/` derive their authority chain from this document and ultimately from `docs/CONSTITUTION.md`.

The planning implementation tree lives at `.omc/plans/milestones/`. `docs/MASTERPLAN.md` (this file) is the canonical product/architecture masterplan; `.omc/plans/MASTERPLAN.md` is deleted — this file is the single source.

---

> **Status:** Accepted (canonical at `docs/MASTERPLAN.md`).
> **Owner:** council-architecture (cross-axis); Founder Jason Lee (north-star arbiter).
> **Date:** 2026-05-13.
> **Supersedes:** pre-2026-05-13 masterplan (7-axis / vertical-grouping / platform-terminology model).

---

## 1. Vision

Oyatie is one cohesive **ecosystem-as-a-service**, expressed as a **flat catalog of independent µservices** that integrate via Workflow and Ontology — the two load-bearing adapter primitives. Any tenant enables any µservice subset à-la-carte. No grouping, no arms, no vertical privilege.

**Oyatie ≡ Bominal** — two parallel codebases of the same product family. Bominal ADR decisions are inherited 1:1 with glossary translation unless an explicit oyatie session override exists (see §5).

**Markets:** Korea-first + US parallel; EU after; jurisdiction-pluggable via regional pack seams.

**Operating posture:** No legacy protocols; own payment rails (M04+); complete-product-not-MVP; modular SME→enterprise; Bominal Proof Ladder L0..L7 + 9 architecture planes green at every milestone gate.

---

## 2. Architecture

### 2.1 Flat µservice catalog

There are no Product Groups, no Verticals, no Arms. Every µservice is independent and modular.

```
Foundry (internal-only engine)
  grit + icm + oya-tooling-agent-read + LEAN check binaries
  + Cedar + Wasmtime + Proof Ladder + 9 planes + Wave integration framework

Application (B2B unified shell — µservice)
  Tenants sign in; enable µservices à-la-carte (AWS-console model).

Flat catalog — customer-facing enable-able µservices (any tenant, any subset):
  medical, pharmacy, healthcare-portal, emergency, clinical
  hr, payroll, accounting, ats, grc, performance
  manufacturing, logistics, facility-ops, procurement, security
  payments, insurance, finance-quant
  connect (dual-context: messenger + mail + community)
  dining, cellar, …

Workflow µservice (cross-µservice action/orchestration adapter)
  State machines, DAGs, approvals, escalations, SLA timers, handoffs.
  Products publish typed events; Workflow routes them; consumers subscribe.

Ontology µservice (cross-µservice information adapter — Palantir Ontology equivalent)
  Typed Object Types + Link Types + Action Types + Functions.
  Audit-chain provenance, RLS-enforced tenant isolation, jurisdiction overlays.
  Bounded contexts: entity, link, action, function, agent-gateway, audit-chain, pillar.

Substrate µservices (always-on; underpin every other µservice):
  tenancy, identity, audit-chain, eventing, secrets,
  observability, kms, policy (Cedar), search, vector,
  data-boundary, finance-library, capability-registry,
  records (FHIR-canonical), ads, analytics

Cloud µservices (runtime substrate):
  cloud-tenancy, cloud-iam, cloud-kms, cloud-compute, cloud-storage,
  cloud-network, cloud-billing, cloud-cell, cloud-region,
  cloud-observability

Connect Personal (B2C entry path — separate from Application shell)
```

### 2.2 Workflow + Ontology = ecosystem adapter layer (load-bearing rule)

All inter-µservice integration flows through Workflow (action/orchestration) or Ontology (information/data). µservices never call each other directly. This is the central architectural invariant enforced by LEAN-A2 (cross-µservice refusal check).

### 2.3 BNF v4.1

```bnf
crate          ::= "oya" "-" microservice ( "-" bc-tokens )? "-" layer
microservice   ::= kebab-token ( "-" kebab-token )*    (* 1..3 tokens; registered in [workspace.metadata.oya.microservices] *)
bc-tokens      ::= kebab-token ( "-" kebab-token )*    (* 0..N; OPTIONAL *)
layer          ::= one of 12 canonical layer values per ADR-0056 §"Layer enum"
                   kernel | domain | application | app | adapter | infrastructure
                   cli | rest | grpc | graphql | worker | sdk
```

BC slot is OPTIONAL. Omit when the µservice has a single concept at the layer. Include when the µservice has multiple BC-level splits. Cross-cutting check crates: `oya-check-<rule-name>` (BNF-exempt).

Examples: `oya-medical-encounter-domain`, `oya-payments-ledger-application`, `oya-workflow-state-machine-domain`, `oya-ontology-entity-kernel`, `oya-foundry-grit-cli`, `oya-application-product-enablement-rest`, `oya-connect-messenger-grpc`.

### 2.4 Glossary (hard rules — no exceptions in docs, code, or plans)

| Retired term | Canonical term |
|---|---|
| platform (architectural) | shared or specific µservice name |
| Object Graph | Ontology |
| Shell / Modular Product Shell | Application |
| Workspace (µservice) | Connect |
| Vertical / Arm / Product Group | flat µservice catalog |
| shared\|vertical slot2 enum | µservice name (open kebab) |

Sales-segmentation labels (Healthcare / Enterprise / FinTech / Social) are GTM-only — NOT architecture. They do not appear in crate names, directory names, or architectural docs.

---

## 3. Inheritance posture (per ADR-0060)

Default: inherit Bominal ADR decisions 1:1 with glossary translation. Explicit oyatie overrides (higher precedence):

| # | Override | Oyatie decision |
|---|---|---|
| 1 | Workflow placement | shared µservice; `oya-workflow-*` |
| 2 | Object Graph naming | Renamed to **Ontology** |
| 3 | Platform glossary | `platform` retired; `shared` canonical |
| 4 | Vertical/Arm grouping | Flat µservice catalog; Arms retired |
| 5 | BNF `shared\|vertical` slot2 | µservice name (open kebab) — retired binary |
| 6 | Workspace product | Workspace → Connect dual-context |
| 7 | Shell terminology | Application (capital A) |
| 8 | Sales segmentation | GTM only — NOT architecture |
| 9 | Workflow+Ontology centrality | THE ecosystem adapter layer |

Inherited from Bominal 1:1 (with glossary translation): ADR-0011, ADR-0017–ADR-0021, ADR-0028, ADR-0100–ADR-0112, ADR-0116–ADR-0128, ADR-0132, ADR-0140, ADR-0208–ADR-0215, ADR-0223–ADR-0232.

---

## 4. Milestones

### M01 — v4 BNF cutover (IN FLIGHT)

**Scope:** Atomic rename of all `oya-platform-*` / `oya-shared-*` / `oya-workspace-*` crates to BNF v4.1 flat µservice names. Amend ADR-0056 to v4.1. Four LEAN check binaries green.

**Status:** Shard 0 landed (commit ec0aee3). Shard 1 queued (114-row TSV regen pending BNF v4.1 flag in xtask-metadata-augment).

**Exit criteria:** 114-row atomic rename merged + 26-row Shard 1.5 deferred rows resolved + 4 LEAN checks green on `main`.

**Phases:**
- P01 — Shard 0 ✓ (landed)
- P02 — Shard 1 atomic rename (114 rows; regenerate TSV with `--bnf-version v4.1`)
- P03 — Shard 1.5 deferred rows (26 rows)
- P04 — iter-4 src-inspection (BNF v4.1 compliance audit across all crates)
- P05 — Post-cutover hardening (LEAN checks flip from `--report-only` to BLOCKER)

### M02 — Substrate ready

**Scope:** Foundry engine + Cloud-Tenancy substrate + Ontology µservice + Workflow µservice + Application B2B shell + all substrate µservices (tenancy, identity, audit-chain, eventing, secrets, observability, kms, policy, search, vector, data-boundary, finance-library, capability-registry, records, ads, analytics).

**Exit criteria:** Sibling team scaffolds + ships any µservice via `grit claim/work/done` with zero build-team help; 9 architecture planes green at L4-L5; all `--report-only` lanes flipped to BLOCKER; Application deployable.

**Phases:** See §7 Implementation-Plan Index for full phase + IP breakdown.

### M03 — First-paying-tenant GA

**Scope** (= Bominal M3, per ADR-0210): Enterprise µservices (HR + Payroll + Accounting + broader Corporate per user scope — not merely payroll/HR SaaS) + Connect Professional (Mail + Messenger with legal hold + eDiscovery per Bominal ADR-0215) + Cloud-Tenancy substrate.

**Exit criteria:** 1 KR group paying tenant live; 4대보험 EDI; 연말정산; audit chain Merkle/Ed25519 segmented per (tenant_id, period); Connect Pro Mail with legal-hold/eDiscovery; Application enabling product subset.

**Phases:** See §7 Implementation-Plan Index.

### M04+ — DEFERRED

Per user instruction 2026-05-13: "missing some aspects later but we can solidify on our near term plans as is."

Deferred scope (to be planned in a follow-up session):
- Healthcare expansion (medical/pharmacy/portal/emergency/clinical) — 의료법, HIRA DUR, KFDA, NHIS, KHIRA, FHIR R5, HIPAA (US)
- FinTech expansion (payments/insurance/finance-quant) — 전자금융업, 간편결제, 인터넷전문은행, PCI DSS, KYC/AML, settlement
- Connect Personal context launch — crypto-audit + cold-start
- Industrial Suite (manufacturing/logistics/facility-ops/security per Bominal ADR-0011) with shared Workflow
- International expansion (US, EU jurisdictions per Bominal ADR-0140)

---

## 5. Known forthcoming regulatory gates (deferred to M04+ but visible)

- Payment: 전자금융업 → 간편결제 → 인터넷전문은행 (M05+)
- Healthcare KR: 의료법, HIRA DUR, KFDA, NHIS, KHIRA (M04)
- Healthcare US/EU: HIPAA, FHIR R5, GDPR (M04+)
- All compliance traits pluggable per Bominal ADR-0140

---

## 6. Operating model

| Concern | Canonical source |
|---|---|
| Proof Ladder L0..L7 | Bominal ADR-0223 |
| 9 architecture planes | Bominal ADR-0224..ADR-0231 |
| Wave integration framework | Bominal ADR-0232 |
| Sanctioned primitives | oyatie ADR-0053 (grit/icm/oya-tooling-agent-read) |
| Naming justification CI | feedback-naming-justification |
| Milestone > Phase > Impl-plan hierarchy | feedback-milestone-phase-hierarchy |
| 4 LEAN check binaries | oya-check-architecture-cli (pending Shard 1) |

**Compound principles:** Final-shape from day one (no MVP → rewrite); provider-agnostic by default (adapter crates only for provider-specific code); distroless + smallest-image containers; hyperscaler-bar engineering (Working Backwards / Design Doc / Postmortem / 1ES / Eng-Excellence); auto-doc + agentic-dev-optimized.

---

## 7. Tech stack

- **Rust** 1.82+; **PostgreSQL** 16 + Citus + RLS; **ClickHouse**; **TimescaleDB**; **Valkey**; **Kafka** KRaft
- **OpenBao** day-1 + HSM per-cell; **Istio** Ambient; **OpenTelemetry** + VictoriaMetrics
- **Wasmtime** + **Firecracker**; **pgroonga** + Tantivy; **pgvector**; Ed25519 + ML-DSA-87; **Cedar**; Typst
- **Trivy** + Cosign + SBOM + Kyverno; distroless containers
- **Clients:** Leptos web + 5 native (Win/Mac/Linux/iOS/Android) + SvelteKit prototype lane
- **Runtime:** OCI A1 → OKE stages (Bominal ADR-0117); on-prem capable; AWS-ready; no GCP/Azure

---

## 8. Quality and Performance Bar

### 8.1 Quality — Industry Leaders

Oyatie's quality bar is set by industry leaders (competitive-benchmarked) and hyperscalers (100M+ user scale). Horizontal scalability is mandatory from day one. No single-instance-only designs. No MVP-quality first releases — feature-complete or not shipped.

| Dimension | Reference standard |
|---|---|
| API design | Stripe (REST/gRPC contracts, idempotency, pagination, error model) |
| Data layer | Palantir Ontology (typed entities + provenance + audit) — Ontology µservice |
| UI/UX craft | Linear / Stripe / Superhuman (flat dual-mode surfaces) |
| Operational telemetry | Palantir Foundry-grade observability + on-call runbooks |
| Auth + identity | Auth0 / Okta capability parity + own-rails per Bominal ADR-0123 |
| Eventing | Confluent Kafka (KRaft) + Apache schema registry parity |
| Search | OpenSearch / Algolia parity; oyatie uses pgroonga + Tantivy |

Every PRD must include a **Competitive Benchmark** section naming the industry leader(s) the µservice targets parity with, listing quality dimensions benchmarked, and citing primary-source research.

### 8.2 Performance — Hyperscaler

| Dimension | Target |
|---|---|
| API p99 latency | ≤50ms read-only (Ontology Functions, per Bominal ADR-0107); ≤200ms write (Action Types) |
| Throughput | 10k+ req/sec per cell baseline; sharding to 100k+ aggregate via cell architecture |
| Concurrency | 100M+ users architecture (Bominal master-plan); cell-bounded blast-radius |
| Event lag | Sub-second propagation outbox → consumer |
| Audit chain | <1s segment-seal latency per (tenant, period) per Bominal ADR-0028 |
| Failover | RTO ≤30s per-cell; RPO ≤5s with outbox + cross-region replication |
| Cold start | ≤500ms per Bominal ADR-0020 multi-runtime standard |
| Tenant onboarding | ≤5min for self-serve SaaS path (Bominal ADR-0118) |

Every PRD must include a **Performance Targets** section with concrete p50/p99/p999 latency targets, throughput targets, error-budget allocation, and SLO burn-rate alarms. Every Implementation Plan must include a `## Load test` section with results meeting declared perf targets before merging to main.

### 8.3 Horizontal Scalability — Mandatory

| Requirement | Enforcement |
|---|---|
| Stateless services | Required for all `application` / `rest` / `grpc` / `graphql` / `worker` layer crates. State lives only in adapter+infrastructure. Enforced by `oya-check-statelessness-cli` (TBD M02-P09). |
| Sharded state | Postgres + Citus per Bominal ADR-0117; ClickHouse + replicas; Valkey cluster. Single-DB-only designs fail `oya-check-shardability-cli` (TBD M02-P09). |
| Event-driven | Outbox → Kafka KRaft. Direct synchronous cross-µservice calls require ADR justification. |
| Cell architecture | All tenant-bound state partitioned per (cell, region); per Bominal ADR-0009 + oyatie ADR-0009. |
| Active-active capable | All `worker` + `adapter` layers declare `active_active_compatibility` per Bominal ADR-0019. |
| Cross-region replication | Required for high-consequence µservices (medical, payments, connect-pro). Per Bominal ADR-0049. |

Every PRD must include a **Horizontal Scalability** section declaring state strategy, active-active compatibility, per-cell capacity envelope, scale-out trigger metrics, and cross-region story.

New CI fitness lanes (authored in M02-P09):
- `oya-check-statelessness-cli` — presentation/application/worker layers have no module-level mutable state
- `oya-check-shardability-cli` — DB designs declare tenant_id partition key + row-level isolation
- `oya-check-perf-budget-cli` — impl plans include load-test results meeting declared perf targets
- `oya-check-benchmark-cli` — PRDs include competitive-benchmark section before µservice graduates Proof-Ladder L4→L5

---

## 9. Industry Competitive Map

| µservice cluster | Oyatie competitive references |
|---|---|
| HR / Payroll | 더존비즈온, ADP, Workday, SAP SuccessFactors |
| Accounting / Finance | 더존 iCUBE, NetSuite, Xero |
| Healthcare (medical/pharmacy/clinical) | 유비케어, 비트컴퓨터, Epic Systems, Cerner |
| FinTech (payments/banking) | Stripe, 토스, 카카오뱅크, 케이뱅크 |
| Connect (messenger/mail/community) | Slack, Gmail (Google Workspace), Signal, Notion |
| Search | Algolia, Elasticsearch/OpenSearch, Naver Search |
| Ontology (data layer) | Palantir Foundry (Ontology + Object Graph) |
| Cloud substrate | AWS, OCI, GCP |
| Identity | Auth0, Okta, Keycloak |
| Eventing | Confluent Kafka, Apache Pulsar |
| Workflow | Temporal, Camunda, AWS Step Functions |

Each µservice PRD lists the competitor set, the specific benchmark dimensions, and the primary-source evidence. Quality parity is a gate on Proof Ladder L4→L5 graduation.

---

## 10. Sales segmentation (GTM only — NOT architecture)

| Label | GTM bucket for |
|---|---|
| Healthcare (의료) | medical / pharmacy / portal / emergency / clinical µservices |
| Enterprise (기업) | hr / payroll / accounting / manufacturing / logistics / facility-ops / procurement / security / grc / ats |
| FinTech (금융) | payments / insurance / finance-quant µservices |
| Social | connect dual-context µservice + future social-graph µservices |

---

## 11. Risk register (top items)

| ID | Description | Prob | Impact | Owner |
|---|---|---|---|---|
| RM-01 | Cross-µservice contract drift | High | High | council-architecture |
| RM-02 | Tenant data leak via PHI/PII into search/ads | Med | Catastrophic | council-privacy |
| RM-03 | Agent runtime escapes autonomy ceiling | Med | Catastrophic | axis-foundry |
| RM-04 | BNF v4.1 rename breaks `main` | Med | High | council-architecture |
| RM-05 | Workflow or Ontology adapter boundary violated | Med | High | council-architecture |

Full register: `docs/RISK-REGISTER.md`.

---

## 12. RACI summary

| Milestone | Responsible | Accountable |
|---|---|---|
| M01 | axis-foundry (rename execution) | council-architecture |
| M02 | platform-substrate + axis-foundry | council-architecture |
| M03 | axis-enterprise + axis-connect + axis-cloud | council-architecture + gtm-customer-success |
| M-CC | per-phase owner | council-architecture |

Full RACI: `docs/RACI-OWNERSHIP.md`.

---

## 13. Implementation-Plan Index

This section lists every (Milestone, Phase, Impl-Plan) tuple. Files marked **[EXISTS]** are already authored under `.omc/plans/milestones/`. Files marked **[TBD]** need authoring in a Wave 2 planning session — the path is the canonical target location.

### M-CC — Cross-cutting workstreams

| Phase | Phase path | Impl Plan | Status |
|---|---|---|---|
| P01 agentic-pipeline-cutover | `.omc/plans/milestones/M-CC-cross-cutting/phases/P01-agentic-pipeline-cutover/` | IP-001-adr-0054-scaffold-claim.md | **[EXISTS]** |
| P01 | | IP-002-inventory-adr-0052.md | **[EXISTS]** |
| P01 | | IP-003-oya-tooling-agent-read.md | **[EXISTS]** |
| P01 | | IP-004-bidirectional-prd-cite.md | **[EXISTS]** |
| P01 | | IP-005-foundry-corpus-cross-cite.md | **[EXISTS]** |
| P01 | | IP-006-agent-facing-memory.md | **[EXISTS]** |
| P01 | | IP-007-hook-skill-audit.md | **[EXISTS]** |
| P01 | | IP-008-archive-glue.md | **[EXISTS]** |
| P01 | | IP-009-delete-active-path.md | **[EXISTS]** |
| P01 | | IP-010-parallel-claim-demo.md | **[EXISTS]** |
| P01 | | IP-011-upstream-grit-bug.md | **[EXISTS]** |
| P01 | | IP-012-authoritative-tracked-audit.md | **[EXISTS]** |
| P02 doc-automation-freshness | `.omc/plans/milestones/M-CC-cross-cutting/phases/P02-doc-automation-freshness/` | IP-001-mdbook-pipeline.md | **[EXISTS]** |
| P02 | | IP-002-doc-freshness-lane.md | **[EXISTS]** |
| P02 | | IP-003-doc-style-lane.md | **[EXISTS]** |
| P03 purpose-orphan-detection | `.omc/plans/milestones/M-CC-cross-cutting/phases/P03-purpose-orphan-detection/` | IP-001-purpose-frontmatter-audit.md | **[EXISTS]** |
| P03 | | IP-002-orphan-detection-lane.md | **[EXISTS]** |
| P04 agentic-navigability | `.omc/plans/milestones/M-CC-cross-cutting/phases/P04-agentic-navigability/` | IP-001-navigability-lane.md | **[EXISTS]** |
| P04 | | IP-002-predictable-naming.md | **[EXISTS]** |
| P05 provider-agnosticism | `.omc/plans/milestones/M-CC-cross-cutting/phases/P05-provider-agnosticism/` | IP-001-provider-coupling-lane.md | **[EXISTS]** |
| P05 | | IP-002-cloud-multi-provider-audit.md | **[EXISTS]** |
| P05 | | IP-003-adapter-substitution-harness.md | **[EXISTS]** |
| P06 distroless-lts-image | `.omc/plans/milestones/M-CC-cross-cutting/phases/P06-distroless-lts-image/` | IP-001-distroless-image-lane.md | **[EXISTS]** |
| P06 | | IP-002-lts-dependency-lane.md | **[EXISTS]** |
| P06 | | IP-003-static-musl-build.md | **[EXISTS]** |
| P07 hyperscaler-practices | `.omc/plans/milestones/M-CC-cross-cutting/phases/P07-hyperscaler-practices/` | IP-001-prfaq-designdoc-postmortem.md | **[EXISTS]** |
| P07 | | IP-002-1es-ci-templates.md | **[EXISTS]** |
| P07 | | IP-003-eng-excellence-merge-gate.md | **[EXISTS]** |
| P07 | | IP-004-rust-toolchain-gates.md | **[EXISTS]** |
| P08 supply-chain-security | `.omc/plans/milestones/M-CC-cross-cutting/phases/P08-supply-chain-security/` | IP-001-cosign-rekor.md | **[EXISTS]** |
| P08 | | IP-002-sbom-pipeline.md | **[EXISTS]** |
| P08 | | IP-003-license-policy-lane.md | **[EXISTS]** |
| P08 | | IP-004-slsa-attestation.md | **[EXISTS]** |
| P09 visualization-as-code | `.omc/plans/milestones/M-CC-cross-cutting/phases/P09-visualization-as-code/` | IP-001-architecture-map-walkers.md | **[EXISTS]** |
| P09 | | IP-002-mermaid-d2-graphviz-emitters.md | **[EXISTS]** |
| P09 | | IP-003-mdbook-publish-integration.md | **[EXISTS]** |
| P09 | | IP-004-architecture-map-freshness-lane.md | **[EXISTS]** |

### M01 — v4 BNF cutover

| Phase | Phase path | Impl Plan | Status |
|---|---|---|---|
| P01 data-use-boundary-tenancy | `.omc/plans/milestones/M01-foundation/phases/P01-data-use-boundary-tenancy/` | IP-001-data-use-boundary-adr.md | **[EXISTS]** |
| P01 | | IP-002-tenant-kernel-contracts.md | **[EXISTS]** |
| P01 | | IP-003-dsr-cascade-engine.md | **[EXISTS]** |
| P02 identity-cedar | `.omc/plans/milestones/M01-foundation/phases/P02-identity-cedar/` | IP-001-identity-kernel.md | **[EXISTS]** |
| P02 | | IP-002-sts-rotation.md | **[EXISTS]** |
| P02 | | IP-003-cedar-policy-substrate.md | **[EXISTS]** |
| P03 audit-chain-evidence | `.omc/plans/milestones/M01-foundation/phases/P03-audit-chain-evidence/` | IP-001-merkle-ed25519-kernel.md | **[EXISTS]** |
| P03 | | IP-002-audit-asyncapi-proto.md | **[EXISTS]** |
| P03 | | IP-003-tamper-evidence-drill.md | **[EXISTS]** |
| P04 eventing-ontology | `.omc/plans/milestones/M01-foundation/phases/P04-eventing-object-graph/` | IP-001-outbox-topic-registry.md | **[EXISTS]** |
| P04 | | IP-002-object-graph-property-tiers.md | **[EXISTS]** (note: "object-graph" slug is legacy; content = Ontology) |
| P04 | | IP-003-eventing-adapters.md | **[EXISTS]** |
| P05 cell-plane | `.omc/plans/milestones/M01-foundation/phases/P05-cell-plane/` | IP-001-cell-routing-primitive.md | **[EXISTS]** |
| P05 | | IP-002-plane-separation-lane.md | **[EXISTS]** |
| P06 regional-pack-flattening | `.omc/plans/milestones/M01-foundation/phases/P06-regional-pack-flattening/` | IP-001-regional-pack-adr-kernel.md | **[EXISTS]** |
| P06 | | IP-002-flat-crates-guard.md | **[EXISTS]** |
| **P-Shard1** BNF-v4.1-rename | `.omc/plans/milestones/M01-foundation/phases/P-shard1-bnf-rename/` | IP-001-tsv-regen-v4.1.md | **[TBD]** |
| P-Shard1 | | IP-002-atomic-rename-114-rows.md | **[TBD]** |
| P-Shard1 | | IP-003-shard-1.5-deferred-26-rows.md | **[TBD]** |
| P-Shard1 | | IP-004-iter4-src-inspection.md | **[TBD]** |
| P-Shard1 | | IP-005-lean-checks-blocker-flip.md | **[TBD]** |

### M02 — Substrate ready

| Phase | Phase path | Impl Plan | Status |
|---|---|---|---|
| P00 account-auth | `.omc/plans/milestones/M02-foundry-preview/phases/P00-account-auth/` | IP-001-clean-arch-skeleton.md | **[EXISTS]** |
| P00 | | IP-002-domain-types-state-machine.md | **[EXISTS]** |
| P00 | | IP-003-secret-store-port.md | **[EXISTS]** |
| P01 provider-gateway | `.omc/plans/milestones/M02-foundry-preview/phases/P01-provider-gateway/` | IP-001-anthropic-adapter.md | **[EXISTS]** |
| P01 | | IP-002-openai-adapter.md | **[EXISTS]** |
| P01 | | IP-003-gemini-adapter.md | **[EXISTS]** |
| P01 | | IP-004-usage-window-route-policy.md | **[EXISTS]** |
| P02 multi-subscription-pool | `.omc/plans/milestones/M02-foundry-preview/phases/P02-multi-subscription-pool/` | IP-001-provider-account-pool-kernel.md | **[EXISTS]** |
| P02 | | IP-002-anthropic-compat-adapter.md | **[EXISTS]** |
| P02 | | IP-003-openai-compat-adapter.md | **[EXISTS]** |
| P02 | | IP-004-oauth-subscription-capture.md | **[EXISTS]** |
| P02 | | IP-005-upstream-api-drift-lane.md | **[EXISTS]** |
| P02 | | IP-006-tos-policy-audit-chain.md | **[EXISTS]** |
| P02-vis visibility-operator-plane | `.omc/plans/milestones/M02-foundry-preview/phases/P02-visibility-operator-plane/` | IP-001-readonly-api-kernel.md | **[EXISTS]** |
| P02-vis | | IP-002-dashboard-svelte.md | **[EXISTS]** |
| P02-vis | | IP-003-dry-run-surface.md | **[EXISTS]** |
| P03 gates-validators-evidence | `.omc/plans/milestones/M02-foundry-preview/phases/P03-gates-validators-evidence/` | IP-001-phase00-evidence-validator.md | **[EXISTS]** |
| P03 | | IP-002-foundry-fitness-lane-ratchet.md | **[EXISTS]** |
| P03 | | IP-003-adr-template-bypass-ledger.md | **[EXISTS]** |
| P04 transport-parity-write-gates | `.omc/plans/milestones/M02-foundry-preview/phases/P04-transport-parity-write-gates/` | IP-001-rest-graphql-transports.md | **[EXISTS]** |
| P04 | | IP-002-sse-websocket-transports.md | **[EXISTS]** |
| P04 | | IP-003-write-gate-foundations.md | **[EXISTS]** |
| P05 capability-registry-autonomy | `.omc/plans/milestones/M02-foundry-preview/phases/P05-capability-registry-autonomy/` | IP-001-capability-registry.md | **[EXISTS]** |
| P05 | | IP-002-autonomy-ceiling.md | **[EXISTS]** |
| P05 | | IP-003-rag-endpoint.md | **[EXISTS]** |
| **P06** ontology-µservice | `.omc/plans/milestones/M02-foundry-preview/phases/P06-ontology-microservice/` | IP-001-ontology-entity-link-kernel.md | **[TBD]** |
| P06 | | IP-002-ontology-action-function-kernel.md | **[TBD]** |
| P06 | | IP-003-ontology-agent-gateway.md | **[TBD]** |
| P06 | | IP-004-ontology-rls-audit-chain.md | **[TBD]** |
| **P07** workflow-µservice | `.omc/plans/milestones/M02-foundry-preview/phases/P07-workflow-microservice/` | IP-001-workflow-state-machine-domain.md | **[TBD]** |
| P07 | | IP-002-workflow-approvals-escalations.md | **[TBD]** |
| P07 | | IP-003-workflow-sla-automation.md | **[TBD]** |
| P07 | | IP-004-workflow-adapter-kafka.md | **[TBD]** |
| **P08** application-shell | `.omc/plans/milestones/M02-foundry-preview/phases/P08-application-shell/` | IP-001-application-product-enablement-api.md | **[TBD]** |
| P08 | | IP-002-application-tenant-onboarding-flow.md | **[TBD]** |
| P08 | | IP-003-application-capability-menu.md | **[TBD]** |
| **P09** substrate-µservices | `.omc/plans/milestones/M02-foundry-preview/phases/P09-substrate-microservices/` | IP-001-search-vector-substrate.md | **[TBD]** |
| P09 | | IP-002-finance-library-capability-registry.md | **[TBD]** |
| P09 | | IP-003-records-data-boundary.md | **[TBD]** |
| P09 | | IP-004-ads-analytics-substrate.md | **[TBD]** |

### M03 — First-paying-tenant GA

| Phase | Phase path | Impl Plan | Status |
|---|---|---|---|
| P01 cloud-foundations | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P01-cloud-foundations/` | IP-001-kms-api-adapters.md | **[EXISTS]** |
| P01 | | IP-002-storage-api-adapters.md | **[EXISTS]** |
| P01 | | IP-003-network-api-adapters.md | **[EXISTS]** |
| P01 | | IP-004-iam-cedar-sso-sts.md | **[EXISTS]** |
| P01 | | IP-005-region-az-cell-taxonomy.md | **[EXISTS]** |
| P02 cloud-compute | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P02-cloud-compute/` | IP-001-vm-api-adapters.md | **[EXISTS]** |
| P02 | | IP-002-k8s-functions-api.md | **[EXISTS]** |
| P02 | | IP-003-capacity-management.md | **[EXISTS]** |
| P03 cloud-data-billing-observability | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P03-cloud-data-billing-observability/` | IP-001-cloud-data-adapters.md | **[EXISTS]** |
| P03 | | IP-002-billing-tax-metering.md | **[EXISTS]** |
| P03 | | IP-003-observability-otel.md | **[EXISTS]** |
| P03 | | IP-004-finops-report.md | **[EXISTS]** |
| P03 | | IP-005-marketplace-isv.md | **[EXISTS]** |
| P04 saas-platform-preview | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P04-saas-platform-preview/` | IP-001-workflow-engine.md | **[EXISTS]** |
| P04 | | IP-002-plugin-substrate.md | **[EXISTS]** |
| P04 | | IP-003-marketplace-listing.md | **[EXISTS]** |
| P05 search-preview | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P05-search-preview/` | IP-001-pgroonga-morphology.md | **[EXISTS]** |
| P05 | | IP-002-pgvector-tenant-private.md | **[EXISTS]** |
| P05 | | IP-003-rag-endpoint-data-boundary.md | **[EXISTS]** |
| P06 workspace-14-surfaces | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P06-workspace-14-surfaces/` | IP-001-mail-calendar.md | **[EXISTS]** |
| P06 | | IP-002-docs-sheets-slides-sites.md | **[EXISTS]** |
| P06 | | IP-003-drive-kms-shred.md | **[EXISTS]** |
| P06 | | IP-004-meet-chat-recordings.md | **[EXISTS]** |
| P06 | | IP-005-forms-address-tasks-notes-translate.md | **[EXISTS]** |
| P07 regional-pack-onboarding | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P07-regional-pack-onboarding/` | IP-001-kr-pack.md | **[EXISTS]** |
| P07 | | IP-002-second-pack.md | **[EXISTS]** |
| P08 cross-axis-contracts | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P08-cross-axis-contracts/` | IP-001-saas-pairs.md | **[EXISTS]** |
| P08 | | IP-002-cloud-pairs.md | **[EXISTS]** |
| P08 | | IP-003-search-ads-pairs.md | **[EXISTS]** |
| P08 | | IP-004-vertical-workspace-pairs.md | **[EXISTS]** |
| **P09** enterprise-µservices-hr-payroll | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P09-enterprise-hr-payroll/` | IP-001-hr-domain-kernel.md | **[TBD]** |
| P09 | | IP-002-payroll-4대보험-edi.md | **[TBD]** |
| P09 | | IP-003-payroll-연말정산.md | **[TBD]** |
| P09 | | IP-004-accounting-domain-kernel.md | **[TBD]** |
| **P10** connect-professional | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P10-connect-professional/` | IP-001-connect-mail-legal-hold.md | **[TBD]** |
| P10 | | IP-002-connect-messenger-ediscovery.md | **[TBD]** |
| P10 | | IP-003-connect-dual-context-boundary.md | **[TBD]** |
| **P11** audit-chain-tenant-segmentation | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P11-audit-chain-tenant-segmentation/` | IP-001-merkle-segmented-per-tenant-period.md | **[TBD]** |
| P11 | | IP-002-ed25519-signing-rotation.md | **[TBD]** |
| **P12** first-paying-tenant-onboarding | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P12-first-paying-tenant-onboarding/` | IP-001-kr-group-tenant-onboarding.md | **[TBD]** |
| P12 | | IP-002-go-live-evidence-pack.md | **[TBD]** |

### M04 — Vertical-Pilot Korea (pre-2026-05-13 plan; may need refresh after M04+ scoping session)

| Phase | Phase path | Impl Plan | Status |
|---|---|---|---|
| P01 vertical-capability-pack | `.omc/plans/milestones/M04-vertical-pilot-korea/phases/P01-vertical-capability-pack/` | IP-001-council-resolution.md | **[EXISTS]** |
| P01 | | IP-002-capability-pack-kernel.md | **[EXISTS]** |
| P01 | | IP-003-vertical-workflows.md | **[EXISTS]** |
| P02 kr-regulatory-binding | `.omc/plans/milestones/M04-vertical-pilot-korea/phases/P02-kr-regulatory-binding/` | IP-001-pipa-csap-evidence.md | **[EXISTS]** |
| P02 | | IP-002-isms-p-kcmvp-hsm.md | **[EXISTS]** |
| P02 | | IP-003-kr-vertical-surfaces.md | **[EXISTS]** |
| P03 design-partner-onboarding | `.omc/plans/milestones/M04-vertical-pilot-korea/phases/P03-design-partner-onboarding/` | IP-001-tenant-onboarding.md | **[EXISTS]** |
| P03 | | IP-002-tenant-workflows.md | **[EXISTS]** |
| P03 | | IP-003-foundry-agents-activation.md | **[EXISTS]** |
| P04 evidence-retention-audit | `.omc/plans/milestones/M04-vertical-pilot-korea/phases/P04-evidence-retention-audit/` | IP-001-evidence-pipeline.md | **[EXISTS]** |
| P04 | | IP-002-retention-kpi.md | **[EXISTS]** |
| P04 | | IP-003-audit-pack-generator.md | **[EXISTS]** |

---

## 14. References

- Memory files: `~/.claude/projects/-Users-jasonlee-oyatie/memory/MEMORY.md`
- ADRs: `docs/decisions/ADR-*.md` (especially ADR-0056 v4.1, ADR-0058..0061 overrides)
- Bominal cross-reference: `/Users/jasonlee/bominal/decisions/` and `/Users/jasonlee/bominal/docs/`
- Planning tree: `.omc/plans/milestones/`

---

## 15. Status footer

Status: **Accepted** (canonical at `docs/MASTERPLAN.md`).
Iteration: 4 — full rewrite 2026-05-13 per /deep-interview session consensus. Adopts flat µservice catalog, BNF v4.1, Ontology/Workflow adapter layer, Bominal inheritance posture, M01-M03 phase+IP index, M04+ deferred per user instruction.
