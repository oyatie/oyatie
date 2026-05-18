//! Thin CLI wrapper around [`oya_check_adr_placeholders`] kernel.
//!
//! Modes:
//! - `--validate` (default): scan docs/, specs/, microservices/, registry/
//!   and exit non-zero if any `ADR-XXXX` / `ADR-NNNN` placeholder is found.
//! - `--fix`: rewrite placeholders in-place using the canonical replacement
//!   table; useful as a one-shot migration before the gate is wired into CI.
//!
//! Both modes accept `--root <path>` (defaults to current dir).
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_check_adr_placeholders::{Error, FileContent, Mode, auto_fix, validate};

const SCAN_ROOTS: &[&str] = &["docs", "specs", "microservices", "registry"];
const EXTENSIONS: &[&str] = &["md", "yaml", "yml", "json", "tsv", "txt"];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut mode = Mode::Validate;
    let mut root = PathBuf::from(".");
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--validate" => mode = Mode::Validate,
            "--fix" => mode = Mode::AutoFix,
            "--root" => {
                if let Some(path) = iter.next() {
                    root = PathBuf::from(path);
                } else {
                    eprintln!("--root requires an argument");
                    return ExitCode::from(2);
                }
            }
            other => {
                eprintln!("unknown flag: {other}");
                eprintln!("usage: oya-check-adr-placeholders [--validate|--fix] [--root <path>]");
                return ExitCode::from(2);
            }
        }
    }

    let files = match load_corpus(&root) {
        Ok(f) => f,
        Err(message) => {
            eprintln!("failed to load corpus: {message}");
            return ExitCode::FAILURE;
        }
    };

    match mode {
        Mode::Validate => match validate(&files) {
            Ok(report) => {
                println!(
                    "adr-placeholders validate passed: {} files scanned, 0 hits",
                    report.files_checked
                );
                ExitCode::SUCCESS
            }
            Err(Error::PlaceholdersFound(report)) => {
                eprintln!(
                    "adr-placeholders validate FAILED: {} hits across {} files",
                    report.hits.len(),
                    report.files_checked
                );
                for hit in &report.hits {
                    eprintln!("  {}:{}  {}", hit.path, hit.line, hit.token);
                }
                ExitCode::FAILURE
            }
        },
        Mode::AutoFix => {
            let rewrites = auto_fix(&files);
            for rewrite in &rewrites {
                if let Err(error) = fs::write(&rewrite.path, &rewrite.content) {
                    eprintln!("write failed {}: {error}", rewrite.path);
                    return ExitCode::FAILURE;
                }
                println!(
                    "[fix] {}  replacements={}",
                    rewrite.path, rewrite.replacements
                );
            }
            let total: usize = rewrites.iter().map(|r| r.replacements).sum();
            println!(
                "\nadr-placeholders auto-fix: rewrote {} files, {} placeholder occurrences",
                rewrites.len(),
                total
            );
            ExitCode::SUCCESS
        }
    }
}

fn load_corpus(root: &Path) -> Result<Vec<FileContent>, String> {
    let mut out = Vec::new();
    for top in SCAN_ROOTS {
        let dir = root.join(top);
        if !dir.is_dir() {
            continue;
        }
        walk(&dir, &mut out)?;
    }
    Ok(out)
}

fn walk(dir: &Path, out: &mut Vec<FileContent>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        let Some(ext) = ext else { continue };
        if !EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        out.push(FileContent {
            path: path.to_string_lossy().to_string(),
            content: text,
        });
    }
    Ok(())
}
