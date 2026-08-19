//! Carrying the source's documentation into the emitted item.
//!
//! Documentation used to be dropped in full: seventeen doc-comment lines in the fixture corpus
//! reached the emitted output as none. That is a SILENT loss, and it sat in the one dimension the
//! coverage refusal does not look at — coverage proves every declaration was translated, not that
//! everything about a declaration survived.

use port_engine_api::{Declaration, DocConvention};

use crate::vocabulary::ATTR_DOC;

/// The doc lines a declaration carries, one per emitted `///`.
///
/// The front end records the block as a single attribute with embedded newlines, because that is
/// how the source stores it; splitting happens here, where the target's one-line-per-`///` shape
/// is known. A declaration with no documentation yields an empty vector rather than a blank
/// comment, so an undocumented item stays undocumented instead of gaining an empty line.
pub(crate) fn docs_of(declaration: &Declaration, convention: &DocConvention) -> Vec<String> {
    declaration
        .attr(ATTR_DOC)
        .map(|block| {
            rewrite_opening(block, &declaration.name, convention)
                .lines()
                // A leading space is what `///` puts between the slashes and the text; the
                // renderer re-adds it, so carrying it here would double it.
                .map(|line| format!(" {}", line.trim_end()))
                .collect()
        })
        .unwrap_or_default()
}

/// Drop a leading repetition of the item's own name, as the target's convention requires.
///
/// The source's convention REQUIRES a doc comment to open with the identifier it documents; the
/// target's requires that it does not. Copying the source form verbatim is what makes emitted
/// documentation read as translated rather than written — a reviewer who did not know the code was
/// generated flagged it on every doc comment as the loudest signal a Rust developer had not
/// written it.
///
/// BOUNDED on purpose, and the bound is the substance. The leading word must equal the
/// declaration's own source name EXACTLY; a copula immediately after it is dropped with it, so
/// `ID is an alias` becomes `An alias` rather than the ungrammatical `Is an alias`. A doc opening
/// any other way is returned untouched, because its author already chose an opening and this has
/// no business rewording prose it was not asked about.
fn rewrite_opening(block: &str, name: &str, convention: &DocConvention) -> String {
    if !convention.strip_leading_name || name.is_empty() {
        return block.to_owned();
    }
    let Some(rest) = block.strip_prefix(name) else {
        return block.to_owned();
    };
    let Some(rest) = rest.strip_prefix(' ') else {
        // The name is a PREFIX of a longer word rather than the word itself — `IDs` opening a doc
        // for `ID`. Not a repetition, so not this rule's business.
        return block.to_owned();
    };

    let trimmed = match rest.split_once(' ') {
        Some((first, tail)) if convention.copulas.contains(first) => tail,
        _ => rest,
    };
    capitalised(trimmed)
}

/// The text with its first letter upper-cased, which is where a sentence now begins.
fn capitalised(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
