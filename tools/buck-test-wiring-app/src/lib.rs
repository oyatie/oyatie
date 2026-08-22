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
    Checked(CheckReport),
    Applied(Vec<AppliedMember>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckReport {
    pub candidates: Vec<CandidateSummary>,
    pub diagnostics: Vec<UnsupportedMemberDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateSummary {
    pub member_path: String,
    pub target_labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedMemberDiagnostic {
    pub member_path: String,
    pub buck_path: String,
    pub code: String,
    pub message: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateCollection {
    candidates: Vec<Candidate>,
    diagnostics: Vec<UnsupportedMemberDiagnostic>,
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
    let collection = collect_candidates(repo_root, options.root_filter.as_deref())?;
    match options.mode {
        CommandMode::List => Ok(Outcome::Listed(summarize_candidates(
            &collection.candidates,
        )?)),
        CommandMode::Check => check_report_from_collection(collection).map(Outcome::Checked),
        CommandMode::Apply => {
            let limit = options
                .limit
                .ok_or_else(|| anyhow!("--apply requires --limit N"))?;
            apply_candidates(repo_root, collection.candidates, limit).map(Outcome::Applied)
        }
    }
}

pub fn render_unsupported_member_diagnostic(diagnostic: &UnsupportedMemberDiagnostic) -> String {
    format!(
        "diagnostic\tcode={}\tmember={}\tbuck={}\tmessage={}",
        diagnostic.code, diagnostic.member_path, diagnostic.buck_path, diagnostic.message
    )
}

fn collect_candidates(repo_root: &Path, root_filter: Option<&str>) -> Result<CandidateCollection> {
    let members = workspace_members_kernel::resolve_member_dirs(repo_root)
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
) -> Result<CandidateCollection>
where
    F: FnMut(&str) -> Result<String>,
{
    let _ = repo_root;
    let tracked: BTreeSet<String> = tracked_paths.into_iter().collect();
    let normalized_root = normalize_root_filter(root_filter);
    let mut candidates = Vec::new();
    let mut diagnostics = Vec::new();

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

        let has_in_crate_tests = member_has_in_crate_tests(&member_path, &tracked, &mut read_text)?;
        let integration_tests = member_integration_tests(&member_path, &tracked);
        let has_tests_dir = member_has_tests_dir(&member_path, &tracked);
        if !has_in_crate_tests && !has_tests_dir {
            continue;
        }

        if find_call_block(&buck_text, "rust_library").is_none() {
            diagnostics.push(UnsupportedMemberDiagnostic {
                member_path: member_path.clone(),
                buck_path: buck_path.clone(),
                code: "unsupported_non_library_buck".to_owned(),
                message: "member has Rust test code but BUCK has no rust_library block; skipped"
                    .to_owned(),
            });
            continue;
        }

        // A rust_library built from computed Starlark (e.g. `crate_root = ROOT + "/src/lib.rs"`
        // with `mapped_srcs`) cannot be mirrored into a rust_test stanza by string reuse. Report
        // it and keep walking: ONE such member must not abort the whole run and hide every other
        // unwired member behind it.
        let library = match parse_rust_library(&buck_text) {
            Ok(library) => library,
            Err(error) => {
                diagnostics.push(UnsupportedMemberDiagnostic {
                    member_path: member_path.clone(),
                    buck_path: buck_path.clone(),
                    code: "unsupported_computed_rust_library".to_owned(),
                    message: format!(
                        "rust_library is not statically mirrorable; skipped: {error:#}"
                    ),
                });
                continue;
            }
        };

        // Dedup PER `crate_root`, never a blanket "this BUCK already says rust_test( so it is
        // wired" skip. A crate whose unit test is wired but whose `tests/*.rs` integration tests
        // are not is exactly the population the ADR-0554 affected-set gate refuses
        // (`RefuseUnowned`), and the blanket skip made every one of them invisible to this tool.
        let wired_roots = wired_test_crate_roots(&buck_text);
        let has_in_crate_tests = has_in_crate_tests && !wired_roots.contains(&library.crate_root);
        let integration_tests: Vec<String> = integration_tests
            .into_iter()
            .filter(|test_path| !wired_roots.contains(test_path))
            .collect();
        if !has_in_crate_tests && integration_tests.is_empty() {
            continue;
        }

        candidates.push(Candidate {
            member_path,
            library,
            has_in_crate_tests,
            integration_tests,
        });
    }

    candidates.sort_by(|left, right| left.member_path.cmp(&right.member_path));
    diagnostics.sort_by(|left, right| left.member_path.cmp(&right.member_path));
    Ok(CandidateCollection {
        candidates,
        diagnostics,
    })
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

fn check_report_from_collection(collection: CandidateCollection) -> Result<CheckReport> {
    Ok(CheckReport {
        candidates: summarize_candidates(&collection.candidates)?,
        diagnostics: collection.diagnostics,
    })
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

/// The `crate_root` values a `rust_test` stanza in this BUCK already covers.
///
/// A block whose `crate_root` is absent or computed yields nothing, so it is treated as covering
/// no root — the conservative direction: the tool then offers a stanza whose name may already be
/// taken, and buck2 rejects the duplicate LOUDLY at parse time. The opposite error (claiming
/// coverage that does not exist) is the silent one this whole change removes.
fn wired_test_crate_roots(buck_text: &str) -> BTreeSet<String> {
    find_call_blocks(buck_text, "rust_test")
        .iter()
        .filter_map(|block| extract_string_field(block, "crate_root").ok())
        .collect()
}

/// Every balanced `function_name(...)` block, in source order.
fn find_call_blocks(text: &str, function_name: &str) -> Vec<String> {
    let needle = format!("{function_name}(");
    let mut blocks = Vec::new();
    let mut cursor = 0usize;
    while let Some(offset) = text[cursor..].find(&needle) {
        let start = cursor + offset;
        let Some(block) = find_call_block(&text[start..], function_name) else {
            break;
        };
        cursor = start + block.len();
        blocks.push(block);
    }
    blocks
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
    for (index, target) in generated_targets(&candidate)?.iter().enumerate() {
        if index > 0 {
            rendered.push('\n');
        }
        rendered.push_str(target.stanza.trim_end_matches('\n'));
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
    candidate_collection_from_fixture(members, tracked, files, root_filter)
        .map(|collection| collection.candidates)
}

#[cfg(test)]
fn check_report_from_fixture(
    members: Vec<String>,
    tracked: Vec<String>,
    files: Vec<(&str, String)>,
    root_filter: Option<&str>,
) -> Result<CheckReport> {
    let collection = candidate_collection_from_fixture(members, tracked, files, root_filter)?;
    check_report_from_collection(collection)
}

#[cfg(test)]
fn candidate_collection_from_fixture(
    members: Vec<String>,
    tracked: Vec<String>,
    files: Vec<(&str, String)>,
    root_filter: Option<&str>,
) -> Result<CandidateCollection> {
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
    name = "example-kernel",
    srcs = glob(["src/**/*.rs"]),
    crate = "example_kernel",
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
            member_path: "libs/example-kernel",
            buck_text: include_str!("../fixtures/library_with_tests.input.txt"),
            has_in_crate_tests: true,
            integration_tests: vec!["tests/contract_check.rs".to_owned()],
        };

        let rendered = render_stanzas_for_fixture(&candidate).unwrap();

        assert_eq!(
            rendered,
            include_str!("../fixtures/library_with_tests.generated.expected.txt")
        );
    }

    /// A `rust_test` stanza already covering `crate_root`, i.e. what a wired unit test looks like.
    fn wired_unit_test_buck() -> String {
        format!(
            "{}\nrust_test(\n    name = \"example-kernel-unittest\",\n    srcs = glob([\"src/**/*.rs\"]),\n    crate = \"example_kernel\",\n    crate_root = \"src/lib.rs\",\n    visibility = [\"PUBLIC\"],\n)\n",
            library_buck()
        )
    }

    /// REGRESSION: the blanket `buck_text.contains("rust_test(")` skip hid every crate whose unit
    /// test was wired but whose `tests/*.rs` were not — the exact population the ADR-0554
    /// affected-set gate refuses with `RefuseUnowned`. Wiring must be decided per `crate_root`.
    #[test]
    fn offers_integration_stanza_when_only_the_unit_crate_root_is_wired() {
        let tracked = vec![
            "libs/example-kernel/BUCK".to_owned(),
            "libs/example-kernel/Cargo.toml".to_owned(),
            "libs/example-kernel/src/lib.rs".to_owned(),
            "libs/example-kernel/tests/contract.rs".to_owned(),
        ];
        let members = vec!["libs/example-kernel".to_owned()];
        let files = vec![
            ("libs/example-kernel/BUCK", wired_unit_test_buck()),
            (
                "libs/example-kernel/src/lib.rs",
                "#[cfg(test)] mod tests {}".to_owned(),
            ),
        ];

        let candidates = candidates_from_fixture(members, tracked, files, None).unwrap();

        assert_eq!(candidates.len(), 1, "the unwired tests/ file must surface");
        // The already-wired unit crate_root must NOT be offered a second time.
        assert!(!candidates[0].has_in_crate_tests);
        assert_eq!(candidates[0].integration_tests, ["tests/contract.rs"]);
        let names: Vec<String> = generated_targets(&candidates[0])
            .unwrap()
            .into_iter()
            .map(|target| target.name)
            .collect();
        assert_eq!(names, ["example-kernel-contract"]);
    }

    /// REGRESSION: a single member whose `rust_library` is built from computed Starlark used to
    /// abort the ENTIRE run with `Err`, hiding every other unwired member behind it.
    #[test]
    fn computed_rust_library_is_a_diagnostic_not_a_run_abort() {
        let computed_buck = r#"ROOT = "libs/computed-kernel"

rust_library(
    name = "computed-kernel",
    srcs = [],
    crate = "computed_kernel",
    crate_root = ROOT + "/src/lib.rs",
    visibility = ["PUBLIC"],
    mapped_srcs = {},
)
"#;
        let tracked = vec![
            "libs/computed-kernel/BUCK".to_owned(),
            "libs/computed-kernel/Cargo.toml".to_owned(),
            "libs/computed-kernel/tests/contract.rs".to_owned(),
            "libs/example-kernel/BUCK".to_owned(),
            "libs/example-kernel/Cargo.toml".to_owned(),
            "libs/example-kernel/tests/contract.rs".to_owned(),
        ];
        let members = vec![
            "libs/computed-kernel".to_owned(),
            "libs/example-kernel".to_owned(),
        ];
        let files = vec![
            ("libs/computed-kernel/BUCK", computed_buck.to_owned()),
            ("libs/example-kernel/BUCK", library_buck().to_owned()),
        ];

        let report = check_report_from_fixture(members, tracked, files, None).unwrap();

        let candidate_paths: Vec<&str> = report
            .candidates
            .iter()
            .map(|candidate| candidate.member_path.as_str())
            .collect();
        assert_eq!(
            candidate_paths,
            ["libs/example-kernel"],
            "the healthy member must still be collected"
        );
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            "unsupported_computed_rust_library"
        );
        assert_eq!(
            report.diagnostics[0].member_path,
            "libs/computed-kernel"
        );
    }

    #[test]
    fn filters_alphabetical_lib_candidates_without_existing_rust_test() {
        let tracked = vec![
            "libs/beta-kernel/BUCK".to_owned(),
            "libs/beta-kernel/Cargo.toml".to_owned(),
            "libs/beta-kernel/src/lib.rs".to_owned(),
            "libs/alpha-kernel/BUCK".to_owned(),
            "libs/alpha-kernel/Cargo.toml".to_owned(),
            "libs/alpha-kernel/tests/contract.rs".to_owned(),
            "libs/gamma-kernel/BUCK".to_owned(),
            "libs/gamma-kernel/Cargo.toml".to_owned(),
            "libs/gamma-kernel/src/lib.rs".to_owned(),
        ];
        let members = vec![
            "libs/beta-kernel".to_owned(),
            "libs/alpha-kernel".to_owned(),
            "libs/gamma-kernel".to_owned(),
        ];
        let files = vec![
            ("libs/alpha-kernel/BUCK", library_buck().to_owned()),
            ("libs/beta-kernel/BUCK", library_buck().to_owned()),
            (
                "libs/beta-kernel/src/lib.rs",
                "#[cfg(test)] mod tests {}".to_owned(),
            ),
            // gamma's in-crate tests are ALREADY wired: a rust_test covering this crate_root
            // exists, so gamma must not be offered again. (The stanza carries a real crate_root —
            // a bare `rust_test(name = "existing")` proves nothing about coverage.)
            ("libs/gamma-kernel/BUCK", wired_unit_test_buck()),
            (
                "libs/gamma-kernel/src/lib.rs",
                "#[cfg(test)] mod tests {}".to_owned(),
            ),
        ];

        let candidates = candidates_from_fixture(members, tracked, files, Some("libs/")).unwrap();

        let paths: Vec<&str> = candidates
            .iter()
            .map(|candidate| candidate.member_path.as_str())
            .collect();
        assert_eq!(paths, ["libs/alpha-kernel", "libs/beta-kernel"]);
        assert_eq!(candidates[0].integration_tests, ["tests/contract.rs"]);
        assert!(candidates[1].has_in_crate_tests);
    }

    #[test]
    fn renders_unit_stanza_when_library_has_no_deps_assignment() {
        let candidate = CandidateFixture {
            member_path: "libs/example-kernel",
            buck_text: r#"rust_library(
    name = "example-kernel",
    srcs = glob(["src/**/*.rs"]),
    crate = "example_kernel",
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
    name = "example-kernel-unittest",
    srcs = glob(["src/**/*.rs"]),
    crate = "example_kernel",
    crate_root = "src/lib.rs",
    visibility = ["PUBLIC"],
    deps = [],
)
"#
        );
    }

    #[test]
    fn append_targets_keeps_single_final_newline() {
        let candidate = CandidateFixture {
            member_path: "libs/example-kernel",
            buck_text: include_str!("../fixtures/library_append.input.txt"),
            has_in_crate_tests: true,
            integration_tests: vec!["tests/contract_check.rs".to_owned()],
        };
        let library = parse_rust_library(candidate.buck_text).unwrap();
        let candidate = Candidate {
            member_path: candidate.member_path.to_owned(),
            library,
            has_in_crate_tests: candidate.has_in_crate_tests,
            integration_tests: candidate.integration_tests.clone(),
        };
        let targets = generated_targets(&candidate).unwrap();
        let rendered = append_targets_to_buck_text(
            include_str!("../fixtures/library_append.input.txt").to_owned(),
            &targets,
        );

        assert_eq!(
            rendered,
            include_str!("../fixtures/library_append.expected.txt")
        );
    }

    #[test]
    fn check_report_skips_test_bearing_buck_without_rust_library() {
        let tracked = vec![
            "tools/binary-only-app/BUCK".to_owned(),
            "tools/binary-only-app/Cargo.toml".to_owned(),
            "tools/binary-only-app/src/main.rs".to_owned(),
            "libs/example-kernel/BUCK".to_owned(),
            "libs/example-kernel/Cargo.toml".to_owned(),
            "libs/example-kernel/src/lib.rs".to_owned(),
        ];
        let members = vec![
            "tools/binary-only-app".to_owned(),
            "libs/example-kernel".to_owned(),
        ];
        let files = vec![
            (
                "tools/binary-only-app/BUCK",
                include_str!("../fixtures/binary_only.input.txt").to_owned(),
            ),
            (
                "tools/binary-only-app/src/main.rs",
                "#[cfg(test)] mod tests {}".to_owned(),
            ),
            (
                "libs/example-kernel/BUCK",
                include_str!("../fixtures/library_with_tests.input.txt").to_owned(),
            ),
            (
                "libs/example-kernel/src/lib.rs",
                "#[cfg(test)] mod tests {}".to_owned(),
            ),
        ];

        let report =
            check_report_from_fixture(members, tracked, files, None).expect("check report");

        assert_eq!(
            report.candidates,
            vec![CandidateSummary {
                member_path: "libs/example-kernel".to_owned(),
                target_labels: vec![
                    "//libs/example-kernel:example-kernel-unittest".to_owned()
                ],
            }]
        );
        assert_eq!(
            report.diagnostics,
            vec![UnsupportedMemberDiagnostic {
                member_path: "tools/binary-only-app".to_owned(),
                buck_path: "tools/binary-only-app/BUCK".to_owned(),
                code: "unsupported_non_library_buck".to_owned(),
                message: "member has Rust test code but BUCK has no rust_library block; skipped"
                    .to_owned(),
            }]
        );
    }

    #[test]
    fn check_report_is_empty_when_member_already_has_rust_test() {
        let tracked = vec![
            "libs/example-kernel/BUCK".to_owned(),
            "libs/example-kernel/Cargo.toml".to_owned(),
            "libs/example-kernel/src/lib.rs".to_owned(),
        ];
        let members = vec!["libs/example-kernel".to_owned()];
        let files = vec![
            (
                "libs/example-kernel/BUCK",
                // The already-wired stanza must name the crate_root it covers. A bare
                // `rust_test(name = ...)` says nothing about which sources are wired, and
                // treating it as full coverage is the defect this suite now pins.
                format!(
                    "{}\nrust_test(\n    name = \"example-kernel-unittest\",\n    crate_root = \"src/lib.rs\",\n)\n",
                    include_str!("../fixtures/library_with_tests.input.txt")
                ),
            ),
            (
                "libs/example-kernel/src/lib.rs",
                "#[cfg(test)] mod tests {}".to_owned(),
            ),
        ];

        let report =
            check_report_from_fixture(members, tracked, files, None).expect("check report");

        assert_eq!(
            report,
            CheckReport {
                candidates: Vec::new(),
                diagnostics: Vec::new(),
            }
        );
    }
}
