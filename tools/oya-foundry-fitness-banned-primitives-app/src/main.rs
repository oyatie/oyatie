use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_foundry_fitness_banned_primitives_kernel::{
    AgentInstructionSource, BannedPrimitivesFitnessReport, PrimitiveKind, PrimitiveUsage,
    check_documented_genuine_need,
};

const DEFAULT_ROOTS: [&str; 4] = ["AGENTS.md", "CLAUDE.md", "docs", ".omc"];
const START_MARKER: &str = "<!-- agent-instructions:start -->";
const END_MARKER: &str = "<!-- agent-instructions:end -->";

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
        let audit = audit_file(&path_display, &contents)?;
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
    "usage: oya-foundry-fitness-banned-primitives-app [--root PATH ...] [--known-rationale ICM_ID ...]"
        .into()
}

#[derive(Debug)]
struct FileAudit {
    source: AgentInstructionSource,
    usages: Vec<PrimitiveUsage>,
}

fn audit_file(path: &str, contents: &str) -> Result<FileAudit, String> {
    let mut in_fence = false;
    let mut fence_count = 0usize;
    let mut usages = Vec::new();

    for (index, line) in contents.lines().enumerate() {
        let line_number = (index + 1) as u32;
        let trimmed = line.trim();
        if trimmed == START_MARKER {
            if in_fence {
                return Err(format!(
                    "{path}:{line_number}: nested agent-instructions fence"
                ));
            }
            in_fence = true;
            fence_count += 1;
            continue;
        }
        if trimmed == END_MARKER {
            if !in_fence {
                return Err(format!(
                    "{path}:{line_number}: agent-instructions end without start"
                ));
            }
            in_fence = false;
            continue;
        }
        if in_fence {
            detect_usages(path, line_number, line, &mut usages);
        }
    }

    if in_fence {
        return Err(format!("{path}: unterminated agent-instructions fence"));
    }

    Ok(FileAudit {
        source: AgentInstructionSource {
            path: path.to_string(),
            fence_count,
            rewrite_verified: fence_count > 0,
        },
        usages,
    })
}

fn detect_usages(path: &str, line: u32, contents: &str, usages: &mut Vec<PrimitiveUsage>) {
    let lower = contents.to_ascii_lowercase();
    let rationale = extract_rationale(contents);

    for (needle, primitive) in [
        ("--no-verify", PrimitiveKind::HookBypass),
        ("force-with-lease", PrimitiveKind::ForcePush),
        ("push --force", PrimitiveKind::ForcePush),
        ("~/.claude/", PrimitiveKind::UserHomeMutation),
        ("~/.codex/", PrimitiveKind::UserHomeMutation),
    ] {
        if lower.contains(needle) {
            usages.push(primitive_usage(
                path,
                line,
                primitive,
                rationale.clone(),
                contents,
            ));
        }
    }

    if lower.contains("kill -9") && lower.contains("pgrep claude") || lower.contains("pkill claude")
    {
        usages.push(primitive_usage(
            path,
            line,
            PrimitiveKind::ProcessKill,
            rationale.clone(),
            contents,
        ));
    }

    if lower.contains("gh pr merge") {
        usages.push(primitive_usage(
            path,
            line,
            PrimitiveKind::ForgeMerge,
            rationale.clone(),
            contents,
        ));
    }
    if contains_word(&lower, "curl") || contains_word(&lower, "wget") {
        usages.push(primitive_usage(
            path,
            line,
            PrimitiveKind::ExternalFetch,
            rationale.clone(),
            contents,
        ));
    }
    if lower.contains("rtk git") {
        usages.push(primitive_usage(
            path,
            line,
            PrimitiveKind::TokenFilteredVcs,
            rationale.clone(),
            contents,
        ));
    } else if contains_word(&lower, "git") {
        usages.push(primitive_usage(
            path,
            line,
            PrimitiveKind::DirectVcs,
            rationale.clone(),
            contents,
        ));
    }
    if lower.contains("rtk gh") {
        usages.push(primitive_usage(
            path,
            line,
            PrimitiveKind::TokenFilteredForge,
            rationale.clone(),
            contents,
        ));
    } else if contains_word(&lower, "gh") {
        usages.push(primitive_usage(
            path,
            line,
            PrimitiveKind::DirectForge,
            rationale,
            contents,
        ));
    }
}

fn primitive_usage(
    path: &str,
    line: u32,
    primitive: PrimitiveKind,
    icm_rationale: Option<String>,
    context: &str,
) -> PrimitiveUsage {
    PrimitiveUsage {
        path: path.to_string(),
        line,
        primitive,
        icm_rationale,
        context: context.trim().to_string(),
    }
}

fn extract_rationale(line: &str) -> Option<String> {
    for marker in ["icm_rationale:", "rationale_id:", "rationale:", "icm:"] {
        if let Some((_, tail)) = line.split_once(marker) {
            let value = tail
                .trim()
                .trim_matches('`')
                .trim_matches('"')
                .trim_matches('\'')
                .split(|ch: char| ch.is_whitespace() || ch == ',' || ch == ';')
                .next()
                .unwrap_or("")
                .trim_matches('`')
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn contains_word(haystack: &str, needle: &str) -> bool {
    let mut search_start = 0usize;
    while let Some(offset) = haystack[search_start..].find(needle) {
        let start = search_start + offset;
        let end = start + needle.len();
        let before = haystack[..start].chars().next_back();
        let after = haystack[end..].chars().next();
        if !is_word_char(before) && !is_word_char(after) {
            return true;
        }
        search_start = end;
    }
    false
}

fn is_word_char(ch: Option<char>) -> bool {
    ch.map(|value| value.is_ascii_alphanumeric() || value == '_')
        .unwrap_or(false)
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

    #[test]
    fn detects_exact_fence_only() {
        let audit = audit_file(
            "docs/example.md",
            "mentions `<!-- agent-instructions:start -->` inline\n<!-- agent-instructions:start -->\ngrit\n<!-- agent-instructions:end -->",
        )
        .expect("audit succeeds");

        assert_eq!(audit.source.fence_count, 1);
    }

    #[test]
    fn detects_direct_vcs_inside_fence() {
        let audit = audit_file(
            "AGENTS.md",
            "<!-- agent-instructions:start -->\nrun git-sha collection\n<!-- agent-instructions:end -->",
        )
        .expect("audit succeeds");

        assert_eq!(audit.usages.len(), 1);
        assert_eq!(audit.usages[0].primitive, PrimitiveKind::DirectVcs);
    }

    #[test]
    fn ignores_sanctioned_grit_word() {
        let audit = audit_file(
            "AGENTS.md",
            "<!-- agent-instructions:start -->\ngrit claim and oya-tooling-agent-read log\n<!-- agent-instructions:end -->",
        )
        .expect("audit succeeds");

        assert!(audit.usages.is_empty());
    }

    #[test]
    fn detects_process_kill_inside_fence() {
        let audit = audit_file(
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
            audit_file("AGENTS.md", "<!-- agent-instructions:start -->\nicm recall").unwrap_err(),
            "AGENTS.md: unterminated agent-instructions fence"
        );
    }
}
