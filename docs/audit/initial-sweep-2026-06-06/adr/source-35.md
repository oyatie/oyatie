# ADR Audit — source-35

- **side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **chunk:** source-35 (coverage-backfill slice rows 239–245 of `ls -1 docs/decisions/ADR-*.md | sort`)
- **range:** ADR-0302 → ADR-0308 (contiguous)
- **ADRs reviewed:** 7 (ADR-0302, ADR-0303, ADR-0304, ADR-0305, ADR-0306, ADR-0307, ADR-0308)
- **auditor posture:** READ-ONLY; only this artifact written. Trust superseding ADR over stale front-matter (keystone map §6). All masterplan flags surfaced under BOTH readings (authored-vs-generated OPEN per keystone §4).

> **Cluster context.** All seven belong to the single `keystone_bundle: 2026-05-20-foundational-doctrine`. ADR-0302–0306 close "critical-path edge-case" rows from `documentation-rigor.md §3.2.5` (deceased-user, cognitive-impairment, cross-jurisdiction, delegated-agent, disaster-mode). ADR-0307–0308 open the **DRMP** ("D"=Detection, then the ML lifecycle) substrate from `§3.2.6`. Every one is `status: Proposed`, every one is a "substrate-primitive not per-µservice afterthought" doctrine, every one defines a shared crate + per-µservice Cedar fragment + CI fitness lanes + ADR-0263 audit-event-classes, and every one has an `enforcement_status: advisory-until-<date>-blocker-thereafter` clock. They are internally coherent, deeply cross-referenced, and individually plausible — but they are also a **34-author-style mass-synthesis batch** with shared retired-vocab leakage (see Chunk notes). None conflicts with the LINUX pilot (no overlap of subject); the tensions are SOURCE-internal (retired vocab) and hyperscaler-scope (breadth/over-engineering).

---

### ADR-0302 — Deceased-User Inheritance Doctrine

- **decision_atom:** Make deceased-user inheritance a substrate primitive — a shared `oya-shared-deceased-user-inheritance` crate exposing Apple-class Legacy-Contact pre-designation + Google-class Inactive-Account-Manager + Microsoft-class legal-rep court-order ingress + per-jurisdiction inheritance overlays (RUFADAA/EU/KR/JP/UK/AU), where the deceased's pre-mortem wish wins within statutory bound, the DSAR-cascade (ADR-0276) executes it, and a Cedar FORBID stops any tenant from unilaterally locking out heirs.
- **domain:** identity-authn (cross-cut: compliance-residency)
- **current_status:** Proposed
- **disposition:** AMEND (then RATIFY-as-amended)
- **proposed_resolution:** RATIFY — the decision is sound, real (every mature platform ships this), and non-conflicting; but ratify only AFTER the AMEND fixes (retired `oya-foundry-fitness-*`/`oya.foundry-fitness.*` lane names → `oya-governance-*` per ADR-0347, which the doc's own justification text already half-acknowledges by naming the lanes `oya-governance-…` while the `bnf_segments` still say `oya.foundry-fitness.…`).
- **governing:** n/a (not superseded)
- **truth_flag:** PARTIAL — TRUE decision, STALE vocabulary (foundry-fitness BNF segments) + internal naming inconsistency (lane `name:` vs `bnf_segments:` disagree).
- **in_masterplan:** PARTIAL — carries `enforced_by` gates + a 2026-08-15 blocker clock + naming_justifications, so under "generated-from-ADRs" it injects 7 CI lanes + 5 lifecycle-state enum values + 2 principal roles + audit classes into the plan; under "masterplan-is-authority" it would need a `masterplan_ref` binding it currently lacks (keystone §4 8.8%-bound problem).
- **tensions:** Heavy fan-out into ADR-0276 (DSAR), ADR-0299 (account-recovery), ADR-0300 (anonymity), ADR-0301 (shelter-mode), ADR-0247 (break-glass quorum) — consistent, not conflicting. Forced-heirship vs pre-mortem-delete is internally resolved in the Cedar fragment. No cross-ADR conflict.
- **hyperscaler_challenge:** ALIGNED. Google/Apple/Microsoft/Meta/Coinbase all ship exactly this (Legacy Contact, IAM, Next-of-Kin court order, memorialization). A hyperscaler WOULD make this decision. Does NOT argue for archive; argues only for the vocabulary amend.
- **ai_slop / refinement / consensus_needed:** Mild slop signal — the front-matter is enormous (≈300 lines of naming_justifications, 5 ADRs deep of intersection prose) for one doctrine; that is documentation-rigor maximalism, not falsehood. **Founder question:** is a 7-CI-lane, advisory→blocker enforcement apparatus for deceased-user inheritance proportionate at pilot stage, or should this be ONE lane (`oya-governance-deceased-user-inheritance`) with the children folded in until there is a product surface to enforce against?

---

### ADR-0303 — Cognitive-Impairment & Decision-Resilience Doctrine

- **decision_atom:** Make decision-resilience a substrate primitive — a shared `oya-shared-decision-resilience` crate providing four orthogonal, **informational-only / never-blocking, opt-in** primitives (cooling-off windows, FINRA-Rule-4512 trusted-contact alerts, 3σ rapid-mutation cool-down, per-jurisdiction guardianship-overlay) on every consequential-mutation path, composed via Cedar with regulator floors (e.g. FINRA 2165) un-overridable.
- **domain:** authz-policy (cross-cut: product-ux)
- **current_status:** Proposed
- **disposition:** AMEND (then RATIFY-as-amended)
- **proposed_resolution:** RATIFY — sound and well-bounded by two strong invariants (autonomy/never-block + non-discrimination/opt-in-never-inferred), which are the right guardrails. AMEND for the same retired `oya.foundry-fitness.*` BNF leakage + a stale internal ref ("ADR-0303 + ADR-0292 + §3.2.5 row 12" WCAG citation reads as self-referential placeholder).
- **governing:** n/a
- **truth_flag:** TRUE (decision) / STALE (foundry-fitness BNF segments).
- **in_masterplan:** PARTIAL — 6 `enforced_by` gates + 2026-09-15 blocker clock; same authored-vs-generated duality as 0302.
- **tensions:** §C.1 maintainability prose still names a "**foundry**" µservice in its list of ≥30 consuming services ("…compliance, foundry, mail, notes…") — that is RETIRED brand (ADR-0335; foundry→intelligence/governance). Composes cleanly with ADR-0297/0298/0299/0304; no decision conflict.
- **hyperscaler_challenge:** ALIGNED. Stripe Radar cool-down, Apple Cash 72h reversal, Schwab/Fidelity FINRA 4512/2165, Chase/BofA cooling-off, Google Family Link — all real substrate patterns. A hyperscaler WOULD ship this. The autonomy/opt-in invariants are exactly the ones that keep it from becoming an age-discrimination surface. Argues for amend, not archive.
- **ai_slop / refinement / consensus_needed:** Citation density is the slop tell (every paragraph carries 3–6 statute/vendor cites). Decision survives it. **Founder question:** the `BEREAVEMENT_PROTECTED`/`COGNITIVE_PROTECTED` audience-types overlap with ADR-0302 deceased-user and ADR-0301 survivor-safety — is "decision-resilience" genuinely a separate crate, or one facet of a single "user-protection substrate"? (Consolidation candidate, not a conflict.)

---

### ADR-0304 — Cross-Jurisdiction Conflict Resolution Doctrine

- **decision_atom:** Make cross-jurisdiction conflict resolution a substrate primitive — a shared `oya-shared-jurisdiction-conflict` crate enforcing per-pack data-residency hard-stops, a total-ordered **higher-restriction-pack-wins** precedence-DAG (FORBID > cooling-off > guardian-co-sign > audit > transparency-report > PERMIT), multi-pack alignment for cross-border transfers, MLAT routing instead of CLOUD-Act compliance, and a mandatory per-request transparency report.
- **domain:** compliance-residency (cross-cut: authz-policy)
- **current_status:** Proposed
- **disposition:** AMEND (then RATIFY-as-amended)
- **proposed_resolution:** RATIFY — this is the keystone that ADR-0303/0302/0306 all defer their multi-pack conflicts to ("higher-restriction wins per ADR-0304"), so it is load-bearing and TRUE. AMEND: retired `oya.foundry-fitness.*` BNF; and the precedence-DAG should reconcile with the source canonical posture for data tiers (it asserts cell-pinning per ADR-0240/0248 — consistent — but should cite the live Milvus/Postgres/best-of-breed substrate, not invent a separate residency engine).
- **governing:** n/a
- **truth_flag:** TRUE / STALE-vocab.
- **in_masterplan:** PARTIAL — 5 gates + 2026-10-15 blocker; it is the dependency root for the cluster's conflict semantics, so under generated-from-ADRs it MUST bind before its dependents can.
- **tensions:** Strong DEPENDENCY-IN edges (0302/0303/0305/0306 all cite 0304's higher-restriction-wins as their conflict resolver). That makes 0304 the cluster's keystone — ratify it first or the others dangle. The GDPR-Art-48-vs-CLOUD-Act "refuse at substrate + route to MLAT" stance is a genuine legal posture, not slop, but it is a **founder-level legal commitment** (oyatie is US-incorporated per the doc's own §B.2.a) — surface, do not auto-accept.
- **hyperscaler_challenge:** ALIGNED-with-caveat. Azure Sovereign / AWS GovCloud / Google Assured Workloads / Cloudflare Data Localization all ship per-jurisdiction hard-stops + transparency reports. BUT no hyperscaler unilaterally "refuses a valid CLOUD-Act subpoena at the substrate" — they litigate (Microsoft v. Ireland) or comply-and-notify; encoding refusal as a substrate default is MORE aggressive than hyperscaler practice and is the one decision a Google/AWS legal team would call **questionable**. Argues for AMEND (soften "refuse" → "route to legal-council/MLAT + warrant-canary", which the doc actually already does in steps 4–6 — the §B headline overstates).
- **ai_slop / refinement / consensus_needed:** Worked examples (GDPR×CLOUD, PIPA×subpoena, PIPL×GDPR) are genuinely useful, low slop. **Founder question (CONTESTED):** does oyatie accept the legal exposure of a substrate-level "refuse foreign court order" default, or is the substrate's job to *surface + route*, leaving compliance to humans? This is a legal-strategy decision, not an engineering one.

---

### ADR-0305 — Delegated-Agent Authority Chain Doctrine

- **decision_atom:** Make delegated-agent authority a substrate primitive — a shared `oya-shared-agent-authority` crate issuing per-tenant JWS/Ed25519 `delegated_agent_token`s with a cryptographic attestation chain back to the authorizing human's passkey (ADR-0188), subset-only scope inheritance (no escalation), hard cross-tenant blocking, ≤2s revocation propagation, and a bot-defence (ADR-0297) attestation-aware allow-path so LLM agents / webhooks / workflow-steps aren't false-positive-blocked.
- **domain:** agentic-platform (cross-cut: identity-authn)
- **current_status:** Proposed
- **disposition:** AMEND (then RATIFY-as-amended)
- **proposed_resolution:** RATIFY — this is arguably the single most strategically-relevant ADR in the chunk (it is the authz model for AI agents acting on user behalf; directly on the cloud-native-Rust-stack roadmap's agentic surface). The token shape, attestation chain, and no-escalation invariant are correct and hyperscaler-standard. AMEND: retired `oya.foundry-fitness.*` BNF AND a retired **brand** leak — §A.2 names `oyatie.foundry.*` principal namespace and "Foundry agents (the self-modification surface)"; foundry is RETIRED (ADR-0335/0347 → intelligence/governance). Rewrite to `oyatie.intelligence.*`.
- **governing:** n/a
- **truth_flag:** PARTIAL — TRUE decision, but WRONG/STALE on the foundry namespace (`oyatie.foundry.*`) and `oya.foundry-fitness.*` lanes; both are retired vocab the doc treats as live.
- **in_masterplan:** PARTIAL — 6 gates + 2026-09-30 blocker. High planning_impact: adds `DELEGATED_AGENT` principal_type, a whole token/attestation spec, bot-defence integration.
- **tensions:** Cites ADR-0255 (intelligence two-layer) AND ADR-0247 (self-modification) AND the retired foundry namespace in the same breath — the foundry→intelligence rename (ADR-0335) was not propagated here. SPIFFE-workload-identity (ADR-0295) is correctly kept distinct from human-delegation. No decision conflict, but the brand drift is a real correctness bug.
- **hyperscaler_challenge:** ALIGNED. Microsoft Graph `act_on_behalf_of`, Anthropic MCP/workspace-scoped keys, OpenAI Assistants, Google Workspace OAuth scopes, GitHub App installation tokens — this IS the canonical pattern. A hyperscaler WOULD make this decision essentially as written. Argues for amend (brand), not archive.
- **ai_slop / refinement / consensus_needed:** Four near-identical worked examples (Claude/IFTTT/n8n/Zapier) are padding but harmless. **Founder question:** given the founder's own agentic stack, should ADR-0305 be promoted from "critical-path edge case" to a **first-class core agentic-platform ADR** (and renumbered/re-homed out of the deceased-user/disaster-mode bundle)? It is more central than its bundle placement implies.

---

### ADR-0306 — Disaster-Mode + Cell-Resilience Doctrine

- **decision_atom:** Make disaster-mode a substrate primitive — a shared `oya-shared-disaster-mode` crate providing 10×-surge load-shed tiers (tier-0…tier-4), CRDT offline-first sync (`oya-collab-crdt-portability-kernel`), progressive-enhancement degraded paths, per-cell DR-pair failover (ADR-0241, RTO≤5min/RPO≤30s) that preserves per-pack residency + cell-isolation, per-pack disaster overlays, and the absolute invariant that emergency-services (ADR-0298) NEVER throttle.
- **domain:** dr-resilience (cross-cut: orchestration-scheduling)
- **current_status:** Proposed
- **disposition:** AMEND (then RATIFY-as-amended)
- **proposed_resolution:** RATIFY — the emergency-services-never-throttle invariant + cross-pack-failover-forbidden + cell-isolation-preserved are exactly right and are the P0 hard-rules a resilient substrate needs. AMEND: retired `oya.foundry-fitness.*` BNF; and reconcile its DR-pair/cell topology with the SOURCE canonical orchestration posture (Talos+CAPI+ArgoCD per ADR-0375) which it does not cite — it leans on ADR-0241/0248 only.
- **governing:** n/a
- **truth_flag:** TRUE / STALE-vocab.
- **in_masterplan:** PARTIAL — 6 gates + 2026-10-31 blocker (the last clock in the bundle) + quarterly game-day + chaos-monkey mandate; high ops planning_impact.
- **tensions:** Depends on ADR-0304 (residency preserved through failover), ADR-0298 (emergency path), ADR-0241 (DR topology) — consistent. Potential tension with LINUX-side framekernel/own-the-host ambition (keystone §5 fault-line 3) only if disaster-mode is later implemented on a Rust node-OS rather than k8s; not a present conflict.
- **hyperscaler_challenge:** ALIGNED. AWS cell-based + region-pair, Google Borg/Spanner isolation, Azure region-pairs, Cloudflare/Akamai/Fastly per-POP failover, Netflix chaos engineering, FEMA/911 surge — textbook. A hyperscaler WOULD make this decision. Argues for amend, not archive.
- **ai_slop / refinement / consensus_needed:** Closes THREE §3.2.5 rows (14+22+30) in one ADR — reasonable bundling. **Founder question:** the quarterly-game-day + mandatory chaos-monkey-per-µservice enforcement is a mature-org operating cost; is that a day-0 blocker or a post-GA promotion? (Same advisory→blocker-clock proportionality question as the whole bundle.)

---

### ADR-0307 — Detection Substrate (Streaming + Batch) — DRMP "D" Layer

- **decision_atom:** Establish `microservices/detection/` as a single-concern substrate µservice with eight primitives (Flink-class streaming, Spark/Polars/Trino batch, feature store, Sigma-class rules engine w/ ≥7-day soak, LightGBM/SHAP composite scorer, Apache-AGE/Neo4j graph + community detection, investigation integration, sandbox-replay) covering all eight detection families (payment-fraud, ATO, synthetic-identity, AML/sanctions, content-abuse, engagement-manipulation, insider-risk, policy-violation), emitting signals to Cedar (never auto-blocking the default/emergency path).
- **domain:** security-supplychain (cross-cut: intelligence-ai)
- **current_status:** Proposed
- **disposition:** AMEND (then RATIFY-as-amended) — heaviest amend in the chunk.
- **proposed_resolution:** RATIFY-as-amended — the "detection is a substrate, not per-µservice" thesis is correct (Stripe Radar / Adyen / Chronicle / GuardDuty all prove it). BUT this ADR has the most retired-vocab debt in the chunk and MUST be amended before ratification: (a) it names **Kafka** as the canonical event bus throughout (§B, §D-1) — Kafka is RETIRED → **Pulsar 4.x + Oxia** per ADR-0377-kafka-to-pulsar (sup. ADR-0005); (b) **Redis** as the feature-store online tier (§B.3, §C.3) — RETIRED → **Valkey** per ADR-0336; (c) `oya.foundry-fitness.*` lanes → `oya-governance-*`; (d) cites `ADR-0293-foundry-meta-trust-root` and the foundry brand — RETIRED → intelligence (ADR-0335).
- **governing:** n/a (not itself superseded; but it consumes ADR-0005 Kafka which IS superseded by ADR-0377 — so it inherits the eventing supersession transitively).
- **truth_flag:** PARTIAL — TRUE thesis, WRONG substrate choices (Kafka, Redis) per current canonical posture, STALE foundry vocab.
- **in_masterplan:** YES (PARTIAL binding) — 10 `enforced_by` gates, a closed 8-family enum, 7 audit classes, and explicit DRMP roadmap positioning ("row 49 of §3.2.1"); this is high-planning_impact and is meant to be a binding masterplan node. Under generated-from-ADRs it injects a major subsystem; under masterplan-authority it needs a `masterplan_ref`.
- **tensions:** DIRECT conflict with eventing canonical posture (Kafka vs Pulsar, keystone §2/§3) and data-storage posture (Redis vs Valkey). Also asserts ClickHouse/Trino/Iceberg/Feast/Flink/Spark as canonical — these are NOT in the keystone canonical-posture table for SOURCE (which names Milvus/SeaweedFS/Ceph/ClickHouse/TimescaleDB/Postgres) — ClickHouse aligns, the rest (Flink/Spark/Feast/Kafka/Redis) are NEW substrate commitments this ADR makes unilaterally. That is a scope-expansion the founder should ratify explicitly, not absorb silently.
- **hyperscaler_challenge:** ALIGNED on thesis, QUESTIONABLE on breadth. Every hyperscaler runs a central detection substrate — but they do NOT commit, in a single ADR, to Flink+Spark+Feast+ClickHouse+AGE+LightGBM+8-families-day-0; they grow it family-by-family. Building all eight detection families from the substrate on day-0 (the ADR explicitly forbids "1-2 families with the rest later") is the kind of breadth a hyperscaler would stage, not commit up front. Argues for AMEND (substrate-vocab fix) + a phased-coverage refinement, not archive.
- **ai_slop / refinement / consensus_needed:** Lower slop than it looks — the family coverage matrix and capacity math are substantive. The slop risk is **premature-precision** (exact $/month, exact partition counts "at platform GA" for a pilot with no GA). **Founder question (CONTESTED):** does the founder accept Kafka→Pulsar / Redis→Valkey rewrites here as mandatory (yes, per canon), AND is day-0 eight-family coverage a real requirement or aspirational? This ADR should not ratify until the eventing/cache substrate names are corrected.

---

### ADR-0308 — ML Model Lifecycle (EU AI Act + NIST AI RMF + ISO/IEC 42001)

- **decision_atom:** Establish a substrate-managed eight-stage ML lifecycle (training w/ per-tenant residency + Iceberg snapshot, validation w/ bias-audit + 4/5ths disparate-impact, champion-challenger A/B shadow→canary→full, daily drift detection, quarterly fairness re-audit, SemVer + Google-Model-Card versioning, ≤15min rollback + EU-AI-Act-Art-73 24h serious-incident reporting, GDPR-Art-22/EU-AI-Act-Art-86/ECOA/NY-AEDT appeal mechanism) that every production ML model in the detection substrate (ADR-0307) MUST register into.
- **domain:** intelligence-ai (cross-cut: compliance-residency)
- **current_status:** Proposed
- **disposition:** AMEND (then RATIFY-as-amended)
- **proposed_resolution:** RATIFY-as-amended — the regulator surface is real and non-negotiable for the high-risk Annex-III workloads ADR-0307 covers (biometric ID, credit scoring, content moderation, employment ranking all = EU AI Act high-risk), so this lifecycle is genuinely required if 0307 ships. AMEND: `oya.foundry-fitness.*` → `oya-governance-*`; cites `ADR-0293-foundry-meta-trust-root` (retired brand). It is otherwise cleaner on substrate-vocab than 0307 (no Kafka/Redis dependency of its own; inherits 0307's).
- **governing:** n/a (couples tightly to 0307; if 0307 is staged/phased, 0308's blocker clock must move with it).
- **truth_flag:** TRUE / STALE foundry-vocab.
- **in_masterplan:** YES (PARTIAL binding) — 10 gates (incl. hard BLOCKER lanes for Art-73 24h reporting + quarterly fairness), 10 audit classes, explicit "row 50 of §3.2.1"; intended binding node, build-ahead-of-certification per ADR-0250.
- **tensions:** Inherits ADR-0307's Kafka/Redis/Feast substrate debt transitively (training reads the feature-store offline tier defined in 0307). The "build-ahead-of-certification → ML lifecycle MUST be in place before detection GA" coupling means 0307 and 0308 ratify/stage as a pair. No standalone conflict.
- **hyperscaler_challenge:** ALIGNED. Google Model Cards, Microsoft Fairlearn/Datasheets, Meta System Cards, OpenAI/Anthropic model reports, Arize/Fiddler/WhyLabs/Evidently drift, Stripe Radar ≤15min rollback — all real. A regulated hyperscaler serving Annex-III workloads WOULD make this decision; it is table-stakes for EU AI Act high-risk. Argues for amend, not archive.
- **ai_slop / refinement / consensus_needed:** The statute-citation wall (EU AI Act + NIST + ISO 42001 + NY AEDT + ECOA + FHA + BIPA + CUBI + 8 state AI laws + KR/JP/UK/AU) is the slop tell, but the underlying obligations are accurate. **Founder question:** build-ahead-of-certification commits a full ML-governance apparatus before there is a single production model — is that day-0 (per ADR-0250) or does the advisory→2026-09-15-blocker clock get reset to track actual detection-substrate readiness?

---

## Chunk notes

**1. One batch, one set of systemic defects.** All seven are the 2026-05-20 keystone bundle. They share IDENTICAL structural DNA (substrate-primitive thesis → shared crate → per-µservice Cedar fragment → CI fitness lanes → ADR-0263 audit classes → advisory→blocker clock) and IDENTICAL retired-vocab leakage. Fix the vocab once, across the batch.

**2. RETIRED-VOCAB LEAKAGE (the dominant finding) — every ADR in the chunk carries it:**
   - **`oya-foundry-fitness-*` / `oya.foundry-fitness.*` CI-lane BNF segments** in ALL 7 (the `name:` fields already say `oya-governance-…`, but the `bnf_segments` and `layer:` comments still say `oya.foundry-fitness.…`). RETIRED → `oya-governance-*` per **ADR-0347** (keystone §2). This is the single most pervasive defect.
   - **`oyatie.foundry.*` principal namespace + "Foundry agents"/"Foundry meta-trust-root" brand** in ADR-0305 (§A.2) and ADR-0307/0308 (cite `ADR-0293-foundry-meta-trust-root`) and ADR-0303 (lists "foundry" as a consuming µservice). Brand RETIRED → **intelligence** (consumer AI) / **governance** (CI) per **ADR-0335** (keystone §2).
   - **Kafka** named as canonical event bus in **ADR-0307** (§B, §D-1 topic lists) and inherited by **ADR-0308**. RETIRED → **Pulsar 4.x + Oxia (KoP)** per **ADR-0377-kafka-to-pulsar** (sup. ADR-0005).
   - **Redis** named as feature-store online tier in **ADR-0307** (§B.3, §C.3, §C.5). RETIRED → **Valkey** per **ADR-0336**.
   - **Milestone ids** (`M-CC-P11`, `M01-P18`-style) referenced in ADR-0305 (§A.1 "Workflow Studio PRD (M-CC-P11 substrate scope)"). RETIRED M0–M3/Milestone vocabulary per keystone §2 → Wave names / function-named ids.

**3. NO UNACCOUNTED PROPOSALS, but a clustered ratify decision.** All 7 are `Proposed` and all 7 resolve to **RATIFY-as-amended** (none DROP). Rationale: every one closes a real, hyperscaler-validated gap; none conflicts with another ADR or with LINUX; the only blockers are vocabulary + (for 0304/0307) scope/legal questions the founder must answer. They should NOT be ratified at face value (the vocab is wrong); they should be amended-then-accepted as a batch.

**4. Dependency order for ratification.** ADR-0304 (higher-restriction-wins precedence) is the cluster keystone — 0302/0303/0305/0306 all defer their multi-pack conflict resolution to it. ADR-0307→0308 are a coupled pair (no detection GA without ML lifecycle, per ADR-0250). Ratify 0304 first; ratify 0307+0308 together (or stage together).

**5. Two genuinely CONTESTED founder questions (beyond vocab):**
   - **ADR-0304:** does the substrate *refuse* foreign court orders (CLOUD Act) as a default, or *surface + route to legal-council/MLAT*? The §B headline says refuse; the §B.2 steps actually do route. This is a legal-strategy commitment for a US-incorporated entity — questionable vs hyperscaler practice (they litigate, not auto-refuse). Surface, do not auto-accept.
   - **ADR-0307:** day-0 eight-family detection coverage + a full Flink/Spark/Feast/ClickHouse/AGE stack committed in one ADR. The thesis is right; the breadth is what a hyperscaler would *stage*. Combined with the mandatory Kafka→Pulsar / Redis→Valkey rewrites, this ADR needs the most work before it is masterplan-ready.

**6. Proportionality (the over-engineering signal, not a falseness signal).** The chunk adds **41 CI fitness lanes** (7+6+5+6+6+10+10 across `enforced_by`) and ~40 audit-event-classes for a pilot with no product surface yet. Each individual lane is justified; the aggregate is a mature-hyperscaler operating apparatus being stood up advisory-now/blocker-by-Oct-2026. The recurring founder question across all 7 is the same: **are these advisory→blocker clocks tied to real product readiness, or are they calendar-driven enforcement of doctrine ahead of anything to enforce against?** Under the founder's "masterplan = only what's needed" goal, the decision atoms are KEEP-worthy; the enforcement-apparatus density is the thing to right-size.

**7. Masterplan binding (BOTH readings, per keystone §4).** None of the 7 carries a `masterplan_ref` (consistent with keystone's 8.8%-bound finding). Under **generated-from-ADRs** (planning-ssot-consolidation), all 7 — once vocab-amended — generate substantial masterplan nodes (crates, enums, gates, audit classes) and are KEEP. Under **masterplan-is-authority** (drift-prevention), all 7 need bidirectional `masterplan_ref` binding added before they count. Flagged under both; not resolved here.

**8. LINUX cross-impact.** Zero subject overlap with LINUX pilot ADRs 0001–0026 (LINUX is DB-engine/policy-language/framekernel/node-OS; this chunk is user-protection + detection + ML-governance). On merge, all renumber per keystone §6.4. The only indirect tie: ADR-0305's delegated-agent authority model is the most strategically-relevant to the cloud-native-Rust agentic stack and is arguably mis-homed in this edge-case bundle.
