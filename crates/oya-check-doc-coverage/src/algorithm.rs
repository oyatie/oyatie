//! ADR-0063 §5 algorithm — 10 ordered steps.
//!
//! Each step records violations to the shared `Report`. Steps are independent
//! (no early-exit) so a single run produces a complete gap list.

use crate::manifest::PackManifest;
use crate::types::{Report, Violation, ViolationKind};
use std::path::Path;
use walkdir::WalkDir;

/// Step 2: reconcile workspace metadata against MASTERPLAN §2.1 catalog.
/// A planned µservice with no workspace registration AND no Phase-Spec
/// referencing it is logged but not failed (it's `planned-only`).
pub fn reconcile_registered_vs_planned(
    registered: &[String],
    planned: &[String],
    _report: &mut Report,
) {
    // Currently advisory-only per ADR-0063 §5 step 2 ("planned-only µservices
    // are exempt from §1 enforcement but logged"). Future iteration: emit
    // an info-level row when planned ⊄ registered ∪ phase-referenced.
    let _ = (registered, planned);
}

/// Step 3: for each registered µservice, verify §1 artifacts exist.
pub fn verify_canonical_suite(
    repo_root: &Path,
    registered: &[String],
    report: &mut Report,
) {
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
    }
}

/// Step 5: per-pack overlay artifacts.
pub fn verify_pack_overlays(
    repo_root: &Path,
    packs: &[PackManifest],
    report: &mut Report,
) {
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
    for entry in WalkDir::new(&dir).min_depth(1).max_depth(1).into_iter().filter_map(|r| r.ok()) {
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
    let phase_required = &["acceptance_lanes:", "depends_on:", "entry_gate:", "exit_gate:"];
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
            if entry.file_name() == "impl-plan.md" || (entry.file_name().to_string_lossy().starts_with("IP-") && entry.file_name().to_string_lossy().ends_with(".md")) {
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
        for entry in WalkDir::new(&dir).max_depth(2).into_iter().filter_map(|r| r.ok()) {
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
        let evidence_dir = repo_root.join(format!("docs/localization-packs/{}/evidence", pack.pack.code));
        if !evidence_dir.exists() {
            continue;
        }
        let scope_names: std::collections::HashSet<&String> =
            pack.microservices_in_scope.iter().map(|s| &s.microservice).collect();
        for entry in WalkDir::new(&evidence_dir).max_depth(1).into_iter().filter_map(|r| r.ok()) {
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

fn check_path(
    repo_root: &Path,
    rel: &str,
    kind: ViolationKind,
    desc: &str,
    report: &mut Report,
) {
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
                name.starts_with("ADR-")
                    && name.contains(&needle)
                    && name.ends_with(".md")
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
                name.starts_with("ADR-")
                    && name.contains(&needle)
                    && name.ends_with(".md")
            })
        })
        .unwrap_or(false)
}

fn check_sections_in_dir(repo_root: &Path, rel_dir: &str, required: &[&str], report: &mut Report) {
    let dir = repo_root.join(rel_dir);
    if !dir.exists() {
        return;
    }
    for entry in WalkDir::new(&dir).max_depth(2).into_iter().filter_map(|r| r.ok()) {
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
