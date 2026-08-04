---
id: ADR-0630
title: "Actions Runner Controller as the interim owned-runner substrate, declared in infra/arc/, behind the ADR-0515 D5 owned-CI destination"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-07-29
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0515, ADR-0560, ADR-0556]
amends: []
related: [ADR-0131, ADR-0523, ADR-0535, ADR-0612, ADR-0554]
milestone: W3
---

# ADR-0630: ARC as the interim owned-runner substrate

## Status

**Proposed — 2026-07-29.** Ratifies, and declares, a runner fleet that was brought up
imperatively the same day. Written because the fleet already holds merge authority and had
**no decision record**: `git grep -iE 'autoscalingrunnerset|gha-runner-scale-set' origin/dev`
returned zero hits while three helm revisions existed only as cluster secrets.

## Context

Two facts forced the question at once.

**(1) GitHub-hosted runners stopped working.** Every job on `oya-ci-required` began failing to
*start* with `"The job was not started because recent account payments have failed or your
spending limit needs to be increased"` — an Actions billing wall, reached because the
repository is private (private repos bill Actions minutes; public repos do not). Merge
authority therefore had no working substrate: `oya-ci-required` is the single required context,
and it could not run at all.

**(2) The founder ruled the local Talos cluster PERMANENT and merge-authoritative.**
Verbatim: *"treat this laptop cluster as a permanent part of the cluster"* and *"should have
merge authority."* An earlier objection — that a sleeping laptop becomes merge authority — was
raised twice, considered, and explicitly overruled.

Self-hosted runners consume no Actions minutes, so owned runners resolve (1) as a side effect
of doing what (2) already required. ADR-0515 D5 names owned in-cluster runners as the
destination but scopes the cutover out; this ADR does not change that destination, it starts
the runner half of it earlier than planned because the hosted path is unavailable.

## Decision

### D1 — ARC (`gha-runner-scale-set`) is the interim runner substrate

Adopt `actions/actions-runner-controller` charts, **pinned at 0.14.2**, as the mechanism that
turns the owned cluster into GitHub Actions capacity. It is Apache-2.0, Kubernetes-native
(CRD + controller + reconciliation, no imperative agent lifecycle), and authored by the
platform we currently integrate with — which is precisely the transient-stack profile: adopted
OSS behind an owned destination, not a new permanent dependency.

**It is explicitly INTERIM.** The destination remains the bespoke Rust `cloud/cloud-ci`
product (ADR-0515 D5). ARC's replacement trigger is that product's runner surface becoming
able to execute the canonical gate set; ARC is a runner *transport*, and nothing in the gate
logic may become ARC-shaped.

### D2 — ONE GENERAL-PURPOSE SCALE SET PER ARCHITECTURE, arch-pinned by nodeSelector

Founder requirement: *"we should be agnostic. arm64 / amd64 should both be a supported path"* —
*"on this laptop arm. on another box that joins the cluster that is amd64, we can run amd64."*

So the cluster is heterogeneous and each scale set is pinned:
`nodeSelector: {kubernetes.io/arch: <arch>}`. **Without that pin a scale set schedules
anywhere**, and an "arm64" runner can land on an amd64 node and silently build for the wrong
platform — poisoning that platform's buck2 cache namespace, because action keys include `cpu:`
and `os:`. Adding capacity is joining a node; adding a PLATFORM is one more values file.
`infra/ci/install-buck2.sh` already carries digest-pinned arms for both architectures.

The one-per-architecture rule applies to the general-purpose fleet. A same-architecture sibling
scale set is allowed only when its separate label is itself an isolation boundary for a job-local
service, credential, trust, or resource profile that must not enter general-purpose Pods. Such a
set remains architecture-pinned and may not fork gate logic; workflows select only its capability
label. The dedicated live-PostgreSQL cell in D7 is the first bounded exception.

### D3 — GitHub App identity, scoped to one repository

Authentication is a GitHub App (`Administration: write`, `Metadata: read`), installed on
`jason931225/oyatie` **only** — verified `repo count: 1`. Not a personal access token, which
would have carried `repo`, `workflow`, `gist` and `admin:ssh_signing_key` across every
repository the founder can reach, in a secret readable by anyone with access to a cluster that
is **shared with another project**. Key material lives in the cluster secret and never in git.

Recorded limitation: an App installation token can push to an **existing** ghcr package but
**cannot create one** — measured
`{"code":"DENIED","message":"permission_denied: installation not allowed to Create organization package"}`.
Package creation requires Actions' `GITHUB_TOKEN`. `packages: write` on the App is therefore
necessary but not sufficient, and image publication must run as a workflow.

### D4 — Runner pods carry the CAS client labels

`infra/nativelink`'s NetworkPolicy `nativelink-cas-ingress` admits `:50051` only from pods
labelled `oya.io/nativelink-cas-writer: "true"` and `:50052` only from
`oya.io/nativelink-cas-reader: "true"`, matching the **source** pod in any namespace. Runner
pods therefore carry both. Without them the connection is refused at **L3** and presents as a
**timeout, not a TLS error** — a failure mode that reads as a certificate problem and is not
one. This is the mechanical precondition for the CAS ever being reachable; the buck2-side
wiring is separate and still blocked (see Consequences).

### D5 — Resource bounds derived by arithmetic, on a shared cluster

`requests: cpu 2 / memory 4Gi / ephemeral-storage 20Gi`;
`limits: memory 8Gi / ephemeral-storage 60Gi`; **no CPU limit**.

- `cpu 2` tracks the MEASURED buck2 scaling knee (j=2 87.19s, j=4 67.06s, j=8 71.29s,
  j=18 98.09s) — over-parallelism degrades, so a wider request buys nothing and crowds
  co-tenants.
- `memory 8Gi` is arithmetic, not taste: node allocatable is 28.6Gi, and the 4Gi **request**
  lets all `maxRunners=3` land on ONE node. At a 12Gi limit they could grow to 36Gi and OOM
  it, evicting co-tenants — `nativelink-cas` holds `local-path` PVCs on these same nodes, and
  the console project shares the cluster. 3 × 8Gi = 24Gi leaves ~4.6Gi headroom. **A 12Gi
  limit was briefly live and is the defect this clause exists to prevent recurring.**
- `ephemeral-storage` is bounded because buck2 build trees are disk-heavy and FRIC-017 was
  literally "No space left on device"; a runaway build is evicted rather than filling the node
  for every tenant.
- **No CPU limit deliberately**: a throttled compile presents as a flaky slow test, which is
  harder to diagnose than a noisy neighbour we can observe.

### D6 — Declared in `infra/arc/`, and the declaration is drift-checked

The fleet ships as values files under `infra/arc/` and is registered in
`infra/gitops/values.yaml` beside `external-secrets` and `registry`, the pattern those charts
already follow. The declaration was verified **field-for-field against the live CR at zero
drift** (maxRunners, minRunners, nodeSelector, both CAS labels, memory limit,
ephemeral-storage limit, cpu request, githubConfigSecret) — it records reality rather than
aspiration, which is the whole point of declaring it after an imperative bring-up.

### D7 — Live-PostgreSQL tests use a dedicated ephemeral same-Pod cell

The two merge-blocking live-PostgreSQL jobs select `oya-live-postgres-arm64`, not the
general-purpose `oya-arm64` set. Each ARC runner Pod owns one digest-pinned PostgreSQL 16 native
sidecar as a restartable init container. Its startup probe completes before the runner starts;
both database state and credentials are size-bounded memory `emptyDir` volumes deleted with the
Pod. An ordinary init container generates separate random admin and application passwords. The
workflow reads those files from `/run/oya-ci-postgres`, masks both values before use, and preserves
the existing admin-versus-`NOBYPASSRLS` application-role assertions.

No Kubernetes Secret or shared CNPG hostname is projected into either runner scale set. The
general-purpose runner receives no database admin environment, and the dedicated Pod exposes no
Service. A namespace NetworkPolicy selects only the dedicated cell label and denies cross-Pod
ingress; the tests reach PostgreSQL only over same-Pod localhost. A Cilium egress-deny selects
both `general` and `live-postgres` runner-cell labels and blocks the shared `oya-data` namespace
without default-denying the GitHub, CAS, and package egress those jobs require. This is
test-fixture isolation, not a production data-plane dependency.

The dedicated set is capped at `maxRunners: 1` and requests 32Gi ephemeral storage, above the
31,347,796Ki observed workspace that triggered a current-node DiskPressure eviction. A 34Gi limit
bounds further growth. This declaration is not rollout readiness: issue #1504 must supply and
verify sufficient node capacity before the GitOps application is independently reviewed and
deployed.

## Alternatives considered

- **Wait for the billing fix and stay on GitHub-hosted runners** — rejected as the *only*
  plan: it leaves merge authority dependent on a payment state, and does nothing toward the
  ADR-0515 D5 destination. Not mutually exclusive; fixing billing remains desirable.
- **A personal access token instead of an App** — rejected on blast radius (D3).
- **Reviving `oya/governance/iac/helm/lane-runner-pool/`** — rejected. It pins *legacy*
  `actions-runner-controller` 0.23.x (appVersion 0.27.0), targets `microservices/governance/…`
  paths that ADR-0131/0512 mark removal-candidate, references a `ghcr.io/oyatie/governance-runner`
  image that does not exist, sizes 8–200 replicas, and has never run. It is superseded in
  substance by this ADR and should be deleted in its own change rather than silently
  contradicted.
- **One scale set spanning both architectures** — rejected: non-deterministic platform per job,
  which breaks cache-namespace reasoning and reproducibility (D2).

## Consequences

**Positive.** Merge authority has a working substrate that consumes no Actions minutes. The
runner half of ADR-0515 D5 exists and is declared. Runners are co-resident with the
NativeLink CAS on the same architecture, which is the first time the cache and its consumer
have shared a namespace — previously the blocker that made warm-cache work unmeasurable.

**Negative / known gaps, tracked not hidden.**
- `image: ghcr.io/actions/actions-runner:latest` is a **mutable tag** in a repo where every
  workflow `uses:` is commit-pinned. It becomes a digest pin when the baked toolchain image
  lands and this reference changes anyway.
- The image carries **no toolchain**: probed, the stock runner has python3/git/curl but no
  compiler, no rustup, no zstd. Every job pays ~2 minutes installing them. This is the
  next slice.
- `minRunners: 0`, so every job pays pod scheduling plus a cold toolchain. A warm pool is
  queued *behind* the baked image, because pre-warming a pod that then installs rustc buys
  little.
- **No trust tiers.** ADR-0556 D1 distinguishes trusted-author from untrusted-author execution
  and prohibits the latter from writing the shared cache. One scale set cannot express that;
  the split arrives with the cache wiring.
- **`buck2 --num-threads` is uncapped**, so buck2 reads the node's core count from a pod
  requesting 2 — past the measured j=4 knee.
- The CAS is reachable at L3 (D4) but **not yet wired**: buck2 resolves `[buck2_re_client]`
  into `DaemonStartupConfig` from project config files only, and the resolver emits
  `--config-file`, which is measured-inert for that section.
- Single-node durability: `local-path` RWO PVCs and a one-replica CAS on a permanent substrate
  that is one laptop.

## Artifact accounting (ADR-0555)

This decision is the justification anchor for `infra/arc/OWNERS`,
`infra/arc/controller-values.yaml`, `infra/arc/runner-scale-set-arm64-values.yaml`,
`infra/arc/runner-scale-set-live-postgres-arm64-values.yaml`,
`infra/arc/live-postgres-runner-network-policy.yaml`, and the ARC registrations in
`infra/gitops/values.yaml`.

It is also the justification anchor for the baked runner image this fleet runs. One path per
bullet, spelled byte-exactly and unwrapped, because the born-accounting producer matches
path-like tokens and an abbreviated or line-broken path matches nothing:

- `infra/ci/runner-image/Dockerfile`
- `infra/ci/runner-image/OWNERS`

Why this decision owns them rather than a new ADR: the image exists only to serve this scale
set, and D4 already commits to a baked toolchain image for it — the recipe is that commitment's
artifact, not a separate decision. It was previously built from a copy living outside the repo,
which made the image holding merge authority unreproducible; committing it closes that hole and
is what brings it under ADR-0555 accounting in the first place.

The image additionally bakes **PowerShell** because the required lane runs its gate matrix under
`shell: pwsh` and the upstream `actions-runner` image ships none — GitHub-hosted images supplied
it invisibly, so the dependency only became apparent once this fleet took the jobs.
