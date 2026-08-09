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
//! Five independent shapes of that defect are mechanically decidable without re-deriving the
//! generator, and this kernel decides exactly those:
//!
//! 1. `TRUNCATED_MIDWORD` — a carried block whose last character is alphanumeric or a hyphen.
//!    Prose that was allowed to finish ends on punctuation; prose cut at a byte budget ends inside
//!    a word. This is the shape named in the defect report: `a hosted-contr`, `Kam`.
//! 2. `UNCLOSED_FENCE` — a carried block containing an ODD number of ``` runs. This one is not
//!    merely lossy, it is CORRUPTING: an unbalanced fence swallows every following block into a
//!    code span, so one truncated member silently changes how the rest of the apex renders.
//! 3. `MEMBER_WITHOUT_GIST` / `MEMBER_WITHOUT_RESIDUAL` — an id named in the apex `supersedes:`
//!    list that the apex body never carries at all. A budgeted member list drops its TAIL, and the
//!    tail of an ascending ADR list is the NEWEST law — the decisions most likely to still be live.
//! 4. `UNBALANCED_TITLE` — a bullet whose `(title)` lead-in never closes, because the TITLE has
//!    its own, shorter budget and the cut landed inside a parenthesised clause. This one is not
//!    about lost prose at all: it destroys the delimiter that separates the title from the
//!    ratified substance, so no two consumers agree on where the substance starts.
//! 5. `TOPIC_DROPPED` — a topic that a superseded member demonstrably carries text about and that
//!    the superseding apex never mentions. This is the cross-check that catches loss the other
//!    three cannot see: a member can be dropped wholesale, or truncated before it ever reaches its
//!    subject, and either way the apex silently stops being law on that subject.
//!
//! The kernel is generic over id STRINGS so its unit fixtures can use tokens with no governed
//! shape. Writing a realistic `ADR-NNNN` id into a `.rs` fixture would be read as a real citation
//! by the sibling citation-closure gate, which scans `.rs`.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

pub const CODE_TRUNCATED_MIDWORD: &str = "apex_gist_truncated_midword";
pub const CODE_UNCLOSED_FENCE: &str = "apex_gist_unclosed_fence";
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
    pub member_id: String,
    pub site: Site,
    /// Line number in the apex file, for a finding a human can navigate to.
    pub line: usize,
    /// The carried text with its `- **ADR-N** (title):` / `**title** —` lead-in already stripped.
    pub text: String,
    /// The block's own `(title)` lead-in does not close its parentheses.
    ///
    /// A separate shape from a truncated BODY, and worse in kind: the title is cut at its own
    /// budget, and when that cut lands inside a parenthesised clause the `(title):` delimiter
    /// structure is destroyed. The bullet stops being machine-readable — balanced matching never
    /// terminates and splitting on the first `): ` lands somewhere arbitrary — so every downstream
    /// consumer disagrees about where the title ends and the ratified substance begins.
    pub title_unbalanced: bool,
}

/// A live apex as OBSERVED. The caller supplies this; the kernel never reads a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApexDoc {
    pub id: String,
    pub path: String,
    /// Ids from the frontmatter `supersedes:` list, normalized.
    pub supersedes: Vec<String>,
    pub blocks: Vec<Block>,
    /// Whole apex body, lowercased once by the caller — the haystack for the topic check.
    pub body_lower: String,
}

/// A topic that must not silently vanish across a supersession, expressed as DATA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic {
    pub name: String,
    /// Lowercase needles. A topic is PRESENT in a text when any needle occurs in it.
    pub needles: Vec<String>,
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
    pub min_live_apexes: usize,
    pub min_members: usize,
    pub min_blocks: usize,
    pub min_archived_members_read: usize,
    pub min_topics: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub apex: String,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Verdict {
    pub findings: Vec<Finding>,
    pub counts: BTreeMap<String, usize>,
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
    let digits = raw.trim().trim_start_matches("ADR-").trim_start_matches("adr-");
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

/// True when `text` was cut inside a word rather than allowed to finish.
///
/// Deliberately decided on the LAST character only. A budget cut lands wherever the budget lands,
/// so the tell is the absence of a terminator, not the presence of any particular ending.
#[must_use]
pub fn ends_midword(text: &str) -> bool {
    match text.trim_end().chars().next_back() {
        Some(c) => c.is_alphanumeric() || c == '-',
        None => false,
    }
}

/// True when the block leaves a Markdown code fence open.
///
/// Counts non-overlapping ``` runs. An odd count means the block ends inside a fence, and every
/// following section of the apex renders as code until something else closes it.
#[must_use]
pub fn has_unclosed_fence(text: &str) -> bool {
    text.matches("```").count() % 2 == 1
}

/// Evaluate every apex against its members.
///
/// `archived_lower` maps a normalized member id to that member's whole archived body, lowercased.
/// Members absent from the map are simply not topic-checked — an unreadable member must never
/// become a silent clean bill of health, so the caller reports read failures as errors and the
/// census floor on map size catches a collapsed archive walk.
#[must_use]
pub fn evaluate(
    apexes: &[ApexDoc],
    archived_lower: &BTreeMap<String, String>,
    topics: &[Topic],
    policy: &Policy,
) -> Verdict {
    let mut verdict = Verdict::default();
    let mut push = |code: &str, apex: &str, detail: String, v: &mut Verdict| {
        *v.counts.entry(code.to_owned()).or_insert(0) += 1;
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
        || archived_lower.len() < policy.min_archived_members_read
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
                archived_lower.len(),
                policy.min_archived_members_read,
                topics.len(),
                policy.min_topics,
            ),
            &mut verdict,
        );
        // A vacuous scan makes every other count meaningless, so stop rather than report zeros.
        return verdict;
    }

    for apex in apexes {
        for block in &apex.blocks {
            if ends_midword(&block.text) {
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
                    archived_lower
                        .get(m.as_str())
                        .is_some_and(|body| topic.present_in(body))
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
            title_unbalanced: false,
        }
    }

    fn block_bad_title(member: &str, site: Site, text: &str) -> Block {
        Block {
            title_unbalanced: true,
            ..block(member, site, text)
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
                block("1", Site::Gist, "hosted control planes run via a hosted-contr"),
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
            ("1".to_owned(), "runtime tier uses sev-snp attestation".to_owned()),
            ("2".to_owned(), "also mentions tdx enclaves".to_owned()),
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
        let archived = BTreeMap::from([("1".to_owned(), "uses sev-snp".to_owned())]);
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
