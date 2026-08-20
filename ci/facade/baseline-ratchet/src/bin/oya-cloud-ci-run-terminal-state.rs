//! Adapter: fetch a required-context run from the GitHub Actions API and emit its TYPED terminal
//! state.
//!
//! All classification logic lives in the pure kernel
//! (`ci_baseline_ratchet::run_terminal_state`); this binary only does I/O. It reads the typed
//! run/jobs/annotations JSON — never a log — and writes the operator artifact.
//!
//! NOT `gh`. The sibling trusted-baseline consumer shells out to `gh api`, and `gh` is absent from
//! the owned arm64 runner image, which is why that fast path is silently dead there. This talks to
//! the REST API directly over `reqwest`, so it works on the owned fleet.
//!
//! ADDITIVE ONLY. This step never decides admission. The `oya-ci-required` fan-in remains the sole
//! merge authority and keeps its own verdict; this binary always exits 0 so that an observability
//! fault can never turn a green run red. Its output tells the operator which of `fix-candidate`,
//! `retry`, `fix-infra`, `wait`, or `needs-human` the run actually permits.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use ci_baseline_ratchet::run_terminal_state::{RunTerminalState, SubjectCounts, classify_run};
use serde_json::{Value, json};

const LOG: &str = "run-terminal-state";
const API: &str = "https://api.github.com";
const USER_AGENT: &str = "oya-cloud-ci-run-terminal-state";

fn arg(raw: &[String], name: &str) -> Option<String> {
    raw.iter()
        .position(|a| a == name)
        .and_then(|i| raw.get(i + 1))
        .cloned()
}

fn require(raw: &[String], name: &str) -> Result<String, String> {
    arg(raw, name).ok_or_else(|| format!("missing required argument `{name}`"))
}

/// `owner/repo`, validated before it is ever interpolated into a URL.
///
/// `.` is a legal character in a GitHub repo name, so a plain character-class check would admit
/// `owner/..` — two non-empty segments of legal characters that resolve to a different API route
/// once the URL is normalized. Segments are therefore rejected outright when they are `.` or `..`.
fn validated_repo(repo: &str) -> Result<String, String> {
    fn segment_ok(s: &str) -> bool {
        !s.is_empty()
            && s != "."
            && s != ".."
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    }
    let mut parts = repo.split('/');
    let ok = matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(o), Some(r), None) if segment_ok(o) && segment_ok(r)
    );
    if ok {
        Ok(repo.to_owned())
    } else {
        Err(format!("`--repo` must be <owner>/<repo>, got `{repo}`"))
    }
}

fn validated_run_id(run_id: &str) -> Result<String, String> {
    if !run_id.is_empty() && run_id.chars().all(|c| c.is_ascii_digit()) {
        Ok(run_id.to_owned())
    } else {
        Err(format!("`--run-id` must be numeric, got `{run_id}`"))
    }
}

struct Client {
    inner: reqwest::blocking::Client,
    token: String,
}

impl Client {
    fn new(token: String) -> Result<Self, String> {
        let inner = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("could not build HTTP client: {e}"))?;
        Ok(Self { inner, token })
    }

    fn get(&self, route: &str) -> Result<Value, String> {
        let url = format!("{API}/{route}");
        let resp = self
            .inner
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .map_err(|e| format!("GET {route}: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("GET {route}: HTTP {status}"));
        }
        resp.json()
            .map_err(|e| format!("GET {route}: response is not JSON: {e}"))
    }
}

/// Assemble the same `{run, jobs, annotations}` payload shape the acceptance fixtures use, so what
/// runs in CI is exactly what the tests classify.
fn fetch_payload(client: &Client, repo: &str, run_id: &str) -> Result<Value, String> {
    let run = client.get(&format!("repos/{repo}/actions/runs/{run_id}"))?;

    let mut jobs: Vec<Value> = Vec::new();
    for page in 1..=20 {
        let batch = client.get(&format!(
            "repos/{repo}/actions/runs/{run_id}/jobs?per_page=100&page={page}&filter=latest"
        ))?;
        let Some(list) = batch.get("jobs").and_then(Value::as_array) else {
            break;
        };
        let len = list.len();
        jobs.extend(list.iter().cloned());
        if len < 100 {
            break;
        }
    }

    // Annotations are failure evidence, so only red lanes need them. This keeps a 56-lane run to a
    // handful of extra calls instead of 56.
    let mut annotations = serde_json::Map::new();
    for job in &jobs {
        if job.get("conclusion").and_then(Value::as_str) == Some("success") {
            continue;
        }
        let Some(id) = job.get("id").and_then(Value::as_u64) else {
            continue;
        };
        // A missing annotations endpoint must not sink the whole classification — the kernel
        // treats annotations as corroborating evidence only.
        if let Ok(list) = client.get(&format!("repos/{repo}/check-runs/{id}/annotations")) {
            let messages: Vec<Value> = list
                .as_array()
                .map(|l| {
                    l.iter()
                        .map(|a| {
                            json!({
                                "annotation_level": a.get("annotation_level").cloned().unwrap_or(Value::Null),
                                "message": a.get("message").cloned().unwrap_or(Value::Null),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            annotations.insert(id.to_string(), Value::Array(messages));
        }
    }

    Ok(json!({
        "run": run,
        "jobs": jobs,
        "annotations": Value::Object(annotations),
    }))
}

/// Optional `{"<job name>": <subjects observed>}` sidecar. A lane reporting 0 is `verified-empty`
/// and goes RED; absent an entry, no emptiness claim is made.
fn load_subjects(path: Option<&str>) -> Result<SubjectCounts, String> {
    let Some(path) = path else {
        return Ok(SubjectCounts::new());
    };
    let text =
        fs::read_to_string(path).map_err(|e| format!("could not read subjects `{path}`: {e}"))?;
    let value: Value =
        serde_json::from_str(&text).map_err(|e| format!("could not parse `{path}`: {e}"))?;
    let map = value
        .as_object()
        .ok_or_else(|| format!("`{path}` must be a JSON object of job-name -> count"))?;
    let mut out: SubjectCounts = BTreeMap::new();
    for (k, v) in map {
        let n = v
            .as_u64()
            .ok_or_else(|| format!("`{path}`: `{k}` must be a non-negative integer"))?;
        out.insert(k.clone(), n);
    }
    Ok(out)
}

/// One line per non-green lane, then the verdict. This is what an operator reads instead of
/// opening the Actions UI.
fn print_report(state: &RunTerminalState) {
    println!(
        "{LOG}: candidate {} (run {})",
        state.candidate_sha, state.run_id
    );
    println!("{LOG}: tally {:?}", state.tally());
    for lane in &state.lanes {
        if lane.state.is_green() {
            continue;
        }
        println!(
            "{LOG}:   [{}] -> {} :: {} :: {}",
            lane.state, lane.next_action, lane.job_name, lane.because
        );
    }
    println!(
        "{LOG}: TERMINAL STATE = {} | PERMITTED NEXT ACTION = {} | classifier {} | digest {}",
        state.state, state.next_action, state.classifier_version, state.input_digest
    );
}

fn run(raw: &[String]) -> Result<(), String> {
    let repo = validated_repo(&require(raw, "--repo")?)?;
    let run_id = validated_run_id(&require(raw, "--run-id")?)?;
    let out = PathBuf::from(require(raw, "--out")?);
    let observed_at = require(raw, "--observed-at")?;
    let subjects = load_subjects(arg(raw, "--subjects").as_deref())?;
    let exclude = arg(raw, "--exclude-job-name");

    let token = std::env::var("GITHUB_TOKEN")
        .map_err(|_| "GITHUB_TOKEN is not set; the Actions API cannot be read".to_owned())?;
    let client = Client::new(token)?;

    let mut payload = fetch_payload(&client, &repo, &run_id)?;

    // The fan-in classifies the run it is itself running inside, so its own lane is always
    // in-progress and would otherwise be reported `blocked`. Drop it: it has no verdict to give
    // about the candidate.
    if let Some(name) = exclude.as_deref()
        && let Some(jobs) = payload.get_mut("jobs").and_then(Value::as_array_mut)
    {
        jobs.retain(|j| j.get("name").and_then(Value::as_str) != Some(name));
    }

    let state = classify_run(&payload, &observed_at, &subjects);
    print_report(&state);

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("could not create `{}`: {e}", parent.display()))?;
    }
    let mut bytes = serde_json::to_vec_pretty(&state.to_json())
        .map_err(|e| format!("could not serialize terminal state: {e}"))?;
    bytes.push(b'\n');
    fs::write(&out, bytes).map_err(|e| format!("could not write `{}`: {e}", out.display()))?;
    println!("{LOG}: wrote {}", out.display());
    Ok(())
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if let Err(err) = run(&raw) {
        // ADDITIVE ONLY: an observability fault must never turn a green run red, and must never
        // mask a red one. Report loudly, exit 0, and leave admission to the fan-in verdict.
        eprintln!("{LOG}: DEGRADED — no typed terminal state was emitted: {err}");
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_validation_rejects_path_traversal_and_injection() {
        assert!(validated_repo("jason931225/oyatie").is_ok());
        for bad in [
            "../../etc",
            "owner",
            "owner/repo/extra",
            "owner/re po",
            "",
            "owner/",
            // `.` is legal INSIDE a repo name, so these are the cases a bare character-class
            // check would wave through into the API route.
            "owner/..",
            "../repo",
            "owner/.",
            "./repo",
        ] {
            assert!(validated_repo(bad).is_err(), "should reject `{bad}`");
        }
        // A dot in a real repo name must still be accepted.
        assert!(validated_repo("owner/my.repo").is_ok());
    }

    #[test]
    fn run_id_validation_rejects_non_numeric() {
        assert!(validated_run_id("30677213867").is_ok());
        for bad in ["", "12a", "../1", "1 2"] {
            assert!(validated_run_id(bad).is_err(), "should reject `{bad}`");
        }
    }

    #[test]
    fn absent_subjects_sidecar_claims_nothing() {
        assert!(load_subjects(None).expect("ok").is_empty());
    }

    #[test]
    fn a_malformed_subjects_sidecar_is_refused_not_ignored() {
        let dir = std::env::temp_dir().join("oya-run-terminal-state-test");
        fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("bad-subjects.json");
        fs::write(&path, b"{\"lane\": -1}").expect("write");
        assert!(load_subjects(path.to_str()).is_err());
        fs::write(&path, b"{\"lane\": 0}").expect("write");
        assert_eq!(
            load_subjects(path.to_str()).expect("ok").get("lane"),
            Some(&0)
        );
    }
}
