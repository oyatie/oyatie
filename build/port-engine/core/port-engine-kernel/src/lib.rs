//! # port-engine-kernel — the neutral seam surface of the owned deterministic port engine (W0).
//!
//! ADR-0637 (archived; live via apex ADR-0704) D1 authorizes the engine at `build/port-engine/*`
//! and enumerates the seams this crate defines: SourceModel, RulePack, TransformPlan, TargetIr,
//! Renderer, receipt, delta, and verification. D4 authorizes **W0 only** — so this crate is a
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
//!   Fails closed on a language mismatch, on a duplicate unit id (which would make plan order
//!   ambiguous, i.e. non-deterministic), and on a rule the pack did not declare.
//! - [`emit`] — drives a [`Renderer`] over a [`TargetIr`]. Fails closed on a language mismatch and
//!   on a renderer whose emitted region set is not exactly the IR's region set (a renderer that
//!   drops or invents a region has silently changed the output surface).
//! - [`verify`] — ADR-0637 D2: an emitted-byte change is explained only by a differing receipt
//!   axis. "An unexplained emitted-byte change is RED."
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

// ---------------------------------------------------------------------------------------------
// Compile-time neutrality enforcement (ADR-0637 D1).
// ---------------------------------------------------------------------------------------------

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

/// Corpus vocabulary the neutral engine may never contain, in code or in prose. Spelled as bytes
/// rather than string literals for one reason: a needle written as text would be a needle in the
/// haystack, and every workaround for that (marker lines, split literals, skipping the tail of the
/// file) is a hole an author can hide a real token in.
///
/// The first needle is a bare four-byte PREFIX on purpose — it subsumes every compound built on
/// it (the node daemon, the admin tool, the proxy, the api server, the scheduler, and the project
/// name itself), so the list cannot silently miss a compound no entry anticipated. Enumerating
/// compounds instead is exactly how the earlier draft of this list let two of them through.
pub const FORBIDDEN_CORPUS_TOKENS: [&[u8]; 5] = [
    &[b'k', b'u', b'b', b'e'],
    &[b'k', b'8', b's'],
    &[
        b'a', b'p', b'i', b'm', b'a', b'c', b'h', b'i', b'n', b'e', b'r', b'y',
    ],
    &[b'e', b't', b'c', b'd'],
    &[b't', b'a', b'l', b'o', b's'],
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
pub const UNSCANNED_CODE_KEYWORDS: [&[u8]; 2] = [
    &[b'm', b'o', b'd'],
    &[b'i', b'n', b'c', b'l', b'u', b'd', b'e'],
];

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
// Seams (ADR-0637 D1).
// ---------------------------------------------------------------------------------------------

/// The source→target language pair a [`RulePack`] is authored for.
///
/// This is DATA, not a type parameter: the rule namespace is `specs/port-rules/lang/<pair>/**`,
/// so a second pair is a second directory of rule data over the same engine, never a second
/// engine. Both fields are opaque slugs — the kernel compares them and never interprets them.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LanguagePair {
    /// Slug of the language being read (matches [`SourceModel::language`]).
    pub source: String,
    /// Slug of the language being emitted (matches [`TargetIr::target_language`]).
    pub target: String,
}

impl LanguagePair {
    /// The `<pair>` path segment of the rule namespace, e.g. `source-target`.
    #[must_use]
    pub fn slug(&self) -> String {
        format!("{}-{}", self.source, self.target)
    }
}

/// A stable identity for one translatable unit of the source model.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UnitId(pub String);

/// A stable identity for one rule in a [`RulePack`].
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RuleId(pub String);

/// A stable identity for one emitted region (the ADR-0597 registered regenerable region).
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RegionId(pub String);

/// An opaque content digest. The kernel COMPARES digests and never computes one — hashing is an
/// adapter concern, and keeping it out of here is what lets the receipt seam stay pure.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Digest(pub String);

/// The canonical semantic model of the source corpus, as produced by a front end.
///
/// The kernel never inspects a unit's contents: the front end owns source-language semantics, this
/// trait owns only identity and order. `units` is order-significant and MUST be deterministic for
/// a given input — [`plan`] rejects a duplicate id because that is the shape in which a
/// non-deterministic model reaches the engine.
pub trait SourceModel {
    /// Slug of the language this model was read from.
    fn language(&self) -> &str;
    /// Digest of the snapshot this model was derived from (the receipt's `snapshot_digest`).
    fn snapshot_digest(&self) -> Digest;
    /// The translatable units, in deterministic order.
    fn units(&self) -> Vec<UnitId>;
}

/// Neutral rule data, addressed by [`LanguagePair`].
///
/// Rule SEMANTICS live in the data, not here. The kernel needs exactly two things: which pair the
/// pack serves, and which of its declared rules apply to a unit — in pack order, because rule
/// order is part of the transform.
pub trait RulePack {
    /// The language pair this pack is authored for.
    fn pair(&self) -> &LanguagePair;
    /// Digest of the pack contents (the receipt's `rulepack_digest`).
    fn digest(&self) -> Digest;
    /// Every rule the pack declares, in pack order.
    fn rules(&self) -> Vec<RuleId>;
    /// The declared rules that apply to `unit`, in pack order. Returning a rule absent from
    /// [`RulePack::rules`] is a pack defect and [`plan`] refuses it.
    fn rules_for(&self, unit: &UnitId) -> Vec<RuleId>;
}

/// One step of a [`TransformPlan`]: apply `rule` to `unit`.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PlanStep {
    /// The unit the rule applies to.
    pub unit: UnitId,
    /// The rule to apply.
    pub rule: RuleId,
}

/// The deterministic, ordered transform to execute. Data only: holding it does not run it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformPlan {
    /// The pair this plan translates.
    pub pair: LanguagePair,
    /// The steps, in execution order (model unit order, then pack rule order).
    pub steps: Vec<PlanStep>,
}

/// The neutral intermediate representation handed to a [`Renderer`].
///
/// As with [`SourceModel`], the kernel sees identity and order, never content.
pub trait TargetIr {
    /// Slug of the language this IR will be emitted as.
    fn target_language(&self) -> &str;
    /// The regions this IR emits, in deterministic order.
    fn regions(&self) -> Vec<RegionId>;
}

/// Turns a [`TargetIr`] into emitted bytes, one blob per region.
pub trait Renderer {
    /// Slug of the language this renderer emits.
    fn target_language(&self) -> &str;
    /// Digest of the formatter this renderer applies (the receipt's `formatter_digest`).
    fn formatter_digest(&self) -> Digest;
    /// Render every region of `ir`. The returned key set MUST equal `ir.regions()`; [`emit`]
    /// enforces that rather than trusting it.
    ///
    /// # Errors
    /// Whatever the implementation refuses with; [`emit`] adds the region-set proof on top.
    fn render(&self, ir: &dyn TargetIr) -> Result<BTreeMap<RegionId, Vec<u8>>, PortError>;
}

/// The six receipt axes ADR-0637 fixes. Every emitted-byte change must be attributable to at least
/// one of them; see [`verify`].
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReceiptAxis {
    /// The upstream pin the source snapshot was taken at.
    Pin,
    /// The digest of the source snapshot.
    Snapshot,
    /// The digest of the engine binary.
    Engine,
    /// The digest of the rule pack.
    RulePack,
    /// The digest of the toolchain.
    Toolchain,
    /// The digest of the formatter.
    Formatter,
}

/// Every axis, in declaration order. Registered as a constant so a seventh axis cannot be added
/// without the comparison in [`Receipt::differing_axes`] being updated alongside it.
pub const RECEIPT_AXES: [ReceiptAxis; 6] = [
    ReceiptAxis::Pin,
    ReceiptAxis::Snapshot,
    ReceiptAxis::Engine,
    ReceiptAxis::RulePack,
    ReceiptAxis::Toolchain,
    ReceiptAxis::Formatter,
];

/// The six-axis provenance of one emission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    /// The upstream pin (an opaque revision identifier).
    pub pin: String,
    /// Digest of the source snapshot.
    pub snapshot_digest: Digest,
    /// Digest of the engine that emitted.
    pub engine_digest: Digest,
    /// Digest of the rule pack in force.
    pub rulepack_digest: Digest,
    /// Digest of the toolchain in force.
    pub toolchain_digest: Digest,
    /// Digest of the formatter in force.
    pub formatter_digest: Digest,
}

impl Receipt {
    /// The axes on which `self` and `other` disagree.
    #[must_use]
    pub fn differing_axes(&self, other: &Self) -> BTreeSet<ReceiptAxis> {
        let mut differing = BTreeSet::new();
        for axis in RECEIPT_AXES {
            let differs = match axis {
                ReceiptAxis::Pin => self.pin != other.pin,
                ReceiptAxis::Snapshot => self.snapshot_digest != other.snapshot_digest,
                ReceiptAxis::Engine => self.engine_digest != other.engine_digest,
                ReceiptAxis::RulePack => self.rulepack_digest != other.rulepack_digest,
                ReceiptAxis::Toolchain => self.toolchain_digest != other.toolchain_digest,
                ReceiptAxis::Formatter => self.formatter_digest != other.formatter_digest,
            };
            if differs {
                differing.insert(axis);
            }
        }
        differing
    }
}

/// The classification of an emitted-byte change between two receipts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Delta {
    /// No region changed.
    Unchanged,
    /// Regions changed and at least one receipt axis moved to account for it.
    Explained {
        /// The regions whose bytes changed.
        regions: BTreeSet<RegionId>,
        /// The axes that moved.
        axes: BTreeSet<ReceiptAxis>,
    },
    /// Regions changed while every receipt axis held. ADR-0637 D2: this is RED, and it is a
    /// defect in the engine, rules, policy, model, or a declared detachment — never something to
    /// be repaired by editing the generated output.
    Unexplained {
        /// The regions whose bytes changed with no axis to account for them.
        regions: BTreeSet<RegionId>,
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
    pub verdict: Verdict,
    /// Why.
    pub delta: Delta,
}

/// A typed, fail-closed refusal. Every variant carries enough to act on without re-deriving.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PortError {
    /// A language slug did not match the one it was paired against.
    LanguageMismatch {
        /// What the consumer required.
        expected: String,
        /// What it was handed.
        actual: String,
    },
    /// The source model emitted the same unit id twice, so step order is ambiguous.
    DuplicateUnit {
        /// The repeated id.
        unit: UnitId,
    },
    /// `rules_for` returned a rule the pack does not declare.
    UndeclaredRule {
        /// The unit it was returned for.
        unit: UnitId,
        /// The undeclared rule.
        rule: RuleId,
    },
    /// A renderer's emitted region set was not exactly the IR's region set.
    RegionSetMismatch {
        /// Regions the IR declared that the renderer did not emit.
        missing: BTreeSet<RegionId>,
        /// Regions the renderer emitted that the IR did not declare.
        unexpected: BTreeSet<RegionId>,
    },
}

impl fmt::Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LanguageMismatch { expected, actual } => {
                write!(
                    f,
                    "language mismatch: expected `{expected}`, got `{actual}`"
                )
            }
            Self::DuplicateUnit { unit } => {
                write!(
                    f,
                    "duplicate source unit `{}`: plan order is ambiguous",
                    unit.0
                )
            }
            Self::UndeclaredRule { unit, rule } => write!(
                f,
                "rule `{}` applied to unit `{}` is not declared by the pack",
                rule.0, unit.0
            ),
            Self::RegionSetMismatch {
                missing,
                unexpected,
            } => write!(
                f,
                "renderer region set mismatch: {} missing, {} unexpected",
                missing.len(),
                unexpected.len()
            ),
        }
    }
}

impl std::error::Error for PortError {}

/// Build the deterministic [`TransformPlan`] for `model` under `pack`.
///
/// Fails closed on a pair/model language mismatch, on a duplicate unit id, and on a rule the pack
/// does not declare. Step order is model unit order, then pack rule order — both supplied, neither
/// invented here.
///
/// # Errors
/// [`PortError::LanguageMismatch`], [`PortError::DuplicateUnit`], [`PortError::UndeclaredRule`].
pub fn plan(model: &dyn SourceModel, pack: &dyn RulePack) -> Result<TransformPlan, PortError> {
    let pair = pack.pair();
    if pair.source != model.language() {
        return Err(PortError::LanguageMismatch {
            expected: pair.source.clone(),
            actual: model.language().to_owned(),
        });
    }

    let declared: BTreeSet<RuleId> = pack.rules().into_iter().collect();
    let mut seen: BTreeSet<UnitId> = BTreeSet::new();
    let mut steps: Vec<PlanStep> = Vec::new();

    for unit in model.units() {
        if !seen.insert(unit.clone()) {
            return Err(PortError::DuplicateUnit { unit });
        }
        for rule in pack.rules_for(&unit) {
            if !declared.contains(&rule) {
                return Err(PortError::UndeclaredRule { unit, rule });
            }
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
/// # Errors
/// [`PortError::LanguageMismatch`], [`PortError::RegionSetMismatch`], or whatever the renderer
/// itself refuses with.
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

    let declared: BTreeSet<RegionId> = ir.regions().into_iter().collect();
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
#[must_use]
pub fn verify(
    previous: &Receipt,
    current: &Receipt,
    changed_regions: &BTreeSet<RegionId>,
) -> Verification {
    if changed_regions.is_empty() {
        return Verification {
            verdict: Verdict::Green,
            delta: Delta::Unchanged,
        };
    }

    let axes = previous.differing_axes(current);
    if axes.is_empty() {
        return Verification {
            verdict: Verdict::Red,
            delta: Delta::Unexplained {
                regions: changed_regions.clone(),
            },
        };
    }

    Verification {
        verdict: Verdict::Green,
        delta: Delta::Explained {
            regions: changed_regions.clone(),
            axes,
        },
    }
}
