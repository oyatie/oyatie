//! # Merge-driver declaration parsing
//!
//! The same invariant this crate already enforces for hooks — declared enforcement must resolve to
//! a live executable — applied to the merge drivers `.gitattributes` declares.
//!
//! `.gitattributes` DECLARES a merge driver by name; per-clone git config BINDS the name to a
//! command in `merge.<name>.driver`. Nothing kept the two halves honest. Three driver crates each
//! carried their own copy-paste `git config` block in their README, so registration was manual,
//! per-clone and per-driver — and it drifted exactly as manual wiring does: of the three declared
//! non-builtin drivers in this repo, the clone this module was written from had ONE registered,
//! and its command pointed into a `buck-out` directory belonging to a worktree that had since been
//! deleted.
//!
//! ## Why a dead declaration is worse than no declaration
//!
//! Git does not re-run its own text merge when the driver command fails. It takes whatever the
//! command left in `%A` as the conflicted working tree, and a command that never ran leaves `ours`
//! standing alone — unmarked, with the other side's content simply absent — so the file reads as
//! clean and complete and a reflexive `git add` commits the loss. Both driver READMEs document
//! this from real incidents.
//!
//! ## What this module is for
//!
//! [`drivers_requiring_registration`] is the declaration side, and [`source_env_var`] is the one
//! spelling that ties a declared name to the built binary the BUCK wiring supplies. The gate test
//! `every_declared_merge_driver_resolves_to_a_built_executable` pins the two sets equal in both
//! directions, so neither a declaration with no implementation nor an implementation with no
//! declaration can land.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.

use std::collections::BTreeSet;

/// Merge drivers git implements itself. `.gitattributes` may name these with no crate behind
/// them, so they are declared-but-not-registerable rather than a missing implementation.
/// `binary` and `text` are the built-in conflict-style drivers; `union` concatenates both sides.
pub const GIT_BUILTIN_MERGE_DRIVERS: [&str; 3] = ["binary", "text", "union"];

/// Env var carrying the built binary for `name`. ONE spelling, shared by the BUCK wiring that
/// supplies the paths and the gate that asserts the set matches what `.gitattributes` declares —
/// so the two cannot drift.
pub fn source_env_var(name: &str) -> String {
    let mut var = String::from("MERGE_DRIVER_SOURCE_");
    for ch in name.chars() {
        var.push(if ch.is_ascii_alphanumeric() {
            ch.to_ascii_uppercase()
        } else {
            '_'
        });
    }
    var
}

/// Every driver name `.gitattributes` declares, git built-ins included.
pub fn declared_drivers(gitattributes: &str) -> BTreeSet<String> {
    let mut declared = BTreeSet::new();
    for line in gitattributes.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // `<pattern> <attr>...`; the pattern is field 0 and is never an attribute. Quoted
        // patterns (`"path with spaces" merge=x`) are not used in this repo, so the split is
        // deliberately plain — a quoted pattern containing whitespace would split into extra
        // fields, none of which can match `merge=`, so it degrades to "declares nothing" rather
        // than to a wrong name.
        for attr in line.split_whitespace().skip(1) {
            // `-merge` / `!merge` / bare `merge` set, unset or unspecify the attribute; only the
            // `merge=<name>` form names a driver.
            if let Some(name) = attr.strip_prefix("merge=") {
                if !name.is_empty() {
                    declared.insert(name.to_owned());
                }
            }
        }
    }
    declared
}

/// The declared names that need a `merge.<name>.driver` binding: everything git does not
/// implement itself.
pub fn drivers_requiring_registration(gitattributes: &str) -> BTreeSet<String> {
    declared_drivers(gitattributes)
        .into_iter()
        .filter(|name| !GIT_BUILTIN_MERGE_DRIVERS.contains(&name.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIVE_SHAPE: &str = "\
# a comment mentioning merge=not-a-declaration
evidence/audit-chain.jsonl merge=union

Cargo.lock merge=cargo-lock
registry/fixuptasks.jsonl merge=fixup-ledger
/.omc/ultragoal/friction-ledger.jsonl merge=friction-ledger
";

    #[test]
    fn comments_are_not_declarations() {
        // The retired hand-registration was kept in sync by reading this file by eye, and it is 79
        // lines of prose around 4 declarations: a parser that matched `merge=` anywhere would
        // report five drivers, one of which does not exist — and the gate would then demand a
        // binary for it.
        assert_eq!(
            declared_drivers(LIVE_SHAPE),
            ["cargo-lock", "fixup-ledger", "friction-ledger", "union"]
                .into_iter()
                .map(str::to_owned)
                .collect::<BTreeSet<_>>()
        );
    }

    #[test]
    fn a_pattern_is_never_read_as_an_attribute() {
        // A path can legitimately contain `merge=`; only fields AFTER the pattern are attributes.
        assert!(declared_drivers("docs/merge=notes.md text\n").is_empty());
    }

    #[test]
    fn unset_and_unspecified_forms_declare_no_driver() {
        assert!(declared_drivers("a -merge\nb !merge\nc merge\nd merge=\n").is_empty());
    }

    #[test]
    fn builtins_need_no_registration() {
        // `union` is git's own; demanding a binary for it would make the gate unsatisfiable.
        assert_eq!(
            drivers_requiring_registration(LIVE_SHAPE),
            ["cargo-lock", "fixup-ledger", "friction-ledger"]
                .into_iter()
                .map(str::to_owned)
                .collect::<BTreeSet<_>>()
        );
    }

    #[test]
    fn env_var_spelling_is_derived_not_typed() {
        assert_eq!(source_env_var("cargo-lock"), "MERGE_DRIVER_SOURCE_CARGO_LOCK");
        assert_eq!(
            source_env_var("friction-ledger"),
            "MERGE_DRIVER_SOURCE_FRICTION_LEDGER"
        );
    }
}
