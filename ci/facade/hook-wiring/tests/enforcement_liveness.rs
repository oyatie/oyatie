// FRIC-012/G011 enforcement-liveness live-corpus gate. Runs the Buck-declared
// accounting-registry producer for the enforcement-liveness face, then asserts today's tracked
// hooks are all either wired in both project wiring files or marked as compatibility stubs.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use ci_hook_wiring::{Verdict, evaluate, evaluate_keyed};
use serde_json::{Value, json};

const PRODUCER_ENV: &str = "OYA_CI_ENFORCEMENT_LIVENESS_PRODUCER";
const CLAUDE_SETTINGS_ENV: &str = "OYA_CI_ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS";
const CODEX_HOOKS_ENV: &str = "OYA_CI_ENFORCEMENT_LIVENESS_CODEX_HOOKS";
const HOOKS_DIR_ENV: &str = "OYA_CI_ENFORCEMENT_LIVENESS_HOOKS_DIR";
const BUCK: &str = include_str!("../BUCK");
const CARGO_CONFIG: &str = include_str!("../../../../.cargo/config.toml");
const CARGO_PRODUCER_BINDING: &str = "cargo-test-binary:oya-cloud-ci-accounting-registry-app";
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct DeclaredCorpus {
    claude_settings: PathBuf,
    codex_hooks: PathBuf,
    hooks_dir: PathBuf,
}

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn load_produced_face(corpus: &DeclaredCorpus) -> Value {
    load_produced_face_with_tracked_paths(corpus, current_enforcement_tracked_paths(corpus))
}

fn load_produced_face_with_tracked_paths(
    corpus: &DeclaredCorpus,
    tracked_paths: Vec<String>,
) -> Value {
    let root = repo_root();
    let producer = required_declared_producer(&root).unwrap_or_else(|error| panic!("{error}"));
    let (scm_facts_dir, scm_facts) = write_enforcement_scm_facts(tracked_paths);
    let output = Command::new(&producer)
        .args([
            "--repo-root",
            root.to_str().expect("repo root utf-8"),
            "--scm-facts",
            scm_facts.to_str().expect("scm facts path utf-8"),
            "--enforcement-liveness-claude-settings",
            corpus
                .claude_settings
                .to_str()
                .expect("claude settings path utf-8"),
            "--enforcement-liveness-codex-hooks",
            corpus.codex_hooks.to_str().expect("codex hooks path utf-8"),
            "--enforcement-liveness-hooks-dir",
            corpus.hooks_dir.to_str().expect("hooks dir path utf-8"),
            "--stdout",
            "--face",
            "enforcement-liveness",
        ])
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "run declared enforcement-liveness producer {}: {e}",
                producer.display()
            )
        });
    let _ = std::fs::remove_dir_all(&scm_facts_dir);
    if !output.status.success() {
        panic!(
            "declared enforcement-liveness producer failed with status {:?}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("parse declared enforcement-liveness face: {e}"))
}

fn required_declared_producer(root: &Path) -> Result<PathBuf, String> {
    let declared = std::env::var_os(PRODUCER_ENV)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{PRODUCER_ENV} must name a declared producer resource"))?;
    resolve_declared_producer(root, &declared)
}

fn resolve_declared_producer(root: &Path, declared: &OsStr) -> Result<PathBuf, String> {
    let producer = ci_path_resolver_adapters::resolve_cargo_test_binary(root, declared)?;
    let metadata = std::fs::symlink_metadata(&producer).map_err(|error| {
        format!(
            "inspect declared {PRODUCER_ENV} producer {}: {error}",
            producer.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{PRODUCER_ENV} must bind a regular non-symlink file, got {}",
            producer.display()
        ));
    }
    Ok(producer)
}

fn assert_cargo_buck_producer_binding_parity() {
    let buck_binding = format!(
        "\"{PRODUCER_ENV}\": \"$(location //ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin)\""
    );
    assert!(
        BUCK.contains(&buck_binding),
        "Buck must keep the accounting-registry producer as its authoritative declared resource"
    );
    let cargo_binding =
        format!("{PRODUCER_ENV} = {{ value = \"{CARGO_PRODUCER_BINDING}\", force = false }}");
    assert!(
        CARGO_CONFIG.lines().any(|line| line == cargo_binding),
        "Cargo must mirror the Buck producer with a portable logical resource and force=false"
    );
}

fn declared_corpus() -> DeclaredCorpus {
    DeclaredCorpus {
        claude_settings: declared_file(CLAUDE_SETTINGS_ENV, "settings.json"),
        codex_hooks: declared_file(CODEX_HOOKS_ENV, "hooks.json"),
        hooks_dir: declared_dir(HOOKS_DIR_ENV),
    }
}

fn declared_file(env: &str, file_name: &str) -> PathBuf {
    let path = PathBuf::from(
        std::env::var(env).unwrap_or_else(|e| panic!("{env} must be a Buck-declared input: {e}")),
    );
    if path.is_file() {
        return path;
    }
    let nested = path.join(file_name);
    if nested.is_file() {
        return nested;
    }
    panic!(
        "{env} must resolve to `{file_name}` or a directory containing it, got {}",
        path.display()
    );
}

fn declared_dir(env: &str) -> PathBuf {
    let path = PathBuf::from(
        std::env::var(env).unwrap_or_else(|e| panic!("{env} must be a Buck-declared input: {e}")),
    );
    if path.is_dir() {
        return path;
    }
    panic!(
        "{env} must resolve to a Buck-declared directory, got {}",
        path.display()
    );
}

fn synthetic_declared_corpus() -> (PathBuf, DeclaredCorpus) {
    let root = unique_temp_path("oya-enforcement-liveness-declared-corpus");
    let hooks_dir = root.join("hooks");
    std::fs::create_dir_all(&hooks_dir).expect("create synthetic hooks dir");

    let claude_settings = root.join("settings.json");
    let codex_hooks = root.join("hooks.json");
    std::fs::write(
        &claude_settings,
        r#"{"hooks":{"PreToolUse":[{"hooks":[{"command":"./tools/hooks/hermetic-fixture.sh --from-claude"}]}]}}"#,
    )
    .expect("write synthetic claude settings");
    std::fs::write(
        &codex_hooks,
        r#"{"hooks":{"UserPromptSubmit":[{"command":"./tools/hooks/hermetic-fixture.sh --from-codex"}]}}"#,
    )
    .expect("write synthetic codex hooks");
    std::fs::write(
        hooks_dir.join("hermetic-fixture.sh"),
        "#!/usr/bin/env bash\necho hermetic fixture\n",
    )
    .expect("write synthetic hook");

    (
        root,
        DeclaredCorpus {
            claude_settings,
            codex_hooks,
            hooks_dir,
        },
    )
}

fn write_enforcement_scm_facts(tracked_paths: Vec<String>) -> (PathBuf, PathBuf) {
    let dir = unique_temp_path("oya-enforcement-liveness-scm-facts");
    std::fs::create_dir_all(&dir).expect("create temp scm facts dir");
    let path = dir.join("scm-facts.json");
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "schema": "oya-ci/scm-facts/v2",
            "tracked_paths": tracked_paths,
        }))
        .expect("serialize current enforcement scm facts")
            + "\n",
    )
    .expect("write current enforcement scm facts");
    (dir, path)
}

fn unique_temp_path(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{label}-{}-{nonce}-{sequence}", std::process::id()))
}

fn current_enforcement_tracked_paths(corpus: &DeclaredCorpus) -> Vec<String> {
    let mut paths = BTreeSet::new();
    paths.insert("tools/hooks/registration/claude-settings.json".to_owned());
    paths.insert("tools/hooks/registration/codex-hooks.json".to_owned());
    paths.extend(current_hook_paths(&corpus.hooks_dir));
    paths.into_iter().collect()
}

fn current_hook_paths(hooks_dir: &Path) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    for entry in std::fs::read_dir(hooks_dir).expect("read Buck-declared tools/hooks corpus") {
        let entry = entry.expect("read hook dir entry");
        let file_type = entry.file_type().expect("hook file type");
        if !file_type.is_file() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_str().expect("hook file name utf-8");
        if name.ends_with(".sh") {
            paths.insert(format!("tools/hooks/{name}"));
        }
    }
    paths
}

fn current_stub_paths(hooks_dir: &Path, hooks: &BTreeSet<String>) -> BTreeSet<String> {
    hooks
        .iter()
        .filter(|hook_path| {
            std::fs::read_to_string(hooks_dir.join(hook_file_name(hook_path)))
                .expect("read hook body")
                .contains("Compatibility stub only")
        })
        .cloned()
        .collect()
}

fn hook_file_name(hook_path: &str) -> &str {
    hook_path
        .strip_prefix("tools/hooks/")
        .expect("hook path must be tools/hooks relative")
}

fn current_hook_command_refs(wiring_file: &Path) -> BTreeSet<String> {
    let text = std::fs::read_to_string(wiring_file).expect("read Buck-declared wiring file");
    let value: Value = serde_json::from_str(&text).expect("parse wiring json");
    let mut refs = BTreeSet::new();
    collect_command_values(&value, &mut refs);
    refs
}

fn collect_command_values(value: &Value, refs: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                if key == "command"
                    && let Some(command) = nested.as_str()
                    && let Some(path) = normalize_hook_command(command)
                {
                    refs.insert(path);
                }
                collect_command_values(nested, refs);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_command_values(item, refs);
            }
        }
        _ => {}
    }
}

fn normalize_hook_command(command: &str) -> Option<String> {
    let first = command.split_whitespace().next()?;
    let path = first.strip_prefix("./").unwrap_or(first);
    if is_top_level_hook_script(path) {
        Some(path.to_owned())
    } else {
        None
    }
}

fn is_top_level_hook_script(path: &str) -> bool {
    let Some(name) = path.strip_prefix("tools/hooks/") else {
        return false;
    };
    !name.contains('/') && name.ends_with(".sh")
}

fn string_set_from_rows(rows: &[Value], row_type: &str, key: &str) -> BTreeSet<String> {
    rows.iter()
        .filter(|row| row["row_type"] == row_type)
        .map(|row| row[key].as_str().expect("row string field").to_owned())
        .collect()
}

#[test]
fn green_rows_allow_dual_wired_hooks_and_marked_compatibility_stubs() {
    let input = json!({
        "rows": [
            {
                "row_type": "hook",
                "hook_path": "tools/hooks/no-cargo-enforcer.sh",
                "wired_in_claude": true,
                "wired_in_codex": true,
                "stub_marked": false
            },
            {
                "row_type": "hook",
                "hook_path": "tools/hooks/session-start-context-inject.sh",
                "wired_in_claude": false,
                "wired_in_codex": false,
                "stub_marked": true
            },
            {
                "row_type": "command_reference",
                "wiring_file": "tools/hooks/registration/claude-settings.json",
                "command_path": "tools/hooks/no-cargo-enforcer.sh",
                "target_exists": true
            }
        ]
    });

    assert_eq!(evaluate(&input).verdict, Verdict::Green);
    assert!(evaluate_keyed(&input).is_empty());
}

#[test]
fn missing_rows_array_is_red_not_empty_green() {
    let input = json!({});

    let findings = evaluate_keyed(&input);
    assert!(findings.iter().any(|finding| {
        finding.code == "malformed_enforcement_liveness_face"
            && finding.key == "rows"
            && finding.remediation.contains("rows")
            && finding.remediation.contains("governance PR")
    }));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn malformed_rows_are_red_with_actionable_keys() {
    let input = json!({
        "rows": [
            {"hook_path": "tools/hooks/no-cargo-enforcer.sh"},
            {"row_type": "hook", "wired_in_claude": true, "wired_in_codex": true},
            {
                "row_type": "command_reference",
                "wiring_file": "tools/hooks/registration/codex-hooks.json",
                "target_exists": true
            }
        ]
    });

    let findings = evaluate_keyed(&input);
    let keyed: BTreeSet<_> = findings
        .iter()
        .map(|finding| (finding.code.as_str(), finding.key.as_str()))
        .collect();
    assert!(
        keyed.contains(&("malformed_enforcement_liveness_row", "rows[0].row_type")),
        "{keyed:?}"
    );
    assert!(
        keyed.contains(&("malformed_enforcement_liveness_row", "rows[1].hook_path")),
        "{keyed:?}"
    );
    assert!(
        keyed.contains(&("malformed_enforcement_liveness_row", "rows[2].command_path")),
        "{keyed:?}"
    );
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn malformed_boolean_fields_are_red_not_coerced_to_false() {
    let input = json!({
        "rows": [
            {
                "row_type": "hook",
                "hook_path": "tools/hooks/fully-wired-but-malformed.sh",
                "wired_in_claude": true,
                "wired_in_codex": true
            },
            {
                "row_type": "hook",
                "hook_path": "tools/hooks/marked-stub-but-malformed.sh",
                "stub_marked": true
            },
            {
                "row_type": "command_reference",
                "wiring_file": "tools/hooks/registration/codex-hooks.json",
                "command_path": "tools/hooks/fully-wired-but-malformed.sh"
            }
        ]
    });

    let findings = evaluate_keyed(&input);
    let keyed: BTreeSet<_> = findings
        .iter()
        .map(|finding| (finding.code.as_str(), finding.key.as_str()))
        .collect();
    for key in [
        "rows[0].stub_marked",
        "rows[1].wired_in_claude",
        "rows[1].wired_in_codex",
        "rows[2].target_exists",
    ] {
        assert!(
            keyed.contains(&("malformed_enforcement_liveness_row", key)),
            "expected malformed boolean field {key} in {keyed:?}"
        );
    }
    assert!(
        !keyed
            .iter()
            .any(|(code, _)| *code == "wired_hook_missing_file"),
        "missing/non-bool target_exists is malformed producer output, not a missing-file fact: {keyed:?}"
    );
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn live_hook_missing_dual_wiring_is_red() {
    let input = json!({
        "rows": [{
            "row_type": "hook",
            "hook_path": "tools/hooks/no-cargo-enforcer.sh",
            "wired_in_claude": false,
            "wired_in_codex": true,
            "stub_marked": false
        }]
    });

    let findings = evaluate_keyed(&input);
    assert!(findings.iter().any(|finding| {
        finding.code == "hook_unwired_without_stub_marker"
            && finding.key == "tools/hooks/no-cargo-enforcer.sh"
            && finding
                .remediation
                .contains("tools/hooks/registration/claude-settings.json")
            && finding
                .remediation
                .contains("tools/hooks/registration/codex-hooks.json")
            && finding.remediation.contains("governance PR")
    }));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn one_sided_live_hook_wiring_is_mirror_drift() {
    let input = json!({
        "rows": [{
            "row_type": "hook",
            "hook_path": "tools/hooks/stale-tool-suggester.sh",
            "wired_in_claude": true,
            "wired_in_codex": false,
            "stub_marked": false
        }]
    });

    let findings = evaluate_keyed(&input);
    assert!(findings.iter().any(|finding| {
        finding.code == "hook_wiring_mirror_drift"
            && finding.key == "tools/hooks/stale-tool-suggester.sh"
            && finding
                .remediation
                .contains("tools/hooks/registration/claude-settings.json")
            && finding
                .remediation
                .contains("tools/hooks/registration/codex-hooks.json")
            && finding.remediation.contains("governance PR")
    }));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn wired_hook_missing_file_is_red() {
    let input = json!({
        "rows": [{
            "row_type": "command_reference",
            "wiring_file": "tools/hooks/registration/codex-hooks.json",
            "command_path": "tools/hooks/deleted-enforcer.sh",
            "target_exists": false
        }]
    });

    let findings = evaluate_keyed(&input);
    assert_eq!(findings.len(), 1);
    let finding = findings.iter().next().unwrap();
    assert_eq!(finding.code, "wired_hook_missing_file");
    assert_eq!(
        finding.key,
        "tools/hooks/registration/codex-hooks.json:tools/hooks/deleted-enforcer.sh"
    );
    assert!(
        finding
            .remediation
            .contains("tools/hooks/registration/codex-hooks.json")
    );
    assert!(finding.remediation.contains("governance PR"));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn evaluate_is_bare_projection_of_evaluate_keyed() {
    let input = json!({
        "rows": [
            {
                "row_type": "hook",
                "hook_path": "tools/hooks/no-cargo-enforcer.sh",
                "wired_in_claude": false,
                "wired_in_codex": true,
                "stub_marked": false
            },
            {
                "row_type": "command_reference",
                "wiring_file": "tools/hooks/registration/codex-hooks.json",
                "command_path": "tools/hooks/deleted-enforcer.sh",
                "target_exists": false
            }
        ]
    });
    let projected: BTreeSet<String> = evaluate_keyed(&input)
        .into_iter()
        .map(|finding| finding.code)
        .collect();
    assert_eq!(evaluate(&input).violations, projected);
    for code in [
        "hook_unwired_without_stub_marker",
        "hook_wiring_mirror_drift",
        "wired_hook_missing_file",
    ] {
        assert!(projected.contains(code), "expected {code} in {projected:?}");
    }
}

#[test]
fn producer_consumes_declared_corpus_for_synthetic_hook_paths() {
    let (declared_root, corpus) = synthetic_declared_corpus();
    let non_file = declared_root.join("not-a-producer");
    std::fs::create_dir(&non_file).expect("create non-file producer fixture");
    let non_file_error = resolve_declared_producer(&declared_root, non_file.as_os_str())
        .expect_err("producer directory must fail closed");
    assert!(non_file_error.contains("regular non-symlink file"));
    let traversal_error = resolve_declared_producer(
        &declared_root,
        OsStr::new("cargo-test-binary:../poisoned-producer"),
    )
    .expect_err("logical producer traversal must fail closed");
    assert!(traversal_error.contains("invalid Cargo test binary resource name"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let regular = declared_root.join("regular-producer");
        let linked = declared_root.join("linked-producer");
        std::fs::write(&regular, b"fixture").expect("write producer fixture");
        symlink(&regular, &linked).expect("create producer symlink fixture");
        let symlink_error = resolve_declared_producer(&declared_root, linked.as_os_str())
            .expect_err("producer symlink must fail closed");
        assert!(symlink_error.contains("regular non-symlink file"));
    }

    let tracked_paths = vec![
        "tools/hooks/registration/claude-settings.json".to_owned(),
        "tools/hooks/registration/codex-hooks.json".to_owned(),
        "tools/hooks/hermetic-fixture.sh".to_owned(),
    ];

    let face = load_produced_face_with_tracked_paths(&corpus, tracked_paths);
    let rows = face["rows"].as_array().expect("enforcement-liveness rows");
    let hook_row = rows
        .iter()
        .find(|row| {
            row["row_type"] == "hook" && row["hook_path"] == "tools/hooks/hermetic-fixture.sh"
        })
        .expect("synthetic hook row");

    assert_eq!(hook_row["wired_in_claude"].as_bool(), Some(true));
    assert_eq!(hook_row["wired_in_codex"].as_bool(), Some(true));
    assert_eq!(hook_row["stub_marked"].as_bool(), Some(false));
    assert!(
        evaluate_keyed(&face).is_empty(),
        "synthetic declared corpus should be green when producer consumes the declared paths: {face}"
    );
    assert_eq!(evaluate(&face).verdict, Verdict::Green);

    std::fs::remove_dir_all(declared_root).expect("remove synthetic declared corpus");
}

#[test]
fn enforcement_liveness_face_reports_current_tree_green() {
    assert_cargo_buck_producer_binding_parity();
    let corpus = declared_corpus();
    let expected_hooks = current_hook_paths(&corpus.hooks_dir);
    let expected_stubs = current_stub_paths(&corpus.hooks_dir, &expected_hooks);
    let expected_command_refs = current_hook_command_refs(&corpus.claude_settings).len()
        + current_hook_command_refs(&corpus.codex_hooks).len();

    let face = load_produced_face(&corpus);
    let rows = face["rows"].as_array().expect("enforcement-liveness rows");
    let hook_rows = string_set_from_rows(rows, "hook", "hook_path");
    let command_rows = rows
        .iter()
        .filter(|row| row["row_type"] == "command_reference")
        .count();
    let stub_rows: BTreeSet<String> = rows
        .iter()
        .filter(|row| row["row_type"] == "hook" && row["stub_marked"].as_bool() == Some(true))
        .map(|row| row["hook_path"].as_str().expect("hook_path").to_owned())
        .collect();

    eprintln!(
        "ENFORCEMENT-LIVENESS live corpus: hooks={} command_refs={command_rows} stubs={}",
        hook_rows.len(),
        stub_rows.len()
    );

    assert_eq!(
        hook_rows, expected_hooks,
        "enforcement-liveness must census today's tools/hooks/*.sh corpus, not a stale fixture"
    );
    assert_eq!(
        command_rows, expected_command_refs,
        "enforcement-liveness must parse today's Claude+Codex hook command references"
    );
    assert_eq!(
        stub_rows, expected_stubs,
        "enforcement-liveness must derive compatibility stubs from today's hook bodies"
    );
    assert!(evaluate_keyed(&face).is_empty());
    assert_eq!(evaluate(&face).verdict, Verdict::Green);
}
