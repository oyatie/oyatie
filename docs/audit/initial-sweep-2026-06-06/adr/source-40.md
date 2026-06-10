# ADR Audit — source-40

- **side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **chunk:** source-40 (slice lines 274–280 of `ls docs/decisions/ADR-*.md | sort`)
- **range:** ADR-0337 … ADR-0343
- **ADRs reviewed:** 7 (ADR-0337, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343)
- **auditor posture:** READ-ONLY; only this artifact written. Keystone map (`_map/canonical-posture-and-supersession-map.md`) consumed as baseline.
- **cluster note:** All seven are the 2026-05-21 `/idea-refine` doctrine cluster. 0337–0339 are the "triplet"; 0340–0343 are part of a "hexad" of six candidate ADRs (0340–0345). Every one is `status: Proposed`, `planning_impact: true`, `authority_tier: 1`. None supersedes another; they amend earlier substrate/doctrine ADRs and chain onto each other (0340 ties 0337/0338/0339 capacity inputs; 0341 consumes 0340; 0343 consumes 0340; 0342 references 0341). They are mutually internally-consistent and all anchor to the retained canonical posture (Valkey/Iceberg/Kata+Cloud-Hypervisor/Cedar/tenant-class).

---

### ADR-0337 — Apache Iceberg is the canonical OLAP table-format write path

- **decision_atom:** Apache Iceberg 1.7+ is the single canonical OLAP table-format write path corpus-wide; Delta Lake and Hudi are demoted to migration-adapter-only (convert-to-Iceberg-on-commit), with ClickHouse retained as the OLAP compute engine layered on Iceberg (not a parallel write path).
- **domain:** data-storage (cross-cut: data-engine-db).
- **current_status:** Proposed (2026-05-21).
- **disposition:** KEEP (ratify). Sound, hyperscaler-convergent, license-clean (Apache-2.0), strict-Rust binding via `iceberg-rust`. No conflict with retained posture.
- **proposed_resolution:** RATIFY — it is the explicit user-directive Decision 1, technically correct, and the canonical posture map already treats best-of-breed managed substrates as TRUE; Iceberg-as-table-format slots cleanly under data-storage with no contradiction.
- **governing:** n/a (not archived).
- **truth_flag:** TRUE.
- **in_masterplan:** YES (planning_impact: true; amends dependency-policy §7, ADR-0211 allow-list, ADR-0212, ADR-0328 wave sequence; binds `substrate_dependencies`).
- **tensions:** Mild internal: it does NOT amend ADR-0192 (Milvus) and explicitly carves vector workloads out — consistent. Against LINUX fault-line #1 (LINUX ADR-0001 "own the DB engine / eliminate Postgres"): this ADR deepens SOURCE's "assemble proven OSS" posture (now even the table format is adopted OSS), widening the own-vs-assemble gap on merge. No source-internal contradiction.
- **hyperscaler_challenge:** ALIGNED. Google/AWS/Azure all converged on Iceberg as the cross-vendor interop format (S3 Tables, BigLake, Synapse, Polaris, UniForm) — this decision literally mirrors what hyperscalers did. Argues for neither amend nor archive.
- **ai_slop / refinement / consensus_needed:** Not slop; 827 lines but substance-dense (real GA dates, real benchmarks). Minor refinement: the "30-day soak → BLOCKER" and "Wave 15-OLAP" machinery is implementation scaffolding that the generated-masterplan need not carry — only the decision_atom binds. No founder question needed.

### ADR-0338 — Pod runtime tier 0..3 (Kata+Cloud-Hypervisor for untrusted/data-plane; runc for first-party/edge)

- **decision_atom:** Every µservice declares a `pod_runtime_tier ∈ {0,1,2,3}` in its manifest: Tier 0 (tenant-untrusted code) and Tier 1 (substrate touching tenant data plane) run under Kata Containers + Cloud Hypervisor; Tier 2 (first-party apps) and Tier 3 (edge/perf-critical) run under runc, enforced by Kyverno admission + RuntimeClass binding.
- **domain:** isolation-runtime (cross-cut: node-os).
- **current_status:** Proposed (2026-05-21).
- **disposition:** KEEP (ratify). It is an explicit, well-justified carve-out (amendment) of ADR-0254's "Kata everywhere" reading, restoring 30–40% pod density on trusted code while keeping VM-isolation where tenant code executes.
- **proposed_resolution:** RATIFY — sound cost/security trade with named hyperscaler precedent (Fargate/Firecracker, Cloud Run/gVisor, ACA/Hyper-V); reuses ADR-0248 tier numbering and ADR-0183 Kyverno-admission split correctly.
- **governing:** n/a.
- **in_masterplan:** YES (planning_impact; amends ADR-0254; adds manifest field + Kyverno policy + 4 governance lanes).
- **truth_flag:** TRUE.
- **tensions:** (1) **Tier-numbering overload** — reuses ADR-0248 "Tier 0..4" cellular axis as a separate "Tier 0..3" pod-runtime axis; ADR-0340 then adds a THIRD "cell_placement_class Tier-0..4". Three co-varying tier axes with overlapping names is a real onboarding/clarity hazard (0340 §B2.008 and 0341 §D-1 spend paragraphs disambiguating). (2) Cites `oyatie.foundry.*` Cedar principal namespace (B2.039) and ADR-0255 titled "intelligence-as-two-layer" in related_adrs while ADR-0247 self-modification still uses `foundry` principal strings — this is the known retired-`foundry`-brand residue (keystone §2; ADR-0335/0347). The principal-namespace string `oyatie.foundry.*` is retired-vocab leakage. (3) Depends on Kata/Cloud-Hypervisor which is SOURCE canon, but collides with LINUX fault-line #3 (Capsule/framekernel "we are the host, no separate containerd").
- **hyperscaler_challenge:** ALIGNED. The trusted-vs-untrusted runtime split is exactly what AWS/Google/Azure do (Firecracker/gVisor/Hyper-V for tenant code, plain containers for first-party). No amend/archive pressure on the decision; only the `foundry` principal-string is stale.
- **ai_slop / refinement / consensus_needed:** Not slop (1036 lines, concrete capacity math). Refinement: rename `oyatie.foundry.*` principal references to the intelligence/governance vocabulary per ADR-0335/0347 before this binds. Consensus_needed (minor): "Do we keep three separately-numbered Tier axes (pod-runtime, cellular, cell_placement_class), or unify the naming to stop the recurring disambiguation tax?"

### ADR-0339 — Shared IaC module library (cloud-iac/modules canonical; per-µservice iac is a thin wrapper)

- **decision_atom:** Reusable OpenTofu IaC primitives live canonically at `microservices/cloud-iac/modules/<context>/<primitive>/` (5 contexts + OCI always-free sub-context), cosign-signed and version-pinned; each µservice's `iac/<context>/main.tf` becomes a ≤80-LOC thin invocation wrapper, collapsing the 77×5=385 from-scratch-module blast-radius to ~50 shared primitives + 385 wrappers.
- **domain:** ci-cd-build (cross-cut: orchestration-scheduling).
- **current_status:** Proposed (2026-05-21).
- **disposition:** KEEP (ratify). Direct hyperscaler-precedent shape (AWS Solutions Constructs / GCP Cloud Foundation Toolkit / Azure Verified Modules); OpenTofu-only + cosign per existing canon.
- **proposed_resolution:** RATIFY — drift-containment and supply-chain win; matches `feedback_zero_handroll_opentofu_only`. No reason to drop.
- **governing:** n/a.
- **in_masterplan:** YES (planning_impact; amends ADR-0216/0212/0211/0218/0331/0254/0336; adds 7 lanes + catalog).
- **truth_flag:** TRUE.
- **tensions:** (1) **Stale on-prem substrate** — Anchor 7 / B2 encode `on-prem`/`colo` modules as "kubeadm + Cilium + Istio-ambient + Envoy-gateway + Kata + Cloud-Hypervisor". Keystone §3 says the canonical orchestration posture is now **Talos + CAPI + ArgoCD** (ADR-0375 supersedes ADR-0121's kubeadm stack). This ADR's amend-list cites `ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor.md` and bakes kubeadm into the module library — that is STALE against ADR-0375. (2) It re-amends ADR-0331's six-context taxonomy down to five — a coherent in-cluster reconciliation but a corpus-wide naming change others must track. (3) References `oyatie-as-cloud-provider/per-cell-nodepool-kata` etc., inheriting ADR-0338's runtime tiers.
- **hyperscaler_challenge:** ALIGNED on the library shape (every hyperscaler ships a signed, versioned module library). PARTIALLY QUESTIONABLE on the encoded substrate: a hyperscaler would not pin kubeadm+Istio-ambient as the canonical on-prem stack if it had already chosen Talos — argues for an AMEND of the on-prem/colo module substrate to Talos+CAPI per ADR-0375, not an archive of the ADR.
- **ai_slop / refinement / consensus_needed:** Not slop. Refinement (real): update the on-prem/colo canonical-stack language from kubeadm/Istio to Talos+CAPI+ArgoCD to align with ADR-0375 before this binds into the masterplan. Consensus_needed: none on the decision itself.

### ADR-0340 — Capacity model per microservice manifest

- **decision_atom:** Every workload-producing µservice declares a `capacity_model` manifest block (`baseline_cpu_per_tenant`, `baseline_ram_per_tenant`, `storage_per_tenant`, `connections_per_tenant`, `scaling_dimension` enum, `cell_placement_class` Tier-0..4) as the single machine-readable source of truth feeding autoscaler, cell-sizing, FinOps projection, and shuffle-sharding/blast-radius determinism.
- **domain:** finops-cost (cross-cut: orchestration-scheduling).
- **current_status:** Proposed (2026-05-21).
- **disposition:** KEEP (ratify). Closes the "every capacity surface reverse-infers from telemetry" gap that ADR-0338 §D-7 and ADR-0339 §D-3 both explicitly left open. Hyperscaler-precedent (AWS Service Quotas / GCP Quotas / Azure Limits / Salesforce Governor Limits).
- **proposed_resolution:** RATIFY — additive manifest schema, well-bounded enums, unblocks 0341/0343/0344.
- **governing:** n/a.
- **in_masterplan:** YES (planning_impact; amends ADR-0212/0244/0245/0248/0331/0338/0339; 7 lanes + Kyverno policy; schema fragment is in-scope for the ADR).
- **truth_flag:** TRUE.
- **tensions:** (1) **Third tier axis** — introduces `cell_placement_class Tier-0..4`, a third independently-declared tier dimension alongside ADR-0248 cellular tier and ADR-0338 pod_runtime_tier; the ADR itself spends B2.008 + D-6 disambiguating, confirming the overload risk flagged under ADR-0338. (2) `connections_per_tenant` hard-codes exactly three substrate kinds (`valkey`, `postgres`, `outbound_http`) — coherent with Valkey/Postgres canon (ADR-0336/0179) but a closed shape that would need amendment if a new connection-bearing substrate (e.g., Pulsar per ADR-0377) becomes per-tenant-pooled; Pulsar is notably absent. (3) `scaling_dimension` closed enum omits `per_token` (the ADR self-flags this for LLM-bound µservices in C.2) — a known gap for the intelligence substrate.
- **hyperscaler_challenge:** ALIGNED. Declared per-tenant capacity model is exactly the hyperscaler quota pattern. No amend/archive pressure; the `per_token` omission is a minor forward-amend candidate.
- **ai_slop / refinement / consensus_needed:** Not slop. Refinement: consider adding `pulsar` to `connections_per_tenant` and `per_token` to `scaling_dimension` (both are foreseeable closed-enum amendments). Consensus_needed: same tier-naming-unification question as ADR-0338 (this ADR is the one that adds the third axis).

### ADR-0341 — Cellular promotion gates (per-Tier 0..4 machine-checkable criteria + auto-promotion via cell-orchestrator)

- **decision_atom:** Every cellular tier promotion/demotion (ADR-0248 Tier 0..4) is gated on six continuously-evaluated machine-checkable inputs (error-budget ≥99%, per-edge warm-soak 7/14/28/56d, canary ≥99.5%, cross-cell call success ≥99.95%, demo_trial+paid both present, all applicable compliance packs signed off) plus a per-edge quiet window, auto-executed by the cell-orchestrator µservice, which emits a signed audit-chain promotion event and mutates cell-tier node labels via Kyverno admission.
- **domain:** orchestration-scheduling (cross-cut: observability).
- **current_status:** Proposed (2026-05-21).
- **disposition:** KEEP (ratify) — but with a flagged self-acknowledged semantic ambiguity (see tensions/consensus).
- **proposed_resolution:** RATIFY the gate-contract; the ADR is a "wiring decision, not a substrate decision" (all six inputs already have canonical substrates). One drafting issue must be resolved before BLOCKER (the promotion-direction reading).
- **governing:** n/a.
- **in_masterplan:** YES (planning_impact; amends ADR-0248/0244/0212/0263/0251/0148/0186/0044/0328; adds 5 lanes + Kyverno policy + manifest fields).
- **truth_flag:** PARTIAL. The gate set, thresholds, and control-plane shape are TRUE; but **D-1.4 through D-1.6 contain a genuinely confusing/possibly-WRONG framing**: the user directive's "Tier 0 → 1 → 2 → 3 → 4 with 7/14/28/56d warm-soak" is reinterpreted as *demotion* (Tier 0→1 = relaxing to less-critical), then the inverse promotion edges are given mirrored soaks (Tier 4→3 = 56d). The ADR labels this "the canonical reading" but the logic ("longer soak when destination is LESS critical") is counter-intuitive and the document visibly strains to reconcile it. This is the one place a careful reader will distrust the spec.
- **tensions:** (1) **`oyatie.foundry.*` Cedar principal** (B2.012, D-4.5) — retired-brand residue per ADR-0335/0347; the cell-orchestrator should run under the intelligence/governance principal namespace, not `foundry`. (2) **Implementation-heavy for an immutable SSOT** — the cell-orchestrator µservice does not yet exist (anchored in ADR-0148, deferred); this ADR is a contract for software that is wholly future. (3) Depends on ADR-0340's `cell_placement_class` and reuses the third tier axis. (4) ADR-0186 canary discipline and ADR-0044 mesh SLO are elevated from "advisory" to "gate input" — a real tightening others must honor.
- **hyperscaler_challenge:** ALIGNED on the mechanism (AWS "cell graduation", Stripe "Pier", Cloudflare "Argo Tier Manager" are cited and the six-input continuous-evaluation shape is genuinely what they do). QUESTIONABLE on the direction-of-promotion semantics (no hyperscaler frames "promotion" as relaxing toward edge with a longer soak — the inverted reading is an Oyatie-specific artifact of mapping the user directive onto ADR-0248's descending-criticality numbering). Argues for an AMEND of D-1.4–D-1.6 wording, not archive.
- **ai_slop / refinement / consensus_needed:** Not slop, but the promotion-direction section is the weakest writing in the chunk. **consensus_needed (crisp founder question):** "In ADR-0341, does 'Tier 0 → 1' mean DEMOTING a cell to a less-critical tier (the ADR's current reading) or PROMOTING it toward foundation? Pick one direction and one warm-soak rule so the gate is unambiguous." Resolve before BLOCKER.

### ADR-0342 — API versioning HYBRID model (date-based for public APIs + semver for SDK packages)

- **decision_atom:** Public APIs (OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3) use date-based versions `YYYY-MM-DD` carried via three canonical channels (`Oyatie-Version` header + `/v/<date>/` URL prefix + `oyatie_version` proto field); the 10-language generated SDKs use semver (each SDK pins a date under the hood); last N=3 public versions supported ≥180 days post-deprecation; per-tenant version pinning in the tenant manifest; every breaking change requires a paired sunset ADR + RFC 8594/9745 headers + audit-chain `api.version.*` events.
- **domain:** api-contracts (cross-cut: product-ux for SDK DX).
- **current_status:** Proposed (2026-05-21).
- **disposition:** KEEP (ratify). Mirrors Stripe/Anthropic/OpenAI/AWS/GitHub date-on-boundary + semver-on-SDK convention exactly; correctly restricts the version triplet to the public boundary (not internal ADR-0145 mesh).
- **proposed_resolution:** RATIFY — strong hyperscaler precedent, clean separation of public-vs-internal, standards-compliant deprecation carrier.
- **governing:** n/a.
- **truth_flag:** TRUE — **except one stale reference**: `amends:` lists `ADR-0316-tier-system-canonical-bronze-silver-gold-platinum.md` and annotates it "(RETIRED)". ADR-0316 IS retired/superseded by ADR-0329 (keystone §1.1/1.2). Amending a retired ADR is odd but here it's used correctly (to assert API versioning does NOT depend on the dead capability-tier ladder, deferring to tenant-class per ADR-0330). The reference is defensible as a "do-not-resurrect" note but should cite ADR-0329 as the governing retirement, not treat 0316 as an amendable live target.
- **in_masterplan:** YES (planning_impact; amends ADR-0145/0211/0212/0216/0218/0244/0263/0316; 8 lanes; binds api-gateway + developer-sdk).
- **tensions:** (1) The ADR-0316 amend (see truth_flag) is the only retired-vocab touch — low severity, self-aware. (2) Depends on the developer-sdk Stainless-class generator (a memory directive, `feedback_developer_sdk_stainless_generator`, not yet an ADR) — large future implementation surface (~25k LOC estimated). (3) Reserves proto tag 8001 corpus-wide — a real cross-µservice coordination commitment.
- **hyperscaler_challenge:** ALIGNED — this is the single most hyperscaler-conventional ADR in the chunk; the date+semver hybrid is precisely the Stripe/Anthropic/AWS pattern. No amend/archive pressure on the decision.
- **ai_slop / refinement / consensus_needed:** Not slop. Refinement: re-point the `amends: ADR-0316` entry to reference ADR-0329 as the governing retirement (treat 0316 as archived, cite the successor). No founder question needed.

### ADR-0343 — DR + RTO/RPO matrix per-µservice + per-compliance-pack

- **decision_atom:** DR is declared at two layers — a per-µservice `manifest.json#dr` block (numeric `rto_p99_seconds`/`rpo_p99_seconds` + `multi_region_active_active` + `backup_substrate` allowlist + `failover_runbook`) and a per-compliance-pack floor in `/specs/compliance-pack-floors.json` (8 initial packs) — and the effective per-tenant contract is their more-stringent combination (MIN over RTO/RPO seconds, OR over multi-region), enforced at pack-activation admission by a Cedar fragment + auditor dashboard.
- **domain:** dr-resilience (cross-cut: compliance-residency).
- **current_status:** Proposed (2026-05-21).
- **disposition:** KEEP (ratify). Two-dimensional DR (µservice baseline × pack floor) is correct and cost-disciplined; the 77×8 pair surface collapses to 77+8 declarations + one algorithm.
- **proposed_resolution:** RATIFY — preserves ADR-0241's T1..T4 shorthand, adds the pack-floor overlay, machine-evaluable and auditor-visible (AWS Audit Manager / GCP Compliance Reports / Azure Purview precedent).
- **governing:** n/a.
- **in_masterplan:** YES (planning_impact; amends ADR-0028/0158/0212/0241/0251/0263; 8 lanes; authors a new spec `/specs/compliance-pack-floors.json` in-scope).
- **truth_flag:** TRUE — with one **terminology landmine the ADR self-flags**: the authority memory says "effective = MAX(µservice, pack floors)" but the numeric realization is MIN over seconds (smaller seconds = stricter). The ADR explicitly reconciles this (§B.1/§D-4: "max-of-stringency = min-of-upper-bound-seconds"), so it is correct but invites misreading; any downstream tool that literally takes `max()` of the seconds would be WRONG.
- **tensions:** (1) `backup_substrate` allowlist is the cleanest cross-substrate-canon consolidation in the chunk and is correctly Valkey/Iceberg/Postgres-WAL-G/SeaweedFS-aware — **no Redis, no Kafka** leakage (good; Pulsar/eventing is simply out of DR-substrate scope here). (2) Couples to ADR-0341 (B2.053: Tier-0/1/2 cell promotion requires DR-floor satisfaction) — a real ordering dependency. (3) Pack floors restate regulator citations (HIPAA/PCI/SOC2/EU-AI-Act/CSAP/ISO/SOX/KR-PIPA) with specific second-values — these are policy assertions a compliance reviewer (not an architecture auditor) must ratify. (4) `oyatie.foundry.*` appears once (Anchor 9, self-modification recursive coverage) — same retired-brand residue.
- **hyperscaler_challenge:** ALIGNED. Per-pack machine-readable DR floors + auditor dashboards is exactly AWS Audit Manager / Azure Purview Compliance Manager shape. No amend/archive pressure on the decision; the MAX-vs-MIN phrasing is a documentation-clarity fix.
- **ai_slop / refinement / consensus_needed:** Not slop (954 lines, genuine regulator mapping). Refinement: lead with the numeric MIN-over-seconds rule and demote the "MAX-of-stringency" prose to a footnote so no implementer mis-codes it. Consensus_needed: the eight per-pack second-values need a compliance/legal sign-off pass (council-legal is already an owner) — not an architecture-auditor call.

---

## Chunk notes

**Overall verdict.** All 7 ADRs are KEEP/RATIFY. This is a coherent, high-quality 2026-05-21 doctrine cluster — substance-dense (827–1036 lines each), grounded in real hyperscaler precedent with concrete GA dates and benchmarks, and self-consistent. None is garbage, none is superseded, none conflicts with another within the chunk. They are exactly the kind of "true + relevant decision" the founder wants backfilled into the generated masterplan. Every one carries `planning_impact: true` and binds a manifest/spec/lane surface, so all are IN the masterplan.

**Cross-cutting issues to surface (not blockers, but pre-BLOCKER cleanups):**

1. **Retired `foundry` brand residue (recurring).** `oyatie.foundry.*` Cedar principal namespace appears in ADR-0338 (B2.039), ADR-0341 (B2.012, D-4.5), and ADR-0343 (Anchor 9), plus ADR-0337's related-list cites `ADR-0138-foundry-six-path-deprecation`. Per keystone §2 + ADR-0335/0347, the `foundry` brand is RETIRED (→ intelligence/governance). These principal strings are retired-vocab leakage that should be renamed before any of these ADRs promote to BLOCKER. None of them depends on `foundry` *meaning* anything — it is purely a stale principal label.

2. **Three overlapping "Tier 0..N" axes.** The chunk operationalizes three independently-numbered tier dimensions that all start at "Tier 0 = most critical": ADR-0248 cellular tier (0..4), ADR-0338 pod_runtime_tier (0..3), ADR-0340 cell_placement_class (Tier-0..4). ADRs 0338/0340/0341 each spend prose disambiguating them. This is the single biggest clarity tax in the cluster and a worthwhile founder/architecture decision: unify the naming (e.g., distinct prefixes) or accept the disambiguation cost permanently.

3. **ADR-0339 stale on-prem substrate.** It bakes kubeadm + Cilium + Istio-ambient + Envoy-gateway as the canonical on-prem/colo module stack, contradicting keystone §3 / ADR-0375 (Talos + CAPI + ArgoCD supersedes ADR-0121's kubeadm stack). This is the clearest STALE-substrate item in the chunk — AMEND the on-prem/colo module language to Talos+CAPI before binding.

4. **Two documentation-clarity landmines (both self-flagged by the ADRs, both invite WRONG downstream code):** ADR-0341's promotion-direction reading (Tier 0→1 = demote? promote?) and ADR-0343's "MAX(stringency) = MIN(seconds)" inversion. Neither is a wrong *decision*, but both are wrong-prone *specifications*. ADR-0341 warrants an explicit founder one-liner to fix the direction; ADR-0343 just needs the numeric rule led first.

5. **Retired-ADR amend (ADR-0342).** ADR-0342 `amends: ADR-0316` (a retired/superseded-by-0329 ADR), used correctly as a "does-not-depend-on-the-dead-tier-ladder" note. Re-point to cite ADR-0329 as governing.

6. **Masterplan authored-vs-generated (keystone §4, OPEN).** Under BOTH readings these seven carry real planning_impact and belong in the masterplan: under "ADRs generate the masterplan" they are immutable SSOT entries to ratify from Proposed→Accepted; under "masterplan is authority, ADRs bind in" they each need a `masterplan_ref` binding (none present — consistent with the 8.8% binding rate noted in the keystone). Flagged under both readings per instruction; no assumption made.

**No ADRs in this range are ARCHIVE/SUPERSEDE/MERGE/UNCLEAR.** No garbage. The only WRONG-prone content is documentation phrasing (0341 direction, 0343 MAX/MIN), not decisions.
