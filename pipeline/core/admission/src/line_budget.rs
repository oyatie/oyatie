//! Touched-file budget. Provenance: ADR-0719 file-budget decision (D-35).

use crate::layout::{APP_PRODUCT_DIRS, is_capability_root};

const MAX_LINES: usize = 300;
const OWNER_LAW: &[&str] = &["ADR.md", "PRD.md", "SPEC.md", "PLAN.md"];

/// Count physical newline characters exactly as `wc -l`; the closed exempt
/// set is path-derived and cannot be expanded by file contents.
pub fn file_budget_violations(path: &str, contents: &[u8]) -> Vec<String> {
    if exempt(path) {
        return Vec::new();
    }
    let lines = contents.iter().filter(|byte| **byte == b'\n').count();
    if lines <= MAX_LINES {
        Vec::new()
    } else {
        vec![format!(
            "{path}: {lines} physical lines exceeds the repository {MAX_LINES}-line file budget"
        )]
    }
}

fn exempt(path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    let name = parts.last().copied().unwrap_or_default();
    path == "Cargo.lock"
        || path.starts_with("third-party/")
        || matches!(name, "AGENTS.md" | "CLAUDE.md")
        || name.contains(".generated.")
        || vendored_lock_step_snapshot(path)
        || live_apex_adr(path)
        || owner_law(&parts)
}

fn vendored_lock_step_snapshot(path: &str) -> bool {
    const SNAPSHOT_PREFIX: &str = "build/port-engine/adapters/snapshot/src/fixture-snapshot-";
    const PORT_GO_PREFIX: &str = "build/port-engine/facade/app/src/port-go-golden-v";
    path.strip_prefix(SNAPSHOT_PREFIX)
        .and_then(|name| name.strip_suffix(".json"))
        .is_some_and(lowercase_versioned_name)
        || path
            .strip_prefix(PORT_GO_PREFIX)
            .and_then(|version| version.strip_suffix(".txt"))
            .is_some_and(|version| {
                !version.is_empty() && version.bytes().all(|byte| byte.is_ascii_digit())
            })
}

fn lowercase_versioned_name(name: &str) -> bool {
    let version = name.strip_prefix('v').or_else(|| {
        name.rsplit_once("-v")
            .and_then(|(stem, version)| (!stem.is_empty()).then_some(version))
    });
    !name.is_empty()
        && !name.contains('/')
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && version.is_some_and(|version| {
            !version.is_empty() && version.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn live_apex_adr(path: &str) -> bool {
    path.strip_prefix("docs/decisions/ADR-07")
        .and_then(|rest| rest.strip_suffix(".md"))
        .is_some_and(|rest| {
            rest.len() > 3
                && rest.as_bytes()[..2].iter().all(u8::is_ascii_digit)
                && rest.as_bytes()[2] == b'-'
        })
}

fn owner_law(parts: &[&str]) -> bool {
    matches!(parts, [owner, name] if is_capability_root(owner) && OWNER_LAW.contains(name))
        || matches!(parts, ["app", product, name]
            if APP_PRODUCT_DIRS.contains(product) && OWNER_LAW.contains(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touched_text_over_three_hundred_lines_is_red() {
        let text = "line\n".repeat(301);
        assert!(!file_budget_violations("network/core/x/src/lib.rs", text.as_bytes()).is_empty());
        let mut non_utf8 = text.into_bytes();
        non_utf8.push(0xff);
        assert!(!file_budget_violations("network/core/x/blob", &non_utf8).is_empty());
    }

    #[test]
    fn closed_exempt_set_is_honored() {
        let text = "line\n".repeat(301);
        for path in [
            "Cargo.lock",
            "third-party/vendor/source.rs",
            "docs/decisions/ADR-0719-example.md",
            "network/ADR.md",
            "app/payroll/SPEC.md",
            "network/observability/slos/a.generated.openslo.yaml",
            "build/port-engine/adapters/snapshot/src/fixture-snapshot-v1.json",
            "build/port-engine/adapters/snapshot/src/fixture-snapshot-interface-v1.json",
            "build/port-engine/facade/app/src/port-go-golden-v1.txt",
        ] {
            assert!(
                file_budget_violations(path, text.as_bytes()).is_empty(),
                "{path}"
            );
        }
    }

    #[test]
    fn exemption_spellings_are_closed() {
        let text = "line\n".repeat(301);
        for path in [
            "base/ADR.md",
            "app/not-a-product/PLAN.md",
            "docs/decisions/ADR-0719evil.md",
            "build/port-engine/adapters/other/src/fixture-snapshot-v1.json",
            "build/port-engine/adapters/snapshot/src/not-a-snapshot-v1.json",
            "build/port-engine/facade/app/src/port-go-golden-vnext.txt",
        ] {
            assert!(
                !file_budget_violations(path, text.as_bytes()).is_empty(),
                "{path}"
            );
        }
    }
}
