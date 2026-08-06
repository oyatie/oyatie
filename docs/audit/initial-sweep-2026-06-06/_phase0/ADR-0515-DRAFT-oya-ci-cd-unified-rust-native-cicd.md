---
id: ADR-0515
title: "oya-ci / oya-cd — the unified Rust-native CI/CD product (two nouns, four faces, six layers; adopts the patterns of Prow + Tekton + Argo-Workflows + Argo-CD + Argo-Rollouts, reimplemented in Rust)"
status: Accepted
planning_impact: true
deciders: founder, council-architecture
date: 2026-06-06
door: one-way
supersedes: [ADR-0124, ADR-0349, ADR-0359, ADR-0361, ADR-0511, ADR-0513, ADR-0514]
superseded_by: []
depends_on: [ADR-0408, ADR-0392]
amends: []
related: [ADR-0111, ADR-0116, ADR-0181, ADR-0247, ADR-0363, ADR-0366, ADR-0367, ADR-0369, ADR-0374, ADR-0392, ADR-0408, ADR-0131, ADR-0512]
related_specs:
  - /specs/phase0-ci-enforcement-baseline.json
  - /specs/phase0-ci-enforcement-result-schema.json
  - /specs/masterplan.json
  - /specs/master-plan-sequencing.json
design_ssot: docs/research/bespoke-ci-design-2026-06-06/40-PRODUCT-SPEC.md  # (currently lives in the linux pilot repo; relocate into source/docs/research/ in Phase-1, A-STRUCT)
homes: [cloud/cloud-scm, cloud/cloud-ci, cloud/cloud-cd]
session_context:
  authored: 2026-06-06
  basis: "Founder rulings D-CICD / D3 / D-LAYER / D-SCOPE-UNIFY / D-PURESPLIT / D-SEQUENCE / D-DOCTRINE (decision-record-oyatie-canon.md). T1: 'bespoke prow + argo rollout + argo workflow + tekton + argo cd and other best-in-class cloud-native CI/CD — adopt the patterns, reimplement in Rust, cloud-native. Do what Go does, in a cloud-native manner, but in Rust.' 0408 stays SEPARATE (depends_on). New ADR = 0515 (additive on stable numbering per D13-AMENDED)."
purpose: >
  Establish ONE canonical CI/CD decision for oyatie — a bespoke, Rust-native, cloud-native
  CI/CD product (oya-ci + oya-cd) that ADOPTS THE PATTERNS of the best-in-class cloud-native
  CI/CD ecosystem (Prow, Tekton, Argo Workflows, Argo CD, Argo Rollouts) and REIMPLEMENTS them in
  Rust — never running the upstream Go binaries — unified by two nouns (a content-addressed `Run`
  + the Buck2 build-graph/CAS brain). Collapses the 7-way CI/CD ADR cluster
  (0124/0349/0359/0361/0511/0513/0514) into this single authority; depends on (does not absorb)
  the Buck2 build substrate (0408/0392). Declares the `oya-ci-required` producer contract that
  makes merge-gate enforcement REAL (the Phase-0 false-green firewall keystone).
---

# ADR-0515: oya-ci / oya-cd — the unified Rust-native CI/CD product

## Status

**Accepted — 2026-06-06 (founder-ruled; door: one-way).** This is the **ratifying ADR** that reshapes the founder-locked **ADR-0513** and resolves the entire CI/CD ADR cluster; it inherits 0513's accepted authority rather than re-opening it. It supersedes ADR-0124/0349/0359/0361/0511/0513/0514 and **depends on** (does not absorb) ADR-0408/0392 (the Buck2 build substrate, a distinct bounded context). Build-first-cutover-later: the physical Jenkins/Argo scaffold + the superseded Jenkins-shaped docs (0349/0359/0361) remain **operative-but-unratified** as an explicit bridge until cutover, then retire ("superseded-on-cutover," not archived now).

## Context

### 1. The cluster problem — seven contradictory CI/CD ADRs, one of them unlinked
The CI/CD decision was spread across seven ADRs that disagree and, in one case, directly contradict each other with **no supersession edge**:
- **ADR-0513** (Accepted, founder-locked) — bespoke-Rust 1:1 reimplementation of Prow's component shape (hook/plank/crier/ProwJob/tide/deck/sinker/plugins).
- **ADR-0511** (Proposed) — adopt **Argo Workflows** wholesale as the orchestrator; explicitly **rejects** "a bespoke CI controller now" and rejects Tekton. `superseded_by: []`.
- **The whiplash:** 0511 (2026-05-29) rejects the very thing 0513 (2026-05-30, Accepted) *is*, and neither lists the other — two live, opposite CI decisions with no link.
- **ADR-0514** (Proposed) — narrows to 0513 Phase-1 with six deliverables (D1–D6); `depends_on: [0392, 0408]`.
- **ADR-0349** (Proposed, never ratified) — Jenkins (LTS) + ArgoCD substrate; Jenkins augments GitHub Actions.
- **ADR-0359** (Superseded by 0511; body still says "Proposed" — drift) — Jenkins completely replaces GitHub Actions.
- **ADR-0361** (Proposed) — executes 0359/0349; a license-vetted supply-chain tool stack.
- **ADR-0124** (Accepted) — own webhook-driven merge-queue (file-overlap clustering).
- **ADR-0408** (Proposed) — Buck2-driven RBE + `cquery rdeps` affected-targets + cache-backed image builds. This is the **build substrate** the CI drives — a *separate* bounded context, **not** CI orchestration.

### 2. The enforcement is a façade (the cost of the drift)
The required merge context `oya-ci-required` **has no live producer**. Both `infra/branch-protection/dev.json` and `.github/branch-protection.yaml` list `required_status_checks: [oya-ci-required]` *and self-disclaim it* as a P0.0 target ("not a live-enforcement claim until the producer posts on the candidate SHA"). Live GitHub `dev` protection actually requires `[cargo-fmt, cargo-check, cargo-clippy, cargo-nextest, oya-pr-review]`, and `oya-pr-review`'s producer returns HTTP 501. **Net: 0 gates block a merge today** (`phase0-ci-enforcement-baseline.json` verdict: `P0.0_RED_blocked_until_cloud_ci_required_context_is_live`). A scattered, contradictory CI canon with no real gate is exactly how the drift accumulated (D-DOCTRINE: drift is a process/enforcement failure).

### 3. The founder ruling (T1 / D-CICD)
> *"bespoke prow + argo rollout + argo workflow + tekton + argo cd and other best-in-class cloud-native CI/CD pipeline. adopt the patterns, reimplement in rust in a cloud-native way. do what Go does, in a cloud-native manner, but in Rust."*

Not "Prow only" (the 0513 framing) and not "Argo wholesale" (the 0511 framing): **adopt the patterns of the whole best-in-class cloud-native CI/CD ecosystem and reimplement them in Rust** — never vendor/run the Go binaries. One canonical decision, owned as a tenant-facing cloud product.

## Decision

Build **`oya-ci` + `oya-cd`**: a bespoke, Rust-native, cloud-native (kube-rs on Talos) CI/CD product that reimplements the *patterns* of Prow + Tekton + Argo Workflows + Argo CD + Argo Rollouts in Rust, unified by two nouns and delivered as tenant-facing dogfood products under `cloud/`.

### D1. The two nouns (the unification primitive)
1. **`Run`** — one typed, content-addressed object that is simultaneously a Prow `ProwJob` × a Tekton `TaskRun`/`PipelineRun` × an Argo `Workflow`. Every producer emits it; every consumer reconciles it. **MVP state = a single Postgres table behind a `RunStore` port** — *not* etcd CRDs (the Argo substrate we refuse), *not* a sharded store until write-rate is measured.
2. **The Buck2 build-graph + CAS (REAPI v2)** — the **brain/moat**, against which `affected-by`, `conflicts(a,b)`, `cache-key`, and `provenance-subject` are one family of query. This depends on ADR-0408/0392 (a separate, authoritative build-substrate decision); oya-ci *drives* the graph, it does not own it.

### D2. Four faces + a CD face + a brain — each adopts a pattern, reimplemented in Rust
**Principle: adopt the PATTERN, reimplement in Rust, never run the Go binary. Substrate that is not the moat (REAPI/CAS, event-bus, kube-scheduler, cosign wire, GitOps CD) is REUSE-behind-port.**

| Face | Adopts the pattern of | Rust reimplementation (what oya OWNS) | Substrate REUSED behind a port | Ownership |
|---|---|---|---|---|
| **A — Ingress + trustless gate + merge queue** | **Prow** `hook` + Tide (+ Uber SubmitQueue) | Forgejo-native webhook → CloudEvent → one `Run` (HMAC fail-closed); **the signature IS the check** (producer ≠ verifier ≠ approver, ADR-0367); serial Tide-invariant merge-queue (batches > singletons, retest-base, abort-on-HEAD-move); graph-exact `conflicts(a,b)` for disjoint concurrent landing | `ForgeAdapter` (no single-token/external-search SPOF) | **OWN-now** (the gate is the keystone); speculation-tree OWN-when-proven (Phase-3, queue-p50 gate) |
| **B — Typed-Task executor** | **Tekton** typed `Task`/`Pipeline`/Step | Rust-typed Task IR; a Step = a REAPI action (typed in Rust, not YAML); `finally`/`when`/`matrix` + dynamic fan-out | `TaskRuntime` | **OWN-now** (single-Task) → combinators OWN-when-proven |
| **C — DAG engine / event correlation** | **Argo Workflows + Argo Events** | DAG engine, brain-driven (graph-affected) selection, CloudEvents correlation (AND/OR), **CAS-backed memoization**; **Argo's etcd-CRD substrate is DROPPED** (keep the ideas, discard the substrate — the ADR-0511 resolution) | `EventBus` (NATS JetStream / Kafka), `Scheduler` (K8s as one impl) | OWN-when-proven (behind ports) |
| **D — Out-of-band provenance signer** | **Tekton Chains** / SLSA | Out-of-band signer attests each `Run` → `slsa/v1`, cosign-signs, writes an immutable ledger; **hermeticity-via-cache → SLSA L3** | `Attestor`/`SigningBackend` (cosign / Fulcio / Rekor) | gate-signature OWN-now → full signer/ledger OWN-when-proven |
| **CD (oya-cd) — GitOps + progressive delivery** | **Argo CD** (declarative GitOps) + **Argo Rollouts** (canary / blue-green / analysis) | `DeliveryPlane` port: oya-cd owns the signed-provenance handoff + cosign-verify-on-sync + audit-chain-emit contract; progressive delivery (canary/blue-green/analysis, Kayenta-class); integration-plane ≠ delivery-plane (kept separate, pluggable) | **Argo CD / Argo Rollouts = REUSE-behind-`DeliveryPlane`** in MVP (reimplemented in Rust only when proven — own-when-proven ratchet) | REUSE-behind-port → OWN-when-proven |
| **Brain (layer d, under all faces)** | Bazel/Buck2 RE + hyperscaler graph | Buck2 graph as a queryable object; REAPI v2 + CAS + ActionCache; `cquery rdeps` graph-sound affected selection; later predictive test-selection / land-ranker / flake-aware culprit-finding | REAPI v2 (NativeLink → owned scheduler when CAS-hit > 60% sustained) | graph+CAS OWN-now (via ADR-0408) → ranker OWN-when-proven |

> **Note on the CD face (per founder T1 "argo rollout + argo cd … reimplement in rust"):** the end-state owns the GitOps + progressive-delivery planes in Rust; the **ratchet** (own-when-proven, ADR-0019/0482) makes Argo-CD/Rollouts a REUSE-behind-`DeliveryPlane` bridge at MVP and reimplements them in Rust once the owned plane is proven — so the reimplement-in-Rust commitment holds without a from-scratch CD build blocking the firewall. *(If the founder wants CD reimplemented in Rust from day-0 rather than ratcheted, flag — this is the one place this ADR chose ratchet over immediate reimplementation.)*

### D3. Build engine (depends on ADR-0408; not owned here)
oya-ci **owns the `BuildSpec`/`BuildStrategy` contract, not the engine**: vendored behind the port — **BuildKit per-pool** (LLB + remote cache + native SPDX SBOM/SLSA `mode=max`) + **Buildah hardened ~7-cap Job-per-build** (hostile-tenant tier). **Scratch/distroless default; SBOM (SPDX) + SLSA provenance HARD-required day-0; Buildpacks opt-in only; Kaniko archived.** Owned Rust build engine = OWN-when-proven (Phase-5), the convergence point with ADR-0408's buck2-native OCI. **ADR-0408 (Buck2 build) and ADR-0392 (build-graph) remain authoritative and separate** — this ADR `depends_on` them.

### D4. The `oya-ci-required` producer contract (the firewall keystone)
The single required merge context is **`oya-ci-required`** (alias `cloud-ci-required`), produced by a **deployed `oya-ci-controller`** (Face A reconciler, kube-rs) — **a service, never config on the branch it gates** (kills the self-deadlock). Contract:
- **Trust model (trustless / trunk-sourced):** gate definitions come from **trusted trunk/controller state**; the candidate PR tree is **data-under-test only**; a PR cannot weaken its own gate (editing the gate script / Job spec / branch-protection mapping / runner isolation in the candidate has no effect); **the signature is the check** (producer ≠ verifier ≠ approver); **no `oya` CLI is merge authority** (`oya verify`/`oya gate` are local migration evidence only).
- **What it posts:** the context on the **candidate PR-head SHA** (40-hex), as a `Phase0CiEnforcementResult` bundle conforming to `/specs/phase0-ci-enforcement-result-schema.json` (`candidate_sha`, `required_context`, `producer{context,kind,trusted_control_state,gate_definition_source,candidate_bytes_policy}`, `fixture_results[]`, `observed_verdict`, `provenance`, `claim_boundary{p0_0_green,phase0_complete}`).
- **Override / kill-switch:** any override requires a controller-enforced packet (TTL + reviewer ack + audit-chain event + named owner + blast-radius + revert/fix follow-up).
- **Tenant isolation:** even at tenant-zero, no leak across `tenant_id` on the 11 surfaces (identity, secrets, runners, workspaces, caches, artifacts, logs/evidence, release ledgers, deploy targets, status callbacks, audit events).

### D5. Homes — tenant-facing dogfood products under `cloud/` (D-PURESPLIT / D-LAYER / backlog #19)
- **`cloud/cloud-scm`** — the bespoke VCS destination (Forgejo transitory → bespoke; the `ForgeAdapter` seam).
- **`cloud/cloud-ci`** — Faces A/B/C + the brain; owns the `oya-ci-required` producer.
- **`cloud/cloud-cd`** — Face D + the CD face (`DeliveryPlane`).
Each lives under `cloud/` only (never a 3rd `microservices/` tree), exactly once, flat-colocated; **no `oya/`→`cloud/` internal dependency** (dogfood purity — oya products consume cloud as a tenant, not via code-coupling). The `cloud/` homing amends ADR-0131/0512 (executed in the A-STRUCT lane).

### D6. Consolidation / supersession map
| ADR | Fate | Edge written | Retained content |
|---|---|---|---|
| **0513** bespoke-Rust Prow | RESHAPED → ratified by 0515 | `0513.superseded_by:[0515]`, status→Superseded | the bespoke instinct (the seed); "clone Prow's 8 components" framing dropped |
| **0511** Argo-WF orchestrator | RESOLVED by rejecting both poles | `0511.superseded_by:[0515]`, status→Superseded | Argo DAG/event **ideas** → Face C behind ports; etcd-CRD substrate dropped |
| **0514** target-arch / hyperscaler remediation | folded as substrate | `0514.superseded_by:[0515]`, status→Superseded | D1–D6 deliverables re-stated as 0515 Phase-1; non-orchestration items (hermetic toolchain, buckify idempotence) → Phase-1 work-items |
| **0349** Jenkins+ArgoCD substrate | SUPERSEDED | `0349.superseded_by:[0515]` | none as authority; bridge operative-but-unratified until cutover; **resolve `byp_adr_0349` bypass record** |
| **0359** Jenkins replaces GHA | re-pointed (already superseded) | `0359.superseded_by:[0511]` → **`[0515]`**; fix body status drift | the anti-GHA-SPOF verdict (remove metered third-party CI) |
| **0361** Jenkins-native execution | SUPERSEDED | `0361.superseded_by:[0515]` | the license-vetted supply-chain tool stack (cargo-deny / Opengrep / gitleaks / Syft / Trivy / osv / cosign / in-toto-SLSA / Kyverno) → layer-e gate steps |
| **0124** own merge-queue (webhook) | superseded-in-mechanism | `0124.superseded_by:[0515]` | merge-queue intent + the 20-row blocker taxonomy → Tide; file-overlap clustering → graph-exact `conflicts(a,b)` |
| **0408** Buck2-driven build/CI | **NOT superseded — depends_on** | `0515.depends_on:[0408]`; amend 0408 refs to cite 0515 for orchestration | Buck2 build substrate stays authoritative (separate bounded context) |

**The 0511↔0513 whiplash is closed by collapsing both into 0515** (not a single cross-edge between two superseded docs): 0515's body records "ADR-0511 and ADR-0513 were contradictory and unlinked; this ADR supersedes both poles — keeping Argo's DAG/event ideas (0511) on the bespoke-Rust controller (0513)."

**Phased-relation ADRs (related, NOT collapsed):** 0367 trustless verification (Phase-1 keystone) · 0369 stacked-trunk speculative merge-train (Phase-3, queue-depth-gated) · 0366 agentic self-enforcing/self-repair (Phase-3→5, the raison d'être, last) · 0392 build-graph (authoritative) · 0181 cosign promotion-ladder (re-homes on the oya-cd signer) · 0374 webhook gateway (the `hook`) · 0116 reviewer-approve (Tide/plugins) · 0111 projected-state merge semantics (fold into Tide).

## Consequences

**Positive:** one CI/CD authority (no contradictory cluster); the `oya-ci-required` producer makes enforcement REAL (the firewall keystone) so Phase-1 amendments are gate-verified and cannot re-drift; pattern-adoption-in-Rust keeps the moat (the build-graph brain) owned while reusing commodity substrate behind ports; dogfood-as-product (cloud-ci/cd built as tenant-zero of the sold product). **Negative/cost:** owning a CI/CD platform (reinvention + edge-case risk, mitigated by adopting proven patterns + lifting Prow's plank state-machine semantics, Argo's DAG ideas, Tekton's typed-task model); a multi-phase build; the CD face starts as REUSE-behind-port (Argo-CD/Rollouts) rather than day-0 Rust. **Neutral:** the Buck2 build substrate (0408/0392) is unchanged (depended-on, not absorbed); the gate *logic* and branch model are unchanged at cutover.

## Phasing (build-first-cutover-later)
- **Phase 0 (the false-green firewall):** this ADR (accepted) + the live `oya-ci-required` producer (trunk-sourced, posts on candidate SHA, blocks a known-bad PR) + the 4 keystone gates over one generated accounting-registry (RED/GREEN-proven). The Jenkins/Argo bridge stays operative-but-unratified.
- **Phase 1 (core, gate-verified):** controller spawns a Job per PR (Face A), typed-Task executor (Face B minimal), Tide minimal admission; cut over and **delete the Jenkins gate path**; re-state ADR-0514 D1–D6.
- **Phase 2:** Tide scale (batch / speculative-retest / auto-rebase), DAG engine (Face C), provenance signer (Face D), oya-cd `DeliveryPlane`.
- **Phase 3–5:** speculation tree (0369), agentic self-repair (0366), owned build engine / owned CD plane / sharded `RunStore` (each own-when-proven, gated).

## Verification
This ADR's enforcement is proven, not asserted (D-DOCTRINE robustness bar — every gate has a known-bad it fails + a known-good it passes + proof it runs in CI and BLOCKS):
- **GREEN:** `tc-0.0-good-cloud-ci-required-and-isolated`.
- **RED (each MUST fail):** `tc-0.0.1-bad-buck2-affected-only-producer`, `-bad-legacy-oya-cli-authority`, `-bad-missing-required-context`, `-bad-oya-gate-run-all-required-producer`, `-bad-oya-verify-affected-producer`, `-bad-required-context-present-not-required`, `tc-0.0.1a-bad-candidate-mutable-producer`, `-bad-candidate-deletes-trusted-target`, `tc-0.0.2-bad-override-without-ttl-audit`, `tc-0.0.3-bad-cross-tenant-shared-cache` (`/specs/phase0-ci-enforcement-baseline.json` fixture set).
- **Proof-it-BLOCKS:** after the producer posts on candidate SHAs, **apply/sync the live GitHub ruleset** so `oya-ci-required` is genuinely required (closing the dev.json/yaml → live-GitHub drift), then re-run T0.0. Only then may `claim_boundary.p0_0_green` flip true.
- **Claim ceiling (until proven):** every authority surface uses gap-packet language only — no "Phase 0 complete" / "P0.0 green" / "mechanically enforced" / "production-ready" / "tenant-facing live service" until the live receipts exist on real SHAs.

---
*Draft for founder review (door:one-way). Authority: D-CICD / D3 / D-LAYER / D-SCOPE-UNIFY / D-PURESPLIT / D-SEQUENCE / D-DOCTRINE. Design SSOT: `bespoke-ci-design-2026-06-06/40-PRODUCT-SPEC.md`. On sign-off → write to `source/docs/adr-archive/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md` + write the reciprocal `superseded_by` edges on 0124/0349/0359/0361/0511/0513/0514 + amend 0408 refs, as the A-CI Phase-1 lane (gate-verified).*
