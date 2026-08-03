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
related: [ADR-0131, ADR-0523, ADR-0535, ADR-0612, ADR-0554, ADR-0631]
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

### D2 — ONE SCALE SET PER ARCHITECTURE, arch-pinned by nodeSelector

Founder requirement: *"we should be agnostic. arm64 / amd64 should both be a supported path"* —
*"on this laptop arm. on another box that joins the cluster that is amd64, we can run amd64."*

So the cluster is heterogeneous and each scale set is pinned:
`nodeSelector: {kubernetes.io/arch: <arch>}`. **Without that pin a scale set schedules
anywhere**, and an "arm64" runner can land on an amd64 node and silently build for the wrong
platform — poisoning that platform's buck2 cache namespace, because action keys include `cpu:`
and `os:`. Adding capacity is joining a node; adding a PLATFORM is one more values file.
`infra/ci/install-buck2.sh` already carries digest-pinned arms for both architectures.

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

## Amendment 1 (2026-07-30) — D6's zero-drift claim was FALSE, and the mechanism was a typo

D6 asserts the declaration was *"verified field-for-field against the live CR at zero drift"* and
lists the fields checked. **That claim did not hold**, and correcting it here rather than in a new
ADR is deliberate: the claim's whole argument is "the declaration records reality", so the
correction belongs where the claim lives.

**Measured drift** between `infra/arc/runner-scale-set-arm64-values.yaml` at merge-base `1612db9e9`
and live `AutoscalingRunnerSet/oya-arm64` (generation 8, helm revision 11):

| field | declared | live |
| --- | --- | --- |
| `template.spec.containers[runner].image` | `ghcr.io/actions/actions-runner:latest` | `registry.oya-registry.svc.cluster.local:5000/oya/runner:arm64@sha256:ab648efd…` |
| `template.spec.containers[runner].env` | absent | `OYA_CI_PG_HOST`, `OYA_CI_PG_SUPERUSER`, `OYA_CI_PG_SUPERPASS` |

Every field D6 actually enumerated (maxRunners, minRunners, nodeSelector, both CAS labels, memory
limit, ephemeral-storage limit, cpu request, githubConfigSecret) still matched. The drift is in the
two fields D6 did **not** enumerate — which is the real lesson: a field-for-field check against a
hand-written field list cannot detect a field nobody thought to list. The replacement check is
mechanical and total: render the chart with the committed values and `kubectl diff` the result.

**MECHANISM — the committed values files were wired to NOTHING.** `infra/gitops/values.yaml`
registered both ARC entries with `valuesFile:` (singular string); `infra/gitops/templates/applications.yaml`
reads `.valueFiles` (plural list). Rendered proof at merge-base: both Applications emitted a
single-source, chart-only `source:` with no `helm:` block and therefore no `githubConfigUrl` — so an
Argo CD install would have deployed **chart defaults** or gone permanently Degraded, taking the
merge-authority fleet down. Because the committed file fed nothing, the only copy that ever reached
the cluster was a scratchpad copy that `helm upgrade` was run from. Drift by duplication, with the
duplicate as the *only* live path.

That is not a values-content bug and fixing the values alone would have been decoration. Three
things change together:

1. `valuesFile:` → `valueFiles: [...]` on both ARC entries.
2. `templates/applications.yaml` now `fail`s the render on `valuesFile`, with the message naming
   the correct key. A render error is the cheapest possible detector: it fires wherever the chart is
   rendered — bootstrap, Argo CD repo-server, or a human running `helm template` — needs no gate
   crate, and cannot be laundered by a stale baseline. Verified: the guarded render exits non-zero.
3. The values file is resynced to live, and the resync is proven by `kubectl diff`, not by reading.

**Standing premise correction, recorded because it changes what any of this means today:** there is
**no reconciler on this cluster**. `kubectl get ns argocd` → NotFound; no `argoproj.io` CRDs; no
Flux. Every `infra/**` file in the repo is currently inert IaC and the whole substrate was
hand-applied. D6's "drift-checked" is therefore aspirational until Argo CD lands — the drift check
that exists right now is the `kubectl diff` in a reviewer's hands, and that is what this amendment
uses.

## Amendment 2 (2026-07-30) — the baked toolchain image landed; its source is now in the repo

The first and second "known gaps" in *Consequences* (mutable tag; no toolchain) are closed. The
image is `…/oya/runner:arm64@sha256:ab648efd…`, digest-pinned, carrying buck2, the
`rust-toolchain.toml` toolchain, and **clang + lld** — `toolchains/BUCK` points the buck2 cxx
toolchain at the absolute path `/usr/bin/clang` as both compiler and linker, so a gcc-only image
fails every C/C++ action. An earlier build shipped gcc-only and passed a `cc --version` check
because `cc` resolves to gcc: the check verified a proxy, not the requirement.

**The image was unreproducible when this amendment was written, and that was the single most
fragile thing in the CI substrate.** Its Dockerfile existed only in a `/tmp` scratchpad, and the
live build Job read its context from a hand-made ConfigMap (`oya-build/runner-build-ctx`) holding
copies of `rust-toolchain.toml` and `infra/ci/install-buck2.sh`. The registry is one `local-path`
PVC on `Delete` reclaim: if that PVC went, the digest became unpullable with **no source to rebuild
from**, and merge authority stopped at `ImagePullBackOff` with no recovery path.

So `infra/ci/runner-image/Dockerfile` and `infra/ci/runner-image/build.yaml` are now the source.
Only the Dockerfile was committed: the scratchpad's `install-buck2.sh` and `rust-toolchain.toml`
were byte-identical (`diff` exit 0) to `infra/ci/install-buck2.sh` and the repo-root
`rust-toolchain.toml`, so committing them would have re-created the duplication. The build context
is the **repo root** and the two `COPY`s read the real files — which is also why `build.yaml` passes
no `context-sub-path`.

Facts in the Dockerfile comments that were each learned from a failed build, kept verbatim:
`rustup default` must be re-derived from `rustup show active-toolchain` or `rustc` works only inside
a directory holding a `rust-toolchain.toml`; `install-buck2.sh` must be run with **bash** (it uses
`local`); the buck2 binary lands under a `sha256-<digest>` directory whose name differs per
architecture, so the `/usr/local/bin/buck2` entry must be a `find`-discovered symlink.

`build.yaml` carries no shell at all — `command: ["buildctl"]` — and passes the git credential as a
BuildKit build secret (`GIT_AUTH_TOKEN`) rather than interpolating it into the context URL, so it
appears in neither the spec nor the pod logs. It is **not** registered in `infra/gitops/values.yaml`:
every app there gets `automated: {prune: true, selfHeal: true}`, and a Job that completes and is
reaped would be recreated forever. That leaves applying it the one remaining imperative step in the
CI substrate — a defect, recorded as such, whose designed-out path is an owned build controller plus
the ADR-0535 bump-bot closing the loop back onto the digest in the values file.

## Amendment 3 (2026-07-30) — D4's stated failure mode cannot occur on this cluster today

D4 calls the CAS client labels *"the mechanical precondition for the CAS ever being reachable"*.
On this cluster that is **false**: the CNI is `kube-flannel`, not Cilium, and Flannel ships no
NetworkPolicy controller — so `nativelink-cas-ingress` and `oya-kms/openbao-ingress` enforce
nothing and the labels are **inert**, not load-bearing. The labels are still correct to keep: they
become load-bearing the moment the registered `cilium` app lands (ADR-0148). But the L3-refusal-
presenting-as-timeout failure mode D4 warns about cannot currently happen, and a reader debugging a
CAS timeout today should not be sent to look at pod labels.

## Artifact accounting (ADR-0555)

This decision is the justification anchor for `infra/arc/OWNERS`,
`infra/arc/controller-values.yaml`, `infra/arc/runner-scale-set-arm64-values.yaml`,
`infra/ci/runner-image/Dockerfile`, `infra/ci/runner-image/build.yaml`, the two
`infra/gitops/values.yaml` ARC chart registrations, and the `valuesFile` render guard in
`infra/gitops/templates/applications.yaml`.
