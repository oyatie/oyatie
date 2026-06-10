# ADR Audit — SOURCE chunk 27

- **side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`)
- **chunk:** 27 (slice lines 183–189 of `ls -1 docs/decisions/ADR-*.md | sort`)
- **range:** ADR-0208 → ADR-0214
- **ADRs reviewed:** 7 (0208, 0209, 0210, 0211, 0212, 0213, 0214)
- **auditor posture:** READ-ONLY; keystone map = canonical baseline; superseding ADR > stale front-matter; masterplan authored-vs-generated treated as OPEN.

---

### ADR-0208 — Realtime transport tier: SSE / WebSocket / gRPC streaming with closed responsibility split

- **decision_atom:** Realtime traffic uses a closed three-tier transport split — SSE for one-way server→client streams, WebSocket for bidirectional client surfaces, gRPC streaming for service-to-service only — with no long-polling, no client-facing gRPC, mandatory reconnect/resume, and per-tenant connection ceilings.
- **domain:** networking-mesh
- **current_status:** Accepted
- **disposition:** AMEND
- **proposed_resolution:** NA (already Accepted)
- **governing:** —
- **truth_flag:** PARTIAL — the transport-tier doctrine is TRUE and clean, but the multi-region routing section names "Redis Cluster / Valkey pub-sub" (§Multi-region routing L74) — `Redis` is retired vocabulary per ADR-0336 (Valkey is canonical). Stale-vocab leakage; should read Valkey-only.
- **in_masterplan:** YES — pins a canonical transport enum + per-tenant ceilings; clearly planning-binding for every client-facing µservice.
- **tensions:** Cites Loro CRDT "per ADR-0145"; ADR-0145 also appears in the keystone supersession graph as a *superseding* ADR (it supersedes 0140/0141) — no conflict, but the body references `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145)` semantics elsewhere in the corpus (see 0213). No direct ADR-vs-ADR contradiction here.
- **hyperscaler_challenge:** Aligned. Google/AWS/Azure all converge on SSE+WebSocket for browser surfaces and gRPC for internal RPC; deferring WebTransport until broad browser support is exactly the conservative hyperscaler call. Argues for AMEND only (Redis→Valkey naming), not archive.
- **ai_slop:** Low. Dense but substantive (per-stack adapter table, concrete ceilings). The per-tenant ceilings (10k WS / 50k SSE / 200k cell) are asserted without a sizing citation — mild over-precision.
- **refinement:** Replace "Redis Cluster / Valkey" with "Valkey" (ADR-0336).
- **consensus_needed:** None contested.

---

### ADR-0209 — Compliance evidence automation: in-house pipeline replacing Drata / Vanta

- **decision_atom:** oyatie builds an in-house compliance-evidence pipeline (SOC 2 Type II continuous collectors, GDPR DSAR export/delete/rectify automation, HIPAA min-necessary logs, PCI-DSS-when-needed) on existing audit-chain/deploy-receipt/Cedar/object-store primitives rather than wrapping Drata/Vanta, with a kernel-enforced cross-tenant DSAR isolation invariant.
- **domain:** compliance-residency (cross-cut: security-supplychain)
- **current_status:** Accepted
- **disposition:** AMEND
- **proposed_resolution:** NA (already Accepted)
- **truth_flag:** PARTIAL — decision is TRUE and a stated Class-C moat (confirmed by ADR-0211 §Class C). But two stale references: (1) auditor portal is specified as a "Backstage view (per ADR-0170)" — ADR-0170 Backstage is SUPERSEDED by ADR-0394 (bespoke-Rust IDP; Backstage quarantined) per keystone §1.1/§2; (2) references "Cedar policy snapshots (ADR-0183)" and "Kyverno admission" lineage — ADR-0183 is SUPERSEDED by ADR-0379 (Kubewarden), though Cedar app-authz survives.
- **governing:** ADR-0394 (dev-portal), ADR-0379 (admission) for the stale sub-references; the ADR-0209 decision itself is not superseded.
- **in_masterplan:** YES — names a day-one differentiator with a concrete µservice (`microservices/compliance/`), collectors, and a coverage gate.
- **tensions:** Backstage dependency (ADR-0170) is dead substrate → auditor-portal surface must re-anchor on ADR-0394 IDP. SeaweedFS "per ADR-0145" object-store attribution is loose (object store is ADR-0196 per keystone §3) — minor mis-cite.
- **hyperscaler_challenge:** Questionable-leaning-aligned. Hyperscalers (AWS Audit Manager, Azure Compliance Manager, GCP Assured Workloads) BUILD compliance tooling in-house — so "own it" is hyperscaler-aligned in principle. The "4-6 week" build estimate for SOC2+GDPR+HIPAA+PCI is optimistic by hyperscaler standards (these are multi-quarter programs). Argues for AMEND (re-anchor portal; sober the estimate), not archive.
- **ai_slop:** Low-moderate. The "$50k-$500k/yr" and "4-6 week" figures are confident but uncited; classic AI over-confidence on effort.
- **refinement:** Re-point auditor portal from Backstage/ADR-0170 to ADR-0394 IDP; fix ADR-0183→ADR-0379 and object-store→ADR-0196 cites.
- **consensus_needed:** "Is the in-house compliance pipeline genuinely a day-one differentiator, or does shipping it day-one starve Class-C product moats (Workflow/Ontology/Intelligence)?" — founder call on sequencing.

---

### ADR-0210 — OpenTelemetry tail-sampling: 100% error / p99 / new-endpoint traces + 1% baseline

- **decision_atom:** Trace sampling is two-stage — 1% head sampling at the agent collector plus OTel tail-sampling that retains 100% of error / p99-slow / new-endpoint-warmup / SLO-burn / audit-event traces — tuned per-µservice via a manifest `trace_sampling_recipe`, achieving ~10× storage reduction vs always-on.
- **domain:** observability (cross-cut: finops-cost)
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (already Accepted)
- **governing:** —
- **truth_flag:** TRUE — wholly consistent with the canonical observability posture (LGTM/Tempo per ADR-0383, OTel emission contract per ADR-0263). Uses the canonical community processor; no retired vocabulary; no superseded dependency. Cleanest ADR in the chunk.
- **in_masterplan:** YES — declares a concrete manifest field shape (`observability.trace_sampling_recipe`) and a Helm chart; directly binds µservice manifests.
- **tensions:** None. Extends ADR-0186 (LGTM backplane) and ADR-0139 (burn-rate SLO) coherently; ADR-0211 even wires Class-B trigger monitoring through this recipe.
- **hyperscaler_challenge:** Aligned. This is literally the Google Dapper / Honeycomb / Lightstep playbook (head-low + tail-on-the-interesting-tail). No hyperscaler would dispute it. Argues for KEEP.
- **ai_slop:** Very low. Concrete policy table, cost model, escape-hatch thresholds, manifest JSON example — genuinely buildable.
- **refinement:** None material. (Cites ADR-0153 "LGTM" lineage which is the older backplane ref vs ADR-0383 destination; harmless.)
- **consensus_needed:** None.

---

### ADR-0211 — In-House Tech Stack Policy

- **decision_atom:** Every external dependency is classified A (community-standard KEEP behind a thin adapter), B (vendor-replaceable now, value-anchored — never date-anchored — Phase-2 native trigger registered in `registry/vendor-lockin-phaseout/index.json`), or C (in-house mandatory differentiation from day one), enforced by CI gates.
- **domain:** governance-process (cross-cut: security-supplychain)
- **current_status:** Accepted
- **disposition:** AMEND
- **proposed_resolution:** NA (already Accepted)
- **governing:** — (doctrine survives; sub-references drift)
- **truth_flag:** PARTIAL — the A/B/C doctrine is TRUE, load-bearing, and a keystone fault-line anchor (the "own when proven" ratchet, keystone §5). BUT the worked examples carry heavy retired-vocab and superseded-dep leakage: "Foundry (internal Hermes agentic substrate per ADR-0136 amendment)" as a Class-C example — **foundry/Hermes are RETIRED** (ADR-0335/0347; →intelligence+governance); ADR-0136 itself is flagged stale-superseded in keystone §1.3. Also lists "Backstage (developer portal)" as a live Class-B vendor (superseded by ADR-0394), and the deciders front-matter is `axis-foundry` (retired axis name).
- **in_masterplan:** YES — this IS the substrate-doctrine that the vendor-lockin gate enforces corpus-wide; deeply planning-binding.
- **tensions:** (1) The Class-B table's value-anchored triggers (≥50K tenants Zitadel, ≥1B vectors Milvus, ≥100TB ClickHouse) are the SOURCE side of the keystone fault-line vs LINUX ADR-0019/0020 "own when proven" — keystone §5 says the *principle* is shared, only the *trigger threshold* differs. (2) Milvus Phase-2 trigger here (≥1B vectors) is looser than LINUX ADR-0020's hard vector-count gate. (3) "Foundry vs Intelligence split per ADR-0136/0220" in §Operational is pre-0335 framing.
- **hyperscaler_challenge:** Aligned (strongly). The named-industry-sources section is exactly right — AWS/Google/Azure/Oracle adopt CNCF + build the moat. This is the most hyperscaler-defensible decision in the chunk. Argues for AMEND (scrub foundry/Hermes/Backstage residue), emphatically not archive.
- **ai_slop:** Low. Genuinely substantive (the doc even self-cites as meeting ADR-0212's bar). The Class-C "Foundry" bullet is the one slop/staleness artifact.
- **refinement:** Replace Class-C "Foundry (Hermes)" example with "Intelligence (consumer AI) + Governance (CI/gates)" per ADR-0335/0347; drop Class-B "Backstage" → ADR-0394 bespoke IDP; rename `axis-foundry` decider.
- **consensus_needed:** "Is the Class-B *trigger threshold* (≥50K tenants / ≥1B vectors / ≥100TB) the right own-when-proven gate, or does the LINUX pilot's lower/harder gate win on merge?" — the keystone §5 founder question, surfaced here concretely.

---

### ADR-0212 — Buildability Doctrine

- **decision_atom:** Every non-code artifact (PRD/ADR/IP/contract/Helm/runbook/SLO/threat-model/standards/scorecard) must meet a per-kind buildability bar — enough substance that a cold stranger or AI agent can produce hyperscaler-grade output from it alone — enforced by a structural-line-count + alternatives/consequences/sources CI gate plus a reviewer-agent stranger-test.
- **domain:** governance-process (cross-cut: docs-ssot-masterplan)
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (already Accepted)
- **governing:** —
- **truth_flag:** TRUE — sound authoring doctrine, self-consistent (it grades ADR-0211 against its own bar), no retired vocabulary in the decision body. Minor: deciders front-matter again lists `axis-foundry` (retired axis label) — cosmetic.
- **in_masterplan:** PARTIAL — it is process/quality doctrine, not a runtime/architecture binding; it shapes how masterplan-feeding artifacts are written rather than declaring a planning deliverable. Carries doctrine weight but limited direct masterplan-binding.
- **tensions:** None architectural. The "150 substantive lines" bar vs current "117-119" honest-disclosure floor is a self-acknowledged staged-promotion gap, not a conflict.
- **hyperscaler_challenge:** Aligned. Stripe/AWS/Cloudflare/Google design-doc discipline is the cited model; the stranger-walks-up-cold test is a real hyperscaler onboarding property. Argues for KEEP.
- **ai_slop:** Low — and notably self-aware about slop ("padding to bar" is the explicit failure mode it fights). Slightly meta/recursive but intentionally so.
- **refinement:** Rename `axis-foundry` decider; otherwise none.
- **consensus_needed:** None contested.

---

### ADR-0213 — Ecosystem-as-a-Service architecture — Plugin/App Store substrate + Developer SDK

- **decision_atom:** Ship Ecosystem-as-a-Service as two single-concern in-house µservices — `plugin-app-store` (third-party plugin discovery/install/vetting/per-plugin Cedar permissions/billing aggregation) and `developer-sdk` (contracts + 6-stack SDK codegen + Wasmtime sandbox + dev portal + in-house KYC/payout) — superseding the thin Bominal ADR-0037 plugin loader, with `marketplace` reserved for future B2C commerce.
- **domain:** marketplace-commerce (cross-cut: agentic-platform / product-ux)
- **current_status:** Proposed
- **disposition:** AMEND (then RATIFY)
- **proposed_resolution:** **RATIFY** — it is a founder-directed decision ("Direction C", user directive 2026-05-18), is cited as a Class-C moat by ADR-0211, and PR #143 (its close-out PR) is referenced as merged context across the chunk; leaving it Proposed is an unaccounted live moat. Ratify after scrubbing dead substrate refs. (Front-matter even targets "Accepted upon PR #143 merge.")
- **governing:** supersedes Bominal ADR-0037 (for new work) — self-declared.
- **truth_flag:** PARTIAL — core architecture (two µservices, Wasmtime sandbox per ADR-0147/0200, Cosign vetting per ADR-0181, per-plugin Cedar) is TRUE and well-formed. STALE deps: developer portal anchored on "Backstage (ADR-0170)" (superseded by ADR-0394); body literally cites `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md` as the inter-service routing path (retired artifact named in-line); ADR-0110 ChangeSet state-machine inheritance (ADR-0110 is SUPERSEDED by ADR-0363 per keystone §1.1). Front-matter `related_adrs` map ADR-0001/0002/0003/0007/0008 to oyatie concerns that look Bominal-inherited-renumbered (ADR-0007=Cedar here, but source canon Cedar is ADR-0150/0243) — a numbering-provenance smell.
- **in_masterplan:** YES — declares two µservices, a phased rollout (M02b→M07), hyperscaler-gate evidence (HG-PAS/HG-SDK), and naming reservation; heavily planning-binding. (Note: M0x milestone tags are retired vocab per keystone §2 — should be Wave names.)
- **tensions:** (1) Backstage portal dependency dead (ADR-0394). (2) ADR-0110 inheritance dead (ADR-0363). (3) Milestone `M02b/M04…M07` labels are retired (Wave names per GLOSSARY). (4) In-house KYC/AML/payout (ACH/SEPA/KFTC/1099/VAT MOSS) is an enormous regulated surface asserted as "100% in-house from day one" — tension with ADR-0211's own Class-B "use a vendor behind an adapter until proven" ratchet (payments KYC is the textbook Class-B, yet declared Class-C).
- **hyperscaler_challenge:** Questionable. The plugin-store/SDK/sandbox/vetting architecture is hyperscaler-aligned (Apple/AWS/VS-Code-class). BUT in-house KYC/AML + multi-jurisdiction payout rails is precisely what hyperscalers DO NOT build — AWS/Apple/Shopify all use Stripe/Adyen/regulated-partner rails for payouts and third-party KYC. Building ACH/SEPA/KFTC settlement + tax-form generation in-house "day one" is misaligned and argues for AMEND (reclassify payout/KYC as Class-B vendor-seamed, not Class-C).
- **ai_slop:** Moderate. Very long, extremely confident, and conflates a genuinely sound plugin architecture with an over-reaching "we build the global payment+KYC+tax stack ourselves" claim. The 8-comparable inheritance list and 6-stack SDK codegen are buildable; the in-house-payments scope is aspiration dressed as decision.
- **refinement:** Re-anchor portal on ADR-0394; replace ADR-0110 lifecycle inheritance with the surviving state-machine doctrine (post-0363); convert M0x → Wave names; reclassify developer-sdk KYC/AML/payout as Class-B (vendor-seamed adapter) per ADR-0211 rather than Class-C day-one.
- **consensus_needed:** "Is third-party-developer KYC/AML and multi-rail payout (ACH/SEPA/KFTC) genuinely Class-C in-house-day-one, or Class-B vendor-seamed (Stripe Connect / Adyen behind an adapter) until proven? Hyperscalers choose the latter." — direct founder/finops call; this is the sharpest contested decision in the chunk.

---

### ADR-0214 — Cross-Tenant Real-Time Visibility (Consent-Graph + Ontology Projection Extension)

- **decision_atom:** Build a single-concern `consent-graph` µservice as the kernel for all cross-tenant data flows — a first-class `DataSharingAgreement` entity with three sharing modes (real-time Projection / k-anonymized Aggregate / AttestedQuery), Cedar-enforced fail-closed at every hop, zero-copy region-pinned Pulsar projection (grantor row never migrates), ≤1s real-time revocation, and bilateral audit-chain entries on both sides.
- **domain:** tenancy (cross-cut: authz-policy / compliance-residency)
- **current_status:** Proposed
- **disposition:** AMEND (then RATIFY)
- **proposed_resolution:** **RATIFY** — front-matter says "target: Accepted upon PR #143 merge to `dev`"; PR #143 is the merged close-out referenced throughout the chunk; ADR-0211 cites Consent Graph as a Class-C moat. It is a live, founder-narrative differentiator left Proposed — ratify after the substrate citations are corrected, else it is an unaccounted proposal. DROP is not warranted (the moat is real and uncontested in concept).
- **governing:** —
- **truth_flag:** PARTIAL — the decision (Open-Banking consent + HIE bilateral-audit + Snowflake zero-copy fusion, fail-closed Cedar, ≤1s revocation) is TRUE, vertical-agnostic, and a genuine moat. BUT a cluster of STALE substrate cites in §5 In-House Roadmap: "Cedar (ADR-0183)" — ADR-0183 superseded by ADR-0379 (Cedar app-authz survives but the canonical Cedar gate is ADR-0243/0246); "ontology µservice (ADR-0058)"; "OpenBao (ADR-0043)"; "Postgres + Citus (ADR-0034)" — Citus is not the canonical relational pooling posture (keystone §3 = Postgres + pgcat / ADR-0179); and §8 Sunset hypothesizes "swap Pulsar for Kafka" though Kafka is RETIRED (ADR-0377). Also "capability tier" / "capability-tier T3" language (§2.3, §6.3) — must not be conflated with the retired tenant tier-system (ADR-0329); here it appears to mean autonomy/maturity tier, but the wording is ambiguous.
- **in_masterplan:** YES — declares a µservice, an entity schema, 9 SLOs, 8 runbooks, 7 day-1 use cases, and follow-on IPs (IP-CT-001..005); heavily planning-binding.
- **tensions:** (1) Pulsar dependency is correct-forward (ADR-0377) but §8 names Kafka as the swap-back target → retired-vocab leak. (2) "Postgres + Citus (ADR-0034)" conflicts with canonical Postgres+pgcat (ADR-0179). (3) Cedar attribution to superseded ADR-0183 rather than ADR-0243/0246. (4) `foundry_pipeline` coordination surface (front-matter L17) = retired brand. (5) "capability tier" wording risks conflation with retired ADR-0329 tier-system.
- **hyperscaler_challenge:** Aligned-to-ambitious. The fusion (consent + bilateral audit + zero-copy projection + sub-second revocation) is genuinely ahead of Snowflake Secure Data Share / Databricks Delta Sharing and is a defensible moat a hyperscaler would respect. The ≤1s end-to-end cross-region revocation SLO with bilateral audit is aggressive but not implausible on Pulsar. Argues for AMEND (fix substrate cites), not archive — the decision is well-reasoned.
- **ai_slop:** Low-moderate. Strong competitive analysis (§1 alternatives are genuinely good), concrete Rust entity, real SLOs/runbooks. The substrate-citation drift (Citus, Kafka-swap, ADR-0183 Cedar) is the staleness, not slop.
- **refinement:** Re-cite Cedar to ADR-0243/0246; replace "Postgres + Citus (ADR-0034)" with Postgres + pgcat (ADR-0179); strike the "swap Pulsar for Kafka" hypothetical in §8; clarify "capability tier" ≠ tenant tier-system; drop `foundry_pipeline` coordination label.
- **consensus_needed:** "Is the ≤1s cross-region bilateral-audit revocation SLO a real day-1 commitment or an aspirational ceiling?" — minor; the architecture is otherwise uncontested.

---

## Chunk notes

**Overall posture.** This is a strong, coherent chunk — a "PR #143 close-out" cluster (0211/0212 explicitly, 0213/0214 target "Accepted upon PR #143 merge", 0208/0209/0210 dated the same 2026-05-18). Five are Accepted, two are Proposed. No GARBAGE, no archive candidates, no ADR-vs-ADR direct supersession within the range. The dominant finding is **stale-dependency drift, not wrong decisions**: every doc that names a developer portal cites the dead **Backstage/ADR-0170** (superseded by ADR-0394), and several cite the superseded **ADR-0183** Cedar/Kyverno lineage instead of ADR-0243/0246/0379. The decisions themselves are sound and largely hyperscaler-aligned.

**Retired-vocabulary leakage tally (auditor must flag for the masterplan generator):**
- `Redis` (ADR-0208 §multi-region) → Valkey (ADR-0336).
- `Backstage` / ADR-0170 dev-portal (ADR-0209, 0211, 0213) → bespoke-Rust IDP (ADR-0394).
- `Foundry` + `Hermes` (ADR-0211 Class-C example; `axis-foundry` decider in 0211/0212; `foundry_pipeline` coordination in 0214) → intelligence + governance (ADR-0335/0347).
- `Kafka` (ADR-0214 §8 swap-target) → Pulsar/Oxia (ADR-0377).
- `M02b/M04…M07` milestone labels (ADR-0213 phased rollout) → Wave names (GLOSSARY, retired 2026-05-09).
- `ADR-0183` Cedar lineage (ADR-0209, 0214) → ADR-0243/0246 (app-authz) + ADR-0379 (admission).
- `ADR-0110` ChangeSet inheritance (ADR-0213, and referenced in 0214 lifecycle) → superseded by ADR-0363.
- `Postgres + Citus / ADR-0034` (ADR-0214) → Postgres + pgcat / ADR-0179.

**Two Proposals to clear (no unaccounted proposals rule):** Both ADR-0213 and ADR-0214 are **RATIFY** — each self-declares "Accepted upon PR #143 merge," each is cited as a Class-C moat by ADR-0211, and PR #143 is treated as merged context across the chunk. Neither should remain Proposed; ratify after the dependency-citation scrub above.

**Sharpest contested decision (founder-level):** ADR-0213's claim that third-party-developer **KYC/AML + multi-rail payout (ACH/SEPA/KFTC) + tax-form generation is Class-C in-house-from-day-one**. This contradicts the spirit of ADR-0211's own value-anchored ratchet (payments/KYC is the canonical Class-B vendor-seam) and is misaligned with every hyperscaler (Apple/AWS/Shopify all use Stripe/Adyen/regulated partners for payouts). Recommend reclassifying as Class-B vendor-seamed adapter pending a value-anchored trigger.

**Cross-side (LINUX) note:** ADR-0211's Class-B trigger thresholds (≥50K tenants / ≥1B vectors / ≥100TB) are the SOURCE anchor of keystone fault-line §5 ("own when proven" — same principle as LINUX ADR-0019/0020, different trigger threshold) and the Milvus row is the looser counterpart to LINUX ADR-0020's hard vector gate. Surface on merge, do not resolve here.

**Masterplan authored-vs-generated (OPEN):** Under the *generated-from-ADRs* reading, 0210/0212 are clean SSOT inputs as-is, while 0208/0209/0211/0213/0214 would need the retired-vocab/stale-dep scrub above *before* they can feed a generated masterplan (or the generator must apply the supersession map). Under the *masterplan-is-authority* reading, the same scrub is the binding-fix. Either way the corrective action set is identical — flagged under both readings per the keystone directive.
