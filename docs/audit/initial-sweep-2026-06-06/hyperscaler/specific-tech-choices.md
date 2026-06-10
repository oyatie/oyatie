# Hyperscaler-Lens Findings Register — Specific Tech Choices

> **Audit pass:** initial sweep 2026-06-06 · **Lens:** "Would Google / AWS / Azure actually do this?"
> **Scope:** SOURCE (`~/Developer/source`, 346 ADRs) + LINUX pilot (`~/Developer/linux`, ADR-0001–0026).
> **Method:** READ-ONLY. Each finding = the decision · the G/A/A reality · verdict (aligned / questionable / misaligned) · recommendation (keep / amend / archive / re-sequence) · founder question if it is a judgment call.
> **Binding baseline:** keystone map `_map/canonical-posture-and-supersession-map.md`. Trust the superseding ADR over stale front-matter. `foundry`/`tier`/`Jenkins-as-destination`/`Kafka`/`Redis`/`Backstage` are RETIRED vocabulary.

This register is organized by tech-choice cluster, not by ADR number. It deliberately separates the **specific-technology** challenges (CI engine, forge, DB engine, sandbox mix) from the **brand/vocabulary** drift that other auditors own — here only where the tech choice itself is the question.

---

## A. CI / CD ENGINE — the choice a hyperscaler challenges first

### A1. Jenkins as destination CI — MISALIGNED (already self-corrected)
- **Decision:** ADR-0349 (Jenkins+ArgoCD augment) → ADR-0359 (Jenkins *sole/destination* CI, removes GitHub Actions) → ADR-0361 (execute Jenkins-native).
- **G/A/A reality:** **No hyperscaler runs Jenkins at scale as the destination CI.** Google uses bespoke (Blaze/TAP/Rapid + Critique/Prow externally); AWS uses CodeBuild/internal pipelines; Azure uses Azure DevOps/1ES. Jenkins is a stateful Groovy controller bolted onto k8s via a plugin — the opposite of the declarative, k8s-native, GitOps-managed substrate the rest of the stack (Talos/ArgoCD, ADR-0375) commits to. The *trigger* for ADR-0359 (GitHub Actions budget cap blocked PR #180's 37-job matrix — a metered third-party SPOF) is a **correct** hyperscaler instinct: own your CI so a vendor quota cannot block your merge gate.
- **Verdict:** Jenkins-as-destination = **misaligned**; the self-hostability motivation = **aligned**.
- **Recommendation:** **archive** ADR-0359's "Jenkins is the sole/destination orchestrator" half. Already done in fact: ADR-0511 supersedes ADR-0359, retains the SPOF-removal half, reframes Jenkins as **transitory bootstrap only**. Keystone is correct. The remaining work is masterplan hygiene (MASTERPLAN.md L126 still names "Jenkins required checks" — stale).

### A2. Argo Workflows as destination CI orchestrator — ALIGNED
- **Decision:** ADR-0511 (CNCF Argo Workflows = k8s-native CI orchestrator; supersedes ADR-0359).
- **G/A/A reality:** This is the **hyperscaler-correct** answer for a k8s-everywhere shop: CNCF-graduated, k8s-native, DAG-per-step CRD workload, GitOps-managed, self-hostable across air-gap/colo/cloud. It is the externally-buildable analogue of Google's container-step DAG model and fits the affected-target + verdict-cache pattern (ADR-0366) far better than Groovy stages.
- **Verdict:** **aligned.**
- **Recommendation:** **keep.** Note status is still `Proposed — DRAFT, do NOT auto-merge`; the masterplan still records the stale Jenkins gate. **Re-sequence:** promote ADR-0511 to Accepted and repoint MASTERPLAN.md's CI gate language before backfill.

### A3. Buck2 as canonical build graph (reverses Bazel rules_rust) — ALIGNED
- **Decision:** ADR-0392 (Buck2 build graph) + ADR-0408 (Buck2 CI/RBE + affected-targets), reversing ADR-0358 §2 (Bazel rules_rust).
- **G/A/A reality:** **Aligned.** Buck2 is Meta's open-sourced monorepo build engine (Rust, Starlark, RBE-native); a Rust-heavy monorepo with affected-target gating is exactly its design center. Bazel `rules_rust` is the alternative a hyperscaler would *also* accept — this is a defensible peer-swap, not a correction of an error. Both Buck2 and Argo Workflows pass the lens.
- **Verdict:** **aligned.**
- **Recommendation:** **keep.** Both ADRs are `Proposed — do NOT auto-merge`; the build-graph reversal is in-flight. **Re-sequence** to Accepted with the CI cluster.

### A4. oya-ci — bespoke-Rust Prow as the eventual CI platform — QUESTIONABLE (correct *destination*, watch the *trigger*)
- **Decision:** ADR-0513 (Accepted, founder-locked): a bespoke-Rust, Prow-shaped CI/CD platform (`tide` merge-queue + `plugins` + `hook` webhook gateway) on the Forgejo substrate, phased to replace the Jenkins gate path.
- **G/A/A reality:** Owning the CI control plane is genuinely what hyperscalers do (Google Prow, TAP). The Prow shape is the right reference. **But:** Google built Prow/TAP at enormous repo scale with full-time platform teams; a startup-stage org building a bespoke Rust Prow *before* the OSS substrate (Argo Workflows + the `oya` gate engine) demonstrably fails to scale is the **own-too-early** risk the founder mandate is meant to catch. This is the recurring own-when-proven vs own-day-0 axis.
- **Verdict:** **questionable** on sequencing (not on direction).
- **Founder question:** Is the bespoke `oya-ci` Prow platform a **day-0 build** (ADR-0513 reads founder-locked, Tide "owned here, not deferred"), or is it gated behind a numeric trigger (lane count / gate latency / Argo-Workflows-proves-insufficient) the way ADR-0510 gates the bespoke VCS? Consistency with the own-when-proven ratchet (ADR-0211/0173, LINUX ADR-0019/0020) argues for the latter.

---

## B. FORGE / SCM — the three-way fault-line (surface, do not resolve)

### B1. Forgejo as canonical forge — QUESTIONABLE under the hyperscaler lens
- **Decision:** ADR-0363 (self-hosted Forgejo canonical host; retire bespoke agentic-VCS; GitHub is bootstrap-only mirror), softened by ADR-0510 (Forgejo *transitory*).
- **G/A/A reality:** **No hyperscaler runs a vanilla git forge at monorepo scale.** Google runs Piper + CitC; Meta runs Mononoke + Sapling/EdenFS; Microsoft built GVFS/Scalar to make even Git survive the Windows monorepo. Forgejo (a Gitea fork) is a fine *self-hosted GitHub-replacement for normal repos* and a defensible vendor-lock-avoidance move, but it is **not** what G/A/A use at scale. Source canon already knows this — ADR-0510 honestly records that at 23k files / 482M `.git` a full clone is "minutes, not hours," so the bespoke-VCS forcing function is **distant** and Forgejo is correct *for now*.
- **Verdict:** Forgejo-as-near-term-host = **aligned-with-caveat** (right for current scale, not a hyperscaler-scale answer); Forgejo-as-permanent-canonical = **misaligned** (correctly downgraded to transitory by ADR-0510).
- **Recommendation:** **keep** ADR-0510's transitory framing; **amend** ADR-0363's bare "canonical" language to inherit the transitory horizon (ADR-0510 already amends it — propagate to front-matter).

### B2. Bespoke hyperscaler monorepo-VCS as the declared destination — ALIGNED (in principle), QUESTIONABLE (in timing)
- **Decision:** ADR-0510: the SCM destination is a Rust Piper/Sapling/Mononoke-class bespoke monorepo-VCS, cutover gated on a numeric trigger.
- **G/A/A reality:** **Directionally aligned** — this *is* what hyperscalers do. But Piper/Mononoke are multi-hundred-engineer-year efforts, and ADR-0510 itself admits "buys nothing today." Building it before the forcing function fires would itself fail the own-when-proven test. The recorded-but-deferred-behind-numeric-trigger posture is the **disciplined hyperscaler answer**.
- **Verdict:** **aligned** (destination + numeric-trigger discipline is exactly right).
- **Recommendation:** **keep.** Best example in the corpus of the own-when-proven ratchet applied correctly.

### B3. GitHub-native automation substrate (Actions / branch-protection REST / merge-queue) — MISALIGNED as *substrate*
- **Decision:** ADR-0041, ADR-0124, ADR-0139, ADR-0170, ADR-0171, ADR-0173 + the retired VCS cluster (ADR-0110/0112/0113) hard-bind branch protection, merge queue, CODEOWNERS, federation IaC `repoURL`, and CI plumbing to GitHub YAML / `gh api` / `workflow_run`.
- **G/A/A reality:** Using GitHub as a *host* is fine (Microsoft owns GitHub; many teams ship on it). But binding the *automation substrate and merge-gate logic* to GitHub-proprietary surfaces is the exact SPOF that triggered ADR-0359, and contradicts source's own Forgejo/bespoke-VCS canon. **The founder's GitHub migration directive resolves the HOST layer, not the AUTOMATION-SUBSTRATE layer** — ADR-0510/0363 retired GitHub-as-substrate even while GitHub remains the bootstrap host.
- **Verdict:** **misaligned** (GitHub-as-automation-substrate); the founder's GitHub *host* directive is orthogonal.
- **Recommendation:** **amend** these ADRs to express forge-neutral gates (Forgejo Commit Status / Argo Workflows / the `oya` gate engine as the sink) with GitHub as one adapter. **Surface, do not resolve** the three-way (founder GitHub directive vs Forgejo-transitory vs bespoke-VCS-destination) — this is keystone fault-line #4.
- **Founder question:** Is the canonical branch/merge-queue model authored **forge-neutrally** (Forgejo + GitHub adapters, Tide as the queue) or **pinned to GitHub** `jason931225/oyatie` per your migration directive? Source canon and your directive disagree at the substrate layer; pick the binding one.

---

## C. POLICY / AUTHZ — "reinventing Cedar?"

### C1. Cedar as the universal authorization engine — ALIGNED
- **Decision:** SOURCE ADR-0007/0099/0243/0246 adopt Cedar as the sole app-authz PDP; ADR-0379 keeps the Cedar/admission separation with Kubewarden as default admission.
- **G/A/A reality:** **Aligned.** Cedar is AWS's own open-source policy language (the engine behind AWS Verified Permissions); using it as a PDP is precisely the hyperscaler pattern (AWS Cedar, Google Zanzibar-class, Azure RBAC/ABAC). Adopting a proven, formally-verified policy engine rather than inventing a DSL is the correct build-vs-buy call at the application-authz layer.
- **Verdict:** **aligned.**
- **Recommendation:** **keep** (amend vocabulary only — `foundry.supervisor.*` namespace, `oya-foundry-*` crate prefixes are retired).

### C2. LINUX owned, compile-to-Rust, tier-aware policy language (Cedar-compatible) — QUESTIONABLE, NOT misaligned
- **Decision:** LINUX ADR-0021: a typed authorization policy language that **compiles to native Rust** at build time, extends Cedar's PARC model + Lean soundness, adds a first-class autonomy-tier (T1–T4) dimension; `cedar-policy` is the vendored adapter now, owned port later (per ADR-0019/0020 OWN_DAY0).
- **G/A/A reality:** This is **not** "reinventing Cedar from scratch" — it is **extending an adopted Cedar** with a compile-to-Rust evaluator and a regulated-AI autonomy-tier. The own-vs-reuse logic is sound (Cedar fragments are embedded across every microservice / capability YAML / audit / workflow step → no single port seam absorbs a switch → canonical OWN_DAY0 lock-in case). The compile-to-Rust differentiator (build-time schema errors, native speed, diffable generated evaluator) is a real, defensible edge. **The hyperscaler challenge is sequencing, not principle:** AWS shipped Cedar-the-interpreter and a startup owning a *compiler* for it day-0 is an own-too-early risk — the same axis as A4/B3.
- **Verdict:** **questionable** on day-0 timing; **aligned** on long-horizon direction. This is keystone fault-line #2 (own-vs-reuse Cedar), explicitly framed by the map as a *trigger-threshold* disagreement, not a contradiction.
- **Founder question:** Does the masterplan adopt Cedar **long-term as the engine** (source ADR-0243), or is Cedar the **vendored adapter pending the LINUX owned compile-to-Rust port** (LINUX ADR-0021)? Load-bearing for both repos. If owned-port, is the compiler a day-0 build or a proof-gated OWN_EARLY?

### C3. Kubewarden default admission (Kyverno demoted to adapter) — ALIGNED
- **Decision:** ADR-0379 (supersedes ADR-0183): Kubewarden = default k8s admission; Kyverno = first-class adapter; Cedar/admission separation retained.
- **G/A/A reality:** **Aligned.** Both Kubewarden (WASM-policy) and Kyverno (CRD-policy) are CNCF admission engines a hyperscaler-grade k8s platform would accept; Kubewarden's WASM model coheres with the WASM-native substrate (wasmtime). The clean Cedar(L7)/admission(k8s-resource) separation is correct hyperscaler layering.
- **Verdict:** **aligned.**
- **Recommendation:** **keep.** **Amend** ADR-0039 (still presents Kyverno as canonical admission) to record Kubewarden-default per ADR-0379.

---

## D. DATA TIER — own-the-engine vs assemble-best-of-breed (keystone fault-line #1)

### D1. SOURCE best-of-breed managed substrates — ALIGNED
- **Decision:** Milvus (vector >10M, ADR-0192) · ClickHouse (OLAP, ADR-0193) · TimescaleDB (tenant TS, ADR-0194) · SeaweedFS+Ceph (object, ADR-0196) · Postgres+pgcat (OLTP pooling, ADR-0179) · Postgres+Citus (OLTP, ADR-0045).
- **G/A/A reality:** **Aligned** as a posture. Hyperscalers run purpose-built engines per workload; assembling proven OSS per workload class (with the named own-when-proven ratchet, ADR-0211) is the disciplined startup analogue. Milvus specifically is hyperscaler-validated (NVIDIA AI Enterprise reference stack, Shopify/Uber/Cloudflare parallels in ADR-0192). Per-workload specialization beats one-engine-for-everything — correct.
- **Verdict:** **aligned.**
- **Recommendation:** **keep.** Note license-bookkeeping defects (ADR-0045 claims Citus columnar = Apache-2 but ADR-0184 records AGPL3; ClickHouse self-maintained-fork tracking) — a hygiene **amend**, not an architecture problem.

### D2. LINUX from-scratch Rust multi-model DB engine ("eliminate PostgreSQL") — QUESTIONABLE (sharpest cross-side conflict)
- **Decision:** LINUX ADR-0001: a from-scratch Rust multi-model engine (relational + KV + vector + FTS + later graph), multi-Raft, cell-native, citing Spanner/CockroachDB/TiKV/FoundationDB/Aurora as proof that hyperscalers build the DB substrate in-house.
- **G/A/A reality:** **Both true and dangerous.** It IS true that G/A/A build core DB engines (Spanner, Aurora, etc.) — owning the data substrate at scale is the competitive baseline. **But** every one of those was a multi-hundred-engineer, multi-year effort built *because* an existing engine demonstrably could not meet a forcing function; none "eliminated PostgreSQL" as a day-0 architectural stance for a staging pilot. Note ADR-0001's own clarification softens the headline: it now says "PostgreSQL+Citus remains the reused OLTP substrate … this engine does NOT eliminate Postgres; it owns the distributed-database differentiator layer" — which is **materially more aligned** than the "eliminate PostgreSQL" framing the keystone map flags. LINUX ADR-0020 already correctly buckets Milvus/Firecracker/OpenBao as DEFER_VENDORED and Valkey/tokio/Iceberg as PERMANENT_REUSE — the same ratchet source uses.
- **Verdict:** **questionable** — the *differentiator-layer* engine is a defensible OWN bet on the own-when-proven ratchet; the *"eliminate PostgreSQL"* headline (still present in ADR-0001's title/context) is the misaligned framing.
- **Recommendation:** **amend** ADR-0001 to lead with the reconciled scope ("owns the distributed-database differentiator + replaces etcd; Postgres+Citus reused for OLTP") and drop the "eliminate PostgreSQL" headline. This collapses keystone fault-line #1 from a contradiction to a bounded scope statement.
- **Founder question:** At merge, is Postgres+Citus the canonical OLTP engine (source ADR-0045/0179/0184) with the owned engine as a *differentiator layer* (ADR-0001's clarification), or does the owned multi-model engine become the Tier-1 source-of-truth long-term (ADR-0184 vs ADR-0001 are mutually exclusive at Tier-1)? Where is the boundary?

### D3. Eventing: Pulsar 4.x + Oxia (Kafka retired) — ALIGNED
- **Decision:** ADR-0377-kafka-to-pulsar (supersedes ADR-0005): Pulsar 4.x + Oxia, KoP wire-compat; transactional-outbox retained.
- **G/A/A reality:** **Aligned.** Pulsar (tiered storage, multi-tenancy-native, geo-replication) is a defensible hyperscaler-grade event backbone; KoP preserves Kafka wire-compat for zero client churn. Note: ADR-0005 still reads `proposed` on disk despite being retired-in-fact — trust the superseding ADR.
- **Verdict:** **aligned.** **Recommendation: keep**; the residual Kafka references in dependent ADRs (0003/0004/0154/0166/0169/0172) are a mechanical **amend**.

### D4. Valkey not Redis — ALIGNED (license-driven, correct)
- **Decision:** ADR-0336: Valkey (BSD-3) replaces Redis 7.4+ (SSPL/RSAL relicense); RESP3 contract preserved.
- **G/A/A reality:** **Aligned** — this is exactly the move AWS/GCP/Oracle made (the Linux Foundation Valkey fork *is* the hyperscaler response to the Redis relicense). PERMANENT_REUSE bucket is correct (RESP3 contract, zero differentiation upside).
- **Verdict:** **aligned.** **Recommendation: keep**; residual `oya-redis-*` StorageClass names (e.g. ADR-0161) are retired-vocab **amend**.

---

## E. ISOLATION / RUNTIME — the sandbox mix

### E1. wasmtime + Firecracker/Kata + Cloud-Hypervisor runtime ladder — ALIGNED
- **Decision:** ADR-0147 (runtime ladder native→sandbox→microvm→confidential; Cloud-Hypervisor/kata-clh primary, gVisor/kata-qemu/kata-fc options) + ADR-0023 (wasmtime+WASI-P2 for short-lived tools, Firecracker for full-kernel tools) + ADR-0200 (wasmtime canonical WASM).
- **G/A/A reality:** **Aligned, and precisely the hyperscaler pattern.** AWS Lambda/Fargate = Firecracker; Google = gVisor + Kata-class; Cloudflare/Fastly = wasmtime/WASM for short-lived isolates. A two-tier (WASM-fast / microVM-strong) sandbox with per-spawn audit and uniform resource caps is exactly how G/A/A run untrusted/agent workloads. The "wasmtime/firecracker mix" the founder flagged as a thing to challenge is, on inspection, **correct** — they target different latency/surface tiers, not a redundant pick.
- **Verdict:** **aligned.**
- **Recommendation:** **keep.** Resolve ADR-0147's intra-body contradiction (post-amendment names kata-clh primary but pre-amendment prose still says "gVisor by default / three RuntimeClasses") — a documentation **amend**, not an architecture change.

### E2. LINUX framekernel "we are the host" / Capsule model — QUESTIONABLE (honestly self-staged)
- **Decision:** LINUX ADR-0018 (accepted-with-reservations): unify container/VM/pod/OS under a Capsule; push isolation into the framekernel; "no separate containerd."
- **G/A/A reality:** Owning the host kernel + isolation primitive is what the most ambitious hyperscaler infra does (gVisor is a userspace kernel; AWS Nitro owns the hardware boundary). **But** building a from-scratch host framekernel is the single largest own-bet in either repo. The ADR is **unusually honest**: it stages the claim H0→H1→H2, records `consensus=FALSE`, admits the framekernel currently **boots as a QEMU guest** with 43 syscall handlers and no device drivers, and concedes that the H1 flagship Native Capsule actually runs on the **Linux** host kernel (a userspace shim over clone/unshare/cgroups). This self-awareness is the correct posture; the literal "we are the host" claim is honestly time-boxed to an uncommitted H2.
- **Verdict:** **questionable** (largest own-bet; not yet real), but **not misaligned** because it is honestly staged. ADR-0014 (one OCI/CRI frontend + pluggable `IsolationBackend` port) is the **aligned** near-term shape.
- **Recommendation:** **keep** ADR-0014 (the port abstraction is hyperscaler-correct); **keep ADR-0018 as staged vision** with the host claim gated. This is keystone fault-line #3 (own-the-host vs assemble-the-substrate). **Surface, do not resolve.**

---

## F. THE "OWN EVERYTHING" BREADTH BETS — where a hyperscaler stages instead of building day-0

### F1. DCIM built in-house from day-0 + absolute no-custom-silicon — QUESTIONABLE (both halves)
- **Decision:** SOURCE ADR-0032: build `oya-cloud-dcops-*` DCIM in-house (12 bounded contexts); hard anti-scope on custom silicon ("own nothing below the OEM line").
- **G/A/A reality:** Two separate challenges. **(a) Day-0 DCIM:** hyperscalers DO eventually build in-house DCIM — but only once they own physical DCs (ADR-0028 Phase 2+); building it before owning a DC is premature. They'd adopt/adapt OSS DCIM behind ports until physical capacity exists. **(b) No-custom-silicon anti-scope:** this is **factually wrong as a permanent stance** — Google (TPU), AWS (Graviton/Nitro/Inferentia), and Azure (Cobalt/Maia) each built custom silicon at scale. The anti-scope is correct *for a startup today* but wrong *as an absolute*.
- **Verdict:** **questionable** on both timing (DCIM day-0) and the absolute silicon ban.
- **Founder question:** Build DCIM in-house from day-0, or adopt OSS DCIM behind ports until Oyatie owns DC capacity? And is the absolute no-custom-silicon anti-scope still correct given G/A/A all eventually built custom silicon at scale — or should it be reframed as "no custom silicon *until* a forcing function" (the same numeric-trigger discipline as ADR-0510)?

### F2. Bespoke hybrid FSM+DAG workflow engine day-0 (rejects Temporal) — QUESTIONABLE
- **Decision:** SOURCE ADR-0035: build `oya-workflow-*`, a hybrid state-machine+DAG engine day-0; explicitly rejects Temporal and pure-BPMN/pure-DAG.
- **G/A/A reality:** A hyperscaler would put a **durable-execution substrate (Temporal / AWS Step Functions class) behind a port** and own only the genuinely-differentiating layer (per-tenant versioning + jurisdiction overlay) until owning the engine is proven necessary. Building a bespoke durable workflow engine day-0 is the own-too-early pattern. (Note the false-contradiction trap: ADR-0035 rejecting "Argo Workflows as a *business-process* engine" is NOT in conflict with ADR-0511 adopting Argo Workflows as the *CI* orchestrator — different layers, both correct.)
- **Verdict:** **questionable** (own-too-early).
- **Founder question:** Own a bespoke FSM+DAG engine day-0, or adopt a durable-execution substrate behind a port and own only the per-tenant-versioning + jurisdiction-overlay layer until the engine is proven necessary (own-when-proven ratchet)?

### F3. Five fully-native first-party client stacks per product — QUESTIONABLE
- **Decision:** SOURCE ADR-0185 (Workflow Studio: per-surface native — Web + Apple ecosystem + Android + Windows + Linux desktop, OpenAPI codegen as the only unifier) + ADR-0051 (web-first, native deferred to later waves).
- **G/A/A reality:** **No hyperscaler maintains five fully-native UI stacks per single product** — Google built Flutter and Microsoft built MAUI/React-Native specifically to *avoid* this cost. The "native is best everywhere" directive maximizes per-platform UX but multiplies team count and parity surface. ADR-0051 (web-first, native deferred) is the more disciplined sibling and partially mitigates.
- **Verdict:** **questionable** (startup-scope-vs-hyperscaler-discipline resourcing bet). OpenAPI-contract-as-unifier is the one **aligned** element.
- **Founder question:** Commit to five fully-native client stacks day-0 (ADR-0185), or a 1–2 platform day-0 set (web + one native) with the rest deferred per ADR-0051's wave gating, given no hyperscaler maintains five native UI stacks per product?

### F4. Flat vertical catalog as first-class owned microservices — QUESTIONABLE (scope, not structure)
- **Decision:** SOURCE ADR-0058/0001: medical→pharmacy→hr→payroll→banking→insurance→ads→analytics→manufacturing→logistics as first-class owned microservices.
- **G/A/A reality:** **No hyperscaler builds this vertical breadth in-house** — G/A/A provide *substrate* and let ISVs build verticals. The flat-catalog *structure* is clean; the *scope* (owning banking + insurance + manufacturing + logistics as products) is GTM aspiration, not day-0 platform architecture.
- **Verdict:** **questionable** on scope; structure aligned.
- **Founder question:** Is the full first-class vertical catalog genuine day-0 architectural scope, or substrate + ISV-built verticals? (Recurring own-everything tension at catalog granularity.)

### F5. LINUX "Rust Talos" node-OS (beat-or-parity vs Talos) + Rust-vs-Go security claim — QUESTIONABLE
- **Decision:** LINUX ADR-0025: a Rust, immutable, Talos-config-compatible node-OS; staged ladder Talos-day-0 → own-node-OS-when-proven; argues Rust beats Go-Talos on security.
- **G/A/A reality:** Owning the node-OS is what the most vertically-integrated infra does (AWS Bottlerocket is exactly a from-scratch minimal node-OS — though Bottlerocket is Rust-userspace on Linux, *adopting* not beating an OS). The **Rust-vs-Go security claim is honestly hedged**: ADR-0025 concedes Go is *also* memory-safe (so that win is shared, unlike Rust-vs-C), and narrows the real Rust edge to no-GC/smaller-TCB, compile-time data-race freedom, bounded `unsafe` vs cgo, and type-state capabilities — and explicitly marks the advantage "structural, not yet proven; no security claim ships until measured vs Talos over a span." That measured-before-claimed gate is the **correct** posture. The staged ladder (adopt Talos day-0, replace when proven) aligns with the own-when-proven ratchet.
- **Verdict:** **questionable** (own-the-node-OS is a large bet, unproven), but the *epistemics* (measured-before-claimed, adopt-then-replace ladder) are **aligned**. Note: this competes with SOURCE ADR-0375's adoption of *actual* Talos — a cross-side tension to surface.
- **Founder question:** Does the merged stack adopt actual Talos (source ADR-0375) and treat the Rust node-OS as a deferred OWN_EARLY behind a beat-or-parity scorecard, or commit to the Rust node-OS as the destination? Both can't be the day-0 node-OS.

---

## G. CHOICES THAT ARE HYPERSCALER-CORRECT — confirmed aligned (keep)

These are the founder-named "confirm the right ones" set. All pass the G/A/A lens; **keep** (vocabulary-amend only where retired terms leak).

| Choice | ADR(s) | Why G/A/A would do this |
|---|---|---|
| **Cellular architecture / bounded blast radius** | SOURCE 0009, LINUX 0012 | AWS cell-based arch, Google per-locale shards, Azure scale-units. LINUX 0012 correctly grounds it in K8s's documented ~5k-node/150k-pod limits → scale *above* a cluster, never one infinite cluster. Textbook-correct. |
| **Assume-breach / strength-by-blast-radius-not-authorship** | LINUX 0023 | Google BeyondProd, AWS Nitro defense-in-depth, NIST 800-207 zero-trust. "Authorship is the weakest trust signal; microVM-per-pod default even for first-party; confidential (SEV-SNP/TDX) for crown jewels." Exactly the hyperscaler doctrine. |
| **Conformance / claims-gate / evidence-gated promotion** | SOURCE 0133/0139/0142, `oya gate`/`oya verify` | Google Readiness reviews, AWS Well-Architected, evidence-before-promote. The "no claim ships unmeasured" discipline (also LINUX 0024/0025 conformance-gates) is hyperscaler-grade. (Caveat: ADR-0133's single continuous mega-BLOCKER lane vs N per-axis scorecards is the one merits-questionable variant — see its founder question.) |
| **Distroless / minimal-TCB / immutable node** | SOURCE 0375 (Talos immutable), LINUX 0025 (no-shell, A/B, signed images), 0023 (read-only rootfs) | Google distroless, Bottlerocket, Talos no-SSH immutable model. Correct. |
| **Zitadel primary OIDC IdP** | SOURCE 0187 | Adopt a proven multi-tenant-native OSS IdP (OIDC+SAML+SCIM+WebAuthn in one binary, air-gappable, API-first) rather than build identity day-0. Correct build-vs-buy at the most-regulated substrate. |
| **Milvus canonical vector store** | SOURCE 0192 | NVIDIA AI Enterprise reference stack; GPU index build; disaggregated compute/storage/coord. Hyperscaler-validated. |
| **Observability: Loki/Tempo/Mimir/Grafana (AGPL self-host carve-out)** | SOURCE 0383 (sup. 0042) | The LGTM stack self-hosted within cells with the AGPL network clause satisfied is the standard self-hosted-observability answer; the license reasoning is sound. (0383 must be declared authoritative over the finer ADR-0186 5-stage layering.) |
| **Cedar app-authz + Kubewarden admission separation** | SOURCE 0243/0246/0379 | AWS Cedar engine + clean L7-authz / k8s-admission separation. Correct layering. |
| **Talos + CAPI + ArgoCD fleet substrate** | SOURCE 0375 | The canonical hyperscaler-pattern OSS substrate for bare-metal k8s fleets; correctly retires the kubeadm/containerd/Istio onprem stack. |
| **Argo Workflows CI + Buck2 build + ArgoCD/Rollouts CD** | SOURCE 0511/0392/0408 | k8s-native, CNCF, self-hostable across every deployment context. The corrected CI destination passes the lens. |

---

## H. ONE DURABILITY OUTLIER A HYPERSCALER WOULD NOT SHIP

### H1. Best-effort (no parent-dir fsync) durability on audit-adjacent JSONL — MISALIGNED
- **Decision:** SOURCE ADR-0098 (Branch Y): supervisor writes inbox/outbox/dead-letter JSONL with best-effort durability (no `fsync` of file + parent dir; power loss can lose the newest record), on the same files ADR-0096 mandates crash-atomic Cedar+audit writes for.
- **G/A/A reality:** **No hyperscaler ships best-effort, no-dirfd-fsync writes for audit/spend-adjacent surfaces.** Audit chains and Cedar-enforcement rows are durability-critical; losing the newest record on power loss is a compliance gap. The zero-net-new-deps motivation is fine; the durability downgrade on audit-adjacent files is not.
- **Verdict:** **misaligned** (for the audit-adjacent subset).
- **Recommendation:** **amend** — force full power-loss durability (Branch X) for any file carrying audit-chain or Cedar-enforcement / spend rows; best-effort is acceptable only for genuinely-reconstructable queue state.
- **Founder question:** Is best-effort durability acceptable for a supervisor that also emits audit-chain + Cedar-enforcement rows, or does the audit/spend content force full fsync(file+parent) durability for those files (ADR-0096 ↔ ADR-0098 contradiction)?

---

## I. SUMMARY VERDICT TABLE

| # | Choice | Verdict | Reco | Founder call? |
|---|---|---|---|---|
| A1 | Jenkins as destination CI | misaligned | archive (done via 0511) | no |
| A2 | Argo Workflows CI | aligned | keep + re-sequence to Accepted | no |
| A3 | Buck2 build graph | aligned | keep + re-sequence | no |
| A4 | Bespoke `oya-ci` Prow day-0 | questionable (timing) | amend to numeric-trigger gate | **yes** |
| B1 | Forgejo canonical forge | questionable (transitory-OK) | keep transitory framing | no |
| B2 | Bespoke monorepo-VCS destination | aligned | keep (model own-when-proven) | no |
| B3 | GitHub-native automation substrate | misaligned (as substrate) | amend forge-neutral | **yes** |
| C1 | Cedar universal authz | aligned | keep (vocab amend) | no |
| C2 | LINUX owned compile-to-Rust policy | questionable (timing) | keep (own-when-proven) | **yes** |
| C3 | Kubewarden default admission | aligned | keep + amend 0039 | no |
| D1 | SOURCE best-of-breed data tier | aligned | keep (license hygiene amend) | no |
| D2 | LINUX own-DB "eliminate Postgres" | questionable (framing) | amend headline to differentiator-layer | **yes** |
| D3 | Pulsar+Oxia (Kafka retired) | aligned | keep + amend refs | no |
| D4 | Valkey not Redis | aligned | keep + amend vocab | no |
| E1 | wasmtime+Firecracker+Kata ladder | aligned | keep (doc reconcile) | no |
| E2 | LINUX framekernel "we are the host" | questionable (largest bet) | keep staged; keep 0014 port | **yes** (fault-line) |
| F1 | DCIM day-0 + no-silicon absolute | questionable (both) | re-sequence DCIM; reframe silicon | **yes** |
| F2 | Bespoke workflow engine day-0 | questionable (own-too-early) | port + own diff-layer | **yes** |
| F3 | 5 native client stacks/product | questionable (resourcing) | 1–2 day-0, defer rest | **yes** |
| F4 | Flat vertical catalog as owned µsvcs | questionable (scope) | substrate + ISV verticals? | **yes** |
| F5 | LINUX Rust-Talos node-OS | questionable (big bet, good epistemics) | adopt Talos day-0, own-when-proven | **yes** |
| G* | cellular / assume-breach / claims-gate / distroless / Zitadel / Milvus / LGTM / Talos / Argo+Buck2 | aligned | keep | no |
| H1 | Best-effort fsync on audit JSONL | misaligned | amend to full durability | **yes** |

---

## J. CROSS-CUTTING OBSERVATION FOR THE FOUNDER MANDATE

The single recurring pattern across every "questionable" verdict (A4, B3-timing, C2, D2, F1, F2, F3, F4, F5, E2) is **own-day-0 vs own-when-proven**. Source and LINUX **already share the own-when-proven ratchet language** (SOURCE ADR-0211/0173, LINUX ADR-0019/0020 four-axis scoring) — the disagreement is never the *principle*, only the *trigger threshold*. The hyperscaler answer is consistent: **adopt proven OSS behind a port, own only when a named numeric forcing-function fires** (ADR-0510's deferred-bespoke-VCS is the model done right; ADR-0513's day-0 bespoke-Prow and ADR-0001's "eliminate Postgres" are the model done early). The cleanest masterplan invariant to backfill from this pass is a **single own-when-proven trigger doctrine** that every OWN_DAY0 bet must satisfy — which would resolve A4, C2, D2, F1, F2, F5 mechanically rather than per-ADR.

*End of register. READ-ONLY pass — no audited doc modified; this artifact is the only write.*
