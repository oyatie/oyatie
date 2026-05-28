---
id: ADR-0380
status: Accepted
planning_impact: true
deciders: founder, ops-platform, council-architecture
date: 2026-05-28
owner: ops-platform
supersedes: []
superseded_by: []
related: [ADR-0374, ADR-0378, ADR-0379, ADR-0363, ADR-0349, ADR-0359, ADR-0361, ADR-0148]
related_specs: [/specs/deployment-ops-contract.json]
milestone: M-LOCAL-CI-SUBSTRATE
depends_on: [ADR-0378, ADR-0374]
door: two-way
affected_surfaces:
  crates: []
  microservices: [ci-webhook-gateway]
  specs: []
deliverables:
  - id: ADR-0380-D1
    description: "Re-establish the Jenkins CI farm on the Talos substrate (ADR-0378) by installing the gating plugins (generic-webhook-trigger + gitea + build-token-root) into the Jenkins managed by infra/talos/local/bring-up.sh (helm upgrade with infra/ci/jenkins/values-local.yaml installPlugins list). The base Talos Jenkins is currently configuration-as-code + workflow-job only; the ADR-0359 plugin set lived on the now-retired colima farm and must be re-installed."
    exit_criteria: "Jenkins on Talos has generic-webhook-trigger + gitea + build-token-root + git installed and visible in /pluginManager/api; helm upgrade completes; Jenkins restart leaves the existing CasC oya-ci-farm cloud config valid (no boot failure)."
    verified_by: "kubectl -n oya-ci-jenkins exec oya-jenkins-0 -c jenkins -- sh -c 'curl -sf -u admin:$PASS http://localhost:8080/pluginManager/api/json?depth=1 | grep -c generic-webhook-trigger'"
  - id: ADR-0380-D2
    description: "Redesign the Jenkins agent pod templates for Talos: drop the SeaweedFS sccache substrate (retired with colima) and the hostPath /Users/jasonlee/Developer/source mount (a Talos VM cannot see the macOS host filesystem). Replace with a self-contained git-clone-on-demand agent: a rust:1-bookworm container that clones the repo (via the gateway-build-git Secret's gh token, ESO-projected) and runs `oya gate run-all` against the cloned tree. Caching is honest-deferred (no sccache) until an in-cluster S3 (registry-coupled or future SeaweedFS-on-Talos) is stood up."
    exit_criteria: "infra/ci/jenkins/values-local.yaml's agent pod templates do NOT reference seaweedfs-s3.oya-ci-jenkins.svc nor hostPath /Users/jasonlee; the rust-ci template clones from git via a Secret-bound token; cargo runs in /workspace (cloned, not bind-mounted) and succeeds."
    verified_by: "a manually-launched rust-ci agent pod clones the repo + runs `oya gate validate fmt` (smoke) end-to-end without sccache or hostPath."
  - id: ADR-0380-D3
    description: "Create the gated Jenkins pipeline job (Generic Webhook Trigger token-authed; dispatched by the ci-webhook-gateway, ADR-0374) that runs `oya gate run-all` against the cloned PR ref and posts Forgejo commit-status (success/failure) via the forgejo-ci-token credential. Job authored via Jenkins JCasC (configScripts.oya-ci-gate) so the configuration is declarative and reproducible; oyaCiLane shared library is OPTIONAL (deferred — inline pipeline acceptable for MVP)."
    exit_criteria: "a Jenkins job named oya-ci-gate exists, accepts a webhook trigger with a token, runs `oya gate run-all`, and posts commit-status to Forgejo. The 14 reported_status_contexts (infra/ci/jenkins/reported-status-contexts.json) appear on a real PR after webhook fire."
    verified_by: "a real PR against dev produces 14 Forgejo commit-status entries (success or failure) within N minutes of the push."
  - id: ADR-0380-D4
    description: "Mint a Forgejo CI access token (in-pod `forgejo admin user generate-access-token --username oya-admin --scopes write:repository`) + project it as the `forgejo-ci-token` k8s Secret in oya-ci-jenkins (Jenkins reads it via JCasC credentials block). Register the Forgejo webhook (repo oya-admin/oyatie) targeting `http://ci-webhook-gateway.oya-ci.svc.cluster.local:8099/webhook/forgejo`, with the HMAC secret in oya-ci/ci-webhook-gateway-secret (already provisioned) matching Forgejo's configured webhook secret."
    exit_criteria: "Forgejo webhook delivery test (UI Test Delivery button) returns 200; the gateway pod logs a verified pull_request event; Jenkins receives the dispatch + runs the gated job."
    verified_by: "Forgejo webhook UI shows Last delivery: 200; gateway pod logs `signature: verified`; Jenkins build queue receives the trigger."
  - id: ADR-0380-D5
    description: "End-state cutover: once D1–D4 are green and a real PR cycle produces commit-status, enable Forgejo auto-merge on dev (or document explicit reviewer-merge), and retire the temporary admin-merge seam (oya-dev-branch-protection-merge memory). The 'gate every merge on verified LOCAL green' rule is replaced by 'gate every merge on the gated CI run + commit-status'."
    exit_criteria: "dev merges happen on green CI without `--admin` override on at least one full PR cycle; the memory oya-dev-branch-protection-merge is updated to reflect the retired seam."
    verified_by: "a PR is merged into dev on green Forgejo commit-status without admin override; the lax-merge memory is updated."
purpose: >
  Close the CI loop on the Talos substrate (ADR-0378) by re-establishing the
  Jenkins CI farm: install the gating plugins, redesign agent pods for Talos
  (drop SeaweedFS + hostPath, which were colima-farm-specific), create the
  gated Jenkins job, register the Forgejo webhook through the ci-webhook-gateway
  (ADR-0374), and retire the temporary admin-merge seam. The CI gateway pod is
  already live (PR #233); this ADR sequences the remaining wiring so that PRs
  against dev are gated by real Jenkins-produced commit-status rather than
  manual local-green + admin-merge.
---

# ADR-0380 — CI-loop closure on Talos: Jenkins farm re-establishment + Forgejo gating

## Status
Accepted (2026-05-28). Builds on ADR-0374 (CI webhook gateway, deployed PR #233),
ADR-0378 (vfkit + Talos canonical substrate), ADR-0379 (Kubewarden default), and
ADR-0363 (git + Jenkins + Forgejo substrate doctrine). Sequences the remaining
work to retire the admin-merge seam.

## Context
The CI webhook gateway is **live** on Talos (oya-ci namespace; `/healthz` ok;
listening on `/webhook/forgejo`; HMAC fail-closed). Forgejo + Jenkins are also up
on Talos. But the Jenkins on Talos is a **base install** — `configuration-as-code`
+ `workflow-job` only, no generic-webhook-trigger plugin, no `oyaCiLane` shared
library, no gated job, no `forgejo-ci-token` credential. The ADR-0359/R1–R5
Jenkins CI configuration was provisioned on the **colima farm**, which was
retired (ADR-0378); none of that configuration carried over.

A second discovery surfaced during planning: `infra/ci/jenkins/values-local.yaml`
defines three agent pod templates (rust-ci, rust-build, rust-parallel) that all
reference **SeaweedFS** (`seaweedfs-s3.oya-ci-jenkins.svc.cluster.local:8333` as
the sccache S3 endpoint) and a **hostPath mount** of `/Users/jasonlee/Developer/
source`. Both assumptions are broken on Talos:
- SeaweedFS was deployed on the retired colima k3s; it does not exist on the
  Talos cluster.
- The Talos node is a vfkit VM; it does not see the macOS host filesystem, so
  the `hostPath` mount has no source.

So closing the loop is **not** "register the webhook"; it is a sub-project that
re-establishes the Jenkins farm on Talos with a Talos-appropriate agent design.

## Decision
Sequence the re-establishment into five deliverables (D1–D5):
1. **D1**: Install the gating plugins (generic-webhook-trigger + gitea +
   build-token-root + git) via `helm upgrade` of the bring-up-managed Jenkins
   with `installPlugins` extensions in `values-local.yaml`. Reboot once; CasC
   error-on-conflict means the existing oya-ci-farm cloud config must not be
   re-declared in any new configScript block.
2. **D2**: Redesign agent pods for Talos — **drop SeaweedFS sccache** (no
   cache substrate until an in-cluster S3 lands), **drop the hostPath mount**
   (use `git clone` from inside the pod with the existing `gateway-build-git`
   Secret's gh token, projected into the agent via env or volume). Caching is
   honest-deferred to a follow-on ADR; the immediate goal is a correct, slow
   gate, not a fast one.
3. **D3**: Author the gated pipeline job (JCasC `configScripts.oya-ci-gate`)
   that the gateway dispatches via Generic Webhook Trigger, runs
   `oya gate run-all`, and posts Forgejo commit-status (the 14 contexts in
   `infra/ci/jenkins/reported-status-contexts.json`). Inline pipeline is
   acceptable; the oyaCiLane shared library is optional/deferred.
4. **D4**: Mint a Forgejo CI access token (via the in-pod `forgejo admin
   user generate-access-token` CLI), store as the `forgejo-ci-token` k8s Secret,
   register the Forgejo webhook → the gateway URL with the matching HMAC secret
   already in `oya-ci/ci-webhook-gateway-secret`.
5. **D5**: Cutover — a real PR cycle posts commit-status, Forgejo auto-merges
   on green, the admin-merge seam is retired.

The **CI gateway** (ADR-0374, PR #233) is the front door and stays. The
gateway's HMAC fail-closed verification is the security boundary; Jenkins
inside the cluster trusts the gateway's verified dispatch.

## Rejected alternatives
- **Skip the gateway, register Forgejo webhook directly at Jenkins Generic
  Webhook Trigger URL** — rejected: forgoes HMAC fail-closed verification +
  PR-against-dev filtering. The gateway exists, is live, and is the proper
  front door (ADR-0374).
- **Keep the SeaweedFS sccache agent and stand SeaweedFS up on Talos** —
  rejected for the MVP: SeaweedFS-on-Talos is its own sub-project; the agent
  redesign without sccache produces a correct (if slower) gate, which is enough
  to retire admin-merge. Add SeaweedFS later if/when warmup times matter.
- **Keep the hostPath agent** — rejected: structurally impossible (Talos can't
  see the Mac host filesystem).
- **Defer CI gating and fan out under admin-merge** — rejected (the wave-3
  silent regression bite proves this is unsafe; see oya-dev-branch-protection-
  merge memory).

## Consequences
- Positive: real CI gating; admin-merge seam retired; fan-out lanes (Phase 1+
  enterprise + Phase 2 healthcare + etc.) gain real PR gating; reproducibility
  via JCasC.
- Negative/cost: builds are slow until caching is restored (no sccache); the
  agent redesign + JCasC + plugin install + Jenkins restart is a focused
  multi-step session, not a quick wiring; signed commits (ADR-0039 Ed25519)
  remain a separate hardening lane.
- Neutral: oyaCiLane shared library is deferred; inline Jenkinsfile is the
  MVP path.

## Verification
Per-deliverable `verified_by`. The terminal acceptance is a real PR cycle that
produces Forgejo commit-status entries within minutes and merges on green
without `--admin`.

## References
ADR-0374 (CI webhook gateway), ADR-0378 (Talos canonical substrate), ADR-0379
(Kubewarden default), ADR-0363 (git + Jenkins + Forgejo substrate), ADR-0349
(CI farm, on retired colima — superseded for Talos by this ADR), ADR-0359/
0361 (Jenkins CI revamp, plugin set), ADR-0148 (Cilium + Istio Ambient,
the network plane for Jenkins agents). Repo: `infra/ci/jenkins/values-local.yaml`
(plugin list + agent templates), `infra/ci/jenkins/Jenkinsfile.sharded`
(pipeline template), `infra/ci/jenkins/reported-status-contexts.json` (the 14
commit-status contexts to post), `microservices/ci-webhook-gateway/SETUP-
RUNBOOK.md` (gateway provisioning runbook).
