use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use oya_foundry_fitness_adr_shape_kernel::{AdrDocument, validate_adr_shape_fitness};

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
