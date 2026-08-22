//! cloud-ci-hub-exclusivity live producer (ADR-0711 / Swarm Delivery Law).
//!
//! Loads hub authority from `specs/integ-branch-envelopes.json#hubs.paths` (cite pointer;
//! never re-list), supplies open-PR changed-file facts (hermetic fixture JSON or live GitHub
//! REST when `--live-open-prs` / env is set), and runs the pure
//! [`ci_affected_target_set::hub_exclusivity`] evaluator. Multi-own → mechanical REFUSE (exit 1).
//!
//! Wired into the binding `oya-ci-required` affected-set admission path so the required
//! context evaluates concurrent integ PR hub ownership — not only synthetic rust_test fixtures.
//!
//! OWNED-RUST, NOT `gh`: the owned arm64 runner image does not install `gh` (same defect class
//! as the trusted-baseline consumer). Live facts go over `reqwest::blocking`.
//!
//! Absent envelopes → SKIP SUCCESS (fail-open only for missing authority file until envelopes
//! land). Hermetic Buck2 unit tests pass without network (fixture path / no `--live-open-prs`).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use ci_affected_target_set::hub_exclusivity::{
    DEFAULT_POLICY_RELPATH, ENVELOPES_RELPATH, GATE_ID, HUBS_PATHS_POINTER, Verdict,
    evaluate_from_producer_docs, filter_findings_for_candidate, open_pr_facts_from_json,
};
use serde_json::{Value, json};

const LOG: &str = "hub-exclusivity";
const GITHUB_API: &str = "https://api.github.com";
const HTTP_USER_AGENT: &str = "oya-cloud-ci-hub-exclusivity";
const GITHUB_API_VERSION: &str = "2026-03-10";
const HTTP_TIMEOUT: Duration = Duration::from_secs(60);
const PER_PAGE: u32 = 100;
const MAX_PAGES: u32 = 20;
/// Must match policy `integ_head_ref_prefix` (DATA). Producer filters here so non-integ PRs do
/// not pay the files-list API tax; the evaluator also filters by the same prefix.
const INTEG_HEAD_REF_PREFIX: &str = "integ/";
/// Opt-in live discovery (workflow sets this). Absent → empty open-prs (hermetic SKIP path).
const LIVE_OPEN_PRS_ENV: &str = "OYA_CI_HUB_EXCLUSIVITY_LIVE_OPEN_PRS";

struct Args {
    policy_path: PathBuf,
    envelopes_path: PathBuf,
    open_prs_fixture: Option<PathBuf>,
    live_open_prs: bool,
    repo: Option<String>,
    /// When set, only multi-own findings that implicate this PR refuse the candidate.
    candidate_pr: Option<u64>,
}

fn env_truthy(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

fn parse_args(mut argv: std::env::Args) -> Result<Args, String> {
    let _bin = argv.next();
    let mut repo_root = PathBuf::from(".");
    let mut policy_path = None;
    let mut envelopes_path = None;
    let mut open_prs_fixture = None;
    let mut live_open_prs = env_truthy(LIVE_OPEN_PRS_ENV);
    let mut repo = None;
    let mut candidate_pr = None;
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(argv.next().ok_or("--repo-root needs a value")?)
            }
            "--policy" => {
                policy_path = Some(PathBuf::from(argv.next().ok_or("--policy needs a value")?))
            }
            "--envelopes" => {
                envelopes_path = Some(PathBuf::from(
                    argv.next().ok_or("--envelopes needs a value")?,
                ))
            }
            "--open-prs-fixture" => {
                open_prs_fixture = Some(PathBuf::from(
                    argv.next().ok_or("--open-prs-fixture needs a value")?,
                ))
            }
            "--live-open-prs" => live_open_prs = true,
            "--repo" => repo = Some(argv.next().ok_or("--repo needs a value")?),
            "--candidate-pr" => {
                let raw = argv.next().ok_or("--candidate-pr needs a value")?;
                let n: u64 = raw
                    .parse()
                    .map_err(|_| format!("--candidate-pr must be a PR number, got {raw:?}"))?;
                candidate_pr = Some(n);
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(Args {
        policy_path: policy_path.unwrap_or_else(|| repo_root.join(DEFAULT_POLICY_RELPATH)),
        envelopes_path: envelopes_path.unwrap_or_else(|| repo_root.join(ENVELOPES_RELPATH)),
        open_prs_fixture,
        live_open_prs,
        repo,
        candidate_pr,
    })
}

fn read_json(path: &Path) -> Result<Value, String> {
    let raw =
        fs::read_to_string(path).map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("cannot parse `{}`: {e}", path.display()))
}

struct GitHubApi {
    client: reqwest::blocking::Client,
    token: String,
}

impl GitHubApi {
    /// `GH_TOKEN` first (workflow), then `GITHUB_TOKEN`. Absent/blank is a capability fault.
    fn from_env() -> Result<Self, String> {
        let token = ["GH_TOKEN", "GITHUB_TOKEN"]
            .iter()
            .find_map(|name| std::env::var(name).ok())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                "neither GH_TOKEN nor GITHUB_TOKEN is set — cannot load live open-PR file facts"
                    .to_owned()
            })?;
        let client = reqwest::blocking::Client::builder()
            .user_agent(HTTP_USER_AGENT)
            .timeout(HTTP_TIMEOUT)
            .build()
            .map_err(|e| format!("could not build the GitHub HTTP client: {e}"))?;
        Ok(Self { client, token })
    }

    fn get_json(&self, route: &str) -> Result<Value, String> {
        let response = self
            .client
            .get(format!("{GITHUB_API}/{route}"))
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .map_err(|e| format!("GET {route}: unreachable: {e}"))?;
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(format!("GET {route}: HTTP {status}"));
        }
        let text = response
            .text()
            .map_err(|e| format!("GET {route}: body read failed: {e}"))?;
        serde_json::from_str(&text).map_err(|e| format!("GET {route}: JSON parse failed: {e}"))
    }
}

fn is_integ_head_ref(head_ref: &str) -> bool {
    head_ref.starts_with(INTEG_HEAD_REF_PREFIX)
}

/// Fetch open PRs + changed files, emitting the shape accepted by [`open_pr_facts_from_json`].
/// Only `integ/*` head refs are retained (files endpoints are skipped for others).
fn fetch_open_pr_file_facts(api: &GitHubApi, repo: &str) -> Result<Value, String> {
    if !repo.contains('/') || repo.split('/').count() != 2 {
        return Err(format!("--repo must be owner/name, got {repo:?}"));
    }
    let mut pulls: Vec<Value> = Vec::new();
    for page in 1..=MAX_PAGES {
        let route = format!("repos/{repo}/pulls?state=open&per_page={PER_PAGE}&page={page}");
        let batch = api.get_json(&route)?;
        let arr = batch
            .as_array()
            .ok_or_else(|| format!("GET {route}: expected JSON array"))?;
        if arr.is_empty() {
            break;
        }
        pulls.extend(arr.iter().cloned());
        if arr.len() < PER_PAGE as usize {
            break;
        }
        if page == MAX_PAGES {
            return Err(format!(
                "open PR list exceeded {MAX_PAGES} pages of {PER_PAGE} — refuse rather than truncate"
            ));
        }
    }

    let mut facts = Vec::new();
    for pr in &pulls {
        let number = pr
            .get("number")
            .and_then(Value::as_u64)
            .ok_or_else(|| "open PR missing numeric `number`".to_owned())?;
        let head_ref = pr
            .get("head")
            .and_then(|h| h.get("ref"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if head_ref.is_empty() {
            return Err(format!("PR #{number} missing head.ref"));
        }
        if !is_integ_head_ref(head_ref) {
            continue;
        }
        let mut files: Vec<Value> = Vec::new();
        let mut oversized = false;
        for page in 1..=MAX_PAGES {
            let route =
                format!("repos/{repo}/pulls/{number}/files?per_page={PER_PAGE}&page={page}");
            let batch = api.get_json(&route)?;
            let arr = batch
                .as_array()
                .ok_or_else(|| format!("GET {route}: expected JSON array"))?;
            if arr.is_empty() {
                break;
            }
            for f in arr {
                let filename = f
                    .get("filename")
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("PR #{number} file entry missing `filename`"))?;
                files.push(json!({ "filename": filename }));
            }
            if arr.len() < PER_PAGE as usize {
                break;
            }
            if page == MAX_PAGES {
                // Do not hard-fail the whole fleet when one oversized integ PR (e.g. dump
                // delete) exceeds the page ceiling. Skip it so it cannot claim hub ownership
                // (truncated facts must not mint exclusivity rights) and cannot poison unrelated
                // integ lanes. Open-PR list overflow above still refuses.
                eprintln!(
                    "{LOG}: skip PR #{number} ({head_ref}) — file list exceeded {MAX_PAGES} pages \
                     of {PER_PAGE}; oversized PRs cannot claim hub exclusivity"
                );
                oversized = true;
                break;
            }
        }
        if oversized {
            continue;
        }
        facts.push(json!({
            "number": number,
            "head_ref_name": head_ref,
            "files": files,
        }));
    }
    Ok(Value::Array(facts))
}

fn resolve_repo(args: &Args) -> Result<String, String> {
    if let Some(r) = args
        .repo
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return Ok(r.to_owned());
    }
    match std::env::var("GITHUB_REPOSITORY") {
        Ok(r) if !r.trim().is_empty() => Ok(r),
        _ => Err("live mode requires --repo owner/name or GITHUB_REPOSITORY \
             (or pass --open-prs-fixture for hermetic runs)"
            .to_owned()),
    }
}

fn load_open_pr_facts(args: &Args) -> Result<Value, ExitCode> {
    if let Some(fixture) = &args.open_prs_fixture {
        return match read_json(fixture) {
            Ok(v) => Ok(v),
            Err(e) => {
                eprintln!("{LOG}: FIXTURE ERROR: {e}");
                Err(ExitCode::from(2))
            }
        };
    }
    if !args.live_open_prs {
        eprintln!(
            "{LOG}: SKIP — no hermetic open-PR fixture and --live-open-prs / {LIVE_OPEN_PRS_ENV} unset"
        );
        return Ok(Value::Array(Vec::new()));
    }
    let repo = match resolve_repo(args) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{LOG}: FAIL — {e}");
            return Err(ExitCode::from(2));
        }
    };
    let api = match GitHubApi::from_env() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{LOG}: FAIL — {e}");
            return Err(ExitCode::from(2));
        }
    };
    match fetch_open_pr_file_facts(&api, &repo) {
        Ok(v) => {
            println!(
                "{LOG}: live open-PR facts loaded — integ_head_ref_prefix={INTEG_HEAD_REF_PREFIX}; count={}",
                v.as_array().map(Vec::len).unwrap_or(0)
            );
            Ok(v)
        }
        Err(e) => {
            eprintln!("{LOG}: FAIL — live open-PR file facts: {e}");
            Err(ExitCode::from(2))
        }
    }
}

fn run(args: &Args) -> ExitCode {
    if !args.envelopes_path.is_file() {
        println!(
            "{LOG}: SKIP — `{}` absent; hub authority pointer {HUBS_PATHS_POINTER} not yet on tip",
            args.envelopes_path.display()
        );
        return ExitCode::SUCCESS;
    }

    let policy_doc = match read_json(&args.policy_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{LOG}: POLICY ERROR: {e}");
            return ExitCode::from(2);
        }
    };
    let envelopes_doc = match read_json(&args.envelopes_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{LOG}: ENVELOPES ERROR: {e}");
            return ExitCode::from(2);
        }
    };

    let open_prs_doc = match load_open_pr_facts(args) {
        Ok(v) => v,
        Err(code) => return code,
    };

    if let Err(finding) = open_pr_facts_from_json(&open_prs_doc) {
        eprintln!(
            "{LOG}: REFUSE — {}: {} ({})",
            finding.code, finding.key, finding.detail
        );
        return ExitCode::from(1);
    }

    let report = evaluate_from_producer_docs(&policy_doc, &envelopes_doc, &open_prs_doc);
    let (report, deferred) = filter_findings_for_candidate(report, args.candidate_pr);
    for f in &deferred {
        eprintln!(
            "{LOG}: observe (not candidate) — [{}] {}: {}",
            f.code, f.key, f.detail
        );
    }
    match report.verdict {
        Verdict::Green => {
            println!(
                "{LOG}: GREEN — {GATE_ID}; authority={HUBS_PATHS_POINTER}; open_prs={}; candidate={:?}",
                open_prs_doc.as_array().map(Vec::len).unwrap_or(0),
                args.candidate_pr
            );
            ExitCode::SUCCESS
        }
        Verdict::Refuse => {
            eprintln!("{LOG}: REFUSE — {GATE_ID} findings:");
            for f in &report.findings {
                eprintln!("{LOG}:   [{}] {}: {}", f.code, f.key, f.detail);
            }
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{LOG}: ARGS ERROR: {e}");
            eprintln!(
                "{LOG}: usage: oya-cloud-ci-hub-exclusivity [--repo-root <path>] [--policy <pack.json>] \
                 [--envelopes <envelopes.json>] [--open-prs-fixture <facts.json> | \
                 --live-open-prs [--repo owner/name]]"
            );
            return ExitCode::from(2);
        }
    };
    run(&args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fetch_shape_round_trips_through_open_pr_parser() {
        let emitted = json!([
            {
                "number": 1643,
                "head_ref_name": "integ/os",
                "files": [{ "filename": "Cargo.lock" }]
            },
            {
                "number": 1647,
                "head_ref_name": "integ/build",
                "files": [{ "filename": "Cargo.lock" }, { "filename": "build/x.rs" }]
            }
        ]);
        let facts = open_pr_facts_from_json(&emitted).expect("parse emitted shape");
        assert_eq!(facts.len(), 2);
        assert!(facts[0].files.contains("Cargo.lock"));
        assert!(facts[1].files.contains("build/x.rs"));
    }

    #[test]
    fn absent_live_flag_skips_network_with_empty_facts() {
        let args = Args {
            policy_path: PathBuf::from("unused"),
            envelopes_path: PathBuf::from("unused"),
            open_prs_fixture: None,
            live_open_prs: false,
            repo: None,
            candidate_pr: None,
        };
        let doc = load_open_pr_facts(&args).expect("hermetic empty facts");
        assert_eq!(doc, json!([]));
    }

    #[test]
    fn integ_head_ref_prefix_filters_non_integ() {
        assert!(is_integ_head_ref("integ/ci"));
        assert!(is_integ_head_ref("integ/os"));
        assert!(!is_integ_head_ref("feature/x"));
        assert!(!is_integ_head_ref("dev"));
        assert!(!is_integ_head_ref(""));
    }

    #[test]
    fn resolve_repo_prefers_explicit_flag() {
        let args = Args {
            policy_path: PathBuf::from("unused"),
            envelopes_path: PathBuf::from("unused"),
            open_prs_fixture: None,
            live_open_prs: true,
            repo: Some("owner/name".to_owned()),
            candidate_pr: None,
        };
        assert_eq!(resolve_repo(&args).expect("repo"), "owner/name");
    }

    #[test]
    fn fixture_path_beats_live_mode_without_network() {
        // Use a tiny in-memory-equivalent: missing fixture path must fail closed (exit 2),
        // proving live mode is not attempted when --open-prs-fixture is set.
        let args = Args {
            policy_path: PathBuf::from("unused"),
            envelopes_path: PathBuf::from("unused"),
            open_prs_fixture: Some(PathBuf::from("/nonexistent/open-prs.json")),
            live_open_prs: true,
            repo: Some("owner/name".to_owned()),
            candidate_pr: None,
        };
        let err =
            load_open_pr_facts(&args).expect_err("missing fixture must not fall through to live");
        assert_eq!(err, ExitCode::from(2));
    }
}
