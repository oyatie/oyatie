# 00 — Pre-lane 0.5 MANIFEST BUNDLE (synthesis of the six `10-*.md` artifacts)

**Lane:** pre-lane 0.5 (manifest synthesis) · **Mode:** READ-ONLY (no edits, no builds) · **Date:** 2026-06-06
**Authority anchors:** `UNIFIED-EXECUTION-PLAN.md` (D-CONFORM §6, WIP gates §11, lane order §… L0–L11), ADR-0513 (oya-ci flip), ADR-0015/0016 (oya-cloud-k8s home), ADR-0111 (placeholder-debt), ADR-0221 (governance gates), ADR-0384 (codex Path B).

This bundle consolidates the six source artifacts in this directory into a single feed for pre-lane 0.6 (kernel-exclude inertness proof) and for the founder WIP gates G2/G4. Each section is a faithful roll-up; the source artifact remains authoritative where evidence detail is needed.

Source artifacts rolled up here (all `/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/_execution/prelane-0.5/`):
`10-tools-targets.md` · `10-source-inventory.md` · `10-k8s-split.md` · `10-merge-surfaces.md` · `10-kernel-exclude.md` · `10-gate-characterize.md`.

---

## 1 — `tools/` standing canonical-homes exception set (G2)

**EXCLUDE-FROM-RETIREMENT — the standing exception set:**

1. **22 BUCK-bearing `//tools/...` targets** (all `rust_binary`, label `//tools/<name>:<name>`, `visibility=["PUBLIC"]`, each has BOTH `BUCK` + `Cargo.toml`):
   - **3 invoked by literal `cargo run -q -p` in the gate command roster** (`libs/oya-governance-gate-catalog-domain/src/lib.rs:262-265`) — hardest KEEP: `oya-vcs-admission-gate-app`, `oya-vcs-provider-execution-gate-app`, `oya-governance-purpose-audit-app`.
   - **16 governance fitness apps** (Buck-affected-target gate set; gate-built when affected): the 12 `oya-governance-*-status-lifecycle-app` (adr / api-stability-tier / capability / crate / dependency / doc / feature-flag / migration / plan + status variants) **plus** `oya-governance-adapter-with-no-importer-app`, `oya-governance-adr-shape-app`, `oya-governance-authoritative-tracked-app`, `oya-governance-banned-primitives-app`, `oya-governance-portfolio-citation-app`, `oya-governance-predictable-naming-app`, `oya-governance-sunset-lifecycle-app`.
   - **3 adjacent gate/agent tooling:** `oya-adapter-substitution-test-app`, `oya-tooling-agent-read`, `oya-xtask-metadata-augment-app`.
   - (Count reconciles: 3 + 16 + 3 = 22.)

2. **Gate shell harnesses (KEEP, not Buck targets):** `tools/governance/` (`adr-0221-governance-gates.sh`, shelled at `lib.rs:256-259`) + `tools/hooks/` (the hook scripts it drives, e.g. `vacuous-green-gate-detect.sh`, `adr-orphan-detect.sh`).

3. **Buck launcher infra (KEEP):** `tools/buck`, `tools/buck2`.

4. **Reserved-but-absent (placeholder-debt, ADR-0111):** `tools/oya-vcs-merge-queue-fix-loop-app` — invoked by the gate (`lib.rs:265`) and referenced in registry (`vcs/concurrent-safe-paths.yaml:17`, `quality/lanes.yaml:607`, `vcs/event-router.yaml`), but **the directory does not exist on disk**. Reserve the name; retirement must not reuse/block it.

**`//services/...` exception set = ∅ (EMPTY).** `source/services/` (analytics, app-shell-frontend, ci-webhook-gateway, policy, treasury) has **zero BUCK and zero Cargo.toml** — no live build target. Buildable service binaries live under `oya/<domain>/` and `microservices/<ms>/`.

**WIP-plan-named tools verification:** `oya-doc-staleness-inventory-app` and `oya-adr-index-regenerator-app` do **NOT exist anywhere** (no dir, no Cargo.toml) — aspirational, not gate-load-bearing; nothing to except (names reserved under a future `tools/` home).

**Framing correction carried forward:** the WF spec's "github-lane-unlocker workflow" does NOT exist in `source/.github/workflows/` — that dir has exactly one file (`backbone-microservices-ci.yml`, 100% cargo, builds zero Buck targets). `github-lane-unlocker-required` is the **legacy live `dev` required context**; the real repo-wide gate is **Jenkins** (`source/Jenkinsfile` → `oyaCiLane(service:'repo')`), a **Buck2 affected-target gate**. Flip target = `oya-ci-required` (ADR-0513, in flight).

**Dispatch note (retirement reasoning):** `oya gate validate <lane>` (~130 lanes in `AGGREGATED_VALIDATE_LANES`) runs **in-process via `oya-dev-cli`**, which depends directly on `libs/oya-governance-*-kernel` + `oya-governance-gate-catalog-domain`. The `tools/oya-governance-*-app` binaries are standalone wrappers over those same kernels → the `libs/` kernels are equally load-bearing and equally out of retirement scope.

---

## 2 — Per-lane source inventory + DENY-GLOBS

Canonical DENY-GLOB universe (applied per-tree, intersected with what physically exists):
`_upstream*` / `third-party` / `vendor` / `target` / `buck-out` / `prelude` / `toolchains` / `__pycache__` / `.omc` / `.omx` / `.claude` / `legacy-*` / `talos-reference`.

| Lane | Source path | #first-party crates | Structure | DENY-GLOBS present in tree |
|---|---|---|---|---|
| office | `/Users/jasonlee/Developer/office` | 19 (13 `crates/` + 6 `apps/`, all `oyaoffice-*`) | single Cargo workspace (resolver 3, ed. 2024); self-declares layout in `[workspace.metadata.oyaoffice.first_party_layout]` | `third-party/`, `target/`, `buck-out/`, `toolchains/`, `.omx/` |
| oyago | `/Users/jasonlee/Developer/oyago` | 3 (`oyago-cli`, `oyago-core`, `oyago-runtime`) | single Cargo workspace; Go `go/`+`fixtures/`+prebuilt binaries are reference | `target/` (covers nested buck-out+toolchains under `target/oyago-test-externalrefs`), `.omc/`, `.omx/` |
| oyapy | `/Users/jasonlee/Developer/oyapy` | 3 (`oyapy-cli`, `oyapy-core`, `oyapy-runtime`) | single Cargo workspace (`unsafe_code=deny`, clippy `all=deny`); `python/`+`fixtures/` reference | `target/`, `.omx/` |
| claude | `/Users/jasonlee/Developer/claude` | 1 (`claude-agent-sdk`) | single top-level package, NOT a workspace; `src/` | `target/`, `.omx/` |
| codex | `/Users/jasonlee/Developer/codex` | 1 (`openai-codex-sdk`, lib `openai_codex_sdk`, at `sdk/rust/`) | single package, NOT a workspace | `sdk/rust/target/`, `.omx/` |
| linux-stack | `/Users/jasonlee/Developer/linux/stack` | 208 (kernel 18 + kubernetes 139 + operating-system 45 + kernel-usermode-tests 6) | META-TREE — NO top-level workspace; 3 independent Cargo workspaces + standalone usermode-test crates; `talos-reference/` is a Go reference subtree (0 first-party Rust, itself a deny target) | `_upstream*`, `_upstream_containerd`, `third-party`, `vendor`, `target`, `buck-out`, `prelude`, `toolchains`, `__pycache__`, `.omc`, `.omx`, `.claude`, `talos-reference` |

**Total across all lanes: 235 first-party Rust crates** (19 + 3 + 3 + 1 + 1 + 208).

linux-stack sub-breakdown (workspaces declare 202 members; kernel tree has 18 manifests on disk):
- **kernel** (Cargo workspace): 18 crates — `kernel`, `hal`, `arch-aarch64`, `arch-x86_64`, `frame`, `ksync`, `user_layout`, `arch-aarch64-layout-tests`, + 9 `user-*`/`*-src` ELF/worker/host-test crates. DENY: `target/`, `.omc/`, `out/`.
- **kubernetes** (Cargo workspace): 139 crates under `crates/` (= 44 `ctrd_*` + 95 non-`ctrd_*`; see §3). DENY: `_upstream/`, `_upstream_containerd/`, `third-party/`, `prelude/`, `toolchains/`, `target/`, `buck-out/`, `__pycache__/`, `.omc/`, `.omx/`.
- **operating-system** (Cargo workspace): 45 crates (44 declared members incl. `difftest`; +root). All `talos-*` + `difftest`. DENY: `target/`, `buck-out/`, `toolchains/`, `.omc/`, `.omx/`, `.claude/`; non-crate dirs `boot/`, `platforms/`, `tools/`.
- **kernel-usermode-tests** (standalone, no shared workspace): 6 crates — `init`, `exec`, `hello`, `signal`, `spawn`, `clock`. No DENY dirs present locally.
- **talos-reference**: Go module (`go.mod`/`go.work`), itself a DENY target; **0 first-party Rust**.

**Structural facts that drive D-CONFORM:** linux-stack has NO top-level Cargo workspace (meta-tree). claude + codex are single-crate (no workspace). The D-CONFORM "collapse-2-STD" applies to the two STD workspaces `operating-system` + `kubernetes` (→ one consolidated STD root); `kernel` is the excluded no_std subtree (§5).

---

## 3 — k8s / containerd 139-crate SPLIT + cloud-k8s relationship (G4)

**Split counts (triangulated 3 ways — `ls crates/`, `[workspace].members`, prefix greps — all agree):**

| Bucket | Count | Definition |
|---|---|---|
| **k8s-MERGE** | **95** | non-`ctrd_*` — apimachinery/api/serializer + the 7 `cv_*` core/v1 proto-split crates + `cri_api_v1` + all apimachinery/api/serializer crates |
| **containerd-CREATE** | **44** | `ctrd_*` prefix (Go→Rust containerd package ports; incl. `*2` extension crates and `*_darwin`/`*_windows` per-OS variants) |
| **vendored-exclude** | **0** | no vendored crate is a workspace member (`_upstream/`, `_upstream_containerd/`, `third-party/` live OUTSIDE the 139) |
| **TOTAL** | **139** | 44 + 95 = 139, reconciles exactly; every dir has a `Cargo.toml` |

**cloud-k8s relationship (G4) — evidence:** `/Users/jasonlee/Developer/source/cloud/cloud-k8s` is **NOT** a Rust crate workspace — empty `crates/`, no `Cargo.toml` anywhere. It is a docs/spec/governance service; `manifest.json` declares bounded_context `cloud-compute` with DDD crates `oya-cloud-compute-{domain,functions-api,k8s-api,vm-api,adapter-aws,adapter-oci}`, and it explicitly states it does NOT implement live k8s bootstrap/CNI/apiserver/REST/SDK. The 4 `managed-k8s-*` siblings DO have `crates/`, but they are `oya-managed-k8s-*` DDD-layer crates (`-kernel/-app/-api/-adapter-cedar/-adapter-inmemory`) with **zero naming/path overlap** with the 139 upstream-port crates. cloud-k8s/managed-k8s = a separate cloud control-plane/governance layer that *consumes* k8s; not a 6th copy.

**Disposition options (analyst recommendation = #1):**
1. **Out-of-scope** of the 139-crate split — split lane stays confined to `stack/kubernetes/crates`. **(RECOMMENDED)**
2. **Docs-only cross-link** — record a reachability edge documenting `oya-cloud-compute-k8s-api` as a *consumer* of the merged k8s crates; no code merge.
3. **6th merge target** — contradicted by evidence (different `oya-*` DDD naming, no Cargo workspace, explicitly non-runtime scope); listed for completeness only.

→ **Still needs founder input** (see §7): option (1) out-of-scope vs (2) docs-only cross-link.

---

## 4 — Merge-surfaces (L5 codex / L6 managed-k8s)

### (a) L5 — codex-adapter
- **Target EXISTS, populated:** `/Users/jasonlee/Developer/source/cloud/cloud-intelligence/crates/oya-cloud-intelligence-codex-adapter` — 1 crate, `src/lib.rs` = **942 LOC**, BUCK + Cargo.toml + `tests/d3_codex_adapter_integration.rs` (8 `#[tokio::test]` httpmock tests). One of 8 `oya-cloud-intelligence-*` siblings. Owns ADR-0384 Path B "D3."
- **Target surface:** server-side HTTP reverse-proxy of an OAuth subscription pool. `CodexAdapter` (refresh ChatGPT session cookie→bearer, stream-proxy `/backend-api/codex/responses`), `OpenAiApiKeyAdapter`, `CodexAdapterError`, `CodexTokens`, `CodexProxyRequest/Response`, `HOP_BY_HOP`; hard-coded `CLI_VERSION = "cli/0.27.0"`. Deps: reqwest 0.13, tokio, bytes, the CI kernel.
- **Source surface:** `/Users/jasonlee/Developer/codex/sdk/rust` = pkg `openai-codex-sdk` (lib `openai_codex_sdk`), ~3.9k LOC / 13 modules. Client-side **CLI JSONL transport** — spawns `codex exec --experimental-json`, parses stdout. Exports `Codex/Thread/Turn/ThreadEvent/*Item`, app-server JSON-RPC layer, options. Deps: serde/serde_json/tempfile + optional tokio; **no HTTP client**.
- **DELTA = complement, not overlay.** Disjoint abstractions (outbound HTTP proxy vs local child-process CLI spawner), ~zero type/dep overlap, grep finds **zero** SDK references anywhere in cloud-intelligence. Recommend vendoring the SDK as a NEW sibling crate (e.g. `oya-cloud-intelligence-codex-sdk`), NOT folding it into the 942-LOC proxy adapter.

### (b) L6 — managed-k8s
- **All 4 targets EXIST, populated — 17 crates total** (no root cargo workspace; each is a hexagonal service dir):
  - cluster-lifecycle: 3 (kernel 1005 LOC / api 289 / app 368)
  - control-plane-host: 5 (kernel, api, adapter-capi, adapter-inmemory, app)
  - sla-observability: 4 (kernel, api, adapter-inmemory, app)
  - tenant-quota: 5 (kernel, api, adapter-cedar, adapter-inmemory, app)
- **Target surface:** product control-plane domain crates (serde-only kernels) — e.g. `ClusterLifecycleState` state machine, `evaluate_drain_admission`, tenant-quota `QuotaDecision` port gating control-plane-host provisioning. Live k8s/Prometheus/CAPI integration deliberately deferred behind ports.
- **Source surface:** `/Users/jasonlee/Developer/linux/stack/kubernetes/crates` = the low-level apimachinery substrate (~95 upstream-shaped k8s crates + ~44 `ctrd_*` = L7).
- **DELTA = vertical layering, not 1:1.** The 95-crate apimachinery substrate sits *under* the 17 product control-plane crates; no shared names, no existing target→source dep. Per ADR-0015/0016 the substrate becomes a NEW `oya-cloud-k8s-*` family whose canonical home is `managed-k8s-control-plane-host/` — so the bulk maps to **control-plane-host**, NOT evenly across all 4 services.
- **6th-surface caveat:** `cloud/cloud-k8s` EXISTS but is **docs/design-only — 0 crates** (no `crates/` dir); it is the design SSOT (ARCH/PRD/IPs/cedar/contracts) for that `oya-cloud-k8s-*` family. Cross-refs §3 / `10-k8s-split.md`.

---

## 5 — The 12-entry no_std kernel-subtree EXCLUDE list (feeds pre-lane 0.6)

**Authority:** D-CONFORM "exclude-the-12-kernel-subtree" (`UNIFIED-EXECUTION-PLAN.md:110`). Each is a self-contained `[workspace]` root; none can be absorbed by the consolidated STD root. Paths relative to repo root `stack/`:

```toml
exclude = [
    "kernel",                                          # framekernel virtual workspace root (7 members)
    "kernel/crates/arch-x86_64/user-src",              # user-hello-x86_64
    "kernel/crates/arch-x86_64/user-spawn-src",        # user-spawn-x86_64
    "kernel/crates/arch-x86_64/user-exec-src",         # user-exec-x86_64
    "kernel/crates/arch-x86_64/user-signal-src",       # user-signal-x86_64
    "kernel/crates/arch-x86_64/user-clock-src",        # user-clock-x86_64
    "kernel/crates/arch-x86_64/user-smpdemo-src",      # user-smpdemo-x86_64
    "kernel/crates/arch-x86_64/user-fsbase-src",       # user-fsbase-x86_64
    "kernel/crates/arch-x86_64/user-init-src",         # user-init-x86_64
    "kernel/crates/arch-aarch64/user-smpdemo-src",     # user-smpdemo (aarch64)
    "kernel/crates/arch-x86_64/fsbase-worker-src",     # fsbase-worker-x86_64 (std/musl, Docker)
    "kernel/crates/arch-aarch64/tests-host",           # arch-aarch64-layout-tests (host std libtest)
]
```

**Count reconciles:** framekernel (1) + 9 `user-*-src` ELF targets (rows 2–10) + fsbase-worker-src (1) + tests-host (1) = 12.

**Build-isolation evidence:**
- Rows 1–10 build on the kernel's OWN `nightly-2026-02-28` (`stack/kernel/rust-toolchain.toml`) targeting bare-metal triples `aarch64-unknown-none-softfloat` / `x86_64-unknown-none` with `-Z build-std` — never the STD root's stable toolchain. Each user-ELF carries its own `.cargo/config.toml` (user.ld @ 0x40_0000, static, no-pie) + an empty `[workspace]` table to detach.
- Row 11 (fsbase-worker): std/musl, built INSIDE Docker (`rust:alpine --platform linux/amd64`) with its own toolchain; no `.cargo/config.toml`; detached `[workspace]`.
- Row 12 (tests-host): host `std` libtest crate; own `.cargo/config.toml` clears `build-std`, sets `target=aarch64-apple-darwin`; detached `[workspace]`.

**Caveat (carried for 0.6 inertness proof):** the kernel ROOT `Cargo.toml`'s *current* `[workspace] exclude` self-lists only 8 of the nested children (intra-kernel hygiene). The consolidated STD root must list all 12 (the whole `stack/kernel` subtree). Excluding `kernel` alone keeps cargo out of the entire subtree; the 11 nested paths are enumerated for inert-proof completeness. **Pre-lane 0.6 must prove the full 12-entry kernel-exclude inert.**

---

## 6 — `oya-ci-required` characterization + DOC-CATALOG/CHANGELOG location

### (a) `oya-ci-required` producer
- Crate **`oya-ci-controller-kernel`** defines `pub const GATE_CONTEXT: &str = "oya-ci-required"` at `/Users/jasonlee/Developer/source/oya/ci-controller/crates/oya-ci-controller-kernel/src/lib.rs:471` (pure state machine `map_job_to_status`, `#![forbid(unsafe_code)]`, no I/O).
- Value is **posted as a Forgejo commit-status** by the I/O adapter **`oya-ci-controller-forgejo-adapter`** (`POST /api/v1/repos/<owner>/<repo>/statuses/<sha>`, accepts 200/201), driven by the kube-rs controller binary **`oya-ci-controller-app`** (watches K8s gate Jobs, posts pending→terminal, crier-style). Job-spawn via `oya-ci-controller-k8s-adapter`.
- Physically under **`oya/ci-controller`** (NOT a `cloud-ci` dir — none exists; ADR-0513's "cloud-ci/oya-ci" is the logical product name).
- Posts **exactly ONE** context: `oya-ci-required` (legacy `oya-ci-gate` is bridge-feedback only, not merge authority). Kernel also knows a sibling `cloud-ci-required` via `PHASE0_REQUIRED_CI_CONTEXTS` (`:171`) but the merge gate is the single `oya-ci-required`.
- Verdict mapping is **fail-closed + last-write-wins** on `(sha, context)` → **flip-safe**.

**Flip-readiness gap:** live `dev` required-context is still the LEGACY gate — `gh api repos/jason931225/oyatie/branches/dev/protection/required_status_checks/contexts` → `["github-lane-unlocker-required"]`. Live monorepo is `jason931225/oyatie` (`jason931225/source` + `jason931225/oya` 404). The FLIP is `github-lane-unlocker-required` → `oya-ci-required` (+ signing), gated on `oya-ci-controller-app` going live. Authority = **ADR-0513** (Accepted/founder-locked 2026-05-30). Branch-protection target/shadow: `/Users/jasonlee/Developer/source/.github/branch-protection.yaml:55-56` (`dev.required_status_checks: [oya-ci-required]`); drift-from-live in `specs/phase0-ci-enforcement-baseline.json`. `oya-pr-review` intentionally absent from required (producer HTTP 501; would deadlock).

### (b) DOC-CATALOG / CHANGELOG location — `docs/`, NOT root
- Canon: `/Users/jasonlee/Developer/source/docs/DOC-CATALOG.md` (47 KB) + `/Users/jasonlee/Developer/source/docs/CHANGELOG.md` (48 KB). **No** root-level copies exist.
- Amendment lanes add rows INTO these `docs/`-rooted files (one-doc-per-PR: DOC-CATALOG row + CHANGELOG row ride with the triggering doc — `AMENDMENT-PLAN.md:36,151,227,255`; CC-3 reclassifies `MASTERPLAN.md → GENERATED-REFERENCE` in DOC-CATALOG).
- Enforced by two gate-internal fitness lanes: `oya-governance-doc-catalog` (BLOCKER), `oya-governance-changelog-row` (HIGH).

**Flag (path drift — evidence, not fixed):** the `doc-catalog` fitness-lane spec reads `docs/CATALOG.md`, but the real file is `docs/DOC-CATALOG.md` (`docs/CATALOG.md` does not exist). Reconcile in the conformance/amendment lane. Both governance `tools/` runners are also G2 standing-exception candidates (§1).

---

## 7 — G2 / G4 DECISIONS SURFACED FOR THE FOUNDER

WIP gates per `UNIFIED-EXECUTION-PLAN.md §11`: **G2** = tools/ standing-exception ratify · **G4** = db-engine/cloud-k8s/codename/no_std-inertness confirmations. (G0 authority-flip HALT, G1 github-mirror push creds, G3 signing = DONE — not in this bundle's scope.)

### ANSWERED-BY-THESE-MANIFESTS (evidence resolved; founder ratification only)
- **G2 — `tools/` standing-exception set:** fully enumerated (§1) — 22 BUCK targets + governance/hooks shell harnesses + buck launcher infra + 1 reserved-absent name; `//services/...` = ∅. **Founder action: ratify the set** (evidence is complete; no ambiguity).
- **G4 — no_std-inertness inputs:** the 12-entry kernel-exclude list is complete and evidenced (§5). Final **inertness PROOF is pre-lane 0.6's job** (run-level), not a founder decision — but the list it must prove inert is locked here.
- **G4 — codename/surface ambiguity (resolved to evidence):** "github-lane-unlocker workflow" does not exist as a GHA workflow (it is the legacy *required-context name*); the real gate is Jenkins→`oya-ci-required` (§1, §6). "cloud-ci/oya-ci" is a logical product name; live producing crates are under `oya/ci-controller` (§6). `doc-catalog` spec path-drift `docs/CATALOG.md` vs actual `docs/DOC-CATALOG.md` flagged for the conformance lane (§6). These are **documented, not open** — no founder input needed beyond acknowledging the reconciliations land in the amendment/conformance lane.

### STILL-NEED-FOUNDER-INPUT (not resolvable by evidence alone)
- **G4-D1 — cloud-k8s 6th-surface disposition:** option (1) **out-of-scope** of the 139-split (RECOMMENDED) vs option (2) **docs-only cross-link** (`oya-cloud-compute-k8s-api` documented as consumer). Option (3) 6th-merge-target is evidence-contradicted. (§3 / `10-k8s-split.md`.) **Decision needed.**
- **G4-D2 — db-engine / drop-L8 confirm:** `UNIFIED-EXECUTION-PLAN.md:63` marks **L8 cloud-data/db-engine as CONDITIONAL — "dropped if source absent, WIP G4"**, and `:90` lists "db-engine … confirmations" under G4. **None of the six pre-lane-0.5 manifests inventoried a db-engine source** (it was out of their scope — the source-inventory lane covers office/oyago/oyapy/claude/codex/linux-stack only). So db-engine source presence is **unconfirmed by this bundle**. **Founder/G4 decision needed: confirm a db-engine source exists (→ keep L8) or drop L8.** (Recommend a targeted source-presence check before ratifying.)
- **L6-D1 — codex SDK vendoring shape:** keep codex-adapter (proxy/pool) and `openai-codex-sdk` (CLI-embedding) as separate concerns — vendor the SDK as a NEW sibling crate rather than folding into the 942-LOC adapter. Recommended YES; **founder confirm.** (§4a)
- **L6-D2 — managed-k8s substrate home:** confirm the ~95 k8s crates land as `oya-cloud-k8s-*` under **`managed-k8s-control-plane-host`** (per ADR-0015/0016), NOT spread across all 4 product services; and resolve whether docs-only `cloud/cloud-k8s` becomes the **6th merge target** (receives the substrate crates) or **stays design-SSOT** while crates land under control-plane-host. **Founder confirm** (couples with G4-D1). (§4b)

---

## Evidence index (absolute paths)
- `…/_execution/prelane-0.5/10-tools-targets.md` · `10-source-inventory.md` · `10-k8s-split.md` · `10-merge-surfaces.md` · `10-kernel-exclude.md` · `10-gate-characterize.md`
- `/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/UNIFIED-EXECUTION-PLAN.md` (`:63` L8-conditional, `:90` G0–G4, `:110` exclude-the-12)
- `/Users/jasonlee/Developer/source/oya/ci-controller/crates/oya-ci-controller-kernel/src/lib.rs:171,471,484,982`
- `/Users/jasonlee/Developer/source/.github/branch-protection.yaml:55-56`
- `/Users/jasonlee/Developer/source/docs/DOC-CATALOG.md`, `/Users/jasonlee/Developer/source/docs/CHANGELOG.md`
- `/Users/jasonlee/Developer/source/docs/decisions/ADR-0709-general-live-apex.md`
- `/Users/jasonlee/Developer/linux/stack/kubernetes/Cargo.toml` (139 members)
- `/Users/jasonlee/Developer/source/cloud/cloud-k8s/manifest.json` (bounded_context `cloud-compute`, 0 crates)
- `libs/oya-governance-gate-catalog-domain/src/lib.rs:256-265` (gate command roster)
