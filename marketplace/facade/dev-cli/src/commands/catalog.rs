use std::path::PathBuf;
use std::process::ExitCode;

use intelligence_catalog_domain::CatalogIndex;

use crate::{read_catalog_records, read_workspace_member_crate_ids};

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("validate") => match parse_validate_args(args.collect(), usage) {
            Ok(args) => match validate(args) {
                Ok(count) => {
                    println!("catalog validation passed: {count} records");
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("catalog validation failed: {message}");
                    ExitCode::FAILURE
                }
            },
            Err(message) => {
                eprintln!("{message}");
                ExitCode::from(2)
            }
        },
        _ => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ValidateArgs {
    workspace_manifest_path: PathBuf,
    registry_dir: PathBuf,
}

fn parse_validate_args(args: Vec<String>, usage: &str) -> Result<ValidateArgs, String> {
    let mut parsed = ValidateArgs {
        workspace_manifest_path: PathBuf::from("Cargo.toml"),
        registry_dir: PathBuf::from("registry/catalog"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage.to_owned());
        };
        match flag.as_str() {
            "--workspace" => parsed.workspace_manifest_path = PathBuf::from(path),
            "--registry" => parsed.registry_dir = PathBuf::from(path),
            _ => return Err(usage.to_owned()),
        }
    }
    Ok(parsed)
}

fn validate(args: ValidateArgs) -> Result<usize, String> {
    let required_crates = read_workspace_member_crate_ids(&args.workspace_manifest_path)?;
    let records = read_catalog_records(&args.registry_dir)?;
    let index = CatalogIndex::from_records(records)
        .map_err(|error| format!("catalog index invalid: {error:?}"))?;
    index
        .validate_required_crates(required_crates.iter().map(String::as_str))
        .map_err(|error| format!("workspace record coverage invalid: {error:?}"))?;
    Ok(index.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage() -> &'static str {
        "usage text"
    }

    #[test]
    fn parse_validate_defaults_to_workspace_and_registry() {
        let args = parse_validate_args(Vec::new(), usage()).expect("parse defaults");

        assert_eq!(args.workspace_manifest_path, PathBuf::from("Cargo.toml"));
        assert_eq!(args.registry_dir, PathBuf::from("registry/catalog"));
    }

    #[test]
    fn parse_validate_accepts_workspace_and_registry_paths() {
        let args = parse_validate_args(
            vec![
                "--workspace".to_owned(),
                "fixtures/Cargo.toml".to_owned(),
                "--registry".to_owned(),
                "fixtures/catalog".to_owned(),
            ],
            usage(),
        )
        .expect("parse args");

        assert_eq!(
            args.workspace_manifest_path,
            PathBuf::from("fixtures/Cargo.toml")
        );
        assert_eq!(args.registry_dir, PathBuf::from("fixtures/catalog"));
    }

    #[test]
    fn parse_validate_rejects_dangling_flag_with_usage() {
        let error = parse_validate_args(vec!["--workspace".to_owned()], usage())
            .expect_err("dangling flag should fail");

        assert_eq!(error, usage());
    }
}
