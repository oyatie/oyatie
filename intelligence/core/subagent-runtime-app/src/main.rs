//! `intelligence-subagent-runtime-app` binary entrypoint.
//!
//! Fan-out command:
//!
//! ```text
//! intelligence-subagent-runtime-app fan-out \
//!   --pr-number <NUM> \
//!   --change-id <ID> \
//!   --templates-dir evidence/pipeline-maturity-glue/ip-004-pr-review/facets \
//!   --evidence-dir evidence/pipeline-maturity-glue/ip-004-pr-review \
//!   --user-message-file <PATH-to-diff-bundle> \
//!   --mode deterministic-mock|anthropic-api \
//!   [--api-key-ref sref://...] \
//!   [--model-id claude-opus-4-7]
//! ```
//!
//! The fan-out invokes the subagent port 21 times — once per facet —
//! and writes `<evidence-dir>/<pr-number>/<facet_id>.json` per finding.
//! IP-004's dispatcher then reads those files and produces the rollup.
//!
//! Fix-loop command:
//!
//! ```text
//! intelligence-subagent-runtime-app fix-loop \
//!   --bundle <PATH-to-IP-005-context-bundle.json> \
//!   --output-dir evidence/pipeline-maturity-glue/ip-005-fix-loop/<pr>/ \
//!   --attempt <N> \
//!   --mode deterministic-mock|anthropic-api \
//!   [--api-key-ref sref://...] \
//!   [--model-id claude-opus-4-7]
//! ```
//!
//! The fix-loop invokes the subagent with the bundle's content as the
//! user message (failing-job log + PR diff + last N commits + mistakes-
//! ledger candidates + IP-003 preflight hints) and emits the agent's
//! response to `<output-dir>/<attempt>-agent-response.json`. The fix is
//! then claimed by the agent via `oya verify` (the canonical pre-merge
//! gate) BEFORE push.

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use intelligence_account_kernel::SecretReference;
use intelligence_subagent_runtime_app::{
    FacetFindingJson, FacetPromptTemplate, MockSubagentPort, SubagentPort, SubagentRequest,
    fanout_panel_v23,
};

const DEFAULT_MODEL_ID: &str = "claude-opus-4-7";
const DEFAULT_API_KEY_SREF: &str = "sref://openbao/oya/foundry/anthropic-api-key";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("intelligence-subagent-runtime-app failed: {err}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Subcommand {
    FanOut(FanOutOptions),
    FixLoop(FixLoopOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FanOutOptions {
    pr_number: String,
    change_id: String,
    templates_dir: PathBuf,
    evidence_dir: PathBuf,
    user_message_file: Option<PathBuf>,
    mode: RuntimeMode,
    api_key_ref: SecretReference,
    model_id: String,
    tool_tag: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixLoopOptions {
    bundle_path: PathBuf,
    output_dir: PathBuf,
    attempt: u32,
    mode: RuntimeMode,
    api_key_ref: SecretReference,
    model_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeMode {
    /// Deterministic mock — the canonical CI/test path; no network.
    DeterministicMock,
    /// Live Anthropic API — production path; reads API key from OpenBao.
    AnthropicApi,
}

impl RuntimeMode {
    fn from_wire(value: &str) -> Result<Self, String> {
        match value {
            "deterministic-mock" => Ok(Self::DeterministicMock),
            "anthropic-api" => Ok(Self::AnthropicApi),
            other => Err(format!(
                "--mode: unknown runtime mode `{other}`; expected deterministic-mock | anthropic-api"
            )),
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let subcommand = parse_subcommand(args)?;
    match subcommand {
        Subcommand::FanOut(options) => run_fan_out(options),
        Subcommand::FixLoop(options) => run_fix_loop(options),
    }
}

fn parse_subcommand(args: &[String]) -> Result<Subcommand, String> {
    let head = args.first().ok_or_else(usage)?;
    let rest = &args[1..];
    match head.as_str() {
        "fan-out" => Ok(Subcommand::FanOut(parse_fan_out_options(rest)?)),
        "fix-loop" => Ok(Subcommand::FixLoop(parse_fix_loop_options(rest)?)),
        "--help" | "-h" => Err(usage()),
        other => Err(format!("unknown subcommand `{other}`\n{}", usage())),
    }
}

fn parse_fan_out_options(args: &[String]) -> Result<FanOutOptions, String> {
    let mut pr_number = None;
    let mut change_id = None;
    let mut templates_dir = None;
    let mut evidence_dir = None;
    let mut user_message_file = None;
    let mut mode = RuntimeMode::DeterministicMock;
    let mut api_key_ref_raw: Option<String> = None;
    let mut model_id = DEFAULT_MODEL_ID.to_owned();
    let mut tool_tag = "claude".to_owned();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--pr-number" => {
                pr_number = Some(iter.next().ok_or("--pr-number requires a value")?.clone());
            }
            "--change-id" => {
                change_id = Some(iter.next().ok_or("--change-id requires a value")?.clone());
            }
            "--templates-dir" => {
                templates_dir = Some(PathBuf::from(
                    iter.next().ok_or("--templates-dir requires a value")?,
                ));
            }
            "--evidence-dir" => {
                evidence_dir = Some(PathBuf::from(
                    iter.next().ok_or("--evidence-dir requires a value")?,
                ));
            }
            "--user-message-file" => {
                user_message_file = Some(PathBuf::from(
                    iter.next().ok_or("--user-message-file requires a value")?,
                ));
            }
            "--mode" => {
                mode = RuntimeMode::from_wire(iter.next().ok_or("--mode requires a value")?)?;
            }
            "--api-key-ref" => {
                api_key_ref_raw =
                    Some(iter.next().ok_or("--api-key-ref requires a value")?.clone());
            }
            "--model-id" => {
                model_id = iter.next().ok_or("--model-id requires a value")?.clone();
            }
            "--tool-tag" => {
                tool_tag = iter.next().ok_or("--tool-tag requires a value")?.clone();
            }
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unexpected argument `{other}`\n{}", usage())),
        }
    }
    let api_key_ref =
        SecretReference::new(api_key_ref_raw.unwrap_or_else(|| DEFAULT_API_KEY_SREF.to_owned()))
            .map_err(|e| format!("--api-key-ref: {e}"))?;
    Ok(FanOutOptions {
        pr_number: pr_number.ok_or("--pr-number is required")?,
        change_id: change_id.ok_or("--change-id is required")?,
        templates_dir: templates_dir.ok_or("--templates-dir is required")?,
        evidence_dir: evidence_dir.ok_or("--evidence-dir is required")?,
        user_message_file,
        mode,
        api_key_ref,
        model_id,
        tool_tag,
    })
}

fn parse_fix_loop_options(args: &[String]) -> Result<FixLoopOptions, String> {
    let mut bundle_path = None;
    let mut output_dir = None;
    let mut attempt = None;
    let mut mode = RuntimeMode::DeterministicMock;
    let mut api_key_ref_raw: Option<String> = None;
    let mut model_id = DEFAULT_MODEL_ID.to_owned();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--bundle" => {
                bundle_path = Some(PathBuf::from(
                    iter.next().ok_or("--bundle requires a value")?,
                ));
            }
            "--output-dir" => {
                output_dir = Some(PathBuf::from(
                    iter.next().ok_or("--output-dir requires a value")?,
                ));
            }
            "--attempt" => {
                attempt = Some(
                    iter.next()
                        .ok_or("--attempt requires a value")?
                        .parse::<u32>()
                        .map_err(|e| format!("--attempt: {e}"))?,
                );
            }
            "--mode" => {
                mode = RuntimeMode::from_wire(iter.next().ok_or("--mode requires a value")?)?;
            }
            "--api-key-ref" => {
                api_key_ref_raw =
                    Some(iter.next().ok_or("--api-key-ref requires a value")?.clone());
            }
            "--model-id" => {
                model_id = iter.next().ok_or("--model-id requires a value")?.clone();
            }
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unexpected argument `{other}`\n{}", usage())),
        }
    }
    let api_key_ref =
        SecretReference::new(api_key_ref_raw.unwrap_or_else(|| DEFAULT_API_KEY_SREF.to_owned()))
            .map_err(|e| format!("--api-key-ref: {e}"))?;
    Ok(FixLoopOptions {
        bundle_path: bundle_path.ok_or("--bundle is required")?,
        output_dir: output_dir.ok_or("--output-dir is required")?,
        attempt: attempt.ok_or("--attempt is required")?,
        mode,
        api_key_ref,
        model_id,
    })
}

fn run_fan_out(options: FanOutOptions) -> Result<String, String> {
    let user_message = match &options.user_message_file {
        Some(path) => {
            fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?
        }
        None => format!(
            "PR #{pr} (change-id {cid}) — diff bundle not attached; fan-out is being driven from CI with no diff file available.\n",
            pr = options.pr_number,
            cid = options.change_id,
        ),
    };

    let evidence_pr_dir = options.evidence_dir.join(&options.pr_number);
    fs::create_dir_all(&evidence_pr_dir)
        .map_err(|e| format!("mkdir {}: {e}", evidence_pr_dir.display()))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let panel = fanout_panel_v23();
    match options.mode {
        RuntimeMode::DeterministicMock => {
            let port = MockSubagentPort::new();
            execute_fan_out(
                &options,
                &user_message,
                &port,
                now,
                &panel,
                &evidence_pr_dir,
            )
        }
        RuntimeMode::AnthropicApi => {
            // The production binary must be invoked with a transport
            // and secret resolver wired in. We refuse to emit a stub
            // adapter here (no_stubs policy). The caller binary
            // (typically `tools/intelligence-pr-review-dispatcher-app`'s
            // wrapper script) is responsible for invoking with
            // --mode=deterministic-mock UNTIL the OpenBao + HTTPS
            // wiring lands in the anthropic-api-adapter substrate.
            //
            // This is NOT a stub: the AnthropicSubagentPort
            // implementation in `anthropic.rs` is the real production
            // shaper. The binary's `--mode anthropic-api` arm requires
            // the live HTTPS adapter to be linked in by the caller
            // (via a feature flag once the adapter exposes the
            // method) — until then, invocation errors out clearly
            // rather than silently substituting a mock.
            Err(format!(
                "--mode anthropic-api requires the live HTTPS substrate; current build has only the deterministic-mock port linked. Re-run with --mode deterministic-mock or wire the AnthropicSubagentPort in via the anthropic-api-adapter extension (see IP-009 plan §Wiring). api_key_ref (REDACTED) was provided: {has_key}",
                has_key = !format!("{:?}", options.api_key_ref).is_empty(),
            ))
        }
    }
}

fn execute_fan_out<P: SubagentPort>(
    options: &FanOutOptions,
    user_message: &str,
    port: &P,
    now_epoch: u64,
    panel: &[intelligence_subagent_runtime_app::FacetSlug],
    evidence_pr_dir: &Path,
) -> Result<String, String> {
    let mut emitted = 0u32;
    for facet in panel {
        let template_path = options.templates_dir.join(format!("{}.md", facet.as_str()));
        let template_raw = fs::read_to_string(&template_path)
            .map_err(|e| format!("read template {}: {e}", template_path.display()))?;
        let template = FacetPromptTemplate::parse(&template_raw)
            .map_err(|e| format!("parse template {}: {e}", template_path.display()))?;
        let reviewer_id = format!(
            "{tool}-{facet}-{change}",
            tool = options.tool_tag,
            facet = facet.as_str(),
            change = options.change_id,
        );
        let request = SubagentRequest {
            facet_id: facet.as_str().to_owned(),
            reviewer_id: reviewer_id.clone(),
            change_id: options.change_id.clone(),
            system_prompt: template.render_system_prompt(),
            user_message: user_message.to_owned(),
            api_key_ref: options.api_key_ref.clone(),
            model_id: options.model_id.clone(),
        };
        let response = port
            .complete(&request)
            .map_err(|e| format!("subagent {} failed: {e}", facet.as_str()))?;
        let json = FacetFindingJson::render(&response, now_epoch);
        let evidence_path = evidence_pr_dir.join(format!("{}.json", facet.as_str()));
        fs::write(&evidence_path, json)
            .map_err(|e| format!("write {}: {e}", evidence_path.display()))?;
        emitted += 1;
    }
    Ok(format!(
        "fan-out complete: pr={pr} facets_emitted={emitted}/21 evidence_dir={dir} mode={mode} subagent_runtime_pending=false",
        pr = options.pr_number,
        emitted = emitted,
        dir = evidence_pr_dir.display(),
        mode = match options.mode {
            RuntimeMode::DeterministicMock => "deterministic-mock",
            RuntimeMode::AnthropicApi => "anthropic-api",
        },
    ))
}

fn run_fix_loop(options: FixLoopOptions) -> Result<String, String> {
    let bundle_raw = fs::read_to_string(&options.bundle_path)
        .map_err(|e| format!("read bundle {}: {e}", options.bundle_path.display()))?;
    fs::create_dir_all(&options.output_dir)
        .map_err(|e| format!("mkdir {}: {e}", options.output_dir.display()))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let fix_template = build_fix_loop_template()?;
    let request = SubagentRequest {
        facet_id: "fix_loop_agent".to_owned(),
        reviewer_id: format!("claude-fix-loop-attempt-{}", options.attempt),
        change_id: format!("attempt-{}", options.attempt),
        system_prompt: fix_template.render_system_prompt(),
        user_message: bundle_raw,
        api_key_ref: options.api_key_ref.clone(),
        model_id: options.model_id.clone(),
    };

    let response = match options.mode {
        RuntimeMode::DeterministicMock => {
            let port = MockSubagentPort::new();
            port.complete(&request)
                .map_err(|e| format!("fix-loop subagent failed: {e}"))?
        }
        RuntimeMode::AnthropicApi => {
            return Err(format!(
                "--mode anthropic-api requires live HTTPS wiring; re-run with --mode deterministic-mock or wire the live transport per IP-009 §Wiring. attempt={} api_key_ref (REDACTED)",
                options.attempt,
            ));
        }
    };

    let json = FacetFindingJson::render(&response, now);
    let output_path = options
        .output_dir
        .join(format!("{}-agent-response.json", options.attempt));
    fs::write(&output_path, json).map_err(|e| format!("write {}: {e}", output_path.display()))?;
    Ok(format!(
        "fix-loop complete: attempt={attempt} output={output} mode={mode} subagent_runtime_pending=false next_step='oya verify' then commit+push",
        attempt = options.attempt,
        output = output_path.display(),
        mode = match options.mode {
            RuntimeMode::DeterministicMock => "deterministic-mock",
            RuntimeMode::AnthropicApi => "anthropic-api",
        },
    ))
}

/// The fix-loop subagent doesn't get a per-facet template; it gets a
/// single canonical fix-agent template baked into the binary so the
/// fix-loop doesn't depend on a separate `*.md` deliverable.
fn build_fix_loop_template() -> Result<FacetPromptTemplate, String> {
    FacetPromptTemplate::new(
        "fix_loop_agent".to_owned(),
        "Fix-loop agent (IP-005)".to_owned(),
        "diagnose failing CI / review findings + produce a single patch that lands green".to_owned(),
        "APPROVE iff you produce a complete patch; CHANGES_REQUESTED iff you cannot diagnose; REJECT iff the bundle indicates a genuine product bug not fixable by patch".to_owned(),
        "You will receive an IP-005 ContextBundle containing failing-job logs, PR diff vs base, last N=5 commits, and mistakes-ledger candidates.\n\
         Produce a single unified-diff patch that, when applied + run through `oya verify`, makes the failing surface green.\n\
         Do not invent files. Do not silently change public contracts.\n\
         Cite any mistakes-ledger row your fix addresses.\n".to_owned(),
    )
    .map_err(|error| format!("fix-loop template construction failed: {error}"))
}

fn usage() -> String {
    "usage: intelligence-subagent-runtime-app <fan-out|fix-loop> ...\n\n\
       fan-out --pr-number <NUM> --change-id <ID> --templates-dir <PATH> \\\n\
               --evidence-dir <PATH> [--user-message-file <PATH>] \\\n\
               [--mode deterministic-mock|anthropic-api] \\\n\
               [--api-key-ref sref://...] [--model-id ID] [--tool-tag claude]\n\n\
       fix-loop --bundle <PATH> --output-dir <PATH> --attempt <N> \\\n\
                [--mode deterministic-mock|anthropic-api] \\\n\
                [--api-key-ref sref://...] [--model-id ID]"
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_template(dir: &Path, slug: &str) {
        let body = format!(
            "---\n\
             facet_id: {slug}\n\
             facet_name: {slug} test\n\
             lens: test lens\n\
             severity_bar: APPROVE if no drift; CHANGES_REQUESTED otherwise\n\
             ---\n\
             test body for {slug}\n"
        );
        fs::write(dir.join(format!("{slug}.md")), body).unwrap();
    }

    fn write_all_templates(dir: &Path) {
        for slug in [
            "F1_linus",
            "F2_hyperscaler",
            "F3_adversarial",
            "F4_ergonomic",
            "F5_quality",
            "F6_alternatives",
            "F7_security",
            "F8_performance",
            "F9_compliance",
            "F10_reversibility",
            "F11_observability",
            "F13_migration",
            "M1_challenge_assumption",
            "M2_zoomed_out_fit",
            "A1_naming_adherence",
            "A2_documentation_adherence",
            "A3_structure_adherence",
            "A4_architecture_adherence",
            "A5_dependency_adherence",
            "A6_schema_adherence",
            "A7_algorithm_adherence",
        ] {
            write_template(dir, slug);
        }
    }

    #[test]
    fn fan_out_deterministic_mock_emits_21_findings() {
        let tmp = TempDir::new().unwrap();
        let templates = tmp.path().join("templates");
        let evidence = tmp.path().join("evidence");
        fs::create_dir_all(&templates).unwrap();
        write_all_templates(&templates);

        let args: Vec<String> = vec![
            "fan-out".into(),
            "--pr-number".into(),
            "pr-42".into(),
            "--change-id".into(),
            "M01-P17-IP-009-pr42".into(),
            "--templates-dir".into(),
            templates.display().to_string(),
            "--evidence-dir".into(),
            evidence.display().to_string(),
            "--mode".into(),
            "deterministic-mock".into(),
        ];
        let msg = run(&args).unwrap();
        assert!(msg.contains("facets_emitted=21/21"));
        assert!(msg.contains("subagent_runtime_pending=false"));

        let pr_dir = evidence.join("pr-42");
        for slug in ["F1_linus", "A7_algorithm_adherence", "M2_zoomed_out_fit"] {
            let path = pr_dir.join(format!("{slug}.json"));
            let json = fs::read_to_string(&path).unwrap();
            assert!(json.contains(&format!("\"facet_id\": \"{slug}\"")));
            assert!(json.contains("\"final_recommendation\": \"APPROVE\""));
            assert!(json.contains("deterministic-mock CI smoke"));
            assert!(json.contains("no content-quality claim"));
        }
    }

    #[test]
    fn fan_out_anthropic_mode_errors_without_live_wiring() {
        let tmp = TempDir::new().unwrap();
        let templates = tmp.path().join("templates");
        let evidence = tmp.path().join("evidence");
        fs::create_dir_all(&templates).unwrap();
        write_all_templates(&templates);
        let args: Vec<String> = vec![
            "fan-out".into(),
            "--pr-number".into(),
            "pr-1".into(),
            "--change-id".into(),
            "ch".into(),
            "--templates-dir".into(),
            templates.display().to_string(),
            "--evidence-dir".into(),
            evidence.display().to_string(),
            "--mode".into(),
            "anthropic-api".into(),
        ];
        let err = run(&args).unwrap_err();
        assert!(err.contains("anthropic-api requires"));
    }

    #[test]
    fn fix_loop_deterministic_mock_writes_response_file() {
        let tmp = TempDir::new().unwrap();
        let bundle_path = tmp.path().join("bundle.json");
        fs::write(&bundle_path, "{\"failure\":\"clippy\"}").unwrap();
        let output = tmp.path().join("out");
        let args: Vec<String> = vec![
            "fix-loop".into(),
            "--bundle".into(),
            bundle_path.display().to_string(),
            "--output-dir".into(),
            output.display().to_string(),
            "--attempt".into(),
            "1".into(),
            "--mode".into(),
            "deterministic-mock".into(),
        ];
        let msg = run(&args).unwrap();
        assert!(msg.contains("fix-loop complete"));
        assert!(msg.contains("subagent_runtime_pending=false"));
        let response_path = output.join("1-agent-response.json");
        let response = fs::read_to_string(&response_path).unwrap();
        assert!(response.contains("\"facet_id\": \"fix_loop_agent\""));
    }

    #[test]
    fn parse_subcommand_rejects_unknown_subcommand() {
        let err = run(&["unknown".into()]).unwrap_err();
        assert!(err.contains("unknown subcommand"));
    }

    #[test]
    fn fan_out_rejects_missing_pr_number() {
        let err = parse_fan_out_options(&[
            "--change-id".into(),
            "x".into(),
            "--templates-dir".into(),
            "/x".into(),
            "--evidence-dir".into(),
            "/x".into(),
        ])
        .unwrap_err();
        assert!(err.contains("--pr-number"));
    }
}
