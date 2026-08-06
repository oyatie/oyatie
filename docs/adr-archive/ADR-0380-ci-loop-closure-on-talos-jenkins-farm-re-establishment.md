---
id: ADR-0380
status: Superseded
planning_impact: true
deciders: founder, ops-platform, council-architecture
date: 2026-05-28
owner: ops-platform
supersedes: []
superseded_by: [ADR-700]
related: [ADR-0374, ADR-0378, ADR-0379, ADR-0363, ADR-0349, ADR-0359, ADR-0361, ADR-0148, ADR-0360, ADR-0111]
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
    description: "Re-establish the Jenkins CI farm on the Talos substrate (ADR-0378) by installing the gating plugins (generic-webhook-trigger + build-token-root + http_request + git) into the Jenkins managed by infra/talos/local/bring-up.sh (helm upgrade with infra/ci/jenkins/values-local.yaml installPlugins list). The base Talos Jenkins is currently configuration-as-code + workflow-job only; the ADR-0359 plugin set lived on the now-retired colima farm and must be re-installed. Amendment (2026-05-28): the gitea plugin is intentionally NOT installed — GitHub-canonical brand correctness + the gateway already does webhook discovery."
    exit_criteria: "Jenkins on Talos has generic-webhook-trigger + build-token-root + http_request + git installed and visible in /pluginManager/api; helm upgrade completes; Jenkins restart leaves the existing CasC oya-ci-farm cloud config valid (no boot failure)."
    verified_by: "kubectl -n oya-ci-jenkins exec oya-jenkins-0 -c jenkins -- sh -c 'curl -sf -u admin:$PASS http://localhost:8080/pluginManager/api/json?depth=1 | grep -c generic-webhook-trigger'"
  - id: ADR-0380-D2
    description: "Redesign the Jenkins agent pod templates for Talos: drop the SeaweedFS sccache substrate (retired with colima) and the hostPath /Users/jasonlee/Developer/source mount (a Talos VM cannot see the macOS host filesystem). Replace with a self-contained git-clone-on-demand agent: a rust:1.96.0-bookworm container that clones the repo (via the gateway-build-git Secret's gh token, ESO-projected) and runs `oya gate run-all` against the cloned tree. Caching is honest-deferred (no sccache) until Oyatie's own SeaweedFS-on-Talos object-store substrate (per ADR-0349; we ship + run an S3-API-compatible store, NOT a dependency on AWS S3) is restored on the Talos cluster."
    exit_criteria: "infra/ci/jenkins/values-local.yaml's agent pod templates do NOT reference seaweedfs-s3.oya-ci-jenkins.svc nor hostPath /Users/jasonlee; the rust-ci template clones from git via a Secret-bound token; cargo runs in /workspace (cloned, not bind-mounted) and succeeds."
    verified_by: "a manually-launched rust-ci agent pod clones the repo + runs `oya gate validate fmt` (smoke) end-to-end without sccache or hostPath."
  - id: ADR-0380-D3
    description: "Create the gated Jenkins pipeline job (Generic Webhook Trigger token-authed; dispatched by the ci-webhook-gateway, ADR-0374) that runs `oya gate run-all` against the cloned PR ref and posts GitHub commit-status (success/failure) via the github-ci-token credential. Job authored via Jenkins JCasC (configScripts.oya-ci-gate) so the configuration is declarative and reproducible; oyaCiLane shared library is OPTIONAL (deferred — inline pipeline acceptable for MVP)."
    exit_criteria: "a Jenkins job named oya-ci-gate exists, accepts a webhook trigger with a token, runs `oya gate run-all`, and posts commit-status to GitHub. The 14 reported_status_contexts (infra/ci/jenkins/reported-status-contexts.json) appear on a real PR after webhook fire."
    verified_by: "a real PR against dev produces 14 GitHub commit-status entries (success or failure) within N minutes of the push."
  - id: ADR-0380-D4
    description: "Mint a GitHub CI access token (in-pod `github admin user generate-access-token --username oya-admin --scopes write:repository`) + project it as the `github-ci-token` k8s Secret in oya-ci-jenkins (Jenkins reads it via JCasC credentials block). Register the GitHub webhook (repo oya-admin/oyatie) targeting `http://ci-webhook-gateway.oya-ci.svc.cluster.local:8099/webhook/github`, with the HMAC secret in oya-ci/ci-webhook-gateway-secret (already provisioned) matching GitHub's configured webhook secret."
    exit_criteria: "GitHub webhook delivery test (UI Test Delivery button) returns 200; the gateway pod logs a verified pull_request event; Jenkins receives the dispatch + runs the gated job."
    verified_by: "GitHub webhook UI shows Last delivery: 200; gateway pod logs `signature: verified`; Jenkins build queue receives the trigger."
  - id: ADR-0380-D5
    description: "End-state cutover: once D1–D4 are green and a real PR cycle produces commit-status, enable GitHub auto-merge on dev (or document explicit reviewer-merge), and retire the temporary admin-merge seam (oya-dev-branch-protection-merge memory). The 'gate every merge on verified LOCAL green' rule is replaced by 'gate every merge on the gated CI run + commit-status'."
    exit_criteria: "dev merges happen on green CI without `--admin` override on at least one full PR cycle; the memory oya-dev-branch-protection-merge is updated to reflect the retired seam."
    verified_by: "a PR is merged into dev on green GitHub commit-status without admin override; the lax-merge memory is updated."
  - id: ADR-0380-D6
    description: "Maximum-parallelism enablement (follow-on, gated on D5 cutover landing first): (a) restore SeaweedFS-on-Talos — Oyatie's own object store per ADR-0349 — for sccache + buildkit cache + artifact storage; (b) switch the gated pipeline from `oya gate run-all` to `oya verify --affected` per ADR-0360 O1 so per-PR scope shrinks to the affected reverse-dependency closure; (c) enable Jenkinsfile.sharded nextest --partition (ADR-0360 O4) for in-build sharding; (d) grow agent capacity (Talos VM resize or multi-node cluster) so many PR builds co-schedule; (e) enable ADR-0111 merge-queue projected/speculative admission so PRs admit in parallel without serial bottleneck. Brings CI throughput to the hyperscaler-grade bar the ADR-0349 farm claimed on colima, now Talos-native."
    exit_criteria: "On a sustained 20-PR concurrent-build test, the Talos cluster runs >=8 parallel agent pods with cache reuse; per-PR gate completion under N minutes (vs the cold-build pre-D6 baseline); ADR-0111 merge-queue admits PRs in parallel without serial bottleneck. Throughput meets or beats the ADR-0349 farm-throughput claims."
    verified_by: "20-PR sustained-load benchmark post-D5; throughput vs ADR-0349 targets recorded; honest pass/fail."
purpose: >
  Close the CI loop on the Talos substrate (ADR-0378) by re-establishing the
  Jenkins CI farm: install the gating plugins, redesign agent pods for Talos
  (drop SeaweedFS + hostPath, which were colima-farm-specific), create the
  gated Jenkins job, register the GitHub webhook through the ci-webhook-gateway
  (ADR-0374), and retire the temporary admin-merge seam. The CI gateway pod is
  already live (PR #233); this ADR sequences the remaining wiring so that PRs
  against dev are gated by real Jenkins-produced commit-status rather than
  manual local-green + admin-merge.
---

# ADR-0380 — CI-loop closure on Talos: Jenkins farm re-establishment + GitHub gating

## Status
Accepted (2026-05-28), Amended (2026-05-28). Builds on ADR-0374 (CI webhook gateway,
deployed PR #233), ADR-0378 (vfkit + Talos canonical substrate), ADR-0379 (Kubewarden
default), and ADR-0363 (git + Jenkins + GitHub substrate doctrine). Sequences the
remaining work to retire the admin-merge seam.

## Amendment (2026-05-28)
Two corrections to the plugin set + agent-redesign detail; the 5-deliverable structure
and end goal stand.

**(1) Drop the `gitea` Jenkins plugin from D1.** It was a carryover from the ADR-0359
plugin manifest (authored when the original upstream was Gitea, before GitHub became
canonical per ADR-0363). The plugin works against GitHub via API compatibility, but
(a) reintroduces the `gitea` brand into a GitHub-canonical stack, and (b) is
unnecessary: the CI webhook gateway (ADR-0374) is the front door (so multibranch /
webhook-discovery features add nothing), and commit-status posting to GitHub is done
via the `http_request` plugin (or an explicit `curl` pipeline step) against
`POST /repos/{owner}/{repo}/statuses/{sha}` authenticated by the `github-ci-token`
credential.
**Revised D1 plugin set:** `generic-webhook-trigger + build-token-root + http_request + git`.
**Revised D3 status posting:** `http_request` plugin OR `curl` step + GitHub statuses
API + `github-ci-token`. No gitea plugin involvement.

**(2) Sharpen D2 agent redesign.** The concrete changes to `infra/ci/jenkins/values-local.yaml`:
- Collapse the three colima-era templates (`rust-ci` + `rust-build` + `rust-parallel`)
  into ONE `rust-ci` template. The split only existed for sccache-cached throughput
  experiments under ADR-0349 (now superseded for Talos).
- Strip all sccache wiring (`RUSTC_WRAPPER`, `SCCACHE_*`, `AWS_*` envs, the
  `seaweedfs-s3` Secret reference). Remote build cache is honest-deferred to a follow-on
  when an in-cluster S3 lands.
- Remove the `hostPath: /Users/jasonlee/Developer/source` mount (structurally broken on
  Talos — the VM cannot see the macOS host filesystem).
- The new template is `rust:1.96.0-bookworm` + `git` installed, with the
  `oya-ci/gateway-build-git` Secret's gh token projected as `GH_TOKEN` for
  pipeline-step clone. PSA-restricted securityContext unchanged.
- D3's Jenkinsfile FIRST stage clones the PR ref into the workspace
  (`git clone --depth 1 --branch $PR_REF https://oya-admin:$GH_TOKEN@github.com/...`),
  then runs `./bin/oya gate run-all`, then posts the status. No bind-mounted source.
- Stage-2 hardening (deferred): flip the clone source from GitHub to the in-cluster
  GitHub (dogfood-correct, no external creds needed) when GitHub dev becomes the
  upstream mirror.

**(4) Maximum-parallelism is a named follow-on, not this MVP.** D1-D5 deliver
REAL CI gating (better than admin-merge), but this is NOT yet hyperscaler-grade
concurrent throughput. The single-node Talos (~6 vCPU) caps concurrent cargo
agents at ~3-4; without remote build cache (sccache->SeaweedFS-on-Talos, ADR-0349
restoration deferred), every parallel build cold-compiles its full dep tree;
the gated pipeline runs `oya gate run-all` per PR instead of the tight
`oya verify --affected` (ADR-0360 O1); no in-build nextest sharding
(ADR-0360 O4); no merge-queue projected/speculative admission (ADR-0111).
**Do not conflate "CI gates merges" (this MVP) with "CI gates merges at
hyperscaler-scale concurrency" (the D6 follow-on).** All five pieces above are
named in D6 as the path; the fan-out's ceiling lifts when D6 lands, not D5.

**(3) Object-store substrate is Oyatie's own, NOT AWS S3.** Earlier drafts of
this ADR loosely referred to "in-cluster S3" as the deferred build-cache
backend. Correcting the language + reasserting the doctrine: Oyatie ships
**SeaweedFS** (Apache 2 — per ADR-0349) as our cluster-internal object store;
its S3-compatible wire protocol exists for client/tooling interop, NOT as a
dependency on the AWS S3 service. As a cloud provider competing with
hyperscalers, Oyatie does not consume hyperscaler-managed services — we provide
them. Any remaining "S3" mention in this ADR is shorthand for SeaweedFS-on-Talos
(our own substrate, restored as a follow-on to ADR-0349), never AWS S3.

**(5) Two further substrate catches surfaced mid-amendment, deferred to ADR-0381.**
Both fail Oyatie's standing **hyperscaler-grade self-hosted substrate** lens
(every new component must be: (a) actively maintained upstream — no archived
projects; (b) license-clean Apache 2 / MIT / BSD / LGPL — never SSPL / BSL /
RSAL; (c) fully self-hostable — no managed-service dependency; (d) the OSS
substrate a hyperscaler would itself run internally, not a thing that only
exists as their managed offering):

- **Kaniko is archived.** Current `infra/ci-webhook-gateway/kaniko-build.yaml`
  + `infra/registry/registry.k8s.yaml` + `microservices/ci-webhook-gateway/Dockerfile`
  reference Kaniko (Google Container Tools), which Google placed into
  maintenance/archive in 2024 — the GitHub repo is read-only. Fails lens (a).
  Migration target: **BuildKit** (Moby project, Apache 2 — what Docker itself
  uses; daemonless `buildctl`, OCI output, content-addressed cache with
  `registry / s3 / inline` backends so SeaweedFS-on-Talos per amendment (3)
  above is the cache backend, fitting our own-substrate doctrine end-to-end).
  Passes lens (a)-(d).

- **Local Talos topology is single-node (~6 vCPU); needs proper CP / Worker /
  Specialty node pools.** Production-credible cell pattern is: 3-CP etcd
  quorum for control-plane HA; worker pool for tenant workloads (PSA
  restricted); CI specialty pool for cargo build agents (this is what unlocks
  D6's "multiple PR builds co-schedule" exit criterion); storage specialty
  pool for SeaweedFS. Cilium L3/L4 (ADR-0148) enforces cell boundaries;
  ADR-0083 pod-runtime-tier maps to node-pool affinity. The current
  single-node hard-limits D6's parallelism ceiling no matter how the cache /
  affected / sharding / merge-queue pieces are tuned. Multi-node Talos via
  vfkit (ADR-0378) passes the lens — Talos is Apache 2, actively maintained,
  and the multi-pool topology is what GKE / EKS / AKS themselves run.

Both decisions are substrate-canonical; they belong in a dedicated ADR rather
than further-bloating this one. **ADR-0381 (this same commit; Proposed status)
captures "Kaniko→BuildKit migration + multi-node Talos cell topology" formally**,
with the hyperscaler-lens applied explicitly per choice (D1-D4 in that ADR).
ADR-0380 ships D1-D5 (CI-loop closure MVP) + D6 (max-parallelism path) as
originally scoped; these substrate corrections are deferred to ADR-0381's
implementation IPs to keep ADR-0380 focused on the gating MVP.

The rest of the ADR (status posting flow, webhook registration, cutover) is unchanged.

## Context
The CI webhook gateway is **live** on Talos (oya-ci namespace; `/healthz` ok;
listening on `/webhook/github`; HMAC fail-closed). GitHub + Jenkins are also up
on Talos. But the Jenkins on Talos is a **base install** — `configuration-as-code`
+ `workflow-job` only, no generic-webhook-trigger plugin, no `oyaCiLane` shared
library, no gated job, no `github-ci-token` credential. The ADR-0359/R1–R5
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
1. **D1**: Install the gating plugins (generic-webhook-trigger + build-token-root
   + http_request + git) via `helm upgrade` of the bring-up-managed Jenkins
   with `installPlugins` extensions in `values-local.yaml`. Reboot once; CasC
   error-on-conflict means the existing oya-ci-farm cloud config must not be
   re-declared in any new configScript block. (Amendment 2026-05-28: gitea plugin
   intentionally NOT installed — brand-correct + unnecessary given the gateway-
   front-door design; status posting uses http_request + GitHub API directly.)
2. **D2**: Redesign agent pods for Talos — **drop SeaweedFS sccache** (no
   cache substrate until SeaweedFS-on-Talos is restored per ADR-0349 — Oyatie's
   own S3-API-compatible object store, never AWS S3), **drop the hostPath mount**
   (use `git clone` from inside the pod with the existing `gateway-build-git`
   Secret's gh token, projected into the agent via env or volume). Caching is
   honest-deferred to a follow-on ADR; the immediate goal is a correct, slow
   gate, not a fast one.
3. **D3**: Author the gated pipeline job (JCasC `configScripts.oya-ci-gate`)
   that the gateway dispatches via Generic Webhook Trigger, runs
   `oya gate run-all`, and posts GitHub commit-status (the 14 contexts in
   `infra/ci/jenkins/reported-status-contexts.json`). Inline pipeline is
   acceptable; the oyaCiLane shared library is optional/deferred.
4. **D4**: Mint a GitHub CI access token (via the in-pod `github admin
   user generate-access-token` CLI), store as the `github-ci-token` k8s Secret,
   register the GitHub webhook → the gateway URL with the matching HMAC secret
   already in `oya-ci/ci-webhook-gateway-secret`.
5. **D5**: Cutover — a real PR cycle posts commit-status, GitHub auto-merges
   on green, the admin-merge seam is retired.

The **CI gateway** (ADR-0374, PR #233) is the front door and stays. The
gateway's HMAC fail-closed verification is the security boundary; Jenkins
inside the cluster trusts the gateway's verified dispatch.

## Rejected alternatives
- **Skip the gateway, register GitHub webhook directly at Jenkins Generic
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
produces GitHub commit-status entries within minutes and merges on green
without `--admin`.

## References
ADR-0374 (CI webhook gateway), ADR-0378 (Talos canonical substrate), ADR-0379
(Kubewarden default), ADR-0363 (git + Jenkins + GitHub substrate), ADR-0349
(CI farm, on retired colima — superseded for Talos by this ADR), ADR-0359/
0361 (Jenkins CI revamp, plugin set), ADR-0148 (Cilium + Istio Ambient,
the network plane for Jenkins agents). Repo: `infra/ci/jenkins/values-local.yaml`
(plugin list + agent templates), `infra/ci/jenkins/Jenkinsfile.sharded`
(pipeline template), `infra/ci/jenkins/reported-status-contexts.json` (the 14
commit-status contexts to post), `microservices/ci-webhook-gateway/SETUP-
RUNBOOK.md` (gateway provisioning runbook).
