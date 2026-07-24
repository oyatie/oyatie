use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use oya_governance_adr_shape_kernel::{
    AdrDocument, audit_adr_shape_fitness, validate_adr_shape_fitness,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("adr-shape failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let paths = input_paths()?;
    run_paths(&paths)
}

fn run_paths(paths: &[PathBuf]) -> Result<(), String> {
    let documents = load_documents(paths)?;
    print!("{}", audit_output(&documents));
    Ok(())
}

fn audit_output(documents: &[AdrDocument]) -> String {
    let report = audit_adr_shape_fitness(&documents);
    let mut output = String::new();
    for finding in &report.findings {
        output.push_str(&format!(
            "{}\t{}\t{}\n",
            finding.path, finding.code, finding.message
        ));
    }
    output.push_str(&format!(
        "adr-shape diagnostic: adrs_checked={} findings={}\n",
        report.adrs_checked,
        report.findings.len()
    ));
    output
}

fn load_documents(paths: &[PathBuf]) -> Result<Vec<AdrDocument>, String> {
    paths
        .iter()
        .map(|path| {
            let text =
                fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
            Ok(AdrDocument {
                path: path.to_string_lossy().into_owned(),
                text,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_and_validates_a_filesystem_fixture() {
        let path = PathBuf::from(
            "tools/oya-governance-adr-shape-app/tests/ADR-9001-enforce-filesystem-adr-validation.md",
        );
        let documents = load_documents(&[path]).expect("fixture is readable");
        assert_eq!(documents.len(), 1);
        let result = validate_adr_shape_fitness(&documents);
        assert!(result.is_ok(), "{result:?}");
        let output = audit_output(&documents);
        assert!(output.ends_with("\n"));
        assert!(output.contains("findings=0"), "{output}");
        assert!(output.contains("adr-shape diagnostic:"));
    }

    #[test]
    fn production_audit_output_handles_the_canonical_template_without_panicking() {
        let paths = [PathBuf::from("docs/templates/adr-template.md")];
        assert!(run_paths(&paths).is_ok());
        let documents = load_documents(&paths).expect("canonical template is readable");
        let report = audit_adr_shape_fitness(&documents);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].code, "ADR_STATUS_INVALID");
        let output = audit_output(&documents);
        assert!(output.contains("adr-shape diagnostic:"));
        assert!(
            output
                .lines()
                .last()
                .is_some_and(|line| line.starts_with("adr-shape diagnostic:"))
        );
    }

    #[test]
    fn production_audit_output_is_sorted_for_adversarial_documents() {
        let documents = vec![
            AdrDocument {
                path: "z.md".to_owned(),
                text: "# ADR-9999: Z\n".to_owned(),
            },
            AdrDocument {
                path: "a.md".to_owned(),
                text: "# ADR-0001: A\n\n   ````md\n## Context\n~~~\n## Decision\n   ````\n"
                    .to_owned(),
            },
        ];
        let output = audit_output(&documents);
        assert!(
            output
                .lines()
                .next()
                .is_some_and(|line| line.starts_with("a.md\t"))
        );
        assert!(output.contains("ADR_SECTION_MISSING"));
    }

    #[test]
    fn executable_path_rejects_four_space_indented_pseudo_structure() {
        let paths = [PathBuf::from(
            "tools/oya-governance-adr-shape-app/tests/ADR-9002-four-space-indented-structure.md",
        )];
        assert!(run_paths(&paths).is_ok());
        let documents = load_documents(&paths).expect("adversarial fixture is readable");
        let output = audit_output(&documents);
        assert!(output.contains("ADR_FRONTMATTER_MISSING"));
        assert!(output.contains("ADR_SECTION_MISSING"));
    }

    #[test]
    fn executable_path_keeps_trailing_text_fence_closer_open() {
        let paths = [PathBuf::from(
            "tools/oya-governance-adr-shape-app/tests/ADR-9003-trailing-fence-closer-structure.md",
        )];
        assert!(run_paths(&paths).is_ok());
        let documents = load_documents(&paths).expect("adversarial fixture is readable");
        let output = audit_output(&documents);
        assert!(output.contains("ADR_FRONTMATTER_MISSING"));
        assert!(output.contains("ADR_SECTION_MISSING"));
    }
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
