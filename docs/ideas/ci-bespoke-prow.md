# ci — Bespoke-Rust Prow (GitHub-native CI/CD platform)

## Problem Statement
How might we replace the flaky Jenkins+Groovy CI gate with a robust, introspectable,
deadlock-proof, **pure-Rust, GitHub-native** CI/CD platform that mirrors Prow's proven
component model — and unifies the gate, the merge-queue, reviewer-approval, and
governance-ChatOps into one substrate?

## Recommended Direction
Build **`ci`**: a bespoke-Rust reimplementation of Prow's component shape, K8s-native
(kube-rs), GitHub-native, on Talos. We ADOPT THE SHAPE (Prow's battle-tested decomposition),
not the code (Prow is Go + GitHub/GCS-coupled). It is the canonical replacement for both
Jenkins (gate execution) AND the externally-specced merge-queue — one platform.

Validated as the hyperscaler pattern: Prow is Kubernetes' own CI (CNCF-scale); the
controller-spawns-Job-per-change + reports-status model + trunk-sourced presubmit
(GitHub `pull_request` vs `pull_request_target`) are the documented robust/secure patterns.

## Component map (Prow → ci, and what each SUBSUMES)
| Prow | ci (Rust) | Subsumes / notes |
|---|---|---|
| **hook** (webhook ingest, event + command routing) | `ci-webhook-gateway` ✅ (extend for plugin/command dispatch) | Already bespoke Rust, GitHub-native, rock-solid |
| **plank** (job controller: K8s Job per ProwJob) | `ci-controller` (kube-rs) | The reliable gate executor |
| **crier** (report status/comments to forge) | reporter (reuse gateway's GitHub client) | Terminal-status-always; failure summary |
| **ProwJob + config** (presubmit/postsubmit/periodic/batch) | `OyaCIJob` CRD + config | buck2-affected-gate = one presubmit job type |
| **tide** (merge automation: pool, batch, retest, auto-merge) | `ci-merge` controller | **= the merge-queue (ADR-0111 projected state)** + the Sweep engine's auto-merge + required-context/approval enforcement (ADR-0116 reviewer-APPROVE) |
| **deck** (web UI: jobs, logs, history) | `ci-deck` (Leptos shell, reuse the canonical oya UI stack) | CI visibility for founder + agents |
| **sinker** (GC) | K8s `ttlSecondsAfterFinished` + a GC loop | Cheap, K8s-native |
| **plugins** (ChatOps: /test /retest /lgtm /approve + governance) | `ci-plugins` on the gateway | The governance pipeline / agent ChatOps; reviewer-agent APPROVE |
| pod-utils → **GCS** artifacts | `kubectl logs` + **SeaweedFS-S3** | No GCS coupling (self-host lens) |

## The unification (why "full shape" is the right call)
`ci` collapses several separately-planned things into one substrate: the **CI gate**
(plank+job), the **merge-queue** (tide = ADR-0111), the **Sweep engine's auto-merge** (tide
does it natively, with batching/retest), **reviewer-APPROVE** (tide required-label + a plugin,
ADR-0116), **governance ChatOps** (plugins), and **CI visibility** (deck). One bespoke-Rust
platform instead of Jenkins + a separate merge-queue + bespoke auto-merge glue.

## Phasing (bridge → platform)
- **Phase 0 — Bridge (now, locked):** harden Jenkins (presubmit Jenkinsfile-parse validation
  + `post{always}` terminal status + tight timeout + warm rust-ci image). Stops today's flakiness; buys time.
- **Phase 1 — Core gate (plank+crier+job):** `ci-controller` spawns a K8s Job per PR running
  the TRUNK gate script vs the PR ref; posts a terminal `ci-gate` status + summary. Cut over;
  delete the Jenkins gate path (Jenkinsfile/genericTrigger/cpsScm). **This is the reliability win.**
- **Phase 2 — tide / merge-queue:** pool gated PRs, batch + speculative-retest, auto-merge on
  green + required approvals (subsumes ADR-0111 + Sweep auto-merge + ADR-0116).
- **Phase 3 — job types + deck:** postsubmit/periodic jobs + the web UI.
- **Phase 4 — plugins / ChatOps:** /test, /retest, /approve, governance commands; the reviewer-agent.

## Key Assumptions to Validate
- [ ] kube-rs Job spawn+watch + terminal-status state machine is tractable — Phase-1 spike (lift plank's phase→state logic).
- [ ] tide's pool/batch/retest model maps cleanly onto GitHub PRs + the buck2 affected gate — Phase-2 design.
- [ ] One Rust platform is less total surface than Jenkins + Prow-adapter + bespoke merge glue — track LOC/ops over phases.

## Not Doing (and Why)
- Adopt Prow's Go code — GitHub/GCS-coupled, would replace our working GitHub-native Rust gateway; we adopt the SHAPE.
- Build all phases at once — phase it; Phase 1 (reliable gate) is the urgent reliability win, the rest follows.
- GCS / pod-utils — SeaweedFS-S3 + kubectl-logs (self-host lens).

## Governance
This is a major bespoke platform component → needs a **bespoke ADR** with the full
Prow-feature-parity table (per the bespoke-over-OSS doctrine). It supersedes the
Jenkins-gate substrate (ADR-0380) and the external merge-queue spec it folds in (ADR-0111).

## Open Questions
- `ci` as one multi-binary crate (controller/merge/deck/plugins as modes) vs a workspace of crates? (lean: workspace, crate-per-component, shared core.)
- Controller↔gateway: extend the gateway, or a sibling that watches a CRD the gateway writes? (lean: CRD-driven, the K8s-idiomatic reconcile pattern.)
- Migration of the merge-queue: does Phase 2 retire ADR-0111's separate plan, or implement it?

See [[ci-gate-pipeline-state]], [[affected-gated-migration-engine]] (the Sweep engine becomes a tide client), [[bespoke-over-oss-doctrine]], [[hyperscaler-lens-architectural-filter]].

<!-- gate healthcheck 1e72f139e: verify ci-gate parses+runs+resolves with post{aborted} bridge -->
