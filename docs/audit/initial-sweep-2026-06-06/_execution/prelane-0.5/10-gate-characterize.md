# 10 — Gate Characterization: `oya-ci-required` producer + DOC-CATALOG/CHANGELOG location

Pre-lane 0.5 manifest lane. **READ-ONLY.** Evidence: live `gh api`, source files, branch-protection.yaml, ADR-0513.

---

## TL;DR (the two RETURN answers)

1. **`oya-ci-required` producer:** crate **`oya-ci-controller-kernel`** (defines `pub const GATE_CONTEXT: &str = "oya-ci-required"`) at
   `/Users/jasonlee/Developer/source/oya/ci-controller/crates/oya-ci-controller-kernel/src/lib.rs:471`.
   The kernel is the pure state machine (`map_job_to_status`); the value is **posted as a Forgejo commit-status** by the I/O adapter
   **`oya-ci-controller-forgejo-adapter`** (`POST /api/v1/repos/<owner>/<repo>/statuses/<sha>`), driven by the controller binary
   **`oya-ci-controller-app`** (kube-rs controller that watches K8s gate Jobs).
2. **DOC-CATALOG / CHANGELOG location:** **`docs/` (NOT repo root).** Canonical files are
   `/Users/jasonlee/Developer/source/docs/DOC-CATALOG.md` and `/Users/jasonlee/Developer/source/docs/CHANGELOG.md`.
   There are **no** root-level `DOC-CATALOG.md`/`CHANGELOG.md`. Amendment lanes add their rows **into these `docs/`-rooted files**
   (DOC-CATALOG row + CHANGELOG row ride with the doc that triggers them — one-doc-per-PR).

---

## (a) The FLIP target: `oya-ci-required` producer + what it posts/checks

### Producer crate (the post-er)

`/Users/jasonlee/Developer/source/oya/ci-controller/` — 4 crates under `crates/`:

| Crate | Role | Evidence |
|---|---|---|
| **`oya-ci-controller-kernel`** | Pure domain. Owns `GATE_CONTEXT = "oya-ci-required"`, the `ForgejoState` vocabulary (pending/success/failure/error), and the TOTAL function `map_job_to_status(obs, grace_cycles) -> ReconcileDecision`. `#![forbid(unsafe_code)]`, no I/O. | `kernel/src/lib.rs:471` `GATE_CONTEXT`; `:484` `map_job_to_status`; test `:982` asserts `GATE_CONTEXT == "oya-ci-required"` |
| **`oya-ci-controller-forgejo-adapter`** | I/O seam impl of `ForgejoStatusPoster`. POSTs the commit-status to `POST /api/v1/repos/<owner>/<repo>/statuses/<sha>` with `Authorization: token …`; accepts HTTP 200/201 (mirrors the legacy Jenkinsfile `postForgejoStatus`). | `forgejo-adapter/src/lib.rs:95-122` (`ForgejoStatusBody { state, context, description, target_url }`, `.post(self.statuses_url(sha))`) |
| **`oya-ci-controller-k8s-adapter`** | kube-rs `JobSpawner` — spawns the trusted K8s gate Job. Header comment warns it must not change the status context, producer, or branch-protection mapping. | `k8s-adapter/src/lib.rs:28` |
| **`oya-ci-controller-app`** | The binary/controller. "ONLY WATCHES Jobs and posts Forgejo commit statuses (crier-style)." Reconcile loop posts pending then terminal status; benign re-posts (Forgejo statuses are last-write-wins on `(sha, context)`). | `app/src/lib.rs:84,217,232,263`; bin at `app/src/bin/oya-ci-controller.rs` |

**Location answer for the WF spec's "ci-controller/... or cloud-ci":** it is in **`oya/ci-controller`** (the Phase-1 bespoke-Prow plank+crier), NOT a `cloud-ci` dir (no `source/oya/cloud-ci` exists). Note ADR-0513's Amendment-2026-06-02 says the merge-queue/tide admission contract is *owned in* "cloud-ci/oya-ci" — that is the logical product name; the live producing crates today are physically under `oya/ci-controller`.

### What it posts / checks (flip-readiness)

- **Exactly one context** is posted: `oya-ci-required` (the GATE_CONTEXT). Not `oya-ci-gate` (legacy bridge feedback, explicitly "not merge or Phase-0 exit authority" — kernel comment `:468-470`).
- **Verdict mapping** (`map_job_to_status`, all terminal descriptions prefixed `oya-ci-required …`):
  - Job Complete / `succeeded>=1` → `success` "oya-ci-required full gate target passed"
  - Job Failed `DeadlineExceeded` → `failure` (timeout); other Failed → `failure` "required gate exited non-zero"
  - Pod `OOMKilled` / `Evicted` → `failure`; `InvalidImageName` → `error` (operator-must-fix)
  - `ImagePullBackOff`/`ErrImagePull`/`CreateContainerError`/`CrashLoopBackOff` → bounded grace (`grace_cycles`, default 12 ≈ 2 min) then `failure`
  - Job GC'd before terminal posted → **fail-closed** `failure` "run disappeared"
  - Active/pending → `pending` "running trusted required gate target"
- **Fail-closed + last-write-wins** semantics = flip-safe: once `oya-ci-controller-app` is live and posting on the candidate SHA, the context can be made required without deadlocking (unlike `oya-pr-review`, which is HTTP 501 and intentionally absent from required checks — branch-protection.yaml `:51-54`).
- **Two-context phase-0 set defined but only one is the gate:** kernel `:171`
  `PHASE0_REQUIRED_CI_CONTEXTS = ["cloud-ci-required", "oya-ci-required"]` — the kernel knows about a sibling `cloud-ci-required`, but the merge gate context is the single `oya-ci-required`.

### Flip-readiness gap (current live state)

- **Live `dev` required-context is the LEGACY gate, not the target.** `gh api repos/jason931225/oyatie/branches/dev/protection/required_status_checks/contexts` → **`["github-lane-unlocker-required"]`**. (Repos `jason931225/source` and `jason931225/oya` return 404 — the live monorepo is **`jason931225/oyatie`**.)
- This confirms the FLIP is: **`github-lane-unlocker-required` → `oya-ci-required` (+ signing)**, exactly as the conformance register's B9 row and UNIFIED-EXECUTION-PLAN §0 describe (ADR-0513, in flight).
- The flip cannot be enacted until `oya-ci-controller-app` is deployed and posting `oya-ci-required` on candidate SHAs; until then `branch-protection.yaml` is a *target/shadow* record, NOT live enforcement (file's own preamble `:1-5,46-50`).

---

## (b) Does the live gate read ROOT vs `docs/` DOC-CATALOG.md/CHANGELOG.md?

### Physical file location: `docs/`, not root

| File | Path | Exists? |
|---|---|---|
| DOC-CATALOG | `/Users/jasonlee/Developer/source/docs/DOC-CATALOG.md` | YES (47 KB) |
| CHANGELOG | `/Users/jasonlee/Developer/source/docs/CHANGELOG.md` | YES (48 KB) |
| root `DOC-CATALOG.md` | `/Users/jasonlee/Developer/source/DOC-CATALOG.md` | **NO** |
| root `CHANGELOG.md` | `/Users/jasonlee/Developer/source/CHANGELOG.md` | **NO** |

DOC-CATALOG's own self-rows (`docs/DOC-CATALOG.md:175,177`) record `doc.changelog → CHANGELOG.md` and `doc.doc_catalog → DOC-CATALOG.md (this doc)` — relative to `docs/`. So the canon for both lives **under `docs/`**, and that is where amendment lanes add rows.

### What enforces the rows (the fitness lanes, not the merge gate directly)

The `oya-ci-required` gate is a single **Buck2-affected-target gate** (runs the affected build/test set); it does not itself special-case catalog files. DOC-CATALOG/CHANGELOG discipline is enforced by two **governance fitness lanes** that run inside the gate target:

| Lane (spec) | Runner | Reads | Severity |
|---|---|---|---|
| `oya-governance-doc-catalog` | `tools/oya-governance-doc-catalog` (kernel `oya-foundry-catalog-kernel`) | `docs/CATALOG.md`, `docs/**/*.md` | **BLOCKER** |
| `oya-governance-changelog-row` | `tools/oya-governance-changelog-row` (kernel `oya-governance-changelog-row-kernel`) | PR diff list, `docs/CHANGELOG.md`, canonical-path list | HIGH |
| Spec files | `/Users/jasonlee/Developer/source/docs/governance-lanes/doc-catalog.md`, `…/changelog-row.md` | | |

⚠ **Path-drift to flag (evidence, not a fix):** the `doc-catalog` fitness-lane spec declares its input as **`docs/CATALOG.md`**, but the real canonical file is **`docs/DOC-CATALOG.md`** (no `docs/CATALOG.md` exists). Both these governance-lane `tools/` runners are also in scope for the G2 tools/ standing-exception set (they are gate-load-bearing). The drift between the spec's `docs/CATALOG.md` reference and the actual `docs/DOC-CATALOG.md` should be reconciled by the conformance/amendment lane.

### Where amendment lanes add rows (from AMENDMENT-PLAN.md)

Per `…/initial-sweep-2026-06-06/AMENDMENT-PLAN.md` (`:36, :151 CC-3, :227, :255`):
- **One-doc-per-PR:** "DOC-CATALOG/CHANGELOG rows and glossary cascade **ride with the doc that triggers them**."
- Rows are added into the `docs/`-rooted DOC-CATALOG.md / CHANGELOG.md as part of the same PR that changes the canonical doc.
- CC-3 specifically reclassifies `MASTERPLAN.md → GENERATED-REFERENCE` *in DOC-CATALOG* — i.e. amendments edit the `docs/DOC-CATALOG.md` rows directly.

---

## Branch-protection config location (for the FLIP)

`/Users/jasonlee/Developer/source/.github/branch-protection.yaml` (target/shadow record; **drift from live is recorded in `specs/phase0-ci-enforcement-baseline.json`**).

- `branches.dev.required_status_checks: [ oya-ci-required ]` — the **target** state (`:55-56`).
- Live `dev` today still requires `github-lane-unlocker-required` (gh api, above) → the file is ahead of live; flip pending controller go-live.
- `oya-pr-review` intentionally **absent** from required (producer returns HTTP 501; would deadlock) — `:51-54`.
- staging/production: no PR/CI gate at those layers (gates fire on `dev`); signed-commits + linear-history + no-force-push only.
- ADR-0513 (`/Users/jasonlee/Developer/source/docs/decisions/ADR-0709-general-live-apex.md`, Accepted/founder-locked 2026-05-30) is the authority for the flip: phased replacement of ADR-0380 Jenkins+Groovy by bespoke-Rust oya-ci; Jenkins stays a hardened BRIDGE until Phase-1 cutover.

---

## Evidence index

- Producer: `source/oya/ci-controller/crates/oya-ci-controller-kernel/src/lib.rs:171,471,484,982`; `…-forgejo-adapter/src/lib.rs:95-122`; `…-app/src/lib.rs:84,217,232,263`; `…-k8s-adapter/src/lib.rs:28`.
- Live gate: `gh api repos/jason931225/oyatie/branches/dev/protection/required_status_checks/contexts` → `["github-lane-unlocker-required"]`.
- Branch protection: `source/.github/branch-protection.yaml:55-56` (target `oya-ci-required`), preamble `:1-5,46-54`.
- Catalog/Changelog: `source/docs/DOC-CATALOG.md` (rows `:175,177`), `source/docs/CHANGELOG.md`; fitness specs `source/docs/governance-lanes/{doc-catalog,changelog-row}.md`.
- Authority: `source/docs/decisions/ADR-0709-general-live-apex.md`.
- Amendment row-placement: `…/initial-sweep-2026-06-06/AMENDMENT-PLAN.md:36,151,227,255`.
