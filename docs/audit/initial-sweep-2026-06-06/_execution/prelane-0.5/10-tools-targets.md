# 10 — G2: `//tools/...` + `//services/...` gate-load-bearing target set (standing canonical-homes exception)

Pre-lane 0.5 manifest lane. **READ-ONLY.** Evidence: `source/.github/workflows/`, `source/Jenkinsfile`, `source/.github/branch-protection.yaml`, `tools/` + `services/` dir listings, BUCK files, `libs/oya-governance-gate-catalog-domain/src/lib.rs`, `registry/`, live `gh api`.

---

## TL;DR — the standing exception set (what retirement MUST EXCLUDE)

**KEEP (gate-load-bearing, retain `//tools/...`):** the **22 BUCK-bearing `tools/oya-*-app` / `tools/oya-tooling-*` crates** below + the **`tools/governance/` + `tools/hooks/`** shell harnesses they shell into. These are the live merge-gate's governance fitness lanes.

**`//services/...` exception set: EMPTY.** `source/services/` exists but contains **zero BUCK and zero Cargo.toml** targets — it is not built by any live gate. No `//services/...` build target exists to except. (The buildable service binaries live under `oya/<domain>/` and `microservices/<ms>/`, not `services/`.)

**Two WIP-plan-named tools do NOT exist anywhere** (not in `tools/`, not under any Cargo.toml): `oya-doc-staleness-inventory-app`, `oya-adr-index-regenerator-app`. They are aspirational/planned, **not** gate-load-bearing today → nothing to except, but they are reserved names that will land under `tools/` (canonical home) when built.

---

## How the live merge gate actually builds tools/ (correcting the WF spec's framing)

The WF lane was told to read `the github-lane-unlocker workflow` in `source/.github/workflows/*.yml`. **That workflow does not exist there.** Findings:

- `source/.github/workflows/` contains exactly **one** file: `backbone-microservices-ci.yml`. It is **100% cargo-based** (`cargo fmt/check/clippy/test` over a 4-microservice package matrix + a `governance-smoke` job). It builds **no** `//tools/` or `//services/` Buck target. Its only tool invocation is `./bin/oya gate validate cargo-prefix` (line 313) → runs in-process via `oya-dev-cli`.
- The **live** required context on `dev` is **`github-lane-unlocker-required`** (`gh api repos/jason931225/oyatie/branches/dev/protection/...` → `["github-lane-unlocker-required"]`, per the sibling `10-gate-characterize.md`). This is the **legacy** gate; the GHA workflow that posted it was **retired** (`Jenkinsfile` preamble: "the canonical repo-wide gate that replaces the retired GitHub Actions workflows"). The FLIP target is `oya-ci-required` (ADR-0513, in flight).
- The actual repo-wide gate is **Jenkins** (`source/Jenkinsfile` → `oyaCiLane(service: 'repo')`, "the whole oya gate set + cargo mirror, affected-scoped on PR") fanning out to `microservices/<ms>/ci/Jenkinsfile`. The "oya gate set" is the catalog below.
- The gate is a **Buck2 affected-target gate** running governance fitness lanes (per `10-gate-characterize.md`). So **every BUCK target under `tools/` is gate-built when its affected scope is touched** — that is why the whole BUCK-bearing `tools/` set is the standing canonical-homes exception.

**Dispatch architecture (important for retirement reasoning):** the `oya gate validate <lane>` lanes are dispatched **in-process by `oya-dev-cli`**, which depends directly on the `libs/oya-governance-*-kernel` crates (confirmed in `oya-dev-cli`'s BUCK: deps `//libs/oya-governance-banned-primitives-kernel`, `//libs/oya-governance-gate-catalog-domain`). The `tools/oya-governance-*-app` binaries are **standalone wrappers around those same kernels**. The authoritative lane roster is `AGGREGATED_VALIDATE_LANES` (180 lines, ~130 lanes) + `AGGREGATED_NON_GATE_COMMANDS` in `libs/oya-governance-gate-catalog-domain/src/lib.rs`.

---

## The KEEP-set: 22 BUCK-bearing `tools/` crates (Buck target = `//tools/<name>:<name>`, all `rust_binary`, `visibility=["PUBLIC"]`)

All confirmed to have BOTH `BUCK` and `Cargo.toml`. Target label form verified from sample BUCK (e.g. `//tools/oya-governance-adr-shape-app:oya-governance-adr-shape-app`).

### Explicitly invoked by name in the gate command list (`AGGREGATED_NON_GATE_COMMANDS`, lib.rs:262-265) — hardest KEEP
| Buck target | Cargo pkg | Gate invocation (lib.rs) |
|---|---|---|
| `//tools/oya-vcs-admission-gate-app` | `oya-vcs-admission-gate-app` | `cargo run -q -p oya-vcs-admission-gate-app` (:262) |
| `//tools/oya-vcs-provider-execution-gate-app` | `oya-vcs-provider-execution-gate-app` | `cargo run -q -p oya-vcs-provider-execution-gate-app -- --mode ci --emit-evidence …` (:263) |
| `//tools/oya-governance-purpose-audit-app` | `oya-governance-purpose-audit-app` | `cargo run -q -p oya-governance-purpose-audit-app` (:264) |

### The `oya-governance-*-status-lifecycle-app` + sibling governance fitness apps (Buck-affected-target gate set)
| Buck target |
|---|
| `//tools/oya-governance-adapter-with-no-importer-app` |
| `//tools/oya-governance-adr-shape-app` |
| `//tools/oya-governance-adr-status-lifecycle-app` |
| `//tools/oya-governance-api-stability-tier-lifecycle-app` |
| `//tools/oya-governance-authoritative-tracked-app` |
| `//tools/oya-governance-banned-primitives-app` |
| `//tools/oya-governance-capability-status-lifecycle-app` |
| `//tools/oya-governance-crate-status-lifecycle-app` |
| `//tools/oya-governance-dependency-status-lifecycle-app` |
| `//tools/oya-governance-doc-status-lifecycle-app` |
| `//tools/oya-governance-feature-flag-status-lifecycle-app` |
| `//tools/oya-governance-migration-status-lifecycle-app` |
| `//tools/oya-governance-plan-status-lifecycle-app` |
| `//tools/oya-governance-portfolio-citation-app` |
| `//tools/oya-governance-predictable-naming-app` |
| `//tools/oya-governance-sunset-lifecycle-app` |

### Adjacent BUCK-bearing tools (gate/agent tooling)
| Buck target | Note |
|---|---|
| `//tools/oya-adapter-substitution-test-app` | hexagonal-seam substitution test (gate-built) |
| `//tools/oya-tooling-agent-read` | agent read surface tooling |
| `//tools/oya-xtask-metadata-augment-app` | xtask/workspace-metadata augmentation |

→ **22 BUCK targets total** (3 explicit + 16 governance fitness + 3 adjacent).

---

## KEEP — non-Buck shell harnesses the gate shells into (retain even though not Buck targets)
`AGGREGATED_NON_GATE_COMMANDS` (lib.rs:256-259) shells into `tools/governance/adr-0221-governance-gates.sh {vacuous-green, orphan-citation, version-pin, buildability-line-count}`, which in turn runs `tools/hooks/*.sh` (e.g. `tools/hooks/vacuous-green-gate-detect.sh`, `tools/hooks/adr-orphan-detect.sh`). These are gate-load-bearing scripts, not Buck targets, but **must NOT be retired**:

- `tools/governance/` (`adr-0221-governance-gates.sh`)
- `tools/hooks/` (the hook scripts the harness drives)

The 9 **non-Buck `tools/` infra dirs** (no BUCK, no Cargo.toml): `agent-skills`, `anchor-sweep`, `buck`, `buck2`, `completions`, `governance`, `hook-bootstrap`, `hooks`, `opensk-vendored`. Of these, **`governance` + `hooks` are gate-load-bearing (KEEP)**; `buck`/`buck2` are toolchain infra (KEEP — Buck launcher home); the rest are tooling-support dirs (not gate-required, but are tooling canonical homes — out of retirement scope as `tools/` is a standing canonical home).

---

## REFERENCED-BUT-ABSENT (gate command exists, directory does NOT — placeholder-debt, reserve the name)
`oya-vcs-merge-queue-fix-loop-app` is invoked by the gate (`AGGREGATED_NON_GATE_COMMANDS` lib.rs:265: `cargo run -p oya-vcs-merge-queue-fix-loop-app -- --gc-staging-refs --max-age-seconds 3600`; also `registry/quality/lanes.yaml:607`, `registry/vcs/event-router.yaml:43,72`, `registry/merge-queue-tick-log.json:12`). Its canonical home is declared as **`tools/oya-vcs-merge-queue-fix-loop-app/src/`** (`registry/vcs/concurrent-safe-paths.yaml:17`), but **no such directory or Cargo.toml exists on disk today** (ADR-0111 scaffolding/placeholder-debt). → Not a current build target to except, but its `tools/` slot is reserved; do not let retirement re-use or block the name.

---

## `//services/...` verdict: NO exception (empty set)
- `source/services/` subdirs: `analytics`, `app-shell-frontend`, `ci-webhook-gateway`, `policy`, `treasury`.
- `find services -maxdepth 3 \( -name BUCK -o -name Cargo.toml \)` → **0 results**. No Buck targets, no cargo crates. The live gate builds **nothing** under `//services/...`.
- Buildable analogues live elsewhere (`oya/<domain>/`, `microservices/<ms>/`) and are governed by their own lanes, not via `//services/`.
- **G2 `//services/...` standing-exception set = ∅.**

---

## The two WIP-plan-named tools (verification)
- `oya-doc-staleness-inventory-app` — `find` across repo: **not present** (no dir, no Cargo.toml). Not gate-load-bearing today.
- `oya-adr-index-regenerator-app` — **not present**. Not gate-load-bearing today.
- Both are planned tools whose canonical home will be `tools/`; reserve the names, nothing to except now.

---

## RETURN — the tools/ standing canonical-homes exception set (G2)

**EXCLUDE FROM RETIREMENT (the standing exception):**
1. **22 BUCK targets** `//tools/oya-*-app` + `//tools/oya-tooling-agent-read` (full list above) — the live Buck-affected-target governance gate set; 3 are invoked by literal `cargo run -p` in the gate command roster, the rest are gate-built when affected.
2. **Gate shell harnesses:** `tools/governance/` (`adr-0221-governance-gates.sh`) + `tools/hooks/` (the hook scripts it drives) — shelled into by the gate (`AGGREGATED_NON_GATE_COMMANDS`).
3. **Buck launcher infra:** `tools/buck`, `tools/buck2`.
4. **Reserved (absent) name:** `tools/oya-vcs-merge-queue-fix-loop-app` — gate command + registry reference it; directory not yet materialized (placeholder-debt) → reserve, don't reuse/block.

**NO `//services/...` exception** — `source/services/` has zero build targets (empty set).

**Authority chain:** Jenkins `oyaCiLane(service:'repo')` → `oya gate` (in-process via `oya-dev-cli` over `libs/oya-governance-*-kernel` + `oya-governance-gate-catalog-domain`) + literal `cargo run -p <tools-app>` + `bash tools/governance/*.sh`. Live required context = `github-lane-unlocker-required` (legacy, pre-flip); flip target = `oya-ci-required` (ADR-0513). All of the above are **standing canonical-homes** under `tools/` and are out of retirement scope regardless of affected-scope on any given PR.
