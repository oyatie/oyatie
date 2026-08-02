// gate-registration completeness meta-test (ADR-0515 D2; CICD-DESIGN-PLAN Stage 1B + Pre-mortem
// Scenario-1c "silent-skip false-green" sibling acceptance test).
//
// INVARIANT (two halves — BOTH are required, and for a long time only the first existed):
//
//   (a) INSIDE the fleet: every gate crate directory under `ci/facade/` — EXCEPT the producer
//       (`oya-cloud-ci-accounting-registry-app`, the rust_binary that EMITS the faces, not a gate
//       lane) — MUST be registered as a job lane in `.github/workflows/oya-ci-required.yml`, the
//       single canonical `oya-ci-required` fan-in. A new gate cannot be added without registering
//       it in the required workflow; an in-tree-but-unregistered gate fails this test (it would be
//       a silent false-green one level below the workflow's `needs:` fan-in).
//
//   (b) OUTSIDE the fleet: a gate crate must not LIVE anywhere else. Half (a) equates "gate crate"
//       with "directory under ci/facade/", so its universe is exactly the fleet directory — a gate
//       parked outside it was never a candidate for the check and could not be reported missing.
//       That is not a strictness gap, it is a SCOPE gap, and it hid seventeen `tools/oya-governance-
//       *-app` fitness gates that built, carried tests, and were referenced by ZERO workflow for
//       their entire lifetime. Half (b) closes the complement, so the two together are total: a
//       gate is either in the fleet and registered, or it does not exist.
//
// The `capability: fitness-*` facet in `registry/catalog/<crate>.yaml` — not the crate NAME — is
// what identifies a gate crate for half (b). Keying on the name would let a rename dodge the check;
// the catalog row is the born-accounting SSOT the crate cannot exist without.
//
// It is a pure filesystem + text gate: it reads the gates dir and greps the workflow yaml. No
// network, no GitHub API — runnable in any presubmit. Keep it deterministic and surface-all
// (it collects EVERY unregistered gate, not just the first).
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
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
///
/// This list is an escape hatch from the invariant, so it is itself gated: every entry must have
/// NO `ci-<crate>-gate` Buck target, enforced by
/// [`non_gate_exclusions_are_falsifiable_against_the_build_graph`]. A real gate cannot be silenced
/// by appending its name here. Note these crates are NOT unrun: `//ci/...` in the required `buck2`
/// job runs their `-unittest` targets, and planning-projection's library code additionally runs
/// inside the bespoke `gate-generated-artifact-freshness` lane that depends on it.
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

/// The fan-in's success CONDITIONAL — the `if [ … ] && [ … ]; then` line, with shell
/// line-continuations folded and comment lines dropped.
///
/// This is the only place a `needs.<job>.result` value is load-bearing. The fan-in runs
/// `if: always()`, so a job whose result is never COMPARED here cannot make the required context
/// red no matter how it fails.
fn fan_in_success_conditional(fan_in_block: &str) -> String {
    workflow_steps(fan_in_block)
        .iter()
        .flat_map(|step| executable_lines(step))
        .find(|line| line.starts_with("if ") && line.contains("; then"))
        .unwrap_or_default()
}

/// True iff the fan-in both DEPENDS ON `job` (`needs:` membership) and CHECKS its result inside
/// the success conditional.
///
/// The second half used to be `fan_in_block.contains("needs.{job}.result")` — a substring search
/// over the whole block, which the step's own diagnostic
/// `echo "  buck2 = ${{ needs.buck2.result }}"` satisfies all by itself. Deleting the real
/// `&& [ "${{ needs.buck2.result }}" = "success" ]` clause therefore left every assertion GREEN
/// while the required context would go green with the `buck2` job FAILING — the entire gate fleet
/// blacked out with no verdict. `fan_in_membership_requires_a_checked_result_not_an_echo` pins
/// that mutation as RED.
fn fan_in_mentions_job(fan_in_block: &str, job: &str) -> bool {
    fan_in_block
        .lines()
        .filter_map(yaml_list_item)
        .any(|v| v == job)
        && fan_in_success_conditional(fan_in_block)
            .contains(&format!("needs.{job}.result }}}}\" = \"success\""))
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

fn merge_base_test_health_forwards_the_owned_rustup_home(workflow: &str) -> bool {
    let job = workflow_job(workflow, "gate-affected-target-set");
    let Some(step) = workflow_steps(&job).into_iter().find(|step| {
        step.iter().any(|line| {
            line.trim()
                == "- name: Materialize merge-base build + test baselines when affected-set needs FULL"
        })
    }) else {
        return false;
    };
    let step = step.join("\n");
    step.contains("--phase materialize-merge-base-test-health-baseline --")
        && step.lines().any(|line| {
            line.trim()
                == "buck2 test //... --keep-going -- --env \"RUSTUP_HOME=${RUSTUP_HOME}\" \\"
        })
}

/// The job names the fan-in actually joins on, read from its `needs:` list. A job outside this
/// list has no admission authority, so nothing it executes can register a gate.
fn fan_in_needs(workflow: &str) -> Vec<String> {
    let block = fan_in_block(workflow);
    let mut jobs = Vec::new();
    let mut in_needs = false;
    for line in block.lines() {
        if line.trim_end() == "    needs:" {
            in_needs = true;
            continue;
        }
        if !in_needs {
            continue;
        }
        // The list ends at the first non-list line (`    steps:`).
        match yaml_list_item(line) {
            Some(job) if !job.is_empty() && !job.contains(':') => jobs.push(job),
            _ => break,
        }
    }
    jobs
}

/// Fold shell line-continuations and DROP comment lines, so a `#`-commented target pattern can
/// never be mistaken for an executed one. This is the specific false-green the retired
/// `is_buck_gate` permitted: it was `workflow.contains("//ci/facade/<crate>:")`, satisfied by any
/// literal mention anywhere in the file — including a comment listing target names while nothing
/// ran. Coverage evidence must come from an executable line.
fn executable_lines(step: &[&str]) -> Vec<String> {
    let mut folded: Vec<String> = Vec::new();
    let mut pending = String::new();
    for line in step {
        let code = line.trim();
        if code.starts_with('#') {
            continue;
        }
        match code.strip_suffix('\\') {
            Some(head) => {
                pending.push_str(head);
                pending.push(' ');
            }
            None => {
                pending.push_str(code);
                folded.push(std::mem::take(&mut pending));
            }
        }
    }
    if !pending.is_empty() {
        folded.push(pending);
    }
    folded
}

/// Every Buck2 target PATTERN this workflow actually executes under `buck2 test`, restricted to
/// jobs the fan-in joins on.
///
/// Two exclusions, both deliberate. A `--keep-going` or `|| true` invocation is a BASELINE
/// measurement whose exit code is discarded by construction (the affected-set lane's merge-base
/// pass runs `buck2 test //... --keep-going ... || true`); counting it would make this whole
/// meta-test vacuous, because `//...` covers every gate while binding nothing.
fn executed_patterns_by_job(workflow: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut by_job = BTreeMap::new();
    for job in fan_in_needs(workflow) {
        let block = workflow_job(workflow, &job);
        let mut patterns = BTreeSet::new();
        for step in workflow_steps(&block) {
            for line in executable_lines(&step) {
                if line.contains("--keep-going") || line.contains("|| true") {
                    continue;
                }
                let Some((_, args)) = line.split_once("buck2 test ") else {
                    continue;
                };
                // Stop at the first statement separator. The `gate` job's pwsh step is ONE line
                // that chains `buck2 test $targets; …; & buck2 run //ci/facade/…-gate-bin`, and
                // without this the `buck2 run` target is harvested as though `buck2 test` had run
                // it — which would let a `buck2 run` of a test-less package "cover" that package.
                let args = args.split(';').next().unwrap_or(args);
                patterns.extend(
                    args.split_whitespace()
                        .filter(|token| token.starts_with("//"))
                        .map(|token| token.trim_matches('"').to_owned()),
                );
            }
        }
        if !patterns.is_empty() {
            by_job.insert(job, patterns);
        }
    }
    by_job
}

fn executed_buck2_test_patterns(workflow: &str) -> BTreeSet<String> {
    executed_patterns_by_job(workflow)
        .into_values()
        .flatten()
        .collect()
}

/// True iff a Buck2 target pattern executes the targets of repo-relative package `pkg`.
/// `//ci/...` (and `//...`) recurse; `//ci/facade/x:tgt` names one package.
fn pattern_covers_package(pattern: &str, pkg: &str) -> bool {
    let body = pattern.strip_prefix("//").unwrap_or(pattern);
    match body.strip_suffix("...") {
        Some(root) => {
            let root = root.trim_end_matches('/');
            root.is_empty() || pkg == root || pkg.starts_with(&format!("{root}/"))
        }
        None => body.split(':').next() == Some(pkg),
    }
}

/// True iff the package's BUCK declares at least one test rule.
///
/// This is the load-bearing half of recursive coverage: `buck2 test //ci/...` executes NOTHING
/// for a package that declares no test target, so pattern coverage alone would green a gate that
/// never runs. That is the dark-gate shape the collapse could otherwise introduce.
fn buck_declares_a_test_rule(buck: &Path) -> bool {
    let Ok(text) = fs::read_to_string(buck) else {
        return false;
    };
    text.lines().any(|line| {
        let t = line.trim_start();
        !t.starts_with('#')
            && t.split_once('(')
                .is_some_and(|(rule, _)| rule.trim_end().ends_with("_test"))
    })
}

/// RED proof for the executed-target-set registration invariant. Each arm is a false-green this
/// meta-test must reject; without them "registered" would be satisfiable without executing
/// anything, which is what the retired substring version allowed.
#[test]
fn registration_evidence_must_come_from_a_binding_execution() {
    let fixture = "\
jobs:
  buck2:
    steps:
      - name: consolidated
        run: |
          buck2 test //ci/... --unstable-write-invocation-record /tmp/r.json
  orphan:
    steps:
      - name: not joined by the fan-in
        run: buck2 test //ci/facade/orphan-only:ci-orphan-only-gate
  baseline:
    steps:
      - name: discarded exit code
        run: buck2 test //... --keep-going > /tmp/log 2>&1 || true
  oya-ci-required:
    needs:
      - buck2
      - baseline
    steps:
      - name: verdict
        run: echo done
";
    let executed = executed_buck2_test_patterns(fixture);

    // GREEN: the one pattern run by a fan-in-joined job on a binding line.
    assert!(executed.contains("//ci/..."), "executed: {executed:?}");
    // RED 1: an orphan job is not admission authority, so its pattern must not register a gate,
    // and a `--keep-going || true` baseline pass must not either.
    assert!(
        !executed
            .iter()
            .any(|p| p.contains("orphan-only") || p.contains("//...")),
        "orphan-job and discarded-exit-code patterns must not count: {executed:?}"
    );
    assert_eq!(executed.len(), 1, "executed: {executed:?}");

    // RED 2: a COMMENTED pattern registers nothing. This is the exact defect in the retired
    // `is_buck_gate` — `workflow.contains("//ci/facade/<crate>:")` matched a comment, so pasting
    // a list of target names into the YAML passed while nothing ran.
    let commented = "\
jobs:
  buck2:
    steps:
      - name: comment only
        run: |
          # buck2 test //ci/facade/ghost:ci-ghost-gate
          echo nothing
  oya-ci-required:
    needs:
      - buck2
    steps:
      - name: verdict
        run: echo done
";
    assert!(
        executed_buck2_test_patterns(commented).is_empty(),
        "a commented-out target pattern must never be registration evidence"
    );

    // RED 3: a `buck2 run` chained after `buck2 test` on the SAME line (the surviving `gate`
    // job's pwsh step does exactly this) is not a test execution and must not be harvested as one.
    let chained = "\
jobs:
  gate:
    steps:
      - name: chained
        run: '& buck2 test //ci/facade/a:t; if ($x) { & buck2 run //ci/facade/b:some-bin -- --x }'
  oya-ci-required:
    needs:
      - gate
    steps:
      - name: verdict
        run: echo done
";
    let chained = executed_buck2_test_patterns(chained);
    assert!(chained.contains("//ci/facade/a:t"), "chained: {chained:?}");
    assert!(
        !chained.iter().any(|p| p.contains("some-bin")),
        "a `buck2 run` target must not be harvested as a test pattern: {chained:?}"
    );

    // Pattern coverage semantics: recursive patterns recurse, exact patterns do not.
    assert!(pattern_covers_package("//ci/...", "ci/facade/x"));
    assert!(pattern_covers_package("//...", "ci/facade/x"));
    assert!(pattern_covers_package("//ci/facade/x:ci-x-gate", "ci/facade/x"));
    assert!(!pattern_covers_package("//ci/facade/x:ci-x-gate", "ci/facade/y"));
    // Prefix matching must respect path segments: `//cider/...` must not cover `ci/facade/x`.
    assert!(!pattern_covers_package("//cider/...", "ci/facade/x"));

    // RED 4: recursive coverage is not execution. A package with no test rule runs nothing under
    // `buck2 test //ci/...`, which is the dark-gate shape the collapse could otherwise introduce.
    let root = repo_root();
    assert!(buck_declares_a_test_rule(
        &root.join("ci/facade/baseline-ratchet/BUCK")
    ));
    assert!(!buck_declares_a_test_rule(&root.join("toolchains/BUCK")));
    assert!(!buck_declares_a_test_rule(&root.join("does/not/exist/BUCK")));
}

/// RED proof for `fan_in_mentions_job`, and the reason this rework exists.
///
/// The fan-in step ECHOES every `needs.<job>.result` for diagnostics and then COMPARES them in an
/// `if [ … ] && [ … ]; then` chain. Only the comparison is load-bearing — the fan-in is
/// `if: always()`, so a job missing from the chain can fail while the required context goes green.
///
/// The retired helper substring-searched the whole fan-in block, so the echo alone satisfied it:
/// deleting the real `&& [ "${{ needs.buck2.result }}" = "success" ]` clause left every assertion
/// in this file GREEN while the entire gate fleet could black out with no verdict. That hole was
/// pre-existing, but the collapse makes it load-bearing — before, blocking power was split across
/// `needs.gate.result` and `needs.buck2.result`; after, it rests on the `buck2` clause alone.
#[test]
fn fan_in_membership_requires_a_checked_result_not_an_echo() {
    // The real shape: echoed AND compared. Must pass.
    let checked = "\n  oya-ci-required:
    if: ${{ always() }}
    needs:
      - buck2
    steps:
      - name: Fan-in verdict
        run: |
          echo \"  buck2 = ${{ needs.buck2.result }}\"
          if [ \"${{ needs.buck2.result }}\" = \"success\" ]; then
            exit 0
          fi
          exit 1
";
    assert!(
        fan_in_mentions_job(fan_in_block(checked), "buck2"),
        "a job that is both depended on and compared in the success chain must register"
    );

    // THE MUTATION (verifier RED-C): the comparison is deleted, the diagnostic echo remains.
    // The retired substring helper returned true here. It must now be FALSE.
    let echo_only = "\n  oya-ci-required:
    if: ${{ always() }}
    needs:
      - buck2
    steps:
      - name: Fan-in verdict
        run: |
          echo \"  buck2 = ${{ needs.buck2.result }}\"
          if [ \"${{ needs.gate.result }}\" = \"success\" ]; then
            exit 0
          fi
          exit 1
";
    assert!(
        !fan_in_mentions_job(fan_in_block(echo_only), "buck2"),
        "a job whose result is only ECHOED, never compared, must NOT count as checked — the \
         fan-in is `if: always()`, so its failure would not make the required context red"
    );

    // `needs:` membership is still required: comparing a job the fan-in does not depend on is a
    // race, not a join (the result would be empty).
    let not_needed = "\n  oya-ci-required:
    if: ${{ always() }}
    needs:
      - gate
    steps:
      - name: Fan-in verdict
        run: |
          if [ \"${{ needs.buck2.result }}\" = \"success\" ]; then
            exit 0
          fi
          exit 1
";
    assert!(
        !fan_in_mentions_job(fan_in_block(not_needed), "buck2"),
        "a compared-but-not-depended-on job must NOT count — it is not joined"
    );

    // The LIVE workflow must satisfy the strengthened form for the lane the collapse makes
    // load-bearing. Without this, the fixtures above could pass while dev regressed.
    let workflow = read_to_string(&workflow_path(&repo_root()));
    let block = fan_in_block(&workflow);
    for job in ["buck2", "gate", "gate-baseline-ratchet"] {
        assert!(
            fan_in_mentions_job(block, job),
            "live fan-in must both depend on and compare `needs.{job}.result`"
        );
    }
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

/// Gate crates that still live OUTSIDE `ci/facade/`. FROZEN and shrink-only: a NEW `fitness-*`
/// crate born outside the fleet is blocking, and a listed crate that is moved or deleted must
/// shrink this array in the SAME PR. Each remaining entry is an open disposition question (move
/// into the fleet, refactor to a library + Buck2 test, rewrite, or delete), NOT an accepted home.
///
/// Nine siblings — the ADR-0109 `oya-governance-*-lifecycle-app` set — were replaced wholesale by
/// the single parameterized `ci/facade/lifecycle-status` lane, which is why they are absent here.
const GATE_CRATES_OUTSIDE_THE_FLEET: [&str; 8] = [
    "tools/oya-governance-adapter-with-no-importer-app",
    "tools/oya-governance-adr-shape-app",
    "tools/oya-governance-authoritative-tracked-app",
    "tools/oya-governance-banned-primitives-app",
    "tools/oya-governance-portfolio-citation-app",
    "tools/oya-governance-predictable-naming-app",
    "tools/oya-governance-purpose-audit-app",
    "tools/oya-governance-sunset-lifecycle-app",
];

/// Roots that may hold crates but are NOT the gate fleet. A `fitness-*` capability found here is a
/// gate living outside its home.
const NON_FLEET_CRATE_ROOTS: [&str; 1] = ["tools"];

/// True when `<crate>`'s born-accounting catalog row declares a `fitness-*` capability — the facet
/// every governance gate crate carries and no other `tools/` crate does.
fn declares_fitness_capability(root: &Path, crate_name: &str) -> bool {
    let catalog = root.join(format!("registry/catalog/{crate_name}.yaml"));
    let Ok(text) = fs::read_to_string(&catalog) else {
        return false;
    };
    text.lines().any(|line| {
        line.split('#')
            .next()
            .unwrap_or("")
            .trim()
            .strip_prefix("capability:")
            .is_some_and(|value| value.trim().starts_with("fitness-"))
    })
}

fn gate_crates_outside_the_fleet(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for non_fleet_root in NON_FLEET_CRATE_ROOTS {
        let dir = root.join(non_fleet_root);
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries {
            let path = entry.expect("dir entry").path();
            if !path.join("Cargo.toml").is_file() {
                continue;
            }
            let name = path
                .file_name()
                .expect("dir file_name")
                .to_string_lossy()
                .into_owned();
            if declares_fitness_capability(root, &name) {
                found.push(format!("{non_fleet_root}/{name}"));
            }
        }
    }
    found.sort();
    found
}

/// Half (b): the complement of the registration invariant. A gate crate parked outside
/// `ci/facade/` is invisible to `every_gate_crate_is_registered_in_oya_ci_required_workflow`,
/// whose universe IS the fleet directory — so an unregistered gate there is not merely unenforced,
/// it is unreportable. This freezes the known set shrink-only in both directions.
#[test]
fn no_new_gate_crate_is_born_outside_the_registered_gate_fleet() {
    let root = repo_root();
    let found = gate_crates_outside_the_fleet(&root);
    let frozen: BTreeSet<&str> = GATE_CRATES_OUTSIDE_THE_FLEET.iter().copied().collect();
    let live: BTreeSet<&str> = found.iter().map(String::as_str).collect();

    let born: Vec<&&str> = live.difference(&frozen).collect();
    assert!(
        born.is_empty(),
        "gate crate(s) born OUTSIDE the registered gate fleet: {born:?}\n\
         A crate whose catalog row declares a `fitness-*` capability is a gate; it belongs under \
         ci/facade/ with a matrix leg in .github/workflows/oya-ci-required.yml. Outside the fleet \
         it is unreachable by the registration invariant and enforces nothing."
    );

    let stale: Vec<&&str> = frozen.difference(&live).collect();
    assert!(
        stale.is_empty(),
        "GATE_CRATES_OUTSIDE_THE_FLEET is stale — {stale:?} no longer exist(s). Shrink the frozen \
         array in the SAME PR that moved or deleted them, or the ratchet silently regains headroom."
    );
}

/// The discriminator must key on the catalog facet, not on the crate name, or a rename defeats it.
#[test]
fn the_outside_fleet_discriminator_reads_the_catalog_facet_not_the_crate_name() {
    let root = repo_root();
    assert!(
        declares_fitness_capability(&root, "oya-governance-adr-shape-app"),
        "a governance gate's catalog row declares a fitness-* capability"
    );
    assert!(
        !declares_fitness_capability(&root, "oya-reorg-codemod-app"),
        "a non-gate tools/ crate must not be swept up by the discriminator"
    );
    assert!(
        !declares_fitness_capability(&root, "oya-governance-adr-status-lifecycle-app"),
        "a crate with no catalog row at all is not a gate — absence must not read as fitness"
    );
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

    // ── REGISTRATION IS EXECUTION (2026-08-01, the 48->2 matrix collapse).
    //
    // This used to be three substring searches over the workflow TEXT: `-p <crate>`, a
    // `crate: <crate>,` matrix line, or `//ci/facade/<crate>:` ANYWHERE in the file. All three
    // were matrix-shaped, and the third was satisfiable by a COMMENT — the cheapest way to pass
    // it was to paste a list of target names while nothing executed, which is precisely the
    // false-green it claimed to prevent. Its docstring said "every gate crate must be
    // registered"; what it enforced was "every ci/facade/ subdir is mentioned in a YAML file".
    //
    // It now asserts the property that actually matters: every gate crate is EXECUTED. A gate is
    // registered iff some job the FAN-IN JOINS ON runs a `buck2 test` pattern that covers the
    // crate's package, from an executable (non-comment, non-`--keep-going`, non-`|| true`) line,
    // AND the crate's BUCK declares a test rule for that pattern to match.
    let executed = executed_buck2_test_patterns(&workflow);
    assert!(
        !executed.is_empty(),
        "no fan-in-reachable job in {} executes any `buck2 test <//pattern>` — the gate fleet has \
         no binding execution at all",
        wf.display()
    );

    // Surface-all: collect EVERY unexecuted gate, then assert the set is empty.
    let mut uncovered: Vec<String> = Vec::new();
    let mut no_test_rule: Vec<String> = Vec::new();
    for crate_dir in &crates {
        if NON_GATE_CRATES.contains(&crate_dir.as_str()) {
            continue;
        }
        let pkg = format!("ci/facade/{crate_dir}");
        if !executed
            .iter()
            .any(|pattern| pattern_covers_package(pattern, &pkg))
        {
            uncovered.push(crate_dir.clone());
        } else if !buck_declares_a_test_rule(&root.join(&pkg).join("BUCK")) {
            no_test_rule.push(crate_dir.clone());
        }
    }

    assert!(
        uncovered.is_empty(),
        "gate crate(s) present under {} but executed by NO fan-in-reachable `buck2 test` pattern \
         in {}: {:?}\n\
         Executed patterns: {:?}\n\
         An in-tree-but-unexecuted gate is a silent false-green one level below the fan-in.",
        gates.display(),
        wf.display(),
        uncovered,
        executed
    );
    assert!(
        no_test_rule.is_empty(),
        "gate crate(s) under {} are covered by an executed target PATTERN but their BUCK declares \
         no `*_test` rule, so `buck2 test` runs nothing for them: {:?}\n\
         Recursive coverage is not execution — add the gate's `rust_test` target.",
        gates.display(),
        no_test_rule
    );
}

/// `NON_GATE_CRATES` is the ONE hole in the registration invariant above, and until this test it
/// was an unfalsifiable ASSERTION: nothing stopped a real gate lane from being permanently
/// silenced by appending its directory name to that list — the exact "declared-but-unregistered
/// machinery silently no-ops" failure the registration meta-test exists to prevent, reintroduced
/// one level up in the meta-test's own escape hatch.
///
/// This anchors each exclusion to the BUILD GRAPH, where "is this a gate lane?" is objectively
/// answerable instead of self-declared. A gate lane is exactly a crate whose BUCK declares
/// `ci-<dir>-gate` — the target the required workflow's matrix leg expands to
/// (`//ci/facade/{0}:ci-{0}-gate`). A crate with no such target CANNOT be run as a matrix leg at
/// all (buck2 exits BUILD FAILED on the unknown target), which is the positive, checkable reason
/// it is excluded rather than merely unregistered.
///
/// So: if a `-gate` target ever appears in an excluded crate, that crate IS a gate lane and MUST
/// be registered in the required workflow — this test REDs and says so, instead of the new gate
/// never running and every subsequent merge going unchecked by it.
///
/// The converse (every NON-excluded crate declares a `-gate` target) is deliberately NOT asserted:
/// `affected-target-set` and `generated-artifact-freshness` are legitimately registered as bespoke
/// jobs that drive `rust_binary` targets, not `-gate` rust_tests.
#[test]
fn non_gate_exclusions_are_falsifiable_against_the_build_graph() {
    let root = repo_root();
    let gates = gates_dir(&root);

    let mut wrongly_excluded: Vec<String> = Vec::new();
    for crate_dir in NON_GATE_CRATES {
        let buck = gates.join(crate_dir).join("BUCK");
        // Same sanity check PRODUCER_CRATE gets: a stale or misspelled exclusion silently widens
        // the hole (it excludes nothing, but hides that the name no longer resolves).
        assert!(
            buck.is_file(),
            "NON_GATE_CRATES lists `{crate_dir}`, but {} does not exist — the exclusion is stale \
             (crate renamed or removed). Drop the entry or fix the name.",
            buck.display()
        );
        if read_to_string(&buck).contains(&format!("\"ci-{crate_dir}-gate\"")) {
            wrongly_excluded.push(crate_dir.to_owned());
        }
    }

    assert!(
        wrongly_excluded.is_empty(),
        "crate(s) excluded by NON_GATE_CRATES that DO declare a `ci-<crate>-gate` Buck target, so \
         they are real gate lanes being silently skipped by the registration invariant: {:?}\n\
         Either register each one in {} (matrix leg `- {{ crate: <crate>, label: \"gate · … \" }}`, \
         a bespoke `-p <crate>` job, or a `//ci/facade/<crate>:` Buck target job) and remove it \
         from NON_GATE_CRATES, or delete the `-gate` target if it was never meant to be a lane.",
        wrongly_excluded,
        workflow_path(&root).display()
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
    let executed = executed_buck2_test_patterns(&workflow);
    for (gate_id, input_kind) in configured {
        let has_required_lane = match input_kind.as_str() {
            // Resolve the gate id to its NEW de-branded ci/facade dir via the committed move-plan
            // SSOT (the ci keystone move renamed the crates semantically), then require that
            // dir's package to be covered by a pattern a fan-in-reachable job actually executes.
            //
            // This used to be `workflow.contains("crate: {dir},")` — a matrix line. The 48->2
            // collapse (2026-08-01) removed those lines; the gates are now executed by
            // `buck2 test //ci/...` in the `buck2` lane. Asserting execution rather than a matrix
            // line is also what this check always MEANT by "required workflow authority".
            "producer-face" => ci_move_new_dir(&root, &format!("oya-{gate_id}-app")).is_some_and(
                |dir| {
                    let pkg = format!("ci/facade/{dir}");
                    executed
                        .iter()
                        .any(|pattern| pattern_covers_package(pattern, &pkg))
                },
            ),
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

    // ── COMPLEMENTARY to `every_gate_crate_is_registered_...`, not a duplicate of it.
    //
    // That test proves each gate is EXECUTED by some job in the fan-in's `needs:`. This one
    // proves the executing job is also JOINED BY THE VERDICT: the fan-in runs `if: always()`, so
    // a job that sits in `needs:` but whose result is never compared in the success chain can
    // fail while the required context still goes green. Being depended on is not the same as
    // being checked — `fan_in_mentions_job` requires both, and
    // `fan_in_membership_requires_a_checked_result_not_an_echo` proves it requires both by
    // pinning the echo-only mutation as RED. (That claim was previously asserted in this comment
    // while the helper only substring-searched the whole block, which the diagnostic `echo` line
    // satisfied — a false comment guarding a real hole.)
    //
    // The retired version mapped a gate crate to a job by SUBSTRING (`fan_in_block.contains(
    // &short)` after stripping `oya-cloud-ci-`/`-app`), which matched any incidental mention in
    // the block — including a comment — and was matrix-shaped besides.
    let crates = gate_crate_dirs(&gates);
    let by_job = executed_patterns_by_job(&workflow);
    let mut missing: Vec<String> = Vec::new();
    for crate_dir in &crates {
        if NON_GATE_CRATES.contains(&crate_dir.as_str()) {
            continue;
        }
        let pkg = format!("ci/facade/{crate_dir}");
        let checked_by: Vec<&String> = by_job
            .iter()
            .filter(|(_, patterns)| {
                patterns
                    .iter()
                    .any(|pattern| pattern_covers_package(pattern, &pkg))
            })
            .map(|(job, _)| job)
            .filter(|job| fan_in_mentions_job(fan_in_block, job))
            .collect();
        if checked_by.is_empty() {
            missing.push(crate_dir.clone());
        }
    }

    assert!(
        missing.is_empty(),
        "gate lane(s) executed by NO job that the `oya-ci-required` fan-in both depends on AND \
         checks via `needs.<job>.result` in {}: {:?}\n\
         The fan-in runs `if: always()`, so an unchecked job's failure cannot make the required \
         context red.",
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
    // Assert the MECHANISM (runner comes from the matrix, with a fallback), not which runner
    // label happens to back the fallback today. This previously pinned the literal
    // `|| 'ubuntu-latest'`, which made it fail the moment the required lane moved to the owned
    // arm64 fleet — an infrastructure change that does not touch this guard's actual invariant.
    // The `matrix.os` hatch is the load-bearing part: it is how the one windows-latest leg keeps
    // requesting Windows while every other leg takes the default runner. The windows entry itself
    // is asserted separately below, so pinning the fallback here bought no coverage.
    assert!(
        gate_job.contains("runs-on: ${{ matrix.os || '"),
        "the reusable gate matrix must select its runner from the matrix (runs-on: \
         ${{{{ matrix.os || '<default>' }}}}), so a per-leg `os:` can override the default"
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

// ===========================================================================
// (c) QUALITY-LANE TARGET RESOLVABILITY.
//
// Halves (a) and (b) above answer "is every gate CRATE registered somewhere".
// They say nothing about the other direction: `registry/quality/lanes.yaml`
// declares 90+ lanes with `status: active`, an `owner_team`, an ADR `source`
// and a `check_command` — and NOTHING ever resolved that command against a
// thing that exists. The registry's own validator (`oya-check-quality-lane`,
// `QualityLaneError::CheckCommandNotWired`) only substring-matches the command
// against `oya-governance-gate-catalog-domain::all_canonical_commands_rendered()`
// — a second hand-maintained list. Two hand-maintained lists agreeing with each
// other is a tautology, not enforcement: when the dev-cli package was renamed
// `oya-dev-cli` -> `marketplace-dev-cli`, BOTH lists kept the dead name and the
// quality-lanes gate stayed green.
//
// This half resolves the DECLARED TARGET of every active lane against the tree:
//   - `gate validate <lane>`  -> `<lane>` must be a real dispatch arm in the
//                                dev-cli gate dispatcher SOURCE (not a mirror list);
//   - `cargo run -p <pkg>`    -> some in-tree Cargo.toml must declare `<pkg>`;
//   - `buck2 ... //cell:tgt`  -> `<cell>/BUCK` must declare target `tgt`;
//   - a repo-relative script  -> that file must exist;
//   - a bare cargo/buck2 toolchain verb resolves by definition.
//
// Pure filesystem + text, deterministic, surface-all — same contract as (a)/(b),
// and it rides the same already-required target
// `//ci/facade/baseline-ratchet:ci-baseline-ratchet-gate-registration`.
// ===========================================================================

/// Targets that are declared `status: active` in the lane registry but do not exist in the tree.
/// This is a SHRINK-ONLY hatch, keyed on the TARGET (one entry can cover many lanes), and it is
/// itself falsifiable: [`known_unresolvable_lane_targets_are_still_unresolvable`] fails the moment
/// an entry starts resolving, so the hatch cannot outlive the defect it documents.
///
/// Retiring an ADR-mandated lane is a governance act, not a way to get green — so these lanes are
/// deliberately left `status: active` and the breakage is recorded here where the required fan-in
/// can see it, instead of being flipped to `planned`.
const KNOWN_UNRESOLVABLE_LANE_TARGETS: [(&str, &str); 2] = [
    (
        "cargo-package:oya-vcs-merge-queue-fix-loop-app",
        "Lane `oya-governance-merge-queue-staging-ref-gc` names a package that no longer exists \
         anywhere in-tree; the ADR-0363 retirement of the bespoke VCS ratchet removed it and left \
         the lane pointing at nothing.",
    ),
    (
        "repo-file:tools/governance/adr-0221-governance-gates.sh",
        "The four ADR-0221 hook-efficacy lanes (vacuous-green, orphan-citation, version-pin, \
         buildability-line-count) still name a shell harness that ADR-0523 (zero-shell posture) \
         deleted. There is no Rust replacement: none of the four names is a `gate validate` \
         dispatch arm either, so these lanes enforce nothing. Repair is an owned-Rust port, which \
         is a governance decision, not a registry edit.",
    ),
];

fn quality_lane_registry_path(root: &Path) -> PathBuf {
    root.join("registry/quality/lanes.yaml")
}

/// The dev-cli gate dispatcher SOURCE. Read deliberately instead of any catalog constant: the
/// catalog is a mirror, the `match` is the thing that actually decides whether `gate validate X`
/// runs or falls through to the usage/exit-2 arm.
fn gate_dispatcher_path(root: &Path) -> PathBuf {
    root.join("marketplace/facade/dev-cli/src/commands/gate/mod.rs")
}

/// `(lane_id, check_command)` for every `status: active` lane, in registry order.
///
/// Deliberately a line parser, not a YAML dependency: the registry is a flat two-level list and
/// this test must not acquire a parser that could itself drift from canonical-json/YAML policy.
fn active_quality_lanes(registry: &str) -> Vec<(String, String)> {
    let mut lanes = Vec::new();
    let mut id = String::new();
    let mut status = String::new();
    let mut check_command = String::new();
    let mut flush = |id: &mut String, status: &mut String, cmd: &mut String| {
        if !id.is_empty() && status == "active" {
            lanes.push((std::mem::take(id), std::mem::take(cmd)));
        }
        id.clear();
        status.clear();
        cmd.clear();
    };
    for line in registry.lines() {
        if let Some(value) = line.strip_prefix("  - id: ") {
            flush(&mut id, &mut status, &mut check_command);
            id = value.trim().to_owned();
        } else if let Some(value) = line.strip_prefix("    status: ") {
            status = value.trim().to_owned();
        } else if let Some(value) = line.strip_prefix("    check_command: ") {
            check_command = value.trim().to_owned();
        }
    }
    flush(&mut id, &mut status, &mut check_command);
    lanes
}

/// Lane names the dev-cli dispatcher actually handles, read out of its `match` arms.
fn gate_dispatch_arms(dispatcher_source: &str) -> BTreeSet<String> {
    const OPEN: &str = "(Some(\"validate\"), Some(\"";
    let mut arms = BTreeSet::new();
    for (index, matched) in dispatcher_source.match_indices(OPEN) {
        let tail = &dispatcher_source[index + matched.len()..];
        if let Some(name) = tail.split('"').next()
            && !name.is_empty()
        {
            arms.insert(name.to_owned());
        }
    }
    arms
}

/// Every cargo package name declared in-tree. Bounded walk: the deepest manifest in the repo sits
/// at depth 6, and the skipped roots hold no first-party manifest.
fn declared_cargo_packages(root: &Path) -> BTreeSet<String> {
    const SKIP: [&str; 6] = [
        ".git",
        "target",
        "buck-out",
        "node_modules",
        ".claude",
        ".omc",
    ];
    let mut names = BTreeSet::new();
    let mut queue = vec![(root.to_path_buf(), 0_usize)];
    while let Some((dir, depth)) = queue.pop() {
        if depth > 7 {
            continue;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                if !SKIP.contains(&name.as_str()) {
                    queue.push((path, depth + 1));
                }
            } else if name == "Cargo.toml"
                && let Ok(text) = fs::read_to_string(&path)
                && let Some(package) = cargo_package_name(&text)
            {
                names.insert(package);
            }
        }
    }
    names
}

/// `[package] name = "x"` -> `x`. Stops at the next section header so a `[dependencies]` entry
/// named `name` can never be mistaken for the package identity.
fn cargo_package_name(manifest: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package && let Some(value) = trimmed.strip_prefix("name") {
            let value = value.trim_start().strip_prefix('=')?.trim();
            return Some(value.trim_matches('"').to_owned());
        }
    }
    None
}

/// What a `check_command` needs to exist before it can run.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum LaneTarget {
    /// `gate validate <lane>` — must be a dispatcher `match` arm.
    GateLane(String),
    /// `cargo run -p <pkg>` — must be a declared package.
    CargoPackage(String),
    /// `//cell/path:target` — `<cell/path>/BUCK` must declare it.
    BuckTarget(String),
    /// A repo-relative script path — must exist.
    RepoFile(String),
}

impl LaneTarget {
    fn key(&self) -> String {
        match self {
            Self::GateLane(v) => format!("gate-lane:{v}"),
            Self::CargoPackage(v) => format!("cargo-package:{v}"),
            Self::BuckTarget(v) => format!("buck-target:{v}"),
            Self::RepoFile(v) => format!("repo-file:{v}"),
        }
    }
}

/// Everything a `check_command` must resolve for the lane to be runnable. A command can carry more
/// than one (`cargo run -p X -- gate validate Y` needs BOTH the package and the dispatch arm) —
/// checking only the last token is how the lane name kept resolving while the package did not.
///
/// A bare toolchain verb (`cargo fmt`, `cargo deny check`, `buck2 test //...`) yields no target and
/// resolves by definition; that is not a silent pass, it is the absence of a repo-local dependency.
fn lane_targets(check_command: &str) -> Vec<LaneTarget> {
    let tokens: Vec<&str> = check_command.split_whitespace().collect();
    let mut targets = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        match *token {
            "-p" | "--package" => {
                if let Some(pkg) = tokens.get(index + 1) {
                    targets.push(LaneTarget::CargoPackage((*pkg).to_owned()));
                }
            }
            "validate" if index > 0 && tokens[index - 1] == "gate" => {
                if let Some(lane) = tokens.get(index + 1) {
                    targets.push(LaneTarget::GateLane((*lane).to_owned()));
                }
            }
            _ if token.starts_with("//") && token.contains(':') => {
                targets.push(LaneTarget::BuckTarget((*token).to_owned()));
            }
            _ if token.contains('/')
                && (token.ends_with(".sh")
                    || token.ends_with(".py")
                    || token.ends_with(".mjs")) =>
            {
                targets.push(LaneTarget::RepoFile((*token).to_owned()));
            }
            _ => {}
        }
    }
    targets
}

fn buck_target_exists(root: &Path, label: &str) -> bool {
    let Some((cell, name)) = label.trim_start_matches('/').split_once(':') else {
        return false;
    };
    let Ok(text) = fs::read_to_string(root.join(cell).join("BUCK")) else {
        return false;
    };
    text.contains(&format!("name = \"{name}\""))
}

fn lane_target_resolves(
    root: &Path,
    target: &LaneTarget,
    arms: &BTreeSet<String>,
    packages: &BTreeSet<String>,
) -> bool {
    match target {
        LaneTarget::GateLane(lane) => arms.contains(lane),
        LaneTarget::CargoPackage(pkg) => packages.contains(pkg),
        LaneTarget::BuckTarget(label) => buck_target_exists(root, label),
        LaneTarget::RepoFile(path) => root.join(path).is_file(),
    }
}

/// `(lane_id, target_key)` for every active lane whose declared target does not exist.
fn unresolvable_active_lane_targets(root: &Path) -> Vec<(String, String)> {
    let registry = read_to_string(&quality_lane_registry_path(root));
    let arms = gate_dispatch_arms(&read_to_string(&gate_dispatcher_path(root)));
    let packages = declared_cargo_packages(root);
    let mut unresolved = Vec::new();
    for (id, command) in active_quality_lanes(&registry) {
        for target in lane_targets(&command) {
            if !lane_target_resolves(root, &target, &arms, &packages) {
                unresolved.push((id.clone(), target.key()));
            }
        }
    }
    unresolved.sort();
    unresolved
}

/// The probe itself must be falsifiable before any of its NEGATIVE results are load-bearing: a
/// parser that silently returns nothing would report a clean tree forever.
#[test]
fn the_lane_resolvability_probe_is_falsifiable_on_known_controls() {
    let root = repo_root();

    let lanes = active_quality_lanes(&read_to_string(&quality_lane_registry_path(&root)));
    assert!(
        lanes.len() > 50,
        "active-lane parser found only {} lanes — the registry shape changed and this gate went blind",
        lanes.len()
    );
    assert!(
        lanes.iter().all(|(_, command)| !command.is_empty()),
        "an active lane parsed with an empty check_command; an empty command resolves vacuously"
    );

    let arms = gate_dispatch_arms(&read_to_string(&gate_dispatcher_path(&root)));
    assert!(
        arms.contains("authority-cohesion") && arms.contains("a11y-discipline"),
        "known-positive control: both lanes have a `(Some(\"validate\"), Some(..))` arm in the dispatcher"
    );
    assert!(
        !arms.contains("vacuous-green"),
        "known-negative control: the ADR-0221 hook-efficacy lanes have no dispatch arm"
    );

    let packages = declared_cargo_packages(&root);
    assert!(
        packages.contains("marketplace-dev-cli"),
        "known-positive control: the renamed gate CLI package must be discovered"
    );
    assert!(
        !packages.contains("oya-dev-cli"),
        "known-negative control: the pre-rename package name must NOT be discovered"
    );

    // Target extraction must see BOTH targets in a compound command, not just the trailing one.
    assert_eq!(
        lane_targets("cargo run -p oya-dev-cli -- gate validate a11y-discipline"),
        vec![
            LaneTarget::CargoPackage("oya-dev-cli".into()),
            LaneTarget::GateLane("a11y-discipline".into()),
        ],
    );
    // ...including in the buck2 form the registry now actually uses, where the binary is a label
    // rather than a `-p` package. If this stopped yielding the BuckTarget the gate would go blind
    // on 80 lanes at once, having just been repaired off the cargo form.
    assert_eq!(
        lane_targets("buck2 run //marketplace/facade/dev-cli:oya -- gate validate a11y-discipline"),
        vec![
            LaneTarget::BuckTarget("//marketplace/facade/dev-cli:oya".into()),
            LaneTarget::GateLane("a11y-discipline".into()),
        ],
    );
    assert_eq!(
        lane_targets("bash tools/governance/adr-0221-governance-gates.sh vacuous-green"),
        vec![LaneTarget::RepoFile(
            "tools/governance/adr-0221-governance-gates.sh".into()
        )],
    );
    assert!(lane_targets("cargo deny check").is_empty());

    assert!(
        buck_target_exists(&root, "//marketplace/facade/dev-cli:oya"),
        "known-positive control: the gate CLI buck2 binary target exists"
    );
    assert!(
        !buck_target_exists(&root, "//marketplace/facade/dev-cli:no-such-target"),
        "known-negative control: a bogus target must not resolve"
    );
}

/// The durable half: an ACTIVE lane must name something that exists. Wiring individual dispatch
/// arms by hand fixes six lanes once; this fixes the class, because the next lane that is declared
/// active against a renamed package, a deleted script, or an unimplemented gate name fails here.
#[test]
fn every_active_quality_lane_resolves_to_a_real_target() {
    let root = repo_root();
    let unresolved = unresolvable_active_lane_targets(&root);
    let known: BTreeSet<&str> = KNOWN_UNRESOLVABLE_LANE_TARGETS
        .iter()
        .map(|(target, _)| *target)
        .collect();

    let regressions: Vec<&(String, String)> = unresolved
        .iter()
        .filter(|(_, target)| !known.contains(target.as_str()))
        .collect();

    assert!(
        regressions.is_empty(),
        "quality lane(s) declared `status: active` name a target that does not exist: {regressions:#?}\n\
         An active lane must resolve: a `gate validate <lane>` name must be a dispatch arm in \
         marketplace/facade/dev-cli/src/commands/gate/mod.rs, a `-p <pkg>` must be a declared cargo \
         package, a `//cell:target` must be declared in that cell's BUCK, and a script path must \
         exist. Registering a lane without its runnable target is a dark gate — it reports nothing \
         and blocks nothing. Do NOT flip the lane to `status: planned` to clear this; retiring an \
         ADR-mandated lane is a governance act."
    );
}

// ===========================================================================
// (d) OPERATING-CONTRACT HOOK MIRROR.
//
// Same defect class as (c), one artifact over: `docs/AGENTS.md` — the operating
// contract agents are told to read first — carried an "Active hooks" line that
// named FIVE behaviours and one file, `scripts/hooks/guard-pr-merge-review.mjs`,
// and not one of them existed. The `enforcement-liveness` face already resolves
// `.claude/settings.json` -> `tools/hooks/`, but nothing ever resolved the PROSE
// mirror, so the contract described an enforcement posture the repo never had.
//
// A mirror that can drift is the defect; equality against the SSOT is the fix.
// ===========================================================================

fn agents_contract_path(root: &Path) -> PathBuf {
    root.join("docs/AGENTS.md")
}

fn claude_hook_wiring_path(root: &Path) -> PathBuf {
    root.join(".claude/settings.json")
}

/// The `Active hooks` line of the Claude Code appendix — the mirror under test.
fn active_hooks_claim(contract: &str) -> &str {
    contract
        .lines()
        .find(|line| line.starts_with("Active hooks"))
        .unwrap_or("")
}

/// Every `<name>.sh` token in `text`, by basename. Deliberately shape-agnostic: it reads the same
/// out of a JSON `"command"` value and out of a backticked markdown span, so the two sides are
/// comparable without either format being parsed.
fn hook_script_basenames(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for (index, _) in text.match_indices(".sh") {
        let head = &text[..index];
        let start = head
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
            .map_or(0, |i| i + 1);
        if start < head.len() {
            names.insert(format!("{}.sh", &head[start..]));
        }
    }
    names
}

/// The probe must be falsifiable before its equality result is load-bearing.
#[test]
fn the_hook_mirror_probe_is_falsifiable_on_known_controls() {
    let root = repo_root();

    assert_eq!(
        hook_script_basenames(
            r#"{ "command": "tools/hooks/no-cargo-enforcer.sh" } and `adr-orphan-detect.sh`"#
        ),
        BTreeSet::from([
            "adr-orphan-detect.sh".to_owned(),
            "no-cargo-enforcer.sh".to_owned(),
        ]),
        "the extractor must read a JSON command value and a markdown span identically"
    );
    assert!(hook_script_basenames("no hooks named here").is_empty());

    let contract = read_to_string(&agents_contract_path(&root));
    let claim = active_hooks_claim(&contract);
    assert!(
        !claim.is_empty(),
        "the Claude Code appendix must still carry an `Active hooks` line; if it was renamed this \
         gate went blind and the mirror is unguarded again"
    );
    assert!(
        !hook_script_basenames(claim).is_empty(),
        "known-positive control: the claim line must name at least one hook script"
    );
}

/// The durable half: the contract's hook list must equal the wiring, in both directions. A name in
/// the doc that is not wired is the original defect (a hook readers believe runs and does not); a
/// wired hook missing from the doc is the same drift facing the other way.
#[test]
fn the_operating_contract_hook_list_equals_the_claude_wiring() {
    let root = repo_root();
    let claimed = hook_script_basenames(active_hooks_claim(&read_to_string(
        &agents_contract_path(&root),
    )));
    let wired = hook_script_basenames(&read_to_string(&claude_hook_wiring_path(&root)));

    let unwired: Vec<&String> = claimed.difference(&wired).collect();
    let undocumented: Vec<&String> = wired.difference(&claimed).collect();

    assert!(
        unwired.is_empty() && undocumented.is_empty(),
        "docs/AGENTS.md `Active hooks` has drifted from .claude/settings.json.\n\
         claimed-but-not-wired: {unwired:?}\n\
         wired-but-not-claimed: {undocumented:?}\n\
         The wiring file is the SSOT; the contract line is a mirror. Naming a hook the harness \
         never loads tells every agent an enforcement runs that does not."
    );

    for name in &wired {
        assert!(
            root.join("tools/hooks").join(name).is_file(),
            "{name} is wired in .claude/settings.json but tools/hooks/{name} does not exist"
        );
    }
}

/// The hatch must not outlive the defect. If a documented-broken target starts resolving, the entry
/// is stale and must be deleted in the SAME change that repaired it, or the ratchet silently regains
/// headroom for the next dark lane.
#[test]
fn known_unresolvable_lane_targets_are_still_unresolvable() {
    let root = repo_root();
    let live: BTreeSet<String> = unresolvable_active_lane_targets(&root)
        .into_iter()
        .map(|(_, target)| target)
        .collect();

    let stale: Vec<&str> = KNOWN_UNRESOLVABLE_LANE_TARGETS
        .iter()
        .map(|(target, _)| *target)
        .filter(|target| !live.contains(*target))
        .collect();

    assert!(
        stale.is_empty(),
        "KNOWN_UNRESOLVABLE_LANE_TARGETS is stale — {stale:?} now resolve(s). Drop the entry (and \
         its rationale) in the same change that repaired the target."
    );

    for (target, rationale) in KNOWN_UNRESOLVABLE_LANE_TARGETS {
        assert!(
            rationale.len() > 80,
            "{target} must carry the reason it is unresolvable and what repairing it requires"
        );
    }
}

#[test]
fn merge_base_test_health_uses_the_same_rustup_executor_contract_as_head() {
    let root = repo_root();
    let wf = workflow_path(&root);
    let workflow =
        fs::read_to_string(&wf).unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));

    assert!(
        merge_base_test_health_forwards_the_owned_rustup_home(&workflow),
        "the merge-base test baseline must forward the owned RUSTUP_HOME exactly like the affected-set head test invocation"
    );
    let without_env = workflow.replace(
        "--keep-going -- --env \"RUSTUP_HOME=${RUSTUP_HOME}\"",
        "--keep-going",
    );
    assert!(
        !merge_base_test_health_forwards_the_owned_rustup_home(&without_env),
        "removing the merge-base rustup executor environment must be detected"
    );
}
