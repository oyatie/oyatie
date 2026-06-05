//! Append mirrored `rust_test` targets for first-party BUCK files.
//!
//! This is the Rust/Buck2 replacement for the retired Python helper that added
//! test targets beside existing `rust_library` rules. It is intentionally
//! append-only and conservative: BUCK files that already contain a rust_test,
//! proc-macro libraries, generated/vendored trees, and known-broken crates are
//! left unchanged.

use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

const SKIP_PREFIXES: &[&str] = &[
    "third-party/",
    "buck-out/",
    "tools/agent-skills/",
    "prelude/",
    ".",
];
const KNOWN_FAILING: &[&str] = &[
    "libs/oya-check-dependency-seam",
    "libs/oya-shared-postgres-command-adapter-sqlx",
    "libs/oya-shared-backbone-rest-runtime-adapter",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    repo_root: PathBuf,
    subsystem: Option<PathBuf>,
    dry_run: bool,
    version: bool,
    help: bool,
}

impl Options {
    fn default_with_repo_root(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
            subsystem: None,
            dry_run: false,
            version: false,
            help: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessResult {
    pub text: String,
    pub added: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub rel_path: String,
    pub added: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSummary {
    pub changed_files: usize,
    pub added_targets: usize,
    pub skipped_files: usize,
    pub changes: Vec<FileChange>,
}

fn usage(program: &str) -> String {
    format!(
        "\
Usage: {program} [--repo-root PATH] [--subsystem DIR] [--dry-run]\n\n\
Append mirrored rust_test targets to first-party BUCK files that have\n\
rust_library rules but no rust_test rules. Existing BUCK content is preserved\n\
except for appended test targets.\n\n\
Options:\n\
  --repo-root PATH   Repository root to scan (default: current directory)\n\
  --subsystem DIR    Restrict scan to a repository-relative path prefix\n\
  --dry-run          Print candidate changes without writing files\n\
  --version          Print tool identity and exit\n\
  --help             Print this help text and exit\n"
    )
}

fn default_repo_root() -> PathBuf {
    env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing {flag} value"))
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default_with_repo_root(default_repo_root());
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--repo-root" => {
                options.repo_root = PathBuf::from(take_value(args, &mut index, "--repo-root")?)
            }
            "--subsystem" => {
                options.subsystem =
                    Some(PathBuf::from(take_value(args, &mut index, "--subsystem")?));
            }
            "--dry-run" => options.dry_run = true,
            "--version" => options.version = true,
            "--help" | "-h" => options.help = true,
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
        index += 1;
    }
    Ok(options)
}

fn normalize_rel(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn validate_relative_scan_path(path: &Path) -> Result<(), String> {
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!(
            "subsystem must be repository-relative and cannot escape the repo: {}",
            path.display()
        ));
    }
    Ok(())
}

pub fn is_skipped_rel(rel: &str) -> bool {
    SKIP_PREFIXES
        .iter()
        .any(|prefix| rel == prefix.trim_end_matches('/') || rel.starts_with(prefix))
        || KNOWN_FAILING.iter().any(|prefix| {
            rel == *prefix
                || rel
                    .strip_prefix(*prefix)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        })
}

fn collect_buck_files(root: &Path, rel: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let abs = root.join(rel);
    let mut entries = fs::read_dir(&abs)
        .map_err(|error| format!("read_dir {} failed: {error}", abs.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read_dir {} failed: {error}", abs.display()))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let file_type = entry
            .file_type()
            .map_err(|error| format!("file_type {} failed: {error}", entry.path().display()))?;
        let entry_rel = rel.join(entry.file_name());
        let normalized = normalize_rel(&entry_rel);
        if is_skipped_rel(&normalized) {
            continue;
        }
        if file_type.is_dir() {
            collect_buck_files(root, &entry_rel, files)?;
        } else if file_type.is_file() && entry.file_name() == "BUCK" {
            files.push(entry_rel);
        }
    }
    Ok(())
}

pub fn rust_rule_blocks<'a>(text: &'a str, rule: &str) -> Vec<&'a str> {
    let needle = format!("{rule}(");
    let mut blocks = Vec::new();
    let mut search_offset = 0;
    while let Some(relative_start) = text[search_offset..].find(&needle) {
        let start = search_offset + relative_start;
        let line_start = text[..start].rfind('\n').map_or(0, |position| position + 1);
        if !text[line_start..start].trim().is_empty() {
            search_offset = start + needle.len();
            continue;
        }

        let mut depth = 0i32;
        let mut end = None;
        for (offset, ch) in text[start..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(start + offset + ch.len_utf8());
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(end) = end {
            blocks.push(&text[start..end]);
            search_offset = end;
        } else {
            break;
        }
    }
    blocks
}

fn first_quoted_value_after_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let after_name = trimmed.strip_prefix("name")?;
    if !after_name.starts_with('=') && !after_name.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    let (_, value_side) = trimmed.split_once('=')?;
    let value_side = value_side.trim_start();
    let rest = value_side.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub fn library_name(block: &str) -> Option<String> {
    block.lines().find_map(first_quoted_value_after_name)
}

fn is_proc_macro(block: &str) -> bool {
    block.lines().any(|line| {
        let compact = line
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>();
        compact == "proc_macro=True," || compact == "proc_macro=True"
    })
}

pub fn make_test_block(lib_block: &str, name: &str) -> String {
    let mut output = lib_block.replacen("rust_library(", "rust_test(", 1);
    let old = format!("name = \"{name}\"");
    let new = format!("name = \"{name}-unittest\"");
    output = output.replacen(&old, &new, 1);
    output
}

pub fn process_text(text: &str) -> ProcessResult {
    if !rust_rule_blocks(text, "rust_test").is_empty() {
        return ProcessResult {
            text: text.to_string(),
            added: 0,
        };
    }

    let additions = rust_rule_blocks(text, "rust_library")
        .into_iter()
        .filter(|block| !is_proc_macro(block))
        .filter_map(|block| library_name(block).map(|name| make_test_block(block, &name)))
        .collect::<Vec<_>>();

    if additions.is_empty() {
        return ProcessResult {
            text: text.to_string(),
            added: 0,
        };
    }

    let mut next = text.to_string();
    if !next.ends_with('\n') {
        next.push('\n');
    }
    next.push('\n');
    next.push_str(&additions.join("\n\n"));
    next.push('\n');

    ProcessResult {
        text: next,
        added: additions.len(),
    }
}

fn run(options: &Options) -> Result<RunSummary, String> {
    let base_rel = options
        .subsystem
        .as_deref()
        .unwrap_or_else(|| Path::new(""));
    validate_relative_scan_path(base_rel)?;
    let mut buck_files = Vec::new();
    collect_buck_files(&options.repo_root, base_rel, &mut buck_files)?;

    let mut summary = RunSummary {
        changed_files: 0,
        added_targets: 0,
        skipped_files: 0,
        changes: Vec::new(),
    };

    for rel in buck_files {
        let abs = options.repo_root.join(&rel);
        let text = fs::read_to_string(&abs)
            .map_err(|error| format!("read {} failed: {error}", abs.display()))?;
        let processed = process_text(&text);
        if processed.added == 0 {
            summary.skipped_files += 1;
            continue;
        }
        summary.changed_files += 1;
        summary.added_targets += processed.added;
        summary.changes.push(FileChange {
            rel_path: normalize_rel(&rel),
            added: processed.added,
        });
        if !options.dry_run {
            fs::write(&abs, processed.text)
                .map_err(|error| format!("write {} failed: {error}", abs.display()))?;
        }
    }

    Ok(summary)
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let program = args
        .first()
        .map(String::as_str)
        .unwrap_or("append-missing-rust-unit-test-targets");
    let options = match parse_args(&args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}\n\n{}", usage(program));
            std::process::exit(2);
        }
    };

    if options.help {
        print!("{}", usage(program));
        return;
    }
    if options.version {
        println!("append-missing-rust-unit-test-targets 1.0.0");
        return;
    }

    let summary = match run(&options) {
        Ok(summary) => summary,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    for change in &summary.changes {
        if options.dry_run {
            println!(
                "[DRY-RUN] +{} rust_test -> {}",
                change.added, change.rel_path
            );
        } else {
            println!("  +{} rust_test -> {}", change.added, change.rel_path);
        }
    }
    let action = if options.dry_run {
        "would be updated"
    } else {
        "updated"
    };
    eprintln!(
        "Summary: {} BUCK files {action}, {} rust_test added, {} skipped (no lib / already has test).",
        summary.changed_files, summary.added_targets, summary.skipped_files
    );
}
