//! Carrying the source's documentation into the emitted item.
//!
//! Documentation used to be dropped in full: seventeen doc-comment lines in the fixture corpus
//! reached the emitted output as none. That is a SILENT loss, and it sat in the one dimension the
//! coverage refusal does not look at — coverage proves every declaration was translated, not that
//! everything about a declaration survived.

use std::collections::BTreeMap;

use port_engine_api::{Declaration, DocConvention};

use crate::resolve::Resolver;

use crate::vocabulary::ATTR_DOC;

/// The doc lines a declaration carries, one per emitted `///`.
///
/// The front end records the block as a single attribute with embedded newlines, because that is
/// how the source stores it; splitting happens here, where the target's one-line-per-`///` shape
/// is known. A declaration with no documentation yields an empty vector rather than a blank
/// comment, so an undocumented item stays undocumented instead of gaining an empty line.
pub(crate) fn docs_of(declaration: &Declaration, resolver: &Resolver<'_>) -> Vec<String> {
    let convention = resolver.doc_convention;
    declaration
        .attr(ATTR_DOC)
        .map(|block| {
            rename_references(
                &rewrite_opening(block, &declaration.name, convention),
                &resolver.scope.renames,
                resolver.prose_type_names,
            )
                .lines()
                // A leading space is what `///` puts between the slashes and the text; the
                // renderer re-adds it, so carrying it here would double it.
                .map(|line| format!(" {}", line.trim_end()))
                .collect()
        })
        .unwrap_or_default()
}

/// Rewrite every word that NAMES a declaration of this unit into the target's name for it.
///
/// The source's documentation refers to `Run` because that is what the method is called there. The
/// emitted method is `run`, and prose that still says `Run` refers to nothing — a reviewer reading
/// the emitted crate found three such words and called them the cheapest possible proof that nobody
/// had read the Rust.
///
/// EXACT and case-sensitive, which is what keeps it from touching English. A method named `Run`
/// does not match the word "run" in a sentence, because the two differ; and where the source name
/// and the target name are the same word, the rewrite is the identity. What it does catch is
/// precisely the case that matters: a capitalised identifier standing where the target has a
/// lower-cased one.
///
/// Word boundaries are non-identifier characters, so `Run` inside `Runner` is not a word and is not
/// touched. A name with an ambiguous target is absent from the map and is left alone.
fn rename_references(
    block: &str,
    renames: &BTreeMap<String, String>,
    types: &BTreeMap<String, String>,
) -> String {
    let mut out = String::with_capacity(block.len());
    let mut word = String::new();
    for character in block.chars() {
        if character.is_alphanumeric() || character == '_' {
            word.push(character);
            continue;
        }
        push_word(&mut out, &mut word, renames, types);
        out.push(character);
    }
    push_word(&mut out, &mut word, renames, types);
    out
}

/// Flush one accumulated word, renamed if it names something.
fn push_word(
    out: &mut String,
    word: &mut String,
    renames: &BTreeMap<String, String>,
    types: &BTreeMap<String, String>,
) {
    // A DECLARATION of this unit first, then a source TYPE the pack names. The two cannot collide:
    // the pack's set holds only the source's own primitive spellings, and a unit declaring one of
    // those is declaring a name the target could not use anyway.
    match renames.get(word.as_str()).or_else(|| types.get(word.as_str())) {
        Some(target) => out.push_str(target),
        None => out.push_str(word),
    }
    word.clear();
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
