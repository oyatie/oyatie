//! Path → occupant. GitHub team is `@oyatie/<occupant>` except the org
//! occupant `oyatie`, whose review identity is the CODEOWNERS `*` user.

use crate::{DATA_ROOTS, META_ROOTS, is_capability_root, path_parts};

pub const ROOT_OCCUPANT: &str = "oyatie";

/// Occupant slug for an `OWNERS` path (repo-relative, `/`-separated).
pub fn owners_occupant(rel: &str) -> Option<String> {
    let parts = path_parts(rel);
    let root = parts.first().copied()?;
    if parts.len() == 1 && root == "OWNERS" {
        return Some(ROOT_OCCUPANT.to_owned());
    }
    if root == ".github" {
        if parts.get(1) == Some(&"workflows") {
            return Some("pipeline".to_owned());
        }
        return Some(ROOT_OCCUPANT.to_owned());
    }
    if root == "app" {
        if parts.len() <= 2 {
            return Some(ROOT_OCCUPANT.to_owned());
        }
        return Some(parts[1].to_owned());
    }
    if root == "base" {
        return Some(ROOT_OCCUPANT.to_owned());
    }
    if DATA_ROOTS.contains(&root) {
        return Some(root.to_owned());
    }
    if META_ROOTS.contains(&root) {
        return Some(ROOT_OCCUPANT.to_owned());
    }
    if is_capability_root(root) {
        return Some(root.to_owned());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn occupant_map() {
        let cases = [
            ("OWNERS", "oyatie"),
            ("app/OWNERS", "oyatie"),
            ("app/README.md", "oyatie"),
            ("app/foundry/OWNERS", "foundry"),
            ("app/foundry/grid/OWNERS", "foundry"),
            ("app/hr/adapters/employment-infrastructure/OWNERS", "hr"),
            ("network/OWNERS", "network"),
            ("iam/adapters/x/OWNERS", "iam"),
            ("docs/decisions/OWNERS", "oyatie"),
            ("build/toolchains/OWNERS", "oyatie"),
            ("base/OWNERS", "oyatie"),
            ("templates/OWNERS", "oyatie"),
            ("third-party/OWNERS", "oyatie"),
            (".github/OWNERS", "oyatie"),
            (".github/CODEOWNERS", "oyatie"),
            (".github/workflows/OWNERS", "pipeline"),
            (".github/workflows/presubmit.yml", "pipeline"),
            ("packs/OWNERS", "packs"),
        ];
        for (path, want) in cases {
            assert_eq!(owners_occupant(path).as_deref(), Some(want), "{path}");
        }
        assert_eq!(owners_occupant("target/OWNERS"), None);
        assert_eq!(owners_occupant(""), None);
    }
}
