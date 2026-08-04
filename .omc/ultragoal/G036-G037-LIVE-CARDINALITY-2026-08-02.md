# G036/G037 live cardinality — 2026-08-02

State: `PLANNING_ONLY_NOT_ACTIVATED`
Authority: origin/dev tree; no declaration executed.

## G036 governance/check
- Directories: **56**
- Cargo crates: **56**
- External tracked-reference census excluding self, Cargo.lock, catalog rows, and the landed move plan: **0 zero-reference crates**
- Zero references are not yet proof that no required-CI target transitively builds/runs a crate; Buck2 graph reachability remains the binding next probe.
- Goal claim “56 kernels observe no repository corpus” is therefore not accepted from names or grep alone.

## G037 quality lanes
- Live rows: **96**, unique IDs: **96**
- Goal expected: **93**; live drift: **+3**
- Rows with transitional check declaration: **91**
- Rows without it: **5**
- Status distribution: `{'active': 91, 'planned': 5}`
- Command transport distribution: `{'buck2_bridge': 80, 'cargo_bridge': 7, 'shell_bridge': 4, 'none': 5}`
- Registry header itself states these declarations are local/transitional catalog data and not merge authority.

## Next binding probes
1. Derive Buck2 reachability from each governance/check test target to the single required-context constituent graph.
2. For every retained quality lane, replace declaration-only evidence with a Rust/Buck2 target and a RED/GREEN acceptance fixture.
3. Reconcile 96 live rows versus G037’s 93 before any fixed-count assertion.
4. Do not baseline the roughly 112 self-conformance findings; born-blocking fixtures only.
