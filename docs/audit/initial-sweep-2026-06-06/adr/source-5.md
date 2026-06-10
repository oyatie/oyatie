# ADR Audit — SOURCE, Chunk 5

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 5 of the source ADR sweep
- **Slice range (ls lines 29–35):** ADR-0030, ADR-0031, ADR-0032, ADR-0034, ADR-0035, ADR-0036, ADR-0037
- **ADRs actually reviewed (7):** 0030, 0031, 0032, 0034, 0035, 0036, 0037
- **Era:** all dated 2026-05-09 (several "rewritten 2026-05-13" to swap the retired "axis"/"vertical" vocabulary for "microservice"). This is the post-foundation product-architecture expansion. Cluster signature: every ADR is a per-microservice/per-substrate design that leans hard on the six cohesion substrates (Tenant/Identity/Audit/Capability/Runtime/Autonomy) and on the (now-retired) "Foundry" brand.

---

### ADR-0030 — Search microservice (crawler/parser/index/ranker/SERP, KR-first morphology, Data-Use-Boundary segregation)

- **decision_atom:** Search is a single flat microservice built as a five-stage pipeline (Crawler→Parser→Indexer→Ranker→SERP) with per-tier physical index segregation enforced at the Indexer and cross-tier query hard-failed without an explicit Data-Use-Boundary grant; KR morphology (mecab-ko/khaiii) and Naver/Kakao integration are day-1, and Foundry/Ads access is Workflow-mediated only.
- **current_status:** accepted / published.
- **disposition:** AMEND. Sound and load-bearing, but carries retired vocabulary and a superseded substrate ref: "Foundry" brand (RAG bridge `oya-search-foundry-bridge-adapter`, capability `workflow.search.rag`, "Foundry agents") must rename to **intelligence** per ADR-0335/0347; the vector-index citation to **ADR-0046 (HNSW+IVF)** is to a now-Superseded ADR (governing = ADR-0192 Milvus / pgvector ≤10M per ADR-0046→0192 edge). Crate `oya-search-indexer-adapter` says "Tantivy + pgvector + HNSW" — reconcile to canonical Milvus-for-vector posture.
- **governing:** ADR-0335 (foundry→intelligence brand retire), ADR-0347 (oya-foundry-*→oya-governance-* lane rename), ADR-0192 (Milvus canonical, supersedes ADR-0046).
- **truth_flag:** TRUE (the architecture is real and current) with STALE refs (foundry brand + ADR-0046 vector store).
- **in_masterplan:** PARTIAL — no `masterplan_ref`/`planning_impact` front-matter; only `id/status/doc_status`. The decision is plausibly reflected as a microservice in the flat catalog but the ADR does not carry the planning binding the drift-prevention gate wants (corpus-wide 8.8% binding problem).
- **tensions:** (a) Retired-brand leakage vs ADR-0335/0347 (foundry strings). (b) ADR-0046 vector-store ref is to a Superseded ADR (vs ADR-0192). (c) Cross-ref to ADR-0031 ads-gate is consistent (reciprocal). (d) References ADR-0047 (search backend) and ADR-0048 (KR morphology) outside this slice — verify those still live. (e) LINUX side has no Search ADR; no cross-side conflict, but LINUX ADR-0001 "own-the-DB" would, if adopted, change the indexer/vector substrate assumptions.
- **hyperscaler_challenge:** Verdict **aligned**. Google/Bing-class search is exactly crawler→parse→index→rank→serve with locale morphology packs and policy-gated tiers; per-tier physical segregation for regulated data is how a hyperscaler isolates blast radius. The KR-first morphology moat is a legitimate regional-incumbent strategy (Naver). The one questionable piece a hyperscaler would push back on: building a full render farm + crawler + in-house Korean NLP day-1 is enormous scope for a startup — argues for staging, not for amend/archive of the decision itself.
- **ai_slop:** Low. Mild fabricated precision (sub-100ms P95 as a "binding constraint", exact crate enumeration) but it is design intent, not invented fact. "Foundry" references are retired-vocab leakage, not slop.
- **refinement:** Rename foundry→intelligence in the bridge crate + capability id; repoint vector-store ref from ADR-0046→ADR-0192; add `planning_impact`/`masterplan_ref` front-matter; mark KR render-farm/crawler as a staged (wave) deliverable rather than implied day-1.
- **consensus_needed:** no (clear amend; brand + ref fixes are mechanical once founder confirms intelligence naming, which is already given).

---

### ADR-0031 — Ads + Analytics microservice (singleton tenant-ads-gate, sub-100ms auction, DP-budgeted analytics, runtime DUBO)

- **decision_atom:** All ad sourcing across the ecosystem flows through exactly one singleton **tenant-ads-gate** (holding Cedar policies + DUBO consent receipts), with a five-pillar Ads design and a differential-privacy/k-anonymity-budgeted Analytics design, so the Data-Use-Boundary is mechanically enforceable and ad-bidding signals never reach Search ranking.
- **current_status:** accepted / published.
- **disposition:** KEEP (with a minor naming nit). The singleton-gate decision is crisp, current, non-conflicting, and well-formed; it is masterplan-ready. Optional amend only for the analytics warehouse ref.
- **governing:** n/a (not superseded). Watch ADR-0045 ("OLAP tier") vs canonical ClickHouse posture (ADR-0193) — the crate `oya-analytics-warehouse-adapter` already says "ClickHouse OLAP tier", which is consistent with canonical, good.
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL — same front-matter gap (no planning binding). The singleton-gate rule is exactly the kind of structural invariant the masterplan should carry; currently not bound.
- **tensions:** (a) Reciprocal with ADR-0030 (Search sponsored slots) — consistent, no conflict. (b) References ADR-0007 (Cedar) + ADR-0008 (DUBO) — consistent with canonical Cedar-as-universal-gate posture (ADR-0243/0246). (c) "ADR-0045 OLAP tier" is the only soft ref; canonical OLAP is ClickHouse (ADR-0193) — verify ADR-0045 is the same decision or a superseded alias. (d) Singleton = single hot reliability surface (the ADR self-flags gate-outage = no ads anywhere).
- **hyperscaler_challenge:** Verdict **aligned**. A single mandatory ad-sourcing chokepoint for consent/policy enforcement mirrors Google's privacy-sandbox/single-ad-server discipline and AWS Clean Rooms-style DP aggregation; sub-100ms auction + last-click/MTA + IVT is industry-standard. The reliability concern (singleton SPOF) is real but is precisely how the majors run a central ad-decision service (with regional replication) — argues for an HA/replication note, not for changing the decision.
- **ai_slop:** Low. KR pre-clearance legal cites (의료/금융/정치/청소년 광고) are specific and plausible. No fabricated internal contradictions.
- **refinement:** Add planning front-matter; add an explicit HA/replication clause for the singleton gate (region-replicated, not a literal global single instance); reconcile ADR-0045 vs ADR-0193 OLAP naming.
- **consensus_needed:** no.

---

### ADR-0032 — DCIM software for Oyatie-owned DC ops (`oya-cloud-dcops-*`) with anti-scope on custom silicon

- **decision_atom:** Build the data-center infrastructure-management stack in-house as `oya-cloud-dcops-*` consuming the six cohesion substrates (rejecting off-the-shelf DCIM as a cohesion violation), while holding a hard anti-scope line that no custom silicon (CPU/NIC/switch-ASIC/optical) is ever designed — commercial silicon only, revisitable solely by founder ratification.
- **current_status:** proposed / published.
- **disposition:** AMEND (and flag as a high-ambition, far-horizon Proposed item). The anti-scope half is excellent and keepable; the build-in-house DCIM half is a Phase-2+ greenfield-mega-DC dependency (depends on ADR-0028's three-phase trajectory) that may be premature relative to source's "best-of-breed-now, own-when-proven" ratchet. Lane name already modernized to `oya-governance-dcim-substrate` (good — post-0347). Owner is bare `cloud`.
- **governing:** n/a (not superseded). Tension owner: ADR-0028 (cloud trajectory) gates whether this is even in scope yet.
- **truth_flag:** PARTIAL — the anti-scope decision is TRUE and durable; the in-house-DCIM build decision is aspirational/STALE-risk (predicated on owning physical mega-DCs that are years out; "6–12 person-years" self-admitted).
- **in_masterplan:** PARTIAL — Proposed, no planning binding; references ADR-0042 (observability, now Superseded→ADR-0383) in Related, a stale ref.
- **tensions:** (a) **Own-vs-buy breadth tension** (keystone fault-line #5): in-house DCIM is the same "OWN_DAY0" ambition the audit flags; source's own ratchet (ADR-0211/0173) and the founder's "own-when-proven" lean argue for buying/adapting DCIM until the owned mega-DC actually exists. (b) Related-ref to **ADR-0042** is to a Superseded ADR (governing ADR-0383 Loki/Tempo/Mimir/Grafana). (c) "Foundry agents"/"Foundry runtime" references = retired brand (→intelligence, ADR-0335). (d) Hard-coded autonomy ceiling (persona-tier ≥ proxy + human approval) is consistent with the autonomy-tier model — good.
- **hyperscaler_challenge:** Verdict **questionable** (on the build-now half), **aligned** (on anti-scope). Google/AWS/Azure absolutely build bespoke DCIM/BMS at scale — but only after they own DCs; none built DCIM before owning a single facility, and all three eventually *did* design custom silicon (TPU, Graviton/Nitro, Cobalt/Maia) once scale justified it. So the blanket "never design silicon" anti-scope is *more* conservative than any hyperscaler and is correctly gated behind founder-ratification revisit — that is the right escape hatch. The build-in-house-DCIM-day-0 posture argues for **amend** (defer to Phase 2, adapt OSS DCIM until then), not archive.
- **ai_slop:** Low-moderate. Heavy enumeration (12 bounded contexts, vendor adapter list, regulatory cert alphabet soup) reads as fabricated-precision/breadth-signaling for a Proposed ADR with no DC yet. Open questions Q1–Q5 are genuine, not filler.
- **refinement:** Split the ADR: (1) durable anti-scope-on-silicon decision (KEEP, promote to Accepted), (2) in-house-DCIM build (defer/Proposed, gated on ADR-0028 Phase 2). Repoint ADR-0042→ADR-0383. Rename foundry→intelligence. Add planning front-matter tying to ADR-0028 phasing.
- **consensus_needed:** yes — *"Do we build DCIM in-house from day-0, or adopt/adapt OSS DCIM behind ports until Oyatie actually owns physical DC capacity (ADR-0028 Phase 2)? And is the absolute no-custom-silicon anti-scope still correct given every hyperscaler eventually built silicon at scale?"*

---

### ADR-0034 — Per-microservice data-class overrides (hard-deny pack tenant admin cannot raise)

- **decision_atom:** Every flat-catalog microservice that handles regulated data ships an immutable, kernel-resident **override pack** (data-class → hard-deny scope) that binds at highest precedence in the Cedar evaluation chain so a tenant admin can only ever narrow (deny more), never raise, the data-use ceiling — making "tenant opts PHI into ad sourcing" structurally impossible.
- **current_status:** accepted / published.
- **disposition:** KEEP. Crisp, current, well-formed, regulator-grounded, non-conflicting; masterplan-ready as a structural invariant. (Minor: one table header literally reads "microservice (mail/calendar/chat/docs)" — a copy-paste artifact where the µservice name was dropped; cosmetic amend.)
- **governing:** n/a (not superseded). Builds on ADR-0007 (Cedar) + ADR-0008 (DUBO).
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL — strong candidate invariant for the masterplan; currently only `id/status/doc_status` front-matter, no planning binding.
- **tensions:** (a) Cedar evaluation-order precedence (override > regional overlay > tenant) must stay consistent with ADR-0007/ADR-0243/0246 (Cedar universal gate) and the regional-pack architecture — appears consistent. (b) References ADR-0030/0031 (ad sourcing) reciprocally — consistent. (c) The ≤1ms p99 hot-path budget is a real perf coupling to the Cedar evaluator. (d) No retired-vocab leakage here (owner `council-architecture`, lane unnamed but clean).
- **hyperscaler_challenge:** Verdict **aligned**. This is exactly how AWS (SCP/permission-boundaries), GCP (org-policy constraints that admins cannot loosen), and Azure (Azure Policy deny effects) implement non-overridable guardrails — a top-precedence deny floor that lower-scope grants cannot relax. Strong fit; argues for KEEP and promotion into the masterplan as a guardrail primitive.
- **ai_slop:** Low. The data-class/hard-deny tables are concrete and legally anchored. The only artifact is the dropped microservice name in one header.
- **refinement:** Fix the "microservice (mail/calendar/chat/docs)" header (likely "connect"/"workspace"); add planning front-matter; cross-link to ADR-0007 evaluation-order spec to keep precedence single-sourced.
- **consensus_needed:** no.

---

### ADR-0035 — Workflow engine: hybrid state-machine + DAG (not pure BPMN), per-tenant versioning, jurisdiction overlay, agent-authored steps

- **decision_atom:** Build one canonical ecosystem workflow engine (`oya-workflow-*`) as a hybrid where every workflow is a top-level finite-state-machine with per-state DAGs, first-class per-tenant version pinning, runtime per-jurisdiction overlays, saga compensation across microservices, sealed (immutable) steps, and agent-authored steps hard-bounded by autonomy ceiling — rejecting pure BPMN, pure DAG, and Temporal.
- **current_status:** proposed / published.
- **disposition:** AMEND. The engine decision is sound and probably canonical, but: (a) owner = `foundry` (retired brand → intelligence/governance, ADR-0335/0347); (b) it predates and ignores the CI-side adoption of **Argo Workflows** as the *CI/CD* orchestrator (ADR-0511) — these are different layers (business-process workflow vs build/deploy DAG) and should be explicitly disambiguated so the "we reject Argo Workflows" line in Alternative B is not misread as contradicting CI canon; (c) heavy `vertical_id`/`VerticalId` typing survives despite the "vertical→microservice" rename elsewhere.
- **governing:** ADR-0335/0347 (brand/owner rename). No supersession of the decision itself. Layer-disambiguation reference: ADR-0511 (Argo Workflows = CI, not business workflow).
- **truth_flag:** TRUE (decision) with STALE wrapping (foundry owner, VerticalId vocabulary, undated W+N milestones).
- **in_masterplan:** PARTIAL — Proposed; no planning binding; a canonical cross-cutting engine like this should be a named masterplan node.
- **tensions:** (a) **Argo Workflows naming collision** — Alt-B rejects "Airflow/Dagster/Argo Workflows" for business workflow while CI canon (ADR-0511) *adopts* Argo Workflows for CI; both are correct (different layers) but the corpus needs a disambiguation note or a reader will flag a false contradiction. (b) Owner `foundry` retired. (c) Cross-ref to "ADR-0034 (per-vertical override)" uses stale title — ADR-0034 is now per-*microservice* override. (d) `VerticalId` typing vs flat-catalog "vertical retired" doctrine (ADR-0058/0362-adjacent). (e) References ADR-0045 (DB tier) and ADR-0042 (observability, Superseded→0383). (f) Temporal rejection rationale ("if we need a layer above it anyway, own it") is the same own-vs-buy posture as fault-line #5 — consistent internally but is exactly the breadth bet a hyperscaler would scrutinize.
- **hyperscaler_challenge:** Verdict **questionable**. The hybrid FSM+DAG model is technically right and mirrors AWS Step Functions (state machine) + the saga/compensation pattern; Google/Azure use Temporal-class durable execution or Cloud Workflows rather than owning a bespoke engine. A hyperscaler would likely *adopt/adapt Temporal or build on Step-Functions-class primitives* rather than own a full BPMN-alternative engine day-0 — so the "reject Temporal, own the engine" call argues for at least a staged adoption (Temporal/durable-exec now, own-when-proven per the ratchet). Argues for **amend** (re-justify own-vs-adopt against ADR-0211/0173 ratchet), not archive.
- **ai_slop:** Low-moderate. Rich but coherent. Minor fabricated precision in W+N defaults (W+12 collaborative editor, 30s agent-step budget). Alternatives A–D are substantive, not filler.
- **refinement:** Rename owner foundry→intelligence; add a one-line "Argo Workflows here = business workflow engine, distinct from ADR-0511 CI orchestration" disambiguation; rename VerticalId→MicroserviceId (or pack id) per flat-catalog doctrine; repoint ADR-0042→0383; re-examine own-vs-Temporal against the own-when-proven ratchet; add planning front-matter.
- **consensus_needed:** yes — *"Do we own a bespoke hybrid FSM+DAG workflow engine from day-0, or adopt a durable-execution substrate (Temporal/Step-Functions-class) behind a port and own only the per-tenant-versioning + jurisdiction-overlay layer until owning the engine is proven necessary (per the own-when-proven ratchet)?"*

---

### ADR-0036 — Plugin substrate: Wasmtime + WASI Preview 2, capability-gated context, Cosign signing, trust tiers, marketplace economics

- **decision_atom:** Adopt Wasmtime + WASI Preview 2 as the one plugin runtime, exposing only a capability-gated `PluginContext` (no raw syscalls), with Cosign-keyless+Rekor signing/transparency, three trust tiers (verified-isv/community/experimental), per-tenant resource caps, an 80/20 marketplace revenue split, and a multi-language SDK (Rust/TS/Python/Go).
- **current_status:** proposed / published.
- **disposition:** AMEND. The runtime + capability-gating + signing decisions are strong and align with canonical wasmtime posture (ADR-0200); amend for retired brand (`oya-foundry-plugin-runtime-kernel`, owner `foundry` → intelligence, ADR-0335/0347) and to confirm the marketplace-economics half (revenue %, payout, refund) is the right authority surface for an ADR vs a business-policy doc.
- **governing:** ADR-0200 (wasmtime canonical — reinforces this choice), ADR-0335/0347 (brand/owner rename).
- **truth_flag:** TRUE (runtime/security decision) with STALE branding (foundry crate prefix + owner).
- **in_masterplan:** PARTIAL — Proposed; no planning binding; the wasmtime+capability-gate substrate is canonical-posture-aligned (ADR-0200) and should be a masterplan node.
- **tensions:** (a) **Reinforces, does not conflict with, canonical wasmtime (ADR-0200)** — good cross-corpus consistency. (b) `oya-foundry-plugin-runtime-kernel` crate prefix = retired `oya-foundry-*` (→ `oya-intelligence-*` or governance, ADR-0347 lane-rename doctrine). (c) License-tier check lists "forbidden: SSPL/AGPL/BUSL/BSL" — consistent with OSI-strict license posture (ADR-0013/0211/0345); the Wasmer-rejection reasoning ("license posture shifted") is consistent with that same policy. (d) Cosign/Rekor/SLSA supply-chain refs tie to ADR-0039 (supply chain) — verify live. (e) Cross-ref to "ADR-0034 (per-vertical override)" again uses the stale per-vertical title.
- **hyperscaler_challenge:** Verdict **aligned**. WASM/WASI capability-sandboxed plugins with Sigstore signing and trust tiers is precisely the modern hyperscaler/edge-plugin model (Cloudflare Workers/Fastly Compute on wasmtime; Shopify Functions on WASM; Sigstore is CNCF/industry standard). Rejecting native .so and per-call containers on isolation/latency grounds is the correct call any of the three would make. The 80/20 marketplace split mirrors Apple/Google's evolved-down app-store economics — defensible. Argues KEEP-the-technical-core, amend-the-brand.
- **ai_slop:** Low. Concrete manifest schema, real runtime/license reasoning. The marketplace-economics block is the softest (precise 80/20, T+30, 14-day refund read as fabricated-precision for a Proposed engineering ADR) — better owned by a business-policy doc.
- **refinement:** Rename foundry→intelligence in crate + owner; split marketplace economics into a separate business-policy ADR/doc (keep the engineering substrate here); cross-link ADR-0200 as the canonical-runtime parent; fix the ADR-0034 "per-vertical" title ref; add planning front-matter.
- **consensus_needed:** no (technical core is clear; brand rename is mandated; the only soft question — marketplace econ home — is editorial, not load-bearing).

---

### ADR-0037 — Public API stability tiers: preview/stable/GA, semver-diff PR gate, contract-first SDK generation, per-deprecation telemetry

- **decision_atom:** Govern every public API with three stability tiers (preview/stable/GA) carrying defined breaking-change/deprecation-lead/SLA rules, a per-PR `oya contract-diff` semver gate, contract-first artifacts under `contracts/` (OpenAPI/proto/AsyncAPI/GraphQL) that auto-generate per-language SDKs, mandatory per-deprecation audit-chain telemetry, and a tenant-facing trust-portal mirror.
- **current_status:** proposed / published. (Has `sunset_topic`/`sunset_milestone: doctrine-not-time-bounded` front-matter — i.e., a standing doctrine, not a dated deliverable.)
- **disposition:** KEEP. Well-formed, internally consistent, doctrine-grade governance; the lane name is already modernized (`oya-governance-api-semver`, post-0347). Masterplan-ready as an API-governance invariant.
- **governing:** n/a (not superseded).
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL — richer front-matter than its siblings (carries `sunset_topic`/`sunset_milestone`) but still no `masterplan_ref`/`planning_impact`; should be a named masterplan governance node.
- **tensions:** (a) The per-axis ownership table still uses "axis" vocabulary (`axis-workspace`, `axis-vertical`, `axis-search`, `axis-ads-analytics`) and "SaaS platform → foundry" / "Foundry → foundry" rows — **retired "axis" + "Foundry" vocabulary** (flat-catalog/0058 + 0335). (b) References ADR-0042 (observability, Superseded→0383) and ADR-0050 (automation pipeline). (c) Reciprocal anti-scope with ADR-0036 (plugin APIs out of scope) — consistent. (d) `oya-shared-semver-check-cli` and `oya contract-diff` are concrete tool commitments — verify they exist/are planned. (e) No direct cross-side LINUX conflict, but if LINUX owns gRPC framing eventually (fault-line #5), the proto-contract pipeline assumptions shift.
- **hyperscaler_challenge:** Verdict **aligned**. preview/beta/GA tiers + semver + contract-first SDK generation + deprecation telemetry is the literal Google API lifecycle (google.aip.dev), AWS API stability tiers, and Azure preview/GA model. A per-PR breaking-change linter (cf. buf breaking, Google's API linter) is exactly what the majors run. Strong fit; argues KEEP + promote into masterplan.
- **ai_slop:** Low. Concrete, tool-backed, semver-grounded. Minor fabricated precision in SLA %s and lead-times, but those are policy choices, not invented facts.
- **refinement:** Replace "axis-*"/"Foundry" owner labels with flat-catalog microservice owners + intelligence/governance naming; repoint ADR-0042→0383; add `masterplan_ref` so this doctrine binds into the planning SSOT; confirm `oya contract-diff` tooling status.
- **consensus_needed:** no.

---

## Chunk notes for synthesis

**Cluster identity.** ADR-0030–0037 are the 2026-05-09 product/substrate-architecture expansion. Two of seven are Accepted (0030 Search, 0031 Ads, 0034 overrides), four are Proposed (0032 DCIM, 0035 Workflow, 0036 Plugins, 0037 API-tiers). None is superseded; none supersedes anything. The decisions themselves are largely TRUE and high quality — the audit findings are about *wrapping* (retired vocabulary, stale cross-refs, missing planning binding), not about wrong decisions.

**Pattern 1 — Retired-brand leakage is pervasive (the dominant amend driver).** Five of seven leak retired vocabulary:
- "Foundry" brand / `oya-foundry-*` crate prefix / owner `foundry`: ADR-0030 (RAG bridge + `workflow.search.rag`), ADR-0032 ("Foundry agents/runtime"), ADR-0035 (owner `foundry`), ADR-0036 (`oya-foundry-plugin-runtime-kernel`, owner `foundry`), ADR-0037 ("SaaS platform→foundry", "Foundry→foundry"). All must rename to **intelligence** (consumer AI) / **governance** (CI/gates) per ADR-0335/0347 + founder ruling ("cloud-intelligence is the valid name").
- "axis" vocabulary: ADR-0037 owner table (`axis-workspace`/`axis-vertical`/`axis-search`/`axis-ads-analytics`) — retired by flat-catalog (ADR-0058).
- "vertical"/`VerticalId` typing: ADR-0035 `WorkflowDefinition.vertical_id: VerticalId` — survives despite the vertical→microservice rename; ADR-0035 and ADR-0036 both cite "ADR-0034 (per-vertical override)" using ADR-0034's *old* title (now per-*microservice*).

**Pattern 2 — Stale superseded-ref drift.** Four ADRs cite ADRs that the keystone map flags as Superseded:
- **ADR-0042 (observability)** → Superseded by ADR-0383: cited in Related by ADR-0032, ADR-0035, ADR-0037 (and ADR-0036/0030 chains touch it indirectly).
- **ADR-0046 (vector store)** → Superseded by ADR-0192 (Milvus): ADR-0030 cites ADR-0046 for HNSW+IVF and its indexer crate names pgvector+HNSW; reconcile to Milvus-canonical.
- These follow keystone §6 guidance: trust the superseding ADR; treat these as mechanical ref-repoints, not decision changes.

**Pattern 3 — Missing masterplan binding (corpus-wide 8.8% problem, local instance).** All seven carry only `id/status/doc_status` front-matter (0037 additionally `sunset_topic`/`sunset_milestone`). None carries `masterplan_ref`/`planning_impact`/`supersedes`/`deliverables`. Under *either* open founder reading this is a gap: if masterplan-is-authority (drift-prevention design), these need `masterplan_ref` to bind in; if masterplan-is-generated (consolidation design), these need the structured `planning_impact`/`status`/`deliverables` front-matter to generate from. Several here are exactly the structural invariants the masterplan should carry: **singleton ads-gate (0031), non-overridable override-pack floor (0034), wasmtime capability-sandbox (0036), API stability doctrine (0037)** — recommend these four be promoted as named masterplan guardrail/governance nodes regardless of which authoring model wins.

**Pattern 4 — Own-vs-buy breadth (ties to keystone fault-line #5).** Three Proposed ADRs make day-0 OWN bets that a hyperscaler would stage:
- ADR-0032 in-house DCIM (before owning a DC) — the sharpest; recommend split anti-scope (keep) from build-now (defer to ADR-0028 Phase 2).
- ADR-0035 own-the-workflow-engine (reject Temporal) — recommend re-justify against the own-when-proven ratchet (ADR-0211/0173).
- ADR-0036 owns the plugin runtime but on *adopted* wasmtime — this is the correctly-balanced version (own the substrate composition, adopt the proven runtime) and is the model the others should follow.
This is consistent with the audit's framing that the disagreement across the corpus is the *own-when-proven trigger threshold*, not the principle.

**Pattern 5 — Possible false-contradiction trap for downstream synthesis: "Argo Workflows".** ADR-0035 Alternative B *rejects* Argo Workflows (as a business-process workflow engine), while CI canon ADR-0511 *adopts* Argo Workflows (as the k8s-native CI/CD orchestrator). These are different layers and both are correct — flagged here so the synthesis pass does not record a spurious contradiction. Recommend a one-line disambiguation note in ADR-0035.

**Cross-side (LINUX) note.** This chunk is all product/application-substrate (search, ads, DCIM, workflow, plugins, API governance); the LINUX pilot ADRs (0001–0026) are kernel/runtime/DB/policy substrate. No direct head-to-head collision in this slice, but two indirect couplings: (a) LINUX ADR-0001 "eliminate PostgreSQL / own multi-model DB" would change ADR-0030's indexer/vector and ADR-0031's analytics-warehouse substrate assumptions; (b) LINUX's eventual own-the-gRPC-framing ambition (fault-line #5) would touch ADR-0037's proto-contract/SDK pipeline. Surface, do not resolve.

**Consensus questions raised in this chunk (2):**
1. ADR-0032: build DCIM in-house day-0 vs adopt/adapt OSS DCIM until Oyatie owns physical DC capacity (ADR-0028 Phase 2) — and is the absolute no-custom-silicon anti-scope still right given every hyperscaler eventually built silicon at scale?
2. ADR-0035: own a bespoke hybrid FSM+DAG workflow engine day-0 vs adopt a durable-execution substrate (Temporal/Step-Functions-class) behind a port and own only the per-tenant-versioning + jurisdiction-overlay layer until ownership is proven necessary?
