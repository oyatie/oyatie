use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use serde_json::{Value, json};

const DEFAULT_MASTER_PLAN: &str = "docs/machine-readable/masterplan.generated.json";
const CLAIM_REF_PREFIX: &str = "refs/heads/claims";

#[derive(Clone, Debug, Eq, PartialEq)]
enum Action {
    Next,
    Claim,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Format {
    Text,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlanArgs {
    action: Action,
    master_plan: PathBuf,
    repo_root: PathBuf,
    deliverable: Option<String>,
    claimant: String,
    dry_run: bool,
    format: Format,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Deliverable {
    id: String,
    description: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ClaimProjection {
    deliverable_id: String,
    claim_ref: String,
    claimant: String,
    labels: Vec<String>,
}

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let parsed = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}\n{usage}");
            return ExitCode::from(2);
        }
    };
    match run_parsed(&parsed) {
        Ok(projection) => {
            print_projection(&projection, &parsed);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("oya plan: {message}");
            ExitCode::from(1)
        }
    }
}

fn parse_args(args: Vec<String>) -> Result<PlanArgs, String> {
    let mut iter = args.into_iter();
    let Some(action_raw) = iter.next() else {
        return Err("oya plan requires next, claim, or claim/next".into());
    };
    let action = match action_raw.as_str() {
        "next" => Action::Next,
        "claim" | "claim/next" => Action::Claim,
        other => return Err(format!("oya plan: unknown action {other:?}")),
    };
    let mut parsed = PlanArgs {
        action,
        master_plan: PathBuf::from(DEFAULT_MASTER_PLAN),
        repo_root: PathBuf::from("."),
        deliverable: None,
        claimant: default_claimant(),
        dry_run: false,
        format: Format::Text,
    };
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--master-plan" => parsed.master_plan = next_path(&mut iter, "--master-plan")?,
            "--repo-root" => parsed.repo_root = next_path(&mut iter, "--repo-root")?,
            "--deliverable" => parsed.deliverable = Some(next_value(&mut iter, "--deliverable")?),
            "--claimant" => parsed.claimant = next_value(&mut iter, "--claimant")?,
            "--dry-run" => parsed.dry_run = true,
            "--format" => {
                parsed.format = match next_value(&mut iter, "--format")?.as_str() {
                    "text" => Format::Text,
                    "json" => Format::Json,
                    other => return Err(format!("oya plan: unsupported format {other:?}")),
                };
            }
            other => return Err(format!("oya plan: unknown flag {other:?}")),
        }
    }
    Ok(parsed)
}

fn next_path(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<PathBuf, String> {
    Ok(PathBuf::from(next_value(iter, flag)?))
}

fn next_value(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    iter.next()
        .ok_or_else(|| format!("oya plan: {flag} requires a value"))
}

fn default_claimant() -> String {
    std::env::var("OYA_PLAN_CLAIMANT")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown-agent".into())
}

fn run_parsed(args: &PlanArgs) -> Result<ClaimProjection, String> {
    let deliverables = read_deliverables(&args.master_plan)?;
    let deliverable = if let Some(id) = &args.deliverable {
        deliverables
            .into_iter()
            .find(|deliverable| deliverable.id == *id)
            .ok_or_else(|| format!("deliverable not found in master plan: {id}"))?
    } else {
        next_unclaimed(&args.repo_root, deliverables)?
    };
    let projection = ClaimProjection {
        deliverable_id: deliverable.id.clone(),
        claim_ref: claim_ref(&deliverable.id),
        claimant: args.claimant.clone(),
        labels: exclusive_labels(&deliverable.id, &args.claimant),
    };
    if args.action == Action::Claim && !args.dry_run {
        acquire_claim(&args.repo_root, &deliverable, &args.claimant, &projection.claim_ref)?;
    }
    Ok(projection)
}

fn read_deliverables(path: &Path) -> Result<Vec<Deliverable>, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("master plan unreadable {}: {error}", path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .map_err(|error| format!("master plan JSON invalid {}: {error}", path.display()))?;
    let mut deliverables = Vec::new();
    collect_deliverables(&value, &mut deliverables);
    if deliverables.is_empty() {
        return Err(format!(
            "master plan contains no deliverables: {}",
            path.display()
        ));
    }
    Ok(deliverables)
}

fn collect_deliverables(value: &Value, deliverables: &mut Vec<Deliverable>) {
    match value {
        Value::Object(object) => {
            if let Some(Value::Array(items)) = object.get("deliverables") {
                for item in items {
                    if let Some(deliverable) = parse_deliverable(item) {
                        deliverables.push(deliverable);
                    }
                }
            }
            for child in object.values() {
                collect_deliverables(child, deliverables);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_deliverables(item, deliverables);
            }
        }
        _ => {}
    }
}

fn parse_deliverable(value: &Value) -> Option<Deliverable> {
    let object = value.as_object()?;
    let id = object.get("id")?.as_str()?.to_string();
    let description = object
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Some(Deliverable { id, description })
}

fn next_unclaimed(repo_root: &Path, deliverables: Vec<Deliverable>) -> Result<Deliverable, String> {
    for deliverable in deliverables {
        if !claim_exists(repo_root, &claim_ref(&deliverable.id))? {
            return Ok(deliverable);
        }
    }
    Err("no unclaimed deliverables found".into())
}

fn acquire_claim(
    repo_root: &Path,
    deliverable: &Deliverable,
    claimant: &str,
    claim_ref: &str,
) -> Result<(), String> {
    if claim_exists(repo_root, claim_ref)? {
        return Err(format!(
            "deliverable {} is already claimed at {claim_ref}",
            deliverable.id
        ));
    }
    let zero_oid = zero_oid(repo_root)?;
    let commit = create_claim_commit(repo_root, deliverable, claimant, claim_ref)?;
    let output = git(repo_root)
        .args(["update-ref", claim_ref, commit.trim(), &zero_oid])
        .output()
        .map_err(|error| format!("git update-ref failed to spawn: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git update-ref CAS failed for {claim_ref}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn create_claim_commit(
    repo_root: &Path,
    deliverable: &Deliverable,
    claimant: &str,
    claim_ref: &str,
) -> Result<String, String> {
    let tree = git_with_stdin(repo_root, ["mktree"], "")?;
    let message = format!(
        "Claim {id}\n\nDeliverable: {id}\nClaimant: {claimant}\nClaim-ref: {claim_ref}\nDescription: {description}\n",
        id = deliverable.id,
        description = deliverable.description
    );
    let output = git(repo_root)
        .env("GIT_AUTHOR_NAME", "oya-plan")
        .env("GIT_AUTHOR_EMAIL", "oya-plan@example.invalid")
        .env("GIT_COMMITTER_NAME", "oya-plan")
        .env("GIT_COMMITTER_EMAIL", "oya-plan@example.invalid")
        .args(["commit-tree", tree.trim(), "-m", &message])
        .output()
        .map_err(|error| format!("git commit-tree failed to spawn: {error}"))?;
    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|error| format!("git commit-tree output not UTF-8: {error}"))
    } else {
        Err(format!(
            "git commit-tree failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn git_with_stdin<const N: usize>(
    repo_root: &Path,
    args: [&str; N],
    stdin: &str,
) -> Result<String, String> {
    let mut child = git(repo_root)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("git failed to spawn: {error}"))?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| "git stdin unavailable".to_string())?
        .write_all(stdin.as_bytes())
        .map_err(|error| format!("git stdin write failed: {error}"))?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("git wait failed: {error}"))?;
    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|error| format!("git output not UTF-8: {error}"))
    } else {
        Err(format!(
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn claim_exists(repo_root: &Path, claim_ref: &str) -> Result<bool, String> {
    let output = git(repo_root)
        .args(["show-ref", "--verify", "--quiet", claim_ref])
        .output()
        .map_err(|error| format!("git show-ref failed to spawn: {error}"))?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(format!(
            "git show-ref failed for {claim_ref}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
    }
}

fn zero_oid(repo_root: &Path) -> Result<String, String> {
    let output = git(repo_root)
        .args(["rev-parse", "--show-object-format"])
        .output()
        .map_err(|error| format!("git rev-parse failed to spawn: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git rev-parse failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let format = String::from_utf8(output.stdout)
        .map_err(|error| format!("git rev-parse output not UTF-8: {error}"))?;
    let width = if format.trim() == "sha256" { 64 } else { 40 };
    Ok("0".repeat(width))
}

fn git(repo_root: &Path) -> Command {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo_root);
    command
}

fn claim_ref(deliverable_id: &str) -> String {
    format!("{CLAIM_REF_PREFIX}/{}", sanitize_ref_segment(deliverable_id))
}

fn exclusive_labels(deliverable_id: &str, claimant: &str) -> Vec<String> {
    vec![
        "state/claimed".into(),
        format!("owner/{}", sanitize_label_segment(claimant)),
        format!("deliverable/{}", sanitize_label_segment(deliverable_id)),
    ]
}

fn sanitize_ref_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn sanitize_label_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn print_projection(projection: &ClaimProjection, args: &PlanArgs) {
    match args.format {
        Format::Text => {
            let mode = match (&args.action, args.dry_run) {
                (Action::Next, _) => "next",
                (Action::Claim, true) => "would claim",
                (Action::Claim, false) => "claimed",
            };
            println!(
                "plan {mode}: {} -> {} labels={}",
                projection.deliverable_id,
                projection.claim_ref,
                projection.labels.join(",")
            );
        }
        Format::Json => {
            println!(
                "{}",
                json!({
                    "deliverable_id": projection.deliverable_id,
                    "claim_ref": projection.claim_ref,
                    "claimant": projection.claimant,
                    "labels": projection.labels,
                    "dry_run": args.dry_run
                })
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclusive_labels_are_scoped_single_value_projection() {
        assert_eq!(
            exclusive_labels("ADR-0377-D2", "Worker 1"),
            vec!["state/claimed", "owner/worker-1", "deliverable/adr-0377-d2"]
        );
    }

    #[test]
    fn collects_deliverables_from_generated_masterplan_shape() {
        let value = json!({
            "milestones": [{
                "adrs": [{
                    "deliverables": [
                        {"id": "ADR-0377-D2", "description": "claim next"},
                        {"id": "ADR-0377-D3", "description": "board sync"}
                    ]
                }]
            }]
        });
        let mut deliverables = Vec::new();
        collect_deliverables(&value, &mut deliverables);
        assert_eq!(
            deliverables,
            vec![
                Deliverable {
                    id: "ADR-0377-D2".into(),
                    description: "claim next".into()
                },
                Deliverable {
                    id: "ADR-0377-D3".into(),
                    description: "board sync".into()
                }
            ]
        );
    }
}
