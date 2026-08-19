//! Carrying the source's documentation into the emitted item.
//!
//! Documentation used to be dropped in full: seventeen doc-comment lines in the fixture corpus
//! reached the emitted output as none. That is a SILENT loss, and it sat in the one dimension the
//! coverage refusal does not look at — coverage proves every declaration was translated, not that
//! everything about a declaration survived.

use std::collections::BTreeMap;

use port_engine_api::{Declaration, DocConvention};

use crate::error::TransformError;
use crate::resolve::Resolver;

use crate::vocabulary::ATTR_DOC;

/// The doc lines a declaration carries, one per emitted `///`.
///
/// The front end records the block as a single attribute with embedded newlines, because that is
/// how the source stores it; splitting happens here, where the target's one-line-per-`///` shape
/// is known. A declaration with no documentation yields an empty vector rather than a blank
/// comment, so an undocumented item stays undocumented instead of gaining an empty line.
pub(crate) fn docs_of(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Vec<String>, TransformError> {
    let Some(block) = declaration.attr(ATTR_DOC) else {
        return Ok(Vec::new());
    };
    refuse_dangling_reference(block, declaration, resolver)?;
    Ok(docs_from_block(block, &declaration.name, resolver))
}

/// The same, from a doc block and the name it opens with, for a declaration not in hand.
///
/// The grouped failure enum builds a variant per sentinel and needs each one's documentation, but
/// it works from the unit's sentinel list rather than from the declarations. Factored so both go
/// through one pipeline: two spellings of a doc rewrite would drift exactly as two spellings of a
/// name did.
///
/// No refusal check here — the declaration this block came from is surveyed on its own, and that is
/// where its dangling references are caught.
pub(crate) fn docs_from_block(
    block: &str,
    name: &str,
    resolver: &Resolver<'_>,
) -> Vec<String> {
    let convention = resolver.doc_convention;
    {
        {
            rename_references(
                &rewrite_opening(block, name, convention),
                &resolver.scope.renames,
                resolver.prose_type_names,
            )
            .lines()
            // A leading space is what `///` puts between the slashes and the text; the renderer
            // re-adds it, so carrying it here would double it.
            .map(|line| format!(" {}", line.trim_end()))
            .collect()
        }
    }
}

/// Refuse prose that NAMES something the emitted crate does not contain.
///
/// Self-containment, at the prose layer. A body that calls a declaration which refused is refused,
/// because the emitted crate would not contain the name; a doc comment that describes one is the
/// same defect and reads worse — it documents an API that is not there, in the voice of somebody
/// who checked. A reviewer reading a real ported package ranked exactly this as their single most
/// decisive piece of evidence that nobody had read the output.
///
/// Two shapes, one reason. A word that names a declaration of THIS unit which is not being emitted;
/// and a qualified name whose package is not a unit of this model, which no module here can reach.
///
/// Deliberately narrow. A word only counts when the unit actually declares it, so ordinary English
/// is untouched; a qualified name only counts in the source's own `package.Exported` shape, so a
/// decimal number and a sentence boundary are not it.
fn refuse_dangling_reference(
    block: &str,
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<(), TransformError> {
    for word in block.split(|ch: char| !(ch.is_alphanumeric() || ch == '_' || ch == '.')) {
        if let Some((package, member)) = qualified(word)
            && !resolver.units.contains(package)
        {
            return Err(TransformError::Unsupported {
                name: declaration.name.clone(),
                detail: format!(
                    "its documentation names `{package}.{member}`, which is in a package this \
                     snapshot does not contain — the emitted crate has nothing by that name, so \
                     the prose describes an API that is not there"
                ),
            });
        }
        // Named by this unit, and not being emitted.
        //
        // EXPORTED only, which is the same bound the rename map's own construction uses and for the
        // same reason: an unexported source name is lower-case and indistinguishable from English.
        // Without it a unit with a field called `con` refused every declaration whose prose used
        // the word — the exact false positive that bound exists to prevent, arrived at from the
        // other direction.
        //
        // A SENTINEL counts as emitted. It is emitted, as a type, by a path that does not go
        // through the reachability set — so asking that set alone reported seven of `semver`'s own
        // error types as absent while they sat in the output.
        let exported = word.chars().next().is_some_and(char::is_uppercase);
        // A declaration's own name is never dangling: the prose describes THIS declaration, and
        // this declaration is the one being emitted.
        if !exported || word == declaration.name || !resolver.scope.renames.contains_key(word) {
            continue;
        }
        // A MEMBER is emitted exactly when its owner is, so that is what is asked about it. The
        // top-level set knows nothing about members, and asking it reported every one of them
        // absent — including the ones in the output.
        let subject = resolver
            .scope
            .member_owners
            .get(word)
            .map_or(word, String::as_str);
        if !resolver.emitted.contains(subject) && !resolver.scope.sentinels.contains_key(subject) {
            return Err(TransformError::Unsupported {
                name: declaration.name.clone(),
                detail: format!(
                    "its documentation names `{word}`, which this unit declares and is not \
                     emitting — the prose would describe an API the crate does not contain"
                ),
            });
        }
    }
    Ok(())
}

/// A word in the source's `package.Exported` shape, split into its two halves.
///
/// Strict on both sides: the package half starts with a lower-case letter and the member half with
/// an upper-case one, which is the source's own convention and is what keeps a decimal number and a
/// sentence boundary from matching.
fn qualified(word: &str) -> Option<(&str, &str)> {
    let (package, member) = word.split_once('.')?;
    let starts_lower = package.chars().next().is_some_and(char::is_lowercase);
    let starts_upper = member.chars().next().is_some_and(char::is_uppercase);
    (starts_lower && starts_upper && package.chars().all(char::is_alphanumeric))
        .then_some((package, member))
}

/// Rewrite source TYPE names inside text the emitted program itself carries.
///
/// The same map the documentation uses, applied to a MESSAGE. Held apart from that path because the
/// justification is different and weaker, and the difference is worth stating rather than blurring.
///
/// A doc comment describes the code, so rewriting a type name in it makes the description true. A
/// message is the program's OUTPUT, and rewriting output is normally changing the program — which
/// is why this engine declined it twice. What changed the answer is that the message in question was
/// no longer TRUE: "version increment would overflow uint64" names a type the emitted crate does not
/// have and never will, in a crate whose own constant beside it says `u64`. Faithfulness to a
/// sentence that has become false is not faithfulness.
///
/// The cost is real and is stated: a program that deliberately prints the SOURCE language's type
/// name — a diagnostic that reports what it was compiled against — is changed by this. That is
/// accepted because the map is deliberately tiny and holds only names that are not English words and
/// have no other use in a message; two independent reviewers ranked the untranslated one among their
/// most decisive evidence that the output was machine-produced.
pub(crate) fn rename_types_in_text(text: &str, types: &BTreeMap<String, String>) -> String {
    rename_references(text, &BTreeMap::new(), types)
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
