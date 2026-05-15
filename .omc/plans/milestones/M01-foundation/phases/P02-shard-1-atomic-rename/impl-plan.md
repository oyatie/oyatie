---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P02-shard-1-atomic-rename
impl_plan_id: IP-001-shard-1-atomic-rename
status: merged
owner: council-architecture
blocked_by: []
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-nextest
- cargo-deny
- cargo-doc
purpose: Renames all 114 Shard-1-scoped crate directories, package names, dep-edge references, root workspace members, and Cargo.lock entries from v3 names to BNF v4.1 names in a single atomic commit.
---
# IP-001-shard-1-atomic-rename: Execute Shard 1 atomic 114-row rename

## Intent

Renames all 114 Shard-1-scoped crate directories, package names, dep-edge
references, root workspace members, and Cargo.lock entries from v3 names to
BNF v4.1 names in a single atomic commit. After merge, zero `oya-platform-*`,
`oya-workspace-*`, `oya-foundation-*`, or `oya-tooling-*` crates exist in
the workspace.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/<old-name>/` (×114) | rename dir → `crates/<new-name>/` | mv per TSV row |
| `crates/<new-name>/Cargo.toml` (×114) | update | `[package] name` = new name; dep keys + paths updated |
| `Cargo.toml` | update | `[workspace.members]` all 114 entries; `[workspace.metadata.oya]` verticals→microservices; connect entry added |
| `Cargo.lock` | rewrite | `xtask-metadata-augment lockfile-rename --rename-map /tmp/rename-map-v4.1.tsv --inplace` |
| `tools/xtask-metadata-augment/src/main.rs` | update | Dual-schema parser (9-col + 11-col) for generate-rename-map; produces 114 rows |

---

## Crate Naming

All 114 renames carry justifications in §3 of
`docs/plans/rename-plan-v4-clean-arch-2026-05-13.md`. Representative examples:

```
NAME: oya-tenancy-domain
JUSTIFICATION:
- microservice = tenancy: Tenant SaaS contract µservice; registered in workspace metadata; ADR-0056 v4.1 flat BNF; per Bominal ADR-0125 naming canon
- bc-tokens = (none): single concept at domain layer; BC-optionality rule: omit
- layer = domain: business logic on kernel types; STUB-pending-iter-4 confirms
- exemptions claimed: none

NAME: oya-data-boundary-kernel
JUSTIFICATION:
- microservice = data-boundary: Data-use-boundary µservice (12 data classes per ADR-0008); registered
- bc-tokens = (none): single concept at kernel layer
- layer = kernel: pure types + ports (named-by-identity; ~95 consumers; highest-blast-radius row)
- exemptions claimed: none

NAME: oya-application-app
JUSTIFICATION:
- microservice = application: B2B unified shell µservice (Application per feedback_flat_product_catalog.md)
- bc-tokens = (none): single concept; composition root
- layer = app: composition-root binary wiring all layers per §2.2.4 step 4
- exemptions claimed: none

NAME: oya-codeview-cli
JUSTIFICATION:
- microservice = codeview: domain noun for agent read-only code view sanctioned primitive
- bc-tokens = (none): single concept at cli layer
- layer = cli: has [[bin]] target; sanctioned-primitive READ slot per git-workflow.md
- exemptions claimed: none

NAME: oya-check-<rule-name>  (×29 foundry check crates)
JUSTIFICATION:
- microservice = check: BNF second production oya-check-<rule-name>; BNF-exempt per ADR-0056 line 79-80
- bc-tokens = <rule-name>: open rule-name token; 1..4 kebab tokens
- layer = check-namespace-exempt
- exemptions claimed: ADR-0056 BNF second production
```

---

## Code Shape

No new logic introduced. All crates are scaffold-empty or retain their existing
source. The rename changes only: directory names, `[package] name`, dep-edge
keys/paths, workspace member list, lockfile crate name entries.

Xtask `generate-rename-map` update (dual-schema parser):
```rust
// 11-column rows: proposed_name at index 7
// 9-column rows: proposed_name at index 6
let proposed_idx = if cells.len() >= 11 { 7 } else { 6 };
```

---

## Acceptance Gates

```bash
# 1. TSV generates 114 rows
cargo run -p xtask-metadata-augment -- generate-rename-map \
  --plan docs/plans/rename-plan-v4-clean-arch-2026-05-13.md \
  --map-out /tmp/rename-map-v4.1.tsv \
  --names-out /tmp/old-crate-names-v4.1.txt
# Expected: "generate-rename-map: 114 rename pairs written"

# 2. Workspace compiles
rtk cargo check --workspace --all-features               # exit 0
rtk cargo build --workspace --all-features               # exit 0
rtk cargo clippy --workspace --all-targets -- -D warnings  # exit 0
rtk cargo nextest run --workspace || cargo test --workspace  # exit 0
rtk cargo deny check                                     # exit 0
rtk cargo doc --workspace --no-deps                      # exit 0

# 3. Reality: zero old-name dirs
ls crates/oya-platform-* 2>&1 | grep -c "No such"   # > 0
ls crates/oya-workspace-* 2>&1 | grep -c "No such"  # > 0
ls crates/oya-foundation-* 2>&1 | grep -c "No such" # > 0
ls crates/oya-tooling-* 2>&1 | grep -c "No such"    # > 0

# 4. Cargo metadata shows zero old names
cargo metadata --format-version 1 | jq '.packages[].name' | \
  grep -E '"oya-(platform|workspace|foundation|tooling)-' | wc -l   # 0
```

---

## Test Plan

### Unit tests

No new logic; existing crate unit tests unchanged. All scaffold-empty crates
have zero tests (correct — no business logic).

### Integration tests

```bash
# Workspace-wide nextest run validates dep graph is intact
cargo nextest run --workspace   # exit 0; 0 failures
```

---

## Clean Architecture Compliance

### Dependency direction check

All renamed crates retain their existing `[dependencies]` structure. The rename
does not introduce new dep edges. Layer ordering is unchanged.

### Cross-product integration check

No cross-product dep edges exist in the current scaffold-empty workspace.
LEAN-A2 (microservice-isolation; v4.1 override) will verify post-P05 flip.

---

## Load Test

Not applicable — rename-only; no new API surfaces introduced.

---

## Grit Symbol-Locks

```bash
# grit session start failed; ICM scaffold-locks-oyatie fallback per ADR-0054
# Branch: grit/shard-1-atomic-rename-2026-05-13 (direct git checkout -b)
# ICM rationale stored: direct-tool-invocations topic
```

Release: direct `git merge --no-ff grit/shard-1-atomic-rename-2026-05-13` to main
with ICM `direct-tool-invocations` rationale row (ADR-0053 bootstrap window).

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-shard-1-atomic-rename merged. 114 crate dirs renamed to BNF v4.1. Zero oya-platform-*/workspace-*/foundation-*/tooling-* on disk. Cargo.lock rewritten. All 6 acceptance gates exit 0. xtask generate-rename-map dual-schema parser: 114 rows. 26 PROTOCOL-UNKNOWN deferred to P03. grit done fallback: direct merge per ADR-0053 bootstrap window. Next: P03+P04 can now run." \
  -i high \
  -k "M01,P02,IP-001,shard-1,114-renames,BNF-v4.1,merged"
```

---

## Halt Conditions

1. `cargo check` fails after fix attempts — diagnose dep-ref mismatch; check remaining old-name refs with `grep -r "oya-platform-\|oya-workspace-" crates/`.
2. Double-substitution artifact (e.g. `applicationlication`) detected — fix the specific Cargo.toml entry directly.
3. Lockfile rewrite fails due to workspace load error — fix dep refs first, then re-run lockfile-rename.
4. TSV produces ≠ 114 rows — parser bug; fix dual-schema column detection.

---

## Next IP Pointer

`../P03-shard-1-5-protocol-unknown-deferred/impl-plan.md` (after P04 completes protocol classification evidence)

---

## Cross-References

- Phase spec: `phase-spec.md`
- Rename plan: `docs/plans/rename-plan-v4-clean-arch-2026-05-13.md`
- ADR-0056, ADR-0057
- TSV: `/tmp/rename-map-v4.1.tsv`
- Memory: `feedback_naming_justification.md`, `feedback_grit_claim_work_done.md`
