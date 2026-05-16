---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate
phase: P01-foundry-engine-consolidation
impl_plan_id: IP-P01-foundry-lean-checks
status: pending
owner: council-foundry
blocked_by: []
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-nextest
- cargo-deny
- lean-a1
- lean-a2
- lean-a3
- lean-a4
purpose: "Scaffolds all 10 foundry and check crates, implements 7 sub-commands on `oya-shared-architecture-check-cli` (dependency-direction, cross-product-refusal, port-location, layer-correctness, lib-name-parity, composition-root-only."
---
# IP-P01-foundry-lean-checks: Scaffold 10 foundry/check crates implementing 14 CI fitness lanes

## Intent

Scaffolds all 10 foundry and check crates, implements 7 sub-commands on `oya-shared-architecture-check-cli` (dependency-direction, cross-product-refusal, port-location, layer-correctness, lib-name-parity, composition-root-only, sdk-kernel-only), and delivers `oya-check-statelessness-cli`, `oya-check-shardability-cli`, `oya-check-perf-budget-cli` as standalone fitness-lane binaries. Together these implement all 14 CI enforcement lanes from ADR-0056 §CI matrix in `--report-only` mode; flipped to BLOCKER at M02 exit gate.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-shared-architecture-check-cli/Cargo.toml` | create | `[package]` + clap + cargo_metadata + walkdir |
| `crates/oya-shared-architecture-check-cli/src/main.rs` | create | clap CLI entry with 7 subcommands |
| `crates/oya-shared-architecture-check-cli/src/lib.rs` | create | module declarations |
| `crates/oya-shared-architecture-check-cli/src/checks/dependency_direction.rs` | create | parse Cargo.toml dep graph; enforce inward-only per 12-layer matrix |
| `crates/oya-shared-architecture-check-cli/src/checks/cross_product_refusal.rs` | create | verify no crate in µservice-A imports crate in µservice-B (except public_layers) |
| `crates/oya-shared-architecture-check-cli/src/checks/port_location.rs` | create | grep trait declarations; assert they live in kernel crates |
| `crates/oya-shared-architecture-check-cli/src/checks/layer_correctness.rs` | create | assert `[package.metadata.oya].layer` value matches crate-name suffix |
| `crates/oya-shared-architecture-check-cli/src/checks/lib_name_parity.rs` | create | assert `[lib] name` = snake_case(`[package] name`) |
| `crates/oya-shared-architecture-check-cli/src/checks/composition_root_only.rs` | create | assert only `app`-layer crates have unrestricted inward deps |
| `crates/oya-shared-architecture-check-cli/src/checks/sdk_kernel_only.rs` | create | assert `sdk`-layer crates import only `kernel`-layer crates |
| `crates/oya-shared-bounded-contexts-check-cli/Cargo.toml` | create | BC boundary checker |
| `crates/oya-shared-bounded-contexts-check-cli/src/main.rs` | create | reads docs/standards/bounded-contexts.md; checks each BC has registered crate family |
| `crates/oya-shared-supply-chain-check-cli/Cargo.toml` | create | wraps `cargo deny check` + SBOM generation |
| `crates/oya-shared-supply-chain-check-cli/src/main.rs` | create | invokes `cargo deny`; emits SBOM JSON |
| `crates/oya-shared-semver-check-cli/Cargo.toml` | create | API stability checker per ADR-0037 tiers |
| `crates/oya-shared-semver-check-cli/src/main.rs` | create | compares public API surface between commits |
| `crates/oya-check-statelessness-cli/Cargo.toml` | create | statelessness verifier |
| `crates/oya-check-statelessness-cli/src/main.rs` | create | AST grep: `static mut`, `lazy_static!`, `once_cell::sync::Lazy` in presentation/application/worker layer crates |
| `crates/oya-check-shardability-cli/Cargo.toml` | create | shardability verifier |
| `crates/oya-check-shardability-cli/src/main.rs` | create | parse migration SQL files; assert all multi-tenant tables have `tenant_id` column + RLS policy |
| `crates/oya-check-perf-budget-cli/Cargo.toml` | create | perf budget verifier |
| `crates/oya-check-perf-budget-cli/src/main.rs` | create | scan impl-plan.md files for `## Load test` section; report missing |
| `crates/oya-foundry-grit-cli/Cargo.toml` | create | thin wrapper over grit binary |
| `crates/oya-foundry-grit-cli/src/main.rs` | create | delegates to `grit` binary via std::process::Command |
| `crates/oya-foundry-icm-cli/Cargo.toml` | create | thin wrapper over icm binary |
| `crates/oya-foundry-icm-cli/src/main.rs` | create | delegates to `icm` binary via std::process::Command |
| `crates/oya-foundry-agent-read-cli/Cargo.toml` | create | agent-read scaffold |
| `crates/oya-foundry-agent-read-cli/src/main.rs` | create | reads AGENTS.md files for workspace agents |
| `Cargo.toml` | update | add all 10 crates to `[workspace.members]` |
| `deny.toml` | create | `cargo deny` configuration: licenses allowlist, bans, advisories |

---

## Crate Naming

```
NAME: oya-shared-architecture-check-cli
JUSTIFICATION:
- microservice = shared-architecture-check: cross-cutting architecture enforcement;
  ADR-0056 check-namespace exemption applies; registered under foundry
- bc-tokens = (none): single-concept binary
- layer = cli: CLI binary with subcommands; ADR-0056 §"Layer semantics"
- exemptions claimed: oya-check-* namespace BNF-exempt per ADR-0056

NAME: oya-check-statelessness-cli
JUSTIFICATION:
- microservice = check-statelessness: BNF-exempt check namespace
- bc-tokens = (none): single-concept
- layer = cli: CLI binary
- exemptions claimed: oya-check-* namespace BNF-exempt per ADR-0056
```

---

## Code Shape

### `crates/oya-shared-architecture-check-cli/src/main.rs`

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oya-shared-architecture-check-cli", version)]
struct Cli {
    #[arg(long, default_value = ".")]
    workspace: PathBuf,
    #[arg(long)]
    report_only: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Enforce inward-only dependency flow per 12-layer matrix
    DependencyDirection,
    /// Refuse direct cross-µservice imports (except public_layers)
    CrossProductRefusal,
    /// Assert port traits live in kernel crates
    PortLocation,
    /// Assert declared layer matches crate-name suffix
    LayerCorrectness,
    /// Assert [lib] name == snake_case([package] name)
    LibNameParity,
    /// Assert only app-layer crates have unrestricted inward deps
    CompositionRootOnly,
    /// Assert sdk-layer crates import only kernel-layer crates
    SdkKernelOnly,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let violations = match cli.command {
        Commands::DependencyDirection => checks::dependency_direction::run(&cli.workspace)?,
        Commands::CrossProductRefusal => checks::cross_product_refusal::run(&cli.workspace)?,
        Commands::PortLocation => checks::port_location::run(&cli.workspace)?,
        Commands::LayerCorrectness => checks::layer_correctness::run(&cli.workspace)?,
        Commands::LibNameParity => checks::lib_name_parity::run(&cli.workspace)?,
        Commands::CompositionRootOnly => checks::composition_root_only::run(&cli.workspace)?,
        Commands::SdkKernelOnly => checks::sdk_kernel_only::run(&cli.workspace)?,
    };
    if violations.is_empty() {
        println!("✓ 0 violations");
        return Ok(());
    }
    for v in &violations {
        eprintln!("VIOLATION: {v}");
    }
    if cli.report_only {
        eprintln!("{} violation(s) — report-only mode, not failing", violations.len());
        Ok(())
    } else {
        std::process::exit(1);
    }
}
```

### `crates/oya-shared-architecture-check-cli/src/checks/dependency_direction.rs`

```rust
use cargo_metadata::{MetadataCommand, Package};
use std::path::Path;

const LAYER_ORDER: &[&str] = &[
    "kernel", "domain", "application", "adapter", "infrastructure",
    "cli", "rest", "grpc", "graphql", "worker", "sdk", "app",
];

pub fn run(workspace: &Path) -> anyhow::Result<Vec<String>> {
    let metadata = MetadataCommand::new()
        .manifest_path(workspace.join("Cargo.toml"))
        .exec()?;
    let mut violations = Vec::new();
    for pkg in &metadata.packages {
        let Some(pkg_layer) = extract_layer(&pkg.name) else { continue };
        let pkg_rank = layer_rank(pkg_layer);
        for dep in &pkg.dependencies {
            let Some(dep_layer) = extract_layer(&dep.name) else { continue };
            let dep_rank = layer_rank(dep_layer);
            // outer layers may not be imported by inner layers (inward-only)
            if dep_rank > pkg_rank && pkg_layer != "app" {
                violations.push(format!(
                    "{} (layer={}) imports {} (layer={}) — forbidden outward dependency",
                    pkg.name, pkg_layer, dep.name, dep_layer
                ));
            }
        }
    }
    Ok(violations)
}

fn extract_layer(crate_name: &str) -> Option<&'static str> {
    LAYER_ORDER.iter().copied().find(|&l| crate_name.ends_with(&format!("-{l}")))
}

fn layer_rank(layer: &str) -> usize {
    LAYER_ORDER.iter().position(|&l| l == layer).unwrap_or(usize::MAX)
}
```

### `crates/oya-check-statelessness-cli/src/main.rs`

```rust
//! Scan presentation/application/worker layer crates for module-level mutable state.
//! Flags: `static mut`, `lazy_static!`, `once_cell::sync::Lazy` without explicit
//! `Send + Sync` bounds, raw `std::sync::Mutex` at module scope.

use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(name = "oya-check-statelessness-cli", version)]
struct Cli {
    #[arg(long, default_value = ".")]
    workspace: PathBuf,
    #[arg(long)]
    report_only: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let violations = statelessness::check_workspace(&cli.workspace)?;
    if violations.is_empty() { return Ok(()); }
    for v in &violations { eprintln!("STATELESS_VIOLATION: {v}"); }
    if cli.report_only { return Ok(()); }
    std::process::exit(1);
}

mod statelessness {
    use std::path::Path;
    pub fn check_workspace(workspace: &Path) -> anyhow::Result<Vec<String>> {
        // Walk crates/ dir; for each crate with layer in
        // [rest, grpc, graphql, worker, cli, application]: scan src/**/*.rs
        // for forbidden patterns via regex.
        let forbidden = [
            r"static\s+mut\s+",
            r"lazy_static!\s*\{",
            r"once_cell::sync::Lazy<(?!.*Send)",
        ];
        let mut violations = Vec::new();
        let crates_dir = workspace.join("crates");
        for entry in walkdir::WalkDir::new(&crates_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |x| x == "rs"))
        {
            let path = entry.path();
            let layer = extract_layer_from_path(path);
            if !matches!(layer, Some("rest"|"grpc"|"graphql"|"worker"|"cli"|"application")) {
                continue;
            }
            let src = std::fs::read_to_string(path)?;
            for pat in &forbidden {
                let re = regex::Regex::new(pat)?;
                if re.is_match(&src) {
                    violations.push(format!("{}: matches forbidden pattern `{pat}`", path.display()));
                }
            }
        }
        Ok(violations)
    }

    fn extract_layer_from_path(path: &std::path::Path) -> Option<&'static str> {
        let crate_dir = path.ancestors().find(|a| a.join("Cargo.toml").exists())?;
        let name = crate_dir.file_name()?.to_str()?;
        for layer in &["rest","grpc","graphql","worker","cli","application","adapter","kernel","domain","app","infrastructure","sdk"] {
            if name.ends_with(&format!("-{layer}")) { return Some(layer); }
        }
        None
    }
}
```

### `crates/oya-check-shardability-cli/src/main.rs`

```rust
//! Parse all migrations/**/*.sql files. For every CREATE TABLE statement,
//! verify the table has a tenant_id column and a matching RLS policy
//! (FORCE ROW LEVEL SECURITY + CREATE POLICY tenant_isolation).
//! Exempt: single-tenant system tables (listed in .oya-shardability-exempt.toml).

use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(name = "oya-check-shardability-cli", version)]
struct Cli {
    #[arg(long, default_value = ".")]
    workspace: PathBuf,
    #[arg(long)]
    report_only: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let violations = shardability::check_migrations(&cli.workspace)?;
    if violations.is_empty() { return Ok(()); }
    for v in &violations { eprintln!("SHARDABILITY_VIOLATION: {v}"); }
    if cli.report_only { return Ok(()); }
    std::process::exit(1);
}

mod shardability {
    use std::path::Path;
    use regex::Regex;
    pub fn check_migrations(workspace: &Path) -> anyhow::Result<Vec<String>> {
        let mut violations = Vec::new();
        for entry in walkdir::WalkDir::new(workspace.join("migrations"))
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |x| x == "sql"))
        {
            let sql = std::fs::read_to_string(entry.path())?;
            let table_re = Regex::new(r"CREATE TABLE (\w+\.\w+)")?;
            for cap in table_re.captures_iter(&sql) {
                let table = &cap[1];
                if !sql.contains(&format!("tenant_id")) {
                    violations.push(format!("{}: table {table} missing tenant_id column", entry.path().display()));
                }
                if !sql.contains("FORCE ROW LEVEL SECURITY") {
                    violations.push(format!("{}: table {table} missing FORCE ROW LEVEL SECURITY", entry.path().display()));
                }
            }
        }
        Ok(violations)
    }
}
```

---

## Acceptance Gates

```bash
# 1. Compile all check crates
cargo check -p oya-shared-architecture-check-cli --all-features       # exit 0
cargo check -p oya-check-statelessness-cli --all-features              # exit 0
cargo check -p oya-check-shardability-cli --all-features               # exit 0
cargo check -p oya-check-perf-budget-cli --all-features                # exit 0

# 2. Build
cargo build -p oya-shared-architecture-check-cli --all-features        # exit 0

# 3. Lint
cargo clippy --workspace --all-features -- -D warnings                 # exit 0

# 4. Tests
cargo nextest run --workspace --all-features                           # exit 0; 0 failures

# 5. Supply chain
cargo deny check                                                       # exit 0

# 6. Docs
cargo doc --workspace --no-deps                                        # exit 0

# 7. Smoke: each subcommand exits 0 in report-only mode on current workspace
oya-shared-architecture-check-cli dependency-direction --workspace . --report-only
oya-shared-architecture-check-cli cross-product-refusal --workspace . --report-only
oya-shared-architecture-check-cli port-location --workspace . --report-only
oya-shared-architecture-check-cli layer-correctness --workspace . --report-only
oya-shared-architecture-check-cli lib-name-parity --workspace . --report-only
oya-shared-architecture-check-cli composition-root-only --workspace . --report-only
oya-shared-architecture-check-cli sdk-kernel-only --workspace . --report-only
oya-check-statelessness-cli --workspace . --report-only
oya-check-shardability-cli --workspace . --report-only
oya-check-perf-budget-cli --workspace . --report-only
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_layer_rank_ordering` | 12 layer values have correct ordinal rank |
| `test_dependency_direction_violation_detected` | cross-layer import flagged |
| `test_dependency_direction_clean` | inward-only dep graph passes |
| `test_cross_product_refusal_violation` | direct cross-µservice import flagged |
| `test_port_location_kernel_trait_passes` | trait in kernel passes |
| `test_port_location_domain_trait_flagged` | trait in domain flagged |
| `test_statelessness_static_mut_flagged` | `static mut` in application layer flagged |
| `test_shardability_missing_tenant_id_flagged` | CREATE TABLE without tenant_id flagged |
| `test_shardability_rls_missing_flagged` | table without FORCE ROW LEVEL SECURITY flagged |
| `test_perf_budget_missing_load_test_section` | impl-plan without `## Load test` flagged |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_architecture_check_clean_workspace` | All 7 subcommands exit 0 on a known-clean fixture workspace |

---

## Clean Architecture Compliance

### Dependency direction check

| Crate | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-shared-architecture-check-cli` | `cli` | `cargo_metadata` (external), `anyhow`, `clap` | no project-internal imports |
| `oya-check-statelessness-cli` | `cli` | `walkdir`, `regex`, `anyhow`, `clap` | no project-internal imports |
| `oya-check-shardability-cli` | `cli` | `walkdir`, `regex`, `anyhow`, `clap` | no project-internal imports |
| `oya-check-perf-budget-cli` | `cli` | `walkdir`, `anyhow`, `clap` | no project-internal imports |

### Cross-product integration check

No product µservice imports. These are pure standalone check tools.

---

## Load Test

Foundry check tools are invoked in CI, not on hot paths. Performance target: each check binary completes in ≤30s on a 500-crate workspace.

```bash
# Time the full architecture check on current workspace
time oya-shared-architecture-check-cli dependency-direction --workspace .
# Pass criterion: real time ≤30s

time oya-check-shardability-cli --workspace .
# Pass criterion: real time ≤10s
```

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent m02-wave-a-executor \
  --intent "IP-P01: scaffold 10 foundry/check crates; implement 14 CI lanes" \
  --ttl 7200 \
  crates/oya-shared-architecture-check-cli/src/main.rs::main \
  crates/oya-check-statelessness-cli/src/main.rs::main \
  crates/oya-check-shardability-cli/src/main.rs::main \
  crates/oya-check-perf-budget-cli/src/main.rs::main
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P01-foundry-lean-checks merged; crates scaffolded: oya-shared-architecture-check-cli + 9 others; 14 CI lanes implemented --report-only; grit symbols released; next IP: P02-ontology/impl-plan" \
  -i high \
  -k "M02,P01,IP-P01,foundry,lean-checks"
```

---

## Halt Conditions

1. `cargo_metadata` parsing fails on workspace — ensure `[workspace]` Cargo.toml is valid.
2. LEAN-A2 cross-product-refusal reports false positives for `public_layers` — fix the `public_layers` allowlist logic before landing.
3. Any acceptance gate exits non-zero after 3 attempts — escalate to architect.

---

## Next IP Pointer

`phases/P02-ontology/impl-plan.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0056 (BNF v4.1 + 14 CI lane matrix)
- ADR-0057 (LEAN checks)
- `feedback_clean_architecture_requirements.md §13`
