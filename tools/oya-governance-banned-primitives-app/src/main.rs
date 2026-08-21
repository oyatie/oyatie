// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use check_banned_primitives_kernel::{
    BannedPrimitivesFitnessReport, check_documented_genuine_need, scan_agent_instruction_file,
};

const DEFAULT_ROOTS: [&str; 4] = ["AGENTS.md", "CLAUDE.md", "docs", ".omc"];

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(report) => {
            println!(
                "banned-primitives ok: sources_checked={} fences_checked={} usages_checked={} documented_exceptions={}",
                report.sources_checked,
                report.fences_checked,
                report.usages_checked,
                report.documented_exceptions,
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("banned-primitives failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run<I>(args: I) -> Result<BannedPrimitivesFitnessReport, String>
where
    I: IntoIterator<Item = String>,
{
    let options = Options::parse(args)?;
    let mut sources = Vec::new();
    let mut usages = Vec::new();
    for path in collect_files(&options.roots)? {
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        let path_display = normalize_path(&path);
        let audit = scan_agent_instruction_file(&path_display, &contents)?;
        if audit.source.fence_count > 0 {
            sources.push(audit.source);
            usages.extend(audit.usages);
        }
    }
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    check_documented_genuine_need(&sources, &usages, &options.known_rationales)
        .map_err(|error| error.message())
}

struct Options {
    roots: Vec<PathBuf>,
    known_rationales: Vec<String>,
}

impl Options {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut roots = Vec::new();
        let mut known_rationales = Vec::new();
        let args = args.into_iter().collect::<Vec<_>>();
        let mut index = 0usize;
        while index < args.len() {
            match args[index].as_str() {
                "--root" => {
                    index += 1;
                    roots.push(
                        args.get(index)
                            .map(PathBuf::from)
                            .ok_or_else(|| "--root requires a path".to_string())?,
                    );
                }
                "--known-rationale" => {
                    index += 1;
                    known_rationales.push(
                        args.get(index)
                            .cloned()
                            .ok_or_else(|| "--known-rationale requires an id".to_string())?,
                    );
                }
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument '{other}'\n{}", usage())),
            }
            index += 1;
        }
        if roots.is_empty() {
            roots = DEFAULT_ROOTS.iter().map(PathBuf::from).collect();
        }
        Ok(Self {
            roots,
            known_rationales,
        })
    }
}

fn usage() -> String {
    "usage: oya-governance-banned-primitives-app [--root PATH ...] [--known-rationale ICM_ID ...]"
        .into()
}

fn collect_files(roots: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for root in roots {
        if root.is_file() {
            files.push(root.clone());
        } else if root.is_dir() {
            collect_dir(root, &mut files)?;
        } else {
            return Err(format!("input path does not exist: {}", root.display()));
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_dir(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(path).map_err(|error| format!("could not read {}: {error}", path.display()))?
    {
        let entry = entry.map_err(|error| format!("could not read dir entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_dir(&path, files)?;
        } else if is_scanned_file(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_scanned_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("md" | "json" | "toml" | "yaml" | "yml")
    )
}

fn normalize_path(path: &Path) -> String {
    path.strip_prefix(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .unwrap_or(path)
        .to_string_lossy()
        .trim_start_matches("./")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use check_banned_primitives_kernel::PrimitiveKind;

    #[test]
    fn detects_exact_fence_only() {
        let audit = scan_agent_instruction_file(
            "docs/example.md",
            "mentions `<!-- agent-instructions:start -->` inline\n<!-- agent-instructions:start -->\ngrit\n<!-- agent-instructions:end -->",
        )
        .expect("audit succeeds");

        assert_eq!(audit.source.fence_count, 1);
    }

    #[test]
    fn detects_direct_vcs_inside_fence() {
        let audit = scan_agent_instruction_file(
            "AGENTS.md",
            "<!-- agent-instructions:start -->\nrun git status collection\n<!-- agent-instructions:end -->",
        )
        .expect("audit succeeds");

        assert_eq!(audit.usages.len(), 1);
        assert_eq!(audit.usages[0].primitive, PrimitiveKind::DirectVcs);
    }

    #[test]
    fn ignores_sanctioned_grit_word() {
        let audit = scan_agent_instruction_file(
            "AGENTS.md",
            "<!-- agent-instructions:start -->\ngrit claim and oya-tooling-agent-read log\n<!-- agent-instructions:end -->",
        )
        .expect("audit succeeds");

        assert!(audit.usages.is_empty());
    }

    #[test]
    fn detects_process_kill_inside_fence() {
        let audit = scan_agent_instruction_file(
            "AGENTS.md",
            "<!-- agent-instructions:start -->\nkill -9 $(pgrep claude)\n<!-- agent-instructions:end -->",
        )
        .expect("audit succeeds");

        assert_eq!(audit.usages.len(), 1);
        assert_eq!(audit.usages[0].primitive, PrimitiveKind::ProcessKill);
    }

    #[test]
    fn rejects_unterminated_fence() {
        assert_eq!(
            scan_agent_instruction_file(
                "AGENTS.md",
                "<!-- agent-instructions:start -->\nicm recall"
            )
            .unwrap_err(),
            "AGENTS.md: unterminated agent-instructions fence"
        );
    }
}
