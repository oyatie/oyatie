//! Closed grammars for repository-level metadata and data roots.

use super::inner::validate_owner_path;

const PACK_NAMESPACES: &[&str] = &["us", "eu", "jp", "kr"];
const ROOT_METADATA: &[&str] = &["OWNERS", "README.md", "BUCK"];

pub(super) fn validate_base_path(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    if parts.len() == 2 && ROOT_METADATA.contains(&parts[1]) {
        return;
    }
    if parts.get(1) != Some(&"core") {
        violations.push(format!(
            "{file}: base admits only root metadata and `core/` primitives"
        ));
        return;
    }
    validate_owner_path(file, parts, 1, violations);
}

pub(super) fn validate_cargo_path(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    if parts.len() != 2 || !matches!(parts[1], "BUCK" | "config" | "config.toml") {
        violations.push(format!(
            "{file}: `.cargo/` admits only BUCK and the canonical Cargo configuration"
        ));
    }
}

pub(super) fn validate_docs_path(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    if parts.len() == 2 && matches!(parts[1], "AGENTS.md" | "OWNERS" | "README.md" | "BUCK") {
        return;
    }
    let valid = match parts.get(1).copied() {
        Some("decisions") if parts.len() == 3 => decision_file(parts[2]),
        Some("standards") if parts.len() == 3 => markdown_or_metadata(parts[2]),
        _ => false,
    };
    if !valid {
        violations.push(format!(
            "{file}: root docs are limited to AGENTS.md, live 07xx decisions, and direct standards"
        ));
    }
}

pub(super) fn validate_packs_path(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    if parts.len() == 2 && ROOT_METADATA.contains(&parts[1]) {
        return;
    }
    let valid = parts.len() == 4
        && PACK_NAMESPACES.contains(&parts[1])
        && kebab_case(parts[2])
        && pack_payload(parts[3]);
    if !valid {
        violations.push(format!(
            "{file}: packs require `<us|eu|jp|kr>/<package>/<name>.cedar|proto|textproto`"
        ));
    }
}

pub(super) fn validate_templates_path(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    let valid = matches!(parts, ["templates", "OWNERS" | "README.md" | "BUCK"])
        || matches!(parts, ["templates", "adr-template.md"])
        || matches!(parts, ["templates", "checklists", "swarm-agent-ritual.md"]);
    if !valid {
        violations.push(format!(
            "{file}: templates are limited to the ADR template and swarm ritual"
        ));
    }
}

fn decision_file(name: &str) -> bool {
    matches!(name, "OWNERS" | "README.md" | "INDEX.md" | "BUCK")
        || name
            .strip_prefix("ADR-07")
            .and_then(|rest| rest.strip_suffix(".md"))
            .is_some_and(|rest| {
                rest.len() > 3
                    && rest.as_bytes()[..2].iter().all(u8::is_ascii_digit)
                    && rest.as_bytes()[2] == b'-'
            })
}

fn markdown_or_metadata(name: &str) -> bool {
    matches!(name, "OWNERS" | "README.md" | "INDEX.md" | "BUCK")
        || name.strip_suffix(".md").is_some_and(kebab_case)
}

fn pack_payload(name: &str) -> bool {
    [".cedar", ".proto", ".textproto"]
        .iter()
        .any(|suffix| name.strip_suffix(suffix).is_some_and(lower_identifier))
}

fn kebab_case(name: &str) -> bool {
    lower_identifier(name) && !name.contains('_')
}

fn lower_identifier(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name
            .as_bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && !name.contains("--")
        && !name.contains("__")
        && name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rejected(path: &str, check: fn(&str, &[&str], &mut Vec<String>)) -> bool {
        let parts: Vec<&str> = path.split('/').collect();
        let mut violations = Vec::new();
        check(path, &parts, &mut violations);
        !violations.is_empty()
    }

    #[test]
    fn root_docs_are_thin() {
        assert!(!rejected(
            "docs/decisions/ADR-0720-example.md",
            validate_docs_path
        ));
        assert!(!rejected(
            "docs/standards/code-style.md",
            validate_docs_path
        ));
        assert!(rejected("docs/scratch/Cargo.toml", validate_docs_path));
        assert!(rejected("docs/decisions/ADR.md", validate_docs_path));
    }

    #[test]
    fn packs_are_typed_cedar_and_ir_packages() {
        assert!(!rejected("packs/eu/gdpr/policy.cedar", validate_packs_path));
        assert!(!rejected(
            "packs/kr/csap/data_residency.textproto",
            validate_packs_path
        ));
        assert!(rejected("packs/eu/plan/todo.md", validate_packs_path));
        assert!(rejected("packs/eu/new-overlay.yaml", validate_packs_path));
    }
}
