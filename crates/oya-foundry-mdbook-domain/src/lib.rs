//! mdBook source fitness kernel.
//!
//! Oyatie publishes public documentation from `docs/site/` as an mdBook-shaped
//! source tree. This kernel keeps that source tree production-safe without
//! introducing a local toolchain dependency on the external `mdbook` binary: it
//! validates the book manifest, summary chapter graph, required chapter files,
//! and local Markdown links before the site is allowed into the active docs
//! pipeline.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MdbookSourceFile {
    pub path: String,     // data_class: INTERNAL_ONLY
    pub contents: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MdbookSourceReport {
    pub source_files_checked: usize, // data_class: INTERNAL_ONLY
    pub chapters_checked: usize,     // data_class: INTERNAL_ONLY
    pub local_links_checked: usize,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MdbookSourceError {
    NoSourceFiles,
    InvalidPath { path: String, reason: String },
    MissingBookToml,
    InvalidBookToml { reason: String },
    MissingSummary { path: String },
    EmptySummary { path: String },
    InvalidSummary { path: String, reason: String },
    DuplicateChapter { path: String },
    MissingChapter { source: String, target: String },
    EmptyChapter { path: String },
    ChapterMissingHeading { path: String },
    UnlistedMarkdownSource { path: String },
    BrokenLocalLink { source: String, target: String },
}

pub fn validate_mdbook_source<I>(files: I) -> Result<MdbookSourceReport, MdbookSourceError>
where
    I: IntoIterator<Item = MdbookSourceFile>,
{
    let files = source_map(files)?;
    let Some(book_toml) = files.get("book.toml") else {
        return Err(MdbookSourceError::MissingBookToml);
    };
    let src_dir = parse_book_toml(book_toml)?;
    let summary_path = format!("{src_dir}/SUMMARY.md");
    let Some(summary) = files.get(&summary_path) else {
        return Err(MdbookSourceError::MissingSummary { path: summary_path });
    };
    if summary.trim().is_empty() {
        return Err(MdbookSourceError::EmptySummary { path: summary_path });
    }

    let chapters = parse_summary_chapters(&summary_path, summary, &src_dir)?;
    if chapters.is_empty() {
        return Err(MdbookSourceError::InvalidSummary {
            path: summary_path,
            reason: "SUMMARY.md must list at least one chapter".into(),
        });
    }

    let mut seen_chapters = BTreeSet::new();
    for chapter in &chapters {
        if !seen_chapters.insert(chapter.clone()) {
            return Err(MdbookSourceError::DuplicateChapter {
                path: chapter.clone(),
            });
        }
        let Some(contents) = files.get(chapter) else {
            return Err(MdbookSourceError::MissingChapter {
                source: summary_path.clone(),
                target: chapter.clone(),
            });
        };
        validate_chapter(chapter, contents)?;
    }

    for path in files.keys() {
        if path.starts_with(&format!("{src_dir}/"))
            && path.ends_with(".md")
            && path != &summary_path
            && !seen_chapters.contains(path)
        {
            return Err(MdbookSourceError::UnlistedMarkdownSource { path: path.clone() });
        }
    }

    let mut local_links_checked = 0usize;
    for (path, contents) in &files {
        if !path.ends_with(".md") {
            continue;
        }
        for target in markdown_link_targets(contents) {
            let Some(resolved) = resolve_local_link(path, &target)? else {
                continue;
            };
            local_links_checked += 1;
            if !files.contains_key(&resolved) {
                return Err(MdbookSourceError::BrokenLocalLink {
                    source: path.clone(),
                    target: resolved,
                });
            }
        }
    }

    Ok(MdbookSourceReport {
        source_files_checked: files.len(),
        chapters_checked: chapters.len(),
        local_links_checked,
    })
}

fn source_map<I>(files: I) -> Result<BTreeMap<String, String>, MdbookSourceError>
where
    I: IntoIterator<Item = MdbookSourceFile>,
{
    let mut map = BTreeMap::new();
    for file in files {
        validate_source_path(&file.path)?;
        map.insert(file.path, file.contents);
    }
    if map.is_empty() {
        return Err(MdbookSourceError::NoSourceFiles);
    }
    Ok(map)
}

fn validate_source_path(path: &str) -> Result<(), MdbookSourceError> {
    if path.trim().is_empty() {
        return Err(MdbookSourceError::InvalidPath {
            path: path.into(),
            reason: "path must be non-empty".into(),
        });
    }
    if path.starts_with('/') || path.contains('\\') || path.contains('\0') {
        return Err(MdbookSourceError::InvalidPath {
            path: path.into(),
            reason: "path must be a relative slash path".into(),
        });
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(MdbookSourceError::InvalidPath {
            path: path.into(),
            reason: "path must not contain empty, dot, or parent components".into(),
        });
    }
    Ok(())
}

fn parse_book_toml(contents: &str) -> Result<String, MdbookSourceError> {
    let mut in_book = false;
    let mut title_present = false;
    let mut src_dir = "src".to_string();

    for line in contents.lines() {
        let line = line
            .split_once('#')
            .map(|(value, _)| value)
            .unwrap_or(line)
            .trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_book = line == "[book]";
            continue;
        }
        if !in_book {
            continue;
        }
        if let Some(value) = parse_toml_string_value(line, "title") {
            if value.trim().is_empty() {
                return Err(MdbookSourceError::InvalidBookToml {
                    reason: "[book].title must be non-empty".into(),
                });
            }
            title_present = true;
        }
        if let Some(value) = parse_toml_string_value(line, "src") {
            validate_source_path(&value)?;
            src_dir = value;
        }
    }

    if !contents.lines().any(|line| line.trim() == "[book]") {
        return Err(MdbookSourceError::InvalidBookToml {
            reason: "book.toml must contain [book]".into(),
        });
    }
    if !title_present {
        return Err(MdbookSourceError::InvalidBookToml {
            reason: "book.toml must contain [book].title".into(),
        });
    }
    Ok(src_dir)
}

fn parse_toml_string_value(line: &str, key: &str) -> Option<String> {
    let (left, right) = line.split_once('=')?;
    if left.trim() != key {
        return None;
    }
    let right = right.trim();
    let quoted = right.strip_prefix('"')?.strip_suffix('"')?;
    Some(quoted.to_string())
}

fn parse_summary_chapters(
    summary_path: &str,
    summary: &str,
    src_dir: &str,
) -> Result<Vec<String>, MdbookSourceError> {
    markdown_link_targets(summary)
        .into_iter()
        .map(|target| {
            resolve_local_link(summary_path, &target)?.ok_or_else(|| {
                MdbookSourceError::InvalidSummary {
                    path: summary_path.into(),
                    reason: format!("summary chapter link must be local markdown: {target}"),
                }
            })
        })
        .filter(|result| match result {
            Ok(path) => path.starts_with(&format!("{src_dir}/")) && path.ends_with(".md"),
            Err(_) => true,
        })
        .collect()
}

fn validate_chapter(path: &str, contents: &str) -> Result<(), MdbookSourceError> {
    if contents.trim().is_empty() {
        return Err(MdbookSourceError::EmptyChapter { path: path.into() });
    }
    let first_visible = first_visible_chapter_line(contents);
    if !first_visible.is_some_and(|line| line.trim_start().starts_with("# ")) {
        return Err(MdbookSourceError::ChapterMissingHeading { path: path.into() });
    }
    Ok(())
}

fn first_visible_chapter_line(contents: &str) -> Option<&str> {
    let mut lines = contents.lines().peekable();
    while lines.peek().is_some_and(|line| line.trim().is_empty()) {
        lines.next();
    }
    if lines.peek().is_some_and(|line| line.trim() == "---") {
        lines.next();
        for line in lines.by_ref() {
            if line.trim() == "---" {
                break;
            }
        }
    }
    lines.find(|line| !line.trim().is_empty())
}

fn markdown_link_targets(contents: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut in_fence = false;
    for line in contents.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let line = strip_inline_code(line);
        let mut rest = line.as_str();
        while let Some(close_bracket) = rest.find("](") {
            let after_bracket = &rest[close_bracket + 2..];
            let Some(close_paren) = after_bracket.find(')') else {
                break;
            };
            let target = after_bracket[..close_paren]
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !target.is_empty() {
                targets.push(target);
            }
            rest = &after_bracket[close_paren + 1..];
        }
    }
    targets
}

fn strip_inline_code(line: &str) -> String {
    let mut output = String::new();
    let mut in_code = false;
    for character in line.chars() {
        if character == '`' {
            in_code = !in_code;
            output.push(' ');
        } else if in_code {
            output.push(' ');
        } else {
            output.push(character);
        }
    }
    output
}

fn resolve_local_link(source: &str, target: &str) -> Result<Option<String>, MdbookSourceError> {
    if is_ignored_link_target(target) {
        return Ok(None);
    }
    let path_part = target
        .split_once('#')
        .map(|(path, _)| path)
        .unwrap_or(target);
    if path_part.is_empty() {
        return Ok(None);
    }
    if path_part.starts_with('/') || path_part.contains('\\') || path_part.contains('\0') {
        return Err(MdbookSourceError::BrokenLocalLink {
            source: source.into(),
            target: path_part.into(),
        });
    }

    let mut parts = source
        .rsplit_once('/')
        .map(|(parent, _)| parent.split('/').collect::<Vec<_>>())
        .unwrap_or_default();
    for part in path_part.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(MdbookSourceError::BrokenLocalLink {
                        source: source.into(),
                        target: path_part.into(),
                    });
                }
            }
            part => parts.push(part),
        }
    }
    Ok(Some(parts.join("/")))
}

fn is_ignored_link_target(target: &str) -> bool {
    let lower = target.to_ascii_lowercase();
    target.starts_with('#')
        || lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_mdbook_source_with_summary_chapters_and_local_links() {
        let report = validate_mdbook_source([
            file(
                "book.toml",
                "[book]\ntitle = \"Oyatie Docs\"\nsrc = \"src\"\n",
            ),
            file(
                "src/SUMMARY.md",
                "# Summary\n\n- [Start](introduction.md)\n- [Guide](guides/admin.md)\n",
            ),
            file(
                "src/introduction.md",
                "# Start\n\nContinue to the [guide](guides/admin.md).\n",
            ),
            file(
                "src/guides/admin.md",
                "# Guide\n\nReturn [home](../introduction.md).\n",
            ),
        ])
        .expect("mdbook source validates");

        assert_eq!(report.chapters_checked, 2);
        assert_eq!(report.local_links_checked, 4);
    }

    #[test]
    fn rejects_missing_book_manifest_or_summary() {
        assert_eq!(
            validate_mdbook_source([file("src/SUMMARY.md", "# Summary\n")]),
            Err(MdbookSourceError::MissingBookToml)
        );
        assert_eq!(
            validate_mdbook_source([file("book.toml", "[book]\ntitle = \"Docs\"\n")]),
            Err(MdbookSourceError::MissingSummary {
                path: "src/SUMMARY.md".into(),
            })
        );
    }

    #[test]
    fn rejects_missing_chapter_and_broken_local_link() {
        assert_eq!(
            validate_mdbook_source([
                file("book.toml", "[book]\ntitle = \"Docs\"\n"),
                file("src/SUMMARY.md", "# Summary\n\n- [Missing](missing.md)\n"),
            ]),
            Err(MdbookSourceError::MissingChapter {
                source: "src/SUMMARY.md".into(),
                target: "src/missing.md".into(),
            })
        );
        assert_eq!(
            validate_mdbook_source([
                file("book.toml", "[book]\ntitle = \"Docs\"\n"),
                file("src/SUMMARY.md", "# Summary\n\n- [Start](start.md)\n"),
                file("src/start.md", "# Start\n\nBroken [link](missing.md).\n"),
            ]),
            Err(MdbookSourceError::BrokenLocalLink {
                source: "src/start.md".into(),
                target: "src/missing.md".into(),
            })
        );
    }

    #[test]
    fn rejects_unlisted_markdown_sources_and_chapter_without_heading() {
        assert_eq!(
            validate_mdbook_source([
                file("book.toml", "[book]\ntitle = \"Docs\"\n"),
                file("src/SUMMARY.md", "# Summary\n\n- [Start](start.md)\n"),
                file("src/start.md", "# Start\n"),
                file("src/orphan.md", "# Orphan\n"),
            ]),
            Err(MdbookSourceError::UnlistedMarkdownSource {
                path: "src/orphan.md".into(),
            })
        );
        assert_eq!(
            validate_mdbook_source([
                file("book.toml", "[book]\ntitle = \"Docs\"\n"),
                file("src/SUMMARY.md", "# Summary\n\n- [Start](start.md)\n"),
                file("src/start.md", "Body without heading.\n"),
            ]),
            Err(MdbookSourceError::ChapterMissingHeading {
                path: "src/start.md".into(),
            })
        );
    }

    #[test]
    fn accepts_chapter_heading_after_frontmatter() {
        assert_eq!(
            validate_mdbook_source([
                file("book.toml", "[book]\ntitle = \"Docs\"\n"),
                file("src/SUMMARY.md", "# Summary\n\n- [Start](start.md)\n"),
                file(
                    "src/start.md",
                    "---\ndoc_status: published\n---\n\n# Start\n",
                ),
            ]),
            Ok(MdbookSourceReport {
                source_files_checked: 3,
                chapters_checked: 1,
                local_links_checked: 1,
            })
        );
    }

    fn file(path: &str, contents: &str) -> MdbookSourceFile {
        MdbookSourceFile {
            path: path.into(),
            contents: contents.into(),
        }
    }
}
