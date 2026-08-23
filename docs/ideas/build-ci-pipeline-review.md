# Build + Gate + CI/CD Pipeline — Refined Target Architecture (idea-refine)

> Synthesis of three code analyses (build-system, gate-cicd, ci-plan) + two research
> reports (hyperscaler CI/CD patterns, buck2+reindeer best practices). Read-only review;
> all claims cite `file:line` in `/tmp/buck-psm`. Lens: SIMPLIFY + IDEA-REFINE.

---

## 1. Problem Statement

One sentence: **oyatie's build is correct on darwin and broken on Linux, the gate that
must catch that is itself PR-controllable and parses the whole tree, and the orchestration
that runs the gate is un-lintable Groovy that can deadlock itself.** Five mechanically
distinct faults compound into one recurring "green-on-dev, red-on-CI, can't-diagnose-why"
loop:

1. **darwin/Linux fixup divergence** — the build links via the *host* `/usr/bin/clang`
   (`toolchains/BUCK:22-29`), so dev (macOS/ld64) and CI (Linux/lld) diverge at the link
   layer. Every "works locally, breaks in CI" bug traces here.
2. **reindeer wipes hand-edits** — `reindeer.toml` has **0** `[platform.*]` sections, so
   `reindeer buckify` regenerates `third-party/BUCK` from static TOML and **wipes** every
   hand-added per-OS `select()` (there are exactly **4** in the 19.7k-line file) and
   `$(location …)` DEP env. Corrections survive only as prose warnings.
3. **prelude double-CRT** — the bundled prelude sets `$CC = clang --ld-path=__ld_shim`
   where `__ld_shim` re-invokes clang **as the linker driver**
   (`cargo_buildscript.bzl:303-327`, `:183`), re-adding C-runtime startfiles. Any build
   script that links an executable (aws-lc-sys memcmp probe) gets
   `ld.lld: duplicate symbol _start/_init` on Linux; ld64 tolerates it.
4. **PR-sourced gate** — the Jenkinsfile is trunk-sourced (`values-local.yaml:287`
   `branch('dev')`) but it `git checkout`s the PR sha and runs
   `infra/ci/buck2-affected-gate.sh` **from the PR workspace** (`Jenkinsfile:53,76`), so
   the gate *script* is still PR-controlled — a half-open `pull_request_target`-class hole.
5. **full-closure scale** — a one-line `third-party/BUCK` change owner-expands to ~1689
   targets → `rdeps` closes to ~1919 near-whole-tree targets, built **and** tested serially
   under a hard 60-min timeout (`Jenkinsfile:31`) with no cache tier.
6. **Jenkins fragility** — un-lintable Groovy/CPS (a stray `'''` downed the gate, commit
   `c042aacff`), self-deadlock (gate-config-on-the-gated-branch), cold-pod no-completion,
   ClusterIP un-introspectable, and JCasC live-job-vs-CM staleness.

The throughline: **the build is not hermetic, the third-party layer is not durable, and the
gate is not isolated** — everything else is downstream of those three.

---

## 2. Analysis Summary (current architecture + ranked issues)

**Pipeline today:** GitHub PR webhook → `ci-webhook-gateway` (bespoke Rust, ADR-0374,
HMAC-verified, fail-closed router) → Jenkins genericTrigger (token `ci-gate`) → cpsScm
loads `Jenkinsfile-ci-gate` from `dev` → ephemeral K8s agent checks out PR sha →
`buck2-affected-gate.sh` (`owner()` + `rdeps(//…, %Ss)` @argfile, fail-closed) →
`buck2 build`+`test` affected closure → GitHub commit-status `ci-gate`. The two bespoke
pieces (gateway + gate.sh) are clean and well-tested; **all fragility is in the
Jenkins+Groovy+cpsScm+JCasC orchestration layer**, which ADR-0513 already plans to retire.

| # | Severity | Issue | Locus | Durable fix |
|---|---|---|---|---|
| 1 | **CRITICAL** | Prelude cc-shim double-CRT (`$CC=clang --ld-path=__ld_shim` re-adds startfiles) | `cargo_buildscript.bzl:303-327,:183`; `fixups/aws-lc-sys/fixups.toml:20-30` | Own the linker layer: hermetic toolchain owns full link cmd, or vendor+patch prelude `__ld_shim` to `-nostartfiles` |
| 2 | **CRITICAL** | #95 trunk-sourcing half-open: gate.sh run from PR workspace | `Jenkinsfile:53,76`; `values-local.yaml:287` | Deployed controller spawns Job that clones `dev` (trusted) + PR-ref as data |
| 3 | **HIGH** | reindeer wipes per-OS `select()`/DEP env on every buckify | `reindeer.toml` (0 `[platform]`); `third-party/BUCK` (4 selects); `fixups/openssl/fixups.toml:14-26` | Idempotent post-buckify patch step + CI clean-diff check |
| 4 | **HIGH** | Host system clang/lld (dev≠CI at link layer) | `toolchains/BUCK:22-29`; `.cargo/config.toml` (mold) vs prelude lld | Default to hermetic clang+lld+sysroot cell (#83), `select()` host archive |
| 5 | **HIGH** | One third-party change → ~1919-target build+test, 60-min race, no cache | `buck2-affected-gate.sh:56-96`; `Jenkinsfile:31` | rdeps depth-cap + presubmit/postsubmit tiers + NativeLink CAS |
| 6 | **HIGH** | Self-deadlock + Groovy/CPS parse fragility (un-lintable, silent load failure) | `Jenkinsfile:40-97`; commit `c042aacff` | Deployed controller (no Groovy on the gated branch) |
| 7 | **MEDIUM** | Observability: 230-char status line; PR-comment best-effort/often silent; kubectl-exec to diagnose | `Jenkinsfile:90-94,118-121` | Consume buck2 event-log JSON → structured failure summary; persist logs to S3 |
| 8 | **MEDIUM** | Controller built but NOT wired (only `JenkinsDispatcher` exists) | `dispatch.rs:145-225` (no controller dispatcher; controller on `feat/ci-controller`) | Add `ControllerDispatcher` impl of `PipelineDispatcher` + cutover |
| 9 | **MEDIUM** | Controller inherits scale + log-harvest gaps; 3600s hard deadline turns scale risk into guaranteed red | controller `values.yaml` `gateActiveDeadlineSecs:3600` | Carry scale + log-harvest into controller design before declaring the win |
| 10 | **MEDIUM** | Two CI definitions (cargo-era `oyaCiLane` 16 ctxs vs `ci-gate` 1 ctx) drift | `reported-status-contexts.json`; `oyaCiLane.groovy` | Delete `oyaCiLane` + reconcile registry |

---

## 3. SIMPLIFY — what to remove / collapse

**Be ruthless. Three layers of accidental complexity get deleted, not refactored.**

- **DELETE the entire Jenkins+Groovy+cpsScm+genericTrigger+JCasC stack.** 6 hops, logic
  split across YAML + Groovy-in-YAML + Groovy-in-a-file, none locally lintable
  (`values-local.yaml:251-301`, `Jenkinsfile:40-97`). ADR-0513 already scopes its removal.
  Collapse to: **gateway → deployed controller → K8s Job → status**. Three hops, one
  language (Rust), one config source (the controller binary + the trunk gate.sh).
- **DELETE the second CI definition.** `oyaCiLane.groovy` + its 16 cargo-era contexts
  (`reported-status-contexts.json`) are dead weight beside `ci-gate`. One pipeline, one
  context registry.
- **COLLAPSE the per-crate darwin-hardcoded fixup whack-a-mole.** The recurring
  `DEP_OPENSSL_*`, `DEP_AWS_LC_*`, `LDFLAGS=-nostartfiles` firefight is N symptoms of ONE
  root (host toolchain + non-durable buckify). Replace with: **one hermetic toolchain**
  (kills the OS-divergence class) + **one post-buckify patch step** (makes DEP env durable).
  After both, the per-crate `select()` count should be expressible *generatively*, not
  hand-maintained.
- **COLLAPSE the double-CRT firefight to one locus.** Every `LDFLAGS=-nostartfiles` fixup is
  a per-crate patch of a prelude bug. Fix it once at the linker layer (toolchain or vendored
  prelude `__ld_shim`), then delete the per-crate workarounds.
- **DO NOT build tide + deck + plugins now.** ADR-0513 scopes the full Prow set; the only
  live problem is **gate reliability**. Ship P1 (controller) alone; defer merge-queue, UI,
  ChatOps behind explicit demand. Saying no to four subsystems is the biggest simplification.

**Minimal robust shape:** `GitHub PR → ci-webhook-gateway → ci-controller (deployed) →
K8s Job [clone dev (trusted) + PR-ref as data → trunk gate.sh → buck2 (NativeLink CAS)] →
controller harvests buck2 event-log → GitHub status + structured summary`. One language,
three hops, hermetic toolchain, durable third-party, isolated gate.

---

## 4. Recommended Target Architecture

### (a) Toolchain — hermetic cell is the default; double-CRT fixed at the linker layer

**Decision: make a pinned hermetic clang+lld+sysroot cell the default execution toolchain
(#83), replacing `system_cxx_toolchain(/usr/bin/clang)`.** Pattern is the prelude's Zig
toolchain (`http_archive` + `cmd_script` wrappers + `cxx_toolchain_infos`); download separate
clang archives per host OS, `select()` on `prelude//os:linux` vs `:macos`. This:
- erases dev≠CI link drift (single-bootstrap / zero-drift doctrine);
- gives us ownership of the *full* linker command, which is the clean locus for the
  double-CRT fix.

**Double-CRT fix locus (ranked):**
1. **Best — hermetic toolchain owns the link command:** set the toolchain's `linker_flags`
   so the build-script `__ld_shim` path emits startfiles exactly once. This is durable and
   deletes every per-crate `LDFLAGS=-nostartfiles`.
2. **If the hermetic cell slips — vendor + patch the prelude:** delete `[external_cells]`
   from `.buckconfig` (the file already documents this at line 16-17), copy the prelude to
   `prelude/`, and patch `_make_cc_shim`'s `__ld_shim` to pass `-nostartfiles` to the inner
   clang. Trade-off: manual prelude-upgrade tracking.
3. **Interim (today) — keep per-crate `LDFLAGS=-nostartfiles`** for each `-sys` crate that
   probe-links an executable (only aws-lc-sys confirmed). Cheap, but a denylist that drifts.

> Note on the cloud-intelligence **final rust_binary link** failure (#96): the final link is a
> *single*-driver path (`cxx.bzl:105`, one `-fuse-ld=lld`), **structurally different** from
> the nested build-script `__ld_shim`. Do not assume it is the same double-CRT. Reproduce
> locally on aarch64-linux before attributing; likely a static-crypto duplicate-symbol or the
> mold-vs-lld drift, both of which the **hermetic toolchain** also resolves.

### (b) Third-party DEP propagation — durable across buckify

**Decision: an idempotent post-buckify patch step, CI-enforced.** `reindeer buckify`
regenerates from static TOML and cannot express `select()` or `$(location …)`; the prelude's
`buildscript_run(env=…)` is `attrs.arg()`, which *does* accept both at config time — so the
generated BUCK is the right home, and the gap is purely at codegen.
- Make `reindeer buckify` + re-injection one named target (e.g. `make third-party`).
- Add a CI check: after `buckify` + patch, `git diff third-party/BUCK` must be clean.
- For *simple* per-OS string values, optionally use reindeer platform-conditional fixups
  + `set_reindeer_platforms` in PACKAGE; but the `$(location)` DEP_AWS_LC case still needs
  the patch step, so standardize on the patch as the single mechanism.

This kills two recurring breakage classes (`DEP_OPENSSL_*`, `DEP_AWS_LC_*`) deterministically.

### (c) The gate — trunk-isolated, scope-capped, cache-backed

**Trunk-sourcing (security):** the deployed controller spawns a K8s Job whose command
`git clone --branch dev` (trusted) then fetches the PR ref **as data** and runs the
*dev* copy of `gate.sh origin/dev origin/pr-N`. The gate logic is never read from the PR
workspace — closes the half-open hole (`Jenkinsfile:76`). Add **namespace + NetworkPolicy
isolation** (Prow trusted/untrusted split): the untrusted Job pod cannot reach the
controller endpoint, OpenBao, or the GitHub token (token held only by controller/crier).

**Affected-detection that does NOT build the whole tree:** adopt Meta `btd`'s `depth` cap.
When the owner set originates from `third-party/BUCK` or `toolchains/BUCK`, run a
**depth-limited** `rdeps(//…, %Ss, N)` (N≈3-5) presubmit, and schedule an **unbounded
postsubmit** full-tree run that attributes failures to the culprit commit (SWE-book
presubmit/postsubmit two-tier). This is the only documented mitigation for the 1919-target
timeout that does not require RE capacity first. Split build-vs-test budgets; do not run both
serially under one 60-min cap.

**RE / NativeLink:** deploy **CAS+AC cache-only first** (decided 2026-05-30,
`nativelink-remote-cache-first.md`) — a warm CAS gives a cold pod a near-instant build for
unchanged targets, which is most of the 1919 closure. Measure action-cache hit rate; only if
the uncached remainder still exceeds budget add the scheduler+worker RE tier (NativeLink is
Rust → doctrine-aligned; skip BuildBarn/EngFlow/BuildBuddy).

### (d) Controller vs hardening Jenkins — phased, controller wins

**Decision: cut over to the deployed `ci-controller` (ADR-0513 Phase 1); do not invest
further in Jenkins beyond the minimum to keep dev landable during cutover.** The controller's
kernel/k8s-adapter (on `feat/ci-controller`) is genuinely good: pure no-IO state machine,
trunk-sourcing enforced in the Job command, least-privilege RBAC, terminal-status-always. It
kills failure modes 1 (no Groovy), 2 (deployed = deadlock-proof), 4 (terminal-always +
fail-closed), 5 (fewer hops). **But it is NOT live**: the gateway has only `JenkinsDispatcher`
(`dispatch.rs:145-225`); the `PipelineDispatcher` trait is the clean seam — add a
`ControllerDispatcher` that POSTs the kickoff to the controller's `/gate-run`. **Carry into
the controller design (do not assume solved):** the scale cap from (c), and a **log-harvest**
step (the controller holds `pods/log` RBAC but never reads it — re-introduces the same
"top-level not root-cause" gap PR#25 just fixed for Jenkins). Defer tide/deck/plugins.

**Doctrine reconciliation:** bespoke-over-OSS (controller + gateway are bespoke Rust; adopt
Prow's *shape*, not code); hyperscaler-lens (NativeLink self-hostable Rust, no managed dep;
skip Datadog/EngFlow); single-bootstrap/zero-drift (hermetic toolchain = dev==CI);
pure-Rust (clang/lld/python are build *inputs*, not product deps); Talos (aarch64-linux) +
Apple-Silicon (aarch64-darwin) — the per-OS `select()` archives cover both hosts.

### (e) Observability

Controller consumes buck2's structured **event-log JSON** (`buck-out/v2/log/.../events`) to
extract per-action failure target + first-N stderr lines (durable replacement for the fragile
`grep 'Action failed:'` at `Jenkinsfile:90-94`); crier posts a structured
`{target, error_type, first_stderr}[]` summary (Datadog Code/Platform/Unknown taxonomy,
implemented bespokely); persist `.gate-output.log` to SeaweedFS-S3 with a direct URL
(eliminates `kubectl exec`).

---

## 5. Key Assumptions to Validate (each with a test)

1. **Double-CRT is the build-script `__ld_shim`, and the final-link failure is a different
   mechanism.** *Test:* on aarch64-linux, build aws-lc-sys with/without `-nostartfiles`
   (expect fix) AND build cloud-intelligence-app final link with the per-crate workaround in
   place (expect it still fails → different mechanism).
2. **A hermetic clang+lld+sysroot cell links cleanly on both hosts and removes the per-crate
   workarounds.** *Test:* swap toolchain default, remove all `LDFLAGS=-nostartfiles`, build
   the aws-lc-sys + reqwest closure on both aarch64-linux and aarch64-darwin.
3. **A post-buckify patch makes `reindeer buckify` idempotent.** *Test:* `make third-party`
   twice; `git diff third-party/BUCK` clean both times and `select()`/`$(location)` present.
4. **NativeLink CAS hit-rate alone brings the 1919-target closure under the timeout.** *Test:*
   warm the CAS on dev, change one third-party crate, measure presubmit wall-time + cache hit %.
5. **The controller's Job-command trunk-sourcing resists a PR that rewrites gate.sh.** *Test:*
   open a PR that edits `buck2-affected-gate.sh` to `exit 0`; confirm the controller still runs
   *dev*'s gate.sh and the malicious script never executes.
6. **A depth-capped presubmit + postsubmit two-tier catches the same failures.** *Test:* seed a
   known third-party regression; confirm depth-N presubmit catches it OR postsubmit attributes
   it to the commit within one cycle.

---

## 6. MVP / Sequencing (smallest steps that de-risk the most)

| Step | Task | Why first | De-risks |
|---|---|---|---|
| 1 | **#96** — local aarch64-linux repro of the rust_binary final-link; confirm it is NOT the build-script double-CRT | Unblocks the controller (kube→aws-lc) and stops CI-re-trigger diagnosis | CRITICAL #1, #8/#9 |
| 2 | **#83** — make the hermetic clang+lld+sysroot cell the default; delete per-crate `-nostartfiles` | Single fix erases the OS-divergence class + double-CRT + mold/lld drift | CRITICAL #1, HIGH #4 |
| 3 | **#95** — verify the LIVE Jenkins job in-cluster; pin gate.sh to dev (controller Job-command or checkout dev's infra/ci/ separately) | Closes the active security hole even before full cutover | CRITICAL #2 |
| 4 | **post-buckify patch + CI clean-diff** (durable DEP propagation) | Stops `reindeer buckify` re-breaking the Linux gate silently | HIGH #3 |
| 5 | **#88 P1 only** — add `ControllerDispatcher` (impl `PipelineDispatcher`) + cutover; carry the scale-cap + log-harvest into it | Deletes the whole Jenkins/Groovy fragility class; defer tide/deck/plugins | CRITICAL #2, HIGH #6, MED #7/#8/#9 |
| 6 | **#94-secondary** — rdeps depth-cap for third-party/toolchains changes + postsubmit tier + NativeLink CAS MVP | Makes third-party PRs gateable under the timeout | HIGH #5 |

Steps 1-4 are **unblocked today** and need no controller. Step 5 depends on steps 1-2
(controller is a rust_binary in the same blast radius). Step 6 layers on top.

---

## 7. NOT Doing (and why)

- **NOT building tide (merge-queue) + deck (Leptos UI) + plugins (ChatOps) now.** Only the
  gate is unreliable; these are separately-justified systems folded into ADR-0513 for
  narrative unity. They expand owned surface against narrow-spine/single-bootstrap. Defer
  behind explicit demand. (Trade-off: ADR-0513's "one platform" story lands incrementally,
  not at once — acceptable.)
- **NOT adopting Zuul / BuildBarn / EngFlow / BuildBuddy / Datadog.** Zuul is Python+Ansible+
  Zookeeper; BuildBarn is Go; the rest are managed services. All fail bespoke-over-OSS or
  hyperscaler-lens. Adopt the *shapes* (speculative queue, REAPI split, failure taxonomy) in
  Rust. (Trade-off: more build effort vs. proven off-the-shelf — doctrine-mandated.)
- **NOT keeping the per-crate `LDFLAGS=-nostartfiles` denylist as the destination.** It is a
  drifting denylist patching a prelude bug per-crate; acceptable only as the interim until the
  hermetic toolchain lands. (Trade-off: interim duplication vs. waiting on #83.)
- **NOT splitting third-party/BUCK into per-crate packages yet.** It would collapse the 1919
  fanout structurally, but it is a large mechanical change with its own buckify-durability
  risk; the depth-cap + CAS achieve most of the win at far lower cost first. (Trade-off:
  structural-correct vs. fast-and-reversible — defer structural.)
- **NOT using reindeer platform-conditional fixups as the primary DEP mechanism.** They cannot
  express `$(location …)` and emit reindeer-platform-named selects (not `prelude//os:linux`).
  The post-buckify patch is the single durable mechanism. (Trade-off: external patch step vs.
  native-but-incomplete fixup support.)
- **NOT deploying full NativeLink RE (scheduler+workers) before measuring CAS hit-rate.** RE
  parallelizes but does not reduce total work; cache hits do. Cache-only first. (Trade-off:
  delayed parallelism vs. avoiding premature ops cost.)

---

## 8. Open Questions

1. Is the cloud-intelligence final-link failure (#96) a static-crypto duplicate-symbol, the
   mold-vs-lld drift, or something else? (Blocks the double-CRT vs. final-link split decision.)
2. Hermetic toolchain (#83): is a maintained prebuilt clang+lld+**sysroot** archive available
   for aarch64-linux + aarch64-darwin, or must we build/host it (object-store + bandwidth cost)?
3. After the hermetic toolchain, can the remaining per-OS DEP `select()`s be *generated*
   (eliminating the patch step), or is the post-buckify patch permanent?
4. What depth N for the third-party rdeps cap balances catch-rate vs. timeout, and does the
   postsubmit tide/attribution exist yet to backstop it?
5. Live Jenkins job re-seed: does JCasC re-seed an existing job on restart, or must it be
   manually deleted+recreated? (Determines whether the committed `branch('dev')` is actually live.)
6. Does the cutover need both pipelines running in parallel (double CI cost) during migration,
   or can the controller take over atomically once the dispatcher seam is wired?
