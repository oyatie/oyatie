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
buck2 build //ci/facade/fixup-ledger-merge-driver:fixup-ledger-merge-driver
```

`.gitattributes` names the driver; git binds it from local config. **Without the config you get a
normal conflict — exactly today's behaviour.** There is no state in which having this driver is
worse than not having it.

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
| present | deleted | present | **preserved** |

Deletion never wins: the registry declares itself append-only, and a row vanishing in a merge is
the failure this driver exists to stop. A legitimate redaction is a single linearised commit on
`dev`.

Rows are copied verbatim, never re-serialised — re-dumping this file has twice produced enormous
phantom diffs by reordering keys and re-escaping em-dashes.

## Exit codes

| code | meaning | effect on `%A` |
|------|---------|----------------|
| 0 | merged | replaced atomically with the result |
| 1 | declined — the sides disagree unambiguously | untouched |
| 2 | unmodelled input, or failed self-validation | untouched |

On any nonzero exit `%A` is left byte-untouched and git falls back to a normal conflict. The driver
never writes a partially-merged ledger.

Before emitting, the result is re-parsed and refused unless the header survived, no `id` present on
any side was lost, and no `id` is duplicated.

## Tests

```shell
buck2 test //ci/facade/fixup-ledger-merge-driver:ci-fixup-ledger-merge-driver-unittest
```

The tests are mutation-checked: reintroducing the original id-keyed header bug turns 8 of them red,
including `header_is_carried_by_position_not_by_id`.

> When running these locally, invoke buck2 **twice** after editing. Its first invocation after a
> source change can report the previous source's result.
