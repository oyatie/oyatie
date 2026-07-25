use std::env;
use std::fs;
use std::io::{self, Write};
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
    if diagnostic_mode() {
        write_diagnostic(&documents)?;
    } else {
        let report = validate_adr_shape_fitness(&documents).map_err(|error| error.to_string())?;
        println!("adr-shape ok: adrs_checked={}", report.adrs_checked);
    }
    Ok(())
}

fn diagnostic_mode() -> bool {
    env::args().any(|argument| argument == "--diagnostic")
}

fn write_diagnostic(documents: &[AdrDocument]) -> Result<(), String> {
    let report = audit_adr_shape_fitness(documents);
    let stdout = io::stdout();
    let mut output = stdout.lock();
    for finding in &report.findings {
        write_line(
            &mut output,
            &format!("{}\t{}\t{}", finding.path, finding.code, finding.message),
        )?;
    }
    write_line(
        &mut output,
        &format!(
            "adr-shape diagnostic: adrs_checked={} findings={}",
            report.adrs_checked,
            report.findings.len()
        ),
    )
}

fn write_line(output: &mut impl Write, line: &str) -> Result<(), String> {
    match writeln!(output, "{line}") {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(error) => Err(format!("stdout: {error}")),
    }
}

fn input_paths() -> Result<Vec<PathBuf>, String> {
    let args = env::args()
        .skip(1)
        .filter(|argument| argument != "--diagnostic")
        .collect::<Vec<_>>();
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

    #[test]
    fn loads_a_filesystem_fixture_for_diagnostic_processing() {
        let path = PathBuf::from(
            "tools/oya-governance-adr-shape-app/tests/ADR-9001-enforce-filesystem-adr-validation.md",
        );
        let documents = [AdrDocument {
            path: path.to_string_lossy().into_owned(),
            text: fs::read_to_string(&path).expect("fixture is readable"),
        }];
        let report = audit_adr_shape_fitness(&documents);
        assert_eq!(report.adrs_checked, 1);
    }

    #[test]
    fn filesystem_pseudo_adrs_do_not_produce_real_sections() {
        for fixture in [
            "ADR-9002-four-space-indented-structure.md",
            "ADR-9003-trailing-fence-closer-structure.md",
        ] {
            let path = PathBuf::from(format!(
                "tools/oya-governance-adr-shape-app/tests/{fixture}"
            ));
            let report = audit_adr_shape_fitness(&[AdrDocument {
                path: path.to_string_lossy().into_owned(),
                text: fs::read_to_string(path).expect("fixture is readable"),
            }]);
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

    #[test]
    fn broken_pipe_is_a_successful_bounded_diagnostic_consumer_exit() {
        struct BrokenPipe;
        impl Write for BrokenPipe {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::from(io::ErrorKind::BrokenPipe))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        assert!(write_line(&mut BrokenPipe, "one finding").is_ok());
    }
}
