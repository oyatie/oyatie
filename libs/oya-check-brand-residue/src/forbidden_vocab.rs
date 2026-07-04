//! Forbidden-vocab SHRINK-ONLY RATCHET census (boundary enforcement).
//!
//! ## What this is
//! The forbidden-vocab eradication (register #25; D-FORGE / D-FOUNDRY-CLARIFY) leaves a
//! large historical residue: ~19k+ line-occurrences of `foundry`, `forgejo`, `jenkins`,
//! `oya-vcs` live in prose, ADR bodies, and `.omc` state. Mass-scrubbing that residue would
//! churn 19k lines of history for no behavioural gain. The hyperscaler discipline is to
//! enforce at the BOUNDARY instead: freeze TODAY's residue as a baseline and make the
//! vocabulary structurally UN-GROW-ABLE — any NEW occurrence is RED, while the frozen
//! residue is tolerated and ages out as files are naturally cleaned.
//!
//! ## Mechanism (ONE canonical gate — the firewall ratchet, no parallel gate)
//! This module is the SSOT for the census; it does NOT introduce a second enforcement
//! engine. The `oya-cloud-ci-accounting-registry-app` producer calls [`census_findings`]
//! over the live corpus and freezes the result as the `cloud-ci-brand-residue` gate inside
//! the existing `gate-baseline.generated.json`. The existing `cloud-ci-firewall` ratchet
//! then blocks any key NOT in the baseline (NEW occurrence => RED) and auto-shrinks the
//! baseline as keys disappear (file cleaned => GREEN). Same compare-mode + ratchet-invariant
//! predicates, zero firewall code change.
//!
//! ## Granularity: per-(stem, file), not per-line
//! The ratchet key is `"<path>"` per `(stem, file)` finding — NOT `path:line`. Per-line keys
//! would churn on every edit above a residue line (line numbers shift => spurious
//! regressions + spurious "fixed"); per-file keys are STABLE under in-file edits, so editing
//! prose in an already-listed file stays GREEN and the set only shrinks when a file is fully
//! cleaned. The cardinality (one key per file containing the stem) is the diagnostic the
//! failing PR needs: it names the exact file that grew a new forbidden token.
//!
//! ## Carve-outs are DATA, not code ([`CARVE_OUT_RULES`])
//! - the deny-list patterns themselves (this crate's own source) — naming a stem is the
//!   deny-list, not residue;
//! - the catalog spec (`registry/catalog/oya-check-brand-residue.yaml`) — same reason;
//! - the Palantir-Foundry proper-noun prose (a line matching `/palantir/i`) — a legitimate
//!   competitor reference, not brand residue;
//! - the append-only audit chain (`evidence/audit-chain.jsonl`) — NEVER rewritten;
//! - the `_legacy-foundry/` archive — the intentional historical archive of the dropped
//!   work;
//! - the generated faces (`*.generated.json`) — produced by the gate machinery itself
//!   (which legitimately records the tokens it tracks); hand-editing them is its own
//!   `ci_inventory_registry_drift` RED.

use std::collections::{BTreeMap, BTreeSet};

/// A keyed census finding, identical in shape to each cloud-ci gate's `Finding`
/// (`{code, key}`) so the producer can fold it into the firewall baseline uniformly.
/// `code` is the per-stem code (`forbidden_foundry`, ...); `key` is the file path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
}

/// One forbidden-vocab stem: the lowercase substring to match and the firewall code it
/// freezes under. SSOT for the executable shrink-only set (distinct from the legacy
/// substring deny-list `FORBIDDEN_BRAND_TOKENS`, which is a separate born-blocking check
/// for tokens that are already at zero live occurrences).
#[derive(Debug, Clone, Copy)]
pub struct ForbiddenStem {
    /// Matched case-insensitively as a substring on each line.
    pub stem: &'static str,
    /// The `cloud-ci-brand-residue` gate code this stem freezes under.
    pub code: &'static str,
}

/// The four forbidden vocab stems the shrink-only ratchet freezes (register #25). Adding a
/// stem here widens the boundary — its current occurrences are baselined on the next regen,
/// and every later occurrence beyond the baseline is RED.
pub const FORBIDDEN_VOCAB_STEMS: &[ForbiddenStem] = &[
    ForbiddenStem { stem: "foundry", code: "forbidden_foundry" },
    ForbiddenStem { stem: "forgejo", code: "forbidden_forgejo" },
    ForbiddenStem { stem: "jenkins", code: "forbidden_jenkins" },
    ForbiddenStem { stem: "oya-vcs", code: "forbidden_oya-vcs" },
];

/// A carve-out rule (DATA, not a scanner branch). The census walks this table; matching the
/// path OR the line exempts the occurrence. Linus' rule: the exception lives in the table.
#[derive(Debug, Clone, Copy)]
pub struct CarveOutRule {
    pub kind: CarveOutKind,
    pub value: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarveOutKind {
    /// The whole file is exempt iff its repo-relative path starts with `value`.
    PathPrefix,
    /// The whole file is exempt iff its repo-relative path equals `value`.
    PathExact,
    /// The whole file is exempt iff its repo-relative path ends with `value`.
    PathSuffix,
    /// A single line is exempt iff (lower-cased) it contains `value` (proper-noun prose).
    LineContainsCi,
}

/// The carve-out DATA table. Path carve-outs drop the whole file; the `LineContainsCi`
/// carve-out drops only the matching line (so a file that legitimately cites
/// "Palantir Foundry" once but also carries real residue elsewhere is still partially
/// scanned). Order is irrelevant — a single match exempts.
pub const CARVE_OUT_RULES: &[CarveOutRule] = &[
    CarveOutRule {
        kind: CarveOutKind::PathPrefix,
        value: "libs/oya-check-brand-residue/",
        reason: "the deny-list patterns themselves are not residue",
    },
    CarveOutRule {
        kind: CarveOutKind::PathPrefix,
        value: "libs/oya-ci-config/",
        reason: "the config-era deny-list SSOT (forbidden-stem table + bundled disposition) — naming a stem here is the deny-list, not residue (same rationale as oya-check-brand-residue)",
    },
    CarveOutRule {
        kind: CarveOutKind::PathExact,
        value: "oya-ci.toml",
        reason: "the repo-root oya-ci config IS the deny-list (it declares the forbidden-stem table) — naming a stem here is the deny-list, not residue",
    },
    CarveOutRule {
        kind: CarveOutKind::PathExact,
        value: "registry/catalog/oya-check-brand-residue.yaml",
        reason: "the catalog deny-list spec is not residue",
    },
    CarveOutRule {
        kind: CarveOutKind::PathPrefix,
        value: "oya/intelligence/_legacy-foundry/",
        reason: "intentional historical archive of the dropped work",
    },
    CarveOutRule {
        kind: CarveOutKind::PathPrefix,
        value: "marketplace/facade/dev-cli/tests/",
        reason: "integration test fixtures that reference live repo contracts/openapi/foundry/ paths and fixture data strings — these are structural references to real contract paths, not brand residue; the file was moved from oya/developer-sdk/crates/oya-dev-cli/tests/ where it was already baselined",
    },
    CarveOutRule {
        kind: CarveOutKind::PathExact,
        value: "evidence/audit-chain.jsonl",
        reason: "append-only audit chain — NEVER rewritten",
    },
    CarveOutRule {
        kind: CarveOutKind::PathSuffix,
        value: ".generated.json",
        reason: "producer-generated faces record the tokens the gates track; a hand-edit is its own ci_inventory_registry_drift RED",
    },
    CarveOutRule {
        kind: CarveOutKind::LineContainsCi,
        value: "palantir",
        reason: "Palantir-Foundry is a competitor proper noun, not brand residue",
    },
];

/// One owned stem (the runtime, config-sourced form of [`ForbiddenStem`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedStem {
    pub stem: String,
    pub code: String,
}

/// One owned carve-out rule (the runtime, config-sourced form of [`CarveOutRule`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedCarveOut {
    pub kind: CarveOutKind,
    pub value: String,
}

/// The INJECTABLE forbidden-vocab policy (OYA-CI-CONFORMANCE-FLOOR-PLAN §3.3 / Stage 3): the
/// stem table + carve-out table lifted out of the `const`s into a value the producer sources
/// from `oya-ci.toml`'s `[vocab]` section. [`VocabPolicy::bundled_default`] reproduces the
/// `const`s exactly, so the `_with` census variants under the default are byte-for-byte
/// identical to the legacy const-based census.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VocabPolicy {
    pub stems: Vec<OwnedStem>,
    pub carve_outs: Vec<OwnedCarveOut>,
}

impl VocabPolicy {
    /// The bundled default — byte-for-byte the legacy `FORBIDDEN_VOCAB_STEMS` + `CARVE_OUT_RULES`.
    pub fn bundled_default() -> Self {
        Self {
            stems: FORBIDDEN_VOCAB_STEMS
                .iter()
                .map(|s| OwnedStem {
                    stem: s.stem.to_owned(),
                    code: s.code.to_owned(),
                })
                .collect(),
            carve_outs: CARVE_OUT_RULES
                .iter()
                .map(|c| OwnedCarveOut {
                    kind: c.kind,
                    value: c.value.to_owned(),
                })
                .collect(),
        }
    }
}

impl Default for VocabPolicy {
    fn default() -> Self {
        Self::bundled_default()
    }
}

/// Whether `path` is wholly carved out by an INJECTED policy's path-level rules.
pub fn is_path_carved_out_with(path: &str, policy: &VocabPolicy) -> bool {
    policy.carve_outs.iter().any(|rule| match rule.kind {
        CarveOutKind::PathPrefix => path.starts_with(rule.value.as_str()),
        CarveOutKind::PathExact => path == rule.value,
        CarveOutKind::PathSuffix => path.ends_with(rule.value.as_str()),
        CarveOutKind::LineContainsCi => false,
    })
}

/// Whether `path` is wholly carved out (bundled-default projection of [`is_path_carved_out_with`]).
pub fn is_path_carved_out(path: &str) -> bool {
    is_path_carved_out_with(path, &VocabPolicy::bundled_default())
}

/// Whether `line_lower` is carved out by an INJECTED policy's line-level rule.
pub fn is_line_carved_out_with(line_lower: &str, policy: &VocabPolicy) -> bool {
    policy.carve_outs.iter().any(|rule| {
        rule.kind == CarveOutKind::LineContainsCi && line_lower.contains(rule.value.as_str())
    })
}

/// A document in the corpus the census scans: a repo-relative path + its contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CensusDocument<'a> {
    pub path: &'a str,
    pub contents: &'a str,
}

/// Whether a non-carved-out line carries ANY forbidden stem (boolean convenience for the
/// adversarial RED test). Returns the first matching stem, or `None` for a clean/carved line.
pub fn line_has_forbidden_stem(line: &str) -> Option<&'static ForbiddenStem> {
    line_forbidden_stems(line).into_iter().next()
}

/// EVERY forbidden stem present on a non-carved-out line. A single line can carry more than
/// one stem (e.g. a sentence naming both `foundry` and `jenkins`), and the census must flag
/// the file for each — so the census uses this, not the first-match convenience above.
/// A line carved out by a line-level rule (e.g. Palantir proper-noun prose) yields nothing.
pub fn line_forbidden_stems(line: &str) -> Vec<&'static ForbiddenStem> {
    let lower = line.to_ascii_lowercase();
    if is_line_carved_out_with(&lower, &VocabPolicy::bundled_default()) {
        return Vec::new();
    }
    FORBIDDEN_VOCAB_STEMS
        .iter()
        .filter(|stem| lower.contains(stem.stem))
        .collect()
}

/// The pure per-(stem, file) census over an INJECTED [`VocabPolicy`] (OYA-CI-CONFORMANCE-FLOOR
/// §3.3 / Stage 3). For each document NOT wholly carved out, a file contributes one [`Finding`]
/// per stem it contains on any non-carved-out line. Deterministic (BTreeSet); the bundled-default
/// policy reproduces the legacy const-based census byte-for-byte.
pub fn census_findings_with<'a, I>(documents: I, policy: &VocabPolicy) -> BTreeSet<Finding>
where
    I: IntoIterator<Item = CensusDocument<'a>>,
{
    let mut findings: BTreeSet<Finding> = BTreeSet::new();
    for doc in documents {
        if is_path_carved_out_with(doc.path, policy) {
            continue;
        }
        // Per-file: which stems appear on at least one non-carved-out line. A line may carry
        // several stems; the file is flagged for every one of them.
        let mut codes_in_file: BTreeSet<String> = BTreeSet::new();
        for line in doc.contents.lines() {
            let lower = line.to_ascii_lowercase();
            if is_line_carved_out_with(&lower, policy) {
                continue;
            }
            for stem in &policy.stems {
                if lower.contains(stem.stem.as_str()) {
                    codes_in_file.insert(stem.code.clone());
                }
            }
        }
        for code in codes_in_file {
            findings.insert(Finding {
                code,
                key: doc.path.to_owned(),
            });
        }
    }
    findings
}

/// The pure per-(stem, file) census (bundled-default projection of [`census_findings_with`]).
pub fn census_findings<'a, I>(documents: I) -> BTreeSet<Finding>
where
    I: IntoIterator<Item = CensusDocument<'a>>,
{
    census_findings_with(documents, &VocabPolicy::bundled_default())
}

/// The de-duplicated, line-number-free SET of normalized (lower-cased) matched-line TEXTS for
/// a single `stem` in `contents`, computed by the SAME line-walk [`census_findings_with`] uses
/// (skip wholly path-carved files; skip `LineContainsCi`-carved lines; case-fold with
/// `to_ascii_lowercase`; substring match). This is the occurrence-identity SSOT for the
/// rename-aware path-keyed CI baseline relabel (task #64): the relabel's P4 content-subset guard
/// (`NEW_OCC ⊆ OLD_OCC`) is `matched_line_occurrences_with(new) ⊆ matched_line_occurrences_with(old)`.
///
/// Returns the EMPTY set when the file is wholly path-carved (so a move into/out of a carve-out
/// is symmetric on both sides — an empty NEW set is trivially a subset, and an empty OLD set
/// makes any non-empty NEW set NOT a subset). `stem` is matched exactly as the census matches it
/// (lower-cased substring); the caller passes the live policy's stem string (decoded from the
/// `forbidden_<stem>` code via the policy table), so carve-outs and case-folding agree
/// byte-for-byte with the producer's `collect_brand_residue` census.
pub fn matched_line_occurrences_with(
    path: &str,
    contents: &str,
    stem: &str,
    policy: &VocabPolicy,
) -> BTreeSet<String> {
    let mut occurrences: BTreeSet<String> = BTreeSet::new();
    if is_path_carved_out_with(path, policy) {
        return occurrences;
    }
    let stem_lower = stem.to_ascii_lowercase();
    for line in contents.lines() {
        let lower = line.to_ascii_lowercase();
        if is_line_carved_out_with(&lower, policy) {
            continue;
        }
        if lower.contains(stem_lower.as_str()) {
            // The normalized (lower-cased) matched-line text, de-duplicated (BTreeSet) and
            // line-number-free (churn-stable): two identical foundry lines collapse to one
            // occurrence, and reordering/whitespace-above edits never change the set.
            occurrences.insert(lower);
        }
    }
    occurrences
}

/// The per-stem file COUNT (the shrink-only ratchet metric), derived from the census. Keyed
/// by the firewall code so it lines up with the baseline `cloud-ci-brand-residue` gate.
pub fn census_counts<'a, I>(documents: I) -> BTreeMap<String, usize>
where
    I: IntoIterator<Item = CensusDocument<'a>>,
{
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    // Seed every code at zero so a fully-cleaned stem reports 0 rather than vanishing.
    for stem in FORBIDDEN_VOCAB_STEMS {
        counts.insert(stem.code.to_owned(), 0);
    }
    for finding in census_findings(documents) {
        *counts.entry(finding.code).or_default() += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc<'a>(path: &'a str, contents: &'a str) -> CensusDocument<'a> {
        CensusDocument { path, contents }
    }

    #[test]
    fn census_keys_per_stem_per_file() {
        let findings = census_findings([
            doc("docs/a.md", "We dropped the foundry idea.\nForgejo was also dropped."),
            doc("docs/b.md", "Jenkins farm re-establishment."),
        ]);
        // a.md carries foundry + forgejo; b.md carries jenkins.
        assert!(findings.contains(&Finding {
            code: "forbidden_foundry".into(),
            key: "docs/a.md".into()
        }));
        assert!(findings.contains(&Finding {
            code: "forbidden_forgejo".into(),
            key: "docs/a.md".into()
        }));
        assert!(findings.contains(&Finding {
            code: "forbidden_jenkins".into(),
            key: "docs/b.md".into()
        }));
        assert_eq!(findings.len(), 3);
    }

    #[test]
    fn one_key_per_file_even_with_many_lines() {
        // Many foundry lines in ONE file => ONE key (per-file granularity, churn-free).
        let findings = census_findings([doc(
            "docs/big.md",
            "foundry\nfoundry\nfoundry\nfoundry",
        )]);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            census_counts([doc("docs/big.md", "foundry\nfoundry")])["forbidden_foundry"],
            1
        );
    }

    #[test]
    fn deny_list_source_is_carved_out() {
        // This crate's own source naming the stems is the deny-list, not residue.
        assert!(is_path_carved_out(
            "libs/oya-check-brand-residue/src/forbidden_vocab.rs"
        ));
        let findings = census_findings([doc(
            "libs/oya-check-brand-residue/src/forbidden_vocab.rs",
            "foundry forgejo jenkins oya-vcs",
        )]);
        assert!(findings.is_empty());
    }

    #[test]
    fn catalog_spec_is_carved_out() {
        assert!(is_path_carved_out(
            "registry/catalog/oya-check-brand-residue.yaml"
        ));
    }

    #[test]
    fn legacy_foundry_archive_is_carved_out() {
        assert!(is_path_carved_out("oya/intelligence/_legacy-foundry/README.md"));
    }

    #[test]
    fn append_only_audit_chain_is_carved_out() {
        assert!(is_path_carved_out("evidence/audit-chain.jsonl"));
    }

    #[test]
    fn generated_faces_are_carved_out() {
        assert!(is_path_carved_out(
            "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/gate-baseline.generated.json"
        ));
    }

    #[test]
    fn palantir_proper_noun_line_is_carved_out() {
        // A line citing the competitor is exempt; a real-residue line in the same file is not.
        let findings = census_findings([doc(
            "docs/competitors.md",
            "Ontology vs Palantir Foundry Ontology.\nOur dropped foundry name.",
        )]);
        // foundry appears on a non-palantir line, so the file is still flagged once.
        assert!(findings.contains(&Finding {
            code: "forbidden_foundry".into(),
            key: "docs/competitors.md".into()
        }));
        // a file whose ONLY foundry mention is the palantir line is NOT flagged.
        let clean = census_findings([doc(
            "docs/only-palantir.md",
            "Ontology layer benchmarked against Palantir Foundry.",
        )]);
        assert!(clean.is_empty());
    }

    #[test]
    fn census_is_case_insensitive() {
        let findings = census_findings([doc("docs/c.md", "FOUNDRY and Forgejo and JENKINS")]);
        assert_eq!(findings.len(), 3);
    }

    #[test]
    fn counts_seed_every_stem_at_zero() {
        let counts = census_counts([doc("docs/clean.md", "nothing forbidden here")]);
        assert_eq!(counts["forbidden_foundry"], 0);
        assert_eq!(counts["forbidden_forgejo"], 0);
        assert_eq!(counts["forbidden_jenkins"], 0);
        assert_eq!(counts["forbidden_oya-vcs"], 0);
    }

    #[test]
    fn census_is_deterministic() {
        let a = census_findings([doc("z.md", "foundry"), doc("a.md", "jenkins")]);
        let b = census_findings([doc("a.md", "jenkins"), doc("z.md", "foundry")]);
        assert_eq!(a, b, "census must be order-independent (BTreeSet)");
    }

    // -----------------------------------------------------------------------
    // matched_line_occurrences_with — the rename-aware relabel P4 SSOT (task #64)
    // -----------------------------------------------------------------------

    #[test]
    fn occurrences_are_lowercased_deduped_and_line_number_free() {
        let policy = VocabPolicy::bundled_default();
        // Two identical foundry lines (different surrounding case) collapse to ONE occurrence;
        // a clean line contributes nothing.
        let occ = matched_line_occurrences_with(
            "docs/a.md",
            "actor: SP_FOUNDRY\nactor: sp_foundry\nclean line\n",
            "foundry",
            &policy,
        );
        assert_eq!(occ.len(), 1, "case-folded duplicates collapse: {occ:?}");
        assert!(occ.contains("actor: sp_foundry"));
    }

    #[test]
    fn occurrences_subset_holds_for_pure_move_and_breaks_on_added_residue() {
        let policy = VocabPolicy::bundled_default();
        // A pure move: identical foundry line at old and new path => NEW ⊆ OLD (equal).
        let old = matched_line_occurrences_with("old/lib.rs", "let x = \"foundry\";", "foundry", &policy);
        let new = matched_line_occurrences_with("new/lib.rs", "let x = \"foundry\";", "foundry", &policy);
        assert!(new.is_subset(&old) && old.is_subset(&new), "pure move keeps the set");
        // A move that ADDS a distinct foundry line => NEW ⊄ OLD (subset breaks).
        let new_grown = matched_line_occurrences_with(
            "new/lib.rs",
            "let x = \"foundry\";\nlet y = \"foundry-extra\";",
            "foundry",
            &policy,
        );
        assert!(!new_grown.is_subset(&old), "an added residue line breaks the subset");
    }

    #[test]
    fn occurrences_skip_palantir_line_carve_out() {
        let policy = VocabPolicy::bundled_default();
        // The Palantir proper-noun line is carved out on BOTH sides identically.
        let occ = matched_line_occurrences_with(
            "docs/c.md",
            "Benchmarked against Palantir Foundry.\nOur dropped foundry name.",
            "foundry",
            &policy,
        );
        assert_eq!(occ.len(), 1, "only the non-palantir foundry line counts: {occ:?}");
        assert!(occ.contains("our dropped foundry name."));
    }

    #[test]
    fn occurrences_empty_for_wholly_path_carved_file() {
        let policy = VocabPolicy::bundled_default();
        // A wholly path-carved file yields the empty set (an empty OLD makes any non-empty
        // NEW fail the subset; an empty NEW is a trivial subset — symmetric carve-out handling).
        let occ = matched_line_occurrences_with(
            "libs/oya-check-brand-residue/src/forbidden_vocab.rs",
            "foundry foundry foundry",
            "foundry",
            &policy,
        );
        assert!(occ.is_empty(), "path-carved files contribute no occurrences");
    }

    #[test]
    fn occurrences_match_census_decision_per_file() {
        // matched_line_occurrences_with non-empty IFF census_findings_with flags the file for
        // that stem — the two must agree (same line-walk, same carve-outs).
        let policy = VocabPolicy::bundled_default();
        let contents = "nothing here\nbut jenkins lives here";
        let occ_foundry = matched_line_occurrences_with("x.md", contents, "foundry", &policy);
        let occ_jenkins = matched_line_occurrences_with("x.md", contents, "jenkins", &policy);
        let findings = census_findings_with([doc("x.md", contents)], &policy);
        let flagged_foundry = findings.contains(&Finding { code: "forbidden_foundry".into(), key: "x.md".into() });
        let flagged_jenkins = findings.contains(&Finding { code: "forbidden_jenkins".into(), key: "x.md".into() });
        assert_eq!(occ_foundry.is_empty(), !flagged_foundry);
        assert_eq!(occ_jenkins.is_empty(), !flagged_jenkins);
    }
}
