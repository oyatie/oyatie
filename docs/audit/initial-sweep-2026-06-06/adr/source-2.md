# ADR Audit — SOURCE chunk source-2

- **Side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`, 346 ADRs in `docs/decisions/`)
- **Chunk id:** source-2
- **Slice:** `ls | sort | sed -n "8,14p"` (lines 8–14 of the sorted ADR listing)
- **ADR range:** ADR-0008 … ADR-0015 (note: ADR-0012 is NOT in this directory range — the sorted slice jumps 0011→0013; ADR-0012 file is absent from the 8–14 window)
- **ADRs reviewed (7):** ADR-0008, ADR-0009, ADR-0010, ADR-0011, ADR-0013, ADR-0014, ADR-0015
- **Baseline:** keystone map `_map/canonical-posture-and-supersession-map.md` (read in full)
- **Posture:** READ-ONLY. Only this artifact was written.

---

### ADR-0008 — Data Use Boundary (twelve data classes, orthogonal subject_class, purpose-permission matrix, four-pillar flow matrix)

- **decision_atom:** Tenant data flow across all axes is governed by a compile-time-first Data Use Boundary: a closed 12-class data taxonomy with HARD_DENY (no consent override) for PHI/PCI/FINANCIAL_KR/SENSITIVE_PIPA-Art23, an orthogonal `subject_class` attribute (minor/elderly/vulnerable, not a 13th class), a purpose-permission matrix (not a linear consent ladder), a four-pillar Org/Person/Public/Opt-in-Consumer flow matrix, and an inference-boundary rule where derived attributes inherit the most-restrictive ancestor class — all enforced across six structural layers with per-decision audit emission.
- **domain:** compliance-residency (primary); authz-policy (secondary — purpose-permission grants and runtime guard are policy decisions).
- **current_status:** Accepted (front-matter `accepted`; body "accepted 2026-05-14").
- **disposition:** KEEP — current, correct, structurally central; this is the privacy keystone the regional-pack (0010) and tenant-class (0008 §5) machinery hang off. Minor AMEND-flavored residue only (see truth_flag).
- **proposed_resolution:** NA (already Accepted).
- **governing:** —
- **truth_flag:** TRUE, with PARTIAL staleness — (a) §6.3 cites `oya-ads-gate`/`oya-analytics-router` as the only ads/analytics publishers and §1–6 reference `oya-governance-*` lanes (correctly the live prefix per keystone §2); (b) "Foundry agents inherit the contract automatically (ADR-0007)" in Consequences uses retired **foundry** vocabulary (now intelligence per ADR-0335/0347) — naming drift, not a logic error; (c) §6.3 cites ADR-0005 eventing layer which is the retired-Kafka ADR (superseded by ADR-0377 Pulsar) — the partition/topic-gate principle survives, the broker name is stale.
- **in_masterplan:** YES — this is a hard production invariant (compile-time privacy contract, DSR 30-day cascade SLA, HARD_DENY classes); it must bind into any generated masterplan as a foundation-tier privacy decision. No explicit `masterplan_ref` front-matter (8.8% binding gap per keystone §4), so binding is currently implicit.
- **tensions:** Leans on ADR-0005 (Kafka, retired→Pulsar 0377) for the eventing-layer source-crate gate and ADR-0007 (Cedar/foundry) for enforcement — both carry retired vocabulary but not contradictory logic. The "12 classes is closed" claim vs. Open-Question Q3 (non-FHIR healthcare exemption) is an internal unresolved edge, not a cross-ADR conflict.
- **hyperscaler_challenge:** ALIGNED. Google/AWS/Azure all run purpose-based, class-tagged data-governance with structural (not memo) enforcement — AWS Macie/Lake Formation tag-based access, GCP DLP + VPC-SC, Azure Purview. A hyperscaler would make essentially this decision; if anything they'd push harder on *automated* classification rather than schema-annotation lift. Argues for KEEP, not amend/archive.
- **ai_slop:** No. Dense but each layer is load-bearing and traces to a real regulation (PIPA Art-references, GDPR, HIPAA, PCI-DSS v4.0) and a contradiction-ledger entry (LEDG-001/003/005).
- **refinement:** Refresh the three stale refs (foundry→intelligence; ADR-0005→ADR-0377 eventing; confirm `oya-governance-*` prefix) when this ADR is next touched; add explicit `masterplan_ref`.
- **consensus_needed:** Mild — Q2 ("no ads ever" in personal surfaces inviolable vs. carve a P-tier) is a genuine founder/GTM question, but defaulted sensibly to inviolable.

---

### ADR-0009 — Cell architecture (per-tenant per-region blast-radius cells; routing at edge/mesh/store/event; per-cell HSM)

- **decision_atom:** Adopt cells as the primary blast-radius isolation primitive — per-tenant per-region, five sizing tiers (Dedicated/Shared-small/medium/large + a Public-corpus tier), with cell-routing primitives at edge/mesh/store/event layers, a per-cell HSM partition for DEK isolation, and quarterly per-pack cell-isolation evidence.
- **domain:** isolation-runtime (primary — blast-radius/isolation topology); tenancy (secondary — cell sizing is keyed on tenant/tenant-class).
- **current_status:** Proposed.
- **disposition:** AMEND (then RATIFY) — the blast-radius/cell-isolation *principle* is sound and industry-standard, but the doc is steeped in retired vocabulary and superseded substrate refs and must be amended before/at ratification: (1) a `Foundry-runtime` cell tier and "every Foundry agent invocation" boundary — **foundry is retired** (ADR-0335→intelligence); (2) "Envoy per ADR-0013 ancestry" and "Istio Ambient per ADR-0044" — current orchestration canon is **Talos+CAPI+ArgoCD** (ADR-0375) and the keystone notes ADR-0013 is the *license* ADR, so "ADR-0013 ancestry" for edge gateway is a **mis-citation/garbage ref**; (3) "Kafka partition keys (ADR-0005)" — retired→Pulsar (0377); (4) "Citus shard (ADR-0006)" for Postgres — consistent with source Postgres+pgcat posture but watch LINUX ADR-0001 own-DB tension.
- **proposed_resolution:** **RATIFY** — the cell-isolation primitive is a true, regulator-driven foundation decision (closes LEDG-010 single-cluster posture) that a generated masterplan needs; ratify *after* the vocabulary/substrate-ref amendment. Do NOT DROP. The risk in leaving it Proposed is that no-unaccounted-proposal discipline is violated and downstream ADRs (0008 §8 DEK shred, 0010 residency) already depend on it.
- **governing:** Not superseded by an ADR→ADR edge, BUT the **"cell-as-service / cell as a deployment object" framing must conform to ADR-0333** (cell retired as a *microservice*; survives only as a deployment *pattern*). ADR-0009 already treats cell as a topology/deployment primitive (not a µsvc), so it is compatible with 0333 — but the `Foundry-runtime` tier wording is the part 0335 governs.
- **truth_flag:** PARTIAL — core isolation logic TRUE; several STALE refs (foundry, Kafka/ADR-0005, Envoy-via-ADR-0013 mis-cite, Istio/ADR-0044). The "ADR-0013 ancestry" for the edge gateway is the closest thing to GARBAGE (wrong ADR number).
- **in_masterplan:** PARTIAL — carries clear planning_impact (cell tiers, HSM lead-time 6–9 months, 5000-cells/region cap, quarterly evidence cadence) but is Proposed, so it cannot bind as TRUE until ratified.
- **tensions:** (a) LINUX ADR-0018 framekernel "we are the host, no separate containerd" isolation model vs. this cell/HSM/mesh topology built on conventional cloud substrate (keystone fault-line §3); (b) per-cell HSM economics for `Dedicated` low-volume sovereign tenants (the ADR self-flags this); (c) substrate refs (Istio/Envoy/Kafka) predate the Talos/Pulsar canon.
- **hyperscaler_challenge:** ALIGNED. Cell-based architecture is explicitly AWS canon (the ADR cites it), Google per-locale shards, Azure scale units — a hyperscaler would absolutely make this decision. Verdict aligned; the only questionable sub-choice is per-cell *dedicated HSM partitions* at the `Dedicated` tier (hyperscalers favor multi-tenant HSM with logical partitions / Nitro-class attestation over physical-partition-per-cell for cost) — argues for AMEND of the HSM-cost section, not archive.
- **ai_slop:** Low. Real engineering content; the slop signal is the retired-vocabulary leakage and the wrong ADR cross-references, which read like generated-from-stale-context.
- **refinement:** Re-cite substrate to Talos/CAPI (0375), Pulsar (0377), and the correct mesh/gateway ADR; rename `Foundry-runtime` tier to `intelligence-runtime`; fix "ADR-0013 ancestry."
- **consensus_needed:** "Per-cell physical HSM partition vs. logical/attested partition" is a real cost/sovereignty founder question worth surfacing.

---

### ADR-0010 — Regional pack architecture (canonical seams + per-locale plug-ins)

- **decision_atom:** Adopt a locale-agnostic canonical architecture plus versioned, Cosign-signed, swappable **regional packs** (one per market: KR/JP/US/EU/IN/BR/KSA/UAE/ANZ/SG/…) that supply every per-locale concern (regulatory, compliance, i18n, currency, calendar, tax, identity, payment, address, ecosystem partners, content safety, ad policy, industry data models, vendor partners) through published seam traits, with per-pack residency intersection at tenant onboarding and a regulator-change watch lane.
- **domain:** compliance-residency (primary); product-ux (secondary — i18n/locale-formatting/address are partly UX, but the spine is regulatory/residency).
- **current_status:** Proposed (with a 2026-06-02 platform-readiness amendment note already in-file keeping `regional-packs/<pack-id>/` valid).
- **disposition:** AMEND (then RATIFY) — sound and arguably the single most defensible "parallel-market" decision in the corpus; needs only a light amend: the seam impls cite `oya-pack-<pack-id>-*` crate naming consistent with ADR-0015 (good), but the doc still presumes ADR-0013 = "supply-chain bar" for Cosign signing — ADR-0013 in this corpus is the *license* policy, and supply-chain signing canon is Cosign+Rekor per ADR-0039 (cited correctly in ADR-0014), so the "Cosign-signed (per ADR-0013 supply-chain bar)" ref is a **mis-citation** (should be ADR-0039).
- **proposed_resolution:** **RATIFY** — parallel-market regional-pack architecture is a true, founder-aligned (PRD "sovereignty is a global window, not Korea-specific") foundation decision that the masterplan must carry; ratify after fixing the Cosign cross-ref. Do NOT DROP.
- **governing:** — (not superseded; the in-file 2026-06-02 note explicitly preserves it until ADR-0010/0064 are superseded).
- **truth_flag:** TRUE (PARTIAL) — architecture and pack-content taxonomy are correct and current; the lone defect is the ADR-0013-vs-ADR-0039 mis-citation for signing. No retired-brand leakage.
- **in_masterplan:** YES — strong planning_impact: `regional-packs/<pack-id>/` authoring root, per-pack semver + tenant version pinning, regulator-watch automation, residency intersection at onboarding. This is masterplan-binding foundation scope.
- **tensions:** Cross-pack tenant residency conflict resolution (Q3) overlaps ADR-0008 cross-tenant identity link and ADR-0009 per-cell residency — coherent dependency, not a conflict. No cross-side LINUX tension (LINUX pilot has no locale-pack equivalent).
- **hyperscaler_challenge:** ALIGNED / slightly QUESTIONABLE on breadth. Hyperscalers do region/partition-based compliance (AWS GovCloud/China partitions, Azure sovereign clouds, Google Assured Workloads) — the *seam-per-locale plug-in* model is more granular than any hyperscaler ships, which is a differentiator but also a maintenance-cost bet (the ADR self-flags linear per-pack burden). Verdict aligned; the 11+ simultaneous markets at day-0 ambition is the questionable part — argues for AMEND (sequence pack rollout, don't archive).
- **ai_slop:** No. The pack-content table is concrete and locale-accurate (real regulators/rails/IdPs per market); this is high-signal.
- **refinement:** Fix Cosign signing ref to ADR-0039; consider a pack-rollout sequencing note so "11 markets in parallel" is staged rather than implied-simultaneous.
- **consensus_needed:** "How many packs ship at GA vs. post-GA" is a real founder sequencing question (breadth vs. focus).

---

### ADR-0011 — Cross-microservice contract registry (`contracts/microservice-contracts.yaml`, oya-check-contracts lane, change protocol, auto-SDKs)

- **decision_atom:** Adopt `contracts/microservice-contracts.yaml` as the single source-of-truth registry for the ~25 cross-microservice contracts, with protocol-specific sub-directories (openapi/proto/asyncapi/cedar/schemas), auto-generated multi-language SDKs (never hand-edited), a breaking-change protocol (oasdiff/buf-breaking/AsyncAPI-delta), and a gating `oya-check-contracts` CI lane.
- **domain:** api-contracts (primary); ci-cd-build (secondary — the enforcement is a CI lane + codegen pipeline).
- **current_status:** Accepted.
- **disposition:** AMEND — keep the decision (contract registry is correct and cohesion-central) but the doc carries notable staleness: (1) Owner is `oya-foundry` (registry surface) — **foundry retired**, now intelligence/governance (ADR-0335/0347); (2) SDK codegen crates named `oya-foundry-sdk-gen-*` — retired prefix; (3) several contract rows hard-code `owner_microservice: foundry` and topics `oya.foundry.capability.invoked.v1` — these are illustrative YAML but use the dead brand; (4) `CloudEvents per ADR-0005` references the retired-Kafka ADR (the CloudEvents/AsyncAPI contract survives, the Kafka broker name is stale→0377); (5) §Operational frankly states `oya-check-contracts` is "advisory P0 lane reference until the crate exists" — so the gate is **aspirational, not shipped** (a truth-flag concern).
- **proposed_resolution:** NA (Accepted).
- **governing:** — (not superseded; references ADR-0058 flat-catalog and ADR-0059 which are outside this chunk).
- **truth_flag:** PARTIAL — the registry decision is TRUE and current; multiple STALE refs (foundry brand throughout owner/crate/topic names; ADR-0005/CloudEvents). The "advisory until the crate exists" admission means the *enforcement* is partly aspirational, which an auditor should flag as not-yet-TRUE-in-fact.
- **in_masterplan:** YES — contract registry + breaking-change waves + generated-SDK discipline is a foundation planning artifact the masterplan needs. Note the Wave-N/N+1/N+2 deprecation cadence correctly uses **Wave** vocabulary (M0–M3/MVP retired per keystone §2) — good.
- **tensions:** None cross-side. Internal: the foundry-branded owner/topics will collide with ADR-0335's intelligence/governance rename on any consolidation. Relationship to ADR-0015 (cites `source_of_truth` flat crates) is a clean dependency.
- **hyperscaler_challenge:** ALIGNED. A single contract registry + generated SDKs + breaking-change gates is exactly how Google (protobuf + bazel + buf-style breaking checks), AWS (Smithy models → multi-lang SDKs), and Azure (TypeSpec/AutoRest) operate. Verdict strongly aligned; argues for KEEP/AMEND (refresh the dead brand), not archive.
- **ai_slop:** Low. Concrete and correct pattern; slop signal is only the retired-brand string leakage in the YAML examples.
- **refinement:** Bulk foundry→intelligence/governance rename of owner/crate/topic strings; re-cite ADR-0005→ADR-0377 for the eventing contract; either ship `oya-check-contracts` or downgrade its "Accepted" enforcement claim to "planned" explicitly.
- **consensus_needed:** None substantive.

---

### ADR-0013 — Product license policy (3-tier: allowed / forbidden-in-product / requires-review; dev-only carve-out; oya-governance-license lane; per-release SBOM)

- **decision_atom:** Adopt a three-tier product-code license policy — Tier 1 allowed (Apache-2/MIT/BSD/ISC/0BSD/MPL-2/Unicode/Zlib), Tier 2 forbidden and CI-hard-failed (GPL/AGPL/commercial-only), Tier 3 requires `council-architecture`+`legal` review (LGPL/SSPL/BUSL/Elastic/RSAL/TSL/Confluent/AWS-FSL/Commons-Clause) — with a strict dev-dependencies-only carve-out, the `oya-governance-license` P0 CI lane (cargo-deny + per-language tools), and a Cosign-signed/Rekor-anchored CycloneDX SBOM per release.
- **domain:** security-supplychain (primary); governance-process (secondary — the tier-review protocol).
- **current_status:** Proposed.
- **disposition:** AMEND (then RATIFY) — this is a **canonical-posture-grade decision** (keystone §3 "License posture: OSI-strict; no AGPL/GPL in product code" governing ADRs = 0013, 0211, 0345). It should NOT be sitting at Proposed. Light amend: the Tier-3 table correctly captures the **Redis RSAL / SSPL relicense** driver (consistent with ADR-0336 Redis→Valkey) and the **Confluent/Kafka** SaaS-restriction (consistent with ADR-0377 Kafka→Pulsar) — these are TRUE; refresh "cloud Kafka offering" wording given Pulsar is now canon.
- **proposed_resolution:** **RATIFY** — the keystone map already treats ADR-0013 as a *governing* canonical-posture ADR for license; leaving it Proposed is a status-drift bug. Accept it. Do NOT DROP — it is load-bearing for ADR-0010 pack signing, ADR-0014 build-vs-buy, and the whole OSI-strict posture.
- **governing:** — (it is itself a governing ADR for the license domain).
- **truth_flag:** TRUE — content is current and correct (AGPL/GPL forbidden, SSPL/RSAL/BUSL require review). Minor STALE: "conflicts with cloud Kafka offering" (Kafka retired→Pulsar 0377) and the implicit "Redis 7.4+ Tier 3" framing is now moot since ADR-0336 retires Redis→Valkey (BSD-3, Tier 1) — so Redis-as-a-dep is no longer the live question.
- **in_masterplan:** YES — OSI-strict license posture + per-release SBOM (CycloneDX, EO 14028, EU CRA alignment) is a hard masterplan invariant.
- **tensions:** Mild internal tension with the AGPL-carve-out posture noted in keystone §3 (Observability stack Loki/Tempo/Mimir is AGPL-3 per ADR-0383, license posture allows "carve-outs for server-side substrate with evidence") — ADR-0013's flat "AGPL forbidden in product code" needs to acknowledge the server-side-substrate carve-out that 0345/0383 rely on, else it reads stricter than current canon. This is the one substantive amend.
- **hyperscaler_challenge:** ALIGNED. AWS/Google/Azure all run hard OSI license gates, forbid AGPL/SSPL in shipped product, and emit SBOMs (all three publish SBOMs; AWS/GCP have internal copyleft-blocking gates). Verdict strongly aligned — this is table-stakes hyperscaler discipline. Argues for RATIFY.
- **ai_slop:** No. The SPDX-mapped tier tables are concrete and legally precise.
- **refinement:** RATIFY (flip Proposed→Accepted); add the explicit AGPL server-side-substrate carve-out reconciling with ADR-0345/0383; drop/soften the "cloud Kafka offering" line (Pulsar canon).
- **consensus_needed:** "Is the AGPL carve-out for server-side substrate (observability) a clean exception or a slippery slope?" — worth a one-line founder confirmation, but 0345/0383 already imply the answer is a bounded carve-out.

---

### ADR-0014 — Build-vs-buy policy (per-microservice matrix, decision flow chart, per-dep metadata, oya-governance-build-vs-buy lane)

- **decision_atom:** Adopt a per-microservice build-vs-buy matrix (in-house-obligatory for foundation/runtime substrate kernels; external-acceptable for mature license-clean OSS at the data/infra layers; everything else requires-review) plus a decision flow chart, mandatory per-dep catalog metadata (license_tier + maturity + isolation + replacement_plan + replacement_trigger + owning_team), and a `oya-governance-build-vs-buy` P0 CI lane that hard-fails external deps on in-house-obligatory surfaces.
- **domain:** ci-cd-build (primary — it's a governance/CI gate over dependency choice); governance-process (secondary).
- **current_status:** Proposed.
- **disposition:** AMEND (then RATIFY) — the build-vs-buy *protocol* is sound and is the source-side keystone for the "own-vs-reuse ratchet" (keystone fault-line §5: both repos share the "own when proven" principle). BUT the matrix table is the **most stale artifact in this chunk** and needs a substantive amend: it still lists **Kafka** as the message broker (retired→Pulsar 0377), **Redis pre-7.4** as KV/cache (retired→Valkey 0336), **Foundry runtime** as an in-house-obligatory surface (retired→intelligence 0335), **Istio Ambient / Envoy / containerd+runc** per old ADR-0044/0013 refs (orchestration canon is now Talos+CAPI 0375), **OpenTelemetry/VictoriaMetrics/Tempo/Loki** observability (keystone §3 canon is Loki/Tempo/**Mimir**/Grafana per 0383 — VictoriaMetrics is drifted), **Argo Rollouts/CD per ADR-0050** and **containerd per ADR-0028** (CI/CD canon churned through 0511/0513). The decision survives; the table is a stale snapshot.
- **proposed_resolution:** **RATIFY** the protocol/flow-chart/metadata-schema; the *matrix rows* should be marked as a living appendix re-derived against current canonical posture (0375/0377/0336/0335/0383) rather than frozen. Net: RATIFY with a mandated matrix refresh. Do NOT DROP — the protocol is foundation governance.
- **governing:** — (not superseded as an ADR; the *matrix contents* are governed/overridden by the newer substrate ADRs 0375/0377/0336/0335/0383).
- **truth_flag:** PARTIAL → STALE on the matrix. The protocol/flow/metadata are TRUE; the per-surface defaults table is broadly STALE (Kafka, Redis, foundry, VictoriaMetrics, Istio/Envoy/containerd via old ADR refs). This is the chunk's biggest stale-content item.
- **in_masterplan:** YES — build-vs-buy governance + per-dep replacement-trigger metadata is a foundation planning decision; but the matrix must be re-derived from current canon before it binds, or the masterplan inherits retired-substrate choices.
- **tensions:** **DIRECT cross-side keystone tension (§6):** source ADR-0014 = "build-vs-buy POLICY"; **LINUX ADR-0014 = "container-runtime" (one OCI/CRI frontend + pluggable IsolationBackend port)** — guaranteed number collision on merge (keystone §6.4) and a semantic mismatch (same number, different topic). Also fault-line §5: source ADR-0014's "external-acceptable, own-when-proven" ratchet vs. LINUX's OWN_DAY0 ambition (DB engine/policy/kernel) — the principle is shared, the trigger threshold differs. Internal tension: the matrix's substrate choices now contradict the canonical-posture table.
- **hyperscaler_challenge:** ALIGNED on method, QUESTIONABLE on the "in-house-obligatory foundation kernels / ads auction / cloud control plane" breadth. Hyperscalers absolutely run build-vs-buy matrices and own their differentiating substrate (Google owns Spanner/Borg; AWS owns Nitro/its control plane) — so owning the control plane + auction is aligned. But owning *all* foundation kernels day-0 while a 3-person-scale org is questionable vs. the hyperscaler pattern of buying mature OSS until scale forces a fork (which this very ADR's "external-acceptable + replacement_trigger" actually endorses). Verdict aligned-in-principle; argues for AMEND the matrix toward later own-triggers, not archive.
- **ai_slop:** Low on structure, MODERATE staleness-as-slop in the matrix (rows generated against a substrate snapshot that has since moved). Not fabricated — just stale.
- **refinement:** Re-derive the matrix against current canon: Kafka→Pulsar (0377), Redis→Valkey (0336), Foundry→intelligence (0335), Istio/containerd → Talos/CAPI substrate (0375), VictoriaMetrics→Mimir (0383), CD/CI rows → 0511/0513; resolve the LINUX-ADR-0014 number collision (renumber LINUX to 0515+ per keystone §6.4).
- **consensus_needed:** "Where is the own-when-proven trigger threshold?" is THE founder question that fault-line §5 says distinguishes source (own-when-proven) from LINUX (own-day-0) — this ADR is the right home for that decision and should state the threshold explicitly.

---

### ADR-0015 — Architectural flattening target (flat-crates naming, role taxonomy, dep-direction validator, legacy-tree migration)

- **decision_atom:** Adopt flat-crates as the canonical workspace shape — `crates/oya-<context>-<role>[-<capability>]/` naming, a closed role taxonomy (kernel/domain/app/api/worker/adapter/runtime) with a forbidden-edge dep-direction graph (kernel←domain←app←{api,worker,adapter}←runtime), a boundary-validator CI gate, and a forward-only migration off the retired `modules/`/`services/`/`platform/` tree (live baseline 2026-05-11: 64 members all under `crates/oya-*`).
- **domain:** ci-cd-build (primary — repo structure + boundary-validation governance); governance-process (secondary).
- **current_status:** Accepted (front-matter `accepted`, `superseded_by: [ADR-0131]` with explicit `supersession_note: "Partial"`).
- **disposition:** KEEP (PARTIAL) — exactly the keystone §1.1 verdict: ADR-0131 supersedes ONLY the docs-vs-crates top-level split for per-service ownership; ADR-0015's **BC and layer/dep-direction rules remain in force**, so status correctly stays Accepted. This is a well-managed partial-supersession with in-file `supersession_note` + an ADR-0106 amendment note (`application`→`usecase`, `app` = composition-root). Keep with the partial caveat intact.
- **proposed_resolution:** NA (Accepted).
- **governing:** **ADR-0131** (per-microservice flat layout) — but PARTIAL only (top-level docs-vs-crates split). ADR-0105 (13-layer enum) + ADR-0106 (role rename) also amend the role taxonomy. None fully archive this ADR.
- **truth_flag:** TRUE (PARTIAL) — the live-baseline numbers (64 members, no legacy roots) and the partial-supersession bookkeeping are accurate and self-aware. The only STALE term is `foundry` appearing in the `<context>` enum and kernel-size table (retired→intelligence per 0335) — naming, not logic. The dep-direction graph and validator are current canon.
- **in_masterplan:** YES — flat-crate naming + role taxonomy + boundary-validator is a foundation repo-structure invariant that every other ADR's `source_of_truth` crate ref depends on; it must bind into the masterplan as structural ground-truth.
- **tensions:** **Number collision (keystone §6.4):** source ADR-0015 (flattening) vs. **LINUX ADR-0015** — and keystone §1.1 separately notes source ADR-0015 itself is the partially-superseded flattening ADR; on merge, LINUX 0015 must renumber. Internal: `foundry` in the context enum will collide with 0335's intelligence rename. The partial-supersession by 0131/0105/0106 is well-tracked, not a live conflict.
- **hyperscaler_challenge:** ALIGNED. Google's monorepo + strict layered build-target dependency rules (BUILD visibility, layering_check), Bazel/Buck2-enforced dep direction (this repo is on Buck2 per ADR-0392), and clean-architecture role separation are exactly hyperscaler practice. Verdict strongly aligned — a flat, boundary-validated monorepo is the hyperscaler default. Argues for KEEP.
- **ai_slop:** No. This is mature, self-aware doctrine with honest live-baseline numbers and explicit retention-as-history framing for the migration section.
- **refinement:** Refresh `foundry` in the context enum + kernel-size table to `intelligence`/`governance` (0335); on merge, renumber LINUX ADR-0015 and reconcile with ADR-0131/0105/0106 as the current role-taxonomy authority.
- **consensus_needed:** None — supersession is already cleanly adjudicated in-file.

---

## Chunk notes

- **Range note:** the sorted slice 8–14 yields ADR-0008, 0009, 0010, 0011, **0013, 0014, 0015** — **ADR-0012 is absent from this window** (the directory listing jumps 0011→0013; an ADR-0012 file may exist outside the slice or be a numbering gap). All 7 present files were read fully. No "no ADRs in range" condition.

- **Status-drift is the headline finding.** Three foundation-tier, canonical-posture-grade decisions sit at **Proposed** despite being treated as governing/binding by the keystone map and by downstream ADRs: **ADR-0013** (license posture — keystone §3 names it a *governing* ADR), **ADR-0010** (regional packs — founder-aligned, in-file 2026-06-02 amendment treats it as live), and **ADR-0014** (build-vs-buy — the source-side ratchet keystone). Per the no-unaccounted-proposals rule, all three resolve to **RATIFY** (with light amends). **ADR-0009** (cells) also RATIFY-after-amend. This is the cleanest signal for the founder: four foundation decisions are de-facto-true but de-jure-Proposed — ratify them.

- **Retired-vocabulary leakage is pervasive but non-fatal.** Every Proposed/Accepted ADR in this chunk except 0013 carries **foundry** strings (0008 consequences, 0009 `Foundry-runtime` cell tier + boundary, 0011 owner/crate/topic names, 0014 in-house-obligatory surface, 0015 context enum). Per keystone §2, foundry is RETIRED→intelligence (consumer AI) + governance (CI). None of these are *logic* errors — they are brand-residue (MFL-0002/0003 lanes), all AMEND-on-next-touch, none ARCHIVE.

- **Substrate-snapshot drift concentrates in ADR-0014's matrix** (Kafka/Redis/VictoriaMetrics/Istio/containerd via old ADR-0044/0013/0028/0050 refs). This is the single most stale artifact in the chunk and the one most likely to poison a generated masterplan if bound as-is — it should be re-derived against current canon (0375/0377/0336/0335/0383/0511) rather than frozen.

- **Mis-citations to fix:** ADR-0009 cites "Envoy per ADR-0013 ancestry" and ADR-0010 cites "Cosign-signed per ADR-0013 supply-chain bar" — but **ADR-0013 is the LICENSE policy** in this corpus. The edge-gateway/supply-chain references should point to the mesh/gateway ADR and to **ADR-0039** (Cosign+Rekor, cited correctly in ADR-0014) respectively. These are concrete wrong-ADR-number defects.

- **Cross-side (LINUX) collisions in this chunk:** source ADR-0014 (build-vs-buy) ≠ LINUX ADR-0014 (container-runtime); source ADR-0015 (flattening) ≠ LINUX ADR-0015 — both guaranteed number collisions on merge (keystone §6.4, renumber LINUX to 0515+). The deeper tension is fault-line §5: source ADR-0014's "own-when-proven" ratchet vs. LINUX OWN_DAY0 — same principle, different trigger threshold; ADR-0014 is the right home to state that threshold explicitly.

- **Eventing dependency chain:** ADR-0008 §6.3, ADR-0009 (event routing), and ADR-0011 (asyncapi/CloudEvents) all lean on **ADR-0005 (Kafka)** for the eventing layer — ADR-0005 is retired→**ADR-0377 (Pulsar+Oxia)**. The outbox/partition/CloudEvents *patterns* survive (keystone §1.1); only the broker name is stale across all three.

- **Net masterplan posture for this chunk:** all 7 ADRs carry real planning_impact and should bind into the generated masterplan; none are GARBAGE or ARCHIVE candidates. Disposition tally: **KEEP** = 0008, 0015 (partial); **AMEND→RATIFY** = 0009, 0010, 0013, 0014. Zero ARCHIVE, zero DROP, zero MERGE. The work is ratification + vocabulary/substrate-refresh, not retirement.
