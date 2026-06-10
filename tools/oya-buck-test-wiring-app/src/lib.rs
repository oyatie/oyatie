#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

#[cfg(test)]
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandMode {
    List,
    Check,
    Apply,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub mode: CommandMode,
    pub root_filter: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Listed(Vec<CandidateSummary>),
    Checked(Vec<CandidateSummary>),
    Applied(Vec<AppliedMember>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateSummary {
    pub member_path: String,
    pub target_labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedMember {
    pub member_path: String,
    pub target_labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Candidate {
    member_path: String,
    library: RustLibrary,
    has_in_crate_tests: bool,
    integration_tests: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustLibrary {
    name: String,
    crate_name: String,
    crate_root: String,
    srcs_expr: String,
    deps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GeneratedTarget {
    name: String,
    stanza: String,
}

pub fn discover_repo_root(start: &Path) -> Result<PathBuf> {
    let mut cursor = if start.is_file() {
        start
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow!("cannot discover repo root from {}", start.display()))?
    } else {
        start.to_path_buf()
    };

    loop {
        if cursor.join("specs/root-hub-pointers.json").is_file()
            && cursor.join(".buckroot").exists()
        {
            return Ok(cursor);
        }
        if !cursor.pop() {
            bail!("could not find repo root from {}", start.display());
        }
    }
}

pub fn run(repo_root: &Path, options: Options) -> Result<Outcome> {
    let candidates = collect_candidates(repo_root, options.root_filter.as_deref())?;
    match options.mode {
        CommandMode::List => Ok(Outcome::Listed(summarize_candidates(&candidates)?)),
        CommandMode::Check => Ok(Outcome::Checked(summarize_candidates(&candidates)?)),
        CommandMode::Apply => {
            let limit = options
                .limit
                .ok_or_else(|| anyhow!("--apply requires --limit N"))?;
            apply_candidates(repo_root, candidates, limit).map(Outcome::Applied)
        }
    }
}

fn collect_candidates(repo_root: &Path, root_filter: Option<&str>) -> Result<Vec<Candidate>> {
    let members = oya_workspace_members_kernel::resolve_member_dirs(repo_root)
        .map_err(|error| anyhow!("resolve workspace members: {error}"))?;
    let tracked_paths = git_tracked_paths(repo_root)?;
    collect_candidates_from_parts(repo_root, members, tracked_paths, root_filter, |relative| {
        fs::read_to_string(repo_root.join(relative))
            .with_context(|| format!("read {}", repo_root.join(relative).display()))
    })
}

fn collect_candidates_from_parts<F>(
    repo_root: &Path,
    members: Vec<String>,
    tracked_paths: Vec<String>,
    root_filter: Option<&str>,
    mut read_text: F,
) -> Result<Vec<Candidate>>
where
    F: FnMut(&str) -> Result<String>,
{
    let _ = repo_root;
    let tracked: BTreeSet<String> = tracked_paths.into_iter().collect();
    let normalized_root = normalize_root_filter(root_filter);
    let mut candidates = Vec::new();

    for member_path in members {
        if !matches_root_filter(&member_path, normalized_root.as_deref()) {
            continue;
        }

        let cargo_path = format!("{member_path}/Cargo.toml");
        if !tracked.contains(&cargo_path) {
            continue;
        }

        let buck_path = format!("{member_path}/BUCK");
        if !tracked.contains(&buck_path) {
            continue;
        }

        let buck_text = read_text(&buck_path)?;
        if buck_text.contains("rust_test(") {
            continue;
        }

        let has_in_crate_tests = member_has_in_crate_tests(&member_path, &tracked, &mut read_text)?;
        let integration_tests = member_integration_tests(&member_path, &tracked);
        let has_tests_dir = member_has_tests_dir(&member_path, &tracked);
        if !has_in_crate_tests && !has_tests_dir {
            continue;
        }

        let library = parse_rust_library(&buck_text)
            .with_context(|| format!("parse rust_library in {buck_path}"))?;
        candidates.push(Candidate {
            member_path,
            library,
            has_in_crate_tests,
            integration_tests,
        });
    }

    candidates.sort_by(|left, right| left.member_path.cmp(&right.member_path));
    Ok(candidates)
}

fn git_tracked_paths(repo_root: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("ls-files")
        .output()
        .with_context(|| format!("run git ls-files in {}", repo_root.display()))?;
    if !output.status.success() {
        bail!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let stdout = String::from_utf8(output.stdout).context("git ls-files stdout was not UTF-8")?;
    Ok(stdout.lines().map(str::to_owned).collect())
}

fn apply_candidates(
    repo_root: &Path,
    candidates: Vec<Candidate>,
    limit: usize,
) -> Result<Vec<AppliedMember>> {
    let mut applied = Vec::new();
    for candidate in candidates.into_iter().take(limit) {
        let targets = generated_targets(&candidate)?;
        if targets.is_empty() {
            bail!(
                "candidate {} has test code but no direct generated rust_test stanzas",
                candidate.member_path
            );
        }

        let buck_path = repo_root.join(&candidate.member_path).join("BUCK");
        let mut buck_text = fs::read_to_string(&buck_path)
            .with_context(|| format!("read {}", buck_path.display()))?;
        buck_text = append_targets_to_buck_text(buck_text, &targets);
        fs::write(&buck_path, buck_text)
            .with_context(|| format!("write {}", buck_path.display()))?;

        applied.push(AppliedMember {
            member_path: candidate.member_path.clone(),
            target_labels: targets
                .into_iter()
                .map(|target| format!("//{}:{}", candidate.member_path, target.name))
                .collect(),
        });
    }
    Ok(applied)
}

fn append_targets_to_buck_text(mut buck_text: String, targets: &[GeneratedTarget]) -> String {
    while buck_text.ends_with('\n') {
        buck_text.pop();
    }
    if !buck_text.is_empty() {
        buck_text.push_str("\n\n");
    }
    for (index, target) in targets.iter().enumerate() {
        if index > 0 {
            buck_text.push('\n');
        }
        buck_text.push_str(target.stanza.trim_end_matches('\n'));
        buck_text.push('\n');
    }
    buck_text
}

fn summarize_candidates(candidates: &[Candidate]) -> Result<Vec<CandidateSummary>> {
    candidates
        .iter()
        .map(|candidate| {
            let target_labels = generated_targets(candidate)?
                .into_iter()
                .map(|target| format!("//{}:{}", candidate.member_path, target.name))
                .collect();
            Ok(CandidateSummary {
                member_path: candidate.member_path.clone(),
                target_labels,
            })
        })
        .collect()
}

fn generated_targets(candidate: &Candidate) -> Result<Vec<GeneratedTarget>> {
    let mut targets = Vec::new();
    if candidate.has_in_crate_tests {
        let name = format!("{}-unittest", candidate.library.name);
        targets.push(GeneratedTarget {
            name,
            stanza: render_unit_stanza(&candidate.library),
        });
    }

    for test_path in &candidate.integration_tests {
        let stem = test_stem(test_path)
            .ok_or_else(|| anyhow!("integration test path has no .rs stem: {test_path}"))?;
        let target_suffix = sanitize_target_fragment(stem);
        let crate_suffix = sanitize_crate_fragment(stem);
        let name = format!("{}-{target_suffix}", candidate.library.name);
        let crate_name = format!("{}_{}", candidate.library.crate_name, crate_suffix);
        targets.push(GeneratedTarget {
            name,
            stanza: render_integration_stanza(
                &candidate.library,
                test_path,
                &target_suffix,
                &crate_name,
            ),
        });
    }
    Ok(targets)
}

fn render_unit_stanza(library: &RustLibrary) -> String {
    format!(
        "rust_test(\n    name = \"{}-unittest\",\n    srcs = {},\n    crate = \"{}\",\n    crate_root = \"{}\",\n    visibility = [\"PUBLIC\"],\n{})\n",
        library.name,
        library.srcs_expr,
        library.crate_name,
        library.crate_root,
        render_deps(&library.deps),
    )
}

fn render_integration_stanza(
    library: &RustLibrary,
    test_path: &str,
    target_suffix: &str,
    crate_name: &str,
) -> String {
    let mut deps = vec![format!(":{}", library.name)];
    deps.extend(library.deps.iter().cloned());
    format!(
        "rust_test(\n    name = \"{}-{}\",\n    srcs = [\"{}\"],\n    crate = \"{}\",\n    crate_root = \"{}\",\n    visibility = [\"PUBLIC\"],\n{})\n",
        library.name,
        target_suffix,
        test_path,
        crate_name,
        test_path,
        render_deps(&deps)
    )
}

fn render_deps(deps: &[String]) -> String {
    if deps.is_empty() {
        return "    deps = [],\n".to_owned();
    }

    let mut rendered = String::from("    deps = [\n");
    for dep in deps {
        rendered.push_str(&format!("        \"{dep}\",\n"));
    }
    rendered.push_str("    ],\n");
    rendered
}

fn parse_rust_library(buck_text: &str) -> Result<RustLibrary> {
    let block = find_call_block(buck_text, "rust_library")
        .ok_or_else(|| anyhow!("missing rust_library block"))?;
    Ok(RustLibrary {
        name: extract_string_field(&block, "name")?,
        crate_name: extract_string_field(&block, "crate")?,
        crate_root: extract_string_field(&block, "crate_root")?,
        srcs_expr: extract_value_expr(&block, "srcs")?,
        deps: extract_deps(&block)?,
    })
}

fn find_call_block(text: &str, function_name: &str) -> Option<String> {
    let needle = format!("{function_name}(");
    let start = text.find(&needle)?;
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in text[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
        } else if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                let end = start + offset + ch.len_utf8();
                return Some(text[start..end].to_owned());
            }
        }
    }
    None
}

fn extract_string_field(block: &str, field: &str) -> Result<String> {
    let value = extract_value_expr(block, field)?;
    let trimmed = value.trim();
    if !trimmed.starts_with('"') {
        bail!("{field} is not a quoted string");
    }
    let mut chars = trimmed.char_indices();
    let _opening = chars.next();
    let mut escaped = false;
    for (index, ch) in chars {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Ok(trimmed[1..index].to_owned());
        }
    }
    bail!("{field} quoted string is unterminated")
}

fn extract_value_expr(block: &str, field: &str) -> Result<String> {
    let key = format!("{field} =");
    let mut found = false;
    let mut expr = String::new();
    for line in block.lines() {
        let trimmed = line.trim();
        if !found {
            let Some(rest) = trimmed.strip_prefix(&key) else {
                continue;
            };
            found = true;
            expr.push_str(rest.trim_start());
        } else {
            if !expr.is_empty() {
                expr.push('\n');
            }
            expr.push_str(trimmed);
        }

        if assignment_complete(&expr) {
            return Ok(expr.trim().trim_end_matches(',').trim().to_owned());
        }
    }
    bail!("missing {field} assignment")
}

fn assignment_complete(expr: &str) -> bool {
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    for ch in expr.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
        } else if matches!(ch, '[' | '(' | '{') {
            depth += 1;
        } else if matches!(ch, ']' | ')' | '}') {
            depth -= 1;
        } else if ch == ',' && depth == 0 {
            return true;
        }
    }
    !in_string && depth == 0 && !expr.trim().is_empty()
}

fn extract_deps(block: &str) -> Result<Vec<String>> {
    if !block
        .lines()
        .any(|line| line.trim_start().starts_with("deps ="))
    {
        return Ok(Vec::new());
    }
    let expr = extract_value_expr(block, "deps")?;
    Ok(quoted_strings(&expr))
}

fn quoted_strings(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut start: Option<usize> = None;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if let Some(value_start) = start {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                values.push(text[value_start..index].to_owned());
                start = None;
            }
        } else if ch == '"' {
            start = Some(index + ch.len_utf8());
        }
    }
    values
}

fn member_has_in_crate_tests<F>(
    member_path: &str,
    tracked: &BTreeSet<String>,
    read_text: &mut F,
) -> Result<bool>
where
    F: FnMut(&str) -> Result<String>,
{
    let src_prefix = format!("{member_path}/src/");
    for path in tracked
        .iter()
        .filter(|path| path.starts_with(&src_prefix) && path.ends_with(".rs"))
    {
        let body = read_text(path)?;
        if body.contains("#[cfg(test)]") || body.contains("#[test]") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn member_has_tests_dir(member_path: &str, tracked: &BTreeSet<String>) -> bool {
    let tests_prefix = format!("{member_path}/tests/");
    tracked.iter().any(|path| path.starts_with(&tests_prefix))
}

fn member_integration_tests(member_path: &str, tracked: &BTreeSet<String>) -> Vec<String> {
    let tests_prefix = format!("{member_path}/tests/");
    let mut tests = Vec::new();
    for path in tracked
        .iter()
        .filter(|path| path.starts_with(&tests_prefix) && path.ends_with(".rs"))
    {
        let Some(relative) = path.strip_prefix(&format!("{member_path}/")) else {
            continue;
        };
        let Some(rest) = relative.strip_prefix("tests/") else {
            continue;
        };
        if rest.contains('/') {
            continue;
        }
        tests.push(relative.to_owned());
    }
    tests.sort();
    tests
}

fn normalize_root_filter(root_filter: Option<&str>) -> Option<String> {
    let root = root_filter?.trim().trim_matches('/');
    if root.is_empty() {
        None
    } else {
        Some(root.to_owned())
    }
}

fn matches_root_filter(member_path: &str, root_filter: Option<&str>) -> bool {
    let Some(root) = root_filter else {
        return true;
    };
    member_path == root || member_path.starts_with(&format!("{root}/"))
}

fn test_stem(test_path: &str) -> Option<&str> {
    let file_name = test_path.rsplit('/').next()?;
    file_name
        .strip_suffix(".rs")
        .filter(|stem| !stem.is_empty())
}

fn sanitize_target_fragment(input: &str) -> String {
    sanitize_fragment(input, '-')
}

fn sanitize_crate_fragment(input: &str) -> String {
    sanitize_fragment(input, '_')
}

fn sanitize_fragment(input: &str, separator: char) -> String {
    let mut output = String::new();
    let mut last_was_separator = false;
    for ch in input.chars() {
        let next = if ch.is_ascii_alphanumeric() {
            ch
        } else {
            separator
        };
        if next == separator {
            if !last_was_separator && !output.is_empty() {
                output.push(separator);
            }
            last_was_separator = true;
        } else {
            output.push(next);
            last_was_separator = false;
        }
    }
    while output.ends_with(separator) {
        output.pop();
    }
    if output.is_empty() {
        "integration".to_owned()
    } else {
        output
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateFixture {
    member_path: &'static str,
    buck_text: &'static str,
    has_in_crate_tests: bool,
    integration_tests: Vec<String>,
}

#[cfg(test)]
fn render_stanzas_for_fixture(candidate: &CandidateFixture) -> Result<String> {
    let library = parse_rust_library(candidate.buck_text)?;
    let candidate = Candidate {
        member_path: candidate.member_path.to_owned(),
        library,
        has_in_crate_tests: candidate.has_in_crate_tests,
        integration_tests: candidate.integration_tests.clone(),
    };
    let mut rendered = String::new();
    for target in generated_targets(&candidate)? {
        rendered.push_str(&target.stanza);
        rendered.push('\n');
    }
    Ok(rendered)
}

#[cfg(test)]
fn candidates_from_fixture(
    members: Vec<String>,
    tracked: Vec<String>,
    files: Vec<(&str, String)>,
    root_filter: Option<&str>,
) -> Result<Vec<Candidate>> {
    let files: BTreeMap<String, String> = files
        .into_iter()
        .map(|(path, contents)| (path.to_owned(), contents))
        .collect();
    collect_candidates_from_parts(Path::new("."), members, tracked, root_filter, |relative| {
        files
            .get(relative)
            .cloned()
            .ok_or_else(|| anyhow!("missing fixture file {relative}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library_buck() -> &'static str {
        r#"rust_library(
    name = "oya-example-kernel",
    srcs = glob(["src/**/*.rs"]),
    crate = "oya_example_kernel",
    crate_root = "src/lib.rs",
    visibility = ["PUBLIC"],
    deps = [
        "third-party//:serde",
    ],
)
"#
    }

    #[test]
    fn renders_unit_and_integration_stanzas_from_library_shape() {
        let candidate = CandidateFixture {
            member_path: "libs/oya-example-kernel",
            buck_text: library_buck(),
            has_in_crate_tests: true,
            integration_tests: vec!["tests/contract_check.rs".to_owned()],
        };

        let rendered = render_stanzas_for_fixture(&candidate).unwrap();

        assert_eq!(
            rendered,
            r#"rust_test(
    name = "oya-example-kernel-unittest",
    srcs = glob(["src/**/*.rs"]),
    crate = "oya_example_kernel",
    crate_root = "src/lib.rs",
    visibility = ["PUBLIC"],
    deps = [
        "third-party//:serde",
    ],
)

rust_test(
    name = "oya-example-kernel-contract-check",
    srcs = ["tests/contract_check.rs"],
    crate = "oya_example_kernel_contract_check",
    crate_root = "tests/contract_check.rs",
    visibility = ["PUBLIC"],
    deps = [
        ":oya-example-kernel",
        "third-party//:serde",
    ],
)

"#
        );
    }

    #[test]
    fn filters_alphabetical_lib_candidates_without_existing_rust_test() {
        let tracked = vec![
            "libs/oya-beta-kernel/BUCK".to_owned(),
            "libs/oya-beta-kernel/Cargo.toml".to_owned(),
            "libs/oya-beta-kernel/src/lib.rs".to_owned(),
            "libs/oya-alpha-kernel/BUCK".to_owned(),
            "libs/oya-alpha-kernel/Cargo.toml".to_owned(),
            "libs/oya-alpha-kernel/tests/contract.rs".to_owned(),
            "libs/oya-gamma-kernel/BUCK".to_owned(),
            "libs/oya-gamma-kernel/Cargo.toml".to_owned(),
            "libs/oya-gamma-kernel/src/lib.rs".to_owned(),
        ];
        let members = vec![
            "libs/oya-beta-kernel".to_owned(),
            "libs/oya-alpha-kernel".to_owned(),
            "libs/oya-gamma-kernel".to_owned(),
        ];
        let files = vec![
            ("libs/oya-alpha-kernel/BUCK", library_buck().to_owned()),
            ("libs/oya-beta-kernel/BUCK", library_buck().to_owned()),
            (
                "libs/oya-beta-kernel/src/lib.rs",
                "#[cfg(test)] mod tests {}".to_owned(),
            ),
            (
                "libs/oya-gamma-kernel/BUCK",
                format!("{}rust_test(name = \"existing\")\n", library_buck()),
            ),
            (
                "libs/oya-gamma-kernel/src/lib.rs",
                "#[cfg(test)] mod tests {}".to_owned(),
            ),
        ];

        let candidates = candidates_from_fixture(members, tracked, files, Some("libs/")).unwrap();

        let paths: Vec<&str> = candidates
            .iter()
            .map(|candidate| candidate.member_path.as_str())
            .collect();
        assert_eq!(paths, ["libs/oya-alpha-kernel", "libs/oya-beta-kernel"]);
        assert_eq!(candidates[0].integration_tests, ["tests/contract.rs"]);
        assert!(candidates[1].has_in_crate_tests);
    }

    #[test]
    fn renders_unit_stanza_when_library_has_no_deps_assignment() {
        let candidate = CandidateFixture {
            member_path: "libs/oya-example-kernel",
            buck_text: r#"rust_library(
    name = "oya-example-kernel",
    srcs = glob(["src/**/*.rs"]),
    crate = "oya_example_kernel",
    crate_root = "src/lib.rs",
    visibility = ["PUBLIC"],
)
"#,
            has_in_crate_tests: true,
            integration_tests: Vec::new(),
        };

        let rendered = render_stanzas_for_fixture(&candidate).unwrap();

        assert_eq!(
            rendered,
            r#"rust_test(
    name = "oya-example-kernel-unittest",
    srcs = glob(["src/**/*.rs"]),
    crate = "oya_example_kernel",
    crate_root = "src/lib.rs",
    visibility = ["PUBLIC"],
    deps = [],
)

"#
        );
    }

    #[test]
    fn append_targets_keeps_single_final_newline() {
        let target = GeneratedTarget {
            name: "oya-example-kernel-unittest".to_owned(),
            stanza: "rust_test(\n    name = \"oya-example-kernel-unittest\",\n)\n".to_owned(),
        };
        let rendered = append_targets_to_buck_text(
            "rust_library(\n    name = \"lib\",\n)\n".to_owned(),
            &[target],
        );

        assert!(rendered.contains(")\n\nrust_test("));
        assert!(rendered.ends_with(")\n"));
        assert!(!rendered.ends_with("\n\n"));
    }
}
