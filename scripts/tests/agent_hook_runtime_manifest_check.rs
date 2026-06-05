#![allow(dead_code)]

#[path = "../ci/assert-agent-hook-runtime-manifest.rs"]
mod checker;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("read {}: {}", path, error))
}

fn loaded_inputs() -> checker::LoadedInputs {
    let manifest = read_repo_file("specs/agent-hook-runtime-manifest.json");
    let runtime_hooks = checker::parse_runtime_hooks(&manifest).expect("runtime hooks parse");
    let mut hooks = BTreeMap::new();
    for hook in runtime_hooks {
        hooks.insert(hook.path.clone(), read_repo_file(&hook.path));
    }
    checker::LoadedInputs {
        manifest,
        codex_config: read_repo_file(".codex/hooks.json"),
        gemini_config: read_repo_file(".gemini/settings.json"),
        claude_config: read_repo_file(".claude/settings.json"),
        hooks,
    }
}

#[test]
fn checked_in_manifest_configs_and_hooks_pass() {
    let evaluation = checker::evaluate(Path::new(&repo_root())).expect("evaluation runs");
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.runtime_hook_count, 3);
    assert_eq!(evaluation.config_reference_count, 6);
    assert!(evaluation.retired_surface_count >= 10);
}

#[test]
fn parser_scopes_runtime_hooks_without_retired_surfaces() {
    let manifest = read_repo_file("specs/agent-hook-runtime-manifest.json");
    let hooks = checker::parse_runtime_hooks(&manifest).expect("runtime hooks parse");
    let paths = hooks
        .iter()
        .map(|hook| hook.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        vec![
            "tools/hooks/no-cargo-enforcer.sh",
            "tools/hooks/vacuous-green-gate-detect.sh",
            "tools/hooks/injection-content-scanner.sh"
        ]
    );
}

#[test]
fn parser_scopes_first_class_config_paths_per_object() {
    let manifest = read_repo_file("specs/agent-hook-runtime-manifest.json");
    let paths = checker::first_class_config_paths(&manifest);
    assert!(paths.contains(".codex/hooks.json"), "{:?}", paths);
    assert!(paths.contains(".gemini/settings.json"), "{:?}", paths);

    let mutated = manifest.replacen(
        "\"path\": \".gemini/settings.json\",\n      \"first_class\": true",
        "\"path\": \".gemini/settings.json\",\n      \"first_class\": false",
        1,
    );
    let failures = checker::manifest_contract_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "manifest_missing_first_class_config:.gemini/settings.json"),
        "{:?}",
        failures
    );
}

#[test]
fn rejects_unmanifested_config_command() {
    let mut inputs = loaded_inputs();
    inputs.codex_config = inputs.codex_config.replacen(
        "tools/hooks/no-cargo-enforcer.sh",
        "tools/hooks/stale-tool-suggester.sh",
        1,
    );
    let evaluation = checker::evaluate_loaded(Path::new(&repo_root()), &inputs).unwrap();
    assert!(
        evaluation
            .failures
            .iter()
            .any(|failure| failure.contains("config_references_unmanifested_hook")),
        "{:?}",
        evaluation.failures
    );
}

#[test]
fn rejects_manifest_hook_that_is_not_referenced_by_configs() {
    let mut inputs = loaded_inputs();
    inputs.manifest = inputs.manifest.replacen(
        "\"runtime_hooks\": [",
        concat!(
            "\"runtime_hooks\": [\n",
            "    {\n",
            "      \"path\": \"tools/hooks/new-read-only-hook.sh\",\n",
            "      \"purpose\": \"Fixture-only unreferenced hook.\",\n",
            "      \"events\": {\"codex\": [\"PreToolUse:Bash\"]},\n",
            "      \"reads\": [],\n",
            "      \"writes\": [],\n",
            "      \"network\": false\n",
            "    },"
        ),
        1,
    );
    inputs.hooks.insert(
        "tools/hooks/new-read-only-hook.sh".to_owned(),
        "#!/usr/bin/env bash\nexit 0\n".to_owned(),
    );
    let evaluation = checker::evaluate_loaded(Path::new(&repo_root()), &inputs).unwrap();
    assert!(
        evaluation.failures.iter().any(|failure| failure
            == "manifest_runtime_hook_unreferenced:tools/hooks/new-read-only-hook.sh"),
        "{:?}",
        evaluation.failures
    );
}

#[test]
fn rejects_claude_project_runtime_hooks() {
    let mut inputs = loaded_inputs();
    inputs.claude_config =
        inputs
            .claude_config
            .replacen("\"sandbox\": {", "\"hooks\": {},\n  \"sandbox\": {", 1);
    let evaluation = checker::evaluate_loaded(Path::new(&repo_root()), &inputs).unwrap();
    assert!(
        evaluation
            .failures
            .iter()
            .any(|failure| failure == "claude_project_runtime_hooks_reintroduced"),
        "{:?}",
        evaluation.failures
    );
}

#[test]
fn rejects_forbidden_network_and_mutation_tokens_in_hook_text() {
    let failures = checker::forbidden_runtime_behavior_failures(
        "tools/hooks/bad.sh",
        "#!/usr/bin/env bash\ncurl https://example.invalid\ngit push origin dev\nrm -rf /tmp/bad\n",
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("forbidden_network_or_remote_command:curl")),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("forbidden_runtime_token:git push")),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("forbidden_runtime_token:rm -rf")),
        "{:?}",
        failures
    );
}

#[test]
fn rejects_unpinned_host_interpreters_in_runtime_hooks() {
    let failures = checker::forbidden_runtime_behavior_failures(
        "tools/hooks/bad.sh",
        "#!/usr/bin/env bash\npython3 - <<'PY'\nPY\nnode tools/bad.js\npnpm install\n",
    );
    for interpreter in ["python3", "node", "pnpm"] {
        assert!(
            failures.iter().any(|failure| failure
                == &format!(
                    "tools/hooks/bad.sh:forbidden_runtime_interpreter:{}",
                    interpreter
                )),
            "{:?}",
            failures
        );
    }
}

#[test]
fn shell_word_match_avoids_substring_false_positives() {
    assert!(!checker::contains_shell_word("function advisory()", "nc"));
    assert!(!checker::contains_shell_word("github-lane-unlocker", "gh"));
    assert!(checker::contains_shell_word(
        "if command -v gh >/dev/null",
        "gh"
    ));
}
