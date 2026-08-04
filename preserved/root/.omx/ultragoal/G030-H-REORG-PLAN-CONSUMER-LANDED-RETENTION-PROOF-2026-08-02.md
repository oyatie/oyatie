# G030-H reorg-plan consumer, landed-state, and retention proof — 2026-08-02

State: **PLANNING_ONLY — EIGHT ROWS PROMOTED TO GRAPH-WIRED; TWO RETAINED BY EXPLICIT COMPATIBILITY/DESIGN CONTRACT**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G030-G-RECONCILED-SEMANTIC-CENSUS-2026-08-02.md`.  
No plan, source path, generated face, policy, gate, PR, GitOps declaration, or cluster state was changed.

## Result

The bounded ten-row `specs/reorg/*` residual is not unused bureaucracy and is not a deletion queue:

- eight ordinary `*-move-plan.json` rows are enumerated by the owned codemod and mechanically classify as **landed** at the immutable authority tip;
- `ci-graph-additions.json` is a non-move companion to `ci-move-plan.json`, retained by the ADR-0563 Cargo.lock graph-mutation contract;
- `kernel-move-plan.BLOCKED.json` deliberately does not match the active-plan filename grammar and records an unapplied, mechanically blocked design whose old tree remains live.

The G030 classification update is therefore:

| Disposition | Rows | G030 class |
|---|---:|---|
| Machine-enumerated, landed/spent move plans | 8 | `GRAPH_WIRED_INPUT` |
| Explicit graph-mutation compatibility companion | 1 | `POLICY_PROTECTED_MACHINE_ARTIFACT` |
| Explicit blocked-design retention row | 1 | `POLICY_PROTECTED_MACHINE_ARTIFACT` |
| Delete candidates | 0 | none |

This promotes eight rows out of G030-G's protected-only queue. The reconciled totals become **152 `MACHINE_SSOT` + 902 `GRAPH_WIRED_INPUT` + 122 `POLICY_PROTECTED_MACHINE_ARTIFACT` = 1,176**. The remaining protected queue is 19 fixture residuals plus 103 non-fixture rows.

## Row-by-row proof

| Path | Shape at immutable tip | Consumer/retention proof | Disposition |
|---|---|---|---|
| `specs/reorg/governance-check-move-plan.json` | 56 moves; 112 move+artifact old probe paths absent; all 56 new move paths present | matches codemod `*-move-plan.json` enumeration; `plan_probe_paths` and `plan_is_landed` make its spent state executable input | `GRAPH_WIRED_INPUT — LANDED` |
| `specs/reorg/iam-pdp-cedar-move-plan.json` | 1 move; old absent; new present | same machine enumeration and landed probe; ADR-0562 cites the exact plan | `GRAPH_WIRED_INPUT — LANDED` |
| `specs/reorg/intelligence-move-plan.json` | 15 moves; 30 move+artifact old probe paths absent; all 15 new move paths present | same machine enumeration and landed probe; ADR-0562 cites the exact plan as applied/spent history | `GRAPH_WIRED_INPUT — LANDED` |
| `specs/reorg/intelligence-sinkbatch-move-plan.json` | 10 moves; 20 move+artifact old probe paths absent; all 10 new move paths present | same machine enumeration and landed probe | `GRAPH_WIRED_INPUT — LANDED` |
| `specs/reorg/intelligence-supervisor-move-plan.json` | 8 moves; 16 move+artifact old probe paths absent; all 8 new move paths present | same machine enumeration and landed probe | `GRAPH_WIRED_INPUT — LANDED` |
| `specs/reorg/messaging-boundary-kernels-move-plan.json` | 3 moves; all old paths absent; all new paths present | same machine enumeration and landed probe; ADR-0562 cites the exact plan | `GRAPH_WIRED_INPUT — LANDED` |
| `specs/reorg/messaging-substrate-kernel-move-plan.json` | 1 move; old absent; new present | same machine enumeration and landed probe; ADR-0562 cites the exact plan | `GRAPH_WIRED_INPUT — LANDED` |
| `specs/reorg/os-move-plan.json` | 41 moves; all old paths absent; all new paths present | same machine enumeration and landed probe; ADR-0562 and the supervisor plan cite the exact plan | `GRAPH_WIRED_INPUT — LANDED` |
| `specs/reorg/ci-graph-additions.json` | no `moves`; keys are `_comment`, `new_members`, and `add_dependencies` | ADR-0563 cites it exactly as the Cargo.lock additions companion to `ci-move-plan.json`; its own contract names the owned `lockfile-move` transform | `POLICY_PROTECTED_MACHINE_ARTIFACT — COMPATIBILITY COMPANION` |
| `specs/reorg/kernel-move-plan.BLOCKED.json` | 7 moves and 10 artifacts; all 17 old probe paths present; all 7 new move paths absent | filename intentionally falls outside `*-move-plan.json`; `_blockers` records three proven blockers and requires fail-closed non-application until resolved | `POLICY_PROTECTED_MACHINE_ARTIFACT — BLOCKED DESIGN` |

The move-only source/destination existence counts agree with the complete landed probe: the eight landed plans are uniformly `old absent / new present`; the kernel plan is uniformly `old present / new absent`. Existence was tested only with `git ls-tree b651080... -- <exact-path>`.

## Machine consumer contract

The owned Rust codemod defines:

- `REORG_PLAN_DIR = "specs/reorg"`;
- `MOVE_PLAN_SUFFIX = "-move-plan.json"`;
- deterministic directory enumeration with a non-empty capability stem;
- `plan_probe_paths`, covering move old paths plus artifact old paths;
- `plan_is_landed`, which is true only for a non-empty probe whose every old path is absent at merge base;
- `select_active_move_plan` / `resolve_effective_active_move_plan`, which exclude landed plans before applying the fail-closed one-active-plan rule.

This is a semantic directory consumer, so absence of a per-file literal in source is not negative consumer proof. The files' names and path contents affect executable plan discovery and landed-state classification.

Downstream machine edges corroborate the contract:

- `ci/facade/crate-registration` discovers a sorted `*-move-plan.json` and invokes codemod manifest regeneration before other accounting faces;
- `ci/facade/inventory-registry-drift` independently discovers a committed move plan and regenerates the manifest for drift comparison;
- `ci/adapters/path-resolver`, generated-artifact freshness/policy, and crate registration consume `specs/reorg/move-manifest.generated.json`.

The downstream helpers retain a first-plan compatibility path, while the codemod owns the fail-closed and landed-aware semantics. This report does not certify that the compatibility helpers are ideal; it establishes that the plan family is machine-observed rather than dark prose.

## Why the two retained rows are not promoted

### `ci-graph-additions.json`

Its filename does not match `*-move-plan.json`, and it has no move/artifact probe. Calling it codemod-enumerated would be false. Its retention is nonetheless explicit: ADR-0563 and the file itself bind it to `ci-move-plan.json` for the beyond-rename Cargo.lock graph mutations. That is compatibility/history evidence, not proof that the generic move-plan loader reads this exact row at current tip.

### `kernel-move-plan.BLOCKED.json`

The `.BLOCKED.json` suffix deliberately prevents active-plan discovery. It is also mechanically unapplied: the complete old-path probe is present and destinations are absent. Its three recorded blockers cover incompatible nested-workspace toolchain/config values, missing workspace-dependency rewrites in the codemod, and old-layout path literals. G030 may retain this settled design, but must not relabel it graph-wired, active, approved, or executable.

## Retention rule

- Keep all ten rows while their current machine/history/design contracts stand.
- Landed/spent does not mean deletable: the codemod explicitly uses old-path absence to prevent stale landed plans from wedging later moves.
- Any future cleanup must be owned by the reorg/codemod lifecycle and prove that removing spent plans preserves discovery, landed self-healing, manifest determinism, ADR provenance, and frozen baseline relabel history.
- The blocked kernel plan may change state only through its separately authorized owner lane after all mechanical blockers are cleared; this census does not activate it.

## Verification boundary

Evidence was derived from the immutable authority commit using:

- exact `git ls-tree <immutable-ref> -- <path>` probes for every move and artifact old path and every move destination;
- source inspection of the codemod discovery/selection/landed functions and CI regeneration helpers;
- exact-path citation searches in ADRs and machine consumers;
- JSON shape/count inspection without editing any source row.

Independent audit transport failed again (`encrypted_content` decrypt error). That is recorded as `FAILED_TRANSPORT_NOT_APPROVE`, not approval. The mechanical evidence supports this census classification only; it does not approve deletion, activation, or a move.

## Non-actions and non-claims

- No plan edited or deleted.
- No generated `move-manifest.generated.json` authored or hand-edited.
- No claim that spent plans are safe to delete.
- No claim that `ci-graph-additions.json` is an active generic move plan.
- No claim that `kernel-move-plan.BLOCKED.json` is executable or approved.
- No new move-plan JSON or multispectrum evidence surface.
- No independent APPROVE; transport failure remains non-approval.
