//! What documentation may not SAY about the code it documents.
//!
//! Split from `docs.rs` because the two are different jobs: that file rewrites prose into the
//! target's conventions, and this decides what cannot be carried at all.
//!
//! All three cases here are one idea — the engine can see that a sentence is false about the thing
//! it documents. One names an API the crate does not have because a sibling refused; one names a
//! package that did not come along; one describes the source language itself. A doc comment that
//! lies is worse than none, because it is written in the voice of somebody who checked.
//!
//! The sentence is DROPPED, and the declaration survives. That was not the first answer: these
//! refused the whole declaration, which is what self-containment does everywhere else and is wrong
//! here. A body naming something absent does not compile; prose naming it is just false. And the
//! cost was not what a coverage number showed — it took out `Digest`, the central type of the most
//! complete package in the corpus, over one sentence reading `Digest implements hash.Hash64.`
//! while the two sentences after it were good documentation that survives the port intact.
//!
//! Dropping is prose surgery, and the engine already performs it: the opening rewrite strips the
//! source's leading identifier, the narration rewrite strips `is returned when`, and the type-name
//! rewrite replaces `uint64` with `u64`. This is the same category with a stronger warrant — those
//! change how a true sentence reads, and this removes a false one.

use std::collections::BTreeMap;

use crate::resolve::Resolver;

/// The doc block with every sentence that names something absent removed.
///
/// A sentence is the unit because a sentence is what carries the claim. Dropping the whole block
/// would lose documentation that is still true, and keeping it would emit a claim the crate does
/// not honour.
pub(crate) fn without_dangling_sentences(
    block: &str,
    subject: &str,
    resolver: &Resolver<'_>,
) -> String {
    let kept: Vec<&str> = sentences(block)
        .into_iter()
        .filter(|sentence| !names_something_absent(sentence, subject, resolver))
        .collect();
    kept.join(" ")
}

/// A block split into sentences, keeping each one whole.
///
/// A sentence ends at a period followed by whitespace or by the end of the text. Newlines inside a
/// block are the source's wrapping rather than boundaries, so they are treated as spaces — a doc
/// wrapped mid-claim is still one claim.
fn sentences(block: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = block.as_bytes();
    for (at, byte) in bytes.iter().enumerate() {
        let ends = *byte == b'.'
            && bytes
                .get(at + 1)
                .is_none_or(|next| next.is_ascii_whitespace());
        if ends {
            let sentence = block[start..=at].trim();
            if !sentence.is_empty() {
                out.push(sentence);
            }
            start = at + 1;
        }
    }
    let tail = block[start..].trim();
    if !tail.is_empty() {
        out.push(tail);
    }
    out
}

/// Whether this sentence names something the emitted crate does not contain.
fn names_something_absent(
    sentence: &str,
    subject: &str,
    resolver: &Resolver<'_>,
) -> bool {
    for word in sentence.split(|ch: char| !(ch.is_alphanumeric() || ch == '_' || ch == '.')) {
        // A trailing period is the sentence's, not the identifier's. Absorbing it made
        // `hash.Hash64.` the member name, which is nobody's identifier.
        let word = word.trim_end_matches('.');
        if names_the_source_language(word, resolver) {
            return true;
        }
        if let Some((package, _)) = qualified(word)
            && !resolver.units.contains(package)
        {
            return true;
        }
        if names_an_unemitted_declaration(word, subject, resolver) {
            return true;
        }
    }
    false
}

/// Whether this word names the SOURCE LANGUAGE or something only it has.
///
/// Safe to include the language's own name because the opening rewrite has already run: the
/// source's convention opens a doc with the identifier, so a package declaring a method called `Go`
/// has lost the word before this looks. The English verb is lower-case, which the list is not.
fn names_the_source_language(word: &str, resolver: &Resolver<'_>) -> bool {
    resolver
        .doc_convention
        .source_language_words
        .iter()
        .any(|listed| listed == word)
}

/// Whether this word names a declaration of THIS unit that is not being emitted.
///
/// EXPORTED only, which is the same bound the rename map's own construction uses and for the same
/// reason: an unexported source name is lower-case and indistinguishable from English. A
/// declaration's own name is never absent — the prose describes it, and it is the one being built.
/// A MEMBER is emitted exactly when its owner is, so that is what is asked about it.
fn names_an_unemitted_declaration(
    word: &str,
    subject: &str,
    resolver: &Resolver<'_>,
) -> bool {
    let exported = word.chars().next().is_some_and(char::is_uppercase);
    if !exported || word == subject || !resolver.scope.renames.contains_key(word) {
        return false;
    }
    let subject = resolver
        .scope
        .member_owners
        .get(word)
        .map_or(word, String::as_str);
    !resolver.emitted.contains(subject) && !resolver.scope.sentinels.contains_key(subject)
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
pub(crate) fn rename_types_in_text(text: &str, types: &BTreeMap<String, String>) -> String {
    crate::docs::rename_types_in_text(text, types)
}
