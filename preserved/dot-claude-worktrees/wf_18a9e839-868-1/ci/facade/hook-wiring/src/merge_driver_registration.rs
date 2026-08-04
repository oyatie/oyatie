//! Merge-driver registration liveness.
//!
//! Same predicate class as the hook rows in this crate's root module — *declared automation must
//! resolve to a real command* — applied to the other versioned surface that names executables git
//! will run: `.gitattributes`.
//!
//! ## The defect this exists to make impossible
//!
//! `.gitattributes` binds paths to merge drivers by NAME. Git resolves the name through git
//! CONFIG, which is not versioned, so registration was per-machine and documented only in
//! per-crate READMEs. Measured on this repo: of the three custom drivers, `cargo-lock` and
//! `fixup-ledger` were registered nowhere at all, and `friction-ledger` resolved to a `buck-out/`
//! artifact inside a different worktree that no longer existed. Git does not warn when a
//! `merge=<name>` attribute has no driver behind it — it silently uses the plain text merge that
//! `.gitattributes`' own comments say is wrong for these files.
//!
//! The registration payload now lives in a versioned file
//! ([`REGISTRATION_FILE`]); this kernel is the detector that keeps the two in agreement, on a
//! fresh clone, with `buck-out` empty.
//!
//! ## What is checkable statically
//!
//! Deliberately NOT "the binary is executable right now": [`STABLE_BIN_DIR`] is gitignored, so on
//! any clean checkout that probe is red for a reason that is not a defect — a gate that measures
//! build state instead of wiring state. What is checked is the whole static chain: declared
//! attribute -> versioned registration -> stable non-`buck-out` path -> a `rust_binary` target
//! that actually produces that name -> a command that degrades to git's own merge when the binary
//! is absent.

use std::collections::{BTreeMap, BTreeSet};

use crate::Finding;

/// Versioned registration file, included into `.git/config` via `include.path`.
pub const REGISTRATION_FILE: &str = "tools/hooks/merge-drivers.gitconfig";

/// The repo's stable binary home (ADR-0523 irreducible glue). Gitignored, populated by
/// `buck2 build <target> --out tools/hooks/bin/<name>`.
pub const STABLE_BIN_DIR: &str = "./tools/hooks/bin/";

/// Low-level merge drivers git implements itself; these need no registration.
pub const BUILTIN_DRIVERS: [&str; 3] = ["text", "binary", "union"];

pub const MERGE_DRIVER_VIOLATION_CODES: [&str; 6] = [
    "merge_driver_absent_binary_unguarded",
    "merge_driver_missing_build_target",
    "merge_driver_missing_git_placeholders",
    "merge_driver_orphan_registration",
    "merge_driver_unregistered",
    "merge_driver_unstable_binary_path",
];

fn finding(code: &str, key: &str, remediation: String) -> Finding {
    Finding {
        code: code.to_owned(),
        key: key.to_owned(),
        remediation,
    }
}

/// Driver names bound by `merge=<name>` attributes, built-ins excluded.
///
/// Attribute lines are `<pattern> <attr>...`; the pattern is skipped because a pattern may itself
/// contain `merge=` (a path can be named anything). Patterns with escaped whitespace are not used
/// in this repo and are not modelled.
#[must_use]
pub fn declared_drivers(gitattributes: &str) -> BTreeSet<String> {
    let mut declared = BTreeSet::new();
    for line in gitattributes.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        for token in line.split_whitespace().skip(1) {
            if let Some(name) = token.strip_prefix("merge=")
                && !name.is_empty()
                && !BUILTIN_DRIVERS.contains(&name)
            {
                declared.insert(name.to_owned());
            }
        }
    }
    declared
}

/// `[merge "<name>"]` sections carrying a `driver` command, mapped name -> command.
///
/// A minimal INI reader, sufficient for the one file it parses. A section without a `driver` key
/// is absent from the result, which is correct: it registers nothing.
#[must_use]
pub fn registered_drivers(gitconfig: &str) -> BTreeMap<String, String> {
    let mut registered = BTreeMap::new();
    let mut section: Option<String> = None;
    for line in gitconfig.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(header) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = header
                .trim()
                .strip_prefix("merge")
                .map(str::trim)
                .and_then(|sub| sub.strip_prefix('"'))
                .and_then(|sub| sub.strip_suffix('"'))
                .map(str::to_owned);
            continue;
        }
        let (Some(name), Some((key, value))) = (section.as_ref(), line.split_once('=')) else {
            continue;
        };
        if key.trim() != "driver" {
            continue;
        }
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .unwrap_or(value);
        registered.insert(name.clone(), value.to_owned());
    }
    registered
}

/// The `./tools/hooks/bin/<name>` binary a driver command runs, if it names one.
fn stable_binary(command: &str) -> Option<&str> {
    let start = command.find(STABLE_BIN_DIR)? + STABLE_BIN_DIR.len();
    let rest = &command[start..];
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    let name = &rest[..end];
    (!name.is_empty()).then_some(name)
}

/// `buck_binaries`: every `rust_binary` target name in the build graph, i.e. the set of binaries a
/// `--out` build can actually produce.
#[must_use]
pub fn evaluate_keyed(
    gitattributes: &str,
    gitconfig: &str,
    buck_binaries: &BTreeSet<String>,
) -> BTreeSet<Finding> {
    let declared = declared_drivers(gitattributes);
    let registered = registered_drivers(gitconfig);
    let mut findings = BTreeSet::new();

    for name in &declared {
        if !registered.contains_key(name) {
            findings.insert(finding(
                "merge_driver_unregistered",
                name,
                format!(
                    "`.gitattributes` binds paths to merge driver `{name}` but {REGISTRATION_FILE} has no `[merge \"{name}\"]` section with a `driver` command, so git silently uses the plain text merge; add the section or drop the attribute"
                ),
            ));
        }
    }

    for (name, command) in &registered {
        if !declared.contains(name) {
            findings.insert(finding(
                "merge_driver_orphan_registration",
                name,
                format!(
                    "{REGISTRATION_FILE} registers merge driver `{name}` but no `.gitattributes` attribute binds any path to it; bind it or remove the registration"
                ),
            ));
        }

        if !(command.contains("%O") && command.contains("%A") && command.contains("%B")) {
            findings.insert(finding(
                "merge_driver_missing_git_placeholders",
                name,
                format!(
                    "the `{name}` driver command must pass git's %O (base) %A (ours/output) %B (theirs) placeholders through to the merge program"
                ),
            ));
        }

        let absolute = command
            .split_whitespace()
            .any(|token| token.starts_with('/'));
        if absolute || command.contains("buck-out") {
            findings.insert(finding(
                "merge_driver_unstable_binary_path",
                name,
                format!(
                    "the `{name}` driver command points at an absolute or `buck-out/` path, which is per-machine and evaporates on a clean or a worktree removal; run the binary from the stable, repo-relative `{STABLE_BIN_DIR}`"
                ),
            ));
            continue;
        }

        let Some(binary) = stable_binary(command) else {
            findings.insert(finding(
                "merge_driver_unstable_binary_path",
                name,
                format!(
                    "the `{name}` driver command does not run a binary from the stable `{STABLE_BIN_DIR}`; a driver resolved from PATH or a build-output path is per-machine"
                ),
            ));
            continue;
        };

        if !buck_binaries.contains(binary) {
            findings.insert(finding(
                "merge_driver_missing_build_target",
                name,
                format!(
                    "the `{name}` driver runs `{STABLE_BIN_DIR}{binary}` but no `rust_binary` target produces `{binary}`, so no `buck2 build --out` can ever populate it; fix the name or add the target"
                ),
            ));
        }

        let guard = format!("test -x {STABLE_BIN_DIR}{binary}");
        if !(command.contains(&guard) && command.contains("git merge-file")) {
            findings.insert(finding(
                "merge_driver_absent_binary_unguarded",
                name,
                format!(
                    "the `{name}` driver command must be guarded by `{guard}` and otherwise exec `git merge-file`: when a registered driver binary is absent git does NOT re-run its own merge, it keeps the unchanged OURS content in %A with no conflict markers and marks the path UU, so the other side is lost on a reflexive `git add`"
                ),
            ));
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    const GUARDED: &str = "test -x ./tools/hooks/bin/demo-driver && exec ./tools/hooks/bin/demo-driver %O %A %B; exec git merge-file -L ours -L base -L theirs %A %O %B";

    fn attrs() -> &'static str {
        "# comment\nCargo.lock merge=demo\nevidence/log.jsonl merge=union\n"
    }

    fn config(driver: &str) -> String {
        format!("[merge \"demo\"]\n\tname = Demo\n\tdriver = \"{driver}\"\n")
    }

    fn binaries() -> BTreeSet<String> {
        BTreeSet::from(["demo-driver".to_owned()])
    }

    fn codes(findings: &BTreeSet<Finding>) -> BTreeSet<String> {
        findings.iter().map(|f| f.code.clone()).collect()
    }

    #[test]
    fn builtin_union_needs_no_registration() {
        assert_eq!(
            declared_drivers(attrs()),
            BTreeSet::from(["demo".to_owned()])
        );
    }

    #[test]
    fn pattern_containing_merge_equals_is_not_a_declaration() {
        assert!(declared_drivers("merge=notadriver text\n").is_empty());
    }

    #[test]
    fn green_on_a_registered_guarded_stable_driver() {
        let findings = evaluate_keyed(attrs(), &config(GUARDED), &binaries());
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn red_when_declared_but_never_registered() {
        // The measured `cargo-lock` / `fixup-ledger` state.
        let findings = evaluate_keyed(attrs(), "", &binaries());
        assert_eq!(
            codes(&findings),
            BTreeSet::from(["merge_driver_unregistered".to_owned()])
        );
    }

    #[test]
    fn red_when_section_carries_no_driver_key() {
        let findings = evaluate_keyed(attrs(), "[merge \"demo\"]\n\tname = Demo\n", &binaries());
        assert_eq!(
            codes(&findings),
            BTreeSet::from(["merge_driver_unregistered".to_owned()])
        );
    }

    #[test]
    fn red_on_the_measured_dead_buck_out_path() {
        // The literal shape found in git config: an absolute buck-out path in another worktree.
        let dead = "/Users/x/wt/buck-out/v2/art/root/abc/tools/d/__d__/demo_driver %O %A %B";
        let findings = evaluate_keyed(attrs(), &config(dead), &binaries());
        assert!(codes(&findings).contains("merge_driver_unstable_binary_path"));
    }

    #[test]
    fn red_on_a_path_resolved_binary() {
        let findings = evaluate_keyed(attrs(), &config("demo-driver %O %A %B"), &binaries());
        assert!(codes(&findings).contains("merge_driver_unstable_binary_path"));
    }

    #[test]
    fn red_when_no_build_target_produces_the_binary() {
        let findings = evaluate_keyed(attrs(), &config(GUARDED), &BTreeSet::new());
        assert!(codes(&findings).contains("merge_driver_missing_build_target"));
    }

    #[test]
    fn red_when_the_absent_binary_guard_is_missing() {
        let unguarded = "./tools/hooks/bin/demo-driver %O %A %B";
        let findings = evaluate_keyed(attrs(), &config(unguarded), &binaries());
        assert!(codes(&findings).contains("merge_driver_absent_binary_unguarded"));
    }

    #[test]
    fn red_when_placeholders_are_dropped() {
        let no_placeholders =
            "test -x ./tools/hooks/bin/demo-driver && exec ./tools/hooks/bin/demo-driver; exec git merge-file";
        let findings = evaluate_keyed(attrs(), &config(no_placeholders), &binaries());
        assert!(codes(&findings).contains("merge_driver_missing_git_placeholders"));
    }

    #[test]
    fn red_on_registration_no_attribute_binds() {
        let findings = evaluate_keyed("", &config(GUARDED), &binaries());
        assert!(codes(&findings).contains("merge_driver_orphan_registration"));
    }

    #[test]
    fn every_emitted_code_is_declared() {
        let declared: BTreeSet<&str> = MERGE_DRIVER_VIOLATION_CODES.into_iter().collect();
        let unguarded_absolute = "/tmp/demo %O";
        let findings = evaluate_keyed("x merge=other\n", &config(unguarded_absolute), &binaries());
        assert!(!findings.is_empty());
        for f in &findings {
            assert!(declared.contains(f.code.as_str()), "undeclared code {f:?}");
        }
    }
}
