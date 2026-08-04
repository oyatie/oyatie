# oya-cargo-lock-merge-driver

Structural Git merge driver for the root `Cargo.lock`.

## Activation

The driver definition is VERSIONED in `tools/hooks/merge-drivers.gitconfig` — do not hand-write a
`git config merge.cargo-lock.driver` line, which is how this driver spent its life registered
nowhere at all while `.gitattributes` claimed otherwise. Two steps, both idempotent:

```bash
# once per clone; all linked worktrees of that clone inherit it
git config --local include.path ../tools/hooks/merge-drivers.gitconfig

# whenever buck-out is cleaned
buck2 build //tools/oya-cargo-lock-merge-driver-app:oya-cargo-lock-merge-driver \
  --out tools/hooks/bin/oya-cargo-lock-merge-driver
```

`ci/facade/hook-wiring` fails closed if the `.gitattributes` declaration, the versioned
registration and the build target ever disagree.

The `.gitattributes` entry is:

```gitattributes
Cargo.lock merge=cargo-lock
```

The driver overwrites `%A` on success and exits `1` on semantic conflicts. It allows disjoint
`[[package]]` additions, preserves the lockfile preamble, and refuses same-package version
divergence or removal-vs-edit merges.
