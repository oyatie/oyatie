---
id: ADR-0612
title: "buck2 Remote-Execution phase: deploy nativelink-scheduler + nativelink-worker, flip remote_enabled=true behind per-identity RE authz + a canary that covers RE'd outputs"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-07-08
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0560, ADR-0556, ADR-0525, ADR-0517, ADR-0515, ADR-0559, ADR-0561]
amends: []
related: [ADR-0522, ADR-0039, ADR-0181, ADR-0562, ADR-0013]
related_specs:
  - /specs/cache-warmth-policy.json
  - /specs/cache-warm-license.json
  - /specs/remote-exec-policy.json
  - /specs/capability-registry.json
  - /specs/substrate-dependency-dag.json
milestone: W3
---

# ADR-0612: buck2 Remote-Execution phase — deploy the NativeLink scheduler + worker tiers, flip `remote_enabled=true`

## Status

**Proposed — 2026-07-08.** This ADR governs the **third tier**
of the owned build-cache substrate: the Remote-Execution (RE) phase. It **implements the unbuilt
delta** of a design that already exists — it does not redesign anything. The three-tier split
was founder-decided 2026-05-30 (`docs/ideas/nativelink-remote-cache-first.md`); ADR-0560 shipped
tier 1 (CAS + Action Cache, cache-first, `remote_enabled=false`); this ADR stands up tiers 2 and
3 (scheduler + workers) and flips `remote_enabled=true` for the warm-eligible classes.

The six open-question positions below are **decided, not deferred** — ratified under the founder's
2026-07-08 autonomous-drive delegation ("drive founder-gated items autonomously at the hyperscaler
quality bar; quality / properly-done over speed"), after an independent adversarial review verified
every cross-ADR claim against `origin/dev` and found all six safe to ratify (each preserves its
one-way door; this ADR is itself a two-way door). The ADR's lifecycle **status stays `Proposed`**
until those decisions propagate to their governance faces (a masterplan/roadmap node + the
`/specs/remote-exec-policy.json` spec + the decisions projection) — the D1–D6 implementation this
ADR anchors. Per the cross-artifact-agreement invariant an Accepted decision must reach all
propagation faces, so ADR-0612 advances Proposed→Accepted when that implementation lands (the same
lifecycle ADR-0560 followed for tier 1); the design is settled now, the formal acceptance rides the
propagation.

Number allocation: 0612 is the next free ADR (0605–0611 already exist as decision files; shard-1
took 0611), verified against the `docs/decisions/` file tree. The `docs/machine-readable/decisions.json`
and `docs/ADR-INDEX.md` indexes lag the file tree (they materialize at land time, not in this
draft), so file-tree freedom — not an index lookup — is the allocation evidence.

**Door.** `two-way` for this ADR itself: the RE flip is opt-in overlay wiring that reverts by
flipping `remote_enabled=false` (the same reversibility ADR-0560 relies on), and the scheduler +
worker tiers scale to zero. Two one-way doors are *touched but deliberately not opened* here:
ADR-0515 D5's owned in-cluster runner destination (OQ1) and ADR-0556 D1's cold-required floor
(OQ2). The ratified positions keep both intact — RE proceeds *behind* the D5 cutover and *around*
the cold floor, so no supersession is proposed.

## Context

### 1. What is already built (consumed wholesale, not re-decided)

- **ADR-0556** classified every build class as cold or warm as **policy-as-data**
  (`/specs/cache-warmth-policy.json`), fixed the one-way cold-required floor (release-image,
  integrity-canary, untrusted-author-presubmit, provenance-attestation), and made the trust
  invariant normative: *warm is admissible IFF the class is warm-eligible AND the most recent
  cold integrity-canary is GREEN* (D2). D3 sequences RE as stage 4: "flips `remote_enabled=true`
  only after cache-first is proven."
- **ADR-0560** deployed tier 1: `infra/nativelink/nativelink-cas.k8s.yaml` (NativeLink **v1.6.2**,
  CAS + AC, filesystem-on-PV slow tier — the SeaweedFS tier is queued behind a projected CA
  bundle per ADR-0560 D1), the two opt-in buckconfig overlays
  (`infra/ci/buckconfig/warm-cache-{rw,ro}.buckconfig`), the cache-only execution platform
  (`toolchains/cache/defs.bzl`, `remote_enabled = False`), the resolver + conformance gate
  (`cloud/cloud-ci/gates/oya-cloud-ci-cache-wiring-app`), the kill-switch
  (`/specs/cache-warm-license.json`), and the cold integrity-canary
  (`.github/workflows/cache-integrity-canary.yml`). It shipped **dark**: no live cluster is
  reachable from the CI lanes, and the remainder is enumerated in §2 from two sources.
- **ADR-0525 D3** is the RE staging authority ("`buck2 test @affected` on RBE, NativeLink
  recommended … aligns with ADR-0515 D5"); **ADR-0517** fixes the one content-addressed
  work-area hash that is simultaneously the SCM id, the buck2/RBE cache key, and the CD artifact
  hash — the invariant that makes an RE'd action's key portable across runners; **ADR-0522** is
  the one-graph/four-runners frame RE completes (the CI runner now dispatches to remote workers).

### 2. What is NOT yet built (the delta this ADR owns)

This delta is named by two sources: the founder-decided three-tier reservation
(`docs/ideas/nativelink-remote-cache-first.md`, which marks the scheduler + worker tiers "RE phase
only") plus ADR-0556 D3 stage 4 (the `remote_enabled=true` flip) for the first two bullets, and
ADR-0560's own "Queued (the honest remainder, tracked)" block for the last two (per-identity authz
and the divergent-key eviction reconciler):

- deploy the **`nativelink-scheduler`** (Scheduler + Execution + Capabilities coordinator) and
  **`nativelink-worker`** (aarch64 execute-on-cluster pods) tiers — the two K8s tiers the
  founder-decided split reserved for "RE phase only";
- flip **`remote_enabled=true`** for warm-eligible classes via a new RE execution platform + a
  buckconfig overlay, dark until licensed, fail-closed;
- **per-identity RE authz** replacing tier 1's coarse two-CA split — RE is remote *code
  execution*, a strictly larger surface than a blob cache, so it consumes the ADR-0559 Cedar PDP
  and ADR-0561 SVID/mTLS substrates;
- the **divergent-key eviction reconciler** (ADR-0556 D4.3 canary-RED remediation; ADR-0560's
  named follow-up) — moved from "hand-operation placeholder" to a real reconciler subcommand.

### 3. The blocker is unchanged and is the gating fact

ADR-0560 stated it plainly and it still holds: `oya-ci-required` runs on GitHub-hosted
`ubuntu-latest`, which **can never resolve an in-cluster `*.svc.cluster.local` endpoint.** For
tier 1 this only deferred *cache reads*; for RE it is fatal in two directions — the CI runner
cannot reach the scheduler to *submit* Execute requests, and the scheduler→worker fan-out is
itself in-cluster DNS. Dev-worktree cache reads hit the same wall. This is why OQ1 (below) is not
a preference but the prerequisite: **there is no RE phase until the runner can route to the
cluster.**

## Decision

Each decision below is the **ratified** design for the corresponding position; the
"Decisions ratified" section records each disposition. Nothing here changes live pipeline behavior
— like ADR-0560, this lands as deployable declarative artifacts + dark opt-in wiring + conformance
tests, and claims no running deployment.

### D1 — Deploy the scheduler + worker tiers (the two reserved K8s tiers)

Two new manifests beside the existing CAS manifest, same NativeLink v1.6.2 image, role selected
by config (the founder-decided "one image/release/version, three K8s tiers" model):

- `infra/nativelink/nativelink-scheduler.k8s.yaml` — Scheduler + Execution + Capabilities. The
  stateful coordinator; **few replicas** (Buildbarn runs a single scheduler; the split exists so
  a scheduler fault never takes down the CAS on every build's critical path). Not on any
  cache-only build's path.
- `infra/nativelink/nativelink-worker.k8s.yaml` — execute-on-cluster pods; **many, compute-
  scaled**, aarch64-native (the hyperscaler-optimal endgame: the cluster becomes the build farm).
  A `HorizontalPodAutoscaler` scales workers on scheduler queue depth; scale-to-zero when idle.

Both are born under the existing `infra/nativelink/OWNERS` (cloud-ci-platform). Worker pods carry
a **pinned worker image digest** (the execution environment is a cache-key input under ADR-0517 —
a worker-image drift is a nondeterminism source the canary must catch; see D5).

**Precedent (what this reimplements).** The scheduler / worker / CAS three-process split is
Buildbarn's `bb-scheduler` / `bb-worker` / `bb-storage` topology and Google's internal Bazel RBE
(the scheduler is separated from the CAS so it never sits on every cache-only build's critical
path); the compute-scaled, scale-to-zero worker pool is the EngFlow / BuildBuddy
remote-worker-farm model. NativeLink's "one image, role-by-config" packaging collapses that
decomposition into a single pinned release rather than three separately-built binaries.

### D2 — The RE execution platform + buckconfig overlay (flip `remote_enabled=true`, dark until licensed)

Mirrors ADR-0560 D3's opt-in shape exactly, one tier up:

- A new RE execution platform `toolchains//remote-exec:re-platform` (`toolchains/remote-exec/`)
  that sets `remote_enabled = True`, declares the worker execution-platform constraints
  (`os:linux`, `cpu:arm64`, pinned worker-image), and keeps `local_enabled = True` as fallback
  (RE with local fallback, not RE-only — a scheduler outage degrades to local exec, never a hard
  stop).
- One checked-in overlay `infra/ci/buckconfig/remote-exec.buckconfig` selecting that platform and
  carrying the `[buck2_re_client]` RE endpoint + the `[oya_re]` knobs (`re_enabled`,
  `re_licensed`) read by the resolver. **`[oya_re].re_licensed` is not an independent switch — it
  mirrors the single source of truth `remote_exec_licensed` in `/specs/cache-warm-license.json`;
  the resolver reads the license DATA and the overlay knob only reflects it, so the two can never
  disagree.** The root `.buckconfig` stays untouched (its prelude default platform is
  conformance-pinned by ADR-0560 D6).
- The resolver (`cloud/cloud-ci/gates/oya-cloud-ci-remote-exec-app`) extends ADR-0560's
  `build_class → {bypass | warm-ro | warm-rw}` mapping with an **RE-eligibility axis**
  (`/specs/remote-exec-policy.json` DATA): a class is RE-eligible IFF it is warm-eligible in
  `/specs/cache-warmth-policy.json` (single source — RE never widens the warm set) AND opted into
  RE in the RE policy DATA. Fail-closed everywhere: not RE-eligible → cache-only overlay (ADR-0560
  behavior, unchanged); unlicensed → cache-only; the four cold classes → bypass (empty argfile,
  never dial CAS or scheduler); RE emission without the caller's SVID identity → **hard error**.
- **Dark until licensed**: `/specs/cache-warm-license.json` gains `remote_exec_licensed: false`.
  Flipping it true is a reviewed change citing (a) the D5 owned-runner cutover done, and (b) the
  first GREEN RE-covering canary run (D5 below). Flipping it false is the RED / rollback response.

**Precedent (what this reimplements).** The RE execution platform with `os:linux` / `cpu:arm64` /
pinned-worker-image constraints is Bazel's `--remote_executor` + platform `exec_properties`
(`OSFamily`, container-image) construct, one-to-one; `local_enabled = True` fallback is Bazel's
`--remote_local_fallback`. buck2's native REAPI client is the owned seam and NativeLink is the
server wired behind it — the same client that already speaks to the tier-1 CAS, now pointed at a
scheduler.

### D3 — Per-identity RE authz: consume the Cedar PDP (ADR-0559) + SVID/mTLS (ADR-0561), fail-closed

Tier 1's two-CA writer/reader split (ADR-0560 D2) is the honest floor for a *blob cache*; it is
**insufficient for remote execution**, which runs attacker-adjacent action inputs on owned
compute. The new-HTTP/gRPC-surface doctrine is binding here (default-deny; verified principal;
the service refuses to execute without authz): RE Execute is the single most powerful surface in
the whole substrate. Therefore RE authz is **not deferred to a follow-up** — it is a bring-up
prerequisite:

- Every RE participant (CI runner client, scheduler, worker) presents an **X.509-SVID** (a SPIFFE
  Verifiable Identity Document, ADR-0561) for mTLS identity; the coarse client-CA split is
  replaced by per-identity SVID.
- The **Execute / write-AC decision is a Cedar PDP call** (ADR-0559): principal = SVID identity,
  action = `re:execute` | `re:write-ac`, resource = the RE platform + build class. The PDP fault
  posture is **fail-closed** (a PDP outage denies RE; the client falls back to local exec, never
  to unauthorized remote exec).
- Worker→scheduler and scheduler→CAS legs are mTLS'd on SVID; a worker that cannot present a
  valid SVID gets no work orders. Untrusted-author presubmits hold no SVID — the handshake
  refuses them (defense in depth atop GitHub's no-secrets-for-forks and the NetworkPolicy).

Consequence: RE bring-up **depends on** ADR-0561 live mTLS and ADR-0559's PDP being commissioned.
Live mTLS is largely satisfied already — ADR-0561 D5-bis marks slice-1b-i (trustd real X.509),
slice-1b-ii (PDP rustls mTLS), and slice-1b-iii-a/b (production mTLS boot) all landed; only
slice-1b-iii-c remains. This is therefore a small, mostly-closed sequencing edge, recorded in OQ4.

**Precedent (what this reimplements).** Per-caller mTLS identity is SPIFFE/SPIRE workload identity
— the CNCF-standard the ADR-0561 X.509-SVID implements — replacing the tier-1 CA-membership check
with a per-workload identity. Externalizing the `re:execute` / `re:write-ac` decision to a policy
decision point is the AWS Verified Permissions / Google Zanzibar externalized-authz pattern
(ADR-0559 Cedar), and matches how BuildBarn and EngFlow gate RBE by per-identity authorization
rather than by shared-CA trust. The default-deny, fail-closed-on-PDP-fault posture is the
new-HTTP/gRPC-surface doctrine applied to the substrate's most powerful verb.

### D4 — Divergent-key eviction reconciler (ADR-0556 D4.3, promoted from placeholder)

ADR-0556 D2 step 2 requires that a RED canary evict/quarantine the divergent action keys via a
reconciler/API action rather than a hand operation; ADR-0560 shipped the canary but named eviction
as a follow-up. This ADR lands it as a **reconcile subcommand of the RE resolver crate** (same crate —
it shares the CAS-API client and the policy DATA; a second crate for a reconciler that reuses both
would be unowned surface for no gain). Given a canary-RED verdict artifact naming divergent keys,
the reconciler drives eviction/quarantine of exactly those Action-Cache entries, idempotently,
emitting a structured receipt. **REAPI v2 exposes no delete/evict RPC** (CAS is
FindMissing/BatchUpdate/BatchRead; AC is Get/Update), so eviction cannot ride the standard REAPI
port — it rides a **NativeLink-specific store-admin surface**, isolated **adapter-side** behind the
owned port (the single RE component that names the server; see "Owned-stack destination"). It is a
controller-driven action (no shell, no operator command — ADR-0556 D4), reconciling toward "divergent
keys absent from the warm AC."

**Precedent (what this reimplements).** Driving CAS eviction through the store API rather than
racing the LRU is BuildBarn's completeness-checking / CAS-GC discipline, expressed as a
Kubernetes-style reconcile toward a declared goal state (divergent keys absent) rather than an
imperative purge — the reconciler is level-triggered on the canary-RED verdict artifact, so a
re-run is a no-op once the goal state holds.

### D5 — The canary is extended to cover RE'd outputs, and is mandatory for the RE flip

ADR-0556 D2's canary compares **cold-from-source** output digests against the **warm CAS**
digests for the same action keys. RE introduces a *new execution environment* (worker pods, a
pinned worker image) distinct from the cold ephemeral runner, so RE outputs can **silently diverge**
from cold outputs via the ADR-0556 D2 hermeticity defect (environment-dependent bytes; see
"RE-on-Rust risks" below). The canary is therefore **extended, not bypassed**:

- The warm-probe comparator (ADR-0560 D4 step 3, buck2 itself as the REAPI client) fetches the
  **RE-produced** warm entries for the pinned trunk graph and byte-compares them against the cold
  from-source build. Only a **divergent compared key** — a worker-image drift or a proc-macro
  nondeterminism producing bytes that differ from cold — surfaces as **RED** → suspend-all-warm +
  evict (D4). A *missing* RE key (e.g. from an eviction) is **not** a divergence: under ADR-0560 D4's
  GREEN/RED semantics (GREEN = ≥1 compared key, all identical; RED = any divergence) a miss merely
  shrinks the compared overlap, so it cannot by itself read RED.
- **Expected-RE-key coverage floor (closes the silent-omission gap):** the RE-covering canary run is
  admissible only if the RE-produced overlap meets the pinned-graph coverage minimum
  (`re_coverage_floor` in `/specs/remote-exec-policy.json`). A silently-evicted RE set that drops
  overlap below the floor **fails the run** (it cannot read GREEN by omission) — the analogue of
  ADR-0560 D4's `UNVERIFIED_EMPTY_OVERLAP`, raised from "zero overlap" to "below the coverage floor".
- `canary-policy.json` gains an RE axis (the pinned worker-image digest) so a worker-image change
  is a canary input, not an ambient surprise.
- **Mandate**: the `remote_exec_licensed` flip is inadmissible without a GREEN RE-covering canary
  run — the same "no canary, no warm" rule ADR-0556 D2 fixed, now "no RE-covering canary, no RE".
  Bounded coverage is unchanged (the pinned trunk graph; PR-cone-unique keys inherit trust from
  the determinism of the trunk actions beneath them — per-hit proof would be re-execution, which
  is just cold).

**Precedent (what this reimplements).** Byte-for-byte cold-vs-warm output comparison is the
reproducible-builds project's bit-identity methodology and Bazel's remote-cache-correctness
verification; treating the pinned worker-image digest as a canary input mirrors SLSA / hermetic-
build guidance that the execution environment is itself a build input, not ambient state. This is
the "buck2 cache cold-vs-warm" doctrine made mechanical: warm-by-default is sound *only* because a
cold integrity-canary keeps proving bit-identity, and RE adds a new execution environment the
canary must therefore re-prove.

### D6 — The cold-required floor stays local; RE does not open ADR-0556 D1's one-way door

The four cold-required classes (release-production-image, integrity-canary,
untrusted-author-presubmit, provenance-attestation) **never** get `remote_enabled=true`. The
resolver keeps them at bypass (ADR-0560's ratcheted floor), and the conformance gate pins that
they resolve bypass even under a licensed RE fixture. Executing a cold-required class on RE would
substitute remotely-produced bytes for a from-source derivation and **supersede ADR-0556's
one-way door** — this ADR proposes the opposite: RE respects the floor, so no supersession is
required. (This is OQ2's recommended position, made mechanical.)

**Precedent (what this reimplements).** Keeping release-image and provenance builds off the shared
executor/cache and on trusted from-source builders is SLSA's hermetic-trusted-builder requirement
and Google's own separation of "build for cache" from "build for release provenance" — a shared
remote executor is a build-integrity trust boundary that release artifacts must not cross.

## Owned-stack destination — NativeLink is a transient adapter behind the owned REAPI port

Per the transient-stack selection bar and the ports-designed-for-owned-stack doctrine, this ADR
does **not** adopt NativeLink as a permanent substrate; it absorbs NativeLink v1.6.2 as a
**transient adapter behind an owned port**, and names the destination explicitly so the eventual
cutover is a data event, not a redesign.

- **The owned port is the Bazel Remote-Execution API (REAPI) gRPC contract** — the same
  Execution / CAS / ActionCache / Capabilities wire protocol Bazel, buck2, Pants, and Reclient
  already speak. buck2's `[buck2_re_client]` config, the `toolchains//remote-exec:re-platform`
  execution platform, and the resolver are the owned seam; **none of them name NativeLink**. The
  review test a port must pass (would this trait/wire-contract change at cutover?) is satisfied:
  REAPI is a stable, multi-implementation standard, so the scheduler/worker behind it is swappable
  without moving a line of buck2-side wiring. The authz overlay (SVID + Cedar PDP, D3) sits on the
  *owned* side of that port, so it survives any server swap. **One exception, isolated adapter-side:**
  the D4 eviction reconciler rides a NativeLink-specific store-admin surface (REAPI v2 has no
  delete/evict RPC), so it is the single server-specific component — kept behind the owned port as a
  store-admin adapter so the REAPI port and buck2 wiring stay clean at cutover.
- **NativeLink qualifies under the transient-stack bar** (would AWS/Google adopt it as a temp?):
  cloud-native (K8s-native container, scale-to-zero), hyperscaler-grade (a production REAPI
  server), and — atypically for a transient dependency — **Rust-native** (rust-purity aligned; it
  enters as a container image, contributing zero third-party Rust crates to the build graph). The
  single caveat is license: it is **FSL-1.1-Apache-2.0, not Apache-2.0 today** (ADR-0013 Tier-3
  requires-review; see the license note), with an automatic per-release Apache-2.0 conversion
  ~2028.
- **The destination is a deliberately deferred two-way choice, resolved at the FSL→Apache
  conversion window, not pre-committed now.** Because the port is REAPI and NativeLink is already
  Rust and Apache-bound, "owned stack" is reachable by *either* (a) permanent adoption once
  NativeLink converts to Apache-2.0 — unlike a JVM substrate (cf. the Pulsar-transient /
  owned-Rust-destination event-bus pattern), no ground-up reimplementation is forced — or (b) an
  owned Rust REAPI scheduler/worker if the substrate outgrows NativeLink. The
  cohesive-owned-substrates rule applies: the unit of eventual replacement is the *whole RE
  substrate behind the REAPI port*, not a crate swap, and the buck2-side wiring does not move in
  either branch. This ADR commits to the **owned port now** and forward-declares the
  adopt-vs-reimplement destination decision to the conversion window as a non-blocking follow-up.

## Decisions ratified (founder autonomous-drive delegation, 2026-07-08)

Each below is the **ratified decision** for what began as an open question. The independent
adversarial review found all six recommendations safe to ratify (M1–M4 fixes applied); under the
2026-07-08 autonomous-drive delegation they are hereby **decided**. Each keeps its stated
**Recommendation** (now the ratified position) and records the **Ratified (delegation)** disposition;
the Decision section above encodes them as the implemented design.

**OQ1 — The gating prerequisite: owned in-cluster runners (ADR-0515 D5) vs a reviewed endpoint
exposure.** GitHub-hosted `ubuntu-latest` cannot resolve `*.svc.cluster.local`, which blocks RE
workers (scheduler submit + scheduler→worker fan-out) *and* dev-worktree cache reads. This is the
real blocker.
- **Recommendation:** owned in-cluster runners (ADR-0515 D5 destination). RE is the forcing
  function that makes the D5 owned-runner cutover a hard prerequisite rather than an eventual
  aspiration. A reviewed public gRPC ingress for the scheduler is the fallback, but it exposes a
  *remote-code-execution endpoint* to the public internet (a categorically worse blast radius
  than exposing a content-addressed blob cache) and still leaves the scheduler→worker in-cluster
  DNS unsolved. Sequence RE **after** D5 runners land.
- **Ratified (delegation):** RE gates on ADR-0515 D5 owned in-cluster runners (this touches, but
  does not open, the D5 one-way owned-runner destination). The reviewed public-endpoint fallback,
  with its RCE-exposure trade-off, is the path not taken.

**OQ2 — The cold-required floor (ADR-0556 D1 one-way door).** release-image / integrity-canary /
untrusted-author-presubmit / provenance-attestation must bypass the substrate; if RE executes or
serves them it supersedes ADR-0556's one-way door.
- **Recommendation:** RE keeps the four cold-required classes **local/cold** — never
  `remote_enabled=true`, conformance-pinned (D6). No supersession of ADR-0556 is proposed.
- **Ratified (delegation):** RE respects the cold floor; the four cold-required classes stay
  local/cold. Any future move of a cold class onto RE is a separate ADR that supersedes ADR-0556 D1.

**OQ3 — SLSA provenance for actions executed on worker pods (a new provenance surface; supply-chain
anchors ADR-0039 / ADR-0181).** RE'd actions run on owned compute; how do they attest?
- **Recommendation:** because OQ2 keeps `provenance-attestation` and `release-production-image`
  **cold** (off RE), RE never sits on the SLSA release-provenance path in W3/W4. RE'd warm actions
  therefore carry **execution attestation** — the worker-pod SVID identity (ADR-0561) + the action
  digest + the CAS output digests, recorded as a provenance predicate into the existing
  `oya-ci/cache-hit-report/v1` surface (ADR-0560 D5), not full per-action SLSA build provenance.
  Full SLSA-predicate emission per RE action is **forward-declared** under the supply-chain
  provenance anchors (ADR-0039 / ADR-0181) and lands only if/when an attested class is ever moved
  onto RE (which OQ2 says it is not).
- **Ratified (delegation):** RE'd actions attest via worker-SVID + action/CAS digests in the
  cache-hit report; a full per-action SLSA build-provenance predicate is deferred to the supply-chain
  provenance anchors (ADR-0039 / ADR-0181) and lands only if an attested class is ever moved onto RE.

**OQ4 — Per-identity RE authz now (ADR-0559 Cedar PDP + ADR-0561 SVID) vs defer.**
- **Recommendation:** **now.** RE Execute is remote code execution on owned compute — a strictly
  larger surface than tier 1's blob cache, and the new-surface doctrine requires fail-closed
  per-identity authz on it. Adopt SVID identity (ADR-0561) for every RE leg and a fail-closed Cedar
  PDP decision (ADR-0559) for `re:execute` / `re:write-ac`, replacing tier 1's coarse two-CA split.
  This makes RE bring-up **depend on** ADR-0559 PDP commissioning + the ADR-0561 mTLS tail (live
  mTLS is largely delivered — slice-1b-i/1b-ii/1b-iii-a-b landed; only slice-1b-iii-c remains).
- **Ratified (delegation):** per-identity RE authz (SVID + Cedar PDP) is a bring-up prerequisite;
  the tier-1 two-CA split is not carried into an interim RE window (it authorizes execution by CA
  membership, not identity).

**OQ5 — `substrate-dependency-dag.json` has no delivery-fabric node (forward-declared, absent by
design).** The DAG `_comment` states the Tier-S5 meta-substrate "de-branded node `delivery-fabric`
per the ci capability … [is] forward-declared per Appendix C and absent from v1.0.0 by design."
- **Recommendation:** RE does **not** force a `delivery-fabric` node into the v1.0.0 Tier-1 DAG.
  RE's bring-up ordering edges (storage/CAS → iam PDP/SVID → delivery-fabric RE) belong in the
  **separate, already-anticipated DAG v1.1.0 amendment** that lands the Tier-2 + S5 nodes — a
  flagged follow-up, not this ADR. This ADR only *records* the ordering; it edits no DAG node.
- **Ratified (delegation):** RE rides the existing forward-declaration; no DAG edit here. Whether
  RE's bootstrap dependency justifies **accelerating** the `delivery-fabric` node into a near-term
  v1.1.0 amendment is left to that separately-flagged DAG change; this ADR edits no DAG node.

**OQ6 — Canary coverage for RE'd outputs (extend ADR-0556 D2 canary or bound it).**
- **Recommendation:** **extend** the canary to compare cold-from-source against **RE-produced**
  warm entries, with the pinned worker-image digest as a canary input (D5). RE nondeterminism
  (proc-macro, worker-env drift) then surfaces as RED. Coverage stays bounded to the pinned trunk
  graph (unchanged price); the RE flip is inadmissible without a GREEN RE-covering canary.
- **Ratified (delegation):** the canary is extended to cover RE'd outputs and the RE flip requires
  a GREEN RE-covering run (D5). A dedicated RE-only canary cadence distinct from the cache canary is
  the alternative not taken.

## License note — NativeLink v1.6.2 is FSL-1.1, a Tier-3 requires-review license (ADR-0013)

The RE phase **deepens** reliance on NativeLink (from a blob cache to the coordinator + execution
farm), so the license posture is recorded consciously here rather than inherited silently.

- The pinned image is `ghcr.io/tracemachina/nativelink:v1.6.2`. NativeLink is licensed
  **FSL-1.1-Apache-2.0** (Functional Source License 1.1, with an Apache-2.0 future grant that
  converts each release to Apache-2.0 two years after its publication) — **not** Apache-2.0 today.
- Per **ADR-0013**, FSL-1.1 is **Tier 3 — requires review** (`council-architecture` + `legal`
  sign-off; the license lane emits a `requires-review` label). NativeLink runs as a **container,
  not a workspace crate** (zero new third-party Rust deps enter the build graph), and the
  Tier-3 gate governs dependency acceptance — but the conscious posture must be on the record
  because this ADR increases the substrate's dependence on that FSL-licensed component.
- **License characterization is consistent on `dev`:** ADR-0560 D1 prose and the CAS manifest
  (`infra/nativelink/nativelink-cas.k8s.yaml`) were corrected from drafting-era "Apache-2.0" to
  FSL-1.1-Apache-2.0 by PR #1215 (merged; commit `19861ee08`, inside this PR's base `dc79b3848`).
  This ADR matches `dev` — nothing is queued on the license characterization.

## RE-on-Rust risks (why the canary is mandatory, not optional)

Two documented **loud** RE-on-Rust failure modes, plus the **silent** hermeticity class the canary
exists to guard, together frame why RE ships with both the fallback and the canary:

- **CAS eviction hard-fail (buck2#862) — loud:** OSS remote execution can hard-fail a build when
  the CAS evicts a blob the action still references (a client-side hard error at output
  materialization). Mitigation posture: `local_enabled = True` fallback (D2) so an eviction miss
  degrades to local exec rather than a red build; the eviction reconciler (D4) drives *deliberate*
  eviction through the store-admin surface rather than racing the LRU.
- **Rust proc-macro cross-platform build failure (buck2#1206) — loud:** this is a **hard build
  failure**, not silent nondeterminism — a proc-macro's dependency `.rlib` is not provided across
  the local→worker exec-platform transition, so the action fails with "can't find crate". The same
  `local_enabled = True` fallback (D2) degrades the affected action to local exec rather than a red
  build.
- **The silent class the canary actually guards (ADR-0556 D2 hermeticity defect):** the reason
  byte-level RE↔cold verification is required is not either loud failure above but the general
  defect that **content-addressing keys an action without guaranteeing its bytes are hermetic** — a
  new execution environment (worker pod, pinned image) can produce environment-dependent bytes that
  are silently laundered into every downstream warm hit. The extended canary (D5) is the detector
  for that silent class; the two loud modes are caught at build time.

Together they are why RE ships **with** both the `local_enabled` fallback (for the loud #862/#1206
modes) and the RE-covering canary (for the silent hermeticity class) in the same enabling change —
the ADR-0556 "no canary, no warm" rule, tightened to "no RE-covering canary, no RE".

## Placement (ADR-0562 closed capability-registry)

- CAS / blob substrate → **`storage`** capability (charter: "Object/blob storage +
  content-addressed store (CAS / NativeLink)"), already homed by ADR-0560. Unchanged here.
- RE scheduler/worker **wiring**, the resolver, and the conformance gate → **`ci`** capability
  (charter: "The delivery fabric: cloud-ci gates, controller, Tide merge queue…"; `dag_node:
  delivery-fabric`). The RE resolver crate lives under `cloud/cloud-ci/gates/`.
- NativeLink K8s manifests → `infra/nativelink/` (the scheduler + worker manifests land beside the
  existing CAS manifest). buck2 overlays → `infra/ci/buckconfig/` + `toolchains/`.
- **No new top-level directory** is created.

## Alternatives considered

- **Ship RE live in one change (scheduler + workers + flip, no dark phase)** — rejected: couples
  reviewed declarative artifacts to operational cluster + runner-cutover work, the exact coupling
  ADR-0560 avoided; the canary + conformance + license lattice is what makes the eventual flip a
  data event, not a design event.
- **RE-only execution platform (no local fallback)** — rejected: a scheduler outage or a
  buck2#862 eviction race would hard-stop every warm build; `local_enabled = True` degrades to
  local exec instead (D2).
- **Keep tier-1's two-CA split for RE authz** — rejected: it authorizes remote *code execution* by
  CA membership, not identity, and cannot express per-principal Execute/write-AC decisions; RE is
  the surface that most needs the ADR-0559 PDP + ADR-0561 SVID (OQ4).
- **Public gRPC ingress instead of owned in-cluster runners** — rejected as the primary path:
  exposes an RCE endpoint publicly and does not solve scheduler→worker in-cluster DNS (OQ1);
  recorded only as the explicitly-worse alternative (OQ1's path not taken).
- **Force a `delivery-fabric` DAG node in this ADR** — rejected: the DAG v1.0.0 forward-declares
  it by design; adding it here would diverge from the verbatim Tier-1 example. It belongs in the
  separately-flagged v1.1.0 amendment (OQ5).

## Consequences

**Positive.** The owned build farm (aarch64-native, cluster-as-build-farm — the hyperscaler
endgame of `docs/ideas/nativelink-remote-cache-first.md`) becomes deployable in review rather than
at bring-up; RE authz is born fail-closed and per-identity (SVID + PDP) instead of retrofitted;
the canary covers the actual RE execution environment, so proc-macro/worker-image nondeterminism
is caught before it launders into warm hits; the cold floor and the one-way doors stay intact
(no supersession).

**Negative / cost.** RE bring-up now has a hard dependency chain (D5 owned runners → ADR-0561 live
mTLS + ADR-0559 PDP → RE flip) that lengthens the critical path; two new K8s tiers + a new overlay
+ a new platform + a new resolver crate are new governed surfaces (mitigated: conformance-gated,
canonical-JSON-governed DATA, ADR-0555 accounting-registered); the FSL-1.1 Tier-3 posture deepens
(license-lane governed, recorded above); the extended canary spends additional RE build cost per
run (the deliberate ADR-0556 D2 price, one tier up).

**Queued (honest remainder).** The live cluster + runner cutover (OQ1) remains operational work on
FRIC-1781070457-buck2-no-shared-cache; the DAG v1.1.0 `delivery-fabric` amendment (OQ5) is a
separately-flagged follow-up.

## Artifact accounting (ADR-0555 — every new file owned + justified)

This decision is the justification anchor for every artifact the RE-phase implementation will add.
Nothing here modifies the root `.buckconfig` or ADR-0556's cold-required floor DATA.

- **Deployment (RE tiers):** `infra/nativelink/nativelink-scheduler.k8s.yaml`,
  `infra/nativelink/nativelink-worker.k8s.yaml` — born under the existing
  `infra/nativelink/OWNERS` (no new OWNERS).
- **RE execution platform:** `toolchains/remote-exec/defs.bzl`, `toolchains/remote-exec/BUCK`,
  ownership seed `toolchains/remote-exec/OWNERS`.
- **Opt-in RE overlay:** `infra/ci/buckconfig/remote-exec.buckconfig` — born under the existing
  `infra/ci/buckconfig/OWNERS` (added by ADR-0560).
- **RE-eligibility + canary-RE DATA:** `specs/remote-exec-policy.json` (RE-eligible classes ⊆
  warm-eligible; pinned worker-image digest; canary RE axis; `re_coverage_floor` — the expected-RE-key
  coverage minimum a canary run must meet to be admissible, D5).
- **License-flip DATA (edit, not add):** `specs/cache-warm-license.json` gains
  `remote_exec_licensed: false` — an edit to the ADR-0560 kill-switch, not a new file.
- **RE resolver + eviction reconciler + conformance gate:**
  `cloud/cloud-ci/gates/oya-cloud-ci-remote-exec-app/Cargo.toml`,
  `cloud/cloud-ci/gates/oya-cloud-ci-remote-exec-app/BUCK`,
  `cloud/cloud-ci/gates/oya-cloud-ci-remote-exec-app/src/lib.rs`,
  `cloud/cloud-ci/gates/oya-cloud-ci-remote-exec-app/src/main.rs`
  (subcommands: `resolve-re`, `reconcile-evict`, `re-canary-verdict`),
  `cloud/cloud-ci/gates/oya-cloud-ci-remote-exec-app/tests/remote_exec_conformance.rs`.
- **Canary extension (edit, not add):** `.github/workflows/cache-integrity-canary.yml` gains the
  RE-probe comparison + worker-image axis — an edit to the ADR-0560 canary, not a new file.
- **This ADR + index materialization (generated at land, listed for completeness, NOT edited in
  this draft):** the ADR file itself
  `docs/decisions/ADR-0612-buck2-remote-execution-phase-nativelink-scheduler-worker.md`, and the
  materializer-generated index updates `docs/machine-readable/decisions.json` (this ADR's record)
  + `docs/ADR-INDEX.md` (this ADR's INDEX row). These are produced at land time by the
  materialize step, never hand-edited in the PR; they are enumerated here only so ADR-0555
  accounting is complete.

## Verification

RE-phase bar (mirrors ADR-0560's slice bar, one tier up): deployable artifacts + dark wiring +
extended canary machinery, mechanically conformance-checked; **no live-enforcement claim** beyond
the conformance gate itself — the live cluster + runner cutover stays queued (OQ1).

- `buck2 test //cloud/cloud-ci/...` green, including the new `oya-cloud-ci-remote-exec-app`
  unittest (resolver RE-eligibility fail-closed lattice: not-RE-eligible → cache-only,
  unlicensed → cache-only, cold class → bypass under a licensed fixture, missing-SVID → hard
  error; eviction-reconciler idempotency + receipt; RE-canary verdict states incl. RED on a
  worker-image-drift fixture) and the conformance gate (cold floor pinned bypass under a licensed
  RE fixture; overlay parses + selects `toolchains//remote-exec:re-platform` + claims
  `remote_enabled=true`; root `.buckconfig` still carries no RE section).
- Overlay reality check: `buck2 audit config --config-file remote-exec.buckconfig` shows the RE
  client + `[oya_re]` keys; `buck2 audit execution-platform-resolution` under the overlay resolves
  `toolchains//remote-exec:re-platform`; without the overlay, resolution stays
  `prelude//platforms:default` (conformance-gated).
- Manifests parse; scheduler/worker `nativelink.json` config is valid JSON with the worker
  execution-platform constraints + pinned image; structural k8s schema validation against a live
  API server + the actual RE round-trip (submit → worker execute → CAS write → warm hit on a
  second runner) are **bring-up checks** (no live cluster reachable from CI — disclosed, same as
  ADR-0560).
- Fallback fixture (proves the "never a hard stop" claim, not just asserts it): a resolver/platform
  unit fixture exercises **scheduler-unreachable** and **CAS-eviction-at-materialization** (buck2#862
  is a client-side hard error when a referenced cached output is evicted at materialization) and
  asserts the action degrades to **local exec** under `local_enabled = True` — no red build.
  `local_enabled` governs the execution fallback; the fixture is what makes the degradation a tested
  property.
- License: the `oya-governance-license` lane records NativeLink v1.6.2 as FSL-1.1 Tier-3
  (requires-review), not Apache-2.0.

---
*Proposed 2026-07-08; the six OQ positions ratified under the founder autonomous-drive delegation,
advancing Proposed→Accepted when the implementation propagates them (cross-artifact-agreement
invariant). Tier 3 (Remote Execution) of the
owned build-cache substrate: ADR-0560's CAS made executable. Consumes ADR-0556 (warmth
classification + cold floor + canary invariant), ADR-0525 D3 (RE staging), ADR-0517 (portable
content-addressed action key), ADR-0515 D5 (owned runner destination). Per-identity RE authz
consumes ADR-0559 (Cedar PDP) + ADR-0561 (SVID/mTLS). Surface model per ADR-0556 D4 (declarative
data + API-driven services; no operator CLI); placement per ADR-0562 (storage = CAS/blob, ci =
delivery-fabric); accounting per ADR-0555. The six one-way/sequencing questions are ratified as the
recommended positions — each preserves its one-way door; this ADR is itself two-way.*
