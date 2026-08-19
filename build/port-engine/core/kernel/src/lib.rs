//! # port-engine-kernel — neutrality enforcement and entry points for the port engine (W0).
//!
//! ADR-0637 (archived; live via apex ADR-0704) D1 authorizes the engine at `build/port-engine/*`.
//! Neutral seam types (`SourceModel`, `RulePack`, `TransformPlan`, `TargetIr`, `Renderer`,
//! `Receipt`) live in `port-engine-api`; this crate owns compile-time neutrality enforcement,
//! `plan` / `emit` / `verify`, and delta classification. D4 authorizes **W0 only** — so this crate is a
//! SKELETON with real seams and real refusals, and deliberately contains:
//!
//! - no source-language front end (no parser, no grammar, no tree-sitter);
//! - no rule DATA (`specs/port-rules/**` is a later lane; this crate defines only its SHAPE);
//! - no corpus knowledge of any kind.
//!
//! ## Neutrality is a BUILD ERROR here, not a lint and not a test
//!
//! D1 rules that a corpus token or a corpus-specific branch in the neutral kernel is a defect that
//! MUST be rejected. The rejection is the `const` assertion block below: this file reads its own
//! bytes at COMPILE time via `include_str!` and refuses to compile if any forbidden sequence is
//! present. A test can be skipped, filtered, or left unrun; a failed const assertion is a build
//! failure with no path around it.
//!
//! WHAT THAT BLOCK DOES AND DOES NOT DECIDE, stated plainly because the difference is the whole
//! honesty of the claim. It rejects a fixed CANARY SET of corpus vocabulary — [`FORBIDDEN_CORPUS_TOKENS`],
//! five needles chosen because they are the ones a corpus-specific author reaches for first. It is
//! NOT a decision procedure for "is this corpus-specific": a branch on some corpus noun no needle
//! anticipates compiles, and review is what catches that. No finite needle list could be complete,
//! because the corpus's vocabulary is open.
//!
//! Deriving the needles from corpus policy instead was considered and REFUSED: it would make the
//! neutral kernel read the corpus it is defined by not knowing, which is the exact coupling D1
//! forbids, and it would let an edit to corpus policy silently widen or narrow what the engine
//! may contain. The structural properties carry the rest of the weight and are complete where the
//! text scan is not — seam types name no corpus type (see `port-engine-api`), and rule
//! semantics live in DATA under `specs/port-rules/**` where a corpus-specific rule is supposed to
//! be. The canary set is a cheap backstop on top of that, never the argument for neutrality.
//!
//! The scan is COMPLETE rather than hopeful because [`UNSCANNED_CODE_KEYWORDS`] also refuses the
//! two constructs that could put kernel code in a file this scan never reads — a submodule
//! declaration and a source-splicing macro. So "the kernel is exactly this file" is a proven
//! property of the build, not a convention. Growing the kernel to a second file therefore fails
//! the BUILD, and the next author must derive the scanned set from the build rule's srcs before
//! that file can exist.
//!
//! Those two needles are matched as WHOLE IDENTIFIERS, not as substrings, and the distinction is
//! the entire defect history of this gate. Both grammar productions are whitespace-INSENSITIVE:
//! the keyword may be followed by a newline, a tab, or a comment, and the splicing macro may carry
//! a space before its bang. An earlier draft spelled the needles as the plain substrings
//! `"m","o","d"," "` and `"i","n","c","l","u","d","e","!"`, so a two-byte edit of the planted
//! defect — a newline in place of the space — compiled a corpus-carrying second file and the gate
//! stayed green. That was proven by execution, not argued. [`contains_word`] anchors on identifier
//! boundaries instead, which is where the grammar itself draws them, so every whitespace form is
//! caught by construction while `include_str!` (a longer identifier, hence not a match) stays
//! usable — this scan is built on it.
//!
//! The seam TEST is corpus-scanned at compile time too. It is compiled into the crate's test
//! binary, so an unscanned `tests/seams.rs` would be a place for a corpus-specific fixture or
//! branch to enter the crate under the gate's nose. Its completeness comes from its Buck target
//! naming exactly one source file rather than from a second const scan — see [`SEAM_TEST_SOURCE`].
//!
//! `tests/neutrality.rs` is the companion proof that the same predicates and the same needle sets
//! go RED on planted defects — a const assertion cannot be shown failing without breaking the
//! build, so its capability to fail is demonstrated there instead. It is deliberately the ONE file
//! nothing scans: it must spell the needles out to prove they bite, so scanning it would make the
//! proof impossible to write. It is also the only file where the needles are the subject rather
//! than the risk.
//!
//! Language neutrality is carried the way the rule namespace already carries it: the ADR's
//! `specs/port-rules/lang/<pair>/**` layout makes the LANGUAGE PAIR a datum, so [`LanguagePair`]
//! is data on the [`RulePack`] rather than a type parameter or a hard-coded source language. The
//! first pair is not the only one, and nothing in this crate names any language.
//!
//! ## What the three entry points actually decide
//!
//! - [`plan`] — pairs a [`SourceModel`] with a [`RulePack`] into an ordered [`TransformPlan`].
//!   Fails closed on a language mismatch, on a duplicate unit id or a duplicate DECLARED rule id
//!   (either one makes plan order ambiguous, i.e. non-deterministic), on a rule the pack did not
//!   declare, and on declared rules handed back in an order that is not the pack's own — the last
//!   one because rule order is part of the transform, so a plan that depends on which pack
//!   answered is not deterministic either.
//! - [`emit`] — drives a [`Renderer`] over a [`TargetIr`]. Fails closed on a language mismatch, on
//!   an IR declaring one region identity twice, and on a renderer whose emitted region set is not
//!   exactly the IR's region set (a renderer that drops or invents a region has silently changed
//!   the output surface).
//! - [`verify`] — ADR-0637 D2: an emitted-byte change is explained only by a differing receipt
//!   axis. "An unexplained emitted-byte change is RED." The changed set is DERIVED from the two
//!   emitted trees, never supplied, so no caller can assert its way to a Green.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![forbid(unsafe_code)]
// The const scan is a bounded linear walk of this file's own bytes, once per needle. `deny` by
// default, `long_running_const_eval` is a HANG detector — its own note says a genuinely long
// evaluation may allow it — and this evaluation is neither long nor unbounded: every loop is
// `while start <= last_start` over a fixed-length slice, so it terminates by construction.
//
// It is allowed here rather than worked around because the review that added the seam refusals
// crossed the step budget purely by making the file longer, and every way of getting back under it
// costs enforcement: dropping the seam test's corpus pass demotes a build error to a test, and
// shortening the needle list narrows what is refused. Compile cost is the cheap side of that
// trade. If it ever stops being cheap, the fix is to derive the scanned set from the build rule's
// srcs and scan at test time with a build-time guard on the srcs list — not to scan less.
#![allow(long_running_const_eval)]

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::{
    PlanStep, PortError, Receipt, ReceiptAxis, RegionId, Renderer, RuleId, RulePack, SourceModel,
    TargetIr, TransformPlan, UnitId,
};

// ---------------------------------------------------------------------------------------------
// Compile-time neutrality enforcement (ADR-0637 D1).
// ---------------------------------------------------------------------------------------------

/// This crate's own source, for the engine-identity axis assembled by the facade.
///
/// INLINE rather than a generated submodule. This crate refuses a submodule declaration as a whole
/// identifier so that "the kernel is exactly this file" is a property of the build, and splitting it
/// would delete that proof to save four lines. One entry, because the same rule makes a second file
/// impossible — and this comment cannot spell the refused keyword, which is the scan working.
pub const CRATE_SOURCES: &[(&str, &str)] = &[("lib.rs", include_str!("lib.rs"))];

/// This file's own bytes, read at compile time. Every assertion below is evaluated against the
/// real source rather than against a hand-maintained description of it.
const KERNEL_SOURCE: &str = include_str!("lib.rs");

/// The seam test's bytes, read the same way. [`FORBIDDEN_CORPUS_TOKENS`] is asserted over it at
/// COMPILE time: it is compiled into this crate's test binary, and corpus vocabulary must not
/// reach the engine through a fixture any more than through a function.
///
/// [`UNSCANNED_CODE_KEYWORDS`] is NOT asserted over it here, for two reasons. The completeness
/// property already holds STRUCTURALLY: the seam test's Buck target lists exactly one source file,
/// so a submodule declaration in it does not build — a stronger guarantee than a text scan, and
/// free. And the seven const passes this scan already costs sit near rustc's
/// `long_running_const_eval` budget; two more crossed it, measured, not guessed.
/// `tests/neutrality.rs` asserts the keyword property over this file at test time as the backstop.
///
/// This is why `tests/seams.rs` appears in the library target's `srcs` in BUCK: the file is an
/// INPUT to compiling the library, never a module of it.
const SEAM_TEST_SOURCE: &str = include_str!("../tests/seams.rs");

/// Corpus vocabulary the neutral engine may never contain, in code or in prose. A CANARY SET, not
/// a decision procedure — see the module docs: no finite list can decide "corpus-specific", and
/// deriving one from corpus policy was refused because it would couple the neutral engine to the
/// corpus. Spelled as bytes
/// rather than string literals for one reason: a needle written as text would be a needle in the
/// haystack, and every workaround for that (marker lines, split literals, skipping the tail of the
/// file) is a hole an author can hide a real token in.
///
/// The first needle is a bare four-byte PREFIX on purpose — it subsumes every compound built on
/// it (the node daemon, the admin tool, the proxy, the api server, the scheduler, and the project
/// name itself), so the list cannot silently miss a compound no entry anticipated. Enumerating
/// compounds instead is exactly how the earlier draft of this list let two of them through.
pub const FORBIDDEN_CORPUS_TOKENS: [&[u8]; 5] = [
    &[107, 117, 98, 101],
    &[107, 56, 115],
    &[97, 112, 105, 109, 97, 99, 104, 105, 110, 101, 114, 121],
    &[101, 116, 99, 100],
    &[116, 97, 108, 111, 115],
];

/// Keywords whose grammar productions could place compiled kernel code in a file the neutrality
/// scan never reads: the submodule declaration and the source-splicing macro. Refusing both is
/// what makes a one-file scan a COMPLETE scan rather than a hopeful one.
///
/// Matched by [`contains_word`], i.e. on IDENTIFIER boundaries, never as substrings. Both
/// productions accept ANY whitespace after the keyword, and the splicing one accepts whitespace
/// before its bang and a leading path — all valid Rust — so a needle carrying a fixed space or a
/// fixed bang is a hole, and was: see the module docs. Anchoring where the grammar anchors closes
/// every form at once. `tests/neutrality.rs` spells the escaping forms out; it can, because
/// nothing scans it, and this file cannot, because everything scans it.
///
/// The first needle refuses ANY submodule declaration, inline as well as file-scoped, and that
/// bluntness is deliberate. Distinguishing the two forms means parsing, a textual rule that tries
/// it acquires edge cases, and an edge case in a neutrality rule is a place to hide a token. The
/// cost is that the crate's tests live in `tests/`, which they would anyway.
///
/// The second needle is the bare identifier, so it also catches the path-qualified call. Prose and
/// longer identifiers are untouched by the boundary rule: `model`, `modular` and `include_str!`
/// are all longer identifiers, and the last of them is what this scan is built on.
pub const UNSCANNED_CODE_KEYWORDS: [&[u8]; 2] =
    [&[109, 111, 100], &[105, 110, 99, 108, 117, 100, 101]];

const fn lowercase_ascii(byte: u8) -> u8 {
    if byte.is_ascii_uppercase() {
        byte + 32
    } else {
        byte
    }
}

/// Case-insensitive substring search that is usable in a `const` context, which is what lets the
/// neutrality rule be a build error. `token` MUST already be lowercase.
///
/// Public so `tests/neutrality.rs` proves the rule with the exact predicate the build uses — a
/// test that reimplemented the search could pass while the enforced one was broken.
#[must_use]
pub const fn contains_token(haystack: &[u8], token: &[u8]) -> bool {
    if token.is_empty() || token.len() > haystack.len() {
        return false;
    }
    let last_start = haystack.len() - token.len();
    let mut start = 0;
    while start <= last_start {
        let mut offset = 0;
        while offset < token.len() && lowercase_ascii(haystack[start + offset]) == token[offset] {
            offset += 1;
        }
        if offset == token.len() {
            return true;
        }
        start += 1;
    }
    false
}

/// True for the bytes Rust admits INSIDE an identifier. Everything else is an identifier boundary,
/// which is exactly where a keyword starts and ends — including at every whitespace form, at a
/// path separator, and at a bang.
const fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Case-insensitive WHOLE-IDENTIFIER search, usable in a `const` context. `word` MUST already be
/// lowercase.
///
/// This is [`contains_token`] plus the boundary condition, and the boundary condition is the whole
/// point: a keyword needle matched as a substring has to guess the byte that follows it, and every
/// guess is a hole. Anchoring on identifier boundaries makes the rule agree with the grammar
/// rather than with one formatting of it.
///
/// Public for the same reason as [`contains_token`]: `tests/neutrality.rs` proves the rule with
/// the exact predicate the build enforces with.
#[must_use]
pub const fn contains_word(haystack: &[u8], word: &[u8]) -> bool {
    if word.is_empty() || word.len() > haystack.len() {
        return false;
    }
    let last_start = haystack.len() - word.len();
    let mut start = 0;
    while start <= last_start {
        let mut offset = 0;
        while offset < word.len() && lowercase_ascii(haystack[start + offset]) == word[offset] {
            offset += 1;
        }
        let end = start + word.len();
        if offset == word.len()
            && (start == 0 || !is_identifier_byte(haystack[start - 1]))
            && (end == haystack.len() || !is_identifier_byte(haystack[end]))
        {
            return true;
        }
        start += 1;
    }
    false
}

const _: () = {
    let source = KERNEL_SOURCE.as_bytes();
    let seam_test = SEAM_TEST_SOURCE.as_bytes();
    let mut i = 0;
    while i < FORBIDDEN_CORPUS_TOKENS.len() {
        assert!(
            !contains_token(source, FORBIDDEN_CORPUS_TOKENS[i]),
            "the neutral kernel carries a corpus token (ADR-0637 D1): corpus vocabulary and \
             corpus-specific behaviour belong in corpus policy, never in the engine"
        );
        assert!(
            !contains_token(seam_test, FORBIDDEN_CORPUS_TOKENS[i]),
            "the seam test carries a corpus token (ADR-0637 D1): a corpus-specific fixture is a \
             corpus-specific branch with extra steps, and belongs in corpus policy"
        );
        i += 1;
    }
    let mut j = 0;
    while j < UNSCANNED_CODE_KEYWORDS.len() {
        assert!(
            !contains_word(source, UNSCANNED_CODE_KEYWORDS[j]),
            "kernel code would live in a file this neutrality scan cannot read: keep the kernel \
             one file, or derive the scanned set from the build rule's srcs before adding another"
        );
        j += 1;
    }
};

// ---------------------------------------------------------------------------------------------
// Delta classification (ADR-0637 D2) — kernel entry points below.
// ---------------------------------------------------------------------------------------------

/// The classification of an emitted-byte change between two receipts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Delta {
    /// No region changed.
    Unchanged,
    /// Regions changed and at least one receipt axis moved to account for it.
    Explained {
        /// The regions whose bytes changed.
        regions: BTreeSet<RegionId>, // data_class: INTERNAL_ONLY
        /// The axes that moved.
        axes: BTreeSet<ReceiptAxis>, // data_class: INTERNAL_ONLY
    },
    /// Regions changed while every receipt axis held. ADR-0637 D2: this is RED, and it is a
    /// defect in the engine, rules, policy, model, or a declared detachment — never something to
    /// be repaired by editing the generated output.
    Unexplained {
        /// The regions whose bytes changed with no axis to account for them.
        regions: BTreeSet<RegionId>, // data_class: INTERNAL_ONLY
    },
    /// Regions changed and at least one receipt cannot be USED to explain them: an axis is empty,
    /// so an apparent difference on it is absence of information, not evidence of a cause.
    ///
    /// The false Green this refuses needs the two receipts to be ASYMMETRICALLY incomplete — a
    /// populated previous against an all-empty current, say, from an adapter that failed to fill
    /// one in. Every axis then "differs", and an unfilled receipt has manufactured a six-axis
    /// explanation for an arbitrary byte change. Two EQUALLY incomplete receipts were never the
    /// risk: they differ on nothing, so a byte change falls to [`Delta::Unexplained`] already.
    IncompleteReceipt {
        /// The regions whose bytes changed while the evidence was unusable.
        regions: BTreeSet<RegionId>, // data_class: INTERNAL_ONLY
    },
}

/// The verdict of a determinism check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// The change is fully accounted for.
    Green,
    /// The change is not accounted for.
    Red,
}

/// The outcome of [`verify`]: a verdict and the delta that produced it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Verification {
    /// Green or red.
    pub verdict: Verdict, // data_class: INTERNAL_ONLY
    /// Why.
    pub delta: Delta, // data_class: INTERNAL_ONLY
}

/// Build the deterministic [`TransformPlan`] for `model` under `pack`.
///
/// Fails closed on a pair/model language mismatch, on a duplicate unit id, and on a rule the pack
/// does not declare. Step order is model unit order, then pack rule order — both supplied, neither
/// invented here.
///
/// Pack order is ENFORCED, not assumed. `rules()` is the pack's declared order, and `rules_for`
/// is contracted to answer in that same order; the engine checks that the rules it is handed for a
/// unit are a strictly increasing subsequence of `rules()` rather than trusting the claim. Without
/// the check two packs over identical rule data can hand back the same rules in different orders
/// and produce different plans — non-determinism arriving through the seam the plan is supposed to
/// make deterministic. Strictly increasing also rules out the same rule twice for one unit, which
/// would duplicate a step for no stated reason.
///
/// # Errors
/// [`PortError::LanguageMismatch`], [`PortError::DuplicateRule`], [`PortError::DuplicateUnit`],
/// [`PortError::UndeclaredRule`], [`PortError::RuleOrderViolation`].
pub fn plan(model: &dyn SourceModel, pack: &dyn RulePack) -> Result<TransformPlan, PortError> {
    let pair = pack.pair();
    if pair.source != model.language() {
        return Err(PortError::LanguageMismatch {
            expected: pair.source.clone(),
            actual: model.language().to_owned(),
        });
    }

    // Declared rules by POSITION, so membership and ORDER are one lookup — and a repeated
    // declaration is refused here, where the position it would make ambiguous is still visible.
    let mut declared: BTreeMap<RuleId, usize> = BTreeMap::new();
    for (position, rule) in pack.rules().into_iter().enumerate() {
        if declared.insert(rule.clone(), position).is_some() {
            return Err(PortError::DuplicateRule { rule });
        }
    }
    let mut seen: BTreeSet<UnitId> = BTreeSet::new();
    let mut steps: Vec<PlanStep> = Vec::new();

    for unit in model.units() {
        if !seen.insert(unit.clone()) {
            return Err(PortError::DuplicateUnit { unit });
        }
        let mut previous: Option<usize> = None;
        for rule in pack.rules_for(&unit) {
            let Some(&position) = declared.get(&rule) else {
                return Err(PortError::UndeclaredRule { unit, rule });
            };
            if previous.is_some_and(|last| position <= last) {
                return Err(PortError::RuleOrderViolation { unit, rule });
            }
            previous = Some(position);
            steps.push(PlanStep {
                unit: unit.clone(),
                rule,
            });
        }
    }

    Ok(TransformPlan {
        pair: pair.clone(),
        steps,
    })
}

/// Render `ir` with `renderer`, proving the emitted region set is exactly the declared one.
///
/// The declared regions are checked for a repeated identity BEFORE they become a set. Collecting
/// straight into a set would collapse a duplicate, and a renderer emitting that region once would
/// then satisfy the set comparison — one declared occurrence lost, silently, by the very step
/// meant to prove nothing was lost. [`plan`] refuses the same condition on the source side.
///
/// # Errors
/// [`PortError::LanguageMismatch`], [`PortError::DuplicateRegion`],
/// [`PortError::RegionSetMismatch`], or whatever the renderer itself refuses with.
pub fn emit(
    renderer: &dyn Renderer,
    ir: &dyn TargetIr,
) -> Result<BTreeMap<RegionId, Vec<u8>>, PortError> {
    if renderer.target_language() != ir.target_language() {
        return Err(PortError::LanguageMismatch {
            expected: ir.target_language().to_owned(),
            actual: renderer.target_language().to_owned(),
        });
    }

    let mut declared: BTreeSet<RegionId> = BTreeSet::new();
    for region in ir.regions() {
        if !declared.insert(region.clone()) {
            return Err(PortError::DuplicateRegion { region });
        }
    }
    let rendered = renderer.render(ir)?;
    let emitted: BTreeSet<RegionId> = rendered.keys().cloned().collect();

    let missing: BTreeSet<RegionId> = declared.difference(&emitted).cloned().collect();
    let unexpected: BTreeSet<RegionId> = emitted.difference(&declared).cloned().collect();
    if !missing.is_empty() || !unexpected.is_empty() {
        return Err(PortError::RegionSetMismatch {
            missing,
            unexpected,
        });
    }

    Ok(rendered)
}

/// Classify an emitted-byte change against the two receipts that bracket it (ADR-0637 D2).
///
/// Unchanged bytes are green. Changed bytes with a moved axis are explained. Changed bytes with
/// every axis held are UNEXPLAINED and red — the engine cannot say why its own output moved.
///
/// THE CHANGED SET IS DERIVED FROM THE EMITTED BYTES, never supplied. An earlier signature took a
/// `changed_regions` argument, which made the verdict a function of the caller's claim rather than
/// of the output: a caller that passed an empty set — by omission, by a bug, or deliberately — got
/// Green with no byte ever compared, so stale or forged output verified clean. The two trees are
/// exactly the maps [`emit`] returns, so the caller has nothing left to get wrong. Both are read
/// in full: a region present on one side only is a change, which a key-wise comparison of the
/// intersection would have missed.
///
/// THE RECEIPT IS CHECKED FOR USABILITY, not only for difference. Removing the changed-set
/// argument moved trust from the caller to the trees and left the RECEIPTS trusted, which is a
/// residue of the same defect: an all-empty receipt against a populated one "differs" on all six
/// axes and would have explained any byte change at all. An empty axis is absence of information,
/// so it buys no explanation — see [`Delta::IncompleteReceipt`].
///
/// This stays a PURE classifier — the trees are values in memory, no filesystem and no hashing,
/// which is what keeps the receipt seam adapter-free.
#[must_use]
pub fn verify(
    previous: &Receipt,
    previous_output: &BTreeMap<RegionId, Vec<u8>>,
    current: &Receipt,
    current_output: &BTreeMap<RegionId, Vec<u8>>,
) -> Verification {
    let mut changed_regions: BTreeSet<RegionId> = BTreeSet::new();
    for (region, bytes) in previous_output {
        if current_output.get(region) != Some(bytes) {
            changed_regions.insert(region.clone());
        }
    }
    for region in current_output.keys() {
        if !previous_output.contains_key(region) {
            changed_regions.insert(region.clone());
        }
    }

    if changed_regions.is_empty() {
        return Verification {
            verdict: Verdict::Green,
            delta: Delta::Unchanged,
        };
    }

    // The receipts are checked for USABILITY only once the bytes have already been found to move.
    // Placement is load-bearing: an incomplete receipt that decided nothing must not turn an
    // identical tree red, so this sits strictly after the unchanged return.
    if !previous.incomplete_axes().is_empty() || !current.incomplete_axes().is_empty() {
        return Verification {
            verdict: Verdict::Red,
            delta: Delta::IncompleteReceipt {
                regions: changed_regions,
            },
        };
    }

    let axes = previous.differing_axes(current);
    if axes.is_empty() {
        return Verification {
            verdict: Verdict::Red,
            delta: Delta::Unexplained {
                regions: changed_regions,
            },
        };
    }

    Verification {
        verdict: Verdict::Green,
        delta: Delta::Explained {
            regions: changed_regions,
            axes,
        },
    }
}
