---
id: ADR-0630
title: "ARC is the owned-runner substrate, and its fleet is declared in git, not installed by shell"
status: Proposed
planning_impact: false
deciders: council-architecture
date: 2026-07-29
door: two-way
owner: cloud-ci-platform
supersedes: []
superseded_by: []
depends_on: [ADR-0515, ADR-0560]
amends: []
related: [ADR-0111, ADR-0523, ADR-0525, ADR-0535, ADR-0554, ADR-0556, ADR-0612]
---

# ADR-0630 — ARC runner fleet: declared, not shell-installed

## Status

Proposed (2026-07-29). Two-way door: the artifacts are reviewed Helm values for an
already-running release. Reverting the file removes the review surface, not the fleet.

## Context

GitHub-hosted runners are billing-blocked, so an owned in-cluster runner fleet is the
only working path to a green `oya-ci-required`. That fleet is live: Actions Runner
Controller (ARC) with an `AutoscalingRunnerSet` named `oya-arm64` on the permanent
3-node arm64 Talos cluster, min=0 / max=3, ephemeral, authenticated by a GitHub App
scoped to one repository. A job has run green on it.

ADR-0515 D5 names owned in-cluster runners as the destination but scopes the cutover
out of that campaign. **No decision record names ARC.** That choice was made by a
`helm install` typed into a shell, and its only durable record was three unreviewed
release secrets — `sh.helm.release.v1.arc.v1` in `arc-systems` and
`sh.helm.release.v1.oya-arm64.v1/v2/v3` in `arc-runners`. Three revisions of the
component that holds merge authority, none of which passed through review.

`git grep -iE 'autoscalingrunnerset|gha-runner-scale-set|arc-runners' origin/dev --
infra/ .github/ specs/` returns zero hits. The repo's binding cloud-native doctrine is
declarative state plus reconcilers with zero imperative ops; the fleet was the largest
standing violation of it, and the one with merge authority attached.

Undeclared state is not merely untidy here. Four concrete defects were invisible
because nothing was reviewable:

1. The runner pods carried no pod-template metadata at all, so they held neither
   `oya.io/nativelink-cas-reader` nor `oya.io/nativelink-cas-writer`. The live
   `nativelink-cas-ingress` NetworkPolicy admits the CAS ports only from pods carrying
   those labels. A NativeLink CAS (ADR-0560) and the runners sit in the same cluster on
   the same nodes, and the runners were denied at L3 — every job did a cold build one
   Service away from a warm cache, with no buck2 configuration able to change it.
2. The runner image was `ghcr.io/actions/actions-runner:latest`, a mutable tag, in a
   repo where every workflow `uses:` is commit-pinned.
3. Resources were requests cpu 2 / memory 4Gi with limits memory 12Gi, no cpu limit,
   and no ephemeral-storage bound at all. maxRunners 3 x 12Gi = 36 GiB against 28.6 GiB
   allocatable per worker, on a cluster SHARED with another project and co-resident
   with `nativelink-cas` on local-path storage.
4. No cap on buck2 action parallelism, so a pod requesting 2 cores drove the node's 5,
   past the measured knee (j=2 87s, j=4 67s, j=8 71s, j=18 98s).

## Decision

**D1 — ARC is the owned-runner substrate, recorded as a decision.** The
`gha-runner-scale-set-controller` / `gha-runner-scale-set` charts remain the runner
control plane for the ADR-0515 D5 destination. This ADR ratifies the choice that was
previously implicit in shell history; it does not introduce it.

**D2 — the fleet is declared in `infra/arc/`.** `infra/arc/controller-values.yaml`
declares the controller release and `infra/arc/runner-scale-set-oya-arm64-values.yaml`
declares the `oya-arm64` scale set, owned per `infra/arc/OWNERS`. These mirror the
proven-in-place shape of `infra/nativelink/nativelink-cas.k8s.yaml`: a reviewed file in
`infra/` that matches live cluster state exactly, with the reasoning for every value in
the file rather than in a commit message.

**D3 — architecture is a per-scale-set property, never a repo-wide one.** The fleet is
heterogeneous by design; amd64 nodes will join. Arch is selected by `nodeSelector` in a
per-scale-set values file, and both arm64 and amd64 stay first class. Nothing else in
the declaration may encode an architecture — in particular container images are pinned
to multi-arch OCI INDEX digests, never to a child manifest, since a digest is resolved
before its tag and a child pin silently forces one architecture.

**D4 — the CAS boundary is crossed by label, at reader tier only.** The runner pod
template carries `oya.io/nativelink-cas-reader: "true"` and NOT the writer label. AC
writes are the cache-poisoning vector; this scale set serves every job for the
repository including fork-PR jobs, so writer reach belongs to a separate scale set.
Trust tier is scale-set granularity, which is how ADR-0556's trusted/untrusted author
split is expressed in Kubernetes. The label grants network reach only: warmth still
requires the ADR-0556 license, and the client certificate is deliberately not mounted
until that flip.

**D5 — bounds are sized against measured node allocatable, not against intent.** Every
runner resource value is justified in-file by a `kubectl describe node` /
`/stats/summary` measurement taken against the live cluster, because the cluster is
shared and an over-committed runner evicts co-tenants — including the cache.

## Consequences

Slice 1 declares current state plus the four corrections above and changes no behavior
that is not named. It is verified the same way `infra/nativelink` was: by diffing the
declaration against the live release.

**This slice ships no reconciler, and that is stated rather than implied.** There is no
Argo CD on this cluster (`kubectl get ns argocd` returns NotFound), so a GitOps
Application row would declare a reconciler that does not exist — flag-only, and
incomplete by the repo's own enforcement-layering doctrine. Until an in-cluster
reconciler lands, drift between `infra/arc/` and the live releases is caught only by
re-reading `helm get values`. Closing that gap is the next slice, not this one.

Recorded, unfixed, so the next slice inherits the findings instead of rediscovering
them: the controller runs with `resources: {}` and is therefore BestEffort QoS while
holding merge authority; `oyatie-arc-app` is a hand-created Secret rather than an
ExternalSecret on the standing OpenBao path; the toolchain is installed per job; min=0
means every job cold-starts; and buck2 exposes no environment variable for
`--num-threads`, so a true parallelism cap remains a workflow-side change.
