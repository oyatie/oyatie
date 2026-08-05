// ADR-0560 cache-wiring conformance gate: live-corpus self-test over the REAL
// policy + license + overlays + canary workflow (the slice-1 instalment of the
// ADR-0556-named cache-policy-conformance successor; asserting the FULL live CI
// cache configuration against the policy remains that gate's scope).
//
// Proves mechanically, on every PR:
//   1. the dark-wiring guarantee — while specs/cache-warm-license.json is
//      unlicensed, EVERY class resolves bypass (today's builds are untouched);
//   2. the cold-required floor — the four ADR-0556 one-way cold classes resolve
//      bypass even under a licensed fixture (pinned here as a ratchet: dropping
//      one from the policy DATA goes RED and requires superseding ADR-0556);
//   3. the kill-switch works — flipping the license fixture flips warm classes
//      between bypass and their classified modes, and never the cold ones;
//   4. the overlays parse, select the cache execution platform, set the posture
//      their name claims, and carry NO keyed identity material;
//   5. the root .buckconfig stays clean of any RE/cache section;
//   6. the canary workflow exists, is scheduled, restores no actions/cache, and
//      wires the cold proof (assert-cold) + structured record.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::{
    collections::HashSet,
    path::{Component, Path, PathBuf},
};

use ci_build_cache_policy as app;
use serde_json::{Value, json};
use serde_yaml::Value as YamlValue;

const CANARY_WORKFLOW_PATH: &str = ".github/workflows/cache-integrity-canary.yml";
const CANARY_SCHEDULE_WORKFLOW_PATH: &str = ".github/workflows/cache-integrity-canary-schedule.yml";
const REQUIRED_WORKFLOW_PATH: &str = ".github/workflows/oya-ci-required.yml";
const COLD_REQUIRED_FLOOR: [&str; 4] = [
    "release-production-image",
    "integrity-canary",
    "untrusted-author-presubmit",
    "provenance-attestation",
];

fn repo_root() -> PathBuf {
    let cwd = std::env::current_dir().expect("current_dir");
    app::repo_root_from(&cwd).expect("failed to locate repo root from test current_dir")
}

fn licensed_fixture() -> Value {
    json!({ "warm_reads_licensed": true, "reason": "conformance fixture", "licensed_by_canary_run": "fixture" })
}

fn invocation_record_fixture(
    cache_hit_rate: f64,
    action_hits: u64,
    local: u64,
    remote: u64,
) -> Value {
    json!({
        "cache_hit_rate": cache_hit_rate,
        "run_action_cache_count": action_hits,
        "run_local_count": local,
        "run_remote_count": remote,
        "run_skipped_count": 0,
        "cache_upload_attempt_count": 0,
        "cache_upload_count": 0,
        "dep_file_upload_attempt_count": 0,
        "dep_file_upload_count": 0,
        "run_remote_dep_file_cache_count": 0,
        "re_upload_bytes": 0,
        "re_download_bytes": if action_hits > 0 { 1024 } else { 0 },
        "exit_result_name": "SUCCESS",
        "run_command_failure_count": 0,
        "errors": [],
        "daemon_connection_failure": false,
        "last_snapshot": {
            "re_action_cache_started": action_hits,
            "re_action_cache_finished_successfully": action_hits,
            "re_action_cache_finished_with_error": 0,
            "re_upload_bytes": 0,
            "re_uploads_started": 0,
            "re_uploads_finished_successfully": 0,
            "re_uploads_finished_with_error": 0,
            "re_download_bytes": if action_hits > 0 { 1024 } else { 0 },
            "re_downloads_started": action_hits,
            "re_downloads_finished_successfully": action_hits,
            "re_downloads_finished_with_error": 0,
            "re_executes_started": 0,
            "re_executes_finished_successfully": 0,
            "re_executes_finished_with_error": 0,
            "re_write_action_results_started": 0,
            "re_write_action_results_finished_successfully": 0,
            "re_write_action_results_finished_with_error": 0,
            "re_get_digest_expirations_started": 0,
            "re_get_digest_expirations_finished_successfully": 0,
            "re_get_digest_expirations_finished_with_error": 0,
            "re_materializes_started": 0,
            "re_materializes_finished_successfully": 0,
            "re_materializes_finished_with_error": 0
        },
    })
}

#[derive(Debug)]
enum GlobToken {
    Literal(char),
    AnyCharacter,
    Star,
    CharacterClass {
        negated: bool,
        ranges: Vec<(char, char)>,
    },
}

fn parse_glob_segment(pattern: &str) -> Result<Vec<GlobToken>, String> {
    let characters: Vec<char> = pattern.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < characters.len() {
        match characters[index] {
            '*' => {
                tokens.push(GlobToken::Star);
                index += 1;
            }
            '?' => {
                tokens.push(GlobToken::AnyCharacter);
                index += 1;
            }
            '\\' => {
                index += 1;
                let Some(character) = characters.get(index).copied() else {
                    return Err(format!("trailing glob escape in {pattern:?}"));
                };
                tokens.push(GlobToken::Literal(character));
                index += 1;
            }
            '[' => {
                index += 1;
                let negated = matches!(characters.get(index), Some('!' | '^'));
                if negated {
                    index += 1;
                }
                let mut ranges = Vec::new();
                if characters.get(index) == Some(&']') {
                    ranges.push((']', ']'));
                    index += 1;
                }
                while index < characters.len() && characters[index] != ']' {
                    let start = if characters[index] == '\\' {
                        index += 1;
                        characters.get(index).copied().ok_or_else(|| {
                            format!("trailing character-class escape in {pattern:?}")
                        })?
                    } else {
                        characters[index]
                    };
                    index += 1;

                    if characters.get(index) == Some(&'-')
                        && characters.get(index + 1).is_some_and(|value| *value != ']')
                    {
                        index += 1;
                        let end = if characters[index] == '\\' {
                            index += 1;
                            characters.get(index).copied().ok_or_else(|| {
                                format!("trailing character-class range escape in {pattern:?}")
                            })?
                        } else {
                            characters[index]
                        };
                        index += 1;
                        if start > end {
                            return Err(format!("reversed character-class range in {pattern:?}"));
                        }
                        ranges.push((start, end));
                    } else {
                        ranges.push((start, start));
                    }
                }
                if characters.get(index) != Some(&']') || ranges.is_empty() {
                    return Err(format!(
                        "unterminated or empty character class in {pattern:?}"
                    ));
                }
                index += 1;
                tokens.push(GlobToken::CharacterClass { negated, ranges });
            }
            character => {
                tokens.push(GlobToken::Literal(character));
                index += 1;
            }
        }
    }

    Ok(tokens)
}

fn add_star_epsilon_closure(tokens: &[GlobToken], states: &mut HashSet<usize>) {
    loop {
        let additions: Vec<usize> = states
            .iter()
            .filter_map(|position| {
                matches!(tokens.get(*position), Some(GlobToken::Star)).then_some(position + 1)
            })
            .filter(|position| !states.contains(position))
            .collect();
        if additions.is_empty() {
            return;
        }
        states.extend(additions);
    }
}

fn glob_segment_matches(pattern: &str, target: &str) -> Result<bool, String> {
    let tokens = parse_glob_segment(pattern)?;
    let mut states = HashSet::from([0]);
    add_star_epsilon_closure(&tokens, &mut states);

    for character in target.chars() {
        let mut next = HashSet::new();
        for position in &states {
            match tokens.get(*position) {
                Some(GlobToken::Star) => {
                    next.insert(*position);
                }
                Some(GlobToken::Literal(expected)) if *expected == character => {
                    next.insert(position + 1);
                }
                Some(GlobToken::AnyCharacter) => {
                    next.insert(position + 1);
                }
                Some(GlobToken::CharacterClass { negated, ranges }) => {
                    let listed = ranges
                        .iter()
                        .any(|(start, end)| *start <= character && character <= *end);
                    if listed != *negated {
                        next.insert(position + 1);
                    }
                }
                _ => {}
            }
        }
        states = next;
        add_star_epsilon_closure(&tokens, &mut states);
    }

    Ok(states.contains(&tokens.len()))
}

fn cache_path_candidate_archives_checkout(candidate: &str) -> Result<bool, String> {
    let mut relative = strip_workspace_expression(candidate)
        .map(|suffix| suffix.trim_start_matches('/'))
        .unwrap_or(candidate)
        .trim_end_matches('/');
    while let Some(stripped) = relative.strip_prefix("./") {
        relative = stripped;
    }

    if relative.contains("${{") || relative.contains("}}") {
        return Err(format!(
            "unresolved dynamic expression controls the cache path: {candidate:?}"
        ));
    }

    if relative.is_empty() || relative == "." {
        return Ok(true);
    }

    let first_component = relative
        .split('/')
        .find(|component| !component.is_empty() && *component != ".")
        .unwrap_or(relative);
    glob_segment_matches(first_component, "buck-out")
}

fn strip_workspace_expression(candidate: &str) -> Option<&str> {
    let expression = candidate.strip_prefix("${{")?;
    let end = expression.find("}}")?;
    let name: String = expression[..end]
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect();
    let suffix = &expression[end + 2..];
    (name.eq_ignore_ascii_case("github.workspace")
        && (suffix.is_empty() || suffix.starts_with('/') || suffix.starts_with('\\')))
    .then_some(suffix)
}

fn cache_path_archives_checkout(raw_path: &str) -> Result<bool, String> {
    let normalized = raw_path.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err("empty include pattern".to_owned());
    }
    let bytes = normalized.as_bytes();
    let windows_drive_path = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if normalized.starts_with('/') || normalized.starts_with('\\') || windows_drive_path {
        return Err(format!(
            "absolute actions/cache paths cannot prove exclusion of runner-local buck-out: {raw_path:?}"
        ));
    }
    if normalized.starts_with('~') {
        const SAFE_TILDE_ROOTS: [&str; 2] = ["~/.rustup/toolchains", "~/.rustup/update-hashes"];
        let proven_safe = SAFE_TILDE_ROOTS.iter().any(|root| {
            normalized == *root
                || normalized
                    .strip_prefix(root)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        });
        if !proven_safe {
            return Err(format!(
                "unproven tilde-expanded actions/cache path can reach the runner checkout: {raw_path:?}"
            ));
        }
    }
    if normalized
        .split(['/', '\\'])
        .any(|component| component == "..")
    {
        return Err(format!(
            "relative parent segments are not supported by @actions/glob: {raw_path:?}"
        ));
    }

    // On Linux/macOS a backslash escapes glob metacharacters; on Windows it can
    // also be a separator. Reject if either supported interpretation reaches the
    // checkout's runner-local `buck-out` root.
    cache_path_candidate_archives_checkout(&normalized).and_then(|archives| {
        if archives || !normalized.contains('\\') {
            Ok(archives)
        } else {
            cache_path_candidate_archives_checkout(&normalized.replace('\\', "/"))
        }
    })
}

fn included_cache_pattern(raw_path: &str) -> Result<Option<&str>, String> {
    let mut pattern = raw_path.trim();
    if pattern.is_empty() || pattern.starts_with('#') {
        return Ok(None);
    }
    let mut excluded = false;
    while let Some(remainder) = pattern.strip_prefix('!') {
        excluded = !excluded;
        pattern = remainder.trim();
    }
    if excluded {
        cache_path_archives_checkout(pattern)?;
        return Ok(None);
    }
    if pattern.is_empty() {
        return Err(format!("empty actions/cache include pattern {raw_path:?}"));
    }
    Ok(Some(pattern))
}

fn action_steps<'a>(doc: &'a YamlValue) -> Vec<(&'a str, &'a [YamlValue])> {
    let mut scopes = Vec::new();
    if let Some(jobs) = doc.get("jobs").and_then(YamlValue::as_mapping) {
        for (job_name, job) in jobs {
            if let Some(steps) = job.get("steps").and_then(YamlValue::as_sequence) {
                scopes.push((
                    job_name.as_str().unwrap_or("<non-string-job>"),
                    steps.as_slice(),
                ));
            }
        }
    }
    if let Some(steps) = doc
        .get("runs")
        .and_then(|runs| runs.get("steps"))
        .and_then(YamlValue::as_sequence)
    {
        scopes.push(("<composite-action>", steps.as_slice()));
    }
    scopes
}

fn local_action_file(repo_root: &Path, action_name: &str) -> Result<Option<PathBuf>, String> {
    let Some(relative) = action_name
        .strip_prefix("./")
        .or_else(|| action_name.strip_prefix(".\\"))
    else {
        return Ok(None);
    };
    if action_name.contains('\\') {
        return Err(format!(
            "backslash-bearing local action paths are host-ambiguous; use portable `./` slash syntax: {action_name:?}"
        ));
    }
    let relative = Path::new(relative);
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "local action path escapes the repository: {action_name:?}"
        ));
    }
    let action_dir = repo_root.join(relative);
    for file_name in ["action.yml", "action.yaml"] {
        let candidate = action_dir.join(file_name);
        if let Ok(metadata) = candidate.symlink_metadata() {
            if metadata.file_type().is_file() {
                return Ok(Some(candidate));
            }
            return Err(format!(
                "local action metadata is not a regular file: {}",
                candidate.display()
            ));
        }
    }
    Err(format!(
        "local action {action_name:?} has no action.yml or action.yaml"
    ))
}

fn local_reusable_workflow_file(repo_root: &Path, reference: &str) -> Result<PathBuf, String> {
    let Some(relative) = reference.strip_prefix("./") else {
        return Err(format!(
            "external job-level reusable workflow is not proven cache-safe: {reference:?}"
        ));
    };
    let relative = Path::new(relative);
    if !relative.starts_with(".github/workflows")
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || !matches!(
            relative
                .extension()
                .and_then(|extension| extension.to_str()),
            Some("yml" | "yaml")
        )
    {
        return Err(format!(
            "invalid same-repository reusable workflow reference: {reference:?}"
        ));
    }
    let workflow_file = repo_root.join(relative);
    match workflow_file.symlink_metadata() {
        Ok(metadata) if metadata.file_type().is_file() => Ok(workflow_file),
        Ok(_) => Err(format!(
            "reusable workflow is not a regular file: {}",
            workflow_file.display()
        )),
        Err(error) => Err(format!(
            "cannot resolve reusable workflow {}: {error}",
            workflow_file.display()
        )),
    }
}

fn inspect_local_yaml_document(
    repo_root: &Path,
    path: PathBuf,
    visited_local_documents: &mut HashSet<PathBuf>,
    violations: &mut Vec<String>,
) {
    if !visited_local_documents.insert(path.clone()) {
        return;
    }
    match std::fs::read_to_string(&path) {
        Ok(text) => match serde_yaml::from_str(&text) {
            Ok(doc) => inspect_actions_cache_steps(
                Some(repo_root),
                &path.display().to_string(),
                &doc,
                visited_local_documents,
                violations,
            ),
            Err(error) => violations.push(format!(
                "{}: malformed local workflow/action YAML: {error}",
                path.display()
            )),
        },
        Err(error) => violations.push(format!(
            "{}: cannot read local workflow/action: {error}",
            path.display()
        )),
    }
}

fn inspect_actions_cache_steps(
    repo_root: Option<&Path>,
    source: &str,
    doc: &YamlValue,
    visited_local_documents: &mut HashSet<PathBuf>,
    violations: &mut Vec<String>,
) {
    if let Some(jobs) = doc.get("jobs").and_then(YamlValue::as_mapping) {
        for (job_name, job) in jobs {
            let scope = job_name.as_str().unwrap_or("<non-string-job>");
            let Some(reference_value) = job.get("uses") else {
                continue;
            };
            let Some(reference) = reference_value.as_str() else {
                violations.push(format!(
                    "{source}:{scope}: non-string reusable workflow reference {reference_value:?}"
                ));
                continue;
            };
            let Some(repo_root) = repo_root else {
                violations.push(format!(
                    "{source}:{scope}: cannot verify reusable workflow {reference:?} without a repository root"
                ));
                continue;
            };
            match local_reusable_workflow_file(repo_root, reference) {
                Ok(path) => inspect_local_yaml_document(
                    repo_root,
                    path,
                    visited_local_documents,
                    violations,
                ),
                Err(error) => violations.push(format!("{source}:{scope}: {error}")),
            }
        }
    }

    for (scope, steps) in action_steps(doc) {
        for step in steps {
            let Some(action) = step.get("uses").and_then(YamlValue::as_str) else {
                continue;
            };
            let local_action = action.starts_with("./") || action.starts_with(".\\");
            let action_name = if local_action {
                action
            } else {
                action.split('@').next().unwrap_or(action)
            };
            let action_name_lower = if local_action {
                action_name.to_ascii_lowercase()
            } else {
                action_name
                    .split(['/', '\\'])
                    .filter(|segment| !segment.is_empty())
                    .collect::<Vec<_>>()
                    .join("/")
                    .to_ascii_lowercase()
            };
            if matches!(
                action_name_lower.as_str(),
                "actions/cache" | "actions/cache/restore" | "actions/cache/save"
            ) {
                let step_name = step
                    .get("name")
                    .and_then(YamlValue::as_str)
                    .unwrap_or("<unnamed-step>");
                let Some(path) = step.get("with").and_then(|with| with.get("path")) else {
                    continue;
                };
                let mut raw_paths = Vec::new();
                match path {
                    YamlValue::String(value) => raw_paths.extend(value.lines()),
                    YamlValue::Sequence(values) => {
                        for value in values {
                            match value.as_str() {
                                Some(value) => raw_paths.extend(value.lines()),
                                None => violations.push(format!(
                                    "{source}:{scope}/{step_name}: non-string actions/cache path {value:?}"
                                )),
                            }
                        }
                    }
                    value => violations.push(format!(
                        "{source}:{scope}/{step_name}: non-string actions/cache path {value:?}"
                    )),
                }

                for raw_path in raw_paths {
                    match included_cache_pattern(raw_path) {
                        Ok(None) => {}
                        Ok(Some(include)) => match cache_path_archives_checkout(include) {
                            Ok(true) => violations.push(format!(
                                "{source}:{scope}/{step_name}: {action_name} archives forbidden path {raw_path:?}"
                            )),
                            Ok(false) => {}
                            Err(error) => violations.push(format!(
                                "{source}:{scope}/{step_name}: malformed actions/cache path {raw_path:?}: {error}"
                            )),
                        },
                        Err(error) => violations.push(format!(
                            "{source}:{scope}/{step_name}: malformed actions/cache path: {error}"
                        )),
                    }
                }
                continue;
            }

            let Some(repo_root) = repo_root else {
                continue;
            };
            match local_action_file(repo_root, action_name) {
                Ok(Some(action_file)) => inspect_local_yaml_document(
                    repo_root,
                    action_file,
                    visited_local_documents,
                    violations,
                ),
                Ok(None) => {}
                Err(error) => violations.push(format!("{source}:{scope}: {error}")),
            }
        }
    }
}

fn actions_cache_buck_out_violations(
    repo_root: Option<&Path>,
    source: &str,
    workflow: &str,
) -> Vec<String> {
    let doc: YamlValue = serde_yaml::from_str(workflow).expect("parse workflow YAML");
    let mut violations = Vec::new();
    let mut visited_local_documents = HashSet::new();
    if let Some(repo_root) = repo_root {
        let source_path = Path::new(source);
        if !source_path.is_absolute()
            && source_path
                .components()
                .all(|component| matches!(component, Component::Normal(_)))
        {
            let candidate = repo_root.join(source_path);
            if candidate
                .symlink_metadata()
                .is_ok_and(|metadata| metadata.file_type().is_file())
            {
                visited_local_documents.insert(candidate);
            }
        }
    }
    inspect_actions_cache_steps(
        repo_root,
        source,
        &doc,
        &mut visited_local_documents,
        &mut violations,
    );
    violations
}

#[test]
fn policy_and_license_parse_and_the_default_is_fail_closed() {
    let root = repo_root();
    let policy = app::load_policy(&root).expect("load real cache-warmth policy");
    let license = app::load_license(&root).expect("load real cache-warm license");

    let default = &policy["default_for_unlisted_classes"];
    assert_eq!(default["warmth"], "cold", "unlisted default must be cold");
    assert_eq!(default["cache_read"], false);
    assert_eq!(default["cache_write"], false);

    assert_eq!(
        app::canary_class(&policy),
        Some("integrity-canary"),
        "the policy must name its canary trust anchor"
    );
    assert!(license["warm_reads_licensed"].is_boolean());
}

#[test]
fn dark_wiring_guarantee_under_the_real_license() {
    let root = repo_root();
    let policy = app::load_policy(&root).expect("policy");
    let license = app::load_license(&root).expect("license");
    let licensed = license["warm_reads_licensed"].as_bool().unwrap();

    let classes: Vec<String> = policy["build_classes"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .chain(std::iter::once("not-a-classified-class".to_string()))
        .collect();

    for class in &classes {
        let r = app::resolve(&policy, &license, class).expect("resolve");
        if !licensed {
            assert_eq!(
                r.mode,
                app::CacheMode::Bypass,
                "DARK-WIRING VIOLATION: class `{class}` resolved `{}` while \
                 warm_reads_licensed=false — no lane may touch the cache today",
                r.mode
            );
        } else {
            // Once the license flips (CAS bring-up + first GREEN canary), warm
            // classes must match their classified posture and cold stays bypass.
            let entry = policy["build_classes"].get(class);
            let warm = entry
                .map(|e| e["warmth"] == "warm" && e["cache_read"] == true)
                .unwrap_or(false)
                && class != "integrity-canary";
            assert_eq!(r.mode != app::CacheMode::Bypass, warm, "class `{class}`");
        }
    }
}

#[test]
fn cold_required_floor_holds_even_under_a_licensed_fixture() {
    let root = repo_root();
    let policy = app::load_policy(&root).expect("policy");
    let license = licensed_fixture();
    for class in COLD_REQUIRED_FLOOR {
        let entry = policy["build_classes"]
            .get(class)
            .unwrap_or_else(|| panic!("ADR-0556 one-way cold class `{class}` missing from policy"));
        assert_eq!(entry["warmth"], "cold", "`{class}` left the cold floor");
        assert_eq!(entry["cache_read"], false, "`{class}` gained cache_read");
        assert_eq!(entry["cache_write"], false, "`{class}` gained cache_write");
        let r = app::resolve(&policy, &license, class).expect("resolve");
        assert_eq!(
            r.mode,
            app::CacheMode::Bypass,
            "one-way floor: `{class}` must bypass even when warm is licensed"
        );
    }
}

#[test]
fn kill_switch_flips_warm_classes_and_only_warm_classes() {
    let root = repo_root();
    let policy = app::load_policy(&root).expect("policy");
    let unlicensed = json!({ "warm_reads_licensed": false, "reason": "fixture" });
    let licensed = licensed_fixture();

    let mut saw_warm = false;
    for (class, entry) in policy["build_classes"].as_object().unwrap() {
        let off = app::resolve(&policy, &unlicensed, class).unwrap().mode;
        let on = app::resolve(&policy, &licensed, class).unwrap().mode;
        assert_eq!(off, app::CacheMode::Bypass);
        if entry["warmth"] == "warm" && entry["cache_read"] == true && class != "integrity-canary" {
            saw_warm = true;
            let expected = if entry["cache_write"] == true {
                app::CacheMode::WarmReadWrite
            } else {
                app::CacheMode::WarmReadOnly
            };
            assert_eq!(on, expected, "licensed warm class `{class}`");
        } else {
            assert_eq!(
                on,
                app::CacheMode::Bypass,
                "cold class `{class}` must stay bypass"
            );
        }
    }
    assert!(
        saw_warm,
        "policy carries no warm-eligible class — fixture rot?"
    );
}

#[test]
fn overlays_parse_select_the_cache_platform_and_carry_no_identity() {
    let root = repo_root();
    for (path, uploads, endpoint_marker) in [
        (app::OVERLAY_RW_PATH, "true", "nativelink-cas-writer"),
        (app::OVERLAY_RO_PATH, "false", "nativelink-cas-reader"),
    ] {
        let text =
            std::fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let cfg = app::parse_buckconfig(&text);

        let build = cfg
            .get("build")
            .unwrap_or_else(|| panic!("{path}: no [build]"));
        assert_eq!(
            build["execution_platforms"], "toolchains//cache:cache-platform",
            "{path} must select the cache execution platform"
        );

        let oya = cfg
            .get("oya_cache")
            .unwrap_or_else(|| panic!("{path}: no [oya_cache]"));
        assert_eq!(oya["remote_cache_enabled"], "true", "{path}");
        assert_eq!(oya["allow_cache_uploads"], uploads, "{path}");

        let default_upload = cfg
            .get("buck2")
            .and_then(|section| section.get("default_allow_cache_upload"))
            .map(String::as_str);
        if uploads == "true" {
            assert_eq!(
                default_upload,
                Some("true"),
                "{path}: locally executed writer actions must opt into Buck2 cache uploads"
            );
        } else {
            assert_ne!(
                default_upload,
                Some("true"),
                "{path}: the reader overlay must never enable cache uploads"
            );
        }

        let re = cfg
            .get("buck2_re_client")
            .unwrap_or_else(|| panic!("{path}: no [buck2_re_client]"));
        assert_eq!(re["tls"], "true", "{path}: keyed transport is TLS-only");
        for key in ["engine_address", "cas_address", "action_cache_address"] {
            assert!(
                re[key].contains(endpoint_marker),
                "{path}: {key} must point at the {endpoint_marker} endpoint, got {}",
                re[key]
            );
        }
        assert!(
            !re.contains_key("tls_client_cert"),
            "{path}: the keyed identity must come from secret-mounted env at emit time, \
             never from the checked-in overlay"
        );
        assert!(
            !text.contains("PRIVATE KEY") && !text.to_lowercase().contains("api-key"),
            "{path}: secret material in a checked-in overlay"
        );
    }
}

#[test]
/// The SIBLING of root_buckconfig_stays_dark, which guards only `.buckconfig`.
///
/// `.buckconfig.local` is the ONLY mechanism that can wire the remote cache: buck2
/// resolves `[buck2_re_client]` into DaemonStartupConfig from project config files
/// ONLY, so `--config` / `--config-file` are inert for that section (measured). A
/// COMMITTED `.buckconfig.local` carrying warm-cache-rw content would therefore make
/// every build in the repo remote-cache-enabled with uploads on, bypassing the
/// resolver and the /specs/cache-warm-license.json kill-switch entirely — and would
/// poison the integrity canary, whose cold build depends on running with no overlay.
///
/// Deliberate asymmetry: `.buckconfig.d/` is NOT forbidden. Committed fragments there
/// are the FAIL-CLOSED way to ship real config, because a missing `--config-file`
/// path silently succeeds (BUILD SUCCEEDED, exit 0) while a committed fragment is
/// always read. This test bans the machine-local file, not the committed-fragment door.
fn buckconfig_local_is_ignored_and_untracked() {
    let root = repo_root();

    let gitignore = std::fs::read_to_string(root.join(".gitignore")).expect(
        "read .gitignore — it is the only thing keeping a warm-cache overlay uncommittable",
    );
    assert!(
        gitignore
            .lines()
            .any(|l| l.trim() == "/.buckconfig.local" || l.trim() == ".buckconfig.local"),
        "UNIGNORED CACHE OVERLAY: .gitignore must ignore .buckconfig.local. It is the only file \
         that can wire [buck2_re_client], so an unignored copy is one `git add -A` away from \
         enabling remote cache + uploads for every build in the repo, bypassing the resolver \
         and the warm-license kill-switch (ADR-0560 D6)"
    );

    let tracked = std::process::Command::new("git")
        .args(["ls-files", "--", ".buckconfig.local"])
        .current_dir(&root)
        .output()
        .expect("run git ls-files");
    assert!(
        String::from_utf8_lossy(&tracked.stdout).trim().is_empty(),
        "TRACKED CACHE OVERLAY: .buckconfig.local is committed. Remove it — its contents apply \
         to every buck2 invocation in this checkout, warm or cold, licensed or not"
    );
}

#[test]
fn root_buckconfig_stays_dark() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(".buckconfig")).expect("read root .buckconfig");
    let cfg = app::parse_buckconfig(&text);
    assert!(
        !cfg.contains_key("buck2_re_client"),
        "root .buckconfig grew a [buck2_re_client] section — cache wiring must stay opt-in \
         (ADR-0560 dark-wiring invariant)"
    );
    assert!(
        !cfg.contains_key("oya_cache"),
        "root .buckconfig grew an [oya_cache] section — cache wiring must stay opt-in"
    );
    assert_eq!(
        cfg["build"]["execution_platforms"], "prelude//platforms:default",
        "the default execution platform must stay the prelude default"
    );
}

#[test]
fn canary_workflow_is_scheduled_cold_and_wires_the_proof() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(CANARY_WORKFLOW_PATH)).unwrap_or_else(|e| {
        panic!(
            "read {CANARY_WORKFLOW_PATH}: {e} — the canary MUST ship \
                                    with the CAS wiring (ADR-0556 D2: no canary, no warm)"
        )
    });
    let schedule = std::fs::read_to_string(root.join(CANARY_SCHEDULE_WORKFLOW_PATH)).unwrap();
    assert!(
        schedule.contains("schedule:"),
        "canary must be cron-scheduled (ADR-0556 D4.3)"
    );
    assert!(
        !text.contains("actions/cache@") && !schedule.contains("actions/cache@"),
        "FROM-EMPTY VIOLATION: the canary workflow restores a cache — the proof is circular \
         (ADR-0556 D5 cold-must-stay)"
    );

    // THE WARM SIDE. Every assertion below this point in the original test covered the
    // COLD step only (--unstable-write-invocation-record + assert-cold), both of which
    // the cold build already satisfied — so the gate LOOKED like it guarded the canary
    // while never checking the half that could lie. canary_verdict compares
    // target->output-digest pairs, so a probe that fetched nothing and rebuilt locally
    // produces byte-identical digests, full overlap, zero divergence => GREEN, and that
    // GREEN licenses warm reads fleet-wide.
    assert!(
        text.contains("--isolation-dir canary-warm-probe")
            && text.contains("--unstable-write-invocation-record /tmp/canary-warm-record.json"),
        "WARM PROBE UNPROVEN: the probe build must write its OWN invocation record, or its \
         cache participation cannot be checked and a zero-fetch local rebuild emits GREEN \
         (ADR-0556 D2)"
    );
    assert!(
        text.contains("--warm /tmp/canary-warm-manifest.json")
            && text.contains("--warm-record /tmp/canary-warm-record.json"),
        "WARM MANIFEST ADMITTED WITHOUT PROOF: canary-verdict must receive --warm-record \
         alongside --warm so the probe's participation gates the comparison (ADR-0556 D2)"
    );
    assert!(
        text.contains("--unstable-write-invocation-record"),
        "canary must capture the structured invocation record"
    );
    assert!(
        text.contains("assert-cold"),
        "canary must mechanically prove zero cache participation (assert-cold)"
    );
    assert!(
        text.contains("integrity-canary"),
        "canary must run under the integrity-canary build class"
    );
    assert!(
        text.contains("canary-verdict"),
        "canary must emit the structured verdict artifact"
    );
}

#[test]
fn required_workflow_cache_hit_report_is_binding() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(REQUIRED_WORKFLOW_PATH)).unwrap_or_else(|e| {
        panic!(
            "read {REQUIRED_WORKFLOW_PATH}: {e} — the required CI workflow must ship the \
             cache-hit report guard"
        )
    });
    let telemetry_step = text
        .split("- name: Cache-hit telemetry + warm-mode guard (ADR-0560)")
        .nth(1)
        .and_then(|tail| {
            tail.split("- name: Upload cache-hit telemetry artifact")
                .next()
        })
        .expect("required workflow must contain the cache-hit telemetry guard step");
    assert!(
        telemetry_step.contains("--unstable-write-invocation-record")
            || text.contains(
                "--unstable-write-invocation-record /tmp/buck2-lane-invocation-record.json"
            ),
        "the buck2 lane must capture a structured invocation record before reporting cache health"
    );
    assert!(
        telemetry_step.contains(" report --record /tmp/buck2-lane-invocation-record.json")
            && telemetry_step.contains("--out /tmp/cache-hit-report.json"),
        "the cache-hit report must be generated from the structured invocation record"
    );
    assert!(
        telemetry_step.contains(" assert-warm --record /tmp/buck2-lane-invocation-record.json"),
        "warm/bypass cache participation must be asserted in the binding telemetry step"
    );
    assert!(
        !telemetry_step.contains("continue-on-error"),
        "the cache-hit telemetry guard must be binding; missing counters or 0% warm hits cannot pass"
    );

    // DELETED: three assertions that matched the `Upload cache-hit telemetry artifact` step's own
    // YAML literals (`name: cache-hit-report-buck2-lane`, `path: /tmp/cache-hit-report.json`,
    // `if-no-files-found: error`) and claimed they made the report "binding". They asserted
    // nothing. The upload step is `if: failure()`, so on a green lane it never runs; on a red
    // lane the job is already failing, so `if-no-files-found: error` cannot change any verdict
    // ever. `cache-hit-report-buck2-lane` also has ZERO consumers — it appears only in the
    // workflow that produces it and in this test — so the artifact going missing breaks nothing.
    // The step's own comment in oya-ci-required.yml says it outright: "`assert-warm` above is the
    // enforcing check; this upload never was." Those three asserts could only fail if somebody
    // edited the YAML, which converted "we have a gate for that" into false assurance.
    //
    // WHERE THE REAL ASSURANCE LIVES — do not re-add a YAML-literal check here:
    //   * that the report is PRODUCED and a cold/0%-hit warm lane goes RED: the binding
    //     `Cache-hit telemetry + warm-mode guard (ADR-0560)` step, which is `if: always()` and
    //     carries no `continue-on-error`. Its wiring is asserted above in THIS test; its
    //     behaviour is asserted directly against the kernel by
    //     `cache_hit_guard_behavior_covers_bypass_warm_and_malformed_records` below.
    //   * that a stale/missing invocation record cannot pass: `app::assert_warm_cache_participation`,
    //     exercised over bypass/warm/zero-hit/malformed records in that same test.
    // Artifact retention and upload success are runtime-only properties of a failure-path
    // diagnostic. A pure test cannot observe them, and nothing depends on them.
}

#[test]
fn workflows_use_the_local_config_controller_and_keep_the_cold_canary_absent() {
    let root = repo_root();
    let required = std::fs::read_to_string(root.join(REQUIRED_WORKFLOW_PATH)).unwrap();
    let required_words = required
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("\\ ", "");
    assert!(
        required_words.contains("-- run --build-class \"${CACHE_BUILD_CLASS}\" --mode-out /tmp/cache-mode -- buck2 test //ci/..."),
        "required CI must execute Buck2 as the controller child"
    );
    assert!(!required.contains("CACHE_MODE=bypass"));

    let canary = std::fs::read_to_string(root.join(CANARY_WORKFLOW_PATH)).unwrap();
    let cold = canary
        .split("- name: Cold from-empty build of the pinned target set")
        .nth(1)
        .and_then(|tail| tail.split("- name: Prove zero cache participation").next())
        .expect("cold canary step");
    assert!(!cold.contains("run --warm-probe"));
    assert!(!cold.contains(".buckconfig.local"));
    assert!(canary.contains(" -- run --workflow-mode"));
    assert!(canary.contains("--mode-out /tmp/canary-cache-mode -- buck2"));
    assert!(!canary.contains("--config-file infra/ci/buckconfig"));
    assert!(!canary.contains("--config \"buck2_re_client"));
}

#[test]
fn workflows_exchange_oidc_only_for_trusted_jobs_and_never_use_static_cert_secrets() {
    let root = repo_root();
    let controller =
        std::fs::read_to_string(root.join("ci/facade/build-cache-policy/src/main.rs")).unwrap();
    for binding in [
        "ACTIONS_ID_TOKEN_REQUEST_URL",
        "/v1/auth/jwt/login",
        "/v1/{pki_mount}/issue/{pki_role}",
        "identity role, PKI mount, PKI role, and URI SAN do not match a trusted tuple",
        "CACHE_SERVER_CA_ENV",
        "write_private_file",
        ".connect_timeout(Duration::from_secs(10))",
        ".timeout(Duration::from_secs(30))",
        "oidc_authorization.set_sensitive(true)",
        "prove_identity_boundary",
        "rustls::Error::AlertReceived",
        "expected a typed peer alert before HTTP/2/gRPC",
        "Capabilities probe requires negotiated HTTP/2 and HTTP 200",
        "Capabilities response must contain exactly one grpc-status trailer",
        "assert_writer_seed_record",
    ] {
        assert!(
            controller.contains(binding),
            "missing identity exchange binding {binding}"
        );
    }
    let stop = controller.find("let stop = kill_buck2").unwrap();
    let remove = controller
        .find("let remove = app::remove_local_buckconfig")
        .unwrap();
    let combine = controller.find("match (stop, remove)").unwrap();
    assert!(
        stop < remove && remove < combine,
        "cache config removal must be attempted before cleanup errors propagate"
    );
    let required = std::fs::read_to_string(root.join(REQUIRED_WORKFLOW_PATH)).unwrap();
    let writer = required
        .split("  cache-writer-identity:")
        .nth(1)
        .and_then(|tail| tail.split("  gate-affected-target-set:").next())
        .expect("trusted writer identity job");
    assert!(writer.contains("github.event_name == 'push'") && writer.contains("refs/heads/dev"));
    assert!(writer.contains("vars.OYA_CAS_IDENTITY_PROOF_ENABLED == 'true'"));
    assert!(writer.contains("id-token: write"));
    assert!(writer.contains("uses: ./.github/workflows/cache-integrity-canary.yml"));
    assert!(writer.contains("writer_seed: true"));
    assert!(!writer.contains("run:"), "writer must add no inline shell");
    let fan_in = required
        .split("  oya-ci-required:")
        .nth(1)
        .expect("required fan-in");
    assert!(!fan_in.contains("needs.cache-writer-identity"));
    assert!(!fan_in.contains("- cache-writer-identity"));

    let untrusted = required
        .split("  buck2:")
        .nth(1)
        .and_then(|tail| tail.split("  cache-writer-identity:").next())
        .expect("untrusted buck2 job");
    assert!(untrusted.contains("CACHE_BUILD_CLASS: untrusted-author-presubmit"));
    assert!(!untrusted.contains("id-token: write"));
    assert!(!untrusted.contains("issue-identity"));

    let canary = std::fs::read_to_string(root.join(CANARY_WORKFLOW_PATH)).unwrap();
    let schedule = std::fs::read_to_string(root.join(CANARY_SCHEDULE_WORKFLOW_PATH)).unwrap();
    assert!(
        !canary.contains("id-token:"),
        "the reusable executor must inherit permissions from its caller so the cold call cannot elevate"
    );
    assert!(
        !schedule.contains("run:"),
        "the privilege-separating scheduler must only call the reviewed Rust-backed executor"
    );
    let cold = schedule
        .split("  cold:")
        .nth(1)
        .and_then(|tail| tail.split("  reader-identity:").next())
        .expect("contents-only cold job");
    assert!(cold.contains("permissions:") && cold.contains("contents: read"));
    assert!(!cold.contains("id-token:"));
    assert!(cold.contains("uses: ./.github/workflows/cache-integrity-canary.yml"));
    let reader = schedule
        .split("  reader-identity:")
        .nth(1)
        .expect("activation-gated reader job");
    assert!(reader.contains("vars.OYA_CAS_IDENTITY_PROOF_ENABLED == 'true'"));
    assert!(reader.contains("github.ref == 'refs/heads/dev'"));
    assert!(reader.contains("github.event_name == 'workflow_dispatch'"));
    assert!(reader.contains("needs.cold.result == 'success'"));
    assert!(reader.contains("needs.cold.outputs.warm_licensed == 'true'"));
    assert!(reader.contains("id-token: write"));
    assert!(reader.contains("actions: read"));
    assert!(reader.contains("reader_probe: true"));
    assert!(reader.contains("writer_run_id:"));
    assert!(canary.contains("workflow_call:"));
    assert!(!canary.contains("\n  schedule:"));
    assert!(!canary.contains("\n  workflow_dispatch:"));
    assert!(canary.contains("writer_seed:"));
    assert!(canary.contains("reader_probe:"));
    assert!(canary.contains("writer_run_id:"));
    assert!(
        canary.contains("--workflow-mode \"${{ inputs.writer_seed && 'writer' || 'reader' }}\"")
    );
    assert!(schedule.contains("vars.OYA_CAS_IDENTITY_PROOF_ENABLED == 'true'"));
    assert!(canary.contains("OYA_CACHE_TLS_SERVER_CA_CERT: /etc/nativelink/ca/ca.crt"));
    assert!(canary.contains("prelicense_probe:"));
    assert!(canary.contains("timeout-minutes: 120"));
    assert!(canary.contains("--prelicense-probe"));
    assert!(canary.contains("OYA_CACHE_TLS_CLIENT_CERT: /tmp/oya-cache-client.pem"));
    assert!(canary.contains("OYA_CACHE_TLS_CA_CERTS: /tmp/oya-cache-server-ca.pem"));
    assert!(!canary.contains("cas-identity-boundary-"));
    assert!(!canary.contains("${{ runner.temp }}"));
    assert!(!canary.contains("name: Exchange GitHub OIDC"));
    assert!(!canary.contains("name: Remove short-lived cache identity"));
    assert!(canary.contains("name: Download cold proof from the zero-OIDC invocation"));
    assert!(canary.contains("cache-integrity-cold"));
    assert!(canary.contains("name: cache-writer-${{ github.sha }}"));
    assert!(canary.contains("github-token: ${{ github.token }}"));
    assert!(canary.contains("repository: ${{ github.repository }}"));
    assert!(canary.contains("run-id: ${{ inputs.writer_run_id }}"));
    assert!(canary.contains("warm-proof --role"));
    assert!(canary.contains("/tmp/canary-writer-report.json"));
    assert!(canary.contains("/tmp/canary-writer-receipt.json"));
    assert!(canary.contains("--report-out \"${{ inputs.writer_seed && '/tmp/canary-writer-report.json' || '/tmp/canary-reader-report.json' }}\""));
    assert!(canary.contains("/tmp/canary-warm-record.json"));
    assert!(canary.contains("--writer-manifest"));
    assert!(canary.contains("--writer-run-id \"$WRITER_RUN_ID\""));
    assert!(canary.contains("/tmp/writer-proof/canary-writer-receipt.json"));
    let writer_upload = canary
        .split("      - name: Upload validated writer proof")
        .nth(1)
        .and_then(|tail| tail.split("      - name: Canary verdict").next())
        .expect("writer proof upload step");
    assert!(!writer_upload.contains("always()"));
    let reader_upload = canary
        .split("      - name: Upload canary artifacts")
        .nth(1)
        .expect("reader/canary artifact upload step");
    assert!(reader_upload.contains("/tmp/canary-warm-record.json"));
    assert!(reader_upload.contains("/tmp/canary-reader-report.json"));
    assert!(schedule.contains("writer_run_id:"));
    assert!(schedule.contains("default: \"\""));
    assert!(
        canary.contains("vars.OYA_CAS_IDENTITY_PROOF_ENABLED != 'true'"),
        "activation-off cold runs must execute the INACTIVE verdict and remain RED"
    );
    assert!(controller.contains("fixed_identity_options"));
    assert!(controller.contains("remove_identity_files"));
    assert!(controller.contains("github-cas-reader-integrity-canary"));
    assert!(controller.contains("github-cas-writer-dev-push"));
    for workflow in [&required, &canary, &schedule] {
        assert!(!workflow.contains("OYA_CACHE_WRITER_TLS_CLIENT_CERT_PATH"));
        assert!(!workflow.contains("OYA_CACHE_READER_TLS_CLIENT_CERT_PATH"));
        assert!(!workflow.contains("OYA_CACHE_TLS_CA_CERTS_PATH"));
    }
}

#[test]
fn live_postgres_coverage_remains_split_across_required_same_pod_jobs() {
    let root = repo_root();
    let required = std::fs::read_to_string(root.join(REQUIRED_WORKFLOW_PATH)).unwrap();
    assert_eq!(
        required.matches("  gate-live-postgres-adapters:").count(),
        1
    );
    assert_eq!(required.matches("  gate-live-postgres-facades:").count(), 1);
    assert!(required.contains("needs.gate-live-postgres-adapters"));
    assert!(required.contains("needs.gate-live-postgres-facades"));
    assert!(!required.contains("  gate-live-postgres:"));
    assert!(required.contains("buck2 test — durable adapters"));
    assert!(required.contains("buck2 test — durable facades"));
    assert!(required.contains("      - gate-live-postgres-adapters # #901:"));
    assert!(required.contains("      - gate-live-postgres-facades  # #901:"));
}

#[test]
fn required_workflow_never_archives_buck_out() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(REQUIRED_WORKFLOW_PATH))
        .unwrap_or_else(|e| panic!("read {REQUIRED_WORKFLOW_PATH}: {e}"));
    let violations = actions_cache_buck_out_violations(Some(&root), REQUIRED_WORKFLOW_PATH, &text);

    assert!(
        violations.is_empty(),
        "{}: {violations:?}",
        concat!(
            "UNSAFE RUNNER SNAPSHOT: the required workflow archives `buck-out`. Buck2's local ",
            "state and materialized outputs are runner-local and the archive can exhaust an ",
            "ephemeral runner during extraction before any binding test executes (ADR-0554 D10)"
        ),
    );
    assert!(
        !text.contains("runner-disk-reclaim-buck2.json"),
        "DEAD ARTIFACT: the retired owned-runner reclaim producer has no output to upload; remove its failure-only artifact path (ADR-0554 D10)"
    );
}

#[test]
fn buck_out_archive_guard_rejects_yaml_path_variants_and_renamed_steps() {
    for path_yaml in [
        "path: ./buck-out",
        "path: buck-out/v2/cache",
        "path: |\n              ~/.rustup\n              ./buck-out/v2/cache",
        "path:\n              - ~/.rustup\n              - buck-out/v2/cache",
        "path: ${{ github.workspace }}/buck-out",
        "path: .",
        "path: ${{ github.workspace }}/",
        "path: ${{ github.workspace }}/**",
        "path: ./**",
        "path: '**'",
        "path: '?uck-out'",
        "path: '[b]uck-out'",
        "path: 'buck-*'",
        "path: '!!buck-out'",
        "path: '! !buck-out'",
        "path: 'toolchain/../buck-out'",
        "path: \"${{ 'buck-out' }}\"",
        "path: \"${{ format('buck-{0}', 'out') }}\"",
        "path: '${{ github.workspace }}/${{ inputs.cache_path }}'",
        "path: 'safe/${{ inputs.cache_path }}'",
        "path: '${{ github.workspace }}/safe/${{ inputs.cache_path }}'",
        "path: '${{ github.workspace }}suffix'",
        "path: '!${{ inputs.cache_path }}'",
        "path: /home/runner/_work/oyatie/oyatie/buck-out",
        "path: /home/runner/_work/oyatie/oyatie",
        "path: /home/runner/_work/**",
        "path: ~/_work/oyatie/oyatie/buck-out",
        "path: ~/_work/**",
        "path: ~/**",
        "path: 'D:\\a\\oyatie\\oyatie\\buck-out'",
        "path: 'C:buck-out'",
        "path: '\\\\server\\share\\oyatie\\buck-out'",
    ] {
        let fixture = format!(
            "jobs:\n  renamed-job:\n    steps:\n      - name: Innocuous renamed step\n        uses: actions/cache/restore@pinned\n        with:\n          key: unrelated-key\n          {path_yaml}\n"
        );
        assert!(
            !actions_cache_buck_out_violations(None, "<fixture>", &fixture).is_empty(),
            "guard accepted forbidden YAML variant:\n{fixture}"
        );
    }

    let mixed_case = "jobs:\n  gate:\n    steps:\n      - uses: AcTiOnS/CaChE@pinned\n        with:\n          path: ./buck-out\n";
    assert!(
        !actions_cache_buck_out_violations(None, "<fixture>", mixed_case).is_empty(),
        "action repository casing must not bypass the guard"
    );

    for action in [
        "actions\\cache@pinned",
        "actions//cache@pinned",
        "actions/cache/@pinned",
        "actions\\cache\\restore@pinned",
        "actions//cache//save@pinned",
        "actions/cache/restore/@pinned",
        "actions/cache/save/@pinned",
    ] {
        let fixture = format!(
            "jobs:\n  gate:\n    steps:\n      - uses: {action}\n        with:\n          path: buck-out\n          key: fixture\n"
        );
        assert!(
            !actions_cache_buck_out_violations(None, "<fixture>", &fixture).is_empty(),
            "runner-equivalent external cache action reference bypassed the guard: {action:?}"
        );
    }

    let safe = "jobs:\n  gate:\n    steps:\n      - uses: actions/cache@pinned\n        with:\n          path: |\n            ~/.rustup/toolchains\n            ~/.rustup/update-hashes\n            toolchain-*\n            rustup-*\n            [rt]ustup-cache\n            tool chain-*\n            ${{ github.workspace }}/toolchain-*\n            !buck-out\n            # buck-out\n";
    assert!(
        actions_cache_buck_out_violations(None, "<fixture>", safe).is_empty(),
        "toolchain-only actions/cache must remain allowed"
    );
}

#[test]
fn buck_out_archive_guard_follows_local_composite_actions() {
    let fixture_root = std::env::temp_dir().join(format!(
        "oya-cache-composite-fixture-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("unnamed")
    ));
    let action_dir = fixture_root.join(".github/actions/cache-wrapper");
    let _ = std::fs::remove_dir_all(&fixture_root);
    std::fs::create_dir_all(&action_dir).expect("create local composite fixture");
    std::fs::write(
        action_dir.join("action.yml"),
        "name: cache wrapper\ndescription: fixture cache wrapper\ninputs:\n  cache_path:\n    description: cache path\n    required: true\nruns:\n  using: composite\n  steps:\n    - uses: ACTIONS/CACHE/SAVE@pinned\n      with:\n        path: '${{ inputs.cache_path }}'\n        key: fixture\n",
    )
    .expect("write local composite fixture");
    let workflow = "jobs:\n  gate:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.github/actions/cache-wrapper\n        with:\n          cache_path: buck-out\n";
    let violations = actions_cache_buck_out_violations(Some(&fixture_root), "<fixture>", workflow);
    assert!(
        !violations.is_empty(),
        "local composite action must not hide a forbidden checkout archive"
    );

    let benign_prefix_dir = fixture_root.join(".github/actions/cache");
    let at_named_dir = fixture_root.join(".github/actions/cache@wrapper");
    std::fs::create_dir_all(&benign_prefix_dir).expect("create benign prefix action fixture");
    std::fs::create_dir_all(&at_named_dir).expect("create at-named action fixture");
    std::fs::write(
        benign_prefix_dir.join("action.yml"),
        "name: benign prefix\ndescription: must not mask at-named sibling\nruns:\n  using: composite\n  steps:\n    - run: echo safe\n      shell: bash\n",
    )
    .expect("write benign prefix action fixture");
    std::fs::write(
        at_named_dir.join("action.yml"),
        "name: unsafe at-named wrapper\ndescription: runner resolves the at sign literally\nruns:\n  using: composite\n  steps:\n    - uses: actions/cache@pinned\n      with:\n        path: buck-out\n        key: fixture\n",
    )
    .expect("write at-named action fixture");
    let at_named_workflow = "jobs:\n  gate:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.github/actions/cache@wrapper\n";
    let at_named_violations =
        actions_cache_buck_out_violations(Some(&fixture_root), "<fixture>", at_named_workflow);
    assert!(
        !at_named_violations.is_empty(),
        "local action references containing `@` must resolve the literal runner path, not a benign truncated prefix"
    );

    let windows_at_named_workflow = "jobs:\n  gate:\n    runs-on: windows-latest\n    steps:\n      - uses: .\\.github\\actions\\cache@wrapper\n";
    let windows_at_named_violations = actions_cache_buck_out_violations(
        Some(&fixture_root),
        "<fixture>",
        windows_at_named_workflow,
    );
    assert!(
        !windows_at_named_violations.is_empty(),
        "Windows-form local action references must fail closed because their host interpretation is ambiguous"
    );

    let normalized_benign_dir = fixture_root.join(".github/actions/cache/wrapper");
    let literal_backslash_unsafe_dir = fixture_root.join(".github/actions/cache\\wrapper");
    std::fs::create_dir_all(&normalized_benign_dir)
        .expect("create normalized benign action fixture");
    std::fs::create_dir_all(&literal_backslash_unsafe_dir)
        .expect("create literal-backslash unsafe action fixture");
    std::fs::write(
        normalized_benign_dir.join("action.yml"),
        "name: normalized benign action\ndescription: must not mask the POSIX literal-backslash sibling\nruns:\n  using: composite\n  steps:\n    - run: echo safe\n      shell: bash\n",
    )
    .expect("write normalized benign action fixture");
    std::fs::write(
        literal_backslash_unsafe_dir.join("action.yml"),
        "name: literal-backslash unsafe action\ndescription: Linux runner preserves the interior backslash\nruns:\n  using: composite\n  steps:\n    - uses: actions/cache@pinned\n      with:\n        path: buck-out\n        key: fixture\n",
    )
    .expect("write literal-backslash unsafe action fixture");
    let cross_host_workflow = "jobs:\n  gate:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.github/actions/cache\\wrapper\n";
    let cross_host_violations =
        actions_cache_buck_out_violations(Some(&fixture_root), "<fixture>", cross_host_workflow);
    std::fs::remove_dir_all(&fixture_root).expect("remove local composite fixture");
    assert!(
        !cross_host_violations.is_empty(),
        "host-sensitive interior backslashes must fail closed rather than inspect only the normalized benign action"
    );
}

#[test]
fn buck_out_archive_guard_follows_local_reusable_workflows() {
    let fixture_root = std::env::temp_dir().join(format!(
        "oya-cache-reusable-workflow-fixture-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("unnamed")
    ));
    let workflow_dir = fixture_root.join(".github/workflows");
    let _ = std::fs::remove_dir_all(&fixture_root);
    std::fs::create_dir_all(&workflow_dir).expect("create reusable workflow fixture");
    std::fs::write(
        workflow_dir.join("cache-wrapper.yml"),
        "name: cache wrapper\non:\n  workflow_call:\n    inputs:\n      cache_path:\n        required: true\n        type: string\njobs:\n  cache:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/cache@pinned\n        with:\n          path: '${{ inputs.cache_path }}'\n          key: fixture\n",
    )
    .expect("write reusable workflow fixture");
    let workflow = "name: caller\non: pull_request\njobs:\n  delegated-gate:\n    uses: ./.github/workflows/cache-wrapper.yml\n    with:\n      cache_path: buck-out\n";
    let violations = actions_cache_buck_out_violations(Some(&fixture_root), "<fixture>", workflow);
    assert!(
        !violations.is_empty(),
        "same-repository reusable workflow must not hide a forbidden checkout archive"
    );

    std::fs::write(
        workflow_dir.join("safe.yml"),
        "name: safe cache wrapper\non:\n  workflow_call:\njobs:\n  safe-cache:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/cache@pinned\n        with:\n          path: toolchain-*\n          key: fixture\n",
    )
    .expect("write safe reusable workflow fixture");
    let safe_workflow = "name: caller\non: pull_request\njobs:\n  delegated-gate:\n    uses: ./.github/workflows/safe.yml\n";
    assert!(
        actions_cache_buck_out_violations(Some(&fixture_root), "<fixture>", safe_workflow)
            .is_empty(),
        "safe same-repository reusable workflows must not produce false positives"
    );

    let external = "jobs:\n  delegated-gate:\n    uses: owner/repo/.github/workflows/cache.yml@0123456789abcdef\n";
    assert!(
        !actions_cache_buck_out_violations(Some(Path::new(".")), "<fixture>", external).is_empty(),
        "uninspected external reusable workflows must fail closed"
    );
    std::fs::remove_dir_all(&fixture_root).expect("remove reusable workflow fixture");
}

#[test]
fn buck_out_archive_guard_terminates_local_document_cycles() {
    let fixture_root = std::env::temp_dir().join(format!(
        "oya-cache-document-cycle-fixture-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("unnamed")
    ));
    let workflow_dir = fixture_root.join(".github/workflows");
    let _ = std::fs::remove_dir_all(&fixture_root);
    std::fs::create_dir_all(&workflow_dir).expect("create cycle fixture");
    std::fs::write(
        workflow_dir.join("cycle-a.yml"),
        "jobs:\n  delegated:\n    uses: ./.github/workflows/cycle-b.yml\n",
    )
    .expect("write first cycle fixture");
    std::fs::write(
        workflow_dir.join("cycle-b.yml"),
        "jobs:\n  delegated:\n    uses: ./.github/workflows/cycle-a.yml\n",
    )
    .expect("write second cycle fixture");
    let workflow = "jobs:\n  delegated:\n    uses: ./.github/workflows/cycle-a.yml\n";
    let violations = actions_cache_buck_out_violations(Some(&fixture_root), "<fixture>", workflow);
    std::fs::remove_dir_all(&fixture_root).expect("remove cycle fixture");
    assert!(
        violations.is_empty(),
        "cycle protection must terminate structural traversal: {violations:?}"
    );
}

#[test]
fn cache_hit_guard_behavior_covers_bypass_warm_and_malformed_records() {
    let bypass_zero = invocation_record_fixture(0.0, 0, 12, 0);
    assert!(
        app::assert_warm_cache_participation(&bypass_zero, "gate-fleet-shared-graph", "bypass")
            .is_ok(),
        "current bypass/cold posture must stay allowed even with zero cache hits"
    );

    let warm_hit = invocation_record_fixture(0.25, 3, 9, 0);
    assert!(
        app::assert_warm_cache_participation(&warm_hit, "gate-fleet-shared-graph", "warm-rw")
            .is_ok(),
        "warm mode with a positive hit rate and positive action-cache count must pass"
    );

    let warm_zero = invocation_record_fixture(0.0, 0, 12, 0);
    let findings =
        app::assert_warm_cache_participation(&warm_zero, "gate-fleet-shared-graph", "warm-rw")
            .unwrap_err();
    assert!(
        findings.iter().any(|f| f.contains("0% hit rate"))
            && findings
                .iter()
                .any(|f| f.contains("run_action_cache_count=0")),
        "warm mode with 0% hits must be RED: {findings:?}"
    );

    let malformed = json!({ "exit_result_name": "SUCCESS" });
    let findings =
        app::assert_warm_cache_participation(&malformed, "gate-fleet-shared-graph", "warm-rw")
            .unwrap_err();
    assert!(
        findings
            .iter()
            .any(|f| f.contains("record-shape violation")),
        "missing or renamed cache counters must be RED: {findings:?}"
    );
}

#[test]
fn bundled_canary_targets_stay_inside_the_binding_gate_cone() {
    let policy = app::canary_policy().expect("bundled canary policy");
    let targets = policy["pinned_targets"].as_array().unwrap();
    assert!(!targets.is_empty());
    for target in targets {
        let t = target.as_str().unwrap();
        assert!(
            t.starts_with("//"),
            "pinned target `{t}` must be a repo-anchored pattern"
        );

        // PATH LIVENESS — the assertion this test was missing, and the reason it never fired.
        // `!targets.is_empty()` above proves the ARRAY has entries; `starts_with("//")` proves each
        // is SHAPED like a pattern. Neither proves a pattern names anything that exists. This block
        // pinned `//cloud/cloud-ci/...` long after the gate-fleet move VACATED that tree, so the
        // pattern resolved to ZERO targets — and the canary that anchors the entire warm-cache/RE
        // trust chain (ADR-0556 D2 licensing, ADR-0612 D5 "no RE-covering canary, no RE") would
        // have built nothing and reported success having verified nothing.
        //
        // Checked as a PATH here, deliberately, not by shelling `buck2 targets`: this stays a pure
        // test, and a vacated root is exactly the reorg-move failure mode that got us. It does not
        // claim the pattern resolves to >=1 buck2 TARGET — the canary job's own from-empty build is
        // what proves that, and it cannot even start against a root that does not exist.
        // The FULL package prefix, not just the first segment. Checking only the first segment is
        // itself the bug this test exists to catch: `//cloud/cloud-ci/...` has root `cloud`, and
        // `cloud/` still exists as a legacy root, so a first-segment check passes while the
        // `cloud-ci` subtree it actually names is gone. Verified by restoring the vacated pattern
        // and watching a first-segment version of this assertion stay GREEN.
        let prefix = t
            .trim_start_matches('/')
            .split("/...")
            .next()
            .unwrap_or_default()
            .split(':')
            .next()
            .unwrap_or_default()
            .trim_end_matches('/');
        assert!(
            !prefix.is_empty(),
            "pinned target `{t}` has no resolvable package prefix"
        );
        let prefix_path = repo_root().join(prefix);
        assert!(
            prefix_path.is_dir(),
            "pinned canary target `{t}` names a package prefix that does not exist: {}. A move \
             vacated it and nothing noticed — the canary would build an EMPTY target set and pass. \
             Re-point the pattern at the tree the gates actually live in, or drop it.",
            prefix_path.display()
        );
    }
}
