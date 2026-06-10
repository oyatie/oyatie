# ADR Audit Artifact — source-41

- **side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **chunk:** source-41
- **range:** lines 281–287 of `ls -1 docs/decisions/ADR-*.md | sort` → **ADR-0344 … ADR-0350** (7 ADRs)
- **ADRs-reviewed:** ADR-0344, ADR-0345, ADR-0346, ADR-0347, ADR-0348, ADR-0349, ADR-0350
- **baseline:** keystone `canonical-posture-and-supersession-map.md` (read in full); masterplan authored-vs-generated treated as OPEN (flag both readings).

---

### ADR-0344 — Sustainability + finops dimensional model (per-call CO2-grams + watt-hours + USD-cost on every audit row; finops-portal six-axis rollup)

- **decision_atom:** Every audit-chain row emitted under ADR-0263 carries, at the same HLC tick, a five-field sustainability/finops tuple `{cost_usd_minor_units, co2_grams, watt_hours, provider, region}` computed from a per-µservice `sustainability_emission_model`, and finops-portal owns a six-axis (tenant/product/capability/provider/cell/compliance-pack) cost+carbon+energy rollup with regulator-export (CSRD/SB-253/SEC) evidence packs.
- **domain:** finops-cost (cross-cutting: observability).
- **current_status:** Proposed.
- **disposition:** AMEND (sound decision; needs naming/ref hygiene before ratify — see truth_flag).
- **proposed_resolution:** RATIFY with amendments — the per-call activity-based carbon/cost contract is a genuine, non-duplicated decision that masterplan needs; but ratify only after the carbon-aware-scheduler deferral logic is re-pinned to the live CI/eventing posture and the Iceberg/ClickHouse OLAP refs are reconciled (ADR-0337 in same batch).
- **governing:** n/a (not archived).
- **truth_flag:** PARTIAL — core contract TRUE, but carries stale couplings: it leans on ADR-0338 pod-runtime-tier + ADR-0341/0343 which are themselves Proposed siblings, and electricityMaps is asserted "canonical" without a stewardship-registry entry (the very registry ADR-0345 mandates) — a cross-ADR gap in the same batch.
- **in_masterplan:** PARTIAL — sustainability tag (ADR-0174) is represented; per-call five-field carbon/cost tuple + six-axis rollup are NOT yet a masterplan line.
- **tensions:** (1) electricityMaps third-party dependency is introduced as "canonical" but is a Consumer/Contributor-class OSS surface unrecorded in ADR-0345's `/specs/oss-stewardship-registry.json`. (2) ~40 bytes/row × 10^6 rows/s = ~3.5 TB/day claim depends on audit-retention shortening not yet decided. (3) carbon-aware-scheduler overlaps ADR-0348 auto-rebalance/dynamic-sharding control plane — two ADRs both add control-plane workload-placement logic without a single owning surface.
- **hyperscaler_challenge:** ALIGNED. AWS CCFT / Google Cloud Carbon Footprint / Azure Emissions Impact / Snowflake Carbon Insights all ship per-tenant carbon dashboards; per-call (vs 90-day-lag) attribution is a defensible *exceeds-baseline* posture. A hyperscaler would make this decision — but would NOT hard-bind a single external carbon provider as "canonical" without a vendor-risk fallback contract (they self-publish grid factors). Amend implication: keep the decision, soften "electricityMaps canonical" to "electricityMaps default + self-published fallback," and register it under ADR-0345.
- **ai_slop:** Low-moderate. 861 lines, B2.001–B2.040 + D-1…D-11 + 15 anchors is the house substance-bar style; content is real but the line_floor:600 + "forty numbered clauses" cadence inflates. `feedback_*` memory citations are unverifiable from the corpus alone.
- **refinement:** Collapse the 40 clauses to the ~8 load-bearing invariants for masterplan; drop the per-anchor restating of every sibling ADR.
- **consensus_needed:** Founder question — "Do we accept a third-party carbon-intensity API (electricityMaps) as a runtime dependency in the audit-emission hot path, or do we self-publish grid factors like the hyperscalers and keep the audit path dependency-free?"

---

### ADR-0345 — OSS stewardship class policy + CVE-response SLA (Maintainer / Contributor / Consumer; 7d P0 + 30d P1)

- **decision_atom:** Every direct upstream OSS dependency is classified into exactly one of three stewardship classes — Maintainer (Oyatie owns commit+release), Contributor (active patch + 7-day-P0/30-day-P1 CVE SLA + dev-days/quarter), Consumer (pin+audit, 14-day pin-update SLA) — enumerated with resourcing, license, owner-team and ADR-provenance in the canonical registry `/specs/oss-stewardship-registry.json`.
- **domain:** security-supplychain (cross-cutting: governance-process).
- **current_status:** Proposed.
- **disposition:** KEEP (ratify) — sound, standalone, low-drift; this is keystone supply-chain doctrine other ADRs (0349 B2.013, 0350 D-3) already cite as live.
- **proposed_resolution:** RATIFY — three-class model + per-class CVE SLA + machine-readable registry is a true decision with no live conflict; downstream ADRs already bind to it, so leaving it Proposed creates an unaccounted dangling dependency.
- **governing:** n/a.
- **truth_flag:** TRUE. Terminology discipline (reserve "tier" for cellular/pod-runtime; "stewardship class" is a relationship label) is internally consistent with ADR-0329 retired-tier vocabulary and ADR-0338.
- **in_masterplan:** NO — supply-chain stewardship posture is not yet a masterplan line; should become one (it is the audit-grade SOC2/ISO-27001 answer).
- **tensions:** (1) Floor enumeration lists **Apache Kafka** as Contributor/Consumer (B2.005/B2.008) and **Redis-substitute Valkey** correctly — but Kafka is RETIRED standalone per ADR-0377-kafka-to-pulsar (keystone §2); the registry seed must read Pulsar+Oxia, not Kafka. (2) Lists "Cilium / Istio Ambient / Cedar / Kyverno / OpenSearch" as Contributor — Kyverno is downgraded by ADR-0379 (Kubewarden default admission) and the Istio mesh / Cilium choices are in the orchestration churn zone; registry seeds risk citing superseded substrate.
- **hyperscaler_challenge:** ALIGNED. AWS/Google/Microsoft/Meta/Netflix/Apple all publish per-upstream stewardship commitments; a hyperscaler would absolutely make this decision. No amend implication on the doctrine; only the seed enumeration needs de-staleing (Kafka→Pulsar, Kyverno-status).
- **ai_slop:** Moderate. The hyperscaler-precedent block (A.2) and 40-clause cadence are padded; the core (three classes, SLA windows, registry schema D-2) is crisp and real.
- **refinement:** Strip the Kafka/Kyverno seed names from the ADR body and defer the concrete enumeration to the registry artifact so the doctrine ADR does not embed retired substrate.
- **consensus_needed:** None on doctrine. Minor: "is Kafka Contributor or Consumer" is moot post-Pulsar — resolve by deleting the Kafka clause.

---

### ADR-0346 — `oya verify --ci-required` MUST locally mirror the full CI matrix (fmt+check+clippy+nextest+gate, block on exit-0 of each)

- **decision_atom:** The canonical local pre-push verifier `oya verify --ci-required` must run the full CI mirror — `cargo fmt --check` + `cargo check` + `cargo clippy -D warnings` + `cargo nextest` + `oya gate run-all` (5 mandatory, surface-all-failures, closed skip-flag allowlist, closed exit-code enum 0/1/2) and block on exit-0 of each step before returning success.
- **domain:** ci-cd-build (cross-cutting: governance-process).
- **current_status:** Proposed.
- **disposition:** AMEND — sound decision, but the named evidence is steeped in RETIRED vocabulary that must be scrubbed before ratify.
- **proposed_resolution:** RATIFY-after-amend — "the local verifier must be a faithful CI mirror" is a true, useful, low-controversy engineering invariant; but the motivating PR #177 incident enumerates `oya-foundry-fitness-aspirational-enforcement` + `oya-foundry-fitness-honest-claims` as two of the seven failures (purpose block + A.1.6/A.1.7), i.e. it cites the exact `oya-foundry-fitness-*` prefix that ADR-0347 (its own sibling) retires to `oya-governance-*`. Amend the lane names before ratify.
- **governing:** n/a.
- **truth_flag:** STALE — decision TRUE, citations WRONG-vocab: uses `oya-foundry-fitness-*` lane names that the same-batch ADR-0347 declares anachronistic; also asserts a generic "CI matrix at `.github/workflows/pr-tests.yml`" which the keystone shows is being replaced (GitHub Actions → Jenkins → Argo Workflows per ADR-0359/0511). The *local-mirror principle* survives any CI-backend change; the *named backend* is already stale.
- **in_masterplan:** NO — verifier-completeness is an implementation-discipline rule; arguably belongs in dev-tooling doctrine rather than the masterplan proper.
- **tensions:** (1) Hard-codes GitHub Actions as the CI matrix to mirror, directly inside the forge/CI churn zone (founder migration = GitHub; source canon moving to Argo Workflows). The verifier should mirror "the canonical CI lane set" abstractly, not `.github/workflows/pr-tests.yml` literally. (2) Self-referential with ADR-0347: both are 2026-05-21 siblings; one retires the prefix the other cites.
- **hyperscaler_challenge:** ALIGNED. Google presubmit / Meta arc presubmit / Amazon brazil-build all ship local CI mirrors; a hyperscaler makes this decision. Amend implication: decouple the verifier from a specific CI backend so the decision survives the GitHub-Actions→Argo-Workflows migration.
- **ai_slop:** Moderate. Genuinely useful decision; the 12-anchor + 40-clause + D-1…D-11 byte-for-byte command enumeration is over-specified for an ADR (belongs in the spec file it itself proposes, `/specs/oya-verify-ci-mirror.json`).
- **refinement:** Replace literal `.github/workflows/pr-tests.yml` references with "the canonical CI lane registry"; rename the two foundry-fitness lanes in the evidence narrative.
- **consensus_needed:** None on principle. Resolve mechanically by scrubbing retired-vocab.

---

### ADR-0347 — Foundry-fitness → governance bulk rename (doctrine-only; all `oya-foundry-fitness-*` → `oya-governance-*`)

- **decision_atom:** Every `oya-foundry-fitness-*` CI-lane/crate/catalog/ADR-citation identifier collapses to `oya-governance-*` via a single deterministic 1:1 bulk-rename PR (Wave 15-ZB) rather than 34 per-lane migration IPs, with a machine-readable pre-rename inventory and three residue/vocabulary/inventory enforcement lanes.
- **domain:** governance-process (cross-cutting: ci-cd-build).
- **current_status:** Proposed.
- **disposition:** KEEP (ratify) — this is the *implementing* ADR of the keystone's retired-vocabulary row ("`oya-foundry-fitness-*` → `oya-governance-*`", keystone §2 L97); it is the canonical executor of a decision already declared TRUE.
- **proposed_resolution:** RATIFY — the rename is mandated by ADR-0132 + ADR-0335 (foundry retired). Leaving it Proposed is precisely why ADR-0346 (its sibling) still cites the dead prefix. Ratify to unblock corpus-wide vocab hygiene.
- **governing:** n/a (it is doctrine that *enables* archival of the foundry-fitness prefix; it does not itself archive an ADR).
- **truth_flag:** TRUE — directly downstream of ADR-0335 (foundry RETIRED, keystone §1.2/§2) and ADR-0132; the 1:1 determinism argument is sound.
- **in_masterplan:** NA — a mechanical rename, not a masterplan-level architecture decision; the *outcome* (governance is the canonical lane prefix) is the masterplan-relevant fact, already captured.
- **tensions:** (1) `related_adrs` lists `ADR-0346-product-readiness-checklist.md` and references "ADR-0346 (product readiness checklist)" in its sibling narrative (¶ "It runs in coordination…") — but on-disk ADR-0346 is **oya-verify-full-ci-mirror**, NOT product-readiness. Title/number drift inside the same batch (the six-candidate authoring renumbered and the cross-refs were not reconciled). (2) Self-allowlists its own historical-context paragraphs in the residue lane — acceptable but means the ADR is partly self-exempting.
- **hyperscaler_challenge:** ALIGNED. AWS/Google/Microsoft bundle deprecation renames into atomic bulk PRs (GCR→Artifact Registry, etc.); a hyperscaler makes this exact call. No amend implication on doctrine; fix the ADR-0346 cross-ref drift.
- **ai_slop:** Low-moderate. The decision is genuinely mechanical and the bulk-vs-per-IP rationale is the substantive content; the hyperscaler-precedent + 7-surface enumeration is appropriately scoped (it is a rename inventory). The ADR-0346 mis-citation is a real drift bug, not slop.
- **refinement:** Fix the `ADR-0346-product-readiness-checklist.md` → `ADR-0346-oya-verify-must-run-full-ci-mirror.md` cross-reference.
- **consensus_needed:** None. Mechanical.

---

### ADR-0348 — Autosharding + auto-rebalance + dynamic sharding (three control-plane automation modes; manifest-declared; reversible; audit-emit)

- **decision_atom:** Cellular topology must support three control-plane-driven (never operator-driven) automation modes — autosharding (tenant→cell/shard placement from capacity-model + compliance-pack + residency + tier + shuffle-sharding), auto-rebalance (hot-cell→cool-cell tenant migration honoring residency/compliance, cross-jurisdiction requires Cedar permit), and dynamic sharding (hot-split/cold-merge on per-µservice thresholds) — each manifest-declared, reversible, and audit-chain-emitting per ADR-0263.
- **domain:** orchestration-scheduling (cross-cutting: tenancy).
- **current_status:** Proposed.
- **disposition:** AMEND — the decision is sound, but the ownership rationale is internally self-superseded and must be reconciled before ratify.
- **proposed_resolution:** RATIFY-after-amend — the three-mode automation contract is a true hyperscaler-grade scalability decision; but the ADR carries a **2026-05-21 amendment-in-place** (line 145 + line 151) stating its own original "cell-orchestrator composed across tenancy+observability" rationale is **superseded by ADR-0351** (dedicated `cell-rebalancer` + `cell-lifecycle` µservices). The ADR thus contradicts itself in-body; ratify only the corrected (ADR-0351) ownership and demote the "tenancy+observability" wording to explicit historical record.
- **governing:** ADR-0351 (governs the ownership portion only — the cell-rebalancer/cell-lifecycle µservices own the rebalance modes; ADR-0348's automation *contract* survives).
- **truth_flag:** PARTIAL — automation contract TRUE; ownership rationale STALE/self-superseded by ADR-0351 (which is in the next chunk, outside this slice, but cited on-disk). The "composed across tenancy + observability — NOT a new µservice" claim is explicitly reversed by the inline amendment that reintroduces dedicated µservices, partially re-contradicting ADR-0333 (cell-as-pattern-not-service).
- **in_masterplan:** PARTIAL — cellular/shuffle-sharding baseline (ADR-0248) is represented; the three automation modes + reversibility + Cedar-gated cross-jurisdiction migration are not yet masterplan lines.
- **tensions:** (1) Self-supersession: ADR-0333 said "cell is a pattern, not a service"; ADR-0348 honored that ("logical responsibility composed across tenancy+observability"); ADR-0351 then re-creates `cell-rebalancer` + `cell-lifecycle` as µservices — a partial walk-back of ADR-0333 worth founder attention. (2) Overlaps ADR-0344 carbon-aware-scheduler (both are control-plane workload-placement logic). (3) Default thresholds declared *and then rejected* (E.4 rejects default-fill) — a deliberate but unusual "defaults exist only to be overridden" pattern.
- **hyperscaler_challenge:** ALIGNED. AWS S3 Cell SHIELD / Spanner auto-resharding / DynamoDB adaptive capacity / Cosmos auto-repartitioning are all control-plane-driven; a hyperscaler makes this decision. Questionable sub-point: the rapid tenancy+observability→dedicated-µservice ownership flip-flop is the kind of churn a hyperscaler would resolve once; amend implication: lock ownership to ADR-0351 and stop oscillating against ADR-0333.
- **ai_slop:** Moderate. 1020+ lines, 20 anchors, B2.001–B2.034, E.1–E.6; the named-precedent block is real. The in-place amendment is good honesty but signals the doctrine was authored before its own ownership was settled.
- **refinement:** Remove the original "tenancy+observability" ownership prose from the Decision body (keep only as a dated historical note); state cell-rebalancer/cell-lifecycle ownership once.
- **consensus_needed:** Founder question — "Does the cellular-automation control plane live in dedicated `cell-rebalancer`/`cell-lifecycle` µservices (ADR-0351) or stay a composed responsibility (ADR-0333/0348)? The corpus has decided both within one day."

---

### ADR-0349 — Jenkins (LTS) + ArgoCD canonical self-hostable CI/CD substrate

- **decision_atom:** Jenkins (LTS) is the canonical self-hostable CI orchestrator for air-gap/on-prem/colo/oyatie-as-provider contexts (GitHub Actions retained for hosted PR review) and ArgoCD is the canonical GitOps CD orchestrator replacing manual kubectl/Helm-CLI deploys, both Class-C OSS + Contributor-stewardship, provisioned as per-context OpenTofu modules with cosign-verified, audit-emitting, Cedar-gated sync.
- **domain:** ci-cd-build (cross-cutting: node-os/orchestration).
- **current_status:** Proposed (on disk) — **but superseded-in-fact** (keystone §1.1/§1.3/§3: Jenkins is now *transitory bootstrap only*; Argo Workflows is the destination CI per ADR-0511; oya-ci bespoke-Rust Prow per ADR-0513).
- **disposition:** ARCHIVE (the Jenkins-as-canonical-CI half) / AMEND (the ArgoCD-CD half survives).
- **proposed_resolution:** DROP the Jenkins-as-canonical-CI decision; the keystone CI/CD churn chain (ADR-0349 *augment* → ADR-0359 *replace* → ADR-0408 Buck2 → ADR-0511 Argo Workflows destination → ADR-0513 oya-ci) has already moved past it. The ADR is the *first link* in that chain and is the very ADR the keystone names as the start of the churn. ArgoCD/Argo-Rollouts as canonical CD *does* survive (keystone §3 CI/CD row), so split: archive the CI claim, retain CD.
- **governing:** ADR-0511 (Argo Workflows = destination CI; Jenkins transitory) + ADR-0513 (oya-ci) for the CI half; ADR-0392/0408 (Buck2 build) upstream. CD half remains live.
- **truth_flag:** STALE/WRONG-in-part — "Jenkins is canonical CI" is WRONG vs current truth; "ArgoCD is canonical CD + cosign-verify + audit-emit on sync" is TRUE and consistent with keystone §3. The ADR's own front-matter (`Proposed`) is itself stale: keystone §1.1 lists the Jenkins line as `Superseded`.
- **in_masterplan:** PARTIAL/NO — masterplan note (keystone §4) explicitly flags the MASTERPLAN.md "Jenkins required checks" line as STALE (now Argo Workflows). So Jenkins is *represented but wrong* in masterplan; ArgoCD CD is correct.
- **tensions:** (1) Direct member of the keystone CI/CD churn chain — the longest supersession chain in the corpus. (2) Pins Cilium/Istio/Kata/Cloud-Hypervisor substrate refs that sit in the contested orchestration zone (source Talos+CAPI per ADR-0375). (3) Hard-codes GitHub Actions as "retained for hosted PR review" — collides with founder GitHub-migration directive *and* the Forgejo-transitory canon (keystone §5). (4) Lists Jenkins as Contributor-class in ADR-0345's registry — that registry seed becomes dead on Jenkins retirement.
- **hyperscaler_challenge:** MISALIGNED (CI) / ALIGNED (CD). A hyperscaler would NOT pick Jenkins as a go-forward canonical CI in 2026 (Google/Meta/Amazon run bespoke Prow/Piper-class systems — exactly where ADR-0513 oya-ci lands); Jenkins is a defensible *bootstrap*, not a destination, which is precisely how the keystone reframes it. ArgoCD-as-CD is hyperscaler-typical (RedHat GitOps, IBM Cloud Pak). Archive implication: keep ArgoCD, retire Jenkins-as-canonical.
- **ai_slop:** Moderate-high. 1020 lines, 20 anchors, B2.001–B2.025+, 12 companion module paths, §F four-forbidden-alternatives — heavy substance-bar styling on a decision the corpus reversed within the same wave window.
- **refinement:** Split into (a) ARCHIVED Jenkins-CI note pointing to ADR-0511/0513, and (b) a retained ArgoCD-CD invariant (cosign-verify + audit-emit + Cedar-gate on sync) that masterplan keeps.
- **consensus_needed:** Already resolved by keystone — surface only: "Confirm Jenkins is bootstrap-only and the destination CI is Argo Workflows + oya-ci, so the masterplan's 'Jenkins required checks' line is corrected."

---

### ADR-0350 — UUIDv7 canonical ID primitive across Oyatie

- **decision_atom:** UUIDv7 (RFC 9562) is the single canonical ID primitive for every ID surface (event/audit/changeset/tenant/cell/principal/resource/request/idempotency/evidence), generated via `Uuid::now_v7()`, stored as Postgres native `UUID` / SQLite `TEXT`, validated through per-domain newtypes that reject non-v7; ULID/Snowflake/KSUID/UUIDv4/hybrid are rejected; a `id_strategy:"uuidv7"` manifest enum and six governance lanes enforce it; ULID corpus scrub deferred to Wave 15-ZH.
- **domain:** api-contracts (cross-cutting: data-storage).
- **current_status:** **Accepted** (the only Accepted ADR in this chunk).
- **disposition:** KEEP.
- **proposed_resolution:** n/a (already Accepted) — confirm KEEP; this is a clean, masterplan-ready primitive with explicit acceptance-criteria checklist and Cedar PDP fragment.
- **governing:** n/a (it is the governing ID ADR; it *amends* ADR-0003/0005/0113/0214/0292/0252).
- **truth_flag:** TRUE — internally consistent, RFC-anchored, cellular-independence rationale (no central allocator) aligns with ADR-0248; correctly disambiguates "Snowflake ID algorithm (rejected)" vs "Snowflake product references in ADR-0214 (retained)".
- **in_masterplan:** PARTIAL → should be YES — the ID primitive is exactly the kind of cross-cutting substrate decision the masterplan should carry; not yet a masterplan line but Accepted and stable, so promote.
- **tensions:** (1) `amends: ADR-0005` (eventing-outbox) and Cross-References say "the outbox pattern and **Kafka** backbone remain unchanged" (¶ ADR-0005) — but Kafka is RETIRED → Pulsar+Oxia per ADR-0377 (keystone §2). The ID decision is correct; the Kafka wording is retired-vocab leakage. (2) Amends ADR-0252 idempotency-key grammar (narrows to UUIDv7) — clean. (3) `related_adrs` cites ADR-0348/0349/0351 (this chunk's siblings + the cell-rebalancer ADR) — consistent.
- **hyperscaler_challenge:** ALIGNED. UUID-as-portable-ID is hyperscaler-universal (Postgres native UUID, Spanner, Stripe/GitHub-class APIs); UUIDv7 specifically is the current best-practice choice over ULID/Snowflake for decentralized + temporally-local IDs. A hyperscaler makes exactly this decision. No amend implication beyond the Kafka wording.
- **ai_slop:** Low. 1020 lines but the one-sentence-per-line style is deliberate and unusually scannable; acceptance-criteria checklist + Cedar fragment + per-alternative rejection are substantive, not padding. Strongest-authored ADR in the chunk.
- **refinement:** Replace "Kafka backbone remains unchanged" with "Pulsar/Oxia eventing backbone (per ADR-0377) remains unchanged" — one-line retired-vocab fix.
- **consensus_needed:** None — Accepted, sound, ratify into masterplan.

---

## Chunk notes

**Shape of the chunk.** ADR-0344–0349 are six of the 2026-05-21 "realignment-wave" siblings (the six-candidate `/idea-refine` batch authored in one session); ADR-0350 is a same-day but independently-Accepted primitive. Only **ADR-0350 is Accepted**; ADR-0344/0345/0346/0347/0348/0349 are all **Proposed**. Per the founder's "no unaccounted proposals" rule, five of the six Proposed ADRs are sound-enough to RATIFY (0344 amend, 0345 keep, 0346 amend, 0347 keep, 0348 amend); **ADR-0349 is the one to DROP/ARCHIVE** (Jenkins-as-canonical-CI half) — it is superseded-in-fact by the keystone CI/CD churn chain even though its front-matter still reads Proposed.

**The load-bearing finding — ADR-0349 stale front-matter.** ADR-0349 reads `status: Proposed` on disk but the keystone (§1.1, §3, §4) establishes it as the *first link* of the long CI/CD supersession chain (0349→0359→0408→0511→0513→0514) whose net current truth is Buck2 + Argo Workflows + oya-ci, with **Jenkins transitory bootstrap only**. Auditors must trust the superseding ADRs over ADR-0349's stale Proposed status. The ArgoCD-CD half survives; the Jenkins-CI half does not. The masterplan's "Jenkins required checks" line (keystone §4) is the visible damage and should be corrected to Argo Workflows.

**Cross-ADR drift inside the batch (3 concrete bugs found).**
1. **ADR-0347 ↔ ADR-0346 number/title drift:** ADR-0347 (and ADR-0348/0349) cite "ADR-0346 (product readiness checklist)" in their sibling narratives and `related_adrs`, but on-disk ADR-0346 is **oya-verify-full-ci-mirror**. The six-candidate batch was renumbered (a "product readiness checklist" slot existed and was reassigned) and the cross-refs were never reconciled. ADR-0349's own `related_adrs` correctly lists `ADR-0346-oya-verify-must-run-full-ci-mirror.md`, so the corpus is internally inconsistent about what ADR-0346 *is*.
2. **ADR-0346 cites the dead `oya-foundry-fitness-*` prefix** that its own same-batch sibling ADR-0347 retires — a self-inflicted retired-vocab leak.
3. **Retired-vocab leakage on Kafka in two ADRs:** ADR-0345 floor-enumerates Kafka as a stewardship-registry seed; ADR-0350 says "the Kafka backbone remains unchanged." Both should read Pulsar+Oxia per ADR-0377 (keystone §2). One-line fixes each.

**Self-supersession in ADR-0348.** ADR-0348 carries an in-place 2026-05-21 amendment declaring its own ownership rationale ("composed across tenancy+observability, NOT a new µservice," honoring ADR-0333) **superseded by ADR-0351**, which re-creates `cell-rebalancer` + `cell-lifecycle` as dedicated µservices. This is a genuine partial walk-back of ADR-0333 (cell-as-pattern-not-service) and is the sharpest unresolved founder question in the chunk: *does cellular automation live in dedicated µservices or as a composed responsibility?* The corpus decided both within one day.

**Hyperscaler verdict across the chunk.** Five of seven are cleanly ALIGNED (0344 carbon, 0345 stewardship, 0346 local-CI-mirror, 0347 bulk-rename, 0350 UUIDv7). ADR-0348 is aligned on substance but shows ownership churn a hyperscaler would resolve once. **ADR-0349 is the lone MISALIGNED** on the Jenkins-as-destination-CI claim — no 2026 hyperscaler picks Jenkins as a go-forward canonical CI; the corpus itself agrees (oya-ci is the Prow-class destination). ArgoCD-CD within 0349 is aligned.

**masterplan-as-authority vs generated-from-ADRs (kept OPEN per keystone §4).** Under *generated-from-ADRs*: only ADR-0350 (Accepted) auto-projects today; the five RATIFY-able Proposed ADRs must reach Accepted/`verified_by` before they appear, and ADR-0349's Jenkins half must be marked superseded so it does NOT generate into the plan. Under *masterplan-is-authority*: the founder backfills 0345 (supply-chain stewardship), 0350 (UUIDv7), the ArgoCD-CD invariant from 0349, and the carbon/cost contract from 0344 as true+relevant lines, while explicitly excluding Jenkins-CI and the dead `oya-foundry-fitness-*` prefix. Both readings converge on the same exclusions (Jenkins-CI, foundry-fitness prefix, Kafka) — so those are safe to treat as "not needed" regardless of which SSOT direction the founder picks.

**No ADRs in range were edited; only this artifact was written.**
