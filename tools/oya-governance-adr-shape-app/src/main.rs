use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use oya_governance_adr_shape_kernel::{AdrDocument, validate_adr_shape_fitness};

fn main() {
    if let Err(error) = run() {
        eprintln!("adr-shape failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let paths = input_paths()?;
    let documents = paths
        .iter()
        .map(|path| {
            let text =
                fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
            Ok(AdrDocument {
                path: path.to_string_lossy().into_owned(),
                text,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let report = validate_adr_shape_fitness(&documents).map_err(|error| error.to_string())?;
    println!("adr-shape ok: adrs_checked={}", report.adrs_checked);
    Ok(())
}

fn input_paths() -> Result<Vec<PathBuf>, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if !args.is_empty() {
        return Ok(args.into_iter().map(PathBuf::from).collect());
    }
    let dir = Path::new("docs/decisions");
    let entries = fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("ADR-") && name.ends_with(".md"))
        {
            paths.push(path);
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err("docs/decisions contains no ADR-*.md files".to_string());
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_governance_adr_shape_kernel::audit_adr_shape_fitness;

    fn filesystem_document(name: &str, text: &str) -> AdrDocument {
        let path = env::temp_dir().join(format!("oya-adr-shape-{}-{name}", std::process::id()));
        fs::write(&path, text).expect("fixture is writable");
        let document = AdrDocument {
            path: path.to_string_lossy().into_owned(),
            text: fs::read_to_string(&path).expect("fixture is readable"),
        };
        fs::remove_file(path).expect("fixture is removable");
        document
    }

    #[test]
    fn loads_a_filesystem_fixture_for_public_audit_api_processing() {
        let documents = [filesystem_document(
            "ADR-9001-enforce-filesystem-adr-validation.md",
            "# ADR-9001: Enforce filesystem ADR validation\n\n> **Status:** Proposed\n\n## Context\n\nThe executable must load an ADR document from the filesystem.\n\n## Decision\n\nReport structural findings only.\n\n## Decision Drivers\n\n- Deterministic diagnostics.\n\n## Consequences\n\n- The fixture is non-admissible test data.\n",
        )];
        let report = audit_adr_shape_fitness(&documents);
        assert_eq!(report.adrs_checked, 1);
    }

    #[test]
    fn filesystem_pseudo_adrs_do_not_produce_real_sections() {
        for (name, text) in [
            (
                "ADR-9002-four-space-indented-structure.md",
                "    # ADR-9002: Four space pseudo ADR\n\n    > **Status:** Proposed\n\n    ## Context\n\n    This must not become an ADR heading.\n",
            ),
            (
                "ADR-9003-trailing-fence-closer-structure.md",
                "```md\nplaceholder\n```still-open\n# ADR-9003: Trailing fence closer\n\n> **Status:** Proposed\n\n## Context\n\nThis must remain inside the unclosed fence.\n",
            ),
        ] {
            let report = audit_adr_shape_fitness(&[filesystem_document(name, text)]);
            assert!(
                report
                    .findings
                    .iter()
                    .any(|finding| finding.code == "ADR_SECTION_MISSING")
            );
        }
    }

    #[test]
    fn blocking_validation_remains_the_default_contract() {
        let documents = [AdrDocument {
            path: "docs/decisions/ADR-9009-blocking.md".to_owned(),
            text: "# ADR-9009: Blocking default\n\n## Context\nA\n\n## Decision\nB\n\n## Consequences\nC\n".to_owned(),
        }];
        assert!(validate_adr_shape_fitness(&documents).is_err());
    }
}
