use std::collections::{BTreeMap, BTreeSet};
use std::str;

use pipeline_repository::{
    ContentId, Entry, EntryKind, EntryState, ManifestMeter, ObjectAlgorithm, ObjectId,
    RepositoryPath, RevisionId, SnapshotFailure, SnapshotLimits,
};

use crate::object::verify_blob_identity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResolvedObjectKind {
    Commit,
    Tree,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedObject {
    pub(crate) id: ObjectId,
    pub(crate) kind: ResolvedObjectKind,
}

pub(crate) fn parse_merge_base(
    output: &[u8],
    algorithm: ObjectAlgorithm,
) -> Result<RevisionId, SnapshotFailure> {
    if output.is_empty() {
        return Err(SnapshotFailure::MissingMergeBase);
    }
    if output.last() != Some(&b'\n') {
        return Err(SnapshotFailure::MalformedOutput(
            "merge-base output is not newline terminated".to_owned(),
        ));
    }
    let lines: Vec<&[u8]> = output[..output.len() - 1]
        .split(|byte| *byte == b'\n')
        .collect();
    if lines.len() != 1 {
        return Err(SnapshotFailure::AmbiguousMergeBase { count: lines.len() });
    }
    let id = parse_object_id(lines[0], algorithm, "merge-base identity")?;
    Ok(RevisionId::from_object_id(id))
}

pub(crate) fn parse_resolved_objects(
    output: &[u8],
    algorithm: ObjectAlgorithm,
    expected: usize,
) -> Result<Vec<ResolvedObject>, SnapshotFailure> {
    if output.last() != Some(&b'\n') {
        return Err(SnapshotFailure::MalformedOutput(
            "object-resolution output is not newline terminated".to_owned(),
        ));
    }
    let lines: Vec<&[u8]> = output[..output.len() - 1]
        .split(|byte| *byte == b'\n')
        .collect();
    if lines.len() != expected {
        return Err(SnapshotFailure::MalformedOutput(format!(
            "object resolution returned {} records, expected {expected}",
            lines.len()
        )));
    }
    lines
        .into_iter()
        .map(|line| {
            let (id, kind) = split_once(line, b' ').ok_or_else(|| {
                SnapshotFailure::MalformedOutput("object-resolution record has no type".to_owned())
            })?;
            if kind.contains(&b' ') || kind.is_empty() {
                return Err(SnapshotFailure::MalformedOutput(
                    "object-resolution record has extra fields".to_owned(),
                ));
            }
            let kind = match kind {
                b"commit" => ResolvedObjectKind::Commit,
                b"tree" => ResolvedObjectKind::Tree,
                other => {
                    return Err(SnapshotFailure::ObjectMismatch(format!(
                        "resolved object has unexpected type {:?}",
                        String::from_utf8_lossy(other)
                    )));
                }
            };
            Ok(ResolvedObject {
                id: parse_object_id(id, algorithm, "resolved object identity")?,
                kind,
            })
        })
        .collect()
}

pub(crate) fn parse_ls_tree(
    output: &[u8],
    algorithm: ObjectAlgorithm,
    limits: SnapshotLimits,
) -> Result<Vec<Entry>, SnapshotFailure> {
    if output.is_empty() {
        return Ok(Vec::new());
    }
    if output.last() != Some(&0) {
        return Err(SnapshotFailure::MalformedOutput(
            "ls-tree output is not NUL terminated".to_owned(),
        ));
    }
    let mut entries = Vec::new();
    let mut meter = ManifestMeter::default();
    for record in output[..output.len() - 1].split(|byte| *byte == 0) {
        let observed = u64::try_from(entries.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        enforce_limit("entry count", limits.max_entries(), observed)?;
        entries.push(parse_tree_entry(record, algorithm, limits, &mut meter)?);
    }
    Ok(entries)
}

fn parse_tree_entry(
    record: &[u8],
    algorithm: ObjectAlgorithm,
    limits: SnapshotLimits,
    meter: &mut ManifestMeter,
) -> Result<Entry, SnapshotFailure> {
    let (header, path) = split_once(record, b'\t').ok_or_else(|| {
        SnapshotFailure::MalformedOutput("ls-tree record has no path separator".to_owned())
    })?;
    let mut fields = header.split(|byte| *byte == b' ');
    let mode = fields.next().unwrap_or_default();
    let object_type = fields.next().unwrap_or_default();
    let object = fields.next().unwrap_or_default();
    if fields.next().is_some() || mode.is_empty() || object_type.is_empty() || object.is_empty() {
        return Err(SnapshotFailure::MalformedOutput(
            "ls-tree record has an invalid header".to_owned(),
        ));
    }
    let kind = match (mode, object_type) {
        (b"040000", b"tree") => EntryKind::Tree,
        (b"100644", b"blob") => EntryKind::Blob,
        (b"100755", b"blob") => EntryKind::ExecutableBlob,
        (b"120000", b"blob") => EntryKind::Symlink,
        (b"160000", b"commit") => EntryKind::Gitlink,
        _ => {
            return Err(SnapshotFailure::ObjectMismatch(format!(
                "unsupported ls-tree mode/type combination {:?}/{:?}",
                String::from_utf8_lossy(mode),
                String::from_utf8_lossy(object_type)
            )));
        }
    };
    let object = parse_object_id(object, algorithm, "tree entry object identity")?;
    meter.admit(path, object, limits)?;
    let path = RepositoryPath::new(path.to_vec())?;
    Ok(Entry::new(path, EntryState::new(kind, object)))
}

pub(crate) fn parse_batch_contents(
    output: &[u8],
    selection: &BTreeSet<ContentId>,
    limits: SnapshotLimits,
) -> Result<BTreeMap<ContentId, Vec<u8>>, SnapshotFailure> {
    let mut cursor = 0_usize;
    let mut total = 0_u64;
    let mut contents = BTreeMap::new();
    for expected in selection {
        let header_end = output[cursor..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|offset| cursor + offset)
            .ok_or_else(|| {
                SnapshotFailure::MalformedOutput(
                    "cat-file content header is not newline terminated".to_owned(),
                )
            })?;
        let header = &output[cursor..header_end];
        cursor = header_end + 1;
        let mut fields = header.split(|byte| *byte == b' ');
        let object = fields.next().unwrap_or_default();
        let object_type = fields.next().unwrap_or_default();
        let size = fields.next();
        if object.is_empty() || fields.next().is_some() {
            return Err(SnapshotFailure::MalformedOutput(
                "cat-file content header is invalid".to_owned(),
            ));
        }
        let object = ContentId::from_object_id(parse_object_id(
            object,
            expected.algorithm(),
            "cat-file content identity",
        )?);
        if object != *expected {
            return Err(SnapshotFailure::ObjectMismatch(format!(
                "cat-file returned {object} while {expected} was selected"
            )));
        }
        if object_type == b"missing" && size.is_none() {
            return Err(SnapshotFailure::MissingContent(object.to_string()));
        }
        let size = size.ok_or_else(|| {
            SnapshotFailure::MalformedOutput("cat-file content header has no size".to_owned())
        })?;
        if object_type != b"blob" || size.is_empty() {
            return Err(SnapshotFailure::MalformedOutput(
                "cat-file content header is invalid".to_owned(),
            ));
        }
        let size = parse_decimal(size)?;
        enforce_limit("content bytes", limits.max_content_bytes(), size)?;
        total = total
            .checked_add(size)
            .ok_or(SnapshotFailure::LimitExceeded {
                limit: "total content bytes",
                maximum: limits.max_total_content_bytes(),
                observed: u64::MAX,
            })?;
        enforce_limit(
            "total content bytes",
            limits.max_total_content_bytes(),
            total,
        )?;
        let size = usize::try_from(size).map_err(|_| SnapshotFailure::LimitExceeded {
            limit: "content address space",
            maximum: usize::MAX as u64,
            observed: size,
        })?;
        let content_end = cursor.checked_add(size).ok_or_else(|| {
            SnapshotFailure::MalformedOutput("cat-file content length overflowed".to_owned())
        })?;
        if content_end >= output.len() {
            return Err(SnapshotFailure::MalformedOutput(
                "cat-file content is truncated".to_owned(),
            ));
        }
        if output[content_end] != b'\n' {
            return Err(SnapshotFailure::MalformedOutput(
                "cat-file content has no terminating newline".to_owned(),
            ));
        }
        let bytes = &output[cursor..content_end];
        verify_blob_identity(object, bytes)?;
        if contents.insert(object, bytes.to_vec()).is_some() {
            return Err(SnapshotFailure::UnexpectedContent(object.to_string()));
        }
        cursor = content_end + 1;
    }
    if cursor != output.len() {
        return Err(SnapshotFailure::MalformedOutput(
            "cat-file returned trailing or unselected data".to_owned(),
        ));
    }
    Ok(contents)
}

fn parse_decimal(value: &[u8]) -> Result<u64, SnapshotFailure> {
    if value.is_empty() || !value.iter().all(u8::is_ascii_digit) {
        return Err(SnapshotFailure::MalformedOutput(
            "cat-file content size is not an unsigned decimal".to_owned(),
        ));
    }
    str::from_utf8(value)
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| {
            SnapshotFailure::MalformedOutput("cat-file content size overflowed".to_owned())
        })
}

fn parse_object_id(
    value: &[u8],
    algorithm: ObjectAlgorithm,
    field: &'static str,
) -> Result<ObjectId, SnapshotFailure> {
    let value = str::from_utf8(value).map_err(|_| {
        SnapshotFailure::MalformedOutput(format!("{field} is not valid ASCII hexadecimal"))
    })?;
    let object = ObjectId::from_hex(value)?;
    if object.algorithm() != algorithm {
        return Err(SnapshotFailure::ObjectMismatch(format!(
            "{field} uses {} but the repository uses {algorithm}",
            object.algorithm()
        )));
    }
    Ok(object)
}

fn split_once(input: &[u8], separator: u8) -> Option<(&[u8], &[u8])> {
    let index = input.iter().position(|byte| *byte == separator)?;
    Some((&input[..index], &input[index + 1..]))
}

fn enforce_limit(limit: &'static str, maximum: u64, observed: u64) -> Result<(), SnapshotFailure> {
    if observed > maximum {
        Err(SnapshotFailure::LimitExceeded {
            limit,
            maximum,
            observed,
        })
    } else {
        Ok(())
    }
}
