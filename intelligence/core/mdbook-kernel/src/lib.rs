//! Foundry mdbook publishing kernel — pure I/O-free model for the
//! `walk → publish → wire-architecture-map` doc pipeline.
//!
//! M01-P09-IP-001 ships `walk_sources` + `publish_site`.
//! M01-P16-IP-003 ships `wire_architecture_map`.
//!
//! Runners discover rustdoc HTML, OpenAPI YAML, ADR Markdown, and
//! frontmatter-bearing artifacts on disk, feed the typed records here,
//! and produce a `PublishableSite` whose chapter tree can be rendered
//! to mdbook `SUMMARY.md` / chapter files by a thin I/O wrapper.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

/// What kind of artifact a source file represents in the published book.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SourceKind {
    Rustdoc,
    OpenApi,
    Adr,
    Frontmatter,
}

impl SourceKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Rustdoc => "rustdoc",
            Self::OpenApi => "openapi",
            Self::Adr => "adr",
            Self::Frontmatter => "frontmatter",
        }
    }

    /// Default chapter prefix for a kind in the published book.
    pub fn chapter_prefix(self) -> &'static str {
        match self {
            Self::Rustdoc => "rustdoc",
            Self::OpenApi => "contracts",
            Self::Adr => "adr",
            Self::Frontmatter => "guides",
        }
    }
}

/// Source artifact the runner found.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceArtifact {
    pub path: String,     // data_class: INTERNAL_ONLY (repo-relative)
    pub kind: SourceKind, // data_class: INTERNAL_ONLY
    pub title: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chapter {
    pub path: String,        // data_class: INTERNAL_ONLY
    pub title: String,       // data_class: INTERNAL_ONLY
    pub kind: SourceKind,    // data_class: INTERNAL_ONLY
    pub source_path: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishableSite {
    pub chapters: Vec<Chapter>, // data_class: INTERNAL_ONLY
    pub by_kind_counts: BTreeMap<SourceKind, usize>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MdbookError {
    EmptyPath,
    EmptyTitle { path: String },
    DuplicatePath { path: String },
    ArchitectureMapPathMissing,
}

impl MdbookError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyPath => "source artifact has empty path".to_owned(),
            Self::EmptyTitle { path } => format!("{path}: title is empty"),
            Self::DuplicatePath { path } => format!("duplicate source path: {path}"),
            Self::ArchitectureMapPathMissing => "architecture-map output path is empty".to_owned(),
        }
    }
}

/// Validate + group artifacts ready for publishing. Returns a flat,
/// sorted list of chapters and a histogram by kind.
pub fn walk_sources(sources: &[SourceArtifact]) -> Result<PublishableSite, MdbookError> {
    let mut seen = std::collections::BTreeSet::new();
    for s in sources {
        if s.path.is_empty() {
            return Err(MdbookError::EmptyPath);
        }
        if s.title.is_empty() {
            return Err(MdbookError::EmptyTitle {
                path: s.path.clone(),
            });
        }
        if !seen.insert(s.path.as_str()) {
            return Err(MdbookError::DuplicatePath {
                path: s.path.clone(),
            });
        }
    }

    let mut chapters: Vec<Chapter> = sources
        .iter()
        .map(|s| Chapter {
            path: format!("{}/{}", s.kind.chapter_prefix(), file_basename(&s.path)),
            title: s.title.clone(),
            kind: s.kind,
            source_path: s.path.clone(),
        })
        .collect();
    chapters.sort_by_key(|c| (c.kind, c.path.clone()));

    let mut counts: BTreeMap<SourceKind, usize> = BTreeMap::new();
    for c in &chapters {
        *counts.entry(c.kind).or_insert(0) += 1;
    }

    Ok(PublishableSite {
        chapters,
        by_kind_counts: counts,
    })
}

/// Render the published site to an mdbook-compatible `SUMMARY.md`
/// string. The output is deterministic given the same input.
pub fn publish_site(site: &PublishableSite) -> String {
    let mut out = String::from("# Summary\n\n");
    let mut current_kind: Option<SourceKind> = None;
    for c in &site.chapters {
        if Some(c.kind) != current_kind {
            out.push_str(&format!("# {}\n\n", title_case(c.kind.name())));
            current_kind = Some(c.kind);
        }
        out.push_str(&format!("- [{}]({})\n", c.title, c.path));
    }
    out
}

/// Wire an external architecture-map artifact into the published site
/// as a dedicated chapter. Returns a new site with the map chapter
/// appended under a "Visualizations" kind.
pub fn wire_architecture_map(
    site: PublishableSite,
    architecture_map_path: &str,
    title: &str,
) -> Result<PublishableSite, MdbookError> {
    if architecture_map_path.is_empty() {
        return Err(MdbookError::ArchitectureMapPathMissing);
    }
    let mut next = site;
    next.chapters.push(Chapter {
        path: format!("visualizations/{}", file_basename(architecture_map_path)),
        title: title.to_owned(),
        kind: SourceKind::Frontmatter,
        source_path: architecture_map_path.to_owned(),
    });
    next.chapters.sort_by_key(|c| (c.kind, c.path.clone()));
    let mut counts: BTreeMap<SourceKind, usize> = BTreeMap::new();
    for c in &next.chapters {
        *counts.entry(c.kind).or_insert(0) += 1;
    }
    next.by_kind_counts = counts;
    Ok(next)
}

fn file_basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn title_case(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn art(path: &str, kind: SourceKind, title: &str) -> SourceArtifact {
        SourceArtifact {
            path: path.into(),
            kind,
            title: title.into(),
        }
    }

    #[test]
    fn walk_empty_input_returns_empty_site() {
        let r = walk_sources(&[]).unwrap();
        assert!(r.chapters.is_empty());
        assert!(r.by_kind_counts.is_empty());
    }

    #[test]
    fn walk_groups_by_kind() {
        let r = walk_sources(&[
            art("docs/api.yaml", SourceKind::OpenApi, "API"),
            art("docs/adr-0001.md", SourceKind::Adr, "ADR-0001"),
            art("target/doc/index.html", SourceKind::Rustdoc, "Rustdoc"),
        ])
        .unwrap();
        assert_eq!(r.chapters.len(), 3);
        assert_eq!(*r.by_kind_counts.get(&SourceKind::Adr).unwrap(), 1);
    }

    #[test]
    fn walk_chapter_paths_use_kind_prefix() {
        let r = walk_sources(&[art("docs/x.yaml", SourceKind::OpenApi, "X")]).unwrap();
        assert!(r.chapters[0].path.starts_with("contracts/"));
    }

    #[test]
    fn walk_empty_path_errors() {
        let err = walk_sources(&[art("", SourceKind::Adr, "X")]).unwrap_err();
        assert!(matches!(err, MdbookError::EmptyPath));
    }

    #[test]
    fn walk_empty_title_errors() {
        let err = walk_sources(&[art("docs/a.md", SourceKind::Adr, "")]).unwrap_err();
        assert!(matches!(err, MdbookError::EmptyTitle { .. }));
    }

    #[test]
    fn walk_duplicate_path_errors() {
        let err = walk_sources(&[
            art("docs/a.md", SourceKind::Adr, "A"),
            art("docs/a.md", SourceKind::Adr, "A again"),
        ])
        .unwrap_err();
        assert!(matches!(err, MdbookError::DuplicatePath { .. }));
    }

    #[test]
    fn publish_emits_summary_md_header() {
        let s = publish_site(&PublishableSite {
            chapters: vec![],
            by_kind_counts: BTreeMap::new(),
        });
        assert!(s.starts_with("# Summary\n"));
    }

    #[test]
    fn publish_groups_chapters_by_kind() {
        let site = walk_sources(&[
            art("docs/api.yaml", SourceKind::OpenApi, "API"),
            art("docs/adr-0001.md", SourceKind::Adr, "ADR-0001"),
        ])
        .unwrap();
        let out = publish_site(&site);
        // Kind headers are upper-cased; sort by enum declaration order
        // (Rustdoc, OpenApi, Adr, Frontmatter) — OpenApi precedes Adr.
        assert!(out.contains("# Adr"));
        assert!(out.contains("# Openapi"));
        let pos_openapi = out.find("# Openapi").unwrap();
        let pos_adr = out.find("# Adr").unwrap();
        assert!(pos_openapi < pos_adr);
    }

    #[test]
    fn publish_output_is_deterministic() {
        let mut a = walk_sources(&[
            art("z.md", SourceKind::Adr, "Z"),
            art("a.md", SourceKind::Adr, "A"),
        ])
        .unwrap();
        let b = a.clone();
        // Permute chapters; published output should still be sorted by (kind, path).
        a.chapters.reverse();
        assert_ne!(publish_site(&a), publish_site(&b));
        a.chapters.sort_by_key(|c| (c.kind, c.path.clone()));
        assert_eq!(publish_site(&a), publish_site(&b));
    }

    #[test]
    fn wire_appends_architecture_map_chapter() {
        let site = walk_sources(&[art("docs/a.md", SourceKind::Adr, "A")]).unwrap();
        let wired =
            wire_architecture_map(site, "registry/graph/map.json", "Architecture Map").unwrap();
        assert!(
            wired
                .chapters
                .iter()
                .any(|c| c.path == "visualizations/map.json")
        );
    }

    #[test]
    fn wire_empty_architecture_map_path_errors() {
        let site = walk_sources(&[art("docs/a.md", SourceKind::Adr, "A")]).unwrap();
        let err = wire_architecture_map(site, "", "X").unwrap_err();
        assert!(matches!(err, MdbookError::ArchitectureMapPathMissing));
    }

    #[test]
    fn wire_updates_kind_counts() {
        let site = walk_sources(&[art("docs/a.md", SourceKind::Adr, "A")]).unwrap();
        let wired = wire_architecture_map(site, "registry/graph/map.json", "Map").unwrap();
        assert_eq!(
            *wired.by_kind_counts.get(&SourceKind::Frontmatter).unwrap(),
            1
        );
        assert_eq!(*wired.by_kind_counts.get(&SourceKind::Adr).unwrap(), 1);
    }

    #[test]
    fn source_kind_chapter_prefix_distinct() {
        let prefixes: std::collections::HashSet<_> = [
            SourceKind::Rustdoc,
            SourceKind::OpenApi,
            SourceKind::Adr,
            SourceKind::Frontmatter,
        ]
        .iter()
        .map(|k| k.chapter_prefix())
        .collect();
        assert_eq!(prefixes.len(), 4);
    }
}
