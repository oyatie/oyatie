# oya-governance-substance-bar

Enforces documentation-rigor doc-class line floors.

## Rule

Every Markdown document with YAML frontmatter `doc_class` must meet the minimum line floor for that class from `docs/standards/documentation-rigor.md` §2. Unknown doc classes fail because no floor can be applied.

## Trigger

Run when adding or changing canonical docs.

```bash
cargo run --manifest-path crates/oya-governance-substance-bar/Cargo.toml -- --root . --strict
```

## Compliant Output

```text
documentation-rigor-1.2-line-floor: Passed (1 markdown files, 1 doc_class docs, 0 violations)
OK: every doc_class document meets its line floor.
```

## Violation Output

```text
docs/standards/example.md:2: BelowLineFloor: doc_class=Standard observed_lines=4 required_lines=250
  fix: Expand this Standard document to at least 250 lines with the required sections and density signals from documentation-rigor §2.
```

## How To Fix

Expand the document to the required floor and include the sections/density signals for its class. If the class is not in documentation-rigor, either change the frontmatter to a known class or update the rigor matrix before enforcing it.
