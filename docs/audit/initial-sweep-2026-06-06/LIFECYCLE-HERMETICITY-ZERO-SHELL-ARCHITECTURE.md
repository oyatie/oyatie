# LIFECYCLE-WIDE HERMETICITY + ZERO-SHELL "IT JUST WORKS" — ARCHITECTURE

- STATUS: pending-approval
- DOOR: one-way (founder sign-off required before any mutation)
- AUTHORED: 2026-06-08
- AUTHORITY: founder apex directive — "hermeticity should apply to build, ci, cd, dev environment. 'it just works' approach with 0 to minimal shell scripts or cli." + standing doctrine (hermetic-just-works: clean checkout builds+runs reproducibly, NO external/prebuilt blobs, NO manual steps, CI/firewall-enforced) + "git is transitional."
- SCOPE: design only. READ-ONLY against BOTH `/Users/jasonlee/Developer/source` (cloud/oya-ci) and `/Users/jasonlee/Developer/linux` (kernel port). This doc mutates nothing, runs no git ops. Other agents are actively building these trees.
- SIBLING DOCS (referenced, NOT duplicated):
  - `OYA-CI-HERMETIC-EXECUTION-DESIGN.md` — the git-facts boundary decision (Option C: committed `git-facts.generated.json` face) + buck2-native hermetic producer/gates. This doc CONSUMES that decision; it does not re-litigate it.
  - `OYA-CI-VCS-AGNOSTIC-SEAM-REFINEMENT-PLAN.md` — the `git→scm-facts` rename + `ScmFactsSource` trait. This doc references the boundary; the rename is OWNED there.
  - `CICD-DESIGN-PLAN.md` (APPROVED) — ONE CANONICAL CI, `oya-ci-required` fan-in, runner-migration-over-time. This doc REFINES the "one logic, many runners" seam toward zero-shell; it does not change the canon.
  - `DEV-CUTOVER-PLAN.md` — the producer-rooted Buck2 tree cutover to `dev`.

---

## 0. HEADLINE (the design in one screen)

**The directive decomposes into one invariant per lifecycle stage, all satisfied by the SAME buck2 target graph:**

> **One graph, four runners.** `buck2 build //...` and `buck2 test //...` are the SINGLE source of truth for what gets built and verified. BUILD (dev), CI, CD, and DEV-ENV are four *runners* of that one graph. The only per-runner difference is a THIN, ideally GENERATED adapter (a forge YAML, a control-plane Job spec, a `.cargo`/`.envrc` shim). Every `run:`/`sh`/Makefile target that does real work is a defect to retire into a buck2 target; the residue is an explicitly-justified, pinned **irreducible-glue ledger** (§D).

**Where we are today (verified):**
- **source/ (cloud):** real buck2 substrate exists — `.buckconfig` (5 cells), `reindeer.toml`, `toolchains/BUCK` (system rust/cxx/python + `noop_test_toolchain` + `remote_test_execution`), **763 BUCK files** (non-buck-out). The `oya-cloud-ci/...` gate logic already builds+tests under buck2 (`oya-ci-required.yml:213-217`). BUT: the canonical CI still has **6 cargo lanes** (`oya-ci-required.yml:78-139`) running ALONGSIDE the one buck2 lane; the buck2 lane itself carries **5 inline `run:` shell blocks** (toolchain curl-install, git-facts-regen, build+test, affected-driver, cache); two other workflows (`backbone-microservices-ci.yml`, `docs-graph-drift.yml`) are **100% cargo**; CD is shell+Makefile+Helm (`Makefile`, `infra/talos/*.sh`, `infra/capi/init.sh`); dev-env is direnv + a hand-PATH `bin/oya` shim (`.envrc`).
- **linux/ (kernel):** **ZERO non-buck-out BUCK files under `stack/kernel`** — the kernel is **100% cargo + shell** today. Only `toolchains/BUCK` (a `system_demo_toolchains()` shim, `toolchains/BUCK:6`) and a root `.buckconfig` exist; buck2 currently builds only the standalone idiomatic primitive crates, NOT the kernel. The kernel build/boot/test path is entirely shell: `build-carriers.sh`, 13 per-program `build.sh`, `run-qemu-{x86_64,aarch64}.sh` wired as the cargo `runner` (`stack/kernel/.cargo/config.toml:25,44`), `conformance-probe.sh`, `diff-oracle.sh`, etc. The "hermetic Stage A/B" landing made the *carrier producer* reproducible (no Docker, bundled `rust-lld`) but it is still a **bash orchestrator over `cargo build`**, not a buck2 target.

**The asymmetry is the headline finding:** the cloud side is ~70% of the way to buck2-native (graph exists, gates build under buck2, the hard git-facts boundary is already designed in the sibling doc). The kernel side is ~5% — it has a buck2 cell but no kernel targets at all. **The largest single piece of net-new work is buckifying the kernel build+boot+test graph.** CD is net-new on both sides (no buck2 deploy primitive exists anywhere).

---

## A. SHELL / CLI / GLUE INVENTORY

Legend — Hermetic today? Y = reproducible from clean clone w/ pinned inputs, no ambient state; N = depends on ambient host state / network / `.git` / unpinned tool; P = partial.
(`buck-out/**/*.sh` are buck2-GENERATED artifacts — NOT source — and are excluded.)

### A.1 source/ (cloud / oya-ci) — CI + BUILD

| Path | What it does | Stage | Consumes → Produces | Hermetic? |
|---|---|---|---|---|
| `.github/workflows/oya-ci-required.yml` | THE canonical blocking CI. 6 cargo gate lanes + 1 buck2 lane + fan-in. `run:` steps: `cargo test -p <gate>` ×7 (lines 100,120,139), `cargo run` regen (56,119) | CI | repo + `.git` (fetch-depth:0) → check-runs | P — cargo lanes ambient (read working tree + `.git`); buck2 lane scoped-hermetic but the lane itself has 5 inline shells |
| `oya-ci-required.yml:164-173` (Install buck2) | `curl` a pinned `BUCK2_RELEASE=2025-05-06` zst from GitHub releases → `/usr/local/bin/buck2` | CI/BUILD | network → buck2 binary | P — version-pinned (good) but a `run: curl` (network at CI time; not a declared toolchain input) |
| `oya-ci-required.yml:189-201` (git-facts-regen) | `buck2 build` the emitter, run it, `diff` committed vs regenerated git-facts face | CI | `.git` → byte-validation | N by nature — this is the ONE intentional ambient git boundary (see §D, sibling HERMETIC-EXECUTION doc) |
| `oya-ci-required.yml:213-217` (binding gate) | `buck2 build //cloud/cloud-ci/...` + `buck2 test //cloud/cloud-ci/...` | CI/BUILD | buck2 graph → verdicts | Y — fully hermetic (the target state, already live) |
| `infra/ci/buck2-affected-gate.sh` (123 ln) | affected-set driver: `git diff` → `buck2 uquery owner()` → `rdeps(//..., %Ss)` → build+test closure. Fails CLOSED on unbuilt Rust change | CI/BUILD | `.git` diff + buck2 graph → affected targets | P — buck2 logic is hermetic; the git-diff + classification is ambient bash. Currently ADVISORY (`continue-on-error`, line 219) due to a stale worktree BUCK pkg |
| `scripts/ci/regen-third-party.sh` (40+ ln) | `reindeer buckify` THEN re-apply `third-party-buckify-handedits.patch` (per-OS `select()`s reindeer can't emit) | BUILD | `Cargo.lock` + fixups + patch → `third-party/BUCK` | P — deterministic given pinned reindeer + patch, but `reindeer` is unpinned-on-PATH and the patch can context-drift |
| `scripts/ci/oya-ci-post.sh` (5 contexts) | local→GitHub status bridge: runs `./bin/oya verify --ci-required`, posts 5 commit-status contexts | CI (transitional) | `gh` + local cargo gates → GH statuses | N — transitional Jenkins/webhook-gap bridge (ADR-0387); pure git/forge coupling |
| `.github/workflows/backbone-microservices-ci.yml` | per-microservice matrix: `cargo fmt/check/clippy/test` (lines 251,264,277,290) + `./bin/oya gate validate cargo-prefix` (305) | CI | repo → check-runs | N — 100% cargo, ambient, NOT buck2 |
| `.github/workflows/docs-graph-drift.yml` | `cargo build/test/run` the arch-graph generator + drift diff (lines 65,66,70) | CI | repo → drift check | N — 100% cargo |
| `Makefile` (66 ln) | deploy entrypoints: `tofu plan/apply/init/fmt`, `cargo run -p oya-dev-cli -- gate validate deployment-ops-contract` (43), `check-tofu` (60) | CD/dev | `tofu` + cargo → Cloudflare edge | N — wraps unpinned `tofu`/`cargo`; `check-tofu` is ambient `command -v` |
| `scripts/build/build-and-push-cloud-intelligence.sh` | build + push the cloud-intelligence container image | CD/BUILD | docker/registry → image | N — docker build+push, ambient |
| `scripts/ci/third-party-buckify-handedits.patch` | captured cross-platform `select()` edits to `third-party/BUCK` | BUILD | (data, applied by regen) | P — reproducible but a patch-over-generator |

### A.2 source/ (cloud) — CD (deploy / fleet)

| Path | What it does | Stage | Consumes → Produces | Hermetic? |
|---|---|---|---|---|
| `infra/talos/installation-media/gen-media.sh` | generate Talos control-plane/node boot media | CD | schematic + endpoint → media | N — hardware-gated, ambient `talosctl`/network |
| `infra/talos/bare-metal/up.sh`, `infra/talos/local/talos-local.sh`, `infra/talos/smoke-kata.sh` | bring up bare-metal / local Talos; kata smoke | CD | talos/qemu → cluster | N — host+hardware ambient |
| `infra/capi/init.sh` + `infra/capi/crs/render.sh` | Cluster API install + ClusterResourceSet render | CD | `clusterctl` + KUBECONFIG → CAPI | N — ambient kubeconfig/network |
| `infra/gitops/*.yaml` (Argo CD: `root-app.yaml`, `bootstrap-sync.yaml`, `values.yaml`, `vcs-substrate.yaml`) | declarative GitOps app-of-apps | CD | git repo → reconciled state | **Y (declarative!)** — Argo CD reconciles desired state from git; this is the CD model to STANDARDIZE on |
| `infra/cloudflare/*` (via Makefile `tofu`) | OpenTofu Cloudflare edge desired-state | CD | tofu state → edge | P — declarative IaC, but driven by ambient `tofu` CLI from Makefile |

### A.3 source/ (cloud) — DEV-ENV

| Path | What it does | Stage | Consumes → Produces | Hermetic? |
|---|---|---|---|---|
| `.envrc` | direnv: `PATH_add bin` so `oya` resolves; doc says manually export PATH if no direnv | DEV | clone → PATH | N — requires `direnv allow` (a manual step) OR a hand-edit to shell rc |
| `bin/oya` (shim) | resolves `oya` CLI without full cargo invocation | DEV | cargo build of oya-dev-cli | P — convenience wrapper over cargo |
| `rust-toolchain.toml` (`channel=1.95.0`, minimal+rustfmt+clippy) | pins the cloud rust channel | DEV/CI/BUILD | rustup → toolchain | **Y** — rustup honors it; CI relies on this (no toolchain action, `oya-ci-required.yml:53-54`) |
| `reindeer.toml` | buckify config for third-party | BUILD | Cargo manifests → BUCK | Y (config; the GLUE is `regen-third-party.sh`) |

### A.4 linux/ (kernel) — BUILD / VERIFY (all shell; ZERO kernel BUCK targets)

| Path | What it does | Stage | Consumes → Produces | Hermetic? |
|---|---|---|---|---|
| `stack/kernel/scripts/build-carriers.sh` (300 ln) | discovers + runs every per-program `build.sh`; Stage B reproduces musl talos carriers via bundled `rust-lld` (no Docker) | BUILD | per-program src → `out/*.elf` carriers | P — Stage A/B reproducible w/ pinned nightly; but it is a bash orchestrator over `cargo build`, runs `rustup target add` (network), gates `docker` for one carrier |
| `stack/kernel/crates/*/*-src/build.sh` (×13) | copy crate to out-of-repo tmpdir (escape parent `.cargo` rustflags), `cargo build --release`, copy ELF back to `out/` | BUILD | crate src → one `out/*.elf` | P — reproducible given pinned nightly; ambient `cargo`/`mktemp`; the tmpdir-escape hack is a cargo-config-leak workaround |
| `stack/kernel/.cargo/config.toml:25,44` | wires `run-qemu-{aarch64,x86_64}.sh` as the cargo `runner` | VERIFY | `cargo run`/test → QEMU boot | N — runner is a shell script invoking ambient `qemu-system-*` |
| `stack/kernel/scripts/run-qemu-{x86_64,aarch64}.sh` | boot kernel ELF under QEMU (TCG), grep serial for `kernel: OK`, map isa-debug-exit code 33 | VERIFY | ELF → boot verdict | N — ambient `qemu-system-*` (unpinned), `timeout`/`gtimeout` host-dependent |
| `stack/kernel/scripts/conformance-probe.sh` (309 ln) | swap candidate ELF into a carrier slot, `cargo build -p kernel --features X`, boot, extract syscall/ENOSYS demand | VERIFY | ELF + kernel → syscall report | N — ambient cargo+qemu+xxd+awk |
| `stack/kernel/scripts/diff-oracle.sh`, `diff-oracle-gicv3.sh` | regression-pin the default syscall trace | VERIFY | boot trace → diff verdict | N — ambient |
| `stack/kernel/scripts/run-loom.sh`, `stress-smp.sh`, `assert-smp-boot.sh`, `assert-talos-boot{,-gicv3}.sh`, `check-tcb.sh` | the named gates (run-loom / assert-smp-boot / diff-oracle / check-tcb per MEMORY) | VERIFY | cargo/qemu → gate verdicts | N — ambient cargo+qemu wrappers |
| `stack/kernel/rust-toolchain.toml` (`nightly-2026-02-28` + rust-src + 2 none targets) | pins the kernel nightly + `-Z build-std` targets | BUILD/DEV | rustup → toolchain | **Y** — exact nightly pin, reproducible |
| `stack/operating-system/boot/{build-init,mkinitramfs,boot-qemu,fetch-kernel,mkimage,boot-image}.sh` | Talos-init initramfs build (Docker `rust:alpine`), fetch prebuilt Alpine kernel+modules, boot under QEMU | BUILD/CD | Docker/network → initramfs/image | N — `fetch-kernel.sh` pulls a PREBUILT external kernel+`.ko` blobs (hermeticity debt), `build-init.sh` uses Docker |
| `stack/operating-system/platforms/*.sh` (qemu-x86_64, qemu-aarch64, firecracker, cloud-hypervisor, vfkit, gvisor-runsc, conformance) | per-VMM boot/conformance launchers | VERIFY/CD | image + VMM → boot | N — ambient VMM CLIs |
| `stack/operating-system/tools/buck/{cargo-test,cargo-check,wave29-tests}.sh` | buck-adjacent cargo wrappers (note the irony: bash under a `buck/` dir) | BUILD/VERIFY | cargo → results | N — cargo wrappers, not buck2 |
| `stack/kernel/out/*.elf` (tracked: `user-hello.elf`, `user-musl.elf`, `user-x86_64.elf`) | git-TRACKED carrier blobs | BUILD | (committed bytes) | N — **tracked prebuilt blobs = direct hermeticity-doctrine violation** (carrier producer exists to retire these) |

### A.5 Cross-cutting (both repos) — agent/session hooks (out of CI/CD scope)

`*/.claude/hooks/*.sh`, `source/tools/hooks/*.sh`, `source/tools/agent-skills/hooks/*.sh` are **agent session-lifecycle hooks**, not build/CI/CD/dev-env glue. They are OUT OF SCOPE for lifecycle hermeticity (they configure the AI harness, not the product build) and are listed here only to mark them excluded.

---

## B. TARGET ARCHITECTURE PER LIFECYCLE STAGE

### B.1 BUILD — everything is a buck2 target

**Principle:** no artifact is produced by a `build.sh`/`cargo build` orchestrated by bash. Every artifact has a buck2 rule whose inputs are declared.

**source/ (cloud) — already mostly there:**
- The `//cloud/cloud-ci/...` gates build under buck2 today (`oya-ci-required.yml:216`). Extend `buck2 build //...` to cover EVERY crate so the cargo lanes can retire (depends on the cargo→buck2 required-context switch, §E).
- `regen-third-party.sh` → keep `reindeer` as the buckify *generator* (it is the standard tool; not in-graph by design) but (a) PIN reindeer as a `toolchains//` cell download (not unpinned-on-PATH), and (b) make the hand-edits a reindeer **fixup** or a checked-in `select()` overlay so the `*.patch` re-apply step disappears. Target: `third-party/BUCK` is regenerated by `buck2 run //tools:buckify` (a `rust_binary` or `sh_binary` wrapping reindeer), not a bash script.

**linux/ (kernel) — the big net-new lift. The recommended target shape:**
- **Carriers as `genrule`/`rust_binary` outputs, not `build.sh`.** Each `crates/*/*-src` user program becomes a buck2 `rust_binary` with its own platform/link config (the `.cargo/config.toml` per-crate flags map to `rustc_flags` + a buck2 linker-script `srcs` dep). `build-carriers.sh`'s tmpdir-escape hack exists ONLY because cargo concatenates parent `.cargo/config.toml` rustflags (`user-spawn-src/build.sh:6-13`) — buck2 has no such ambient config inheritance, so the hack *evaporates* under buck2. This is a case where buck2 is strictly cleaner than the cargo+bash status quo.
- **The kernel image as a buck2 target** that `include_bytes!`-embeds the carrier rule outputs via `$(location //...:carrier)` (resolve the embed paths from buck2 outputs, not git-tracked `out/*.elf`). This RETIRES the tracked `out/*.elf` blobs (A.4 hermeticity violation) — the carriers become build outputs, never committed.
- **The QEMU boot as a buck2 `rust_test` (or `command_test`).** The boot+grep-marker logic in `run-qemu-*.sh` becomes a small `rust_binary` test harness (parse serial, assert marker, map exit 33) invoked as the test's runner, with `qemu-system-*` provided by a **pinned QEMU toolchain target** (see §F PM2). The cargo `runner` wiring in `.cargo/config.toml` is replaced by `buck2 test //stack/kernel:boot-x86_64`.
- **The verify gates** (run-loom, assert-smp-boot, diff-oracle, check-tcb, conformance-probe) become `rust_test` targets that shell QEMU through the pinned toolchain, replacing the bash. Their oracle/trace-extraction `awk`/`grep` pipelines become Rust in the harness.
- **`fetch-kernel.sh`'s prebuilt Alpine kernel + `.ko` blobs** are an external-blob hermeticity debt the doctrine forbids; flag for a separate decision (build-from-source vs vendored-with-pinned-hash) — out of this doc's net-new scope but recorded as PM/debt.

**The affected-set driver** (`buck2-affected-gate.sh`) → reimplement as a `rust_binary` (`//tools:affected-gate`) that calls `buck2 uquery`/`rdeps` via subprocess but contains the git-diff + classification + fail-closed logic in typed Rust, not bash. This is the same logic, VCS-agnostic-ready (it can read the scm-facts `tracked_paths` instead of `git diff` once the sibling VCS doc lands).

### B.2 CI — the forge workflow is a thin, generated, swappable adapter

**Principle (from CICD-DESIGN-PLAN ONE CANONICAL CI):** the forge YAML does `~buck2 test //...` and NOTHING ELSE. All gate logic lives in buck2 targets.

**Target `oya-ci-required.yml` shape (post-cutover):**
```yaml
jobs:
  oya-ci-required:
    steps:
      - uses: actions/checkout@v4 with: { fetch-depth: 0 }   # ambient git fetch (adapter-level, see §D)
      - run: <pinned buck2 toolchain bootstrap>               # §D irreducible (or a setup-action that is itself generated)
      - run: <scm-facts emitter regen + byte-validate>        # §D irreducible git boundary (sibling doc owns the rename)
      - run: buck2 test //...                                 # THE gate — one line, all logic in-graph
```
- **Retire the 6 cargo gate lanes** (`oya-ci-required.yml:78-139`) once `buck2 test //cloud/cloud-ci/...` is proven byte-parity-equal (the cargo→buck2 required-context switch, §E). The matrix-of-gates fan-out becomes buck2's internal parallelism + the affected-set driver.
- **`backbone-microservices-ci.yml` and `docs-graph-drift.yml`** (both 100% cargo) fold into the same `buck2 test //...` — they are currently parallel cargo CIs that violate ONE CANONICAL CI. Their gates become buck2 `rust_test` targets.
- **The forge YAML itself should be GENERATED** from the buck2 target list by a `rust_binary` (`//tools:ci-adapter-gen`) so it cannot drift from the graph (the false-green pre-mortem, §F PM3). The same generator emits the owned-runner control-plane Job spec — one generator, two adapter outputs.
- **`oya-ci-post.sh`** (the local→GH status bridge) is pure transitional forge coupling (ADR-0387 webhook gap); it RETIRES when the owned runner lands. Do not buckify it; mark it transitional-die.

### B.3 CD — hermetic, declarative deploy

**Principle:** no `deploy.sh`. Desired state is declared; a reconciler converges it.

**What exists vs net-new:**
- **EXISTS (and is the model to standardize on):** `infra/gitops/*` is Argo CD app-of-apps (`root-app.yaml`, `bootstrap-sync.yaml`, `values.yaml`) — this is ALREADY declarative, git-driven CD (the one Y-hermetic CD surface in A.2). CD's target is: **everything reconciled by Argo CD from declared manifests**, not pushed by a script.
- **EXISTS but shell-driven:** `Makefile` `tofu apply` (Cloudflare edge), `infra/capi/init.sh` (CAPI install), `infra/talos/*.sh` (Talos media/bring-up). These are inherently hardware/endpoint-gated and partly irreducible (you cannot buckify "boot media on a physical node"). Target: wrap the *automatable* parts (tofu plan/apply, helm template, kubectl apply) as buck2 targets or fold into the GitOps reconcile; keep the irreducible hardware bring-up as documented one-shot steps (§D).
- **NET-NEW (the gap):** there is **no buck2-native deploy primitive** anywhere (PM4). The artifact-producing half of CD (container images: `build-and-push-cloud-intelligence.sh`, the kernel/OS images: `mkimage.sh`) SHOULD become buck2 image targets (e.g. a `genrule`/OCI-layer rule producing a content-addressed image), so CD consumes a buck2-built, hashed artifact. The deploy half stays declarative (Argo CD/CAPI/tofu desired-state). The seam: **buck2 builds the hashed artifact → GitOps reconciles a manifest that pins that hash.** This is the same "git is transitional" pattern — the manifest is the contract, the reconciler is the swappable adapter.

### B.4 DEV-ENV — clone-and-`buck2 build`-just-works

**Principle:** zero setup script. A fresh clone + a pinned toolchain = green build.

**Target:**
- **`buck2 build //...` works with no setup.** The toolchains cell (`toolchains/BUCK`) provides rustc/cxx/python. Today both repos use `system_*_toolchain` (host PATH, `toolchains/BUCK:10`) / `system_demo_toolchains()` (linux `toolchains/BUCK:6`) — that is NOT hermetic (it trusts host rustc). **Recommendation: move from `system_rust_toolchain` to a buck2-managed downloaded toolchain** keyed off `rust-toolchain.toml` (cloud `1.95.0`; kernel `nightly-2026-02-28`), so the exact channel is a declared, downloaded input — not the host's rustup.
- **Retire `.envrc` + `bin/oya` manual PATH step** — `direnv allow` is a manual step the directive forbids ("no manual steps"). The `oya` CLI becomes `buck2 run //:oya`; no PATH munging.
- **Minimal-option recommendation (Nix vs devcontainer vs buck2-toolchains-only):**
  - **buck2-managed toolchains ALONE suffice for rustc/cxx/python** (download_toolchain keyed on the pinned channel). This is the minimal, in-graph option and is PREFERRED for the Rust toolchain.
  - **They do NOT cleanly pin QEMU/musl-std/talosctl/tofu** (PM2): QEMU is a host VMM, musl-std is a rustup component, talosctl/tofu are external CLIs. A buck2 `http_archive`/`download_file` rule CAN pin QEMU and musl-std by URL+sha256 as toolchain cell artifacts (this is the recommended path — pin them as declared downloads, not host installs).
  - **A Nix flake is WARRANTED ONLY IF** the buck2 `download_file`-pinned set proves insufficient for the QEMU/talosctl/host-lib closure (e.g. QEMU's own dynamic-lib deps on the runner). Recommendation: **default to buck2-managed pinned downloads; adopt a minimal Nix flake as the dev-env fallback ONLY for the irreducible external binaries (QEMU, talosctl) if download_file pinning can't make them reproducible across dev/CI hosts.** Do NOT adopt a devcontainer — it adds a Docker dependency that itself breaks "no external blobs" and is heavier than the flake-fallback.

---

## C. THE "ONE LOGIC, MANY RUNNERS" SEAM

The identical buck2 targets are invoked by all four runners; the ONLY per-runner difference is a thin adapter:

| Runner | Invocation (identical core) | Thin adapter (the ONLY difference) | Adapter is generated? |
|---|---|---|---|
| **DEV** | `buck2 build //...` / `buck2 test //...` | none — the bare command | n/a |
| **CI (forge, today GitHub)** | `buck2 test //...` (+ affected-set scope) | `oya-ci-required.yml` (checkout + toolchain bootstrap + scm-facts regen) | **YES — generate from the buck2 target list** (`//tools:ci-adapter-gen`) so it can't drift (PM3) |
| **CI (owned runner, future)** | `buck2 test //...` (same) | the `oya-ci-controller` K8s-Job spec (CICD-DESIGN-PLAN §3) | YES — same generator, second output. This IS "git is transitional": the forge YAML and the control-plane Job are interchangeable adapters over one graph |
| **CD** | `buck2 build //...:image` (hashed artifact) | Argo CD manifest pinning the artifact hash; CAPI/tofu desired-state | the manifest's image-hash field is generated by the build; the reconciler is declarative |

**The seam invariant:** the gate LOGIC is a buck2 `rust_test` target (the same Rust crate the cargo lanes test today — `oya-cloud-ci-*-app`). Dev runs it, CI runs it, CD's pre-deploy check runs it. **No runner re-implements a narrower subset** (the cardinal false-green sin, CICD-DESIGN-PLAN Principle 1) — they all run `//...` or its affected-scoped subset. The git→scm-facts rename (sibling VCS doc) makes the ONE ambient input (the facts snapshot) VCS-agnostic, so swapping GitHub for the bespoke SCM changes only the emitter adapter, not the graph.

---

## D. IRREDUCIBLE-GLUE LEDGER

The glue that genuinely CANNOT be a pure in-graph buck2 action — each justified, pinned, reproducible. "Minimal," not "zero."

| Glue | Why irreducible | Pinning / reproducibility requirement |
|---|---|---|
| **Toolchain bootstrap** (buck2 binary itself; rustc/QEMU/musl downloads) | buck2 is the build tool (like cargo/rustc) — it cannot build itself in-graph; the first rustc/QEMU must come from somewhere | Pin buck2 by release tag (`oya-ci-required.yml:166` already does — `2025-05-06`). Move the `curl` into a buck2 `download_file` (toolchains cell, sha256-pinned) so it is a declared input, not a CI `run: curl`. Pin rustc via `rust-toolchain.toml` (✓ exists both repos). Pin QEMU/musl-std as sha256 `download_file` toolchain artifacts (§B.4, PM2). |
| **The git→scm-facts emitter** (the ambient VCS boundary) | git is inherently ambient; it CANNOT be a pure function of declared inputs inside a hermetic action (HERMETIC-EXECUTION §0). Resolution: ONE emitter step at the graph edge whose OUTPUT is committed + content-addressed; every in-graph action is pure over that frozen face | Run ONLY as a CI pre-step + local regen hook, never in a cacheable action. Byte-validated by `registry-drift` (no new trust root). The `git`→`scm-facts` rename + `ScmFactsSource` trait is OWNED BY `OYA-CI-VCS-AGNOSTIC-SEAM-REFINEMENT-PLAN.md` — DO NOT duplicate. This makes the boundary VCS-agnostic, honoring "git is transitional." |
| **CI checkout** (`actions/checkout@v4 fetch-depth:0`) | the forge runner must materialize the repo + full history before buck2 runs (the scm-facts emitter derives last-touch via `git log`) | Adapter-level (in the generated forge YAML); not a build action. `fetch-depth:0` is required (shallow collapses ages → false-green, PM1, documented `oya-ci-required.yml:46-52`). |
| **Hardware/endpoint CD bring-up** (`infra/talos/installation-media/gen-media.sh`, `infra/capi/init.sh`, bare-metal `up.sh`) | "boot media on a physical node" / "install CAPI against a live KUBECONFIG" is inherently a side-effecting, hardware-gated, one-shot act — not a pure build | Keep as documented one-shot runbook steps (Makefile `fleet` target already only PRINTS them, `Makefile:46-53`). Pin `talosctl`/`clusterctl`/`tofu` versions; drive the AUTOMATABLE desired-state via Argo CD/tofu, not ad-hoc shell. |
| **reindeer buckify** (third-party BUCK generation) | the standard, intentionally-out-of-graph generator (like a codegen step); reindeer reads Cargo.lock → emits BUCK | Pin reindeer (toolchains cell, not unpinned PATH); fold the `*.patch` hand-edits into reindeer fixups or a checked-in `select()` overlay so the patch-re-apply bash (`regen-third-party.sh`) disappears. Wrap as `buck2 run //tools:buckify`. |

**Everything NOT in this ledger is a retirement target.** In particular: the 6 cargo CI lanes, the per-program `build.sh` ×13, `run-qemu-*.sh`, `conformance-probe.sh`, `diff-oracle*.sh`, the kernel verify-gate scripts, `build-carriers.sh` (as an orchestrator), and the tracked `out/*.elf` blobs.

---

## E. SEQUENCING (retire glue in dependency order)

Retirement must follow dependency order; each step has a proof gate. Relationship to the in-flight work is explicit.

**Stage 0 — already landed / in-flight (do not redo):**
- (i) The buck2 CI lane the executor just added (`oya-ci-required.yml:152-223`) — its 5 inline `run:` steps are the FIRST retirement targets (Stage 2).
- The kernel Stage A/B carrier producer (`build-carriers.sh`) — reproducible but still bash; becomes the buckify source for Stage 4.
- The git-facts boundary (HERMETIC-EXECUTION doc) + scm-facts rename (VCS-AGNOSTIC doc) — referenced, owned elsewhere.

**Stage 1 — cloud build-graph completion (prereq for everything CI).**
- Extend `buck2 build //...` to every cloud crate (not just `//cloud/cloud-ci/...`). Pin reindeer + fold hand-edits → retire `regen-third-party.sh`'s patch step. Proof: `buck2 build //...` green from clean clone.
- DEPENDS ON: the producer-rooted Buck2 tree being on `dev` (DEV-CUTOVER-PLAN).

**Stage 2 — cargo→buck2 CI required-context switch (the PENDING switch).**
- Prove `buck2 test //cloud/cloud-ci/...` byte-parity-equal to the 6 cargo lanes (HERMETIC-EXECUTION Stage P2 byte-parity obligation). THEN retire the 6 cargo lanes + fold `backbone-microservices-ci.yml` + `docs-graph-drift.yml` into `buck2 test //...`. Proof: `oya-ci-required` green with ZERO cargo `run:` steps; only `buck2 test //...` + the §D irreducibles remain.
- DEPENDS ON: Stage 1; the founder-paired required-context identity door (CICD-DESIGN-PLAN, the 🛑 admin door).

**Stage 3 — generate the forge adapter.**
- Build `//tools:ci-adapter-gen` (emits the forge YAML + the owned-runner Job spec from the buck2 target list). Add a CI check that the committed YAML == generated (drift gate). Proof: regenerate == committed, byte-equal. Retires the false-green-drift risk (PM3).
- DEPENDS ON: Stage 2 (the YAML is stable as `buck2 test //...`).

**Stage 4 — kernel buckification (the big net-new lift; INDEPENDENT of cloud Stages 1-3).**
- 4a: carriers → `rust_binary`/`genrule` targets (tmpdir-escape hack evaporates under buck2). Retires `build.sh` ×13 + the tracked `out/*.elf` blobs. Proof: `buck2 build //stack/kernel:carriers` produces byte-identical ELFs; `out/*.elf` removed from git.
- 4b: kernel image as a buck2 target embedding carrier outputs via `$(location)`. Proof: `buck2 build //stack/kernel:image-{x86_64,aarch64}`.
- 4c: QEMU boot + verify gates → `rust_test` targets over a PINNED QEMU toolchain (PM2). Retires `run-qemu-*.sh`, `conformance-probe.sh`, `diff-oracle*.sh`, the gate scripts. Proof: `buck2 test //stack/kernel/...` green == the current bash gates' verdicts.
- DEPENDS ON: pinned QEMU/musl toolchain (§B.4 / PM2) FIRST.

**Stage 5 — CD buck2 artifact primitive + dev-env toolchain pin.**
- Image targets (`build-and-push` / `mkimage` → buck2 OCI-layer rules); GitOps manifest pins the hash. Move `system_*_toolchain` → downloaded pinned toolchains; retire `.envrc`/`bin/oya` manual step. Proof: `buck2 build //...:image` hashed output; clean clone `buck2 build //...` with no host rustc.
- DEPENDS ON: Stages 1 + 4 (artifacts exist to deploy).

**Relationship summary:** Cloud (1→2→3) and Kernel (4) proceed in PARALLEL after the cutover; CD (5) is last (needs artifacts from both); the scm-facts rename (sibling doc) slots between Stage 2 and Stage 3 (rename the boundary before generating the adapter that references it).

---

## F. PRE-MORTEM (≥3)

**PM1 — Zero-shell dogma breaks hermeticity by removing a pinned bootstrap.**
Failure: an over-zealous "delete all shell" pass removes the pinned buck2 `download_file` or the `fetch-depth:0` checkout, and CI either can't bootstrap buck2 or silently shallow-checks-out (ages collapse → false-green, the exact PM the workflow already guards, `oya-ci-required.yml:46-52`).
Mitigation: the §D ledger is AUTHORITATIVE — the bootstrap and the git boundary are EXPLICITLY irreducible and MUST survive. "Minimal," not "zero." Any retirement PR that touches a §D-ledger item is a one-way door requiring re-justification. The directive says "0 to minimal" — the ledger IS the minimal.

**PM2 — buck2-managed toolchain can't pin QEMU/musl reproducibly.**
Failure: `download_file` pins the QEMU binary by URL+sha256 but QEMU's dynamic-lib closure (glibc, libpixman, libslirp) differs across dev macOS vs CI ubuntu, so the "same" QEMU behaves differently → non-reproducible boot verdicts. Same risk for musl-std cross-host.
Mitigation: (a) pin QEMU + musl-std as `download_file` toolchain artifacts FIRST and verify boot-verdict reproducibility across both host classes BEFORE Stage 4c. (b) If the dynamic-lib closure proves non-reproducible, fall back to a MINIMAL Nix flake for QEMU/talosctl ONLY (§B.4) — accept it as the irreducible dev-env adapter rather than breaking hermeticity. (c) Statically-linked QEMU or a pinned QEMU container are alternatives if the flake is rejected. This is the single biggest kernel-side technical risk and gates Stage 4c.

**PM3 — the generated forge adapter drifts from the buck2 targets (false-green).**
Failure: a gate target is added to the graph but the hand-edited `oya-ci-required.yml` (or the owned-runner Job spec) isn't regenerated, so CI runs a STALE narrower set and goes green while a real gate is unrun — the cardinal false-green sin.
Mitigation: Stage 3 makes the adapter GENERATED (`//tools:ci-adapter-gen`) + adds a CI drift gate (committed YAML == regenerated, byte-equal), mirroring the existing `registry-drift` + `gate_registration` meta-test pattern (`oya-ci-required.yml:64-77` already enforces "every in-tree gate crate is listed"). Until generation lands, the `gate_registration` meta-test is the interim defense. The adapter must NEVER carry its own command subset (it runs `buck2 test //...`, full).

**PM4 — CD has no buck2-native deploy primitive yet.**
Failure: Stage 5 tries to make deploy hermetic but there is no buck2 rule for "deploy," so the team either (a) writes a `deploy.sh` (re-introducing the shell we're eliminating) or (b) stalls.
Mitigation: SPLIT CD into (1) artifact BUILD (buck2-native: OCI-layer/image `genrule`s producing content-addressed artifacts — this IS achievable in-graph) and (2) RECONCILE (declarative, NOT a build action: Argo CD/CAPI/tofu desired-state, the already-Y-hermetic `infra/gitops/*` model). The seam is "buck2 builds the hashed artifact → the GitOps manifest pins that hash." Do NOT try to make "deploy" a buck2 action — deploy is a side-effecting reconcile, correctly OUTSIDE the pure graph (it belongs in the §D irreducible/declarative tier). Recording this split now prevents the deploy.sh regression.

**PM5 (bonus) — kernel buckification stalls the verified cargo track.**
Failure: the kernel's CURRENTLY-PASSING cargo+QEMU verification (the goal-ladder gates per MEMORY: run-loom/assert-smp-boot/diff-oracle/check-tcb) is mid-flight (P4·SMP S4b building in a worktree); ripping out the bash gates for buck2 targets mid-stream breaks the verified floor.
Mitigation: Stage 4 is ADDITIVE-FIRST — buck2 targets run ALONGSIDE the cargo+bash gates (exactly as the cloud buck2 lane runs alongside the cargo lanes today, `oya-ci-required.yml:152` "runs ALONGSIDE the cargo lanes") with byte/verdict-parity proof, and the bash gates retire ONLY after the buck2 equivalents are proven green over the SAME verdicts. Never retire a bridge before its replacement is proven (CICD-DESIGN-PLAN Principle 5).

---

## Consensus Addendum (ralplan-style review of this design)

- **Antithesis (steelman against buckifying the kernel):** The kernel's cargo+QEMU+bash track is *currently green and verified across both arches* (the goal-ladder is DONE per MEMORY). Buckifying it is enormous net-new work (kernel has ZERO BUCK targets today) whose payoff is "the same verdicts, via buck2." The strongest counter to this whole design is: **maybe the kernel should stay cargo+bash and ONLY the cloud side goes fully buck2-native** — keeping `rust-toolchain.toml`-pinned cargo + reproducible Stage A/B carriers is *already* substantially hermetic, and the marginal hermeticity gain from buck2 may not justify the risk to a working kernel. The directive says hermetic across all four stages, but does NOT say "buck2 everywhere" — a cargo build pinned by `rust-toolchain.toml` with reproducible carriers and pinned QEMU could satisfy "hermetic + it just works" without buck2. This design should be approved ONLY if the founder wants buck2 as the literal single substrate (the "must run buck2" authority in HERMETIC-EXECUTION supports this) vs. "hermetic by any means."
- **Tradeoff tension (cannot be ignored):** zero-shell purity vs. reproducibility. The directive pairs "0-to-minimal shell" with "do NOT break hermeticity in pursuit of zero-shell purity." These actively conflict at the QEMU/musl/toolchain boundary (PM2): the most reproducible option (a Nix flake or pinned container for QEMU) is MORE external machinery, not less — it trades shell-script-count for a Nix/Docker dependency. You cannot maximize both. This design resolves it by ranking reproducibility ABOVE shell-count (the §D ledger), but the founder must accept that "minimal shell" may mean "a Nix flake for QEMU" — which is arguably more, not less, external tooling.
- **Synthesis (preserve strengths of both):** Adopt buck2-native for the CLOUD side fully (it's 70% there, the hard boundary is designed) AND for kernel ARTIFACT production (carriers/image — where buck2 is strictly cleaner than the tmpdir-escape hack). Keep the kernel VERIFY gates (QEMU boot) buckified but ADDITIVE-first (PM5) so the verified cargo floor is never at risk. This captures buck2's hermeticity where it's cheap/clean and defers the risky kernel-verify swap behind a parity gate.
- **Principle violations (deliberate-mode flags):**
  - **VIOLATION (active, doctrine):** git-tracked `stack/kernel/out/*.elf` blobs (A.4) directly violate "NO external/prebuilt blobs." Severity: HIGH (it is the named hermeticity debt; Stage 4a retires it).
  - **VIOLATION (active, doctrine):** `fetch-kernel.sh` pulls a PREBUILT external Alpine kernel + `.ko` modules (A.4). Severity: HIGH (external blob; flagged as out-of-scope debt requiring its own build-from-source decision).
  - **VIOLATION (active, ONE CANONICAL CI):** `backbone-microservices-ci.yml` + `docs-graph-drift.yml` are parallel cargo CIs alongside `oya-ci-required` (A.1). Severity: MEDIUM (Stage 2 folds them in).
  - **VIOLATION (active, "no manual steps"):** `.envrc` requires `direnv allow` / manual PATH edit (A.3). Severity: LOW (Stage 5; `buck2 run //:oya` retires it).

---

## Appendix — key file references (verified, absolute)

- `/Users/jasonlee/Developer/source/.github/workflows/oya-ci-required.yml:78-223` — the canonical CI: 6 cargo lanes + 1 buck2 lane + the 5 inline `run:` shells (the first retirement targets).
- `/Users/jasonlee/Developer/source/infra/ci/buck2-affected-gate.sh:60-122` — the affected-set driver (bash; should become `//tools:affected-gate` rust_binary).
- `/Users/jasonlee/Developer/source/toolchains/BUCK:10,63` — `system_rust_toolchain` + `noop_test_toolchain` (host-PATH, NOT hermetic; move to downloaded pinned toolchain).
- `/Users/jasonlee/Developer/source/scripts/ci/regen-third-party.sh` — reindeer buckify + patch re-apply glue.
- `/Users/jasonlee/Developer/source/Makefile:42-53` — CD entrypoints (tofu + the `fleet` print-only target).
- `/Users/jasonlee/Developer/source/infra/gitops/root-app.yaml` — the one already-declarative/hermetic CD surface (the model to standardize on).
- `/Users/jasonlee/Developer/linux/stack/kernel/scripts/build-carriers.sh:1-300` — the Stage A/B carrier producer (reproducible but bash; buckify source for Stage 4a).
- `/Users/jasonlee/Developer/linux/stack/kernel/crates/arch-x86_64/user-spawn-src/build.sh:6-13` — the cargo-config-leak tmpdir-escape hack that evaporates under buck2.
- `/Users/jasonlee/Developer/linux/stack/kernel/.cargo/config.toml:25,44` — QEMU runner wiring (replaced by `buck2 test`).
- `/Users/jasonlee/Developer/linux/toolchains/BUCK:6` — `system_demo_toolchains()` shim; the kernel has ZERO other BUCK targets.

**Verified facts driving the design:** the kernel has **zero non-buck-out BUCK files** (100% cargo+shell); source has **763**; `stack/kernel/out/*.elf` carriers are **git-tracked** (HIGH-severity doctrine violation); two source CI workflows are **100% cargo** (parallel to the canonical CI). The git-facts boundary and the `scm-facts` rename are owned by the two sibling docs and were referenced, not duplicated.
