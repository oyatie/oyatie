//! ADR-0063 §5 algorithm — 10 ordered steps.
//!
//! Each step records violations to the shared `Report`. Steps are independent
//! (no early-exit) so a single run produces a complete gap list.

use crate::manifest::PackManifest;
use crate::types::{Report, Violation, ViolationKind};
use std::path::Path;
use walkdir::WalkDir;

/// Step 2: reconcile workspace metadata against MASTERPLAN §2.1 catalog.
/// Per ADR-0063 §5 step 2: every planned µservice MUST appear in workspace
/// metadata once it has a Phase-Spec referencing it. Planned-only µservices
/// with no Phase-Spec are exempt from §1 enforcement but logged.
pub fn reconcile_registered_vs_planned(
    repo_root: &Path,
    registered: &[String],
    planned: &[String],
    report: &mut Report,
) {
    let registered_set: std::collections::HashSet<&String> = registered.iter().collect();
    // Index every phase-spec.md once; record (microservice → phase-spec path) refs
    let phases_dir = repo_root.join(".omc/plans/milestones");
    let mut phase_refs: std::collections::HashMap<String, Vec<String>> = Default::default();
    if phases_dir.exists() {
        for entry in WalkDir::new(&phases_dir).into_iter().filter_map(|r| r.ok()) {
            if entry.file_name() == "phase-spec.md" {
                let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                for ms in planned {
                    // Match µservice token as a kebab-bounded reference: `<ms>` or `<ms>-` or whitespace-bounded
                    let needle1 = format!("`{}`", ms);
                    let needle2 = format!("`{}-", ms);
                    let needle3 = format!("oya-{}-", ms);
                    let needle4 = format!(" {} ", ms);
                    if content.contains(&needle1)
                        || content.contains(&needle2)
                        || content.contains(&needle3)
                        || content.contains(&needle4)
                    {
                        phase_refs
                            .entry(ms.clone())
                            .or_default()
                            .push(entry.path().display().to_string());
                    }
                }
            }
        }
    }
    for ms in planned {
        if registered_set.contains(ms) {
            continue;
        }
        if phase_refs.contains_key(ms) {
            // Planned µservice referenced by ≥1 Phase-Spec but not in workspace metadata → violation
            report.push(Violation {
                kind: ViolationKind::UnreconciledPlanned,
                path: format!("Cargo.toml [workspace.metadata.oya.microservices.{}]", ms),
                description: format!(
                    "µservice `{}` is referenced by Phase-Spec(s) [{}] but is not registered in [workspace.metadata.oya.microservices]",
                    ms,
                    phase_refs[ms].join(", ")
                ),
            });
        }
        // else: planned-only, no introducing phase yet — advisory only, not a violation
    }
}

/// Step 3: for each registered µservice, verify §1 artifacts exist.
pub fn verify_canonical_suite(repo_root: &Path, registered: &[String], report: &mut Report) {
    let phases_dir = repo_root.join(".omc/plans/milestones");
    for ms in registered {
        // Microservice record
        check_path(
            repo_root,
            &format!("docs/microservices/{}.md", ms),
            ViolationKind::MissingCanonicalArtifact,
            &format!("missing microservice record for `{}`", ms),
            report,
        );
        // PRD
        check_path(
            repo_root,
            &format!("docs/prds/{}.md", ms),
            ViolationKind::MissingCanonicalArtifact,
            &format!("missing canonical PRD for `{}`", ms),
            report,
        );
        // Naming ADR — match any ADR-NNNN-microservice-<ms>.md
        if !has_naming_adr(repo_root, ms) {
            report.push(Violation {
                kind: ViolationKind::MissingCanonicalArtifact,
                path: format!("docs/decisions/ADR-NNNN-microservice-{}.md", ms),
                description: format!("missing naming-scope ADR for `{}`", ms),
            });
        }
        // BC registrations — at least one BC registration must exist when the µservice has BCs
        // (currently we cannot enumerate BCs from metadata, so we check at least one entry exists
        // whose filename starts with `<ms>-` under docs/bounded-contexts/, OR the µservice has no
        // declared BCs in any phase-spec).
        if !has_bounded_context_registration(repo_root, ms) {
            report.push(Violation {
                kind: ViolationKind::MissingCanonicalArtifact,
                path: format!("docs/bounded-contexts/{}-*.md", ms),
                description: format!(
                    "missing bounded-context registration(s) for `{}` (at least one BC registration required per ADR-0063 §1)",
                    ms
                ),
            });
        }
        // Phase-Spec reference — at least one phase-spec must reference this µservice
        if !has_phase_spec_reference(&phases_dir, ms) {
            report.push(Violation {
                kind: ViolationKind::MissingCanonicalArtifact,
                path: format!(".omc/plans/milestones/M*/phases/*/phase-spec.md"),
                description: format!(
                    "no phase-spec references µservice `{}` — every registered µservice MUST have an introducing phase per ADR-0063 §1",
                    ms
                ),
            });
        }
        // Impl-Plan reference — at least one impl-plan must reference this µservice
        if !has_impl_plan_reference(&phases_dir, ms) {
            report.push(Violation {
                kind: ViolationKind::MissingCanonicalArtifact,
                path: format!(".omc/plans/milestones/M*/phases/*/impl-plan.md"),
                description: format!(
                    "no impl-plan references µservice `{}` per ADR-0063 §1",
                    ms
                ),
            });
        }
    }
}

fn has_bounded_context_registration(repo_root: &Path, microservice: &str) -> bool {
    let dir = repo_root.join("docs/bounded-contexts");
    if !dir.exists() {
        return false;
    }
    let prefix = format!("{}-", microservice);
    std::fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok()).any(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with(&prefix) && name.ends_with(".md")
            })
        })
        .unwrap_or(false)
}

fn has_phase_spec_reference(phases_dir: &Path, microservice: &str) -> bool {
    if !phases_dir.exists() {
        return false;
    }
    let needle = format!("oya-{}-", microservice);
    let needle_bareword = format!("`{}`", microservice);
    for entry in WalkDir::new(phases_dir).into_iter().filter_map(|r| r.ok()) {
        if entry.file_name() == "phase-spec.md" {
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if content.contains(&needle) || content.contains(&needle_bareword) {
                return true;
            }
        }
    }
    false
}

fn has_impl_plan_reference(phases_dir: &Path, microservice: &str) -> bool {
    if !phases_dir.exists() {
        return false;
    }
    let needle = format!("oya-{}-", microservice);
    let needle_bareword = format!("`{}`", microservice);
    for entry in WalkDir::new(phases_dir).into_iter().filter_map(|r| r.ok()) {
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname == "impl-plan.md"
            || (fname.starts_with("IP-") && fname.ends_with(".md"))
        {
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if content.contains(&needle) || content.contains(&needle_bareword) {
                return true;
            }
        }
    }
    false
}

/// Step 5: per-pack overlay artifacts.
pub fn verify_pack_overlays(repo_root: &Path, packs: &[PackManifest], report: &mut Report) {
    for pack in packs {
        if pack.pack.status == "retired" {
            continue;
        }
        for scope in &pack.microservices_in_scope {
            // Regulatory ADR (always required)
            if !has_pack_regulatory_adr(repo_root, &pack.pack.code, &scope.microservice) {
                report.push(Violation {
                    kind: ViolationKind::MissingPackOverlay,
                    path: format!(
                        "docs/decisions/ADR-NNNN-{}-{}-regulatory.md",
                        pack.pack.code, scope.microservice
                    ),
                    description: format!(
                        "missing pack regulatory ADR for ({}, {})",
                        pack.pack.code, scope.microservice
                    ),
                });
            }
            // Acceptance evidence (always required)
            check_path(
                repo_root,
                &format!(
                    "docs/localization-packs/{}/evidence/{}.md",
                    pack.pack.code, scope.microservice
                ),
                ViolationKind::MissingPackOverlay,
                &format!(
                    "missing pack acceptance evidence for ({}, {})",
                    pack.pack.code, scope.microservice
                ),
                report,
            );
            // Overlay PRD (only when material_scope=true)
            if scope.material_scope {
                check_path(
                    repo_root,
                    &format!("docs/prds/{}-{}.md", scope.microservice, pack.pack.code),
                    ViolationKind::MissingPackOverlay,
                    &format!(
                        "missing pack overlay PRD for ({}, {}) — material_scope=true",
                        pack.pack.code, scope.microservice
                    ),
                    report,
                );
            }
        }
    }
}

/// Step 6: every milestone directory has README + acceptance-evidence dir.
pub fn verify_milestone_artifacts(repo_root: &Path, report: &mut Report) {
    let dir = repo_root.join(".omc/plans/milestones");
    if !dir.exists() {
        return;
    }
    for entry in WalkDir::new(&dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|r| r.ok())
    {
        if !entry.file_type().is_dir() {
            continue;
        }
        let milestone = entry.file_name().to_string_lossy().to_string();
        if !entry.path().join("README.md").exists() {
            report.push(Violation {
                kind: ViolationKind::MissingMilestoneArtifact,
                path: format!(".omc/plans/milestones/{}/README.md", milestone),
                description: format!("missing milestone README for `{}`", milestone),
            });
        }
        if !entry.path().join("acceptance-evidence").exists() {
            report.push(Violation {
                kind: ViolationKind::MissingMilestoneArtifact,
                path: format!(".omc/plans/milestones/{}/acceptance-evidence/", milestone),
                description: format!("missing acceptance-evidence dir for `{}`", milestone),
            });
        }
    }
}

/// Step 7: PRD / Phase-Spec / Impl-Plan section-completeness checks.
pub fn verify_section_completeness(repo_root: &Path, report: &mut Report) {
    // PRDs need: Competitive Benchmark / Performance Targets / Horizontal Scalability / Bounded Contexts
    let prd_required = &[
        "## Competitive Benchmark",
        "## Performance Targets",
        "## Horizontal Scalability",
        "## Bounded Contexts",
    ];
    check_sections_in_dir(repo_root, "docs/prds", prd_required, report);

    // Phase-Specs need: acceptance_lanes / depends_on / entry_gate / exit_gate in frontmatter
    let phase_required = &[
        "acceptance_lanes:",
        "depends_on:",
        "entry_gate:",
        "exit_gate:",
    ];
    let phases = repo_root.join(".omc/plans/milestones");
    if phases.exists() {
        for entry in WalkDir::new(&phases).into_iter().filter_map(|r| r.ok()) {
            if entry.file_name() == "phase-spec.md" {
                let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                for marker in phase_required {
                    if !content.contains(marker) {
                        report.push(Violation {
                            kind: ViolationKind::MissingSection,
                            path: entry.path().display().to_string(),
                            description: format!("phase-spec missing `{}` in frontmatter", marker),
                        });
                    }
                }
            }
        }
    }

    // Impl-Plans need: Concrete File Targets / Code Shape / Acceptance Gates / Load test / Grit Claim Symbols / ICM Rows
    let impl_required = &[
        "## Concrete File Targets",
        "## Code Shape",
        "## Acceptance Gates",
        "## Load test",
        "## Grit Claim Symbols",
        "## ICM Rows",
    ];
    if phases.exists() {
        for entry in WalkDir::new(&phases).into_iter().filter_map(|r| r.ok()) {
            if entry.file_name() == "impl-plan.md"
                || (entry.file_name().to_string_lossy().starts_with("IP-")
                    && entry.file_name().to_string_lossy().ends_with(".md"))
            {
                let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                for marker in impl_required {
                    if !content.contains(marker) {
                        report.push(Violation {
                            kind: ViolationKind::MissingSection,
                            path: entry.path().display().to_string(),
                            description: format!("impl-plan missing `{}`", marker),
                        });
                    }
                }
            }
        }
    }
}

/// Step 8: orphan-scan — doc files referencing retired µservices.
///
/// A doc is an orphan only when its referenced µservice token is NOT in the
/// workspace registry AND NOT in the MASTERPLAN §2.1 planned catalog. Planned-only
/// µservices with docs are acceptable (they predate their introducing phase).
///
/// Whitelisted filenames (templates / index / governance docs) are skipped.
pub fn orphan_scan(
    repo_root: &Path,
    registered: &[String],
    packs: &[PackManifest],
    report: &mut Report,
) {
    const FILENAME_WHITELIST: &[&str] = &[
        "INDEX.md",
        "README.md",
        "MASTERPLAN.md",
        "DOC-COVERAGE.md",
        "RETIRED.md",
        "CHANGELOG.md",
    ];
    const SUFFIX_WHITELIST: &[&str] = &["-template.md"];
    let planned = crate::registry::read_masterplan_catalog(repo_root).unwrap_or_default();
    let registered_set: std::collections::HashSet<&String> = registered.iter().collect();
    let planned_set: std::collections::HashSet<&String> = planned.iter().collect();
    let scan_dirs = ["docs/microservices", "docs/prds", "docs/bounded-contexts"];
    for dir_rel in &scan_dirs {
        let dir = repo_root.join(dir_rel);
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(&dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|r| r.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let fname = entry.file_name().to_string_lossy().to_string();
            if !fname.ends_with(".md") {
                continue;
            }
            if FILENAME_WHITELIST.contains(&fname.as_str()) {
                continue;
            }
            if SUFFIX_WHITELIST.iter().any(|s| fname.ends_with(s)) {
                continue;
            }
            // Filename stem typically encodes µservice; e.g. "hr.md" or "hr-kr.md" or "hr-payroll.md"
            let stem = fname.trim_end_matches(".md");
            let ms_token = stem.split('-').next().unwrap_or(stem).to_string();
            if ms_token.is_empty() {
                continue;
            }
            if registered_set.contains(&ms_token) || planned_set.contains(&ms_token) {
                continue;
            }
            report.push(Violation {
                kind: ViolationKind::OrphanDoc,
                path: entry.path().display().to_string(),
                description: format!(
                    "doc references µservice token `{}` not in workspace metadata AND not in MASTERPLAN §2.1 catalog",
                    ms_token
                ),
            });
        }
    }
    // Pack evidence orphan scan
    for pack in packs {
        let evidence_dir = repo_root.join(format!(
            "docs/localization-packs/{}/evidence",
            pack.pack.code
        ));
        if !evidence_dir.exists() {
            continue;
        }
        let scope_names: std::collections::HashSet<&String> = pack
            .microservices_in_scope
            .iter()
            .map(|s| &s.microservice)
            .collect();
        for entry in WalkDir::new(&evidence_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|r| r.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let fname = entry.file_name().to_string_lossy().to_string();
            if !fname.ends_with(".md") {
                continue;
            }
            let stem = fname.trim_end_matches(".md").to_string();
            if !scope_names.contains(&stem) {
                report.push(Violation {
                    kind: ViolationKind::OrphanDoc,
                    path: entry.path().display().to_string(),
                    description: format!(
                        "evidence file for `{}` but pack `{}` does not include it in microservices_in_scope",
                        stem, pack.pack.code
                    ),
                });
            }
        }
    }
}

// --- helpers ---

fn check_path(repo_root: &Path, rel: &str, kind: ViolationKind, desc: &str, report: &mut Report) {
    if !repo_root.join(rel).exists() {
        report.push(Violation {
            kind,
            path: rel.to_string(),
            description: desc.to_string(),
        });
    }
}

fn has_naming_adr(repo_root: &Path, microservice: &str) -> bool {
    let dir = repo_root.join("docs/decisions");
    if !dir.exists() {
        return false;
    }
    let needle = format!("microservice-{}", microservice);
    std::fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok()).any(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("ADR-") && name.contains(&needle) && name.ends_with(".md")
            })
        })
        .unwrap_or(false)
}

fn has_pack_regulatory_adr(repo_root: &Path, pack: &str, microservice: &str) -> bool {
    let dir = repo_root.join("docs/decisions");
    if !dir.exists() {
        return false;
    }
    let needle = format!("{}-{}-regulatory", pack, microservice);
    std::fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok()).any(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("ADR-") && name.contains(&needle) && name.ends_with(".md")
            })
        })
        .unwrap_or(false)
}

fn check_sections_in_dir(repo_root: &Path, rel_dir: &str, required: &[&str], report: &mut Report) {
    let dir = repo_root.join(rel_dir);
    if !dir.exists() {
        return;
    }
    for entry in WalkDir::new(&dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|r| r.ok())
    {
        let fname = entry.file_name().to_string_lossy().to_string();
        if !fname.ends_with(".md") || fname == "INDEX.md" {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        for marker in required {
            if !content.contains(marker) {
                report.push(Violation {
                    kind: ViolationKind::MissingSection,
                    path: entry.path().display().to_string(),
                    description: format!("doc missing `{}` section", marker),
                });
            }
        }
    }
}
