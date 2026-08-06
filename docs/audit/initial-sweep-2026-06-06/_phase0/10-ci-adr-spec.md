# Phase-0 / 10 — The ONE Canonical CI/CD ADR: Authoring Spec (the `oya-ci-required` producer spec)

**Lane:** Phase-0 (false-green firewall) · D-SEQUENCE Step-0→Phase-0 · D-CICD.
**Status of THIS file:** READ-ONLY authoring spec. It does NOT author the ADR; it specifies what the
ADR must say. No source file was edited. Final-message digest follows the canon.
**Authority chain:** founder rulings **D-CICD / D3 / D-LAYER / D-SCOPE-UNIFY / D-PURESPLIT / D-SEQUENCE / D-DOCTRINE**
in `docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md` (lines 70, 156–161, 168–169, 174–175, 177–184, 195–200) — the ADRs are SSOT, this spec is generated FROM them.

---

## 0. What I read (coverage; no silent caps)

| File (real path) | Lines | What it is |
|---|---|---|
| `source/docs/decisions/ADR-0709-general-live-apex.md` | 114 (full) | **Accepted, founder-locked.** Bespoke-Rust 1:1 Prow reimplementation (hook/plank/crier/ProwJob/tide/deck/sinker/plugins). |
| `source/docs/decisions/ADR-0709-general-live-apex.md` | 122 (full) | **Proposed.** Adopt Argo Workflows wholesale; explicitly REJECTS "bespoke CI controller now" and REJECTS Tekton. `supersedes:[ADR-0359]`, `superseded_by:[]`. |
| `source/docs/decisions/ADR-0709-general-live-apex.md` | 237 (full) | **Proposed.** Narrows to ADR-0513 Phase-1; 6 deliverables (D1–D6); `depends_on:[0392,0408]`. |
| `source/docs/decisions/ADR-0709-general-live-apex.md` | 1019 (front-matter + Status §181 + Decision §270–302 + Rejected §694–775 read; §G multispectrum, §H sunset, §I/J skimmed via heading grep) | **Proposed.** Jenkins(LTS)+ArgoCD substrate; Jenkins AUGMENTS GitHub Actions. §F.2 rejects Tekton, §F.3 rejects Flux (selects ArgoCD), §F.1 rejects Jenkins X. |
| `source/docs/decisions/ADR-0709-general-live-apex.md` | 42 (full) | **Superseded** (`superseded_by:[ADR-0511]`; `amends:[ADR-0349]`). Body still says "Proposed" (status/body drift). |
| `source/docs/decisions/ADR-0709-general-live-apex.md` | 68 (full) | **Proposed.** Executes 0359/0349; license-vetted supply-chain stack; `amends:[ADR-0359]`. |
| `source/docs/decisions/ADR-0408-buck2-driven-ci-cd.md` | 75 (full) | **Proposed.** Buck2 RBE + `cquery rdeps` affected-targets + cache-backed image builds; `supersedes:[ADR-0358]`; "complementary to 0359, NOT superseded." |
| `linux/docs/research/bespoke-ci-design-2026-06-06/40-PRODUCT-SPEC.md` | 518 (full) | **The design SSOT** (task #17 output). Two nouns / four faces / six layers reshape; §11 supersession edges. **NOTE: lives in the LINUX repo, NOT in `source/docs/research/`** (the lane brief's expected source path is ABSENT — see §6). |
| `source/infra/branch-protection/dev.json` | 26 (full) | Live façade: requires only `oya-ci-required`; self-disclaims as P0.0 target "not a live-enforcement claim until the producer posts on the candidate SHA." |
| `source/.github/branch-protection.yaml` | dev block (L55–56) | Mirror: `required_status_checks: [oya-ci-required]`; header self-disclaims "not Phase-0 exit authority until a trusted cloud-ci/oya-ci required context is live." |
| `source/specs/phase0-ci-enforcement-baseline.json` | 282 (full) | The P0.0 RED gap packet + RED/GREEN fixture contract + producer requirements (`p0_0_green:false`). |
| `source/specs/phase0-ci-enforcement-result-schema.json` | 162 (full) | The structured result-bundle schema the producer must emit (candidate_sha, producer{context,kind,trusted_control_state}, fixture_results[], provenance, claim_boundary). |
| `source/docs/decisions/ADR-0709-general-live-apex.md` | status line only | **accepted.** Own merge-queue (webhook-driven), the `_adr` cited by dev.json; superseded-in-mechanism by graph `conflicts(a,b)`. |
| `source/registry/foundation-bypasses/byp_adr_0349_jenkins_argocd_substrate.yaml` | path confirmed (content not opened) | The ADR-0349 bypass record D-LAYER ruling §78(2) says to RESOLVE. |

**NOT covered (stated honestly):** ADR-0349 §G multispectrum facets, §H sunset, §I cross-refs, §J completion (skimmed by heading-grep only — not load-bearing for the consolidation map). ADR-0124 body (only status + first line read). ADR-0367/0366/0369/0392/0358 NOT read this lane (referenced by the canon as phased-relations; their edges are taken from D3/the product-spec §11, not re-verified against their own front-matter). The bypass-record YAML content was located but not opened.

---

## A. Consolidation / supersession map

### A.0 The founder ruling that governs this (verbatim anchors)

- **D-CICD (canon L169):** *"Consolidate ADR-0349/0359/0361/0408/0511/0513/0514 → ONE canonical CI/CD ADR. Build-first-cutover-later. Door: one-way."*
- **D3 (canon L157–161):** *"RATIFY the unified `Run`+graph reshape of ADR-0513… seed ONE clean ratifying ADR (ADR-0000+ series) reshaping 0513; supersede/relate 0511/0124; mark 0369/0367/0366 phased. Build-first-cutover-later: Jenkins (0349/0359/0361) + Argo stay OPERATIVE until oya-ci is built and proven → cutover → THEN retire (mark 'superseded-on-cutover', NOT archived now)."*
- **D-LAYER ruling §78(2) (canon L78):** *"CI-cluster = DROP the Jenkins/Argo ADR debt — ARCHIVE/DROP 0349+0361 (Proposed, never-ratified), 0359 (already Superseded), 0511 Argo (conflicts with the D3 oya-ci ruling); only oya-ci 0513 is ratified canon; the physical Jenkins scaffold … stay OPERATIVE as an explicitly-UNRATIFIED de-facto bridge under build-first-cutover-later, retired when oya-ci is built+proven; resolve the byp_adr_0349 bypass record."*
- **The two rulings are reconciled, not in conflict:** D3's "mark superseded-on-cutover, NOT archived now" (the *doc-lifecycle* verb) and D-LAYER's "DROP the ADR debt" (the *authority* verb) compose: the seven cluster ADRs lose decision-AUTHORITY immediately (superseded by the new ADR on its acceptance) but the physical Jenkins/Argo scaffold + the Jenkins-shaped 0349/0359/0361 docs remain operative/on-disk as an explicitly-unratified bridge until cutover, then are retired. **The new ADR is the single authority from day-0; the bridge is operative-but-unratified until cutover.**

### A.1 The new ADR

- **Number:** next free in the clean `ADR-0000+` ratifying series (per D3 L161 + D-SEQUENCE Phase-1 A-CI lane). Exact integer assigned at renumber-time (A-INTEGRITY lane resolves the dup-0377 + numbering drift first; do NOT hard-code a number in this spec). Working handle: **`ADR-00NN-oya-ci-cd-unified-rust-native-cicd`**.
- **Title:** *"oya-ci / oya-cd — the unified Rust-native CI/CD product (two nouns, four faces, six layers; adopts the patterns of Prow + Tekton + Argo-WF + Argo-CD + Argo-Rollouts in Rust)."*
- **Status on author:** Accepted (door:one-way; founder-ruled). It is the **ratifying ADR** that reshapes the founder-locked ADR-0513 — so it inherits 0513's Accepted authority rather than re-opening it.
- **Home product surfaces (D-CICD L169 + D-LAYER L70 + D-SCOPE-UNIFY L175):** `cloud/cloud-scm` + `cloud/cloud-ci` + `cloud/cloud-cd` — tenant-facing dogfood products under the `cloud/` tree (D-PURESPLIT: a service dir exists ONLY under `oya/` or `cloud/`). Backlog #19 home.

### A.2 The 7→1 collapse table (each ADR's fate + exact edges)

| ADR | Current status (front-matter, verified) | Fate in the new ADR | Exact `superseded_by` / `supersedes` edge to write | Doc-lifecycle verb |
|---|---|---|---|---|
| **0513** oya-ci bespoke-Rust Prow | **Accepted** (founder-locked) | **RESHAPED → ratified by the new ADR.** Its bespoke instinct is the seed; the "clone Prow's 8 components" framing is superseded by two-nouns/four-faces. | new ADR `supersedes:[0513]`; 0513 gets `superseded_by:[00NN]`, status→`Superseded` (reshaped, not discarded). | Authority moves to new ADR on accept. Keep 0513 on disk (lineage). |
| **0511** Argo-WF orchestrator | **Proposed** (`supersedes:[0359]`, `superseded_by:[]`) | **RESOLVED by rejecting both poles** (not adopt-Argo-wholesale, not clone-all-Prow). Argo DAG/event IDEAS re-enter as Face C / layer (b) behind `EventBus`/`Scheduler`; etcd-CRD substrate dropped. | new ADR `supersedes:[0511]`; 0511 gets `superseded_by:[00NN]`, status `Proposed→Superseded`. | **Drop the ADR debt** (D-LAYER): never-ratified; supersede + archive-eligible after Phase-1. |
| **0514** target-arch + hyperscaler remediation | **Proposed** (`depends_on:[0392,0408]`) | **HONORED + EXTENDED → folded as substrate.** Its "minimal robust shape / 3 hops / one language" = Phase-1; CAS-hit>60% deferral kept as a literal promotion gate; D1–D6 deliverables re-homed onto the new ADR's Phase-1 deliverable list. | new ADR `supersedes:[0514]` (status `Proposed→Superseded`); the new ADR **re-states 0514's D1–D6** so nothing is lost. **(NOTE: D-LAYER §78(1) earlier said "0408/0514 = AMEND in-place, NOT archived"; D-CICD L169 (later, T1) names 0514 in the 7→1 collapse. Resolve to: collapse 0514's CI/CD-target content INTO the new ADR (supersede), but keep its non-CI deliverable tracking (hermetic toolchain #83, buckify idempotence) alive as Phase-1 work-items. Flag for founder: confirm collapse-vs-amend for 0514.)** | Superseded; keep on disk until Phase-1 deliverables land. |
| **0408** Buck2-driven CI/CD | **Proposed** (`supersedes:[0358]`) | **ADOPTED as the core (layer d).** Buck2 + NativeLink CAS + `cquery rdeps` = Phase-1 substrate; "buck2-native OCI images" = long-run convergence of the owned build-engine endpoint. | **Relate, do NOT supersede the build-engine decision** — but the *CI/CD-orchestration* half collapses into the new ADR. Cleanest: new ADR `supersedes:[0408]` for its CI/CD-orchestration content, `related:[0392]` for the build-graph it drives. **(Same D-LAYER §78(1) "amend-in-place" vs D-CICD L169 "collapse" tension as 0514 — same founder flag.)** | Superseded-for-CI-orchestration; Buck2 build-graph (0392) stays authoritative. |
| **0349** Jenkins+ArgoCD substrate | **Proposed** (never ratified) | **SUPERSEDED.** Jenkins-augments-GHA + ArgoCD-CD are replaced by oya-ci (orchestrator) + oya-cd `DeliveryPlane` (which still hands off to ArgoCD/Argo-Rollouts as REUSE-behind-port). | new ADR `supersedes:[0349]`; 0349 `superseded_by:[00NN]`, status `Proposed→Superseded`. **Resolve `byp_adr_0349_jenkins_argocd_substrate.yaml`** (close/retarget the bypass record). | Bridge stays OPERATIVE-but-unratified until cutover; doc superseded now. |
| **0359** Jenkins replaces GHA | **Superseded** (`superseded_by:[0511]`, body says "Proposed" — drift) | **ALREADY superseded; re-point.** Its anti-GHA-SPOF verdict (remove metered third-party CI) is RETAINED in the new ADR. | **Re-point `superseded_by:[0511]` → `[00NN]`** (0511 itself is being superseded, so the chain must skip to the new ADR). Fix the status/body drift (body→Superseded). | Already superseded; keep, re-pointed. |
| **0361** Jenkins-native execution | **Proposed** (`amends:[0359]`) | **SUPERSEDED.** Its license-vetted supply-chain tool stack (cargo-deny / Opengrep / gitleaks / Syft / Trivy / osv / cosign / in-toto-SLSA / Kyverno) is RETAINED as the new ADR's gate-step tool list (layer e). | new ADR `supersedes:[0361]`; 0361 `superseded_by:[00NN]`, status `Proposed→Superseded`. `amends:[0359]` becomes historical. | Bridge promote-pipelines stay operative until cutover; doc superseded now. |

### A.3 The missing 0511↔0513 edge (the central seam the audit flagged) — FIXED

- **The defect (verified):** ADR-0511 (`superseded_by:[]`) and ADR-0513 (no `supersedes`/`superseded_by` front-matter at all; only prose `relates:`) have **no edge between them**, yet they directly contradict: 0511 §"Rejected alternatives" L87 rejects "Bespoke CI controller now"; 0513 IS the bespoke CI controller, Accepted one day later (0511 dated 2026-05-29, 0513 dated 2026-05-30). 0511 never lists 0513 as a relation; 0513 never lists 0511. The corpus carries two live, opposite CI decisions with no link — the "0511↔0513 whiplash" the product-spec `00`/§11 calls the central seam.
- **The fix (D3 L158 "resolves 0511"; product-spec §11 L483–487 "the 0511↔0513 whiplash is closed"):** the new ADR is the single node that resolves BOTH. It supersedes 0511 (reject-Argo-wholesale pole) AND reshapes 0513 (clone-all-Prow pole). The edge is not 0511→0513 directly; it is **0511→00NN and 0513→00NN**, with the new ADR's body explicitly recording: "ADR-0511 and ADR-0513 were contradictory and unlinked; this ADR supersedes both poles and keeps Argo's DAG/event ideas (0511) on the bespoke-Rust controller (0513)." This closes the seam by collapsing both into one authority rather than patching a single cross-edge between two superseded docs.

### A.4 Phased-relation ADRs (NOT collapsed — related + marked phased, per D3 L161 / product-spec §11)

These are NOT in the 7→1 set; the new ADR `related:`-links them and marks each phase:

| ADR | Status | Relation the new ADR records |
|---|---|---|
| 0367 trustless pre-merge verification | (per canon) | **Keystone, built EARLY** (Phase-1): signature *is* the check. `related`. |
| 0124 own merge-queue webhook-driven | **accepted** | **Superseded-in-mechanism, preserved-in-intent**: file-overlap clustering → graph-exact `conflicts(a,b)`; 20-row blocker taxonomy re-derived. new ADR `supersedes:[0124]` (mechanism) per D3 L161. |
| 0369 stacked-trunk + speculative merge-train | (per canon) | **Adopted, phased to Phase-3** (queue-depth-gated). `related`. |
| 0366 agentic self-enforcing/self-repair | (per canon) | **Raison d'être, sequenced LAST** (Phase-3 semi → Phase-5 full). `related`. |
| 0392 Buck2 canonical build-graph | (per canon) | **Build-graph the CI drives** — stays authoritative, `related` (NOT superseded). |
| 0181 cosign image-promotion ladder | (per canon) | Re-homes onto the oya-cd signer; `related`. |

---

## B. Component shape — which face reimplements which pattern in Rust

The new ADR's normative architecture is the product-spec's **two nouns / four faces / six layers + a brain** (`40-PRODUCT-SPEC.md` §4, L168–199). This is the canonical mapping the ADR must declare. **"Do what Go does, cloud-native, in Rust" (D-CICD L169) — adopt the PATTERN, reimplement in Rust; never vendor the Go binary** (Argo/Tekton/Prow Go is NOT run; the patterns are reimplemented). Substrate that is not the moat (REAPI/CAS, event-bus, kube-scheduler, cosign wire) is REUSE-behind-port.

### B.1 The two nouns (the unification primitive)

1. **`Run`** — one typed, content-addressed object that is simultaneously a Prow `ProwJob` × a Tekton `TaskRun`/`PipelineRun` × an Argo `Workflow`. Every producer emits it; every consumer reconciles it. MVP state = single Postgres table behind `RunStore` (NOT etcd CRDs — "the Argo original sin we refuse"; NOT the sharded store until write-rate is measured).
2. **The build graph (Buck2) + CAS (REAPI v2)** — against which `affected-by`, `conflicts(a,b)`, `cache-key`, `provenance-subject` are the **same family of query**. This is the brain / the moat (the only thing no off-the-shelf system gives).

### B.2 Face → pattern → Rust reimplementation table (the ADR's normative §)

| Face | Adopts the pattern of | Rust reimplementation (what the ADR commits to OWN) | Substrate REUSED behind a port | Ownership tag (product-spec §3.1/§7) |
|---|---|---|---|---|
| **A — Ingress + trustless gate + submit queue** | **Prow** (`hook` + Tide merge-queue) + Uber SubmitQueue | Forgejo-native webhook gateway → CloudEvent → one `Run` (HMAC fail-closed, ADR-0374 exists); trustless gate where **the signature IS the check** (producer ≠ verifier ≠ approver, ADR-0367); serial Tide-invariant merge queue (batches>singletons, retest-base, abort-on-HEAD); graph-exact `conflicts(a,b)` for disjoint concurrent landing. | `ForgeAdapter` (no external-search / single-token SPOF — the Prow GitHub coupling oyatie does not copy). | **OWN-now** (gate = keystone). Speculation tree = OWN-when-proven (Phase-3, queue-p50 gate). |
| **B — Typed-Task executor** | **Tekton** (typed `Task`/`Pipeline` + Step) | Rust-typed Task IR (single-Task / multi-Step / one-pod) where a Step is a REAPI action — **typed in Rust, not YAML**. Combinators (`finally`/`when`/`matrix`) + dynamic fan-out = v1. | `TaskRuntime` (vendored TaskRun → owned Rust executor). | **OWN-now** (single-Task) → combinators OWN-when-proven. |
| **C — DAG engine / scheduler** | **Argo Workflows** + **Argo Events** | DAG engine + brain-driven (graph-affected) selection; CloudEvents correlation (AND/OR); node memoization is **CAS-backed, not ConfigMaps**. **Argo's etcd-CRD substrate is explicitly DROPPED** (keep ideas, discard substrate — the 0511 resolution). | `EventBus` (NATS JetStream / Kafka), `Scheduler` (K8s as one impl). | DAG/events = OWN-when-proven (v1, behind ports). |
| **D — Out-of-band provenance signer** | **Tekton Chains** / SLSA | Out-of-band signer that attests each completed `Run` to `slsa/v1`, signs (cosign), writes an immutable ledger; **hermeticity-via-cache → SLSA L3** (exceeds Chains' L2). | `Attestor`/`SigningBackend` (cosign / Fulcio / Rekor wire). | gate-signature OWN-now → full signer/ledger OWN-when-proven (gate-proven). |
| **CD face (oya-cd) — GitOps + progressive delivery** | **Argo CD** (declarative GitOps) + **Argo Rollouts** (canary / blue-green / analysis) | `DeliveryPlane` port: oya-cd owns the signed-provenance handoff + the cosign-verified-on-sync + audit-chain-emit contract (the parts ADR-0349 vested in ArgoCD). **Integration plane ≠ delivery plane** — kept separate and pluggable. Progressive delivery = canary/blue-green/analysis (Kayenta-class). | ArgoCD / Argo-Rollouts / Spinnaker-class CD = **REUSE-behind-`DeliveryPlane`** (NOT reimplemented in MVP). | REUSE-behind-port (CD-plane phase). Owned later only if proven. |
| **The brain (layer d, under all faces)** | Bazel/Buck2 RE + hyperscaler graph | Buck2 build graph as a queryable object; REAPI v2 + CAS + ActionCache; `cquery rdeps` graph-sound affected selection; later: predictive test-selection / land-ranker / flake-aware culprit-finding. | REAPI v2 (NativeLink → owned scheduler when CAS-hit>60% sustained). | graph+CAS OWN-now → ranker/scheduler OWN-when-proven. |
| **Core (the two nouns)** | Prow event-sourced control plane × hyperscaler graph | `RunStore` (Postgres single-table → sharded), K8s as a `Scheduler` impl. | Postgres (REUSE). | Postgres OWN-now → sharded OWN-when-proven (write-rate gate). |

### B.3 Build engine (explicit, product-spec §5; canon D3 L159)

oya-ci **owns the `BuildSpec`/`BuildStrategy` contract, NOT the engine** (founder deferred owning the engine). Vendored bridge behind the port: **BuildKit per-pool** (LLB + remote cache + native SPDX SBOM/SLSA `mode=max`) + **Buildah Job-per-build hardened ~7-cap** (hostile-tenant tier). **Scratch/distroless default; SBOM(SPDX)+SLSA provenance HARD-required day-0. Buildpacks opt-in only. Kaniko ARCHIVED** (Google archived it 2025-06-03). Owned Rust build engine = OWN-when-proven endpoint (Phase-5), convergence point with ADR-0408's buck2-native OCI.

### B.4 Reshape of the existing Prow-shaped scaffold (product-spec §6 L277–290)

Current scaffold maps Prow 1:1 — `oya/ci-webhook-gateway`=`hook`, `oya/ci-controller`=`plank`, `oya/ci-tide`=`tide`. The new ADR: **KEEP** gateway (becomes the `Run` producer) + controller (KEEP shape, OWN the `Run` lifecycle — do NOT transliterate Prow's plank state-machine) + tide (KEEP invariants, RESHAPE engine to serial-now/speculation-later); **SUPERSEDE** the frozen ProwJob enum (presubmit/postsubmit/periodic/batch → scheduling policy over the graph); **DEFER** deck/plugins/sinker to the roadmap.

---

## C. The `oya-ci-required` PRODUCER CONTRACT (the firewall keystone)

### C.1 The façade being killed (live-verified, the exact mechanism of the drift)

- `source/infra/branch-protection/dev.json` L9–12: `required_status_checks.contexts: ["oya-ci-required"]` — and L2/L8/L25 self-disclaim: *"this file is not Phase-0 exit authority until a trusted cloud-ci/oya-ci producer is live and applied"* + *"not a live-enforcement claim until the ruleset is applied and the producer posts on the candidate SHA."*
- `source/.github/branch-protection.yaml` L55–56: `required_status_checks: [oya-ci-required]` — header self-disclaims *"not Phase-0 exit authority until a trusted cloud-ci/oya-ci required context is live."*
- `source/specs/phase0-ci-enforcement-baseline.json` L73–84: **live GitHub** `dev/protection` actually requires `[cargo-fmt, cargo-check, cargo-clippy, cargo-nextest, oya-pr-review]` — and `oya-pr-review`'s producer returns **HTTP 501** (dev.json L8), so it cannot block. `oya-ci-required` is **required by NOBODY live**; gap = *"no oya-ci-required or cloud-ci-required context required live."* Verdict L214: `P0.0_RED_blocked_until_cloud_ci_required_context_is_live`. **0 gates block a merge today.** This IS D-SEQUENCE's "enforcement is a FAÇADE."

### C.2 What the producer is (the first real piece of oya-ci)

- **Identity:** `oya-ci-controller` (the kept `plank`/Face-A reconciler), deployed as a service (kube-rs on Talos), OR a minimal Rust bridge adapter for the firewall slice (baseline L267 allows either). It is a **deployed service, not config on the branch it gates** — so a bad PR cannot break it (kills ADR-0513 failure-mode-2 self-deadlock).
- **Trust model (trunk-sourced / trustless, baseline L201–207, L263–268; ADR-0367; ADR-0514-D2):**
  1. **Gate definitions come from TRUSTED trunk/controller state** — the controller clones `dev`'s gate catalog SEPARATELY; the candidate PR tree is passed as **DATA UNDER TEST ONLY** (PR-ref as data, never as a producer/gate-definition source).
  2. **A PR cannot weaken its own gate** — editing `buck2-affected-gate.sh` / the Job spec / the branch-protection mapping / runner isolation in the candidate has NO effect (RED fixtures `tc-0.0.1-bad-*-producer`, `tc-0.0.1a-bad-candidate-mutable-producer`, `tc-0.0.1a-bad-candidate-deletes-trusted-target` MUST fail). This is the `pull_request` vs `pull_request_target` security model, in Rust.
  3. **The signature IS the check** (Face A / ADR-0367): merge authority = a verifiable signature over hermetically re-executed evidence, producer ≠ verifier ≠ approver.
  4. **No `oya` CLI is merge authority** (baseline L208–213, L276): `oya verify` / `oya gate` may remain LOCAL migration evidence only; they are NOT in any protected required context and NOT the cloud-ci producer command. RED fixtures `tc-0.0.1-bad-legacy-oya-cli-authority`, `-bad-oya-verify-affected-producer`, `-bad-oya-gate-run-all-required-producer`, `-bad-buck2-affected-only-producer` MUST fail.

### C.3 What it posts, on which SHA (the contract)

- **Context name:** `oya-ci-required` (canon name; `cloud-ci-required` is the accepted alias — result-schema `producer.context` enum = `{cloud-ci-required, oya-ci-required, missing}`).
- **On which SHA:** the **candidate commit SHA** of the PR head (result-schema `candidate_sha`: 40-hex, the commit the required context is posted to). One required context per candidate SHA, posted from **trusted control state**.
- **Output bundle (MUST conform to `source/specs/phase0-ci-enforcement-result-schema.json`):** a `Phase0CiEnforcementResult` object with required keys —
  - `candidate_sha` (40-hex),
  - `required_context` ∈ {oya-ci-required, cloud-ci-required, missing},
  - `producer{ context, kind, trusted_control_state:bool, gate_definition_source, candidate_bytes_policy }`,
  - `fixture_results[]` (each: `fixture_id`, `expected_verdict` RED|GREEN, `observed_verdict` RED|GREEN, `violations[]`),
  - `observed_verdict` RED|GREEN,
  - `provenance{ recorded_at, sources[] }`,
  - `claim_boundary{ p0_0_green:bool, phase0_complete:bool }`.
- **Override / kill-switch (baseline L215–220):** any override requires a controller-enforced packet — TTL + reviewer acknowledgment + audit-chain event + named owner + blast-radius statement + revert/fix follow-up. RED fixture `tc-0.0.2-bad-override-without-ttl-audit` MUST fail.
- **Tenant isolation (baseline L245–262):** even at tenant-zero, the producer must not leak across `tenant_id` on any of 11 surfaces (identity, secrets, runners, workspaces, caches, artifacts, logs/evidence, release ledgers, deploy targets, status callbacks, audit events). RED fixture `tc-0.0.3-bad-cross-tenant-shared-cache` MUST fail.

### C.4 Robustness bar — RED/GREEN fixtures the producer MUST honor (founder bar; baseline `fixture_set`)

The ADR's `Verification` section MUST point at these real fixtures (no thin/flaky enforcement; every gate proven by a known-bad it fails + a known-good it passes + proof it runs in CI and BLOCKS):

- **GREEN (the one it MUST pass):** `tc-0.0-good-cloud-ci-required-and-isolated.json`.
- **RED (each MUST fail), 10 pairs (baseline L145–195):** `tc-0.0.1-bad-buck2-affected-only-producer`, `tc-0.0.1-bad-legacy-oya-cli-authority`, `tc-0.0.1-bad-missing-required-context`, `tc-0.0.1-bad-oya-gate-run-all-required-producer`, `tc-0.0.1-bad-oya-verify-affected-producer`, `tc-0.0.1-bad-required-context-present-not-required`, `tc-0.0.1a-bad-candidate-mutable-producer`, `tc-0.0.1a-bad-candidate-deletes-trusted-target`, `tc-0.0.2-bad-override-without-ttl-audit`, `tc-0.0.3-bad-cross-tenant-shared-cache`.
- **Proof-it-BLOCKS:** after the producer posts `oya-ci-required` on candidate SHAs, **apply/sync the live GitHub ruleset** so `oya-ci-required` is genuinely required (closing the dev.json/yaml→live-GitHub drift), then re-run T0.0. Only then may `claim_boundary.p0_0_green` flip true. Until then the ADR + every authority surface MUST use gap-packet language only (baseline L37–47 forbids: "Phase 0 complete", "P0.0 green", "mechanically enforced", "production-ready", "tenant-facing live service", "secure live isolation").

### C.5 Acceptance gate for THIS ADR (D-SEQUENCE Phase-0)

The ADR is the **producer spec**; Phase-0 is complete only when (1) this ADR is authored + accepted, (2) the live `oya-ci-required` producer posts conforming bundles on candidate SHAs from trusted control state, (3) the 4 keystone gates (cross-artifact-agreement · total-accounting · staleness-reaper · automation-ratchet, D-SEQUENCE L198) land over one generated accounting-registry, each with RED/GREEN fixtures it actually blocks. Only then do Phase-1 amendments (the 7→1 consolidation itself, A-CI lane) proceed GATE-VERIFIED.

---

## D. Home = cloud-scm / cloud-ci / cloud-cd as tenant-facing dogfood products (backlog #19)

- **Doctrine (D-LAYER L70 + D-SCOPE-UNIFY L175):** oyatie `cloud/` is WHERE oya products dogfood; products run on cloud as tenant workloads. **Cloud is sold (IaaS/PaaS); products are sold (SaaS); the products prove the cloud by running on it.** oya-ci builds products and deploys them TO oyatie cloud → the full dogfood loop. Think like a hyperscaler: cloud is sold; oya = tenant `oyatie-internal`.
- **The three homes (D-CICD L169 "cloud-scm/cloud-ci/cloud-cd as tenant-facing dogfood products"):**
  - **`cloud/cloud-scm`** — the bespoke VCS destination (Forgejo transitory → bespoke; the `ForgeAdapter` seam).
  - **`cloud/cloud-ci`** — Faces A/B/C + the brain (gate, merge-queue, typed-Task executor, DAG engine, build-graph/CAS). Owns the `oya-ci-required` producer.
  - **`cloud/cloud-cd`** — Face D + the CD face (`DeliveryPlane`, GitOps + progressive delivery; ArgoCD/Argo-Rollouts REUSE-behind-port).
- **D-PURESPLIT constraint (canon L172):** these live under `cloud/` ONLY (never a 3rd `microservices/` tree); each service dir exactly once, flat-colocated; **no `oya/`→`cloud/` internal dep** (dogfood purity — oya products consume cloud as a tenant, not via code-coupling). Amend ADR-0131/0512 accordingly (A-STRUCT lane).
- **Dogfood→product duality (product-spec §8.1 #5):** built as tenant-zero of the eventual multi-tenant, API-first, self-hostable product; the dogfood hardens the exact control plane later sold. Multi-tenant isolation machinery is Phase-5 (gated on a real second tenant) — day-0 `tenant_id` is a seam, not machinery.

---

## E. Flags for the founder (do not silently resolve)

1. **0408/0514 collapse-vs-amend conflict (real, two founder rulings):** D-LAYER §78(1) (canon L78) says *"0408/0514 = AMEND in-place … NOT archived"*; D-CICD L169 (later, T1) lists BOTH 0408 and 0514 in the 7→1 collapse. This spec proposes: collapse their **CI/CD-orchestration** content into the new ADR (supersede), but keep their **non-orchestration deliverables** (0514 hermetic-toolchain #83, buckify idempotence; 0408's relation to the 0392 build-graph) alive as Phase-1 work-items / `related` edges. **Confirm collapse-vs-amend per ADR.**
2. **Product spec lives in the LINUX repo, not source.** The lane brief expected `source/docs/research/bespoke-ci-design-2026-06-06/40-PRODUCT-SPEC.md`; it is at `linux/docs/research/bespoke-ci-design-2026-06-06/40-PRODUCT-SPEC.md` (518 lines, full design SSOT). The new ADR must cite the real path; consider relocating the research bundle into `source/` during Phase-1 so the ADR's references resolve in the source corpus.
3. **ADR-0359 status/body drift:** front-matter `status: Superseded` but body line says "Proposed — 2026-05-25". Fix during re-pointing (A-INTEGRITY status-enum lane).
4. **New-ADR number not assigned here** (depends on A-INTEGRITY dup-0377 renumber + numbering re-derivation). Do not hard-code.
