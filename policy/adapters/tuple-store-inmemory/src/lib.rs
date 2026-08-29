//! In-memory tuple store.
//!
//! Writes are totally ordered by an increasing version, and each write returns
//! a zookie naming that version. A read at an earlier snapshot cannot observe
//! a later write, so "deny before the grant was written, allow after" is a
//! real property here and not a simulation of one.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use policy_cedar_domain::rebac::{
    RebacReadSnapshot, RebacTuple, RebacTuplePage, RebacTupleQuery, RebacTupleStore,
    RebacTupleStoreError, SnapshotToken, Zookie,
};

/// Default tuples per page. Small on purpose: a reader that stops at the first
/// page is wrong, and a store that only ever returns one page never exercises
/// that path.
const DEFAULT_PAGE_SIZE: usize = 50;

#[derive(Clone, Debug, Default)]
pub struct InMemoryTupleStore {
    /// `(version_written, tuple)`, in write order.
    written: Vec<(u64, RebacTuple)>,
    version: u64,
    page_size: usize,
}

impl InMemoryTupleStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            written: Vec::new(),
            version: 0,
            page_size: DEFAULT_PAGE_SIZE,
        }
    }

    /// Page size for reads. Used by conformance suites to force pagination.
    ///
    /// # Panics
    /// When `page_size` is zero: a zero page can never make progress, and a
    /// reader looping on it would hang rather than fail.
    #[must_use]
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        assert!(page_size > 0, "page size must be positive");
        self.page_size = page_size;
        self
    }

    /// The zookie naming the most recent write.
    ///
    /// # Errors
    /// When the version cannot be rendered as a valid token.
    pub fn head(&self) -> Result<Zookie, RebacTupleStoreError> {
        Zookie::new(self.version.to_string()).map_err(RebacTupleStoreError::InvalidZookie)
    }

    fn visible_at(&self, snapshot: &RebacReadSnapshot) -> Result<u64, RebacTupleStoreError> {
        let token = snapshot.clone().into_snapshot_token();
        if token.as_str() == "latest" {
            return Ok(self.version);
        }
        let requested = token.as_str().parse::<u64>().map_err(|_| {
            RebacTupleStoreError::Backend(format!(
                "snapshot token {:?} was not minted by this store",
                token.as_str()
            ))
        })?;
        if requested > self.version {
            return Err(RebacTupleStoreError::StaleSnapshot {
                requested: token,
                current: SnapshotToken::new(self.version.to_string())
                    .map_err(RebacTupleStoreError::InvalidZookie)?,
            });
        }
        Ok(requested)
    }
}

impl RebacTupleStore for InMemoryTupleStore {
    fn write_tuple(&mut self, tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError> {
        self.version = self.version.saturating_add(1);
        self.written.push((self.version, tuple));
        self.head()
    }

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: RebacReadSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError> {
        let ceiling = self.visible_at(&snapshot)?;
        let matched: Vec<RebacTuple> = self
            .written
            .iter()
            .filter(|(version, tuple)| *version <= ceiling && query.matches(tuple))
            .map(|(_, tuple)| tuple.clone())
            .collect();

        let start = match query.page_token.as_deref() {
            None => 0,
            Some(token) => token.parse::<usize>().map_err(|_| {
                RebacTupleStoreError::Backend(format!("page token {token:?} is not one of ours"))
            })?,
        };
        let end = start.saturating_add(self.page_size).min(matched.len());
        let tuples = matched.get(start..end).unwrap_or_default().to_vec();
        let next_page_token = (end < matched.len()).then(|| end.to_string());

        Ok(RebacTuplePage {
            tuples,
            snapshot: SnapshotToken::new(ceiling.to_string())
                .map_err(RebacTupleStoreError::InvalidZookie)?,
            next_page_token,
        })
    }
}
