# ADR Audit — source-25

- **Side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`)
- **Chunk:** source-25 (slice 169–175 of `ls -1 docs/decisions/ADR-*.md | sort`)
- **Range:** ADR-0194 → ADR-0200
- **ADRs reviewed:** 7 (ADR-0194, ADR-0195, ADR-0196, ADR-0197, ADR-0198, ADR-0199, ADR-0200)
- **Auditor:** ADR AUDITOR (coverage backfill)
- **Date:** 2026-06-06
- **Theme of slice:** The "storage/substrate batch" of 2026-05-18 — tenant time-series, stream processing, object storage, backup, node autoscaling, FinOps, and WASM runtime. This is the operational data/cloud-substrate spine that the keystone-map §3 canonical-posture table cites directly (ADR-0192/0193/0194/0196 data-tier; ADR-0200 isolation/runtime). All seven are `Accepted`, all carry `supersedes: [] / superseded_by: []`, and all share the same in-house-ladder doctrine (Phase-0 vendor-via-adapter → Phase-2 own-when-triggered).

---

### ADR-0194 — Tenant-facing time-series canonical: TimescaleDB 2.26 Community Edition (Apache-2.0) as a Postgres 18 extension

- **decision_atom:** Tenant-facing time-series is canonically stored in TimescaleDB 2.26 Community Edition (Apache-2.0) as a per-µservice opt-in Postgres-18 extension on the Tier-1 OLTP fleet, bound only to the Apache-2.0 community feature surface (TSL functions CI-forbidden), with ops/SRE metrics staying in Prometheus+Mimir and wide aggregates staying in ClickHouse.
- **domain:** data-storage (cross-cut: data-engine-db)
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (already Accepted)
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** YES — keystone-map §3 names TimescaleDB as the canonical tenant-TS store (governing ADR-0194); carries explicit SLO/operational bindings and a manifest field (`data.timeseries.enabled`).
- **tensions:** (1) The hardest cross-side tension is the keystone fault-line #1: LINUX ADR-0001 wants a from-scratch Rust multi-model engine that *eliminates the Postgres/sqlx dependency*, while this ADR doubles down on Postgres-as-substrate ("Replacing Postgres is out of scope for any plausible roadmap"). Direct own-the-engine vs assemble-OSS conflict — surface, do not resolve. (2) Internal boundary tension with ADR-0193 (ClickHouse) is well-managed via the `oya-check-vendor-lockin-discipline` advisory check, not a real conflict. (3) Uses the retired-vocabulary token "Tier 1/Tier 2" for Postgres *storage tiers* — this is the storage-layering axis (ADR-0184), NOT the retired tenant "tier-system" of ADR-0329; no leakage, but a lint-adjacent naming collision auditors should not mis-flag.
- **hyperscaler_challenge:** ALIGNED. Google Cloud SQL and Azure Database for Postgres both ship the TimescaleDB community extension; the ADR's own §"Industry parallels" is accurate. The TSL-fence (avoid `add_retention_policy`/compression/hyperfunctions, reimplement via ~30-LOC workers) is a defensible license-purity choice a hyperscaler-grade shop would make. Does NOT argue for amend/archive.
- **ai_slop:** None material. Version/date claims (TimescaleDB 2.26 released 2026-03-24; PG18 support since v2.23) are specific and plausible for the doc's 2026-05-18 horizon but are unverifiable future-dated facts — treat as as-of-authoring, refresh per ADR-0098 LTS cadence.
- **refinement:** None required for KEEP. The reimplement-refresh/retention-via-worker pattern is sound but adds per-µservice surface; the documented standard mitigates.
- **consensus_needed:** NO at the ADR level. The Postgres-elimination tension belongs to LINUX ADR-0001's reconciliation, not to amending this source ADR.

---

### ADR-0195 — Stream processing tier: ClickHouse Materialized Views + Kafka Engine default; Apache Flink 2.2 escalation under explicit ADR amendment

- **decision_atom:** Stream processing defaults to ClickHouse Materialized Views + Kafka Engine for ~95% of workloads (in-OLAP, sub-second freshness, no separate cluster), with Apache Flink 2.2.x (Apache-2.0) reserved as a per-workload ADR-amendment-gated escalation for the ~5% genuinely stateful/multi-stream/CEP cases.
- **domain:** data-engine-db (cross-cut: data-storage)
- **current_status:** Accepted
- **disposition:** AMEND (minor — stale eventing-substrate naming)
- **proposed_resolution:** NA (already Accepted)
- **governing:** —
- **truth_flag:** PARTIAL — core decision TRUE; one stale reference vector.
- **in_masterplan:** YES — keystone-map §3 eventing row lists ADR-0195 alongside ADR-0377/0397 as governing the Pulsar+Oxia eventing posture; this ADR's MV-default carries the stream-processing binding.
- **tensions:** Naming/ref drift, NOT a decision conflict. The ADR title and decision body still say "**Kafka** Engine" and the source events "land in the log-broker substrate (Apache Pulsar 4.2.x ... Kafka-on-Pulsar proxy)." Per keystone §2 retired-vocab, standalone **Kafka is retired → Pulsar 4.x + Oxia (KoP wire-compat)** per ADR-0377-kafka-to-pulsar (supersedes ADR-0005). This ADR is *substantively correct* (it already routes through Pulsar's KoP endpoint and cites Pulsar 4.2.x) — but the prominent "Kafka Engine" label in the title is retired-vocab leakage that will read as a contradiction at a glance. AMEND: retitle the connector reference to "ClickHouse Kafka-protocol (KoP) Engine on Pulsar" and note ADR-0377 explicitly. Also references ADR-0005 (eventing backbone outbox) which is itself superseded-in-fact by ADR-0377 — outbox pattern survives, but the bare ADR-0005 cite should point to the live successor.
- **hyperscaler_challenge:** ALIGNED. The MV-default / Flink-escalation asymmetry mirrors Cloudflare (Workers Analytics Engine on ClickHouse), Stripe, and Uber practice the ADR cites; AWS/GCP/Azure all run managed Flink only for the heavy tier. A hyperscaler would make exactly this asymmetric default-cheap/escalate-heavy call. Does NOT argue for archive; argues only for the naming amend above.
- **ai_slop:** Low. Flink 2.2.1 "released 2026-05-15" is a future-dated specific that's unverifiable but consistent with the batch horizon. Materialize-BSL/RisingWave "deferred" reasoning is genuine and well-argued, not filler.
- **refinement:** Fold the Kafka→Pulsar terminology so the title doesn't carry a retired brand; confirm the Phase-2 "in-house OLAP warehouse MV layer" inherits cleanly from ADR-0193's Phase-2.
- **consensus_needed:** NO. Mechanical retired-vocab cleanup; no founder decision required.

---

### ADR-0196 — Object storage canonical: SeaweedFS primary, Ceph RGW scale-up path

- **decision_atom:** SeaweedFS 4.22 (Apache-2.0) is the canonical S3-compatible object store behind the `oya-shared-object-store-kernel::ObjectStore` trait, with Ceph RGW as the pre-designed scale-up path (trigger ≥800 TB / 8·10⁸ objects), AWS S3/GCS/Azure adapters for egress-permitted packs, and a Phase-2 in-house `oya-object-store-server` gated on the SeaweedFS ceiling.
- **domain:** data-storage
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (already Accepted)
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** YES — keystone-map §3 data/storage row names "SeaweedFS primary + Ceph RGW scale-up (object)" governed by ADR-0196; it is the backup target (ADR-0197) and FinOps export sink (ADR-0199), so it is load-bearing across the slice.
- **tensions:** Minor — Garage rejected for primary is described as "AGPL3" then "permissive (AGPL3 — but small enough...)"; AGPL3 is copyleft, not permissive, so the parenthetical is internally loose wording (does not change the verdict). The MinIO-AGPL3+feature-gating rejection is consistent with the OSI-strict posture (keystone §3 license row, ADR-0173/0211/0345). The "own when proven" Phase-2 build aligns with keystone fault-line #5's shared ratchet language. No cross-side contradiction with LINUX (LINUX has no competing object-store ADR in 0001–0026).
- **hyperscaler_challenge:** ALIGNED with a caveat. CERN/DigitalOcean/WD references for Ceph at scale are accurate; SeaweedFS-primary is a reasonable cost/ops choice at the 2026 floor. A pure hyperscaler would more likely go straight to Ceph (or cloud S3) and skip SeaweedFS — QUESTIONABLE on the "two migrations" critique (SeaweedFS→Ceph→in-house is three object backends over the roadmap). But the kernel-trait seam makes the staged path defensible; does NOT argue for archive.
- **ai_slop:** Low. CERN ~1.5 EB and Sina Weibo "billions of objects" are real reference points. FOCUS 1.3 ratified 2025-12-05 cited (consistent with ADR-0199).
- **refinement:** Fix the "permissive (AGPL3)" wording for Garage; otherwise sound. Consider whether the three-stage object-backend ladder (SeaweedFS→Ceph→in-house) is one migration too many vs going Ceph-direct — a cost/ops question, not a correctness one.
- **consensus_needed:** Soft — "Is a 3-stage object-store ladder (SeaweedFS→Ceph→bespoke) worth the carrying cost vs Ceph-direct?" Worth a founder line but not blocking.

---

### ADR-0197 — Backup substrate: 3-pronged (Velero + pgBackRest + Restic) on SeaweedFS, age-encrypted

- **decision_atom:** Backups use three concern-scoped prongs — Velero 1.18 (K8s state/PVs), pgBackRest 2.58 (Postgres PITR), Restic 0.18 (non-K8s filesystem) — all landing age-encrypted in SeaweedFS behind the `oya-shared-backup-kernel::BackupExecutor` trait, with RPO/RTO per workload class, regulatory-pack retention floors, quarterly chaos-driven restore drills, and Velero flagged IN-HOUSE-TARGETED (Phase-2 `oya-backup-orchestrator`).
- **domain:** dr-resilience (cross-cut: data-storage)
- **current_status:** Accepted
- **disposition:** AMEND (minor — internal cross-ref drift)
- **proposed_resolution:** NA (already Accepted)
- **governing:** —
- **truth_flag:** PARTIAL — decision TRUE; one self-inconsistent ADR cross-reference.
- **in_masterplan:** YES — carries explicit RPO/RTO bindings, regulatory retention floors, and a CI gate (`oya-check-backup-retention-discipline`); the DR posture is plan-load-bearing for compliance packs.
- **tensions:** Internal cross-ref inconsistency: the Context says "ADR-0180 (DR + business continuity portfolio policy) establishes RPO/RTO tiers," but the front-matter `related` lists **ADR-0241-dr-business-continuity-portfolio-policy** (and ADR-0152) for the same role — the prose cites ADR-0180 while the metadata points at ADR-0241. One of these is a stale number; auditors should reconcile which ADR is the live DR-portfolio policy (likely ADR-0241 per the descriptive filename). Also leans on the pgBackRest maintainer-transition / pgxbackup continuity story (2026-dated) — hedged correctly via the kernel trait, not a tension per se. No cross-side LINUX conflict.
- **hyperscaler_challenge:** ALIGNED. Pinterest/GitLab/Stripe "Velero + pgBackRest + Restic, quarterly restore drills" is real, well-cited public practice; AWS Backup / Google Backup-and-DR / Azure Backup parallels are accurate. The age-over-GPG and one-tool-per-concern choices are exactly what a hyperscaler-grade SRE org does. Does NOT argue for archive; argues for the cross-ref amend.
- **ai_slop:** Low. Velero "CNCF Sandbox per 2026 KubeCon" + "Broadcom-acquired-VMware steward" is the genuine governance-risk basis for the IN-HOUSE-TARGETED flag. pgxbackup continuity URL is future-dated but plausible.
- **refinement:** Reconcile ADR-0180-vs-ADR-0241 DR-policy reference; confirm the Velero Phase-2 `oya-backup-orchestrator` trigger conditions stay coherent with ADR-0196's object-store Phase-2 horizon (the ADR already ties them).
- **consensus_needed:** NO. Mechanical cross-ref fix.

---

### ADR-0198 — Kubernetes node autoscaling: Karpenter primary, NodePool per workload class

- **decision_atom:** Karpenter 1.11 (kubernetes-sigs/CNCF, Apache-2.0) is the canonical K8s node autoscaler with Cluster Autoscaler fully removed (no fallback), four NodePool CRDs per workload class (app/batch/gpu/regulatory) carrying taints, capacity-type, disruption budgets, and cost-attribution labels, KEEP-not-rebuilt because AWS itself runs it.
- **domain:** orchestration-scheduling
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (already Accepted)
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** PARTIAL — operationally binding (NodePool CRDs, taints, budgets) and feeds ADR-0199 FinOps labels, but it is a substrate-implementation decision under the broader orchestration posture (keystone §3 names Talos+CAPI+ArgoCD via ADR-0375 as the orchestration spine; Karpenter is a node-autoscaling detail beneath that, not separately enumerated in the §3 canonical table).
- **tensions:** (1) Substrate-coherence check vs keystone §3 orchestration row: source canon is **Talos immutable node-OS + CAPI + ArgoCD** (ADR-0375). Karpenter's on-prem path here is "Cluster-API cloud provider drives node provisioning (KubeVirt/Equinix/OpenStack)" — this must dovetail with Talos+Sidero/CAPI, not an alternate node lifecycle. No hard contradiction (Karpenter-on-CAPI is the documented on-prem mode) but the Talos/Sidero interaction is unspecified here and should be cross-checked against ADR-0375. (2) The ADR uses "Foundry capability invocations" as a bursty-workload example (§Context) — **retired brand leakage**: per keystone §2, "foundry" is RETIRED → cloud-intelligence (ADR-0335/0347). Cosmetic, but it's exactly the brand-residue MFL-0002/0003 lanes flag. (3) Mild cross-side resonance with LINUX ADR-0025 (Rust "Talos") — but that's a Talos tension, not a Karpenter one.
- **hyperscaler_challenge:** ALIGNED, strongest in the slice. "Use what AWS uses because AWS uses it for the same reasons" is literally correct — Karpenter IS AWS's donated-to-CNCF tool; AKS/OKE support it via provider plugins. Pinterest/Anthropic 15–25% cost-reduction references are real. A hyperscaler-aligned shop makes exactly this call. Does NOT argue for amend/archive on substance — only the cosmetic "Foundry" rename.
- **ai_slop:** Low. Karpenter 1.0 (2024-08), kubernetes-sigs governance, no-CA-fallback are all accurate-to-real-world. The CA-vs-Karpenter comparison table is correct.
- **refinement:** Replace the "Foundry capability invocations" example with "cloud-intelligence invocations" (retired-vocab fix). Cross-reference ADR-0375 explicitly so the Karpenter-on-CAPI on-prem path is anchored to the Talos/Sidero substrate rather than floating.
- **consensus_needed:** NO on substance. The only open item is brand-residue hygiene (covered by the corpus-wide foundry→intelligence retirement, not a new decision).

---

### ADR-0199 — Per-tenant cost attribution + FinOps substrate (OpenCost + FOCUS 1.3)

- **decision_atom:** Per-tenant cost attribution is enforced via a fixed CI-required Kubernetes/cloud label block (`oya.io/tenant-id|cost-center|workload-class|regulatory-pack`), aggregated by OpenCost 1.110 (CNCF incubating, Apache-2.0), normalized to FOCUS 1.3, with anomaly alerts and quarterly chargeback to the audit chain, and a Phase-2 in-house `oya-finops-portal` for the tenant-billing UX layer only (aggregation stays OpenCost-backed).
- **domain:** finops-cost (cross-cut: tenancy)
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (already Accepted)
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** YES — defines a CI-enforced label vocabulary and chargeback emission used fleet-wide (consumed by ADR-0198 nodes and ADR-0197 backup retention); strongly plan-binding.
- **tensions:** (1) Uses retired tenant vocabulary indirectly: the label `oya.io/workload-class` enum (app/batch/gpu/regulatory) is the *workload* axis, NOT the retired tenant "tier-system" — clean. But cost/chargeback "tier" language elsewhere in the corpus must map to **tenant-class** (`demo_trial`|`paid`) per ADR-0329; this ADR correctly avoids "tier" for tenants and uses `regulatory-pack`/`cost-center`, so no leakage here. (2) Light dependency-coupling with ADR-0174 (chargeback formula), ADR-0184/0186 (metrics path), ADR-0196 (FOCUS export sink), ADR-0198 (node labels) — coherent web, no conflict. (3) Kubecost-vs-OpenCost rejection rests on the accurate "Kubecost donated OpenCost to CNCF" relationship.
- **hyperscaler_challenge:** ALIGNED. FOCUS 1.3 is the genuine cross-cloud cost standard every hyperscaler is adopting; OpenCost (Spotify/Adobe references) is real. "FOCUS as publishing standard, not vendor product; build the tenant-billing UX in-house" mirrors AWS Cost Explorer / GCP Billing / Azure Cost Management exactly. Does NOT argue for amend/archive.
- **ai_slop:** Low. FOCUS 1.3 ratified 2025-12-05 and OpenCost CNCF-incubating-2024-10-25 are consistent real-world anchors. The Helm helper snippet is concrete, not filler.
- **refinement:** None for KEEP. The advisory→strict promotion path for `oya-check-tenant-cost-labels-coverage` is the right rollout shape; ensure the Phase-2 portal scope-fence (UX only, not aggregation) survives into the masterplan so it doesn't drift into an OpenCost replacement.
- **consensus_needed:** NO.

---

### ADR-0200 — WASM runtime canonical: Wasmtime

- **decision_atom:** Wasmtime (BytecodeAlliance, CNCF graduated, Apache-2.0+LLVM-exception, 30.x LTS floor) is the single canonical WASM runtime behind `oya-shared-wasm-runtime-kernel::WasmRuntime`, with WASI-Preview-2 + Component-Model as the canonical ABI, three versioned capability-limited sandbox classes (envoy-filter / workflow-studio-node / foundry-tool) each with hard fuel/memory/wall-clock ceilings, no ambient authority, and no in-house engine ever planned (own the integration layer, contribute upstream).
- **domain:** isolation-runtime (cross-cut: security-supplychain)
- **current_status:** Accepted
- **disposition:** AMEND (minor — retired-brand sandbox-class name + superseded ADR cite)
- **proposed_resolution:** NA (already Accepted)
- **governing:** —
- **truth_flag:** PARTIAL — core decision TRUE and strong; carries one retired-brand identifier and one stale ADR reference.
- **in_masterplan:** YES — keystone-map §3 isolation/runtime row names "wasmtime canonical WASM" governed by ADR-0200 alongside ADR-0147/0200/0254; this is a directly-cited canonical-posture decision.
- **tensions:** (1) **Retired-brand leakage in a load-bearing identifier.** One of the three canonical sandbox classes is named `foundry-tool` ("Foundry tool execution sandbox," citing ADR-0136). Per keystone §2, **foundry is RETIRED → cloud-intelligence** (ADR-0335/0347); ADR-0136 itself is superseded-in-fact by ADR-0335 (keystone §1.3 foundry-dissolution chain, and §5 fault-line #6 notes ADR-0136's stale `Accepted/superseded_by:[]` front-matter). Because `foundry-tool` is a *versioned class name in code* (kernel enum + import allowlist), this is more than cosmetic — it bakes a dead brand into the runtime contract. AMEND: rename the sandbox class to `intelligence-tool` (or `cloud-intelligence-tool`) and re-point the ADR-0136 cite to ADR-0335/the intelligence substrate. (2) Cites **ADR-0183 (policy-engine separation, Cedar/Kyverno)** which is `Superseded` by **ADR-0379** (Kubewarden default admission) per keystone §1.1 — the "WASM filters are NOT a policy engine; Cedar+Kyverno own that lane" aside should re-point to ADR-0379/the live Cedar+Kubewarden posture. (3) References ADR-0182 (north-south/east-west) and ADR-0185 (Workflow Studio) — live, no issue. Cross-side: resonates with LINUX ADR-0014/0018 isolation fault-line #3, but WASM-in-process sandboxing is a different layer than container/microVM isolation — complementary, not conflicting.
- **hyperscaler_challenge:** ALIGNED. Wasmtime is the correct canonical pick — Fastly Compute@Edge, Shopify, Microsoft Azure (Component Model co-development) all run it; CNCF-graduated; the capability/no-ambient-authority model is the Cloudflare-Workers/Fastly posture. A hyperscaler makes exactly this choice and exactly this "own the integration layer, not the runtime" call. Does NOT argue for archive; argues only for the two reference/naming amends above.
- **ai_slop:** Low. This ADR is notably self-aware ("doubt-driven check," "doubt: is this mature?", footnoted version floor owed to parent-wiring). Wasmtime 30.x LTS / WASI-P2-stable-since-Wasmtime-14 are accurate. Format note: ADR-0200 uses the *older* prose front-matter style (Markdown bullet `- Status:` / `- Supersedes: none`) rather than YAML front-matter like 0194–0199 — a format-consistency drift worth flagging for the eventual ADR-log re-founding (keystone §4), not a correctness issue.
- **refinement:** (a) Rename `foundry-tool` sandbox class → intelligence-tool; (b) re-point ADR-0136 → ADR-0335 and ADR-0183 → ADR-0379; (c) normalize front-matter to the YAML style used by the rest of the batch.
- **consensus_needed:** Soft — "Renaming the `foundry-tool` sandbox class is a (small) breaking change to a code-level enum/contract — confirm the corpus-wide foundry→intelligence rename (ADR-0335/0347) is authorized to touch runtime identifiers, not just docs/CI-lane prefixes." This is the one item where the retirement reaches into a shipped contract.

---

## Chunk notes

**Overall posture.** This is a clean, high-quality, internally coherent slice — the storage/cloud-substrate batch of 2026-05-18. All seven are `Accepted`, none are `Proposed` (so no RATIFY/DROP proposals are owed), none supersede or are superseded by other ADRs, and four are straight **KEEP** (0194, 0196, 0198, 0199). Three are **AMEND** for minor, mechanical reasons (0195, 0197, 0200) — none for substance. **Zero ARCHIVE, zero GARBAGE, zero WRONG.** Every ADR shares the same disciplined doctrine: vendor-via-kernel-trait Phase-0 → Phase-2 own-when-triggered, with explicit numeric build triggers — which directly matches keystone fault-line #5's shared "own when proven" ratchet (the disagreement with LINUX is the *trigger threshold*, not the principle).

**The three AMENDs are all the same disease — retired-vocabulary / stale-cross-ref leakage:**
- **ADR-0195** — title says "Kafka Engine" though the body already correctly routes through Pulsar KoP; per keystone §2 Kafka→Pulsar (ADR-0377). Retitle + re-point the ADR-0005 cite.
- **ADR-0197** — prose cites ADR-0180 for the DR-portfolio policy while front-matter `related` lists ADR-0241 for the same role; reconcile the live number.
- **ADR-0200** — `foundry-tool` sandbox class + ADR-0136 cite (foundry RETIRED → intelligence, ADR-0335/0347; ADR-0136 superseded-in-fact) **and** ADR-0183 cite (Superseded by ADR-0379). The `foundry-tool` one is the most consequential finding in the slice because it bakes a dead brand into a *code-level runtime contract*, not just prose.

**Cross-slice retired-brand residue (lint signal for the masterplan generation):** ADR-0198 also uses "Foundry capability invocations" as a prose example. Combined with ADR-0200's `foundry-tool` class, this slice alone shows two foundry-residue hits — consistent with keystone §2's warning that hundreds of `foundry`/`oya-foundry-*` strings persist corpus-wide post-ADR-0335. Recommend a single corpus-wide foundry→cloud-intelligence sweep rather than per-ADR edits; flag whether that sweep is authorized to rename *code identifiers* (the ADR-0200 sandbox-class question) and not just docs/CI-lane prefixes.

**Biggest cross-side fault-line touched:** ADR-0194's "replacing Postgres is out of scope for any plausible roadmap" is the sharpest restatement of keystone fault-line #1 — it is the direct foil to LINUX ADR-0001's "eliminate the PostgreSQL/sqlx dependency" own-the-engine ambition. This slice does not need amending for it; the tension is real architecture (own-the-DB-engine vs assemble-best-of-breed-on-Postgres) and belongs in the founder consensus queue, surfaced from the LINUX ADR-0001 side per the keystone verdict (§"Verdict on the LINUX auto-reconciliation").

**Masterplan authored-vs-generated reading (keystone §4, kept OPEN):** Under *both* readings these seven survive as live decisions feeding the plan. Under "masterplan-as-authority-ADRs-bind-in," they need `masterplan_ref` front-matter (none carry it today — consistent with the 8.8%-binding finding). Under "masterplan-generated-from-ADRs," their `planning_impact`/`status`/`deliverables` front-matter would feed generation directly — but ADR-0200's older prose front-matter style (no YAML `status:`/`related:` keys, uses `- Status: Accepted` bullets) would NOT parse cleanly into a generated masterplan. Flag ADR-0200's front-matter normalization as a prerequisite for the generated-from-ADRs design regardless of which way the founder resolves §4.

**Founder consensus questions distilled from this slice (crisp):**
1. *(ADR-0200, soft-blocking)* Is the foundry→cloud-intelligence retirement (ADR-0335/0347) authorized to rename **code-level identifiers** like the `foundry-tool` WASM sandbox class — i.e., is it a breaking-contract sweep or a docs-only sweep?
2. *(ADR-0196, soft)* Is the three-stage object-store ladder (SeaweedFS → Ceph RGW → bespoke `oya-object-store-server`) worth the carrying cost, or should it collapse to Ceph-direct?
3. *(ADR-0194, surface-only)* The Postgres-as-permanent-substrate stance here vs LINUX ADR-0001's own-the-DB-engine ambition — resolve on the LINUX side, do not amend this source ADR.
