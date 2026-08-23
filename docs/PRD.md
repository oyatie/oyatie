---
purpose: Canonical product requirements entry point for Oyatie product scope, users, capabilities, constraints, and acceptance boundaries.
doc_status: published
---

# Oyatie — Product Requirements Document (PRD)

## Doctrinal authority — [decision-principles.json](../specs/decision-principles.json) + [forbidden-operations.json](../specs/forbidden-operations.json)


> **Brand:** Oyatie (logo: `oYa`, domain: `oyatie.com`).
> **Repo:** `jason931225/oyatie` (slug + filesystem path retained per ADR-0017; *only* the product/package/UI brand is rebranded — see [ROADMAP.md](ROADMAP.md) "Rename batch").
> **Status:** Draft v0.2 — 2026-05-09. Authoritative-deep consolidation. Expected to grow over the next two consolidation revisions.
> **Owners:** Architecture council (cross-axis); Founder Jason Lee (north-star arbiter).
>
> **2026-05-09 consolidation:** Two changes happened the same day. (a) The former Builder-OS / engineering-platform axis was folded into Foundry per ADR-0025; Foundry is now the unified "AI agent runtime + control plane + engineering platform" axis with multi-provider support (Anthropic Claude / OpenAI / Gemini, both subscription auth and API auth). (b) Workspace / Productivity Platform was added as Axis 2 (Mail / Docs / Sheets / Slides / Drive / Calendar / Meet / Chat / Forms / Sites / Tasks / Notes / Translate / Recordings — Google Workspace / Naver Works / Microsoft 365 / AWS Productivity class). **Net result: axis count is 7.** See [DESIGN.md §1](DESIGN.md) for the canonical 7-axis table and [DESIGN.md §3](DESIGN.md) for the consolidated Foundry surface.

> **Portfolio-parent citation (A1):** Bominal's consolidated PRD is the portfolio-parent strategy surface for this product-family lineage; this Oyatie PRD is the canonical implementation PRD for the `oyatie/` repo.
>
<!-- portfolio-citation:start -->
- role: PortfolioParent
  target_path: bominal/docs/consolidated/PRD.md
  target_repo: bominal
  target_prd: docs/consolidated/PRD.md
  anchor: product-requirements-document
<!-- portfolio-citation:end -->

---

## 1. North Star

**Oyatie is one cohesive ecosystem-as-a-service.** Not a portfolio of products. One product expressed across **seven** axes that interlock at well-defined contracts (Foundry consolidates the former Foundry engineering platform axis on 2026-05-09; Workspace / Productivity Platform was added as Axis 2 on 2026-05-09):

> *Run your business on Oyatie's vertically-tuned SaaS, hosted on Oyatie's own cloud, indexed and discoverable through Oyatie Search, monetizable through Oyatie Ads — all governed by one tenancy model, one identity surface, one capability registry, one audit chain, one autonomy ceiling, one foundry that compounds engineering quality. The same agent runtime that runs your workflows also operates the cloud, indexes the search corpus, and bids in the ad auctions.*

The strategic premise: a single tenant boundary, single identity, single audit chain, and single agent runtime spanning every layer is *more valuable than the sum of best-of-breed substitutes* because it removes the integration tax that every multi-vendor stack pays. Korea-as-launch-locale is the test bed; global is the follow-on.

### Why now

1. **AI-agent runtimes are vertically integrating.** A SaaS without an agent runtime, a cloud without an agent runtime, or a search engine without an agent runtime each lose to the integrated stack within 5 years. Oyatie's bet is to integrate first.
2. **Cloud sovereignty is a global window, not a Korea-specific one.** Korea is mid-cycle on CSAP / KISA / PIPA + 망분리 + 전자세금계산서; Japan is mid-cycle on 政府情報システム / Information System Security Management & Assessment Program (ISMAP); EU on GAIA-X / EU Data Act / DORA; India on MeitY empanelment + DPDP Act; Brazil on LGPD + ICP-Brasil; KSA on PDPL + NDMO + SDAIA controls; UAE on TDRA + ADGM data laws. Every major market has a *similar* regulatory moat that prefers in-locale integrated providers over US hyperscalers. Oyatie's posture is **canonical-architecture + regional-pack plug-ins**, so we can ride every market's window in parallel rather than serializing on Korea.
3. **The compliance-graph compounds.** A single tenancy model, one audit chain, one consent surface, one DSR pipeline lets each new vertical (healthcare → industrial → fintech → public sector) inherit the predecessor's regulatory machinery rather than rebuild it.
4. **The catalog/registry-driven engineering substrate already exists** (per ADR-0015 architectural-flattening-target, ADR-0011 cross-axis-contract-registry, ADR-0025 foundry-as-engineering-platform, capability records per ADR-0021, claim-ceiling validator per ADR-0022). Oyatie can stand up new axes onto the existing foundry rather than from scratch.
5. **Automation-first principle (Google + Amazon doctrine).** What can be automated **must** be automated. Chores, pipelines, repetitive operations are the highest-yield optimization surface — *especially* given Rust's slow build time and the concurrent-agent execution model. The git / CI/CD / PR pipeline is where this matters most: every additional minute of build-cache hit, every additional parallel-agent dispatch with contained blast radius, every auto-rebased / auto-reviewed / auto-merged PR compounds across thousands of CI runs per week. Specific commitments: sccache + Bazel-remote-class remote execution; per-crate affected-graph testing; per-agent worktree isolation with branch-name collision detection; merge-queue with one-PR-at-a-time root-Cargo-touch; auto-rebase + auto-review-bot + auto-merge gates; CI-time budget per lane; Foundry-driven PR triage / labeling / changelog drafting / release-note authoring; per-test flaky-quarantine; nightly affected-rebuild on `main`. See [DESIGN §3.0.5](DESIGN.md) and [TOOLCHAIN §5](TOOLCHAIN.md).

6. **Compute trajectory: OCI + AWS first, scale to our own mega-datacenter.** Phase 1: every Oyatie surface runs on OCI (per ADR-0044 cloud foundation + ADR-0009 cell architecture) and AWS as the consumed hyperscaler substrate. Phase 2: as scale economics flip, capacity moves to Oyatie-operated colo + (long-horizon) own mega-datacenter. The Cloud axis is BOTH (a) the in-house compute substrate we *consume* AND (b) the cloud product we *sell* to customers. Both halves share the same control plane; the underlying compute substrate evolves over waves without changing the product surface.

7. **Foundry early is the force multiplier.** Building Foundry (the AI agent runtime + control plane) early accelerates *every other axis* exponentially: Foundry agents run the cloud control plane, populate the search index, author vertical workflows, operate the ad auctions, and execute the Foundry lanes. Every additional month before Foundry preview is a month of linear-only progress in all six other axes. *This makes Foundry a P0 force-multiplier — not just one axis among seven, but the substrate that compresses the build-out of the other six.* See [DESIGN.md §3 Foundry-as-accelerator](DESIGN.md) and [ROADMAP.md W-Foundry-Preview](ROADMAP.md).

### Non-goals (explicit)

- **Not a generalist AGI lab.** Frontier-LLM R&D (GPT / Claude / Gemini-class general-purpose pre-training) is consumed via Anthropic / OpenAI / Google Gemini providers, not produced. *(Note: in-house **production model training and inference** for Oyatie-specific tasks — vertical-tuned models, embedding models, KR-first NLP, speech, vision, safety/eval — is **in-scope at the W-AI-Model-Substrate wave** down the road, after vertical proof. See [DESIGN §3.0.1](DESIGN.md). The boundary: we train models for our product, not foundation-model R&D for sale.)*
- **Not a chip designer.** Custom silicon (ASIC / FPGA / proprietary CPU / GPU IP) is anti-scope; we use commercial silicon (NVIDIA, AMD, Intel, Ampere, ARM-licensed, future Korean / Asian fabs).
- **(Updated 2026-05-09 per user directive)** Building / operating own datacenters from scratch — including DC shell construction or full greenfield DC build-out — is **NO LONGER ANTI-SCOPE**. It is in scope at the long-horizon **W-DataCenter-Operations** wave (post-Cloud-Stable, when scale economics flip). Trajectory per [DESIGN §3.0.4](DESIGN.md): OCI + AWS hyperscaler now → Oyatie-operated colo at scale → own greenfield mega-datacenter.
- **Not a consumer social network.** Search & ads axes serve business intent, not consumer attention. *(Re-evaluate at W6+ if data shows organic consumer pull.)*
- **Not a multi-region day-one product.** Korea launches first; global is W4+. Cell-routing infrastructure exists from day one to make this reversible.

---

## 2. Target Users

| User class | Persona | What they get | What they pay for |
|---|---|---|---|
| **Tenant operator** | A KR Group HR director, a manufacturing plant manager, a clinic admin | Vertical-tuned SaaS workflows on regulated infrastructure with embedded search and AI assistance | Per-seat or per-volume SaaS subscription |
| **Tenant builder / IT** | A tenant's internal engineer or a partner | Workflow Studio, Ontology (legacy: Object Graph — renamed per MASTERPLAN.md §2.4), capability authoring, plugin marketplace consumption | Tenant-builder seats + plugin runtime usage |
| **External developer / ISV** | A KR adtech vendor, an integration partner, a vertical specialist | Public REST/SDK, OpenAPI, webhook console, plugin SDK, marketplace listing | Marketplace revenue share + API call metering |
| **Cloud customer (axis 5)** | A startup or large enterprise that wants KR-resident IaaS without US hyperscaler exposure | Compute / storage / network / IAM / regions / billing — same surface as AWS/GCP/Azure | Cloud usage (per-resource-hour, egress, per-API-call) |
| **Search advertiser (axes 6+7)** | A KR DTC brand, an ISV reaching tenant-app users, a vertical agency | SERP slots, display network slots, intent targeting (privacy-gated), attribution | Ad spend (CPC/CPM/CPA auction) |
| **Internal engineer** | An Oyatie or partner engineer | Foundry surfaces (repoctl, catalog, claim-ceiling, scorecards, fitness functions, foundation-bypass ledger, plane-gated CI lanes) | (Internal — productivity is the price.) |
| **Regulator / auditor** | KISA, MFDS, PIPC, FSC, KCC, NIS, foreign equivalents | Evidence portal, audit trail, control-evidence packs, DSR/withdrawal pipelines | (Cost of doing business; satisfied by control evidence.) |

### Tenancy taxonomy

- **Tenant** = an organization with its own data boundary. PHI/PII/PCI never cross.
- **Organization hierarchy** = parent / subsidiary / business-unit (handled in cross-product auth, ADR-0006).
- **Workspace** = a team-scoped slice of a tenant.
- **Capability namespace** = the contract surface that a tenant binds to (search, ads, vertical-X, cloud-Y).
- **Plane** = control / data / analytics (ADR-0017) — every surface declares its plane.

---

## 3. Scope

### 3.1 Optimal-path scope (no time/resource constraint — drawing-board re-framing 2026-05-09)

> **Vocabulary update (2026-05-09):** the legacy milestone shorthand is retired (token mapping in [GLOSSARY.md §11](GLOSSARY.md) "Forbidden legacy terms" appendix). All in-progress docs and the v2 backlog are being rebuilt around the *optimal path forward* under unconstrained time and resources, **with global launch and scaling in mind from day one** — not a Korea-first plan with global retrofit. The new sequencing language is `Foundation → Substrate → Axis-Preview → Axis-Stable → Vertical-Preview → Vertical-Stable → Public-GA → Region-Fan-Out`. Status labels are `preview / stable / GA` (industry-standard) with no internal milestone numbering.

The **optimal path** assumes we can afford to build everything correctly the first time. The sequencing therefore prioritizes:
1. **Foundation correctness** before any product surface.
2. **Force-multiplier substrate (Foundry)** before any axis fan-out.
3. **Canonical-architecture + regional-pack plug-ins.** No locale is a special case at the architecture level; every locale is a **regional pack** that plugs into canonical seams. Korea, Japan, US, EU, India, Brazil, KSA, UAE, ANZ, SEA can all be onboarded *in parallel* once the seams are correct. See [DESIGN.md §12 Regional Pack Architecture](DESIGN.md).
4. **In-house build over external dep** wherever the dep is not as mature as `axum` / `tokio` / `serde` / a Postgres driver / OS kernel-grade tools. License posture is a hard gate — AGPL forbidden in product code; GPL forbidden in product code; Apache-2 / MIT / BSD / Mozilla-2 allowed; SSPL and BUSL require ADR review.
5. **Quality of contract** over time-to-launch. Schemas, public APIs, audit-chain, plane separation, tenancy must be correct before any commercial commitment.
6. **Optimization built in, not bolted on.** Cell-routing, partitioning, caching tiers, idempotency, bulk endpoints, batch dispatch, agent-driven self-optimization loops, build-cache hygiene, CI affected-graph testing, FinOps unit-economics from day one. See [DESIGN.md §15 Optimization Practices](DESIGN.md). Optimization that costs less when designed in is a structural invariant; optimization-as-afterthought is the failure mode.

#### In-scope (optimal-path waves — names are descriptive, not date-bound)

| Wave | Description | Axes touched | Gate to next wave |
|---|---|---|---|
| **W-Foundation** | Foundation correctness: tenancy kernel, identity, audit chain, plane separation, Data Use Boundary ADR, cell architecture, evidence emission, Cedar/policy substrate, schema-class annotation, fitness functions | cross-cutting | All foundation ADRs Accepted; fitness functions hard-fail on violations |
| **W-Foundry-Preview** | Foundry preview, encompassing both agent runtime AND consolidated foundry surfaces: SecretProvider/KMS; **multi-provider adapter** (Claude/OpenAI/Gemini × subscription-auth + API-auth); daemon hardening, smoke lane, capability registry, autonomy ceiling, evidence emission, RAG endpoint; **foundry surfaces** (catalog, claim-ceiling validator, foundation-bypass ledger, plane-gated CI lanes, repoctl, scorecards, fitness functions, branch-protection-as-code, signed commits, supply-chain Cosign+Trivy+SBOM, license-policy gate, plugin substrate trust gates) | agent-runtime (Foundry, consolidated) | Foundry runs ≥ N capabilities end-to-end with full evidence emission across all 3 providers in both auth modes; all cross-axis contracts have CI fitness checks; license-policy gate hard-fails on violations |
| **W-Cloud-Preview** | Cloud provider preview, *region-agnostic core* + first regional packs running in parallel (initial regions per council decision; KR-Seoul, JP-Tokyo, US-Northern Virginia, EU-Frankfurt as parallel candidate beachheads). Surfaces: IAM (Cedar + SSO + STS), region/AZ/cell taxonomy, compute (managed k8s + functions), storage (object + block + KMS-shred), network (VPC + LB + DNS + interconnect), billing (per-resource metering + per-region tax-invoice format via regional pack), observability (audit log + SLO dashboards) | cloud + cross-cutting | Cloud control-plane API frozen at v1; cell-isolation evidence collected; ≥ 2 regional packs onboarded |
| **W-SaaS-Preview** | SaaS shared-substrate preview: workflow engine, Ontology property tiers (legacy: Object Graph), plugin substrate (signing + sandbox), public REST API stability tier, webhook signing, plugin marketplace catalog | saas + foundry | Tenant onboarding + plugin install + marketplace listing all functional |
| **W-Search-Preview** | Search preview: pgroonga day-1, KR morphology (mecab-ko/khaiii), inverted index sharding, vector index (pgvector), tenant-private indexes, RAG endpoint exposed to Foundry, per-class data boundary enforcement | search | Search index lifecycle gated on Data Use Boundary ADR |
| **W-Vertical-Pilot** | First vertical full implementation as design-partner pilot (likely Corporate KR — HR/payroll/GL/mail — given existing depth, but vertical choice is a council decision under the new framing) | vertical | Pilot tenant runs end-to-end on the foundation+axes preview stack |
| **W-Vertical-Fan-Out** | Additional verticals built in parallel using Foundry-authored vertical packs | vertical | Each vertical has regulatory-pack adoption + control evidence |
| **W-Cloud-Stable** | Public cloud-provider GA: marketplace, ISV onboarding, multi-AZ failover automation, FinOps surfaces, KR CSAP + K-ISMS-P + KCMVP HSM in production | cloud | Public cloud SLA committed (99.99%) |
| **W-Search-Stable** | Public web search (crawler + freshness + KG + SERP), with sponsored-result slot infrastructure ready (ad serving still off) | search | Public search SLO + KR ranking quality bar |
| **W-Ads-Preview** | Internal ad-serving + advertiser console preview; tenant-facing-only at first; auction ML loops trained without cross-tenant data leakage | ads + analytics | Data Use Boundary ADR satisfied + per-tenant auction quality |
| **W-Ads-Stable** | External ad platform serving advertisers outside the current Oyatie tenant base | ads + analytics | Cross-tenant aggregate consent flows + KR adtech compliance evidence |
| **W-DataCenter-Operations** (long-horizon; sequenced when we operate physical or colocated DC capacity at scale, post-W-Cloud-Stable) | Datacenter Infrastructure Management (DCIM): per-rack inventory, capacity, power, cooling, PUE/WUE/CUE; BMS/BAS integration (HVAC, lighting, fire suppression, water-leak); power distribution monitoring (PDU/ATS/UPS/generator/fuel); cooling control (CRAH, chilled-water, free-air economization, hot-aisle containment); network ops (cable mapping, fiber budget, patch-panel inventory); physical security (badge access, CCTV, mantrap, environmental sensors); asset lifecycle (procurement, deployment, RMA, decommission, e-waste compliance); capacity + thermal planning; vendor + spare-parts inventory; workorder management (technician dispatch + SLAs); DC incident tracking; sustainability metrics + carbon accounting per region; regulatory (Uptime Institute Tier-III/IV, EU EN 50600, KR ISMS-DC certification, CSA STAR-Cloud) | cloud (DC-ops sub-axis: `crates/cloud-dcops-{dcim,bms,power,cooling,network,security,asset,capacity,workorder,sustainability}-*`) | First DC operationalized end-to-end on Oyatie DCIM stack; PUE within target |
| **W-Robotics-Vision-Speech** (long-horizon; runs in parallel with verticals that consume) | **Vision Intelligence** substrate (`crates/intelligence-model-vision-*`): OCR (multilingual + KR HWPX), document understanding, image classification, object detection, video analytics, scene/anomaly detection, facial recognition where lawful (per [PRIVACY-PROGRAM §2.2.3](PRIVACY-PROGRAM.md) tenant-class override), AMR/CCTV vision per ADR-0027 / ADR-0027. **Speech Intelligence** substrate (`crates/intelligence-model-speech-*`): STT (Whisper-class + Naver Clova Speech), TTS (XTTS / Naver Clova Voice / multilingual incl. KR voices), voice biometrics, wake-word, dialect adaptation; Meet transcription, voice-charting (healthcare), voice agents, contact-center per vertical. **Robotics** control plane (`crates/vertical-industrial-robotics-*` for fleet + `crates/intelligence-robotics-control-*` for agent-mediated control under autonomy ceiling): AGV/AMR fleet, robotic arms, drones (where lawful per anti-scope), autonomous vehicles, real-time control loops, simulation harness, fleet management. | foundry + vertical (industrial / logistics / healthcare / retail) + cloud (GPU + edge compute) | Per-substrate eval pass; per-vertical pilot proves end-to-end; safety-critical loops verified via simulator + staged real-world trials |
| **W-AI-Model-Substrate** (long-horizon, post-Vertical-Stable; assumes time/resource unconstrained) | In-house model training + inference substrate. (a) GPU fleet provisioning + scheduling on the Cloud axis; (b) distributed training (Megatron / DeepSpeed / FSDP-class); (c) inference serving (vLLM / TensorRT-LLM / in-house Rust serving); (d) data pipelines (per Data Use Boundary, training only on consented + public corpus); (e) safety + red-team + eval harness; (f) **first in-house models**: KR-first foundation LLM (alongside HyperCLOVA-X / Upstage Solar / EXAONE class), embedding models for Search RAG (alongside open-source `bge-large-ko` / `gte-multilingual`), STT/TTS for KR + JP + EN voices, vision models for OCR + Workspace doc understanding, vertical-tuned safety + eval models (clinical safety, KR-FSS compliance check). *Rationale: provider-adapter independence + cost control at scale + KR-data-residency-trained models for regulated tenants. Sequenced AFTER tenant proof so we know which task profiles to train for.* | foundry + cloud (GPU fleet) | Per-model: in-house variant outperforms or cost-matches the external provider on the per-vertical eval set |
| **W-AI-Model-Stable** (further out) | Public Foundry capabilities default to in-house models for the supported task profiles; external providers stay as failover under autonomy ceiling | foundry | per-capability eval pass + cost / latency parity |
| **W-Region-Fan-Out** | Adds regional packs in parallel (whichever markets are commercially ready: secondary KR regions, JP-Osaka, US-West, EU-Paris, EU-Stockholm, IN-Mumbai, BR-São Paulo, KSA-Riyadh, UAE-Dubai, ANZ-Sydney, SG-Singapore, etc.); cross-region replication contract per residency class | cloud + cross-cutting | Per-region regulator-equivalent (CSAP/ISMAP/FedRAMP/GAIA-X/MeitY/LGPD/NDMO/TDRA/IRAP) + residency contracts |

This wave list is the *target-state sequence*. ROADMAP.md decomposes each wave into bands and Foundry batches.

### 3.2 Out-of-scope at any wave (anti-scope, will not do)

- Hardware / chip / data-center construction (always — leased always; cloud axis sits on colo + leased racks).
- Consumer social network (always; reconsider only on substantial organic evidence).
- Selling raw tenant data to advertisers (always — even with consent).
- Targeting ads using PHI / PII / PCI / KR-신용정보 / KR-PIPA Art-23 sensitive data (always — see [PRIVACY-PROGRAM.md §2.2.1](PRIVACY-PROGRAM.md)).
- Multi-region day-one (foundation must be cell-routed first; expansion follows).
- AGPL/GPL code in product surfaces (license policy hard gate).
- General-purpose AGI lab; foundational model R&D is consumed (Codex / Claude / open-source), not produced in-house.

### 3.3 Anti-scope (will not do in any wave)

- Selling raw tenant data to advertisers. Period.
- Targeting ads using PHI / PII / PCI even with "consent." (Privacy posture stronger than Google's.)
- Deploying customer workloads on shared multi-tenant compute without cell-isolation evidence.
- Shipping any axis surface that contradicts another axis's contract — see Cross-Axis Contradiction Audit (DESIGN.md §10).

---

## 4. Success Metrics

### 4.1 First commercial-wave launch metrics (date TBD; council-set under unconstrained-time framing)

| Metric | Target | Why this number |
|---|---|---|
| KR Group Payroll tenants live | ≥ 3 design-partner groups | Below 3 = not a product, just a custom build. |
| Foundry agent runs in production | ≥ 50K / week, ≥ 99.5% success | Proves agent runtime is operationally real. |
| Audit-chain evidence completeness | 100% of regulated capability invocations | Below 100% = compliance posture is theater. |
| Plane-gated CI lane block rate | ≥ 1 block per 100 PRs | Proves the gates work; 0 means they're vacuous. |
| Internal cloud control-plane uptime | ≥ 99.9% on W-Cloud-Preview surfaces | Internal-only target — public-GA target is 99.99% at W-Cloud-Stable. |
| Capability namespace count under autonomy ceiling | ≥ 80% of regulated capabilities | Coverage of the policy surface. |
| Claim-ceiling validator ratchet | every wave promotes one WARN→BLOCK | Compounding engineering rigor. |

### 4.2 Multi-year structural metrics (W4-W6)

| Metric | Target | Why |
|---|---|---|
| Cross-axis contract violations on `main` | 0 detected per quarter | Cohesion guarantee. |
| Foundation-bypass ledger expiry SLA | 100% of bypasses retire within declared expiry | Bypasses must not become permanent. |
| Tenant data egress without consent receipt | 0 events ever | Hard zero. Audit chain failure-mode = unrecoverable. |
| Search index coverage of tenant content | per-tier consent target hit | Per-tier = consent gradient is real. |
| Ad auction RPM lift from semantic ranker | TBD when ads-axis lands W5 | Placeholder. |
| Regulatory evidence pack regeneration time | ≤ 4 hours from request | Auditor experience. |
| Mean time to provision a new region (cloud axis) | ≤ 2 weeks (post-W3 IaC profile) | Region = repeatable artifact, not bespoke. |

### 4.3 Non-metrics (explicit)

- **Lines of code** — never a target.
- **PR throughput** — never a target.
- **Headcount growth** — never a target.
- **Vanity DAU / MAU** for the SaaS surface alone — meaningless without per-vertical retention.

---

## 5. The cohesion thesis (why one product, not seven)

Each pair of axes shares a contract that would otherwise be a multi-vendor integration tax. Enumerated:

| Axes | Shared contract | What goes wrong if we separate |
|---|---|---|
| SaaS ↔ Cloud | Tenant resource lives in tenant's compute; tenant data resides in tenant's storage region; one billing trail; one IAM hierarchy | Two billing systems, two IAM models, two audit trails, two consent surfaces — the integration tax kills enterprise sales. |
| SaaS ↔ Search | Tenant content is search-indexable per consent tier; same Ontology (legacy: Object Graph — renamed per MASTERPLAN.md §2.4) powers both | Separate search index = stale, half-complete, requires per-tenant ingestion bespoke to each vertical. |
| SaaS ↔ Agent runtime | Workflows are agent-authored or agent-executed; capability registry is the same | Agent-runtime-as-feature = bolted-on chatbots; integrated = workflows are first-class agent surface. |
| Cloud ↔ Agent runtime | Foundry runs *on* the cloud and *operates* the cloud (control-plane API) | Without integration, agents can't manage cloud — the "AI manages cloud" story is empty. |
| Cloud ↔ Search | Search shards run as cloud resources; cloud capacity drives search index lifecycle | Without integration, search has its own infra story (cost + ops duplication). |
| Cloud ↔ Ads | Ad auction runs on cloud cells; ad inventory is cloud-billable like every other resource | Without integration, ads has its own infra (replication + cost + ops). |
| Search ↔ Ads | Sponsored slots in SERP; ad-quality score uses search-relevance signals; impression and click streams shared | Without integration, ads ranks by bid alone and SERP rotates between organic+sponsored as different products — bad UX. |
| Search ↔ Agent runtime | RAG endpoint exposes search to Foundry; agent decisions are query-driven | Without integration, every agent re-invents retrieval. |
| Ads ↔ Agent runtime | Agents bid on behalf of tenants (autonomy-ceiling-gated) | Without integration, agentic-buying is third-party automation, not native. |
| Agent runtime ↔ Foundry | Capability registry is one surface; autonomy ceiling is one policy | Without integration, foundry catalog and runtime registry diverge — every PR misroutes. |
| Vertical industry cloud ↔ each of the others | Vertical-specific privacy boundary is one contract per vertical, not one per product surface | Without integration, healthcare's de-identification has to be re-implemented in each surface. |

The PRD's central commercial claim: **the cohesion is the moat.** Customers can buy any axis individually and migrate to integrated; they cannot buy integrated from anyone else.

---

## 6. Constraints (hard)

1. **Global-canonical regulatory posture, regional-pack-driven.** The architecture is locale-agnostic. Every regulated surface declares its `regulatory_packs:` set in the catalog, and a regional pack supplies the per-jurisdiction implementation. Initial pack set: KR (PIPA/KISA/MFDS/FSC/KCC/NIS/CSAP/K-ISMS-P/KCMVP), JP (APPI/ISMAP), US (HIPAA/HITECH/SOX/CCPA-CPRA/StateAGs/FedRAMP), EU (GDPR/DORA/EU-AI-Act/GAIA-X), IN (DPDP/RBI/MeitY), BR (LGPD/ICP-Brasil), KSA (PDPL/NDMO/SDAIA), UAE (TDRA/ADGM), ANZ (Privacy Act/IRAP). Every axis ships with the *canonical* contract; every regional pack ships with the *jurisdictional* control evidence in its first commercial wave for that region.
2. **Audit-chain immutability.** Every capability invocation, every consent decision, every cross-axis data flow emits a tamper-evident record per ADR-0003 audit-chain-merkle-sealed-ed25519. *Without this, the cohesion thesis fails on first audit.*
3. **Tenancy isolation under formal proof.** PHI/PII/PCI never cross tenant boundary; cell-isolation evidence required for every axis. *Move #0* (per maturity 8-move program) is the substrate.
4. **Architectural flattening.** Per ADR-0015 architectural-flattening-target / Issue #1458, the codebase is migrating to flat `crates/oyatie-<context>-<role>[-<capability>]/`. No new `modules/` `services/` `platform/` tiers. Every consolidated doc and the v2 backlog assume the flat target.
5. **Clean-architecture boundaries inside each crate.** Entities → use-cases → adapters → frameworks; dependency direction always inward. Validator hard-fails forbidden edges.
6. **Horizontal scaling end-to-end.** No single-instance assumptions. Every state lives in a partitionable store. Every queue is partitioned. Every search shard is replicated.
7. **Single brand surface (Oyatie).** Product, package, and UI surfaces standardize on `Oyatie` per ADR-0017, with `oyatie-` as the Cargo prefix. Repo slug + filesystem path retained (no GitHub repo rename in scope yet).
8. **Data Use Boundary ADR (P0 prereq).** Defines per-consent-tier which tenant data is search-indexable and which can feed ad targeting; PHI/PII/PCI walled off; emits audit-chain on every cross-axis flow. *No cloud / search / ads work begins before this ADR is Accepted.*
9. **Plane separation per ADR-0017 / DESIGN §2.** Every surface declares its plane (control / data / analytics). Cross-plane calls are explicit contracts.
10. **Claim ceiling.** No preview slice may claim foundation guarantees the foundation hasn't yet shipped. Foundation-bypasses are tracked, expirable, ledgered.

---

## 7. Risks (top 10) and mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| Cross-axis contract drift — a SaaS surface evolves a contract the search axis has frozen | High | DESIGN.md §10 contradiction audit; CI fitness function checks every contract against every consumer. |
| Tenant data leak into search/ads via PHI/PII | Catastrophic | Data Use Boundary ADR (P0 prereq); per-cell isolation; audit-chain; per-tenant consent receipts. |
| Agent runtime escaping autonomy ceiling for a regulated capability | Catastrophic | Cedar policy + runtime enforcement (#112) + audit emission + per-capability break-glass + automated revoke. |
| Architectural flattening migration breaks `main` | High | Per-crate move PR shape (ADR-0015 plan §7); workspace-stays-green invariant; one-PR-at-a-time `members =` modification. |
| Brand rename touches public APIs incorrectly | Medium | Standalone PG-0a precursor PR per bounded context (ADR-0017); aliases retire in Phase 7 sweep. |
| Cloud axis built before tenancy/Move #0 | High | Move #0 tenancy is gating prereq for the cloud surface roadmap. |
| Search axis ingests tenant data without consent | Catastrophic | Same Data Use Boundary ADR; per-tenant opt-in at index level. |
| Ads axis monetizes regulated-vertical tenant data | Catastrophic | Healthcare/fintech tenants explicitly excluded from any ad-targeting feedback loop. |
| Korea-locale regulatory shift mid-build | Medium | Quarterly regulatory-change watch lane per ADR-0050 governance umbrella. |
| Cohesion erodes as teams scale | Medium | Plane-gated PR class + cross-axis review pairing on every contract-shaped change. |

---

## 8. Open questions for product council

These are open at PRD draft v0.1 and need a council decision before promoting to v1:

1. **Cloud axis pricing model at public-GA (W3):** per-resource-hour AWS-style, or per-tenant-bundle Connect-style?
2. **Search axis monetization at W4:** purely sponsored-result auction, or also subscription tiers for ad-free SERP?
3. **Ads axis advertiser onboarding at W5:** open self-serve from day 1, or invite-only pilot for the first 6 months?
4. **Vertical sequencing after the first vertical pilot wave**: Manufacturing-MES (#791), Logistics-Spine (#888), or Fintech-PG (#1141) next?
5. **Geographic next step after KR:** Japan (regulatory adjacency), US (market size), or stay KR-deep through W4?
6. **Consumer-search re-evaluation trigger:** what evidence at W6 would shift us from business-intent search to also serving consumer queries?
7. **Hardware position:** if a KR partner offers GPU-fleet co-investment, do we accept (and dilute the "leased always" stance)?

---

## 9. Decision log (PRD-level)

| Date | Decision | Rationale |
|---|---|---|
| 2026-05-08 | Brand standardized as Oyatie (logo `oYa`, domain `oyatie.com`) | User directive; `oyatie-*` Cargo prefix is cleaner; KR-recognizable. |
| 2026-05-08 | Repo slug + filesystem path stay `oyatie` | Migration cost > brand purity; ADR-0017. |
| 2026-05-09 | Repositioned to one cohesive ecosystem-as-a-service across 7 axes | The integration tax kills multi-vendor stacks; cohesion is the moat. |
| 2026-05-09 | Cost-of-deferral horizon: multi-year / structural | Schemas, audit-chain, tenancy, plane separation, search-index shape, cloud control-plane API are unrecoverable failure modes. |
| 2026-05-09 | Korea-as-launch-locale (re-affirmed) | Regulatory moat + design-partner gravity + existing KR-Group pipeline. |
| 2026-05-09 | Data Use Boundary ADR is P0 prereq | Without it, every cloud/search/ads item lands as a contradiction with compliance band. |

---

## 10. Sources scanned

- ADRs: `decisions/ADR-0001..ADR-0051` (51 files)
- Roadmap: `docs/ROADMAP.md`, `decisions/ADR-0015-architectural-flattening-target.md`
- Doctrine: `/specs/decision-principles.json` + `/specs/forbidden-operations.json`
- Source of truth: `docs/DOC-CATALOG.md` (per [`DOC-CATALOG.md`](DOC-CATALOG.md))
- Mistakes & fixes: `docs/MISTAKES-LEDGER.md`
- Audits: `docs/engineering/audits/2026-05-09-foundry-upstream-spec-conformance-audit.md` (separate parallel-track artifact; lives outside `docs/` because it was authored by the foundry-agent-daemon track)
- Ultragoal: `.omx/ultragoal/brief.md`, `.omx/ultragoal/goals.json`, `.omx/ultragoal/issue-priority-pipeline/queue.md`
- v1 backlog: `~/.claude/plans/look-at-all-outstanding-buzzing-teacup.md`
- All 605 open issues at `jason931225/oyatie`
- Greenfield references for cloud (AWS/GCP/Azure/Naver Cloud/NHN/KT/Kakao Cloud), search (Google/Naver/Bing/Yandex/Daum), ads (Google Ads/Naver 검색광고/Kakao Moment/Meta Ads/Criteo)
- User directives 2026-05-08 (rename) and 2026-05-09 (axes + cohesion)

*Footer regenerated whenever any consolidated doc is edited.*
