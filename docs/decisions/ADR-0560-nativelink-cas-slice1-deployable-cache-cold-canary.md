---
id: ADR-0560
title: "NativeLink CAS slice 1: deployable cache-only substrate + opt-in buck2 wiring + cold integrity-canary"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-06-12
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0556, ADR-0515, ADR-0525]
amends: []
related: [ADR-0039, ADR-0131, ADR-0181, ADR-0392, ADR-0510, ADR-0523, ADR-0544, ADR-0548, ADR-0552, ADR-0554, ADR-0555, ADR-0558]
related_specs:
  - /specs/cache-warmth-policy.json
  - /specs/cache-warm-license.json
milestone: W3
---

# ADR-0560: NativeLink CAS slice 1 — deployable cache-only substrate + opt-in buck2 wiring + cold integrity-canary

## Status

**Proposed — 2026-06-12.** The deployment decision for the W3 warm-cache vertical, slice 1.
This ADR deploys what ADR-0556 classified; it re-decides nothing ADR-0556 owns. The number was
pre-allocated by the lane leader and verified free against the #703 allocator at lane start
(`--next-adr => ADR-0559`, sibling-reserved; 0560 untaken).

## Context

ADR-0556 is the foundation this slice consumes wholesale (cited, not restated): the
cold/warm classification is policy-as-data at `/specs/cache-warmth-policy.json`; warm is
admissible IFF the class is warm-eligible AND the most recent cold integrity-canary is GREEN
(D2); the warm substrate is NativeLink CAS+AC and cold classes bypass it entirely (D3); the
surface model is declarative data + API-driven services, with cache-write authorization
enforced at the service boundary (D4). ADR-0515 D4 rules the destination (wall-clock tracks
the change, not the repo) and ADR-0525 D3 stages RBE/CAS with NativeLink recommended.

The friction is measured, not hypothetical: buck2 reports 0% cache hits across every fresh
worktree and ephemeral runner (FRIC-1781070457-buck2-no-shared-cache; re-surfaced by the
serial-CI audit FRIC-1781071664; this lane's own allocator run was 190 actions, 0 cached).

**The slice-1 deployment boundary, stated honestly:** no live cluster is reachable from the
CI lanes — `oya-ci-required` runs on GitHub-hosted `ubuntu-latest`, which can never resolve
an in-cluster `*.svc.cluster.local` endpoint, and cluster bring-up itself (Talos apply,
ArgoCD sync, OpenBao seeding, bucket creation) is operational work outside a PR. Slice 1
therefore ships **deployable declarative artifacts + dark opt-in wiring + the live canary
workflow + conformance tests** — and does NOT claim a running deployment. "Deployed" in this
change means: every artifact needed for bring-up is reviewed, byte-ready, and mechanically
conformance-checked; nothing in the live pipeline changes behavior today.

## Decision

### D1 — Declarative deployment artifacts: the `nativelink-cas` tier only

`infra/nativelink/nativelink-cas.k8s.yaml` deploys NativeLink **v1.6.2** (current upstream
release, 2026-07-17) as the cache-only tier of the founder-decided 2026-05-30 three-tier
split (`docs/ideas/nativelink-remote-cache-first.md`): CAS + Action Cache, no scheduler, no
workers. Precedent accuracy, per the hyperscaler lens: NativeLink is **Rust-native**
(rust-purity aligned), speaks the **Bazel Remote-Execution gRPC CAS/AC API** (the wire
protocol Bazel/Buck2/Pants/Reclient already consume), **FSL-1.1-Apache-2.0** (FSL 1.1,
converts to Apache-2.0 ~2028-07-17 for the pinned v1.6.2 release; deployed as a
container, not a linked Rust
dependency), self-hostable; its own production guidance runs CAS and scheduler as
separate processes, the same decomposition Buildbarn ships (bb-storage / bb-scheduler /
bb-worker). It runs as a **container, not a workspace crate** — zero new third-party
Rust dependencies.

License posture note: FSL-1.1-Apache-2.0 sits in ADR-0013's requires-review tier
(AWS-FSL / Sentry-FSL family); this deployment is a conscious, reviewed exception, not
an Apache-2.0 dependency.

Storage: fast tier = bounded node-local filesystem LRU on `emptyDir` (cache-of-cache, safe
to lose); slow tier = **filesystem on a ReadWriteOnce PersistentVolume** (`local-path`, the
standing `infra/seaweedfs` / `infra/registry` pattern), CAS reads/writes wrapped in a
`verify` store (`verify_hash: true` — a blob that does not match its own digest is refused).
The originally intended slow tier was **SeaweedFS S3** at the staged `oya-storage` substrate
(`infra/seaweedfs/`), dedicated `nativelink-cas` bucket, reached through the standard AWS
SDK env chain (`AWS_ENDPOINT_URL`, key pair via ExternalSecret→OpenBao); **validating that
chain against SeaweedFS end-to-end was the named bring-up check**, with filesystem-on-PV as
the documented fallback if the SDK chain disappointed. It disappointed (evidence below), so
the manifest ships the fallback active and the SeaweedFS tier is queued behind a projected
CA bundle. The fallback's cost, stated: the slow tier is pod-local durable state rather than
a shared object store, so the deployment is pinned at one replica until the SeaweedFS
revisit restores horizontal scale-out.

**Bring-up evidence recorded at the v1.6.2 bump (partial — the named check is not closed;
what it guarded is decided).** Running the pinned image against the `cas.json` the manifest
then carried, on aarch64, with **no `AWS_*` env set**, the config parses and store
construction begins, then the process panics: `failure to initialize platform verifier:
General("No CA certificates were loaded from the system")` (hyper-rustls) — the image
carries no system CA bundle. The same config with the `experimental_cloud_object_store`
slow tier removed reaches `Ready, listening on 0.0.0.0:50051` cleanly. So the failure is
specific to the cloud-object-store tier, not the image or the schema.

**That run's open sub-question is now answered: a plaintext `http://` endpoint does *not*
avoid the platform-verifier initialization.** The first run omitted the `AWS_*` environment
entirely, so it was re-run with exactly the **three** variables the Deployment then set on
the pod — `AWS_ENDPOINT_URL=http://seaweedfs-bucket-api…:8333`, `AWS_ACCESS_KEY_ID`,
`AWS_SECRET_ACCESS_KEY`. It panics **identically**. So **within the active
cloud-object-store configuration the panic is unconditional**: the verifier is constructed
during startup store construction, before any request is issued, irrespective of endpoint
scheme or credential presence. Scoped precisely, and no wider — it is a property of that
slow tier, not of the image, which starts clean without it.

**What is not closed.** The named check was *end-to-end validation of the SDK chain against
SeaweedFS*, and this evidence cannot close it: the process dies before it dials, so nothing
about SeaweedFS itself was exercised. What is closed is the branch that check guarded — the
SDK chain disappointed, so the documented fallback is taken, and the end-to-end validation
stays queued behind the CA bundle rather than being claimed.

Consequently **filesystem-on-PV is the indicated slow tier**, not a contingency, and the
SeaweedFS path requires a CA bundle projected into the pod before it can be revisited.

### D2 — Keyed authn at the service boundary (the founder-decided posture, mapped to what OSS NativeLink actually enforces)

Honest capability statement: OSS NativeLink (v1.6.2) has **no per-identity API-key authz**
(re-verified against the v1.5.x/v1.6.x release notes at bump time — no authz feature landed);
what it does enforce at the listener is **mTLS client-certificate verification**
(`client_ca_file`) and a **read-only Action Cache** (`ac.read_only`). Slice 1 builds the
keyed boundary from exactly those primitives:

- **Writer listener :50051** — mTLS against the *writer* client CA; AC read-write. Key
  holders: trusted CI lanes only (same-repo branches admitted into the governance pipeline).
- **Reader listener :50052** — mTLS against the broader *reader* client CA; **AC served
  `read_only: true` at the service boundary**. This is the Bazel/Google RBE seam: AC writes
  are the poisoning vector (mapping action keys to attacker blobs); CAS uploads are
  content-addressed and digest-verified, so a reader identity cannot poison what trusted
  builds resolve.
- **Untrusted authors hold no certificate**: the TLS handshake refuses them — no
  participation (ADR-0556 D1 `untrusted-author-presubmit`), enforced by the service, with
  GitHub's no-secrets-for-forks as runner-side defense in depth, and a NetworkPolicy
  restricting even TCP reach to labeled client pods beneath that.
- All key material lives in OpenBao, projected by ExternalSecret (the standing
  `infra/external-secrets/` pattern); the repo carries zero key bytes. Client-side, buck2's
  OSS RE client supports `tls_client_cert`; the resolver injects it from secret-mounted env
  at emit time and **hard-errors** on a warm mode without it.
- Named follow-up (not slice 1): per-identity authz/audit beyond the two-CA split — an
  authenticating gRPC proxy or upstream NativeLink authz when available.

### D3 — Opt-in buck2 wiring, policy-as-data, dark until licensed

The root `.buckconfig` is untouched. The wiring is: two checked-in overlays
(`infra/ci/buckconfig/warm-cache-{rw,ro}.buckconfig`) that select a new cache-only execution
platform (`toolchains//cache:cache-platform` — mirrors the prelude default, `local_enabled`,
`remote_enabled = False`, adds `remote_cache_enabled`/`allow_cache_uploads` knobs read from
`[oya_cache]`), plus a resolver
(`ci/facade/build-cache-policy`) that maps
`build_class → {bypass | warm-ro | warm-rw}` from `/specs/cache-warmth-policy.json` (single
source — the classification is not duplicated) gated by the
`/specs/cache-warm-license.json` kill-switch, and emits a buck2 argfile. Bypass = an
**empty argfile**: a cold class never dials the CAS (ADR-0556 D3 bypass, not read-only).
Fail-closed everywhere: unlisted class → bypass; unlicensed → bypass; the canary class →
bypass unconditionally (wins over tampered data); warm emission without the mTLS identity →
hard error.

The kill-switch ships `warm_reads_licensed: false` and is the mechanical carrier of the
ADR-0556 D2 IFF clause (b): flipping it true is a reviewed change citing the first GREEN
canary run against the live substrate; flipping it false is the RED response
(suspend-all-warm; shrink needs no door) — today by PR (transitional bridge), by the canary
reconciler successor per ADR-0556 D4.3 when it lands.

### D4 — The cold integrity-canary ships in this same change (ADR-0556 D2: no canary, no warm)

`.github/workflows/cache-integrity-canary.yml`: a scheduled (cron + dispatch), declarative,
**from-empty** build of the pinned target set (policy DATA in the wiring app's
`canary-policy.json`) on a clean ephemeral runner — **no buckconfig overlay, no
`actions/cache` step anywhere in the workflow** (ADR-0556 D5 cold-must-stay). The job then:

1. proves zero cache participation mechanically (`assert-cold` over buck2's structured
   invocation record: `run_action_cache_count == 0`, `run_remote_count == 0`,
   `cache_upload_attempt_count == 0`) — the "genuinely from-empty" claim is asserted, not
   narrated;
2. hashes the build outputs into a digest manifest (sha-256, deterministic dir walking);
3. compares against the warm substrate's entries for the same keys **when licensed**: the
   warm-probe step fetches via the read-only overlay in a separate isolation dir (the cold
   build itself never touches the cache; the probe is the comparator's API-driven
   retrieval, buck2 itself being the REAPI client — no bespoke gRPC client, no new deps);
4. emits a structured verdict artifact: `GREEN` (≥1 compared key, all byte-identical) |
   `RED` (any divergence — job fails; response = flip the kill-switch false + evict
   divergent keys via the reconciler successor; eviction automation is named follow-up) |
   `INACTIVE_NO_ENDPOINT` (the slice-1 dark state — explicitly **not** green, licensing
   nothing) | `UNVERIFIED_EMPTY_OVERLAP` (a manifest with zero shared keys must never
   license — job fails).

Bring-up sequencing note: the first GREEN requires warm entries to exist, so bring-up
populates the CAS first (the `postmerge-dev-trunk` trusted-populator class in an
upload-capable run under the founder-reviewed license flip), then the canary's first GREEN
licenses reads. Canary coverage and cadence semantics are ADR-0556 D2's; the cron is a
tunable.

### D5 — Per-lane cache-hit instrumentation (the audit's missing SLO)

The `oya-ci-required` buck2 lane now writes buck2's structured invocation record and
publishes a per-lane cache-hit artifact (`oya-ci/cache-hit-report/v1`: `cache_hit_rate`,
`run_action_cache_count`, `run_local_count`, `run_remote_count`, upload counters), labeled
with the lane's ADR-0556 build class — parsed from the record JSON, never grepped from
logs. The report and upload path are binding for the required lane: missing reports,
missing/renamed counters, malformed records, and 0%-hit warm runs are CI RED rather than
advisory noise. Today the lane remains `bypass` under the dark-license posture; after
bring-up it is the warm-substrate's primary SLO feed.

### D6 — Zero impact on current CI green-ness (the dark-wiring guarantee, mechanically held)

The conformance gate (`oya-cloud-ci-cache-wiring-app-gate`, riding the binding
`buck2 test //cloud/cloud-ci/...`) asserts on every PR: while the license is false, **every**
build class resolves bypass; the four ADR-0556 one-way cold classes resolve bypass even
under a licensed fixture (pinned as a ratchet — removing one from the policy DATA is RED
and requires superseding ADR-0556); the overlays parse, select the cache platform, claim
the posture their names promise, and contain no identity material; the root `.buckconfig`
carries no `[buck2_re_client]`/`[oya_cache]` section and keeps the prelude default
platform; the canary workflow exists, is scheduled, restores no cache, and wires the cold
proof. No lane passes an overlay today, so no behavior changes today.

## Alternatives considered

- **Wait for cluster bring-up and ship everything live in one change** — rejected: couples
  reviewed declarative artifacts to operational cluster work, inflating both risk and
  latency; the canary + conformance lattice is exactly what makes the later bring-up a
  data-flip rather than a design event.
- **Anonymous in-cluster cache (sccache-bucket style)** — rejected by the founder
  2026-05-30 (open-cache footgun); keyed mTLS posture above.
- **API-key header authn** — OSS NativeLink does not enforce key validation
  (`experimental_identity_header` is identity propagation, not authn); claiming "keyed
  auth" on it would be security theater. mTLS is what the service actually refuses on.
- **Bespoke gRPC comparator client for the canary** — rejected: buck2 IS the REAPI client;
  a read-only probe build in an isolated dir retrieves the warm entries without new
  third-party deps (tonic/prost) or a parallel client to keep correct.
- **Single shared listener with one CA** — rejected: collapses the writer/reader seam; AC
  write capability would extend to every reader identity, recreating the poisoning surface
  ADR-0556 D1 prohibits.

## Consequences

**Positive.** The W3 vertical's design risk is retired in review rather than at bring-up;
the trust lattice (canary + kill-switch + conformance ratchet) exists and runs before any
warmth, so the first warm hit is born governed; the 0%-hit baseline is now measured per-lane
(SLO feed); untrusted-author non-participation is a service-boundary property with a
mechanical client-side proof.

**Negative / cost.** The canary spends a daily full cold build while INACTIVE (deliberate:
it exercises the from-empty machinery and the cold price is the ADR-0556 D2 anchor's cost);
two overlays + a license file are new governed surfaces (mitigated: conformance-gated,
canonical-JSON governed, accounting-registered); the SeaweedFS SDK-env endpoint chain never
reached validation — the store panics before it dials — so the documented fallback is what
ships, and the slow tier is a single-node PersistentVolume rather than the shared bucket
(horizontal scale-out and cross-node reuse both wait on the CA-bundle revisit).

**Queued (the honest remainder, tracked):** cluster bring-up (Talos/ArgoCD apply, OpenBao
seeding, PV provisioning, probe-path verification) stays on
FRIC-1781070457-buck2-no-shared-cache; the SeaweedFS slow-tier revisit — CA bundle projected
into the pod, then bucket provisioning and the end-to-end SDK-chain validation that the
panic pre-empted, then shared-tier scale-out; divergent-key eviction
reconciler (ADR-0556 D4.3); owned in-cluster runners or a reviewed endpoint exposure before
any lane can reach the CAS (ADR-0515 D5); per-identity authz beyond the two-CA split.

## Artifact accounting (ADR-0555 — every new file owned + justified)

This decision is the justification anchor for every artifact the slice introduces:

- Deployment: `infra/nativelink/nativelink-cas.k8s.yaml`, ownership seed
  `infra/nativelink/OWNERS` (cloud-ci-platform).
- Opt-in overlays: `infra/ci/buckconfig/warm-cache-rw.buckconfig`,
  `infra/ci/buckconfig/warm-cache-ro.buckconfig`, ownership seed
  `infra/ci/buckconfig/OWNERS`.
- Execution platform: `toolchains/cache/defs.bzl`, `toolchains/cache/BUCK`,
  ownership seed `toolchains/cache/OWNERS`.
- Kill-switch DATA: `specs/cache-warm-license.json`.
- Wiring app (gate matrix + binding buck2 lane):
  `ci/facade/build-cache-policy/Cargo.toml`,
  `ci/facade/build-cache-policy/BUCK`,
  `ci/facade/build-cache-policy/src/lib.rs`,
  `ci/facade/build-cache-policy/src/main.rs`,
  `ci/facade/build-cache-policy/src/canary-policy.json`,
  `ci/facade/build-cache-policy/tests/cache_conformance.rs`.
- Canary: `.github/workflows/cache-integrity-canary.yml`, with the workflows tree's
  ownership seed `.github/workflows/OWNERS` added so workflow files are born owned.

## Verification

Slice-1 bar: deployable artifacts + dark wiring + live canary machinery, mechanically
conformance-checked; no live-enforcement claim beyond the conformance gate itself.

- `buck2 test //cloud/cloud-ci/...` green, including the new
  `oya-cloud-ci-cache-wiring-app` unittest (22 fixtures: resolver fail-closed lattice,
  argfile emission, record parsing, verdict states incl. RED and empty-overlap refusal)
  and the live-corpus conformance gate (dark-wiring proof, one-way cold floor ratchet,
  kill-switch flip semantics, overlay posture, clean root config, canary workflow shape).
- Local toolkit evidence (lane scratch `toolkit-evidence.txt`): `resolve --require-bypass`
  on the canary class; a warm class resolving bypass with an **empty argfile** while
  unlicensed; `assert-cold` green on a real cold record; `canary-verdict` emitting
  `INACTIVE_NO_ENDPOINT` (exit 0, licensing nothing) and `RED` (exit 1) on a tampered warm
  fixture; the structured cache-hit report from a real buck2 invocation record.
- Overlay reality check: `buck2 audit config --config-file warm-cache-rw.buckconfig` shows
  the RE client + `[oya_cache]` keys; `buck2 audit execution-platform-resolution` under the
  overlay resolves `toolchains//cache:cache-platform` (lane scratch
  `overlay-evidence.txt`). Without the overlay, resolution stays
  `prelude//platforms:default` (conformance-gated).
- Manifests: 7 documents parse; the embedded `cas.json` is valid JSON with the reader AC
  `read_only: true`; structural k8s schema validation against a live API server is a
  bring-up check (no kubeconform in the toolchain today — disclosed).
- Ledger: FRIC-1781450000 (this slice; lineage FRIC-1781071664 → FRIC-1781070457) with
  disposition fixed-in-PR for the wiring+canary+instrumentation scope; the cluster gap
  stays queued on FRIC-1781070457.

---
*Proposed 2026-06-12. Slice 1 of the W3 warm-cache vertical: ADR-0556 classification made
deployable. Surface model per ADR-0556 D4 (declarative data + API-driven services; CLI
bridges transitional per `cli_surface_policy`); accounting per ADR-0555; pack-shape per
ADR-0548 R0 (classes, license, pinned targets, endpoints all DATA).*
