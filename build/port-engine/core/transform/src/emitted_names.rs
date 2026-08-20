//! Rules that ask the OUTPUT, not the model that produced it.
//!
//! Two of them so far, and they share a premise the model cannot supply: whether the unit actually
//! emitted something that needs a name. The source can say a unit is fallible and every fallible
//! function in it can still refuse, and it can say a sentinel is declared when the sentinel did not
//! survive — so a rule reading the declarations adds an import nothing uses and an alias nothing
//! refers to. Reading the emitted items instead is the only evidence that cannot be wrong about
//! what was emitted.
//!
//! Both fail SAFE, and in the same direction: where the output cannot be inspected they keep the
//! name. A name kept needlessly is dead code; a name dropped wrongly is a unit that does not build.

use std::collections::{BTreeMap, BTreeSet};

use port_engine_rust_ir::{RustItem, RustType, Visibility};

/// The imports a set of EMITTED items needs.
///
/// Pure, and asked of the items themselves, so both emission paths get the same answer from the
/// same evidence. Asked of the output rather than of the declarations because an import nothing
/// uses is a denied warning, where an unused alias is only dead code — a unit whose sentinels all
/// refused must not gain an import for them.
pub(crate) fn import_items(items: &[RustItem], declared: &BTreeMap<String, String>) -> Vec<RustItem> {
    let mut paths: BTreeSet<String> = BTreeSet::new();
    // The MESSAGE IMPL renders the same three names the sentinel form does, for the same reason:
    // it IS a display impl plus the error impl that follows from it. Adding it to the same question
    // rather than a second one is what keeps the import and the form derived from one fact.
    let has_sentinel = items.iter().any(|item| {
        matches!(
            item,
            RustItem::SentinelError { .. }
                | RustItem::SentinelEnum { .. }
                | RustItem::MessageImpl { .. }
        )
    });
    if has_sentinel {
        // The sentinel form spells `fmt::Display`, `fmt::Formatter` and `fmt::Result`, so a unit
        // with seven sentinels names one std module twenty-one times. The short form and this
        // import are one decision, derived from one fact, and cannot drift apart.
        paths.insert("std::fmt".to_owned());
    }

    // What the unit's emitted TYPES actually name. Asked of the types rather than of the rendered
    // text, because a type is a tree and a text scan would match a name inside a longer one — and
    // an import nothing uses is a denied warning, so a false positive is a build failure.
    let mut named: BTreeSet<String> = BTreeSet::new();
    for item in items {
        named.extend(item.type_spellings());
    }
    for (short, path) in declared {
        // The sentinel form NAMES the error trait in its own impl, which no type field carries. A
        // unit that emits one needs that import whatever its types say.
        let by_sentinel = has_sentinel && path.ends_with("::Error");
        if by_sentinel || named.iter().any(|spelling| names(spelling, short)) {
            paths.insert(match path.rsplit("::").next() == Some(short.as_str()) {
                true => path.clone(),
                // A RENAME, because the short form and the path's own last segment differ — which
                // is how the error trait is imported beside a unit that declares its own `Error`.
                false => format!("{path} as {short}"),
            });
        }
    }
    paths
        .into_iter()
        .map(|path| RustItem::Use { path })
        .collect()
}

/// Whether a type spelling NAMES this short form, as a whole identifier rather than as a substring.
///
/// `MyOrdering` does not name `Ordering`, and treating it as though it did would emit an import
/// nothing uses — which the compile proof denies.
fn names(spelling: &str, short: &str) -> bool {
    spelling
        .match_indices(short)
        .any(|(at, _)| {
            let before = spelling[..at].chars().next_back();
            let after = spelling[at + short.len()..].chars().next();
            let boundary = |ch: Option<char>| {
                ch.is_none_or(|c| !c.is_alphanumeric() && c != '_')
            };
            boundary(before) && boundary(after)
        })
}


/// The prelude items the unit's OWN OUTPUT refers to, and no others.
///
/// A prelude name is the engine's INTRODUCTION, not the source's declaration — nothing upstream
/// asked for it and no caller of the source can be relying on it. That is what makes dropping an
/// unused one safe, and it is exactly the line the reverse rule crosses: an unreferenced
/// declaration the SOURCE wrote still means something, and in a partial port it is unreferenced
/// mostly because whatever would have referred to it refused. The engine may withdraw its own
/// offer; it may not delete the author's work for going unused.
///
/// Asked of the emitted items for the same reason [`import_items`] is: `unit_can_fail` reads the
/// SOURCE, and a unit whose every fallible function refused still answers yes. A reviewer of that
/// output named the two aliases as a design baked in before anything needed it, and read them as
/// the mechanical rendering of a language where every function returns an error.
///
/// Fails SAFE. A prelude that cannot be rendered for inspection is kept, because an alias nobody
/// uses is dead code and an alias wrongly dropped is a unit that does not compile.
pub(crate) fn retain_used(prelude: Vec<RustItem>, rest: &[RustItem]) -> Vec<RustItem> {
    let Some(text) = port_engine_rust_ir::rendered_text(rest) else {
        return prelude;
    };
    // A QUALIFIED name is a different name. `fmt::Result` is the formatter's, not this alias, and
    // counting it kept the alias alive in three packages that never used it — the alias is
    // introduced UNQUALIFIED and can only be referred to that way inside its own module.
    let mentioned = |name: &str| {
        text.match_indices(name).any(|(at, _)| {
            let before = text[..at].chars().next_back();
            let after = text[at + name.len()..].chars().next();
            !before.is_some_and(|ch| ch.is_alphanumeric() || ch == '_' || ch == ':')
                && !after.is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
        })
    };
    // In DECLARATION order, so an alias kept only because a later one names it is decided after
    // that one: the result alias mentions the boxed alias in its own default, and asking about the
    // boxed alias against the emitted items alone would drop the name its neighbour spells.
    let mut kept: Vec<RustItem> = Vec::new();
    for item in prelude.into_iter().rev() {
        let name = match &item {
            RustItem::TypeAlias { name, .. } => name.clone(),
            _ => {
                kept.push(item);
                continue;
            }
        };
        let named_by_kept = kept.iter().any(|other| match other {
            RustItem::TypeAlias { generics, ty, .. } => {
                generics.iter().any(|generic| generic.contains(&name)) || ty.spelling().contains(&name)
            }
            _ => false,
        });
        if mentioned(&name) || named_by_kept {
            kept.push(item);
        }
    }
    kept.reverse();
    kept
}
