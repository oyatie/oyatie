use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::Value;

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("retired-and-renumber") => match RetiredRenumberArgs::parse(args.collect(), usage) {
            Ok(parsed) => match run_retired_and_renumber(parsed) {
                Ok(report) => {
                    println!(
                        "retired-and-renumber {}: plan={}, renumber_map={}, deletes={}, renumbered={}, rewrite_files={}",
                        if report.applied { "applied" } else { "planned" },
                        report.plan_path.display(),
                        report.renumber_map_path.display(),
                        report.delete_count,
                        report.renumber_count,
                        report.rewrite_file_count,
                    );
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("retired-and-renumber failed: {message}");
                    ExitCode::FAILURE
                }
            },
            Err(message) => {
                eprintln!("{message}");
                ExitCode::from(2)
            }
        },
        _ => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RetiredRenumberArgs {
    plan_path: PathBuf,
    renumber_map_path: PathBuf,
    apply: bool,
}

impl RetiredRenumberArgs {
    fn parse(args: Vec<String>, usage: &str) -> Result<Self, String> {
        let mut plan_path = None;
        let mut renumber_map_path = None;
        let mut apply = false;
        let mut iter = args.into_iter();
        while let Some(flag) = iter.next() {
            match flag.as_str() {
                "--plan" => {
                    let Some(value) = iter.next() else {
                        return Err(usage.to_owned());
                    };
                    plan_path = Some(PathBuf::from(value));
                }
                "--renumber-map" => {
                    let Some(value) = iter.next() else {
                        return Err(usage.to_owned());
                    };
                    renumber_map_path = Some(PathBuf::from(value));
                }
                "--apply" => apply = true,
                _ => return Err(usage.to_owned()),
            }
        }
        Ok(Self {
            plan_path: plan_path.ok_or_else(|| usage.to_owned())?,
            renumber_map_path: renumber_map_path.ok_or_else(|| usage.to_owned())?,
            apply,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CleanupReport {
    plan_path: PathBuf,
    renumber_map_path: PathBuf,
    applied: bool,
    delete_count: usize,
    renumber_count: usize,
    rewrite_file_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AdrRecord {
    path: PathBuf,
    old_id: String,
    new_id: Option<String>,
    status: String,
    delete_reason: Option<String>,
    successor_old_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CleanupPlan {
    tombstones: Vec<PathBuf>,
    retired_specs: Vec<PathBuf>,
    retired_microservice_dirs: Vec<PathBuf>,
    capability_tier_paths: Vec<PathBuf>,
    adr_records: Vec<AdrRecord>,
    tier_reference_files: Vec<PathBuf>,
    primitive_reference_files: Vec<PathBuf>,
}

fn run_retired_and_renumber(args: RetiredRenumberArgs) -> Result<CleanupReport, String> {
    let mut plan = build_cleanup_plan()?;
    assign_renumber_ids(&mut plan.adr_records);
    write_plan(&args.plan_path, &plan)?;
    write_renumber_map(&args.renumber_map_path, &plan.adr_records)?;
    let delete_count = plan.delete_path_count();
    let renumber_count = plan
        .adr_records
        .iter()
        .filter(|record| record.delete_reason.is_none())
        .filter(|record| {
            record
                .new_id
                .as_ref()
                .is_some_and(|new_id| new_id != &record.old_id)
        })
        .count();
    let rewrite_file_count = plan.rewrite_file_count();

    if args.apply {
        apply_cleanup(&plan)?;
    }

    Ok(CleanupReport {
        plan_path: args.plan_path,
        renumber_map_path: args.renumber_map_path,
        applied: args.apply,
        delete_count,
        renumber_count,
        rewrite_file_count,
    })
}

impl CleanupPlan {
    fn delete_path_count(&self) -> usize {
        self.tombstones.len()
            + self.retired_specs.len()
            + self.retired_microservice_dirs.len()
            + self.capability_tier_paths.len()
            + self
                .adr_records
                .iter()
                .filter(|record| record.delete_reason.is_some())
                .count()
    }

    fn rewrite_file_count(&self) -> usize {
        self.tier_reference_files
            .iter()
            .chain(self.primitive_reference_files.iter())
            .cloned()
            .collect::<BTreeSet<_>>()
            .len()
    }
}

fn build_cleanup_plan() -> Result<CleanupPlan, String> {
    let tombstones = find_tombstones(Path::new("."))?;
    let retired_specs = find_retired_specs(Path::new("specs"), Path::new("docs"))?;
    // Scan every service root for retired directories. Roots come from
    // `crate::service_roots` (derived from the closed capability registry);
    // an expected root that is absent is an error there, not an empty scan.
    let retired_microservice_dirs: Vec<PathBuf> = {
        let mut dirs = Vec::new();
        for root in crate::service_roots::default_service_roots()? {
            dirs.extend(find_retired_microservice_dirs(&root)?);
        }
        dirs
    };
    let capability_tier_paths = find_capability_tier_paths(Path::new("."))?;
    let adr_records = read_adr_records(Path::new("docs/decisions"))?;
    let tier_reference_files = find_files_containing_any(
        &[
            PathBuf::from("docs"),
            PathBuf::from("crates"),
            PathBuf::from("specs"),
        ],
        &[
            "Bronze",
            "Silver",
            "Gold",
            "Platinum",
            "capability_tier",
            "tier_bronze",
            "tier_silver",
            "tier_gold",
            "tier_platinum",
            "capability-tier",
            "capability tier",
        ],
    )?;
    let primitive_reference_files = find_files_containing_token_any(
        &[
            PathBuf::from("docs"),
            PathBuf::from("specs"),
            PathBuf::from("scripts"),
            PathBuf::from(".github"),
            PathBuf::from("tools"),
        ],
        &["grit", "rtk", "icm", "vox"],
    )?;

    Ok(CleanupPlan {
        tombstones,
        retired_specs,
        retired_microservice_dirs,
        capability_tier_paths,
        adr_records,
        tier_reference_files,
        primitive_reference_files,
    })
}

fn find_tombstones(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    walk_files(root, &mut |path| {
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("RETIRED") && name.ends_with(".md"))
        {
            paths.push(path.to_path_buf());
        }
        Ok(())
    })?;
    paths.sort();
    Ok(paths)
}

fn find_retired_specs(specs_root: &Path, docs_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for root in [specs_root, docs_root] {
        if !root.exists() {
            continue;
        }
        walk_files(root, &mut |path| {
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                return Ok(());
            }
            let Ok(content) = fs::read_to_string(path) else {
                return Ok(());
            };
            if json_has_retired_marker(&content) {
                paths.push(path.to_path_buf());
            }
            Ok(())
        })?;
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn json_has_retired_marker(content: &str) -> bool {
    let Ok(json) = serde_json::from_str::<Value>(content) else {
        return content.contains("\"status\": \"Retired\"")
            || content.contains("\"doc_class\": \"RetiredMicroserviceMarker\"");
    };
    let meta = json.get("_meta");
    value_string_eq(meta.and_then(|value| value.get("status")), "Retired")
        || value_string_eq(json.get("status"), "Retired")
        || value_string_eq(
            meta.and_then(|value| value.get("doc_class")),
            "RetiredMicroserviceMarker",
        )
        || value_string_eq(
            meta.and_then(|value| value.get("doc_class")),
            "Microservice-Retirement-Marker",
        )
}

fn value_string_eq(value: Option<&Value>, expected: &str) -> bool {
    value.and_then(Value::as_str) == Some(expected)
}

fn find_retired_microservice_dirs(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut dirs = Vec::new();
    if !root.exists() {
        return Ok(dirs);
    }
    for entry in fs::read_dir(root)
        .map_err(|error| format!("microservices dir unreadable {}: {error}", root.display()))?
    {
        let entry =
            entry.map_err(|error| format!("microservices dir entry unreadable: {error}"))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest = path.join("manifest.json");
        if !manifest.exists() {
            continue;
        }
        let content = fs::read_to_string(&manifest)
            .map_err(|error| format!("manifest unreadable {}: {error}", manifest.display()))?;
        if json_has_retired_marker(&content) {
            dirs.push(path);
        }
    }
    dirs.sort();
    Ok(dirs)
}

fn find_capability_tier_paths(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    walk_paths(root, &mut |path| {
        let slash = slash_path(path);
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if slash.starts_with("./docs/decisions/ADR-") {
            return Ok(());
        }
        if slash.starts_with("./registry/capability-tiers")
            || slash.contains("/capability-tiers/")
            || (!slash.starts_with("./crates/")
                && (file_name.contains("capability-tier") || file_name.contains("capability_tier")))
            || slash == "./specs/capability-tier-schema.json"
        {
            paths.push(path.to_path_buf());
        }
        Ok(())
    })?;
    paths.sort_by(|left, right| {
        right
            .components()
            .count()
            .cmp(&left.components().count())
            .then(left.cmp(right))
    });
    paths.dedup();
    Ok(paths)
}

fn read_adr_records(decisions_dir: &Path) -> Result<Vec<AdrRecord>, String> {
    let mut records = Vec::new();
    let mut paths = fs::read_dir(decisions_dir)
        .map_err(|error| format!("ADR dir unreadable {}: {error}", decisions_dir.display()))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|error| format!("ADR dir entry unreadable: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("ADR-") && name.ends_with(".md"))
    });
    paths.sort();
    for path in paths {
        let content = fs::read_to_string(&path)
            .map_err(|error| format!("ADR unreadable {}: {error}", path.display()))?;
        let Some(old_id) = adr_id_from_path(&path) else {
            continue;
        };
        let status = frontmatter_value(&content, "status").unwrap_or_else(|| "unknown".to_owned());
        let delete_reason = adr_delete_reason(&path, &status);
        let successor_old_id = if delete_reason.is_some() {
            successor_for_deleted_adr(&old_id, &path, &content)
        } else {
            None
        };
        records.push(AdrRecord {
            path,
            old_id,
            new_id: None,
            status,
            delete_reason,
            successor_old_id,
        });
    }
    Ok(records)
}

fn adr_delete_reason(path: &Path, status: &str) -> Option<String> {
    let lower_status = status.to_ascii_lowercase();
    if matches!(
        lower_status.as_str(),
        "superseded" | "deprecated" | "retired"
    ) {
        return Some(format!("frontmatter status={status}"));
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let hard_retired = [
        "inventory-grit-cutover",
        "grit-icm-as-sanctioned-primitives",
        "grit-scaffold-claim-pattern",
        "grit-cutover-inventory",
        "retire-external-agent-coordination-tooling",
        "retire-archive-orphan-fitness-lane",
        "foundry-six-path-deprecation",
        "capability-tier-over-product-fragmentation",
        "capability-tier-pricing-anchors-public",
        "tier-system-retired-replaced-by-tenant-class",
        "cell-microservice-retired-pattern-not-service",
        "shorts-microservice-merged-into-social",
        "foundry-microservice-retired-absorbed-by-intelligence",
    ];
    hard_retired
        .iter()
        .find(|needle| name.contains(**needle))
        .map(|needle| format!("Wave 15-ZH retired/deprecated marker slug={needle}"))
}

fn successor_for_deleted_adr(old_id: &str, path: &Path, content: &str) -> Option<String> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let hardcoded = [
        ("ADR-0052", "ADR-0113"),
        ("ADR-0053", "ADR-0113"),
        ("ADR-0054", "ADR-0113"),
        ("ADR-0103", "ADR-0113"),
        ("ADR-0116", "ADR-0113"),
        ("ADR-0118", "ADR-0113"),
        ("ADR-0138", "ADR-0255"),
        ("ADR-0316", "ADR-0330"),
        ("ADR-0325", "ADR-0330"),
        ("ADR-0329", "ADR-0330"),
        ("ADR-0333", "ADR-0351"),
        ("ADR-0334", "ADR-0132"),
        ("ADR-0335", "ADR-0255"),
    ];
    if let Some((_, successor)) = hardcoded.iter().find(|(id, _)| *id == old_id) {
        return Some((*successor).to_owned());
    }
    if name.contains("retire-external-agent-coordination-tooling") {
        return Some("ADR-0113".to_owned());
    }
    frontmatter_successor(content).or_else(|| body_successor(content))
}

fn assign_renumber_ids(records: &mut [AdrRecord]) {
    let mut next = 1usize;
    for record in records {
        if record.delete_reason.is_some() {
            record.new_id = None;
            continue;
        }
        record.new_id = Some(format!("ADR-{next:04}"));
        next += 1;
    }
}

fn write_plan(path: &Path, plan: &CleanupPlan) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("plan dir unwritable {}: {error}", parent.display()))?;
    }
    let mut out = String::new();
    out.push_str("# Wave 15-ZH deletion and renumber plan\n\n");
    out.push_str("Generated by `oya cleanup retired-and-renumber`.\n\n");
    push_path_section(&mut out, "Pass A delete: tombstones", &plan.tombstones);
    push_path_section(
        &mut out,
        "Pass A delete: retired spec stubs",
        &plan.retired_specs,
    );
    push_path_section(
        &mut out,
        "Pass A delete: retired microservice directories with retired manifests",
        &plan.retired_microservice_dirs,
    );
    push_path_section(
        &mut out,
        "Pass A delete: capability-tier registry/docs",
        &plan.capability_tier_paths,
    );
    out.push_str("## Pass A delete: ADRs\n\n");
    for record in plan
        .adr_records
        .iter()
        .filter(|record| record.delete_reason.is_some())
    {
        let successor = record.successor_old_id.as_deref().unwrap_or("none");
        out.push_str(&format!(
            "- `{}` -> DELETE; reason: {}; successor rewrite: `{}`\n",
            slash_path(&record.path),
            record.delete_reason.as_deref().unwrap_or("unspecified"),
            successor,
        ));
    }
    out.push('\n');
    push_path_section(
        &mut out,
        "Pass B edit: capability-tier and Bronze/Silver/Gold/Platinum references",
        &plan.tier_reference_files,
    );
    push_path_section(
        &mut out,
        "Pass B edit: grit/rtk/icm/vox primitive references",
        &plan.primitive_reference_files,
    );
    out.push_str("## Pass B edit: deleted ADR citation rewrites\n\n");
    for record in plan
        .adr_records
        .iter()
        .filter(|record| record.delete_reason.is_some())
    {
        match &record.successor_old_id {
            Some(successor) => out.push_str(&format!("- `{}` -> `{successor}`\n", record.old_id)),
            None => out.push_str(&format!(
                "- `{}` -> remove/skip when only present in deleted files\n",
                record.old_id
            )),
        }
    }
    out.push('\n');
    out.push_str("## Pass C renumber\n\n");
    for record in plan
        .adr_records
        .iter()
        .filter(|record| record.delete_reason.is_none())
    {
        let new_id = record.new_id.as_deref().unwrap_or("UNASSIGNED");
        out.push_str(&format!(
            "- `{}` -> `{}` for `{}`\n",
            record.old_id,
            new_id,
            slash_path(&record.path)
        ));
    }
    out.push('\n');
    out.push_str("## Explicit deferral\n\n");
    out.push_str(
        "- ULID references are intentionally deferred to Wave 15-ZH2 per mission scope.\n",
    );
    fs::write(path, out).map_err(|error| format!("plan unwritable {}: {error}", path.display()))
}

fn push_path_section(out: &mut String, title: &str, paths: &[PathBuf]) {
    out.push_str(&format!("## {title}\n\n"));
    out.push_str(&format!("Count: {}\n\n", paths.len()));
    for path in paths {
        out.push_str(&format!("- `{}`\n", slash_path(path)));
    }
    out.push('\n');
}

fn write_renumber_map(path: &Path, records: &[AdrRecord]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("renumber dir unwritable {}: {error}", parent.display()))?;
    }
    let mut out = String::from("old_id\tnew_id\tstatus\n");
    for record in records {
        match &record.delete_reason {
            Some(reason) => out.push_str(&format!("{}\tDELETED\t{}\n", record.old_id, reason)),
            None => out.push_str(&format!(
                "{}\t{}\tkept:{}\n",
                record.old_id,
                record.new_id.as_deref().unwrap_or("UNASSIGNED"),
                slash_path(&record.path)
            )),
        }
    }
    fs::write(path, out)
        .map_err(|error| format!("renumber map unwritable {}: {error}", path.display()))
}

fn apply_cleanup(plan: &CleanupPlan) -> Result<(), String> {
    for path in &plan.tombstones {
        remove_file_if_exists(path)?;
    }
    for path in &plan.retired_specs {
        remove_file_if_exists(path)?;
    }
    for path in &plan.capability_tier_paths {
        remove_path_if_exists(path)?;
    }
    for path in &plan.retired_microservice_dirs {
        remove_path_if_exists(path)?;
    }
    for record in plan
        .adr_records
        .iter()
        .filter(|record| record.delete_reason.is_some())
    {
        remove_file_if_exists(&record.path)?;
    }

    rewrite_deleted_adr_citations(plan)?;
    rewrite_retired_terms()?;
    apply_adr_renumber(plan)?;
    Ok(())
}

fn rewrite_deleted_adr_citations(plan: &CleanupPlan) -> Result<(), String> {
    let mut replacements = Vec::new();
    for record in plan
        .adr_records
        .iter()
        .filter(|record| record.delete_reason.is_some())
    {
        if let Some(successor) = &record.successor_old_id {
            replacements.push((record.old_id.clone(), successor.clone()));
        }
    }
    rewrite_text_roots(&text_roots()?, |content| {
        replace_all_pairs(content, &replacements)
    })
}

fn rewrite_retired_terms() -> Result<(), String> {
    rewrite_text_roots(&text_roots()?, |content| {
        let mut next = content.to_owned();
        let literal_replacements = [
            ("Bronze / Silver / Gold / Platinum", "demo_trial / paid"),
            ("Bronze/Silver/Gold/Platinum", "demo_trial/paid"),
            ("bronze / silver / gold / platinum", "demo_trial / paid"),
            ("bronze/silver/gold/platinum", "demo_trial/paid"),
            ("capability_tier", "tenant_class"),
            ("capability-tier", "tenant-class"),
            ("capability tiers", "tenant classes"),
            ("capability tier", "tenant class"),
            ("Capability tiers", "Tenant classes"),
            ("Capability tier", "Tenant class"),
            ("tier_bronze", "tenant_class_demo_trial"),
            ("tier_silver", "tenant_class_paid"),
            ("tier_gold", "tenant_class_paid"),
            ("tier_platinum", "tenant_class_paid"),
        ];
        for (from, to) in literal_replacements {
            next = next.replace(from, to);
        }
        for (from, to) in [
            ("Bronze", "demo_trial"),
            ("Silver", "paid"),
            ("Gold", "paid"),
            ("Platinum", "paid"),
            ("grit", "oya git"),
            ("rtk", "oya git"),
            ("icm", "Oya VCS"),
            ("vox", "Oya VCS"),
        ] {
            next = replace_ascii_word(&next, from, to);
        }
        next
    })
}

fn apply_adr_renumber(plan: &CleanupPlan) -> Result<(), String> {
    let kept = plan
        .adr_records
        .iter()
        .filter(|record| record.delete_reason.is_none())
        .cloned()
        .collect::<Vec<_>>();
    let mut temp_to_final = Vec::new();
    for record in &kept {
        let new_id = record
            .new_id
            .as_deref()
            .ok_or_else(|| format!("ADR {} missing new id", record.old_id))?;
        let old_name = record
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("ADR path has no filename {}", record.path.display()))?;
        let suffix = old_name
            .strip_prefix(&format!("{}-", record.old_id))
            .unwrap_or(old_name);
        let final_path = record.path.with_file_name(format!("{new_id}-{suffix}"));
        let temp_path = record
            .path
            .with_file_name(format!(".renumber-tmp-{new_id}-{suffix}"));
        if record.path != temp_path {
            fs::rename(&record.path, &temp_path).map_err(|error| {
                format!(
                    "ADR temp rename failed {} -> {}: {error}",
                    record.path.display(),
                    temp_path.display()
                )
            })?;
        }
        temp_to_final.push((temp_path, final_path));
    }
    for (temp_path, final_path) in temp_to_final {
        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!("ADR final parent unwritable {}: {error}", parent.display())
            })?;
        }
        fs::rename(&temp_path, &final_path).map_err(|error| {
            format!(
                "ADR final rename failed {} -> {}: {error}",
                temp_path.display(),
                final_path.display()
            )
        })?;
    }

    let mut replacements = Vec::new();
    for record in &kept {
        let new_id = record
            .new_id
            .as_ref()
            .ok_or_else(|| format!("ADR {} missing new id", record.old_id))?;
        if &record.old_id != new_id {
            replacements.push((record.old_id.clone(), new_id.clone()));
        }
    }
    rewrite_text_roots(&text_roots()?, |content| {
        replace_all_pairs(content, &replacements)
    })
}

fn replace_all_pairs(content: &str, replacements: &[(String, String)]) -> String {
    let mut next = content.to_owned();
    for (from, to) in replacements {
        next = next.replace(from, to);
    }
    next
}

/// Roots whose text content the cleanup rewriters walk.
///
/// The fixed entries are non-service surfaces; the service roots are
/// appended from `crate::service_roots` rather than hardcoded. The
/// hardcoded tail used to be `"cloud", "oya", "microservices"`, so once
/// `cloud/` and `microservices/` were removed from the tree the rewriters
/// silently stopped covering every capability root that replaced them.
fn text_roots() -> Result<Vec<PathBuf>, String> {
    let mut roots: Vec<PathBuf> = [
        "AGENTS.md",
        "CLAUDE.md",
        "README.md",
        "docs",
        "specs",
        "crates",
        ".github",
        "tools",
        "scripts",
        "registry",
    ]
    .iter()
    .map(PathBuf::from)
    .collect();
    roots.extend(crate::service_roots::default_service_roots()?);
    Ok(roots)
}

fn rewrite_text_roots<F>(roots: &[PathBuf], mut rewrite: F) -> Result<(), String>
where
    F: FnMut(&str) -> String,
{
    let mut files = Vec::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        if root.is_file() {
            files.push(root.clone());
        } else {
            walk_files(root, &mut |path| {
                files.push(path.to_path_buf());
                Ok(())
            })?;
        }
    }
    files.sort();
    files.dedup();
    for path in files {
        if is_cleanup_tool_path(&path) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let next = rewrite(&content);
        if next != content {
            fs::write(&path, next)
                .map_err(|error| format!("rewrite failed {}: {error}", path.display()))?;
        }
    }
    Ok(())
}

fn find_files_containing_any(roots: &[PathBuf], needles: &[&str]) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        walk_files(root, &mut |path| {
            if is_cleanup_tool_path(path) {
                return Ok(());
            }
            let Ok(content) = fs::read_to_string(path) else {
                return Ok(());
            };
            if needles.iter().any(|needle| content.contains(needle)) {
                paths.push(path.to_path_buf());
            }
            Ok(())
        })?;
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn find_files_containing_token_any(
    roots: &[PathBuf],
    needles: &[&str],
) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        walk_files(root, &mut |path| {
            if is_cleanup_tool_path(path) {
                return Ok(());
            }
            let Ok(content) = fs::read_to_string(path) else {
                return Ok(());
            };
            if needles
                .iter()
                .any(|needle| contains_ascii_word(&content, needle))
            {
                paths.push(path.to_path_buf());
            }
            Ok(())
        })?;
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn walk_files<F>(root: &Path, visit: &mut F) -> Result<(), String>
where
    F: FnMut(&Path) -> Result<(), String>,
{
    walk_paths(root, &mut |path| {
        if path.is_file() {
            visit(path)?;
        }
        Ok(())
    })
}

fn walk_paths<F>(root: &Path, visit: &mut F) -> Result<(), String>
where
    F: FnMut(&Path) -> Result<(), String>,
{
    if should_skip_path(root) {
        return Ok(());
    }
    visit(root)?;
    if !root.is_dir() {
        return Ok(());
    }
    let entries = fs::read_dir(root)
        .map_err(|error| format!("dir unreadable {}: {error}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("dir entry unreadable: {error}"))?;
        let path = entry.path();
        if should_skip_path(&path) {
            continue;
        }
        walk_paths(&path, visit)?;
    }
    Ok(())
}

fn should_skip_path(path: &Path) -> bool {
    path.components().any(|component| {
        let value = component.as_os_str().to_string_lossy();
        matches!(value.as_ref(), ".git" | "target")
    })
}

fn is_cleanup_tool_path(path: &Path) -> bool {
    slash_path(path) == "crates/oya-dev-cli/src/commands/cleanup.rs"
}

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(path).map_err(|error| format!("remove file failed {}: {error}", path.display()))
}

fn remove_path_if_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|error| format!("remove dir failed {}: {error}", path.display()))
    } else {
        remove_file_if_exists(path)
    }
}

fn adr_id_from_path(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    if name.len() < "ADR-0000".len() {
        return None;
    }
    let id = &name[..8];
    if id.starts_with("ADR-") && id[4..].chars().all(|ch| ch.is_ascii_digit()) {
        Some(id.to_owned())
    } else {
        None
    }
}

fn frontmatter_value(content: &str, key: &str) -> Option<String> {
    let mut lines = content.lines();
    if lines.next()? != "---" {
        return None;
    }
    for line in lines {
        if line == "---" {
            return None;
        }
        let trimmed = line.trim();
        let prefix = format!("{key}:");
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            return Some(clean_scalar(rest.trim()));
        }
    }
    None
}

fn frontmatter_successor(content: &str) -> Option<String> {
    let mut lines = content.lines();
    if lines.next()? != "---" {
        return None;
    }
    for line in lines {
        if line == "---" {
            return None;
        }
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("superseded_by:") {
            return first_adr_id(rest);
        }
    }
    None
}

fn body_successor(content: &str) -> Option<String> {
    for line in content.lines().take(80) {
        let lower = line.to_ascii_lowercase();
        if lower.contains("superseded by") {
            return first_adr_id(line);
        }
    }
    None
}

fn first_adr_id(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index + 8 <= bytes.len() {
        if &bytes[index..index + 4] == b"ADR-" {
            let candidate = &value[index..index + 8];
            if candidate[4..].chars().all(|ch| ch.is_ascii_digit()) {
                return Some(candidate.to_owned());
            }
        }
        index += 1;
    }
    None
}

fn clean_scalar(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_owned()
}

fn contains_ascii_word(content: &str, needle: &str) -> bool {
    let mut start = 0usize;
    while let Some(offset) = content[start..].find(needle) {
        let absolute = start + offset;
        let end = absolute + needle.len();
        if is_boundary(content, absolute, end) {
            return true;
        }
        start = end;
    }
    false
}

fn replace_ascii_word(content: &str, needle: &str, replacement: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut cursor = 0usize;
    while let Some(offset) = content[cursor..].find(needle) {
        let absolute = cursor + offset;
        let end = absolute + needle.len();
        if is_boundary(content, absolute, end) {
            out.push_str(&content[cursor..absolute]);
            out.push_str(replacement);
            cursor = end;
        } else {
            out.push_str(&content[cursor..end]);
            cursor = end;
        }
    }
    out.push_str(&content[cursor..]);
    out
}

fn is_boundary(content: &str, start: usize, end: usize) -> bool {
    let before = content[..start].chars().next_back();
    let after = content[end..].chars().next();
    !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
