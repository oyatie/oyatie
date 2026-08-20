//! Apex gist integrity kernel — PURE, no filesystem, no process, no clock.
//!
//! A live apex ADR SUPERSEDES a set of archived member ADRs and is supposed to carry their
//! ratified substance forward. The apexes in this repository were produced by a mechanical
//! consolidation that cut every carried-forward block at a fixed CHARACTER BUDGET and cut the
//! member list itself at a fixed COUNT. Both cuts are silent: the result is well-formed Markdown
//! with no ellipsis, no marker and no pointer, so a reader cannot distinguish a gist that was
//! TRUNCATED from one that was deliberately TERSE. That is the defect — the apex asserts less than
//! was ratified while READING as complete.
//!
//! THE ORACLE IS THE ARCHIVED SOURCE; SHAPE IS THE FALLBACK. A carried block was cut if and only
//! if the member it came from says so, and the member is on disk. Every shape predicate below is a
//! cheap secondary for the blocks whose member cannot be read or compared, and each one
//! UNDER-detects on its own — a cut that lands on a comma looks exactly like a finished sentence,
//! and a title cut inside its own budget usually still balances its parentheses.
//!
//! Seven independent shapes of that defect are mechanically decidable without re-deriving the
//! generator, and this kernel decides exactly those:
//!
//! 1. `CUT_FROM_SOURCE` — the SOURCE comparison, and the only one that decides a cut rather than
//!    guessing at one. Whitespace-normalized, the carried text occurs verbatim inside the member's
//!    own body and is NOT the end of it: the carry is a PROPER PREFIX with ratified text still
//!    following. Containment simply fails on a paraphrase or a rename, so this cannot false-positive
//!    on either; what it cannot see is a block that was reworded rather than sliced.
//! 2. `TRUNCATED_MIDWORD` — a carried block whose last character is alphanumeric or a hyphen.
//!    Prose cut at a byte budget often ends inside a word: `a hosted-contr`, `Kam`. The converse
//!    does NOT hold and this predicate must not be read as if it did — `ADR-0701:54` ends on
//!    `PRD-frontmatter field,`, which is punctuation and is still a cut. Reported only for blocks
//!    the source comparison did not already claim, so the two counts stay disjoint and additive.
//! 3. `UNCLOSED_FENCE` — a carried block containing an ODD number of ``` runs. This one is not
//!    merely lossy, it is CORRUPTING: an unbalanced fence swallows every following block into a
//!    code span, so one truncated member silently changes how the rest of the apex renders.
//! 4. `MEMBER_WITHOUT_GIST` / `MEMBER_WITHOUT_RESIDUAL` — an id named in the apex `supersedes:`
//!    list that the apex body never carries at all. A budgeted member list drops its TAIL, and the
//!    tail of an ascending ADR list is the NEWEST law — the decisions most likely to still be live.
//! 5. `TITLE_UNRESOLVED` — a bullet whose `(title)` lead-in matches NONE of the spellings the
//!    member itself offers (filename stem, frontmatter `title:`, H1 heading). Fail-closed, and it
//!    is the title check that actually covers the population: the generator emits its OWN closing
//!    `)`, so a cut title is BALANCED by default and `UNBALANCED_TITLE` only fires by accident.
//! 6. `UNBALANCED_TITLE` — a bullet whose `(title)` lead-in never closes, because the cut landed
//!    inside a parenthesised clause. Kept as a distinct and WORSE defect than an unresolved title:
//!    it destroys the delimiter that separates the title from the ratified substance, so no two
//!    consumers agree on where the substance starts.
//! 7. `TOPIC_DROPPED` — a topic that a superseded member demonstrably carries text about and that
//!    the superseding apex never mentions. This is the cross-check that catches loss the others
//!    cannot see: a member can be dropped wholesale, or truncated before it ever reaches its
//!    subject, and either way the apex silently stops being law on that subject.
//!
//! The kernel is generic over id STRINGS so its unit fixtures can use tokens with no governed
//! shape. Writing a realistic `ADR-NNNN` id into a `.rs` fixture would be read as a real citation
//! by the sibling citation-closure gate, which scans `.rs`.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

pub const CODE_CUT_FROM_SOURCE: &str = "apex_gist_cut_from_source";
pub const CODE_TRUNCATED_MIDWORD: &str = "apex_gist_truncated_midword";
pub const CODE_UNCLOSED_FENCE: &str = "apex_gist_unclosed_fence";
pub const CODE_TITLE_UNRESOLVED: &str = "apex_gist_title_unresolved";
pub const CODE_UNBALANCED_TITLE: &str = "apex_gist_unbalanced_title";
pub const CODE_MEMBER_WITHOUT_GIST: &str = "apex_member_without_gist";
pub const CODE_MEMBER_WITHOUT_RESIDUAL: &str = "apex_member_without_residual";
pub const CODE_TOPIC_DROPPED: &str = "apex_topic_dropped";
pub const CODE_VACUOUS_SCAN: &str = "apex_scan_vacuous";

/// Where a carried block sits in the apex. Both sites truncate, at different budgets, and the
/// distinction is worth keeping in the finding: a missing gist still leaves a residual to read,
/// while a truncated residual is the end of the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Site {
    Gist,
    Residual,
}

impl Site {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Site::Gist => "gist",
            Site::Residual => "residual",
        }
    }
}

/// One block of member substance carried into an apex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub member_id: String, // data_class: INTERNAL_ONLY
    pub site: Site,        // data_class: INTERNAL_ONLY
    /// Line number in the apex file, for a finding a human can navigate to.
    pub line: usize, // data_class: INTERNAL_ONLY
    /// The carried text with its `- **ADR-N** (title):` / `**title** —` lead-in already stripped.
    pub text: String, // data_class: INTERNAL_ONLY
    /// The `(title)` lead-in's CONTENT, as carried. Empty for a site that has no title lead-in.
    ///
    /// Kept rather than discarded because the title has its OWN, shorter budget than the body and
    /// is cut independently of it: a bullet can carry a complete body under a title that lost its
    /// last forty characters. Before this field existed the balanced-parenthesis branch of the
    /// caller's lead-in stripper threw the title away, so a cut title was unobservable unless it
    /// happened to also unbalance its parentheses — which is the accident, not the population.
    pub title: String, // data_class: INTERNAL_ONLY
    /// The block's own `(title)` lead-in does not close its parentheses.
    ///
    /// A separate shape from a truncated BODY, and worse in kind: the title is cut at its own
    /// budget, and when that cut lands inside a parenthesised clause the `(title):` delimiter
    /// structure is destroyed. The bullet stops being machine-readable — balanced matching never
    /// terminates and splitting on the first `): ` lands somewhere arbitrary — so every downstream
    /// consumer disagrees about where the title ends and the ratified substance begins.
    pub title_unbalanced: bool, // data_class: INTERNAL_ONLY
}

/// A live apex as OBSERVED. The caller supplies this; the kernel never reads a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApexDoc {
    pub id: String,   // data_class: INTERNAL_ONLY
    pub path: String, // data_class: INTERNAL_ONLY
    /// Ids from the frontmatter `supersedes:` list, normalized.
    pub supersedes: Vec<String>, // data_class: INTERNAL_ONLY
    pub blocks: Vec<Block>, // data_class: INTERNAL_ONLY
    /// Whole apex body, lowercased once by the caller — the haystack for the topic check.
    pub body_lower: String, // data_class: INTERNAL_ONLY
}

/// An archived member as OBSERVED — the ORACLE a carried block is judged against.
///
/// The caller supplies this; the kernel never reads a file. Members absent from the map are not
/// source-compared and not topic-checked, which UNDER-detects deliberately: an unreadable member
/// must never become a silent clean bill of health, so the caller reports read failures as errors
/// and the census floor on map size catches a collapsed archive walk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchivedMember {
    /// Every spelling of this member's title the generator could legitimately have carried:
    /// the filename stem, the frontmatter `title:`, and the H1 heading text.
    ///
    /// A LIST rather than one canonical title because the corpus genuinely offers several and the
    /// generator used different ones for different members — 254 gist bullets, most carrying the
    /// stem, some the H1, some the frontmatter title. Measured: only 171 of 448 archived members
    /// declare a frontmatter `title:` at all, so a rule that knew about that spelling alone would
    /// report the other 277 as unresolved and be a false-positive engine rather than a gate.
    pub titles: Vec<String>, // data_class: INTERNAL_ONLY
    /// The member's whole archived body, lowercased once by the caller.
    pub body_lower: String, // data_class: INTERNAL_ONLY
}

/// A topic that must not silently vanish across a supersession, expressed as DATA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic {
    pub name: String, // data_class: INTERNAL_ONLY
    /// Lowercase needles. A topic is PRESENT in a text when any needle occurs in it.
    pub needles: Vec<String>, // data_class: INTERNAL_ONLY
}

impl Topic {
    #[must_use]
    pub fn present_in(&self, haystack_lower: &str) -> bool {
        self.needles.iter().any(|n| haystack_lower.contains(n))
    }
}

/// Census floors. A collapsed walk reports a clean corpus, so the floors are asserted BEFORE any
/// finding count and a shortfall is itself a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    pub min_live_apexes: usize,           // data_class: INTERNAL_ONLY
    pub min_members: usize,               // data_class: INTERNAL_ONLY
    pub min_blocks: usize,                // data_class: INTERNAL_ONLY
    pub min_archived_members_read: usize, // data_class: INTERNAL_ONLY
    pub min_topics: usize,                // data_class: INTERNAL_ONLY
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,   // data_class: INTERNAL_ONLY
    pub apex: String,   // data_class: INTERNAL_ONLY
    pub detail: String, // data_class: INTERNAL_ONLY
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Verdict {
    pub findings: Vec<Finding>, // data_class: INTERNAL_ONLY
    /// Per-CODE totals. Convenient, and too coarse to ratchet on — see `counts_by_apex`.
    pub counts: BTreeMap<String, usize>, // data_class: INTERNAL_ONLY
    /// Per-`(code, apex)` totals, keyed `"{code}@{apex}"`.
    ///
    /// THIS is what the live gate freezes, because a per-code total cannot see an OFFSETTING
    /// change: repair one `apex_gist_truncated_midword` and introduce another and the total is
    /// unmoved, so an equality pin on it passes while the corpus changed underneath. Splitting by
    /// apex does not close that hole, it NARROWS it — the offsetting pair must now cancel within a
    /// single apex. The residual is stated in the gate's non_claims rather than papered over, and
    /// it disappears entirely for any code the repair drives to 0, where equality is exact.
    pub counts_by_apex: BTreeMap<String, usize>, // data_class: INTERNAL_ONLY
}

impl Verdict {
    #[must_use]
    pub fn count(&self, code: &str) -> usize {
        self.counts.get(code).copied().unwrap_or(0)
    }
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Normalize `ADR-0376`, `376`, `ADR-376` to a bare digit string with leading zeros removed.
///
/// The corpus writes member ids BOTH ways — `supersedes:` uses the zero-padded four-digit spelling
/// while the body gist writes `**ADR-376**` unpadded — so comparing the two without normalizing
/// finds no overlap at all and reports every member missing. That failure is silent and looks like
/// a catastrophic finding rather than a bug, which is why it is a named function with its own test.
#[must_use]
pub fn normalize_id(raw: &str) -> Option<String> {
    let digits = raw
        .trim()
        .trim_start_matches("ADR-")
        .trim_start_matches("adr-");
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let trimmed = digits.trim_start_matches('0');
    Some(if trimmed.is_empty() {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    })
}

/// True when `text` ends inside a WORD. A one-way tell, never a decision that the text is whole.
///
/// Decided on the LAST character only, and that makes it a SUFFICIENT condition for a cut and not
/// remotely a necessary one. A byte budget lands wherever it lands, including on a comma: measured
/// on this corpus, `ADR-0701:54` carries `…Sales segmentation remains a PRD-frontmatter field,`
/// while the archived ADR-0131 continues `not a directory split. Historical references to …`. That
/// block is cut and this predicate returns false for it, which is why the SOURCE comparison in
/// `evaluate` is the oracle and this is the fallback for members that cannot be read.
///
/// Do not "fix" it by adding punctuation to the ending set: a comma can end a truncated clause and
/// a finished one, so shape alone cannot tell them apart at any threshold. Only the source can.
#[must_use]
pub fn ends_midword(text: &str) -> bool {
    match text.trim_end().chars().next_back() {
        Some(c) => c.is_alphanumeric() || c == '-',
        None => false,
    }
}

/// A member body prepared for the source comparison: reflowed to one line, with the offsets where
/// its blank-line-separated blocks END retained.
///
/// BOTH halves are load-bearing, and each replaces a scoping rule that was measured WRONG on this
/// corpus before it was written down here:
///
/// - Comparing against the whole body as ONE string reports 611 of 639 carries as cut, because a
///   faithful COMPLETE carry from anywhere but the member's last line still has document after it.
///   "There is more document" is not "this block was cut".
/// - Comparing against each blank-line block SEPARATELY reports only 87, because the consolidation
///   reflowed ACROSS blank lines: a carry reading `para one para two` is contained in no single
///   source block, so a per-block search cannot see the cuts that span one.
///
/// So the haystack is the reflowed whole body — which tolerates a cross-paragraph carry — and the
/// block ends are kept so the question asked at the match site is the right one: does the carry end
/// where a source block ends, or in the middle of one?
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIndex {
    /// Every block, whitespace-collapsed and joined with a single space. Equal to
    /// `normalize_ws(body)`, since collapsing runs of whitespace already collapses a blank line.
    pub reflowed: String, // data_class: INTERNAL_ONLY
    /// Byte offsets into `reflowed` at which some source block ends. Always includes
    /// `reflowed.len()`.
    pub block_ends: BTreeSet<usize>, // data_class: INTERNAL_ONLY
}

impl SourceIndex {
    /// Segment on blank lines, collapse each block, and record where each one ends.
    #[must_use]
    pub fn build(body_lower: &str) -> Self {
        let blocks: Vec<String> = body_lower
            .split("\n\n")
            .map(normalize_ws)
            .filter(|b| !b.is_empty())
            .collect();
        let mut block_ends = BTreeSet::new();
        let mut pos = 0usize;
        for block in &blocks {
            pos += block.len();
            block_ends.insert(pos);
            pos += 1; // the single space this block is joined to the next with
        }
        Self {
            reflowed: blocks.join(" "),
            block_ends,
        }
    }
}

/// True when `carried` is a verbatim PROPER PREFIX of the source block it came from — i.e. CUT.
///
/// `source` is that member's `SourceIndex`; `None` (member unreadable) yields false, because an
/// unread member is not evidence of anything.
///
/// Two conditions, carrying different weight. Verbatim containment is what makes this an ORACLE
/// rather than a heuristic: prose a human wrote as a deliberately terse summary is not a contiguous
/// substring of the source, so a paraphrase or a rename simply fails to match and emits nothing.
/// Ending INSIDE a block rather than at a block end is what makes it a CUT rather than a faithful
/// carry: ratified text still follows in the same block.
///
/// Conservative wherever it is uncertain. A carry matching at SEVERAL sites is exonerated if ANY of
/// them ends at a block end, and a reworded carry reports nothing at all. Both are under-detection,
/// which is the safe direction for a code that pins an equality ratchet — `ends_midword` stays as
/// the fallback that catches part of what this misses.
/// Source-oracle outcome for a carried block.
///
/// `None` — no source, empty carry, or no verbatim occurrence (inconclusive; shape fallback may run).
/// `Some(false)` — at least one occurrence ends on a source block boundary (positively faithful).
/// `Some(true)` — every occurrence is a proper prefix of a longer source block (cut).
#[must_use]
pub fn cut_from_source(carried: &str, source: Option<&SourceIndex>) -> Option<bool> {
    let source = source?;
    let carried = normalize_ws(&carried.to_lowercase());
    if carried.is_empty() {
        return None;
    }
    let mut found = false;
    let mut from = 0usize;
    while let Some(rel) = source.reflowed[from..].find(&carried) {
        let start = from + rel;
        let end = start + carried.len();
        if source.block_ends.contains(&end) {
            return Some(false); // carried right up to a block boundary — a faithful carry
        }
        found = true;
        // Advance by one UTF-8 character from the match start so the next slice stays char-aligned.
        let step = source.reflowed[start..]
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        from = start + step;
    }
    if found { Some(true) } else { None }
}

/// True when the block leaves a Markdown code fence open.
///
/// Counts non-overlapping ``` runs. An odd count means the block ends inside a fence, and every
/// following section of the apex renders as code until something else closes it.
#[must_use]
pub fn has_unclosed_fence(text: &str) -> bool {
    text.matches("```").count() % 2 == 1
}

/// Collapse every run of whitespace to a single space and trim.
///
/// The comparison unit for the source oracle. The consolidation reflowed what it carried — a
/// member's block spans several source lines and arrives in the apex as one — so a byte-for-byte
/// comparison against the member finds nothing and reports a comprehensively cut corpus as clean.
#[must_use]
pub fn normalize_ws(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Evaluate every apex against its members.
///
/// `archived` maps a normalized member id to that member's observed archived form. Members absent
/// from the map are neither source-compared, title-checked nor topic-checked — an unreadable member
/// must never become a silent clean bill of health, so the caller reports read failures as errors
/// and the census floor on map size catches a collapsed archive walk.
#[must_use]
pub fn evaluate(
    apexes: &[ApexDoc],
    archived: &BTreeMap<String, ArchivedMember>,
    topics: &[Topic],
    policy: &Policy,
) -> Verdict {
    let mut verdict = Verdict::default();
    let mut push = |code: &str, apex: &str, detail: String, v: &mut Verdict| {
        *v.counts.entry(code.to_owned()).or_insert(0) += 1;
        *v.counts_by_apex
            .entry(format!("{code}@{apex}"))
            .or_insert(0) += 1;
        v.findings.push(Finding {
            code: code.to_owned(),
            apex: apex.to_owned(),
            detail,
        });
    };

    let total_members: usize = apexes.iter().map(|a| a.supersedes.len()).sum();
    let total_blocks: usize = apexes.iter().map(|a| a.blocks.len()).sum();
    if apexes.len() < policy.min_live_apexes
        || total_members < policy.min_members
        || total_blocks < policy.min_blocks
        || archived.len() < policy.min_archived_members_read
        || topics.len() < policy.min_topics
    {
        push(
            CODE_VACUOUS_SCAN,
            "-",
            format!(
                "census below floor: apexes {} (min {}), members {} (min {}), blocks {} (min {}), \
                 archived read {} (min {}), topics {} (min {})",
                apexes.len(),
                policy.min_live_apexes,
                total_members,
                policy.min_members,
                total_blocks,
                policy.min_blocks,
                archived.len(),
                policy.min_archived_members_read,
                topics.len(),
                policy.min_topics,
            ),
            &mut verdict,
        );
        // A vacuous scan makes every other count meaningless, so stop rather than report zeros.
        return verdict;
    }

    // The member source indexes, built ONCE. Doing it per carried block instead would re-segment a
    // 448-file corpus 639 times for no gain.
    let sources: BTreeMap<&str, SourceIndex> = archived
        .iter()
        .map(|(id, member)| (id.as_str(), SourceIndex::build(&member.body_lower)))
        .collect();

    for apex in apexes {
        for block in &apex.blocks {
            // The source oracle, tried FIRST. `ends_midword` is consulted only for the blocks this
            // could not decide, so the two codes partition the cut population instead of
            // double-counting the blocks that happen to satisfy both.
            let source_cut = cut_from_source(&block.text, sources.get(block.member_id.as_str()));
            if source_cut == Some(true) {
                push(
                    CODE_CUT_FROM_SOURCE,
                    &apex.id,
                    format!(
                        "{}:{} {} for member {} is a verbatim PROPER PREFIX of the archived member \
                         — ratified text follows the cut in the source: ...{}",
                        apex.path,
                        block.line,
                        block.site.as_str(),
                        block.member_id,
                        tail(&block.text, 24)
                    ),
                    &mut verdict,
                );
            } else if source_cut.is_none() && ends_midword(&block.text) {
                push(
                    CODE_TRUNCATED_MIDWORD,
                    &apex.id,
                    format!(
                        "{}:{} {} for member {} ends mid-word: ...{}",
                        apex.path,
                        block.line,
                        block.site.as_str(),
                        block.member_id,
                        tail(&block.text, 24)
                    ),
                    &mut verdict,
                );
            }
            // A title that matches NO spelling the member offers. Checked independently of the
            // body: the title carries its own, shorter budget and is cut on its own.
            if !block.title.is_empty() {
                if let Some(member) = archived.get(block.member_id.as_str()) {
                    let carried = normalize_ws(&block.title);
                    if !member.titles.iter().any(|t| normalize_ws(t) == carried) {
                        push(
                            CODE_TITLE_UNRESOLVED,
                            &apex.id,
                            format!(
                                "{}:{} {} for member {} carries title '{}', which matches neither \
                                 the member's filename stem, its frontmatter title, nor its H1",
                                apex.path,
                                block.line,
                                block.site.as_str(),
                                block.member_id,
                                carried
                            ),
                            &mut verdict,
                        );
                    }
                }
            }
            if block.title_unbalanced {
                push(
                    CODE_UNBALANCED_TITLE,
                    &apex.id,
                    format!(
                        "{}:{} {} for member {} has a title cut inside its own parentheses, so the \
                         (title): delimiter never closes",
                        apex.path,
                        block.line,
                        block.site.as_str(),
                        block.member_id
                    ),
                    &mut verdict,
                );
            }
            if has_unclosed_fence(&block.text) {
                push(
                    CODE_UNCLOSED_FENCE,
                    &apex.id,
                    format!(
                        "{}:{} {} for member {} leaves a code fence open",
                        apex.path,
                        block.line,
                        block.site.as_str(),
                        block.member_id
                    ),
                    &mut verdict,
                );
            }
        }

        let with_gist: BTreeSet<&str> = apex
            .blocks
            .iter()
            .filter(|b| b.site == Site::Gist)
            .map(|b| b.member_id.as_str())
            .collect();
        let with_residual: BTreeSet<&str> = apex
            .blocks
            .iter()
            .filter(|b| b.site == Site::Residual)
            .map(|b| b.member_id.as_str())
            .collect();

        for member in &apex.supersedes {
            if !with_gist.contains(member.as_str()) {
                push(
                    CODE_MEMBER_WITHOUT_GIST,
                    &apex.id,
                    format!("member {member} is superseded by this apex but has no gist"),
                    &mut verdict,
                );
            }
            if !with_residual.contains(member.as_str()) {
                push(
                    CODE_MEMBER_WITHOUT_RESIDUAL,
                    &apex.id,
                    format!("member {member} is superseded by this apex but has no residual"),
                    &mut verdict,
                );
            }
        }

        // Topic drop: reported once per (apex, topic) with the members that evidence it, because
        // fifty members carrying one dropped topic is ONE hole in the law, not fifty.
        for topic in topics {
            if topic.present_in(&apex.body_lower) {
                continue;
            }
            let carriers: Vec<&str> = apex
                .supersedes
                .iter()
                .filter(|m| {
                    archived
                        .get(m.as_str())
                        .is_some_and(|member| topic.present_in(&member.body_lower))
                })
                .map(String::as_str)
                .collect();
            if !carriers.is_empty() {
                push(
                    CODE_TOPIC_DROPPED,
                    &apex.id,
                    format!(
                        "topic '{}' is carried by superseded members [{}] but appears nowhere in {}",
                        topic.name,
                        carriers.join(", "),
                        apex.path
                    ),
                    &mut verdict,
                );
            }
        }
    }

    verdict.findings.sort();
    verdict
}

fn tail(text: &str, n: usize) -> String {
    let trimmed = text.trim_end();
    let start = trimmed
        .char_indices()
        .rev()
        .take(n)
        .last()
        .map_or(0, |(i, _)| i);
    trimmed[start..].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fixtures use tokens with NO governed ADR shape on purpose: this file has a `.rs` extension
    // and the sibling citation-closure gate scans `.rs`, so a plausible `ADR-NNNN` here would be
    // read as a real citation and redden a gate this change never touched.
    fn block(member: &str, site: Site, text: &str) -> Block {
        Block {
            member_id: member.to_owned(),
            site,
            line: 1,
            text: text.to_owned(),
            title: String::new(),
            title_unbalanced: false,
        }
    }

    fn block_bad_title(member: &str, site: Site, text: &str) -> Block {
        Block {
            title_unbalanced: true,
            ..block(member, site, text)
        }
    }

    fn block_titled(member: &str, site: Site, title: &str, text: &str) -> Block {
        Block {
            title: title.to_owned(),
            ..block(member, site, text)
        }
    }

    /// An archived member offering exactly the titles given, with `body` as its whole body.
    fn member(titles: &[&str], body: &str) -> ArchivedMember {
        ArchivedMember {
            titles: titles.iter().map(|t| (*t).to_owned()).collect(),
            body_lower: body.to_lowercase(),
        }
    }

    fn permissive_policy() -> Policy {
        Policy {
            min_live_apexes: 1,
            min_members: 1,
            min_blocks: 1,
            min_archived_members_read: 0,
            min_topics: 0,
        }
    }

    fn apex(supersedes: &[&str], blocks: Vec<Block>, body: &str) -> ApexDoc {
        ApexDoc {
            id: "T-APEX".to_owned(),
            path: "fixture".to_owned(),
            supersedes: supersedes.iter().map(|s| (*s).to_owned()).collect(),
            blocks,
            body_lower: body.to_lowercase(),
        }
    }

    #[test]
    fn a_block_that_finishes_its_sentence_is_clean() {
        let a = apex(
            &["1"],
            vec![
                block("1", Site::Gist, "The substrate is owned end to end."),
                block("1", Site::Residual, "The substrate is owned end to end."),
            ],
            "the substrate is owned end to end.",
        );
        let v = evaluate(&[a], &BTreeMap::new(), &[], &permissive_policy());
        assert!(v.is_clean(), "unexpected findings: {:?}", v.findings);
    }

    #[test]
    fn a_block_cut_inside_a_word_is_caught_at_both_sites() {
        let a = apex(
            &["1"],
            vec![
                block(
                    "1",
                    Site::Gist,
                    "hosted control planes run via a hosted-contr",
                ),
                block("1", Site::Residual, "hosted control planes run via Kam"),
            ],
            "hosted",
        );
        let v = evaluate(&[a], &BTreeMap::new(), &[], &permissive_policy());
        assert_eq!(v.count(CODE_TRUNCATED_MIDWORD), 2);
    }

    #[test]
    fn trailing_whitespace_does_not_hide_a_midword_cut() {
        // The real corpus truncates to a byte budget and often lands on a space, so a detector
        // that looks at the literal last character misses the cut entirely.
        let a = apex(
            &["1"],
            vec![block("1", Site::Gist, "six-phase canonical workflow   ")],
            "six",
        );
        let v = evaluate(&[a], &BTreeMap::new(), &[], &permissive_policy());
        assert_eq!(v.count(CODE_TRUNCATED_MIDWORD), 1);
    }

    /// The regression that motivated the source oracle, carrying the REAL text of the exhibit.
    ///
    /// `ADR-0701:54` ends on `…PRD-frontmatter field,` and the archived member continues `not a
    /// directory split. Historical references to …`. A comma is not alphanumeric, so `ends_midword`
    /// returns false and every shape predicate in this kernel is blind to it. Only the source sees
    /// it. The member id is a fixture token with no governed ADR shape, per the module note.
    #[test]
    fn a_cut_landing_on_punctuation_is_invisible_to_shape_and_caught_by_the_source() {
        let carried = "Sales segmentation remains a PRD-frontmatter field,";
        let source = "sales segmentation remains a prd-frontmatter field, not a directory split. \
                      historical references to legacy paths are examples and must be read as such.";
        assert!(
            !ends_midword(carried),
            "the shape predicate is supposed to MISS this; if it now hits, this test no longer \
             proves the oracle is load-bearing"
        );

        let archived = BTreeMap::from([("1".to_owned(), member(&[], source))]);
        let a = apex(&["1"], vec![block("1", Site::Gist, carried)], carried);
        let v = evaluate(&[a], &archived, &[], &permissive_policy());
        assert_eq!(v.count(CODE_CUT_FROM_SOURCE), 1);
        assert_eq!(v.count(CODE_TRUNCATED_MIDWORD), 0);
    }

    #[test]
    fn a_carry_that_reaches_the_end_of_its_member_is_not_a_cut() {
        // Containment alone would report EVERY faithful whole carry as a defect. The `!ends_with`
        // half is what makes the oracle decide CUT rather than merely QUOTED.
        let text = "The substrate is owned end to end.";
        let archived = BTreeMap::from([(
            "1".to_owned(),
            member(&[], "Preamble prose. The substrate is owned end to end."),
        )]);
        let a = apex(
            &["1"],
            vec![
                block("1", Site::Gist, text),
                block("1", Site::Residual, text),
            ],
            text,
        );
        let v = evaluate(&[a], &archived, &[], &permissive_policy());
        assert!(v.is_clean(), "unexpected findings: {:?}", v.findings);
    }

    #[test]
    fn a_carry_the_consolidation_reflowed_across_a_blank_line_is_still_compared() {
        // The under-detection that per-block scoping produces. The generator joined two source
        // paragraphs into one apex line; searching each paragraph separately finds neither, and
        // measured on the real corpus that scoping saw only 87 cuts where this one sees more.
        let archived = BTreeMap::from([(
            "1".to_owned(),
            member(
                &[],
                "First paragraph ends here.\n\nSecond paragraph continues on.",
            ),
        )]);
        let a = apex(
            &["1"],
            vec![block(
                "1",
                Site::Gist,
                "First paragraph ends here. Second paragraph contin",
            )],
            "first",
        );
        let v = evaluate(&[a], &archived, &[], &permissive_policy());
        assert_eq!(v.count(CODE_CUT_FROM_SOURCE), 1);
    }

    #[test]
    fn a_carry_ending_exactly_at_a_block_boundary_is_faithful_even_mid_document() {
        // The exoneration the block-end index exists for: this carry spans a blank line and stops
        // precisely where its source block stops, so nothing of THAT block was lost.
        let archived = BTreeMap::from([(
            "1".to_owned(),
            member(
                &[],
                "First paragraph ends here.\n\nSecond one too.\n\nA third says something else.",
            ),
        )]);
        let a = apex(
            &["1"],
            vec![
                block(
                    "1",
                    Site::Gist,
                    "First paragraph ends here. Second one too.",
                ),
                block(
                    "1",
                    Site::Residual,
                    "First paragraph ends here. Second one too.",
                ),
            ],
            "first",
        );
        let v = evaluate(&[a], &archived, &[], &permissive_policy());
        assert!(v.is_clean(), "unexpected findings: {:?}", v.findings);
    }

    /// The false positive that whole-body scoping produces, and the reason the unit is the BLOCK.
    ///
    /// Measured on the real corpus: comparing against the whole member body reported 611 of 639
    /// blocks cut, because a complete carry from anywhere but the member's last line still has
    /// document after it. "There is more document" is not "this block was cut".
    #[test]
    fn a_complete_carry_with_a_later_paragraph_after_it_is_not_a_cut() {
        let archived = BTreeMap::from([(
            "1".to_owned(),
            member(
                &[],
                "The substrate is owned end to end.\n\nA later section says something else entirely.",
            ),
        )]);
        let a = apex(
            &["1"],
            vec![
                block("1", Site::Gist, "The substrate is owned end to end."),
                block("1", Site::Residual, "The substrate is owned end to end."),
            ],
            "the substrate is owned end to end.",
        );
        let v = evaluate(&[a], &archived, &[], &permissive_policy());
        assert!(v.is_clean(), "unexpected findings: {:?}", v.findings);
    }

    #[test]
    fn a_paraphrase_is_not_reported_however_short_the_apex_made_it() {
        // The false-positive this oracle must not have: a hand-authored terse summary is not a
        // contiguous substring of the source, so containment fails and nothing is emitted.
        let archived = BTreeMap::from([(
            "1".to_owned(),
            member(
                &[],
                "We adopt a single owned substrate for the whole delivery path.",
            ),
        )]);
        let a = apex(
            &["1"],
            vec![block("1", Site::Gist, "One owned substrate, end to end.")],
            "one owned substrate",
        );
        let v = evaluate(&[a], &archived, &[], &permissive_policy());
        assert_eq!(v.count(CODE_CUT_FROM_SOURCE), 0);
    }

    #[test]
    fn the_source_oracle_reflows_whitespace_before_comparing() {
        // The consolidation joined multi-line source blocks into one apex line. Comparing raw text
        // finds nothing and reports a comprehensively cut corpus as clean.
        let archived = BTreeMap::from([(
            "1".to_owned(),
            member(
                &[],
                "the rule holds\n   across every\nline of source. and then more.",
            ),
        )]);
        let a = apex(
            &["1"],
            vec![block(
                "1",
                Site::Gist,
                "The rule holds across every line of source.",
            )],
            "the rule holds",
        );
        let v = evaluate(&[a], &archived, &[], &permissive_policy());
        assert_eq!(v.count(CODE_CUT_FROM_SOURCE), 1);
    }

    #[test]
    fn a_block_that_is_both_a_proper_prefix_and_midword_is_counted_once() {
        // The two codes PARTITION the cut population. Counting a block under both would double the
        // same defect and make the two frozen numbers non-additive.
        let archived = BTreeMap::from([(
            "1".to_owned(),
            member(
                &[],
                "hosted control planes run via a hosted-controller model.",
            ),
        )]);
        let a = apex(
            &["1"],
            vec![block(
                "1",
                Site::Gist,
                "hosted control planes run via a hosted-contr",
            )],
            "hosted",
        );
        let v = evaluate(&[a], &archived, &[], &permissive_policy());
        assert_eq!(v.count(CODE_CUT_FROM_SOURCE), 1);
        assert_eq!(v.count(CODE_TRUNCATED_MIDWORD), 0);
    }

    #[test]
    fn a_title_matching_any_spelling_the_member_offers_resolves() {
        // Stem, frontmatter title and H1 are all legitimate carries; the generator used each.
        let archived = BTreeMap::from([(
            "1".to_owned(),
            member(
                &["the-file-stem", "A Declared Title", "T-9 — An H1 Heading"],
                "body",
            ),
        )]);
        for carried in ["the-file-stem", "A Declared Title", "T-9 — An H1 Heading"] {
            let a = apex(
                &["1"],
                vec![block_titled("1", Site::Gist, carried, "Body finishes.")],
                "body finishes.",
            );
            let v = evaluate(&[a], &archived, &[], &permissive_policy());
            assert_eq!(
                v.count(CODE_TITLE_UNRESOLVED),
                0,
                "rejected a real title: {carried}"
            );
        }
    }

    #[test]
    fn a_title_cut_at_its_own_budget_is_caught_even_though_its_parentheses_balance() {
        // The population UNBALANCED_TITLE misses. The generator emits its own closing `)`, so a cut
        // title balances by default and the body here finishes normally — nothing else fires.
        let archived = BTreeMap::from([(
            "1".to_owned(),
            member(&["cedar-policy-extension-supervisor-capabilities"], "body"),
        )]);
        let a = apex(
            &["1"],
            vec![block_titled(
                "1",
                Site::Gist,
                "cedar-policy-extension-supervisor-capabil",
                "Body finishes.",
            )],
            "body finishes.",
        );
        let v = evaluate(&[a], &archived, &[], &permissive_policy());
        assert_eq!(v.count(CODE_TITLE_UNRESOLVED), 1);
        assert_eq!(v.count(CODE_UNBALANCED_TITLE), 0);
        assert_eq!(v.count(CODE_TRUNCATED_MIDWORD), 0);
    }

    #[test]
    fn an_unreadable_member_yields_no_title_or_source_finding_rather_than_a_guess() {
        // Fail-QUIET is deliberate here and is stated in the gate's non_claims: the census floor on
        // archive size is what catches a collapsed walk, not a burst of unresolvable titles.
        let a = apex(
            &["1"],
            vec![block_titled(
                "1",
                Site::Gist,
                "any-title-at-all",
                "Body finishes.",
            )],
            "body finishes.",
        );
        let v = evaluate(&[a], &BTreeMap::new(), &[], &permissive_policy());
        assert_eq!(v.count(CODE_TITLE_UNRESOLVED), 0);
        assert_eq!(v.count(CODE_CUT_FROM_SOURCE), 0);
    }

    #[test]
    fn counts_are_keyed_by_apex_so_an_offsetting_pair_cannot_cancel_across_apexes() {
        // The narrowing that makes the equality ratchet mean something. Per-CODE totals are equal
        // here (one truncation each way); per-(code, apex) they are not, which is the whole point.
        let archived = BTreeMap::new();
        let mut left = apex(&["1"], vec![block("1", Site::Gist, "cut mid-wor")], "cut");
        left.id = "700".to_owned();
        let mut right = apex(
            &["2"],
            vec![block("2", Site::Gist, "also cut mid-wor")],
            "cut",
        );
        right.id = "709".to_owned();
        let v = evaluate(&[left, right], &archived, &[], &permissive_policy());
        assert_eq!(v.count(CODE_TRUNCATED_MIDWORD), 2);
        assert_eq!(
            v.counts_by_apex.get("apex_gist_truncated_midword@700"),
            Some(&1)
        );
        assert_eq!(
            v.counts_by_apex.get("apex_gist_truncated_midword@709"),
            Some(&1)
        );
    }

    #[test]
    fn an_odd_fence_run_is_an_unclosed_fence_and_an_even_one_is_not() {
        assert!(has_unclosed_fence("prose ```rust //"));
        assert!(!has_unclosed_fence("prose ```rust let x = 1; ``` done."));
        assert!(!has_unclosed_fence("no fence here."));
    }

    #[test]
    fn a_title_cut_inside_its_own_parentheses_is_caught_separately_from_a_cut_body() {
        // The body here FINISHES, so this must be reported as a title defect on its own and must
        // not be conflated with the mid-word body cut.
        let a = apex(
            &["1"],
            vec![
                block_bad_title("1", Site::Gist, "Oyatie ships a plugin substrate."),
                block("1", Site::Residual, "Oyatie ships a plugin substrate."),
            ],
            "oyatie ships a plugin substrate.",
        );
        let v = evaluate(&[a], &BTreeMap::new(), &[], &permissive_policy());
        assert_eq!(v.count(CODE_UNBALANCED_TITLE), 1);
        assert_eq!(v.count(CODE_TRUNCATED_MIDWORD), 0);
    }

    #[test]
    fn a_member_named_in_supersedes_but_never_carried_is_caught() {
        let a = apex(
            &["1", "2"],
            vec![
                block("1", Site::Gist, "Carried in full."),
                block("1", Site::Residual, "Carried in full."),
            ],
            "carried in full.",
        );
        let v = evaluate(&[a], &BTreeMap::new(), &[], &permissive_policy());
        assert_eq!(v.count(CODE_MEMBER_WITHOUT_GIST), 1);
        assert_eq!(v.count(CODE_MEMBER_WITHOUT_RESIDUAL), 1);
        assert!(v.findings.iter().any(|f| f.detail.contains("member 2")));
    }

    #[test]
    fn a_topic_a_member_carries_and_the_apex_never_mentions_is_caught_once() {
        let topics = vec![Topic {
            name: "confidential-computing".to_owned(),
            needles: vec!["sev-snp".to_owned(), "tdx".to_owned()],
        }];
        let archived = BTreeMap::from([
            (
                "1".to_owned(),
                member(&[], "runtime tier uses sev-snp attestation"),
            ),
            ("2".to_owned(), member(&[], "also mentions tdx enclaves")),
        ]);
        let a = apex(
            &["1", "2"],
            vec![
                block("1", Site::Gist, "Runtime tiers are enforced."),
                block("1", Site::Residual, "Runtime tiers are enforced."),
                block("2", Site::Gist, "Runtime tiers are enforced."),
                block("2", Site::Residual, "Runtime tiers are enforced."),
            ],
            "runtime tiers are enforced.",
        );
        let v = evaluate(&[a], &archived, &topics, &permissive_policy());
        // ONE hole in the law, naming BOTH members as evidence — not one finding per member.
        assert_eq!(v.count(CODE_TOPIC_DROPPED), 1);
        let f = v
            .findings
            .iter()
            .find(|f| f.code == CODE_TOPIC_DROPPED)
            .expect("topic finding");
        assert!(f.detail.contains('1') && f.detail.contains('2'));
    }

    #[test]
    fn a_topic_the_apex_still_states_is_not_a_drop() {
        let topics = vec![Topic {
            name: "confidential-computing".to_owned(),
            needles: vec!["sev-snp".to_owned()],
        }];
        let archived = BTreeMap::from([("1".to_owned(), member(&[], "uses sev-snp"))]);
        let a = apex(
            &["1"],
            vec![
                block("1", Site::Gist, "Runtime uses SEV-SNP attestation."),
                block("1", Site::Residual, "Runtime uses SEV-SNP attestation."),
            ],
            "runtime uses sev-snp attestation.",
        );
        let v = evaluate(&[a], &archived, &topics, &permissive_policy());
        assert_eq!(v.count(CODE_TOPIC_DROPPED), 0);
    }

    #[test]
    fn a_collapsed_walk_fails_closed_instead_of_reporting_a_clean_corpus() {
        // The failure this guards is the dangerous one: a walk that finds nothing produces zero
        // findings, which is indistinguishable from a repaired corpus without a floor.
        let policy = Policy {
            min_live_apexes: 10,
            min_members: 100,
            min_blocks: 100,
            min_archived_members_read: 100,
            min_topics: 1,
        };
        let v = evaluate(&[], &BTreeMap::new(), &[], &policy);
        assert_eq!(v.count(CODE_VACUOUS_SCAN), 1);
        assert!(!v.is_clean());
    }

    #[test]
    fn a_vacuous_scan_suppresses_every_other_count() {
        let policy = Policy {
            min_live_apexes: 10,
            min_members: 1,
            min_blocks: 1,
            min_archived_members_read: 0,
            min_topics: 0,
        };
        let a = apex(&["1"], vec![block("1", Site::Gist, "cut mid-wor")], "cut");
        let v = evaluate(&[a], &BTreeMap::new(), &[], &policy);
        assert_eq!(v.count(CODE_VACUOUS_SCAN), 1);
        assert_eq!(v.count(CODE_TRUNCATED_MIDWORD), 0);
    }

    #[test]
    fn padded_and_unpadded_member_ids_normalize_onto_each_other() {
        // `supersedes:` writes ADR-0376 while the body gist writes **ADR-376**. Without this the
        // two sides never intersect and every member reports missing.
        assert_eq!(normalize_id("ADR-0376").as_deref(), Some("376"));
        assert_eq!(normalize_id("376").as_deref(), Some("376"));
        assert_eq!(normalize_id("ADR-0009").as_deref(), Some("9"));
        assert_eq!(normalize_id("0000").as_deref(), Some("0"));
        assert_eq!(normalize_id("ADR-nope"), None);
        assert_eq!(normalize_id(""), None);
    }

    #[test]
    fn tail_never_splits_a_multibyte_character() {
        // The corpus is full of box-drawing and em-dashes; slicing by byte offset panics on them.
        let t = tail("policy is ─────────────────────────┘", 12);
        assert!(!t.is_empty());
    }
}
