//! What documentation may NOT say about the code it documents.
//!
//! Split from `docs.rs` because the two are different jobs: that file REWRITES prose into the
//! target's conventions, and this decides when prose cannot be carried at all.
//!
//! Both refusals here are the same idea at the prose layer — the engine can see that a sentence is
//! false about the thing it documents, and it cannot write a true one. One names an API the crate
//! does not have; the other describes a program that was not ported. A doc comment that lies is
//! worse than none, because it is written in the voice of somebody who checked.

use port_engine_api::Declaration;

use crate::error::TransformError;
use crate::resolve::Resolver;

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
pub(crate) fn refuse_dangling_reference(
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

/// Refuse prose that documents the SOURCE LANGUAGE rather than the emitted code.
///
/// A doc naming the source language, its runtime, or something only it has is documentation about a
/// program that was not ported. `The consts are used when possible in Go code to avoid MOVs but we
/// need a contiguous array for the assembly code.` reached rustdoc on a crate that denies `unsafe`
/// and has no assembly; a reviewer called it conclusive on its own, and it was.
///
/// Checked on the REWRITTEN prose, and that is what makes the language's own name safe to look for.
/// The source's convention opens a doc with the identifier, so a package declaring a method called
/// `Go` has already lost the word by the time this runs — and the English verb is lower-case, which
/// the list is not.
///
/// The declaration refuses rather than losing the sentence, for the reason prose naming an absent
/// API refuses: the engine can see the documentation is false about what it documents, and it
/// cannot write a true one.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the declaration and the word that gave it away.
pub(crate) fn refuse_source_language_prose(
    lines: &[String],
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<(), TransformError> {
    let words = &resolver.doc_convention.source_language_words;
    if words.is_empty() {
        return Ok(());
    }
    for line in lines {
        for word in line.split(|ch: char| !(ch.is_alphanumeric() || ch == '_')) {
            if words.iter().any(|listed| listed == word) {
                return Err(TransformError::Unsupported {
                    name: declaration.name.clone(),
                    detail: format!(
                        "its documentation says `{word}`, which names the source language or \
                         something only it has — the prose describes a program that was not \
                         ported, and the engine can see it is false about what it documents \
                         without being able to write a true one"
                    ),
                });
            }
        }
    }
    Ok(())
}
