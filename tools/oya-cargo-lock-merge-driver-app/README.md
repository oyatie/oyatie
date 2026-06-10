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

The driver overwrites `%A` on success and exits `1` on semantic conflicts. It allows disjoint
`[[package]]` additions, preserves the lockfile preamble, and refuses same-package version
divergence or removal-vs-edit merges.
