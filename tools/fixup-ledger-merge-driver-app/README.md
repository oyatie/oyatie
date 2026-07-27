# fixup-ledger-merge-driver

Structural three-way git merge driver for `registry/fixuptasks.jsonl` (GH #1412, ADR-0626).

## Why

The registry is append-mostly JSONL: line 1 is a schema header with **no `id`**, every other line
is one task row keyed by `id`. Two lanes that each file a finding append at the end of the file, so
git's text merge conflicts on essentially every pair of concurrent PRs even though the change is
disjoint.

Hand-resolving it is also unsafe. A resolver that indexes rows by `id` skips the header, because
the header has no `id` to key on — `if i and i not in seen`, where `None` is falsy. That deleted
the schema header from four branches in a single session, invisibly: the diff showed only the
appended rows.

`merge=union` is not the answer here, and `.gitattributes` already says so. This ledger's history
contains in-place row replacements, so union keeps both the stale and the updated variant of one
logical record. Consumers key on `id`, so that is silent corruption — worse than the conflict.

## Registration (per-clone)

```shell
git config merge.fixup-ledger.name "Oyatie fixup-ledger structural merge"
git config merge.fixup-ledger.driver "fixup-ledger-merge-driver %O %A %B"
```

Build and put the binary on `PATH`:

```shell
buck2 build //tools/fixup-ledger-merge-driver-app:fixup-ledger-merge-driver
```

`.gitattributes` names the driver; git binds it from local config. Without the config you get a
normal conflict — exactly today's behaviour.

## Semantics

Per `id`:

| base | ours | theirs | result |
|------|------|--------|--------|
| any | unchanged | edited | theirs |
| any | edited | unchanged | ours |
| any | edited | edited, same bytes | that row |
| any | edited | edited, differently | **conflict** |
| absent | added | absent | ours |
| absent | added | added, same bytes | that row |
| absent | added | added, differently | **conflict** |
| absent | absent | added | theirs |
| present | deleted | unchanged | **preserved** |
| present | deleted | deleted | **preserved** |
| present | deleted | edited | **the edit wins, silently** |

Deletion never wins: the registry declares itself append-only, and a row vanishing in a merge is
the failure this driver exists to stop. A legitimate redaction is a single linearised commit on
`dev`.

Note the last row: delete-vs-edit resolves in favour of the edit **without asking**, where git's own
text merge would raise a modify/delete conflict. Nothing is lost, and it follows from
deletion-never-wins, but it is one case where this driver is deliberately *less* conservative than
git.

Rows are copied verbatim, never re-serialised — re-dumping this file has twice produced enormous
phantom diffs by reordering keys and re-escaping em-dashes. Preservation is per-ROW, not per-file:
blank lines are dropped and exactly one trailing newline is emitted.

## Exit codes

| code | meaning | effect on `%A` |
|------|---------|----------------|
| 0 | merged cleanly | replaced atomically with the result |
| 1 | merged **with conflict markers** | replaced atomically; a human must resolve |
| 2 | unmodelled input or I/O | untouched |

**Exit 1 still writes, and that is the point.** Git does not re-run its own text merge when a
driver exits nonzero — it takes whatever the driver left in `%A` as the conflicted working tree. A
driver that exits 1 without writing leaves `ours` alone, with no markers and the other side's rows
absent: the file reads as clean and complete, so a reflexive `git add` loses rows silently. Verified
with a real `git merge`. On conflict this driver writes a file containing **every row from every
side**, with diff3 markers only around the regions needing a human.

On a clean merge the result is re-parsed and refused unless the header survived, no `id` present on
any side was lost, and none is duplicated. That guard cannot fire on the current kernel — every
`id` is emitted unconditionally — so it protects against a future edit, not today's code.

## Tests

```shell
buck2 test //tools/fixup-ledger-merge-driver-app:fixup-ledger-merge-driver-app-unittest
```

The tests are mutation-checked: reintroducing the original id-keyed header bug turns 8 of them red,
including `header_is_carried_by_position_not_by_id`.

> When running these locally, invoke buck2 **twice** after editing. Its first invocation after a
> source change can report the previous source's result.
