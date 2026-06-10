# ADR Audit — source-38

- **side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`)
- **chunk:** 38 (`sed -n "260,266p"` slice of `ls -1 docs/decisions/ADR-*.md | sort`)
- **range:** ADR-0323 .. ADR-0329 (contiguous, 7 ADRs)
- **ADRs reviewed:** 0323, 0324, 0325, 0326, 0327, 0328, 0329
- **auditor posture:** READ-ONLY; only this artifact written. Keystone map consulted for supersession/retired-vocab/masterplan rulings.

> **Cluster framing.** 0322–0327 are the "W4 substance-bar doctrine" wave (governance process meta-doctrine about *how ADRs/docs get authored and promoted*). 0325/0326 are the product/compliance pair (pricing anchors + residency). 0328 is the realignment-sequence doctrine. 0329 is the **one Accepted keystone** in the chunk: tier-system retirement → tenant-class. The chunk is heavy on `foundry`-branded governance vocabulary (retired per ADR-0335/0347/0363) and tier vocabulary (retired *by 0329 itself*), so almost every Proposed member needs an AMEND for retired-vocab leakage even where the underlying decision is sound.

---

### ADR-0323 — Multi-Wave Sequencing Doctrine
- **decision_atom:** Substantive authoring and >5-service refactors are delivered as named, sequential **waves** (descriptor + substance-density-calibrated batch-size cap + per-wave verification cadence + evidence ledger), sequential within a governance lane and concurrent only across non-coupled lanes.
- **domain:** governance-process (cross-cut: docs-ssot-masterplan)
- **current_status:** Proposed
- **disposition:** AMEND
- **proposed_resolution:** RATIFY — the wave/batch-cap discipline is a genuinely useful authoring-process control and is already the operative cadence; but amend before/at acceptance to (a) strip `foundry`/`council-foundry`/`oya-foundry`/Foundry-pipeline branding → `intelligence`/`governance` per ADR-0335/0347/0363, (b) drop dependence on ADR-0110/0111/0112/0113 changeset-SM/webhook/VCS-orchestrator primitives which are **retired** by ADR-0363, re-expressing wave admission on plain-git + Forgejo/oya-ci.
- **governing:** n/a (not archived) — but binds to ADR-0335/0347/0363 (foundry retirement) and ADR-0363 (agentic-VCS retirement) for the AMEND.
- **truth_flag:** PARTIAL — core wave doctrine TRUE; cited substrate (Foundry pipeline, ADR-0110/0112/0113 VCS machinery) is STALE/retired.
- **in_masterplan:** PARTIAL — no `planning_impact:` flag in front-matter (unlike 0327/0329), but it is an authoring-cadence doctrine that the generated-masterplan process would consume. Not a product/architecture decision_atom for the masterplan's substance.
- **tensions:** Heavy reliance on ADR-0110/0112/0113 (all `Superseded` by ADR-0363 per keystone §1.1) and on the Foundry pipeline (retired brand). The "wave" naming overlaps the **retired** M0–M3/Wave-name vocabulary churn (keystone §2: "Wave names" themselves were a milestone-replacement that is itself partly anachronistic). Internal: cites `omc-teams` substrate — an agent-orchestration coupling that may not belong in an architecture ADR.
- **hyperscaler_challenge:** QUESTIONABLE. Google/AWS do gate large doc/code rollouts in waves (KEP-style staged enablement, change-freeze windows), so the *principle* is aligned; but the level of ceremony (12 audit-event classes, 5 dedicated `oya-governance-wave-*` crates, SLOs for descriptor validation) is far heavier than a hyperscaler would spend on an internal authoring-process gate. Argues for AMEND (slim the enforcement surface), not archive.
- **ai_slop:** Moderate. Symptoms: invented incident IDs with false precision (I-1..I-4 with exact PR counts/SHAs), the deliberately convoluted D-2 "mixed-tier cap" arithmetic that contradicts its own table, 16-named-reviewer-agent appendix. The decision survives the slop; the mechanics are over-specified.
- **refinement:** Collapse to the decision_atom + the D-2 cap table; move the 14 detailed-mechanics/SLO sections to a standard.
- **consensus_needed:** "Does wave-sequencing process doctrine belong as an immutable ADR feeding the masterplan, or is it an authoring-SOP that lives in a standards doc and never reaches the masterplan?"

### ADR-0324 — Anti-Script Anti-Template Doctrine
- **decision_atom:** Substantive content artifacts (ADRs, journeys, IP slices, READMEs, PRDs) must be authored per-artifact and bespoke; scripted/templated/lambda-wrap/batch-prompt body generation (AP-1..AP-8) is categorically forbidden and detected via provenance attestation + loop detector + template-stamping fingerprints.
- **domain:** governance-process
- **current_status:** Proposed
- **disposition:** AMEND
- **proposed_resolution:** RATIFY — the anti-template/anti-slop rule is exactly the discipline this whole audit exists to enforce and is clearly correct; amend to strip `foundry` dispatcher branding → intelligence/governance dispatcher, and to drop the assumption that a bespoke "Foundry pipeline dispatcher" exists (re-express against the actual agent-runtime).
- **governing:** n/a — binds to ADR-0335/0347 for the brand AMEND.
- **truth_flag:** TRUE (principle) / PARTIAL (foundry-branded enforcement substrate is stale).
- **in_masterplan:** NO — pure authoring-discipline meta-doctrine; no planning_impact binding; would not appear as a substance decision in a generated masterplan, though it governs *how* the masterplan's source ADRs are written.
- **tensions:** Companion-coupled to 0322/0323/0327 (cite each other circularly). Self-referential irony to flag: this very ADR-cluster (0322–0327) is itself a six-ADR batch authored in one wave with near-identical section skeletons — the kind of structural similarity the doctrine targets (it pre-empts this with the D-9/F-6 "structural_similarity_justified" carve-out). Couples to `omc-teams`.
- **hyperscaler_challenge:** ALIGNED in spirit (Google/Amazon design-doc culture forbids template-stamped design docs; code review rejects copy-paste), but no hyperscaler builds a `provenance.json` ed25519-signed per-artifact attestation chain + dispatcher-edge prompt-rewriter to enforce it — they rely on human review + culture. Argues for AMEND (keep the rule, drop the bespoke crypto-attestation enforcement theater).
- **ai_slop:** Moderate-high. The doctrine is anti-slop yet is itself padded (8 detailed-mechanics sections, 4 appendices, incident-replay with fabricated bash + commit SHA `4e2f...c19`). Substance survives.
- **refinement:** Reduce to AP-1..AP-8 list + the "author per-artifact, not in a batch" rule. Drop the attestation-signature machinery to a future enforcement ADR if ever needed.
- **consensus_needed:** "Is the anti-script rule enforced by tooling (signed provenance + loop detector) or by reviewer culture? The hyperscaler answer is culture; the ADR builds tooling."

### ADR-0325 — Capability Tier Pricing Anchors Public
- **decision_atom:** Publish a deterministic public per-tier × per-category MRR price-anchor table (Bronze/Silver/Gold/Platinum × plugin/app/workflow/agent/model/dataset) with multiplicative compliance-pack/residency uplifts, BYOK + prepayment discounts, resolved by a single `resolve_price()` function so any (category, tier, pack, residency, byok, prepayment) tuple self-quotes without sales contact.
- **domain:** finops-cost (cross-cut: marketplace-commerce)
- **current_status:** Proposed
- **disposition:** SUPERSEDE (de facto) → effectively ARCHIVE/AMEND
- **proposed_resolution:** DROP as written — this ADR is built entirely on the **Bronze/Silver/Gold/Platinum capability-tier ladder of ADR-0316**, and ADR-0316 is **retired by ADR-0329** (in this same chunk, Accepted). The tier axis the whole anchor table is keyed on no longer exists; pricing must be re-expressed against the `tenant_class {demo_trial, paid}` + composable `billing_components {revenue_share, per_seat, per_usage}` model (ADR-0330/0331). The *public-self-serve-anchor principle* and the pack/residency uplift + BYOK/prepayment mechanics survive and should be re-authored; the tier-keyed table is dead. So: DROP the tier-table, RATIFY the principle into a tenant-class-keyed successor.
- **governing:** **ADR-0329** (retires the tier ladder this ADR prices) → and ADR-0330 (tenant-class replacement) as the basis for the re-authored pricing.
- **truth_flag:** WRONG (now) — the core artifact (tier price table) prices a retired primitive. Was TRUE at 2026-05-20 authoring; STALE/WRONG one day later after 0329 (2026-05-21).
- **in_masterplan:** PARTIAL — pricing is a real masterplan concern, but the *current form* must not bind into the masterplan because it encodes retired tier vocabulary.
- **tensions:** Direct, dated conflict with ADR-0329 (same chunk): 0329 §A.2.4 explicitly names "we don't have tiers"; 0329 B2.047 says marketplace offers are not tier-gated. 0325's entire D-1..D-14 is tier-gated. Also amends ADR-0316 (the very ADR 0329 retires) and ADR-0314/0249 (marketplace settlement) which inherit the tier residue.
- **hyperscaler_challenge:** QUESTIONABLE→MISALIGNED. Public self-serve anchored pricing (Stripe-class) is ALIGNED and correct. But the steep 4×-per-tier ladder with multiplicative compliance-pack stacking caps is exactly the "we have tiers because competitors have tiers" parity-trap that 0329 §A.2.4 calls invalid; AWS/GCP price by usage/SKU, not by a 4-rung capability ladder. Argues for ARCHIVE-and-re-author against usage/seat billing.
- **ai_slop:** Moderate — plausible-looking but fabricated ARR sensitivity tables ($8.7M deltas), worked KRW examples to the cent, 27-survey citations. Confident invented numbers.
- **refinement:** Re-author as "public self-serve pricing on tenant_class + billing_components" with usage/seat anchors; keep pack/residency/BYOK uplift logic; delete the Bronze..Platinum table.
- **consensus_needed:** "Pricing is per tenant_class + per-usage/seat (ADR-0329/0330) — so does the public price-anchor concept survive at all, or does usage-metered billing make a fixed public anchor table the wrong shape entirely?"

### ADR-0326 — Per-Tenant Data Residency Attestation
- **decision_atom:** Data residency is a first-class tenant attribute with four tiers (`multi_region` / `single_region` / `sovereign_cell` / `airgapped_cell`), each backed by a signed attestation record; cell placement and cross-border data movement are gated by Cedar against the tenant's attested residency footprint, with per-compliance-pack minimum-residency bindings.
- **domain:** compliance-residency (cross-cut: tenancy)
- **current_status:** Proposed
- **disposition:** KEEP (→ AMEND lightly)
- **proposed_resolution:** RATIFY — this is the strongest, most genuinely-needed decision in the chunk; residency-as-tenant-attribute + signed attestation + Cedar cross-border bar is real, hyperscaler-grade, and survives the tier retirement intact (residency tiers are an availability/jurisdiction classification, NOT the retired capability-tier ladder — 0329 explicitly preserves non-capability "tier" vocab, B2.039). Light AMEND only: it carries a `residency` *uplift* coupling to ADR-0325's retired tier-priced table (D-8 cross-ref to 0325) — re-point that to the tenant-class pricing successor; and it composes with ADR-0246 "cellular architecture / Cedar substrate" which the keystone notes is part of the Cedar/policy chain.
- **governing:** n/a (not archived). Pricing cross-ref re-points to ADR-0330 successor.
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL — no explicit `planning_impact:` flag, but residency is a substrate compliance primitive the masterplan should carry; binds into tenancy/compliance domains.
- **tensions:** Only soft: its residency-uplift pricing dependency rides on ADR-0325's tier table (transitively retired by 0329). Composes cleanly with the SOURCE canonical posture (Talos/cell substrate, Cedar gate, Zitadel IdP). No conflict with LINUX pilot ADRs. The `sovereign_cell`/`airgapped_cell` dedicated-control-plane claims are ambitious vs current pilot ground-truth but not wrong as a target.
- **hyperscaler_challenge:** ALIGNED. AWS (Local Zones / GovCloud / sovereign EU), Azure (sovereign cloud, confidential cells), Google (Assured Workloads, data-residency controls) all implement exactly this jurisdiction-pinned-cell + attestation model. The named regime registry (PIPA/DPDP/LGPD/PDPL/PIPL/CSRD/AI-Act) is credible. Argues KEEP.
- **ai_slop:** Low-moderate — regulatory citations are real and accurate (Reg (EU) 2016/679, 2024/1689, KR Act 17347, etc.), which is unusually grounded. Some invented incident IDs but the substance is sound.
- **refinement:** Trim incident fabrications; otherwise solid.
- **consensus_needed:** None major. Possibly: "Is full air-gapped-cell offline-courier ops in scope for the viable-kernel-first goal, or deferred?"

### ADR-0327 — Wave 3 Completion Criteria and Promotion Gates
- **decision_atom:** An ADR moves Drafted→Proposed→Accepted only when named promotion gates G-1..G-10 pass (substance bar, provenance, 16-facet multispectrum sign-off, Cedar lint, audit-class registration, wave-ledger, cross-ref density, amend-chain, tenancy, authority-chain), with ≥4/5 council concurrence, and a wave is "complete" only when every member ADR is Accepted.
- **domain:** governance-process (cross-cut: docs-ssot-masterplan)
- **current_status:** Proposed
- **disposition:** AMEND
- **proposed_resolution:** RATIFY the core (an explicit ADR state machine Drafted/Proposed/Accepted/Superseded/Withdrawn + promotion gates is genuinely the backbone the founder's "ADRs are immutable SSOT, status is derived" model needs — this is the *closest ADR in the chunk to the masterplan-generation doctrine*). Amend heavily: strip `council-foundry`/`axis-foundry`/Foundry-pipeline branding (ADR-0335/0347); drop the ADR-0110/0111/0112/0113 changeset-SM/merge-queue/webhook/VCS-orchestrator dependencies (retired by ADR-0363) and re-express promotion on plain-git + Forgejo Commit Status / oya gate; reconcile its 16-facet "multispectrum" ceremony against what the actual generated-masterplan pipeline (planning-ssot-consolidation.md) requires.
- **governing:** n/a (not archived). AMEND binds to ADR-0363 (VCS retirement), ADR-0335/0347 (foundry brand), and the planning-ssot-consolidation/drift-prevention docs.
- **truth_flag:** PARTIAL — state machine + gates TRUE and load-bearing; the VCS/foundry substrate it gates against is STALE.
- **in_masterplan:** **YES** — carries `planning_impact: true` in front-matter (the only chunk member besides 0329). It directly defines ADR lifecycle, which the generated masterplan depends on. This is the chunk's primary masterplan-binding process ADR.
- **tensions:** (1) Directly relevant to **THE OPEN FOUNDER QUESTION** (keystone §4): 0327 designs status as gate-derived and ADRs as append-only — i.e. it *leans toward the generated-from-ADRs / immutable-SSOT design* (planning-ssot-consolidation.md), against the masterplan-is-authority design. FLAG under both readings. (2) Defines "Wave-3 cluster = 30 ADRs 0297-0327" as the unit of completion — collides with ADR-0323's wave model and the retired Wave/M0-M3 vocab. (3) Self-bootstrapping paradox (the ADR promotes itself) honestly acknowledged but unresolved.
- **hyperscaler_challenge:** QUESTIONABLE. Kubernetes KEP lifecycle (provisional→implementable→implemented) and Rust RFC FCP are the real precedents and ARE aligned with a gated ADR state machine. But 16 distinct reviewer-agent facets each signing an ed25519 verdict + ≥4/5 council quorum is far heavier governance than any hyperscaler applies to a design doc. Argues for AMEND (adopt KEP-lite, drop the facet/quorum ceremony).
- **ai_slop:** High ceremony / moderate slop — fabricated incidents, a 17th "W1" cluster facet invented mid-doc, self-application "proof obligation" section that restates the gates. The state-machine table (D-1) is the load-bearing TRUE core.
- **refinement:** Extract D-1 state machine + G-1..G-10 (slimmed) as the durable ADR; everything else → standards.
- **consensus_needed:** "ADR-0327 hard-codes status-derived-from-gates + append-only ADRs (generated-masterplan model). Is THIS the ratified SSOT design, resolving the open founder question in favor of generated-from-ADRs?" — this is the single most important founder question in the chunk.

### ADR-0328 — Substance Bar as Canonical Sequence and Batch Discipline
- **decision_atom:** All realignment work follows a normative 5-phase canonical build sequence (Phase 0 cloud-substrate → 1 platform → 2 core-capability [absorbs foundry] → 3 comms/collab → 4 distribution+B2B/ERP, Big-8 ordered HR→ERP→CRM→…), audited one-microservice-per-agent on a 5-dimension coherence protocol with 4 (later 3) deliverables, batched ≤8 Codex agents, verified by mandatory artifact-read before `done`.
- **domain:** governance-process (cross-cut: docs-ssot-masterplan)
- **current_status:** Proposed
- **disposition:** AMEND (large)
- **proposed_resolution:** RATIFY the canonical-phase-ordering + audit-before-remediate + read-to-verify spine (these are sound and align with hyperscaler "substrate-before-product" layering, which it correctly cites). But AMEND extensively: it is saturated with now-retired/contradicted detail — (a) it is **explicitly amended by ADR-0329** (same chunk): §D-19 "OCI Bronze = Always Free" reworded to tenant-class, four-deliverable schema (incl. `capability-tier-deltas`) retired to three; (b) `foundry` brand absorption language (D-12) must use ADR-0335 intelligence/governance split; (c) hard-codes **`Codex-only` dispatch** and `omc-teams`/`oya vcs claim/verify/done/promote` (the retired agentic-VCS of ADR-0363) as doctrine — both are operational-runtime couplings that do not belong in an immutable architecture ADR and are partly retired. The `oyatie.foundry.*` principal namespace it preserves is legitimately retained (0329 B2.097 confirms).
- **governing:** **ADR-0329** (amends 0328 §D-19 + retires its 4th deliverable schema); ADR-0335/0347 (foundry brand); ADR-0363 (oya-vcs retirement).
- **truth_flag:** PARTIAL — phase-sequence + audit protocol TRUE; tier deliverable WRONG (retired by 0329 next day); Codex-only/oya-vcs/foundry couplings STALE.
- **in_masterplan:** PARTIAL — no `planning_impact:` flag; it is a realignment-orchestration doctrine, and the literal phase/service roster (cloud-iam, identity, intelligence, …) IS the substrate inventory the masterplan should reflect, but the doc's value as an ADR is the sequencing rule, not the transient roster.
- **tensions:** (1) Self-amended by 0329 within one day — clearest in-chunk supersession churn. (2) Front-matter uses a *different schema* from its siblings (`related_adrs` list, `owner_team`, `line_floor`, `source_anchors` with absolute `/Users/jasonlee/...` paths) — schema drift + leaks the author's local filesystem path into a "canonical" doc (slop/hygiene). (3) Hard-codes a 79/77-microservice roster and Big-8 vendor list as doctrine — high churn risk. (4) "Codex-only, no Claude/Gemini" dispatch baked into an architecture ADR is a category error.
- **hyperscaler_challenge:** PARTIALLY ALIGNED. "Establish identity/network/KMS/storage/compute substrate before product surfaces" is exactly how AWS/GCP/Azure layer — correct and well-argued (D-1 named precedent). But encoding a specific vendor-parity Big-8 ordering + a single-model (Codex-only) agent-dispatch rule as immutable doctrine is something no hyperscaler would freeze into an architecture decision. Argues AMEND.
- **ai_slop:** High volume / structurally bloated — 4397 lines, ~110 numbered D-1.x clauses many of which are single-sentence roster entries ("Phase 0 service 07: cloud-data."), the exact pattern ADR-0322/0324 ban. Decision spine is real but buried.
- **refinement:** Reduce to the 5-phase rule + audit-5-dimensions + verify-by-reading; move the service roster to a registry spec; delete Codex-only and oya-vcs couplings.
- **consensus_needed:** "Is the 5-phase canonical build sequence (substrate→product) a permanent architectural invariant for the masterplan, or a one-time realignment-wave SOP? And does an architecture ADR get to mandate a specific agent model (Codex-only)?"

### ADR-0329 — Tier System Retired; Replaced by Tenant-Class Model
- **decision_atom:** The Bronze/Silver/Gold/Platinum **capability-tier** doctrine (ADR-0316) is retired in full across the corpus (registry, per-service tier-matrices, N-014/N-015 naming forms, tier-gated standards clauses), replaced by the binary `tenant_class {demo_trial, paid}` + composable `billing_components {revenue_share, per_seat, per_usage}` (defined by ADR-0330, adopted per-service by ADR-0331); non-capability "tier" vocabularies (ADR-0248 cellular criticality, ADR-0037 API-stability, ADR-0083 Rust error-handling) are explicitly preserved.
- **domain:** tenancy (cross-cut: finops-cost)
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** n/a (already Accepted) — this is the chunk keystone and is correct, current, and canonical. It is the **governing ADR** that forces AMEND/ARCHIVE on 0325 (tier pricing) and AMEND on 0328 (§D-19, deliverable schema) within this very chunk.
- **governing:** This ADR IS the governor. It supersedes **ADR-0316** (front-matter `supersedes: [ADR-0316]`, status Accepted) and amends 0328/0064/0249/0251/0255. Matches keystone §1.2 / §2 exactly (tier → tenant-class, ADR-0329 keystone).
- **truth_flag:** TRUE.
- **in_masterplan:** **YES** — carries `planning_impact: true`; it is a binding canonical decision (the tenant-class model is in the keystone's canonical-posture tenancy row). Defines vocabulary the masterplan must use and forbid.
- **tensions:** (1) Forward-references ADR-0330/0331 which must exist and be Accepted for the replacement to be complete — auditor should confirm 0330/0331 landed (outside this chunk; flagged for the chunk owner). (2) Cross-chunk: it retires the basis of ADR-0325 (this chunk) and ADR-0316; any downstream ADR still tier-keyed is now stale-by-0329. (3) Preserves `oyatie.foundry.*` principal namespace (B2.097) even though the **foundry brand** is retired by 0335 — consistent (namespace ≠ brand) but a reviewer must not "fix" the principal namespace during foundry cleanup. (4) Front-matter schema drift like 0328 (`owner_team` list, `line_floor`, absolute local paths in `enforced_by`/`source_anchors`).
- **hyperscaler_challenge:** ALIGNED. Retiring a 4-rung feature-gating capability ladder in favor of (trial vs paid) + usage/seat/rev-share billing is exactly the modern hyperscaler/SaaS posture (AWS/GCP: same features, meter by usage; trial vs paid account, not Bronze/Gold feature tiers). The explicit "we don't have tiers; tiers were a parity-trap" reasoning (§A.2.4) is the right call. Strongly argues KEEP.
- **ai_slop:** Low on decision / high on volume — 2570 lines, 100 B2.0xx clauses many of which are "the retirement does not affect X" boilerplate (B2.040–B2.100). Substance is real and the clause-by-clause preservation list is arguably warranted for a corpus-wide retirement, but it is heavily padded.
- **refinement:** The decision + the surviving-vocabulary allow-list (§D-1.7 / B2.036-039) + the supersession of 0316 are the durable core; the 60+ "does not affect" clauses could compress.
- **consensus_needed:** None — Accepted and correct. Only operational confirm: "Are ADR-0330 and ADR-0331 landed/Accepted so the tenant-class replacement is actually wired, or is the corpus mid-retirement with no live replacement?"

---

## Chunk notes

**One TRUE keystone, one self-inflicted supersession, five process/product Proposals.**

1. **ADR-0329 is the load-bearing decision** and the only Accepted ADR in the chunk. It is correct, hyperscaler-aligned, masterplan-binding (`planning_impact: true`), and it governs two of its own chunk-mates: it **retires the tier ladder that ADR-0325's entire pricing table is keyed on**, and it **amends ADR-0328** (§D-19 + deliverable schema). The dating is the tell — 0323–0328 authored 2026-05-20, 0329 Accepted 2026-05-21: the cluster priced and sequenced against a primitive that was retired the next day. **ADR-0325 is the casualty: DROP/re-author against tenant-class (ADR-0330).**

2. **0322–0327 are a governance-PROCESS meta-cluster, not architecture decisions.** They describe *how ADRs and docs get authored, batched, and promoted* (substance bar, waves, anti-script, pricing-publication-gate, residency-rollout-gate, promotion gates). Under the founder's "if it isn't a live ADR feeding the generated masterplan, it isn't needed" test, most of these are authoring-SOP that belong in `docs/standards/`, not as immutable substance ADRs — **except ADR-0327**, which defines the ADR state machine itself and therefore *is* load-bearing for the masterplan-generation pipeline (and carries `planning_impact: true`).

3. **ADR-0327 is the chunk's tie to THE OPEN FOUNDER QUESTION (keystone §4).** It hard-codes "status is derived from gate output, ADRs are append-only/immutable" — i.e. it embodies the **generated-from-ADRs** design (planning-ssot-consolidation.md), opposite the masterplan-is-authority design. Whoever resolves the founder question should treat 0327 as a vote already cast for generated-from-ADRs; flag it under both readings as instructed.

4. **Pervasive retired-vocabulary leakage drives most dispositions to AMEND.** `foundry`/`council-foundry`/`axis-foundry`/`oya-foundry-*`/Foundry-pipeline appears as the enforcement substrate in 0323/0324/0327/0328 (retired → intelligence/governance per ADR-0335/0347/0363). The retired **agentic-VCS** primitives (ADR-0110/0111/0112/0113 changeset-SM, merge-queue, webhook, VCS-orchestrator; and `oya vcs claim/verify/done/promote` in 0328) are cited as live substrate but are **superseded by ADR-0363**. The retired **Wave/M0-M3** milestone vocabulary is the native unit in 0323/0327. None of these invalidate the underlying decisions, but every Proposed member needs a vocabulary/substrate AMEND before it could feed a clean masterplan.

5. **Two strong KEEPs:** ADR-0326 (residency attestation) — genuinely hyperscaler-grade, real regulatory grounding, survives tier retirement cleanly; light AMEND only to re-point its pricing-uplift cross-ref off the retired 0325. And ADR-0329 itself.

6. **No conflicts with LINUX pilot ADRs (0001-0026) in this chunk** — it is all SOURCE-side tenancy/pricing/compliance/process; no fault-line overlap with the framekernel/own-DB/own-policy tensions of the keystone §5.

7. **Front-matter hygiene flag:** ADR-0328 and ADR-0329 use a divergent front-matter schema (`owner_team`/`related_adrs`/`line_floor`/`source_anchors`) from their 0323-0327 siblings, and both leak absolute author-local paths (`/Users/jasonlee/oyatie/...`) into "canonical" `enforced_by`/`source_anchors` fields — a portability/hygiene defect for any merge or masterplan generation.

8. **No "plain garbage" ADRs in range.** Worst truth_flag is ADR-0325 = WRONG (prices a primitive retired one day later). Heaviest AI-slop/padding is ADR-0328 (4397 lines, roster-as-clauses) and ADR-0329 (2570 lines, "does-not-affect" boilerplate) — but in both the decision spine is true; the issue is volume, not falsity.
