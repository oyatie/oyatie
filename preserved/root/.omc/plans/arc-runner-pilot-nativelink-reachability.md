# Reaching the CAS — fix the client, then the anchor, then measure

**Status: PENDING APPROVAL.** Planning artifact, revision 3. No mutation performed.
Rev 2 was **REJECTED** by the Critic pass; rev 3 addresses those findings. **Rev 3 has not
itself been re-reviewed** — do not read it as consensus-approved.

## The through-line

Four separate mechanisms in this cache subsystem report success without the cache doing
anything. Three are verified, one is suspected. That is not four bugs — it is one property:
**every success signal in this path is client-side, and none of them observe the server.**

| # | Mechanism | What it reports | What is actually true |
|---|---|---|---|
| 1 | `kubectl port-forward` + buck2 | `Cache hits: 100%` | server logged `tls handshake eof`; nothing served |
| 2 | `canary_verdict` | `GREEN` | compares local rebuild digests; cache need never be dialed |
| 3 | `argfile_lines` | emits "warm" config | buck2 **ignores** it; the lane runs cold |
| 4 | `toolchains/cache/BUCK` (suspected) | cache platform selected | platform may activate while RE addresses silently do not |

Any plan here that adds a step before fixing this class is building on instruments that lie.

## Finding A — the resolver emits a mechanism buck2 cannot read (blocks everything)

`ci/facade/build-cache-policy/src/lib.rs:216-250`, `argfile_lines`, emits:

```
--config-file  infra/ci/buckconfig/warm-cache-rw.buckconfig
--config       buck2_re_client.tls_client_cert=<path>
```

buck2 resolves `[buck2_re_client]` into `DaemonStartupConfig` **from project config files
only**. Both emitted forms are inert for that section. Measured on this repo against a live
CAS (GH #1245, 2026-07-26), same content, fresh daemon each run:

| Mechanism | Result |
|---|---|
| `--config-file overlay.buckconfig` | `BUILD FAILED` — `No engine address` |
| `--config buck2_re_client.engine_address=…` | `BUILD FAILED` — `No engine address` |
| same content in `.buckconfig.local` | `BUILD SUCCEEDED` — `RE Session: …`, `Network: Up 30KiB` |

**Every warm path in the repo routes through `argfile_lines`.** The canary's warm probe uses
both broken forms directly (`cache-integrity-canary.yml:116`).

Compounding: **`default_allow_cache_upload` appears nowhere on `origin/dev`** (verified,
zero hits). `warm-cache-rw.buckconfig` sets `[oya_cache] allow_cache_uploads = true`, which
`toolchains/cache/BUCK` reads via `read_root_config` into the *platform* — a different knob.
So even with addresses resolving, uploads would not happen.

**Fix:** change the application mechanism to materialize the selected overlay into
`.buckconfig.local` (or `.buckconfig.d/`) and remove it afterward; add
`[buck2] default_allow_cache_upload = true` to the rw overlay. Two consequences the fix must
carry: `.buckconfig.local` is **not** in `.gitignore` today (only `/buck-out/`), so
materializing dirties the tree; and `ci/facade/build-cache-policy/tests/cache_conformance.rs`
currently asserts the **argfile shape** — it asserts the broken thing and must change with it.

Note this is *also* why rev 2's "supersede D4" argument collapsed: its second supporting fact
was "the control-plane seam is already used." It never worked, and
`git log -S'"warm_reads_licensed": true'` returns nothing — the probe step has **never
executed once**.

## Finding B — the trust anchor's GREEN does not require a cache hit

`canary_verdict` returns GREEN when `compared >= 1` and no divergence. Keys come from
`digest_manifest_from_show_output` (`lib.rs:499-506`), which splits `--show-full-output`
lines into `(target-label, path)` and hashes the **local file on disk**. The warm probe runs
with no `--unstable-write-invocation-record` and no `assert-warm`.

A probe that fetches zero blobs and rebuilds locally yields identical digests, full label
overlap, zero divergence → **GREEN**. `UNVERIFIED_EMPTY_OVERLAP` cannot catch it: it fires on
zero *label* overlap, and both runs build the same pinned target set by construction.

**Fix:** write an invocation record on the probe and gate the warm manifest's admissibility
on `assert_warm_cache_participation` (`lib.rs:321`, already tested by
`assert_warm_fails_on_zero_hit_warm_mode`, already invoked by the required lane).

**And ratchet it.** `cache_conformance.rs:245-277` asserts the canary's cron, no-cache-restore,
invocation record, `assert-cold`, and `canary-verdict` — **nothing about the warm probe's
guard.** Landing the YAML fix without a matching assertion lets the same false GREEN return
on any future edit. Per repo doctrine the recurrence-prevention *is* the deliverable.

## Finding C — `assert-warm` does not check bytes

`assert_warm_cache_participation` checks exactly `cache_hit_rate != 0`,
`run_action_cache_count != 0`, `last_snapshot.re_action_cache_started != 0`. **No
bytes-fetched check.** Rev 2's criterion "cache_hit_rate > 0 with bytes fetched, asserted by
`assert-warm`" attributed an assertion the tool does not make. It matters: #1245 measured
`Down 8.3MiB` on a run with **0%** hits — bytes move without hits, and hits are asserted
without bytes.

## Finding D — the license has fewer dimensions than the substrate

`resolve(policy, license, build_class)` takes **no platform**, and
`specs/cache-warm-license.json` carries one global `warm_reads_licensed` — while
`toolchains/cache/BUCK` derives the platform from `host_configuration.cpu`/`.os`, giving three
disjoint namespaces. A license with fewer dimensions than the substrate will eventually
license something it never measured.

**This also supplies the correct fix to the populate/license cycle.** Rev 2 proposed
superseding ADR-0560 D4's reviewed flip and populating outside the resolver. The Critic was
right that this creates a second write path invisible to the D6 conformance ratchet, with no
governed record of who populated the bytes the first GREEN will bless — the divergent second
reality rev 2 rejected elsewhere. And the license file's own `_comment` defines the flag as
carrying **both** reachability and canary-GREEN, so "the workflow conflates them" was a
misreading of a documented choice.

**Amend, don't supersede:** add a second DATA field (`populate_window`) distinct from
`warm_reads_licensed`. The resolver stays the single authority, the change is reviewable and
revertible, and it is the same "add a dimension" move Finding D already argues for.

## Finding E — ARC has no ADR, and a stale competing chart exists

Zero ARC references on `origin/dev`. ADR-0515 D5's destination is the bespoke Rust
`cloud/cloud-ci` product and a runner-*swap*; it says nothing about runners-as-pods or
GitHub's Go operator. Meanwhile `oya/governance/iac/helm/lane-runner-pool/` pins *legacy*
`actions-runner-controller` 0.23.x against `microservices/governance/…` paths, has never run,
and its NetworkPolicy egress list has no NativeLink host.

**This is a founder decision gate, not a "not doing" bullet.** The original ask was "deploy
ARC." The honest answer is that ARC is a new third-party substrate adoption needing its own
ADR and a selection-bar pass, and that it must supersede or delete `lane-runner-pool`.
Deferring it is a recommendation, not a refusal — it goes to the founder before any step
that assumes it.

## Corrected step order

Steps 1–3 are prerequisites; nothing downstream can be measured until they land.

| | Step | Note |
|---|---|---|
| 1 | **Fix the application mechanism** (Finding A): materialize overlay → `.buckconfig.local`; add `default_allow_cache_upload`; gitignore; update `cache_conformance.rs`. | blocks all |
| 2 | **Repair the anchor** (Finding B): invocation record + `assert-warm` gate on the probe, **plus** the conformance ratchet. | blocks licensing |
| 3 | **Amend the license DATA** (Finding D): `populate_window` as a field distinct from `warm_reads_licensed`. | replaces "supersede D4" |
| 4 | **Pilot — one reachable lane**, target chosen per the siting question below. | the measurement |
| 5 | **ARC**, only if the founder gate says so, with its own ADR. | out of the critical path |

Steps 1–3 all edit `run:` blocks or governed DATA and therefore hit
`rust_first_automation_unbaselined_workflow_inline_shell` (shrink-only baseline) and the
canonical-JSON/accounting gates. Rev 2 applied that blocker only to a hypothetical new
workflow; it applies to steps 1 and 2 as well, and the "~10 lines of YAML" sizing was wrong.

## Siting: the tunnel is probably the wrong door too

Rev 2 favored routing an L4 Cloudflare tunnel to the CAS. The Critic's adversarial pass found
that option was measured with a shorter ruler than the ARC option it replaced:

- It requires **three undeclared control-plane mutations** — a remotely-managed tunnel ingress
  rule (not in `infra/cloudflare/main.tf`, which manages a *different* tunnel), a proxied DNS
  CNAME, and a Cloudflare Access app + service token (`access.tf` has none for
  `k8s.oyatie.dev`) — none with committed declarative source, against `zero imperative ops`.
- It puts **proprietary SaaS in the CI cache data path** with no selection-bar pass, while the
  plan rejects ARC partly for lacking one.
- It requires labeling the **shared multi-tenant cloudflared pod** `oya.io/nativelink-cas-writer`
  — the NetworkPolicy `podSelector` matches the *source* pod in any namespace — granting CAS
  L3 reach to everything else that tunnel carries, collapsing the layer ADR-0560 D2 calls
  "defense in depth UNDER the mTLS boundary."
- It exports the **writer** certificate — the AC-poisoning-capable identity — out of the
  cluster into GitHub Actions secrets, which ADR-0560 D2 scopes to "trusted CI lanes only."
- Its backing cluster is the same NAT'd laptop (ADR-0371: apiserver VIP `10.211.55.240`,
  Parallels, no public IP), so rev 2's claimed durability advantage over the arm64 pilot
  **does not exist**.

**Better candidate: the OCI Talos cluster** (`140.245.68.253`) already has a public IP and
needs no tunnel, no Access app, no DNS record, and no shared-connector labeling. Caveats to
respect: it is **shared** and **dev-DB-bearing**, so any work there is non-destructive only,
and its credentials live in **OCI Vault, never `/tmp`** (this cluster was bricked once
already). Siting is Open Question 1 and should be settled before step 4.

## De-brand / path=namespace — applies here, as a SEPARATE PR

`ci/facade/build-cache-policy/` is already capability-first and already de-branded:
package `ci-build-cache-policy`, `rust_library` `ci-build-cache-policy`, `rust_test`
`ci-build-cache-policy-{unittest,gate}`. **One straggler:**

```
rust_binary(name = "oya-cloud-ci-cache-wiring-bin")
```

Three defects in one target: brand prefix (`oya-`), path doubling (`cloud-ci` when the path
already says `ci/`), and a stale concept name (`cache-wiring` vs the directory's
`build-cache-policy`). It is the target every call site invokes — 6 in
`cache-integrity-canary.yml`, 2 in `oya-ci-required.yml:567,569`. Repo-wide, **17 of 53 `ci/`
crates still carry `oya-`** (~68% de-branded).

This *is* the hyperscaler monorepo pattern, not a cosmetic preference: in Bazel/buck2 the
label's package path is the namespace (`//search/query:query`, `//fbcode/folly:folly`). A
prefix carried by 100% of targets has zero information content; at 32% it encodes only "was
this migrated yet," which is drift.

Two drifts fall out: **ADR-0560 D3** and the **`warm-cache-rw.buckconfig` header** both cite
`//cloud/cloud-ci/gates/oya-cloud-ci-cache-wiring-app:oya-cloud-ci-cache-wiring-bin` — a path
that no longer exists.

**Sequencing (binding).** The rename is a pure mechanical MOVE; the Finding-A fix is a
REFACTOR. Different dispositions do not share a PR, and the reorg is frozen at 45% with
serial one-move-per-PR. Land the rename **first** as its own no-behavior PR
(`ci-build-cache-policy-bin`) so the mechanism fix never mentions the stale name. This is
"touching it anyway," not a new reorg batch — but it is still its own PR, and it needs the
catalog row a codemod never authors.

## Acceptance criteria (replacing rev 2's)

Rev 2's "uploads > 0 and blobs land on the PVC" is **deleted** — #1245 already observed
uploads succeeding (`Up 4.7MiB`) alongside `0%` read hits. Uploads are never evidence of a
working cache. The only non-vacuous criterion is **paired**:

1. **Populate**, then **fresh daemon + deleted `buck-out` + fresh isolation dir** read.
2. Read run: `run_action_cache_count > 0` **and** `last_snapshot.re_download_bytes > 0`
   **and** `run_local_count` materially below the populate run's. (The bytes clause needs
   `assert-warm` extended per Finding C, or a separate assertion — it is not there today.)
3. `run_remote_count == 0` — cache-only, no RE.
4. **Wall-clock strictly below the cold baseline for the same target set.** Rev 2 had no
   time criterion at all, while citing "40.5 min vs 6.6 s" as its driver. A remote cache
   slower than a local rebuild is a real outcome, and every other criterion passes in that
   world.
5. Server-side corroboration. Two obstacles rev 2 ignored: the manifest sets
   `RUST_LOG: "warn"`, so successful handshakes are not logged; and the pod has no shell,
   `readOnlyRootFilesystem: true`, so `kubectl exec ls /data` is impossible — PVC inspection
   needs a debug pod co-scheduled onto the RWO volume's node. Name the method or drop the
   criterion.

## Blockers

**#0 — the application mechanism (Finding A).** Everything else is downstream.

**#1 — NetworkPolicy source-pod labels.** `oya.io/nativelink-cas-{writer,reader}: "true"` on
the *client* pod, any namespace. Refusal is at **L3** → presents as a timeout, not a TLS
error. Applies to **both** pilots; rev 2 wrongly exempted the tunnel path.

**#2 — client cert material does not exist on any runner.**
`cache-integrity-canary.yml:113` sets `OYA_CACHE_TLS_CLIENT_CERT` to a *path*, and no step
writes a cert to disk. There is no private-key secret at all. Both pilots need cert+key
materialized from OpenBao.

**#3 — certificate SAN.** Certs are for `*.svc.cluster.local`; any out-of-cluster client
connects by another name. Resolutions differ by an order of magnitude — a buck2 RE-client
server-name override (**existence unverified**), `/etc/hosts` on the runner (collides with
blocker #5), or re-issuing the server cert with a public SAN (an OpenBao PKI role change,
since the cert arrives via `ExternalSecret nativelink-cas-tls`).

**#4 — readiness probe path unverified.** `httpGet /status` on :50061 against a config
declaring `{"health": {}, "admin": {}}`. Wrong path → no endpoints → connection-refused,
indistinguishable from a routing fault.

**#5 — `gate · rust-first automation hygiene`** ratchets workflow inline shell shrink-only.
Applies to steps 1 and 2, not just to any new workflow.

**#6 — ESO + OpenBao** if the pilot runs in-cluster. Governed path already committed
(`clustersecretstore-openbao-oya.yaml`, the auth-delegator binding, `infra/kms/openbao.k8s.yaml`,
RUNBOOK). The RUNBOOK's lead invariant is the whole lesson: `auth/kubernetes/config` must set
`disable_local_ca_jwt=true` or every ExternalSecret 403s on TokenReview. Note the cluster is
ArgoCD-managed — `kubectl apply` is an imperative deviation that must be declared, not
narrated as the governed path.

**#7 — aarch64 buck2**, only if piloting on arm64. Verified obtainable: pinned `2026-07-15`
release publishes `buck2-aarch64-unknown-linux-gnu.zst`, sha256
`e239bf72f40a7987db9024eb6d5e325642f6496c589dec6be54c1008d2618a19` (37,190,010 bytes). The
`OYA_CI_ALLOW_AMBIENT_BUCK2=1` hatch is adequate for a reachability probe and **must not** be
used for a populate — an unpinned buck2 writing into a shared CAS breaks the digest-pin
discipline that makes the substrate admissible.

**NOT a blocker — PodSecurity.** The governed CAS pod spec is already `restricted`-compliant.

## Still open

1. **Where does the pilot run?** OCI (public IP, shared, dev-DB-bearing, non-destructive
   only) vs the laptop Talos lab vs the tunnel. Settle before step 4.
2. **Does buck2's RE client expose a TLS server-name override?** Decides blocker #3's cost.
   Not establishable from repo contents.
3. **Does `read_root_config("oya_cache", …)` honor `--config-file` even though
   `[buck2_re_client]` does not?** If yes, the overlays are in the worst state: cache platform
   activates while RE addresses silently do not — a partial-application failure nobody has
   characterized. This is false-green candidate #4.
4. **Is ARC wanted at all** — founder gate (Finding E).
5. **Bandwidth/latency budget.** No model exists. Criterion 4 is the backstop, not a substitute.
6. **Teardown.** No revert path is specified for any pilot artifact.

## What survives from earlier revisions

Verified against `origin/dev` and unchanged: `oya-ci-required.yml:566` hardcodes
`CACHE_MODE=bypass` and never calls `resolve`; `install-buck2.sh` is `Linux-x86_64`-only;
ADR-0515 D5's actual content; the governed-vs-hand-deployed CAS divergence (my running
StatefulSet uses the wrong Service names, one listener, an unpinned `v1.6.3`, and a
single-CA hand-made Secret, so the overlays cannot resolve it); the pinned `v1.6.2` digest is
a multi-arch index including `linux/arm64`; and the principle that no required lane may
depend on a laptop VM.
