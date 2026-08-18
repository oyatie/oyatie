//! # check-adr-citation-closure — ADR citations must be MEANINGFULLY, not syntactically, correct
//!
//! ## The defect this exists to make impossible
//! A mechanical de-stale pass rewrote thousands of ADR path citations onto the ten live apex ADRs
//! (`docs/decisions/ADR-070x-*-live-apex.md`). Every rewritten path is a VALID PATH TO AN EXISTING
//! FILE, so a link checker passes all of them. Many nonetheless point at an apex that never claimed
//! the ADR the surrounding sentence is actually about:
//!
//! ```text
//! - docs/decisions/ADR-0709-general-live-apex.md # folded into ADR-0515 cloud-ci/oya-ci Tide
//! ```
//!
//! ADR-0515 was consolidated into ADR-0700, not ADR-0709. The path resolves; the MEANING does not.
//! Only checking the citation against an independent supersession oracle catches it.
//!
//! ## The oracle is built from BOTH sides
//! An apex-side-only map (`supersedes:` on the ten apexes) is incomplete, because supersession
//! CHAINS through archived intermediates: `ADR-0110 -> ADR-0363 -> ADR-0701` and
//! `ADR-0349 -> ADR-0515 -> ADR-0700`. A member-side-only map is equally incomplete: 37 members
//! declare a successor no apex acknowledges. The successor edge set is therefore the UNION of
//!
//! * member side — `superseded_by:` in `docs/adr-archive/ADR-NNNN-*.md` frontmatter, and
//! * apex side   — membership in some record's `supersedes:` frontmatter list,
//!
//! and closure is the transitive walk of that union until a LIVE record is reached.
//!
//! ## Only FRONTMATTER is parsed
//! ADR bodies mention ADR ids constantly — in prose, in tables, inside fenced code blocks. One
//! archived ADR even contains a fenced `superseded_by:` block naming ITSELF. Grepping whole files
//! for supersession edges produces false matches; the parser here reads only the `---` delimited
//! frontmatter head and only its top-level `status`/`supersedes`/`superseded_by` keys.
//!
//! ## Anti-vacuity is the load-bearing part
//! A walk that finds no ADRs, or no citations, computes zero mismatches, which reads as a PERFECT
//! corpus. That is the dangerous failure of any gate. Five floors fail it closed instead.
//!
//! PURE: no I/O, no clock, no rand. The caller walks the tree and passes observations as DATA.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

/// The gate's stable identifier.
pub const GATE_ID: &str = "governance-check-adr-citation-closure";

/// A citation whose surrounding context names an ADR whose closure resolves to a DIFFERENT apex,
/// and no named ADR resolves to the cited one. Blocking — this is the de-stale defect.
pub const CODE_CITATION_MISMATCH: &str = "adr_citation_closure_mismatch";

/// An authority surface cites an ADR whose status carries NO implement authority. Blocking — a
/// decision that is not law looks exactly like a citation to live doctrine on the surfaces that
/// ARE the governance contract.
///
/// Checked over BOTH halves of the line. `scan_line` moves an id that appears as a `decisions/`
/// PATH out of `context`, so a surface that writes the id twice — `[ADR-0347](decisions/ADR-0347-…)`
/// — leaves `context` empty. Scanning only `context` therefore blinded this rule to exactly the
/// citations that state the doctrine most explicitly: the ones that both name and link it.
///
/// The code name is historical: the rule began as `Rejected`-only and the id is pinned by the
/// policy's equality ceilings, so it is kept rather than renamed for a widened predicate.
pub const CODE_REJECTED_AUTHORITY: &str = "adr_citation_rejected_authority";

/// Statuses that carry NO implement authority, taken verbatim from the repository's own rule at
/// `docs/decisions/_disposition/2026-08-06-live-resolution-rule.json`: "If status
/// Proposed/Deprecated/Rejected -> not implement authority".
///
/// `Superseded` is DELIBERATELY ABSENT, and the difference matters. A superseded ADR resolves
/// THROUGH `superseded_by` to a live successor and is cited as history on purpose — root
/// `CLAUDE.md` carries `historical_substrate_adrs` and `historical_vcs_ratchet_adrs` blocks that
/// do exactly that. Measured on the three authority surfaces: 21 of the ids they name are
/// `Superseded`, so widening this to `status != "Accepted"` would manufacture 21 accusations
/// against citations that are correct. Supersession already has its own rules (the closure
/// oracle, [`CODE_UNRESOLVABLE`], [`CODE_ASYMMETRY`]); this one is about LIFECYCLE, and the two
/// questions must not be overloaded onto one predicate.
pub const NON_ENFORCING_STATUSES: [&str; 3] = ["Proposed", "Deprecated", "Rejected"];

/// A `decisions/` path citation naming an ADR that is not a live apex — either no record exists at
/// all, or the record is an archived member. Blocking — the path does not resolve to a file under
/// `docs/decisions/`, so the citation is broken however true the sentence around it may be.
///
/// This was previously a silent `continue`: the mismatch rule required the cited id to be live and
/// skipped everything else, so a citation pointing at a decision that no longer lives under
/// `docs/decisions/` produced NO finding of any kind.
pub const CODE_DANGLING_CITATION: &str = "adr_citation_dangling_path";

/// Two observed records share one normalized id. Blocking — the oracle is keyed by id, so one
/// record silently displaces the other while both censuses still count two, leaving every floor
/// satisfied and the closure answering from a record the reader never sees.
pub const CODE_DUPLICATE_ID: &str = "adr_duplicate_id";

/// `supersedes` and `superseded_by` disagree about an edge. Blocking — a one-sided edge means the
/// two halves of the oracle disagree, and whichever half a reader consults decides the answer.
pub const CODE_ASYMMETRY: &str = "adr_closure_asymmetry";

/// A superseded ADR whose successor chain reaches no live apex. Blocking — every citation of it is
/// unrepairable until the chain is completed, so the corpus has no answer to give.
pub const CODE_UNRESOLVABLE: &str = "adr_closure_unresolvable";

/// The scan collapsed: too few ADRs, citations, or files to be evidence. Blocking.
pub const CODE_VACUOUS_SCAN: &str = "adr_citation_scan_vacuous";

/// An ADR whose closure reaches more than one distinct live apex. Advisory: the corpus is
/// ambiguous, so no citation of it can be called wrong, and the mismatch rule stands down.
pub const CODE_AMBIGUOUS_CLOSURE: &str = "adr_closure_ambiguous";

/// One ADR document as observed in the tree. `live` distinguishes an apex under
/// `docs/decisions/` from an archived member under `docs/adr-archive/`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AdrRecord {
    /// Normalized id, always `ADR-` plus four digits.
    pub id: String,
    /// Repo-relative path the record was read from.
    pub path: String,
    /// Is this a LIVE apex (a citation may legitimately point here) or an archived member?
    pub live: bool,
    /// Frontmatter `status:` verbatim (`Accepted`, `Superseded`, `Rejected`, ...).
    pub status: String,
    /// Frontmatter `supersedes:` ids, normalized.
    pub supersedes: Vec<String>,
    /// Frontmatter `superseded_by:` ids, normalized.
    pub superseded_by: Vec<String>,
}

/// One line of one live surface that cites at least one ADR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationLine {
    /// Repo-relative path of the citing file.
    pub path: String,
    /// 1-indexed line number.
    pub line: usize,
    /// Ids cited as a `…decisions/ADR-NNNN-…` PATH on this line.
    pub cited: Vec<String>,
    /// Ids named BARE on this line — the residual evidence of what the sentence is about.
    pub context: Vec<String>,
    /// Is this file an authority surface (governance contract), where a Rejected citation is law?
    pub authority_surface: bool,
}

/// Anti-vacuity floors and the citation-defect ceiling. All repo-specifics are DATA: another repo
/// adopts this gate by repointing these numbers and the caller-side path lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    /// Fewer ADR records than this means the ADR walk broke.
    pub min_adr_records: usize,
    /// Fewer live apexes than this means `docs/decisions/` was not read.
    pub min_live_apexes: usize,
    /// Fewer archived records declaring a successor than this means frontmatter parsing broke —
    /// the case that silently empties the ORACLE while leaving both censuses intact.
    pub min_archived_with_successor: usize,
    /// Fewer citation lines than this means the surface walk broke. Zero citations computes zero
    /// mismatches, which is the false green this gate exists to prevent.
    pub min_citation_lines: usize,
    /// Fewer authority surfaces observed than this means the surface list went stale — the rejected
    /// -authority rule would then silently check nothing.
    pub min_authority_surfaces: usize,
}

/// The census the floors are checked against. Counted by the kernel from the observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Census {
    /// Every ADR record observed, live and archived.
    pub adr_records: usize,
    /// Records under `docs/decisions/` — the apexes a citation may point at.
    pub live_apexes: usize,
    /// Archived records.
    pub archived: usize,
    /// Archived records with at least one successor edge from EITHER oracle side — their own
    /// `superseded_by`, or membership in some record's `supersedes`. Counting only the member side
    /// left this floor blind to the half of the oracle it claims to guard: apex-side parsing could
    /// return nothing while the count stayed comfortably above the floor.
    pub archived_with_successor: usize,
    /// Archived records with `status: Rejected`.
    pub rejected: usize,
    /// Observed citation lines. A line qualifies by carrying an ADR PATH citation, or by naming an
    /// ADR bare on an authority surface — the rejected-authority rule needs the latter, so the
    /// count is not "lines carrying a path citation" and must not be read as one.
    pub citation_lines: usize,
    /// Distinct authority-surface files observed.
    pub authority_surfaces: usize,
}

/// One gate finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Violation code.
    pub code: String,
    /// Where it is: `path:line` for citations, the ADR id for oracle findings.
    pub subject: String,
    /// Human-readable detail.
    pub detail: String,
    /// Does this finding fail the gate?
    pub blocking: bool,
}

/// The gate verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// Counted census over the observations.
    pub census: Census,
    /// Every finding, blocking and advisory.
    pub findings: Vec<Finding>,
}

impl Verdict {
    /// Does the gate fail? True iff any finding is blocking.
    #[must_use]
    pub fn failed(&self) -> bool {
        self.findings.iter().any(|finding| finding.blocking)
    }

    /// Only the blocking findings.
    #[must_use]
    pub fn blocking(&self) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.blocking).collect()
    }

    /// How many findings carry `code`.
    #[must_use]
    pub fn count(&self, code: &str) -> usize {
        self.findings.iter().filter(|f| f.code == code).count()
    }
}

/// Normalize an ADR id to `ADR-` plus four digits. `ADR-335` and `ADR-0335` are the same decision;
/// the archive contains both spellings, and treating them as different ids silently drops edges.
#[must_use]
pub fn normalize_id(digits: &str) -> Option<String> {
    if digits.is_empty() || digits.len() > 4 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(format!("ADR-{:0>4}", digits))
}

/// Extract the `---` delimited frontmatter head, or `None` when the document has none.
///
/// Deliberately strict about the OPENING delimiter being the first line: a `---` appearing later is
/// a horizontal rule or a YAML document separator in an example, and treating it as a frontmatter
/// start is how body prose gets parsed as metadata.
#[must_use]
pub fn frontmatter(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// Does this document DECLARE ITSELF an authority surface while `declared` omits it?
///
/// The rejected-authority rule is scoped to `authority_surfaces`, a hand-curated list, and a
/// hand-curated list can only ever check what someone remembered to put in it. The list held
/// `docs/AGENTS.md` — which mandates Rejected ADR-0347 as active doctrine — while
/// `docs/AGENTS-OPERATING-CONTRACT.md` asserted the SAME rejected doctrine under its own
/// `## ADR-0347` heading, one directory over, and was structurally invisible: the rule cannot fire
/// on a surface nobody declared. That is the omission half of the staleness problem, and the
/// declared-half guard (`every_declared_authority_surface_exists_and_was_scanned`) cannot see it,
/// because it iterates the very list that is incomplete.
///
/// The remedy is to stop trusting the list as the definition and derive candidates from what each
/// document says about ITSELF. `marker` is a whole frontmatter line, supplied as policy DATA
/// (`authority_surface_marker`), so another repo adopts this by repointing it. Read from the `---`
/// head only: the same string in body prose is a document TALKING about operating contracts, not
/// claiming to be one, and matching it there would manufacture surfaces.
///
/// Per-file rather than corpus-wide on purpose — the caller already streams every tracked file's
/// text through the walk, so a per-file answer keeps the 16.5k-file corpus out of memory.
#[must_use]
pub fn undeclared_authority_surface(
    path: &str,
    text: &str,
    marker: &str,
    declared: &[String],
) -> Option<String> {
    let head = frontmatter(text)?;
    if !head.lines().any(|line| line.trim() == marker) {
        return None;
    }
    if declared.iter().any(|surface| surface == path) {
        return None;
    }
    Some(path.to_owned())
}

/// A fail-closed frontmatter parse error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmatterError(pub String);

fn ids_in(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut ids = Vec::new();
    let mut index = 0usize;
    while index + 5 <= bytes.len() {
        if &bytes[index..index + 4] == b"ADR-" {
            let start = index + 4;
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if let Some(id) = normalize_id(&text[start..end]) {
                ids.push(id);
            }
            index = end.max(index + 4);
        } else {
            index += 1;
        }
    }
    ids
}

/// Parse the supersession frontmatter of one ADR document.
///
/// Reads ONLY the top-level `status`, `supersedes` and `superseded_by` keys of the frontmatter
/// head, in both inline (`[ADR-0700]`) and block (`\n  - ADR-0700`) forms. Body text, fenced code
/// and prose are never consulted.
///
/// # Errors
/// Returns an error when the document has no frontmatter, or when one of the three keys appears
/// more than once at top level — a duplicate key has no single meaning and must not be guessed.
pub fn parse_supersession(
    id: &str,
    path: &str,
    live: bool,
    text: &str,
) -> Result<AdrRecord, FrontmatterError> {
    let head = frontmatter(text)
        .ok_or_else(|| FrontmatterError(format!("{path}: no --- delimited frontmatter")))?;

    let mut status: Option<String> = None;
    let mut supersedes: Option<Vec<String>> = None;
    let mut superseded_by: Option<Vec<String>> = None;
    let mut active: Option<&'static str> = None;

    for raw in head.lines() {
        let indented_item = {
            let trimmed = raw.trim_start();
            // ANY indentation, not the exact two spaces the first version required. A
            // `supersedes:` or `superseded_by:` written at four-space indent used to fall
            // through to the blanket "skip anything indented" arm below and vanish with no
            // error, in a parser whose contract is to fail closed on what it cannot read.
            // The archive already carries 18 four-space list items today (against 5783
            // two-space); they sit under unrelated keys, so `active` is None for them and
            // the match below still ignores them exactly as before. What changes is only
            // that a supersession edge written that way is now SEEN.
            if raw.len() > trimmed.len() {
                trimmed.strip_prefix("- ")
            } else {
                None
            }
        };
        if let Some(item) = indented_item {
            // A block-list item belongs to whichever list key opened it, and to nothing otherwise.
            match active {
                Some("supersedes") => supersedes.get_or_insert_with(Vec::new).extend(ids_in(item)),
                Some("superseded_by") => superseded_by
                    .get_or_insert_with(Vec::new)
                    .extend(ids_in(item)),
                _ => {}
            }
            continue;
        }
        if raw.starts_with(' ') || raw.starts_with('\t') {
            continue;
        }
        let Some((key, value)) = raw.split_once(':') else {
            active = None;
            continue;
        };
        let value = value.trim();
        match key {
            "status" => {
                if status.is_some() {
                    return Err(FrontmatterError(format!("{path}: duplicate status key")));
                }
                status = Some(value.to_owned());
                active = None;
            }
            "supersedes" => {
                if supersedes.is_some() {
                    return Err(FrontmatterError(format!(
                        "{path}: duplicate supersedes key"
                    )));
                }
                supersedes = Some(ids_in(value));
                active = Some("supersedes");
            }
            "superseded_by" => {
                if superseded_by.is_some() {
                    return Err(FrontmatterError(format!(
                        "{path}: duplicate superseded_by key"
                    )));
                }
                superseded_by = Some(ids_in(value));
                active = Some("superseded_by");
            }
            _ => active = None,
        }
    }

    let mut supersedes = supersedes.unwrap_or_default();
    let mut superseded_by = superseded_by.unwrap_or_default();
    // A record never supersedes itself; the archive contains at least one self-referential entry
    // and a self-edge turns closure into an infinite loop dressed up as data.
    supersedes.retain(|other| other != id);
    superseded_by.retain(|other| other != id);
    supersedes.sort();
    supersedes.dedup();
    superseded_by.sort();
    superseded_by.dedup();

    Ok(AdrRecord {
        id: id.to_owned(),
        path: path.to_owned(),
        live,
        status: status.unwrap_or_default(),
        supersedes,
        superseded_by,
    })
}

/// Split one line into ADR ids cited as a PATH and ADR ids named BARE.
///
/// The distinction is the whole detection mechanism. The de-stale pass rewrote PATHS; it left the
/// surrounding sentence — and the bare id in it — untouched. A bare id that disagrees with the path
/// beside it is the residual evidence of what the citation was supposed to mean.
#[must_use]
pub fn scan_line(line: &str) -> (Vec<String>, Vec<String>) {
    let bytes = line.as_bytes();
    let mut cited = Vec::new();
    let mut context = Vec::new();
    // Ids this line writes as an `adr-archive/` path. The line is thereby STATING they are
    // historical, which is not evidence about which apex the sentence is about.
    let mut archived_here: Vec<String> = Vec::new();
    let mut index = 0usize;
    while index + 5 <= bytes.len() {
        if &bytes[index..index + 4] != b"ADR-" {
            index += 1;
            continue;
        }
        let start = index + 4;
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if let Some(id) = normalize_id(&line[start..end]) {
            let before = &line[..index];
            if before.ends_with("decisions/") {
                cited.push(id);
            } else if before.ends_with("adr-archive/") {
                // Collected, then subtracted from `context` in a post-pass below. A
                // per-occurrence skip is NOT enough: a markdown link writes the id twice,
                // once as bare text and once in the path — `[ADR-0111](docs/adr-archive/
                // ADR-0111-x.md)` — and the bare occurrence reaches `context` before the
                // path one is ever seen. The first version of this fix skipped only the
                // path and the regression test below caught it.
                archived_here.push(id);
            } else {
                context.push(id);
            }
        }
        index = end.max(index + 4);
    }
    cited.sort();
    cited.dedup();
    context.sort();
    context.dedup();
    context.retain(|id| !cited.contains(id));
    // An id the line writes as an archive path has been declared historical BY THIS LINE.
    // Leaving it in `context` made `evaluate` read it as residual evidence of the
    // sentence's subject, so a correct line naming a historical ADR beside a live apex
    // fired a mismatch. `known_limitations` promises ambiguous attribution is skipped
    // rather than guessed; here the line has disambiguated itself and the gate guessed.
    context.retain(|id| !archived_here.contains(id));
    (cited, context)
}

/// Where an ADR's supersession chain lands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// The chain reaches exactly one live apex.
    Live(String),
    /// No record with this id was observed.
    Unknown,
    /// The chain stops at an archived record with no successor.
    Dead(String),
    /// The chain reaches more than one distinct live apex.
    Ambiguous(Vec<String>),
    /// The chain revisits a record.
    Cycle,
}

/// The both-sides supersession oracle: records indexed by id, plus the apex-side reverse edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Oracle {
    records: BTreeMap<String, AdrRecord>,
    /// `member -> {records whose supersedes list names member}`.
    claimed_by: BTreeMap<String, BTreeSet<String>>,
}

impl Oracle {
    /// Build the oracle from every observed record.
    ///
    /// On an id collision a LIVE record always wins, regardless of input order. Collecting into the
    /// map let the last record seen win, and the caller appends the archive after the apexes, so an
    /// archived record sharing an id silently replaced a live apex — the apex then resolved as an
    /// archived member and every citation of it became unresolvable. Order-independence here is
    /// what makes [`CODE_DUPLICATE_ID`] a report rather than a behaviour change.
    #[must_use]
    pub fn new(records: &[AdrRecord]) -> Self {
        let mut claimed_by: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for record in records {
            for member in &record.supersedes {
                claimed_by
                    .entry(member.clone())
                    .or_default()
                    .insert(record.id.clone());
            }
        }
        let mut by_id: BTreeMap<String, AdrRecord> = BTreeMap::new();
        for record in records {
            match by_id.get(&record.id) {
                Some(existing) if existing.live || !record.live => {}
                _ => {
                    by_id.insert(record.id.clone(), record.clone());
                }
            }
        }
        Self {
            records: by_id,
            claimed_by,
        }
    }

    /// Normalized ids observed on more than one record, with the paths that carry them.
    #[must_use]
    pub fn duplicate_ids(records: &[AdrRecord]) -> BTreeMap<String, Vec<String>> {
        let mut paths: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for record in records {
            paths
                .entry(record.id.clone())
                .or_default()
                .push(record.path.clone());
        }
        paths.retain(|_, seen| seen.len() > 1);
        paths
    }

    /// The record for `id`, if observed.
    #[must_use]
    pub fn record(&self, id: &str) -> Option<&AdrRecord> {
        self.records.get(id)
    }

    /// Successor edges from BOTH sides: the member's own declaration UNION every record claiming it.
    #[must_use]
    pub fn successors(&self, id: &str) -> BTreeSet<String> {
        let mut out: BTreeSet<String> = self
            .records
            .get(id)
            .map(|record| record.superseded_by.iter().cloned().collect())
            .unwrap_or_default();
        if let Some(claimers) = self.claimed_by.get(id) {
            out.extend(claimers.iter().cloned());
        }
        out.remove(id);
        out
    }

    /// Walk the union closure until a live apex is reached.
    #[must_use]
    pub fn resolve(&self, id: &str) -> Resolution {
        let mut on_path = BTreeSet::new();
        self.walk(id, &mut on_path)
    }

    /// `on_path` is the ids on the CURRENT branch, popped on unwind — not every id ever visited.
    ///
    /// Sharing one cumulative set across sibling branches made a DAG indistinguishable from a
    /// cycle: `A -> B -> D` and `A -> C -> D` re-visits D on the second branch, which reported
    /// `Cycle` for a graph that has none. Verdicts survived it, because `Cycle` and `Dead` are both
    /// unresolvable, but the `adr_closure_unresolvable` detail text named the wrong diagnosis on
    /// real broken chains, and `resolve` is public and was order-dependent.
    //
    // Cost ceiling: re-walking a shared successor once per path is exponential in a dense DAG. The
    // ADR graph is a shallow forest of short chains, so this is not memoized; add a memo keyed by
    // id if a corpus ever makes it bite.
    fn walk(&self, id: &str, on_path: &mut BTreeSet<String>) -> Resolution {
        if !on_path.insert(id.to_owned()) {
            return Resolution::Cycle;
        }
        let resolution = self.walk_from(id, on_path);
        on_path.remove(id);
        resolution
    }

    fn walk_from(&self, id: &str, on_path: &mut BTreeSet<String>) -> Resolution {
        let Some(record) = self.records.get(id) else {
            return Resolution::Unknown;
        };
        if record.live {
            return Resolution::Live(id.to_owned());
        }
        let successors = self.successors(id);
        if successors.is_empty() {
            return Resolution::Dead(id.to_owned());
        }
        let mut reached = BTreeSet::new();
        let mut cycled = false;
        for successor in &successors {
            match self.walk(successor, on_path) {
                Resolution::Live(apex) => {
                    reached.insert(apex);
                }
                Resolution::Ambiguous(apexes) => reached.extend(apexes),
                Resolution::Cycle => cycled = true,
                Resolution::Unknown | Resolution::Dead(_) => {}
            }
        }
        match reached.len() {
            0 if cycled => Resolution::Cycle,
            0 => Resolution::Dead(id.to_owned()),
            1 => Resolution::Live(reached.into_iter().next().unwrap_or_default()),
            _ => Resolution::Ambiguous(reached.into_iter().collect()),
        }
    }
}

fn census(records: &[AdrRecord], citations: &[CitationLine], oracle: &Oracle) -> Census {
    let archived: Vec<&AdrRecord> = records.iter().filter(|r| !r.live).collect();
    let surfaces: BTreeSet<&str> = citations
        .iter()
        .filter(|c| c.authority_surface)
        .map(|c| c.path.as_str())
        .collect();
    Census {
        adr_records: records.len(),
        live_apexes: records.iter().filter(|r| r.live).count(),
        archived: archived.len(),
        archived_with_successor: archived
            .iter()
            .filter(|r| !oracle.successors(&r.id).is_empty())
            .count(),
        rejected: archived.iter().filter(|r| r.status == "Rejected").count(),
        citation_lines: citations.len(),
        authority_surfaces: surfaces.len(),
    }
}

/// Evaluate observed ADR records and citation lines against the frozen policy.
///
/// Pure: every repo fact arrives as an argument.
#[must_use]
pub fn evaluate(records: &[AdrRecord], citations: &[CitationLine], policy: &Policy) -> Verdict {
    let oracle = Oracle::new(records);
    let census = census(records, citations, &oracle);
    let mut findings = Vec::new();

    for (id, paths) in Oracle::duplicate_ids(records) {
        findings.push(Finding {
            code: CODE_DUPLICATE_ID.to_owned(),
            subject: id.clone(),
            detail: format!(
                "{id} is carried by {} records ({}) — the oracle keys by id, so all but one are \
                 invisible to closure while every census still counts them",
                paths.len(),
                paths.join(", ")
            ),
            blocking: true,
        });
    }

    // Anti-vacuity FIRST. Every verdict below is computed by counting violations, so a walk that
    // saw nothing counts zero of them and reads as a flawless corpus.
    let mut floor = |observed: usize, minimum: usize, what: &str| {
        if observed < minimum {
            findings.push(Finding {
                code: CODE_VACUOUS_SCAN.to_owned(),
                subject: what.to_owned(),
                detail: format!(
                    "observed only {observed} {what}, expected at least {minimum} — the walk is \
                     broken, so a clean result is not evidence"
                ),
                blocking: true,
            });
        }
    };
    floor(census.adr_records, policy.min_adr_records, "ADR records");
    floor(census.live_apexes, policy.min_live_apexes, "live apexes");
    floor(
        census.archived_with_successor,
        policy.min_archived_with_successor,
        "archived ADRs declaring a successor",
    );
    floor(
        census.citation_lines,
        policy.min_citation_lines,
        "ADR citation lines",
    );
    floor(
        census.authority_surfaces,
        policy.min_authority_surfaces,
        "authority surfaces",
    );

    // Oracle health: asymmetry, dead chains, ambiguity. These are checked before the citations
    // because a citation verdict is only as good as the oracle behind it.
    for record in records {
        for successor in &record.superseded_by {
            let acknowledged = oracle
                .record(successor)
                .is_some_and(|s| s.supersedes.iter().any(|m| m == &record.id));
            if !acknowledged {
                findings.push(Finding {
                    code: CODE_ASYMMETRY.to_owned(),
                    subject: record.id.clone(),
                    detail: format!(
                        "{} declares superseded_by {successor}, but {successor} does not list it in \
                         supersedes",
                        record.id
                    ),
                    blocking: true,
                });
            }
        }
        for member in &record.supersedes {
            let acknowledged = oracle
                .record(member)
                .is_some_and(|m| m.superseded_by.iter().any(|s| s == &record.id));
            if !acknowledged {
                findings.push(Finding {
                    code: CODE_ASYMMETRY.to_owned(),
                    subject: member.clone(),
                    detail: format!(
                        "{} lists {member} in supersedes, but {member} does not declare \
                         superseded_by {}",
                        record.id, record.id
                    ),
                    blocking: true,
                });
            }
        }
        if record.live || record.status == "Rejected" {
            continue;
        }
        match oracle.resolve(&record.id) {
            Resolution::Live(_) => {}
            Resolution::Ambiguous(apexes) => findings.push(Finding {
                code: CODE_AMBIGUOUS_CLOSURE.to_owned(),
                subject: record.id.clone(),
                detail: format!(
                    "closure reaches {} distinct apexes: {apexes:?}",
                    apexes.len()
                ),
                blocking: false,
            }),
            other => findings.push(Finding {
                code: CODE_UNRESOLVABLE.to_owned(),
                subject: record.id.clone(),
                detail: format!(
                    "status {} but the successor closure reaches no live apex ({other:?}) — every \
                     citation of it is unrepairable",
                    record.status
                ),
                blocking: true,
            }),
        }
    }

    for citation in citations {
        let at = format!("{}:{}", citation.path, citation.line);

        // BOTH halves. `scan_line` removes an id from `context` when the same line also writes it
        // as a `decisions/` path, so the strongest instances — a surface that both NAMES and LINKS
        // the decision it mandates — carry the id only in `cited` and were invisible here.
        if citation.authority_surface {
            for id in citation.cited.iter().chain(citation.context.iter()) {
                // LIFECYCLE, not location. `record.live` answers "does this path resolve to a
                // file" and is the terminator for CODE_DANGLING_CITATION; authority is a separate
                // question and gets a separate rule rather than an overload of that predicate.
                // See NON_ENFORCING_STATUSES for why this is not `!= "Accepted"`.
                let Some(record) = oracle.record(id) else {
                    continue;
                };
                if !NON_ENFORCING_STATUSES.contains(&record.status.as_str()) {
                    continue;
                }
                findings.push(Finding {
                    code: CODE_REJECTED_AUTHORITY.to_owned(),
                    subject: at.clone(),
                    detail: format!(
                        "authority surface cites {id}, whose status is {} — a {} decision is not \
                         implement authority",
                        record.status,
                        record.status.to_lowercase()
                    ),
                    blocking: true,
                });
            }
        }

        // A `decisions/` path naming anything that is not a live apex does not resolve to a file
        // under `docs/decisions/`. Reported per id, so a line carrying several links names the
        // broken one instead of leaving a repairer to guess which.
        for id in &citation.cited {
            match oracle.record(id) {
                Some(record) if record.live => {}
                Some(record) => findings.push(Finding {
                    code: CODE_DANGLING_CITATION.to_owned(),
                    subject: at.clone(),
                    detail: format!(
                        "cites {id} as a decisions/ path, but {id} is an archived member ({}) with \
                         status {} — nothing lives at that path",
                        record.path, record.status
                    ),
                    blocking: true,
                }),
                None => findings.push(Finding {
                    code: CODE_DANGLING_CITATION.to_owned(),
                    subject: at.clone(),
                    detail: format!(
                        "cites {id} as a decisions/ path, but no ADR record with that id was \
                         observed anywhere in the corpus"
                    ),
                    blocking: true,
                }),
            }
        }

        // A line citing more than one apex cannot be attributed, and one citing none has nothing to
        // check. Both are skipped rather than guessed: guessing is what produced the false
        // positives an apex-side-only map generated.
        let [cited] = citation.cited.as_slice() else {
            continue;
        };
        // Not live: already reported as dangling just above. The mismatch rule compares a cited
        // APEX against the sentence, and has nothing to say about a broken path.
        if !oracle.record(cited).is_some_and(|r| r.live) {
            continue;
        }
        let mut resolved: Vec<(&String, String)> = Vec::new();
        let mut ambiguous = false;
        for id in &citation.context {
            // A bare id that is ITSELF a live apex resolves to itself and carries no supersession
            // evidence about the path beside it. Counting it made every apex-to-apex cross
            // reference — "see ADR-0709" on a line linking ADR-0700 — a mismatch with no way to
            // suppress it. The defect class this gate detects is a rewritten path contradicted by
            // an ARCHIVED id whose closure went elsewhere.
            if oracle.record(id).is_some_and(|r| r.live) {
                continue;
            }
            match oracle.resolve(id) {
                Resolution::Live(apex) => resolved.push((id, apex)),
                Resolution::Ambiguous(_) => ambiguous = true,
                _ => {}
            }
        }
        // THE FALSE-POSITIVE GUARD. A line often names several ADRs — a range endpoint, a lineage,
        // a "see also". If ANY of them resolves to the cited apex, the citation is consistent with
        // what the sentence is about and is not a defect. Only when every resolvable named ADR
        // points somewhere else has the rewrite genuinely lost the meaning.
        if ambiguous || resolved.is_empty() || resolved.iter().any(|(_, apex)| apex == cited) {
            continue;
        }
        let elsewhere: Vec<String> = resolved
            .iter()
            .map(|(id, apex)| format!("{id}->{apex}"))
            .collect();
        findings.push(Finding {
            code: CODE_CITATION_MISMATCH.to_owned(),
            subject: at,
            detail: format!(
                "cites {cited} but every ADR named on the line resolves elsewhere: {}",
                elsewhere.join(", ")
            ),
            blocking: true,
        });
    }

    Verdict { census, findings }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REGRESSION, review of PR #1616: a supersession list written at four-space indent
    // used to be silently dropped. FAILS before the indented_item change: the block-item
    // branch matched only the exact prefix "  - ", so these two ids never reached
    // superseded_by and the record read as having no successor at all.
    #[test]
    fn a_four_space_supersession_list_is_parsed() {
        let doc =
            "---\nstatus: Superseded\nsuperseded_by:\n    - ADR-0700\n    - ADR-0701\n---\nbody\n";
        let parsed = parse_supersession("ADR-0001", "docs/adr-archive/ADR-0001-x.md", false, doc)
            .expect("parses");
        assert_eq!(
            parsed.superseded_by,
            vec!["ADR-0700".to_owned(), "ADR-0701".to_owned()],
            "a four-space block list must be READ, not silently skipped by a parser whose \
             contract is to fail closed"
        );
    }

    // And the nested-key case must stay unaffected: a four-space list under some OTHER
    // key still belongs to nothing, because `active` is None for it.
    #[test]
    fn a_four_space_list_under_an_unrelated_key_is_still_ignored() {
        let doc = "---\nstatus: Superseded\nci_lanes:\n    - ADR-0700\n---\nbody\n";
        let parsed = parse_supersession("ADR-0002", "docs/adr-archive/ADR-0002-x.md", false, doc)
            .expect("parses");
        assert!(
            parsed.superseded_by.is_empty(),
            "a list under an unrelated key claims nothing"
        );
    }

    // REGRESSION, review of PR #1616: an `adr-archive/` link is the sentence stating the
    // ADR is historical. Routing it to `context` made it residual evidence of the
    // sentence's subject, so a CORRECT line naming a historical ADR alongside a live apex
    // fired a mismatch. FAILS before the fix: ADR-0111 appeared in context.
    #[test]
    fn an_archive_link_is_neither_cited_nor_context() {
        let line = "historical: [ADR-0111](docs/adr-archive/ADR-0111-x.md); current rule is \
                    [ADR-0709](docs/decisions/ADR-0709-general-live-apex.md)";
        let (cited, context) = scan_line(line);
        assert_eq!(
            cited,
            vec!["ADR-0709".to_owned()],
            "only the decisions/ path is a citation"
        );
        assert!(
            !context.contains(&"ADR-0111".to_owned()),
            "an archive link states the ADR is historical; it is not evidence about the apex"
        );
    }

    fn archived(id: &str, superseded_by: &[&str]) -> AdrRecord {
        AdrRecord {
            id: id.to_owned(),
            path: format!("docs/adr-archive/{id}-x.md"),
            live: false,
            status: "Superseded".to_owned(),
            supersedes: Vec::new(),
            superseded_by: superseded_by.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    fn apex(id: &str, supersedes: &[&str]) -> AdrRecord {
        AdrRecord {
            id: id.to_owned(),
            path: format!("docs/decisions/{id}-live-apex.md"),
            live: true,
            status: "Accepted".to_owned(),
            supersedes: supersedes.iter().map(|s| (*s).to_owned()).collect(),
            superseded_by: Vec::new(),
        }
    }

    fn cite(path: &str, cited: &[&str], context: &[&str], authority: bool) -> CitationLine {
        CitationLine {
            path: path.to_owned(),
            line: 1,
            cited: cited.iter().map(|s| (*s).to_owned()).collect(),
            context: context.iter().map(|s| (*s).to_owned()).collect(),
            authority_surface: authority,
        }
    }

    fn permissive() -> Policy {
        Policy {
            min_adr_records: 0,
            min_live_apexes: 0,
            min_archived_with_successor: 0,
            min_citation_lines: 0,
            min_authority_surfaces: 0,
        }
    }

    // Symmetric two-hop chain, the shape the real corpus uses: 0349 -> 0515 -> 0700.
    fn chain() -> Vec<AdrRecord> {
        let mut member = archived("ADR-0349", &["ADR-0515"]);
        let mut middle = archived("ADR-0515", &["ADR-0700"]);
        member.supersedes = Vec::new();
        middle.supersedes = vec!["ADR-0349".to_owned()];
        let mut top = apex("ADR-0700", &["ADR-0515"]);
        top.supersedes = vec!["ADR-0515".to_owned()];
        vec![member, middle, top, apex("ADR-0709", &[])]
    }

    #[test]
    fn ids_normalize_across_both_spellings_in_the_archive() {
        assert_eq!(normalize_id("335").as_deref(), Some("ADR-0335"));
        assert_eq!(normalize_id("0335").as_deref(), Some("ADR-0335"));
        assert_eq!(normalize_id("00335"), None);
        assert_eq!(normalize_id(""), None);
    }

    #[test]
    fn closure_chains_through_an_archived_intermediate() {
        let oracle = Oracle::new(&chain());
        assert_eq!(
            oracle.resolve("ADR-0349"),
            Resolution::Live("ADR-0700".to_owned())
        );
    }

    // The apex-side-only map is INCOMPLETE: drop the member's own declaration and the edge must
    // still exist, because the successor claims it. This is the "both sides" requirement.
    #[test]
    fn the_member_side_alone_and_the_apex_side_alone_each_miss_edges() {
        let mut apex_side_only = chain();
        apex_side_only[0].superseded_by.clear();
        apex_side_only[1].superseded_by.clear();
        assert_eq!(
            Oracle::new(&apex_side_only).resolve("ADR-0349"),
            Resolution::Live("ADR-0700".to_owned()),
            "apex supersedes lists alone must still carry the chain"
        );

        let mut member_side_only = chain();
        member_side_only[1].supersedes.clear();
        member_side_only[2].supersedes.clear();
        assert_eq!(
            Oracle::new(&member_side_only).resolve("ADR-0349"),
            Resolution::Live("ADR-0700".to_owned()),
            "member superseded_by declarations alone must still carry the chain"
        );
    }

    #[test]
    fn a_supersession_cycle_does_not_hang() {
        let records = vec![
            archived("ADR-0001", &["ADR-0002"]),
            archived("ADR-0002", &["ADR-0001"]),
        ];
        assert_eq!(Oracle::new(&records).resolve("ADR-0001"), Resolution::Cycle);
    }

    // THE META-TEST. A synthetic bad citation is injected into an otherwise clean corpus and the
    // gate must catch it. Without this the gate can rot into permanently green and nobody notices.
    #[test]
    fn a_new_bad_citation_fails_the_gate() {
        let good = cite("docs/x.md", &["ADR-0700"], &["ADR-0349"], false);
        let clean = evaluate(&chain(), std::slice::from_ref(&good), &permissive());
        assert!(!clean.failed(), "control corpus must be green: {clean:?}");

        // Inject exactly the measured defect: the path was rewritten to the wrong apex while the
        // sentence still names the ADR it is about.
        let bad = cite("docs/x.md", &["ADR-0709"], &["ADR-0349"], false);
        let verdict = evaluate(&chain(), &[good, bad], &permissive());
        assert!(
            verdict.failed(),
            "the injected bad citation must fail closed"
        );
        assert_eq!(verdict.count(CODE_CITATION_MISMATCH), 1);
    }

    // The Bun regression class in one assertion: the path is VALID and the file EXISTS, so nothing
    // syntactic separates the two citations. Only the closure does.
    #[test]
    fn both_apexes_are_real_files_so_only_meaning_separates_them() {
        let records = chain();
        assert!(records.iter().any(|r| r.id == "ADR-0700" && r.live));
        assert!(records.iter().any(|r| r.id == "ADR-0709" && r.live));
    }

    // The 17-false-positive class: a line naming a RANGE ("the ADR-0516..ADR-0535 cluster") where
    // one endpoint resolves to the cited apex and the other does not. Not a defect.
    #[test]
    fn a_line_naming_several_adrs_passes_when_one_resolves_to_the_cited_apex() {
        let mut records = chain();
        let mut other = archived("ADR-0535", &["ADR-0709"]);
        other.supersedes.clear();
        records[3].supersedes = vec!["ADR-0535".to_owned()];
        records.push(other);
        let line = cite("docs/x.md", &["ADR-0700"], &["ADR-0349", "ADR-0535"], false);
        let verdict = evaluate(&records, &[line], &permissive());
        assert_eq!(verdict.count(CODE_CITATION_MISMATCH), 0);
    }

    #[test]
    fn a_line_citing_two_apexes_is_not_attributed() {
        let line = cite("docs/x.md", &["ADR-0700", "ADR-0709"], &["ADR-0349"], false);
        assert_eq!(
            evaluate(&chain(), &[line], &permissive()).count(CODE_CITATION_MISMATCH),
            0
        );
    }

    #[test]
    fn an_unknown_context_id_never_accuses_a_citation() {
        let line = cite("docs/x.md", &["ADR-0709"], &["ADR-9999"], false);
        assert_eq!(
            evaluate(&chain(), &[line], &permissive()).count(CODE_CITATION_MISMATCH),
            0
        );
    }

    /// A LIVE apex whose status is `Proposed` is authority-invisible on the old `== "Rejected"`
    /// rule, and being live it is also invisible to the dangling-path rule — so before this it
    /// produced no finding of any kind. This is the exact shape ADR-0710 introduced: the first
    /// non-`Accepted` file ever placed under `docs/decisions/`.
    #[test]
    fn an_authority_surface_citing_a_proposed_apex_fails_closed() {
        let mut records = chain();
        let mut proposed = apex("ADR-0710", &[]);
        proposed.status = "Proposed".to_owned();
        records.push(proposed);

        let cited_as_path = cite("CLAUDE.md", &["ADR-0710"], &[], true);
        let verdict = evaluate(
            &records,
            std::slice::from_ref(&cited_as_path),
            &permissive(),
        );
        assert_eq!(verdict.count(CODE_REJECTED_AUTHORITY), 1);
        assert!(verdict.failed());
        assert_eq!(
            verdict.count(CODE_DANGLING_CITATION),
            0,
            "it is LIVE: the path resolves, so widening lifecycle must not manufacture a \
             broken-link accusation against a file that demonstrably exists"
        );

        let named_only = cite("AGENTS.md", &[], &["ADR-0710"], true);
        assert_eq!(
            evaluate(&records, &[named_only], &permissive()).count(CODE_REJECTED_AUTHORITY),
            1,
            "both halves of the line, exactly as the Rejected case already requires"
        );
    }

    /// The narrowing that keeps this rule honest. `Superseded` resolves through `superseded_by` to
    /// a live successor and is cited as HISTORY on the authority surfaces deliberately — root
    /// CLAUDE.md's `historical_substrate_adrs` block is 21 such ids. A `!= "Accepted"` predicate
    /// would accuse every one of them.
    #[test]
    fn an_authority_surface_citing_a_superseded_adr_is_not_accused() {
        let records = chain(); // ADR-0349 and ADR-0515 are Superseded with successors
        let line = cite("CLAUDE.md", &[], &["ADR-0515"], true);
        assert_eq!(
            evaluate(&records, &[line], &permissive()).count(CODE_REJECTED_AUTHORITY),
            0,
            "superseded is resolvable history, not a lifecycle defect — supersession has its own \
             rules and must not be overloaded onto the authority predicate"
        );
    }

    #[test]
    fn an_authority_surface_citing_a_rejected_adr_fails_closed() {
        let mut records = chain();
        let mut rejected = archived("ADR-0111", &[]);
        rejected.status = "Rejected".to_owned();
        records.push(rejected);
        let line = cite("CLAUDE.md", &[], &["ADR-0111"], true);
        let verdict = evaluate(&records, std::slice::from_ref(&line), &permissive());
        assert_eq!(verdict.count(CODE_REJECTED_AUTHORITY), 1);
        assert!(verdict.failed());

        let elsewhere = CitationLine {
            authority_surface: false,
            ..line
        };
        assert_eq!(
            evaluate(&records, &[elsewhere], &permissive()).count(CODE_REJECTED_AUTHORITY),
            0,
            "the rule must be scoped to authority surfaces, not every mention in the repo"
        );
    }

    #[test]
    fn a_one_sided_supersession_edge_fails_closed() {
        let mut records = chain();
        records[2].supersedes.clear();
        let verdict = evaluate(&records, &[], &permissive());
        assert!(verdict.failed());
        assert!(verdict.count(CODE_ASYMMETRY) >= 1);
    }

    #[test]
    fn a_chain_reaching_no_apex_fails_closed() {
        let records = vec![archived("ADR-0349", &[]), apex("ADR-0700", &[])];
        let verdict = evaluate(&records, &[], &permissive());
        assert!(verdict.failed());
        assert_eq!(verdict.count(CODE_UNRESOLVABLE), 1);
    }

    // A rejected ADR is not "unresolvable": it was never superseded, it was refused. Reporting it
    // as a broken chain would bury the 16 real rejections in noise.
    #[test]
    fn a_rejected_adr_is_not_reported_as_an_unresolvable_chain() {
        let mut rejected = archived("ADR-0111", &[]);
        rejected.status = "Rejected".to_owned();
        let verdict = evaluate(&[rejected, apex("ADR-0700", &[])], &[], &permissive());
        assert_eq!(verdict.count(CODE_UNRESOLVABLE), 0);
    }

    #[test]
    fn an_ambiguous_closure_is_advisory_and_stands_the_mismatch_rule_down() {
        let mut member = archived("ADR-0349", &["ADR-0700", "ADR-0709"]);
        member.supersedes.clear();
        let mut a = apex("ADR-0700", &["ADR-0349"]);
        let mut b = apex("ADR-0709", &["ADR-0349"]);
        a.supersedes = vec!["ADR-0349".to_owned()];
        b.supersedes = vec!["ADR-0349".to_owned()];
        let records = vec![member, a, b];
        let line = cite("docs/x.md", &["ADR-0700"], &["ADR-0349"], false);
        let verdict = evaluate(&records, &[line], &permissive());
        assert_eq!(verdict.count(CODE_AMBIGUOUS_CLOSURE), 1);
        assert_eq!(verdict.count(CODE_CITATION_MISMATCH), 0);
    }

    // THE FALSE GREEN. An empty walk computes zero mismatches, which is indistinguishable from a
    // perfect corpus without the floors.
    #[test]
    fn an_empty_scan_fails_closed_instead_of_reporting_a_clean_corpus() {
        let policy = Policy {
            min_adr_records: 100,
            min_live_apexes: 10,
            min_archived_with_successor: 100,
            min_citation_lines: 100,
            min_authority_surfaces: 1,
        };
        let verdict = evaluate(&[], &[], &policy);
        assert_eq!(verdict.count(CODE_CITATION_MISMATCH), 0);
        assert!(verdict.failed(), "a vacuous scan must not pass");
        assert_eq!(verdict.count(CODE_VACUOUS_SCAN), 5);
    }

    // THE CASE THE RECORD AND CITATION FLOORS CANNOT SEE. Both censuses are intact — every ADR and
    // every citation was found — but frontmatter parsing silently returned no edges, so the oracle
    // is EMPTY and resolves nothing, so no citation can ever be called wrong.
    #[test]
    fn a_collapsed_oracle_fails_closed_while_both_census_floors_hold() {
        let policy = Policy {
            min_adr_records: 4,
            min_live_apexes: 2,
            min_archived_with_successor: 2,
            min_citation_lines: 1,
            min_authority_surfaces: 0,
        };
        let line = cite("docs/x.md", &["ADR-0709"], &["ADR-0349"], false);

        // Control: with the oracle intact this shape is red for the RIGHT reason.
        let intact = evaluate(&chain(), std::slice::from_ref(&line), &policy);
        assert_eq!(intact.count(CODE_VACUOUS_SCAN), 0);
        assert_eq!(intact.count(CODE_CITATION_MISMATCH), 1);

        let mut edgeless = chain();
        for record in &mut edgeless {
            record.supersedes.clear();
            record.superseded_by.clear();
        }
        let verdict = evaluate(&edgeless, &[line], &policy);
        assert_eq!(
            verdict.count(CODE_CITATION_MISMATCH),
            0,
            "an empty oracle cannot accuse anything — which is exactly why it needs a floor"
        );
        assert!(verdict.failed());
        assert_eq!(verdict.count(CODE_VACUOUS_SCAN), 1);
    }

    #[test]
    fn frontmatter_is_only_the_head_block() {
        assert_eq!(frontmatter("---\na: 1\n---\nbody\n"), Some("a: 1"));
        assert_eq!(frontmatter("body\n---\na: 1\n---\n"), None);
        assert_eq!(frontmatter("no frontmatter"), None);
    }

    // THE OMISSION GUARD, proven to FIRE without a filesystem. The live-tree binding asserts this
    // returns nothing; that assertion is only evidence if the function is capable of returning
    // something, and only the synthetic case can show that while the corpus is clean.
    //
    // Executed RED before the fix, on the real tree, by the live-tree binding
    // `a_document_declaring_itself_an_operating_contract_is_a_declared_surface` at
    // tests/adr_citation_closure.rs:640 — see the full panic text recorded there.
    #[test]
    fn a_document_declaring_the_marker_and_missing_from_the_list_is_reported() {
        const MARKER: &str = "doc_class: Operating-Contract";
        let doc = "---\ndoc_class: Operating-Contract\nauthority_tier: 2\n---\n# body\n";
        let declared = vec!["docs/AGENTS.md".to_owned()];

        assert_eq!(
            undeclared_authority_surface("docs/OTHER-CONTRACT.md", doc, MARKER, &declared),
            Some("docs/OTHER-CONTRACT.md".to_owned()),
            "a document that declares itself an operating contract and is not in the list is \
             exactly the invisible-surface case this guard exists for"
        );
        assert_eq!(
            undeclared_authority_surface("docs/AGENTS.md", doc, MARKER, &declared),
            None,
            "already declared"
        );
        assert_eq!(
            undeclared_authority_surface(
                "docs/notes.md",
                "---\ndoc_class: Reference\n---\nbody\n",
                MARKER,
                &declared
            ),
            None,
            "a different doc_class claims nothing"
        );
        assert_eq!(
            undeclared_authority_surface(
                "docs/README.md",
                "no frontmatter at all\n",
                MARKER,
                &declared
            ),
            None,
            "no frontmatter, no self-declaration"
        );
    }

    // The marker is read from the HEAD ONLY. A document that DISCUSSES operating contracts — this
    // repo has several, and the mapping document for this very goal is one — must not be promoted
    // into an authority surface by quoting the key in its prose. Body matching would silently widen
    // the rejected-authority rule onto commentary.
    #[test]
    fn the_marker_is_read_from_frontmatter_and_never_from_body_prose() {
        const MARKER: &str = "doc_class: Operating-Contract";
        assert_eq!(
            undeclared_authority_surface(
                "docs/commentary.md",
                "---\ndoc_class: Reference\n---\nSurfaces are keyed by `doc_class: \
                 Operating-Contract` in frontmatter.\ndoc_class: Operating-Contract\n",
                MARKER,
                &[]
            ),
            None,
            "the same line in the BODY is a document talking about surfaces, not claiming to be one"
        );
    }

    const HEAD: &str = "---\nstatus: Superseded\nsupersedes: []\nsuperseded_by: [ADR-0700]\n---\n";

    #[test]
    fn inline_and_block_list_forms_both_parse() {
        let inline = parse_supersession("ADR-0515", "p", false, HEAD).unwrap();
        assert_eq!(inline.superseded_by, ["ADR-0700"]);
        assert_eq!(inline.status, "Superseded");

        let block = parse_supersession(
            "ADR-0515",
            "p",
            false,
            "---\nsupersedes:\n  - ADR-0349\n  - ADR-0509\namends:\n  - ADR-0131\n---\n",
        )
        .unwrap();
        assert_eq!(block.supersedes, ["ADR-0349", "ADR-0509"]);
        assert!(
            block.superseded_by.is_empty(),
            "a block item must belong to the key that opened it, never to a later one"
        );
    }

    #[test]
    fn short_form_and_prose_wrapped_ids_parse() {
        let record = parse_supersession(
            "ADR-0500",
            "p",
            false,
            "---\nsuperseded_by: [ADR-335, ADR-562]\n---\n",
        )
        .unwrap();
        assert_eq!(record.superseded_by, ["ADR-0335", "ADR-0562"]);

        let partial = parse_supersession(
            "ADR-0500",
            "p",
            false,
            "---\nsupersedes:\n  - \"ADR-0015 (partial — the split only)\"\n---\n",
        )
        .unwrap();
        assert_eq!(partial.supersedes, ["ADR-0015"]);
    }

    // The exact false-match the brief names: an archived ADR whose BODY contains a fenced
    // `superseded_by:` block naming ITSELF. Grepping the file finds it; parsing frontmatter cannot.
    #[test]
    fn a_superseded_by_line_in_the_body_is_not_an_edge() {
        let text = "---\nstatus: Superseded\nsuperseded_by: [ADR-0709]\n---\n# body\n```\nsuperseded_by:\n  - ADR-0255-intelligence.md\n```\n";
        let record = parse_supersession("ADR-0255", "p", false, text).unwrap();
        assert_eq!(record.superseded_by, ["ADR-0709"]);
    }

    #[test]
    fn empty_tilde_and_prose_values_parse_as_no_edge() {
        for value in ["[]", "~", "none"] {
            let record = parse_supersession(
                "ADR-0500",
                "p",
                false,
                &format!("---\nsuperseded_by: {value}\n---\n"),
            )
            .unwrap();
            assert!(record.superseded_by.is_empty(), "{value}");
        }
    }

    #[test]
    fn a_self_edge_is_dropped_rather_than_looped_on() {
        let record = parse_supersession(
            "ADR-0255",
            "p",
            false,
            "---\nsuperseded_by: [ADR-0255]\n---\n",
        )
        .unwrap();
        assert!(record.superseded_by.is_empty());
    }

    #[test]
    fn a_duplicate_frontmatter_key_fails_closed() {
        assert!(
            parse_supersession(
                "ADR-0500",
                "p",
                false,
                "---\nsuperseded_by: [ADR-0700]\nsuperseded_by: [ADR-0709]\n---\n"
            )
            .is_err()
        );
        assert!(parse_supersession("ADR-0500", "p", false, "no frontmatter").is_err());
    }

    #[test]
    fn scan_line_separates_the_path_from_the_sentence() {
        let (cited, context) = scan_line(
            "  - docs/decisions/ADR-0709-general-live-apex.md # folded into ADR-0515 cloud-ci Tide",
        );
        assert_eq!(cited, ["ADR-0709"]);
        assert_eq!(context, ["ADR-0515"]);
    }

    #[test]
    fn scan_line_reads_relative_links_as_paths_and_link_text_as_context() {
        let (cited, context) =
            scan_line("[ADR-0346](decisions/ADR-0700-ci-admission-live-apex.md)");
        assert_eq!(cited, ["ADR-0700"]);
        assert_eq!(context, ["ADR-0346"]);
    }

    #[test]
    fn scan_line_drops_an_id_from_context_when_it_is_also_the_cited_path() {
        let (cited, context) = scan_line("see ADR-0700 at docs/decisions/ADR-0700-x.md");
        assert_eq!(cited, ["ADR-0700"]);
        assert!(context.is_empty());
    }

    #[test]
    fn scan_line_ignores_malformed_ids() {
        let (cited, context) = scan_line("ADR- ADR-12345 ADRS-0001");
        assert!(cited.is_empty());
        assert!(context.is_empty());
    }

    // ---- regressions ----

    // FINDINGS 1+2, the instance the gate could not see. `docs/AGENTS.md:57` writes each id TWICE —
    // `[ADR-0347](decisions/ADR-0347-…)` — so scan_line moves it out of `context` and into `cited`,
    // and a rule reading only `context` saw an empty list. Measured: this raised the live
    // rejected-authority count from 2 to 3.
    #[test]
    fn a_rejected_adr_cited_as_a_path_is_caught_not_only_when_named_bare() {
        let mut records = chain();
        let mut rejected = archived("ADR-0347", &[]);
        rejected.status = "Rejected".to_owned();
        records.push(rejected);

        // Exactly what scan_line produces for `[ADR-0347](decisions/ADR-0347-x.md)`: the id is in
        // `cited` and `context` is EMPTY, because scan_line drops the duplicate.
        let (cited, context) = scan_line("[ADR-0347](decisions/ADR-0347-governance-rename.md)");
        assert_eq!(cited, ["ADR-0347"], "the path half must be the cited id");
        assert!(context.is_empty(), "scan_line drops the bare duplicate");

        let line = cite("docs/AGENTS.md", &["ADR-0347"], &[], true);
        let verdict = evaluate(&records, &[line], &permissive());
        assert_eq!(
            verdict.count(CODE_REJECTED_AUTHORITY),
            1,
            "an authority surface that both NAMES and LINKS a Rejected ADR must fire"
        );
    }

    // FINDING 2. A `decisions/` path naming a non-apex resolves to no file; it used to be a silent
    // `continue` in the mismatch rule and produced no finding of any kind.
    #[test]
    fn a_decisions_path_that_resolves_to_nothing_is_reported() {
        let records = chain();

        let archived_target = cite("docs/x.md", &["ADR-0349"], &[], false);
        let verdict = evaluate(&records, &[archived_target], &permissive());
        assert_eq!(
            verdict.count(CODE_DANGLING_CITATION),
            1,
            "citing an ARCHIVED member as a decisions/ path is a dangling link"
        );

        let unknown = cite("docs/x.md", &["ADR-9999"], &[], false);
        assert_eq!(
            evaluate(&records, &[unknown], &permissive()).count(CODE_DANGLING_CITATION),
            1,
            "citing an id with no record at all is a dangling link"
        );

        let apex_target = cite("docs/x.md", &["ADR-0700"], &[], false);
        assert_eq!(
            evaluate(&records, &[apex_target], &permissive()).count(CODE_DANGLING_CITATION),
            0,
            "a live apex is exactly what a decisions/ path is allowed to name"
        );
    }

    // FINDING 3. `resolve()` on a live id returns Live(itself), so a bare apex on a line citing a
    // DIFFERENT apex used to be read as evidence the path was wrong. Zero instances in the corpus
    // today, which is why nothing caught it — the first legitimate cross-reference would have been
    // blocked with no way to suppress it.
    #[test]
    fn naming_one_apex_beside_a_link_to_another_is_not_a_mismatch() {
        let line = cite("docs/x.md", &["ADR-0700"], &["ADR-0709"], false);
        let verdict = evaluate(&chain(), &[line], &permissive());
        assert_eq!(
            verdict.count(CODE_CITATION_MISMATCH),
            0,
            "an apex-to-apex cross reference carries no supersession evidence and must not accuse"
        );

        // The real defect class still fires: a bare ARCHIVED id whose closure lands elsewhere.
        let genuine = cite("docs/x.md", &["ADR-0709"], &["ADR-0349"], false);
        assert_eq!(
            evaluate(&chain(), &[genuine], &permissive()).count(CODE_CITATION_MISMATCH),
            1,
            "suppressing apex context must not disarm the rule it guards"
        );
    }

    // FINDING 6. Records collected into a map keyed by id let the LAST one win, and the caller
    // appends the archive after the apexes — so an archived record sharing an id silently displaced
    // a live apex while both censuses still counted two.
    #[test]
    fn a_duplicate_id_is_reported_and_never_lets_an_archive_shadow_an_apex() {
        let apex_record = apex("ADR-0700", &[]);
        let shadow = archived("ADR-0700", &[]);

        for order in [
            vec![apex_record.clone(), shadow.clone()],
            vec![shadow.clone(), apex_record.clone()],
        ] {
            let oracle = Oracle::new(&order);
            assert!(
                oracle.record("ADR-0700").is_some_and(|r| r.live),
                "a live apex must win an id collision regardless of input order"
            );
            assert_eq!(
                oracle.resolve("ADR-0700"),
                Resolution::Live("ADR-0700".to_owned())
            );
        }

        let verdict = evaluate(&[apex_record, shadow], &[], &permissive());
        assert_eq!(verdict.count(CODE_DUPLICATE_ID), 1);
        assert!(
            verdict.failed(),
            "a duplicate id must fail closed, not be silently resolved"
        );
    }

    // FINDING 8. One cumulative `seen` set across sibling branches made a DAG look like a cycle:
    // the second branch re-visits a node the first already entered. Both are unresolvable, so the
    // VERDICT survived it — but `adr_closure_unresolvable` then printed the wrong diagnosis.
    #[test]
    fn a_diamond_is_diagnosed_dead_not_cycle() {
        let mut top = archived("ADR-0001", &["ADR-0002", "ADR-0003"]);
        top.supersedes.clear();
        let left = archived("ADR-0002", &["ADR-0004"]);
        let right = archived("ADR-0003", &["ADR-0004"]);
        let bottom = archived("ADR-0004", &[]); // archived, no successor: a DEAD end, not a loop

        let oracle = Oracle::new(&[top, left, right, bottom]);
        assert_eq!(
            oracle.resolve("ADR-0001"),
            Resolution::Dead("ADR-0001".to_owned()),
            "re-reaching a node down a second branch is a DAG, not a cycle"
        );

        // A genuine loop must still be a loop.
        let looped = vec![
            archived("ADR-0010", &["ADR-0011"]),
            archived("ADR-0011", &["ADR-0010"]),
        ];
        assert_eq!(Oracle::new(&looped).resolve("ADR-0010"), Resolution::Cycle);
    }

    // FINDING 8, the other half: a diamond that DOES reach an apex resolves through it.
    #[test]
    fn a_diamond_reaching_one_apex_resolves_to_it() {
        let mut top = archived("ADR-0001", &["ADR-0002", "ADR-0003"]);
        top.supersedes.clear();
        let left = archived("ADR-0002", &["ADR-0700"]);
        let right = archived("ADR-0003", &["ADR-0700"]);
        let oracle = Oracle::new(&[top, left, right, apex("ADR-0700", &[])]);
        assert_eq!(
            oracle.resolve("ADR-0001"),
            Resolution::Live("ADR-0700".to_owned()),
            "two paths to the SAME apex is one destination, not an ambiguity"
        );
    }

    // FINDING 11. The floor exists to catch a silently-empty ORACLE, but counted only the member
    // side — so apex-side parsing could return nothing while the count sailed over the floor.
    #[test]
    fn archived_with_successor_counts_the_apex_side_too() {
        let mut member = archived("ADR-0349", &[]);
        member.superseded_by.clear(); // member side declares NOTHING
        let mut top = apex("ADR-0700", &["ADR-0349"]);
        top.supersedes = vec!["ADR-0349".to_owned()]; // only the apex claims the edge

        let verdict = evaluate(&[member, top], &[], &permissive());
        assert_eq!(
            verdict.census.archived_with_successor, 1,
            "an edge visible only from the apex side still means the oracle is not empty"
        );
    }
}
