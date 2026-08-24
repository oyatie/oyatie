//! Closed root-workspace membership policy for ADR-0719 D-8/D-41.

use std::collections::BTreeSet;

use workspace_members_kernel::workspace_manifest_entries_from_str;

pub const WORKSPACE_MEMBER_GLOBS: &[&str] = &[
    "*/core/*",
    "*/ports/*/src/..",
    "*/adapters/*/src/..",
    "*/facade/*/src/..",
    "app/*/core/*",
    "*/ports/draft/*/src/..",
    "*/adapters/draft/*/src/..",
    "app/*/ports/*/src/..",
    "app/*/adapters/*/src/..",
    "app/*/ports/draft/*/src/..",
    "app/*/adapters/draft/*/src/..",
    "app/*/facade/*/src/..",
    "app/foundry/pages/crates/*",
    "app/foundry/grid/core/*",
    "build/port-engine/*/*",
];

pub const WORKSPACE_EXCLUDES: &[&str] = &[
    "*/ports/draft/*",
    "*/adapters/draft/*",
    "app/*/ports/draft/*",
    "app/*/adapters/draft/*",
];

/// Refuse missing, added, or duplicate workspace membership policy entries.
pub fn workspace_membership_violations(contents: &str) -> Vec<String> {
    let entries = match workspace_manifest_entries_from_str(contents) {
        Ok(entries) => entries,
        Err(error) => return vec![format!("Cargo.toml: {error}")],
    };
    let mut violations = Vec::new();
    compare_closed_entries(
        "member glob",
        &entries.members,
        WORKSPACE_MEMBER_GLOBS,
        &mut violations,
    );
    compare_closed_entries(
        "exclude",
        &entries.exclude,
        WORKSPACE_EXCLUDES,
        &mut violations,
    );
    violations
}

fn compare_closed_entries(
    surface: &str,
    actual: &[String],
    expected: &[&str],
    violations: &mut Vec<String>,
) {
    let actual_set: BTreeSet<&str> = actual.iter().map(String::as_str).collect();
    let expected_set: BTreeSet<&str> = expected.iter().copied().collect();
    for missing in expected_set.difference(&actual_set) {
        violations.push(format!(
            "Cargo.toml: required workspace {surface} `{missing}` is missing"
        ));
    }
    for unexpected in actual_set.difference(&expected_set) {
        violations.push(format!(
            "Cargo.toml: unexpected workspace {surface} `{unexpected}`"
        ));
    }
    if actual.len() != actual_set.len() {
        violations.push(format!(
            "Cargo.toml: duplicate workspace {surface} entries are forbidden"
        ));
    } else if actual_set == expected_set
        && !actual
            .iter()
            .map(String::as_str)
            .eq(expected.iter().copied())
    {
        violations.push(format!(
            "Cargo.toml: workspace {surface} entries must retain canonical order"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(members: &[&str], excludes: &[&str]) -> String {
        let members = members
            .iter()
            .map(|entry| format!("  {entry:?},\n"))
            .collect::<String>();
        let excludes = excludes
            .iter()
            .map(|entry| format!("  {entry:?},\n"))
            .collect::<String>();
        format!("[workspace]\nmembers = [\n{members}]\nexclude = [\n{excludes}]\nresolver = '2'\n")
    }

    #[test]
    fn exact_workspace_policy_admits() {
        assert!(
            workspace_membership_violations(&manifest(WORKSPACE_MEMBER_GLOBS, WORKSPACE_EXCLUDES))
                .is_empty()
        );
    }

    #[test]
    fn workspace_policy_rejects_exclusions_and_member_drift() {
        let mut excludes = WORKSPACE_EXCLUDES.to_vec();
        excludes.push("network/core/route");
        let violations =
            workspace_membership_violations(&manifest(&WORKSPACE_MEMBER_GLOBS[1..], &excludes));
        assert!(violations.iter().any(|item| item.contains("is missing")));
        assert!(
            violations
                .iter()
                .any(|item| item.contains("network/core/route"))
        );

        let mut reordered = WORKSPACE_MEMBER_GLOBS.to_vec();
        reordered.swap(0, 1);
        assert!(
            workspace_membership_violations(&manifest(&reordered, WORKSPACE_EXCLUDES))
                .iter()
                .any(|item| item.contains("canonical order"))
        );
    }
}
