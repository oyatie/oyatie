# check-no-template-stamping

Enforces the synthesis-audit P0 anti-template-stamping threshold.

## Rule

Within each directory under `docs/` and `microservices/`, adjacent Markdown files are sorted by path. If three or more adjacent files have pairwise line-shape Jaccard similarity above `0.70`, the run fails.

## Trigger

Run when adding or bulk-generating documentation sets.

```bash
cargo run --manifest-path crates/check-no-template-stamping/Cargo.toml -- --root . --strict
```

## Compliant Output

```text
synthesis-audit-P0-template-stamping: Passed (2 markdown files, 1 directories, 0 violations)
OK: no adjacent template-stamped doc runs detected.
```

## Violation Output

```text
docs/a: 3 files above 0.70 line-shape Jaccard threshold
  files: docs/a/001.md, docs/a/002.md, docs/a/003.md
  pair_similarities: 1.000, 1.000
  fix: Collapse duplicated template prose into a shared standard or rewrite each doc with artifact-specific structure, evidence, and sections.
```

## How To Fix

Replace repeated generated structure with artifact-specific sections, evidence, examples, and operational detail. If the shared shape is intentional, move the repeated material to a shared standard and keep each local doc focused on unique content.
