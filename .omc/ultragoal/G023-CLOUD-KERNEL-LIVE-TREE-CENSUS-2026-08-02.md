# G023 cloud-kernel live-tree census — 2026-08-02

State: **PENDING — APPROVED DELETION, BLOCKED BY #1523 PROMOTION**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`; approved execution contract: `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/DELETION-PLAN-cloud-kernel.md`.

## Correction

The prior statement that `cloud/cloud-kernel` was absent on `origin/dev` was a false negative. Robust tree queries prove the source tree is live:

- `git ls-tree -d --name-only origin/dev -- cloud/cloud-kernel` → `cloud/cloud-kernel`
- `git ls-tree -r --name-only origin/dev -- cloud/cloud-kernel` → **170 tracked files**
- coordinator worktree path exists and the worktree is not sparse
- no deletion commit appears in the path history; last touch is `11a59590c` (#1507)

The captured grep output itself contained 170 live-tree paths. It was therefore internally inconsistent to infer absence from an earlier command. The likely failure class was a malformed or context-sensitive existence probe, not repository state. Never infer source absence from a grep result or a worktree-local directory test; use `git ls-tree <immutable-ref> -- <path>`.

## Live source inventory

| Metric | Current `origin/dev` |
|---|---:|
| Tracked files under `cloud/cloud-kernel` | **170** |
| `Cargo.toml` files (workspace root + packages) | **21** |
| BUCK files | **8** |
| Rust source files | **87** |
| Tracked `.elf` files | **7** |
| Membership freeze rows naming the path | **20** |
| Embedded-asset-hermeticity baseline rows | **21** |

The 21 manifests are the nested workspace root plus the approved plan's 20-crate deletion population. The tracked tree is the bespoke framekernel to delete; it is not merely stale references or fixtures.

## Kept destination/substrate

`kernel/` is independently live and must survive:

- `kernel/Cargo.toml`
- `kernel/core/asterinas-boundary/Cargo.toml`
- `kernel/harness/asterinas-real-boot/Cargo.toml`

Recovery is already secured by tag `kernel-snapshot-2026-06-08`; `072a66f37` remains the dev-shaped restore commit. G023 deletes `cloud/cloud-kernel`, not `kernel/`.

## Why this is DELETE, not MOVE

`specs/reorg/kernel-move-plan.BLOCKED.json` is still present and deliberately non-executable. Its blockers remain structural:

1. `kernel/` and `cloud/cloud-kernel` are incompatible nested workspaces (toolchain, target config, workspace inheritance).
2. The codemod does not rewrite `[workspace.dependencies]` path dependencies.
3. Cross-crate `include!` / `include_bytes!` path literals encode the old layout.

The founder-approved 2026-08-02 ruling supersedes attempting that blocked MOVE: keep Asterinas under `kernel/`; delete the unowned framekernel.

## Same-commit cleanup obligation

Deleting the 170-file tree alone is incomplete. The approved plan requires the coupled cleanup in the same commit, including:

- regenerate module-membership freeze (**350 → 330**), never hand-edit it
- remove all 20 catalog entries (**197 → 177**)
- remove all 21 hermeticity baseline entries (**21 → 0**)
- remove the two affected-set linker paths and update the coupled tests while preserving the exact-literal `RefuseUnowned` proof with a synthetic fixture path
- remove mirrored scratch rules from both unit-class policy copies
- remove tier classification, root workspace exclude, capability-registry absorb path, dependency exclusion, root ignores, blocked move plan, and narrowed fixup/friction rows
- update only the approved ADR/source-anchor surfaces; preserve owned-stack rung names and masterplan rung
- do not edit append-only historical evidence or dated audit snapshots

A content grep currently finds 185 `cloud/cloud-kernel` occurrences. They span live source self-references plus CI policy, specs, tests/fixtures, docs/evidence, root registry/config, and cloud-os metadata. This count is a sweep input, not a blanket deletion count: the approved plan explicitly preserves historical evidence, test fixtures that remain valid, and masterplan ladder naming.

## Activation gate

G023 remains **pending**. Execution order is binding:

1. #1526 admitted and promoted-tip green
2. #1523 restacked, independently approved, protected green, merged, and promoted-tip green
3. create a fresh isolated branch from then-current `origin/dev`
4. execute `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/DELETION-PLAN-cloud-kernel.md` exactly
5. cold Buck2/control-differential verification, independent review, protected admission, promoted observation

No source deletion, plan mutation, push, or live-system mutation occurred in this census lane.
