#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// The two ways a lockfile merge can fail. Neither is an exit code: the binary writes every side
/// under conflict markers for both, so `%A` is complete whichever one it was (see `main.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeErrorKind {
    Parse,
    Conflict,
}

const OURS_MARKER: &str = "<<<<<<< ours";
const BASE_MARKER: &str = "||||||| base";
const SPLIT_MARKER: &str = "=======";
const THEIRS_MARKER: &str = ">>>>>>> theirs";

/// Every side of a failed merge, carried verbatim under diff3 markers.
///
/// Git does NOT re-run its own text merge when a merge driver exits nonzero — it takes whatever
/// the driver left in `%A` as the conflicted working tree. A driver that exits nonzero without
/// writing therefore leaves `ours` standing alone, unmarked, with `theirs` simply absent: the file
/// reads as clean and complete, so a reflexive `git add` commits the loss silently. Verified with a
/// real `git merge`. Every nonzero exit of this driver writes this document instead, so no side can
/// disappear and the result cannot parse as TOML until a human has resolved it.
pub fn whole_file_conflict(base: &str, ours: &str, theirs: &str) -> String {
    let trim = |text: &str| text.trim_end_matches('\n').to_owned();
    format!(
        "{OURS_MARKER}\n{}\n{BASE_MARKER}\n{}\n{SPLIT_MARKER}\n{}\n{THEIRS_MARKER}\n",
        trim(ours),
        trim(base),
        trim(theirs)
    )
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
    lockfile_version: i64,
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
        let ours_status = side_status(base_pkg, &base_doc, &ours_doc)?;
        let theirs_status = side_status(base_pkg, &base_doc, &theirs_doc)?;
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
        let lockfile_version = parse_lockfile_version(label, &preamble)?;
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
            lockfile_version,
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

fn parse_lockfile_version(label: &'static str, preamble: &str) -> Result<i64, MergeError> {
    let doc: toml_edit::DocumentMut = preamble.parse().map_err(|err| {
        MergeError::new(
            MergeErrorKind::Parse,
            format!("{label}: Cargo.lock preamble is not valid TOML: {err}"),
        )
    })?;
    doc.get("version")
        .and_then(|item| item.as_integer())
        .ok_or_else(|| {
            MergeError::new(
                MergeErrorKind::Parse,
                format!("{label}: Cargo.lock preamble must contain integer version"),
            )
        })
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
    if base.lockfile_version != ours.lockfile_version
        || base.lockfile_version != theirs.lockfile_version
    {
        return Err(MergeError::new(
            MergeErrorKind::Conflict,
            format!(
                "lockfile-version: cannot merge Cargo.lock format versions base={} ours={} theirs={}",
                base.lockfile_version, ours.lockfile_version, theirs.lockfile_version
            ),
        ));
    }
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
) -> Result<SideStatus, MergeError> {
    if let Some(package) = side.by_key.get(&base_pkg.key) {
        if package.raw == base_pkg.raw {
            Ok(SideStatus::Unchanged)
        } else {
            Ok(SideStatus::Modified(package.clone()))
        }
    } else {
        let replacements = side.non_base_packages_with_stem(base, &base_pkg.stem);
        if replacements.is_empty() {
            Ok(SideStatus::Removed)
        } else {
            let removed_count = base
                .packages
                .iter()
                .filter(|package| {
                    package.stem == base_pkg.stem && !side.by_key.contains_key(&package.key)
                })
                .count();
            if removed_count == 1 && replacements.len() == 1 {
                Ok(SideStatus::Replaced(replacements))
            } else {
                conflict(format!(
                    "ambiguous-stem-replacement: package {} has {} removed base versions and {} replacement candidates",
                    base_pkg.key.label(),
                    removed_count,
                    replacements.len()
                ))
            }
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
                merge_modified_package(base_pkg, ours_block, theirs_block).map(|block| vec![block])
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

#[derive(Debug, Clone)]
struct PackageShape {
    non_dependency_doc: String,
    dependencies: Vec<String>,
    has_dependencies: bool,
}

fn merge_modified_package(
    base_pkg: &PackageBlock,
    ours_block: PackageBlock,
    theirs_block: PackageBlock,
) -> Result<PackageBlock, MergeError> {
    let base_shape = package_shape("base", &base_pkg.raw)?;
    let ours_shape = package_shape("ours", &ours_block.raw)?;
    let theirs_shape = package_shape("theirs", &theirs_block.raw)?;

    let selected_block = if ours_shape.non_dependency_doc == theirs_shape.non_dependency_doc {
        ours_block
    } else if ours_shape.non_dependency_doc == base_shape.non_dependency_doc {
        theirs_block
    } else if theirs_shape.non_dependency_doc == base_shape.non_dependency_doc {
        ours_block
    } else {
        return conflict(format!(
            "edit-conflict: package {} changed incompatible non-dependency fields on both sides",
            base_pkg.key.label()
        ));
    };

    let selected_shape = package_shape("selected", &selected_block.raw)?;
    let merged_dependencies = merge_dependency_lists(
        &base_shape.dependencies,
        &ours_shape.dependencies,
        &theirs_shape.dependencies,
    );
    let should_write_dependencies =
        base_shape.has_dependencies || ours_shape.has_dependencies || theirs_shape.has_dependencies;

    if selected_shape.dependencies == merged_dependencies
        && selected_shape.has_dependencies == should_write_dependencies
    {
        return Ok(selected_block);
    }

    let raw = rewrite_dependencies(
        &selected_block.raw,
        &merged_dependencies,
        should_write_dependencies,
    )?;
    Ok(PackageBlock {
        key: selected_block.key,
        stem: selected_block.stem,
        raw,
    })
}

fn package_shape(label: &'static str, raw: &str) -> Result<PackageShape, MergeError> {
    let mut doc: toml_edit::DocumentMut = raw.parse().map_err(|err| {
        MergeError::new(
            MergeErrorKind::Parse,
            format!("{label}: package block is not valid TOML: {err}"),
        )
    })?;
    let table = single_package_table(label, &doc)?;
    let dependencies = dependency_strings(label, table)?;
    let has_dependencies = table.contains_key("dependencies");
    remove_dependencies_field(label, &mut doc)?;
    Ok(PackageShape {
        non_dependency_doc: doc.to_string(),
        dependencies,
        has_dependencies,
    })
}

fn remove_dependencies_field(
    label: &'static str,
    doc: &mut toml_edit::DocumentMut,
) -> Result<(), MergeError> {
    let packages = doc
        .get_mut("package")
        .and_then(|package| package.as_array_of_tables_mut());
    let Some(packages) = packages else {
        return Err(MergeError::new(
            MergeErrorKind::Parse,
            format!("{label}: package block is missing [[package]]"),
        ));
    };
    let Some(table) = packages.iter_mut().next() else {
        return Err(MergeError::new(
            MergeErrorKind::Parse,
            format!("{label}: package table is empty"),
        ));
    };
    table.remove("dependencies");
    Ok(())
}

fn single_package_table<'a>(
    label: &'static str,
    doc: &'a toml_edit::DocumentMut,
) -> Result<&'a toml_edit::Table, MergeError> {
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
    match packages.iter().next() {
        Some(table) => Ok(table),
        None => Err(MergeError::new(
            MergeErrorKind::Parse,
            format!("{label}: package table is empty"),
        )),
    }
}

fn dependency_strings(
    label: &'static str,
    table: &toml_edit::Table,
) -> Result<Vec<String>, MergeError> {
    let Some(item) = table.get("dependencies") else {
        return Ok(Vec::new());
    };
    let Some(array) = item.as_array() else {
        return Err(MergeError::new(
            MergeErrorKind::Parse,
            format!("{label}: package.dependencies must be an array"),
        ));
    };
    let mut dependencies = Vec::new();
    for item in array.iter() {
        let Some(dependency) = item.as_str() else {
            return Err(MergeError::new(
                MergeErrorKind::Parse,
                format!("{label}: package.dependencies entries must be strings"),
            ));
        };
        dependencies.push(dependency.to_owned());
    }
    Ok(dependencies)
}

fn merge_dependency_lists(base: &[String], ours: &[String], theirs: &[String]) -> Vec<String> {
    let removed: BTreeSet<&str> = base
        .iter()
        .filter(|dependency| !ours.contains(dependency) || !theirs.contains(dependency))
        .map(String::as_str)
        .collect();
    let mut merged = ours
        .iter()
        .chain(theirs)
        .filter(|dependency| !removed.contains(dependency.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    merged.sort();
    merged.dedup();
    merged
}

fn rewrite_dependencies(
    raw: &str,
    dependencies: &[String],
    include_field: bool,
) -> Result<String, MergeError> {
    let lines = raw.split_inclusive('\n').collect::<Vec<_>>();
    let start = lines
        .iter()
        .position(|line| line.trim_start().starts_with("dependencies = ["));
    if !include_field {
        if let Some(start) = start {
            let end = dependency_array_end(&lines, start)?;
            return Ok(remove_line_range(&lines, start, end));
        }
        return Ok(raw.to_owned());
    }

    let formatted = format_dependencies(dependencies);
    if let Some(start) = start {
        let end = dependency_array_end(&lines, start)?;
        return Ok(replace_line_range(&lines, start, end, &formatted));
    }

    let mut output = raw.to_owned();
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(&formatted);
    Ok(output)
}

fn dependency_array_end(lines: &[&str], start: usize) -> Result<usize, MergeError> {
    for (offset, line) in lines[start..].iter().enumerate() {
        if line.trim() == "]" {
            return Ok(start + offset);
        }
    }
    Err(MergeError::new(
        MergeErrorKind::Parse,
        "package.dependencies array is missing closing bracket",
    ))
}

fn replace_line_range(lines: &[&str], start: usize, end: usize, replacement: &str) -> String {
    let mut output = String::new();
    for line in &lines[..start] {
        output.push_str(line);
    }
    output.push_str(replacement);
    for line in &lines[(end + 1)..] {
        output.push_str(line);
    }
    output
}

fn remove_line_range(lines: &[&str], start: usize, end: usize) -> String {
    replace_line_range(lines, start, end, "")
}

fn format_dependencies(dependencies: &[String]) -> String {
    let mut output = String::from("dependencies = [\n");
    for dependency in dependencies {
        output.push(' ');
        output.push_str(&toml_basic_string(dependency));
        output.push_str(",\n");
    }
    output.push_str("]\n");
    output
}

fn toml_basic_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            _ => output.push(character),
        }
    }
    output.push('"');
    output
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
    if !output.is_empty() && !output.ends_with("\n\n") {
        if output.ends_with('\n') {
            output.push('\n');
        } else {
            output.push_str("\n\n");
        }
    }
    output.push_str(raw);
}

fn conflict<T>(message: String) -> Result<T, MergeError> {
    Err(MergeError::new(MergeErrorKind::Conflict, message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_file_conflict_carries_every_side_under_diff3_markers() {
        let document = whole_file_conflict("BASE\n", "OURS\n", "THEIRS\n");
        assert_eq!(
            document,
            "<<<<<<< ours\nOURS\n||||||| base\nBASE\n=======\nTHEIRS\n>>>>>>> theirs\n"
        );
    }

    /// The invariant the data-loss defect violated: whatever the driver leaves in `%A` must still
    /// contain `theirs`, and must not be mistakable for a resolved file.
    #[test]
    fn whole_file_conflict_never_reads_as_a_clean_lockfile() {
        let theirs = "[[package]]\nname = \"theirs-only\"\nversion = \"0.2.0\"\n";
        let document = whole_file_conflict("", "[[package]]\nname = \"alpha\"\n", theirs);
        assert!(document.contains("theirs-only"));
        assert!(document.contains(OURS_MARKER));
        assert!(document.contains(THEIRS_MARKER));
        assert!(
            document.parse::<toml_edit::DocumentMut>().is_err(),
            "a conflicted %A must not parse as TOML"
        );
    }
}
