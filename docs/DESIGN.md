---
purpose: "Oyatie — Architecture & Design Document (DESIGN)"
doc_status: published
---

# Oyatie — Architecture & Design Document (DESIGN)

## Constitutional authority — [CONSTITUTION.md](CONSTITUTION.md)


> **Status:** Draft v0.1 — 2026-05-09. Authoritative-deep. Expected ~60-80 pages with reviewed contributions.
> **Companion docs:** [PRD.md](PRD.md) (the *what* and *why*), [SPEC.md](SPEC.md) (the *what surfaces*), [ROADMAP.md](ROADMAP.md) (the *when*), [ADR-INDEX.md](ADR-INDEX.md) (the *what's been decided*).
> **Key sources cited inline:** ADR-0015 repo structure, ADR-0050 master plan, ADR-0015 flat crates, ADR-0003 audit chain, ADR-0050 AI/ML governance, ADR-0040 launch readiness, ADR-0017 wave integration framework.

---

## 1. Cohesion thesis (one product, seven axes)

Oyatie is **not** an AWS clone, **not** a Salesforce clone, **not** a Naver clone, **not** a Google clone — and **not** a portfolio of any of them. Oyatie is a single product that **contains** all four kinds of surface, joined at a single tenancy model, a single identity, a single capability registry, a single audit chain, and a single agent runtime + engineering platform (Foundry).

The seven axes are *(the former separate engineering-platform axis is now part of Foundry, 2026-05-09; Workspace / Productivity Platform added as Axis 2 on 2026-05-09)*:

| Axis | Reads as | Owning bounded context (flat-crates target) |
|---|---|---|
| 1. SaaS multi-tenant shared substrate | "The shared substrate — tenancy, workflows, plugins, Ontology, marketplace" (per MASTERPLAN.md §2.4: `platform` retired → `shared`; Object Graph renamed → Ontology) | `crates/platform-tenant-*`, `crates/saas-workflow-*`, `crates/saas-plugin-*` (BNF paths retained pending ADR-grade rename per ADR-0015 migration ledger) |
| 2. **Workspace / Productivity Platform (NEW 2026-05-09)** — Mail / Docs / Sheets / Slides / Drive / Calendar / Meet / Chat / Forms / Sites / Tasks / Notes / Translate / Recordings | "The canonical end-user apps everyone uses every day" — Google Workspace / Naver Works / Microsoft 365 / AWS Productivity (WorkMail / WorkDocs / Chime) class | `crates/workspace-{mail,calendar,docs,sheets,slides,drive,meet,chat,forms,sites,tasks,notes,translate,recordings}-*` |
| 3. Vertical industry cloud | "How that work is shaped per industry" | `crates/vertical-{healthcare,industrial,logistics,fintech,legal,corporate,retail,education,public,hospitality,construction,realestate,agriculture,food}-*` |
| 4. **Foundry: AI agent runtime + control plane + engineering platform** | "Who or what *executes* the work AND how engineers + customers build all of the above" | `crates/foundry-*` covering: agent runtime (`-runtime-*`, `-capability-*`, `-policy-*`, `-evidence-*`), provider adapters (`-adapter-{anthropic,openai,gemini}-{api,subscription}-*`), and engineering-platform surfaces (`-catalog-*`, `-repoctl-*`, `-gates-*`, `-scorecard-*`, `-fitness-*`, `-marketplace-*`) |
| 5. Cloud provider | "What runs everything" | `crates/cloud-{compute,storage,network,iam,billing,observability}-*` |
| 6. Search engine | "How any object becomes findable" | `crates/search-{crawler,parser,index,rank,query,serp}-*` |
| 7. Advertising + analytics | "How attention and intent are monetized" | `crates/ads-{auction,target,attribute,console}-*`, `crates/analytics-{event,warehouse,report}-*` |

> **Crate naming convention** per ADR-0015 §1: `oya-<context>-<role>[-<capability>]`. `<role>` ∈ {`kernel`, `domain`, `app`, `api`, `worker`, `adapter`, `runtime`}. The context names above (`platform`, `saas`, `vertical`, `foundry`, `cloud`, `search`, `ads`, `analytics`) are the axis-bounded contexts; per-axis `<role>` decomposition is enumerated in [SPEC.md](SPEC.md).

The **central design insight** is that any one axis — taken alone — is a worse product than the integrated whole, because each axis shares two or more contracts with each other axis (see §10). Splitting an axis off forces re-implementing those contracts as multi-vendor integrations, and every multi-vendor integration leaks privacy, leaks audit trail, leaks cost attribution, leaks identity. Oyatie's competitive moat is the *non-leakage*.

---

## 2. Plane separation (per ADR-0017)

Every surface in every axis declares one of three planes:

- **Control plane** — what configures, schedules, gates. Low-frequency, high-trust, audit-heavy. Examples: capability registration, tenant onboarding, autonomy-ceiling policy publish, ad-campaign authoring, cloud-resource provisioning, search-index lifecycle.
- **Data plane** — what *executes* requests. High-frequency, latency-bounded, fan-out scaled. Examples: workflow execution, search query, ad serving, cloud-resource I/O, agent-step execution.
- **Analytics plane** — what observes, aggregates, learns. Read-mostly on materialized projections. Examples: per-tenant analytics, ranker training, ad attribution, FinOps reports, capacity planning.

> **Strict invariant**: a control-plane API never reads from the data-plane store directly; it must read via a published projection or replay an audited event log. This invariant fails open (silently) without ADR-0003's audit-chain enforcement, which is why audit-chain immutability is a P0 prereq (see §11).

Plane assignment is declared at the catalog level (`registry/catalog/<crate>.yaml: plane:`) and validated in CI. PRs that change a surface's plane class trigger a *cross-plane review* requirement.

### Plane × Axis matrix (which planes each axis owns)

| Axis | Control plane | Data plane | Analytics plane |
|---|---|---|---|
| 1. SaaS | tenant onboarding, workflow publish, plugin install | workflow execution, plugin invocation | per-tenant retention, NPS |
| 2. Workspace / Productivity Platform | mailbox provisioning, doc/sheet/site publish, calendar policy, meet recording policy | mail send/receive, doc/sheet edit + share, meet/chat/calls, drive read/write | per-tenant comms graph, doc collaboration patterns, content-safety telemetry |
| 3. Vertical | per-vertical onboarding, regulatory-pack install | per-vertical execution, FHIR/EDI exchanges | per-vertical KPI dashboards |
| 4a. Foundry — agent runtime | capability publish, autonomy-ceiling policy publish, model registration, provider-adapter auth bind | agent step execution, RAG retrieval, provider invocation | agent telemetry, eval runs, replay |
| 4b. Foundry — engineering platform | catalog publish, claim-ceiling policy publish, gate authoring, fitness-fn registration | CI lane execution, scorecard ingestion | scorecard rollups, fitness-fn alerts, supply-chain attestation telemetry |
| 5. Cloud | resource provisioning, IAM publish, region/AZ register | tenant compute/storage/network I/O | FinOps, capacity planning |
| 6. Search | index lifecycle, crawl scheduling | query, retrieve, rank, serve | ranker training, click stream (privacy-gated) |
| 7. Ads + Analytics | campaign publish, audience publish, advertiser onboarding, analytics pipeline publish | auction, ad serve, click & impression record, event ingestion | attribution, advertiser reporting, DP-bounded reports |

Every cell above maps to one or more flat-crates targets. Cross-cell contracts are enumerated in §10.

---

## 3. Foundry as accelerator (the force-multiplier axis)

> **Premise:** Building Foundry-Preview *immediately after* W-Foundation accelerates *all five other axes* exponentially. Foundry is **second**, not first; Foundation correctness gates are the prerequisite.

> **No-shortcut clause:** Foundry-Preview must NOT operate on real tenant data until Data Use Boundary, secrets/SecretProvider, audit chain, and sandbox contracts are accepted in W-Foundation. Live RAG over tenant or public data is blocked until the Search Substrate gate passes.

> **2026-05-09 consolidation:** Foundry covers BOTH (a) the AI agent runtime + control plane (capability registry, autonomy ceiling, evidence emission, multi-provider adapters) AND (b) the engineering platform for engineers + customers (repoctl, catalog, claim-ceiling validator, foundation-bypass ledger, plane-gated CI lanes, scorecards, fitness functions, ADR templates, plugin substrate trust gates, marketplace authoring). The unification is intentional: the same agent runtime that authors the catalog also enforces it; the same fitness function that gates a PR also evaluates an agent step; the same plugin substrate that hosts customer plugins hosts the engineering-platform gates. One product, one control plane, two persona surfaces (engineer + customer).

### 3.0.1 In-house model production (long-horizon W-AI-Model-Substrate wave, added 2026-05-09)

> **Premise:** Oyatie's *short-horizon* posture is to consume external frontier-LLM providers (Anthropic / OpenAI / Google Gemini) via the multi-provider adapter trait. The *long-horizon* posture is to ALSO train and serve in-house models for tenant-tuned, KR-first, regulated-vertical, and cost-sensitive task profiles. This is **not an AGI lab** — it's a model substrate scoped to Oyatie's product needs.

**Triggering wave:** W-AI-Model-Substrate, sequenced after vertical proof so we know which task profiles to invest in.

**Scope of in-house production models:**

| Model class | Reference | Why in-house |
|---|---|---|
| KR-first foundation LLM | HyperCLOVA-X (Naver), Upstage Solar, LG EXAONE, KT Mi:dm | KR-data-residency-trained for regulated KR tenants; PIPA-compliant training corpus; cost control at scale |
| Embedding models for Search RAG | bge-large-ko, gte-multilingual, e5-multilingual | Per-tenant fine-tunable; consent-tier-respecting training data |
| STT (Speech-to-Text) | OpenAI Whisper, NeMo, Naver Clova Speech | KR / JP / EN dialect coverage; on-device inference for low latency Meet transcription |
| TTS (Text-to-Speech) | XTTS, Tortoise, Naver Clova Voice | Multilingual + Korean voices; per-region voice packs |
| Vision (OCR + doc understanding) | Donut, Pix2Struct, Florence-2, Naver Clova OCR | Workspace-Drive doc understanding; KR/JP form OCR (HWPX, 한국어 forms) |
| Safety + red-team + eval | constitutional-AI-class | Per-vertical safety constraints (clinical / fintech / minors / public-sector) |
| Vertical-tuned task models | various | E.g. clinical-coding model trained on FHIR + KR-EDI; KR fintech-compliance checker; KR labor-law summarizer |

**Architecture of model production (added to the Foundry axis):**

| Crate family | Role |
|---|---|
| `crates/intelligence-model-train-*` | Distributed training orchestration (multi-node, multi-GPU, FSDP / DeepSpeed / Megatron-LM-class scheduler) |
| `crates/intelligence-model-data-pipeline-*` | Training-data ingestion + filtering + deduplication + DP/k-anonymity gates per Data Use Boundary |
| `crates/intelligence-model-eval-*` | Eval harness: golden sets + adversarial + per-vertical safety + per-region linguistic |
| `crates/intelligence-model-registry-*` | Model artifact + version + lineage + provenance (signed via Cosign) |
| `crates/intelligence-model-serve-*` | Inference serving: vLLM-class for Transformers; in-house Rust serving for embedding models; per-capability routing |
| `crates/intelligence-model-finetune-*` | Per-tenant fine-tuning (consent-gated; per-tenant LoRA adapters) |
| `crates/intelligence-model-redteam-*` | Continuous red-team + adversarial-eval pipeline |
| `crates/intelligence-adapter-oya-{api,subscription}-*` | The "Oyatie-as-provider" adapter; uses same `ProviderAdapter` trait as Anthropic / OpenAI / Gemini adapters |

**Critical contracts:**
- Training data must satisfy `purpose = model_training_oya` permission per [PRIVACY-PROGRAM §2.2.2](PRIVACY-PROGRAM.md). Tenant data without that grant is excluded.
- Per-capability `provider` field can route to `internal-<model-id>` instead of (or in addition to) external providers; failover supported per autonomy ceiling.
- Inference serving SLO: p99 < 500ms for embedding; < 2s for chat completion at moderate length; per-capability bound.
- Model artifacts are Cosign-signed + Rekor-anchored per ADR-0039 (supply chain) — proof of training-data lineage included.
- GPU fleet is part of the Cloud axis (`crates/cloud-compute-gpu-*`) — Foundry consumes via standard cloud-IAM + cost-attribution.
- KCMVP-validated HSM-backed model encryption for any model trained on regulated data (PHI / 신용정보 / PCI training corpora).

**Anti-scope (still):**
- No frontier-model pre-training as a standalone product (we are not an AGI lab; we don't sell our base LLM as a service to other LLM consumers).
- No general-purpose model marketplace; in-house models are bound to Oyatie capabilities.
- Frontier-LLM API consumption (Claude / OpenAI / Gemini) remains the primary path until the in-house variant beats or matches on the per-vertical eval set.

### 3.0.2 Vision + Speech + Robotics intelligence (long-horizon, added 2026-05-09)

> Folded into Foundry as additional model substrates + per-vertical applications, not as separate axes.

| Surface | Foundry crate (model substrate) | Per-vertical consumers |
|---|---|---|
| Vision | `crates/intelligence-model-vision-*` (OCR, image classification, object detection, video analytics, facial recognition where lawful, scene anomaly) | Industrial (CCTV facility per ADR-0027; AMR mapping per ADR-0027); Healthcare (clinical imaging per ADR-0033); Logistics (yard / dock vision); Retail (anti-theft, customer flow); Workspace (Drive doc OCR + understanding) |
| Speech | `crates/intelligence-model-speech-*` (STT, TTS, voice biometrics, wake-word, multilingual incl. KR/JP/EN/ES/PT/HI/AR) | Workspace (Meet transcription + AI summary); Healthcare (voice-charting); Vertical contact-center (per-vertical voice agents); (voice messaging) |
| Robotics control | `crates/intelligence-robotics-control-*` (agent-mediated under autonomy ceiling T1-T3 default; T4 disabled for actuation) + `crates/vertical-industrial-robotics-*` (fleet, simulator, kinematics) | Industrial (AGV/AMR/robotic arms/drones per ADR-0027+0143); Logistics (yard jockeys, ASRS, automated trucks); Healthcare (surgical robotics — anti-scope unless founder ratifies; disinfection robots OK) |

**Critical design constraints** for Robotics in particular:
- Real-time control loops (deterministic latency budgets) — not Foundry's default async; runs on dedicated runtime with guaranteed scheduling
- Safety-critical actuation — autonomy-tier T4 (auto-execute) is **disabled by default** for any actuation; T3 (human-approves-each-step) only via per-tenant founder-approved uplift
- Per-vertical safety regulator binding (KR-MFDS for medical robotics; KR-OSHA for industrial; FAA / KR-MOLIT for drones; per-region road-vehicle regulator)
- Simulation-first — every robotic capability must have a simulator harness; production rollout gated on sim coverage
- Per-region anti-scope: defense / weaponized robotics — anti-scope unless explicit founder + legal carve-out (see PRD anti-scope and CONTRADICTION-LEDGER LEDG-017)

### 3.0.5 Automation-first pipeline (Google + Amazon doctrine; added 2026-05-09)

> **Premise:** What can be automated must be automated. The single highest-yield optimization surface is the **git → CI/CD → PR pipeline**, because (a) Rust has slow build times, (b) agents work concurrently with contained blast radius, and (c) every minute saved compounds across thousands of CI runs per week.

#### 3.0.5.1 Build-time optimization (Rust-specific)

| Optimization | Tool | Impact |
|---|---|---|
| Compilation caching | sccache (local + S3-backed remote) — Apache-2 license-clean | 60-90% cache hit on incremental builds |
| Workspace-affected build | `cargo nextest` + custom affected-graph (per-crate change detection) | Build only what changed + downstream |
| Remote execution | Bazel-remote-class via `build-cache` (or Bazel itself if council ratifies; see TOOLCHAIN §9 open question) | 5-10x speedup on cold builds across distributed agents |
| Incremental check vs full check | `cargo check` for fast iteration; `cargo build --release` only at release-tag time | seconds vs minutes per iteration |
| Test sharding | `cargo nextest` partitioning across N runners | linear speedup with shard count |
| Insta snapshot caching | per-test golden snapshots in CI cache | avoid re-running snapshot derivation |
| Per-target compilation | `--target x86_64-unknown-linux-gnu` only when needed; skip cross-targets in PR CI | per-PR savings |
| Codegen unit count tuning | `codegen-units = 16` (debug); `1` (release) | trade-off across profiles |
| Link-time optimization gating | LTO only at release; skip in PR | minutes saved per PR |
| Per-crate compile-time budget | hard ceiling (e.g. ≥ 60s = warn) per `governance-build-time` lane | catches slow-dep regressions |

#### 3.0.5.2 Agent-concurrent CI / PR pipeline

| Mechanism | Purpose |
|---|---|
| Per-agent worktree | One agent per worktree; no shared mutable state in source tree |
| Branch-name collision detection at spawn | Per Issue #58 mistakes ledger; pre-empt the race |
| Merge queue | Only one PR at a time may modify root `Cargo.toml [workspace.members]`; per ADR-0015 §3 PM-3 mitigation |
| Auto-rebase on conflict | Merge queue auto-rebases; agent retries with updated base |
| Auto-labeling | Foundry capability `pr.label.suggest` runs on PR open; suggests change-class labels (cross-axis, cross-plane, regulatory, brand-rename, …) |
| Auto-review bot | Foundry capability `pr.review.draft` drafts review comments per change-class reviewer agent (`rust-reviewer`, `typescript-reviewer`, `database-reviewer`, `security-reviewer`); human approves the verdict per CLAUDE.md `## Code Review` rules |
| Auto-changelog | Per-merge changelog row appended; per-release notes drafted from merged PRs |
| Auto-issue triage | New issues auto-labeled per repo's 5-label triage vocabulary (per `docs/agents/triage-labels.md`); auto-routed to owning team |
| Auto-flaky-quarantine | Tests failing intermittently auto-quarantined to a separate lane until fixed |
| Auto-bypass-expiry | Foundation bypass past expiry auto-emits `EVT-FOUNDATION-BYPASS-EXPIRED` and opens a renewal PR |
| Per-tenant CI lane | Per-tenant override / regression set runs only on PRs touching tenant-bound code |
| Speculative parallel dispatch | For agent-authored PRs: fire 3 alternative approaches in parallel; pick first to pass acceptance criteria |
| Replay-as-eval | New PR's affected-test set is replayed against past trace set for regression detection |
| Nightly affected-rebuild | `main` re-builds nightly with full `--all-features` to catch feature-flag drift (PM-2 from ADR-0015 plan) |
| Per-lane CI-time budget | `governance-ci-time` lane warns when any lane regresses ≥ 20% over baseline |

#### 3.0.5.3 Blast-radius containment (the cohesion-side guarantee)

Every agent + automation invocation declares its blast radius. The CI pipeline enforces it.

| Blast-radius class | Containment guarantee | Examples |
|---|---|---|
| `worktree-local` | No mutation outside the agent's worktree | doc edits, cosmetic refactors |
| `single-crate` | Mutates one flat crate's source + tests + catalog record | feature add inside a crate |
| `single-axis` | Mutates one axis's crate set | axis-internal refactor |
| `cross-axis-contract` | Mutates a DESIGN §10 contract row + ≥ 2 axes | new contract; requires cross-axis review label |
| `flat-crates-move` | Mutates root `Cargo.toml [workspace.members]` | per ADR-0015 phase PR; requires merge-queue serialization |
| `cross-region-pack` | Mutates ≥ 2 regional packs | requires per-pack review |
| `regulatory-impact` | Touches a regulator-bound surface | requires `ops-compliance` review |
| `data-class-impact` | Changes a `data_class` annotation | requires `council-privacy` review |

The classification is auto-detected by `governance-blast-radius` from the
diff and emitted as a PR label. One author-distinct reviewer agent applies the
relevant blast-class lenses on the exact PR head; affected teams are notified
for non-binding input, not counted as a reviewer quorum.

#### 3.0.5.4 Foundry capabilities for the pipeline

The pipeline ITSELF is agent-extended. Foundry capabilities authored for the pipeline:

- `pr.label.suggest` — per-change-class label suggestions
- `pr.review.draft` — per-change-class drafted review per reviewer-agent persona
- `pr.changelog.row` — emit changelog row from PR title + body
- `pr.release-note.draft` — assemble release notes from merged PRs since last tag
- `issue.triage.label` — 5-label triage applied per docs/agents/triage-labels.md
- `issue.assign.suggest` — owning-team suggestion based on changed files
- `runbook.draft.from-incident` — extract draft runbook from incident postmortem
- `adr.promotion.review` — verify shipped evidence for Proposed → Accepted
- `dep.license.review` — review new dep license tier
- `flaky.test.classify` — classify a flaky test for quarantine
- `bypass.renewal.check` — verify foundation-bypass renewal eligibility

Each capability is autonomy-tier T2 (flag + freeze, human approves) by default; tenant admins can uplift to T3 / T4 per case.

#### 3.0.5.5 Investment payback (rough estimates)

| Optimization | Cost (one-time) | Saving (recurring) |
|---|---|---|
| sccache + remote cache | ~2 weeks | 30-50% of CI minutes ⇒ thousands of $/month |
| Affected-graph test runner | ~3 weeks | 50-80% of test minutes |
| Auto-rebase + merge queue | ~2 weeks | hours/week of human babysitting |
| Foundry PR triage capabilities | ~6 weeks | 1-2 reviewers worth of leverage |
| Per-lane CI-time budget | ~1 week | drift prevention; saves regressions later |
| Agent-driven changelog + release notes | ~3 weeks | hours per release |
| Branch-name collision detection | ~1 day | prevents PR #1548-class incidents |
| Replay-as-eval | ~4 weeks | catches regressions before merge |

Per [PRD §6 constraint 5](PRD.md), this is a structural commitment, not a "we'll do it later" item.

### 3.0.4 Cloud compute substrate trajectory (added 2026-05-09 per user directive)

The Cloud axis has TWO halves: (a) compute substrate we **consume** (everything runs on it), and (b) cloud product we **sell** (others run on it). The substrate trajectory evolves while the product surface stays stable.

| Phase | Substrate | Trigger to next |
|---|---|---|
| **Phase 1: OCI + AWS hyperscaler consumption** | Oyatie services run on OCI (per ADR-0021 OCI A1 Always Free + ADR-0044 cloud-native + ADR-0044 data tier + ADR-0044 OCI managed-service inventory) and AWS as opportunistically. The Cloud axis product surface (IAM / region / cell / billing) is built on top of the hyperscaler primitives via ports. | Cost / latency / control / regulatory pressure crosses a threshold |
| **Phase 2: Hybrid (OCI + AWS + Oyatie colo)** | We add Oyatie-operated colo cages (KR-first, then JP / US / EU / etc.) for hot tenant workloads + GPU fleet (post W-AI-Model-Substrate). Hyperscaler stays for spillover + DR. The DCIM software per [§3.0.3](#303-datacenter-operations-software-long-horizon-w-datacenter-operations-wave-added-2026-05-09) goes operational. | Per-region tenant volume + sovereignty obligations exceed colo capacity |
| **Phase 3: Own mega-datacenter** | Oyatie operates own DC shells (still leased real-estate; fully Oyatie-built-out interior) at megawatt-class scale. Hyperscaler + colo continue as hybrid for failover + bursty / spiky workloads. Custom power/cooling/networking; possible custom hardware bake (verified-for-Oyatie commercial silicon) — but NOT chip design. | (Long-horizon, market-conditional) |

The product surface — `oya cloud` IAM / region / cell / billing / observability — is **identical** across phases. Tenants that bind to Oyatie cloud at Phase 1 keep working at Phase 2 / 3 transparently. Tenancy + cell + region kernels do not change shape.

### 3.0.3 Datacenter Operations Software (long-horizon W-DataCenter-Operations wave, added 2026-05-09)

> Folded into Cloud axis as a sub-axis (DC-ops). Triggered when Oyatie operates physical or colo DC capacity at scale (post W-Cloud-Stable when the cost of consuming hyperscaler / 3rd-party colo crosses the build-our-own threshold).

| Sub-surface | Crate family | Reference |
|---|---|---|
| DCIM (Datacenter Infrastructure Mgmt) — rack/asset/capacity/power/PUE inventory | `crates/cloud-dcops-dcim-*` | Sunbird DCIM, Schneider EcoStruxure, Nlyte; Google internal Borg-DC integration |
| BMS / BAS (Building / HVAC / fire / water) | `crates/cloud-dcops-bms-*` | Siemens Desigo, Honeywell EBI, Johnson Controls Metasys |
| Power monitoring (PDU/ATS/UPS/generator/fuel) | `crates/cloud-dcops-power-*` | Schneider, Eaton, ABB |
| Cooling control (CRAH, chilled water, free-air, hot-aisle containment) | `crates/cloud-dcops-cooling-*` | (custom + BMS adapters) |
| Network ops (cable maps, fiber budget, patch panel) | `crates/cloud-dcops-network-*` | NetBox-class |
| Physical security (badge, CCTV, mantrap, env sensors) | `crates/cloud-dcops-security-*` | Genetec, Lenel; integrates with vision substrate |
| Asset lifecycle (procurement → decommission → e-waste) | `crates/cloud-dcops-asset-*` | (custom; integrates with vendor-partner-ledger) |
| Capacity + thermal planning | `crates/cloud-dcops-capacity-*` | Per-rack power + thermal budget; growth modeling |
| Workorder + technician dispatch | `crates/cloud-dcops-workorder-*` | (custom; integrates with Workspace tasks/calendar) |
| Sustainability + carbon accounting | `crates/cloud-dcops-sustainability-*` | PUE/WUE/CUE per region; carbon attribution per tenant |
| Regulatory (Uptime Institute Tier, EN 50600, KR ISMS-DC, CSA STAR-Cloud) | `crates/cloud-dcops-compliance-*` | Per regional pack regulator |

**Anti-scope inside DC-ops:** designing custom chips / ASICs / FPGAs (we use commercial silicon); building DC shells / civil construction (we lease the shell + build out the interior).

### 3.0 Multi-provider authentication model (NEW per 2026-05-09)

Foundry must work with **both subscription auth and API auth** across **Claude, OpenAI, and Gemini**. This is a hard constraint.

| Provider | Subscription mode | API mode |
|---|---|---|
| Anthropic Claude | Claude Pro / Claude for Work session token via headless adapter | `ANTHROPIC_API_KEY` direct API |
| OpenAI | ChatGPT Plus / Team / Enterprise session token via headless adapter | `OPENAI_API_KEY` direct API |
| Google Gemini | Gemini Advanced session token via headless adapter | `GOOGLE_GEMINI_API_KEY` direct API |

The `ProviderAdapter` trait in `intelligence-adapter-kernel` exposes a uniform `invoke(prompt, tools, policy) -> Stream<Event>` interface; concrete impls live in `intelligence-adapter-{anthropic,openai,gemini}-{api,subscription}-*`. Per-tenant per-capability the `ProviderAuth` enum selects the auth mode. Subscription mode requires the `intelligence-session-vault` to hold rotating session tokens with refresh logic; API mode hits the SecretProvider for the API key. Capability-level routing supports failover across providers (e.g. `prefer: claude-api → fallback: openai-api → fallback: gemini-subscription`) per autonomy-ceiling and per FinOps cost ceiling.

Foundry is axis 3 of the six, but it is also the *substrate* for the other five. Once Foundry preview is online with provider adapters (Anthropic / OpenAI / Gemini × subscription + API auth), capability registry, autonomy-ceiling enforcement, evidence-chain emission, AND the consolidated foundry surfaces, then:

- **SaaS axis** ships workflows authored *by* agents, not just *executed* by agents.
- **Vertical axis** ships regulatory-pack adoption with agent-driven evidence collection (HIPAA controls, KISA controls, MFDS controls — all become agent procedures).
- **Foundry's own foundry surfaces** ship repoctl/catalog/CI lanes that are agent-extended (every ADR drafted by agent and human-reviewed; every catalog record written by agent and validated by human; every plugin manifest signed-off by an agent + human pair).
- **Cloud axis** ships its control plane *operated by* agents (provisioning, IAM publish, region register, capacity rebalance — all agent-mediated under autonomy ceiling).
- **Search axis** ships its index lifecycle (crawl scheduling, ranker tuning, freshness decisions) *operated by* agents.
- **Ads axis** ships smart-bidding ML loops *operated by* agents under explicit privacy gates.

Without Foundry early, every one of those gains is replaced by linear human effort. Quantitatively: Oyatie's headcount doesn't grow; only Foundry's capability catalog grows. **Foundry is therefore the single highest-leverage investment in the v2 backlog and is sequenced first** (see [ROADMAP.md](ROADMAP.md) W-Foundry-Preview wave).

### Foundry's own internal sequencing (per home-dir-agent finding 2026-05-09)

Foundry preview is itself a *sequential dependency chain*, not parallel band. Order:

1. **SecretProvider + KMS** (currently the reason live-provider Foundry execution is disabled, Issue #1315)
2. **Codex provider adapter** with isolated `CODEX_HOME` per run
3. **Claude provider adapter** with isolated `CLAUDE_CONFIG_DIR` per run
4. **PTY/process launch backend** — direct `openpty`/`forkpty` per spawned provider. *(Not tmux.)* tmux was on an early research list as a convenience for human-attached debug sessions; for production agent dispatch at cloud scale, a tmux dependency is unnecessary surface area. Direct PTY allocation + structured stdout/stderr capture is the clean-arch fit. tmux remains optional for *developer-attached* debugging only.
5. **Daemon hardening** (#1266 hook_bus stale anchor, #1267 subscription_router credential shadowing, #1268 shutdown checkpoint)
6. **Live provider smoke lane** (Issue #1316) — gated env-flag tests need a real lane
7. **Live pilot** — first capability that runs end-to-end with real provider, real tenant data, real evidence emission

After step 7, Foundry can fan out *parallel* capability authorship across all six other axes. The sequencing penalty is only paid once.

### Foundry contracts that the other axes depend on

| Contract | Owners | Consumers |
|---|---|---|
| Capability invocation API (`intelligence-api`) | Foundry | Every axis (SaaS workflows, vertical runbooks, cloud control-plane mutators, search index ops, ad-campaign mutators) |
| Autonomy-ceiling enforcement (`intelligence-policy`) | Foundry + ADR-0050 governance | Every regulated capability across all axes |
| Evidence-chain emission (`intelligence-evidence`, ties to ADR-0003) | Foundry + audit subsystem | Every axis that touches regulated data |
| Capability registry (`intelligence-registry`, projection from `registry/catalog/`) | Foundry + Foundry catalog | Every axis (every capability they expose) |
| Provider adapter trait (`intelligence-adapter`) | Foundry | Codex, Claude, future model providers |
| RAG endpoint (`intelligence-rag`) | Foundry + Search | Every axis that needs retrieval |

---

## 4. Per-axis bounded contexts (clean architecture inside each)

Each axis is a bounded context with the four-layer hexagonal stack:

```
                ┌─────────────────────────────────────────┐
                │  frameworks-and-drivers (HTTP, Kafka,   │
                │  Postgres, S3, KMS, gRPC, OpenTelemetry)│
                └────────────────┬────────────────────────┘
                                 │
                ┌────────────────▼────────────────────────┐
                │  interface adapters (REST handlers,     │
                │  Kafka consumers, DB adapters,          │
                │  cloud SDKs, search indexers)           │
                └────────────────┬────────────────────────┘
                                 │
                ┌────────────────▼────────────────────────┐
                │  use cases (commands, queries,          │
                │  policies, sagas, projections)          │
                └────────────────┬────────────────────────┘
                                 │
                ┌────────────────▼────────────────────────┐
                │  entities (domain types, invariants,    │
                │  value objects)                         │
                └─────────────────────────────────────────┘
```

Dependency direction: always inward. Adapters import use-cases; use-cases import entities. The validator (ADR-0015 §3.3, `oya gate validate architecture-boundaries`) hard-fails any forbidden edge.

The flat-crates target encodes the layers as crate-level roles per ADR-0015:

- `oya-<context>-kernel-*` = entities (no I/O, no async, no framework)
- `oya-<context>-domain-*` = use cases + sealed-port traits
- `oya-<context>-app-*` = orchestration / sagas / commands
- `oya-<context>-adapter-*` = adapter implementations (DB, HTTP client, KMS, etc.)
- `oya-<context>-api-*` = inbound HTTP/gRPC servers
- `oya-<context>-worker-*` = inbound Kafka/queue consumers
- `oya-<context>-runtime-*` = composition root (binaries, deploy)

The forbidden-edge graph: `kernel ← domain ← app ← {api, worker, adapter} ← runtime`. Reverse edges are CI errors.

### Per-axis kernel size targets

| Axis | Kernel crate count | Why | Example kernels |
|---|---|---|---|
| 1. SaaS | 6-10 | Stable platform invariants (tenant, workspace, identity, RBAC, plane, metering) | `platform-tenant-kernel`, `platform-identity-kernel` |
| 2. Vertical | 1-3 per vertical | Per-vertical entity model (Patient, WorkOrder, Shipment, Loan) | `vertical-healthcare-kernel`, `vertical-industrial-kernel` |
| 3. Foundry | 4-6 | Capability, Step, Run, Evidence, Provider, AutonomyCeiling | `intelligence-capability-kernel`, `intelligence-evidence-kernel` |
| 4. Foundry | 3-5 | Catalog, Lane, Gate, Bypass | `intelligence-catalog-kernel`, `governance-gate-kernel` |
| 5. Cloud | 5-8 | Resource, Region, AZ, Cell, IAM, Billing | `cloud-resource-kernel`, `cloud-iam-kernel` |
| 6. Search | 3-5 | Document, Index, Query, Result, Ranker | `search-document-kernel`, `search-index-kernel` |
| 7. Ads | 4-6 | Campaign, Auction, Impression, Click, Conversion, Audience | `ads-campaign-kernel`, `ads-auction-kernel` |

---

## 5. The unifying tenancy model

Every axis shares one tenancy abstraction (per Maturity Move #0, Issue #1558). One `Tenant` entity lives in `platform-tenant-kernel` and is referenced by every other axis:

```rust
// platform-tenant-kernel
pub struct TenantId(pub Uuid);
pub struct Tenant {
    pub id: TenantId,
    pub region: RegionCode,        // KR-Seoul1, KR-Busan, ... (cloud axis)
    pub residency: ResidencyClass, // strict_kr, kr_with_us_failover, global, ...
    pub regulatory_packs: Vec<RegulatoryPackId>, // PIPA, HIPAA, MFDS, FSC, ...
    pub plane_grants: TenantPlaneGrants, // which planes this tenant can call
    pub autonomy_tier: AutonomyTier, // foundry axis
    pub data_use_consent: DataUseConsent, // search/ads axes (P0 prereq ADR)
    pub billing_account: BillingAccountId, // cloud axis
}
```

Every regulated cross-axis call carries `TenantId` + a consent receipt, validated at the boundary. Cell-routing (cloud axis) reads `region`. The autonomy ceiling (foundry axis) reads `autonomy_tier`. Search-index ingestion (search axis) reads `data_use_consent.search_indexable_classes`. Ad targeting (ads axis) reads `data_use_consent.ad_targeting_classes`. Regulatory packs (vertical axis) drive control evidence.

**Consequence:** any change to the `Tenant` shape is a cross-axis change and goes through the cross-axis review gate. This is the single most reviewed kernel in the codebase, by design.

---

## 6. The Data Use Boundary (P0 prereq, will become an ADR)

> **The hardest contract in Oyatie.**

Tenant data flows naturally toward search and ads. Without a formal boundary, a single bad PR can land tenant PHI in the search index or as an ad-targeting feature. The Data Use Boundary ADR declares:

1. **Class taxonomy** for tenant data:
   - `internal_only` (never leaves the tenant boundary)
   - `tenant_searchable` (indexed in tenant-private search index only)
   - `cross_tenant_searchable_with_consent` (indexed in shared search index after explicit per-record consent)
   - `analytics_aggregated` (feeds analytics in k-anonymous form)
   - `ad_targetable_low_sensitivity` (e.g., declared interest categories)
   - `ad_targetable_blocked` (PHI/PII/PCI — never feeds ads regardless of consent)

2. **Consent gradient**: each tenant declares which classes are active, with separate opt-in for search vs ads.

3. **Vertical-specific overrides**: healthcare tenants are *forced* to `ad_targetable_blocked` for any record touching the FHIR resource graph. Fintech tenants are *forced* to `ad_targetable_blocked` for any record touching account/payment instruments. PIPA-protected tenants in KR get tighter defaults than global GDPR baseline.

4. **Audit-chain emission requirement**: every cross-axis data flow (SaaS → search, SaaS → analytics, search → ads) emits an evidence record to the audit chain (per ADR-0003). Missing emission = CI failure on the consuming axis.

5. **DSR + withdrawal**: any GDPR/PIPA DSR or consent withdrawal triggers cascading deletes across search index and ads attribution stores. Proof-of-erasure record per cascade.

6. **Class transitions**: a record can only weaken its class via explicit human approval; tightening (e.g., revoking consent) is automatic.

This ADR is **P0 prereq**. The cloud, search, and ads axes do not begin substantive work until this ADR is Accepted, because every leaf in those axes either reads from this contract or contradicts it.

---

## 7. The audit chain (per ADR-0003)

The single tamper-evident record-keeping surface across all axes. Built on hash-chained append-only event log with periodic anchoring.

### Records that MUST emit

| Axis | Event class | Example |
|---|---|---|
| SaaS | tenant onboarding, plugin install, tenant-data export, DSR fulfillment | "tenant `x` exported PHI to local storage" |
| Vertical | regulatory-pack adoption, control-evidence collection, break-glass invocation | "user `y` exercised break-glass for patient `z` at time `t`" |
| Foundry | capability invocation, autonomy-ceiling decision, model invocation, RAG retrieval | "agent `a` invoked capability `c` on tenant `x` under autonomy tier `T`" |
| Foundry | catalog mutation, gate ratchet (WARN→BLOCK), foundation-bypass create/expire | "bypass `b` created against gate `g` with expiry `t`" |
| Cloud | IAM mutation, region/AZ register, capacity grant, resource provision | "tenant `x` provisioned `n` instances in region `r`" |
| Search | index lifecycle (create, snapshot, delete), DSR-driven cascade delete | "tenant `x` requested DSR; index entries `e_1..e_n` deleted" |
| Ads | campaign create, audience create, ad-targeting decision, conversion attribution | "campaign `c` targeted audience `a` for impression `i`" |

### Properties

- **Append-only**: no record is ever rewritten. Erasure = a "deletion-evidence" record + cryptographic invalidation pointer.
- **Hash-chain integrity**: each record references the prior block hash; periodic root anchoring published to a customer-facing trust portal (search-axis is the surface for it).
- **Per-tenant chain shard**: chains are tenant-scoped to keep regulator queries scoped; cross-tenant root index for global proofs.
- **Replayable**: chain replay must reconstruct the regulatory state at any prior `t`. No live mutation can break replay.
- **Foundry-emitted**: agent invocations write evidence directly via `intelligence-evidence` so non-agent paths are the exception, not the rule.

The audit-chain is the backbone of cross-axis trust. The PRD's hard zero on "tenant data egress without consent receipt" depends on the audit chain working — which is why audit-chain immutability is a P0 structural requirement, not a follow-up.

---

## 8. Architectural flattening (per ADR-0015)

> **State of the migration as of 2026-05-11:** ADR-0015 Accepted; the live workspace contains 64 `crates/oya-*` members and 64 `registry/catalog/<crate>.yaml` records; top-level `modules/`, `services/`, `platform/`, and `tools/` are retired. The historical REV7 split inventory remains planning context for additive split/extraction work, not the live tree.

Every consolidated doc and the v2 backlog assume the **flat target**, not the legacy `modules/` `services/` `platform/` tree.

### The flat target shape

```
crates/
  oya-<context>-<role>[-<capability>]/   # live flat workspace; 281 crates on 2026-05-16
contracts/                                 # OpenAPI specs, gRPC protos, event schemas
infra/                                     # admission policies (kyverno/), Argo Application
                                           #   manifests, GitOps topology (per ADR-0117 +
                                           #   per ADR-0119); replaces the prior deploy/gitops/
                                           #   single-file root for admission concerns.
deploy/                                    # per-deployable Helm/IaC (future use; not yet
                                           #   populated. Admission-policy + GitOps-Application
                                           #   moved to infra/kyverno/ per ADR-0117).
registry/                                  # flat singular registry (per ADR-0115);
                                           #   includes catalog/, capability-templates/,
                                           #   quality/, fixuptasks.jsonl
specs/                                     # flat-root machine-readable specs (per ADR-0119);
                                           #   replaces former specs/cross-cutting/ nesting
```

### The 9 splits (per ADR-0015 plan §6.5 inventory)

| Source crate | Split → kernel half | Split → platform half |
|---|---|---|
| `forms` | `platform-forms-kernel` | `platform-forms-app` (+ adapters) |
| `analytics` | `analytics-kernel` | `analytics-app` |
| `data-policy` | `platform-data-policy-kernel` | `platform-data-policy-app` |
| `audit-chain` | `platform-audit-chain-kernel` | `platform-audit-chain-app` |
| `crypto` | `platform-crypto-kernel` | `platform-crypto-app` |
| `observability` | `platform-observability-kernel` | `platform-observability-app` |
| `secrets` | `platform-secrets-kernel` | `platform-secrets-app` |
| `web` | `platform-web-kernel` | `platform-web-app` |
| `workflow-sdk` | `saas-workflow-sdk-kernel` | `saas-workflow-sdk-app` |

Two of nine shipped in the historical plan as 1-PR atomic (`forms`, `analytics`); seven shipped as 2-PR pairs. **Historical inventory: 89 move PRs, 91 target crates.** Treat these counts as planning evidence only; the live workspace count is the Cargo metadata count.

### Phase tier order

```
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6 → Phase 7
kernel    contracts  domain    app       api/worker/adapter  runtime  sweep
```

Service runtime split (Axis E, ADR-0015 §6) is deferred until after all domain/lib crates land, because services are composition roots.

### Why this matters for the v2 backlog

- **No new `modules/` `services/` `platform/` work.** Every leaf in the v2 backlog cites a flat-crates target.
- **Contracts (`contracts/`) get their own batch tag** because cross-axis contract changes touch this directory.
- **`infra/` is the canonical root for admission policies + GitOps Application manifests** (ADR-0117 consolidated `deploy/gitops/oya-vcs-admission/` under `infra/kyverno/oya-vcs-admission/`). `deploy/` is reserved for future per-deployable Helm/IaC that does not fit under `infra/`; the historical "split out of services/" rationale stands but admission concerns are now resolved.
- **Catalog remains at `registry/catalog/`** (per ADR-0115 the `registry/` root is canonical, singular, flat; the prior `registries/cross-cutting/` is retired). Any future `catalog/` relocation requires a new catalog protocol update; do not infer it from the historical phase plan.
- **Specs flattened** (per ADR-0119): machine-readable specs live at `specs/<basename>.json` (flat root). The former `specs/cross-cutting/` nesting is retired; `specs/cross-cutting/lifecycle-configs/` is retained as a documented typed-family exception.

### Risk-managed assumption

If the flat-crates migration stalls or reverses, the v2 backlog must be re-derived. We assume forward-only progress on ADR-0015; if that breaks, ROADMAP.md highlights this as a *blast-radius re-rank trigger*.

---

## 9. Horizontal scalability primitives

Every axis ships with the following horizontal-scale primitives. The CI fitness function (Issue #1566) checks each new flat crate for compliance.

| Primitive | What it means | Per-axis examples |
|---|---|---|
| **Cell routing** | A request is steered to a cell within a region/AZ; cells are isolation boundaries. | Cloud: tenant compute lives in a cell; SaaS: tenant workflow runs in a cell; Search: query routes to nearest replica cell; Ads: auction in nearest cell. |
| **Stateless or partitionable services** | No single-node state. State lives in a partitionable store. | Foundry: agent runs are stateless; Foundry: lanes execute statelessly; Vertical: workflow steps stateless. |
| **Sharded data stores** | Per-tenant or per-key shards; horizontal capacity. | SaaS: per-tenant Postgres shard; Search: per-term shard; Ads: per-campaign shard for impression stream; Cloud: per-region resource catalog. |
| **Partitioned queues** | Kafka-style partitions keyed by tenant or capability. | Audit chain: per-tenant partition; Foundry: per-capability partition; Ads: per-campaign partition. |
| **Replication for availability** | Every write replicated; every read can hit a replica. | Postgres: streaming replication; Search: index replicas; Ads: auction state replicated. |
| **Backpressure + admission control** | Every consumer applies backpressure to keep upstream healthy. | Cloud control plane; ad auction; search query router. |
| **Idempotency + replay safety** | Every API and event consumer tolerates re-delivery. | Audit-chain consumers must be idempotent; billing events must dedupe; capability invocations must be replay-safe. |
| **Region failover drill** | Quarterly proven failover in non-prod; annual in prod. | Cloud (#214 multi-AZ failover automation); SaaS (#1302 DR drills); Search (cross-region index replication). |

The fitness function is the enforcement; the primitive list is the contract. Both live in Foundry axis (`governance-*`).

---

## 10. Cross-axis contract surface (the cohesion audit point)

This is the table that is auditable. Every entry is an inter-axis contract; any change goes through cross-axis review (a labeled PR class).

| Contract | Owner axis | Consumer axis(es) | Where it lives | Contract change requires |
|---|---|---|---|---|
| `Tenant` kernel | SaaS | All others | `platform-tenant-kernel` | All-axis review; ADR amendment if shape changes |
| `Identity / RBAC / Cedar policy` | SaaS | All others | `platform-identity-kernel` + Cedar | Cross-axis review; security-reviewer agent |
| `Capability invocation` | Foundry | All others | `intelligence-api` + `contracts/openapi/foundry/capability-v1.yaml` | Foundry review + the consuming axis review |
| `Autonomy ceiling policy` | Foundry + Governance (ADR-0050) | All regulated capabilities | `intelligence-policy-kernel` | Governance + security review |
| `Audit-chain event` | Audit subsystem (per ADR-0003) | All emitters | `platform-audit-chain-kernel` + event schema | Audit + downstream-consumer review |
| `Capability registry record` | Foundry catalog | Foundry, all axes | `registry/catalog/<crate>.yaml` (any future relocation requires a catalog protocol update) | Catalog gate + consuming-axis review |
| `Plane class` | Foundry catalog | All surfaces | `registry/catalog/<crate>.yaml: plane:` | Cross-plane review trigger |
| `Cloud resource type` | Cloud | Cloud customers, tenant resource lifecycle, billing | `cloud-resource-kernel` | Cloud + billing review |
| `Region / AZ / Cell` | Cloud | All others (tenant residency, search shard placement, ad cell) | `cloud-region-kernel` | Multi-axis review (residency-impact) |
| `IAM / SSO / SAML / OIDC IdP` | Cloud (also SaaS) | All others | `cloud-iam-kernel` (cloud-customer-facing) and `platform-identity-kernel` (SaaS) | Two ADRs in lockstep |
| `Ontology property tier` (ADR-0006..0112; legacy name "Object Graph property tier" — renamed per MASTERPLAN.md §2.4) | SaaS | Search (indexable), Vertical (regulatory), Ads (targetable) | `platform-object-graph-kernel` (BNF path retained pending ADR-grade rename per ADR-0015 migration ledger) | Ontology review + Data Use Boundary check |
| `Search index lifecycle` | Search | Foundry (RAG), SaaS (tenant search), Ads (sponsored slot) | `search-index-kernel` | Search + downstream review |
| `Ad slot inventory` | Ads | Search (SERP), SaaS (in-app), Vertical (in-vertical-app) | `ads-slot-kernel` | Ads + surface-owner review |
| `Billing event` | Cloud + SaaS metering | Billing + Tax + Marketplace | `platform-metering-kernel` + `cloud-billing-kernel` | Billing + tax review |
| `DSR / consent withdrawal cascade` | Privacy/Compliance | All emitters of indexed/targetable data | `platform-dsr-kernel` | All data-touching axes acknowledge cascade ack |
| `Webhook delivery + signing` | SaaS / Cloud | External callers + ISVs | `platform-webhook-kernel` | API stability gate (ADR-0040) |
| `Public REST stability tier` | SaaS / Cloud / Search / Ads | External callers | `contracts/openapi/**/*.yaml` | API stability gate (ADR-0040) |
| `Marketplace listing` | Foundry + SaaS | ISVs, Plugin runtime, Ads | `saas-marketplace-kernel` | Marketplace gate + plugin signing/sandbox |
| `Eventing backbone (outbox + Kafka topic)` | Foundation contracts (`foundation-contracts`, not "Platform") | All axes | `platform-eventing-kernel` | Cross-axis on topic shape |
| `CLOUD_SEARCH_CAPACITY_AND_RESIDENCY` | Cloud + Search | Search shard placement; crawl/index capacity; data residency per region; deletion propagation; cost attribution | `crates/cloud-capacity-kernel` + `crates/search-shard-placement` | Cloud + Search review + Privacy review (residency) |
| `SEARCH_ADS_SERP_AND_QUERY_PRIVACY` | Search + Ads | Sponsored-result labeling; query-privacy; ad-eligibility per query class; ranking separation; minors / medical / financial ad exclusions; click + conversion attribution | `crates/search-serp-kernel` + `crates/ads-eligibility-kernel` | Search + Ads + Privacy review |
| `FOUNDRY_CLOUD_MUTATION_CONTROL` | Foundry + Cloud | Agent-driven control-plane mutation: dry-run, M-of-N approval, break-glass, rollback, per-mutation audit evidence | `crates/intelligence-cloud-mutation-kernel` | Foundry + Cloud + Security review |
| `FOUNDRY_SEARCH_RETRIEVAL_BOUNDARY` | Foundry + Search | RAG retrieval isolation: per-tenant boundary, prompt + tool-trace data class, source-citation evidence, public-corpus rights enforcement, minor-subject filter | `crates/intelligence-rag-kernel` + `crates/search-retrieval-kernel` | Foundry + Search + Privacy review |
| `TENANT_ADS_ANALYTICS_ELIGIBILITY` | SaaS / Vertical + Ads / Analytics | Tenant ad-free modes; per-vertical override (healthcare / fintech / education hard-deny); consent inheritance; event-schema; attribution; DSR cascade | `crates/platform-ads-eligibility-kernel` + per-vertical override pack | SaaS + Vertical + Ads + Privacy review |
| `REVENUE_METERING_TAX_INVOICE` | SaaS + Cloud + Ads + Marketplace | Cross-axis billing-event contract; per-region tax-invoice format (KR 전자세금계산서, JP 適格請求書, EU per-country e-invoicing, IN GST, BR NF-e, KSA FATOORA, UAE e-invoicing); dispute / chargeback / reconciliation flow | `crates/platform-billing-kernel` + per-pack tax adapter | Billing + Tax + per-pack regional review |
| `FOUNDATION_BUILDER_CONTRACT_REGISTRY` | Foundry (foundry surface) | Source of truth for every cross-axis contract row in this table; CI fitness function generates this table from the registry | `crates/intelligence-contract-registry-*` | Foundation Council |

Anytime a PR touches a row above, the cross-axis label is required, and the labeled review block in the PR template is mandatory. The fitness function (`governance-contracts`) checks for orphan contract changes.

---

## 11. Cross-axis contradiction audit (the cohesion check)

This section is the *active* audit point: every quarter, the consolidated docs are re-reviewed for axes that have evolved contracts in a way that contradicts another axis's ground truth.

### Known contradiction risks (heading into v2 plan)

| Contradiction | Source axis | Conflicting axis | Resolution path |
|---|---|---|---|
| "Tenant data is never indexed externally" (legacy privacy ADR draft) | SaaS / privacy | Search (which must index tenant data into the cross-tenant search-axis index for any cross-tenant findability) | Data Use Boundary ADR (P0 prereq) — class taxonomy resolves it |
| "No ads ever" (some legacy roadmap text) | SaaS / brand | Ads | PRD §1 amendment + supersede legacy text via ROADMAP changelog |
| "Oyatie does not host customer-builder workflows" (suspect legacy ADR text — agent to confirm) | SaaS / scope | Plugin substrate (#29), Workflow Studio | If found, supersede; SaaS axis is multi-tenant-builder by design |
| "Vertical-only product" (architecture-sweep finding 2026-04-22) | Architecture | All axes (we are also SaaS, cloud, search, ads) | Update CONSTITUTION.md and source-of-truth.md |
| "All metering goes through one path" (per ADR-0007) | SaaS metering | Cloud billing (which has its own meter ingest) | Cross-axis billing event contract — already in §10 row |
| "Tenant resources stay in their region of registration" (ADR-0044 corp data tier) | SaaS residency | Cloud cross-region replication for DR (which moves data) | Tenancy `residency:` field + per-class allowed-replication policy |

### How the audit happens

1. Cross-axis PR class label is required on any PR touching a row in §10.
2. The audit happens via the rename+contradiction agent every quarter (and on demand before any wave-gate).
3. Findings get added to ROADMAP.md as P0 resolution leaves until the contradiction is closed.
4. Any contradiction left open at a wave gate blocks the gate.

---

## 12. Regional Pack Architecture (canonical + plug-in localization)

> **Update 2026-05-09:** The earlier "Korea-as-launch-locale" framing is retired. Korea is now one regional pack among many, not the architecture default. Oyatie ships a **canonical** architecture and **regional packs** plug into well-defined seams. This lets us launch and scale into multiple global markets *in parallel* rather than retrofit Korea-specific assumptions for every new locale.

### 12.1 What a regional pack is

A regional pack is a versioned, swappable bundle that supplies *all* per-locale concerns to the canonical architecture through declared seams. One pack per market.

A regional pack contains:

| Pack section | What's inside |
|---|---|
| `regulatory` | Regulator names, control-mapping tables, evidence-collection cadence, ADR cross-references (e.g. PIPA / GDPR / HIPAA / DPDP / LGPD / PDPL / APPI / Privacy Act AU). |
| `compliance_packs` | Vertical-cross-locale: e.g. healthcare regulator (MFDS, FDA, EMA, PMDA, MHRA, CDSCO), fintech regulator (FSC, OCC, FCA, FSA-JP, RBI), payment scheme (NACHA, FedNow, RTP, Pix, UPI, FPS, KFTC), labor/payroll (KR Labor Standards Act, US FLSA, EU Working Time Directive, JP 労働基準法). |
| `i18n` | Language(s), morphology / tokenizer impl (mecab-ko, MeCab-ja, NLTK-en, IndicNLP, Stanza, …), date/time conventions, address normalization (도로명/지번, 〒 + 都道府県市区町村, USPS, Royal Mail, AusPost, SEPA-CIF), name conventions, RTL support (Arabic, Hebrew). |
| `currency` | ISO-4217 currency, decimal precision, formatting, FX-source identity. |
| `calendar` | Holidays, working days, fiscal year boundary, school year, business-quarter convention. |
| `tax` | Tax-invoice format (전자세금계산서, 適格請求書 in JP, e-invoicing in EU mandates per country, GST in IN, CFDI in MX, NF-e in BR), tax-id format, tax-engine selection. |
| `identity_providers` | Local SSO/identity surfaces (KR 본인확인서비스 / 아이디카드, JP マイナンバーカード, EU eIDAS, US Login.gov, IN Aadhaar, BR ICP-Brasil, KSA Absher, UAE UAE-Pass, ANZ Digital ID). |
| `payment_rails` | Local rails (KR 카카오페이/네이버페이/토스/계좌이체, JP 振込/Pay, US ACH/Wire/RTP, EU SEPA/SEPA-Inst, IN UPI/RTGS, BR Pix, KSA SADAD/Mada, UAE UAEFTS/AaniPay, AU NPP, SG FAST). |
| `address_book` | Address validation, post code, geocoding source. |
| `ecosystem_partners` | Integrations the locale expects (Naver/Kakao in KR; Yahoo!JP/LINE in JP; Google/Facebook in US-EU; WeChat/Alipay where ToS allows; Apple/Google ID universally; KR `정부24` / EU `Once-Only`). |
| `content_safety` | Local content moderation rules (KR 청소년보호법 + 정보통신망법 + 게임물관리위원회; JP 児童ポルノ法 + 不正アクセス禁止法; US COPPA + child-safety statutes; EU DSA + online-platform rules; IN IT Rules 2021; UAE federal media council). |
| `ad_policy_gate` | Local ad review workflows (KR 의료광고/금융광고/정치광고; US FTC/AdSafe; EU consumer-protection; ANZ TGA medical advertising; IN ASCI; KSA GAEH/SFDA). |
| `industry_data_models` | Per-locale clinical coding extensions (e.g. KR-EDI 보건의료, US-LOINC, JP-ReceiptCode), labor classifications (e.g. KR 통상임금, JP 賞与, US W-2/1099, EU GDPR-art-9), accounting standards (K-IFRS, J-GAAP, US-GAAP, IFRS, Ind-AS). |
| `vendor_partners` | Local cloud / colo partners (KR Naver Cloud / NHN / KT / Kakao; JP Sakura / IDC Frontier; EU OVH / Hetzner / Scaleway / IONOS; IN Yotta / Reliance Jio Cloud; …). |

### 12.2 The seams that regional packs plug into

Per the cohesion thesis, a regional pack must NEVER fork the canonical architecture. It plugs in via **published seams**:

| Seam | Where it lives | What the pack supplies |
|---|---|---|
| Regulator → control-mapping seam | `platform-regulatory-kernel` | A trait `RegulatoryPack { regulator_id, controls(), evidence_collector(), reporting_cadence() }` impl per pack. |
| Tokenizer seam (search axis) | `search-tokenizer-kernel` | `Tokenizer` trait impl per language family. |
| Tax-invoice formatter seam (cloud + saas billing) | `platform-billing-tax-kernel` | `TaxInvoiceFormatter` trait impl per locale. |
| Identity-provider adapter seam | `platform-identity-kernel` | `IdentityProvider` trait impl per local SSO. |
| Payment-rails adapter seam | `saas-billing-rail-kernel` (and `vertical-fintech-*` for vertical-fintech specifics) | `PaymentRail` trait impl. |
| Address-validator seam | `platform-address-kernel` | `AddressValidator` trait impl. |
| Ad-policy-gate seam | `ads-policy-kernel` | `LocalAdPolicy` trait impl per locale. |
| Content-safety seam | `platform-content-safety-kernel` | `ContentSafetyRules` trait impl. |
| Calendar / locale formatting seam | `platform-locale-kernel` | `LocaleFormatter` trait impl. |
| Industry-data-model extension seam | per vertical kernel | `LocalIndustryExtension` trait impl per (vertical × region). |

Regional packs are versioned independently and published as `pack-<region-code>-<version>` artifacts. A tenant binds to *one or more* regional packs at onboarding (rare to be more than one).

### 12.3 Why this is better than locale-special-case

- **Parallel global launch.** No one market gates another. KR-pack and JP-pack can ship in parallel.
- **No architectural drift.** Every market has the same canonical stack; only the pack differs.
- **Acquisition-ready.** A new market is "implement a pack" not "rewrite product."
- **Audit cleanliness.** Each pack is independently audited; the canonical core is audited once.
- **Vertical-cross-region.** A healthcare vertical can ship US-pack + EU-pack + KR-pack in parallel because each has its own regulator binding.

### 12.4 Initial pack roster (v0.1 — to be ratified by Architecture Council)

| Pack id | Region | Wave first onboarded | Initial regulators | Local payment / identity highlights |
|---|---|---|---|---|
| `pack-kr` | South Korea | W-Cloud-Preview | PIPA, KISA, MFDS, FSC, KCC, NIS, CSAP, K-ISMS-P, KCMVP | 본인확인서비스, 카카오/네이버/토스, 전자세금계산서 |
| `pack-jp` | Japan | W-Cloud-Preview (parallel) | APPI (PPC), PMDA, JFSA, ISMAP | マイナンバー, MUFG/Mizuho/SMBC, 適格請求書 |
| `pack-us` | United States | W-Cloud-Preview (parallel) | HIPAA/HITECH, SOX, CCPA/CPRA, FedRAMP, OCC, FDA | Login.gov, ACH/Wire/RTP, NACHA, IRS forms |
| `pack-eu` | European Union (DE first, then FR/SE/NL/IE) | W-Cloud-Preview (parallel) | GDPR, DORA, EU AI Act, GAIA-X, EMA | eIDAS, SEPA / SEPA-Inst, e-invoicing per country |
| `pack-in` | India | W-Region-Fan-Out wave 1 | DPDP Act, RBI, MeitY, CDSCO | Aadhaar, UPI/RTGS/IMPS, GST e-invoicing |
| `pack-br` | Brazil | W-Region-Fan-Out wave 1 | LGPD, ANS, ANVISA, BACEN, ICP-Brasil | gov.br, Pix, NF-e |
| `pack-ksa` | Saudi Arabia | W-Region-Fan-Out wave 2 | PDPL, NDMO, SDAIA, SAMA, SFDA | Absher, SADAD/Mada, FATOORA |
| `pack-ae` | UAE | W-Region-Fan-Out wave 2 | TDRA / ADGM / DIFC, UAE-CB | UAE-PASS, UAEFTS / AaniPay, e-invoicing |
| `pack-au` | Australia + NZ | W-Region-Fan-Out wave 2 | Privacy Act 1988, IRAP, TGA, ASIC, RBA | myGovID, NPP, OSKO |
| `pack-sg` | Singapore | W-Region-Fan-Out wave 2 | PDPA-SG, MAS, HSA, IMDA | Singpass, FAST/PayNow, e-invoicing IMDA |
| `pack-mx` | Mexico | W-Region-Fan-Out wave 3 | LFPDPPP, INAI, COFEPRIS, CNBV | e.firma, SPEI, CFDI |
| `pack-id` | Indonesia | W-Region-Fan-Out wave 3 | UU PDP, BPOM, OJK | PeduliLindungi heir, RTGS, BI-FAST |
| `pack-ph` | Philippines | W-Region-Fan-Out wave 3 | DPA-2012, BSP, FDA-PH | PhilSys, InstaPay/PESONet |
| `pack-vn` | Vietnam | W-Region-Fan-Out wave 3 | Cybersecurity Law, MOH, SBV | NAPAS, e-invoicing |
| `pack-th` | Thailand | W-Region-Fan-Out wave 3 | PDPA-TH, BoT, TFDA | NDID, PromptPay |
| `pack-tr` | Turkey | W-Region-Fan-Out wave 3 | KVKK, BDDK, TİTCK | e-Devlet, FAST, e-Fatura |
| `pack-ng` | Nigeria | W-Region-Fan-Out wave 4 | NDPR, CBN, NAFDAC | NIN, NIBSS Instant Payment |
| `pack-za` | South Africa | W-Region-Fan-Out wave 4 | POPIA, SARB, SAHPRA | RSA-ID, RPP |

The list expands as council ratifies new packs. Pack onboarding is an **independent workstream** per pack; no pack blocks another after the seam contracts are stable.

### 12.5 Data residency interaction

A regional pack declares its **default residency class** for tenants in that region. Cross-region replication is opt-in per residency class (per Data Use Boundary ADR §2.2.1). The canonical residency model lives in `platform-tenant-kernel`; the pack supplies the local residency choice and any cross-border transfer constraints (e.g., GDPR Schrems III, KR Art 28-8, Russia data-localization law if onboarded).

### 12.6 Industry-data model extension

A vertical (e.g. healthcare) declares its canonical model in its kernel; per-region extensions plug in. Example: `vertical-healthcare-kernel` defines `Patient`, `Encounter`, `Observation` (FHIR-aligned); `pack-kr` extends with `KrPatientId`, `KrInsurancePayer (NHIS)`, `KrRRN`; `pack-jp` extends with `JpPatientId`, `JpHokenSha`; etc. The vertical kernel never imports a region; regions extend it.

This pattern repeats for every vertical and every region — the math is `verticals × regions` extensions but the **canonical core is `verticals` + `regions`** (sum, not product). That sum-not-product property is why parallel global launch is possible.

---

## 13. Per-axis design notes (one section per axis — v0.2 expansion ledger)

This section will expand to ~3-5 pages per axis in v0.2 once the recon agents land.

### 13.1 SaaS multi-tenant platform

> **v0.2 expansion scope** — workflow engine (ADR-0035), Ontology (ADR-0006..0112; legacy "Object Graph" — renamed per MASTERPLAN.md §2.4), plugin substrate (#29), surface, marketplace, public API surface, tenancy isolation enforcement (#1570), cross-product auth (ADR-0006), metering/quotas substrate (#1576).

### 13.2 Vertical industry cloud

> **v0.2 expansion scope** — per-vertical depth: healthcare (clinical canonical record, FHIR/HL7, medication/allergy/problem reconciliation, e-prescribing, claims), industrial (MES, OEE, ISA-95, OPC UA, SCADA/historian), logistics (shipment, dock, EDI 214/990/997, route optimization, HOS, cold-chain), fintech (PG, open-banking, KYC/KYB, AML, NACHA/RTP), corporate (HR, payroll, GL, mail, comms), legal (regulated corpus, contract analysis), and others. Each per-vertical ADR cluster cited.

### 13.3 AI agent runtime (Foundry — consolidated 2026-05-09)

> **v0.2 expansion scope** — capability registry, autonomy ceiling, evidence chain, model registry, eval harness, RAG pipeline, multi-provider adapters (Anthropic / OpenAI / Gemini × subscription + API), worker host execution, app-server turn proof, secrets / SecretProvider, smoke lane, daemon hardening (#1266-#1268), AutonomyCeiling policy (#1279).

#### 13.3.1 Top-20 Foundry improvements (synthesized from `/Users/jasonlee/oyatie/docs/raw/foundry-improvements.md`)

The Foundry-improvements research (2026-05-09, 128 leaves) surfaced these high-impact additions. Each maps to a backlog leaf in [ROADMAP.md](ROADMAP.md):

| # | Improvement | Impact / Urgency | Notes |
|---|---|---|---|
| 1 | **Autonomy ceiling per tenant per data-class as RUNTIME gate** (not docs) | 5 / 5 | Closes ADR-0022 + ADR-0003 implementation gap |
| 2 | **MCP gateway exposing capability registry** (industry-standard tool surface) | 5 / 5 | Now in [TOOLCHAIN.md §4.A](TOOLCHAIN.md) |
| 3 | **Capability whitelist per tenant + workflow** (least privilege) | 5 / 5 | Cedar policy + runtime gate |
| 4 | **Anthropic prompt-cache breakpoint management** | 4 / 5 | Cost reduction; API-mode-specific |
| 5 | **Token + cost budgets per (tenant, capability, time-window)** | 5 / 5 | Hard ceiling at exhaustion |
| 6 | **Structured output enforcement** (JSON Schema validation in adapter) | 4 / 4 | Industry standard |
| 7 | **Provider-agnostic capability descriptor** | 4 / 5 | Same capability runs on any provider |
| 8 | **Streaming response normalization** across providers | 4 / 4 | Adapter-layer concern |
| 9 | **Prompt-injection taint zones** | 5 / 5 | Mark untrusted content; downstream tools refuse |
| 10 | **LangGraph-style state-machine for agent runs** | 5 / 4 | Replaces ad-hoc orchestration |
| 11 | **Cross-axis evidence chain in ONE Merkle DAG** (only-Foundry-can-do) | 5 / 3 | Top differentiator; spans Cloud + Workflow + Search + Ads + Foundry |
| 12 | **Privacy-aware autonomy ceiling per region** | 5 / 4 | Couples Cedar + Data Use Boundary + Regional Pack |
| 13 | **Cohesion fitness function for cross-axis drift** | 5 / 4 | CI lane; without this the cohesion thesis collapses |
| 14 | **OpenTelemetry `gen_ai` semconv** for observability | 4 / 4 | Industry semantic conventions |
| 15 | **Cost-aware model routing** | 4 / 3 | Multi-provider router selects cheapest viable |
| 16 | **Subscription token rotation** + idle refresh | 4 / 5 | Subscription-mode-specific |
| 17 | **Macaroons-style capability tokens** | 5 / 4 | Fine-grained delegation |
| 18 | **Reflection / self-critique with budget caps** | 4 / 3 | Quality lift + budget bound |
| 19 | **Tool-call parallelism with dependency DAG** | 4 / 4 | Faster agent runs |
| 20 | **Rerank-after-call** (sample N, pick best) | 4 / 3 | Cost-budgeted quality lift |

#### 13.3.2 PRD-shaping consequences (10 items that change product shape)

1. **MCP gateway** is a top-level Foundry product surface (now in TOOLCHAIN.md §4.A).
2. **Autonomy ceiling becomes a runtime gate**, not a descriptive ADR. Implementation must precede external agent traffic.
3. **Capability semver + sunset** is a schema requirement, not a convention.
4. **Cohesion fitness function CI lane** is mandatory; without it cross-axis drift goes undetected.
5. **Cross-axis evidence chain** spans every axis in ONE Merkle DAG. This is Oyatie's strongest differentiator.
6. **Subscription vs API treated as access modes** of one runtime with first-class failover, not two products.
7. **Foundry-as-a-product** deployment: customers can consume Foundry directly via MCP without using Oyatie SaaS. Materially expands TAM.
8. **"Live provider execution disabled by default" posture (per ADR-0025)** must flip with a clear gate; current posture blocks GA.
9. **Recursive Foundry**: foundry surfaces themselves are agent-extended; this needs explicit PRD callout to direct plugin authors.
10. **Per-capability eval contract**: golden tasks + nightly run + regression gating is a top-level PRD requirement, not a per-team option.

### 13.4 Foundry — Foundry surfaces (consolidated 2026-05-09)

The foundry is a *persona slice* of Foundry, not a separate axis. The same agent runtime that authors workflows also authors PRs; the same fitness function that gates a PR also evaluates an agent step.

> **v0.2 expansion scope** — detailed coverage of repoctl split (see ROADMAP.md §8 Q16), catalog, claim-ceiling validator, foundation-bypass ledger, plane-gated CI lanes, ADR templates, scorecards (Move #7), fitness functions (Move #8), CODEOWNERS coverage, branch-protection-as-code (#239 / #1295), Kyverno/OPA admission, signed commits (#1299), supply-chain (#614 ADR-0039 Trivy 4-layer), plugin substrate trust gates (ADR-0036/0157/0161/0162), plugin marketplace authoring, marketplace economics.

#### 13.4.1 Persona-split CLI (recommended)

The current monolithic `repoctl` is recommended for split into 8 persona-CLIs all under the `oya` brand binary group:

| CLI | Persona | Loaded dep tree | Versioned independently |
|---|---|---|---|
| `oya dev` | Internal engineer + OSS contributor | clean / fmt / build / lint / arch-boundary | yes |
| `oya admin` | Tenant admin (customer) | tenant + identity + billing + consent + DSR | yes (mirror of REST) |
| `oya build` | Customer builder (workflow / plugin authoring) | workflow + plugin + pack scaffold + sign | yes |
| `oya agent` | Foundry agent (internal) | foundry runtime + capability + evidence | yes |
| `oya ops` | SRE / Ops | cell / region / deploy / runbook / drill | yes |
| `oya pack` | Regional pack maintainer | pack build / verify / publish | yes |
| `oya catalog` | Catalog + capability authoring | catalog scaffold + promote + supersede | yes |
| `oya gate` | Gates + bypasses + claim-ceiling | gate ratchet + bypass + claim verify | yes |

Crate targets: `crates/tooling-cli-{dev,admin,build,agent,ops,pack,catalog,gate}-*`. Migration: `repoctl <cmd>` continues as a deprecated alias for ~2 waves per ADR-0001.

#### 13.4.2 Why the split is structurally important

- **Clean-arch persona boundary:** each persona's CLI imports only its own use-cases; no leak.
- **Smaller blast radius:** a bug in `oya admin` cannot affect `oya dev`.
- **Versioning independence:** `oya admin` versions track tenant-API contract; `oya dev` versions track engineering tools. Decoupled release cadence.
- **Documentation isolation:** each CLI has its own `--help` tree owned by its persona team.
- **Auth model:** `oya admin` ships with OAuth2 device flow for customers; `oya dev` uses developer SSO; `oya agent` uses Foundry capability tokens. Mixing auth models in one binary is a source of confusion.

### 13.5 Cloud provider (AWS-class)

> **v0.2 expansion scope** — compute (k8s/serverless/VM/bare-metal/GPU/edge), storage (object/block/file/archive/databases/backup), network (VPC/LB/DNS/CDN/interconnect/DDoS/mesh), IAM (Cedar + SSO + STS), regions/AZs/cells, billing (metering + invoice + tax + reserved), marketplace, observability, FinOps, KR CSAP/K-ISMS-P/망분리/전자세금계산서.

### 13.6 Search engine (Google/Naver-class)

> **v0.2 expansion scope** — crawler (politeness/robots/sitemaps/render farm/dup-detect/spam/tenant-private), parser (HTML/PDF/OCR/transcript/Korean morphology/NER/embeddings), indexing (inverted/vector/KG/geo/image/video/per-tenant), ranking (BM25 + semantic rerank + freshness + authority + diversity + Korean signals), QU (parser/expansion/spelling/autocomplete/QA/RAG), SERP, infra (sharding/replication/cell/tiering), safety (RTBF/PIPA/GDPR/youth-protection/audit), search↔Foundry RAG, search↔Ads sponsored slot, KR (Naver-API/Daum-API/저작권 compliance).

### 13.7 Advertising + analytics (Google-class)

> **v0.2 expansion scope** — ad serving (sub-100ms / auction / inventory / pacing / brand-safety / quality / targeting / retargeting / multi-format), pricing (manual/CPA/ROAS/smart-bidding/fraud), attribution (click/view/multi-touch/cross-device/server-API/offline/privacy-preserving), advertiser console (campaign/asset/audience/tag/budget/recommendations/API), publisher inventory (onboarding/header-bidding/payout), analytics (event/journey/cohort/funnel/retention/lift/A-B/dashboard/warehouse/streaming/DP), Data Use Boundary enforcement, ad quality + KR policy (KODA/의료/금융/정치), clean-arch + horizontal-scale, KR (Naver 검색광고 conversion-API parity, Kakao Moment integration possibility), ads↔search, ads↔SaaS-tenant-data, ads↔agent-runtime (autonomy-ceiling-gated agentic buying).

---

## 14. Anti-patterns (will not do)

1. **Per-axis tenancy.** One tenancy model. Forever.
2. **Per-axis billing.** One billing event stream. Forever.
3. **Per-axis identity.** One identity. Forever.
4. **Per-axis audit chain.** One audit chain. Forever.
5. **Bolted-on agent runtime.** Foundry is an axis from day one, not a chatbot UI on top.
6. **Region as a per-service decision.** Region is a tenant-level decision; every service inherits.
7. **PHI / PII / PCI in ad targeting via "consent."** Always blocked, regardless of consent.
8. **Multi-region day one.** KR-Seoul1 first; multi-region when tenancy + DR + regulatory machinery are stable.
9. **`modules/` `services/` `platform/` re-deepening.** Migration is forward-only.
10. **Self-approval in agent contexts.** Authoring and review pass are separate (per project memory).

---

## 15. Sources scanned

- ADRs: `decisions/ ADR-0001..ADR-0051` (51 files; index at [ADR-INDEX.md](ADR-INDEX.md))
- Flat-crates plan: `decisions/ADR-0015-architectural-flattening-target.md` (REV7)
- Master plan: `docs/ROADMAP.md`
- Constitution: `CONSTITUTION.md`
- Source of truth: `docs/DOC-CATALOG.md` (per [`DOC-CATALOG.md`](DOC-CATALOG.md))
- Mistakes & fixes: `docs/MISTAKES-LEDGER.md`
- Audits 2026-05-01 (ADR consistency), 2026-04-22 (architecture sweep), 2026-05-09 (Foundry conformance)
- Ultragoal: `.omx/ultragoal/brief.md`, `goals.json`, `issue-priority-pipeline/queue.md`
- Home-dir gap inventory: `/Users/jasonlee/oyatie/docs/raw/gap-home-dir.md`
- ADR index: `/Users/jasonlee/oyatie/docs/raw/adr-index.md`
- Greenfield research artifacts (cloud/search/ads — sourced inline as the agents land)
- User directives 2026-05-08 (rename) + 2026-05-09 (axes + cohesion + Foundry-as-accelerator)

*Footer regenerated whenever DESIGN.md is edited.*
