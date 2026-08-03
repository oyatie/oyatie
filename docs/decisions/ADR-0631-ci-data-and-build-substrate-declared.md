---
id: ADR-0631
title: "The CI data + build substrate is declared: CloudNativePG as a shared version-floor operator, Cluster/oya-pg, exactly one BuildKit daemon, and an ESO-projected superuser credential instead of a copy"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-07-30
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0339, ADR-0378, ADR-0381, ADR-0515]
amends: []
related: [ADR-0043, ADR-0131, ADR-0148, ADR-0374, ADR-0510, ADR-0523, ADR-0606, ADR-0630]
milestone: W3
---

# ADR-0631: the CI data + build substrate, declared

## Status

**Proposed — 2026-07-30.** Ratifies four pieces of live, merge-authoritative substrate that had
**no decision record and no manifest anywhere in the repo**: the CloudNativePG operator,
`Cluster/oya-pg`, the BuildKit daemon that builds every in-cluster image, and the Postgres
superuser credential the CI gates authenticate with. Written alongside the ADR-0630 amendments,
which cover the runner fleet half of the same bring-up.

Proposed rather than Accepted deliberately: a freshly-`Accepted` ADR reds the
cross-artifact-agreement gate, and nothing here needs Accepted status to be true.

## Context

Everything below was brought up by hand on 2026-07-29/30 to unblock the live-Postgres and
in-cluster-build halves of `oya-ci-required`. Each piece was created from a `/tmp` scratchpad file.
Three properties of that make it urgent rather than merely untidy:

1. **`helm list` does not show it.** The operator was applied with `kubectl apply --server-side`
   from an upstream release YAML, so it is not a helm release and appears in no inventory.
2. **The only copies were scratchpad copies.** So were the ARC values (ADR-0630 amendment 1). Two
   independent instances of the same drift-by-duplication class in one day is a class, not bad luck.
3. **One PVC from unrecoverable.** `local-path` is `Delete` reclaim + `WaitForFirstConsumer`. A lost
   PVC re-bootstraps Postgres empty and makes the runner image unpullable with no source to rebuild
   from.

**Premise corrections found by measuring, recorded because they change what "declared" buys:**

- **There is no reconciler on this cluster.** `kubectl get ns argocd` → NotFound; no `argoproj.io`
  CRDs; no Flux. Every `infra/**` file is inert IaC today. Registering an app in
  `infra/gitops/values.yaml` therefore buys *reviewability and reproducibility now*, and
  *reconciliation later* — it does not buy drift correction today.
- **`infra/**` is largely unregistered.** Grepping every `infra/<dir>` against `values.yaml`:
  `external-secrets`, `nativelink`, `ci-webhook-gateway`, `branch-protection`, `capi`, `cloudflare`,
  and `sidero-metal` all have **zero** registrations. The live `ClusterSecretStore`s and
  NetworkPolicies were hand-applied from copies too. This ADR registers what it declares and does
  not pretend to close that backlog.
- **`buildkitd` was not undeclared — it was DOUBLE-declared, divergently** (see D3).
- **`console-rehearsal` does not currently exist**, and `kubectl get cluster.postgresql.cnpg.io -A`
  returns only `oya-data/oya-pg`. CNPG is shared *by policy* and sole-consumer *in fact* right now.
  It is treated as shared anyway: the console project recreates that namespace on demand, and a
  downgrade already broke it once (2026-07-30).

## Decision

### D1 — The CNPG operator is reconciled from the pinned UPSTREAM RELEASE MANIFEST: not the helm chart, and not vendored

Registered as `cnpg` in `infra/gitops/values.yaml`, sourced from
`github.com/cloudnative-pg/cloudnative-pg` at commit
`4b5e244a7d031f67e025c83c1555e7726ecbbfa1` (== tag `v1.30.0`), path `releases`, file
`cnpg-1.30.0.yaml`.

**Verified, not asserted.** `kubectl diff --server-side` against that exact file produced empty
output and exit 0 — the declaration is a proven no-op against live. The file at the pinned commit is
byte-identical to the `release-1.30` branch copy
(sha256 `f8bede43fe4ee0d478c2355b204a36876b2ae4faac60f2a9452280b293da3b88`), and the running operator
reports `cloudNativePGCommitHash: 4b5e244a7` — the live binary was built from this commit.

**Not the helm chart, with evidence.** `cnpg/cloudnative-pg` **0.29.0** has appVersion exactly
`1.30.0`, so the version constraint is satisfiable — but its render is not this install:

| | live | chart 0.29.0 |
| --- | --- | --- |
| Deployment | `cnpg-controller-manager` | `cnpg-cloudnative-pg` |
| ServiceAccount | `cnpg-manager` | `cnpg-cloudnative-pg` |
| args | includes `--secret-name=cnpg-controller-manager-config` | absent |
| resources | cpu 100m / mem 200Mi limits, cpu 100m / mem 100Mi requests | `{}` |
| imagePullPolicy | `Always` | `IfNotPresent` |

Because the **names** differ, adopting the chart would create a *second* operator Deployment beside
the running one — two controllers reconciling the same `Cluster` CRs — and `prune` would not remove
the original, which was never tracked. On an operator another project shares, that is materially
worse than any amount of YAML.

**Not vendored, either.** Copying 20,846 lines of an upstream file into `infra/` *is* a second copy,
and drift-by-duplication is the defect this whole change exists to remove: a vendored copy silently
diverges from upstream and nothing detects it. Pinning the upstream commit gives the same bytes with
no copy to rot. The cost is honest and stated: reconciling this app requires Argo CD to reach
`github.com` — which it already must, since `repoURL` is a GitHub repository — so the new exposure is
an upstream *repository*, not a new network trust boundary. Enabled by a two-line
`templates/applications.yaml` change letting a `type: path` app override
`repoURL`/`targetRevision`.

**SHARED SUBSTRATE, VERSION FLOOR, NEVER A CEILING.** Declaring this operator does **not** claim
exclusive ownership. The console project creates `Cluster` CRs against it. A downgrade broke console
once. Bumps are deliberate PRs that re-run `kubectl diff --server-side` against the new release
manifest **first**.

`ServerSideApply=true` (already set globally by the template) is a **requirement** here, not a
preference: the 1.2 MB manifest blows past the 256 KB client-side `last-applied-config` annotation
limit.

**To verify at adoption, not assumed:** the live objects carry field manager `kubectl` from the hand
`apply --server-side`; Argo CD applies as `argocd-controller`. Same-value fields co-own cleanly, but
confirm the first sync reports no conflict before trusting `selfHeal` on a shared operator.

### D2 — `Cluster/oya-pg` is declared at `infra/oya-pg/oya-pg.k8s.yaml`, and `enableSuperuserAccess` is part of it

PG 18.4, 3 instances, `minSyncReplicas: 1` / `maxSyncReplicas: 1` (so exactly one synchronous
standby, matching the decided ADR-0339 D-4 `pg-cluster` contract), 5Gi `local-path`,
`max_connections: 200`, modest resources with no CPU limit. `kubectl diff` → **exit 0**.

Two authoring rules make that exit-0 durable:

- **Operator-defaulted fields are deliberately absent.** `affinity`, `bootstrap.initdb`, `enablePDB`,
  `logLevel`, `postgresGID/UID`, ~20 `postgresql.parameters`, `probes`, `replicationSlots`,
  `primaryUpdate*`, `smartShutdownTimeout`, `start/stop/switchoverDelay`, `failoverDelay`,
  `storage.resizeInUseVolumes` are all injected by the CNPG mutating webhook. Writing them down would
  make the file a snapshot of an operator *version* rather than a declaration of intent, and every
  operator bump would then read as drift.
- **`enableSuperuserAccess: true` IS authored**, precisely because it was **patched on after
  creation** — the exact class of change that survives only if it lives in the file. The RLS gates
  assert that `rolsuper`/`rolbypassrls` cannot bypass row-level security, so the gate needs a real
  superuser *and* a plain app role; the **distinction is the test**. Without this, CNPG mints no
  superuser Secret, the D4 projection resolves NotFound, the runner env vars resolve **empty**, and
  the RLS gates go **green-on-nothing**. Deleting this line disarms a gate rather than weakening it.

### D3 — Exactly ONE BuildKit daemon, and it is the privileged one in `oya-build`

`infra/buildkit/buildkitd.k8s.yaml`. `kubectl diff` → **exit 0**.

`infra/ci-webhook-gateway/buildkit-build.yaml` already declared a *second*, divergent buildkitd
Deployment+Service:

| | committed twin | live |
| --- | --- | --- |
| namespace | `oya-ci` | `oya-build` |
| image | `moby/buildkit:v0.31.2-rootless` | `moby/buildkit:v0.18.2` |
| security | rootless, `runAsUser 1000`, `drop [ALL]` | `privileged: true` |
| args | `--addr=tcp://…`, `--addr=unix://…`, `--oci-worker-no-process-sandbox` | `--addr tcp://0.0.0.0:1234` |
| probe | `buildctl debug workers` | `buildctl --addr tcp://localhost:1234 debug workers` |

The `oya-ci` pair never existed on the cluster and never could — nothing reconciles that directory.
So this drift-by-duplication instance predates the ARC one and went unnoticed. **One daemon wins:
the privileged one that actually builds.** The twin's Deployment+Service are deleted and its Job is
repointed at `tcp://buildkitd.oya-build.svc.cluster.local:1234`. Keeping a rootless declaration as
aspiration while a privileged daemon serves traffic is exactly the ambiguity that produced this ADR.

This **contradicts ADR-0381's rootless framing and says so.** The rootless posture is not retracted
and is not what runs; making it rootless is a deliberate migration PR — which must also change the
probe, because the rootless daemon listens on a unix socket the privileged one does not.

Three traps recorded in the manifest because each cost a debugging cycle:

- **Privileged ⇒ MUST NOT pass `--oci-worker-no-process-sandbox`.** That flag is rootless-only and
  crash-loops a privileged daemon. The live object's own `last-applied-configuration` still records
  the flag being applied and then patched back out. Copying args between the two postures is the trap.
- **The readiness probe MUST pass `--addr`.** There is no unix listener, so the default
  `buildctl debug workers` dials a socket that does not exist and the pod never goes Ready — which
  reads as a broken daemon rather than a broken probe.
- **`pod-security.kubernetes.io/enforce: privileged` on ns `oya-build` is part of the declaration.**
  PSA `baseline` rejects this pod outright; that label is what makes it admissible.

`spec.clusterIP` is deliberately **not** pinned, unlike `infra/registry`'s Service. That one is
pinned because Talos `machine.registries.mirrors` resolves it by IP outside CoreDNS (and the runner
image reference in `infra/arc/` depends on that pin). Nothing resolves buildkitd outside CoreDNS, so
pinning here would add a collision hazard for no contract.

### D4 — The Postgres superuser credential is ESO-PROJECTED, never copied

`infra/oya-pg/superuser-projection.yaml`: a dedicated `ServiceAccount` + single-Secret `Role` +
`RoleBinding` in `oya-data`, a `ClusterSecretStore` using ESO's **`kubernetes`** provider, and an
`ExternalSecret` in `arc-runners` projecting only `username` + `password`.

What it replaces was **drift by construction**. CNPG owns `oya-data/oya-pg-superuser`
(`ownerReferences → Cluster/oya-pg`, `cnpg.io/userType: superuser`, `cnpg.io/reload: "true"`) and
rotates it on its own schedule. A hand copy cannot follow, and its failure mode is an auth error that
reads as a Postgres problem rather than a stale-credential problem. The copy also carried the
plaintext base64 password inside its `kubectl.kubernetes.io/last-applied-configuration` annotation,
so it leaked to anyone with `get` on object metadata, not merely on `.data`.

**Viability was verified on this cluster, not assumed.** ESO v2.5.0 is deployed;
`kubectl explain secretstore.spec.provider.kubernetes` confirms `remoteNamespace`,
`auth.serviceAccount`, `server.caProvider`. RBAC checked live: ESO can create the
ServiceAccount token in `oya-data` (`yes`) and Secrets in `arc-runners` (`yes`). All five objects were
admitted by the live `secretstore-validate` and `externalsecret-validate` webhooks under a server
dry-run — real admission, not schema-only.

Design choices that are decisions rather than defaults:

- **Least privilege costs three tiny objects and is worth it.** ESO's controller ClusterRole already
  grants cluster-wide secret get/list/watch, so `auth.serviceAccount` could just point at
  `external-secrets/external-secrets`. Rejected: this is a Postgres **superuser**.
- **Two keys, not eleven.** The source carries `username, user, password, dbname, host, port,
  pgpass, uri, jdbc-uri, fqdn-uri, fqdn-jdbc-uri`. A connection URI in a CI pod env is a credential
  in every log line that echoes the environment.
- **`refreshInterval: 1h` bounds staleness.** The interval is not the point; the **bound** is. The
  hand copy's window was unbounded.
- **Co-located with the `Cluster` that mints the source**, not in `infra/external-secrets/` — that
  directory holds OpenBao *store* definitions; this is a store *consumer*.

**Migration step, do not skip:** `creationPolicy: Owner` refuses to adopt a Secret ESO does not own —
it errors rather than overwriting. Delete `arc-runners/oya-pg-superuser` first. The window is always
safe because the scale set runs `minRunners: 0`, so no runner pod holds the old value between jobs.

### D5 — Enforcement is the render, not a new gate

The mechanical cause of the ARC drift was a silently-ignored key (`valuesFile` for `valueFiles`), so
`infra/gitops/templates/applications.yaml` now **fails the render** on it, naming the correct key.
Verified: the guarded render exits non-zero with that message.

A gate crate was considered and rejected as a second copy of the same rule. A render error fires
wherever the chart is rendered — bootstrap, Argo CD repo-server, or a human running
`helm template` — needs no baseline, and cannot be laundered. Its scope is honest: it guards the one
key measured to have cost a day, not every possible typo. Generalize to an unknown-key sweep if a
second silent key ever costs another.

The larger detector this change does **not** ship: nothing yet fails when an `infra/<dir>` exists
with no `infra/gitops/values.yaml` registration, which is the condition that let seven directories
become inert. That is a gate-shaped problem and belongs in its own change.

## Alternatives considered

- **The CNPG helm chart** — rejected on the object-rename evidence in D1: it stands up a second
  operator beside the running one, on a shared operator, with the original unprunable.
- **Vendoring `cnpg-1.30.0.yaml` into `infra/cnpg/`** — rejected: a 20,846-line copy of an upstream
  file is itself the duplication class under repair, and the pinned upstream commit delivers
  byte-identical content with nothing to drift. The hermeticity given up (Argo CD must reach the
  upstream repository) is stated in D1 rather than hidden.
- **Keeping the hand-copied Secret and documenting that it does not rotate** — rejected because a
  declarative projection was verified viable on this cluster. Documenting a rotation hole you can
  close is choosing the hole.
- **Pointing the `ClusterSecretStore` at ESO's own ServiceAccount** — rejected on least privilege
  (D4).
- **Keeping both buildkitd declarations, rootless as the aspiration** — rejected: an aspirational
  declaration alongside a divergent live one is precisely the ambiguity that produced this ADR.
- **Registering the build Jobs in `infra/gitops/values.yaml`** — rejected: every app there gets
  `prune + selfHeal`, so a completed-then-reaped Job would be recreated forever. Named as a defect
  in ADR-0630 amendment 2 with the designed-out path, not silently accepted.

## Consequences

**Positive.** The CI data + build substrate is reviewable and reproducible from the repo for the
first time. Three of the four items are proven to match live at `kubectl diff` exit 0. The superuser
credential now has a bounded staleness window and least-privilege read path instead of an unbounded
one and a metadata leak. One buildkitd exists instead of two, one of which was fiction.

**Negative / known gaps, tracked not hidden.**
- **No reconciler.** Until Argo CD lands, these files are reviewed IaC, not enforced state. Sync
  waves (`1` operator + buildkit → `2` Cluster → `3` projection) are written for a reconciler that
  does not yet exist.
- **The D4 projection is not live yet.** Its five objects are pure additions; the hand copy must be
  deleted before the first sync. Until then the unbounded-staleness defect is still present on the
  cluster, and this ADR is ahead of reality on exactly that one item.
- **Field-manager co-ownership on a shared operator is unverified** (D1).
- **Single-node durability.** `local-path`, `Delete` reclaim, three PVCs on two nodes, on a
  "permanent" substrate that is one laptop. Recreating `Cluster/oya-pg` re-bootstraps empty, so gate
  data must be reconstructible from the gate.
- **`infra/**` remains largely unregistered** (seven directories), and no detector exists for that
  condition (D5).
- **The privileged daemon is a real posture regression against ADR-0381's rootless intent**, held
  open deliberately rather than papered over.
- **CNPG postgres image is tag-pinned, not digest-pinned** (`ghcr.io/cloudnative-pg/postgresql:18.4`):
  the field drives CNPG's major-version upgrade path and must stay legible against
  `.status.pgDataImageInfo`. The operator records the resolved image in status.

## Artifact accounting (ADR-0555)

This decision is the justification anchor for `infra/oya-pg/oya-pg.k8s.yaml`,
`infra/oya-pg/superuser-projection.yaml`, `infra/buildkit/buildkitd.k8s.yaml`, the `cnpg`,
`oya-pg`, `oya-pg-superuser-projection` and `buildkit` registrations in
`infra/gitops/values.yaml`, the per-app `repoURL`/`targetRevision` override in
`infra/gitops/templates/applications.yaml`, and the removal of the duplicated buildkitd
Deployment+Service from `infra/ci-webhook-gateway/buildkit-build.yaml`.
