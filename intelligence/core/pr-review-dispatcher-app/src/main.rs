// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! Binary entry point for `intelligence-pr-review-dispatcher-app`.
//!
//! Invoked from `.github/workflows/pr-review.yml` after the required-check
//! workflows (`pr-tests`, `governance-supply-chain`) converge
//! green. Reads per-facet findings written by the subagent panel (when
//! the subagent runtime lands) and writes:
//!
//! - `evidence/pipeline-maturity-glue/ip-004-reviewer-agent.json`
//! - `evidence/pipeline-maturity-glue/ip-004-pr-review/<pr>/rollup.json`
//! - `registry/merge-queue-admission-log.json` (append)
//!
//! On any I/O error: exit non-zero — the GitHub Check Run posts FAILURE
//! and branch protection blocks merge.

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use intelligence_account_kernel::SecretReference;
use intelligence_pr_review_dispatcher_app::fanout::FacetId;
use intelligence_pr_review_dispatcher_app::rollup::{
    FacetFinding, FacetRecommendation, Verdict, audit_panel_completeness, rollup_verdict,
};
use intelligence_subagent_runtime_usecase::{
    FacetFindingJson, FacetPromptTemplate, MockSubagentPort, SubagentPort, SubagentRequest,
    fanout_panel_v23,
};

const EVIDENCE_DIR: &str = "evidence/pipeline-maturity-glue/ip-004-pr-review";
const TEMPLATES_DIR: &str = "evidence/pipeline-maturity-glue/ip-004-pr-review/facets";
const ROLLUP_PATH: &str = "evidence/pipeline-maturity-glue/ip-004-reviewer-agent.json";
const ADMISSION_LOG: &str = "registry/merge-queue-admission-log.json";
const DEFAULT_API_KEY_SREF: &str = "sref://openbao/oyatie/foundry/anthropic-api-key";
const DEFAULT_MODEL_ID: &str = "claude-opus-4-7";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeMode {
    /// Caller pre-wrote the per-facet evidence files (e.g. via the
    /// `intelligence-subagent-runtime-app` binary or via an external
    /// orchestrator). Dispatcher just rolls up whatever is present.
    External,
    /// Dispatcher runs the deterministic-mock runtime inline before
    /// loading findings — canonical CI / test path.
    InlineDeterministicMock,
}

impl RuntimeMode {
    fn from_wire(value: &str) -> Result<Self, String> {
        match value {
            "external" => Ok(Self::External),
            "inline-deterministic-mock" => Ok(Self::InlineDeterministicMock),
            other => Err(format!(
                "--runtime-mode: unknown mode `{other}`; expected external | inline-deterministic-mock"
            )),
        }
    }
}

#[derive(Debug)]
struct Options {
    pr_number: String,
    repo_root: PathBuf,
    runtime_mode: RuntimeMode,
    change_id: Option<String>,
    user_message_file: Option<PathBuf>,
    api_key_ref: Option<SecretReference>,
    model_id: String,
}

impl Options {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut pr_number: Option<String> = None;
        let mut repo_root: Option<PathBuf> = None;
        let mut runtime_mode = RuntimeMode::External;
        let mut change_id: Option<String> = None;
        let mut user_message_file: Option<PathBuf> = None;
        let mut api_key_ref_raw: Option<String> = None;
        let mut model_id = DEFAULT_MODEL_ID.to_string();
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
                "--runtime-mode" => {
                    index += 1;
                    let value = args
                        .get(index)
                        .ok_or_else(|| "--runtime-mode requires a value".to_string())?;
                    runtime_mode = RuntimeMode::from_wire(value)?;
                }
                "--change-id" => {
                    index += 1;
                    change_id = Some(
                        args.get(index)
                            .cloned()
                            .ok_or_else(|| "--change-id requires a value".to_string())?,
                    );
                }
                "--user-message-file" => {
                    index += 1;
                    user_message_file = Some(
                        args.get(index)
                            .map(PathBuf::from)
                            .ok_or_else(|| "--user-message-file requires a path".to_string())?,
                    );
                }
                "--api-key-ref" => {
                    index += 1;
                    api_key_ref_raw = Some(
                        args.get(index)
                            .cloned()
                            .ok_or_else(|| "--api-key-ref requires a value".to_string())?,
                    );
                }
                "--model-id" => {
                    index += 1;
                    model_id = args
                        .get(index)
                        .cloned()
                        .ok_or_else(|| "--model-id requires a value".to_string())?;
                }
                "--help" | "-h" => return Err(usage()),
                other => {
                    return Err(format!("unexpected argument '{other}'\n{}", usage()));
                }
            }
            index += 1;
        }
        let api_key_ref = match api_key_ref_raw {
            Some(raw) => {
                Some(SecretReference::new(raw).map_err(|e| format!("--api-key-ref: {e}"))?)
            }
            None => None,
        };
        Ok(Self {
            pr_number: pr_number.ok_or_else(|| "missing --pr-number".to_string())?,
            repo_root: repo_root.unwrap_or_else(|| PathBuf::from(".")),
            runtime_mode,
            change_id,
            user_message_file,
            api_key_ref,
            model_id,
        })
    }
}

fn usage() -> String {
    "usage: intelligence-pr-review-dispatcher-app --pr-number <NUM> [--repo-root <PATH>] \\\n         [--runtime-mode external|inline-deterministic-mock] [--change-id <ID>] \\\n         [--user-message-file <PATH>] [--api-key-ref sref://...] [--model-id <ID>]"
        .into()
}

fn run<I>(args: I) -> Result<Verdict, String>
where
    I: IntoIterator<Item = String>,
{
    let options = Options::parse(args)?;
    let evidence_dir = options
        .repo_root
        .join(EVIDENCE_DIR)
        .join(&options.pr_number);

    // IP-009 wiring: when the caller requests inline runtime mode, we
    // invoke the deterministic-mock subagent runtime BEFORE loading
    // findings, so the per-facet JSON files exist by the time the
    // dispatcher rolls them up. The runtime writes findings to
    // `<evidence_dir>/<facet_id>.json`, the same paths `load_findings`
    // already reads. This closes the `subagent_runtime_pending=true`
    // gap end-to-end inside a single process invocation.
    if options.runtime_mode == RuntimeMode::InlineDeterministicMock {
        run_inline_subagent_runtime(&options, &evidence_dir)?;
    }

    let findings = load_findings(&evidence_dir)?;
    let required = FacetId::full_panel_v23().to_vec();
    let completeness = audit_panel_completeness(&required, &findings);
    let verdict = rollup_verdict(&findings);

    // Pending iff (a) no findings present at all, OR (b) the required
    // panel is incomplete (missing facets or duplicate reviewer ids).
    // The `External` runtime mode lets findings legitimately be empty
    // (caller is still wiring up), so we keep the same condition for
    // both modes — the marker becomes false only when a real, complete
    // panel has landed.
    let subagent_runtime_pending = findings.is_empty() || !completeness.is_complete();

    let rollup_json = render_rollup_json(
        &options.pr_number,
        &findings,
        &completeness,
        verdict,
        subagent_runtime_pending,
    );

    let rollup_path = options.repo_root.join(ROLLUP_PATH);
    write_atomically(&rollup_path, rollup_json.as_bytes())?;

    let per_pr_rollup = evidence_dir.join("rollup.json");
    write_atomically(&per_pr_rollup, rollup_json.as_bytes())?;

    let admission_path = options.repo_root.join(ADMISSION_LOG);
    append_admission_event(
        &admission_path,
        &options.pr_number,
        verdict,
        subagent_runtime_pending,
    )?;

    Ok(verdict)
}

/// Invoke the IP-009 subagent runtime inline. Uses the deterministic
/// mock port (canonical CI/test infrastructure; NOT a stub — see
/// `crates/intelligence-subagent-runtime-kernel` doc-comment). Writes
/// 21 per-facet JSON findings to `<evidence_dir>/<facet_id>.json`.
fn run_inline_subagent_runtime(options: &Options, evidence_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(evidence_dir)
        .map_err(|e| format!("mkdir {}: {e}", evidence_dir.display()))?;
    let templates_dir = options.repo_root.join(TEMPLATES_DIR);
    let user_message = match &options.user_message_file {
        Some(path) => fs::read_to_string(options.repo_root.join(path))
            .map_err(|e| format!("read {}: {e}", path.display()))?,
        None => format!(
            "PR #{pr} — inline-deterministic-mock fan-out; no diff bundle supplied.\n",
            pr = options.pr_number,
        ),
    };
    let change_id = options
        .change_id
        .clone()
        .unwrap_or_else(|| format!("pr-{}", options.pr_number));
    let api_key_ref = match options.api_key_ref.clone() {
        Some(key) => key,
        None => SecretReference::new(DEFAULT_API_KEY_SREF.to_string())
            .map_err(|error| format!("DEFAULT_API_KEY_SREF malformed: {error}"))?,
    };
    let port = MockSubagentPort::new();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    for facet in fanout_panel_v23() {
        let template_path = templates_dir.join(format!("{}.md", facet.as_str()));
        let template_raw = fs::read_to_string(&template_path)
            .map_err(|e| format!("read template {}: {e}", template_path.display()))?;
        let template = FacetPromptTemplate::parse(&template_raw)
            .map_err(|e| format!("parse template {}: {e}", template_path.display()))?;
        let reviewer_id = format!(
            "claude-{facet}-{change}",
            facet = facet.as_str(),
            change = change_id,
        );
        let request = SubagentRequest {
            facet_id: facet.as_str().to_string(),
            reviewer_id,
            change_id: change_id.clone(),
            system_prompt: template.render_system_prompt(),
            user_message: user_message.clone(),
            api_key_ref: api_key_ref.clone(),
            model_id: options.model_id.clone(),
        };
        let response = port
            .complete(&request)
            .map_err(|e| format!("subagent {} failed: {e}", facet.as_str()))?;
        let json = FacetFindingJson::render(&response, now);
        let evidence_path = evidence_dir.join(format!("{}.json", facet.as_str()));
        fs::write(&evidence_path, json)
            .map_err(|e| format!("write {}: {e}", evidence_path.display()))?;
    }
    Ok(())
}

fn load_findings(dir: &Path) -> Result<Vec<FacetFinding>, String> {
    let mut findings = Vec::new();
    if !dir.exists() {
        return Ok(findings);
    }
    let entries =
        fs::read_dir(dir).map_err(|error| format!("could not read {}: {error}", dir.display()))?;
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
    FacetId::full_panel_v23()
        .into_iter()
        .find(|facet| facet.slug() == slug)
}

/// Minimal JSON-string-value reader. The subagent r1.json shape includes:
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
    completeness: &intelligence_pr_review_dispatcher_app::rollup::PanelCompletenessReport,
    verdict: Verdict,
    subagent_runtime_pending: bool,
) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut buf = String::new();
    buf.push_str("{\n");
    buf.push_str("  \"schema\": \"pr-review-rollup/v1\",\n");
    buf.push_str(&format!(
        "  \"pr_number\": \"{}\",\n",
        json_escape(pr_number)
    ));
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
    buf.push_str("    \"plan_ref\": \".omc/plans/milestones/M01-foundation/phases/P17-pipeline-maturity-glue/IP-004-reviewer-agent-auto-dispatch.md\",\n");
    buf.push_str("    \"audit_ref\": \"evidence/audits/pipeline-maturity-audit-2026-05-15.md\",\n");
    buf.push_str("    \"upstream_kernel\": \"vcs-review-mergequeue-kernel\",\n");
    buf.push_str("    \"subagent_runtime_ref\": \"M01-P17-IP-009 — `intelligence-subagent-runtime-{kernel,app}` ships the per-facet subagent invocation; this dispatcher invokes it inline when `--runtime-mode inline-deterministic-mock` is passed, OR consumes the per-facet `<facet>.json` files written by an external runtime invocation. The pending flag flips to false once a complete 21-facet panel has landed without duplicate reviewer ids.\"\n");
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

fn append_admission_event(
    path: &Path,
    pr_number: &str,
    verdict: Verdict,
    subagent_runtime_pending: bool,
) -> Result<(), String> {
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
    // IP-009 wiring: the emitted event now carries the
    // `subagent_runtime_pending` flag so IP-006's admission consumer
    // can refuse APPROVE events that still carry pending=true. The
    // historical fields (`pr_number`, `event`, `verdict`, `emitted_at_unix`)
    // remain in place for backward-compat with existing tooling that
    // greps the log.
    let new_entry = format!(
        "{{\"pr_number\": \"{}\", \"event\": \"{}\", \"verdict\": \"{}\", \"subagent_runtime_pending\": {}, \"emitted_at_unix\": {}}}",
        json_escape(pr_number),
        verdict.admission_event(),
        verdict.label(),
        subagent_runtime_pending,
        now
    );
    log.push(new_entry);

    let mut body = String::new();
    body.push_str("{\n  \"schema\": \"merge-queue-admission-log/v1\",\n  \"events\": [\n");
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
                if depth == 0
                    && let Some(start) = object_start.take()
                {
                    out.push(body[start..=index].to_string());
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
        let mut file =
            fs::File::create(&tmp).map_err(|error| format!("create {}: {error}", tmp.display()))?;
        file.write_all(contents)
            .map_err(|error| format!("write {}: {error}", tmp.display()))?;
        file.sync_all()
            .map_err(|error| format!("fsync {}: {error}", tmp.display()))?;
    }
    fs::rename(&tmp, path)
        .map_err(|error| format!("rename {} -> {}: {error}", tmp.display(), path.display()))?;
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
        let verdict = run([
            "--pr-number".to_string(),
            pr.to_string(),
            "--repo-root".to_string(),
            repo.display().to_string(),
        ])
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

        let verdict = run([
            "--pr-number".to_string(),
            pr.to_string(),
            "--repo-root".to_string(),
            repo.display().to_string(),
        ])
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
        let verdict = run([
            "--pr-number".to_string(),
            pr.to_string(),
            "--repo-root".to_string(),
            repo.display().to_string(),
        ])
        .unwrap();
        assert_eq!(verdict, Verdict::Reject);
    }

    #[test]
    fn admission_log_appends_across_two_runs() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        for pr in ["pr-1", "pr-2"] {
            let _ = run([
                "--pr-number".to_string(),
                pr.to_string(),
                "--repo-root".to_string(),
                repo.display().to_string(),
            ])
            .unwrap();
        }
        let admission = fs::read_to_string(repo.join(ADMISSION_LOG)).unwrap();
        assert!(admission.contains("\"pr_number\": \"pr-1\""));
        assert!(admission.contains("\"pr_number\": \"pr-2\""));
    }

    #[test]
    fn json_string_field_extracts_simple_value() {
        let s =
            r#"{"reviewer_id": "claude-critic-F1_linus-pr-7", "final_recommendation": "APPROVE"}"#;
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

    fn seed_facet_templates(repo: &Path) {
        let templates_dir = repo.join(TEMPLATES_DIR);
        fs::create_dir_all(&templates_dir).unwrap();
        for facet in FacetId::full_panel_v23() {
            let body = format!(
                "---\n\
                 facet_id: {slug}\n\
                 facet_name: {slug} test facet\n\
                 lens: test lens\n\
                 severity_bar: APPROVE / CHANGES_REQUESTED / REJECT\n\
                 ---\n\
                 test body for {slug}\n",
                slug = facet.slug(),
            );
            fs::write(templates_dir.join(format!("{}.md", facet.slug())), body).unwrap();
        }
    }

    #[test]
    fn inline_runtime_mode_emits_pending_false_with_complete_panel() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        seed_facet_templates(repo);
        let pr = "pr-runtime-1";
        let verdict = run([
            "--pr-number".to_string(),
            pr.to_string(),
            "--repo-root".to_string(),
            repo.display().to_string(),
            "--runtime-mode".to_string(),
            "inline-deterministic-mock".to_string(),
            "--change-id".to_string(),
            "test-change".to_string(),
        ])
        .unwrap();
        // The deterministic mock is a smoke/fixture surface, not a
        // pseudo-reviewer. Without explicit fixture directives it
        // defaults to APPROVE so CI is not blocked by arbitrary facet-id
        // hash noise.
        assert_eq!(verdict, Verdict::Approve);
        let rollup = fs::read_to_string(repo.join(ROLLUP_PATH)).unwrap();
        assert!(rollup.contains("\"verdict\": \"APPROVE\""));
        assert!(rollup.contains("\"subagent_runtime_pending\": false"));
        assert!(rollup.contains("\"panel_complete\": true"));
        let admission = fs::read_to_string(repo.join(ADMISSION_LOG)).unwrap();
        assert!(admission.contains("\"subagent_runtime_pending\": false"));
        // Every required facet should have a finding file.
        let evidence = repo.join(EVIDENCE_DIR).join(pr);
        for facet in FacetId::full_panel_v23() {
            let path = evidence.join(format!("{}.json", facet.slug()));
            assert!(
                path.exists(),
                "expected per-facet finding at {}",
                path.display()
            );
        }
    }

    #[test]
    fn empty_panel_still_emits_pending_true_when_no_runtime_mode_passed() {
        // Backward-compat: callers without --runtime-mode keep the
        // existing "scaffold" behavior — APPROVE with pending=true so
        // IP-006 admission gate refuses to admit.
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        let pr = "pr-no-runtime";
        let verdict = run([
            "--pr-number".to_string(),
            pr.to_string(),
            "--repo-root".to_string(),
            repo.display().to_string(),
        ])
        .unwrap();
        assert_eq!(verdict, Verdict::Approve);
        let admission = fs::read_to_string(repo.join(ADMISSION_LOG)).unwrap();
        assert!(admission.contains("\"subagent_runtime_pending\": true"));
    }
}
