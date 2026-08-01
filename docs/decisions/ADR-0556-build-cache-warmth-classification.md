---
id: ADR-0556
title: "Build cache-warmth classification: deliberate cold/warm policy-as-data + the cold integrity-canary trust anchor"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-06-12
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0515, ADR-0525, ADR-0392]
amends: []
related: [ADR-0039, ADR-0181, ADR-0516, ADR-0522, ADR-0523, ADR-0526, ADR-0544, ADR-0546, ADR-0548, ADR-0551, ADR-0552, ADR-0554, ADR-0555]
related_specs:
  - /specs/cache-warmth-policy.json
milestone: W3
---

# ADR-0556: Build cache-warmth classification — deliberate cold/warm policy-as-data + the cold integrity-canary trust anchor

## Status

**Proposed — 2026-06-12 (authored for founder sign-off; door: one-way for the cold-required
floor and the trust invariant; warm-eligible membership is reversible DATA).**

Founder directive 2026-06-12: *"some things should be cold. some things can be warm. make that
distinction well."* This ADR is the foundation both the interim CI quick-wins and the NativeLink
CAS deployment (the queued W3 vertical) consume. It classifies; it deploys nothing.

## Context

### 1. Today is blanket-cold, and blanket-cold is measurable waste

The live `oya-ci-required` workflow (`.github/workflows/oya-ci-required.yml`) is cold almost
everywhere:

- The generated-face materialization (`buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin`) runs in
  **every one of the 16 gate matrix legs** plus the producer-regen, registry-drift,
  cloud-ci-firewall, buck2, and affected-set lanes — the same accounting-registry producer hub is
  rebuilt over and over per workflow run, even though every invocation derives from the same
  candidate tree.
- The **per-gate cargo matrix compiles the shared workspace dependency graph once per leg**, and
  the buck2 lane then builds the same graph again — a structural double-build of one graph.
- Every job is `ubuntu-latest` local execution; buck2 has **no remote cache configured**
  (`.buckconfig` carries no `[buck2_re_client]`). The only warmth is the interim stable-key
  `actions/cache` buck-out restore (PR #659, FRIC-1781070457-buck2-no-shared-cache).

ADR-0515 D4 already rules the destination — *wall-clock tracks the size of the change, not the
repo* — and ADR-0525 D3 stages RBE/CAS (NativeLink recommended). What neither decides is **which
builds are allowed to be warm**, so every cache conversation re-litigates safety from scratch.

### 2. Naive blanket-warm is the opposite failure

One shared cache that everything reads and writes is unsafe on well-understood grounds:

- **Cache poisoning** (the Bazel/Google RBE security model): an untrusted writer who controls
  action inputs can seed malicious outputs under action keys that trusted builds will later hit —
  arbitrary code injection into every downstream consumer. Bazel's remote-caching guidance is
  explicit that only trusted builders may write to a shared cache.
- **Non-determinism laundering**: content-addressing guarantees *keying*, not *hermeticity*. A
  non-hermetic action poisons the cache with environment-dependent bytes (ADR-0515 D4:
  "non-hermetic actions poison the cache") and warm hits then silently mask the defect.
- **Release/provenance integrity**: a release artifact assembled from cache hits attests build
  steps that did not happen in that build (SLSA provenance must describe the actual build;
  reproducible-build verification requires re-derivation). ADR-0181 (Accepted) already requires
  signing + provenance on the image promotion path, with ADR-0039's fuller supply-chain stack
  (SBOM dual-format, Cosign, SLSA) Proposed alongside.

### 3. The doctrine that resolves the tension

Warm-by-default is sound **only because** a cold integrity-canary continuously proves the warm
cache is bit-identical to cold. The canary is the trust anchor: warmth is never an article of
faith in content-addressing, it is a continuously re-verified property. This is the same
fail-closed shape the substrate already uses — `registry-drift` proves committed == regenerated
byte-for-byte instead of trusting that nobody hand-edited a face (ADR-0515 D1), and ADR-0552
splits stable from volatile facts so cacheable inputs are deliberately chosen, not ambient
(the in-repo instance of Bazel's stable-status / volatile-status split).

## Decision

### D1 — The cache-warmth classification is POLICY-AS-DATA (R0 pack-shape)

The classification ships as `/specs/cache-warmth-policy.json` — born pack-shaped per the
ADR-0548 paved-road rule and the ADR-0544/ADR-0546 precedent: every build class maps to
`{warmth: cold|warm, cache_read: bool, cache_write: bool, reason}`, all repo-specifics live in
the DATA, and downstream consumers (interim CI quick-wins, the NativeLink CAS vertical, the
future conformance gate) read the policy rather than re-deciding warmth per change.

**COLD-REQUIRED** (warmth would defeat the purpose — the one-way floor):

| build_class | read | write | WHY cold |
|---|---|---|---|
| `release-production-image` | no | no | Reproducibility + SBOM/provenance integrity (ADR-0039, ADR-0181): the shipped artifact must derive from exactly its sources via a from-source build whose SBOM describes what actually went in; a cache hit substitutes bytes whose derivation was attested elsewhere or nowhere, and any poisoned or non-deterministic entry flows straight into the signed artifact. No write-back: release builds run with the most-privileged signing identity; their outputs are signed artifacts, not cache fodder, and writing from the highest-privilege context maximizes blast radius. The rust-purity doctrine's sole cargo exception (`cargo --release` + lto fat + locked) lives on this path and is outside the buck2 graph anyway. |
| `integrity-canary` | no | no | It exists to prove warm == cold; any cache participation makes the proof circular (a read tests the cache against itself; a write is redundant when green and harmful when red). Scheduled from-empty build of the pinned graph whose output digests are byte-compared against the warm CAS digests for the same action keys. cold != warm is a hermeticity/non-determinism defect — fail closed (D2). |
| `untrusted-author-presubmit` | no | no | Anti-poisoning (Bazel/Google RBE model): an untrusted PR controls action inputs; with write access it seeds poisoned outputs under keys trusted builds will hit. The write prohibition is one-way. Default is full isolation (defense in depth; no cache-probing side channel). A later read-only relaxation (`cache_read: true`) is admissible without weakening anti-poisoning — reads cannot inject — but is a separate reviewed policy change. Enforced at the CAS service boundary (keyed authn — untrusted runs hold no key), never by client-side discipline (D4). |
| `provenance-attestation` | no | no | SLSA provenance must describe the build that actually happened; serving cached outputs while attesting build steps fabricates provenance. Reproducible-build *verification* requires re-derivation — cache reuse silently converts "verified reproducible" into "trusted cached claim". |

**WARM-ELIGIBLE** (warmth is pure velocity: content-addressed actions make a hit bit-identical
to a cold build, so there is no correctness trade — *licensed by D2*):

| build_class | read | write | WHY warm |
|---|---|---|---|
| `presubmit-trusted-dep-closure` | yes | yes | The third-party crate closure (reindeer-vendored, lockfile-pinned) is identical across every PR sharing a lockfile; rebuilding it per leg and per run is pure waste. |
| `presubmit-trusted-affected-cone` | yes | yes | The affected-target cone (ADR-0525 D3 uquery owner→rdeps, now binding via the ADR-0554 affected-set lane): only genuinely changed actions miss; the unchanged cone is a hit. This is ADR-0515 D4 made real — ADR-0554's own cost note names the shared CAS direction as what amortizes its full-tier runs. |
| `dev-agentic-iteration` | yes | yes | Agent-lane and dev-loop builds in throwaway worktrees see 0% hits today (FRIC-1781070457); a warm shared cache makes the agent fleet's wall-clock track the size of each change. |
| `gate-fleet-shared-graph` | yes | yes | The gate fleet's shared dependency hub — the accounting-registry producer and the common workspace graph the CI audit found rebuilt ~13x per workflow run across legs. One build, many consumers. |
| `postmerge-dev-trunk` | yes | yes | The canonical trusted populator (the Bazel/Google deployment pattern: post-merge CI fills the cache, presubmits hit it). Trunk content is by definition admitted content — it passed `oya-ci-required`. |

**Trust boundary** (who is a trusted author): a same-repo branch pushed by an authorized writer
and admitted into the governance pipeline (the `required_workflow` lanes) is trusted; fork PRs
and any context without the CAS write key are untrusted. GitHub already enforces a natural seam
(fork PRs receive read-only tokens and no secrets), but the binding enforcement is the CAS
service boundary (D4), not runner configuration.

**Residual risk, stated honestly**: a trusted presubmit lane builds *unreviewed* code while
holding the CAS write key, so a compromised trusted lane could write poisoned entries under
arbitrary keys (the reason some Google deployments restrict writes to post-submit). Accepted
here because the writer set is small, keyed, and auditable (CAS API logs every write — D4), the
canary detects cold/warm divergence on covered keys, and the hardening escape hatch —
postsubmit-only writes, presubmit read-only — is an admissible shrink-of-warmth DATA edit
(no door) if the threat materializes.

**Door asymmetry**: the cold-required floor is one-way — moving a cold-required class to warm
requires superseding this ADR. Warm-eligible membership is two-way DATA — any warm class may be
degraded to cold by a policy edit (shrink-of-warmth is always allowed; growth of warmth is a
reviewed change).

### D2 — The trust invariant: warm is admissible IFF the cold integrity-canary is green

Stated normatively, the invariant the whole architecture hangs on:

> **A build class MAY run warm if and only if (a) it is warm-eligible in
> `/specs/cache-warmth-policy.json` AND (b) the most recent scheduled cold integrity-canary run
> is GREEN (cold output digests byte-identical to the warm CAS digests for the same action
> keys).**

The canary is a **periodic, scheduled, declarative job** (D4): a from-empty build of the pinned
graph on a clean runner, with zero cache read and zero cache write, whose action-output digests
are compared against the warm substrate's entries for the same keys. GREEN licenses warmth;
nothing else does.

Scope honesty: the canary is a continuously re-verified anchor **over the keys it covers**, not
a per-hit proof. Divergence detection applies to canary-covered keys — the pinned trunk graph:
trunk-graph keys minted between runs are covered at the next scheduled run (a lag of at most one
canary period), while keys unique to an unmerged PR cone are never canary-covered and inherit
trust from the determinism of the shared trunk actions beneath them (per-hit proof would be
re-execution, which is just cold). That bounded coverage is the deliberate price of warmth;
cadence is a tunable of the CAS vertical.

**Canary RED response (fail-closed, mechanical — never "keep serving and hope"):** per the IFF
above, a RED canary **suspends ALL warm reads fleet-wide pending the next GREEN run** — the
steps below are the durable remediation, never a license to keep serving warm on non-divergent
keys while RED stands.

1. RED is a blocking hermeticity/non-determinism defect (an ADR-0525 D4 invariant violation),
   never tolerable noise. The warm cache is structurally suspect from that moment.
2. The divergent action keys are evicted/quarantined from the CAS immediately — a
   reconciler/API action driven by the canary controller, not a hand operation.
3. A friction-ledger row opens mechanically (ADR-0544 closed loop) and the divergence is
   root-caused. If root cause is not established within one canary period, the warm-eligible
   classes covering the divergent cone degrade to cold via the policy DATA (shrink-of-warmth
   needs no door).
4. **Not permitted**: serving ANY warm hit while RED stands (the IFF is unsatisfied), resuming
   on divergent keys after GREEN returns without the step-2 eviction, or widening the canary's
   comparison tolerance. cold == warm is byte-equality, the same bar as `registry-drift`.

Sequencing consequence: the CAS vertical MUST ship the canary in the same change that enables
fleet-wide warm reads — no canary, no warm. The former pre-CAS stable-key `actions/cache` snapshot
of `buck-out` is **retired by ADR-0554 D10** after repeatable node-pressure eviction during archive
extraction. Required CI is cold until the Buck2-aware remote action cache + CAS is separately
licensed; the canary regime applies at that future bring-up.

### D3 — The owned-stack destination: NativeLink CAS as the warm substrate; cold bypasses it

The warm substrate is **NativeLink CAS + Action Cache** (cache-first, per
`docs/ideas/nativelink-remote-cache-first.md` and ADR-0525 D3's NativeLink recommendation):
Rust-native (rust-purity aligned), content-addressed, owned-stack deployable on the cluster,
keyed-authenticated (founder decision 2026-05-30 — never an anonymous open cache). Cold-required
classes **bypass it entirely**: a cold build does not dial the CAS at all, so a CAS fault can
never widen into the release or canary paths.

The sequence, each stage consuming the previous:

1. **Classification (this ADR)** — the policy DATA exists and is reviewable.
2. **Interim quick-wins respecting it (D5)** — workflow-level reuse inside the existing runner
   substrate; no CAS required.
3. **NativeLink CAS deployment (the queued W3 vertical)** — `[buck2_re_client]` cache-only
   (`remote_cache_enabled=true`, `allow_cache_uploads` per-class from this policy,
   `remote_enabled=false`), shipped WITH the integrity-canary (D2).
4. **Remote execution** — ADR-0525 D3's staged endpoint; flips `remote_enabled=true` only after
   cache-first is proven.

### D4 — Surface model: declarative data + API-driven services; assume shell does not exist

Normative, so the downstream CAS vertical inherits it (founder directives 2026-06-12: *"no cli.
talos style api driven cloud native"* and *"assume shell isn't available. this is true
throughout"*; ADR-0515 D3; ADR-0523 zero-shell posture; ADR-0555 D4 precedent):

1. **The classification is declarative policy DATA consumed by services and controllers** —
   never a CLI tool an operator runs to manage cache state. Nothing in this decision creates an
   operator command surface.
2. **The CAS destination is an API-driven service.** NativeLink natively speaks the Bazel
   Remote-Execution gRPC CAS/AC API — a declarative, API-only cache substrate with no shell/CLI
   operator surface, consistent with Talos's no-shell model. Operations ride console + API per
   the standing `cli_surface_policy`.
3. **The integrity-canary is a scheduled, controller-driven job** (declarative
   workflow/CronJob → reconciled), never a hand-run check. Its result is surfaced via API and
   artifact (check-run/face), not a log someone shells in to read.
4. **Cache-write authorization is enforced at the SERVICE boundary** — CAS API authn/authz
   (keyed writers; untrusted contexts hold no key) — never by client-side discipline. The
   trust-boundary class of D1 is a property the service refuses, not a convention clients
   follow.
5. **Cache diagnostics** (hit-rate, integrity status, canary verdicts) are structured
   telemetry/API responses; **canary-RED remediation** (eviction, quarantine, class degradation)
   is a reconciler/API action.
6. **Transitional bridges are named as such**: the workflow's bash steps this ADR touches or
   references (`buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin`, `infra/ci/install-buck2.sh`,
   `infra/ci/buck2-affected-gate.sh`) are transitional local bridges per `cli_surface_policy`
   whose successors are Rust services/jobs under the G011 zero-shell ratchet (ADR-0523); any
   interim gate-binary modes remain local bridges whose successors are reconcilers per ADR-0555
   D4.
7. **The standing design-review question for every component of this architecture**: *does this
   work when shell does not exist?* If a step's answer is "an operator runs a script", the step
   is misdesigned.

### D5 — Interim quick-wins MUST respect the classification

The classification binds the quick-wins that predate the CAS:

**Warm-safe (same-run trusted reuse — no CAS required):**

- **QW-1 — consume the producer-regen artifact instead of re-materializing per leg.** The
  producer-regen lane already uploads the regenerated faces; the 16 gate matrix legs and the
  cloud-ci-firewall/buck2 lanes may consume that artifact instead of each re-running the
  materializer (~20 materializations → 1). This is `gate-fleet-shared-graph` reuse within a
  single trusted run: the artifact is content-derived from the same candidate tree. **Deliberate
  exception**: `registry-drift` KEEPS its own in-job rematerialization — it is the byte-parity
  detector, and feeding it the artifact it is supposed to verify would make the check
  self-referential. Registry-drift is to faces what the integrity-canary is to the CAS; detectors
  never consume the thing they attest.
- **QW-2 — collapse the cargo-matrix double-build into the buck2 graph.** The buck2 lane already
  builds and tests `//cloud/cloud-ci/...` hermetically; the per-gate cargo legs recompile the
  same workspace once per leg. Consolidating that compilation is `gate-fleet-shared-graph`
  warm-eligible reuse. The cargo→buck2 **required-context content swap stays the ADR-0525
  founder-paired door** — the quick-win is removing redundant compilation, not moving merge
  authority.
- **QW-3 — RETIRED: stable-key `buck-out` `actions/cache` restore.** ADR-0554 D10 removes the
  whole-tree writer and both readers after the 6.37 GB archive repeatedly evicted owned runner pods
  during extraction. No partial `buck-out/v2/cache` salvage is permitted: local materializer state
  is coupled to on-disk outputs. Warm eligibility survives only for the future Buck2-aware remote
  action-cache + CAS contract.
- **QW-4 — toolchain/buck2-binary caching** on ephemeral runners (ADR-0515 D4's
  `actions/cache` item) — warm-eligible; the toolchain is digest-pinned input, not build output.

**Cold-must-stay (no quick-win may touch these):**

- No cache restore step may ever be added to a release/production-image or
  provenance/attestation workflow, nor to the integrity-canary job when it lands.
- Required CI must not archive or restore runner-local `buck-out`; it remains cold until the
  separately licensed Buck2-aware remote cache satisfies D2.
- `registry-drift`'s independent in-job rematerialization stays (above).
- Nothing grants fork-PR/untrusted contexts cache participation.

## Precedent grounding

- **Bazel remote caching + trust model** — shared content-addressed cache; documented guidance
  that only trusted builders write; the canonical deployment is CI-populates/presubmit-reads.
  The cache-poisoning threat model for untrusted authors is taken from this lineage.
- **Bazel stable/volatile workspace status** — deliberately classifying inputs by cache effect
  rather than letting ambient state poison keys; the in-repo instance is ADR-0552's
  stable/volatile scm-facts split.
- **Google TAP/Forge** ("Build in the Cloud" lineage) — content-addressed actions + shared
  cache/execution make presubmit cost track the delta, at monorepo scale; the warm-eligible
  classes are this model applied to oyatie's graph.
- **Meta Buck2 RE** — buck2 is itself Meta's content-addressed RE client; the local/remote
  bit-identity premise of D1's warm table is native to the tool we already run (ADR-0392).
- **SLSA reproducible-build/provenance requirements** — provenance must describe the build that
  actually ran; hermetic/reproducible verification requires re-derivation; grounds the
  `release-production-image` and `provenance-attestation` cold classes (with ADR-0039/ADR-0181).
- **NativeLink production guidance** (and Buildbarn's decomposition precedent) — CAS/AC as an
  API-only gRPC service split from scheduler/workers; grounds D3/D4
  (`docs/ideas/nativelink-remote-cache-first.md`, founder-decided 2026-05-30).

## Alternatives considered

- **Blanket cold forever (status quo)** — rejected: violates ADR-0515 D4; the measured waste of
  Context §1 is the cost, paid on every PR and every agent iteration, with no safety gain over
  classified warmth (cold-required classes stay cold either way).
- **Naive blanket warm (one cache, all readers, all writers)** — rejected: cache poisoning at
  the trust boundary, provenance fabrication on the release path, non-determinism laundering
  everywhere (Context §2).
- **Signed cache entries (the Nix binary-cache model) as the primary trust mechanism** —
  defensible, heavier: per-entry signing + key distribution + verification on every read.
  Writer-authz at the service boundary + content-addressing + the cold canary achieves the same
  poisoning resistance for a closed writer set, and matches the founder's 2026-05-30 keyed-auth
  decision. Signing can layer on later for cross-org distribution without reversing this ADR.
- **Per-PR isolated caches (no sharing)** — rejected: kills the cross-pod/cross-PR hit that is
  the entire point (the idea doc's first assumption-to-validate).
- **Canary as statistical sampling of warm hits** — rejected as the anchor: a sample proves the
  sampled subset; the licensing invariant needs the from-empty full-graph proof. Sampling may
  ADD telemetry between canary runs, but only the cold canary licenses warmth.
- **Trusting "content-addressed ⇒ safe" with no canary** — rejected: content-addressing
  guarantees keying, not hermeticity; a non-hermetic action poisons the cache with
  environment-dependent bytes the keys cannot see. The canary converts the assumption into a
  continuously verified property — without it, warm-by-default is faith.
- **Read-only shared cache for untrusted authors from day one** — deferred, not rejected: reads
  cannot inject, so the relaxation is sound in principle, but default-isolation is cheaper to
  reason about at bring-up and the fork-PR population is currently ~zero. Encoded as a reviewed
  two-way policy edit, explicitly NOT a door.

## Consequences

**Positive.** Warmth decisions stop being per-PR safety debates — the policy answers them; the
W3 CAS vertical inherits a ready classification + trust invariant + surface model instead of
deriving them under deployment pressure; the release/provenance/trust-boundary paths are
structurally outside the cache fabric (a CAS compromise cannot reach a signed artifact); the
quick-wins (QW-1/2/4) are unblocked with their safety already argued; the canary gives
hermeticity regressions a standing mechanical detector (automation-maximalism: staleness and
drift get detectors, not vigilance).

**Negative / cost.** The canary is a recurring full cold build — a deliberate, scheduled price
for the trust anchor (bounded by cadence, not by PR volume); `/specs/cache-warmth-policy.json`
is one more policy surface to keep live (mitigated: it is accounting-registered, canonical-JSON
governed, and its conformance gate is the named successor); fork-PR contributors stay cold/slow
until the read-only relaxation is reviewed (accepted — the contributor population today is the
agent fleet, which is trusted-class).

**Neutral.** This ADR remains classification + invariant + surface model. ADR-0554 D10 independently
retires the live interim `actions/cache` filesystem snapshot; the warm classes here continue to
describe the future Buck2-aware remote-cache posture and become canary-governed only at bring-up.

## Verification

This ADR is Proposed and claims **no live enforcement** (the GATE-4 `advisory_claiming_enforced`
bar): the deliverable is the reviewed classification.

- The policy file `specs/cache-warmth-policy.json` parses, is canonical-JSON governed
  (ADR-0546 — `specs/` is a governed root), and is accounting-registered (owned via
  `specs/OWNERS`, justified by this ADR, reachable via a reviewed
  `specs/reachability-registry.json` registration, ADR-0555 D1/D3).
- The classification's consumers are named in the policy (`consumers`): the interim quick-wins,
  the W3 NativeLink CAS vertical (which MUST ship the D2 canary), and a future
  cache-policy-conformance gate (asserting the CI cache configuration matches the policy —
  e.g. no cache step on a cold-required path), which is the enforcement successor and lands as
  gate-test-plus-policy-DATA per ADR-0555 D4.
- Ledger: FRIC-1781360200 records the friction (cold/warm strategy undefined) and this
  classification as its enforcement fix; the prior diagnosis row
  FRIC-1781070457-buck2-no-shared-cache remains the W3 deployment tracker.

---
*Proposed 2026-06-12 (founder directive: "some things should be cold. some things can be warm.
make that distinction well."). Foundation for: interim CI quick-wins + the NativeLink CAS W3
vertical + eventual RE (ADR-0525 D3). Surface model per the 2026-06-12 Talos-style API-driven +
no-shell directives, ADR-0515 D3, ADR-0523, ADR-0555 D4.*
