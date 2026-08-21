//! Pure decisions: label syntax, normalization, the bounded ASCII
//! confusable skeleton, and matching against the reservation list.
//!
//! Nothing here reads a clock, draws randomness, or performs I/O. Every
//! function is a total function of its arguments, so a decision can be
//! replayed from an audit record and reproduce byte-for-byte.

use std::collections::BTreeMap;

use crate::kernel::{MAX_LABEL_LEN, NamespaceAction, NamespaceUsecaseError};

/// Characters treated as label separators inside a candidate.
const CANDIDATE_SEPARATORS: [char; 2] = ['-', '_'];

/// The character that splits a reservation entry into namespace segments.
///
/// `oyatie.tenancy.lifecycle-controller` is three segments; its ROOT is
/// `oyatie`, and reserving the leaf therefore reserves the root, because a
/// namespace is owned at its root or it is not owned at all.
const RESERVED_SEGMENT_SEPARATOR: char = '.';

/// Why a candidate is not a syntactically legal label.
///
/// Carried rather than collapsed into a bare
/// [`crate::NamespaceDecision::DenyMalformed`] so the caller can tell a
/// tenant WHICH rule they broke without re-deriving it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MalformedReason {
    /// The candidate was empty, or whitespace only.
    Empty,
    /// The NORMALIZED form is shorter than
    /// [`NamespaceAction::min_label_len`]. `len` is the normalized length,
    /// not the raw one, so `a-b` reports 2 rather than 3 — the identity a
    /// caller would obtain is what the minimum protects.
    TooShort { len: usize, min: usize },
    /// Longer than [`MAX_LABEL_LEN`].
    TooLong { len: usize, max: usize },
    /// A character outside `[A-Za-z0-9_-]` at this byte offset.
    ///
    /// Non-ASCII input lands here. See the crate-level Gaps note: this
    /// blunt charset rule, not any Unicode awareness, is the only thing
    /// standing between a Cyrillic homograph and the reservation list.
    ForbiddenCharacter { character: char, at: usize },
    /// The label starts with `-` or `_`.
    LeadingSeparator,
    /// The label ends with `-` or `_`.
    TrailingSeparator,
    /// Two separators in a row, at this byte offset.
    ConsecutiveSeparators { at: usize },
}

impl core::fmt::Display for MalformedReason {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => f.write_str("label is empty"),
            Self::TooShort { len, min } => {
                write!(f, "label normalizes to {len} bytes; the minimum is {min}")
            }
            Self::TooLong { len, max } => {
                write!(f, "label is {len} bytes; the maximum is {max}")
            }
            Self::ForbiddenCharacter { character, at } => write!(
                f,
                "character {character:?} at byte {at} is outside the permitted set [A-Za-z0-9_-]"
            ),
            Self::LeadingSeparator => f.write_str("label starts with a separator"),
            Self::TrailingSeparator => f.write_str("label ends with a separator"),
            Self::ConsecutiveSeparators { at } => {
                write!(f, "two separators in a row at byte {at}")
            }
        }
    }
}

impl std::error::Error for MalformedReason {}

/// Whether `character` is a candidate separator.
#[must_use]
pub fn is_separator(character: char) -> bool {
    CANDIDATE_SEPARATORS.contains(&character)
}

/// Whether `character` may appear in a candidate label.
#[must_use]
pub fn is_permitted(character: char) -> bool {
    character.is_ascii_alphanumeric() || is_separator(character)
}

/// The syntactic rule for a namespace label.
///
/// A legal label is, in order of the checks applied:
///
/// 1. non-empty;
/// 2. at most [`MAX_LABEL_LEN`] RAW bytes;
/// 3. at least [`NamespaceAction::min_label_len`] NORMALIZED bytes;
/// 4. drawn only from `[A-Za-z0-9_-]` — ASCII, so a non-ASCII character is
///    a [`MalformedReason::ForbiddenCharacter`];
/// 5. not starting or ending with a separator;
/// 6. free of two adjacent separators.
///
/// Mixed case is legal here on purpose. `OYATIE` must be refused as
/// RESERVED, not as malformed, so case folding belongs to the comparison
/// stage and never to the syntax stage.
///
/// # Why the two length rules count different things
///
/// The maximum is a DNS-label rule, so it counts the raw string that has to
/// fit in a hostname. The minimum is a confusability rule, and every
/// identity comparison in this crate is done on the [`normalize`] form, so
/// it counts that form: otherwise `a-b` buys exactly the two-character
/// identity `ab` that the minimum exists to refuse, and the crate's own
/// doctrine that separator padding cannot change an outcome would hold
/// everywhere except here.
///
/// # Errors
///
/// Returns the first rule the candidate breaks, in the order above.
pub fn validate_syntax(candidate: &str, action: NamespaceAction) -> Result<(), MalformedReason> {
    if candidate.trim().is_empty() {
        return Err(MalformedReason::Empty);
    }
    let len = candidate.len();
    if len > MAX_LABEL_LEN {
        return Err(MalformedReason::TooLong {
            len,
            max: MAX_LABEL_LEN,
        });
    }
    let min = action.min_label_len();
    let identity_len = normalize(candidate).len();
    if identity_len < min {
        return Err(MalformedReason::TooShort {
            len: identity_len,
            min,
        });
    }

    let mut previous_was_separator = false;
    let mut first = true;
    for (at, character) in candidate.char_indices() {
        if !is_permitted(character) {
            return Err(MalformedReason::ForbiddenCharacter { character, at });
        }
        let separator = is_separator(character);
        if separator && first {
            return Err(MalformedReason::LeadingSeparator);
        }
        if separator && previous_was_separator {
            return Err(MalformedReason::ConsecutiveSeparators { at });
        }
        previous_was_separator = separator;
        first = false;
    }
    if previous_was_separator {
        return Err(MalformedReason::TrailingSeparator);
    }
    Ok(())
}

/// Case-fold and drop every separator, including the reservation-entry
/// segment separator `.`.
///
/// This is the *identity* form: `O-Y-A-T-I-E` and `oyatie` normalize to the
/// same token, which is why the guard is not a string equality check.
/// Non-ASCII characters pass through untouched — see the crate Gaps note.
#[must_use]
pub fn normalize(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter(|character| !is_separator(*character) && *character != RESERVED_SEGMENT_SEPARATOR)
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

/// Whether `character` ends a segment of a candidate or of a reservation
/// entry.
///
/// The candidate separators plus [`RESERVED_SEGMENT_SEPARATOR`]. These are
/// exactly the characters [`normalize`] removes, which is the point: a
/// boundary is a place where the normalized form loses information, so it
/// is the only place a prefix of the normalized form may legitimately be
/// claimed to end.
#[must_use]
pub fn is_boundary(character: char) -> bool {
    is_separator(character) || character == RESERVED_SEGMENT_SEPARATOR
}

/// Every normalized prefix of `candidate` that ends on a segment boundary,
/// shortest first, with the whole normalized candidate last.
///
/// `o-yatie-support` yields `["o", "oyatie", "oyatiesupport"]`;
/// `oyatier-customer` yields `["oyatier", "oyatiercustomer"]`. That
/// difference is the whole namespace-root rule: `oyatie` is a prefix of
/// both normalized forms, but only in the first does it END where a
/// separator stood, so only the first is claiming a child label OF the
/// `oyatie` namespace. The second is claiming a different root that merely
/// starts alike.
///
/// Computing this on the candidate — rather than splitting the candidate on
/// separators and comparing single segments — is what stops separator
/// padding from escaping a reservation: `o-yatie-support`, `oyati-e-support`
/// and `oyatie-support` all yield `oyatie` as a boundary prefix even though
/// their FIRST segments are `o`, `oyati` and `oyatie`.
///
/// Empty segments (a leading separator, or a run of them) contribute
/// nothing, and consecutive boundaries never yield the same prefix twice.
#[must_use]
pub fn boundary_prefixes(candidate: &str) -> Vec<String> {
    let mut prefixes: Vec<String> = Vec::new();
    let mut current = String::new();
    let push = |current: &String, prefixes: &mut Vec<String>| {
        if !current.is_empty() && prefixes.last() != Some(current) {
            prefixes.push(current.clone());
        }
    };
    for character in candidate.trim().chars() {
        if is_boundary(character) {
            push(&current, &mut prefixes);
        } else {
            current.push(character.to_ascii_lowercase());
        }
    }
    push(&current, &mut prefixes);
    prefixes
}

/// The root label a candidate claims: its first non-empty boundary-delimited
/// segment, normalized.
///
/// Kept as the published name it has always been, but note that it is NOT
/// the rule the guard matches on — see [`boundary_prefixes`], of which this
/// is only the first element. Matching on the first segment alone is exactly
/// the hole that lets `o-yatie-support` walk past a reservation on `oyatie`.
#[must_use]
pub fn candidate_root(candidate: &str) -> String {
    boundary_prefixes(candidate)
        .into_iter()
        .next()
        .unwrap_or_default()
}

/// The root segment a reservation entry owns: everything before its first `.`.
#[must_use]
pub fn reserved_root(entry: &str) -> String {
    let trimmed = entry.trim();
    let head = trimmed
        .split(RESERVED_SEGMENT_SEPARATOR)
        .next()
        .unwrap_or(trimmed);
    normalize(head)
}

/// Fold one character onto its visual class representative.
///
/// The table is deliberately small, ASCII-only and fixed:
///
/// | from | to | | from | to |
/// |---|---|---|---|---|
/// | `0` | `o` | | `5` | `s` |
/// | `1` | `l` | | `6` | `b` |
/// | `i` | `l` | | `7` | `t` |
/// | `2` | `z` | | `8` | `b` |
/// | `3` | `e` | | `9` | `g` |
/// | `4` | `a` | | | |
///
/// Many-to-one is intended: `1`, `i` and `l` all become `l`, so any mixture
/// of them collapses to one token.
#[must_use]
pub fn fold_character(character: char) -> char {
    match character {
        '0' => 'o',
        '1' | 'i' => 'l',
        '2' => 'z',
        '3' => 'e',
        '4' => 'a',
        '5' => 's',
        '6' | '8' => 'b',
        '7' => 't',
        '9' => 'g',
        other => other,
    }
}

/// Fold a two-character visual sequence onto the single character it
/// imitates: `rn`→`m`, `vv`→`w`, `cl`→`d`.
///
/// Applied AFTER [`fold_character`], so `c1` and `ci` fold to `d` as well —
/// both have already become `cl`.
#[must_use]
pub fn fold_digraph(first: char, second: char) -> Option<char> {
    match (first, second) {
        ('r', 'n') => Some('m'),
        ('v', 'v') => Some('w'),
        ('c', 'l') => Some('d'),
        _ => None,
    }
}

/// The bounded ASCII confusable skeleton, in four documented passes:
///
/// 1. ASCII case fold;
/// 2. drop separators (`-`, `_`, `.`), so `o-y-a-t-i-e` and `oyatie` agree;
/// 3. per-character folding via [`fold_character`];
/// 4. left-to-right digraph folding via [`fold_digraph`].
///
/// Two names with equal skeletons are treated as visually confusable.
///
/// # What this does NOT do
///
/// It implements none of UTS #39. There is no Unicode confusables table, no
/// script-mixing detection, no NFKC width or compatibility folding, and no
/// collapsing of repeated characters (`ooyatie` keeps both `o`s). A
/// non-ASCII homograph is not caught HERE at all; it is refused one stage
/// earlier, by [`validate_syntax`]'s ASCII-only charset rule, and that is
/// the only defense the crate has against it.
#[must_use]
pub fn skeleton(input: &str) -> String {
    let folded: Vec<char> = normalize(input).chars().map(fold_character).collect();
    let mut out = String::with_capacity(folded.len());
    let mut index = 0;
    while let Some(&character) = folded.get(index) {
        if let Some(&next) = folded.get(index + 1)
            && let Some(merged) = fold_digraph(character, next)
        {
            out.push(merged);
            index += 2;
            continue;
        }
        out.push(character);
        index += 1;
    }
    out
}

/// FNV-1a offset basis (64-bit).
pub const FNV_OFFSET_BASIS_64: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a prime (64-bit).
pub const FNV_PRIME_64: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a over the bytes of `input`.
///
/// Used as the audit correlation digest so a refusal event can be joined
/// across services without republishing the raw candidate. It is NOT a
/// cryptographic commitment: FNV is fast and short slugs are trivially
/// brute-forced, so the digest hides the candidate from a casual reader of
/// the log and from nobody else.
#[must_use]
pub fn fnv1a_64(input: &str) -> u64 {
    let mut hash = FNV_OFFSET_BASIS_64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME_64);
    }
    hash
}

/// The reservation list, indexed in the four forms the guard compares
/// against, each mapped back to the entry that produced it for audit.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReservedSet {
    exact: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    roots: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    exact_skeletons: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    root_skeletons: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

impl ReservedSet {
    /// Index a reservation list.
    ///
    /// # Errors
    ///
    /// - [`NamespaceUsecaseError::EmptyReservationList`] if `entries` is
    ///   empty. An empty list would permit the platform owner's own name,
    ///   which ADR-0242 forbids, so it is read as an unresolved binding.
    /// - [`NamespaceUsecaseError::MalformedReservationEntry`] if an entry
    ///   normalizes to nothing. Refused rather than skipped.
    pub fn build(entries: &[String]) -> Result<Self, NamespaceUsecaseError> {
        if entries.is_empty() {
            return Err(NamespaceUsecaseError::EmptyReservationList);
        }
        let mut set = Self::default();
        for entry in entries {
            let normalized = normalize(entry);
            let root = reserved_root(entry);
            if normalized.is_empty() || root.is_empty() {
                return Err(NamespaceUsecaseError::MalformedReservationEntry {
                    entry: entry.clone(),
                });
            }
            set.exact_skeletons
                .entry(skeleton(&normalized))
                .or_insert_with(|| entry.clone());
            set.root_skeletons
                .entry(skeleton(&root))
                .or_insert_with(|| entry.clone());
            set.exact.entry(normalized).or_insert_with(|| entry.clone());
            set.roots.entry(root).or_insert_with(|| entry.clone());
        }
        Ok(set)
    }

    /// How many distinct normalized tokens are indexed.
    #[must_use]
    pub fn len(&self) -> usize {
        self.exact.len()
    }

    /// Whether the index holds no tokens. Never true for a set returned by
    /// [`ReservedSet::build`], which refuses to build an empty one.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.exact.is_empty()
    }

    /// The entry this candidate collides with outright, if any.
    ///
    /// One rule, applied to each of the candidate's [`boundary_prefixes`]
    /// shortest first: if a prefix equals an indexed token — either a whole
    /// reservation entry or the root segment of one — the candidate is
    /// claiming that namespace.
    ///
    /// It subsumes both of the rules this crate used to state separately,
    /// and closes the gap between them:
    ///
    /// - **identity** — the whole normalized candidate is the last prefix,
    ///   so `OYATIE`, `Oyatie` and `o-y-a-t-i-e` all hit `oyatie`;
    /// - **namespace root** — `oyatie-support` hits, and so do
    ///   `o-yatie-support` and `oyati-e-support`, because normalization
    ///   strips the padding BEFORE the boundary is looked for. Comparing
    ///   only the first separator-delimited segment (`o`, `oyati`) missed
    ///   every one of those.
    ///
    /// `oyatier-customer` still does not hit: `oyatie` is a prefix of
    /// `oyatiercustomer`, but it does not end on a boundary, so it is a
    /// different root rather than a child label.
    ///
    /// Shortest prefix first, so the entry reported is the outermost
    /// namespace the candidate intruded on rather than the most specific
    /// sub-entry.
    #[must_use]
    pub fn reserved_hit(&self, candidate: &str) -> Option<&str> {
        for prefix in boundary_prefixes(candidate) {
            if let Some(entry) = self.exact.get(&prefix) {
                return Some(entry.as_str());
            }
            if let Some(entry) = self.roots.get(&prefix) {
                return Some(entry.as_str());
            }
        }
        None
    }

    /// The entry this candidate is visually confusable with, if any — the
    /// same boundary-prefix rule as [`ReservedSet::reserved_hit`], compared
    /// on [`skeleton`]s instead of on normalized tokens.
    #[must_use]
    pub fn confusable_hit(&self, candidate: &str) -> Option<&str> {
        for prefix in boundary_prefixes(candidate) {
            let prefix_skeleton = skeleton(&prefix);
            if let Some(entry) = self.exact_skeletons.get(&prefix_skeleton) {
                return Some(entry.as_str());
            }
            if let Some(entry) = self.root_skeletons.get(&prefix_skeleton) {
                return Some(entry.as_str());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner_set() -> ReservedSet {
        ReservedSet::build(&[
            "oyatie".to_owned(),
            "oyatie.tenancy.lifecycle-controller".to_owned(),
            "admin-console".to_owned(),
        ])
        .expect("fixture reservation list is well formed")
    }

    #[test]
    fn syntax_accepts_a_plain_slug_and_mixed_case() {
        assert!(validate_syntax("acme", NamespaceAction::CreateTenant).is_ok());
        assert!(validate_syntax("Acme-Corp_7", NamespaceAction::CreateTenant).is_ok());
    }

    #[test]
    fn syntax_names_the_rule_that_was_broken() {
        assert_eq!(
            validate_syntax("", NamespaceAction::CreateTenant),
            Err(MalformedReason::Empty)
        );
        assert_eq!(
            validate_syntax("   ", NamespaceAction::CreateTenant),
            Err(MalformedReason::Empty)
        );
        assert_eq!(
            validate_syntax("ab", NamespaceAction::CreateTenant),
            Err(MalformedReason::TooShort { len: 2, min: 3 })
        );
        assert_eq!(
            validate_syntax(&"a".repeat(64), NamespaceAction::CreateTenant),
            Err(MalformedReason::TooLong {
                len: 64,
                max: MAX_LABEL_LEN,
            })
        );
        assert_eq!(
            validate_syntax("-acme", NamespaceAction::CreateTenant),
            Err(MalformedReason::LeadingSeparator)
        );
        assert_eq!(
            validate_syntax("acme-", NamespaceAction::CreateTenant),
            Err(MalformedReason::TrailingSeparator)
        );
        assert_eq!(
            validate_syntax("ac--me", NamespaceAction::CreateTenant),
            Err(MalformedReason::ConsecutiveSeparators { at: 3 })
        );
        assert_eq!(
            validate_syntax("ac me", NamespaceAction::CreateTenant),
            Err(MalformedReason::ForbiddenCharacter {
                character: ' ',
                at: 2,
            })
        );
    }

    #[test]
    fn syntax_boundary_is_inclusive_at_both_ends() {
        assert!(validate_syntax("abc", NamespaceAction::CreateTenant).is_ok());
        assert!(validate_syntax(&"a".repeat(MAX_LABEL_LEN), NamespaceAction::CreateTenant).is_ok());
        assert!(validate_syntax("ab", NamespaceAction::CreateSubScope).is_ok());
        assert_eq!(
            validate_syntax("a", NamespaceAction::CreateSubScope),
            Err(MalformedReason::TooShort { len: 1, min: 2 })
        );
    }

    #[test]
    fn non_ascii_is_refused_by_the_charset_rule() {
        // Cyrillic small a (U+0430) as the first character.
        let reason = validate_syntax("\u{0430}cme", NamespaceAction::CreateTenant)
            .expect_err("non-ASCII is outside the permitted set");
        assert!(matches!(
            reason,
            MalformedReason::ForbiddenCharacter { at: 0, .. }
        ));
    }

    #[test]
    fn normalization_folds_case_and_separators() {
        assert_eq!(normalize("O-Y_A-T.I-E"), "oyatie");
        assert_eq!(normalize("  Acme-Corp  "), "acmecorp");
    }

    #[test]
    fn skeleton_folds_the_documented_substitutions() {
        assert_eq!(skeleton("oyatie"), skeleton("0yatie"));
        assert_eq!(skeleton("oyatie"), skeleton("0y4t1e"));
        assert_eq!(skeleton("rn"), skeleton("m"));
        assert_eq!(skeleton("vvide"), skeleton("wide"));
        assert_eq!(skeleton("clay"), skeleton("day"));
        assert_eq!(skeleton("5ales"), skeleton("sales"));
    }

    #[test]
    fn skeleton_does_not_collapse_unrelated_names() {
        assert_ne!(skeleton("oyatie"), skeleton("oyatier"));
        assert_ne!(skeleton("oyatie"), skeleton("ooyatie"));
        assert_ne!(skeleton("acme"), skeleton("oyatie"));
    }

    #[test]
    fn reserved_set_refuses_an_empty_or_blank_list() {
        assert_eq!(
            ReservedSet::build(&[]),
            Err(NamespaceUsecaseError::EmptyReservationList)
        );
        assert_eq!(
            ReservedSet::build(&["  ".to_owned()]),
            Err(NamespaceUsecaseError::MalformedReservationEntry {
                entry: "  ".to_owned(),
            })
        );
        assert_eq!(
            ReservedSet::build(&["-_-".to_owned()]),
            Err(NamespaceUsecaseError::MalformedReservationEntry {
                entry: "-_-".to_owned(),
            })
        );
    }

    #[test]
    fn reserved_hit_is_case_and_separator_insensitive() {
        let set = owner_set();
        for candidate in ["oyatie", "Oyatie", "OYATIE", "o-y-a-t-i-e", "O_Yatie"] {
            assert_eq!(
                set.reserved_hit(candidate),
                Some("oyatie"),
                "{candidate} must hit the owner reservation"
            );
        }
    }

    #[test]
    fn reserved_hit_covers_the_namespace_root_and_the_flattened_principal() {
        let set = owner_set();
        assert_eq!(set.reserved_hit("oyatie-support"), Some("oyatie"));
        assert_eq!(
            set.reserved_hit("oyatietenancylifecyclecontroller"),
            Some("oyatie.tenancy.lifecycle-controller")
        );
        // A dotted entry with no bare root token still reserves its root.
        let dotted = ReservedSet::build(&["acme.internal.bot".to_owned()])
            .expect("dotted entry is well formed");
        assert_eq!(dotted.reserved_hit("acme-shop"), Some("acme.internal.bot"));
    }

    #[test]
    fn reserved_hit_does_not_over_reach_past_the_root_label() {
        let set = owner_set();
        assert_eq!(set.reserved_hit("oyatier-customer"), None);
        assert_eq!(set.reserved_hit("acme"), None);
        // `admin-console` has no dot, so it reserves itself and not `admin`.
        assert_eq!(set.reserved_hit("admin"), None);
        assert_eq!(set.reserved_hit("admin-console"), Some("admin-console"));
    }

    #[test]
    fn confusable_hit_catches_digit_substitutions_and_digraphs() {
        let set = owner_set();
        assert_eq!(set.confusable_hit("0yatie"), Some("oyatie"));
        assert_eq!(set.confusable_hit("0y4t13"), Some("oyatie"));
        assert_eq!(set.confusable_hit("0yatie-support"), Some("oyatie"));
        assert_eq!(set.confusable_hit("adrnin-console"), Some("admin-console"));
        assert_eq!(set.confusable_hit("acme"), None);
    }

    #[test]
    fn fnv_digest_is_deterministic_and_input_sensitive() {
        assert_eq!(fnv1a_64("oyatie"), fnv1a_64("oyatie"));
        assert_ne!(fnv1a_64("oyatie"), fnv1a_64("oyatif"));
        assert_eq!(fnv1a_64(""), FNV_OFFSET_BASIS_64);
    }

    #[test]
    fn boundary_prefixes_are_shortest_first_and_drop_empty_segments() {
        assert_eq!(
            boundary_prefixes("o-yatie-support"),
            vec!["o", "oyatie", "oyatiesupport"]
        );
        assert_eq!(
            boundary_prefixes("oyatier-customer"),
            vec!["oyatier", "oyatiercustomer"]
        );
        assert_eq!(boundary_prefixes("OYATIE"), vec!["oyatie"]);
        // A run of separators contributes one boundary, not three.
        assert_eq!(boundary_prefixes("--a--b--"), vec!["a", "ab"]);
        assert!(boundary_prefixes("  ").is_empty());
        assert_eq!(candidate_root("o-yatie-support"), "o");
        // The last prefix is always the whole normalized candidate.
        assert_eq!(
            boundary_prefixes("O_Y-A.T-I_E").last().map(String::as_str),
            Some(normalize("O_Y-A.T-I_E").as_str())
        );
    }

    #[test]
    fn a_separator_inside_the_owner_token_does_not_escape_the_root_rule() {
        let set = owner_set();
        // Each of these carries a separator INSIDE `oyatie` and a trailing
        // child label, so neither the whole normalized form nor the first
        // segment equals a reserved token. The boundary-prefix rule is what
        // catches them.
        for candidate in [
            "oyatie-support",
            "o-yatie-support",
            "oyati-e-support",
            "oyat-ie-billing",
            "oy_atie-admin",
            "o-y-a-t-i-e-support",
        ] {
            assert_eq!(
                set.reserved_hit(candidate),
                Some("oyatie"),
                "{candidate} claims a child label of the owner namespace"
            );
        }
    }

    #[test]
    fn a_hyphenated_reservation_reserves_its_own_children_either_way_it_is_spelled() {
        let set = owner_set();
        // `admin-console` is one reserved entry whose token contains a
        // separator. Both spellings of a child label of it must agree.
        assert_eq!(
            set.reserved_hit("admin-console-eu"),
            Some("admin-console"),
            "a child of a hyphenated reservation is reserved"
        );
        assert_eq!(
            set.reserved_hit("adminconsole-eu"),
            set.reserved_hit("admin-console-eu"),
            "the two spellings of the same identity must not disagree"
        );
        assert_eq!(
            set.reserved_hit("admin-console-oyatie"),
            Some("admin-console")
        );
        // Still no over-reach onto the bare first segment.
        assert_eq!(set.reserved_hit("admin"), None);
        assert_eq!(set.reserved_hit("admin-portal"), None);
    }

    #[test]
    fn the_confusable_rule_follows_the_same_boundaries() {
        let set = owner_set();
        assert_eq!(set.confusable_hit("0-yatie-support"), Some("oyatie"));
        assert_eq!(set.confusable_hit("0yati-e-ops"), Some("oyatie"));
        assert_eq!(
            set.confusable_hit("adrnin-console-eu"),
            Some("admin-console")
        );
        // A different root that merely starts alike is still not a hit.
        assert_eq!(set.confusable_hit("0yatier-customer"), None);
    }

    #[test]
    fn the_minimum_length_is_counted_after_normalization() {
        assert_eq!(
            validate_syntax("a-b", NamespaceAction::CreateTenant),
            Err(MalformedReason::TooShort { len: 2, min: 3 })
        );
        assert_eq!(
            validate_syntax("a_b", NamespaceAction::CreateTenant),
            Err(MalformedReason::TooShort { len: 2, min: 3 })
        );
        assert_eq!(
            validate_syntax("1-2", NamespaceAction::CreateTenant),
            Err(MalformedReason::TooShort { len: 2, min: 3 })
        );
        assert_eq!(
            validate_syntax("a-b-c", NamespaceAction::CreateTenant),
            Ok(())
        );
        // A sub-scope alias only needs two, so `a-b` is legal there.
        assert_eq!(
            validate_syntax("a-b", NamespaceAction::CreateSubScope),
            Ok(())
        );
        assert_eq!(
            validate_syntax("a-", NamespaceAction::CreateSubScope),
            Err(MalformedReason::TooShort { len: 1, min: 2 })
        );
        // The maximum still counts raw bytes, because that is what has to
        // fit in a DNS label.
        assert_eq!(
            validate_syntax(&"a-".repeat(32), NamespaceAction::CreateTenant),
            Err(MalformedReason::TooLong {
                len: 64,
                max: MAX_LABEL_LEN,
            })
        );
    }
}
