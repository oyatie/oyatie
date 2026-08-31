use std::collections::BTreeMap;

use pipeline_repository::{
    ContentId, Entry, EntryKind, ObjectAlgorithm, ObjectId, SnapshotFailure, TreeId,
};
use sha1_checked::{Digest, Sha1};
use sha2::Sha256;

pub(crate) fn verify_blob_identity(
    identity: ContentId,
    contents: &[u8],
) -> Result<(), SnapshotFailure> {
    match identity.algorithm() {
        ObjectAlgorithm::Sha1 => verify_blob_with::<Sha1>(identity, contents),
        ObjectAlgorithm::Sha256 => verify_blob_with::<Sha256>(identity, contents),
    }
}

fn verify_blob_with<D: GitObjectHasher>(
    identity: ContentId,
    contents: &[u8],
) -> Result<(), SnapshotFailure> {
    let mut digest = git_object_digest::<D>("blob", contents.len());
    GitObjectHasher::update(&mut digest, contents);
    if digest.matches(identity.object_id())? {
        return Ok(());
    }
    Err(SnapshotFailure::ObjectMismatch(format!(
        "Git blob {identity} content does not match its object identity"
    )))
}

pub(crate) fn verify_tree_identities(
    root: TreeId,
    entries: &[Entry],
) -> Result<(), SnapshotFailure> {
    let mut children: BTreeMap<&[u8], Vec<&Entry>> = BTreeMap::new();
    let mut trees = BTreeMap::from([(b"".as_slice(), root.object_id())]);
    children.insert(b"", Vec::new());
    for entry in entries {
        let (parent, _) = parent_and_name(entry.path().as_bytes());
        children.entry(parent).or_default().push(entry);
        if entry.kind().is_tree() {
            let path = entry.path().as_bytes();
            children.entry(path).or_default();
            trees.insert(path, entry.object());
        }
    }

    for (path, identity) in trees {
        let entries = children
            .get(&path)
            .expect("every declared tree has a child collection");
        verify_tree(identity, path, entries)?;
    }
    Ok(())
}

fn verify_tree(identity: ObjectId, path: &[u8], entries: &[&Entry]) -> Result<(), SnapshotFailure> {
    match identity.algorithm() {
        ObjectAlgorithm::Sha1 => verify_tree_with::<Sha1>(identity, path, entries),
        ObjectAlgorithm::Sha256 => verify_tree_with::<Sha256>(identity, path, entries),
    }
}

fn verify_tree_with<D: GitObjectHasher>(
    identity: ObjectId,
    path: &[u8],
    entries: &[&Entry],
) -> Result<(), SnapshotFailure> {
    let size = entries.iter().try_fold(0_usize, |size, entry| {
        let (_, name) = parent_and_name(entry.path().as_bytes());
        size.checked_add(tree_mode(entry.kind()).len())
            .and_then(|size| size.checked_add(name.len()))
            .and_then(|size| size.checked_add(entry.object().as_bytes().len() + 2))
    });
    let size = size.ok_or_else(|| {
        SnapshotFailure::MalformedOutput("Git tree serialization length overflowed".to_owned())
    })?;
    let mut digest = git_object_digest::<D>("tree", size);
    for entry in entries {
        let (_, name) = parent_and_name(entry.path().as_bytes());
        GitObjectHasher::update(&mut digest, tree_mode(entry.kind()));
        GitObjectHasher::update(&mut digest, b" ");
        GitObjectHasher::update(&mut digest, name);
        GitObjectHasher::update(&mut digest, b"\0");
        GitObjectHasher::update(&mut digest, entry.object().as_bytes());
    }
    if digest.matches(identity)? {
        Ok(())
    } else {
        Err(SnapshotFailure::ObjectMismatch(format!(
            "Git tree {identity} at {path:?} does not match its entries"
        )))
    }
}

const fn tree_mode(kind: EntryKind) -> &'static [u8] {
    match kind {
        EntryKind::Tree => b"40000",
        _ => kind.canonical_mode(),
    }
}

fn parent_and_name(path: &[u8]) -> (&[u8], &[u8]) {
    path.iter()
        .rposition(|byte| *byte == b'/')
        .map_or((&[], path), |index| (&path[..index], &path[index + 1..]))
}

trait GitObjectHasher: Sized {
    fn new() -> Self;
    fn update(&mut self, bytes: &[u8]);
    fn matches(self, identity: ObjectId) -> Result<bool, SnapshotFailure>;
}

impl GitObjectHasher for Sha1 {
    fn new() -> Self {
        <Self as Digest>::new()
    }

    fn update(&mut self, bytes: &[u8]) {
        Digest::update(self, bytes);
    }

    fn matches(self, identity: ObjectId) -> Result<bool, SnapshotFailure> {
        let result = self.try_finalize();
        if result.has_collision() {
            Err(SnapshotFailure::ObjectCollision(identity.to_string()))
        } else {
            Ok(result.hash().as_slice() == identity.as_bytes())
        }
    }
}

impl GitObjectHasher for Sha256 {
    fn new() -> Self {
        <Self as Digest>::new()
    }

    fn update(&mut self, bytes: &[u8]) {
        Digest::update(self, bytes);
    }

    fn matches(self, identity: ObjectId) -> Result<bool, SnapshotFailure> {
        Ok(&Digest::finalize(self)[..] == identity.as_bytes())
    }
}

fn git_object_digest<D: GitObjectHasher>(kind: &str, size: usize) -> D {
    let mut digest = D::new();
    GitObjectHasher::update(&mut digest, kind.as_bytes());
    GitObjectHasher::update(&mut digest, b" ");
    update_decimal(&mut digest, size);
    GitObjectHasher::update(&mut digest, b"\0");
    digest
}

fn update_decimal<D: GitObjectHasher>(digest: &mut D, mut value: usize) {
    let mut digits = [0_u8; usize::MAX.ilog10() as usize + 1];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            GitObjectHasher::update(digest, &digits[start..]);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tree_identity_is_verified() {
        let empty = TreeId::from_hex("4b825dc642cb6eb9a060e54bf8d69288fbee4904").unwrap();
        verify_tree_identities(empty, &[]).unwrap();
    }

    #[test]
    fn forged_tree_identity_is_refused() {
        let forged = TreeId::from_hex("1111111111111111111111111111111111111111").unwrap();
        assert!(matches!(
            verify_tree_identities(forged, &[]),
            Err(SnapshotFailure::ObjectMismatch(_))
        ));
    }
}
