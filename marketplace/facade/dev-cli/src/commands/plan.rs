use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

const DEFAULT_MASTER_PLAN: &str = "docs/machine-readable/masterplan.generated.json";
const CLAIM_REF_PREFIX: &str = "refs/heads/claims";
const ID_RESERVATION_REF_PREFIX: &str = "refs/heads/id-reservations";
const DEFAULT_REMOTE: &str = "origin";
const DEFAULT_LEASE_SECONDS: u64 = 24 * 60 * 60;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Action {
    Next,
    Claim,
    ReserveId,
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
    remote: String,
    deliverable: Option<String>,
    reservation_id: Option<String>,
    claimant: String,
    dry_run: bool,
    format: Format,
    lease_seconds: u64,
    recover_stale: bool,
    recovery_reason: Option<String>,
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
        return Err("oya plan requires next, claim, claim/next, or reserve-id".into());
    };
    let action = match action_raw.as_str() {
        "next" => Action::Next,
        "claim" | "claim/next" => Action::Claim,
        "reserve-id" => Action::ReserveId,
        other => return Err(format!("oya plan: unknown action {other:?}")),
    };
    let mut parsed = PlanArgs {
        action,
        master_plan: PathBuf::from(DEFAULT_MASTER_PLAN),
        repo_root: PathBuf::from("."),
        remote: DEFAULT_REMOTE.to_owned(),
        deliverable: None,
        reservation_id: None,
        claimant: default_claimant(),
        dry_run: false,
        format: Format::Text,
        lease_seconds: DEFAULT_LEASE_SECONDS,
        recover_stale: false,
        recovery_reason: None,
    };
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--master-plan" => parsed.master_plan = next_path(&mut iter, "--master-plan")?,
            "--repo-root" => parsed.repo_root = next_path(&mut iter, "--repo-root")?,
            "--remote" => parsed.remote = next_value(&mut iter, "--remote")?,
            "--deliverable" => parsed.deliverable = Some(next_value(&mut iter, "--deliverable")?),
            "--id" => parsed.reservation_id = Some(next_value(&mut iter, "--id")?),
            "--claimant" => parsed.claimant = next_value(&mut iter, "--claimant")?,
            "--dry-run" => parsed.dry_run = true,
            "--lease-seconds" => {
                parsed.lease_seconds = next_value(&mut iter, "--lease-seconds")?
                    .parse::<u64>()
                    .map_err(|error| format!("oya plan: --lease-seconds must be u64: {error}"))?;
            }
            "--recover-stale" => parsed.recover_stale = true,
            "--recovery-reason" => {
                parsed.recovery_reason = Some(next_value(&mut iter, "--recovery-reason")?)
            }
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

fn next_path(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
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
    if args.action == Action::ReserveId {
        return reserve_id(args);
    }

    let deliverables = read_deliverables(&args.master_plan)?;
    let deliverable = if let Some(id) = &args.deliverable {
        deliverables
            .into_iter()
            .find(|deliverable| deliverable.id == *id)
            .ok_or_else(|| format!("deliverable not found in master plan: {id}"))?
    } else {
        next_unclaimed(&args.repo_root, &args.remote, deliverables)?
    };
    let projection = ClaimProjection {
        deliverable_id: deliverable.id.clone(),
        claim_ref: claim_ref(&deliverable.id),
        claimant: args.claimant.clone(),
        labels: exclusive_labels(&deliverable.id, &args.claimant),
    };
    if args.action == Action::Claim && !args.dry_run {
        acquire_claim(&args.repo_root, &deliverable, args, &projection.claim_ref)?;
    }
    Ok(projection)
}

fn reserve_id(args: &PlanArgs) -> Result<ClaimProjection, String> {
    let id = args
        .reservation_id
        .as_ref()
        .ok_or_else(|| "oya plan reserve-id requires --id <ADR-NNNN|PRD-...>".to_string())?;
    validate_reservation_id(id)?;
    let claim_ref = id_reservation_ref(id);
    ensure_id_not_in_flight(&args.repo_root, &args.remote, id, &claim_ref)?;
    let deliverable = Deliverable {
        id: id.clone(),
        description: format!("Canonical id reservation for {id}"),
    };
    let projection = ClaimProjection {
        deliverable_id: id.clone(),
        claim_ref: claim_ref.clone(),
        claimant: args.claimant.clone(),
        labels: id_reservation_labels(id, &args.claimant),
    };
    if !args.dry_run {
        acquire_claim(&args.repo_root, &deliverable, args, &claim_ref)?;
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

fn next_unclaimed(
    repo_root: &Path,
    remote: &str,
    deliverables: Vec<Deliverable>,
) -> Result<Deliverable, String> {
    for deliverable in deliverables {
        if remote_claim_oid(repo_root, remote, &claim_ref(&deliverable.id))?.is_none() {
            return Ok(deliverable);
        }
    }
    Err("no unclaimed deliverables found".into())
}

fn acquire_claim(
    repo_root: &Path,
    deliverable: &Deliverable,
    args: &PlanArgs,
    claim_ref: &str,
) -> Result<(), String> {
    let observed = remote_claim_oid(repo_root, &args.remote, claim_ref)?;
    let now = unix_now()?;
    let push_mode = match observed {
        None => ClaimPushMode::Create,
        Some(existing_oid) if args.recover_stale => {
            fetch_claim_ref(repo_root, &args.remote, claim_ref)?;
            let metadata = read_claim_metadata(repo_root, &existing_oid)?;
            let expires_at = metadata
                .lease_expires_at
                .ok_or_else(|| format!("existing claim {claim_ref} has no Lease-expires-at"))?;
            if expires_at > now {
                return Err(format!(
                    "deliverable {} is already claimed at {claim_ref}; lease active until {expires_at}",
                    deliverable.id
                ));
            }
            ClaimPushMode::Recover {
                expected_old_oid: existing_oid,
            }
        }
        Some(_) => {
            return Err(format!(
                "deliverable {} is already claimed at {claim_ref}",
                deliverable.id
            ));
        }
    };
    let source_commit = current_source_commit(repo_root)?;
    let commit = create_claim_commit(
        repo_root,
        ClaimCommitInput {
            deliverable,
            claimant: &args.claimant,
            claim_ref,
            source_commit: &source_commit,
            lease_started_at: now,
            lease_seconds: args.lease_seconds,
            recovery_reason: args.recovery_reason.as_deref(),
        },
    )?;
    push_claim(repo_root, &args.remote, claim_ref, commit.trim(), push_mode)?;
    mirror_claim_ref(repo_root, claim_ref, commit.trim())?;
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ClaimPushMode {
    Create,
    Recover { expected_old_oid: String },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ClaimMetadata {
    lease_expires_at: Option<u64>,
}

fn push_claim(
    repo_root: &Path,
    remote: &str,
    claim_ref: &str,
    commit: &str,
    mode: ClaimPushMode,
) -> Result<(), String> {
    let mut command = git(repo_root);
    command.arg("push");
    match mode {
        ClaimPushMode::Create => {}
        ClaimPushMode::Recover { expected_old_oid } => {
            command.arg(format!("--force-with-lease={claim_ref}:{expected_old_oid}"));
        }
    }
    command.arg(remote).arg(format!("{commit}:{claim_ref}"));
    let output = command
        .output()
        .map_err(|error| format!("git push failed to spawn: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("fetch first")
        || stderr.contains("stale info")
        || stderr.contains("already exists")
        || stderr.contains("non-fast-forward")
        || stderr.contains("failed to push some refs")
    {
        Err(format!(
            "remote claim CAS lost for {claim_ref}: {}",
            stderr.trim()
        ))
    } else {
        Err(format!(
            "git push failed for {claim_ref}: {}",
            stderr.trim()
        ))
    }
}

struct ClaimCommitInput<'a> {
    deliverable: &'a Deliverable,
    claimant: &'a str,
    claim_ref: &'a str,
    source_commit: &'a str,
    lease_started_at: u64,
    lease_seconds: u64,
    recovery_reason: Option<&'a str>,
}

fn create_claim_commit(repo_root: &Path, input: ClaimCommitInput<'_>) -> Result<String, String> {
    let tree = git_with_stdin(repo_root, ["mktree"], "")?;
    let lease_expires_at = input.lease_started_at.saturating_add(input.lease_seconds);
    let recovery = input
        .recovery_reason
        .filter(|reason| !reason.trim().is_empty())
        .map(|reason| format!("Recovery-reason: {}\n", reason.trim()))
        .unwrap_or_default();
    let message = format!(
        "Claim {id}\n\nDeliverable: {id}\nClaimant: {claimant}\nClaim-ref: {claim_ref}\nSource-commit: {source_commit}\nLease-started-at: {lease_started_at}\nLease-expires-at: {lease_expires_at}\n{recovery}Description: {description}\n",
        id = input.deliverable.id,
        claimant = input.claimant,
        claim_ref = input.claim_ref,
        source_commit = input.source_commit,
        lease_started_at = input.lease_started_at,
        description = input.deliverable.description
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

fn remote_claim_oid(
    repo_root: &Path,
    remote: &str,
    claim_ref: &str,
) -> Result<Option<String>, String> {
    let output = git(repo_root)
        .args(["ls-remote", "--exit-code", remote, claim_ref])
        .output()
        .map_err(|error| format!("git ls-remote failed to spawn: {error}"))?;
    match output.status.code() {
        Some(0) => {
            let stdout = String::from_utf8(output.stdout)
                .map_err(|error| format!("git ls-remote output not UTF-8: {error}"))?;
            let oid = stdout
                .split_whitespace()
                .next()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| format!("git ls-remote returned no oid for {claim_ref}"))?;
            Ok(Some(oid.to_owned()))
        }
        // `git ls-remote --exit-code` returns 2 when the remote is reachable
        // but the ref pattern matched nothing.
        Some(2) => Ok(None),
        _ => Err(format!(
            "git ls-remote failed for {remote} {claim_ref}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
    }
}

fn fetch_claim_ref(repo_root: &Path, remote: &str, claim_ref: &str) -> Result<(), String> {
    let output = git(repo_root)
        .args(["fetch", "--quiet", remote, claim_ref])
        .output()
        .map_err(|error| format!("git fetch failed to spawn: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git fetch failed for {remote} {claim_ref}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn read_claim_metadata(repo_root: &Path, oid: &str) -> Result<ClaimMetadata, String> {
    let output = git(repo_root)
        .args(["cat-file", "-p", oid])
        .output()
        .map_err(|error| format!("git cat-file failed to spawn: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git cat-file failed for {oid}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text = String::from_utf8(output.stdout)
        .map_err(|error| format!("git cat-file output not UTF-8: {error}"))?;
    let mut metadata = ClaimMetadata::default();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("Lease-expires-at: ") {
            metadata.lease_expires_at = value.trim().parse::<u64>().ok();
        }
    }
    Ok(metadata)
}

fn current_source_commit(repo_root: &Path) -> Result<String, String> {
    let output = git(repo_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("git rev-parse HEAD failed to spawn: {error}"))?;
    if output.status.success() {
        String::from_utf8(output.stdout)
            .map(|value| value.trim().to_owned())
            .map_err(|error| format!("git rev-parse HEAD output not UTF-8: {error}"))
    } else {
        Err(format!(
            "git rev-parse HEAD failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn unix_now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| format!("system clock before epoch: {error}"))
}

fn mirror_claim_ref(repo_root: &Path, claim_ref: &str, commit: &str) -> Result<(), String> {
    let output = git(repo_root)
        .args(["update-ref", claim_ref, commit])
        .output()
        .map_err(|error| format!("git update-ref failed to spawn: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git update-ref failed for local mirror {claim_ref}: {}",
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

fn git(repo_root: &Path) -> Command {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo_root);
    command
}

fn claim_ref(deliverable_id: &str) -> String {
    format!(
        "{CLAIM_REF_PREFIX}/{}",
        sanitize_ref_segment(deliverable_id)
    )
}

fn id_reservation_ref(id: &str) -> String {
    format!("{ID_RESERVATION_REF_PREFIX}/{}", sanitize_ref_segment(id))
}

fn exclusive_labels(deliverable_id: &str, claimant: &str) -> Vec<String> {
    vec![
        "state/claimed".into(),
        format!("owner/{}", sanitize_label_segment(claimant)),
        format!("deliverable/{}", sanitize_label_segment(deliverable_id)),
    ]
}

fn id_reservation_labels(id: &str, claimant: &str) -> Vec<String> {
    vec![
        "state/reserved".into(),
        format!("owner/{}", sanitize_label_segment(claimant)),
        format!("canonical-id/{}", sanitize_label_segment(id)),
    ]
}

fn validate_reservation_id(id: &str) -> Result<(), String> {
    let valid_adr = id
        .strip_prefix("ADR-")
        .is_some_and(|digits| digits.len() == 4 && digits.bytes().all(|b| b.is_ascii_digit()));
    let valid_prd = id.strip_prefix("PRD-").is_some_and(|rest| {
        !rest.is_empty()
            && rest
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-')
    });
    if valid_adr || valid_prd {
        Ok(())
    } else {
        Err(format!(
            "invalid canonical id {id:?}; expected ADR-NNNN or PRD-<UPPERCASE-SLUG>"
        ))
    }
}

fn ensure_id_not_in_flight(
    repo_root: &Path,
    remote: &str,
    id: &str,
    claim_ref: &str,
) -> Result<(), String> {
    if remote_claim_oid(repo_root, remote, claim_ref)?.is_some() {
        return Err(format!(
            "canonical id {id} is already reserved at {claim_ref}"
        ));
    }
    if let Some(source) = remote_inflight_id_source(repo_root, remote, id)? {
        return Err(format!(
            "canonical id {id} is already in-flight at {source}"
        ));
    }
    Ok(())
}

fn remote_inflight_id_source(
    repo_root: &Path,
    remote: &str,
    id: &str,
) -> Result<Option<String>, String> {
    let output = git(repo_root)
        .args(["ls-remote", "--heads", remote])
        .output()
        .map_err(|error| format!("git ls-remote --heads failed to spawn: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-remote --heads failed for {remote}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("git ls-remote --heads output not UTF-8: {error}"))?;
    for line in stdout.lines() {
        let mut fields = line.split_whitespace();
        let _oid = fields.next();
        let Some(ref_name) = fields.next() else {
            continue;
        };
        if ref_name.starts_with(CLAIM_REF_PREFIX) || ref_name.starts_with(ID_RESERVATION_REF_PREFIX)
        {
            continue;
        }
        fetch_remote_head_for_scan(repo_root, remote, ref_name)?;
        if tree_mentions_reserved_id(repo_root, "FETCH_HEAD", id)? {
            return Ok(Some(ref_name.to_owned()));
        }
    }
    Ok(None)
}

fn fetch_remote_head_for_scan(
    repo_root: &Path,
    remote: &str,
    ref_name: &str,
) -> Result<(), String> {
    let output = git(repo_root)
        .args(["fetch", "--quiet", "--depth=1", remote, ref_name])
        .output()
        .map_err(|error| format!("git fetch failed to spawn: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git fetch failed for {remote} {ref_name}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn tree_mentions_reserved_id(repo_root: &Path, treeish: &str, id: &str) -> Result<bool, String> {
    if id.starts_with("ADR-") && tree_has_adr_filename(repo_root, treeish, id)? {
        return Ok(true);
    }
    if git_grep_fixed(
        repo_root,
        treeish,
        id,
        &[
            "docs/decisions",
            "specs",
            "docs/products",
            "docs/prds",
            "microservices",
        ],
    )? {
        return Ok(true);
    }
    Ok(false)
}

fn tree_has_adr_filename(repo_root: &Path, treeish: &str, id: &str) -> Result<bool, String> {
    let output = git(repo_root)
        .args(["ls-tree", "-r", "--name-only", treeish])
        .output()
        .map_err(|error| format!("git ls-tree failed to spawn: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-tree failed for {treeish}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("git ls-tree output not UTF-8: {error}"))?;
    Ok(stdout.lines().any(|path| {
        path.strip_prefix("docs/decisions/")
            .is_some_and(|name| name.starts_with(id) && name.ends_with(".md"))
    }))
}

fn git_grep_fixed(
    repo_root: &Path,
    treeish: &str,
    pattern: &str,
    pathspecs: &[&str],
) -> Result<bool, String> {
    let mut command = git(repo_root);
    command.args(["grep", "-F", "-q", pattern, treeish, "--"]);
    command.args(pathspecs);
    let output = command
        .output()
        .map_err(|error| format!("git grep failed to spawn: {error}"))?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(format!(
            "git grep failed for {treeish}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
    }
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
                (Action::ReserveId, true) => "would reserve",
                (Action::ReserveId, false) => "reserved",
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
