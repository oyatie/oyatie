use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_foundry_fitness_archive_orphan_kernel::{
    ArchiveOrphanFitnessReport, ArchivedPath, DEFAULT_ARCHIVE_ROOT, InboundRef, check,
};

const DEFAULT_ADR: &str = "docs/decisions/ADR-0052-inventory-grit-cutover.md";
const DEFAULT_PATH_ROOT: &str = "..";
const ARCHIVE_MARKER: &str = "| ARCHIVE |";

const DEFAULT_ALLOWED_SOURCES: &[&str] = &[
    "docs/decisions/ADR-0052-inventory-grit-cutover.md",
    ".omc/plans/ralplan-oyatie-sst-consolidation.md",
    ".omc/plans/milestones/M-CC-cross-cutting/phases/P01-agentic-pipeline-cutover/IP-008-archive-glue.md",
    ".omc/plans/milestones/M-CC-cross-cutting/phases/P01-agentic-pipeline-cutover/IP-009-delete-active-path.md",
    ".omc/specs/foundry-salvage-from-ultragoal-2026-05-12.md",
    ".omc/specs/inventory-draft-oyatie-cutover.md",
    ".omc/fitness-lanes/archive-orphan.md",
    "docs/fitness-lanes/archive-orphan.md",
    ".omc/evidence/agentic-pipeline/ip-008-archive-glue.json",
    "docs/plans/M-CC-01-cutover/INDEX.md",
    "docs/plans/M-CC-01-cutover/architect-review-iter-1.md",
    "docs/plans/M-CC-01-cutover/architect-review-iter-2.md",
    "docs/plans/M-CC-01-cutover/open-questions-resolutions.md",
    "docs/plans/M-CC-01-cutover/cross-cutting-amendments.md",
    "bominal/agents/ultragoal/DEPRECATED.md",
];

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(report) => {
            println!(
                "archive-orphan ok: archives_checked={} archive_files_present={} originals_absent={} inbound_refs_checked={}",
                report.archives_checked,
                report.archive_files_present,
                report.originals_absent,
                report.inbound_refs_checked,
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("archive-orphan failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run<I>(args: I) -> Result<ArchiveOrphanFitnessReport, String>
where
    I: IntoIterator<Item = String>,
{
    let options = Options::parse(args)?;
    let adr_contents = fs::read_to_string(&options.adr)
        .map_err(|error| format!("could not read {}: {error}", options.adr.display()))?;
    let archive_rows = parse_archive_rows(&adr_contents)?;
    let archived = archive_rows
        .iter()
        .map(|original_path| archived_path(original_path, &options.path_root))
        .collect::<Vec<_>>();
    let refs = collect_inbound_refs(&options, &archive_rows)?;
    check(&archived, &refs).map_err(|error| error.message())
}

struct Options {
    adr: PathBuf,
    path_root: PathBuf,
    scan_roots: Vec<PathBuf>,
    allowed_sources: Vec<String>,
}

impl Options {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut adr = env::var("ARCHIVE_ORPHAN_ADR_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_ADR));
        let mut path_root = env::var("ARCHIVE_ORPHAN_PATH_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_PATH_ROOT));
        let mut scan_roots = Vec::new();
        let mut allowed_sources = DEFAULT_ALLOWED_SOURCES
            .iter()
            .map(|source| (*source).to_string())
            .collect::<Vec<_>>();

        let args = args.into_iter().collect::<Vec<_>>();
        let mut index = 0usize;
        while index < args.len() {
            match args[index].as_str() {
                "--adr" => {
                    index += 1;
                    adr = args
                        .get(index)
                        .map(PathBuf::from)
                        .ok_or_else(|| "--adr requires a path".to_string())?;
                }
                "--path-root" => {
                    index += 1;
                    path_root = args
                        .get(index)
                        .map(PathBuf::from)
                        .ok_or_else(|| "--path-root requires a path".to_string())?;
                }
                "--scan-root" => {
                    index += 1;
                    scan_roots.push(
                        args.get(index)
                            .map(PathBuf::from)
                            .ok_or_else(|| "--scan-root requires a path".to_string())?,
                    );
                }
                "--allow-source" => {
                    index += 1;
                    allowed_sources.push(
                        args.get(index)
                            .cloned()
                            .ok_or_else(|| "--allow-source requires a path".to_string())?,
                    );
                }
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument '{other}'\n{}", usage())),
            }
            index += 1;
        }

        if scan_roots.is_empty() {
            scan_roots = default_scan_roots(&path_root);
        }

        Ok(Self {
            adr,
            path_root,
            scan_roots,
            allowed_sources,
        })
    }
}

fn usage() -> String {
    "usage: oya-foundry-fitness-archive-orphan [--adr PATH] [--path-root PATH] [--scan-root PATH ...] [--allow-source PATH ...]".into()
}

fn default_scan_roots(path_root: &Path) -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("docs"),
        PathBuf::from(".omc"),
        PathBuf::from("crates"),
        PathBuf::from("tools"),
        PathBuf::from("Cargo.toml"),
        PathBuf::from("AGENTS.md"),
        PathBuf::from("CLAUDE.md"),
    ];
    let bominal_ultragoal = path_root.join("bominal/agents/ultragoal");
    if bominal_ultragoal.exists() {
        roots.push(bominal_ultragoal);
    }
    roots
}

fn parse_archive_rows(contents: &str) -> Result<Vec<String>, String> {
    let mut rows = Vec::new();
    for line in contents.lines() {
        if !line.starts_with("| bominal/") || !line.contains(ARCHIVE_MARKER) {
            continue;
        }
        let cells = line
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cells.len() < 4 {
            return Err(format!("malformed archive row: {line}"));
        }
        if cells[2] == "ARCHIVE" {
            rows.push(cells[0].to_string());
        }
    }
    rows.sort();
    rows.dedup();
    if rows.is_empty() {
        return Err("ADR has no ARCHIVE rows under bominal/".into());
    }
    Ok(rows)
}

fn archived_path(original_path: &str, path_root: &Path) -> ArchivedPath {
    let file_name = Path::new(original_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(original_path);
    let archive_path = format!("{DEFAULT_ARCHIVE_ROOT}/{file_name}");
    ArchivedPath {
        original_path: original_path.to_string(),
        original_exists: path_root.join(original_path).exists(),
        archive_exists: path_root.join(&archive_path).exists(),
        archive_path,
    }
}

fn collect_inbound_refs(
    options: &Options,
    archived_paths: &[String],
) -> Result<Vec<InboundRef>, String> {
    let mut targets = Vec::new();
    for original_path in archived_paths {
        targets.push(original_path.clone());
        let file_name = Path::new(original_path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("archive row has no filename: {original_path}"))?;
        targets.push(format!("{DEFAULT_ARCHIVE_ROOT}/{file_name}"));
    }

    let mut refs = Vec::new();
    for root in &options.scan_roots {
        for path in collect_files(root)? {
            let source_path = normalize_source_path(&path, &options.path_root);
            if should_skip_source(&source_path, &options.allowed_sources) {
                continue;
            }
            let contents = match fs::read_to_string(&path) {
                Ok(contents) => contents,
                Err(_) => continue,
            };
            for (index, line) in contents.lines().enumerate() {
                for target in &targets {
                    if line.contains(target) {
                        refs.push(InboundRef {
                            source_path: source_path.clone(),
                            target_path: target.clone(),
                            line: (index + 1) as u32,
                            context: line.trim().to_string(),
                        });
                    }
                }
            }
        }
    }
    Ok(refs)
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    if root.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(&path)
            .map_err(|error| format!("could not read directory {}: {error}", path.display()))?;
        for entry in entries {
            let entry =
                entry.map_err(|error| format!("could not read directory entry: {error}"))?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| format!("could not read file type {}: {error}", path.display()))?;
            if file_type.is_dir() {
                if should_skip_dir(&path) {
                    continue;
                }
                stack.push(path);
            } else if file_type.is_file() && is_textish(&path) {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn should_skip_dir(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    if normalized.contains("/archive/pre-grit-cutover-2026-05-12") {
        return true;
    }
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".git" | "target" | "node_modules" | ".next" | "dist" | "build"
            )
        })
}

fn is_textish(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| {
            matches!(
                extension,
                "md" | "json"
                    | "jsonl"
                    | "toml"
                    | "yaml"
                    | "yml"
                    | "rs"
                    | "ts"
                    | "tsx"
                    | "js"
                    | "jsx"
                    | "py"
                    | "sh"
                    | "txt"
                    | "lock"
            )
        })
}

fn normalize_source_path(path: &Path, path_root: &Path) -> String {
    if let Ok(relative) = path.strip_prefix(path_root) {
        return slash(relative);
    }
    if let Ok(cwd) = env::current_dir()
        && let Ok(relative) = path.strip_prefix(cwd)
    {
        return slash(relative);
    }
    slash(path)
}

fn slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn should_skip_source(path: &str, allowed_sources: &[String]) -> bool {
    if path.starts_with(DEFAULT_ARCHIVE_ROOT) {
        return true;
    }
    allowed_sources
        .iter()
        .any(|allowed| path == allowed || path.starts_with(&format!("{allowed}/")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_archive_rows_from_inventory_table() {
        let ledger = original("ledger.jsonl");
        let table = format!(
            "\n| Path | Type | Classification | Archived at |\n| {ledger} | file | ARCHIVE | 2026-05-14T00:00:00Z |\n| bominal/agents/ultragoal/README.md | file | KEEP | null |\n"
        );
        let rows = parse_archive_rows(&table).expect("rows parse");

        assert_eq!(rows, vec![original("ledger.jsonl")]);
    }

    #[test]
    fn computes_archive_path_from_original_filename() {
        let path = archived_path(
            &original("codex-goal-G001-active.json"),
            Path::new("/definitely/missing"),
        );

        assert_eq!(
            path.archive_path,
            format!("{DEFAULT_ARCHIVE_ROOT}/codex-goal-G001-active.json")
        );
        assert!(!path.original_exists);
        assert!(!path.archive_exists);
    }

    #[test]
    fn skips_archive_dir_and_allowed_sources() {
        assert!(should_skip_dir(Path::new(
            "../bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12"
        )));
        assert!(should_skip_source(
            "docs/decisions/ADR-0052-inventory-grit-cutover.md",
            &DEFAULT_ALLOWED_SOURCES
                .iter()
                .map(|source| (*source).into())
                .collect::<Vec<String>>()
        ));
        assert!(!should_skip_source(
            "docs/runbooks/live-consumer.md",
            &DEFAULT_ALLOWED_SOURCES
                .iter()
                .map(|source| (*source).into())
                .collect::<Vec<String>>()
        ));
    }

    #[test]
    fn normalizes_sibling_bominal_paths_under_path_root() {
        let normalized = normalize_source_path(
            Path::new("../bominal/agents/ultragoal/DEPRECATED.md"),
            Path::new(".."),
        );

        assert_eq!(normalized, "bominal/agents/ultragoal/DEPRECATED.md");
    }

    fn original(name: &str) -> String {
        ["bominal", "agents", "ultragoal", name].join("/")
    }

    #[test]
    fn collect_missing_root_is_empty() {
        let files = collect_files(Path::new("/definitely/missing/archive-orphan"))
            .expect("missing roots are ignored");

        assert!(files.is_empty());
    }
}
