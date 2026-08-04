# Materializer kernel move plan proposal — 2026-08-02

Terminal: `BLOCKED_NO_EXECUTABLE_MOVE_PLAN`

## Authority finding

Existing authority maps `libs/oya-ci-materializer-kernel` only to the coarse meta-home `build/`.
It does **not** approve:

- exact destination leaf under `build/`
- package / Buck target namespace grammar
- Cargo workspace membership shape for `build/*`
- module-membership `scan_roots` treatment despite `owns_crates:false`
- catalog / SLO treatment for a pure build kernel

Therefore no move may execute.

## Provisional destination (proposal only — not authorized)

```json
{
  "capability": "build",
  "moves": [
    {
      "old_path": "libs/oya-ci-materializer-kernel",
      "new_path": "build/materializer-kernel",
      "old_cargo_name": "oya-ci-materializer-kernel",
      "new_cargo_name": "build-materializer-kernel"
    }
  ],
  "artifacts": []
}
```

## Hard blockers

1. No `build/` directory and no approved build-root crate topology.
2. Root Cargo membership covers capability faces + legacy `libs/oya-*` only; no reusable `build/*` rule.
3. Module-membership scan roots omit `build/` because registry `owns_crates=false`.
4. ADR-0597 is **Proposed**, commissions the legacy path/name, and says no catalog row; current crate-catalog policy requires a row for moved/new-name crates.
5. PR #1523 codemod oracle is not promoted; apply is not dry-run-gated; snapshot CLI emits booleans/lengths, not parity receipts.
6. No representative trial / inverse-apply byte-identical parity oracle exists for this leaf.

## Bun-style prerequisites before any writer

1. Census complete (zero production consumers confirmed).
2. Stage A build-root contract approved independently.
3. Codemod oracle promoted with dry-run gate + content/digest snapshots + targeted Buck build/test.
4. One disposable shadow trial with inverse-apply byte identity.
5. One PR atomic cutover; no dual authority / compatibility crate.

## Non-claims

This document is planning evidence only. It is not move approval, not a mergeable plan JSON under `specs/reorg/`, and not authorization to create `build/`.
