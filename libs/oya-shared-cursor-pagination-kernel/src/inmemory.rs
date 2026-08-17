//! Pure in-memory reference implementation of `CursorPaginationKernel`.
//!
//! `InMemoryCursorPaginator<T, F>` holds an owned item slice and a filter
//! predicate. It is the canonical reference implementation for consumers
//! that need deterministic, I/O-free cursor pagination in tests.

use crate::{
    Cursor, CursorPaginationKernel, Page, PageSize, PaginationError,
    cursor::{CursorPayload, scope_hash},
};

/// Pure in-memory reference implementation of [`CursorPaginationKernel`].
///
/// - `T`: item type; must be `Clone + Send + Sync`.
/// - `F`: filter predicate `Fn(&T, &String) -> bool + Send + Sync`.
///
/// The `Filter` associated type is `String`; callers supply the canonical
/// string representation of their filter set, which is used as the
/// `scope_hash` input to detect cursor/filter mismatches.
pub struct InMemoryCursorPaginator<T, F> {
    items: Vec<T>,
    filter_fn: F,
}

impl<T, F> InMemoryCursorPaginator<T, F>
where
    T: Clone + Send + Sync,
    F: Fn(&T, &String) -> bool + Send + Sync,
{
    /// Create a paginator over `items` with the given `filter_fn`.
    pub fn new(items: Vec<T>, filter_fn: F) -> Self {
        Self { items, filter_fn }
    }
}

impl<T, F> CursorPaginationKernel for InMemoryCursorPaginator<T, F>
where
    T: Clone + Send + Sync,
    F: Fn(&T, &String) -> bool + Send + Sync,
{
    type Item = T;
    type Filter = String;

    /// Fetch one page of items.
    ///
    /// # Errors
    /// - [`PaginationError::CursorScopeMismatch`] when the cursor was
    ///   produced for a different filter set.
    /// - [`PaginationError::CursorMalformed`] when the cursor cannot be
    ///   decoded.
    fn fetch_page(
        &self,
        cursor: Option<&Cursor>,
        page_size: PageSize,
        filter: &Self::Filter,
    ) -> Result<Page<Self::Item>, PaginationError> {
        let active_scope = scope_hash(filter);

        // Decode cursor and validate scope binding.
        let offset: usize = if let Some(c) = cursor {
            let payload = CursorPayload::from_cursor(c)?;
            if payload.scope != active_scope {
                return Err(PaginationError::CursorScopeMismatch {
                    recorded_scope: payload.scope.to_string(),
                    attempted_scope: active_scope.to_string(),
                });
            }
            payload.offset as usize
        } else {
            0
        };

        // Apply filter to produce the full logical sequence.
        let filtered: Vec<T> = self
            .items
            .iter()
            .filter(|item| (self.filter_fn)(item, filter))
            .cloned()
            .collect();

        let size = page_size.get() as usize;
        let page_items: Vec<T> = filtered.iter().skip(offset).take(size).cloned().collect();

        let next_offset = offset + page_items.len();
        let has_more = next_offset < filtered.len();

        let next_cursor = if has_more {
            Some(
                CursorPayload {
                    offset: next_offset as u64,
                    scope: active_scope,
                }
                .to_cursor(),
            )
        } else {
            None
        };

        Ok(Page {
            items: page_items,
            next_cursor,
            has_more,
            page_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PageSize;

    fn make_pager(items: Vec<i32>) -> InMemoryCursorPaginator<i32, impl Fn(&i32, &String) -> bool> {
        // Filter: items whose string representation equals the filter value,
        // OR if filter is "all", accept everything.
        InMemoryCursorPaginator::new(items, |item: &i32, filter: &String| {
            filter == "all" || item.to_string() == *filter
        })
    }

    fn make_passthrough(
        items: Vec<i32>,
    ) -> InMemoryCursorPaginator<i32, impl Fn(&i32, &String) -> bool> {
        InMemoryCursorPaginator::new(items, |_item: &i32, _filter: &String| true)
    }

    #[test]
    fn first_page_returns_bounded_items() {
        let pager = make_passthrough((0..10).collect());
        let ps = PageSize::try_new(3).unwrap();
        let page = pager.fetch_page(None, ps, &"all".to_string()).unwrap();
        assert_eq!(page.items, vec![0, 1, 2]);
        assert!(page.has_more);
        assert!(page.next_cursor.is_some());
        assert_eq!(page.page_size, ps);
    }

    #[test]
    fn cursor_advances_through_pages() {
        let pager = make_passthrough((0..10).collect());
        let ps = PageSize::try_new(3).unwrap();
        let filter = "all".to_string();

        let page1 = pager.fetch_page(None, ps, &filter).unwrap();
        assert_eq!(page1.items, vec![0, 1, 2]);
        assert!(page1.has_more);

        let page2 = pager
            .fetch_page(page1.next_cursor.as_ref(), ps, &filter)
            .unwrap();
        assert_eq!(page2.items, vec![3, 4, 5]);
        assert!(page2.has_more);

        let page3 = pager
            .fetch_page(page2.next_cursor.as_ref(), ps, &filter)
            .unwrap();
        assert_eq!(page3.items, vec![6, 7, 8]);
        assert!(page3.has_more);

        let page4 = pager
            .fetch_page(page3.next_cursor.as_ref(), ps, &filter)
            .unwrap();
        assert_eq!(page4.items, vec![9]);
        assert!(!page4.has_more);
        assert_eq!(page4.next_cursor, None);
    }

    #[test]
    fn final_page_has_more_false_and_no_cursor() {
        let pager = make_passthrough(vec![1, 2, 3]);
        let ps = PageSize::try_new(10).unwrap();
        let page = pager.fetch_page(None, ps, &"all".to_string()).unwrap();
        assert_eq!(page.items, vec![1, 2, 3]);
        assert!(!page.has_more);
        assert_eq!(page.next_cursor, None);
    }

    #[test]
    fn cursor_scope_mismatch_returns_error() {
        let pager = make_passthrough((0..20).collect());
        let ps = PageSize::try_new(5).unwrap();

        // Produce a cursor for filter "filter-a".
        let page1 = pager.fetch_page(None, ps, &"filter-a".to_string()).unwrap();
        let cursor_a = page1.next_cursor.unwrap();

        // Reuse cursor with filter "filter-b".
        let err = pager
            .fetch_page(Some(&cursor_a), ps, &"filter-b".to_string())
            .unwrap_err();
        assert!(
            matches!(err, PaginationError::CursorScopeMismatch { .. }),
            "expected CursorScopeMismatch, got {err:?}"
        );
    }

    #[test]
    fn malformed_cursor_returns_error() {
        let pager = make_passthrough((0..5).collect());
        let bad_cursor = Cursor("!!!not-valid-base64!!!".to_string());
        let err = pager
            .fetch_page(Some(&bad_cursor), PageSize::default(), &"all".to_string())
            .unwrap_err();
        assert!(
            matches!(err, PaginationError::CursorMalformed(_)),
            "expected CursorMalformed, got {err:?}"
        );
    }

    #[test]
    fn cursor_roundtrips_opaque_base64url() {
        let pager = make_passthrough((0..10).collect());
        let ps = PageSize::try_new(4).unwrap();
        let filter = "all".to_string();

        let page = pager.fetch_page(None, ps, &filter).unwrap();
        let c = page.next_cursor.as_ref().unwrap();

        // Cursor value must be opaque base64-URL (no padding, URL-safe chars only).
        assert!(
            c.0.chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'),
            "cursor contains non-base64url characters: {:?}",
            c.0
        );

        // Round-trip: using the cursor against the same filter must work.
        let page2 = pager.fetch_page(Some(c), ps, &filter).unwrap();
        assert_eq!(page2.items, vec![4, 5, 6, 7]);
    }

    #[test]
    fn empty_item_list_returns_empty_page() {
        let pager = make_passthrough(vec![]);
        let page = pager
            .fetch_page(None, PageSize::default(), &"all".to_string())
            .unwrap();
        assert!(page.items.is_empty());
        assert!(!page.has_more);
        assert_eq!(page.next_cursor, None);
    }

    #[test]
    fn filter_applied_correctly() {
        // items: [1..=9]; filter matches only even numbers (as string "even" → custom predicate)
        let pager = InMemoryCursorPaginator::new((1..=9).collect::<Vec<i32>>(), |item, filter| {
            if filter == "even" {
                item % 2 == 0
            } else {
                true
            }
        });
        let ps = PageSize::try_new(2).unwrap();
        let filter = "even".to_string();

        let page1 = pager.fetch_page(None, ps, &filter).unwrap();
        assert_eq!(page1.items, vec![2, 4]);
        assert!(page1.has_more);

        let page2 = pager
            .fetch_page(page1.next_cursor.as_ref(), ps, &filter)
            .unwrap();
        assert_eq!(page2.items, vec![6, 8]);
        assert!(!page2.has_more);
        assert_eq!(page2.next_cursor, None);
    }

    #[test]
    fn page_size_boundary_1() {
        let pager = make_passthrough((0..5).collect());
        let ps = PageSize::try_new(1).unwrap();
        let filter = "all".to_string();
        let page = pager.fetch_page(None, ps, &filter).unwrap();
        assert_eq!(page.items, vec![0]);
        assert!(page.has_more);
    }

    #[test]
    fn page_size_boundary_100() {
        let pager = make_passthrough((0..100).collect());
        let ps = PageSize::try_new(100).unwrap();
        let page = pager.fetch_page(None, ps, &"all".to_string()).unwrap();
        assert_eq!(page.items.len(), 100);
        assert!(!page.has_more);
    }
}
