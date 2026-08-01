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

## Exit codes

| code | meaning | effect on `%A` |
|------|---------|----------------|
| 0 | merged cleanly | replaced atomically with the merged lockfile |
| 1 | conflict | replaced atomically with **every side under diff3 markers**; a human must resolve |
| 2 | `%A` unknown (bad driver argument list) or unwritable | untouched |

**Exit 1 still writes, and that is the point.** Git does not re-run its own text merge when a
driver exits nonzero — it takes whatever the driver left in `%A` as the conflicted working tree. A
driver that exits 1 without writing leaves `ours` alone, with no markers and the other side's
packages simply absent: the path is `UU`, but the file reads as clean and complete, so a reflexive
`git add` loses `theirs` silently. Verified with a real `git merge`: before this was fixed, the
driver produced a merge commit byte-identical to `ours` with `theirs`' package gone and nothing on
screen to make anyone look — strictly worse than having no driver registered at all, since git's
own text merge does leave markers.

So a semantic conflict, a lockfile that does not parse, and a side that cannot be read all take the
same path: write every side under diff3 markers, then signal. The result deliberately does not
parse as TOML, so cargo fails loudly as a second backstop. Exit 2 survives only for the two states
where writing is impossible — an argument list that never named `%A`, and a filesystem that refused
the write.

## Tests

```bash
buck2 test //tools/oya-cargo-lock-merge-driver-app:oya-cargo-lock-merge-driver-app-unittest \
           //tools/oya-cargo-lock-merge-driver-app:oya-cargo-lock-merge-driver-app-fixtures \
           //tools/oya-cargo-lock-merge-driver-app:oya-cargo-lock-merge-driver-app-cli-fixtures
```

`tests/cli_fixtures.rs` carries one fixture per process exit path; every nonzero one asserts that
`%A` still contains `theirs`.
