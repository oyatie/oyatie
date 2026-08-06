---
id: ADR-700
title: "Live CI admission, build hermeticity, and runner substrate"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-06
door: two-way
owner: council-architecture
supersedes: [ADR-9, ADR-10, ADR-21, ADR-28, ADR-29, ADR-30, ADR-31, ADR-35, ADR-40, ADR-44, ADR-47, ADR-56, ADR-83, ADR-91, ADR-94, ADR-99, ADR-104, ADR-117, ADR-118, ADR-123, ADR-128, ADR-129, ADR-135, ADR-139, ADR-148, ADR-157, ADR-162, ADR-167, ADR-175, ADR-182, ADR-185, ADR-190, ADR-194, ADR-204, ADR-213, ADR-215, ADR-234, ADR-243, ADR-248, ADR-273, ADR-295, ADR-297, ADR-303, ADR-305, ADR-309, ADR-312, ADR-313, ADR-325, ADR-328, ADR-334, ADR-340, ADR-341, ADR-346, ADR-348, ADR-360, ADR-366, ADR-367, ADR-369, ADR-373, ADR-374, ADR-380, ADR-383, ADR-392, ADR-515, ADR-517, ADR-519, ADR-521, ADR-522, ADR-523, ADR-524, ADR-525, ADR-526, ADR-527, ADR-528, ADR-529, ADR-530, ADR-531, ADR-533, ADR-534, ADR-535, ADR-536, ADR-537, ADR-538, ADR-539, ADR-540, ADR-544, ADR-545, ADR-546, ADR-547, ADR-549, ADR-551, ADR-554, ADR-559, ADR-560, ADR-563, ADR-565, ADR-566, ADR-567, ADR-570, ADR-581, ADR-582, ADR-586, ADR-587, ADR-588, ADR-590, ADR-595, ADR-597, ADR-600, ADR-605, ADR-606, ADR-608, ADR-609, ADR-612, ADR-613, ADR-616, ADR-618, ADR-624, ADR-627, ADR-628, ADR-629, ADR-630, ADR-631, ADR-633, ADR-634, ADR-636, ADR-639]
superseded_by: []
amends: []
amended_by: []
depends_on: []
related: []
milestone: W0
---
# ADR-700: Live CI admission, build hermeticity, and runner substrate

## Status

**Accepted** — live consolidated source-of-truth entry for topic `ci_admission` (E5 2026-08-06).

## Context

Oyatie ADR corpus cleanup: agents must not treat every historical Accepted file as equal live law.
This apex consolidates **126** Accepted ADRs in the `ci_admission` topic. Member files are
**Superseded** by this apex and then archived; full text remains in git history.

Live resolution: prefer this apex; follow `supersedes` for provenance.

## Decision

### Live hard norms (restated from ADR-0515 lineage — full text in archive)

1. **Single required admission context:** `oya-ci-required` is the sole protected merge context (no dual CI authority).
2. **Cloud-native gates:** binding verification is Rust/Buck2 gate apps; legacy shell/CLI is bridge feedback only.
3. **No dual-authority CI:** GitHub Actions is transitional adapter; no re-introduction of Prow/Jenkins as merge authority.
4. **Warm CAS / RE activation:** fail-closed until explicit go-gate (credentials #1541, cache-only proof, Accepted activation ADR). Apex gists that mention `remote_enabled=true` are **historical design**, not activation authority.
5. **Generated faces:** never hand-edit `*.generated.json`; materialize via sanctioned producers.

1. **This ADR is the live reading entry** for topic `ci_admission` under the end-state ADR policy.
2. **Member ADRs listed in `supersedes`** are historical; normative gist is preserved below.
3. **Contradictions** among members are resolved by later higher-number members and by
   ADR-0515 / ADR-0363 / ADR-0562 / ADR-0615 / ADR-0635 / ADR-0637–0639 when applicable.
4. **Activation-sensitive** items (warm CAS, RE workers) remain fail-closed until explicit go-gate.

## Preserved member gists

- **ADR-9** (ADR-0009-cell-architecture-per-tenant-per-region): We adopt **cells as the primary blast-radius isolation primitive**, sized per-tenant per-region with five cell tiers, cell-routing primitives at edge / mesh / store / event layers, and per-cell HSM partitions. Cell-isolation evidence is collected quarterly per regulatory pack. ### Cell sizing tiers | Tier | Reads as | Tenant count | Use case | |---
- **ADR-10** (ADR-0010-regional-pack-architecture): We adopt **canonical-architecture + regional-pack plug-ins** as the locale model. The architecture is locale-agnostic; every per-locale concern lives in a regional pack that plugs into published seams. One pack per market, versioned and signed. ### Pack contents (per pack) | Pack section | What's inside | |---|---| | `regulatory` | Regulator names,
- **ADR-21** (ADR-0021-intelligence-capability-registry-and-mcp-gateway): We define the canonical `Capability` record in `oya-intelligence-capability-kernel` and serve it via an MCP-compatible gateway that exposes a per-tenant endpoint. The catalog YAML in `registry/catalog/` is the source of truth; the kernel projects it into typed records at runtime. ### Capability primitive (`oya-intelligence-capability-kernel`) ```ru
- **ADR-28** (ADR-0028-cloud-microservice-architecture): We adopt a **three-phase compute trajectory** with a **phase-invariant product surface**. Customers consume the same APIs, the same SKUs, the same IAM model, and the same audit shape regardless of whether the underlying capacity is rented, leased in a colo, or owned in a mega-DC. **Naming justification (BNF v4.1, ADR-0056):** - Cloud µservice crate
- **ADR-29** (ADR-0029-connect-dual-context-architecture): We adopt as a **suite of twelve canonical apps** plus three adjunct surfaces, each its own bounded context under `oya-connect-<app>-*`, sharing the six substrates from ADR-0001 plus a Connect-internal **document-format kernel** and **collab-runtime kernel**. **Naming justification (BNF v4.1, ADR-0056):** - `oya-mail-kernel`: slot2 = `connector` (re
- **ADR-30** (ADR-0030-search-microservice-architecture): We adopt a **five-stage Search architecture** — Crawler → Parser → Indexer → Ranker → SERP — plus three cross-cutting subsystems (Query Understanding, Safety, Search↔Foundry/Ads bridges). Each stage is its own bounded context under `oya-search-<stage>-*`. Per-tier index segregation is enforced at the Indexer layer; cross-tier query is forbidden by 
- **ADR-31** (ADR-0031-ads-and-analytics-microservice-architecture): We adopt a **singleton tenant-ads-gate sourcing rule** plus a **five-pillar Ads architecture** (Serving / Pricing / Attribution / Advertiser console / Publisher inventory) plus a **DP-budgeted Analytics architecture**. **Naming justification (BNF v4.1, ADR-0056):** - `oya-ads-gate-kernel`: slot2 = `ads` (registered µservice); slot3 = `gate` (BC); s
- **ADR-35** (ADR-0035-workflow-engine-state-machine-and-dag-hybrid): We build `crates/oya-workflow-*` as the canonical workflow engine for the entire ecosystem. The engine is a **hybrid state-machine + DAG**: at the top level, every workflow is a state machine; within each state, computation can be expressed as a DAG. Per-tenant workflow definition versioning is first-class; per-jurisdiction overlays bind at runtime
- **ADR-40** (ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback): We adopt **Argo Rollouts** as the canonical progressive-delivery controller; **canary 5% → 25% → 50% → 100%** as the default stage progression; **metric-gated rollback** at SLO 1h burn-rate ≥ 14.4× (Sev-1-class trigger); **blue-green** for stateful surfaces; **per-region phased rollout** as the geographic progression; **per-cell rollback** as the u
- **ADR-44** (ADR-0044-service-mesh-istio-ambient-and-envoy-gateway): We adopt **Istio Ambient mode** as the canonical east-west service mesh; **Envoy** (gateway-class) as the canonical north-south edge gateway; **mTLS everywhere** as the default with per-traffic-type opt-out only via ADR; **per-cell namespace** as the isolation unit; **cross-cell traffic** explicit + Cedar-policied + audit-chained per call. ### Isti
- **ADR-47** (ADR-0047-search-backend-strategy): We adopt **pgroonga** day-1 with **legal isolation** per License Policy ADR + replacement plan; **Tantivy** (MIT) in-Rust at scale; **OpenSearch** (Apache-2) only as an adapter behind a port; **Elasticsearch SSPL forbidden** in product surface; **in-house long-horizon** (KR morphology + Tantivy + custom ranker) under `crates/oya-search-backend-*`. 
- **ADR-56** (ADR-0056-rust-clean-architecture-bnf): ### Canonical BNF v4.1 ```bnf crate ::= "oya" "-" microservice ( "-" bc-tokens )? "-" layer | "oya" "-" "check" "-" rule-name microservice ::= kebab-token ( "-" kebab-token )* (* 1..3 tokens; registry-validated *) bc-tokens ::= kebab-token ( "-" kebab-token )* (* 0..N; OPTIONAL *) layer ::= "kernel" | "domain" | "usecase" | "app" | "adapter" | "inf
- **ADR-83** (ADR-0083-rust-error-handling-tier-decision): We adopt a **three-tier** error-handling policy applied uniformly across every `oya-*` Rust crate. Per-tier rules below are normative (RFC-2119 keywords as defined in docs/standards/error-handling.md§1). ### Tier 1 — Library crates (kernel / domain / app / adapter / api / worker / infrastructure / service / rest / cli / bindings) - Public errors **
- **ADR-91** (ADR-0091-governance-write-gate-foundations): The write-gate kernel owns the canonical write-gate state machine: ``` Proposed → Reviewed { reviewer } → Approved { approver } → Executed \ / +----------> Rejected { reason } <--------------+ ``` Linear forward path: `Proposed → Reviewed → Approved → Executed`. Any non-terminal state may transition to `Rejected`. `Executed` and `Rejected` are term
- **ADR-94** (ADR-0094-handler-trait-with-associated-error): Add a typed `Handler` trait in `oya-http-middleware-kernel`: ```rust pub trait Handler: Send + Sync { type Error: Into<HttpResponse>; fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error>; } pub fn call_into_response<H: Handler>(handler: &H, req: HttpRequest) -> HttpResponse { match handler.call(req) { Ok(r) => r, Err(e) => e.into()
- **ADR-99** (Cedar policy extension — foundry supervisor capabilities in docs/policies/foundr): Add `docs/policies/foundry-supervisor.cedar` containing tier-gated policies for the five supervisor capabilities. The file is created by Wave 4b (Task #12); this ADR records the design. ### Policy design Autonomy tiers (from ADR-0007): | Tier | Label | Principal class | |---|---|---| | T1 | `read-only-observer` | Monitoring/observability systems, r
- **ADR-104** (Ecosystem-expansion principle for check-lane + adapter crate reintroduction): **Ecosystem-expansion rule.** A crate is shipped iff: 1. The kernel/domain layer it implements is itself shipped, AND 2. At least one consumer in the workspace imports it, AND 3. The crate has a real implementation (not a doc-stub). If any condition fails, the crate is not shipped. Documentation of the trigger that would unblock the crate lives in 
- **ADR-117** (ADR-0117-repo-hygiene-gitignore-audit-config-and-kyverno-consolidation): 1. Add `.audit/` to `.gitignore` and untrack `.audit/agent-read.jsonl` via `git rm --cached`. Session-scoped audit logs stay local-only. Keep `.config/nextest.toml` tracked because it is CI configuration, not per-developer config. 2. `git mv deploy/gitops/oya-vcs-admission infra/kyverno/oya-vcs-admission` (history-preserving), removing the now-empt
- **ADR-118** (ADR-0118-retire-archive-orphan-fitness-lane): Retire `archive-orphan` as an executable fitness lane. The retirement removes: - `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/` - `crates/oya-governance-archive-orphan-kernel` - `tools/oya-governance-archive-orphan-app` - workspace members for both retired crates - catalog entries for the retired kernel/app capability The retiremen
- **ADR-123** (ADR-0123-hyperscaler-maturity-claim-gate): Use `/specs/hyperscaler-gates.json` as the machine-readable maturity claim registry. The exact phrase "we are hyperscaler mature" is forbidden unless the registry claim rule is allowed and all required gates have fresh evidence. Add the repo-native gate: ```text oya gate validate hyperscaler-maturity-claims ``` The gate validates: - Required maturi
- **ADR-128** (ADR-0128-hyperscaler-architecture-invariants): `specs/hyperscaler-architecture-invariants.json` (spec_id: EXE-HYPERSCALER-ARCH-INVARIANTS, version 1.0.0) is the canonical, machine-readable, binding source of truth for what "hyperscaler-grade" means in the Oyatie portfolio. This PR lands the catalog validator; it does not claim that product PRDs are already blocked on the catalog. Binding rules:
- **ADR-129** (ADR-0129-changeset-plan-dag-and-honest-claims-gate): The existing ImplementationPlan frontmatter `id` is the canonical ChangeSet ID. No separate `changeset_id` field is introduced. The validator treats these fields as the exact ChangeSet graph contract: | Field | Status | Meaning | |---|---|---| | `doc_class` | required | Must be `ImplementationPlan`. | | `id` | required | Canonical ChangeSet ID, mat
- **ADR-135** (ADR-0135-aspirational-enforcement-gate): `cloud-ci/Rust gate packet aspirational-enforcement` scans the normative docs, specs, and registry corpus for binding enforcement claims that name repository enforcement surfaces. The default corpus roots are: - `docs` - `specs` - `registry` Callers can narrow or replace coverage with `--clear-default-corpus --corpus-root <path>` for fixture and lo
- **ADR-139** (ADR-0139-agentic-slo-gated-promotion): oyatie adopts a two-layer design: **adopted OSS observability runtime (Layer A)** plus **oyatie owned agentic-gate differentiator (Layer B)**. Both layers ship together as one M01 phase; neither is scheduled-for-distinct-tracked-work. The deployment substrate is the canonical Grafana stack, self-hosted; the gate logic is a new oyatie µservice `obse
- **ADR-148** (ADR-0148-service-mesh-cilium-ambient-layered): Oyatie adopts a **layered service-mesh substrate** in which **each layer owns exactly one concern**: ### Layer ownership (canonical; zero overlap) | Layer | Owner | Responsibilities | Out-of-scope | |---|---|---|---| | **Layer 3/4 (kernel-level dataplane)** | **Cilium 1.19.x** (pin 1.19.4) [amended 2026-05-26 — see note] | CNI (pod networking, IPAM
- **ADR-157** (ADR-0157-api-gateway-tier): Oyatie adopts a dedicated **`api-gateway` µservice** as the canonical north-south entry tier. Every external HTTPS REST or realtime request transits the api-gateway tier before the cell-µservice tenant-routing layer hands it to a workload µservice. ### Operational shape 1. **Termination.** TLS 1.3 termination at the api-gateway edge (cert rotation 
- **ADR-162** (ADR-0162-per-tenant-audit-log-slicing): Audit-chain seals partition by `tenant_id`. The audit-chain µservice maintains: ### Sharding scheme - **Per-pack shared shard.** Multi-tenant cells (e.g. `pack-us-shared`) use a *per-pack* audit-chain Merkle tree; tenant_id partition is a *leaf-level* partition within the shared tree. The Merkle root covers all tenants in the pack; per-tenant retri
- **ADR-167** (ADR-0167 — Tenant-facing CLI binary `oya` (separate from internal `oya-dev-cli`)): Oyatie introduces a SECOND CLI binary, also named `oya` from the tenant's perspective, distributed as the crate `oya-tenant-cli` and packaged under the name `oya` in tenant-facing artifact channels (Homebrew tap, apt repo, container image `ghcr.io/oyatie/oya:<semver>`, MSI installer). The two binaries are kept distinct via: - **Repo layout**: `crat
- **ADR-175** (ADR-0175-tenant-lifecycle-workflow): ### D-1. Canonical six-state machine ``` Pending ──onboard_saga──▶ Active ──┬──suspend_saga──▶ Suspended ──unsuspend_saga──▶ Active │ └──migrate_saga──▶ Migrating ──migrate_completion──▶ Active (in target cell) │ └──offboard_saga──▶ Offboarded │ delete_saga │ ▼ DeletionConfirmed ``` State semantics: | State | Meaning | Allowed transitions | | --- |
- **ADR-182** (ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation): Oyatie adopts a **two-substrate ingress model** with zero feature overlap: ### North-south (public → cluster): Envoy Gateway 1.8.0 The canonical north-south substrate is **Envoy Gateway 1.8.0** (CNCF; Kubernetes Gateway API v1.0 conformant; vendor-neutral; deployed as a dedicated `api-gateway` µservice per ADR-0157). Envoy Gateway owns: - **TLS ter
- **ADR-185** (ADR-0185-workflow-studio-client-stack): Oyatie adopts the following per-surface client matrix. Each surface is native; each ecosystem shares logic via its own idiomatic shared-layer; the cross-ecosystem unifier is the OpenAPI 3.2.0 contract. ### Per-surface client matrix | Surface | Stack | API client codegen | Ecosystem-shared layer | Status | |---|---|---|---|---| | Web (Phase 1) | **S
- **ADR-190** (ADR-0190-scim-2-provisioning-enterprise-tenants): **Identity µservice exposes a SCIM 2.0 RFC 7643/7644 endpoint at `/scim/v2/{tenant}` per tenant. Inbound provisioning from Okta / Entra / Workspace pushes Users + Groups; lifecycle states (active, suspended, deleted) propagate. A pluggable adapter contract (`HrisAdapter` trait) handles non-SCIM HRIS sources by translating to internal SCIM-shaped op
- **ADR-194** (ADR-0194-tenant-facing-timeseries-timescaledb): Oyatie adopts **TimescaleDB 2.26.x Community Edition (Apache-2.0)** as a Postgres 18 extension installed onto the existing Tier 1 Postgres OLTP cluster (per ADR-0184), opt-in per µservice via manifest. TimescaleDB community-edition v2.26 supports Postgres 18 since v2.23 (March 2026 release line). ### Scope (in-scope features — Apache-2.0 community 
- **ADR-204** (ADR-0204-workflow-studio-canvas-library): Per stack: | Stack | Phase 1 canvas | Phase 2 canvas (in-house) | |---|---|---| | SvelteKit (web, Phase 1) | **svelte-flow** (`@xyflow/svelte`, MIT) | `oya-canvas-svelte` (built on Svelte 5 + signals + SVG/Canvas2D) | | Leptos (web, Phase 2) | (skipped — Leptos web ships Phase 2) | `oya-canvas-leptos` (Rust-native, Leptos signals + SVG/Canvas2D + W
- **ADR-213** (Ecosystem-as-a-Service architecture — Plugin/App Store substrate (third-party de): Oyatie ships an **Ecosystem-as-a-Service** product surface, composed of **two single-concern µservices** under the ADR-0131 flat layout, citing the industry inheritances listed in the frontmatter: ### 1. Two µservices, single-concern each (per ADR-0132) | µservice | Concern | Persona served | Inheritance | |---|---|---|---| | `microservices/plugin-
- **ADR-215** (ADR-0215-multi-context-platform-architecture): Adopt a multi-context principal model across Oyatie. One human principal can hold multiple active data contexts simultaneously: - `work-context-{employer}` for each employer or client relationship; - `personal-context` for B2C artifacts; - `healthcare-patient-context`; - `healthcare-provider-context`; - `education-student-context`; - `government-ci
- **ADR-234** (ADR-0234-connect-social-expansion-planning-contract): Accept the expansion PRDs as a **planning contract** for PR #130, with these constraints: - The new sub-products are catalog/planning surfaces only until their crates, validators, gates, and CI lanes land. - `industry_patterns_adopted`, `anti_patterns_avoided`, `hyperscaler_bar`, and `production_readiness_gates` are advisory unless a concrete valid
- **ADR-243** (ADR-0243-cedar-as-universal-gate): ### D-1. Cedar evaluates every policy-class decision The 23 policy-class decisions enumerated in §Context are migrated to Cedar evaluation. New policy-class decisions introduced by future ADRs are Cedar from inception. The canonical Cedar evaluation contract: ```rust // microservices/policy-engine/src/api.rs pub struct EvaluationRequest { pub princ
- **ADR-248** (ADR-0248-amazon-shape-cellular-architecture): The platform adopts AWS cell-based architecture as the canonical topology. Sixteen decisions follow. REGCLOUD-001 planning-artifact registration: the non-mutating planning/spec artifact `plan/compliance-selective-cell-placement-architecture.md`, its ownership seed `plan/OWNERS`, and its multispectrum evidence packet `evidence/multispectrum/regcloud
- **ADR-273** (ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability): We adopt the twelve decisions D-1 through D-12 below. They form the contract between the `mail` µservice, the `cloud-secrets` and `cloud-network-dns` µservices, the `audit-chain` and `events-bus` substrates, and the per-tenant control plane. Every decision is mandatory; any partial deployment fails the §Verification gates. The implementation surfac
- **ADR-295** (ADR-0295-bootstrap-ci-spiffe-kill-switch): The keystone establishes seven decision sub-sections, D-1 through D-7. ### D-1. SPIFFE workload identity for every Stage-1 runner Every Stage-1 external CI runner — whether GitHub Actions, CircleCI, a temporary self-hosted runner, or a multi-cloud bake- in-place provisioner — receives a SPIFFE Verifiable Identity Document (SVID) issued by the one-s
- **ADR-297** (ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape): ### §B. Three orthogonal control families wired at three layers The abuse-defence baseline is **three orthogonal control families** (anti-bot, anti-spoof, anti-scrape) wired at **three layers** (Tier-0 edge, per-µservice, Cedar policy). The 3×3 matrix produces nine cells; each cell has a defined primitive. The matrix is defence-in-depth: **no singl
- **ADR-303** (ADR-0303-cognitive-impairment-decision-resilience): ### §B. Four orthogonal primitives composed at three layers The decision-resilience baseline is **four orthogonal primitives** (cooling-off, trusted-contact, mutation-cadence, guardianship-overlay) composed at **three layers** (Tier-0 substrate shared crate, per-µservice gate, Cedar policy fragment). The 4×3 matrix produces twelve cells; each cell 
- **ADR-305** (ADR-0305-delegated-agent-authority-chain): ### §B. Five core primitives at three layers The delegated-agent authority chain is **five core primitives** (token issuance; attestation chain; scope inheritance; cross-tenant block; audit linkage) wired at **three layers** (Tier-0 shared crate, per-µservice gate, Cedar policy fragment). The 5×3 matrix produces fifteen cells; each cell has a defin
- **ADR-309** (ADR-0309-detection-fairness-audit-civil-rights): ### §B. Five fairness invariants — substrate-enforced Establish the canonical five fairness invariants as substrate-level gates enforced before any ML model serves production traffic in the detection substrate (per ADR-0307) or any other product-facing AI surface. ### §B.1. Invariant 1 — No proxy discrimination Features that proxy protected classes
- **ADR-312** (ADR-0312-court-warrant-scoped-piercing): Bundled with the keystone-bundle 2026-05-20 foundational doctrine synthesis as the **court-warrant-scoped-piercing** ADR, the companion to ADR-0311 (dual-tenant identity boundary). Surfaced by the Wave-3-E ecosystem journey catalog j129 (`court-warrant-pierces-personal-tenant-with-judicial-oversight`) and cross-linked with j130 (bribery attempt via
- **ADR-313** (ADR-0313-conglomerate-tenant-hierarchy-sovereign-children): # ADR-0313: Conglomerate-Tenant Hierarchy — Sovereign-Child + Policy-Engine-Mediated Controlling-Entity Grant
- **ADR-325** (ADR-0325-capability-tier-pricing-anchors-public): The per-tier anchor table is below. All amounts in USD, expressed as monthly recurring revenue (MRR) per tenant per category. Annual prepayment carries a 12% discount (D-3); BYOK carries 15% discount on the LLM-cost component (D-4). | Tier | Plugin | App | Workflow | Agent | Model | Dataset | |-----------|--------|--------|----------|--------|-----
- **ADR-328** (Substance Bar as Canonical Sequence and Batch Discipline): ### B.1 Decision statement Oyatie realignment work MUST follow the five-phase canonical build sequence in Section D-1. Phase 4 work MUST follow the Big 8 sub-sequence in Section D-2. Every agent dispatch in the realignment wave MUST include an agent-class-specific five-anchor set as defined in Section D-3. Every microservice ownership audit MUST us
- **ADR-334** (ADR-0334-shorts-microservice-merged-into-social): D-1. `microservices/shorts/` is retired as a standalone µservice. D-2. `microservices/shorts/` keeps only a `RETIRED.md` redirect marker. D-3. Historical shorts service content is not the live authority after this ADR. D-4. `microservices/social/` is the canonical owner of short-form video. D-5. `microservices/social/` is the canonical owner of lon
- **ADR-340** (Capacity model per microservice manifest (baseline_cpu_per_tenant + baseline_ram): ### B.1 Decision statement Every µservice's `microservices/<name>/manifest.json` MUST declare a top-level `capacity_model` block with the following required fields: - `baseline_cpu_per_tenant`: decimal vCPU value per active tenant at steady state (e.g., `0.1`). - `baseline_ram_per_tenant`: integer MiB value per active tenant at steady state (e.g., 
- **ADR-341** (Cellular promotion gates — explicit per-Tier 0..4 machine-checkable criteria + a): ### B.1 Decision statement Every cellular promotion or demotion event between ADR-0248 tiers (Tier 0..Tier 4, where Tier 0 = highest blast-radius / most isolated, Tier 4 = best-effort / edge / lowest blast-radius) MUST be evaluated against six machine-checkable gate inputs by the new CI lane `oya-check-cell-promotion-gates` plus the in-cluster cell
- **ADR-346** (oya verify --ci-required MUST locally mirror the full CI matrix (cargo fmt + car): ### B.1 Decision statement `./bin/oya verify --ci-required` is the canonical local pre-push verifier. The verifier MUST locally mirror the full CI matrix at `.github/workflows/pr-tests.yml` and MUST block on exit-0 of EACH step before returning success to the caller. The five mandatory mirror steps are: 1. **D-1:** `cargo fmt --all --check` — forma
- **ADR-348** (Autosharding + auto-rebalance + dynamic sharding (cellular topology MUST support): ### B.1 Decision statement Cellular topology MUST support three control-plane-driven automation modes underneath the cell-level promotion gates already doctrined in ADR-0341: 1. **Autosharding** — tenant→cell/shard placement is computed automatically by the control plane; no human operator picks placement; inputs are capacity_model (ADR-0340) + com
- **ADR-360** (CI/CD pipeline optimization program — affected-target precision, gate-only overl): Adopt a seven-part CI/CD optimization program. Each part has a hard correctness rule so optimization never weakens the governance gates. - **O1 — Affected-target precision.** Add an additive `oya verify --affected [--base <ref>]` presubmit mode. Classify the changed-file set vs the base into: **Full** (any of `Cargo.lock`, root/`[workspace]` `Cargo
- **ADR-366** (ADR-0366-agentic-high-throughput-self-enforcing-pipeline): ### 1. Parallelism with conflict PREVENTION (not just resolution) A **single-threaded owner-agent per service/lane** (AWS STO) owns **disjoint paths** — the flat / no-grouping doctrine (ADR-0362) makes service paths naturally disjoint. One **isolated worktree per lane**. A **concurrent-safe-paths** admission gate rejects two in-flight lanes touchin
- **ADR-367** (ADR-0367-trustless-pre-merge-verification-gateway): ### 1. The producer never certifies its own work The author-agent's self-reported evidence (claimed test/gate results) is **never trusted** for the merge decision. It is at most a hint. ### 2. Trusted-runner-signed evidence The **trusted runner** (the farm / Jenkins, ADR-0349/0361) re-executes every gate **hermetically from a clean checkout** and *
- **ADR-369** (ADR-0369-gated-stacked-trunk-change-flow): **Gated stacked-trunk with a speculative train**, on plain git (ADR-0363) + GitHub PRs as the cheap mechanism + audit record (ADR-0367 §4): 1. **Conflict prevention — ownership-sharding (now, D1).** One owner-agent per service on disjoint flat paths (ADR-0362); `concurrent-safe-paths` admission + CODEOWNERS. Disjoint paths → most concurrent work ne
- **ADR-373** (ADR-0373-llm-gateway-production-design): Adopt the design recorded in the gateway design dossier, summarized as four decisions: 1. **Provider abstraction + canonical OpenAI-compatible surface** (ADR-0373-D1): one canonical OpenAI-shaped request/response with per-provider adapter traits; OpenAPI 3.2.0 contract; byte-passthrough SSE; OpenAI error envelope; 429 + `Retry-After`; two security 
- **ADR-374** (ADR-0374-ci-webhook-gateway-github-actions): Build a **flat single-concern Rust microservice**, `ci-webhook-gateway` (`microservices/ci-webhook-gateway/`, `src/` root per ADR-0131; package `oya-ci-webhook-gateway-app`), that is the FIRST hop of the gated pipeline: 1. **Receive** GitHub webhook deliveries at `POST /webhook/github` (axum/Tokio/ Tower/Hyper — blessed runtime deps). 2. **Verify**
- _…plus 66 additional members listed in supersedes frontmatter; full text in git history / archive._

## Consequences

- Agent default read path: `docs/decisions/ADR-0xxx` apex files + this topic.
- Citations to member numbers remain valid via `docs/decisions/_disposition/adr-redirect.v1.json`.
- Further body merge refinements may land as amendments to this apex only.

### ADR-21 residual

**ADR-0021-intelligence-capability-registry-and-mcp-gateway** — We define the canonical `Capability` record in `oya-intelligence-capability-kernel` and serve it via an MCP-compatible gateway that exposes a per-tenant endpoint. The catalog YAML in `registry/catalog/` is the source of truth; the kernel projects it into typed records at runtime. ### Capability primitive (`oya-intelligence-capability-kernel`) ```rust // crates/oya-intelligence-capability-kernel/sr

### ADR-273 residual

**ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability** — We adopt the twelve decisions D-1 through D-12 below. They form the contract between the `mail` µservice, the `cloud-secrets` and `cloud-network-dns` µservices, the `audit-chain` and `events-bus` substrates, and the per-tenant control plane. Every decision is mandatory; any partial deployment fails the §Verification gates. The implementation surface (crates, helm charts, runbooks, specs) is enumer

### ADR-297 residual

**ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape** — ### §B. Three orthogonal control families wired at three layers The abuse-defence baseline is **three orthogonal control families** (anti-bot, anti-spoof, anti-scrape) wired at **three layers** (Tier-0 edge, per-µservice, Cedar policy). The 3×3 matrix produces nine cells; each cell has a defined primitive. The matrix is defence-in-depth: **no single cell gates a request alone**; the nine cells com

### ADR-551 residual

**Merge-base frozen baseline ratchet** — 1. **The frozen reference is the merge-base baseline.** Both firewall predicates compare against the gate-baseline face exactly as committed at `git merge-base <base_ref> HEAD` — never the working-tree copy. This is the standard ratchet comparison root (proven methodology, Rust-native implementation): Betterer and eslint-ratchet-style tightening gates compare against the merge-base; Bazel target d

### ADR-367 residual

**ADR-0367-trustless-pre-merge-verification-gateway** — ### 1. The producer never certifies its own work The author-agent's self-reported evidence (claimed test/gate results) is **never trusted** for the merge decision. It is at most a hint. ### 2. Trusted-runner-signed evidence The **trusted runner** (the farm / Jenkins, ADR-0349/0361) re-executes every gate **hermetically from a clean checkout** and **signs** the results (SLSA provenance + cosign). T

### ADR-303 residual

**ADR-0303-cognitive-impairment-decision-resilience** — ### §B. Four orthogonal primitives composed at three layers The decision-resilience baseline is **four orthogonal primitives** (cooling-off, trusted-contact, mutation-cadence, guardianship-overlay) composed at **three layers** (Tier-0 substrate shared crate, per-µservice gate, Cedar policy fragment). The 4×3 matrix produces twelve cells; each cell has a defined primitive. The matrix is defence-in-

### ADR-629 residual

**Crate-catalog coverage: every live crate carries a catalog row, closing the crate→row direction** — Every live first-party crate MUST carry `registry/catalog/<package-name>.yaml`. Born-blocking against a FROZEN, shrink-only baseline of the crates lacking a row at adoption (197 of 926). Pre-existing gaps are tolerated debt. A NEW crate must ship its row. A MOVED crate has a new package name, is therefore absent from the baseline, and fails unless its row moves in the same change — which is exactl

### ADR-524 residual

**Kernel buckification + additive-first verify parity; pinned QEMU/musl toolchain; tracked out/*.elf blob retirement; exte** — Bring the kernel port (`stack/kernel`) into the one buck2 graph (ADR-0522) in dependency order, with verify ADDITIVE-FIRST so the currently-green cargo + QEMU goal-ladder floor is never at risk. (a) **CARRIERS become buck2 outputs** — the per-crate `.cargo/config.toml` rustflags map to `rustc_flags` + a linker-script `srcs` dep; the `build.sh` tmpdir-escape hack (which exists only because cargo co

### ADR-340 residual

**Capacity model per microservice manifest (baseline_cpu_per_tenant + baseline_ram_per_tenant + storage_per_tenant + conne** — ### B.1 Decision statement Every µservice's `microservices/<name>/manifest.json` MUST declare a top-level `capacity_model` block with the following required fields: - `baseline_cpu_per_tenant`: decimal vCPU value per active tenant at steady state (e.g., `0.1`). - `baseline_ram_per_tenant`: integer MiB value per active tenant at steady state (e.g., `256`). - `storage_per_tenant`: integer GB value p

### ADR-538 residual

**Globbed root workspace membership and coverage gate** — The root workspace uses globbed membership: ```toml members = [ "libs/oya-*", "cloud/*/crates/oya-*", "cloud/cloud-ci/gates/*", "oya/*/crates/oya-*", "oya/office/oya-*", "tools/oya-*", ] exclude = [ "cloud/cloud-kernel", "ci/facade/automation-language-policy", ] ``` Consumers requiring a Cargo-valid concrete member set MUST call `libs/oya-workspace-members-kernel::resolve_member_dirs(repo_root)`.

### ADR-44 residual

**ADR-0044-service-mesh-istio-ambient-and-envoy-gateway** — We adopt **Istio Ambient mode** as the canonical east-west service mesh; **Envoy** (gateway-class) as the canonical north-south edge gateway; **mTLS everywhere** as the default with per-traffic-type opt-out only via ADR; **per-cell namespace** as the isolation unit; **cross-cell traffic** explicit + Cedar-policied + audit-chained per call. ### Istio Ambient mode ```yaml # infra/istio/profile.yaml

### ADR-341 residual

**Cellular promotion gates — explicit per-Tier 0..4 machine-checkable criteria + auto-promotion via cell-orchestrator** — ### B.1 Decision statement Every cellular promotion or demotion event between ADR-0248 tiers (Tier 0..Tier 4, where Tier 0 = highest blast-radius / most isolated, Tier 4 = best-effort / edge / lowest blast-radius) MUST be evaluated against six machine-checkable gate inputs by the new CI lane `oya-check-cell-promotion-gates` plus the in-cluster cell-orchestrator µservice. The six gates are: (1) err

### ADR-369 residual

**ADR-0369-gated-stacked-trunk-change-flow** — **Gated stacked-trunk with a speculative train**, on plain git (ADR-0363) + GitHub PRs as the cheap mechanism + audit record (ADR-0367 §4): 1. **Conflict prevention — ownership-sharding (now, D1).** One owner-agent per service on disjoint flat paths (ADR-0362); `concurrent-safe-paths` admission + CODEOWNERS. Disjoint paths → most concurrent work never conflicts. 2. **Authoring — stacked diffs on p

### ADR-515 residual

**Phase-0 firewall + one-canonical-CI + the 2026-06-07 cloud-native posture — oya-ci-required is the single GitHub-Actions** — ### D1. The Phase-0 firewall is the enforcement substrate The Phase-0 false-green firewall is the substrate that makes merge-gate enforcement **real**: 1. **One generated accounting-registry.** `oya-cloud-ci-accounting-registry-app` emits one deterministic record per tracked path (+ the TTL / crosswalk / enforcement-inventory faces); `registry-drift` enforces `committed == regenerated` byte-for-by

### ADR-631 residual

**A capability that spans strata has a wrong boundary, not a tier problem: split iam into iam (S1 PDP) and identity (S3 pr** — ### D1 — a stratum span is a BOUNDARY DEFECT, not a tier defect **If a capability's absorbed services span strata, the boundary is wrong.** A capability is simultaneously a unit of ownership and a unit of dependency position; if its members sit at different depths in the ADR-0280 DAG, it is not one capability. The unanimity rule is therefore **a boundary detector**, not an obstacle to be routed ar

### ADR-537 residual

**Dogfood bootstrap order + Rust-owned stack doctrine — the circular-dependency-free ten-step bring-up, the buck2 tier-dep** — ### §1 The circular-dependency-free bring-up (step 0 ceremony + steps 1–10) - **Step 0 — Root ceremony (offline, witnessed, recorded).** Generate the root CA, the KMS domain root material, the sealed FIDO2 break-glass credential set (ADR-0536 D-1), and the hand-signed DNS seed snapshot (ADR-0536 D-9). This is the only manual step in the estate's life. - **Step 1 — KMS unseal.** cloud-kms boots on

### ADR-99 residual

**Cedar policy extension — foundry supervisor capabilities in docs/policies/foundry-supervisor.cedar** — Add `docs/policies/foundry-supervisor.cedar` containing tier-gated policies for the five supervisor capabilities. The file is created by Wave 4b (Task #12); this ADR records the design. ### Policy design Autonomy tiers (from ADR-0007): | Tier | Label | Principal class | |---|---|---| | T1 | `read-only-observer` | Monitoring/observability systems, read-only operators | | T2 | `suggest-only` | Workf

### ADR-94 residual

**ADR-0094-handler-trait-with-associated-error** — Add a typed `Handler` trait in `oya-http-middleware-kernel`: ```rust pub trait Handler: Send + Sync { type Error: Into<HttpResponse>; fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error>; } pub fn call_into_response<H: Handler>(handler: &H, req: HttpRequest) -> HttpResponse { match handler.call(req) { Ok(r) => r, Err(e) => e.into(), } } ``` Plus a bridge helper in the adapter that

### ADR-605 residual

**Supply-chain audit gate (owned RustSec advisory scan over a vendored mirror)** — Ship a **self-contained cloud-ci gate**, `cloud-ci-supply-chain-audit` (`ci/facade/supply-chain-audit`), mirroring the kernel-purity (ADR-0547) / authz-coverage (ADR-0566) registration footprint: own crate, own policy JSON, one appended matrix line in `.github/workflows/oya-ci-required.yml`, no `libs/oya-ci-config` edit, no producer-face binding. The advisory parsing/normalization lives in a reusa

### ADR-348 residual

**Autosharding + auto-rebalance + dynamic sharding (cellular topology MUST support three control-plane-driven automation m** — ### B.1 Decision statement Cellular topology MUST support three control-plane-driven automation modes underneath the cell-level promotion gates already doctrined in ADR-0341: 1. **Autosharding** — tenant→cell/shard placement is computed automatically by the control plane; no human operator picks placement; inputs are capacity_model (ADR-0340) + compliance_pack constraints (ADR-0251) + ResidencyCla

### ADR-627 residual

**Enforce ADR-0562's facade→core layering rule, keyed to survive the remaining capability migration** — **1. Ship a gate for the layering rule.** `ci/facade/facade-core-layering` freezes the 35 current violators and makes a **new** one born-blocking. Two codes, deliberately separate: - `facade_core_direct_dep` — the 30 genuine violations. - `facade_core_no_ports_layer` — the 5 in `compute` and `intelligence`, which ADR-0562 §10.6 declares dependency-legal *while no ports layer exists*. A distinct co

### ADR-167 residual

**ADR-0167 — Tenant-facing CLI binary `oya` (separate from internal `oya-dev-cli`)** — Oyatie introduces a SECOND CLI binary, also named `oya` from the tenant's perspective, distributed as the crate `oya-tenant-cli` and packaged under the name `oya` in tenant-facing artifact channels (Homebrew tap, apt repo, container image `ghcr.io/oyatie/oya:<semver>`, MSI installer). The two binaries are kept distinct via: - **Repo layout**: `crates/oya-tenant-cli/` (this ADR) vs `crates/oya-dev-

### ADR-527 residual

**oya-ci conformance FLOOR as a config-driven portable engine: engine-vs-policy seam + closed-schema oya-ci.toml loader + ** — Extract ALL oyatie POLICY out of compiled-in `const`s + producer literals into a CLOSED-schema repo-rooted config (`oya-ci.toml` at repo root, OR `.oya-ci/config.json` — format is OQ-1, carried to the founder; one canonical format, identical schema either way), loaded by a NEW pure I/O-free `oya-ci-config` (kernel-role) crate that parses + validates into typed structs and REJECTS unknown keys. The

### ADR-83 residual

**ADR-0083-rust-error-handling-tier-decision** — We adopt a **three-tier** error-handling policy applied uniformly across every `oya-*` Rust crate. Per-tier rules below are normative (RFC-2119 keywords as defined in docs/standards/error-handling.md§1). ### Tier 1 — Library crates (kernel / domain / app / adapter / api / worker / infrastructure / service / rest / cli / bindings) - Public errors **MUST** be matchable enums exported via [`thiserror

### ADR-162 residual

**ADR-0162-per-tenant-audit-log-slicing** — Audit-chain seals partition by `tenant_id`. The audit-chain µservice maintains: ### Sharding scheme - **Per-pack shared shard.** Multi-tenant cells (e.g. `pack-us-shared`) use a *per-pack* audit-chain Merkle tree; tenant_id partition is a *leaf-level* partition within the shared tree. The Merkle root covers all tenants in the pack; per-tenant retrieval traverses the per-tenant subtree. - **Per-sov

### ADR-528 residual

**Remediation is first-class on the gate contract: remediate() ships WITH every gate (pure, returns described edits, never** — Extend the published, semver'd gate-trait crate `oya-ci-gate-contract` (the home of `Finding` / `evaluate_keyed`, per ADR-0515 WS-D) so every gate carries a remediation sibling to detection: ```text fn remediate(&self, finding: &Finding, face: &Value) -> Remediation where Remediation = AutoFix(Edit) | AutoGenerate(NewFile) | None ``` `remediate()` is as PURE as `evaluate_keyed` — it returns a *des

### ADR-525 residual

**oya-ci hermetic buck2 execution: git-facts boundary (Option C, committed content-addressed face) + buck2-native gates + ** — Four condensed parts: **D1 — git-facts boundary (Option C).** Push ALL ambient git access to ONE out-of-graph emitter and make every downstream action consume a frozen, content-addressed snapshot. A NEW non-hermetic `rust_binary` `oya-cloud-ci-git-facts-emitter` (the four `Command::new("git")` calls moved verbatim out of the producer) emits a COMMITTED canonical-JSON face `git-facts.generated.json

### ADR-194 residual

**ADR-0194-tenant-facing-timeseries-timescaledb** — Oyatie adopts **TimescaleDB 2.26.x Community Edition (Apache-2.0)** as a Postgres 18 extension installed onto the existing Tier 1 Postgres OLTP cluster (per ADR-0184), opt-in per µservice via manifest. TimescaleDB community-edition v2.26 supports Postgres 18 since v2.23 (March 2026 release line). ### Scope (in-scope features — Apache-2.0 community only) | Feature | Edition | In-scope? | |---|---|-

### ADR-373 residual

**ADR-0373-llm-gateway-production-design** — Adopt the design recorded in the gateway design dossier, summarized as four decisions: 1. **Provider abstraction + canonical OpenAI-compatible surface** (ADR-0373-D1): one canonical OpenAI-shaped request/response with per-provider adapter traits; OpenAPI 3.2.0 contract; byte-passthrough SSE; OpenAI error envelope; 429 + `Retry-After`; two security schemes (ingress vs admin). 2. **Key-pool resilien

### ADR-328 residual

**Substance Bar as Canonical Sequence and Batch Discipline** — ### B.1 Decision statement Oyatie realignment work MUST follow the five-phase canonical build sequence in Section D-1. Phase 4 work MUST follow the Big 8 sub-sequence in Section D-2. Every agent dispatch in the realignment wave MUST include an agent-class-specific five-anchor set as defined in Section D-3. Every microservice ownership audit MUST use the five-dimension protocol in Section D-4. Ever

### ADR-539 residual

**Cloud CI freshness gate for Cargo.lock member parity and generated-face byte parity** — Add `ci/facade/generated-artifact-freshness` as a single-concern Rust gate. NAME: oya-cloud-ci-freshness-app JUSTIFICATION: - microservice = cloud-ci: the cloud-ci admission product owns gate execution per ADR-0515. - bc-tokens = freshness: the bounded concern is candidate-tree freshness, not general registry accounting. - layer = app: the crate exposes a composition-root binary plus a pure librar

### ADR-313 residual

**ADR-0313-conglomerate-tenant-hierarchy-sovereign-children** — # ADR-0313: Conglomerate-Tenant Hierarchy — Sovereign-Child + Policy-Engine-Mediated Controlling-Entity Grant

### ADR-9 residual

**ADR-0009-cell-architecture-per-tenant-per-region** — We adopt **cells as the primary blast-radius isolation primitive**, sized per-tenant per-region with five cell tiers, cell-routing primitives at edge / mesh / store / event layers, and per-cell HSM partitions. Cell-isolation evidence is collected quarterly per regulatory pack. ### Cell sizing tiers | Tier | Reads as | Tenant count | Use case | |---|---|---|---| | `Dedicated` | One tenant per cell

### ADR-609 residual

**Masterplan v2 single plan authority + fabric drive-loop increment (four plan gates wired into oya-ci-required)** — 1. **Single plan authority.** `/specs/masterplan.json` `masterplan_v2` is the SOLE live plan authority: one `MPV2-*` work-item ID space, an explicit dependency DAG, program-sharded coverage, per-claim evidence refs, and an auditable `surface_dispositions` ledger that absorbs or archives-with-provenance every legacy surface. Surviving human-facing surfaces (`docs/MASTERPLAN.md`) are GENERATED proje

### ADR-531 residual

**Auto-remediation delivery + safety model: the oya-bot-autofix PROPOSE-only fleet member, shrink-only burn-down, HMAC web** — The privileged delivery process that APPLIES `remediate()` output (ADR-0528) is `oya-bot-autofix`, a member of the repo-automation-bot fleet alongside `oya-bot-depupdate` / `oya-bot-release`, sharing the host, signed-capability trust model, and merge-gate kernel. Five binding safety invariants: 1. **DETERMINISTIC + REPRODUCIBLE** — same face ⇒ byte-identical edits, tested with the RED/GREEN fixtur

### ADR-47 residual

**ADR-0047-search-backend-strategy** — We adopt **pgroonga** day-1 with **legal isolation** per License Policy ADR + replacement plan; **Tantivy** (MIT) in-Rust at scale; **OpenSearch** (Apache-2) only as an adapter behind a port; **Elasticsearch SSPL forbidden** in product surface; **in-house long-horizon** (KR morphology + Tantivy + custom ranker) under `crates/oya-search-backend-*`. ### pgroonga day-1 (KR launch) ```sql -- per-tenan

### ADR-581 residual

**Fail-closed verified-caller + PDP authorization for the workload-principal lifecycle control plane (:suspend/:retire)** — 1. **Two clean PORTS owned by the boundary crate** (`iam-identity-workload-rest`), concrete adapters outside it (owned-W5 shape): - `CallerVerifier::verify_principal(&HeaderMap) -> Option<VerifiedCaller>` — caller authentication. A `VerifiedCaller` has private fields and no public constructor, so it can ONLY be minted by a verifier that proved an UNFORGEABLE credential (`constant_time_eq` bearer i

### ADR-346 residual

**oya verify --ci-required MUST locally mirror the full CI matrix (cargo fmt + cargo check + cargo clippy + cargo nextest ** — ### B.1 Decision statement `./bin/oya verify --ci-required` is the canonical local pre-push verifier. The verifier MUST locally mirror the full CI matrix at `.github/workflows/pr-tests.yml` and MUST block on exit-0 of EACH step before returning success to the caller. The five mandatory mirror steps are: 1. **D-1:** `cargo fmt --all --check` — formatting validation. 2. **D-2:** `cargo check --works

### ADR-540 residual

**Cargo workspace to Buck2 target parity gate** — Add `ci/facade/build-target-parity` as a pure cloud-ci gate. NAME: oya-cloud-ci-target-parity-app JUSTIFICATION: - microservice = cloud-ci: the cloud-ci admission product owns gate execution per ADR-0515. - bc-tokens = target-parity: the bounded concern is Cargo member to Buck target parity. - layer = app: the crate is an executable CI gate surface with pure evaluator logic. - exemptions claimed:

### ADR-560 residual

**NativeLink CAS slice 1: deployable cache-only substrate + opt-in buck2 wiring + cold integrity-canary** — ### D1 — Declarative deployment artifacts: the `nativelink-cas` tier only `infra/nativelink/nativelink-cas.k8s.yaml` deploys NativeLink **v1.6.2** (current upstream release, 2026-07-17) as the cache-only tier of the founder-decided 2026-05-30 three-tier split (`docs/ideas/nativelink-remote-cache-first.md`): CAS + Action Cache, no scheduler, no workers. Precedent accuracy, per the hyperscaler lens:

### ADR-566 residual

**Authz-coverage gate (unauthenticated HTTP control-plane backstop)** — Ship a **self-contained cloud-ci gate**, `cloud-ci-authz-coverage` (`ci/facade/endpoint-authorization-coverage`), mirroring the kernel-purity (ADR-0547) registration footprint: own crate, own policy JSON, one appended matrix line in `.github/workflows/oya-ci-required.yml`, no `libs/oya-ci-config` edit, no producer-face binding. The gate's neutral Rust engine lives in `ci/facade/endpoint-authorizat

### ADR-616 residual

**De-commit the firewall frozen-reference baseline — regenerate it from merge-base SOURCE (reverses ADR-0596)** — 1. **De-commit `gate-baseline.generated.json`** (manifest `not-tracked-in-git` + merge_policy `never-manual-merge-regenerate-from-source-tree`; `git rm --cached`; `.gitignore` drop the `!` negation). Its freshness/registry-drift checks fall to the regenerate-twice determinism class (mirroring ADR-0604/0613), the same class as every other de-committed face. 2. **Regenerate the frozen baseline from

### ADR-118 residual

**ADR-0118-retire-archive-orphan-fitness-lane** — Retire `archive-orphan` as an executable fitness lane. The retirement removes: - `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/` - `crates/oya-governance-archive-orphan-kernel` - `tools/oya-governance-archive-orphan-app` - workspace members for both retired crates - catalog entries for the retired kernel/app capability The retirement keeps a small historical lane record at `docs/fi

### ADR-526 residual

**oya-ci scm-facts boundary: VCS-agnostic identifiers (git-facts->scm-facts rename, schema v1 retained) + the ScmFactsSour** — Two coupled, byte-parity-preserving moves, and nothing else: **(1) RENAME every git-flavored identifier in the boundary to the VCS-agnostic family `scm-facts` / `scm_facts` / `SCM_FACTS` / `ScmFacts`.** The emitter crate dir + binary `oya-cloud-ci-git-facts-emitter-app` → `oya-cloud-ci-scm-facts-emitter-app`; the committed snapshot `git-facts.generated.json` → `scm-facts.generated.json`; the schem

### ADR-234 residual

**ADR-0234-connect-social-expansion-planning-contract** — Accept the expansion PRDs as a **planning contract** for PR #130, with these constraints: - The new sub-products are catalog/planning surfaces only until their crates, validators, gates, and CI lanes land. - `industry_patterns_adopted`, `anti_patterns_avoided`, `hyperscaler_bar`, and `production_readiness_gates` are advisory unless a concrete validator exists in this repo. - Planned crate names mu

### ADR-534 residual

**Gate/pipeline-step SDK + gate-artifact marketplace: trait Gate extraction, runtime GateRegistry, three binding kinds (pr** — (1) **SDK EXTRACTION (highest-leverage):** extract ONE versioned crate (`oya-ci-gate-sdk`) holding the gate contract — `trait Gate { gate_id; codes; evaluate_keyed(&FaceValue) -> BTreeSet<Finding> [SSOT]; evaluate() -> Report [provided] }` plus the `Finding`/`Report`/`Verdict` types — replacing the copied `struct Finding` definitions. Preserve the pure/I-O-free/panic-free shape (`#![forbid(unsafe_

### ADR-597 residual

**oya-ci-materializer-kernel (E1): universal generated-artifact lifecycle — pure planner kernel** — Ship `libs/oya-ci-materializer-kernel` — a pure Rust planner + predicate kernel with zero I/O, zero clock, zero subprocess, zero git, zero net. Dependencies: `serde` + `serde_json` only. ### Public API ```rust // Pure analysis phase — no filesystem, clock, buck2, git. pub fn plan(manifest: &ControlPlane, scope: MaterializeScope) -> Result<MaterializePlan, PlanError>; // Pure verdict predicate — fe

### ADR-547 residual

**Kernel-purity dependency gate** — Ship a **self-contained cloud-ci gate**, `cloud-ci-kernel-purity` (`ci/facade/core-dependency-isolation`), that asserts: **no crate matching the kernel-name globs (`*-kernel`, `*-core`) — nor any workspace-internal crate reachable through its path-dependency closure — directly depends on a denylisted transient-tech crate**, unless an explicit, reasoned per-(crate, dep) exception is declared in pol

### ADR-148 residual

**ADR-0148-service-mesh-cilium-ambient-layered** — Oyatie adopts a **layered service-mesh substrate** in which **each layer owns exactly one concern**: ### Layer ownership (canonical; zero overlap) | Layer | Owner | Responsibilities | Out-of-scope | |---|---|---|---| | **Layer 3/4 (kernel-level dataplane)** | **Cilium 1.19.x** (pin 1.19.4) [amended 2026-05-26 — see note] | CNI (pod networking, IPAM); `CiliumNetworkPolicy` (L4 identity-based rules)

### ADR-628 residual

**Scan-root liveness: a declared coverage root that no longer resolves is a gate blind spot, not clean coverage** — A declared **coverage-bearing** scan root MUST resolve to a real path, or to a glob matching at least one path, or be declared FORWARD with a stated reason. Three declaration classes are distinguished, because only one is a defect when dead: | Class | Keys | Dead entry means | |---|---|---| | coverage-bearing | `roots`, `scan_roots`, `crate_root_globs`, `manifest_paths`, `store_manifest_paths` | t

### ADR-600 residual

**Root-workspace-hygiene allowlist gate — make committed repo-root scratch structurally impossible** — Ship `oya-cloud-ci-root-workspace-hygiene-app` — a born-blocking, UNIVERSAL, HERMETIC gate with a default-DENY posture and the legitimate root surface expressed as DATA. **Allowlist-as-DATA (universal).** `root-workspace-hygiene-policy.json` declares `allowed_root_files` (a rule table of `exact`/`suffix`/`prefix` basename matchers) and `allowed_root_dirs` (the permitted top-level capability/meta h

### ADR-570 residual

**Clean-arch port-placement gate (ports defined in core, not adapters)** — Ship `cloud-ci-port-placement`, a born-blocking cloud-ci gate that flags a `pub trait <Name>` whose name matches a storage/repository/port suffix heuristic and is DEFINED in a crate whose repo-relative path contains a forbidden layer-dir segment (`adapters`). - **HERMETIC pure-Rust predicate.** `collect_port_traits(root, policy)` enumerates workspace members (reusing `oya-workspace-members-kernel`

### ADR-608 residual

**Cedar deploy-parity gate (deployed ConfigMap ⊆ authored policy; no action-agnostic blanket permit)** — Ship a **self-contained cloud-ci gate**, `cloud-ci-cedar-deploy-parity` (`ci/facade/policy-deploy-parity`), mirroring the registration footprint of the supply-chain-audit (ADR-0605) and operator-secret-bootstrap (ADR-0606) gates: own crate, own policy JSON, one appended matrix line in `.github/workflows/oya-ci-required.yml`, no `libs/oya-ci-config` edit, no producer-face binding. ### D1 — Pure, po

### ADR-624 residual

**Stage the immutable ADR census epoch transition** — ### 1. Use a four-step protected merge train The epoch transition proceeds in exactly this order. Every step is a separate protected pull request against `dev`, with an SSH-signed commit, independent review, resolved threads, and a green `oya-ci-required` context on the exact candidate head. 1. **Bootstrap the generic epoch producer and gate.** Protect the generic producer, validator, receipt sche

### ADR-185 residual

**ADR-0185-workflow-studio-client-stack** — Oyatie adopts the following per-surface client matrix. Each surface is native; each ecosystem shares logic via its own idiomatic shared-layer; the cross-ecosystem unifier is the OpenAPI 3.2.0 contract. ### Per-surface client matrix | Surface | Stack | API client codegen | Ecosystem-shared layer | Status | |---|---|---|---|---| | Web (Phase 1) | **SvelteKit 2.55 + Svelte 5.55 (runes) + Vite 8.0 + T

### ADR-535 residual

**Cross-product versioning + reproducible-build attestation + distribution channel (OCI + pinned-git, crates.io optional-m** — **(1) VERSIONING — the 3-axis model (re-authored from ADR-0037 + ADR-0342).** Each of the seven products (ADR-0532) carries its OWN SemVer 2.0.0 + a published versioned config schema (`$id` + `schema_version`) so an adopter pins a product version and a config-schema version independently. The three axes: - **Crate/SDK axis (SemVer, re-authored from ADR-0037):** SDK packages use `MAJOR.MINOR.PATCH`

### ADR-312 residual

**ADR-0312-court-warrant-scoped-piercing** — Bundled with the keystone-bundle 2026-05-20 foundational doctrine synthesis as the **court-warrant-scoped-piercing** ADR, the companion to ADR-0311 (dual-tenant identity boundary). Surfaced by the Wave-3-E ecosystem journey catalog j129 (`court-warrant-pierces-personal-tenant-with-judicial-oversight`) and cross-linked with j130 (bribery attempt via personal Messenger audit-only via ombudsman) + j1

### ADR-565 residual

**Zero GraphQL in the owned API surface — the canonical surface set is REST + gRPC + async + realtime, and GraphQL returns** — **The owned stack carries NO GraphQL surface.** Not a REST-named husk, not a generated Backend-for-Frontend (BFF). The canonical owned API surface set is: - **REST** — OpenAPI 3.2.0 - **internal-only gRPC** — proto3 over HTTP/2 - **event / async** — AsyncAPI 3.1.0 - **realtime** — public SSE (one-way server push) and WebSocket (bidirectional); gRPC streaming is internal-only All of these are gener

### ADR-634 residual

**Approval attaches to the PRODUCER of a change, not to a reader of its diff: a mechanical auto-approval predicate over de** — **Proposed — 2026-08-02.** Landed `Proposed`, not `Accepted`, for the reason ADR-0633 states in its own Status section: a fresh `Accepted` reddens `cloud-ci-cross-artifact-agreement` until the evidence it claims has propagated. Nothing in this ADR is enforced by its own merge; every decision below carries the assertion that would enforce it, and that assertion is the follow-up work.

### ADR-31 residual

**ADR-0031-ads-and-analytics-microservice-architecture** — We adopt a **singleton tenant-ads-gate sourcing rule** plus a **five-pillar Ads architecture** (Serving / Pricing / Attribution / Advertiser console / Publisher inventory) plus a **DP-budgeted Analytics architecture**. **Naming justification (BNF v4.1, ADR-0056):** - `oya-ads-gate-kernel`: slot2 = `ads` (registered µservice); slot3 = `gate` (BC); slot4 = `kernel` - `oya-analytics-event-router-work

### ADR-521 residual

**Staged W0-W6 fabric roadmap: convergence-first, interface-locking, cutover-gated; W4 bespoke-SCM gated by ADR-0510** — Ratify the founder-chosen staged roadmap. - **W0 (DONE):** hermetic buck2 build + de-cargo gates (fresh-checkout verified); firewall LIVE on `dev`; the JSON SSOT stores; kernel S4b/S4c/WAVE1 + carriers; transitional substrates (git, GitHub Actions, buck2, SeaweedFS/Ceph, cargo). - **W1 (NEXT):** convergence (ratify the design-corpus + the fabric / AST / owned-stack / safety decisions into clean AD

### ADR-104 residual

**Ecosystem-expansion principle for check-lane + adapter crate reintroduction** — **Ecosystem-expansion rule.** A crate is shipped iff: 1. The kernel/domain layer it implements is itself shipped, AND 2. At least one consumer in the workspace imports it, AND 3. The crate has a real implementation (not a doc-stub). If any condition fails, the crate is not shipped. Documentation of the trigger that would unblock the crate lives in this ADR, in `specs/masterplan.json`, and in the `

### ADR-588 residual

**Fail-closed verified-principal + PDP authorization for the audit.event.emit boundary (C15 tamper-evidence remediation)** — 1. **Two clean PORTS owned by the boundary crate** (`audit-usecase`), concrete adapters outside it (owned-W5 shape), in the new `audit/core/usecase/src/authz.rs`: - `PrincipalVerifier::verify_principal(&CallerCredential) -> Result<VerifiedProducerPrincipal, PrincipalVerificationError>` — caller authentication. `VerifiedProducerPrincipal` has PRIVATE fields and a `pub(crate)` constructor, so it can

### ADR-523 residual

**Zero-shell posture + the closed irreducible-glue ledger (minimal not zero; pinned; reproducible); refines ADR-0515 D3 no** — The lifecycle target is ZERO shell/CLI glue that does real work — every `build.sh` / `cargo` / Makefile orchestrator is a defect to retire into a buck2 target — EXCEPT a closed, authoritative **IRREDUCIBLE-GLUE LEDGER** of six items that genuinely cannot be a pure in-graph buck2 action: 1. **Toolchain bootstrap** — the buck2 binary itself + the first rustc/QEMU/musl downloads (a build tool cannot

### ADR-30 residual

**ADR-0030-search-microservice-architecture** — We adopt a **five-stage Search architecture** — Crawler → Parser → Indexer → Ranker → SERP — plus three cross-cutting subsystems (Query Understanding, Safety, Search↔Foundry/Ads bridges). Each stage is its own bounded context under `oya-search-<stage>-*`. Per-tier index segregation is enforced at the Indexer layer; cross-tier query is forbidden by default and gated by the Data Use Boundary policy

### ADR-305 residual

**ADR-0305-delegated-agent-authority-chain** — ### §B. Five core primitives at three layers The delegated-agent authority chain is **five core primitives** (token issuance; attestation chain; scope inheritance; cross-tenant block; audit linkage) wired at **three layers** (Tier-0 shared crate, per-µservice gate, Cedar policy fragment). The 5×3 matrix produces fifteen cells; each cell has a defined primitive. ``` Tier-0 shared Per-µservice Cedar

### ADR-544 residual

**Friction-ledger closed-loop accounting meta-gate** — Add `ci/facade/action-item-accounting` as a pure cloud-ci meta-gate. NAME: oya-cloud-ci-friction-accounting-app JUSTIFICATION: - microservice = cloud-ci: the cloud-ci admission product owns gate execution per ADR-0515. - bc-tokens = friction-accounting: the bounded concern is closed-loop friction-ledger accounting. - layer = app: the crate is an executable CI gate surface with a pure evaluator ker

### ADR-295 residual

**ADR-0295-bootstrap-ci-spiffe-kill-switch** — The keystone establishes seven decision sub-sections, D-1 through D-7. ### D-1. SPIFFE workload identity for every Stage-1 runner Every Stage-1 external CI runner — whether GitHub Actions, CircleCI, a temporary self-hosted runner, or a multi-cloud bake- in-place provisioner — receives a SPIFFE Verifiable Identity Document (SVID) issued by the one-shot bootstrap CA. The SVID binds the runner's iden

### ADR-243 residual

**ADR-0243-cedar-as-universal-gate** — ### D-1. Cedar evaluates every policy-class decision The 23 policy-class decisions enumerated in §Context are migrated to Cedar evaluation. New policy-class decisions introduced by future ADRs are Cedar from inception. The canonical Cedar evaluation contract: ```rust // microservices/policy-engine/src/api.rs pub struct EvaluationRequest { pub principal: Principal, // who is acting pub action: Acti

### ADR-636 residual

**Bound interim cross-run affected-set baseline reuse to immutable producer provenance** — 1. The affected-set job MAY reuse the exact build/test report pair from one completed canonical push-to-`dev` `oya-ci-required` run at the merge-base, even if that run's aggregate conclusion is red, only when the unique exact affected-set producer job completed successfully. 2. `actions: read` MUST be job-scoped to `gate-affected-target-set`; job permissions re-declare `contents: read`. No workflo

### ADR-633 residual

**Enforcement belongs to the layer that OWNS the fact: T1 mutation coupling, T2 non-emptiability, a promotion gate that ke** — ### D1 — a check belongs to the layer that OWNS the fact, and ownership is exactly two tests Assign each enforced fact to the **lowest-numbered layer that passes BOTH T1 and T2**. Not the earliest that *can express* it — the lowest that *owns* it. ``` T1 MUTATION COUPLING Changing the fact requires editing an artifact the enforcer reads. T2 NON-EMPTIABILITY No refactor that PRESERVES the fact can

### ADR-213 residual

**Ecosystem-as-a-Service architecture — Plugin/App Store substrate (third-party developer plugins/apps) + Developer SDK** — Oyatie ships an **Ecosystem-as-a-Service** product surface, composed of **two single-concern µservices** under the ADR-0131 flat layout, citing the industry inheritances listed in the frontmatter: ### 1. Two µservices, single-concern each (per ADR-0132) | µservice | Concern | Persona served | Inheritance | |---|---|---|---| | `microservices/plugin-app-store/` | Consumer-facing plugin/app discovery

### ADR-182 residual

**ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation** — Oyatie adopts a **two-substrate ingress model** with zero feature overlap: ### North-south (public → cluster): Envoy Gateway 1.8.0 The canonical north-south substrate is **Envoy Gateway 1.8.0** (CNCF; Kubernetes Gateway API v1.0 conformant; vendor-neutral; deployed as a dedicated `api-gateway` µservice per ADR-0157). Envoy Gateway owns: - **TLS termination** at the public edge (TLS 1.3; per-FQDN S

### ADR-10 residual

**ADR-0010-regional-pack-architecture** — We adopt **canonical-architecture + regional-pack plug-ins** as the locale model. The architecture is locale-agnostic; every per-locale concern lives in a regional pack that plugs into published seams. One pack per market, versioned and signed. ### Pack contents (per pack) | Pack section | What's inside | |---|---| | `regulatory` | Regulator names, control-mapping tables, evidence-collection caden

### ADR-595 residual

**De-commit the pure-derivation cloud-ci accounting faces — derive-on-demand + gate teaching** — STOP committing the six producer faces above. They are declared `materialization_mode: not-tracked-in-git` in `registry/generated-artifact-control-plane.json`, removed from git (`git rm --cached`), and covered by the existing `**/*.generated.json` ignore. They are derived on demand via `buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin` and materialize

### ADR-586 residual

**Fail-closed verified-principal + server-side PDP authorization for tenant.create and the tenant-lifecycle operator scope** — ### Common posture - **Default-deny**: no code path reaches the mutation/sensitive op without passing the gate. 401 without a verified principal, 403 without authorization. - **Unforgeable verified principal**: authority derives ONLY from a verified credential. Any `VerifiedTenantPrincipal` / membership-bound scope has **private fields + a `pub(crate)`-only constructor + accessors + a `#[cfg(test)

### ADR-546 residual

**Canonical-JSON determinism gate** — Add `ci/facade/canonical-json` as a pure cloud-ci determinism gate. NAME: oya-cloud-ci-canonical-json-app JUSTIFICATION: - microservice = cloud-ci: the cloud-ci admission product owns gate execution per ADR-0515. - bc-tokens = canonical-json: the bounded concern is deterministic JSON serialization. - layer = app: the crate is an executable CI gate surface with a pure canonicalizer kernel. - single

### ADR-129 residual

**ADR-0129-changeset-plan-dag-and-honest-claims-gate** — The existing ImplementationPlan frontmatter `id` is the canonical ChangeSet ID. No separate `changeset_id` field is introduced. The validator treats these fields as the exact ChangeSet graph contract: | Field | Status | Meaning | |---|---|---| | `doc_class` | required | Must be `ImplementationPlan`. | | `id` | required | Canonical ChangeSet ID, matching `Mxx-Pxx-IP-xxx` with optional numeric suffi

### ADR-630 residual

**Actions Runner Controller as the interim owned-runner substrate, declared in infra/arc/, behind the ADR-0515 D5 owned-CI** — ### D1 — ARC (`gha-runner-scale-set`) is the interim runner substrate Adopt `actions/actions-runner-controller` charts, **pinned at 0.14.2**, as the mechanism that turns the owned cluster into GitHub Actions capacity. It is Apache-2.0, Kubernetes-native (CRD + controller + reconciliation, no imperative agent lifecycle), and authored by the platform we currently integrate with — which is precisely

### ADR-567 residual

**Commission auth durable stores with Postgres + RLS (tenant-lifecycle-store-postgres and identity-scim-store-postgres)** — Commission two Postgres adapter crates and their service-catalog registrations: ### D1 — tenant-lifecycle-store-postgres `tenancy/adapters/tenant-lifecycle-store-postgres` — a `TenantLifecycleStore` realization backed by sqlx + Postgres with RLS enforced at the database layer. Every transaction sets the `oyatie.tenant_id` session GUC via `SET LOCAL` before touching any data row. Two RLS policies p

### ADR-563 residual

**Rename-aware path-keyed CI baseline relabel at the scm-facts emitter — the systemic productization that unblocks strangl** — Fix the staleness at the SINGLE sanctioned git boundary — the scm-facts emitter (ADR-0515 D3) — with a content-aware RELABEL of the FROZEN merge-base snapshot's path-keyed keys, driven by an AUTHORITATIVE committed move-manifest emitted by the reorg codemod. The firewall stays byte-for-byte UNCHANGED (pure DATA-over-DATA on opaque string keysets): this is data-over-data, not a firewall code change

### ADR-28 residual

**ADR-0028-cloud-microservice-architecture** — We adopt a **three-phase compute trajectory** with a **phase-invariant product surface**. Customers consume the same APIs, the same SKUs, the same IAM model, and the same audit shape regardless of whether the underlying capacity is rented, leased in a colo, or owned in a mega-DC. **Naming justification (BNF v4.1, ADR-0056):** - Cloud µservice crates: `oya-cloud-<bc>-<layer>` where `cloud` is the r

### ADR-248 residual

**ADR-0248-amazon-shape-cellular-architecture** — The platform adopts AWS cell-based architecture as the canonical topology. Sixteen decisions follow. REGCLOUD-001 planning-artifact registration: the non-mutating planning/spec artifact `plan/compliance-selective-cell-placement-architecture.md`, its ownership seed `plan/OWNERS`, and its multispectrum evidence packet `evidence/multispectrum/regcloud-001-compliance-placement-20260701-1782912506.json

### ADR-559 residual

**Commission the cloud-iam Cedar PDP service (G004 slice 1): a runnable authorization-decision service over the shared emb** — Commission `cloud/cloud-iam`'s policy-decision-point service as G004 slice 1: a **runnable authorization-decision service** in the ADR-0550 kernel/adapter/app shape, reusing the shared PDP kernel + Cedar adapter wholesale (zero forked evaluation logic). ### D1 — Service shape (ADR-0550 seams) - `oya-cloud-iam-pdp-kernel` — pure ports: `PolicyBundleStore` (policy-store backend seam), `DecisionAudit

### ADR-334 residual

**ADR-0334-shorts-microservice-merged-into-social** — D-1. `microservices/shorts/` is retired as a standalone µservice. D-2. `microservices/shorts/` keeps only a `RETIRED.md` redirect marker. D-3. Historical shorts service content is not the live authority after this ADR. D-4. `microservices/social/` is the canonical owner of short-form video. D-5. `microservices/social/` is the canonical owner of long-form video where it appears in the social produc

### ADR-35 residual

**ADR-0035-workflow-engine-state-machine-and-dag-hybrid** — We build `crates/oya-workflow-*` as the canonical workflow engine for the entire ecosystem. The engine is a **hybrid state-machine + DAG**: at the top level, every workflow is a state machine; within each state, computation can be expressed as a DAG. Per-tenant workflow definition versioning is first-class; per-jurisdiction overlays bind at runtime via the regional-pack architecture. ### Engine ar

### ADR-383 residual

**ADR-0383-observability-stack-reconciliation-loki-tempo-mimir-grafana** — **KEEP** the Grafana Labs Loki / Tempo / Mimir / Grafana stack as the canonical observability storage and visualization layer, subject to all three of the following gates: 1. **Fully self-hosted in oya-cells.** Every Grafana Labs component runs inside an oyatie-operated cell. No traffic is routed to Grafana Cloud SaaS. No managed-service dependency on Grafana Labs infrastructure is introduced. The

### ADR-380 residual

**ADR-0380-ci-loop-closure-on-talos-jenkins-farm-re-establishment** — Sequence the re-establishment into five deliverables (D1–D5): 1. **D1**: Install the gating plugins (generic-webhook-trigger + build-token-root + http_request + git) via `helm upgrade` of the bring-up-managed Jenkins with `installPlugins` extensions in `values-local.yaml`. Reboot once; CasC error-on-conflict means the existing oya-ci-farm cloud config must not be re-declared in any new configScrip

### ADR-530 residual

**The enforced engineering-excellence property set: falsifiable structural gates + fenced advisory remainder over the owne** — Extend the floor-gate family (ADR-0515) with the named engineering-excellence property gates, all on the same `Finding` / `remediate()` contract (ADR-0528) + shrink-only ratchet, each split into a gated falsifiable structural core and a fenced advisory remainder (`advisory-until-infra` with an `infra_prereq` corpus, never flipping the verdict). FULLY GATED: (1) documentation — `cloud-ci-doc-covera

### ADR-612 residual

**buck2 Remote-Execution phase: deploy nativelink-scheduler + nativelink-worker, flip remote_enabled=true behind per-ident** — Each decision below is the **ratified** design for the corresponding position; the "Decisions ratified" section records each disposition. Nothing here changes live pipeline behavior — like ADR-0560, this lands as deployable declarative artifacts + dark opt-in wiring + conformance tests, and claims no running deployment. ### D1 — Deploy the scheduler + worker tiers (the two reserved K8s tiers) Two

### ADR-157 residual

**ADR-0157-api-gateway-tier** — Oyatie adopts a dedicated **`api-gateway` µservice** as the canonical north-south entry tier. Every external HTTPS REST or realtime request transits the api-gateway tier before the cell-µservice tenant-routing layer hands it to a workload µservice. ### Operational shape 1. **Termination.** TLS 1.3 termination at the api-gateway edge (cert rotation per ADR-0064 canonical-base + per-pack overlay). H

### ADR-366 residual

**ADR-0366-agentic-high-throughput-self-enforcing-pipeline** — ### 1. Parallelism with conflict PREVENTION (not just resolution) A **single-threaded owner-agent per service/lane** (AWS STO) owns **disjoint paths** — the flat / no-grouping doctrine (ADR-0362) makes service paths naturally disjoint. One **isolated worktree per lane**. A **concurrent-safe-paths** admission gate rejects two in-flight lanes touching the same path. Cross-cutting changes flow throug

### ADR-29 residual

**ADR-0029-connect-dual-context-architecture** — We adopt as a **suite of twelve canonical apps** plus three adjunct surfaces, each its own bounded context under `oya-connect-<app>-*`, sharing the six substrates from ADR-0001 plus a Connect-internal **document-format kernel** and **collab-runtime kernel**. **Naming justification (BNF v4.1, ADR-0056):** - `oya-mail-kernel`: slot2 = `connector` (registered µservice); slot3 = `mail` (BC); slot4 = `

### ADR-392 residual

**ADR-0392-buck2-canonical-build-graph** — 1. **Buck2 + `buck2-prelude` + Reindeer-buckified third-party is the canonical build graph.** Buck2 (the Rust binary) drives the build/test action DAG with content-addressed, graph-exact incrementality. `buck2-prelude` supplies the first-party Rust toolchain rules. This reverses ADR-0358 §2's "Bazel `rules_rust` build graph"; everything else in ADR-0358 stands. 2. **Reindeer buckifies `Cargo.lock`

### ADR-128 residual

**ADR-0128-hyperscaler-architecture-invariants** — `specs/hyperscaler-architecture-invariants.json` (spec_id: EXE-HYPERSCALER-ARCH-INVARIANTS, version 1.0.0) is the canonical, machine-readable, binding source of truth for what "hyperscaler-grade" means in the Oyatie portfolio. This PR lands the catalog validator; it does not claim that product PRDs are already blocked on the catalog. Binding rules: 1. **Portfolio-wide applicability.** All 11 produ

### ADR-139 residual

**ADR-0139-agentic-slo-gated-promotion** — oyatie adopts a two-layer design: **adopted OSS observability runtime (Layer A)** plus **oyatie owned agentic-gate differentiator (Layer B)**. Both layers ship together as one M01 phase; neither is scheduled-for-distinct-tracked-work. The deployment substrate is the canonical Grafana stack, self-hosted; the gate logic is a new oyatie µservice `observability` with the BNF v4.1 crate family `oya-obs

### ADR-190 residual

**ADR-0190-scim-2-provisioning-enterprise-tenants** — **Identity µservice exposes a SCIM 2.0 RFC 7643/7644 endpoint at `/scim/v2/{tenant}` per tenant. Inbound provisioning from Okta / Entra / Workspace pushes Users + Groups; lifecycle states (active, suspended, deleted) propagate. A pluggable adapter contract (`HrisAdapter` trait) handles non-SCIM HRIS sources by translating to internal SCIM-shaped operations.** ### SCIM 2.0 surface | Endpoint | Meth

### ADR-175 residual

**ADR-0175-tenant-lifecycle-workflow** — ### D-1. Canonical six-state machine ``` Pending ──onboard_saga──▶ Active ──┬──suspend_saga──▶ Suspended ──unsuspend_saga──▶ Active │ └──migrate_saga──▶ Migrating ──migrate_completion──▶ Active (in target cell) │ └──offboard_saga──▶ Offboarded │ delete_saga │ ▼ DeletionConfirmed ``` State semantics: | State | Meaning | Allowed transitions | | --- | --- | --- | | `Pending` | Tenant record created;

### ADR-325 residual

**ADR-0325-capability-tier-pricing-anchors-public** — The per-tier anchor table is below. All amounts in USD, expressed as monthly recurring revenue (MRR) per tenant per category. Annual prepayment carries a 12% discount (D-3); BYOK carries 15% discount on the LLM-cost component (D-4). | Tier | Plugin | App | Workflow | Agent | Model | Dataset | |-----------|--------|--------|----------|--------|--------|---------| | Bronze | $39 | $99 | $149 | $199

### ADR-135 residual

**ADR-0135-aspirational-enforcement-gate** — `cloud-ci/Rust gate packet aspirational-enforcement` scans the normative docs, specs, and registry corpus for binding enforcement claims that name repository enforcement surfaces. The default corpus roots are: - `docs` - `specs` - `registry` Callers can narrow or replace coverage with `--clear-default-corpus --corpus-root <path>` for fixture and local validation. Production CI and branch-protectio

### ADR-606 residual

**Operator secret-bootstrap RBAC gate (least-privilege secrets + declarative join-token provisioning)** — Ship a **self-contained cloud-ci gate**, `cloud-ci-operator-secret-bootstrap` (`ci/facade/operator-secret-rbac`), mirroring the registration footprint of the authz-coverage (ADR-0566) and supply-chain-audit (ADR-0605) gates: own crate, own policy JSON, one appended matrix line in `.github/workflows/oya-ci-required.yml`, no `libs/oya-ci-config` edit, no producer-face binding. ### D1 — Pure, policy-

### ADR-309 residual

**ADR-0309-detection-fairness-audit-civil-rights** — ### §B. Five fairness invariants — substrate-enforced Establish the canonical five fairness invariants as substrate-level gates enforced before any ML model serves production traffic in the detection substrate (per ADR-0307) or any other product-facing AI surface. ### §B.1. Invariant 1 — No proxy discrimination Features that proxy protected classes MUST be flagged + either excluded or explicitly j

### ADR-374 residual

**ADR-0374-ci-webhook-gateway-github-actions** — Build a **flat single-concern Rust microservice**, `ci-webhook-gateway` (`microservices/ci-webhook-gateway/`, `src/` root per ADR-0131; package `oya-ci-webhook-gateway-app`), that is the FIRST hop of the gated pipeline: 1. **Receive** GitHub webhook deliveries at `POST /webhook/github` (axum/Tokio/ Tower/Hyper — blessed runtime deps). 2. **Verify** the `X-Hub-Signature-256` (or legacy `X-Gitea-Sig

### ADR-360 residual

**CI/CD pipeline optimization program — affected-target precision, gate-only overlay, warm shared cache, test sharding, pi** — Adopt a seven-part CI/CD optimization program. Each part has a hard correctness rule so optimization never weakens the governance gates. - **O1 — Affected-target precision.** Add an additive `oya verify --affected [--base <ref>]` presubmit mode. Classify the changed-file set vs the base into: **Full** (any of `Cargo.lock`, root/`[workspace]` `Cargo.toml`, a `workspace-hack` manifest, `rust-toolcha

### ADR-522 residual

**Lifecycle-wide hermeticity: one buck2 graph, four runners (build·CI·CD·dev-env)** — `buck2 build //...` and `buck2 test //...` are the SINGLE source of truth for what gets built and verified across the entire engineering lifecycle. BUILD (dev), CI, CD, and DEV-ENV are four **RUNNERS** of that one target graph; the only per-runner difference is a THIN, ideally GENERATED adapter (a forge YAML, an owned-runner control-plane Job spec, an Argo CD manifest, a `.cargo`/`.envrc` shim). T

### ADR-215 residual

**ADR-0215-multi-context-platform-architecture** — Adopt a multi-context principal model across Oyatie. One human principal can hold multiple active data contexts simultaneously: - `work-context-{employer}` for each employer or client relationship; - `personal-context` for B2C artifacts; - `healthcare-patient-context`; - `healthcare-provider-context`; - `education-student-context`; - `government-citizen-context`; - future sector contexts admitted

### ADR-549 residual

**oya-buck-syntax-kernel: one sound BUCK/Starlark parsing oracle + fixer self-validation harness** — Extract **`libs/oya-buck-syntax-kernel`**: the single shared, SOUND lexer/parser for the Starlark subset the gates consume, plus span-accurate safe-edit primitives and the fixer self-validation harness. Migrate the two cloud-ci gate consumers onto it. ### D1 — Sound parsing core (bespoke rowan-style; W2 doctrine) A hand-rolled lexer + recursive-descent parser, std-only, with byte-exact spans on ev

### ADR-204 residual

**ADR-0204-workflow-studio-canvas-library** — Per stack: | Stack | Phase 1 canvas | Phase 2 canvas (in-house) | |---|---|---| | SvelteKit (web, Phase 1) | **svelte-flow** (`@xyflow/svelte`, MIT) | `oya-canvas-svelte` (built on Svelte 5 + signals + SVG/Canvas2D) | | Leptos (web, Phase 2) | (skipped — Leptos web ships Phase 2) | `oya-canvas-leptos` (Rust-native, Leptos signals + SVG/Canvas2D + WebGL escape hatch for >5k nodes) | | SwiftUI (Appl

### ADR-117 residual

**ADR-0117-repo-hygiene-gitignore-audit-config-and-kyverno-consolidation** — 1. Add `.audit/` to `.gitignore` and untrack `.audit/agent-read.jsonl` via `git rm --cached`. Session-scoped audit logs stay local-only. Keep `.config/nextest.toml` tracked because it is CI configuration, not per-developer config. 2. `git mv deploy/gitops/oya-vcs-admission infra/kyverno/oya-vcs-admission` (history-preserving), removing the now-empty `deploy/gitops/` and `deploy/` parents. Rewrite

### ADR-529 residual

**The AUTO/ADVISE/GATE safety governor made operational at the gate layer (per-code tier DATA + meta-gate + advisory-until** — Ratify the founder safety governor that bounds the automation-first default of ADR-0528, operational at the gate layer. Every finding-code is classified into exactly one tier, stored as DATA per code (in `gate-disposition.json`), and a meta-gate REJECTS any registered code with no tier tag: - **AUTO** iff (deterministic ∧ behavior-preserving ∧ mechanically-falsifiable ∧ reversible+idempotent ∧ rev

### ADR-639 residual

**Path- and event-conditional constituents under the single oya-ci-required fan-in** — ### D1 — Singleton preserved Branch protection and merge admission continue to require **only** the context name `oya-ci-required`. No second required check for “docs CI,” “fast CI,” or “full CI.” Optional legs remain **constituents** of the same fan-in aggregator. ### D2 — Leg classes Every job listed in the `oya-ci-required` fan-in `needs:` list MUST be classified as exactly one of: | Class | Sy

### ADR-618 residual

**Contract-slice conformance gate scope boundary: single-document internal-shape validation, cross-reference/registry inte** — **The contract-slice conformance gate validates the internal shape of one committed JSON document per slice (extendable to N documents evaluated in isolation via an optional `additional_specs`, with no joins between them). Cross-document joins, cross-fixture negative joins, filesystem path-existence, non-JSON/YAML corpora, and full JSON-Schema-instance validation (Group C, items C1–C5) are out of

### ADR-123 residual

**ADR-0123-hyperscaler-maturity-claim-gate** — Use `/specs/hyperscaler-gates.json` as the machine-readable maturity claim registry. The exact phrase "we are hyperscaler mature" is forbidden unless the registry claim rule is allowed and all required gates have fresh evidence. Add the repo-native gate: ```text oya gate validate hyperscaler-maturity-claims ``` The gate validates: - Required maturity gate IDs, including plan, pipeline, toolchain,

### ADR-533 residual

**The config-driven public boundary: profile/neutral_default() + schema_version + faces_dir/cross_artifact/test-harness po** — Generalize the proven closed-schema `oya-ci-config` seam into a policy-FREE public boundary. Five additive changes (all preserve `#[serde(deny_unknown_fields)]`): 1. Split `bundled_default()` into `OyaCiConfig::neutral()` (empty forbidden_stems, no required_prefix, generic root_markers defaulting to `.git`, no governance_lanes, gates present-but-quiet, ZERO oyatie path literals) vs `OyaCiConfig::o

### ADR-40 residual

**ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback** — We adopt **Argo Rollouts** as the canonical progressive-delivery controller; **canary 5% → 25% → 50% → 100%** as the default stage progression; **metric-gated rollback** at SLO 1h burn-rate ≥ 14.4× (Sev-1-class trigger); **blue-green** for stateful surfaces; **per-region phased rollout** as the geographic progression; **per-cell rollback** as the unit of revert. ### Argo Rollouts as canonical cont

### ADR-517 residual

**One owned AST substrate (tree-sitter-our-way, rowan-style, content-addressed) read by every consumer; one work-area hash** — OWN the parser. Build a single bespoke AST substrate in safe Rust — **"tree-sitter-our-way," rowan-style, with content-addressed node identity** — NOT a reuse of tree-sitter. ONE substrate is read by every consumer: work-area identity (locking), practice enforcement (the GATE = AST queries for hyperscaler / cloud-native patterns and anti-patterns), auto-remediation (AST rewrites), doc-tracking, an

### ADR-519 residual

**AUTO/ADVISE/GATE safety governor: safety (not convenience) decides the automation tier; per-finding-code tier as DATA, m** — Every quality finding-code carries a mandatory automation tier, decided by **SAFETY, not convenience**, stored as DATA per code (in `gate-disposition.json`), with a meta-gate that REJECTS any registered code with no tier tag: - **AUTO** iff deterministic ∧ behavior-preserving ∧ mechanically-falsifiable ∧ reversible+idempotent ∧ reviewable → auto-fix / auto-gen PR. - **ADVISE** iff semantics-changi

### ADR-91 residual

**ADR-0091-governance-write-gate-foundations** — The write-gate kernel owns the canonical write-gate state machine: ``` Proposed → Reviewed { reviewer } → Approved { approver } → Executed \ / +----------> Rejected { reason } <--------------+ ``` Linear forward path: `Proposed → Reviewed → Approved → Executed`. Any non-terminal state may transition to `Rejected`. `Executed` and `Rejected` are terminal. ### Default-deny Every newly proposed gate s

### ADR-582 residual

**DTO-authz-trust gate (caller-supplied authorization decision backstop)** — Ship `ci/facade/caller-supplied-authorization`, a born-blocking cloud-ci gate that makes a NEW instance of the caller-supplied-authz-trust antipattern IMPOSSIBLE to ship while frozen-baselining the existing debt (shrink-only). It is a SIBLING of the authz-coverage gate (ADR-0566) and registers in the same `oya-ci-required` matrix gate family. It mirrors the kernel-purity (ADR-0547) / port-placemen

### ADR-56 residual

**ADR-0056-rust-clean-architecture-bnf** — ### Canonical BNF v4.1 ```bnf crate ::= "oya" "-" microservice ( "-" bc-tokens )? "-" layer | "oya" "-" "check" "-" rule-name microservice ::= kebab-token ( "-" kebab-token )* (* 1..3 tokens; registry-validated *) bc-tokens ::= kebab-token ( "-" kebab-token )* (* 0..N; OPTIONAL *) layer ::= "kernel" | "domain" | "usecase" | "app" | "adapter" | "infrastructure" | "cli" | "rest" | "grpc" | "graphql"

### ADR-536 residual

**Hyperscaler-grounded substrate decision matrix (FD-001 + cloud substrate) — sixteen normative domain decisions, each wit** — Adopt the following sixteen normative domain decisions as the substrate contract for FD-001 and the cloud substrate. Precedent is cited inline; each domain names its rejected anti-patterns. ### D-1 Identity provider (IdP) **Decision.** Single-homed write control plane + cell-replicated offline-verify authentication data plane: all identity writes (principal, credential, policy-binding mutations) c

### ADR-545 residual

**Embedded-asset hermeticity gate** — Ship a standalone, born-blocking, pack-shaped cloud-ci gate `cloud-ci-embedded-asset-hermeticity` (crate `ci/facade/embedded-asset-hermeticity`) that mirrors the ADR-0544 gate family (pure kernel + policy DATA + reviewed shrink-only baseline + a `*-gate` rust_test self-test). The kernel contract (the **tree-namespace rule**): - **D(T)** = { package-relative short paths of every plain/glob/list src

### ADR-590 residual

**Fail-closed verified-principal + server-side PDP authz for the Cloud Observability audit-read surface (C18 / AUTH-005 re** — Close both gaps by applying the proven fail-closed doctrine from ADR-0572 (`iam/ports/policy-cedar-api`, #815) and the secrets KMS-API boundary (#817), adapted to this pure-library boundary (this crate has no HTTP router; the boundary is a function the facade calls): 1. **Verified, unforgeable principal.** A new `authz` module owns a `PrincipalVerifier` PORT producing a `VerifiedPrincipal` whose f

### ADR-554 residual

**Binding buck2 coverage for the full workspace: affected-set lane with fail-closed full-run escalation** — Ship the **tiered** design (option c), evaluated against the alternatives below: - **D1 — Binding affected-set lane on every PR.** A new bespoke gate lane `gate · affected-set (ADR-0554)` in `oya-ci-required.yml` (additive; fan-in `needs:` wired; registered per the gate-registration meta-test as a Buck target lane) derives the merge-base diff's target cone via `buck2 uquery owner()` (per-file, bat

### ADR-587 residual

**Fail-closed verified-principal + PDP authorization for the Cloud Network LB/VPC/DNS create control planes** — For each of the three boundary crates, add an in-crate `authz` module with two clean PORTS owned by the boundary crate; the concrete adapters (cloud-iam PDP client, mTLS/SPIFFE or bearer credential store) live OUTSIDE the crate (owned-W5 shape, so the port shapes do not change at cutover): 1. **`PrincipalVerifier::verify_principal(&CallerCredential) -> Result<VerifiedPrincipal, _>`** — caller auth

### ADR-613 residual

**De-commit the remaining controller-materialized projection faces (masterplan + product-graph) — finish the pure-derivati** — STOP committing `docs/machine-readable/masterplan.generated.json` and `docs/architecture/product-graph.html`. Both are declared `materialization_mode: not-tracked-in-git` in `registry/generated-artifact-control-plane.json`, removed from git (`git rm --cached`), and covered by `.gitignore` (masterplan by the existing `**/*.generated.json` rule; product-graph.html needs an explicit line — it is a `.
