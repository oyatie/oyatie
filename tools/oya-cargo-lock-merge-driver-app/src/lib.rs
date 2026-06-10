#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeErrorKind {
    Parse,
    Conflict,
    Io,
    Usage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeError {
    kind: MergeErrorKind,
    message: String,
}

impl MergeError {
    pub fn new(kind: MergeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> MergeErrorKind {
        self.kind
    }
}

impl Display for MergeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for MergeError {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PackageStem {
    name: String,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PackageKey {
    name: String,
    version: String,
    source: String,
}

impl PackageKey {
    fn stem(&self) -> PackageStem {
        PackageStem {
            name: self.name.clone(),
            source: self.source.clone(),
        }
    }

    fn label(&self) -> String {
        if self.source.is_empty() {
            format!("{} {}", self.name, self.version)
        } else {
            format!("{} {} ({})", self.name, self.version, self.source)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackageBlock {
    key: PackageKey,
    stem: PackageStem,
    raw: String,
}

#[derive(Debug, Clone)]
struct ParsedLockfile {
    preamble: String,
    packages: Vec<PackageBlock>,
    by_key: BTreeMap<PackageKey, PackageBlock>,
}

#[derive(Debug, Clone)]
enum SideStatus {
    Unchanged,
    Modified(PackageBlock),
    Removed,
    Replaced(Vec<PackageBlock>),
}

/// Merge three Cargo.lock snapshots using package-level structural semantics.
pub fn merge_lockfiles(base: &str, ours: &str, theirs: &str) -> Result<String, MergeError> {
    let base_doc = ParsedLockfile::parse("base", base)?;
    let ours_doc = ParsedLockfile::parse("ours", ours)?;
    let theirs_doc = ParsedLockfile::parse("theirs", theirs)?;

    let preamble = merge_preamble(&base_doc, &ours_doc, &theirs_doc)?;
    let mut merged: BTreeMap<PackageKey, PackageBlock> = BTreeMap::new();
    let mut replacement_keys: BTreeSet<PackageKey> = BTreeSet::new();

    for base_pkg in &base_doc.packages {
        let ours_status = side_status(base_pkg, &base_doc, &ours_doc);
        let theirs_status = side_status(base_pkg, &base_doc, &theirs_doc);
        let merged_blocks = merge_existing_package(base_pkg, ours_status, theirs_status)?;
        for block in merged_blocks {
            replacement_keys.insert(block.key.clone());
            insert_merged(&mut merged, block)?;
        }
    }

    merge_additions(
        &base_doc,
        &ours_doc,
        &theirs_doc,
        &replacement_keys,
        &mut merged,
    )?;

    Ok(render_merged(
        &preamble,
        &base_doc,
        &ours_doc,
        &theirs_doc,
        &merged,
    ))
}

impl ParsedLockfile {
    fn parse(label: &'static str, input: &str) -> Result<Self, MergeError> {
        let (preamble, blocks) = split_package_blocks(input);
        let mut packages = Vec::new();
        let mut by_key = BTreeMap::new();

        for raw in blocks {
            let block = parse_package_block(label, raw)?;
            if by_key.contains_key(&block.key) {
                return Err(MergeError::new(
                    MergeErrorKind::Parse,
                    format!("{}: duplicate package key {}", label, block.key.label()),
                ));
            }
            by_key.insert(block.key.clone(), block.clone());
            packages.push(block);
        }

        Ok(Self {
            preamble,
            packages,
            by_key,
        })
    }

    fn non_base_packages_with_stem(
        &self,
        base: &ParsedLockfile,
        stem: &PackageStem,
    ) -> Vec<PackageBlock> {
        self.packages
            .iter()
            .filter(|package| package.stem == *stem && !base.by_key.contains_key(&package.key))
            .cloned()
            .collect()
    }
}

fn split_package_blocks(input: &str) -> (String, Vec<String>) {
    let mut starts = Vec::new();
    for (index, _) in input.match_indices("[[package]]") {
        if index == 0 || input.as_bytes().get(index.saturating_sub(1)) == Some(&b'\n') {
            starts.push(index);
        }
    }

    let Some(first) = starts.first().copied() else {
        return (input.to_owned(), Vec::new());
    };

    let mut blocks = Vec::new();
    for (position, start) in starts.iter().enumerate() {
        let end = match starts.get(position + 1).copied() {
            Some(next) => next,
            None => input.len(),
        };
        blocks.push(input[*start..end].to_owned());
    }

    (input[..first].to_owned(), blocks)
}

fn parse_package_block(label: &'static str, raw: String) -> Result<PackageBlock, MergeError> {
    let doc: toml_edit::DocumentMut = raw.parse().map_err(|err| {
        MergeError::new(
            MergeErrorKind::Parse,
            format!("{label}: package block is not valid TOML: {err}"),
        )
    })?;

    let packages = doc
        .get("package")
        .and_then(|package| package.as_array_of_tables());
    let Some(packages) = packages else {
        return Err(MergeError::new(
            MergeErrorKind::Parse,
            format!("{label}: package block is missing [[package]]"),
        ));
    };
    if packages.len() != 1 {
        return Err(MergeError::new(
            MergeErrorKind::Parse,
            format!("{label}: package block must contain exactly one [[package]]"),
        ));
    }
    let Some(table) = packages.iter().next() else {
        return Err(MergeError::new(
            MergeErrorKind::Parse,
            format!("{label}: package table is empty"),
        ));
    };

    let name = string_field(label, table, "name")?;
    let version = string_field(label, table, "version")?;
    let source = table
        .get("source")
        .and_then(|item| item.as_str())
        .map_or_else(String::new, ToOwned::to_owned);
    let key = PackageKey {
        name,
        version,
        source,
    };
    let stem = key.stem();
    Ok(PackageBlock { key, stem, raw })
}

fn string_field(
    label: &'static str,
    table: &toml_edit::Table,
    field: &str,
) -> Result<String, MergeError> {
    table
        .get(field)
        .and_then(|item| item.as_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            MergeError::new(
                MergeErrorKind::Parse,
                format!("{label}: package.{field} must be a string"),
            )
        })
}

fn merge_preamble(
    base: &ParsedLockfile,
    ours: &ParsedLockfile,
    theirs: &ParsedLockfile,
) -> Result<String, MergeError> {
    if ours.preamble == theirs.preamble {
        return Ok(ours.preamble.clone());
    }
    if ours.preamble == base.preamble {
        return Ok(theirs.preamble.clone());
    }
    if theirs.preamble == base.preamble {
        return Ok(ours.preamble.clone());
    }
    Err(MergeError::new(
        MergeErrorKind::Conflict,
        "header-conflict: Cargo.lock preamble changed on both sides",
    ))
}

fn side_status(
    base_pkg: &PackageBlock,
    base: &ParsedLockfile,
    side: &ParsedLockfile,
) -> SideStatus {
    if let Some(package) = side.by_key.get(&base_pkg.key) {
        if package.raw == base_pkg.raw {
            SideStatus::Unchanged
        } else {
            SideStatus::Modified(package.clone())
        }
    } else {
        let replacements = side.non_base_packages_with_stem(base, &base_pkg.stem);
        if replacements.is_empty() {
            SideStatus::Removed
        } else {
            SideStatus::Replaced(replacements)
        }
    }
}

fn merge_existing_package(
    base_pkg: &PackageBlock,
    ours: SideStatus,
    theirs: SideStatus,
) -> Result<Vec<PackageBlock>, MergeError> {
    match (ours, theirs) {
        (SideStatus::Unchanged, SideStatus::Unchanged) => Ok(vec![base_pkg.clone()]),
        (SideStatus::Unchanged, SideStatus::Modified(block))
        | (SideStatus::Modified(block), SideStatus::Unchanged) => Ok(vec![block]),
        (SideStatus::Unchanged, SideStatus::Removed)
        | (SideStatus::Removed, SideStatus::Unchanged)
        | (SideStatus::Removed, SideStatus::Removed) => Ok(Vec::new()),
        (SideStatus::Modified(ours_block), SideStatus::Modified(theirs_block)) => {
            if ours_block.raw == theirs_block.raw {
                Ok(vec![ours_block])
            } else {
                conflict(format!(
                    "edit-conflict: package {} changed differently on both sides",
                    base_pkg.key.label()
                ))
            }
        }
        (SideStatus::Modified(_), SideStatus::Removed)
        | (SideStatus::Removed, SideStatus::Modified(_))
        | (SideStatus::Removed, SideStatus::Replaced(_))
        | (SideStatus::Replaced(_), SideStatus::Removed) => conflict(format!(
            "removal-vs-edit: package {} was removed on one side and edited on the other",
            base_pkg.key.label()
        )),
        (SideStatus::Unchanged, SideStatus::Replaced(blocks))
        | (SideStatus::Replaced(blocks), SideStatus::Unchanged) => Ok(blocks),
        (SideStatus::Replaced(ours_blocks), SideStatus::Replaced(theirs_blocks)) => {
            if package_sets_equal(&ours_blocks, &theirs_blocks) {
                Ok(ours_blocks)
            } else {
                conflict(format!(
                    "version-divergence: package {} resolved to different versions",
                    base_pkg.key.label()
                ))
            }
        }
        (SideStatus::Modified(_), SideStatus::Replaced(_))
        | (SideStatus::Replaced(_), SideStatus::Modified(_)) => conflict(format!(
            "version-divergence: package {} was edited and version-replaced",
            base_pkg.key.label()
        )),
    }
}

fn merge_additions(
    base: &ParsedLockfile,
    ours: &ParsedLockfile,
    theirs: &ParsedLockfile,
    replacement_keys: &BTreeSet<PackageKey>,
    merged: &mut BTreeMap<PackageKey, PackageBlock>,
) -> Result<(), MergeError> {
    let ours_additions = additions_by_stem(base, ours, replacement_keys);
    let theirs_additions = additions_by_stem(base, theirs, replacement_keys);
    let stems: BTreeSet<PackageStem> = ours_additions
        .keys()
        .chain(theirs_additions.keys())
        .cloned()
        .collect();

    for stem in stems {
        let ours_blocks = match ours_additions.get(&stem) {
            Some(blocks) => blocks.clone(),
            None => Vec::new(),
        };
        let theirs_blocks = match theirs_additions.get(&stem) {
            Some(blocks) => blocks.clone(),
            None => Vec::new(),
        };
        let selected = if ours_blocks.is_empty() {
            theirs_blocks
        } else if theirs_blocks.is_empty() || package_sets_equal(&ours_blocks, &theirs_blocks) {
            ours_blocks
        } else {
            return conflict(format!(
                "version-divergence: package {} was added with different versions",
                stem.name
            ));
        };
        for block in selected {
            insert_merged(merged, block)?;
        }
    }
    Ok(())
}

fn additions_by_stem(
    base: &ParsedLockfile,
    side: &ParsedLockfile,
    replacement_keys: &BTreeSet<PackageKey>,
) -> BTreeMap<PackageStem, Vec<PackageBlock>> {
    let mut additions: BTreeMap<PackageStem, Vec<PackageBlock>> = BTreeMap::new();
    for package in &side.packages {
        if !base.by_key.contains_key(&package.key) && !replacement_keys.contains(&package.key) {
            additions
                .entry(package.stem.clone())
                .or_default()
                .push(package.clone());
        }
    }
    additions
}

fn package_sets_equal(left: &[PackageBlock], right: &[PackageBlock]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let left_map = package_map(left);
    let right_map = package_map(right);
    left_map == right_map
}

fn package_map(blocks: &[PackageBlock]) -> BTreeMap<PackageKey, String> {
    let mut map = BTreeMap::new();
    for block in blocks {
        map.insert(block.key.clone(), block.raw.clone());
    }
    map
}

fn insert_merged(
    merged: &mut BTreeMap<PackageKey, PackageBlock>,
    block: PackageBlock,
) -> Result<(), MergeError> {
    if let Some(existing) = merged.get(&block.key) {
        if existing.raw != block.raw {
            return conflict(format!(
                "edit-conflict: package {} has two merged bodies",
                block.key.label()
            ));
        }
        return Ok(());
    }
    merged.insert(block.key.clone(), block);
    Ok(())
}

fn render_merged(
    preamble: &str,
    base: &ParsedLockfile,
    ours: &ParsedLockfile,
    theirs: &ParsedLockfile,
    merged: &BTreeMap<PackageKey, PackageBlock>,
) -> String {
    let mut output = preamble.to_owned();
    let mut emitted = BTreeSet::new();
    append_in_order(&mut output, &mut emitted, &base.packages, merged);
    append_in_order(&mut output, &mut emitted, &ours.packages, merged);
    append_in_order(&mut output, &mut emitted, &theirs.packages, merged);
    for (key, block) in merged {
        if !emitted.contains(key) {
            push_block(&mut output, &block.raw);
            emitted.insert(key.clone());
        }
    }
    output
}

fn append_in_order(
    output: &mut String,
    emitted: &mut BTreeSet<PackageKey>,
    order: &[PackageBlock],
    merged: &BTreeMap<PackageKey, PackageBlock>,
) {
    for package in order {
        if emitted.contains(&package.key) {
            continue;
        }
        if let Some(block) = merged.get(&package.key) {
            push_block(output, &block.raw);
            emitted.insert(package.key.clone());
        }
    }
}

fn push_block(output: &mut String, raw: &str) {
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(raw);
}

fn conflict<T>(message: String) -> Result<T, MergeError> {
    Err(MergeError::new(MergeErrorKind::Conflict, message))
}
