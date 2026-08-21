// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode};

use check_authoritative_tracked_kernel::{
    AuthoritativeArtifact, AuthoritativeTrackedFitnessReport, check,
};

const DEFAULT_AUTHORITY_SOURCE: &str = "docs/AGENTS.md";
const CANONICAL_SECTION: &str = "## Canonical doc map";
const NEXT_SECTION_PREFIX: &str = "## ";

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(report) => {
            println!(
                "authoritative-tracked ok: artifacts_checked={} tracked_artifacts={}",
                report.artifacts_checked, report.tracked_artifacts,
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("authoritative-tracked failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run<I>(args: I) -> Result<AuthoritativeTrackedFitnessReport, String>
where
    I: IntoIterator<Item = String>,
{
    let options = Options::parse(args)?;
    let authority_contents = fs::read_to_string(&options.authority_source).map_err(|error| {
        format!(
            "could not read authority source {}: {error}",
            options.authority_source.display()
        )
    })?;
    let authoritative_paths =
        parse_authoritative_paths(&authority_contents, &options.authority_source)?;
    let tracked_paths = git_ls_files(&options.repo_root)?;

    let mut artifacts = Vec::new();
    for path in authoritative_paths {
        let on_disk = options.repo_root.join(&path).exists();
        let tracked = is_tracked(&path, &tracked_paths);
        let gitignored = git_check_ignored(&options.repo_root, &path)?;
        artifacts.push(AuthoritativeArtifact {
            path,
            tracked,
            on_disk,
            gitignored,
        });
    }

    check(&artifacts).map_err(|error| error.message())
}

struct Options {
    repo_root: PathBuf,
    authority_source: PathBuf,
}

impl Options {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut repo_root = PathBuf::from(".");
        let mut authority_source = PathBuf::from(DEFAULT_AUTHORITY_SOURCE);
        let args = args.into_iter().collect::<Vec<_>>();
        let mut index = 0usize;
        while index < args.len() {
            match args[index].as_str() {
                "--repo-root" => {
                    index += 1;
                    repo_root = args
                        .get(index)
                        .map(PathBuf::from)
                        .ok_or_else(|| "--repo-root requires a path".to_string())?;
                }
                "--authority-source" => {
                    index += 1;
                    authority_source = args
                        .get(index)
                        .map(PathBuf::from)
                        .ok_or_else(|| "--authority-source requires a path".to_string())?;
                }
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument '{other}'\n{}", usage())),
            }
            index += 1;
        }

        Ok(Self {
            repo_root,
            authority_source,
        })
    }
}

fn usage() -> String {
    "usage: oya-governance-authoritative-tracked-app [--repo-root PATH] [--authority-source PATH]"
        .into()
}

fn parse_authoritative_paths(
    contents: &str,
    authority_source: &Path,
) -> Result<Vec<String>, String> {
    let section = canonical_doc_map_section(contents)?;
    let base_dir = authority_source.parent().unwrap_or_else(|| Path::new("."));
    let mut paths = BTreeSet::new();
    for link in markdown_links(section) {
        if let Some(path) = normalize_authority_link(base_dir, &link) {
            paths.insert(path);
        }
    }
    if paths.is_empty() {
        return Err("docs/AGENTS.md canonical doc map contains no local authority links".into());
    }
    Ok(paths.into_iter().collect())
}

fn canonical_doc_map_section(contents: &str) -> Result<&str, String> {
    let start = contents
        .find(CANONICAL_SECTION)
        .ok_or_else(|| format!("missing '{CANONICAL_SECTION}' section"))?;
    let after_start = &contents[start + CANONICAL_SECTION.len()..];
    let end = after_start
        .find(NEXT_SECTION_PREFIX)
        .unwrap_or(after_start.len());
    Ok(&after_start[..end])
}

fn markdown_links(contents: &str) -> Vec<String> {
    let mut links = Vec::new();
    let bytes = contents.as_bytes();
    let mut index = 0usize;
    while index + 1 < bytes.len() {
        if bytes[index] == b']' && bytes[index + 1] == b'(' {
            let start = index + 2;
            if let Some(end) = contents[start..].find(')') {
                links.push(contents[start..start + end].trim().to_string());
                index = start + end + 1;
                continue;
            }
        }
        index += 1;
    }
    links
}

fn normalize_authority_link(base_dir: &Path, link: &str) -> Option<String> {
    let without_title = link.split_whitespace().next().unwrap_or(link);
    let without_fragment = without_title.split('#').next().unwrap_or(without_title);
    let without_query = without_fragment
        .split('?')
        .next()
        .unwrap_or(without_fragment);
    if without_query.is_empty()
        || without_query.starts_with("http://")
        || without_query.starts_with("https://")
        || without_query.starts_with("mailto:")
    {
        return None;
    }
    Some(slash(&normalize_path(&base_dir.join(without_query))))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn git_ls_files(repo_root: &Path) -> Result<BTreeSet<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("ls-files")
        .arg("-z")
        .output()
        .map_err(|error| format!("could not run tracked-file snapshot command: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "tracked-file snapshot command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn git_check_ignored(repo_root: &Path, path: &str) -> Result<bool, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("check-ignore")
        .arg("-q")
        .arg("--")
        .arg(path)
        .output()
        .map_err(|error| format!("could not run ignore-rule check: {error}"))?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(format!(
            "ignore-rule check failed for {path}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
    }
}

fn is_tracked(path: &str, tracked_paths: &BTreeSet<String>) -> bool {
    tracked_paths.contains(path)
        || tracked_paths
            .iter()
            .any(|tracked| tracked.starts_with(&format!("{path}/")))
}

fn slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_doc_map_links_and_normalizes_relative_paths() {
        let contents = r#"
# Contract

## Canonical doc map

| Question | Authority |
|---|---|
| Mission | [`specs/masterplan.json`](../specs/masterplan.json), [`RACI`](RACI-OWNERSHIP.md) |
| Docs | [`products/`](products/) |

## Pre-flight checklist
"#;

        let mut paths =
            parse_authoritative_paths(contents, Path::new("docs/AGENTS.md")).expect("paths parse");
        paths.sort();

        assert_eq!(
            paths,
            vec![
                "docs/RACI-OWNERSHIP.md".to_string(),
                "docs/products".to_string(),
                "specs/masterplan.json".to_string(),
            ]
        );
    }

    #[test]
    fn skips_external_and_anchor_only_links() {
        assert_eq!(
            normalize_authority_link(Path::new("docs"), "https://example.test"),
            None
        );
        assert_eq!(normalize_authority_link(Path::new("docs"), "#local"), None);
    }

    #[test]
    fn treats_directory_as_tracked_when_a_child_is_tracked() {
        let tracked = BTreeSet::from([
            "docs/products/foundry/PRD.md".to_string(),
            "docs/AGENTS.md".to_string(),
        ]);

        assert!(is_tracked("docs/products", &tracked));
        assert!(is_tracked("docs/AGENTS.md", &tracked));
        assert!(!is_tracked("docs/missing", &tracked));
    }
}
