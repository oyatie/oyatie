# Hyperscaler-Lens Findings: Build-vs-Buy & Scope Breadth

> **Audit pass:** initial-sweep-2026-06-06 · **Lens:** "Would Google / AWS / Azure actually do this?"
> **Scope:** every "own X" bet across both repos (SOURCE company monorepo + LINUX substrate pilot).
> **Mode:** READ-ONLY. No audited doc modified. This register surfaces tensions; it does not resolve them.
> **Founder mandate:** challenge each ownership decision against G/A/A reality; where the answer is "no, they wouldn't,"
> say why and whether it argues for amend / archive / re-sequence. Backfill only TRUE+relevant atoms into the masterplan.

---

## 0. Executive verdict

**The doctrine layer is hyperscaler-correct; specific day-0 ownership bets are where the hubris risk concentrates.**

There are TWO distinct postures in the corpus and they must not be conflated:

1. **The ratchet doctrine (aligned).** SOURCE ADR-0173 (Tier I/II/III vendor lock-in) + ADR-0211 (Class A/B/C, *value-anchored* not date-anchored Phase-2 triggers) and LINUX ADR-0019/0020/0022 (vendored-now / owned-when-*ready*-AND-*proven*, four-axis no-cherry-pick scorecard, sustained production burn-in) are **exactly what a hyperscaler does**: adopt the open standard where it is the standard, own only the differentiator, prove before escalating. ADR-0211 even names the AWS/Google/Azure/Oracle precedent correctly and *explicitly rejects* "build everything in-house from day one" (Alternative 2). **These are the strongest, most aligned decisions in the whole audit and should anchor the masterplan's build-vs-buy invariant.**

2. **The day-0 ownership bets (mixed).** Underneath the disciplined doctrine sit a cluster of **simultaneous, multi-year, from-scratch OWN_DAY0 commitments**: a distributed SQL engine, a Kubernetes control-plane reshape, a full L0–L8 container platform, a node-OS ("Rust Talos"), an owned VMM, an owned policy language, an in-house workflow engine, an in-house DCIM, a bespoke IdP, an in-house AI-model substrate, and five native client stacks per product. **No hyperscaler built all of these from scratch simultaneously** — each of AWS/Google/Azure owns a *subset* (their differentiator) and standardizes-on-OSS or buys-then-owns the rest, and they did it *sequentially over 10–20 years with thousands of engineers*. The pilot's own ratchet (ADR-0019/0020) is the correct instrument to stage these; the open question is whether the day-0 set is honestly small or whether "own when proven" has quietly become "own everything, day 1, in parallel."

**The honest-staging tension:** LINUX ADR-0019/0020 deliberately keep OWN_DAY0 = **{Cedar policy semantics, core kernel crates}** — genuinely small. But ADR-0001 (DB engine), ADR-0015 (orchestration), ADR-0017 (container platform), and ADR-0025 (node-OS) are each *separately* marked `Accepted` with day-0 starts and multi-year horizons. Summed, the day-0 program is **not** small. The ratchet is honored *per-component in prose* but the *portfolio-level* sequencing/opportunity-cost question is unbound and unanswered.

---

## 1. The G/A/A reality table (what the big-three actually do per surface)

| Surface | Google | AWS | Azure | Pattern | Corpus bet |
|---|---|---|---|---|---|
| **Distributed SQL engine** | Built Spanner from scratch (differentiator, ~5+ yr, huge team) | Built Aurora/DynamoDB (own) | Built Cosmos DB (own) | **BUILD — but it IS the cloud's crown-jewel differentiator** | LINUX ADR-0001 (own) — *aligned in kind, questionable in timing for a pilot* |
| **Kubernetes control plane** | INVENTED Borg→k8s, donated k8s to CNCF | Runs upstream k8s (EKS); did NOT rewrite it | Runs upstream k8s (AKS) | **STANDARDIZE-ON-OSS (k8s); only Google had the Borg head-start** | LINUX ADR-0015 (Rust rewrite) — *most hubristic single bet* |
| **Container runtime/platform** | Built gVisor (sandbox); uses containerd | Built Firecracker (VMM); uses containerd | Uses containerd/runc | **OWN the differentiating sliver (sandbox/VMM); REUSE the L4–L8 plumbing** | LINUX ADR-0017 (own all L0–L8) — *L1–L3 aligned, L4–L8+L7 build is the over-reach* |
| **Node OS** | Container-Optimized OS (custom, minimal) | Bottlerocket (custom, Rust-adjacent, built on existing kernel) | Flatcar/Mariner | **BUILD a thin immutable OS — but on the Linux KERNEL, never a from-scratch kernel** | LINUX ADR-0025 (Rust Talos) — *aligned (Bottlerocket precedent); kernel-replacement (ADR-0018 H2) is the over-reach* |
| **VMM** | (uses KVM/QEMU + gVisor) | Built Firecracker (Rust, minimal) | (Hyper-V, pre-existing) | **OWN — small, security-critical, proven valuable** | LINUX ADR-0014 owned VMM — *aligned; correctly behind a proof gate* |
| **Policy engine/language** | Zanzibar (own, internal) | Open-sourced Cedar (own the engine) | (Azure RBAC) | **OWN authz semantics — yes; a NEW DSL only if Cedar-class isn't enough** | LINUX ADR-0021 owned, Cedar-compatible; SOURCE ADR-0007/0243 adopt Cedar — *own-vs-reuse, see §5* |
| **CI/CD platform** | Blaze/Bazel + internal Prow/Critique | Internal (CodeBuild/Pipelines productized) | ADO pipelines | **BUILD internal CI at scale — but only AT Google/AWS scale** | SOURCE oya-ci Prow-class (ADR-0513/0514) — *aligned in kind, premature at current scale* |
| **IdP** | Internal (Google Identity) | Built Cognito; uses internal IAM | Built Entra/AAD (own) | **OWN identity eventually; START on OSS/managed (Keycloak-class)** | SOURCE ADR-0394 bespoke-Rust IdP vs ADR-0187 Zitadel + ADR-0211 Class-B — *doctrine-internal conflict, see §4* |
| **Workflow engine** | (internal); Temporal/Cadence lineage is ex-Uber/AWS-adopted | Adopted Temporal pattern → Step Functions (built on durable-exec research) | Logic Apps / Durable Functions | **BUILD durable-execution — but typically ADOPT the proven substrate (Temporal-class) first** | SOURCE ADR-0035 own bespoke FSM+DAG day-0, explicitly rejects Temporal — *questionable, see §3* |
| **DCIM** | Internal DC software (own, at planetary scale) | Internal (own) | Internal (own) | **BUILD — but only AFTER you own physical DCs at scale** | SOURCE ADR-0032 build DCIM day-0 before owning a DC — *premature, see §3* |
| **AI model substrate** | Gemini (frontier lab) | Bedrock (mostly broker + Titan/Nova) | Mostly OpenAI partnership + Phi | **Frontier only if you're a lab; else BROKER + small task-specific models** | SOURCE ADR-0026 task-specific in-house, provider-until-eval-wins — *aligned, correctly gated* |
| **Office/productivity + verticals** | Workspace (own) | (none — substrate only) | M365 (own) | **Only Google/MS own productivity; NONE own medical/pharmacy/banking verticals — ISVs do** | SOURCE ADR-0058/0321 flat catalog of owned verticals — *structure aligned, breadth questionable* |
| **Native client stacks** | Built Flutter to AVOID 5× native | (mostly web + 1–2 native) | (web + native Windows) | **CONVERGE clients (1 cross-platform); 5× native per product is what Flutter exists to prevent** | SOURCE ADR-0185 five fully-native stacks per product — *questionable resourcing, see §6* |
| **Transpiler (Go→Rust)** | (internal migration tools, never a product) | (internal) | (internal) | **Internal accelerator, never a deliverable; output always hand-hardened** | LINUX `oyago` (ADR-0017/0025) — *correctly scoped as scaffold-only* |

---

## 2. Findings register

Each finding: **decision** · **G/A/A reality** · **verdict** (aligned / questionable / misaligned) · **recommendation** (keep / amend / archive / re-sequence) · **founder question** (where it is a judgment call).

### FINDING BVB-01 — The ratchet doctrine itself (LINUX ADR-0019/0020/0022; SOURCE ADR-0173/0211)
- **Decision:** Universal port/adapter ratchet — vendored-now, owned-when-ready-AND-proven; scored inventory with OWN_DAY0/OWN_EARLY/DEFER_VENDORED/PERMANENT_REUSE buckets; value-anchored (never date-anchored) triggers; four-axis no-cherry-pick scorecard + sustained production burn-in.
- **G/A/A reality:** This is *precisely* the hyperscaler discipline. AWS Well-Architected OPS-4, CNCF "avoid single-vendor lock-in," Bottlerocket/Firecracker/Cedar being the *narrow* owned slivers — all match. ADR-0211 cites the exact precedent and rejects own-everything-day-1.
- **Verdict:** **ALIGNED** (the single strongest cluster in the audit).
- **Recommendation:** **KEEP** + **promote to a named masterplan invariant.** This doctrine is the founder's build-vs-buy answer made mechanical; it belongs in the masterplan as the governing rule that every "own X" decision must satisfy.
- **Founder Q:** Should the masterplan bind a SINGLE canonical ratchet that BOTH repos share (LINUX ADR-0019/0020 and SOURCE ADR-0173/0211 are near-duplicates with different vocabulary — "OWN_DAY0/DEFER_VENDORED" vs "Class A/B/C / Tier I/II/III")? They agree on principle; reconciling them into one rubric removes a guaranteed merge conflict and gives one gate.

### FINDING BVB-02 — Distributed SQL engine from scratch (LINUX ADR-0001)
- **Decision:** Build `cloud-data`, a from-scratch Rust multi-model distributed engine (LSM/MVCC/WAL/Raft/HLC/planner all owned), day-0 `Accepted`, 2–3 yr horizon, 5–8 senior engineers. Replaces etcd as the orchestration datastore; offers optional pg-wire; **does NOT eliminate Postgres** (the amended clarification note retains Postgres+Citus as reused OLTP).
- **G/A/A reality:** Spanner/Aurora/Cosmos prove this is *the* cloud differentiator worth owning — but each took 5+ years and a large dedicated team, and each was the company's crown-jewel bet, not one of a dozen simultaneous from-scratch substrates. A hyperscaler does NOT build a Spanner *while also* building its own kernel, node-OS, container platform, and CI system in the same window.
- **Verdict:** **QUESTIONABLE** on timing/portfolio-load (aligned in kind; the engine itself is a legitimate differentiator).
- **Recommendation:** **KEEP the decision, AMEND the framing** — the keystone-map's "eliminate PostgreSQL" reading (fault-line #1) is already softened on disk by the clarification note (Postgres+Citus *retained*). The masterplan should record the *retained* posture, not the sharper "eliminate" framing. **Re-sequence** explicitly against the portfolio: is `cloud-data` the ONE crown-jewel day-0 bet, with kernel/node-OS/container-platform staged behind it?
- **Founder Q:** Of the day-0 from-scratch substrates (DB engine, orchestration, container platform L4–L8, node-OS), which ONE is the crown jewel that gets the senior team now, and which get DEFER_VENDORED behind their ports until the first proves out? Spanner was built *alone*, not in a portfolio of ten.

### FINDING BVB-03 — Kubernetes control-plane reshape from scratch (LINUX ADR-0015)
- **Decision:** Own a Rust, Kubernetes-API-compatible apiserver + multi-shard scheduler + controller-manager + client-go-equivalent + cell-directory + Manifold typed-config, designing-out etcd write-wall / watch storms / scheduler ceiling. Day-0 `Accepted`.
- **G/A/A reality:** **This is the single most hubristic bet in the corpus.** Neither AWS (EKS) nor Azure (AKS) rewrote the k8s control plane — they run upstream Go k8s and add managed glue. ONLY Google could "design out" k8s complaints because Google had Borg/Omega *first*. The ADR itself honestly concedes "Go scheduler is 15+ years work; Gödel/plugins are large teams" and flags the etcd-v3-over-HLC revision emulation as an **unproven design risk that may move rather than remove the write wall.**
- **Verdict:** **QUESTIONABLE→MISALIGNED** as a day-0 own (the differentiating sliver — cellular + owned datastore — is defensible; rewriting the entire apiserver/scheduler/controller-manager is not what any of the big-three did).
- **Recommendation:** **AMEND + re-sequence.** Keep the cellular architecture and the owned-datastore-behind-etcd-v3-port as the differentiators (those ARE novel and align with the "design out the write wall" thesis). Reconsider rewriting apiserver/scheduler/controller-manager from scratch vs. running upstream k8s control plane on the owned datastore initially (the EKS/AKS pattern) and owning components only as each clears the proof gate.
- **Founder Q:** Is the differentiator the **cellular + owned-datastore** substrate (which can sit UNDER an upstream k8s control plane via the etcd-v3 adapter), or the **full Rust control-plane rewrite**? AWS and Azure prove you can own the hard part (datastore/scale) without rewriting the apiserver. Which is the actual moat?

### FINDING BVB-04 — Full L0–L8 container platform (LINUX ADR-0017)
- **Decision:** Own the entire container stack — runtime (L1–L3), storage/image/distribution (L4–L6), build+supply-chain (L7, BuildKit-class LLB solver), manager+CRI (L8), AND an owned zot-class registry server. 30+ crates, multi-year, solo/small-team pilot.
- **G/A/A reality:** Hyperscalers own the *differentiating slivers* — Google built gVisor (sandbox), AWS built Firecracker (VMM) — and **reuse containerd/runc/BuildKit/registry** for the commodity plumbing. Nobody rebuilds the L4–L8 spine *and* a BuildKit-class build engine *and* a registry from scratch. The ADR itself flags L7 (build) as a "second mountain… different product class… shares almost nothing with L1–L6."
- **Verdict:** **PARTIALLY MISALIGNED.** L1–L3 (isolation + owned VMM) is aligned (Firecracker/gVisor precedent). L4–L6 cohesion argument is defensible. **L7 (LLB build engine) is the clearest over-reach** — it is a BuildKit reimplementation that no hyperscaler would build day-0 for a small team.
- **Recommendation:** **AMEND** — execute the ADR's own §sprawl-risk mitigation NOW, not at "Stage-7 review": split L7 (build) out as DEFER_VENDORED (reuse BuildKit/Buildah behind a port) with a named ownership gate. Keep L1–L6+L8 as the cohesive owned spine. The registry-server own is cheap (thin policy over the L4 content store) and acceptable.
- **Founder Q:** Should the BuildKit-class L7 build engine be DEFER_VENDORED (reuse BuildKit behind the Transfer/content port) rather than owned in the first multi-year window? The ADR already concedes it's a separable "second mountain" — does the founder commit the build engine day-0 or stage it?

### FINDING BVB-05 — Node-OS "Rust Talos" + kernel replacement (LINUX ADR-0025 + ADR-0018)
- **Decision:** ADR-0025 — own an immutable, API-managed Rust node-OS (Talos-equivalent), beat-or-parity vs Talos, Talos-config-compatible during transition. ADR-0018 — the H2 endgame where the framekernel boots bare-metal and *replaces the Linux kernel* (owns drivers + in-TCB hypervisor).
- **G/A/A reality:** The node-OS layer is **aligned** — AWS Bottlerocket is exactly "build a thin immutable node-OS" (and it's Rust-adjacent). BUT Bottlerocket runs on the **Linux kernel**; no hyperscaler replaced the Linux kernel with a from-scratch kernel for production cloud nodes. ADR-0018 is honest about this: it reaches `consensus=FALSE`, times-boxes the literal "we are the host" claim to an *optional, uncommitted, gated, budgeted H2*, and admits H1 ships the flagship on Linux.
- **Verdict:** Node-OS (ADR-0025) = **ALIGNED** (Bottlerocket precedent, correctly Talos-compatible ratchet). Kernel-replacement (ADR-0018 H2) = **QUESTIONABLE but correctly fenced** — it is explicitly an uncommitted research bet behind a go/no-go gate with an unsafe-token budget, which is the *honest* way to carry a moonshot.
- **Recommendation:** **KEEP both** as written. ADR-0018's reservations-and-staging discipline is a model for how to carry an aspirational own without letting it corrupt the committed roadmap. The masterplan should bind the *committed* layers (node-OS-on-Linux, H1) and record H2 as an explicitly-uncommitted research node, NOT a promised deliverable.
- **Founder Q:** None required — the ADR already fences this correctly. (Optional: confirm H2 kernel-replacement stays OFF the masterplan's committed critical path and is recorded only as a gated research option.)

### FINDING BVB-06 — In-house workflow engine, day-0, rejecting Temporal (SOURCE ADR-0035)
- **Decision:** Build `oya-workflow-*` bespoke hybrid FSM+DAG engine day-0; Alternative C explicitly rejects Temporal ("if we need a layer above it anyway, we should own the engine").
- **G/A/A reality:** Durable-execution engines are real differentiators (AWS Step Functions, Temporal/Cadence from ex-Uber/AWS) — but the industry pattern is to **adopt the proven durable-execution substrate first** and own only the per-tenant-versioning + jurisdiction-overlay layer on top, escalating to owning the engine only when proven necessary. Day-0 bespoke FSM+DAG is the own-when-proven ratchet inverted.
- **Verdict:** **QUESTIONABLE** (the per-tenant-versioning + jurisdiction-overlay + agent-authored-step requirements are genuinely unusual and may justify ownership — but "we'd need a layer anyway, so own the whole engine" skips the ratchet's proof gate).
- **Recommendation:** **AMEND** — re-issue against the ADR-0211 Class-B pattern: adopt a durable-execution substrate (Temporal-class) behind a port as DEFER_VENDORED, own only the versioning/overlay/autonomy layer, with a value-anchored trigger to own the engine. Note also: ADR-0035 owner is the **retired `foundry` brand** (→ intelligence per ADR-0335) and it cites the **stale Kafka backbone** (ADR-0005→0377 Pulsar) — vocabulary needs amend regardless.
- **Founder Q:** Own the durable-execution ENGINE day-0 (ADR-0035 as written), or own only the per-tenant-versioning + jurisdiction-overlay + autonomy-ceiling layer over an adopted Temporal-class substrate, with engine-ownership gated on a proven trigger (per the shared own-when-proven ratchet)?

### FINDING BVB-07 — In-house DCIM before owning a DC (SOURCE ADR-0032)
- **Decision:** Build `oya-cloud-dcops-*` in-house DCIM (12 bounded contexts: power/cooling/BMS-BAS/network-ops/physical-security/etc.), 6–12 person-years, starting day-0; hard anti-scope on custom silicon.
- **G/A/A reality:** Hyperscalers DO own DC software — but only *after* and *because* they operate physical DCs at planetary scale where off-the-shelf DCIM genuinely can't cope. ADR-0028 puts Oyatie-owned DCs at **Phase 2+**; building a 12-BC DCIM before owning a single DC is owning the management layer for an asset you don't yet have. The cohesion-substrate-consumption argument is real but doesn't require day-0 build — it requires a *port*.
- **Verdict:** **QUESTIONABLE** (premature; correct end-state, wrong sequencing). The custom-silicon anti-scope is **aligned** as written — BUT note the absolute "never" is itself questionable given Google (TPU), AWS (Graviton/Nitro/Trainium), and Azure (Cobalt/Maia) ALL eventually built custom silicon at scale; the ADR's "revisit at founder ratification" escape hatch is the right hedge.
- **Recommendation:** **AMEND / re-sequence** — defer the DCIM build to align with ADR-0028 Phase-2 (actual DC ownership); until then, adopt/adapt OSS DCIM behind the cohesion-substrate ports. Keep the anti-scope-on-silicon but soften "never" to "not until Phase-3 scale + founder ratification" (matching the big-three trajectory).
- **Founder Q:** Build the 12-BC DCIM in-house from day-0, or adopt OSS DCIM behind cohesion ports until Oyatie actually owns physical DC capacity (ADR-0028 Phase 2)? And: is the absolute no-custom-silicon anti-scope still correct given all three hyperscalers eventually built TPU/Graviton/Cobalt-class silicon?

### FINDING BVB-08 — Bespoke-Rust IdP vs adopted Zitadel (SOURCE ADR-0394 vs ADR-0187 + ADR-0211)
- **Decision:** ADR-0394 (Proposed, founder-review-gated) reverses ADR-0170 Backstage AND positions a bespoke-Rust IDP central hub; the OIDC issuer question (Zitadel ADR-0187 vs bespoke `oya-identity-oidc-issuer-kernel`) is left as a "reconciliation pre-req." Meanwhile ADR-0211 lists **Zitadel as Class B (vendored-now, Phase-2 trigger ≥50K tenants)** and ADR-0187 makes Zitadel the primary OIDC IdP.
- **G/A/A reality:** Every hyperscaler eventually owns identity (Cognito/Entra/Google Identity) — but they did NOT build it day-0; Cognito and Entra were buy/build-then-own over years. ADR-0211's Class-B treatment of Zitadel (own at ≥50K tenants) is the *correct* hyperscaler posture. A bespoke IdP day-0 contradicts the org's own ADR-0211 ratchet.
- **Verdict:** **QUESTIONABLE + doctrine-internal conflict.** The bespoke *developer-portal* (IDP hub) is reasonable as Leptos/Rust aggregation over existing seams. The bespoke *identity provider / OIDC issuer* day-0 conflicts with ADR-0187 (Zitadel primary) and ADR-0211 (Zitadel Class-B-until-50K-tenants). Note "IDP" is overloaded here: **Internal Developer Platform** (portal) vs **Identity Provider** (OIDC) — a naming collision that itself risks a wrong decision.
- **Recommendation:** **AMEND** — disambiguate "IDP" (developer-portal) from "IdP" (identity provider). Keep ADR-0394's developer-portal-as-bespoke-Rust (aligned with the Leptos doctrine). Hold the OIDC-issuer ownership to the ADR-0211 Class-B trigger (front Zitadel now, own when ≥50K tenants). Do NOT let the portal decision smuggle in a day-0 identity-provider rewrite.
- **Founder Q:** Confirm the split: bespoke-Rust **developer portal** now (ADR-0394), but the **OIDC identity provider** stays Zitadel-fronted until the ADR-0211 Class-B trigger (≥50K tenants / multi-region active-active) fires? These are two different "IDP" decisions currently entangled in one ADR.

### FINDING BVB-09 — In-house AI model substrate (SOURCE ADR-0026)
- **Decision:** Long-horizon W-AI-Model-Substrate: consume Anthropic/OpenAI/Gemini until a per-vertical eval set favors in-house, then flip one router preference. Explicitly NOT a frontier lab; task-specific models (embedding/STT/TTS/OCR/safety-classifier) only.
- **G/A/A reality:** **This is exactly the AWS Bedrock pattern** — broker frontier providers, build small task-specific models (Titan/Nova) where volume/residency/cost justify it, never compete on frontier general reasoning unless you're Google. The eval-gated, one-router-change cutover is textbook own-when-proven.
- **Verdict:** **ALIGNED** (correctly scoped, correctly gated, correctly NOT a frontier-lab bet).
- **Recommendation:** **KEEP.** Only vocabulary amend needed (owner = retired `foundry` brand → intelligence; `oya-foundry-model-*` crates → cloud-intelligence per ADR-0335/0347). The decision is a model of disciplined build-vs-buy.
- **Founder Q:** None on the merits. (Vocabulary-only: confirm the `foundry`→intelligence rename applies to the model-substrate crates.)

### FINDING BVB-10 — Flat catalog of owned verticals (SOURCE ADR-0001 / ADR-0058 / ADR-0321)
- **Decision:** One cohesive product across a flat catalog of owned first-class microservices: medical, pharmacy, hr, payroll, banking, insurance, manufacturing, logistics, ads, analytics — all built in-house, joined at six shared substrates.
- **G/A/A reality:** **No hyperscaler builds the verticals.** AWS/Azure/Google provide *substrate* (compute/data/identity/AI) and let ISVs build medical/pharmacy/banking on top. Only Google/Microsoft own *productivity* (Workspace/M365), and that's the ceiling of their first-party app ambition. Owning medical+pharmacy+banking+insurance+manufacturing as first-class microservices is a breadth no infra company sustains in-house — it's a portfolio of vertical SaaS companies.
- **Verdict:** **QUESTIONABLE** (structure aligned — the cohesion/shared-substrate thesis is sound and genuinely differentiating; *breadth* is the issue: this is a GTM/portfolio scope question, not an architecture defect).
- **Recommendation:** **KEEP the cohesion architecture; flag the breadth for founder scope ruling.** The six-substrate cohesion thesis is the moat and belongs in the masterplan. Whether all verticals are day-0 OWNED microservices vs. substrate + ISV-built/partner verticals is a sequencing decision the architecture already supports (à-la-carte enablement) — it does not require building all ten verticals at once.
- **Founder Q:** Is the full first-class vertical catalog (medical/pharmacy/banking/insurance/manufacturing/logistics) genuine day-0 build scope, or is the day-0 scope the **substrate + cohesion layer** with verticals built incrementally (or by partners/ISVs on the capability registry)? The architecture supports either; the resourcing does not support all-at-once.

### FINDING BVB-11 — Five fully-native client stacks per product (SOURCE ADR-0185)
- **Decision:** Per-surface native rendering: SvelteKit/Leptos (web) + Swift/SwiftUI (Apple) + Kotlin/Compose (Android) + WinUI3/.NET (Windows) + GTK4/Rust (Linux), unified only by the OpenAPI contract. Five client teams per product. Directive: "native is best everywhere."
- **G/A/A reality:** **Google built Flutter specifically to AVOID this.** Maintaining five fully-native UI stacks per product is the cost that cross-platform frameworks exist to eliminate. Hyperscalers ship web-first + 1–2 strategic native clients per product; five native stacks × N products is a headcount multiplier no startup sustains.
- **Verdict:** **QUESTIONABLE** (a startup-scope-vs-hyperscaler-discipline resourcing bet; native quality is real but the 5× maintenance cost per product is the trap).
- **Recommendation:** **AMEND** — keep native quality as the *aspiration* but stage it: pick a 1–2 platform day-0 set (web + one native) with the rest deferred behind the same OpenAPI contract (the contract-first design already makes this cheap to defer). The OpenAPI-as-unifier decision is excellent and should be kept; the "all five day-0" part is the over-commitment.
- **Founder Q:** Commit to five fully-native first-party client stacks per product day-0 (ADR-0185), or a 1–2 platform day-0 set (web + one native) with the rest deferred behind the OpenAPI contract until the product/revenue justifies each additional native team?

### FINDING BVB-12 — Owned policy language vs adopted Cedar (LINUX ADR-0021 vs SOURCE ADR-0007/0243)
- **Decision:** LINUX ADR-0021 — own a typed, compile-to-Rust, tier-aware policy language, *explicitly Cedar-compatible* (extends Cedar's PARC + Lean soundness; `cedar-policy` vendored adapter now, owned port later). SOURCE ADR-0007/0243 — Cedar as the universal authz gate. LINUX ADR-0020 marks Cedar as the LONE OWN_DAY0.
- **G/A/A reality:** Google owns Zanzibar; AWS open-sourced Cedar (owns the engine). Owning authz *semantics* is a legitimate differentiator. But authoring a NEW policy DSL day-0 (rather than owning a Cedar-compatible engine) is the riskier path — the value is in the engine + soundness, not a new surface language. The ADR's Cedar-compatibility framing is the mitigation.
- **Verdict:** **ALIGNED-with-watch** (own-vs-reuse, not a contradiction — LINUX positions as the owned successor to the same Cedar model; the autonomy-tier T1–T4 dimension is genuinely novel and NOT the retired tenant tier-system).
- **Recommendation:** **KEEP** — but the masterplan must record this as ONE policy decision with a clear boundary: Cedar-the-engine adopted now (ADR-0211 Class-A treats Cedar as community-standard KEEP), owned Cedar-*compatible* port later when proven. This is consistent across both repos if framed as own-the-engine-behind-the-Cedar-contract (an external-standard-contract port per ADR-0019).
- **Founder Q:** Does the masterplan adopt Cedar long-term as the authz engine (ADR-0007/0211 Class-A), with the LINUX owned port as a *Cedar-compatible engine-replacement behind the same contract* (external-standard port) — NOT a new incompatible DSL? Confirming "keep the Cedar contract, own the engine" resolves the apparent own-vs-reuse conflict cleanly.

---

## 3. Is the own-when-proven ratchet HONORED, or is it own-everything-day-1?

**Per-component: mostly honored. Portfolio-level: at risk.**

- **Honored (correctly staged behind proof gates):** owned VMM (ADR-0014, Stage-3 benchmark-gated), AI model substrate (ADR-0026, eval-gated), node-OS-on-Linux (ADR-0025, beat-or-parity gate), kernel-replacement H2 (ADR-0018, go/no-go + token budget), most of ADR-0020's inventory (Milvus/Firecracker/OpenBao/ClickHouse/Pulsar all DEFER_VENDORED or OWN_EARLY with named triggers), all of SOURCE ADR-0211 Class-B (Zitadel/Milvus/ClickHouse/SeaweedFS/Meilisearch vendored-now with value-anchored triggers).

- **At risk (day-0 `Accepted` from-scratch, not behind a DEFER gate):** ADR-0001 (DB engine), ADR-0015 (full control-plane rewrite), ADR-0017 (L4–L8 + L7 build engine), SOURCE ADR-0035 (workflow engine), ADR-0032 (DCIM). Each is individually defensible; the **sum** is a simultaneous multi-year from-scratch program across DB + orchestration + container + workflow + DCIM that no hyperscaler attempted in parallel.

- **The structural gap:** the ratchet operates **per-component** (each ADR scores itself) but there is **no portfolio-level capacity/sequencing gate** asking "can a small team actually carry N simultaneous day-0 owns?" ADR-0020's "day-0 set kept deliberately small" (= Cedar + core kernel) is the right instinct, but it is contradicted by the separately-`Accepted` ADR-0001/0015/0017/0025 each launching its own multi-year day-0 build. **The masterplan should bind a single cross-component day-0 budget**, forcing the crown-jewel-vs-staged decision the founder questions in §2 demand.

---

## 4. Masterplan-binding implications (build-vs-buy atoms to backfill)

Under BOTH open founder readings (authored-as-SSOT vs generated-from-ADRs), these TRUE+relevant build-vs-buy atoms are currently **unbound** and should enter the masterplan:

1. **The shared ratchet invariant** (BVB-01) — one canonical own-when-proven rubric reconciling LINUX ADR-0019/0020 + SOURCE ADR-0173/0211. *Highest-value backfill.*
2. **A portfolio-level day-0 ownership budget** (§3) — the missing cross-component gate; not currently any ADR's job.
3. **The crown-jewel ruling** (BVB-02/03/04) — which ONE from-scratch substrate is day-0; which are DEFER_VENDORED behind ports.
4. **Cedar = own-the-engine-behind-the-contract** (BVB-12) — resolves the only apparent source↔linux policy conflict.
5. **Vertical-breadth scope ruling** (BVB-10) — substrate-day-0 vs all-verticals-day-0.

Note: almost every source-side build-vs-buy ADR in this register carries **retired-`foundry` vocabulary** (owner fields, `oya-foundry-*` crates) per the keystone map §2 — vocabulary amend is needed corpus-wide regardless of the build-vs-buy rulings, and must not be confused with the architectural decisions, which are mostly sound.

---

## 5. One-line dispositions

| ID | Decision | Verdict | Disposition |
|---|---|---|---|
| BVB-01 | Ratchet doctrine (0019/0020/0173/0211) | ALIGNED | KEEP + promote to masterplan invariant |
| BVB-02 | DB engine from scratch (LINUX 0001) | QUESTIONABLE (timing) | KEEP decision, AMEND framing, re-sequence |
| BVB-03 | K8s control-plane rewrite (LINUX 0015) | QUESTIONABLE→MISALIGNED | AMEND + re-sequence (own datastore, not apiserver) |
| BVB-04 | Full L0–L8 container platform (LINUX 0017) | PARTIAL MISALIGNED (L7) | AMEND (DEFER L7 build engine) |
| BVB-05 | Node-OS + kernel replacement (LINUX 0025/0018) | ALIGNED (OS); fenced research (kernel) | KEEP (model staging) |
| BVB-06 | Bespoke workflow engine day-0 (SOURCE 0035) | QUESTIONABLE | AMEND (adopt Temporal-class, own overlay) |
| BVB-07 | DCIM before owning a DC (SOURCE 0032) | QUESTIONABLE (premature) | AMEND / re-sequence to Phase-2 |
| BVB-08 | Bespoke IdP day-0 (SOURCE 0394 vs 0187/0211) | QUESTIONABLE + internal conflict | AMEND (split portal vs OIDC issuer) |
| BVB-09 | In-house AI model substrate (SOURCE 0026) | ALIGNED | KEEP (vocab amend only) |
| BVB-10 | Owned vertical catalog (SOURCE 0001/0058/0321) | QUESTIONABLE (breadth) | KEEP arch, founder scope ruling |
| BVB-11 | Five native client stacks (SOURCE 0185) | QUESTIONABLE (resourcing) | AMEND (stage to 1–2 day-0) |
| BVB-12 | Owned policy language (LINUX 0021 vs 0007/0243) | ALIGNED-with-watch | KEEP (own engine behind Cedar contract) |

---
*End of register. The doctrine is hyperscaler-correct; the day-0 portfolio is where "own when proven" must be enforced against "own everything now." No audited doc was modified in producing this artifact.*
