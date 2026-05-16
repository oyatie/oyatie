---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate
phase: P20-ci-lanes-operational
impl_plan_id: IP-001-ci-lanes-statelessness-shardability
status: pending
owner: council-foundry
blocked_by: []
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-nextest
- cargo-deny
purpose: "Implements the two scalability-enforcement fitness lane binaries: `oya-check-statelessness` (detects module-level mutable state in presentation/application/ worker crates) and `oya-check-shardability` (verifies every tenant-bound table decl."
---
# IP-001-ci-lanes-statelessness-shardability: Implement oya-check-statelessness + oya-check-shardability CLI Binaries

## Intent

Implements the two scalability-enforcement fitness lane binaries:
`oya-check-statelessness` (detects module-level mutable state in presentation/application/
worker crates) and `oya-check-shardability` (verifies every tenant-bound table declares
`tenant_id` distribution column + RLS policy). Both run in `--report-only` mode until
P22 flips them to BLOCKER. Self-tests validate known-violation and known-clean fixtures.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `Cargo.toml` | update | Add oya-check-statelessness + oya-check-shardability to workspace members |
| `crates/oya-check-statelessness/Cargo.toml` | create | `[[bin]]` target; deps: cargo_metadata, syn, clap, anyhow |
| `crates/oya-check-statelessness/src/main.rs` | create | CLI entry: --workspace, --crate, --report-only, --blocker flags |
| `crates/oya-check-statelessness/src/checker.rs` | create | Parse Cargo.toml layer; for presentation/application/worker layers: AST-scan for `static mut`, `lazy_static!`, `once_cell::sync::Lazy` with mutable interior |
| `crates/oya-check-statelessness/src/report.rs` | create | Violation report: file + line + symbol; --report-only prints; --blocker exits nonzero |
| `crates/oya-check-statelessness/tests/self_test.rs` | create | Known-violation fixture (static mut counter) → detected; Known-clean fixture → 0 violations |
| `crates/oya-check-shardability/Cargo.toml` | create | `[[bin]]` target; deps: glob, regex, clap, anyhow |
| `crates/oya-check-shardability/src/main.rs` | create | CLI entry: --migrations-dir, --workspace, --report-only, --blocker |
| `crates/oya-check-shardability/src/checker.rs` | create | Parse migration SQL files; for each CREATE TABLE: verify (1) tenant_id column present for tenant-bound tables, (2) RLS FORCE policy present, (3) COMMENT ON TABLE includes 'distribution_column:tenant_id' |
| `crates/oya-check-shardability/src/report.rs` | create | Violation report: migration file + table name + missing element |
| `crates/oya-check-shardability/tests/self_test.rs` | create | Known-violation fixture (table missing distribution_column comment) → detected; Known-clean → 0 |

---

## Crate Naming

```
NAME: oya-check-statelessness
JUSTIFICATION:
- oya-check-* namespace exemption per ADR-0056 v4.1; rule-name = statelessness
- No microservice slot; not bound to product crate; cross-cutting workspace check
- exemptions claimed: oya-check-* namespace exemption

NAME: oya-check-shardability
JUSTIFICATION:
- oya-check-* namespace exemption; rule-name = shardability
- Parses migration SQL; verifies tenant_id distribution column on all tenant-bound tables
- exemptions claimed: oya-check-* namespace exemption
```

---

## Code Shape

### `crates/oya-check-statelessness/src/checker.rs`

```rust
use cargo_metadata::MetadataCommand;
use std::path::Path;

#[derive(Debug)]
pub struct StatelessnessViolation {
    pub crate_name: String,
    pub layer: String,
    pub file: String,
    pub line: u32,
    pub symbol: String,
    pub reason: String,
}

/// Layers that must be stateless (no module-level mutable state)
const STATEFUL_FORBIDDEN_LAYERS: &[&str] = &[
    "application", "rest", "grpc", "graphql", "worker",
];

pub fn check_workspace(manifest_path: &Path, report_only: bool) -> Vec<StatelessnessViolation> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec()
        .expect("cargo metadata failed");

    let mut violations = Vec::new();
    for package in &metadata.packages {
        let layer = extract_layer_from_name(&package.name);
        if STATEFUL_FORBIDDEN_LAYERS.contains(&layer.as_str()) {
            violations.extend(scan_package_for_mutable_statics(package));
        }
    }
    violations
}

fn extract_layer_from_name(name: &str) -> String {
    // Last token after final '-' is the layer
    name.split('-').last().unwrap_or("").to_string()
}

fn scan_package_for_mutable_statics(pkg: &cargo_metadata::Package) -> Vec<StatelessnessViolation> {
    // Walk pkg.manifest_path/../src/**/*.rs
    // Use syn to parse each file; detect:
    //   - `static mut <IDENT>` items
    //   - `lazy_static! { static ref <IDENT>: Mutex<...>` or `RwLock<...>`
    //   - `once_cell::sync::Lazy<Mutex<...>>` or `Lazy<RwLock<...>>`
    // Return violations for each detected pattern
    vec![] // implementation body
}
```

### `crates/oya-check-shardability/src/checker.rs`

```rust
use std::path::Path;

#[derive(Debug)]
pub struct ShardabilityViolation {
    pub migration_file: String,
    pub table_name: String,
    pub missing: ShardabilityRequirement,
}

#[derive(Debug)]
pub enum ShardabilityRequirement {
    TenantIdColumn,
    RlsPolicy,
    DistributionColumnComment,
}

pub fn check_migrations_dir(dir: &Path, report_only: bool) -> Vec<ShardabilityViolation> {
    // Walk dir/**/*.sql
    // For each CREATE TABLE statement:
    //   If table has any column pattern (tenant_id, user_id referencing tenants):
    //     1. Check tenant_id uuid NOT NULL column exists
    //     2. Check FORCE ROW LEVEL SECURITY + CREATE POLICY tenant_isolation
    //     3. Check COMMENT ON TABLE ... 'distribution_column:tenant_id'
    // Return violations for each missing requirement
    vec![] // implementation body
}
```

---

## Acceptance Gates

```bash
cargo check -p oya-check-statelessness --all-features                # exit 0
cargo check -p oya-check-shardability --all-features                 # exit 0
cargo build -p oya-check-statelessness --all-features                # exit 0
cargo build -p oya-check-shardability --all-features                 # exit 0
cargo clippy -p oya-check-statelessness -- -D warnings               # exit 0
cargo clippy -p oya-check-shardability -- -D warnings                # exit 0
cargo nextest run -p oya-check-statelessness --test self_test        # exit 0; known violation detected
cargo nextest run -p oya-check-shardability --test self_test         # exit 0; known violation detected
# Run against workspace (report-only; may have violations in early substrate)
cargo run -p oya-check-statelessness -- --workspace --report-only   # exit 0; report generated
cargo run -p oya-check-shardability -- --migrations-dir migrations/ --report-only  # exit 0
cargo deny check                                                     # exit 0
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_extract_layer_from_name` | `oya-workflow-engine-worker` → `worker`; `oya-tenancy-kernel` → `kernel` |
| `test_statelessness_static_mut_detected` | `static mut COUNTER: u32` in application crate → violation |
| `test_statelessness_lazy_mutex_detected` | `Lazy<Mutex<Vec<...>>>` at module level → violation |
| `test_statelessness_clean_const_ok` | `const MAX: u32 = 100` → no violation |
| `test_shardability_missing_tenant_id` | Table without tenant_id column → TenantIdColumn violation |
| `test_shardability_missing_rls` | Table with tenant_id but no RLS → RlsPolicy violation |
| `test_shardability_missing_comment` | Table with RLS but no distribution_column comment → DistributionColumnComment violation |
| `test_shardability_clean_ok` | Fully compliant table → 0 violations |
| `test_shardability_global_table_exempt` | `cloud.cells` (no tenant_id by design) → no violation |

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent council-foundry \
  --intent "IP-001-ci-lanes-statelessness-shardability: two new fitness lane binaries" \
  --ttl 3600 \
  crates/oya-check-statelessness/src/main.rs::main \
  crates/oya-check-shardability/src/main.rs::main
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-ci-lanes-statelessness-shardability merged; oya-check-statelessness + oya-check-shardability operational in --report-only mode; self-tests green; next: IP-002-ci-lanes-perf-budget-benchmark" \
  -i high \
  -k "M02,P20,IP-001,ci-lanes"
```

---

## Halt Conditions

1. `syn` AST parsing fails on any workspace Rust file — fix the parser, not the source file.
2. SQL parser misclassifies a global infrastructure table (e.g., `cloud.cells`) as tenant-bound — add exemption list.
3. Self-test known-violation fixture is NOT detected — checker logic is broken; do not skip.

---

## Next IP Pointer

`IP-002-ci-lanes-perf-budget-benchmark.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0056 v4.1 (check-namespace exemption), ADR-0062 (quality/perf bar)
- Memory: `feedback_quality_performance_scalability_bar.md`, `feedback_clean_architecture_requirements.md`
