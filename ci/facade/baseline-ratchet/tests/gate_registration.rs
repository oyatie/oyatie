// gate-registration completeness meta-test (ADR-0515 D2; CICD-DESIGN-PLAN Stage 1B + Pre-mortem
// Scenario-1c "silent-skip false-green" sibling acceptance test).
//
// INVARIANT: every gate crate directory under `ci/facade/` — EXCEPT the producer
// (`oya-cloud-ci-accounting-registry-app`, the rust_binary that EMITS the faces, not a gate
// lane) — MUST be registered as a job lane in `.github/workflows/oya-ci-required.yml`, the
// single canonical `oya-ci-required` fan-in. A new gate cannot be added without registering
// it in the required workflow; an in-tree-but-unregistered gate fails this test (it would be a
// silent false-green one level below the workflow's `needs:` fan-in).
//
// It is a pure filesystem + text gate: it reads the gates dir and greps the workflow yaml. No
// network, no GitHub API — runnable in any presubmit. Keep it deterministic and surface-all
// (it collects EVERY unregistered gate, not just the first).
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// The crates under `ci/facade/` that are NOT gate lanes — they are the rust_binaries
/// that EMIT the faces (registered in the workflow via a `run` step, not a `cargo test -p ...`
/// gate lane) or the on-demand born-accounting orchestrator. These are the intentional exclusions
/// from the gate-registration invariant:
///   - the accounting producer (emits the five accounting faces);
///   - the scm-facts emitter (the single out-of-graph git boundary that emits
///     scm-facts.generated.json; OYA-CI-HERMETIC-EXECUTION-DESIGN §1.5);
///   - the born-accounting register_crate ORCHESTRATOR (G011 slice 3b, ADR-0568): it MUTATES the
///     source SSOTs (OWNERS/registry/ADR/catalog/reachability) to onboard a NEW crate. It is not a
///     gate lane and not a face-emitter — it is invoked ON DEMAND to register a crate, never in the
///     required fan-in (it would have nothing to assert and would mutate the tree under presubmit).
///   - the planning-projection renderer: a pure library invoked by generated-artifact freshness to
///     materialize the untracked board projection; it has no independent admission verdict.
const PRODUCER_CRATE: &str = "artifact-inventory-registry";
const NON_GATE_CRATES: [&str; 3] = [
    "artifact-inventory-registry",
    "crate-registration",
    "planning-projection",
];

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`). Mirrors the helper in `firewall.rs` so both meta-gates
/// resolve the root identically.
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

fn gates_dir(root: &Path) -> PathBuf {
    root.join("ci/facade")
}

fn workflow_path(root: &Path) -> PathBuf {
    root.join(".github/workflows/oya-ci-required.yml")
}

fn phase0_automation_matrix_path(root: &Path) -> PathBuf {
    root.join("specs/phase0-automation-matrix.json")
}

fn branch_protection_path(root: &Path) -> PathBuf {
    root.join(".github/branch-protection.yaml")
}

fn agent_contract_path(root: &Path) -> PathBuf {
    root.join("docs/AGENTS.md")
}

fn pr_template_path(root: &Path) -> PathBuf {
    root.join("docs/templates/pull-request-template.md")
}
fn root_pr_template_path(root: &Path) -> PathBuf {
    root.join("templates/pull-request-template.md")
}

fn pr_template_v2_path(root: &Path) -> PathBuf {
    root.join("docs/templates/pull-request-template-v2.md")
}

fn oya_ci_config_path(root: &Path) -> PathBuf {
    root.join("oya-ci.toml")
}

fn bundled_gate_disposition_path(root: &Path) -> PathBuf {
    root.join("libs/oya-ci-config/src/bundled/gate-disposition.json")
}

/// Resolve the NEW de-branded `ci/facade/<dir>` directory a moved gate crate now lives in,
/// keyed on its OLD cargo/crate name (`oya-<gate_id>-app`). The committed move-plan
/// (`specs/reorg/ci-move-plan.json`, ADR-0562/0563) is the SSOT for the ci keystone rename;
/// the required-workflow matrix `crate:` value is this NEW dir. The de-brand renamed
/// SEMANTICALLY (e.g. cloud-ci-total-accounting -> artifact-accountability), so there is no
/// textual prefix-strip from the gate id to the lane — the move-plan is the only authority.
fn ci_move_new_dir(root: &Path, old_cargo_name: &str) -> Option<String> {
    let plan: Value =
        serde_json::from_str(&read_to_string(&root.join("specs/reorg/ci-move-plan.json"))).ok()?;
    if let Some(moves) = plan.get("moves").and_then(Value::as_array) {
        for m in moves {
            if m.get("old_cargo_name").and_then(Value::as_str) == Some(old_cargo_name) {
                return m
                    .get("new_path")
                    .and_then(Value::as_str)
                    .and_then(|p| p.rsplit('/').next())
                    .map(str::to_owned);
            }
        }
    }
    if let Some(arts) = plan.get("artifacts").and_then(Value::as_array) {
        for a in arts {
            let old_tail = a
                .get("old_path")
                .and_then(Value::as_str)
                .and_then(|p| p.rsplit('/').next());
            if old_tail == Some(old_cargo_name) {
                return a
                    .get("new_path")
                    .and_then(Value::as_str)
                    .and_then(|p| p.rsplit('/').next())
                    .map(str::to_owned);
            }
        }
    }
    None
}

fn code_review_standard_path(root: &Path) -> PathBuf {
    root.join("docs/standards/code-review.md")
}

fn fixuptasks_path(root: &Path) -> PathBuf {
    root.join("registry/fixuptasks.jsonl")
}

/// Every directory directly under `ci/facade/` that is a gate lane. Most lanes have a
/// Cargo manifest, but Buck2-only productized gates are equally required workflow authority and must
/// not be silently skipped by this meta-test.
fn gate_crate_dirs(gates: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(gates)
        .unwrap_or_else(|e| panic!("read gates dir {}: {e}", gates.display()))
        .map(|entry| entry.expect("dir entry").path())
        .filter(|p| p.is_dir() && (p.join("Cargo.toml").is_file() || p.join("BUCK").is_file()))
        .map(|p| {
            p.file_name()
                .expect("dir file_name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

/// The trimmed value of a YAML list item (`- value   # comment`) → `value`, or None if the line
/// is not a list item. Strips an end-of-line comment so matrix entries / `needs:` entries that
/// carry trailing comments still compare cleanly.
fn yaml_list_item(line: &str) -> Option<String> {
    let no_comment = line.split('#').next().unwrap_or("").trim();
    no_comment.strip_prefix("- ").map(|v| v.trim().to_owned())
}

fn fan_in_block(workflow: &str) -> &str {
    let fan_in_anchor = "\n  oya-ci-required:";
    let idx = workflow
        .find(fan_in_anchor)
        .expect("fan-in job `oya-ci-required:` not found in workflow");
    &workflow[idx..]
}

fn workflow_job(workflow: &str, job_name: &str) -> String {
    let anchor = format!("  {job_name}:");
    let mut found = false;
    let mut lines = Vec::new();
    for line in workflow.lines() {
        if line == anchor {
            found = true;
        } else if found && !line.trim().is_empty() && !line.starts_with("    ") {
            break;
        }
        if found {
            lines.push(line);
        }
    }
    assert!(found, "workflow job `{job_name}` not found");
    lines.join("\n")
}

/// Extract structural YAML step blocks. YAML mapping keys may be ordered arbitrarily, and
/// comments are never interpreted as fields.
fn workflow_steps(gate_job: &str) -> Vec<Vec<&str>> {
    let mut steps = Vec::new();
    let mut current = Vec::new();

    for line in gate_job.lines() {
        if line.starts_with("      - ") {
            if !current.is_empty() {
                steps.push(current);
            }
            current = vec![line];
        } else if !current.is_empty() {
            current.push(line);
        }
    }
    if !current.is_empty() {
        steps.push(current);
    }

    steps
}

fn step_runs_buck2_test(step: &[&str]) -> bool {
    step.iter()
        .filter_map(|line| {
            line.strip_prefix("      - run:")
                .or_else(|| line.strip_prefix("        run:"))
        })
        .any(|run| run.contains("buck2 test"))
}

fn buck2_matrix_step_uses_pwsh(gate_job: &str) -> bool {
    let mut buck2_steps = workflow_steps(gate_job)
        .into_iter()
        .filter(|step| step_runs_buck2_test(step));
    let Some(step) = buck2_steps.next() else {
        return false;
    };

    buck2_steps.next().is_none()
        && step
            .iter()
            .any(|line| *line == "      - shell: pwsh" || *line == "        shell: pwsh")
}

#[test]
fn workflow_job_does_not_swallow_less_indented_or_top_level_content() {
    let workflow = "jobs:\n  gate:\n    runs-on: ubuntu-latest\n    steps: []\n\nname: unrelated top-level workflow name\n  unrelated-job:\n    runs-on: ubuntu-latest\n";

    let gate = workflow_job(workflow, "gate");

    assert!(gate.contains("runs-on: ubuntu-latest"));
    assert!(
        !gate.contains("name: unrelated top-level workflow name")
            && !gate.contains("unrelated-job:"),
        "a workflow job must end at every nonblank line with indentation below its body, including a column-zero top-level key"
    );
}

#[test]
fn buck2_matrix_step_requires_pwsh_structurally_regardless_of_key_order() {
    let reordered = "  gate:\n    steps:\n      - shell: pwsh\n        # shell: bash must remain a comment, not a field\n        name: buck2 test ${{ matrix.crate }}\n        run: buck2 test\n";
    let dummy_named_pwsh_with_bash_buck_run = "  gate:\n    steps:\n      - name: buck2 test ${{ matrix.crate }}\n        shell: pwsh\n        run: Write-Host dummy\n      - name: actual execution\n        shell: bash\n        run: buck2 test $targets\n";
    let two_buck_runs = "  gate:\n    steps:\n      - name: first\n        shell: pwsh\n        run: buck2 test first\n      - name: second\n        shell: pwsh\n        run: buck2 test second\n";

    assert!(buck2_matrix_step_uses_pwsh(reordered));
    assert!(
        !buck2_matrix_step_uses_pwsh(&reordered.replace("shell: pwsh", "shell: bash")),
        "the exact Buck2 matrix-test step must reject Bash even when its shell key precedes name"
    );
    assert!(
        !buck2_matrix_step_uses_pwsh(dummy_named_pwsh_with_bash_buck_run),
        "a dummy named PowerShell step must not mask a differently named Bash step that actually runs Buck2"
    );
    assert!(
        !buck2_matrix_step_uses_pwsh(two_buck_runs),
        "the gate job must expose exactly one structural Buck2 test step"
    );
}

const WINDOWS_RESOLVER_DIFFERENTIAL_TARGET: &str =
    "//libs/oya-workspace-members-kernel:oya-workspace-members-kernel-cargo-differential";

/// The matrix currently uses compact YAML objects, so keep this deliberately narrow parser
/// coupled to that stable workflow shape.  The important invariant is that the Windows runner
/// and its one permitted target live in the same `matrix.include` entry; independent substring
/// checks could pair fields from two different legs and silently accept a false-green split.
fn windows_resolver_matrix_entry(gate_job: &str) -> Option<&str> {
    gate_job.lines().map(str::trim).find(|line| {
        line.starts_with("- {")
            && line.contains("os: windows-latest")
            && line.contains(WINDOWS_RESOLVER_DIFFERENTIAL_TARGET)
    })
}

fn is_exact_windows_resolver_matrix_entry(entry: &str) -> bool {
    entry.starts_with("- {")
        && entry.ends_with('}')
        && entry.contains("os: windows-latest")
        && entry.contains(&format!(
            "targets: \"{WINDOWS_RESOLVER_DIFFERENTIAL_TARGET}\""
        ))
        && !entry.contains("cargo test")
        && !entry.contains("cargo.exe")
}

/// Extract the complete outer PowerShell `if ($IsWindows) { ... }` branch. Nested `if`/`else`
/// pairs are legal in the workflow command, so a textual `} else {` delimiter is not a safe
/// branch boundary. Quoted command text may contain braces without affecting block depth.
fn windows_branch(gate_job: &str) -> Option<&str> {
    let start = gate_job.find("if ($IsWindows)")?;
    let open = gate_job[start..].find('{')? + start;
    let bytes = gate_job.as_bytes();
    let mut depth = 0usize;
    let mut quote = None;
    let mut index = open;

    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(delimiter) = quote {
            if delimiter == b'"' && byte == b'`' {
                index += 2;
                continue;
            }
            if byte == delimiter {
                if delimiter == b'\'' && bytes.get(index + 1) == Some(&b'\'') {
                    index += 2;
                    continue;
                }
                quote = None;
            }
            index += 1;
            continue;
        }

        if byte == b'`' {
            index += 2;
            continue;
        }

        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(&gate_job[start..=index]);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// True when the PowerShell source contains a direct `cargo` or `cargo.exe` executable token.
/// Delimiters include every non-command-word character, so tabs and newlines cannot bypass it,
/// while similarly named commands such as `cargo-fmt` remain distinct.
fn contains_direct_cargo_executable(branch: &str) -> bool {
    branch
        .split(|character: char| {
            !character.is_ascii_alphanumeric() && !matches!(character, '.' | '_' | '-')
        })
        .any(|token| token.eq_ignore_ascii_case("cargo") || token.eq_ignore_ascii_case("cargo.exe"))
}

/// The Windows branch must provision MSVC before Buck2, and must never introduce a direct
/// Cargo executable. The target itself owns the resolver/Cargo differential under Buck2.
fn windows_branch_is_buck_only(gate_job: &str) -> bool {
    let Some(branch) = windows_branch(gate_job) else {
        return false;
    };
    // YAML single-quoted scalars escape a literal PowerShell quote as `''`; normalize that
    // presentation detail before enforcing the executed-script receipt contract.
    let branch = branch.replace("''", "'");
    let Some(vsdevcmd) = branch.find("VsDevCmd.bat") else {
        return false;
    };
    let Some(buck2) = branch.find("buck2 test $targets") else {
        return false;
    };
    vsdevcmd < buck2 && !contains_direct_cargo_executable(&branch)
}

/// The hosted Windows image may change Visual Studio major version, edition, or install root.
/// Resolve the current MSVC installation with Microsoft's bundled `vswhere.exe`, fail closed
/// when either discovery executable is absent, and only then hand the derived `VsDevCmd.bat`
/// path to the native Buck command.
fn windows_branch_discovers_msvc_toolchain(gate_job: &str) -> bool {
    let Some(branch) = windows_branch(gate_job) else {
        return false;
    };
    let branch = branch.replace("''", "'");
    let required_in_order = [
        "$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\\Installer\\vswhere.exe'",
        "Test-Path -LiteralPath $vswhere -PathType Leaf",
        "$vsInstallCandidates = @(& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath)",
        "$vswhereExitCode = $LASTEXITCODE",
        "if ($vswhereExitCode -ne 0) { exit $vswhereExitCode }",
        "$vsInstall = $vsInstallCandidates | Select-Object -First 1",
        "[string]::IsNullOrWhiteSpace($vsInstall)",
        "$vsDevCmd = Join-Path $vsInstall 'Common7\\Tools\\VsDevCmd.bat'",
        "Test-Path -LiteralPath $vsDevCmd -PathType Leaf",
        "$windowsBuckCommand = \"call `\"$vsDevCmd`\" -arch=amd64 -host_arch=amd64 >nul && buck2 test $targets\"",
        "cmd.exe /d /s /c $windowsBuckCommand",
    ];

    let mut cursor = 0usize;
    for required in required_in_order {
        let Some(relative_index) = branch[cursor..].find(required) else {
            return false;
        };
        cursor += relative_index + required.len();
    }

    !branch.contains("Microsoft Visual Studio\\2022")
        && !branch.contains("Microsoft Visual Studio\\2026")
        && !branch.contains("%ProgramFiles%")
}

fn has_positive_buck_test_pass_summary(receipt: &str) -> bool {
    receipt.lines().any(|line| {
        let Some(rest) = line.strip_prefix("Tests finished:") else {
            return false;
        };
        let mut rest = rest.trim_start();
        while let Some(ansi) = rest.strip_prefix("\u{1b}[") {
            let Some(end) = ansi.find('m') else {
                return false;
            };
            rest = ansi[end + 1..].trim_start();
        }
        let Some(count) = rest.strip_prefix("Pass").map(str::trim_start) else {
            return false;
        };
        matches!(count.chars().next(), Some('1'..='9'))
    })
}

/// The Windows cmd handoff is only a real Buck execution when it leaves a receipt with both
/// Buck's build identifier and its successful-test summary.  `cmd.exe` can otherwise return a
/// successful shell invocation while the intended Buck command was never observably run.
fn windows_branch_has_buck_execution_receipt(gate_job: &str) -> bool {
    let Some(branch) = windows_branch(gate_job) else {
        return false;
    };
    // YAML single-quoted scalars escape a literal PowerShell quote as `''`; normalize that
    // presentation detail before enforcing the executed-script receipt contract.
    let branch = branch.replace("''", "'");

    let receipt_assignment =
        "$windowsBuckReceipt = Join-Path $env:RUNNER_TEMP 'buck2-windows-receipt.log'";
    let tee = "| Tee-Object -FilePath $windowsBuckReceipt";
    let exit_capture = "$windowsBuckExitCode = $LASTEXITCODE";
    let exit_print = "Write-Host \"Windows Buck2 cmd exit code: $windowsBuckExitCode\"";
    let print = "Get-Content -Path $windowsBuckReceipt";
    let read = "$windowsBuckReceiptText = Get-Content -Path $windowsBuckReceipt -Raw";
    let build_id = ".Contains('Build ID:')";
    let positive_pass_summary = "$windowsBuckPassed = $windowsBuckReceiptText -match '(?m)^Tests finished:\\s*(?:\\x1b\\[[0-9;]*m)?Pass\\s+[1-9][0-9]*\\b'";
    let exit_propagation = "if ($windowsBuckExitCode -ne 0) { exit $windowsBuckExitCode }";

    let Some(receipt_assignment_index) = branch.find(receipt_assignment) else {
        return false;
    };
    let Some(tee_index) = branch.find(tee) else {
        return false;
    };
    let Some(exit_capture_index) = branch.find(exit_capture) else {
        return false;
    };
    let Some(exit_print_index) = branch.find(exit_print) else {
        return false;
    };
    let Some(print_index) = branch.find(print) else {
        return false;
    };
    let Some(read_index) = branch.find(read) else {
        return false;
    };
    let Some(build_id_index) = branch.find(build_id) else {
        return false;
    };
    let Some(positive_pass_summary_index) = branch.find(positive_pass_summary) else {
        return false;
    };
    let Some(exit_propagation_index) = branch.find(exit_propagation) else {
        return false;
    };

    receipt_assignment_index < tee_index
        && tee_index < exit_capture_index
        && exit_capture_index < exit_print_index
        && exit_print_index < print_index
        && print_index < read_index
        && read_index < positive_pass_summary_index
        && positive_pass_summary_index < build_id_index
        && exit_capture_index < exit_propagation_index
}

fn fan_in_mentions_job(fan_in_block: &str, job: &str) -> bool {
    fan_in_block
        .lines()
        .filter_map(yaml_list_item)
        .any(|v| v == job)
        && fan_in_block.contains(&format!("needs.{job}.result"))
}

fn live_postgres_split_fan_in_is_complete(workflow: &str) -> bool {
    let block = fan_in_block(workflow);
    fan_in_mentions_job(block, "gate-live-postgres-adapters")
        && fan_in_mentions_job(block, "gate-live-postgres-facades")
        && !block.contains("needs.gate-live-postgres.result")
}

fn affected_set_long_step_telemetry_is_wired(workflow: &str) -> bool {
    workflow.contains("oya-cloud-ci-step-telemetry-bin")
        && workflow.contains("--phase derive-affected-set-tier --")
        && workflow.contains("--phase materialize-merge-base-build-health-baseline --")
        && workflow.contains("--phase binding-affected-set-build-test --")
}

/// True iff `crate_dir` is an entry in the `gate` job's `strategy.matrix` — i.e. it is a
/// homogeneous matrix gate run via `cargo test -p ${{ matrix.crate }}`. Recognizes both the
/// simple list form (`- <crate>`) and the `include`-object form (`{ crate: <crate>, label: … }`).
/// Crate names are unique and distinct from the `needs:` job names, so scanning the whole
/// workflow is safe; the per-line comment strip keeps trailing `# …` from defeating the match.
fn is_matrix_gate(workflow: &str, crate_dir: &str) -> bool {
    workflow.lines().any(|line| {
        let t = line.split('#').next().unwrap_or("").trim();
        t == format!("- {crate_dir}") || t.contains(&format!("crate: {crate_dir},"))
    })
}

fn is_buck_gate(workflow: &str, crate_dir: &str) -> bool {
    workflow.contains(&format!("//ci/facade/{crate_dir}:"))
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn fixuptask_by_id(fixuptasks_jsonl: &str, id: &str) -> Option<Value> {
    fixuptasks_jsonl.lines().find_map(|line| {
        let value: Value = serde_json::from_str(line).ok()?;
        (value.get("id").and_then(Value::as_str) == Some(id)).then_some(value)
    })
}

fn bool_field(row: &Value, key: &str) -> bool {
    row.get(key).and_then(Value::as_bool) == Some(true)
}

fn quoted_value(raw: &str) -> String {
    raw.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_owned()
}

fn oya_ci_enabled_gate_specs(config: &str) -> Vec<(String, String)> {
    let mut specs = Vec::new();
    let mut in_gate = false;
    let mut id: Option<String> = None;
    let mut input_kind: Option<String> = None;

    for line in config.lines() {
        let trimmed = line.split('#').next().unwrap_or("").trim();
        if trimmed == "[[gates.enabled]]" {
            if let Some(gate_id) = id.take() {
                specs.push((gate_id, input_kind.take().unwrap_or_default()));
            }
            input_kind = None;
            in_gate = true;
            continue;
        }
        if in_gate && trimmed.starts_with('[') {
            if let Some(gate_id) = id.take() {
                specs.push((gate_id, input_kind.take().unwrap_or_default()));
            }
            input_kind = None;
            in_gate = false;
            continue;
        }
        if !in_gate {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("id = ") {
            id = Some(quoted_value(value));
        } else if let Some(value) = trimmed.strip_prefix("input_kind = ") {
            input_kind = Some(quoted_value(value));
        }
    }

    if let Some(gate_id) = id {
        specs.push((gate_id, input_kind.unwrap_or_default()));
    }

    specs
}

fn assert_pr_template_authority(root: &Path, path: &Path) {
    let text = read_to_string(path);
    for required in [
        "oya-ci-required",
        "F-PR5-06",
        "trusted server-side/cloud-ci",
        "not live cloud admission enforcement",
        "## Code Review",
    ] {
        assert!(
            text.contains(required),
            "{} must align PR authority on `{required}`.",
            path.display()
        );
    }

    for forbidden in [
        "grit",
        "icm",
        "oya-tooling-agent-read",
        "cargo nextest",
        "cargo clippy",
        "cargo deny",
        "oya verify",
        "oya gate validate",
        "guard-pr-merge-review.mjs",
        ".omc/plans",
        ".omc/scratch",
        "Grit-claim",
    ] {
        assert!(
            !text.contains(forbidden),
            "{} still carries retired local/CLI review instruction `{forbidden}`; \
             use Buck2/cloud-ci, `oya-ci-required`, and reviewer evidence instead.",
            path.display()
        );
    }

    let canonical = root.join("docs/templates/pull-request-template.md");
    if path != canonical {
        assert!(
            text.contains("docs/templates/pull-request-template.md")
                || text.contains("Canonical PR body"),
            "{} must identify the canonical docs PR-template authority.",
            path.display()
        );
    }
}

fn row_claims_live_pre_merge_review_authority(row: &Value) -> bool {
    bool_field(row, "review_authority_live")
        && (bool_field(row, "has_durable_review_evidence")
            || bool_field(row, "has_machine_verifiable_review_status"))
        && bool_field(row, "review_blocks_merge")
        && bool_field(row, "reviewer_identity_distinct_from_author")
}

fn phase0_pre_merge_review_rows(root: &Path) -> Vec<Value> {
    let path = phase0_automation_matrix_path(root);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let value: Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
    let rows = value
        .get("seed_rows")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{} must contain a seed_rows array", path.display()));
    let review_rows: Vec<Value> = rows
        .iter()
        .filter(|row| bool_field(row, "requires_pre_merge_review_authority"))
        .cloned()
        .collect();
    assert!(
        !review_rows.is_empty(),
        "{} must contain the pre-merge review-authority row; removing it would hide F-PR5-06",
        path.display()
    );
    review_rows
}

#[test]
fn every_gate_crate_is_registered_in_oya_ci_required_workflow() {
    let root = repo_root();
    let gates = gates_dir(&root);
    let wf = workflow_path(&root);

    let workflow =
        fs::read_to_string(&wf).unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));

    let crates = gate_crate_dirs(&gates);
    assert!(
        !crates.is_empty(),
        "found no gate crates under {} — the gates dir layout changed",
        gates.display()
    );

    // The producer must be present in-tree (sanity: the exclusion is real, not a typo).
    assert!(
        crates.iter().any(|c| c == PRODUCER_CRATE),
        "producer crate {PRODUCER_CRATE} not found under {} — update PRODUCER_CRATE if it was \
         renamed",
        gates.display()
    );

    // Surface-all: collect EVERY unregistered gate, then assert the set is empty.
    let mut missing: Vec<String> = Vec::new();
    for crate_dir in &crates {
        if NON_GATE_CRATES.contains(&crate_dir.as_str()) {
            continue;
        }
        // A gate is "registered" iff the workflow runs it as a cargo lane, matrix lane, or
        // dedicated Buck target lane. Freshness is a Buck-built binary because it must use the
        // same face producer targets as the materialization boundary without mutating the tree.
        let registered = workflow.contains(&format!("-p {crate_dir}"))
            || is_matrix_gate(&workflow, crate_dir)
            || is_buck_gate(&workflow, crate_dir);
        if !registered {
            missing.push(crate_dir.clone());
        }
    }

    assert!(
        missing.is_empty(),
        "gate crate(s) present under {} but NOT registered in {} — add the crate to the `gate` \
         job's `strategy.matrix.crate` list (homogeneous gates), give it a bespoke `-p <crate>` \
         job, or wire a dedicated `//ci/facade/<crate>:` Buck target \
         job: {:?}\n\
         An in-tree-but-unregistered gate is a silent false-green one level below the fan-in.",
        gates.display(),
        wf.display(),
        missing
    );
}

#[test]
fn census_epoch_receipt_is_a_buck_live_face_gate_not_a_cargo_only_false_green() {
    let root = repo_root();
    let workflow = read_to_string(&workflow_path(&root));
    let buck = read_to_string(&root.join("ci/facade/scm-facts-snapshot/BUCK"));
    let gate_matrix_entry = "crate: scm-facts-snapshot,";
    let download_step = workflow
        .find("Download regenerated faces")
        .expect("gate matrix must download controller-generated faces");
    let test_step = workflow
        .find("buck2 test ${{ matrix.crate }}")
        .expect("gate matrix must execute Buck tests");

    assert!(workflow.contains(gate_matrix_entry));
    assert!(
        download_step < test_step,
        "the live face must be downloaded before validation"
    );
    assert!(
        buck.contains("name = \"ci-scm-facts-snapshot-gate\"")
            && buck.contains("src/bin/adr-census-epoch-receipt-gate.rs"),
        "the matrix target must execute the census-epoch receipt live validator under Buck"
    );
    assert!(
        workflow.contains("//ci/facade/${{ matrix.crate }}:ci-${{ matrix.crate }}-gate"),
        "a Cargo-only test command is not required-CI authority"
    );
}

/// The review-admission gap must never be hidden behind aspirational docs or repo-local target
/// matrices. Until a trusted server-side/cloud-ci review producer exists and this test is replaced
/// with live producer evidence, the gap must stay explicitly bounded in the docs and the backlog
/// SSOT with concrete evidence from the most recent admission incident.
#[test]
fn review_admission_gap_is_bounded_by_open_ssot_debt_until_trusted_producer() {
    let root = repo_root();
    let branch_path = branch_protection_path(&root);
    let branch_protection = fs::read_to_string(&branch_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", branch_path.display()));

    let review_rows = phase0_pre_merge_review_rows(&root);

    let docs_path = agent_contract_path(&root);
    let docs = fs::read_to_string(&docs_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", docs_path.display()));
    assert!(
        docs.contains("REVIEW-ADMISSION-GAP-LIVE-BOUNDARY")
            && docs.contains("F-PR5-06")
            && docs.contains("PR #964")
            && docs.contains("not a cloud-enforced review admission gate"),
        "{} still describes review admission as live cloud enforcement while the producer is not \
         wired. Add an explicit REVIEW-ADMISSION-GAP-LIVE-BOUNDARY note with F-PR5-06 and PR #964 \
         evidence.",
        docs_path.display()
    );

    for template_path in [
        pr_template_path(&root),
        pr_template_v2_path(&root),
        root_pr_template_path(&root),
    ] {
        assert_pr_template_authority(&root, &template_path);
    }

    let standard_path = code_review_standard_path(&root);
    let standard = fs::read_to_string(&standard_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", standard_path.display()));
    assert!(
        standard.contains("F-PR5-06")
            && standard.contains("advisory only")
            && standard.contains("trusted server-side/cloud-ci review producer"),
        "{} must not present local hooks as live cloud review authority while F-PR5-06 is open.",
        standard_path.display()
    );

    assert!(
        branch_protection.contains("F-PR5-06") && branch_protection.contains("PR #964"),
        "{} must name the bounded review-admission debt while required_approving_reviews is 0 and \
         oya-pr-review is absent.",
        branch_path.display()
    );

    let fixuptasks = fs::read_to_string(fixuptasks_path(&root)).expect("read fixuptasks.jsonl");
    let task =
        fixuptask_by_id(&fixuptasks, "F-PR5-06").expect("registry/fixuptasks.jsonl has F-PR5-06");
    assert_eq!(task.get("status").and_then(Value::as_str), Some("open"));

    for row in review_rows {
        assert!(
            !row_claims_live_pre_merge_review_authority(&row),
            "F-PR5-06 is still open, so repo-local phase0 matrix rows must not be allowed to \
             false-clear the review-admission gap as live authority. Close F-PR5-06 only in the \
             same change that adds trusted server-side/cloud-ci producer evidence. Row was: {row}"
        );
    }

    let task_text = task.to_string();
    for required in [
        "PR #964",
        "reviewDecision",
        "COMMENTED",
        "required_approving_reviews: 0",
        "single oya-ci-required",
        "oya-pr-review",
        "not live",
    ] {
        assert!(
            task_text.contains(required),
            "F-PR5-06 must carry fresh bounded-debt evidence term `{required}`; row was: {task_text}"
        );
    }
}

#[test]
fn oya_ci_configured_gates_have_disposition_and_required_workflow_authority() {
    let root = repo_root();
    let workflow = read_to_string(&workflow_path(&root));
    let config = read_to_string(&oya_ci_config_path(&root));
    let configured = oya_ci_enabled_gate_specs(&config);
    assert!(
        !configured.is_empty(),
        "oya-ci.toml must declare the accounting/firewall gate set under [[gates.enabled]]"
    );

    let disposition_path = bundled_gate_disposition_path(&root);
    let disposition: Value = serde_json::from_str(&read_to_string(&disposition_path))
        .unwrap_or_else(|e| panic!("parse {}: {e}", disposition_path.display()));
    let disposition_gates = disposition
        .get("gates")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("{} must contain gates object", disposition_path.display()));

    let configured_ids: BTreeSet<String> = configured
        .iter()
        .map(|(gate_id, _)| gate_id.clone())
        .collect();
    let disposition_ids: BTreeSet<String> = disposition_gates.keys().cloned().collect();

    let missing_disposition: Vec<String> = configured_ids
        .difference(&disposition_ids)
        .cloned()
        .collect();
    let extra_disposition: Vec<String> = disposition_ids
        .difference(&configured_ids)
        .cloned()
        .collect();
    assert!(
        missing_disposition.is_empty() && extra_disposition.is_empty(),
        "oya-ci.toml [[gates.enabled]] and bundled gate-disposition.json must converge exactly; \
         missing disposition for {:?}, extra disposition rows {:?}",
        missing_disposition,
        extra_disposition
    );

    let mut missing_workflow_authority = Vec::new();
    for (gate_id, input_kind) in configured {
        let has_required_lane = match input_kind.as_str() {
            // The matrix `crate:` value is the NEW de-branded ci/facade dir; resolve it via the
            // committed move-plan SSOT (the ci keystone move renamed the crates semantically).
            "producer-face" => ci_move_new_dir(&root, &format!("oya-{gate_id}-app"))
                .is_some_and(|dir| workflow.contains(&format!("crate: {dir},"))),
            "raw-corpus-collector" => {
                workflow.contains("producer-regen") && workflow.contains("gate-baseline-ratchet")
            }
            "frozen-empty-meta" => {
                gate_id == "cloud-ci-freshness"
                    && workflow.contains("gate-generated-artifact-freshness")
            }
            other => panic!("unknown gate input_kind `{other}` for {gate_id} in oya-ci.toml"),
        };
        if !has_required_lane {
            missing_workflow_authority.push(format!("{gate_id} ({input_kind})"));
        }
    }

    assert!(
        missing_workflow_authority.is_empty(),
        "oya-ci.toml enabled gate(s) lack required workflow authority in .github/workflows/oya-ci-required.yml: {:?}",
        missing_workflow_authority
    );
}

/// Defense-in-depth: every registered gate must ALSO be wired into the fan-in job's `needs:`
/// list (a `cargo test -p <crate>` lane that the `oya-ci-required` job does not depend on
/// would never gate the required context). We assert each non-producer gate's directory name
/// appears somewhere after the `oya-ci-required:` job header in a `needs:` context.
#[test]
fn every_gate_lane_is_a_dependency_of_the_fan_in_job() {
    let root = repo_root();
    let gates = gates_dir(&root);
    let wf = workflow_path(&root);
    let workflow =
        fs::read_to_string(&wf).unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));

    let fan_in_block = fan_in_block(&workflow);
    assert!(
        fan_in_block.contains("needs:"),
        "fan-in job `oya-ci-required` has no `needs:` — it must depend on every gate lane"
    );

    // Map each non-producer gate crate to its expected job-name token in the fan-in `needs:`.
    // Job names in this workflow embed the gate identity (e.g. `gate-total-accounting`,
    // `gate-registry-drift`). We assert the gate's short identity appears in the fan-in block.
    let crates = gate_crate_dirs(&gates);
    let mut missing: Vec<String> = Vec::new();
    for crate_dir in &crates {
        if NON_GATE_CRATES.contains(&crate_dir.as_str()) {
            continue;
        }
        // A matrix gate runs under the single `gate` job → the fan-in must depend on `gate`.
        // A bespoke gate has its own `gate-<short>` job → the fan-in must reference that short
        // identity. (`registry-drift` is already short; others strip `oya-cloud-ci-`/`-app`.)
        let wired = if is_matrix_gate(&workflow, crate_dir) {
            fan_in_block
                .lines()
                .filter_map(yaml_list_item)
                .any(|v| v == "gate")
        } else {
            let short = crate_dir
                .strip_prefix("oya-cloud-ci-")
                .unwrap_or(crate_dir)
                .strip_suffix("-app")
                .map(str::to_owned)
                .unwrap_or_else(|| crate_dir.clone());
            fan_in_block.contains(&short)
        };
        if !wired {
            missing.push(format!(
                "{crate_dir} (matrix gate ⇒ needs `- gate`; bespoke ⇒ needs its `gate-<short>` job)"
            ));
        }
    }

    assert!(
        missing.is_empty(),
        "gate lane(s) not wired into the `oya-ci-required` fan-in job's `needs:` in {}: {:?}\n\
         Every gate lane must be a dependency of the fan-in or its failure cannot make the \
         required context red.",
        wf.display(),
        missing
    );
}

#[test]
fn live_postgres_split_lanes_are_both_required_by_fan_in() {
    let root = repo_root();
    let wf = workflow_path(&root);
    let workflow =
        fs::read_to_string(&wf).unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));
    assert!(
        live_postgres_split_fan_in_is_complete(&workflow),
        "fan-in must require both split live-postgres jobs and must not keep the retired monolithic needs token"
    );

    let without_adapters =
        workflow.replace("      - gate-live-postgres-adapters", "      # removed");
    assert!(
        !live_postgres_split_fan_in_is_complete(&without_adapters),
        "missing adapter sublane must be detected as fan-in incomplete"
    );

    let without_facades = workflow.replace("      - gate-live-postgres-facades", "      # removed");
    assert!(
        !live_postgres_split_fan_in_is_complete(&without_facades),
        "missing facade sublane must be detected as fan-in incomplete"
    );
}

#[test]
fn windows_workspace_resolver_differential_is_a_buck2_matrix_leg() {
    let root = repo_root();
    let wf = workflow_path(&root);
    let workflow =
        fs::read_to_string(&wf).unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));

    // YAML single-quoted workflow scalars escape PowerShell's literal quote as `''`; inspect
    // the decoded script shape so the guard validates execution semantics, not YAML spelling.
    let gate_job = workflow_job(&workflow, "gate").replace("''", "'");
    assert!(
        gate_job.contains("runs-on: ${{ matrix.os || 'ubuntu-latest' }}"),
        "the reusable gate matrix must select its runner from the matrix"
    );
    let windows_entry = windows_resolver_matrix_entry(&gate_job)
        .expect("the Windows resolver differential must be one matrix.include entry");
    assert!(
        is_exact_windows_resolver_matrix_entry(windows_entry),
        "the Windows matrix leg must pair exactly `os: windows-latest` with the exact resolver differential Buck2 target: {windows_entry}"
    );
    assert!(
        buck2_matrix_step_uses_pwsh(&gate_job) && gate_job.contains("$IsWindows"),
        "the exact Buck2 matrix-test step must use PowerShell, which is native on Windows and avoids Bash/MSYS rewriting"
    );
    assert!(
        !buck2_matrix_step_uses_pwsh(&gate_job.replace("shell: pwsh", "shell: bash")),
        "changing the exact Buck2 matrix-test step to Bash must be rejected without relying on comment text or YAML key ordering"
    );
    assert!(
        gate_job.contains("[string]::IsNullOrWhiteSpace($targets)")
            && gate_job.contains("$targetArgs = $targets -split '\\s+' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }")
            && gate_job.contains("& buck2 test @targetArgs")
            && !gate_job.contains("else { & buck2 test $targets }")
            && gate_job.contains("cmd.exe /d /s /c")
            && windows_branch_is_buck_only(&gate_job)
            && windows_branch_discovers_msvc_toolchain(&gate_job)
            && windows_branch_has_buck_execution_receipt(&gate_job),
        "the non-Windows path must splat separate Buck2 arguments while Windows rejects an empty exact target, discovers the installed MSVC toolchain without a version/edition assumption, captures and prints the Buck receipt, and fails closed unless it contains Build ID plus a passing test summary"
    );
    assert!(
        !workflow.contains("\n  windows-workspace-member-resolver:")
            && !gate_job.contains("cargo test --locked -p oya-workspace-members-kernel"),
        "the Windows differential must stay inside the Buck2 target, not add a direct Cargo job"
    );
    assert!(
        fan_in_mentions_job(fan_in_block(&workflow), "gate"),
        "the Windows matrix leg must remain under the single oya-ci-required fan-in"
    );
}

#[test]
fn windows_workspace_resolver_registration_rejects_split_or_direct_cargo_mutations() {
    let valid = format!(
        "- {{ targets: \"{WINDOWS_RESOLVER_DIFFERENTIAL_TARGET}\", os: windows-latest, label: \"Windows\" }}"
    );
    assert!(is_exact_windows_resolver_matrix_entry(&valid));
    assert!(!is_exact_windows_resolver_matrix_entry(
        &valid.replace("os: windows-latest", "os: ubuntu-latest")
    ));
    assert!(!is_exact_windows_resolver_matrix_entry(&valid.replace(
        WINDOWS_RESOLVER_DIFFERENTIAL_TARGET,
        "//libs/oya-workspace-members-kernel:wrong-target"
    )));
    assert!(!is_exact_windows_resolver_matrix_entry(&format!(
        "{valid} cargo test"
    )));

    let branch = "if ($IsWindows) { call `\"VsDevCmd.bat`\" && buck2 test $targets } else { }";
    assert!(windows_branch_is_buck_only(branch));
    assert!(!windows_branch_is_buck_only(
        &branch.replace("VsDevCmd.bat`\" && buck2", "buck2")
    ));
    assert!(!windows_branch_is_buck_only(
        &branch.replace("buck2 test", "cargo test")
    ));
    assert!(!windows_branch_is_buck_only(
        &branch.replace("buck2 test", "cargo.exe test")
    ));
    assert!(windows_branch_is_buck_only(
        "if ($IsWindows) { call `\"VsDevCmd.bat`\" && buck2 test $targets; cargo-fmt --version } else { }"
    ));
    assert!(windows_branch_is_buck_only(
        "if ($IsWindows) { Write-Host '} else {'; call `\"VsDevCmd.bat`\" && buck2 test $targets } else { }"
    ));
    assert!(
        !windows_branch_is_buck_only(
            "if ($IsWindows) { call `\"VsDevCmd.bat`\" && buck2 test $targets; if ($nested) { Write-Host nested } else { cargo\t test } } else { }",
        ),
        "a nested else must not truncate the Windows branch before a direct Cargo invocation"
    );
    assert!(
        !windows_branch_is_buck_only(
            "if ($IsWindows) { call `\"VsDevCmd.bat`\" && buck2 test $targets; cargo\t test } else { }",
        ),
        "Cargo followed by whitespace other than a literal space is still a direct Cargo invocation"
    );
    assert!(
        !windows_branch_is_buck_only(
            "if ($IsWindows) { call `\"VsDevCmd.bat`\" && buck2 test $targets; cargo.exe\n test } else { }",
        ),
        "cargo.exe followed by a newline is still a direct Cargo invocation"
    );

    let discovered_toolchain_branch = "if ($IsWindows) { $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\\Installer\\vswhere.exe'; if (-not (Test-Path -LiteralPath $vswhere -PathType Leaf)) { exit 1 }; $vsInstallCandidates = @(& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath); $vswhereExitCode = $LASTEXITCODE; if ($vswhereExitCode -ne 0) { exit $vswhereExitCode }; $vsInstall = $vsInstallCandidates | Select-Object -First 1; if ([string]::IsNullOrWhiteSpace($vsInstall)) { exit 1 }; $vsDevCmd = Join-Path $vsInstall 'Common7\\Tools\\VsDevCmd.bat'; if (-not (Test-Path -LiteralPath $vsDevCmd -PathType Leaf)) { exit 1 }; $windowsBuckCommand = \"call `\"$vsDevCmd`\" -arch=amd64 -host_arch=amd64 >nul && buck2 test $targets\"; cmd.exe /d /s /c $windowsBuckCommand } else { }";
    assert!(windows_branch_discovers_msvc_toolchain(
        discovered_toolchain_branch
    ));
    assert!(
        !windows_branch_discovers_msvc_toolchain(&discovered_toolchain_branch.replace(
            "Microsoft Visual Studio\\Installer\\vswhere.exe",
            "Microsoft Visual Studio\\2022\\Enterprise\\Common7\\Tools\\VsDevCmd.bat"
        )),
        "a hard-coded Visual Studio major version and edition must not satisfy discovery"
    );
    assert!(
        !windows_branch_discovers_msvc_toolchain(&discovered_toolchain_branch.replace(
            "Test-Path -LiteralPath $vsDevCmd -PathType Leaf",
            "Test-Path -LiteralPath $vsDevCmd"
        )),
        "the discovered developer-command path must be a real file"
    );
    assert!(
        !windows_branch_discovers_msvc_toolchain(&discovered_toolchain_branch.replace(
            "-requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 ",
            ""
        )),
        "Visual Studio discovery must require the MSVC x64/x86 tool component"
    );
    assert!(
        !windows_branch_discovers_msvc_toolchain(
            &discovered_toolchain_branch.replace("$vswhereExitCode = $LASTEXITCODE; ", "")
        ),
        "the vswhere exit code must be captured immediately after discovery"
    );
    assert!(
        !windows_branch_discovers_msvc_toolchain(&discovered_toolchain_branch.replace(
            "cmd.exe /d /s /c $windowsBuckCommand",
            "Invoke-Expression $windowsBuckCommand"
        )),
        "the discovered batch environment and Buck2 test must stay in one native cmd process"
    );

    let receipt_branch = "if ($IsWindows) { $windowsBuckReceipt = Join-Path $env:RUNNER_TEMP 'buck2-windows-receipt.log'; cmd.exe /d /s /c \"call `\"VsDevCmd.bat`\" && buck2 test $targets\" 2>&1 | Tee-Object -FilePath $windowsBuckReceipt; $windowsBuckExitCode = $LASTEXITCODE; Write-Host \"Windows Buck2 cmd exit code: $windowsBuckExitCode\"; Get-Content -Path $windowsBuckReceipt; $windowsBuckReceiptText = Get-Content -Path $windowsBuckReceipt -Raw; $windowsBuckPassed = $windowsBuckReceiptText -match '(?m)^Tests finished:\\s*(?:\\x1b\\[[0-9;]*m)?Pass\\s+[1-9][0-9]*\\b'; if (-not ($windowsBuckReceiptText.Contains('Build ID:') -and $windowsBuckPassed)) { exit 1 }; if ($windowsBuckExitCode -ne 0) { exit $windowsBuckExitCode } } else { }";
    assert!(windows_branch_has_buck_execution_receipt(receipt_branch));
    assert!(has_positive_buck_test_pass_summary(
        "Tests finished: Pass 1"
    ));
    assert!(has_positive_buck_test_pass_summary(
        "Tests finished: Pass 12"
    ));
    assert!(has_positive_buck_test_pass_summary(
        "Tests finished: \u{1b}[mPass 1"
    ));
    assert!(!has_positive_buck_test_pass_summary(
        "Tests finished: Pass 0"
    ));
    assert!(!has_positive_buck_test_pass_summary(
        "Tests finished: Fail 1"
    ));
    assert!(!has_positive_buck_test_pass_summary(
        "Buck completed without a summary"
    ));
    assert!(
        !windows_branch_has_buck_execution_receipt(
            &receipt_branch.replace("$windowsBuckExitCode = $LASTEXITCODE; ", "")
        ),
        "the cmd exit code must be captured before PowerShell inspection can overwrite it"
    );
    assert!(
        !windows_branch_has_buck_execution_receipt(
            &receipt_branch.replace(".Contains('Build ID:')", ".Contains('Build identifier:')")
        ),
        "a cmd prompt without Buck's Build ID is a false-green execution receipt"
    );
    assert!(
        !windows_branch_has_buck_execution_receipt(
            &receipt_branch.replace("[1-9][0-9]*", "[0-9]*")
        ),
        "the Windows receipt regex must reject Pass 0 rather than accepting any numeric count"
    );
    assert!(
        !windows_branch_has_buck_execution_receipt(
            &receipt_branch.replace("Get-Content -Path $windowsBuckReceipt; ", "")
        ),
        "the raw Windows job log must print the captured execution receipt"
    );
    assert!(
        !windows_branch_has_buck_execution_receipt(&receipt_branch.replace(
            "Write-Host \"Windows Buck2 cmd exit code: $windowsBuckExitCode\"; ",
            ""
        )),
        "the raw Windows job log must print the original cmd exit code"
    );
}

/// The Rust toolchain must be the digest-pinned hermetic one by default, and the composed sysroot
/// must stay SELF-SUFFICIENT.
///
/// Both halves are structural because neither failure is observable from a build verdict:
/// `system_rust_toolchain` puts the compiler in the action key by PATH only (a rustup default swap
/// leaves every action key unchanged), and a symlinked composition or an `explicit_sysroot_deps`
/// switch hands `buildscript_run`'s `--sysroot`-less `$RUSTC` a std-less compiler, whereupon
/// `build.rs` cfg probes fail into `/dev/null`, codegen silently changes, and buck2 still reports
/// BUILD SUCCEEDED. Reintroducing either defect means editing exactly these two files.
#[test]
fn rust_toolchain_is_hermetic_by_default_with_a_self_sufficient_sysroot() {
    let root = repo_root();
    let toolchains = fs::read_to_string(root.join("toolchains/BUCK"))
        .expect("read system toolchain declarations");
    let defs = fs::read_to_string(root.join("toolchains/rust/defs.bzl"))
        .expect("read hermetic Rust toolchain rule");

    assert!(
        !toolchains.contains("system_rust_toolchain(\n    name = \"rust\",\n"),
        "toolchains//:rust must not be the ambient PATH-resolved compiler"
    );
    assert!(
        toolchains.contains("_RUST_TOOLCHAIN_DEFAULT = rust_toolchain_for_mode(\n    read_root_config(\"oya_toolchain\", \"rust\", \"hermetic\"),\n)"),
        "an absent [oya_toolchain] section must yield the hermetic toolchain, not the ambient one"
    );
    assert!(
        toolchains.contains("\"DEFAULT\": _RUST_TOOLCHAIN_DEFAULT,")
            && toolchains.contains("\"prelude//os:windows\": \":rust_system\","),
        "toolchain_alias must route every non-Windows platform through the configured mode"
    );
    assert_eq!(
        toolchains.matches("**RUST_TOOLCHAIN_CONFIG").count(),
        2,
        "both Rust toolchain declarations must consume one shared configuration, so there is nothing to keep in parity by hand"
    );
    assert!(
        defs.contains("fail(\"[oya_toolchain] rust must be"),
        "an unrecognised toolchain mode must fail at parse time rather than degrade to the ambient compiler"
    );

    assert!(
        defs.contains("ctx.actions.copied_dir(") && !defs.contains("symlinked_dir"),
        "the sysroot must be COPIED: rustc derives its sysroot from the canonicalised path of the loaded driver dylib, and dyld resolves symlinks out of the composed tree"
    );
    for key in [
        "\"bin/rustc\":",
        "\"bin/rustdoc\":",
        "\"bin/clippy-driver\":",
        "\"bin/rustfmt\":",
        "\"lib\":",
    ] {
        assert!(
            defs.contains(key),
            "the composed sysroot must contain a real {key} entry"
        );
    }
    assert!(
        defs.contains("sysroot_path = composed,") && !defs.contains("explicit_sysroot_deps ="),
        "compile and rustdoc actions get --sysroot from sysroot_path; explicit_sysroot_deps would instead point rustc at an empty sysroot"
    );
}

#[test]
fn windows_buck2_toolchain_uses_prelude_msvc_defaults() {
    let root = repo_root();
    let toolchains = fs::read_to_string(root.join("toolchains/BUCK"))
        .expect("read system toolchain declarations");

    assert!(
        toolchains.contains("CXX_TOOLCHAIN_CONFIG = {")
            && toolchains.contains("**CXX_TOOLCHAIN_CONFIG"),
        "both C++ toolchain declarations must consume one shared platform configuration"
    );
    for field in ["compiler", "compiler_type", "linker", "archiver"] {
        assert!(
            toolchains.contains(&format!(
                "\"{field}\": None if host_info().os.is_windows else"
            )),
            "the shared C++ toolchain configuration must own the Windows prelude MSVC {field} default"
        );
    }
    assert_eq!(
        toolchains.matches("**CXX_TOOLCHAIN_CONFIG").count(),
        2,
        "both `cxx` and `cxx_no_default_deps` must consume the same configuration exactly once"
    );
    assert_eq!(
        toolchains.matches("host_info().os.is_windows").count(),
        4,
        "platform conditionals belong only in the shared configuration, never duplicated per toolchain declaration"
    );
}

#[test]
fn affected_set_long_step_telemetry_wraps_long_running_phases() {
    let root = repo_root();
    let wf = workflow_path(&root);
    let workflow =
        fs::read_to_string(&wf).unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));

    assert!(
        affected_set_long_step_telemetry_is_wired(&workflow),
        "affected-set long-running derive/baseline/binding phases must be wrapped by the telemetry helper"
    );

    let without_baseline = workflow.replace(
        "--phase materialize-merge-base-build-health-baseline --",
        "--phase removed --",
    );
    assert!(
        !affected_set_long_step_telemetry_is_wired(&without_baseline),
        "missing baseline materialization telemetry must be detected"
    );
}
