use std::collections::BTreeMap;

use crate::{DigestBuilder, EntryState, EvidenceDigest, RepositoryManifest, RepositoryPath};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeltaEntry {
    path: RepositoryPath,
    before: Option<EntryState>,
    after: Option<EntryState>,
}

impl DeltaEntry {
    pub const fn path(&self) -> &RepositoryPath {
        &self.path
    }

    pub const fn before(&self) -> Option<EntryState> {
        self.before
    }

    pub const fn after(&self) -> Option<EntryState> {
        self.after
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryDelta {
    entries: Vec<DeltaEntry>,
    exact_moves: BTreeMap<RepositoryPath, RepositoryPath>,
    digest: EvidenceDigest,
}

impl RepositoryDelta {
    pub fn between(base: &RepositoryManifest, head: &RepositoryManifest) -> Self {
        let entries = if base.entries_digest() == head.entries_digest() {
            Vec::new()
        } else {
            changed_entries(base, head)
        };
        let exact_moves = exact_moves(&entries);
        let mut digest = DigestBuilder::new(b"pipeline-repository-delta-v1");
        digest.push_bytes(base.digest().as_bytes());
        digest.push_bytes(head.digest().as_bytes());
        digest.push_u64(entries.len() as u64);
        for entry in &entries {
            entry.path.digest_into(&mut digest);
            digest_optional_state(&mut digest, entry.before);
            digest_optional_state(&mut digest, entry.after);
        }
        Self {
            entries,
            exact_moves,
            digest: digest.finish(),
        }
    }

    pub fn entries(&self) -> &[DeltaEntry] {
        &self.entries
    }

    pub fn exact_moves(&self) -> &BTreeMap<RepositoryPath, RepositoryPath> {
        &self.exact_moves
    }

    pub const fn digest(&self) -> EvidenceDigest {
        self.digest
    }
}

fn changed_entries(base: &RepositoryManifest, head: &RepositoryManifest) -> Vec<DeltaEntry> {
    let mut base_entries = base
        .entries()
        .iter()
        .filter(|entry| !entry.kind().is_tree())
        .peekable();
    let mut head_entries = head
        .entries()
        .iter()
        .filter(|entry| !entry.kind().is_tree())
        .peekable();
    let mut entries = Vec::new();
    loop {
        match (base_entries.peek().copied(), head_entries.peek().copied()) {
            (Some(before), Some(after)) => match before.path().cmp(after.path()) {
                std::cmp::Ordering::Less => {
                    entries.push(DeltaEntry {
                        path: before.path().clone(),
                        before: Some(before.state()),
                        after: None,
                    });
                    base_entries.next();
                }
                std::cmp::Ordering::Greater => {
                    entries.push(DeltaEntry {
                        path: after.path().clone(),
                        before: None,
                        after: Some(after.state()),
                    });
                    head_entries.next();
                }
                std::cmp::Ordering::Equal => {
                    if before.state() != after.state() {
                        entries.push(DeltaEntry {
                            path: before.path().clone(),
                            before: Some(before.state()),
                            after: Some(after.state()),
                        });
                    }
                    base_entries.next();
                    head_entries.next();
                }
            },
            (Some(before), None) => {
                entries.push(DeltaEntry {
                    path: before.path().clone(),
                    before: Some(before.state()),
                    after: None,
                });
                base_entries.next();
            }
            (None, Some(after)) => {
                entries.push(DeltaEntry {
                    path: after.path().clone(),
                    before: None,
                    after: Some(after.state()),
                });
                head_entries.next();
            }
            (None, None) => break,
        }
    }

    entries
}

fn exact_moves(entries: &[DeltaEntry]) -> BTreeMap<RepositoryPath, RepositoryPath> {
    #[derive(Clone, Copy)]
    struct Candidate<'a> {
        path: &'a RepositoryPath,
        unique: bool,
    }

    fn record<'a>(
        candidates: &mut BTreeMap<EntryState, Candidate<'a>>,
        state: EntryState,
        path: &'a RepositoryPath,
    ) {
        candidates
            .entry(state)
            .and_modify(|candidate| candidate.unique = false)
            .or_insert(Candidate { path, unique: true });
    }

    let mut deleted = BTreeMap::new();
    let mut added = BTreeMap::new();
    for entry in entries {
        match (entry.before, entry.after) {
            (Some(state), None) if state.content_id().is_some() => {
                record(&mut deleted, state, &entry.path);
            }
            (None, Some(state)) if state.content_id().is_some() => {
                record(&mut added, state, &entry.path);
            }
            _ => {}
        }
    }
    let mut moves = BTreeMap::new();
    for (state, source) in deleted {
        if source.unique
            && let Some(destination) = added.get(&state)
            && destination.unique
        {
            moves.insert(destination.path.clone(), source.path.clone());
        }
    }
    moves
}

fn digest_optional_state(digest: &mut DigestBuilder, state: Option<EntryState>) {
    match state {
        Some(state) => {
            digest.push_bytes(b"present");
            state.digest_into(digest);
        }
        None => digest.push_bytes(b"absent"),
    }
}
