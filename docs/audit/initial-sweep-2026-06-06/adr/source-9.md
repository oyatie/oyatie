# ADR Audit — source-9

- **side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`)
- **chunk:** source-9 (slice rows 57–63 of `ls ADR-*.md | sort`)
- **range:** ADR-0059 → ADR-0065
- **ADRs reviewed:** 7 (0059, 0060, 0061, 0062, 0063, 0064, 0065)
- **auditor posture:** READ-ONLY; keystone map is the shared baseline. All 7 are from the same 2026-05-13 `/deep-interview` session and are mutually cross-referencing; they form the "Bominal-inheritance + Workflow/Ontology + quality-bar + doc-coverage" foundation cluster. Heavy retired-vocab leakage throughout (Foundry, Kafka, Redis, M0-M3/Mxx milestone codes, "Arms", "tier"), plus a structural dependency on **Bominal** — a prior-product brand that the keystone map does not even track. These ADRs predate every retirement ADR (0329/0333/0335/0336/0347/0362/0377) and predate the Cedar/Talos/Pulsar/Valkey canon.

---

### ADR-0059 — Workflow + Ontology = ecosystem adapter layer

- **decision_atom:** All cross-microservice integration in the platform flows through exactly two canonical adapter primitives — **Workflow** (typed-event action/orchestration adapter) and **Ontology** (typed Object/Link/Action/Function information adapter, Palantir-Ontology-equivalent) — with direct cross-microservice imports forbidden and CI-enforced.
- **domain:** workflow-ontology (primary); api-contracts (secondary — it is the inter-service contract surface).
- **current_status:** Accepted.
- **disposition:** AMEND. The core architectural rule (Workflow+Ontology = the only cross-service integration plane; no direct cross-µsvc imports) is sound, durable, and load-bearing — it is the keystone-map "object-graph→ontology" canon (ADR-0055/0122/0130). But the body is stale: it mandates "outbox → **Kafka KRaft**" (retired → Pulsar+Oxia per ADR-0377), inherits a chain of **Bominal ADR-####** references that are an untracked foreign series, and pins p99 numbers to "Bominal ADR-0107" rather than the current ADR-0062 source.
- **proposed_resolution:** NA (Accepted, not Proposed).
- **governing:** N/A for the core rule; the Kafka clause is governed by **ADR-0377-kafka-to-pulsar** (Pulsar 4.x + Oxia, KoP wire-compat).
- **truth_flag:** PARTIAL — core decision TRUE; the eventing substrate (Kafka) and the Bominal-inheritance scaffolding are STALE.
- **in_masterplan:** YES — this is a top-tier architectural invariant (the "integration plane") and clearly carries planning impact; it should bind into the generated masterplan as the canonical cross-service contract.
- **tensions:** (1) Eventing-substrate conflict with ADR-0377 (Kafka retired). (2) Ontology adapter is "Postgres + RLS" — collides with LINUX ADR-0001 "eliminate PostgreSQL" framing (keystone §5 fault-line 1) and with SOURCE's own best-of-breed data posture (ADR-0179/0192). (3) Depends on "Bominal ADR-0103/0106/0107" — an external/legacy brand not in the supersession graph; provenance unverifiable from the corpus.
- **hyperscaler_challenge:** Google/AWS/Azure verdict = **aligned**. "Services integrate only via typed contracts + an event bus, never direct imports" is exactly AWS-internal service-decoupling doctrine and Palantir's actual Ontology model. The *specific* choice of a single shared "Workflow µservice" as the universal orchestration hot-path is more questionable (hyperscalers would federate orchestration, not centralize) — but this argues for refinement, not archive. Net: keep the principle, amend the substrate.
- **ai_slop:** Low. Dense but internally coherent; the crate-layout block is plausible. The "(= Palantir Ontology equivalent)" tic and the inherited-p99-numbers-without-measurement are the only soft spots.
- **refinement:** Replace Kafka clause with Pulsar+Oxia; re-anchor p99 targets to ADR-0062 directly (drop the Bominal-0107 inheritance); decide whether the Ontology data substrate is Postgres-only or best-of-breed (reconcile with ADR-0179/0192).
- **consensus_needed:** Founder question — "Is Workflow a single centralized orchestration µservice (current ADR) or a per-domain orchestration *pattern*? And is Ontology's storage canonically Postgres, or does it inherit the best-of-breed data tier?"

---

### ADR-0060 — Bominal-inheritance precedence (default inherit Bominal ADRs; session overrides)

- **decision_atom:** Oyatie adopts the prior **Bominal** product's architectural ADRs 1:1 (with a glossary-translation table) as the lower-precedence default, while any decision made in the 2026-05-13 session overrides Bominal where they conflict — a 10-item locked-override list defines the divergences.
- **domain:** governance-process (primary); docs-ssot-masterplan (secondary — it is a decision-provenance/precedence meta-rule).
- **current_status:** Accepted.
- **disposition:** ARCHIVE (as a live binding) / convert to historical record. This is a *bootstrapping* meta-ADR whose entire value was importing a foreign ADR series ("Bominal ADR-0011..0232") during genesis. Under the founder's "masterplan generated from the live ADR log; if it's not a LIVE ADR feeding the masterplan it's not needed" doctrine, an inheritance-pointer to an untracked external corpus is precisely the kind of decision that should NOT survive: the inherited decisions must be re-authored as first-class oyatie ADRs (the keystone-map "re-found from ADR-0000 with `consolidates:` provenance" path), not carried by reference. The 10 overrides are already individually captured in their own ADRs (0055/0056/0058/0059/0061).
- **proposed_resolution:** NA (Accepted).
- **governing:** No single superseding ADR yet; governed in spirit by the planning-ssot-consolidation re-founding design (keystone §4) and by the retired-vocab rules that already kill most of its glossary table ("Arms", "platform", "Object Graph").
- **truth_flag:** STALE — the override table references retired concepts as if live ("Kafka KRaft" via inherited ADR-0116; "Redis" not yet Valkey; "M03" milestone codes; "Foundry"/"axis-foundry" implied downstream). The *mechanism* (inherit-then-override) was true at genesis; as a standing binding it is stale and structurally undesirable.
- **in_masterplan:** NO — a "go read another product's ADRs" pointer cannot bind into a self-contained generated masterplan. Its *outputs* (the 10 overrides) belong; the pointer itself does not.
- **tensions:** (1) Directly contradicts the ADR-immutability/self-contained-SSOT doctrine (keystone §4): you cannot generate a masterplan from a log that defers half its decisions to an external series. (2) Glossary table already partially retired by ADR-0058 ("Arms" gone), ADR-0055 ("Object Graph" gone), and the "platform→shared" rename. (3) Inherits "Bominal ADR-0116 Kafka" → conflicts with ADR-0377.
- **hyperscaler_challenge:** Verdict = **questionable→misaligned**. No hyperscaler maintains a permanent "inherit-by-reference from a sibling product's decision log" rule — they fork-and-own or consolidate. This argues for **archive/absorb**: dissolve the inheritance into concrete owned ADRs.
- **ai_slop:** Low-to-moderate. The override table is genuinely useful provenance, but the "Bominal ADR-0011..0232" inheritance block lists ~40 inherited decisions by foreign number with no in-corpus target — that is unverifiable scaffolding that reads as slop to anyone without the Bominal repo.
- **refinement:** Re-author the still-true inherited decisions as native oyatie ADRs (with `consolidates: [Bominal-ADR-####]` provenance), then archive ADR-0060 to a "genesis/historical" frozen series.
- **consensus_needed:** Founder question — "Is 'Bominal' a real sibling corpus we must keep importing from, or a genesis artifact we should fully absorb and stop referencing? The generated-masterplan goal effectively requires the latter."

---

### ADR-0061 — Application: B2B unified shell with à-la-carte product enablement

- **decision_atom:** The B2B entry point is a single **Application** shell µservice where tenants sign in once and enable catalog products à-la-carte (AWS-Console-style), rendering product surfaces dynamically via the capability registry with no hardcoded product list, while B2C ("Personal") uses a separate person-pillar entry path.
- **domain:** product-ux (primary); identity-authn (secondary — owns the two-cookie/PKCE B2B auth shell + per-enablement Cedar gating).
- **current_status:** Accepted.
- **disposition:** AMEND. The decision (unified B2B shell + à-la-carte enablement + capability-registry-driven dynamic surfaces) is sound and hyperscaler-aligned. Stale bits: session state declared on "**Redis** (session cache)" (retired → Valkey, ADR-0336), and it leans on "Bominal ADR-0121/0123" inheritance and "cell" provisioning (cell is now a *pattern* not a service per ADR-0333 — usage here is as a deployment unit, which survives, but the wording should be checked).
- **proposed_resolution:** NA (Accepted).
- **governing:** Redis clause governed by **ADR-0336** (Valkey); "cell" semantics by **ADR-0333** (pattern-not-service).
- **truth_flag:** PARTIAL — core TRUE; "Redis session cache" WRONG (retired substrate); Bominal-inheritance scaffolding STALE.
- **in_masterplan:** YES — the B2B Application shell is a real product-surface decision with planning impact (it is even named in ADR-0065 as the host of the first product, the Docs Portal).
- **tensions:** (1) Redis vs Valkey (ADR-0336). (2) "Cedar policy gate (ADR-0007)" for product access — aligned with SOURCE Cedar canon (ADR-0243/0246), so this is a *positive* coherence, but note LINUX ADR-0021 owned-policy tension (keystone §5 fault-line 2). (3) References its own ADR-0035 (Workflow) for provisioning — fine, but couples shell to the centralized-Workflow question raised in 0059.
- **hyperscaler_challenge:** Verdict = **aligned**. "One console, enable services à-la-carte, surfaces discovered dynamically, no hardcoded product list" is literally the AWS/GCP/Azure console model. Google/AWS would absolutely make this decision. No archive argument; minor amend only (substrate naming).
- **ai_slop:** Low. The Rust port-trait sketch and crate layout are concrete and plausible; perf targets are specific. Good ADR.
- **refinement:** s/Redis/Valkey/; re-anchor inherited auth contract (two-cookie+PKCE+nonce) as a native identity ADR rather than "Bominal ADR-0123 (inherited)".
- **consensus_needed:** None of substance; this is one of the cleaner ADRs in the chunk.

---

### ADR-0062 — Quality/Performance/Scalability bar (industry-leaders + hyperscaler scale, day-one)

- **decision_atom:** Every µservice must meet a mandatory, CI-enforced bar — competitive-benchmarked against the named industry leader for its domain, hitting hyperscaler-grade p99/throughput/failover targets, and horizontally scalable from day one — gated by a 14-lane fitness matrix (4 new check crates: statelessness, shardability, perf-budget, benchmark).
- **domain:** ci-cd-build (primary — it is fundamentally a fitness-lane/gate regime); observability (secondary — SLOs, burn-rate alarms, perf budgets).
- **current_status:** Accepted.
- **disposition:** AMEND. The bar itself (benchmark-against-leaders, hyperscaler perf targets, mandatory horizontal scalability, CI-enforced) is exactly the kind of durable invariant the masterplan should carry. But it is laced with retired substrates and milestone codes: "**Confluent Kafka (KRaft)**" benchmark + "Outbox → **Kafka KRaft**" (→ Pulsar/ADR-0377), "**Valkey/Redis** cluster" (half-retired wording), "**Palantir Foundry-grade** observability" (Foundry brand retired per ADR-0335 — though here it means the *Palantir product*, not oyatie's Foundry, so it's a naming-collision landmine), "**Foundry (internal engine)**" + "`oya-foundry-*`" (retired → intelligence/governance, ADR-0335/0347), and "M02/M03" milestone codes (retired → Wave names per GLOSSARY). The lane list also predates the observability move to Loki/Tempo/Mimir (ADR-0383) — it cites "VictoriaMetrics" and "Prometheus gauge".
- **proposed_resolution:** NA (Accepted).
- **governing:** Kafka clause → **ADR-0377**; `oya-foundry-*`/"Foundry internal engine" → **ADR-0335/0347** (intelligence/governance); milestone codes → GLOSSARY Wave-name retirement; Redis → **ADR-0336**.
- **truth_flag:** PARTIAL — the bar (principle) is TRUE and strong; the concrete substrate/benchmark/milestone references are STALE.
- **in_masterplan:** YES — this is a cross-cutting quality contract that every µservice ADR inherits; high planning impact.
- **tensions:** (1) "No exceptions for internal µservices — Foundry must be scalable" hardcodes the retired Foundry brand and `oya-foundry-*` prefix (MFL brand-residue). (2) Kafka/Redis substrate drift. (3) Observability stack named here (VictoriaMetrics/Prometheus) predates ADR-0383's Loki/Tempo/Mimir/Grafana — a quiet conflict. (4) "Citus" for Postgres sharding vs keystone canon "Postgres+pgcat" (ADR-0179) — minor substrate drift.
- **hyperscaler_challenge:** Verdict = **aligned** on intent, **questionable** on rigidity. Hyperscalers absolutely benchmark against leaders and enforce scalability gates — but "horizontally scalable from day one, no single-instance designs, feature-complete-or-not-shipped" is *more* absolutist than Google/AWS, who explicitly allow staged/regional single-cell launches. The bar is aspirational-correct but the "day-one 100M users, no prototype releases" framing would be softened by any real hyperscaler to a maturity ladder. Argues for amend (keep the gates, soften the absolutism, refresh substrates), not archive.
- **ai_slop:** Moderate. The 14-lane matrix and 4 check-crate definitions are concrete; but the inherited-perf-numbers (all sourced to "Bominal ADR-####" with no measurement) and the "feature-complete or not shipped" maximalism read as aspirational slogan more than measured target.
- **refinement:** Swap Kafka→Pulsar, Redis→Valkey, VictoriaMetrics/Prometheus→Loki/Tempo/Mimir; strip `oya-foundry-*` and the "Foundry internal engine" exemption (rename to intelligence/governance); replace Mxx milestone codes with Wave names; re-derive p99 targets from a real benchmark rather than inheritance.
- **consensus_needed:** Founder question — "Is the bar a hard day-one gate (current ADR) or a Proof-Ladder maturity ramp? And does 'industry leader' benchmarking remain mandatory-per-µservice now that the catalog is large?"

---

### ADR-0063 — Documentation set coverage (every feature ships a complete doc set, CI-enforced)

- **decision_atom:** Every registered µservice must ship a complete, CI-enforced documentation set (microservice record + PRD + naming ADR + BC registrations + phase-specs + impl-plans, plus per-localization-pack overlays) in the same commit that introduces it — no stubs, no deferrals — gated by the `oya-check-documentation` (LEAN-A5) lane that report-only-then-blocks.
- **domain:** docs-ssot-masterplan (primary); ci-cd-build (secondary — it is a fitness lane).
- **current_status:** Accepted.
- **disposition:** AMEND. The "complete doc-set or not Complete; CI-enforced; stale removed not flagged" contract is strong and directly serves the founder's SSOT goal. Stale bits: required impl-plan sections include "**## Grit Claim Symbols**" and "**## ICM Rows to Emit**" — grit/icm are *retired external coordination tooling* (ADR-0116 supersedes ADR-0054), and `icm recall` appears in the re-verification recipe; "**axis-foundry**" owner (retired → governance/intelligence); milestone codes "M02-P22" etc. (retired → Waves); and it presumes the markdown-is-source model that ADR-0065 (next ADR, same session) immediately revises toward Leptos+generated.
- **proposed_resolution:** NA (Accepted).
- **governing:** grit/icm references → **ADR-0116** (retire external agent-coordination tooling, sup. ADR-0054); "axis-foundry" → **ADR-0335/0347**; milestone codes → GLOSSARY Waves; doc-pipeline superset → **ADR-0065** (this ADR's §"Enforcement" is extended by 0065).
- **truth_flag:** PARTIAL — the coverage contract is TRUE and valuable; the toolchain bindings (grit/icm/axis-foundry/Mxx) are STALE/retired-vocab.
- **in_masterplan:** YES — doc-coverage enforcement is a planning invariant (and is the mechanism by which the masterplan stays complete). Strong masterplan binding.
- **tensions:** (1) grit/icm retired (ADR-0116) but mandated as required impl-plan sections — a live ADR demanding retired-tool artifacts is a genuine drift. (2) Self-referential coupling with ADR-0065 (which supersets the same `oya-check-documentation` lane to add schema+generated-output validation) — 0063 and 0065 should be reconciled/merged. (3) "axis-foundry" lane-owner = retired brand.
- **hyperscaler_challenge:** Verdict = **aligned**. "Docs/tests/code land together or the change isn't complete; coverage is CI-gated; stale docs are deleted not flagged" is mainstream Google/Meta monorepo discipline (readability + required docs gates). Strongly aligned; amend the retired-tool references only.
- **ai_slop:** Low-to-moderate. The pre-mortem (3 scenarios) and test-plan tiers are genuinely substantive, not filler. The "Grit Claim Symbols / ICM Rows" required sections and the `icm recall` recipe line are the clearest retired-tooling slop.
- **refinement:** Remove grit/icm required sections (or replace with the in-repo intelligence/oya-ci equivalents per ADR-0116); retire "axis-foundry" owner → governance; replace Mxx with Waves; explicitly fold/reconcile with ADR-0065 so there is one doc-coverage contract, not two.
- **consensus_needed:** Founder question — "Should ADR-0063 and ADR-0065 be MERGED into a single doc-coverage+generation ADR? They share one lane (`oya-check-documentation`) and 0065 silently supersets 0063."

---

### ADR-0064 — Canonical base + localization packs (pack-pluggable µservices; Korea is pack #1)

- **decision_atom:** Every customer-facing µservice has a jurisdiction-agnostic **canonical base** plus zero-or-more **localization packs** (composed of seams + adapters + Cedar fragments + Workflow/Typst templates), where the canonical base is mechanically forbidden from containing any jurisdiction-specific identifier, a pack (or explicit pack-neutral ADR) is mandatory before shipping to a paying tenant, and **Korea is the foundational pack #1**.
- **domain:** compliance-residency (primary); product-ux (secondary — the pack/seam/adapter architecture shapes how localized product surfaces compose).
- **current_status:** Accepted (carries a 2026-06-02 platform-readiness amendment note re: `{oya,cloud}` pure-split — so it has already been touched post-genesis and affirmed canonical).
- **disposition:** KEEP (with light AMEND). This is the strongest, most durable ADR in the chunk: the canonical-base-neutrality + pack-pluggability rule is a genuine, mechanically-enforceable architecture invariant with a thorough Alternatives-considered section, and it was *re-affirmed* by the 2026-06-02 amendment. Only light staleness: "Bominal ADR-0140 (retired per ADR-0145)" inheritance reference (note the irony — it cites a SOURCE ADR-0145 retirement against a *Bominal* 0140, conflating the two series), and "M01–M07" milestone codes (→ Waves).
- **proposed_resolution:** NA (Accepted).
- **governing:** N/A (kept). Milestone codes → GLOSSARY Waves; the Bominal-0140 inheritance pointer → should be re-authored native per the ADR-0060 disposition above.
- **truth_flag:** TRUE — the decision is current, correct, non-conflicting, and recently re-affirmed. Only the milestone-code wording is stale.
- **in_masterplan:** YES — pack-pluggability + canonical-base-neutrality is a top-tier architectural invariant and a compliance/residency cornerstone; high masterplan binding.
- **tensions:** (1) Minor: the "Bominal ADR-0140 / SOURCE ADR-0145" cross-reference conflates two ADR series (same drift class as ADR-0060). (2) Depends on ADR-0059 Workflow+Ontology for cross-pack integration — inherits 0059's centralized-Workflow question. (3) Cedar policy fragments per pack — *positively* aligned with SOURCE Cedar canon (ADR-0243/0246/0379). No hard conflicts.
- **hyperscaler_challenge:** Verdict = **aligned**. "Universal core + per-region overlay packs, region code never leaks into core, compile-time-enforced" mirrors how AWS/GCP/Azure structure regionalization and how Workday/SAP structure localization. Google/AWS would make this decision. Keep.
- **ai_slop:** Very low. This is a high-quality ADR — the seam/adapter/pack trichotomy, the §1.5 semantic-localization decision table (K-GAAP/FHIR/PIPA), and the 8-row Alternatives-considered table are substantive and would survive a hyperscaler design review.
- **refinement:** Replace M01–M07 with Wave names; re-author the Bominal-0140 inheritance as a native ADR; otherwise leave intact.
- **consensus_needed:** None of substance — this ADR is close to masterplan-ready as-is.

---

### ADR-0065 — Documentation as Leptos web pages with machine-readable JSON/YAML/TOML co-emission

- **decision_atom:** All documentation shifts to a triple-output pipeline — markdown stays the authoring source-of-truth, while a deterministic Rust generator co-emits Leptos web pages (for humans) and machine-readable JSON/YAML/TOML manifests (for agents/fitness-lanes), delivered by a new `docs` µservice in the flat catalog, with CI enforcing that generated output is regenerated-deterministic.
- **domain:** docs-ssot-masterplan (primary); product-ux (secondary — the Leptos portal is a real rendered product surface, "the first product alongside Workflow Studio").
- **current_status:** Accepted.
- **disposition:** AMEND. The triple-output model (markdown source → Leptos pages + structured manifests; deterministic, CI-checked, no hand-editing of generated output) is genuinely good and directly serves the machine-readable-masterplan goal — it is essentially the mechanism that would *feed* a generated masterplan. Stale bits: "Foundry primitives" / "`oya-foundry-*` fitness lanes" / "axis-foundry" owner (retired → intelligence/governance, ADR-0335/0347); Mxx milestone codes (→ Waves); and the frontmatter `cluster:` enum still lists "foundry" as a valid cluster value (retired).
- **proposed_resolution:** NA (Accepted).
- **governing:** Foundry references → **ADR-0335/0347**; milestone codes → GLOSSARY Waves; shares the `oya-check-documentation` lane with **ADR-0063** (reconcile/merge candidate).
- **truth_flag:** PARTIAL — the pipeline decision is TRUE and forward-looking; the Foundry/axis-foundry/Mxx bindings are STALE.
- **in_masterplan:** YES — this is the doc-generation substrate; under the "masterplan generated from the ADR log" reading (keystone §4), ADR-0065's generator is plausibly the *literal machinery* of generation. High planning relevance.
- **tensions:** (1) Supersets ADR-0063's doc-coverage lane in the same session without superseding it — 0063↔0065 reconciliation needed (MERGE candidate). (2) Hardcodes retired Foundry brand in the cluster enum and lane references. (3) Inherits "Bominal ADR-0208/0209" (dual-context + Leptos client stack) — same external-series provenance issue as ADR-0060. (4) Frontmatter `status` enum here ("Proposed|Accepted|Rejected|Superseded|Retired") is the canonical one to standardize on — actually a *positive*, worth promoting platform-wide.
- **hyperscaler_challenge:** Verdict = **aligned**, with a build-vs-buy footnote. "Docs-as-code with a deterministic generator emitting both human pages and a machine-readable API" is exactly Google's internal docs model and the "docs as data" pattern. The *questionable* part is building a bespoke Rust/Leptos doc generator + portal µservice rather than adopting an existing docs-as-code toolchain (the ADR rejects Hugo/Docusaurus/mdbook explicitly on "must be Rust-native + reuse Workflow-Studio Leptos" grounds) — a hyperscaler would scrutinize that own-vs-reuse call (it echoes the keystone §5 fault-line 5 "own everything" breadth concern). Aligned on intent; the bespoke-generator scope is the amend/challenge axis.
- **ai_slop:** Low. The manifest JSON schema, the per-crate BNF layer table, and the 5-phase migration are concrete. The "Foundry primitives" references and the somewhat-grand "all docs are now web pages" framing are the soft spots.
- **refinement:** Strip Foundry from the cluster enum + lane references (→ intelligence/governance); replace Mxx with Waves; MERGE or explicitly subordinate ADR-0063 under one doc pipeline; revisit own-vs-reuse for the generator (or document why Rust/Leptos-native is non-negotiable).
- **consensus_needed:** Founder question — "Bespoke Rust/Leptos doc generator + `docs` portal µservice, or adopt a proven docs-as-code toolchain with a JSON sidecar? This is a concrete instance of the broader 'own everything' breadth question." (Plus the 0063/0065 MERGE question.)

---

## Chunk notes

**Cluster identity.** All 7 ADRs are a single 2026-05-13 `/deep-interview` foundation batch, mutually cross-referencing, all `status: accepted, doc_status: published`. They are the genesis "how the platform is shaped" layer: integration plane (0059), decision-inheritance (0060), B2B shell (0061), quality bar (0062), doc coverage (0063), localization architecture (0064), doc pipeline (0065). No Proposed ADRs in this slice → no RATIFY/DROP decisions required.

**Pervasive cross-cutting drift (applies to nearly every ADR here):**
- **Kafka → Pulsar+Oxia** (ADR-0377): 0059 and 0062 still mandate "Kafka KRaft" / "Confluent Kafka". STALE.
- **Redis → Valkey** (ADR-0336): 0061 ("Redis session cache") and 0062 ("Valkey/Redis cluster") have residue. STALE.
- **Foundry → intelligence/governance** (ADR-0335/0347): 0062 ("Foundry internal engine", `oya-foundry-*`, "no exceptions for internal µservices"), 0063 ("axis-foundry" owner), 0065 ("Foundry primitives", `oya-foundry-*` lanes, "foundry" in the cluster enum). This is the single most widespread retired-vocab leak in the chunk. RETIRED-VOCAB.
- **grit / icm** (ADR-0116, sup. 0054): 0063 mandates "## Grit Claim Symbols" and "## ICM Rows" as required impl-plan sections and uses `icm recall` in its re-verify recipe. A *live Accepted ADR requiring artifacts from retired tooling* — the sharpest concrete drift in the chunk.
- **Mxx milestone codes (M01..M07, M02-P22, etc.) → Wave names** (GLOSSARY, MFL-0003): every ADR except arguably 0060 carries these. STALE wording, low-severity.
- **Bominal external-series dependency:** 0059/0060/0061/0062/0063/0064/0065 all cite "Bominal ADR-####" as inherited authority. The keystone map does not track a "Bominal" corpus at all — these are pointers into an external/legacy product decision log. Under the generated-masterplan / self-contained-SSOT goal, this is structurally undesirable; the inherited decisions should be re-authored native (the keystone §4 "re-found with `consolidates:` provenance" path). ADR-0060 is the root of this pattern and is the natural ARCHIVE/absorb target.

**Disposition summary:**
- KEEP: **0064** (re-affirmed 2026-06-02; light wording amend only).
- AMEND (sound core, refresh retired substrates/refs): **0059, 0061, 0062, 0063, 0065**.
- ARCHIVE/absorb (genesis inheritance pointer, contrary to self-contained-SSOT): **0060**.
- MERGE candidate flagged: **0063 ⊕ 0065** (one doc-coverage+generation contract; they share the `oya-check-documentation` lane and 0065 silently supersets 0063).

**Truth-flag summary:** 0064 = TRUE; 0059/0061/0062/0063/0065 = PARTIAL (true core, stale substrate); 0060 = STALE. No GARBAGE, no WRONG-in-whole. Quality is generally high — these are dense, internally-coherent, low-slop ADRs; the problem is *time*, not *correctness*: they predate every retirement ADR (0116/0329/0333/0335/0336/0347/0362/0377) and the Cedar/Talos/Pulsar/Valkey/Loki canon.

**Hyperscaler verdict (chunk-level):** The *principles* are strongly hyperscaler-aligned — typed-contract-only service integration (0059), unified console with à-la-carte enablement (0061), benchmark-against-leaders + scalability gates (0062), docs-and-code-land-together (0063), universal-core + region-packs (0064), docs-as-data (0065). The two questionable-on-rigidity items are 0062's "day-one 100M-user / no-prototype" absolutism (hyperscalers stage launches) and 0065/0060's "own everything" instinct (bespoke doc generator; inherit-by-reference) — both argue for amend/refinement, not archive.

**Three founder consensus questions surfaced (ranked):**
1. **(0060)** "Is 'Bominal' a sibling corpus we keep importing from, or a genesis artifact to fully absorb into native ADRs?" — The generated-masterplan goal effectively requires absorb. Highest-leverage question because it determines whether ~40 inherited decisions get re-authored as first-class ADRs.
2. **(0059)** "Is Workflow a single centralized orchestration µservice or a per-domain pattern; and is Ontology's store canonically Postgres or best-of-breed?" — Touches the keystone §5 LINUX-vs-SOURCE data-tier fault-line.
3. **(0063⊕0065)** "Merge the two doc ADRs into one coverage+generation contract?" — They share a lane and a brand of staleness; one ADR is cleaner for the masterplan.
