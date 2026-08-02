//! Disk-walking runners for the 4 scalability lanes that delegate to:
//!   - check-statelessness
//!   - check-shardability
//!   - check-perf-budget
//!   - check-benchmark
//!
//! Each runner: parse CLI args → harvest typed Node list from disk →
//! call the I/O-free kernel → emit success or failure report.

use std::fs;
use std::path::{Path, PathBuf};

use check_benchmark::{
    Prd, Report as BenchmarkReport, ViolationKind as BenchmarkViolationKind,
    check as check_benchmark,
};
use check_perf_budget::{
    ImplementationPlan, Report as PerfBudgetReport, ViolationKind as PerfBudgetViolationKind,
    check as check_perf_budget,
};
use check_shardability::{
    MigrationFile, Report as ShardabilityReport, check as check_shardability,
};
use check_statelessness::{
    Report as StatelessnessReport, SCOPED_LAYERS, SourceFile, check as check_statelessness,
};

// ─── statelessness ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StatelessnessValidateArgs {
    workspace_root: PathBuf,
    allow_empty: bool,
}

pub(crate) fn parse_statelessness_validate_args(
    args: Vec<String>,
) -> Result<StatelessnessValidateArgs, String> {
    let mut workspace_root = PathBuf::from(".");
    let mut allow_empty = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--workspace-root" => {
                workspace_root = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--workspace-root requires a path".to_string())?,
                );
            }
            "--allow-empty" => allow_empty = true,
            other => return Err(format!("unexpected statelessness argument: {other}")),
        }
    }
    Ok(StatelessnessValidateArgs {
        workspace_root,
        allow_empty,
    })
}

pub(crate) fn validate_statelessness_gate(
    args: StatelessnessValidateArgs,
) -> Result<StatelessnessReport, String> {
    let crates_dir = args.workspace_root.join("crates");
    if !crates_dir.is_dir() {
        return Err(format!("crates dir not present: {}", crates_dir.display()));
    }
    let mut files = Vec::new();
    for entry in
        fs::read_dir(&crates_dir).map_err(|error| format!("{}: {error}", crates_dir.display()))?
    {
        let entry = entry.map_err(|error| format!("{}: {error}", crates_dir.display()))?;
        let crate_dir = entry.path();
        if !crate_dir.is_dir() {
            continue;
        }
        let crate_id = crate_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        if crate_id.is_empty() {
            continue;
        }
        let layer = derive_layer_from_crate_id(&crate_id);
        if !SCOPED_LAYERS.contains(&layer.as_str()) {
            continue;
        }
        let src_dir = crate_dir.join("src");
        if !src_dir.is_dir() {
            continue;
        }
        collect_rust_sources(&src_dir, &crate_id, &layer, &mut files)?;
    }
    let report = check_statelessness(&files)
        .map_err(|error| format!("statelessness kernel error: {error}"))?;
    if report.files_in_scope == 0 && !args.allow_empty {
        return Err(format!(
            "statelessness validation has zero outer-ring files in scope (scanned {} total). \
             Pass --allow-empty to acknowledge a tree with no usecase/app/worker/rest/grpc/\
             api/cli/sdk crates is intentional; otherwise the lane refuses to falsely claim pass.",
            report.files_checked
        ));
    }
    if !report.violations.is_empty() {
        let mut msg = format!(
            "statelessness violations: {} (in {} files in scope; {} files total scanned)\n",
            report.violations.len(),
            report.files_in_scope,
            report.files_checked
        );
        for v in &report.violations {
            msg.push_str(&format!(
                "  {} ({}): {} at line {} — {}\n",
                v.crate_id, v.path, v.kind, v.line, v.excerpt
            ));
        }
        return Err(msg);
    }
    Ok(report)
}

fn derive_layer_from_crate_id(crate_id: &str) -> String {
    crate_id
        .rsplit_once('-')
        .map(|(_, last)| last.to_string())
        .unwrap_or_default()
}

fn collect_rust_sources(
    dir: &Path,
    crate_id: &str,
    layer: &str,
    out: &mut Vec<SourceFile>,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))? {
        let entry = entry.map_err(|error| format!("{}: {error}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_sources(&path, crate_id, layer, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let content = fs::read_to_string(&path)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            out.push(SourceFile {
                crate_id: crate_id.to_string(),
                layer: layer.to_string(),
                path: path.display().to_string(),
                content,
            });
        }
    }
    Ok(())
}

// ─── shardability ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShardabilityValidateArgs {
    migrations_dir: PathBuf,
    allow_empty: bool,
}

pub(crate) fn parse_shardability_validate_args(
    args: Vec<String>,
) -> Result<ShardabilityValidateArgs, String> {
    let mut migrations_dir = PathBuf::from("migrations");
    let mut allow_empty = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--migrations-dir" => {
                migrations_dir = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--migrations-dir requires a path".to_string())?,
                );
            }
            "--allow-empty" => allow_empty = true,
            other => return Err(format!("unexpected shardability argument: {other}")),
        }
    }
    Ok(ShardabilityValidateArgs {
        migrations_dir,
        allow_empty,
    })
}

pub(crate) fn validate_shardability_gate(
    args: ShardabilityValidateArgs,
) -> Result<ShardabilityReport, String> {
    let mut files = Vec::new();
    if args.migrations_dir.is_dir() {
        collect_sql_files(&args.migrations_dir, &mut files)?;
    }
    let report = check_shardability(&files)
        .map_err(|error| format!("shardability kernel error: {error}"))?;
    if report.tables_seen == 0 && !args.allow_empty {
        return Err(format!(
            "shardability validation has zero CREATE TABLE statements (migrations_dir: {}). \
             Pass --allow-empty to acknowledge the M02 substrate has not yet shipped migrations; \
             otherwise the lane refuses to falsely claim pass.",
            args.migrations_dir.display()
        ));
    }
    if !report.violations.is_empty() {
        let mut msg = format!(
            "shardability violations: {} (tables seen: {}, global opt-outs: {})\n",
            report.violations.len(),
            report.tables_seen,
            report.tables_global
        );
        for v in &report.violations {
            msg.push_str(&format!(
                "  {} table `{}` at line {} — missing tenant_id\n",
                v.path, v.table_name, v.line
            ));
        }
        return Err(msg);
    }
    Ok(report)
}

fn collect_sql_files(dir: &Path, out: &mut Vec<MigrationFile>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))? {
        let entry = entry.map_err(|error| format!("{}: {error}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_sql_files(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("sql") {
            let content = fs::read_to_string(&path)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            out.push(MigrationFile {
                path: path.display().to_string(),
                content,
            });
        }
    }
    Ok(())
}

// ─── perf-budget ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PerfBudgetValidateArgs {
    plans_dir: PathBuf,
    allow_empty: bool,
}

pub(crate) fn parse_perf_budget_validate_args(
    args: Vec<String>,
) -> Result<PerfBudgetValidateArgs, String> {
    let mut plans_dir = PathBuf::from(".omc/plans/milestones");
    let mut allow_empty = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--plans-dir" => {
                plans_dir = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--plans-dir requires a path".to_string())?,
                );
            }
            "--allow-empty" => allow_empty = true,
            other => return Err(format!("unexpected perf-budget argument: {other}")),
        }
    }
    Ok(PerfBudgetValidateArgs {
        plans_dir,
        allow_empty,
    })
}

pub(crate) fn validate_perf_budget_gate(
    args: PerfBudgetValidateArgs,
) -> Result<PerfBudgetReport, String> {
    let mut plans = Vec::new();
    if args.plans_dir.is_dir() {
        collect_ip_markdowns(&args.plans_dir, &mut plans)?;
    }
    let report =
        check_perf_budget(&plans).map_err(|error| format!("perf-budget kernel error: {error}"))?;
    if report.plans_checked == 0 && !args.allow_empty {
        return Err(format!(
            "perf-budget validation has zero IP markdowns (plans_dir: {}). \
             Pass --allow-empty to acknowledge an empty plans tree is intentional.",
            args.plans_dir.display()
        ));
    }
    if !report.violations.is_empty() {
        let mut msg = format!(
            "perf-budget violations: {} (plans checked: {})\n",
            report.violations.len(),
            report.plans_checked
        );
        for v in &report.violations {
            let label = match v.kind {
                PerfBudgetViolationKind::SectionMissing => "missing `## Load test` section",
                PerfBudgetViolationKind::SectionEmpty => "empty `## Load test` section",
                PerfBudgetViolationKind::SectionMissingNumbers => {
                    "`## Load test` section has no concrete performance measurements"
                }
            };
            msg.push_str(&format!("  {} — {}\n", v.path, label));
        }
        return Err(msg);
    }
    Ok(report)
}

fn collect_ip_markdowns(dir: &Path, out: &mut Vec<ImplementationPlan>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))? {
        let entry = entry.map_err(|error| format!("{}: {error}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_ip_markdowns(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if file_name.starts_with("IP-") {
                let content = fs::read_to_string(&path)
                    .map_err(|error| format!("{}: {error}", path.display()))?;
                out.push(ImplementationPlan {
                    path: path.display().to_string(),
                    content,
                });
            }
        }
    }
    Ok(())
}

// ─── benchmark ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BenchmarkValidateArgs {
    prds_dir: PathBuf,
    products_dir: PathBuf,
    competitors: Vec<String>,
    allow_empty: bool,
}

pub(crate) fn parse_benchmark_validate_args(
    args: Vec<String>,
) -> Result<BenchmarkValidateArgs, String> {
    // Default competitor registry — known hyperscaler-grade reference points
    // (per docs/standards/hyperscaler-best-practices.md). Extend via repeated
    // --competitor flags at invocation time.
    let mut competitors: Vec<String> = vec![
        "stripe",
        "linear",
        "palantir",
        "n8n",
        "snowflake",
        "databricks",
        "anthropic",
        "openai",
        "google",
        "aws",
        "azure",
        "salesforce",
        "github",
        "atlassian",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let mut prds_dir = PathBuf::from("docs/prds");
    let mut products_dir = PathBuf::from("docs/products");
    let mut allow_empty = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--prds-dir" => {
                prds_dir = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--prds-dir requires a path".to_string())?,
                );
            }
            "--products-dir" => {
                products_dir = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--products-dir requires a path".to_string())?,
                );
            }
            "--competitor" => {
                competitors.push(
                    iter.next()
                        .ok_or_else(|| "--competitor requires a name".to_string())?,
                );
            }
            "--allow-empty" => allow_empty = true,
            other => return Err(format!("unexpected benchmark argument: {other}")),
        }
    }
    Ok(BenchmarkValidateArgs {
        prds_dir,
        products_dir,
        competitors,
        allow_empty,
    })
}

pub(crate) fn validate_benchmark_gate(
    args: BenchmarkValidateArgs,
) -> Result<BenchmarkReport, String> {
    let mut prds = Vec::new();
    if args.prds_dir.is_dir() {
        collect_prd_markdowns(&args.prds_dir, &mut prds)?;
    }
    if args.products_dir.is_dir() {
        collect_prd_markdowns(&args.products_dir, &mut prds)?;
    }
    let competitor_slice: Vec<&str> = args.competitors.iter().map(|s| s.as_str()).collect();
    let report = check_benchmark(&prds, &competitor_slice)
        .map_err(|error| format!("benchmark kernel error: {error}"))?;
    if report.prds_checked == 0 && !args.allow_empty {
        return Err(format!(
            "benchmark validation has zero PRD markdowns (prds_dir: {}, products_dir: {}). \
             Pass --allow-empty to acknowledge an empty PRD tree is intentional.",
            args.prds_dir.display(),
            args.products_dir.display()
        ));
    }
    if !report.violations.is_empty() {
        let mut msg = format!(
            "benchmark violations: {} (PRDs checked: {})\n",
            report.violations.len(),
            report.prds_checked
        );
        for v in &report.violations {
            let label = match v.kind {
                BenchmarkViolationKind::SectionMissing => {
                    "missing `## Competitive benchmark` section"
                }
                BenchmarkViolationKind::SectionEmpty => "empty `## Competitive benchmark` section",
                BenchmarkViolationKind::SectionUnsubstantiated => {
                    "`## Competitive benchmark` has no digit and no recognized competitor"
                }
            };
            msg.push_str(&format!("  {} — {}\n", v.path, label));
        }
        return Err(msg);
    }
    Ok(report)
}

fn collect_prd_markdowns(dir: &Path, out: &mut Vec<Prd>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))? {
        let entry = entry.map_err(|error| format!("{}: {error}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_prd_markdowns(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            // Accept both `PRD.md` (under docs/products/<name>/) and any
            // `*.md` under docs/prds/ (where filename is the product slug).
            let is_prd = file_name.eq_ignore_ascii_case("prd.md")
                || file_name.to_ascii_lowercase().starts_with("prd-")
                || dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.eq_ignore_ascii_case("prds"))
                    .unwrap_or(false);
            if is_prd {
                let content = fs::read_to_string(&path)
                    .map_err(|error| format!("{}: {error}", path.display()))?;
                out.push(Prd {
                    path: path.display().to_string(),
                    content,
                });
            }
        }
    }
    Ok(())
}
