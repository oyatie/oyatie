//! Carrying the source's documentation into the emitted item.
//!
//! Documentation used to be dropped in full: seventeen doc-comment lines in the fixture corpus
//! reached the emitted output as none. That is a SILENT loss, and it sat in the one dimension the
//! coverage refusal does not look at — coverage proves every declaration was translated, not that
//! everything about a declaration survived.

use port_engine_api::Declaration;

use crate::vocabulary::ATTR_DOC;

/// The doc lines a declaration carries, one per emitted `///`.
///
/// The front end records the block as a single attribute with embedded newlines, because that is
/// how the source stores it; splitting happens here, where the target's one-line-per-`///` shape
/// is known. A declaration with no documentation yields an empty vector rather than a blank
/// comment, so an undocumented item stays undocumented instead of gaining an empty line.
pub(crate) fn docs_of(declaration: &Declaration) -> Vec<String> {
    declaration
        .attr(ATTR_DOC)
        .map(|block| {
            block
                .lines()
                // A leading space is what `///` puts between the slashes and the text; the
                // renderer re-adds it, so carrying it here would double it.
                .map(|line| format!(" {}", line.trim_end()))
                .collect()
        })
        .unwrap_or_default()
}
