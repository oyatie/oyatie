# OMX Runtime Repair and Verification Plan — 2026-08-03

## Status

The active global installation is still the unchanged `oh-my-codex 0.20.3` package. The v2 npm-artifact repair reproduced and diagnosed the reported failures, but its local-evidence Ralplan consensus allowance is an unsafe prototype and must not ship.

| Surface | Current state | Evidence |
| --- | --- | --- |
| Active global OMX | Unchanged; read-only from this session | `/Users/jasonlee/.local/share/fnm/node-versions/v24.16.0/installation/lib/node_modules/oh-my-codex/package.json` reports `0.20.3` |
| v2 npm-artifact prototype | Diagnostic only; superseded for implementation | `.omx/tmp/omx-runtime-repair-20260803-v2/` |
| Fresh source repair | Created from official `v0.20.4` tag `a62d5bd`; repair/build in progress | `.omx/tmp/oh-my-codex-v0204-repair-20260803/` |
| Active installation | Not updated | Global prefix and `/Users/jasonlee/.codex` remain outside writable roots |
| v0.20.4 build and full suite | **PENDING** | Placeholders are listed under Final evidence |

Correction to the obsolete blocker report: the failure is not merely missing alias normalization. Official `v0.20.4` already recognizes flattened collaboration names, but intentionally refuses to treat local typed routing, trackers, or Architect/Critic artifacts as consensus authority. Without a documented, non-user-mintable official host receipt, production Ralplan must fail closed with `documented_host_consensus_receipt_unavailable`.

The supported recovery is to persist the exact blocker, terminalize Ralplan through an authorized terminal path, and process an explicit execution handoff. It is not valid to synthesize consensus from local lifecycle evidence. The v2 verifier's positive Ralplan → Ralph/Ultragoal consensus simulation is retained only as a state-transition diagnostic and is not shippable authorization evidence.

## Target outcome

Build OMX from the clean official `v0.20.4` source base, carry forward only repairs compatible with its authority contract, run preservation-aware setup, restart the native hook surface, and verify the focused and full workflow matrices without weakening planning, Ralplan consensus, or Conductor boundaries.

Official `v0.20.4` is the required base because its release targets native-hook trust, path canonicalization, Team/state-root behavior, and read-only discovery. The clean local worktree resolves the prior partial-source and source-map/build limitation. Its source already contains a finite flattened collaboration-name map, plus explicit tests for accepted names and near-miss rejection. Upgrade alone is still not completion evidence: the local repairs, exact Team informational guard, build, and full suite must be reviewed against `v0.20.4`.

The release contract explicitly states that typed routing and tracker evidence are lifecycle/diagnostic only. The reported relationship to [issue #3427](https://github.com/Yeachan-Heo/oh-my-codex/issues/3427) remains unverified because the issue page was unavailable through the cache and API/network path.

Authoritative local `v0.20.4` references:

- `.omx/tmp/oh-my-codex-v0204-repair-20260803/docs/adr/3212-same-user-native-child-auth-boundary.md`
- `.omx/tmp/oh-my-codex-v0204-repair-20260803/docs/contracts/ralplan-consensus-gate.md`
- `.omx/tmp/oh-my-codex-v0204-repair-20260803/docs/codex-native-hooks.md`
- `.omx/tmp/oh-my-codex-v0204-repair-20260803/src/leader/contract.ts`

## v2 diagnostic prototype

The v2 bundle remains useful as reproduction evidence, but the fresh `v0.20.4` source worktree supersedes it for implementation and release.

| Area | Repair | Preserved boundary |
| --- | --- | --- |
| Native tool aliases | Demonstrated a finite exact map for dotted, underscored, double-underscored, and flattened known collaboration operations | Superseded by the official `v0.20.4` finite map; unknown and near-miss transports must remain fail-closed |
| Ralplan and Ralph orchestration | Demonstrated shared classification of collaboration operations | Ralph/leader routing may use the repaired aliases; Ralplan spawn/lifecycle evidence must not become consensus authority |
| Unsupported native recovery | Admits session- and canonical-cwd-bound Ralplan terminal states `blocked`, `cancelled`, and `failed` when persisted unsupported-native evidence matches | Clean `complete` cannot release execution without official host-receipt verification; foreign evidence is denied |
| Consensus handoff simulation | Demonstrates state cleanup and destination Conductor ownership after a positive gate | **Unsafe as authorization proof:** the prototype used locally synthesized consensus that official `v0.20.4` intentionally rejects without a host receipt |
| macOS paths | Canonicalizes existing ancestors before root equality and containment checks, treating `/var/...` and `/private/var/...` as the same root | Paths outside the canonical root remain blocked |
| Git inspection | Positively classifies a narrow read-only Git grammar and verifies the resolved Git executable, PATH topology, file permissions, and `GIT_*` environment | Repository/PATH shadows, wrappers, unsafe environment overrides, mutation subcommands, output flags, and dynamic expansion remain blocked |
| Team outside tmux | Allows only exact `team --help` and `team -h` informational probes, including the direct Node CLI form | Team launch/status/version/compound commands remain blocked outside attached tmux; no Team state is created on refusal |
| `exec --help` | Passes directly to `codex exec --help` before setup/session side effects | No `.omx` directory or `CODEX_HOME` artifact is created by the help probe |
| Doctor skills | Counts symlinked skill directories by following their directory targets | Broken/non-directory entries are not counted as installed skills |
| Doctor Git objects | Does not report non-writability for immutable object files under `.omx/worktrees/<lane>/.git/objects/**` | Ownership mismatches are still reported; the repair does not chmod Git objects |

The exact prototype inventory and limitations are recorded in `.omx/tmp/omx-runtime-repair-20260803-v2/REPAIR-MANIFEST.md`. New implementation evidence belongs to `.omx/tmp/oh-my-codex-v0204-repair-20260803/`.

## Focused verification

All commands below were executed against the isolated v2 package unless noted.

| Verification | Result |
| --- | --- |
| `node verify-runtime-repair.mjs` | PASS as a diagnostic prototype: finite aliases, unknown fail-closed, Git read-only/compound inspection, unsupported-native recovery, handoff state simulation, outside-tmux Team refusal, `exec --help`, and symlinked Doctor skills |
| Hook syntax: `codex-native-hook.js`, `codex-native-pre-post.js` | PASS |
| CLI syntax: `index.js`, `doctor.js` | PASS |
| Exact Team help + macOS path focused compiled tests | 2 passed, 0 failed |
| `exec` plus Doctor artifact-ownership compiled tests | 14 passed, 0 failed |
| Doctor artifact-ownership tests alone | 6 passed, 0 failed, including immutable worktree Git objects |
| State operations | Passed in the earlier focused run |
| Team Doctor | All available Team diagnostics passed in the earlier focused run |

The Ralph compiled persistence command produced 92 passes and 4 failures. All four failures are package-completeness checks caused by the partial v2 npm artifact (`src/mcp/trace-server.ts`, Ralph reference/contract documents, and `.github/workflows/ci.yml`), not failing Ralph state or persistence assertions. The source-complete `v0.20.4` worktree supersedes this packaging limitation; its replacement build/test result is pending.

## Broad workflow and skill matrix

This matrix records tested behavior without treating missing npm-source artifacts or unavailable live runtimes as product regressions.

| Surface | Result | Qualification |
| --- | --- | --- |
| Ralplan | Prototype focused verifier PASS | Diagnostic only; official `v0.20.4` correctly fails closed without a host receipt; fresh-source suite pending |
| Ralph | 92 prototype runtime/state/persistence assertions passed | 4 v2 packaging checks unavailable; fresh-source suite pending |
| Team | 274 focused assertions passed; 102 isolated-state assertions passed; 28 runtime-preflight assertions passed before bounded stop | One source-dependent packaging check unavailable; live worker startup/ACK/mailbox/terminal/shutdown requires attached tmux |
| Ultragoal | 99 passed | 5 checks unavailable because `docs/ultragoal.md` is absent from the npm artifact; the v2 Ralplan handoff was diagnostic state simulation only |
| Autopilot | 52 passed | 1 check unavailable because `docs/skills.html` is absent |
| UltraQA | 5 targeted checks passed | Live adversarial workflow was not launched in this outside-tmux session |
| Pipeline | 110 passed | — |
| Deep Interview | 53 passed | — |
| Autoresearch | 78 passed | — |
| Performance Goal | 6 passed | — |
| Wiki | 108 passed | 7 checks unavailable because referenced docs/source are absent from the npm artifact |
| HUD | 339 passed | — |
| Notifications | 421 passed | 7 environment probes unavailable because localhost/process introspection is sandbox-blocked |
| State operations | 222 passed | — |
| Skill/catalog | 88 passed | 4 checks unavailable because catalog/troubleshooting source artifacts are absent |
| Native agent assets | PASS | 22 installable native agents and 37 setup prompt assets verified |
| Plugin mirror | PASS | 29 canonical skill directories and plugin metadata synchronized |
| Managed skill links | PASS | 39 managed links correct |
| Catalog inventory | PASS | 50 catalog skills, 28 active skills, and 20 active agents reported |

## Doctor evidence

| Package | Result | Warnings |
| --- | --- | --- |
| Active global `0.20.3` | 15 passed, 3 warnings, 0 failed | Missing OMX config entries; false one-skill count; 20 repo-artifact findings |
| Isolated v2, refreshed after the source checkout appeared | 16 passed, 2 warnings, 0 failed | Missing OMX config entries; 6 immutable pack files under `.omx/tmp/oh-my-codex-source-20260803/.git/objects/**`, outside the intentionally narrow worktree-object exception |
| Isolated v2, earlier observation | 17 passed, 1 warning, 0 failed | Historical only: recorded before the source checkout introduced the second current warning |

The remaining config warning is real: `/Users/jasonlee/.codex/config.toml` exists without OMX entries. Do not suppress it. Do not chmod immutable Git object files to silence Doctor.

## Packaging and installation boundary

The v2 directory is a diagnostic bundle, not a releasable source checkout:

- It is a partial npm artifact with compiled modules that have no corresponding packaged TypeScript source.
- It has no root `tsconfig.json`; the packaged `build` script deletes `dist/` before invoking TypeScript, so running it would destroy the only complete compiled surface and then fail.
- Hand-edited compiled JavaScript has stale source maps.
- Missing docs/source/CI files account for several broad-suite unavailable checks.
- Its local-evidence positive Ralplan consensus path conflicts with the official `v0.20.4` authority contract and must not be ported.

The clean `.omx/tmp/oh-my-codex-v0204-repair-20260803/` worktree is source-complete at official tag `v0.20.4` / commit `a62d5bd`, resolving the v2 build/source-map limitation. The active global prefix and `/Users/jasonlee/.codex` remain read-only from this session.

Use `/Users/jasonlee/.agents/scripts/run-omx-setup.sh` for the eventual user-scope setup. It temporarily isolates managed skill symlinks, quarantines setup-created collisions, restores the links, and re-synchronizes reasoning lenses. This wrapper itself must write under `/Users/jasonlee/.codex`, so it cannot run within the current sandbox.

## Completion sequence

1. Use the clean official `v0.20.4` worktree at `.omx/tmp/oh-my-codex-v0204-repair-20260803/`.
2. Re-implement only repairs not already present upstream. Do not port the v2 local-evidence consensus allowance.
3. Preserve the exact `documented_host_consensus_receipt_unavailable` blocker and prove typed routing/tracker/local artifacts cannot authorize release.
4. Add or retain source regressions for finite aliases, authorized terminal/handoff recovery, macOS path identity, secure Git, exact Team help, `exec --help`, and Doctor behavior.
5. Build from complete source so JavaScript and source maps are generated together.
6. Install into the active global prefix and run the preservation-aware wrapper with the required setup options.
7. Restart Codex so hooks load the new installation.
8. Re-run focused verification, Doctor, asset/link/catalog integrity, and the full compiled suite.
9. In attached tmux, prove Team startup, worker ACK, mailbox/task progress, terminal state, and clean shutdown.
10. Replace the placeholders below only with fresh final evidence.

## Final evidence

- **`V0204_BUILD_RESULT: PENDING — leader must replace after the clean-source build finishes.`**
- **`V0204_FOCUSED_RESULT: PENDING — leader must replace with authority-safe focused regression evidence.`**
- **`FINAL_FULL_HOOK_SUITE: PENDING — leader must replace after concurrent hook edits settle and the complete suite is rerun.`**
