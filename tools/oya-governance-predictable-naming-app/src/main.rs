use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use check_predictable_naming_kernel::{CrateNaming, NamingReport, check};

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(report) => {
            if report.violations.is_empty() {
                println!(
                    "predictable-naming ok: crates_checked={}",
                    report.crates_checked,
                );
                ExitCode::SUCCESS
            } else {
                for v in &report.violations {
                    eprintln!(
                        "predictable-naming violation: {} — {}",
                        v.crate_name,
                        v.kind.as_str(),
                    );
                }
                eprintln!(
                    "predictable-naming failed: {} violation(s) in {} crate(s) checked",
                    report.violations.len(),
                    report.crates_checked,
                );
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("predictable-naming failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run<I>(args: I) -> Result<NamingReport, String>
where
    I: IntoIterator<Item = String>,
{
    let options = Options::parse(args)?;
    let rows: Result<Vec<CrateNaming>, String> =
        options.paths.iter().map(|p| naming_from_path(p)).collect();
    check(&rows?).map_err(|e| e.message())
}

struct Options {
    paths: Vec<PathBuf>,
}

impl Options {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut paths = Vec::new();
        let args = args.into_iter().collect::<Vec<_>>();
        let mut index = 0usize;
        while index < args.len() {
            match args[index].as_str() {
                "--check" => {
                    index += 1;
                    while index < args.len() && !args[index].starts_with('-') {
                        paths.push(PathBuf::from(&args[index]));
                        index += 1;
                    }
                }
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument '{other}'\n{}", usage())),
            }
        }
        if paths.is_empty() {
            return Err(format!("no paths provided\n{}", usage()));
        }
        Ok(Self { paths })
    }
}

fn usage() -> String {
    "usage: oya-governance-predictable-naming-app --check <path>...".into()
}

/// Derive a [`CrateNaming`] record from a crate directory path.
///
/// The crate name is taken from the final path component.
/// The role is the trailing dash-segment of the name.
/// The context is the second dash-segment (the one after "oya-").
fn naming_from_path(path: &Path) -> Result<CrateNaming, String> {
    let crate_name = path
        .file_name()
        .ok_or_else(|| format!("cannot determine crate name from path: {}", path.display()))?
        .to_str()
        .ok_or_else(|| format!("non-UTF-8 path: {}", path.display()))?
        .to_string();

    let segments: Vec<&str> = crate_name.split('-').collect();
    // oya-<context>-...-<role>  →  context at index 1, role at last index
    let declared_context = segments.get(1).map(|s| s.to_string());
    let declared_role = segments.last().map(|s| s.to_string());

    Ok(CrateNaming {
        crate_name,
        declared_role,
        declared_context,
    })
}
