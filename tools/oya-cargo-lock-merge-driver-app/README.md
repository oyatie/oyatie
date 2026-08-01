# oya-cargo-lock-merge-driver

Structural Git merge driver for the root `Cargo.lock`.

## Activation

Build the driver:

```bash
buck2 build //tools/oya-cargo-lock-merge-driver-app:oya-cargo-lock-merge-driver
```

Install or point Git at the built binary:

```bash
git config merge.cargo-lock.name "Oyatie Cargo.lock structural merge"
git config merge.cargo-lock.driver "oya-cargo-lock-merge-driver %O %A %B"
```

The `.gitattributes` entry is:

```gitattributes
Cargo.lock merge=cargo-lock
```

## Semantics

The driver allows disjoint `[[package]]` additions, preserves the lockfile preamble, and refuses
same-package version divergence or removal-vs-edit merges.

## Placeholders

`%O %A %B` are read positionally. Git may substitute more — `%L` (conflict-marker size), `%P`
(pathname), `%S`/`%X`/`%Y` (conflict labels) — and `merge.oya-faces` in this repo is already
registered as `%O %A %B %P`, so **any trailing placeholders are accepted and ignored**.

Ignored rather than decoded: git substitutes whatever order the config names, so a positional guess
at which extra is `%P` would print the marker size as a filename. Nothing is lost by ignoring them,
because git already names the real path in its own `CONFLICT (content): Merge conflict in <path>`
line. Demanding exactly three arguments was itself a data-loss bug — see below.

## Exit codes

| code | meaning | effect on `%A` | effect on the merge |
|------|---------|----------------|---------------------|
| 0 | merged cleanly | replaced atomically with the merged lockfile | resolved |
| 1 | conflict | replaced atomically with **every side under diff3 markers** | `UU`, human resolves |
| 129 | `%A` unknown or unwritable | untouched | git **abandons** the merge |

**No exit leaves `%A` as unmarked `ours` while git believes the merge happened.** Git does not
re-run its own text merge when a driver exits nonzero — it takes whatever the driver left in `%A`
as the conflicted working tree. A driver that exits nonzero without writing leaves `ours` alone,
with no markers and the other side's packages simply absent: the path is `UU`, but the file reads
as clean and complete, so a reflexive `git add` loses `theirs` silently. Verified twice with a real
`git merge`, each time producing a merge commit byte-identical to `ours` — strictly worse than
having no driver registered at all, since git's own text merge does leave markers.

Two mechanisms make that unreachable:

1. `resolve` in `src/main.rs` returns `Resolution`, **not** `Result<Resolution, _>`. A semantic
   conflict, a lockfile that does not parse, a side that cannot be read and a panic inside the
   parser are all expressible as "these bytes, and a human must look", so none of them needs an
   early return and there is nowhere for a `?` to skip the write. The result deliberately does not
   parse as TOML, so cargo fails loudly as a second backstop.
2. Where writing is impossible at all — an argument list that never named `%A`, a filesystem that
   refused the write — the exit code is **129**. gitattributes(5) defines >128 as the driver having
   crashed, and git then fails the merge outright instead of recording a conflict: no `MERGE_HEAD`,
   no `UU` path, nothing for a `git add` to commit. A merge that did not happen loses nothing.

Exit 2 no longer exists. It was the one code that told git "conflict" while guaranteeing nothing
about `%A`, which is exactly the shape of the data loss.

## Tests

```bash
buck2 test //tools/oya-cargo-lock-merge-driver-app/...
```

`tests/cli_fixtures.rs` carries one fixture per process exit path — including the four- and
five-placeholder registrations — and every nonzero one asserts that `%A` still contains `theirs`.

`tests/git_merge_e2e.rs` drives a **real `git merge`**. Unit fixtures can only prove what the
driver writes and what it exits with; they cannot prove what git does with those two facts, and
that is precisely where the data loss lived. Its central assertion is that the working tree is
never byte-identical to `ours` while git believes a merge happened.
