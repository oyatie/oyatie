//! Durable third-party/BUCK hand-edit checker and normalizer.
//!
//! Reindeer cannot express the selected aws-lc-sys musl sysroot env block. This
//! Rust checker replaces the retired Python post-processor for active CI gates:
//! by default it verifies the checked-in `third-party/BUCK` already contains the
//! durable selected env without mutating the worktree. `--write` is reserved for
//! the explicit `regen-third-party.sh` regeneration utility after `reindeer
//! buckify` and the durable patch have run.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const THIRD_PARTY_BUCK: &str = "third-party/BUCK";
const WORKFLOW_PATH: &str = ".github/workflows/github-lane-unlocker-ci-cd.yml";
const BUCK_PATH: &str = "BUCK";
const REGEN_SCRIPT_PATH: &str = "scripts/ci/regen-third-party.sh";
const AFFECTED_GATE_PATH: &str = "infra/ci/buck2-affected-gate.sh";
const BUCK2_COMMAND: &str = "buck2 build //:third-party-durable-handedits-check";
const START: &str = "    env = ";
const ANCHOR: &str = "\"CARGO_MANIFEST_LINKS\": \"aws_lc_0_41_0\"";
const END: &str = "    features = [\"prebuilt-nasm\"],";

const BASE_PAIRS: &[(&str, &str)] = &[
    ("CARGO_MANIFEST_LINKS", "aws_lc_0_41_0"),
    ("CARGO_PKG_VERSION_MAJOR", "0"),
    ("CARGO_PKG_VERSION_MINOR", "41"),
    ("CARGO_PKG_VERSION_PATCH", "0"),
    ("CARGO_PKG_VERSION_PRE", ""),
    ("DEBUG", "false"),
    ("LDFLAGS", "-nostartfiles"),
    ("OPT_LEVEL", "3"),
    ("PROFILE", "release"),
];

const MUSL_PAIRS: &[(&str, &str)] = &[
    ("CC_aarch64_unknown_linux_musl", "clang"),
    (
        "CFLAGS_aarch64_unknown_linux_musl",
        "--target=aarch64-unknown-linux-musl -nostdlibinc -isystem $(location toolchains//cxx/clang_hermetic:aarch64-musl-sysroot)/aarch64-linux-musl/include -isystem $(location toolchains//cxx/clang_hermetic:aarch64-musl-sysroot)/aarch64-linux-musl/include/linux",
    ),
];

const COMMENT: &str = r#"            # aws-lc-sys's cc_builder compiler feature-test (memcmp_invalid_stripped_check)
            # LINKS an executable via $CC. Under buck2 the build-script $CC is the prelude
            # cc-shim `clang --ld-path=__ld_shim.sh`, and __ld_shim re-invokes clang as the
            # link driver — which RE-ADDS the C-runtime startfiles (Scrt1.o/crti.o/crtbeginS.o)
            # on top of the complete ld line the outer clang already passed → `ld.lld: error:
            # duplicate symbol: _start/_init/...` on aarch64-linux (darwin ld64 tolerates it).
            # aws-lc-sys explicitly respects LDFLAGS for this probe (cc_builder.rs ~745, "brings
            # us back to parity with CMake" for custom-linker setups), so -nostartfiles makes the
            # OUTER clang omit its CRT; the inner __ld_shim clang adds exactly one set → links
            # clean and the probe still runs. Scoped to this build script only (compile steps are
            # -c, where -nostartfiles is ignored). Verified on the rust-ci image (clang 19.1.7).
            # The durable class-wide fix is a prelude patch (-nostartfiles on the __ld_shim clang);
            # this targeted fixup unblocks aws-lc-sys (and thus all aws-lc-rs Linux binaries) now.
"#;

const MUSL_COMMENT: &str = r#"            # musl-static (#83 musl lane): compile bcm.c against MUSL headers. -nostdlibinc
            # drops glibc /usr/include (whose stdlib.h redirects strtol->__isoc23_strtol, a
            # glibc-2.38 symbol undefined when linking musl). Keep the $(location) sysroot
            # hidden dep behind the musl platform select so default GitHub affected builds
            # do not materialize optional external toolchain proof archives.
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub failures: Vec<String>,
    pub would_update: bool,
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn pair_lines(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(key, value)| format!("            \"{key}\": \"{value}\",\n"))
        .collect::<Vec<_>>()
        .join("")
}

pub fn selected_env() -> String {
    format!(
        "    env = select({{\n        \"root//platforms:libc_musl\": {{\n{}{}{}{}        }},\n        \"DEFAULT\": {{\n{}{}        }},\n    }}),\n",
        COMMENT,
        pair_lines(BASE_PAIRS),
        MUSL_COMMENT,
        pair_lines(MUSL_PAIRS),
        COMMENT,
        pair_lines(BASE_PAIRS)
    )
}

fn env_range(text: &str) -> Result<(usize, usize), String> {
    let anchor = text
        .find(ANCHOR)
        .ok_or_else(|| "aws-lc-sys buildscript env anchor not found".to_owned())?;
    let start = text[..anchor]
        .rfind(START)
        .ok_or_else(|| "aws-lc-sys env start not found".to_owned())?;
    let end = text[anchor..]
        .find(END)
        .map(|offset| anchor + offset)
        .ok_or_else(|| "aws-lc-sys features anchor not found".to_owned())?;
    Ok((start, end))
}

pub fn normalize_text(text: &str) -> Result<String, String> {
    let (start, end) = env_range(text)?;
    let mut updated = String::with_capacity(text.len() + selected_env().len());
    updated.push_str(&text[..start]);
    updated.push_str(&selected_env());
    updated.push_str(&text[end..]);
    Ok(updated)
}

pub fn third_party_failures(text: &str) -> (Vec<String>, bool) {
    let mut failures = Vec::new();
    let normalized = match normalize_text(text) {
        Ok(normalized) => normalized,
        Err(error) => {
            failures.push(error);
            return (failures, false);
        }
    };
    let would_update = normalized != text;
    if would_update {
        failures.push("third-party/BUCK durable aws-lc-sys selected env drifted; run scripts/ci/regen-third-party.sh or assert-third-party-durable-handedits.rs --write after reindeer buckify".to_owned());
    }
    for needle in [
        "env = select({",
        "\"root//platforms:libc_musl\": {",
        "\"DEFAULT\": {",
        "\"LDFLAGS\": \"-nostartfiles\"",
        "\"CC_aarch64_unknown_linux_musl\": \"clang\"",
        "\"CFLAGS_aarch64_unknown_linux_musl\": \"--target=aarch64-unknown-linux-musl -nostdlibinc -isystem $(location toolchains//cxx/clang_hermetic:aarch64-musl-sysroot)/aarch64-linux-musl/include -isystem $(location toolchains//cxx/clang_hermetic:aarch64-musl-sysroot)/aarch64-linux-musl/include/linux\"",
    ] {
        if !text.contains(needle) {
            failures.push(format!("third-party/BUCK durable env missing {needle:?}"));
        }
    }
    let (env_start, env_end) = env_range(text).unwrap_or((0, 0));
    let env_block = &text[env_start..env_end];
    if env_block.starts_with("    env = {\n")
        && env_block.contains("CFLAGS_aarch64_unknown_linux_musl")
    {
        failures.push(
            "third-party/BUCK musl sysroot env must be select-gated, not a raw env map".to_owned(),
        );
    }
    (failures, would_update)
}

fn read(root: &Path, rel: &str, failures: &mut Vec<String>) -> String {
    match fs::read_to_string(root.join(rel)) {
        Ok(text) => text,
        Err(error) => {
            failures.push(format!("{rel}: read failed: {error}"));
            String::new()
        }
    }
}

fn require(condition: bool, failures: &mut Vec<String>, message: impl Into<String>) {
    if !condition {
        failures.push(message.into());
    }
}

fn retired_python_path() -> String {
    ["scripts/ci/", "apply-third-party-", "durable-handedits.py"].concat()
}

fn retired_python_command() -> String {
    format!("python3 {}", retired_python_path())
}

pub fn evaluate(root: &Path) -> Evaluation {
    let mut failures = Vec::new();
    let third_party = read(root, THIRD_PARTY_BUCK, &mut failures);
    let workflow = read(root, WORKFLOW_PATH, &mut failures);
    let buck = read(root, BUCK_PATH, &mut failures);
    let regen = read(root, REGEN_SCRIPT_PATH, &mut failures);
    let affected_gate = read(root, AFFECTED_GATE_PATH, &mut failures);

    let (third_party_failures, would_update) = third_party_failures(&third_party);
    failures.extend(third_party_failures);

    require(
        workflow.contains(BUCK2_COMMAND),
        &mut failures,
        "GitHub lane unlocker workflow must use Buck2-owned third-party durable hand-edit check",
    );
    require(
        !workflow.contains(&retired_python_command()),
        &mut failures,
        "GitHub lane unlocker workflow must not invoke retired Python third-party mutator",
    );
    require(
        !workflow.contains("git diff --exit-code -- third-party/BUCK"),
        &mut failures,
        "GitHub lane unlocker workflow should not rely on mutation-plus-diff for durable hand edits",
    );
    for needle in [
        "third-party-durable-handedits-check",
        "assert-third-party-durable-handedits.rs",
        "third_party_durable_handedits_check.rs",
        "third-party/BUCK",
    ] {
        require(
            buck.contains(needle),
            &mut failures,
            format!("BUCK must reference {needle}"),
        );
    }
    require(
        !buck.contains(&retired_python_path()),
        &mut failures,
        "BUCK must not depend on retired Python third-party mutator",
    );
    require(
        regen.contains("scripts/ci/assert-third-party-durable-handedits.rs")
            && regen.contains("--write"),
        &mut failures,
        "regen-third-party.sh must use the Rust normalizer in --write mode",
    );
    require(
        !regen.contains(&retired_python_command()),
        &mut failures,
        "regen-third-party.sh must not invoke retired Python third-party mutator",
    );
    require(
        affected_gate.contains("//:third-party-durable-handedits-check"),
        &mut failures,
        "buck2-affected-gate.sh must validate third-party/BUCK hand edits with the Buck2 target",
    );
    require(
        !affected_gate.contains(&retired_python_command()),
        &mut failures,
        "buck2-affected-gate.sh must not invoke retired Python third-party mutator",
    );
    Evaluation {
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned(),
        failures,
        would_update,
    }
}

fn render_json(evaluation: &Evaluation) -> String {
    let failures = evaluation
        .failures
        .iter()
        .map(|failure| format!("\"{}\"", json_escape(failure)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"verdict\":\"{}\",\"target\":\"{}\",\"local_static_only\":true,\"live_mutation_performed\":false,\"checker_language\":\"rust\",\"would_update\":{},\"failures\":[{}]}}",
        evaluation.verdict,
        THIRD_PARTY_BUCK,
        if evaluation.would_update {
            "true"
        } else {
            "false"
        },
        failures
    )
}

fn config() -> (PathBuf, bool, bool) {
    let mut json = false;
    let mut write = false;
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--json" => json = true,
            "--write" => write = true,
            unknown => {
                eprintln!("assert-third-party-durable-handedits: unknown argument {unknown}");
                std::process::exit(2);
            }
        }
    }
    let root = env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    (root, json, write)
}

fn main() {
    let (root, json, write) = config();
    if write {
        let path = root.join(THIRD_PARTY_BUCK);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("{THIRD_PARTY_BUCK}: read failed: {error}");
                std::process::exit(1);
            }
        };
        let normalized = match normalize_text(&text) {
            Ok(normalized) => normalized,
            Err(error) => {
                eprintln!("{THIRD_PARTY_BUCK}: {error}");
                std::process::exit(1);
            }
        };
        if normalized != text {
            if let Err(error) = fs::write(&path, normalized) {
                eprintln!("{THIRD_PARTY_BUCK}: write failed: {error}");
                std::process::exit(1);
            }
            println!("updated third-party/BUCK aws-lc-sys musl env select");
        } else {
            println!("third-party/BUCK aws-lc-sys musl env select already current");
        }
    }

    let evaluation = evaluate(&root);
    if json || evaluation.failures.is_empty() {
        println!("{}", render_json(&evaluation));
    }
    if !evaluation.failures.is_empty() {
        if !json {
            eprintln!("third-party-durable-handedits: RED");
            for failure in &evaluation.failures {
                eprintln!("- {failure}");
            }
        }
        std::process::exit(1);
    }
}
