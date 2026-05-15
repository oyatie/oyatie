// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! Binary entry point for `oya-foundry-pr-review-dispatcher-app`.
//!
//! Invoked from `.github/workflows/pr-review.yml` after the required-check
//! workflows (`pr-tests`, `oya-foundry-fitness-supply-chain`) converge
//! green. Reads per-facet findings written by the subagent panel (when
//! the subagent runtime lands) and writes:
//!
//! - `evidence/pipeline-maturity-glue/ip-004-reviewer-agent.json`
//! - `evidence/pipeline-maturity-glue/ip-004-pr-review/<pr>/rollup.json`
//! - `registries/cross-cutting/merge-queue-admission-log.json` (append)
//!
//! On any I/O error: exit non-zero — the GitHub Check Run posts FAILURE
//! and branch protection blocks merge.

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use oya_foundry_pr_review_dispatcher_app::fanout::FacetId;
use oya_foundry_pr_review_dispatcher_app::rollup::{
    FacetFinding, FacetRecommendation, Verdict, audit_panel_completeness, rollup_verdict,
};

const EVIDENCE_DIR: &str = "evidence/pipeline-maturity-glue/ip-004-pr-review";
const ROLLUP_PATH: &str = "evidence/pipeline-maturity-glue/ip-004-reviewer-agent.json";
const ADMISSION_LOG: &str = "registries/cross-cutting/merge-queue-admission-log.json";

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(verdict) => {
            println!(
                "pr-review-dispatcher: verdict={} event={}",
                verdict.label(),
                verdict.admission_event()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("pr-review-dispatcher failed: {error}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Options {
    pr_number: String,
    repo_root: PathBuf,
}

impl Options {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut pr_number: Option<String> = None;
        let mut repo_root: Option<PathBuf> = None;
        let args = args.into_iter().collect::<Vec<_>>();
        let mut index = 0usize;
        while index < args.len() {
            match args[index].as_str() {
                "--pr-number" => {
                    index += 1;
                    pr_number = Some(
                        args.get(index)
                            .cloned()
                            .ok_or_else(|| "--pr-number requires a value".to_string())?,
                    );
                }
                "--repo-root" => {
                    index += 1;
                    repo_root = Some(
                        args.get(index)
                            .map(PathBuf::from)
                            .ok_or_else(|| "--repo-root requires a path".to_string())?,
                    );
                }
                "--help" | "-h" => return Err(usage()),
                other => {
                    return Err(format!("unexpected argument '{other}'\n{}", usage()));
                }
            }
            index += 1;
        }
        Ok(Self {
            pr_number: pr_number.ok_or_else(|| "missing --pr-number".to_string())?,
            repo_root: repo_root.unwrap_or_else(|| PathBuf::from(".")),
        })
    }
}

fn usage() -> String {
    "usage: oya-foundry-pr-review-dispatcher-app --pr-number <NUM> [--repo-root <PATH>]".into()
}

fn run<I>(args: I) -> Result<Verdict, String>
where
    I: IntoIterator<Item = String>,
{
    let options = Options::parse(args)?;
    let evidence_dir = options.repo_root.join(EVIDENCE_DIR).join(&options.pr_number);
    let findings = load_findings(&evidence_dir)?;
    let required = FacetId::full_panel_v23().to_vec();
    let completeness = audit_panel_completeness(&required, &findings);
    let verdict = rollup_verdict(&findings);

    let rollup_json = render_rollup_json(
        &options.pr_number,
        &findings,
        &completeness,
        verdict,
        findings.is_empty(),
    );

    let rollup_path = options.repo_root.join(ROLLUP_PATH);
    write_atomically(&rollup_path, rollup_json.as_bytes())?;

    let per_pr_rollup = evidence_dir.join("rollup.json");
    write_atomically(&per_pr_rollup, rollup_json.as_bytes())?;

    let admission_path = options.repo_root.join(ADMISSION_LOG);
    append_admission_event(&admission_path, &options.pr_number, verdict)?;

    Ok(verdict)
}

fn load_findings(dir: &Path) -> Result<Vec<FacetFinding>, String> {
    let mut findings = Vec::new();
    if !dir.exists() {
        return Ok(findings);
    }
    let entries = fs::read_dir(dir)
        .map_err(|error| format!("could not read {}: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("could not read entry: {error}"))?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_none_or(|ext| ext != "json")
        {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        if stem == "rollup" {
            continue;
        }
        let Some(facet) = parse_facet_slug(&stem) else {
            continue;
        };
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        let recommendation = parse_recommendation(&contents)
            .ok_or_else(|| format!("{}: missing/invalid final_recommendation", path.display()))?;
        let reviewer_id = parse_reviewer_id(&contents).unwrap_or_else(|| stem.clone());
        findings.push(FacetFinding {
            facet,
            reviewer_id,
            recommendation,
        });
    }
    findings.sort_by_key(|f| f.facet);
    Ok(findings)
}

fn parse_facet_slug(slug: &str) -> Option<FacetId> {
    for facet in FacetId::full_panel_v23() {
        if facet.slug() == slug {
            return Some(facet);
        }
    }
    None
}

/// Minimal JSON-string-value reader. The subagent r1.json shape per
/// `specs/cross-cutting/multispectrum-review.json` includes:
///   { "reviewer_id": "...", "final_recommendation": "APPROVE|CHANGES_REQUESTED|REJECT", ... }
/// We intentionally avoid a serde dependency at this scaffold stage —
/// the input contract has only two string fields the dispatcher needs.
fn parse_recommendation(json: &str) -> Option<FacetRecommendation> {
    let value = json_string_field(json, "final_recommendation")?;
    match value.as_str() {
        "APPROVE" | "approve" => Some(FacetRecommendation::Approve),
        "CHANGES_REQUESTED" | "changes_requested" => Some(FacetRecommendation::ChangesRequested),
        "REJECT" | "reject" => Some(FacetRecommendation::Reject),
        _ => None,
    }
}

fn parse_reviewer_id(json: &str) -> Option<String> {
    json_string_field(json, "reviewer_id")
}

fn json_string_field(json: &str, field: &str) -> Option<String> {
    let needle = format!("\"{field}\"");
    let mut cursor = json.find(&needle)? + needle.len();
    let bytes = json.as_bytes();
    // Skip whitespace + ':'.
    while cursor < bytes.len() && (bytes[cursor].is_ascii_whitespace() || bytes[cursor] == b':') {
        cursor += 1;
    }
    if cursor >= bytes.len() || bytes[cursor] != b'"' {
        return None;
    }
    cursor += 1;
    let start = cursor;
    while cursor < bytes.len() && bytes[cursor] != b'"' {
        if bytes[cursor] == b'\\' && cursor + 1 < bytes.len() {
            cursor += 2;
        } else {
            cursor += 1;
        }
    }
    if cursor > bytes.len() {
        return None;
    }
    let value = &json[start..cursor];
    Some(value.to_string())
}

fn render_rollup_json(
    pr_number: &str,
    findings: &[FacetFinding],
    completeness: &oya_foundry_pr_review_dispatcher_app::rollup::PanelCompletenessReport,
    verdict: Verdict,
    subagent_runtime_pending: bool,
) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut buf = String::new();
    buf.push_str("{\n");
    buf.push_str("  \"schema\": \"oya-pr-review-rollup/v1\",\n");
    buf.push_str(&format!("  \"pr_number\": \"{}\",\n", json_escape(pr_number)));
    buf.push_str(&format!("  \"emitted_at_unix\": {now},\n"));
    buf.push_str(&format!("  \"verdict\": \"{}\",\n", verdict.label()));
    buf.push_str(&format!(
        "  \"admission_event\": \"{}\",\n",
        verdict.admission_event()
    ));
    buf.push_str(&format!(
        "  \"subagent_runtime_pending\": {subagent_runtime_pending},\n"
    ));
    buf.push_str(&format!(
        "  \"panel_complete\": {},\n",
        completeness.is_complete() && !subagent_runtime_pending
    ));
    buf.push_str("  \"required_facets\": [");
    for (index, facet) in completeness.required.iter().enumerate() {
        if index > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&format!("\"{}\"", facet.slug()));
    }
    buf.push_str("],\n");
    buf.push_str("  \"present_facets\": [");
    for (index, facet) in completeness.present.iter().enumerate() {
        if index > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&format!("\"{}\"", facet.slug()));
    }
    buf.push_str("],\n");
    buf.push_str("  \"missing_facets\": [");
    for (index, facet) in completeness.missing.iter().enumerate() {
        if index > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&format!("\"{}\"", facet.slug()));
    }
    buf.push_str("],\n");
    buf.push_str("  \"duplicate_reviewer_ids\": [");
    for (index, reviewer) in completeness.duplicate_reviewer_ids.iter().enumerate() {
        if index > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&format!("\"{}\"", json_escape(reviewer)));
    }
    buf.push_str("],\n");
    buf.push_str("  \"findings\": [\n");
    for (index, finding) in findings.iter().enumerate() {
        if index > 0 {
            buf.push_str(",\n");
        }
        buf.push_str(&format!(
            "    {{\"facet\": \"{}\", \"reviewer_id\": \"{}\", \"recommendation\": \"{}\"}}",
            finding.facet.slug(),
            json_escape(&finding.reviewer_id),
            match finding.recommendation {
                FacetRecommendation::Approve => "APPROVE",
                FacetRecommendation::ChangesRequested => "CHANGES_REQUESTED",
                FacetRecommendation::Reject => "REJECT",
            }
        ));
    }
    buf.push_str("\n  ],\n");
    buf.push_str("  \"audit_trail\": {\n");
    buf.push_str("    \"plan_ref\": \".omc/plans/milestones/M-CC-cross-cutting/phases/P10-pipeline-maturity-glue/IP-004-reviewer-agent-auto-dispatch.md\",\n");
    buf.push_str("    \"audit_ref\": \"evidence/audits/pipeline-maturity-audit-2026-05-15.md\",\n");
    buf.push_str("    \"upstream_kernel\": \"oya-foundry-vcs-review-mergequeue-kernel\",\n");
    buf.push_str("    \"subagent_runtime_followup\": \"TODO: wire actual per-facet subagent runtime; until then, panel-complete is gated false and APPROVE carries subagent_runtime_pending=true so IP-005/IP-006 downstreams refuse to trust it\"\n");
    buf.push_str("  }\n");
    buf.push_str("}\n");
    buf
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out
}

fn append_admission_event(path: &Path, pr_number: &str, verdict: Verdict) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("mkdir {}: {error}", parent.display()))?;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut log: Vec<String> = if path.exists() {
        let raw = fs::read_to_string(path)
            .map_err(|error| format!("read {}: {error}", path.display()))?;
        parse_admission_log(&raw)
    } else {
        Vec::new()
    };
    let new_entry = format!(
        "{{\"pr_number\": \"{}\", \"event\": \"{}\", \"verdict\": \"{}\", \"emitted_at_unix\": {}}}",
        json_escape(pr_number),
        verdict.admission_event(),
        verdict.label(),
        now
    );
    log.push(new_entry);

    let mut body = String::new();
    body.push_str("{\n  \"schema\": \"oya-merge-queue-admission-log/v1\",\n  \"events\": [\n");
    for (index, entry) in log.iter().enumerate() {
        if index > 0 {
            body.push_str(",\n");
        }
        body.push_str("    ");
        body.push_str(entry);
    }
    body.push_str("\n  ]\n}\n");
    write_atomically(path, body.as_bytes())
}

/// Pull the inner event JSON objects out of an existing admission-log
/// file. Intentionally tolerant — if parsing fails we treat the log as
/// empty rather than crashing CI; the dispatcher's job is to make
/// forward progress, and the parser keeps the structure-rebuild
/// deterministic.
fn parse_admission_log(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Some(events_start) = raw.find("\"events\"") else {
        return out;
    };
    let after = &raw[events_start..];
    let Some(bracket_open) = after.find('[') else {
        return out;
    };
    let body = &after[bracket_open + 1..];
    let mut depth = 0i32;
    let mut object_start: Option<usize> = None;
    for (index, ch) in body.char_indices() {
        match ch {
            '{' => {
                if depth == 0 {
                    object_start = Some(index);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = object_start.take() {
                        out.push(body[start..=index].to_string());
                    }
                }
            }
            ']' if depth == 0 => break,
            _ => {}
        }
    }
    out
}

fn write_atomically(path: &Path, contents: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("mkdir {}: {error}", parent.display()))?;
    }
    let tmp = path.with_extension("tmp");
    {
        let mut file = fs::File::create(&tmp)
            .map_err(|error| format!("create {}: {error}", tmp.display()))?;
        file.write_all(contents)
            .map_err(|error| format!("write {}: {error}", tmp.display()))?;
        file.sync_all()
            .map_err(|error| format!("fsync {}: {error}", tmp.display()))?;
    }
    fs::rename(&tmp, path).map_err(|error| {
        format!(
            "rename {} -> {}: {error}",
            tmp.display(),
            path.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn end_to_end_empty_panel_emits_approve_with_pending_flag() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        let pr = "pr-1";
        let verdict = run(
            [
                "--pr-number".to_string(),
                pr.to_string(),
                "--repo-root".to_string(),
                repo.display().to_string(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(verdict, Verdict::Approve);
        let rollup = fs::read_to_string(repo.join(ROLLUP_PATH)).unwrap();
        assert!(rollup.contains("\"subagent_runtime_pending\": true"));
        assert!(rollup.contains("\"verdict\": \"APPROVE\""));
        assert!(rollup.contains("\"panel_complete\": false"));
        let admission = fs::read_to_string(repo.join(ADMISSION_LOG)).unwrap();
        assert!(admission.contains("\"event\": \"pr-review-approved\""));
        assert!(admission.contains("\"pr_number\": \"pr-1\""));
    }

    #[test]
    fn end_to_end_changes_requested_emits_fix_requested_event() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        let pr = "pr-7";
        let evidence_dir = repo.join(EVIDENCE_DIR).join(pr);
        write(
            &evidence_dir.join("F1_linus.json"),
            r#"{"reviewer_id": "claude-critic-F1_linus-pr-7", "final_recommendation": "APPROVE"}"#,
        );
        write(
            &evidence_dir.join("F7_security.json"),
            r#"{"reviewer_id": "claude-security-F7_security-pr-7", "final_recommendation": "CHANGES_REQUESTED"}"#,
        );

        let verdict = run(
            [
                "--pr-number".to_string(),
                pr.to_string(),
                "--repo-root".to_string(),
                repo.display().to_string(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(verdict, Verdict::ChangesRequested);
        let admission = fs::read_to_string(repo.join(ADMISSION_LOG)).unwrap();
        assert!(admission.contains("\"event\": \"pr-review-fix-requested\""));
        assert!(admission.contains("\"verdict\": \"CHANGES_REQUESTED\""));
    }

    #[test]
    fn end_to_end_reject_dominates() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        let pr = "pr-9";
        let evidence_dir = repo.join(EVIDENCE_DIR).join(pr);
        write(
            &evidence_dir.join("F7_security.json"),
            r#"{"reviewer_id": "claude-security-F7_security-pr-9", "final_recommendation": "REJECT"}"#,
        );
        write(
            &evidence_dir.join("F1_linus.json"),
            r#"{"reviewer_id": "claude-critic-F1_linus-pr-9", "final_recommendation": "APPROVE"}"#,
        );
        let verdict = run(
            [
                "--pr-number".to_string(),
                pr.to_string(),
                "--repo-root".to_string(),
                repo.display().to_string(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(verdict, Verdict::Reject);
    }

    #[test]
    fn admission_log_appends_across_two_runs() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        for pr in ["pr-1", "pr-2"] {
            let _ = run(
                [
                    "--pr-number".to_string(),
                    pr.to_string(),
                    "--repo-root".to_string(),
                    repo.display().to_string(),
                ]
                .into_iter(),
            )
            .unwrap();
        }
        let admission = fs::read_to_string(repo.join(ADMISSION_LOG)).unwrap();
        assert!(admission.contains("\"pr_number\": \"pr-1\""));
        assert!(admission.contains("\"pr_number\": \"pr-2\""));
    }

    #[test]
    fn json_string_field_extracts_simple_value() {
        let s = r#"{"reviewer_id": "claude-critic-F1_linus-pr-7", "final_recommendation": "APPROVE"}"#;
        assert_eq!(
            json_string_field(s, "reviewer_id"),
            Some("claude-critic-F1_linus-pr-7".to_string())
        );
        assert_eq!(
            json_string_field(s, "final_recommendation"),
            Some("APPROVE".to_string())
        );
        assert_eq!(json_string_field(s, "nonexistent"), None);
    }

    #[test]
    fn parse_facet_slug_round_trips_for_all_panel_facets() {
        for facet in FacetId::full_panel_v23() {
            assert_eq!(parse_facet_slug(facet.slug()), Some(facet));
        }
        assert_eq!(parse_facet_slug("not_a_facet"), None);
    }

    #[test]
    fn options_require_pr_number() {
        let err = Options::parse(["--repo-root".to_string(), ".".to_string()]).unwrap_err();
        assert!(err.contains("--pr-number"));
    }
}
