# ADR Audit — SOURCE chunk 20

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 20
- **Slice requested:** `ls | sort | sed -n "134,140p"` → ADR-0159 … ADR-0165
- **ADRs actually reviewed (7):** ADR-0159, ADR-0160, ADR-0161, ADR-0162, ADR-0163, ADR-0164, ADR-0165
- **Cluster identity:** This is a contiguous "cloud-substrate / SaaS-platform-completeness" run, all dated 2026-05-18, all `owner: council-architecture`, all `status: Accepted`, all `supersedes:[] superseded_by:[]`. They are operational-fabric ADRs (feature-flags, progressive delivery, storage classes, audit slicing, env tiers, sovereign air-gap, chaos) that pin *concrete OSS tooling* onto earlier abstract contracts. None carries `masterplan_ref` / `planning_impact` / `deliverables` front-matter — the whole chunk is **unbound to the masterplan** (consistent with the keystone map's 8.8%-binding finding).

---

### ADR-0159 — Dedicated Feature-Flag µservice (runtime gradual rollout) separate from ChangeSet acceptance
- **decision_atom:** Runtime feature flagging is a first-class, OpenFeature-compliant `feature-flags` µservice with per-tenant/per-cohort Cedar-predicate targeting, per-cell active-active deployment, mandatory flag lifecycle (`sunset_at` + CI cleanup gate), and audit-chain seals on flag *definition* changes — orthogonal to code-deploy gating and traffic-shape gating.
- **current_status:** Accepted (2026-05-18).
- **disposition:** AMEND.
- **governing:** Self-governing as the runtime-gate decision, but **depends on retired/stale anchors**: ADR-0110 (ChangeSet state machine) is **superseded by ADR-0363** (retire agentic-VCS; plain git + Forgejo PRs). The ADR's whole framing ("three gating tiers: ChangeSet code-deploy / Flagger traffic / feature-flag runtime") leans on ADR-0110's `acceptance_status` lifecycle that no longer exists as authored.
- **truth_flag:** PARTIAL — the *runtime-flag* decision is TRUE and durable; the *ChangeSet-as-code-deploy-gate* leg is STALE (ADR-0110 retired).
- **in_masterplan:** NO — no `masterplan_ref`; OpenFeature/feature-flag substrate not reflected in MASTERPLAN.md. Companion `/specs/feature-flag-substrate-canonical.json` is the only binding artifact.
- **tensions:**
  - ADR-0110 (related + load-bearing) — retired by ADR-0363; the code-deploy-gate concept must be re-expressed as "branch → PR against `dev` → required checks" (the masterplan's own promotion model), not ChangeSet `acceptance_status`.
  - ADR-0007 (Cedar authz) — ADR-0159 *reuses the Cedar evaluator* for flag predicates while Alternative E explicitly rejects "Cedar policy AS the flag substrate." Subtle but coherent (same engine, different surface); worth a one-line clarification so future readers don't read it as contradiction.
  - "persona_tier" / "persona-tier" targeting dimension — uses the word *tier*; this is NOT the retired tenant tier-system (ADR-0329) and NOT autonomy-tier; a third overloaded use of "tier." Naming-hygiene flag.
- **hyperscaler_challenge:** ALIGNED. Google/AWS/Azure all run a runtime flag plane distinct from deploy gating (LaunchDarkly/AppConfig/OpenFeature). Building it in-house rather than buying LaunchDarkly is justified by the sovereign-cell containment requirement (ADR-0049/0164) — a real constraint a hyperscaler-for-regulated-markets would also hit. Verdict argues KEEP-the-decision / AMEND-the-refs.
- **ai_slop:** Mild. Fabricated-precision smell: "~0.5-1 ms added," "30-sec TTL," "<1 ms p99," "5 sec eventual consistency" are asserted without source. The Martin-Fowler taxonomy citation is legitimately load-bearing (it is the actual justification for orthogonality). Otherwise low slop.
- **refinement:** (1) Replace every ADR-0110 reference with the masterplan promotion model or a successor ADR id. (2) Add `masterplan_ref` + `planning_impact` front-matter. (3) Drop or source the latency micro-numbers. (4) De-conflict "persona_tier" naming vs retired "tier."
- **consensus_needed:** no (decision is sound; only ref-reconciliation needed).

### ADR-0160 — Progressive Delivery via Flagger 1.x (canary + blue-green + A/B), ArgoCD-integrated, SLO-gated
- **decision_atom:** Flagger 1.x is the canonical progressive-delivery controller: one `Canary` CRD per workload, Istio `VirtualService` weight management, ArgoCD-applied, with first-class PromQL/SLO gates (ADR-0139) and automatic SLO-breach rollback, scoped per cell.
- **current_status:** Accepted (2026-05-18).
- **disposition:** AMEND (technically borderline ARCHIVE-the-rationale).
- **governing:** Decision stands, but its *comparative rationale* is now inverted by canon. Map §3 CI/CD: ArgoCD/**Argo-Rollouts** is the canonical CD layer. ADR-0160 explicitly **rejects Argo Rollouts (Alternative A)** in favor of Flagger. Also leans on ADR-0121 (onprem k8s) which is **superseded by ADR-0375 (Talos+CAPI+ArgoCD)**; the "Istio already in-cluster per ADR-0148" premise is also a later-contested mesh choice.
- **truth_flag:** PARTIAL — Flagger-as-controller is a defensible TRUE choice, but the keystone canonical-posture row names Argo-Rollouts in the CD slot, so this ADR is in **unflagged tension with current canon**; "ADR-0121 in-stack" justification is STALE.
- **in_masterplan:** NO — no binding front-matter; progressive-delivery tooling not surfaced in MASTERPLAN.md.
- **tensions:**
  - **Flagger vs Argo-Rollouts** — Map §3 lists "ArgoCD/Argo-Rollouts (CD)" as canonical; ADR-0160 picks Flagger and demotes Argo-Rollouts. Either the map's CD row is loose shorthand or ADR-0160 needs a supersession/reconciliation. **Load-bearing contradiction to surface.**
  - ADR-0121 (onprem k8s) — superseded by ADR-0375; "ArgoCD already in-stack per ADR-0121" should re-point to ADR-0375.
  - ADR-0120 (rust-first toolchain) cited to reject Spinnaker — ADR-0120 itself superseded by ADR-0375 (the rust-first principle may survive but the citation is stale).
  - ADR-0148 (Istio) — Flagger's "first-class Istio" is the whole differentiator; if Istio is later swapped (Cilium/Linkerd mentioned as deferred), the dispositive reason collapses.
- **hyperscaler_challenge:** QUESTIONABLE. Google/AWS would absolutely run SLO-gated progressive delivery — but the *specific* Flagger-over-Argo-Rollouts call is contestable, and a shop already standardizing on the Argo ecosystem (ArgoCD + Argo Workflows per ADR-0511) would more likely pick Argo Rollouts for ecosystem coherence. Argues for AMEND (re-justify against the now-Argo-centric stack) or a reconciliation ADR.
- **ai_slop:** Low-moderate. "rollback within 30 seconds; total worst-case impact < 6 minutes," "~50 ms per query" are fabricated-precision. The "Why Flagger over Argo Rollouts" section is genuine engineering reasoning, not filler.
- **refinement:** (1) Re-evaluate Flagger vs Argo-Rollouts now that Argo Workflows (ADR-0511) + ArgoCD are the canonical CI/CD spine — ecosystem-coherence argument may now flip the decision. (2) Re-point ADR-0121/0120 citations to ADR-0375/0392. (3) Add masterplan binding.
- **consensus_needed:** yes — **"Is the canonical progressive-delivery controller Flagger (ADR-0160) or Argo-Rollouts (keystone §3 CD row)? Pick one; the stack is now Argo-centric."**

### ADR-0161 — CSI Driver + StorageClass Abstraction (`oya-{pg,s3,redis,object}-{hot,warm,cold}`; per-pack CSI pin)
- **decision_atom:** Workloads reference cloud-agnostic canonical StorageClass names `oya-<kind>-<tier>`; each regional pack pins those names to a concrete CSI driver, with mandatory CSI-v1.8+ / encryption-at-rest / VolumeSnapshot / topology-aware / `Retain`-on-hot requirements.
- **current_status:** Accepted (2026-05-18).
- **disposition:** AMEND.
- **governing:** Decision is sound and self-governing for storage abstraction; but the `<kind>` enum hard-codes **`redis`**, and **Redis is RETIRED → Valkey by ADR-0336** (`ADR-0336-valkey-not-redis-substrate.md`, confirmed on disk). The canonical StorageClass name `oya-redis-hot` is therefore retired-vocab leakage baked into a *naming standard* — exactly the kind that propagates. Also leans on ADR-0121 (superseded by ADR-0375) for the portability invariant.
- **truth_flag:** PARTIAL — abstraction pattern TRUE and hyperscaler-correct; the `redis` kind token is STALE (should be `valkey`); ADR-0121 portability-invariant citation STALE.
- **in_masterplan:** NO — `/specs/csi-storage-class-canonical.json` is the only binding artifact; not in MASTERPLAN.md.
- **tensions:**
  - **`oya-redis-*` vs ADR-0336 Valkey** — a canonical, CI-enforced (`oya gate validate storage-class-canonical`) name embeds a retired brand. High propagation risk because it's a *standard*.
  - ADR-0121 portability invariant — re-anchor to ADR-0375 (Talos) / the K8s-everywhere posture.
  - Data-tier cross-side tension (Map fault-line #1): the StorageClass kinds presuppose Postgres+Redis+S3 best-of-breed substrates; LINUX ADR-0001 wants to *eliminate Postgres* and own the DB engine. This standard is squarely on the SOURCE "assemble proven OSS" side.
  - `s3` vs `object` aliasing — the ADR itself admits `object` is "= s3 alias"; redundant enum member, minor.
- **hyperscaler_challenge:** ALIGNED. This is textbook AWS/GCP/Azure practice (workloads reference StorageClass, operator binds CSI; never reference `gp3` directly). The per-pack CSI matrix is exactly how a real multi-cloud platform team operates. Verdict: KEEP-the-pattern, AMEND-the-redis-token.
- **ai_slop:** Low. The 10-row per-pack CSI matrix is concrete and useful, not filler — though several sovereign packs (G42, Sakura, STC) may be fabricated-precision (asserting exact SKU classes for clouds that may not have been validated). Reasonable for an ADR; flag as "verify SKU claims."
- **refinement:** (1) Rename `redis` kind → `valkey` (or generic `kv`) per ADR-0336; add a migration/alias note. (2) Re-point ADR-0121. (3) Collapse `object`/`s3` alias. (4) Verify exotic-pack SKU rows or mark them "TBD-on-onboarding."
- **consensus_needed:** no (mechanical rename + ref fix; not a contested architecture choice).

### ADR-0162 — Per-tenant Audit-Chain Slicing (partition by tenant_id; sovereign dedicated shard; per-tenant retrieval API)
- **decision_atom:** Audit-chain Merkle seals partition by `tenant_id` (per-pack shared tree with per-tenant subtree for multi-tenant packs; dedicated per-tenant Merkle tree + in-region key custody for sovereign packs), exposed via a Cedar-gated, tenant-scoped retrieval API with Merkle inclusion proofs — the CloudTrail-per-account pattern.
- **current_status:** Accepted (2026-05-18).
- **disposition:** KEEP.
- **governing:** Self-governing; cleanly refines ADR-0003 (audit-chain emission). No retired-vocab dependencies. References ADR-0157/0158/0161/0164 which are all live (0161 only via the `oya-s3-cold` storage name).
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL — no `masterplan_ref` front-matter, but per-tenant audit-log slicing IS a keystone tenancy posture (Map §3 Tenancy row cites ADR-0162 directly) and aligns with masterplan FD-001 (Tenant RBAC at production depth). The *decision* is canon-aligned even though the *front-matter binding* is absent.
- **tensions:**
  - Minor: depends on `oya-s3-cold` (ADR-0161) for daily root anchor — inherits ADR-0161's storage-naming health but no Redis token here.
  - Sealing-cadence numbers (100ms append / hourly subtree / daily root) are asserted; should be reconciled with ADR-0003's actual contract, not re-stated.
  - Cross-shard root NOT covering sovereign tenants is an honestly-disclosed limitation (good), but it means "fleet-wide verifiability" is conditional — worth a masterplan note.
- **hyperscaler_challenge:** ALIGNED. This is literally the CloudTrail-per-account / GCP-audit-per-project / Azure-per-subscription model; the per-row-Merkle-tree alternative is correctly rejected as operationally unbounded. A hyperscaler would make exactly this shared-vs-dedicated split. Verdict: KEEP.
- **ai_slop:** Very low. Well-structured; the AWS/GCP/Azure precedent is accurate; RFC 9162 (Certificate Transparency) citation is apt.
- **refinement:** (1) Add `masterplan_ref` to bind into the tenancy keystone. (2) Reference ADR-0003 for cadence numbers rather than re-asserting. Otherwise strong.
- **consensus_needed:** no.

### ADR-0163 — Per-tenant Environment Tiers (test / staging / prod), Cell-Isolated, Stripe `sk_test_` pattern
- **decision_atom:** Every tenant has three cell-isolated environment tiers (`test`/`staging`/`prod`) with prefix-tagged API keys (`sk_test_`/`sk_stage_`/`sk_live_`), per-tier outbound side-effect modes (intercept/test-recipients/live), per-tier audit-chain subtrees, and a Cedar+header-gated destructive-op acknowledgment on prod.
- **current_status:** Accepted (2026-05-18).
- **disposition:** AMEND.
- **governing:** Self-governing as the tenant-environment decision. Two contamination issues: (1) references **ADR-0110 ChangeSet** (retired by ADR-0363) for the promotion model; (2) the "**Foundry isolation**" / "Foundry workflows" sections use the **RETIRED `foundry` brand** (ADR-0335 → cloud-intelligence). Also references ADR-0029 ("hero product / Workflow Studio") which is unverified in this slice.
- **truth_flag:** PARTIAL — the three-tier + Stripe-key-prefix decision is TRUE and durable; the `foundry` naming is STALE; the ChangeSet promotion reference is STALE.
- **in_masterplan:** NO — confirmed not present in MASTERPLAN.md (grep negative). `/specs/tenant-environment-tiers-canonical.json` is the only binding artifact. **Naming-collision risk for the masterplan:** the title says "Environment **Tiers**" while ADR-0329 *retired* "tier/tier-system." These are genuinely different axes (env-tier ≠ tenant-class), but a flat masterplan index keyed on "tier" will collide. Map §2 already warns tier is overloaded (tenant-class / autonomy-tier / now env-tier / persona-tier).
- **tensions:**
  - **"tier" overload** — env-tier vs retired tenant tier-system (ADR-0329) vs autonomy-tier T1–T4 vs persona-tier (ADR-0159). Four distinct "tier" axes. Founder masterplan-as-SSOT will need disambiguated vocabulary.
  - **Foundry brand** (ADR-0335 retired) appears in the isolation/cost-budget sections — must become cloud-intelligence.
  - ADR-0110 ChangeSet — retired (ADR-0363); promotion-model reference must be reframed.
  - ADR-0162 alignment: claims per-tier audit subtree via `(tenant_id, env_tier)` — consistent with ADR-0162's `(tenant_id)` partition (env_tier as secondary). Coherent, good.
- **hyperscaler_challenge:** ALIGNED. Stripe test/live key-prefix isolation is the canonical SaaS pattern; AWS/GitHub/Vercel all do per-environment isolation. The structural api-gateway-prefix-routing enforcement (a `sk_test_` request can never reach prod schema) is exactly what a hyperscaler-grade platform builds. Verdict: KEEP-the-decision, AMEND-the-naming.
- **ai_slop:** Low. Real precedent table (Stripe/Twilio/AWS/GitHub/Vercel) is accurate and load-bearing. "3× storage baseline," "90-day TTL" are mild fabricated-precision but reasonable defaults.
- **refinement:** (1) Rename "Foundry" → cloud-intelligence throughout. (2) Reframe ChangeSet/ADR-0110 promotion reference. (3) Consider renaming "Environment Tiers" → "Environment Stages/Modes" to escape the retired-"tier" collision before masterplan backfill. (4) Add masterplan binding.
- **consensus_needed:** yes — **"Given ADR-0329 retired 'tier,' should per-tenant test/staging/prod be called 'environment tiers' or renamed (e.g. 'environment stages') to keep masterplan vocabulary unambiguous?"**

### ADR-0164 — Sovereign Cloud / Air-Gapped Deployment (per-pack variant; on-prem registry/Bao/audit-shard/no egress)
- **decision_atom:** A per-pack `air_gap: true` overlay swaps every external dependency for an in-cell equivalent — Harbor registry, OpenBao secrets (HSM-sealed), in-cell audit-chain shard, **forbidden external LLM egress** with vLLM/Ollama on-prem open-weight models, in-cell observability, in-region CI runners — to satisfy KSA NCA / KR FSC / EU sovereign / US-Gov regulators.
- **current_status:** Accepted (2026-05-18).
- **disposition:** AMEND.
- **governing:** Self-governing as the air-gap decision (and it is a genuine market-enabling keystone). Contamination: (1) **`foundry` / `foundry-providers` RETIRED brand** (ADR-0335 → cloud-intelligence) used throughout the no-egress LLM section and the `axis-foundry` decider; (2) **ADR-0121 (onprem k8s) superseded by ADR-0375 (Talos)** — the "kubeadm+containerd+Istio+Envoy baseline" premise is stale; (3) the "in-region **GitHub Actions** self-hosted runners" CI-runner option (§f) is in the retired-CI churn lane (GitHub Actions → Jenkins → Argo Workflows per ADR-0359/0511); (4) Kyverno admission (§a) is superseded by **Kubewarden (ADR-0379)** per Map §3.
- **truth_flag:** PARTIAL — the air-gap/sovereign decision is TRUE and strategically load-bearing; multiple named tools (foundry brand, kubeadm baseline, GitHub Actions runner, Kyverno) are STALE.
- **in_masterplan:** NO — `/specs/sovereign-cloud-air-gapped-canonical.json` only. Sovereign air-gap is a major market commitment that *should* be a masterplan FD but isn't bound.
- **tensions:**
  - **Foundry brand** (ADR-0335) — `foundry-providers`, `microservices/foundry/`, `axis-foundry` decider all retired → cloud-intelligence.
  - **ADR-0121 → ADR-0375** — onprem baseline citation stale; Talos is the immutable node-OS now.
  - **Kyverno → Kubewarden (ADR-0379)** — §a names Kyverno admission; canon is Kubewarden default.
  - **GitHub Actions runner (§f)** — collides with both the CI-churn retirement (Argo Workflows) and the forge fault-line (founder GitHub directive vs Forgejo/bespoke-VCS canon). A sovereign in-region *self-hosted runner* is defensible regardless of CI engine, but the literal "GitHub Actions" naming is retired-vocab.
  - OpenBao / Harbor / vLLM / Cosign-SLSA-L3 are all live and well-chosen — the substrate selection is good; only the cross-refs rotted.
  - LINUX side resonance: air-gap "own the substrate" instinct aligns with LINUX's OWN_DAY0 posture (Map fault-line #5) — but here SOURCE assembles OSS (Harbor/OpenBao/vLLM), staying on the "assemble proven OSS" side.
- **hyperscaler_challenge:** ALIGNED (strongly). AWS GovCloud / Azure Government / Google Assured Workloads are exactly this pattern; every serious cloud serving regulated markets builds air-gap variants. The per-pack overlay granularity (rejecting a single global toggle) is the correct hyperscaler call. Verdict: KEEP-the-decision, AMEND-the-tooling-refs.
- **ai_slop:** Low-moderate. The regulator citations (KR FSC 전자금융감독규정, KSA NCA ECC-1, BSI C5, SecNumCloud) are specific and plausibly real — but **`[Bominal-inheritance precedence]` in Alternative A is an unexplained/likely-fabricated internal reference** (dangling token, no definition) — flag as fabricated precision / dangling cite. Pack-matrix model choices (HyperCLOVA-X, Falcon/G42) are concrete and plausible.
- **refinement:** (1) Rename foundry → cloud-intelligence everywhere. (2) Re-point ADR-0121→0375, Kyverno→Kubewarden(0379). (3) Reframe "GitHub Actions self-hosted runner" as "in-region CI runner (Argo Workflows / bespoke oya-ci)" to dodge both CI-churn and forge fault-line. (4) Resolve or delete `[Bominal-inheritance precedence]`. (5) Promote to a masterplan FD (market-enabling).
- **consensus_needed:** yes — **"Is sovereign/air-gapped deployment a committed masterplan FD (market-enabling, like FD-001), or an optional per-pack capability? It is currently unbound despite gating entire markets."**

### ADR-0165 — Chaos Engineering Substrate (Chaos Mesh 2.x; SLO-driven nightly drills against staging)
- **decision_atom:** Chaos Mesh 2.x is the canonical chaos substrate; every µservice with production SLOs ships a minimum chaos-scenario catalog (pod-kill/network-delay/dependency-failure/disk-slow/time-skew) run nightly against staging, with SLO breach during a drill as a hard release blocker emitting a `ChaosScenarioFailed` audit seal.
- **current_status:** Accepted (2026-05-18).
- **disposition:** AMEND.
- **governing:** Self-governing as the chaos decision; sound technical choice. Contamination: (1) **nightly drill driven by a `.github/workflows/chaos-nightly.yml` GitHub Actions workflow** — squarely in the retired CI lane (GitHub Actions → Argo Workflows per ADR-0511); (2) ADR-0121 onprem-k8s portability-invariant citation stale (→ ADR-0375); (3) references `microservices/foundry/` (foundry RETIRED → cloud-intelligence) as a production-chaos opt-in example.
- **truth_flag:** PARTIAL — chaos-engineering-as-first-class-lane and Chaos-Mesh-selection are TRUE; the GitHub-Actions scheduler and foundry example are STALE.
- **in_masterplan:** NO — `/specs/chaos-engineering-substrate-canonical.json` only.
- **tensions:**
  - **GitHub Actions chaos-nightly workflow** vs Argo Workflows canon (ADR-0511) — the *scheduler* should be Argo Workflows/CronWorkflow, not GitHub Actions, to match the CI spine.
  - **Litmus vs Chaos Mesh** — ADR-0165 itself notes Litmus is CNCF-*graduated* (higher maturity than Chaos-Mesh-incubating) and has "good Argo Workflows integration"; given the stack is now Argo-centric (ADR-0511) + Argo-CD, the Litmus-over-Chaos-Mesh case is arguably stronger now than when authored. Worth re-evaluation (parallels the Flagger-vs-Argo-Rollouts tension in ADR-0160).
  - ADR-0121 → ADR-0375 ref rot.
  - foundry → cloud-intelligence.
  - ADR-0160 dependency ("Chaos Mesh Workflow CRD composes with Flagger") inherits ADR-0160's own Flagger-vs-Argo-Rollouts uncertainty.
- **hyperscaler_challenge:** ALIGNED. Netflix Simian Army / AWS FIS / Google DiRT are exactly this; continuous failure injection with SLO gates is hyperscaler-standard. The *which-tool* (Chaos Mesh vs Litmus) is the only soft spot, and it leans the same way as the Argo-ecosystem-coherence argument. Verdict: KEEP-the-practice, AMEND-tool-and-scheduler-refs.
- **ai_slop:** Low. Citations (Simian Army, principlesofchaos.org, SRE Workbook Ch.17, DiRT) are accurate. The relaxed-SLO-during-drill numbers (95% vs 99.5%, 2× latency) are reasonable, lightly fabricated-precision.
- **refinement:** (1) Replace `.github/workflows/chaos-nightly.yml` with an Argo Workflows CronWorkflow (ADR-0511). (2) Re-evaluate Litmus vs Chaos Mesh given Argo-centric stack. (3) Re-point ADR-0121→0375; foundry→cloud-intelligence. (4) Add masterplan binding.
- **consensus_needed:** no (scheduler/ref fixes; tool re-eval is advisory not blocking).

---

## Chunk notes for synthesis

**Pattern 1 — "Concretize-the-abstract" cluster, uniformly unbound.** All 7 (0159–0165) are the same archetype: take an earlier *contract* ADR (0003 audit-chain, 0114 canary, 0128 invariants, 0139 SLO-gate, 0049 residency) and pin a *specific OSS tool* or *concrete naming scheme* onto it. The decisions themselves are largely sound and hyperscaler-aligned; the rot is almost entirely in **cross-references and brand names**, not in the core architecture. **None carries `masterplan_ref`/`planning_impact`/`deliverables`** — this whole operational-fabric layer is invisible to the masterplan, yet several entries (per-tenant audit slicing 0162, env-tiers 0163, sovereign air-gap 0164) are market/compliance-load-bearing and *should* be masterplan FDs.

**Pattern 2 — Three retired-vocabulary leakage vectors recur across the chunk:**
- **foundry brand** (ADR-0335 retired → cloud-intelligence): appears in 0163, 0164 (`foundry-providers`, `axis-foundry`), 0165 (`microservices/foundry/`).
- **redis** (ADR-0336 retired → Valkey): baked into ADR-0161's *canonical, CI-enforced* StorageClass name `oya-redis-*` — highest-propagation instance because it is itself a standard.
- **CI-engine churn** (GitHub Actions / Jenkins → Argo Workflows per ADR-0511): ADR-0164 §f "GitHub Actions self-hosted runners," ADR-0165 `.github/workflows/chaos-nightly.yml`.

**Pattern 3 — ADR-0121 ref-rot is chunk-wide.** ADR-0121 (onprem k8s, superseded by ADR-0375 Talos) is cited as a live "portability invariant / in-stack" anchor by 0160, 0161, 0164, 0165. A single mechanical re-point (0121 → 0375) fixes four ADRs. ADR-0110 (ChangeSet, retired by 0363) is the secondary recurring stale anchor (0159, 0163).

**Pattern 4 — "Argo-ecosystem-coherence" is an emerging meta-tension.** Two independent tool choices in this chunk were made *against* the Argo ecosystem and now look re-litigable because the stack has since centralized on Argo (ArgoCD + Argo Workflows ADR-0511):
- ADR-0160 picks **Flagger over Argo-Rollouts** (and the keystone §3 CD row actually names Argo-Rollouts as canonical — a direct contradiction).
- ADR-0165 picks **Chaos Mesh over Litmus** (Litmus has better Argo Workflows integration by the ADR's own admission).
Recommend a single synthesis question: *given the stack is now Argo-centric, should Flagger and Chaos Mesh be re-evaluated against Argo-Rollouts and Litmus respectively?* These are the two genuinely contested decisions in the chunk; the other five are KEEP/AMEND-on-refs.

**Pattern 5 — "tier" is now 4-way overloaded.** This chunk adds two more "tier" axes to the retired tenant tier-system: **env-tier** (0163 test/staging/prod), **persona-tier** (0159 targeting), atop the live **autonomy-tier T1–T4** and **storage-tier** (0161 hot/warm/cold). For a masterplan-as-SSOT vocabulary this is a disambiguation hazard; ADR-0163's literal title "Environment **Tiers**" is the sharpest collision with retired ADR-0329.

**Cross-chunk tensions to escalate:**
- **CD-controller contradiction (0160 vs keystone §3):** Flagger vs Argo-Rollouts — must be reconciled; one is wrong.
- **Sovereign air-gap (0164) unbound despite gating entire markets** — candidate masterplan FD.
- **`oya-redis-*` standard name (0161)** propagates a retired brand through a CI gate — fix at the standard, not per-consumer.
- **Data-tier fault-line (Map #1):** 0161's StorageClass kinds (`pg`, `redis`, `s3`) institutionalize the SOURCE "assemble best-of-breed Postgres+Valkey+S3" posture that LINUX ADR-0001 explicitly wants to eliminate (own-DB). This chunk is strong evidence of the SOURCE side of that fault-line.
- **Dangling internal cite:** ADR-0164's `[Bominal-inheritance precedence]` is an undefined token — likely fabricated/orphaned; verify or delete.

**Net disposition tally:** KEEP ×1 (0162), AMEND ×6 (0159, 0160, 0161, 0163, 0164, 0165). Zero ARCHIVE/SUPERSEDE — no ADR in this slice is retired or redundant; the universal problem is **stale cross-references + retired brand tokens + missing masterplan binding**, not wrong decisions. Truth: TRUE ×1, PARTIAL ×6 (every PARTIAL = "core decision true, refs/names stale").
