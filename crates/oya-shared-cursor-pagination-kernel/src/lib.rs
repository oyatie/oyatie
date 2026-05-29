//! Cursor-pagination kernel — per-µservice trait surface for ADR-0150.
//!
//! # ADR-0150 (Tier-A hyperscaler pattern)
//!
//! Every list endpoint across the 33 µservices exposes opaque cursor
//! pagination (AWS `NextToken` + Stripe `starting_after`). Offset
//! pagination is FORBIDDEN. Cursors are base64-URL-encoded payloads
//! binding to the filter set via a `scope_hash` so reuse across
//! mismatched filters errors deterministically.
//!
//! # Naming justification
//!
//! `oya-shared-cursor-pagination-kernel` follows BNF v4.1:
//! `oya-<axis:shared>-<topic:cursor-pagination>-<layer:kernel>`.
//!
//! # References
//!
//! - docs/standards/cursor-pagination-canonical.md
//! - ADR-0150-cursor-pagination-canonical.md

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

pub mod cursor;
pub mod inmemory;

/// Opaque cursor value (base64-URL-encoded payload).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cursor(pub String); // data_class: INTERNAL_ONLY

/// Bounded page size with canonical clamp `[1, 100]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageSize(u32); // data_class: INTERNAL_ONLY

impl PageSize {
    /// # Errors
    /// - `PaginationError::PageSizeOutOfBounds` when not in `[1, 100]`.
    pub fn try_new(n: u32) -> Result<Self, PaginationError> {
        if !(1..=100).contains(&n) {
            return Err(PaginationError::PageSizeOutOfBounds { requested: n });
        }
        Ok(PageSize(n))
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl Default for PageSize {
    fn default() -> Self {
        PageSize(25)
    }
}

/// Page result envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Page<T> {
    pub items: Vec<T>,               // data_class: INTERNAL_ONLY
    pub next_cursor: Option<Cursor>, // data_class: INTERNAL_ONLY
    pub has_more: bool,              // data_class: INTERNAL_ONLY
    pub page_size: PageSize,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaginationError {
    PageSizeOutOfBounds {
        requested: u32,
    },
    CursorScopeMismatch {
        recorded_scope: String,
        attempted_scope: String,
    },
    CursorMalformed(String),
    SkeletonNotYetImplemented(&'static str),
}

impl fmt::Display for PaginationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaginationError::PageSizeOutOfBounds { requested } => write!(
                f,
                "oya-shared-cursor-pagination-kernel: page_size {requested} out of [1, 100]"
            ),
            PaginationError::CursorScopeMismatch {
                recorded_scope,
                attempted_scope,
            } => write!(
                f,
                "oya-shared-cursor-pagination-kernel: cursor-scope mismatch (recorded={recorded_scope:?}, attempted={attempted_scope:?})"
            ),
            PaginationError::CursorMalformed(value) => write!(
                f,
                "oya-shared-cursor-pagination-kernel: malformed cursor {value:?}"
            ),
            PaginationError::SkeletonNotYetImplemented(method) => write!(
                f,
                "oya-shared-cursor-pagination-kernel: {method} is skeleton-only \
                 (tracked under registry/placeholder-debt/adr-follow-ups.yaml#adr-0150-cursor-impl)"
            ),
        }
    }
}

impl std::error::Error for PaginationError {}

/// The trait every µservice integrates to expose cursor pagination.
pub trait CursorPaginationKernel: Send + Sync {
    type Item;
    type Filter;

    /// Fetch one page worth of items.
    ///
    /// # Errors
    /// - `CursorScopeMismatch` when the cursor's recorded filter set
    ///   does not match the active filter.
    /// - `CursorMalformed` when the cursor cannot be decoded.
    fn fetch_page(
        &self,
        cursor: Option<&Cursor>,
        page_size: PageSize,
        filter: &Self::Filter,
    ) -> Result<Page<Self::Item>, PaginationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_size_clamps_low() {
        assert_eq!(
            PageSize::try_new(0),
            Err(PaginationError::PageSizeOutOfBounds { requested: 0 })
        );
    }

    #[test]
    fn page_size_clamps_high() {
        assert_eq!(
            PageSize::try_new(101),
            Err(PaginationError::PageSizeOutOfBounds { requested: 101 })
        );
    }

    #[test]
    fn page_size_accepts_valid() {
        assert_eq!(PageSize::try_new(25).expect("ok").get(), 25);
    }

    #[test]
    fn default_page_size_is_25() {
        assert_eq!(PageSize::default().get(), 25);
    }

    #[test]
    fn error_display_carries_follow_up_pointer() {
        let err = PaginationError::SkeletonNotYetImplemented("fetch_page");
        let msg = format!("{err}");
        assert!(msg.contains("adr-0150-cursor-impl"));
    }
}
