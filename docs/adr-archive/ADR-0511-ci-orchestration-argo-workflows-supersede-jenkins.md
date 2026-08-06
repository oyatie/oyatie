---
id: ADR-0511
title: "CI orchestration = Argo Workflows (k8s-native); cloud-ci transitory; supersede Proposed ADR-0359 (superseded by ADR-0515)"
status: Superseded
authority: founder
deciders: founder, council-architecture
date: 2026-05-29
owner: council-architecture
planning_impact: true
door: two-way
supersedes: [ADR-0359]
superseded_by: [ADR-0515]
amends: []
related: [ADR-0359, ADR-0358, ADR-0349, ADR-0361, ADR-0363, ADR-0366, ADR-0367, ADR-0369, ADR-0111, ADR-0181, ADR-0387, ADR-0510]
related_specs: [/specs/ci-farm-substrate-canonical.json, /specs/hyperscaler-architecture-invariants.json]
numbering_note: "decisions.json records next_adr=ADR-0392, but the index is stale: origin/dev carries ADRs through ADR-0509 and ADR-0392/ADR-0408 are reserved by the in-flight Buck2-reversal branch. This ADR takes ADR-0511 (the number immediately after its sibling ADR-0510). decisions.json next_adr must be re-derived from the on-disk corpus."
session_context:
  authored: 2026-05-29
  basis: "Founder decision 2026-05-29: the CI/CD destination stack = Buck2 (build/RBE, ADR-0392/0408 in-flight) + Argo Workflows (k8s-native CI orchestration, REPLACES transitory cloud-ci) + ArgoCD/Argo Rollouts (CD). ADR-0359 (cloud-ci-sole-CI) is only status=Proposed (never Accepted) — supersede it. The oya gate engine stays the bespoke governance overlay Argo Workflows invokes; the cloud-scm Commit Status API remains the gate-result sink until the SCM cutover (ADR-0510). This composes with the Buck2 reversal PR — it does not re-decide it."
purpose: Name Argo Workflows as the destination CI orchestrator (CNCF, k8s-native, self-hostable — passes the hyperscaler-lens), supersede the Proposed cloud-ci-sole-CI ADR-0359, and frame cloud-ci as the transitory bootstrap orchestrator. Keep the oya gate engine as the bespoke governance overlay and the cloud-scm Commit Status API as the gate-result sink. Compose around the in-flight Buck2 reversal (ADR-0392/0408) without re-deciding it.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0511: CI orchestration = Argo Workflows (k8s-native); cloud-ci transitory; supersede Proposed ADR-0359 (superseded by ADR-0515)

## Status

Superseded by ADR-0515 — 2026-06-06: the Argo Workflows DAG/event IDEAS are adopted (Face C, reimplemented in Rust behind ports); the etcd-CRD substrate and 'adopt Argo wholesale' are rejected.

**Supersedes ADR-0359** ("cloud-ci completely replaces GitHub Actions as the CI orchestrator"), which was only `status: Proposed` (never Accepted). ADR-0359's correct half — *remove the GitHub-Actions-budget single point of failure; one self-hostable CI surface covering every deployment context* — is **retained**. Its incorrect half — *cloud-ci as the **sole, destination** orchestrator* — is replaced: cloud-ci is the **transitory bootstrap**; **Argo Workflows is the destination** k8s-native CI orchestrator.

This ADR **composes around** the in-flight Bazel→Buck2 reversal (ADR-0392/0408, branch `feat/adr-0392-0408-buck2-reversal-2026-05-29`) and **does not re-decide it**. It also reconciles with ADR-0358 (whose §2 Bazel `rules_rust` choice is itself being superseded by the Buck2 reversal).

## Context

ADR-0349 made cloud-ci + ArgoCD the self-hostable CI/CD substrate. ADR-0359 (Proposed) then pushed cloud-ci to be the *sole* CI orchestrator, removing GitHub Actions after PR #180's entire 37-job matrix was blocked by a GitHub Actions budget cap — a metered third-party single point of failure. ADR-0361 specced the cloud-ci-native execution; ADR-0387/0374 built the cloud-scm→cloud-ci→commit-status webhook bridge. ADR-0363 confirmed cloud-ci as the external-CI that posts cloud-scm Commit Statuses.

cloud-ci **works today** and is genuine self-hostable substrate — but it is not the cloud/k8s-native destination:

- **Not k8s-native by design.** The legacy cloud-ci controller+agent model is bolted onto Kubernetes via a k8s plugin; pipelines run on a stateful controller. The platform substrate is Talos/k8s + ArgoCD GitOps (ADR-0370/0375/0378); a CI orchestrator that is itself a k8s-native, declarative, GitOps-managed CRD workload is the architecturally-coherent destination.
- **Agentic-development-optimized feedback.** The N-lane parallel agent swarm (ADR-0391, parallel-swarm model) needs fast affected-gating feedback with per-lane isolation and CAS-shared results. A DAG-native engine that models each gate as a container step with explicit inputs/artifacts fits the affected-target + verdict-cache model (ADR-0366) more directly than stateful controller stages.
- **Pipeline-optimized.** The destination pipeline is Buck2 RBE + remote cache + speculative merge-train (ADR-0111/0369). Argo Workflows composes cleanly with Buck2 container steps, Argo Events (cloud-scm webhook → workflow trigger), and ArgoCD/Argo Rollouts CD — one CNCF-aligned, self-hostable family.

Argo Workflows passes the **hyperscaler-lens**: CNCF Graduated, active upstream, Apache-2.0, fully self-hostable (runs as k8s CRDs on our own Talos clusters), **no managed-service dependency**, with a hyperscaler-internal equivalent (k8s-native DAG CI is the cloud-native analog of Google TAP/Borg-driven and Amazon-internal pipeline orchestration). It does **not** replace the bespoke differentiator: the **oya gate engine** remains the governance overlay; Argo Workflows is the *orchestrator that invokes it*.

## Decision

### 1. Destination CI orchestrator = Argo Workflows; cloud-ci = transitory bootstrap

The CI/CD destination stack is:

- **Build / RBE:** Buck2 + remote build execution (ADR-0392/0408, in-flight — *not re-decided here*).
- **CI orchestration:** **Argo Workflows** (k8s-native DAG engine) + **Argo Events** (cloud-scm webhook → workflow trigger), **replacing** cloud-ci.
- **CD:** **ArgoCD** (GitOps) + **Argo Rollouts** (progressive delivery, ADR-0366 D5/D6).

**cloud-ci is the transitory bootstrap orchestrator** — it stays the working CI surface (ADR-0361/0387) until the Argo Workflows lanes reach parity, then is retired. This mirrors the SCM transitory pattern (ADR-0510: cloud-scm transitory → bespoke VCS): keep what works, name the destination, migrate when the replacement is green.

### 2. ADR-0359 is superseded; its anti-GitHub-Actions verdict is retained

ADR-0359's removal of GitHub Actions as a metered third-party SPOF **stands** — the destination remains fully self-hosted, covering air-gap/on-prem/colo/oyatie-as-cloud contexts (ADR-0215/0164). What changes: the self-hosted orchestrator destination is **Argo Workflows on our own k8s**, not cloud-ci-sole. No regression to GitHub Actions is implied or permitted.

### 3. The oya gate engine stays the bespoke governance overlay Argo Workflows invokes

Argo Workflows orchestrates; it does **not** absorb governance. Each workflow step invokes the bespoke `oya gate` checks (the ~20 oyatie-specific governance gates with no off-the-shelf equivalent, per ADR-0363 §4) and standard tools (cargo/nextest, cargo-deny, Trivy/cosign/Syft) as container steps. The orchestrator is OSS-adopted; the **differentiator (governance-as-code / AI-slop-defense) stays bespoke** — exactly the bespoke-over-OSS split ADR-0363 drew for cloud-ci, now carried to Argo Workflows.

### 4. cloud-scm Commit Status remains the gate-result sink (until the SCM cutover)

Argo Workflows posts per-context and the single trustless-gateway signed status (ADR-0367) to **the cloud-scm Commit-Status API**, which gates merges via required status checks (ADR-0363). This is unchanged from the cloud-ci path — only the *poster* changes (Argo Workflows replaces cloud-ci as the status producer). At the SCM cutover (ADR-0510 §3) the sink moves to the bespoke VCS status surface; the trust model (ADR-0367) is host-independent.

### 5. Optimization principles (binding for the lane design)

- **Cloud/k8s-native:** orchestrator is a declarative CRD workload, GitOps-managed by ArgoCD — no snowflake controller.
- **Agentic-development-optimized:** fast affected-gating presubmit feedback for the N-lane swarm (ADR-0391); per-lane isolation; verdict-cache + CAS result sharing across lanes/agents (ADR-0366).
- **Pipeline-optimized:** Buck2 RBE + remote cache + the speculative merge-train (ADR-0111/0369) compose as workflow steps; affected-target selection drives the DAG.

### 6. Reconciliations (composed, not re-decided)

- **ADR-0358 (Bazel→Buck2 reversal in-flight):** ADR-0358 §2 chose Bazel `rules_rust`; the in-flight ADR-0392/0408 reverses that to Buck2. This ADR's build layer **defers to that reversal** — Argo Workflows invokes whatever the build decision lands as. No re-decision here.
- **ADR-0111 (merge-queue):** the speculative projected-state train is a workflow the orchestrator drives; deferred behind the numeric concurrency trigger (ADR-0363 §3 / ADR-0369 D4). Unchanged.
- **ADR-0181 (image-promotion ladder):** the cosign-signed dev→staging→prod promotion re-homes onto Argo Workflows promote steps (off any GitHub Actions path); per-tier Fulcio-OIDC verifier IaC + Kyverno tier admission unchanged.
- **ADR-0366 (progressive delivery):** Argo Rollouts AnalysisTemplate (SLO burn-rate, auto-rollback, error-budget promotion-freeze, DORA emission) is the CD half of the same Argo family.

## Rejected alternatives

- **Keep cloud-ci as the sole/destination orchestrator (ratify ADR-0359 as-is)** — rejected: not k8s-native (stateful controller bolted onto k8s), weaker fit for the affected-gating + CAS-sharing + Buck2-RBE destination pipeline and the GitOps/Talos substrate. cloud-ci is kept as the transitory bootstrap, not ratified as destination.
- **GitHub Actions / any metered hosted CI** — rejected (ADR-0359's standing verdict): third-party budget SPOF (PR #180), no air-gap/on-prem coverage.
- **Tekton instead of Argo Workflows** — rejected for now: Argo Workflows + Argo Events + ArgoCD + Argo Rollouts is one coherent, already-partially-adopted CNCF family (ArgoCD is the CD substrate); a second pipeline ecosystem adds surface for no benefit. (Revisit only if a concrete Argo limitation surfaces.)
- **Bespoke CI controller now** — rejected: Argo Workflows is OSS-adoptable, self-hostable, and passes the hyperscaler-lens; the bespoke differentiator is the gate engine, which is already bespoke and *invoked by* the orchestrator. A bespoke controller is a future option behind the same evidence bar, not this decision.

## Consequences

### Positive
- CI orchestration becomes k8s-native and GitOps-managed, coherent with the Talos/ArgoCD substrate (ADR-0370/0375/0378) and one CNCF family across CI+CD.
- Removes the cloud-ci-as-destination lock-in while retaining ADR-0359's correct anti-GitHub-Actions SPOF removal.
- The bespoke governance differentiator (oya gate) is preserved and sharpened — Argo Workflows is plumbing, gates are the value.
- Composes cleanly with the in-flight Buck2 reversal and the trustless-gateway/merge-train/promotion-ladder work without re-deciding any of them.

### Negative / risk
- A migration period runs two orchestrators (cloud-ci transitory + Argo Workflows building parity); mitigated by the local `oya verify` mirror as the gate of record during transition (ADR-0346) and by retiring cloud-ci lanes only when the Argo equivalent is green.
- Argo Workflows + Argo Events operational footprint (CRDs, controller, RBAC, OpenBao-sourced credentials) must be GitOps-managed; this is in-family with the existing ArgoCD operation.

### Neutral
- Docs-only today; no build change. cloud-scm Commit-Status gating, ADR-0367 trust model, ADR-0111 merge-train deferral, and ADR-0181 promotion ladder are unchanged in *what* they do — only the orchestrator that drives them changes.

## Verification
- Frontmatter `supersedes: [ADR-0359]` was set. **Superseded note (2026-06-06):** this ADR is itself superseded by ADR-0515; the live supersession edge is ADR-0359 `superseded_by: [ADR-0515]` (the chain skips the now-superseded 0511). Historical content retained for lineage.
- `oya gate validate aspirational-enforcement` — no binding claim asserts Argo Workflows is the live orchestrator today; cloud-ci remains the transitory surface until parity.
- `oya doc adr-index` regenerates the machine-readable mirror; `numbering_note` records the stale `next_adr`.
- No GitHub Actions CI definition is reintroduced (ADR-0359 standing verdict preserved).

## References
- ADR-0359 — cloud-ci replaces GitHub Actions (Proposed; **superseded here**; anti-GHA-SPOF verdict retained).
- ADR-0358 — ideal production roadmap; §2 Bazel choice being superseded by the Buck2 reversal (composed-around).
- ADR-0392 / ADR-0408 — Bazel→Buck2 reversal (in-flight; build/RBE layer; not re-decided here).
- ADR-0349 / 0361 — cloud-ci + ArgoCD substrate; cloud-ci-native execution (transitory bootstrap).
- ADR-0387 / 0374 — cloud-scm→cloud-ci→commit-status webhook bridge (the poster Argo Workflows replaces).
- ADR-0363 — git + cloud-ci + cloud-scm; oya = governance-gate engine; cloud-scm Commit-Status sink.
- ADR-0366 — agentic high-throughput pipeline; Argo Rollouts AnalysisTemplate / DORA (CD half).
- ADR-0367 / 0369 / 0111 — trustless gateway / stacked-trunk / speculative merge-train (orchestrated, unchanged).
- ADR-0181 — cosign-signed image-promotion ladder (re-homed onto Argo Workflows promote steps).
- ADR-0510 — SCM bespoke-destination / cutover-trigger (sibling; same transitory→destination pattern; status sink moves at SCM cutover).
- Argo Workflows: CNCF Graduated, Apache-2.0, k8s-native DAG CI; Argo Events (webhook triggers); self-hostable on Talos — passes the hyperscaler-lens.
- Founder decision 2026-05-29 + scm-cicd-overhaul-campaign reconciliation_note (session_context above).
