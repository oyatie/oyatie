//! # cloud-ci-corpus-index-coverage — the northstar as a burn-down invariant
//!
//! "Everything in the code graph and the build graph for full visibility" is a slogan until a
//! number is attached to it and that number is only allowed to move one way. This gate attaches the
//! number.
//!
//! ## What it measures
//! A buck2 package (a directory holding a `BUCK` file) that OWNS at least one YAML file is either
//! INDEXED — it declares a `corpus-yaml-facts` extraction target, so its YAML is a build-graph
//! input whose facts are a build output — or UNCOVERED.
//!
//! `coverage = indexed / total`, COMPUTED from the observed corpus. Nothing here is asserted: the
//! caller supplies observations, the kernel counts them.
//!
//! ## Why it ratchets rather than blocks
//! Born-ADVISORY, shrink-only. Today almost every YAML-owning package is uncovered, so a blocking
//! gate would be permanently red and would be switched off within a week. Instead the current
//! uncovered count is frozen as a ceiling: existing debt is reported, and a NEW uncovered package
//! is a REGRESSION that fails closed. The ceiling is lowered as extraction targets land, so the
//! slogan becomes a burn-down.
//!
//! ## The anti-vacuity rule, which is the important one
//! The dangerous failure of a coverage gate is not a false red, it is a walk that silently sees
//! nothing: zero YAML packages observed means zero uncovered, which reads as PERFECT COVERAGE. A
//! suspiciously total number means the probe is broken until proven otherwise, so
//! `min_expected_yaml_packages` fails the gate closed when the observed corpus collapses.
//!
//! The same instinct applied to the NORTHSTAR term was a design error, and it is worth recording
//! because the shape recurs. `min_expected_unpackaged_yaml_files` was a FLOOR asserting that at
//! least N YAML files still sat outside the build graph — on a term whose target value is ZERO.
//! Honest progress therefore tripped the guard, six consecutive waves had to lower it, and the
//! final wave could never have satisfied it. A guard must be monotone in the same direction as the
//! thing it guards against. The replacement is `CODE_UNPACKAGED_DROP_UNATTRIBUTED`: the ceiling is
//! two-sided, a drop must be re-frozen in the change that caused it, and zero is a fixed point.
//!
//! PURE: no I/O, no clock, no rand. The caller walks the tree and passes observations as DATA.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use oya_buck_syntax_kernel::{CallExpr, Expr, Stmt, glob_match, parse};
use serde::{Deserialize, Serialize};

/// The gate's stable identifier.
pub const GATE_ID: &str = "cloud-ci-corpus-index-coverage";

/// A NEW YAML-owning package that declares no extraction target, pushing the uncovered count above
/// the frozen ceiling. Blocking.
pub const CODE_COVERAGE_REGRESSION: &str = "corpus_index_coverage_regression";

/// The observed corpus collapsed below the expected floor — the walk is broken, and its "no
/// uncovered packages" result is meaningless. Blocking.
pub const CODE_VACUOUS_SCAN: &str = "corpus_index_scan_vacuous";

/// A YAML-owning package with no extraction target, within the frozen ceiling. Advisory: this is
/// the debt being burned down.
pub const CODE_UNCOVERED_PACKAGE: &str = "corpus_index_uncovered_package";

/// MORE YAML files now live outside every buck2 package than the frozen ceiling allows. Blocking.
///
/// This is the northstar ratchet. A file in no package cannot be indexed at all, and the fix is to
/// pull it INTO the build graph — never to index it through a side channel, which would let it stay
/// outside forever while the coverage number improved.
pub const CODE_UNPACKAGED_REGRESSION: &str = "corpus_index_unpackaged_regression";

/// The frozen ceiling is higher than the observed uncovered count — the ratchet has slack and
/// should be lowered so it keeps biting. Advisory.
pub const CODE_STALE_CEILING: &str = "corpus_index_stale_ceiling";

/// The unpackaged count is BELOW its frozen ceiling and the ceiling was not re-frozen in the same
/// change. Advisory (PROCESS_TAX) — same posture as [`CODE_STALE_CEILING`].
///
/// A drop in the northstar term has exactly two causes and they are indistinguishable from the
/// counts alone: artifacts were genuinely pulled into the build graph, or the ownership walk
/// mis-attributed them. Hand re-freeze of the absolute census is not tip-entitled; the advisory
/// still surfaces slack so reviewers can lower the ceiling without blocking honest progress.
/// Regression above the ceiling remains blocking. See [`Policy::baseline_unpackaged_yaml_files`].
pub const CODE_UNPACKAGED_DROP_UNATTRIBUTED: &str = "corpus_index_unpackaged_drop_unattributed";

const CANONICAL_SHARD_MODULE: &str = "//governance/corpus/extract:yaml_facts.bzl";
const CANONICAL_SHARD_SYMBOL: &str = "corpus_yaml_facts_shards";
const EXTRACTION_TARGET: &str = "corpus-yaml-facts";
const MAX_SHARD_SIZE: usize = 512;

/// A package's recognized YAML extraction declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionDeclaration {
    /// No extraction declaration is present.
    None,
    /// One literal `corpus-yaml-facts` genrule owns the package's YAML corpus.
    LiteralSingle,
    /// The canonical fixed-size sharding macro owns the package's YAML corpus.
    FixedShards { shard_size: usize },
}

/// One YAML input and its measured source size.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CorpusInput {
    pub path: String,
    pub source_bytes: u64,
}

/// One source-derived extraction face.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaceObservation {
    pub label: String,
    pub package: String,
    pub paths: Vec<String>,
    pub source_bytes: u64,
}

/// Per-face limits enforced by the corpus gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaceLimits {
    pub max_files: usize,
    pub max_source_bytes: u64,
}

/// Frozen Oya YAML corpus expectations and extraction-face limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OyaCorpusPolicy {
    pub expected_yaml_files: usize,
    pub max_files_per_extraction_face: usize,
    pub max_source_bytes_per_extraction_face: u64,
}

impl OyaCorpusPolicy {
    /// Convert policy data into the generic face-limit evaluator input.
    #[must_use]
    pub const fn face_limits(self) -> FaceLimits {
        FaceLimits {
            max_files: self.max_files_per_extraction_face,
            max_source_bytes: self.max_source_bytes_per_extraction_face,
        }
    }
}

/// A fail-closed declaration parsing error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationError(String);

impl fmt::Display for DeclarationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for DeclarationError {}

fn declaration_error(message: impl Into<String>) -> DeclarationError {
    DeclarationError(message.into())
}

fn string_arg(call: &CallExpr, index: usize) -> Option<&str> {
    let arg = call.args.get(index)?;
    if arg.name.is_some() {
        return None;
    }
    match &arg.value.expr {
        Expr::Str(value) => Some(value),
        _ => None,
    }
}

fn canonical_glob(call: &CallExpr) -> bool {
    if call.func != "glob" || call.args.len() != 1 || call.args[0].name.is_some() {
        return false;
    }
    let Expr::List(list) = &call.args[0].value.expr else {
        return false;
    };
    let values: Vec<&str> = list
        .elements
        .iter()
        .filter_map(|element| match &element.value.expr {
            Expr::Str(value) => Some(value.as_str()),
            _ => None,
        })
        .collect();
    values == ["**/*.yaml", "**/*.yml"] && values.len() == list.elements.len()
}

fn canonical_srcs(call: &CallExpr) -> bool {
    let Some(srcs) = call.kwarg("srcs") else {
        return false;
    };
    match &srcs.value.expr {
        Expr::Call(glob) => canonical_glob(glob),
        _ => false,
    }
}

fn literal_declaration(package: &str, call: &CallExpr) -> Result<bool, DeclarationError> {
    if call.func != "genrule" {
        return Ok(false);
    }
    let is_target = call.kwarg("name").is_some_and(
        |arg| matches!(&arg.value.expr, Expr::Str(value) if value == EXTRACTION_TARGET),
    );
    if !is_target {
        return Ok(false);
    }
    let canonical_out = call.kwarg("out").is_some_and(
        |arg| matches!(&arg.value.expr, Expr::Str(value) if value == "yaml-facts.json"),
    );
    let expected_cmd = format!(
        "$(exe //governance/corpus/extract:yaml-facts) --target root//{package}:{EXTRACTION_TARGET} --prefix {package} --out $OUT $SRCS"
    );
    let canonical_cmd = call
        .kwarg("cmd")
        .is_some_and(|arg| matches!(&arg.value.expr, Expr::Str(value) if value == &expected_cmd));
    if call.args.len() != 4
        || !canonical_srcs(call)
        || !canonical_out
        || !canonical_cmd
        || call.has_opaque()
    {
        return Err(declaration_error(
            "incomplete or unsupported literal corpus extraction genrule",
        ));
    }
    Ok(true)
}

/// Parse one BUCK document and recognize only the canonical YAML extraction declarations.
///
/// # Errors
/// Returns an error for malformed, conflicting, aliased, computed, or otherwise unsupported
/// declaration syntax. Comments and strings never count as declarations.
pub fn extraction_declaration(
    package: &str,
    text: &str,
) -> Result<ExtractionDeclaration, DeclarationError> {
    let doc =
        parse(text).map_err(|error| declaration_error(format!("BUCK parse failed: {error}")))?;
    let mut canonical_loads = 0usize;
    let mut shard_calls = Vec::new();
    let mut literal_calls = 0usize;
    let mut suspicious_shard_call = false;

    for statement in &doc.stmts {
        match statement {
            Stmt::Call(call) if call.func == "load" => {
                if string_arg(call, 0) == Some(CANONICAL_SHARD_MODULE)
                    && string_arg(call, 1) == Some(CANONICAL_SHARD_SYMBOL)
                    && call.args.len() == 2
                {
                    canonical_loads += 1;
                } else if call.args.iter().any(|arg| {
                    matches!(&arg.value.expr, Expr::Str(value) if value == CANONICAL_SHARD_SYMBOL || value == CANONICAL_SHARD_MODULE)
                }) {
                    suspicious_shard_call = true;
                }
            }
            Stmt::Call(call) if call.func == CANONICAL_SHARD_SYMBOL => shard_calls.push(call),
            Stmt::Call(call) => {
                if literal_declaration(package, call)? {
                    literal_calls += 1;
                }
            }
            Stmt::Opaque { span } => {
                let raw = span.slice(text);
                if raw.contains(CANONICAL_SHARD_SYMBOL) || raw.contains(EXTRACTION_TARGET) {
                    suspicious_shard_call = true;
                }
            }
            Stmt::Assign { value, .. } | Stmt::IndexAssign { value, .. } => {
                value.visit_calls(&mut |call| {
                    if call.func == CANONICAL_SHARD_SYMBOL {
                        suspicious_shard_call = true;
                    }
                });
            }
        }
    }

    if suspicious_shard_call || canonical_loads > 1 || shard_calls.len() > 1 || literal_calls > 1 {
        return Err(declaration_error(
            "ambiguous or unsupported corpus extraction declaration",
        ));
    }
    if literal_calls == 1 && (!shard_calls.is_empty() || canonical_loads != 0) {
        return Err(declaration_error(
            "literal and sharded extraction declarations conflict",
        ));
    }
    if literal_calls == 1 {
        return Ok(ExtractionDeclaration::LiteralSingle);
    }
    if canonical_loads == 0 && shard_calls.is_empty() {
        return Ok(ExtractionDeclaration::None);
    }
    if canonical_loads != 1 || shard_calls.len() != 1 {
        return Err(declaration_error(
            "canonical shard load and call must appear exactly once",
        ));
    }

    let call = shard_calls[0];
    if call.has_opaque() || !canonical_srcs(call) || call.args.len() != 2 {
        return Err(declaration_error(
            "canonical shard call requires direct canonical srcs and shard_size",
        ));
    }
    let Some(shard_arg) = call.kwarg("shard_size") else {
        return Err(declaration_error(
            "canonical shard call is missing shard_size",
        ));
    };
    let Expr::Int(raw) = &shard_arg.value.expr else {
        return Err(declaration_error(
            "shard_size must be a direct integer literal",
        ));
    };
    let shard_size = raw
        .parse::<usize>()
        .map_err(|_| declaration_error("shard_size is not a valid integer"))?;
    if shard_size == 0 || shard_size > MAX_SHARD_SIZE {
        return Err(declaration_error("shard_size must be between 1 and 512"));
    }
    Ok(ExtractionDeclaration::FixedShards { shard_size })
}

/// Derive deterministic extraction faces from package-owned inputs.
///
/// # Errors
/// Returns an error for empty fixed declarations, invalid shard sizes, duplicate paths, or source
/// byte overflow.
pub fn derive_faces(
    package: &str,
    owned_paths: &[CorpusInput],
    declaration: ExtractionDeclaration,
) -> Result<Vec<FaceObservation>, DeclarationError> {
    if declaration == ExtractionDeclaration::None {
        return Ok(Vec::new());
    }
    if owned_paths.is_empty() {
        return Err(declaration_error(
            "an extraction declaration cannot own an empty corpus",
        ));
    }
    let shard_size = match declaration {
        ExtractionDeclaration::None => return Ok(Vec::new()),
        ExtractionDeclaration::LiteralSingle => owned_paths.len(),
        ExtractionDeclaration::FixedShards { shard_size }
            if (1..=MAX_SHARD_SIZE).contains(&shard_size) =>
        {
            shard_size
        }
        ExtractionDeclaration::FixedShards { .. } => {
            return Err(declaration_error("shard_size must be between 1 and 512"));
        }
    };
    let prefix = format!("{package}/");
    let mut ordered: Vec<CorpusInput> = owned_paths
        .iter()
        .filter(|input| {
            let package_local = input.path.strip_prefix(&prefix).unwrap_or(&input.path);
            glob_match("**/*.yaml", package_local) || glob_match("**/*.yml", package_local)
        })
        .cloned()
        .collect();
    if ordered.is_empty() {
        return Err(declaration_error(
            "an extraction declaration has no canonical YAML inputs",
        ));
    }
    ordered.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    if ordered.windows(2).any(|pair| pair[0].path == pair[1].path) {
        return Err(declaration_error("owned corpus contains duplicate paths"));
    }
    let face_count =
        ordered.len() / shard_size + usize::from(!ordered.len().is_multiple_of(shard_size));
    if face_count > 10_000 {
        return Err(declaration_error(
            "fixed sharding would exceed the four-digit ordinal grammar",
        ));
    }

    ordered
        .chunks(shard_size)
        .enumerate()
        .map(|(ordinal, chunk)| {
            let name = if ordinal == 0 {
                EXTRACTION_TARGET.to_owned()
            } else {
                format!("{EXTRACTION_TARGET}-shard-{ordinal:04}")
            };
            let source_bytes = chunk.iter().try_fold(0u64, |sum, input| {
                sum.checked_add(input.source_bytes)
                    .ok_or_else(|| declaration_error("face source byte total overflowed"))
            })?;
            Ok(FaceObservation {
                label: format!("root//{package}:{name}"),
                package: package.to_owned(),
                paths: chunk.iter().map(|input| input.path.clone()).collect(),
                source_bytes,
            })
        })
        .collect()
}

/// Validate exact face coverage and per-face limits.
///
/// # Errors
/// Returns every observed missing, extra, duplicate, empty, oversized, or inconsistent face error.
pub fn evaluate_face_coverage(
    expected_paths: &[CorpusInput],
    faces: &[FaceObservation],
    limits: FaceLimits,
) -> Result<(), Vec<String>> {
    let expected: BTreeMap<&str, u64> = expected_paths
        .iter()
        .map(|input| (input.path.as_str(), input.source_bytes))
        .collect();
    let mut observed = BTreeSet::new();
    let mut errors = Vec::new();

    if expected.len() != expected_paths.len() {
        errors.push("expected corpus contains duplicate paths".to_owned());
    }
    for input in expected_paths {
        if input.source_bytes > limits.max_source_bytes {
            errors.push(format!(
                "input {} exceeds the source-byte limit",
                input.path
            ));
        }
    }
    for face in faces {
        if face.paths.is_empty() {
            errors.push(format!("face {} is empty", face.label));
        }
        if face.paths.len() > limits.max_files {
            errors.push(format!("face {} exceeds the file-count limit", face.label));
        }
        if face.source_bytes > limits.max_source_bytes {
            errors.push(format!("face {} exceeds the source-byte limit", face.label));
        }
        let mut measured = 0u64;
        for path in &face.paths {
            if !observed.insert(path.as_str()) {
                errors.push(format!("path {path} is assigned more than once"));
            }
            match expected.get(path.as_str()) {
                Some(bytes) => match measured.checked_add(*bytes) {
                    Some(sum) => measured = sum,
                    None => {
                        errors.push(format!("face {} source byte total overflowed", face.label))
                    }
                },
                None => errors.push(format!("unexpected path {path}")),
            }
        }
        if measured != face.source_bytes {
            errors.push(format!(
                "face {} source byte measurement is inconsistent",
                face.label
            ));
        }
    }
    for path in expected.keys() {
        if !observed.contains(path) {
            errors.push(format!("expected path {path} is missing"));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// One observed buck2 package that owns at least one YAML file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PackageObservation {
    /// Repo-relative directory of the package (the dir holding its `BUCK` file).
    pub package: String,
    /// How many YAML files this package owns (files whose NEAREST ancestor `BUCK` is this one).
    pub yaml_files: usize,
    /// Does the package declare a `corpus-yaml-facts` extraction target?
    pub indexed: bool,
}

/// The frozen policy. All repo-specifics are DATA: another repo adopts this gate by repointing it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    /// The shrink-only ceiling: observed uncovered packages may not exceed this.
    pub baseline_uncovered_packages: usize,
    /// The northstar ceiling: YAML files outside every buck2 package may not exceed this — AND may
    /// not sit below it either, because a drop must be attributed in the change that caused it.
    ///
    /// This field is EQUALITY-pinned in effect: above it is [`CODE_UNPACKAGED_REGRESSION`], below
    /// it is [`CODE_UNPACKAGED_DROP_UNATTRIBUTED`]. That two-sided shape is what replaced the
    /// former `min_expected_unpackaged_yaml_files` floor, which was structurally defective: it was
    /// a FLOOR on a term whose northstar is ZERO, so it failed the gate closed on honest progress
    /// and every wave that packaged more YAML had to lower it again — six times — while the final
    /// wave could not have satisfied it at all. An anti-vacuity guard must be monotone in the same
    /// direction as the thing it guards against; this one is, and zero is a stable fixed point of
    /// it (`0 < 0` is false), so reaching the northstar does not require touching the guard.
    ///
    /// It still catches everything the floor caught. A walk whose out-of-package census collapses
    /// reports a number below the ceiling and BLOCKS, exactly as before. What changed is the
    /// remedy: the author lowers the ceiling to the truth and says what moved, instead of lowering
    /// a floor below the truth, which is the false green this whole ratchet exists to prevent.
    pub baseline_unpackaged_yaml_files: usize,
    /// Anti-vacuity floor: fewer observed YAML-owning packages than this means a broken walk.
    pub min_expected_yaml_packages: usize,
    /// Anti-vacuity floor on FILES. A walk that finds packages but no files is equally broken, and
    /// would report a shrinking unpackaged count that is pure measurement collapse.
    pub min_expected_yaml_files: usize,
}

/// Computed coverage over the observed corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coverage {
    /// YAML-owning packages observed.
    pub total_packages: usize,
    /// Of those, how many declare an extraction target.
    pub indexed_packages: usize,
    /// `total_packages - indexed_packages`. The number the ratchet drives to zero.
    pub uncovered_packages: usize,
    /// EVERY tracked YAML file in the repo.
    ///
    /// The denominator deliberately includes files that belong to no buck2 package. Counting only
    /// in-package files would let the gate report flawless coverage while most of the corpus sat
    /// outside the build graph — the exact false green this gate exists to prevent.
    pub total_yaml_files: usize,
    /// YAML files owned by INDEXED packages — the files actually reaching the graph.
    pub indexed_yaml_files: usize,
    /// YAML files that belong to NO buck2 package, and so cannot be indexed at all today.
    /// Structurally the largest term, and the one the northstar ratchet drives down.
    pub unpackaged_yaml_files: usize,
}

impl Coverage {
    /// Indexed packages per ten-thousand, as an INTEGER.
    ///
    /// Basis points, never a float: a float in a gate verdict is a formatting hazard in every
    /// serialized artifact downstream, and integer bps carries all the precision anyone reads.
    /// Returns 0 when nothing was observed — a vacuous scan reports no coverage, never 100%.
    #[must_use]
    pub const fn package_coverage_bps(&self) -> u32 {
        if self.total_packages == 0 {
            return 0;
        }
        #[allow(clippy::cast_possible_truncation)]
        {
            ((self.indexed_packages * 10_000) / self.total_packages) as u32
        }
    }

    /// Indexed YAML FILES per ten-thousand, as an integer. Packages vary wildly in how much YAML
    /// they own, so the file-level number is the honest view of how much corpus is really visible.
    #[must_use]
    pub const fn file_coverage_bps(&self) -> u32 {
        if self.total_yaml_files == 0 {
            return 0;
        }
        #[allow(clippy::cast_possible_truncation)]
        {
            ((self.indexed_yaml_files * 10_000) / self.total_yaml_files) as u32
        }
    }
}

/// One gate finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// The violation code.
    pub code: String,
    /// The package the finding is about (empty for corpus-wide findings).
    pub package: String,
    /// Human-readable detail.
    pub detail: String,
    /// Does this finding fail the gate?
    pub blocking: bool,
}

/// The gate verdict: the computed coverage plus every finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verdict {
    /// Computed coverage over the observations.
    pub coverage: Coverage,
    /// All findings, blocking and advisory.
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
}

/// Compute coverage over the observed packages. Pure counting — no policy involved.
///
/// `unpackaged_yaml_files` is the count of tracked YAML files belonging to NO buck2 package. It is
/// a separate argument rather than derivable from `observations` precisely because those files have
/// no package to be observed under, which is the whole problem.
#[must_use]
pub fn coverage(observations: &[PackageObservation], unpackaged_yaml_files: usize) -> Coverage {
    let total_packages = observations.len();
    let indexed_packages = observations.iter().filter(|o| o.indexed).count();
    let packaged_yaml_files: usize = observations.iter().map(|o| o.yaml_files).sum();
    Coverage {
        total_packages,
        indexed_packages,
        uncovered_packages: total_packages - indexed_packages,
        total_yaml_files: packaged_yaml_files + unpackaged_yaml_files,
        indexed_yaml_files: observations
            .iter()
            .filter(|o| o.indexed)
            .map(|o| o.yaml_files)
            .sum(),
        unpackaged_yaml_files,
    }
}

/// Evaluate the observed corpus against the frozen policy.
#[must_use]
pub fn evaluate(
    observations: &[PackageObservation],
    unpackaged_yaml_files: usize,
    policy: &Policy,
) -> Verdict {
    let coverage = coverage(observations, unpackaged_yaml_files);
    let mut findings = Vec::new();

    // Anti-vacuity FIRST: every other verdict below is meaningless if the walk saw nothing, and a
    // broken walk otherwise presents as flawless coverage.
    if coverage.total_packages < policy.min_expected_yaml_packages {
        findings.push(Finding {
            code: CODE_VACUOUS_SCAN.to_owned(),
            package: String::new(),
            detail: format!(
                "observed only {} YAML-owning packages, expected at least {} — the walk is broken, \
                 so its coverage result is not evidence",
                coverage.total_packages, policy.min_expected_yaml_packages
            ),
            blocking: true,
        });
    }
    if coverage.total_yaml_files < policy.min_expected_yaml_files {
        findings.push(Finding {
            code: CODE_VACUOUS_SCAN.to_owned(),
            package: String::new(),
            detail: format!(
                "observed only {} YAML files, expected at least {} — a collapsed file census makes \
                 the unpackaged count shrink for the wrong reason",
                coverage.total_yaml_files, policy.min_expected_yaml_files
            ),
            blocking: true,
        });
    }
    // ATTRIBUTION slack. Both floors above are invariant under mis-attribution (packaged rises by
    // exactly what unpackaged loses), so they hold while the northstar term goes to zero and reads
    // as the debt being paid off. Counts cannot tell mis-attribution from real progress. PROCESS_TAX
    // DELETE: hand re-freeze of that drop is not a merge blocker — advisory only (mirror uncovered
    // stale-ceiling). Silent on equality, and silent forever once the northstar is reached.
    if coverage.unpackaged_yaml_files < policy.baseline_unpackaged_yaml_files {
        findings.push(Finding {
            code: CODE_UNPACKAGED_DROP_UNATTRIBUTED.to_owned(),
            package: String::new(),
            detail: format!(
                "{} YAML files belong to no buck2 package but the frozen ceiling is {} — a drop is \
                 either artifacts pulled into the build graph or an ownership walk that \
                 mis-attributed them, and the counts cannot tell those apart. Lower \
                 baseline_unpackaged_yaml_files to {} when attributing the drop (advisory; not a \
                 merge blocker).",
                coverage.unpackaged_yaml_files,
                policy.baseline_unpackaged_yaml_files,
                coverage.unpackaged_yaml_files
            ),
            blocking: false,
        });
    }

    // The northstar ratchet: artifacts must move INTO the build graph, never around it.
    if coverage.unpackaged_yaml_files > policy.baseline_unpackaged_yaml_files {
        findings.push(Finding {
            code: CODE_UNPACKAGED_REGRESSION.to_owned(),
            package: String::new(),
            detail: format!(
                "{} YAML files belong to no buck2 package, above the frozen ceiling of {}. New YAML \
                 must land inside a buck2 package so it is a build-graph input.",
                coverage.unpackaged_yaml_files, policy.baseline_unpackaged_yaml_files
            ),
            blocking: true,
        });
    }

    if coverage.uncovered_packages > policy.baseline_uncovered_packages {
        findings.push(Finding {
            code: CODE_COVERAGE_REGRESSION.to_owned(),
            package: String::new(),
            detail: format!(
                "{} YAML-owning packages declare no corpus-yaml-facts target, above the frozen \
                 ceiling of {}. A new YAML-owning package must either declare an extraction target \
                 or lower the ceiling in the same change.",
                coverage.uncovered_packages, policy.baseline_uncovered_packages
            ),
            blocking: true,
        });
    } else if coverage.uncovered_packages < policy.baseline_uncovered_packages {
        // Slack means the ratchet has stopped biting: coverage improved but the ceiling was not
        // lowered, so a regression back to the old ceiling would pass unnoticed.
        findings.push(Finding {
            code: CODE_STALE_CEILING.to_owned(),
            package: String::new(),
            detail: format!(
                "uncovered is {} but the ceiling is {} — lower baseline_uncovered_packages to {} so \
                 the ratchet keeps biting",
                coverage.uncovered_packages,
                policy.baseline_uncovered_packages,
                coverage.uncovered_packages
            ),
            blocking: false,
        });
    }

    for observation in observations.iter().filter(|o| !o.indexed) {
        findings.push(Finding {
            code: CODE_UNCOVERED_PACKAGE.to_owned(),
            package: observation.package.clone(),
            detail: format!(
                "{} YAML file(s) outside the corpus graph",
                observation.yaml_files
            ),
            blocking: false,
        });
    }

    Verdict { coverage, findings }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(name: &str, files: usize, indexed: bool) -> PackageObservation {
        PackageObservation {
            package: name.to_owned(),
            yaml_files: files,
            indexed,
        }
    }

    fn policy(ceiling: usize, unpackaged: usize) -> Policy {
        Policy {
            baseline_uncovered_packages: ceiling,
            baseline_unpackaged_yaml_files: unpackaged,
            min_expected_yaml_packages: 1,
            min_expected_yaml_files: 1,
        }
    }

    #[test]
    fn coverage_is_computed_not_asserted() {
        let observed = [pkg("a", 3, true), pkg("b", 7, false), pkg("c", 5, true)];
        let computed = coverage(&observed, 0);
        assert_eq!(computed.total_packages, 3);
        assert_eq!(computed.indexed_packages, 2);
        assert_eq!(computed.uncovered_packages, 1);
        assert_eq!(computed.total_yaml_files, 15);
        assert_eq!(computed.indexed_yaml_files, 8);
        assert_eq!(computed.package_coverage_bps(), 6_666);
        assert_eq!(computed.file_coverage_bps(), 5_333);
    }

    // The false green this gate was REBUILT to prevent: an earlier version counted only in-package
    // YAML, so indexing every package reported 100% coverage while most of the corpus sat outside
    // the build graph entirely. Unpackaged files belong in the DENOMINATOR.
    #[test]
    fn unpackaged_files_stay_in_the_denominator() {
        let computed = coverage(&[pkg("a", 100, true)], 900);
        assert_eq!(computed.total_yaml_files, 1_000);
        assert_eq!(computed.indexed_yaml_files, 100);
        assert_eq!(
            computed.file_coverage_bps(),
            1_000,
            "every package indexed must NOT read as full coverage while 900 files are unpackaged"
        );
        assert_eq!(computed.package_coverage_bps(), 10_000);
    }

    #[test]
    fn at_the_ceiling_is_green() {
        let observed = [pkg("a", 1, false), pkg("b", 1, true)];
        assert!(!evaluate(&observed, 5, &policy(1, 5)).failed());
    }

    // The ratchet: one more uncovered package than the ceiling must FAIL.
    #[test]
    fn a_new_uncovered_package_regresses_and_blocks() {
        let observed = [pkg("a", 1, false), pkg("b", 1, false)];
        let verdict = evaluate(&observed, 0, &policy(1, 0));
        assert!(verdict.failed());
        assert!(
            verdict
                .blocking()
                .iter()
                .any(|f| f.code == CODE_COVERAGE_REGRESSION)
        );
    }

    // The northstar ratchet: new YAML outside every buck2 package must FAIL, so the fix is to pull
    // artifacts INTO the build graph rather than index them through a side channel.
    #[test]
    fn new_unpackaged_yaml_regresses_and_blocks() {
        let verdict = evaluate(&[pkg("a", 1, true)], 11, &policy(0, 10));
        assert!(verdict.failed());
        assert!(
            verdict
                .blocking()
                .iter()
                .any(|f| f.code == CODE_UNPACKAGED_REGRESSION)
        );
    }

    // Landing an extraction target must move the number and never fail the gate.
    #[test]
    fn indexing_a_package_burns_the_number_down() {
        let before = coverage(&[pkg("a", 4, false), pkg("b", 6, false)], 0);
        let after = coverage(&[pkg("a", 4, true), pkg("b", 6, false)], 0);
        assert_eq!(before.file_coverage_bps(), 0);
        assert_eq!(after.file_coverage_bps(), 4_000);
        assert!(!evaluate(&[pkg("a", 4, true), pkg("b", 6, false)], 0, &policy(2, 0)).failed());
    }

    // The failure that matters: a broken walk sees nothing, so uncovered is 0, which would read as
    // FLAWLESS COVERAGE without the floor.
    #[test]
    fn an_empty_scan_fails_closed_instead_of_reporting_perfection() {
        let verdict = evaluate(&[], 0, &policy(10, 10));
        assert_eq!(verdict.coverage.uncovered_packages, 0);
        assert_eq!(verdict.coverage.package_coverage_bps(), 0);
        assert_eq!(verdict.coverage.file_coverage_bps(), 0);
        assert!(verdict.failed(), "a vacuous scan must not pass");
        assert!(
            verdict
                .blocking()
                .iter()
                .any(|f| f.code == CODE_VACUOUS_SCAN)
        );
    }

    // A collapsed file census would shrink the unpackaged count for the wrong reason, which would
    // read as northstar progress.
    #[test]
    fn a_collapsed_file_census_fails_closed() {
        let strict = Policy {
            baseline_uncovered_packages: 10,
            baseline_unpackaged_yaml_files: 0,
            min_expected_yaml_packages: 1,
            min_expected_yaml_files: 5_000,
        };
        let verdict = evaluate(&[pkg("a", 1, true)], 0, &strict);
        assert!(
            verdict.failed(),
            "a collapsed census must not read as progress"
        );
        assert!(
            verdict
                .blocking()
                .iter()
                .any(|f| f.code == CODE_VACUOUS_SCAN)
        );
    }

    // THE CASE NEITHER CENSUS FLOOR CAN SEE. Every file is attributed to a package, so the total
    // census is INTACT (5000 packaged + 0 unpackaged = 5000) and the package count is intact —
    // both floors above pass. Only the attribution rule fails it. Without this the northstar term
    // reads zero and looks like the out-of-graph debt was paid off in full.
    #[test]
    fn an_attribution_collapse_surfaces_advisory_while_both_census_floors_hold() {
        let strict = Policy {
            baseline_uncovered_packages: 10,
            baseline_unpackaged_yaml_files: 5_000,
            min_expected_yaml_packages: 1,
            min_expected_yaml_files: 5_000,
        };
        let observed = [pkg("a", 5_000, true)];

        // Control: both census floors are genuinely satisfied on this shape, so the advisory below
        // can only be the attribution rule — otherwise this test would prove nothing about it.
        let census_only = Policy {
            baseline_unpackaged_yaml_files: 0,
            ..strict
        };
        assert!(
            !evaluate(&observed, 0, &census_only).failed(),
            "control: both census floors must PASS on this shape"
        );

        let verdict = evaluate(&observed, 0, &strict);
        assert!(
            !verdict.failed(),
            "PROCESS_TAX: unpackaged collapse is advisory, not a merge blocker"
        );
        assert!(
            verdict
                .findings
                .iter()
                .any(|f| f.code == CODE_UNPACKAGED_DROP_UNATTRIBUTED && !f.blocking)
        );

        // GUARD: at the ceiling the rule is silent, so it is not always-on.
        assert!(!evaluate(&observed, 5_000, &strict).failed());
    }

    // THE BEAD (oyatie-ln1). The guard this replaced was a FLOOR on a term whose northstar is
    // ZERO, so it failed the gate closed on honest progress. PROCESS_TAX: honest drops are
    // advisory (not merge-blocking); re-freeze clears the advisory; northstar zero stays green.
    #[test]
    fn honest_progress_toward_zero_is_never_blocked_by_lowering_the_guard() {
        let wave25 = Policy {
            baseline_uncovered_packages: 18,
            baseline_unpackaged_yaml_files: 75,
            min_expected_yaml_packages: 20,
            min_expected_yaml_files: 4_000,
        };
        // 20 packages so the CENSUS floors are genuinely satisfied — otherwise this test would
        // prove nothing about the attribution rule, which is the only thing it is about.
        let observed: Vec<_> = (0..20)
            .map(|i| pkg(&format!("root{i}"), 282, true))
            .collect();

        // At the frozen anchor: silent.
        assert!(!evaluate(&observed, 75, &wave25).failed());

        // Honest wave packages more YAML; unpackaged 75 -> 34. Advisory surfaces slack; gate stays
        // green (PROCESS_TAX — hand re-freeze is not a merge blocker).
        let verdict = evaluate(&observed, 34, &wave25);
        assert!(
            !verdict.failed(),
            "honest unpackaged drop must not merge-block"
        );
        assert!(
            verdict
                .findings
                .iter()
                .any(|f| f.code == CODE_UNPACKAGED_DROP_UNATTRIBUTED && !f.blocking)
        );
        let re_frozen = Policy {
            baseline_unpackaged_yaml_files: 34,
            ..wave25
        };
        assert!(
            !evaluate(&observed, 34, &re_frozen).failed(),
            "re-freezing the ceiling at the measured number must clear the finding"
        );

        // THE PROPERTY THE OLD FLOOR COULD NOT HAVE: the northstar itself is a fixed point. The
        // final wave reaches zero, freezes at zero, and the guard is silent forever after.
        let northstar = Policy {
            baseline_unpackaged_yaml_files: 0,
            ..wave25
        };
        assert!(
            !evaluate(&observed, 0, &northstar).failed(),
            "unpackaged == 0 against a ceiling of 0 must be GREEN — the goal must be reachable"
        );
    }

    // The other half of two-sided: a REGRESSION above the ceiling still blocks; a drop below is
    // advisory (PROCESS_TAX), and the codes stay distinct.
    #[test]
    fn the_ceiling_stays_two_sided_and_the_codes_stay_distinct() {
        let p = policy(5, 10);
        let over = evaluate(&[pkg("a", 1, true)], 11, &p);
        let under = evaluate(&[pkg("a", 1, true)], 9, &p);
        assert!(
            over.blocking()
                .iter()
                .any(|f| f.code == CODE_UNPACKAGED_REGRESSION)
        );
        assert!(
            !over
                .blocking()
                .iter()
                .any(|f| f.code == CODE_UNPACKAGED_DROP_UNATTRIBUTED)
        );
        assert!(!under.failed(), "unpackaged drop is advisory, not blocking");
        assert!(
            under
                .findings
                .iter()
                .any(|f| f.code == CODE_UNPACKAGED_DROP_UNATTRIBUTED && !f.blocking)
        );
        assert!(
            !under
                .blocking()
                .iter()
                .any(|f| f.code == CODE_UNPACKAGED_REGRESSION)
        );
    }

    #[test]
    fn slack_in_the_ceiling_is_reported_so_the_ratchet_keeps_biting() {
        let observed = [pkg("a", 1, true), pkg("b", 1, true)];
        let verdict = evaluate(&observed, 0, &policy(5, 0));
        assert!(!verdict.failed());
        assert!(
            verdict
                .findings
                .iter()
                .any(|f| f.code == CODE_STALE_CEILING)
        );
    }

    #[test]
    fn uncovered_packages_are_reported_individually_as_advisory_debt() {
        let observed = [pkg("a", 2, false), pkg("b", 3, false)];
        let verdict = evaluate(&observed, 0, &policy(2, 0));
        let advisory: Vec<&Finding> = verdict
            .findings
            .iter()
            .filter(|f| f.code == CODE_UNCOVERED_PACKAGE)
            .collect();
        assert_eq!(advisory.len(), 2);
        assert!(advisory.iter().all(|f| !f.blocking));
    }

    fn input(path: &str, source_bytes: u64) -> CorpusInput {
        CorpusInput {
            path: path.to_owned(),
            source_bytes,
        }
    }

    fn limits() -> FaceLimits {
        FaceLimits {
            max_files: 512,
            max_source_bytes: 1_048_576,
        }
    }

    fn face(paths: &[&str], source_bytes: u64) -> FaceObservation {
        FaceObservation {
            label: "root//oya:corpus-yaml-facts".to_owned(),
            package: "oya".to_owned(),
            paths: paths.iter().map(|path| (*path).to_owned()).collect(),
            source_bytes,
        }
    }

    #[test]
    fn missing_path_blocks() {
        assert!(
            evaluate_face_coverage(
                &[input("a.yaml", 1), input("b.yaml", 1)],
                &[face(&["a.yaml"], 1)],
                limits()
            )
            .is_err()
        );
    }

    #[test]
    fn duplicate_path_blocks() {
        assert!(
            evaluate_face_coverage(
                &[input("a.yaml", 1)],
                &[face(&["a.yaml", "a.yaml"], 2)],
                limits()
            )
            .is_err()
        );
    }

    #[test]
    fn empty_face_blocks() {
        assert!(evaluate_face_coverage(&[], &[face(&[], 0)], limits()).is_err());
    }

    #[test]
    fn face_with_513_files_blocks() {
        let inputs: Vec<_> = (0..513).map(|i| input(&format!("{i}.yaml"), 1)).collect();
        let paths: Vec<_> = inputs.iter().map(|i| i.path.clone()).collect();
        let observed = FaceObservation {
            paths,
            source_bytes: 513,
            ..face(&[], 0)
        };
        assert!(evaluate_face_coverage(&inputs, &[observed], limits()).is_err());
    }

    #[test]
    fn face_with_1048577_bytes_blocks() {
        assert!(
            evaluate_face_coverage(
                &[input("a.yaml", 1_048_577)],
                &[face(&["a.yaml"], 1_048_577)],
                limits()
            )
            .is_err()
        );
    }

    #[test]
    fn single_oversized_input_blocks() {
        assert!(
            evaluate_face_coverage(
                &[input("a.yaml", 1_048_577)],
                &[face(&["a.yaml"], 1)],
                limits()
            )
            .is_err()
        );
    }

    #[test]
    fn count_boundary_512_passes() {
        let inputs: Vec<_> = (0..512).map(|i| input(&format!("{i}.yaml"), 1)).collect();
        let paths: Vec<_> = inputs.iter().map(|i| i.path.clone()).collect();
        let observed = FaceObservation {
            paths,
            source_bytes: 512,
            ..face(&[], 0)
        };
        assert!(evaluate_face_coverage(&inputs, &[observed], limits()).is_ok());
    }

    #[test]
    fn byte_boundary_1048576_passes() {
        assert!(
            evaluate_face_coverage(
                &[input("a.yaml", 1_048_576)],
                &[face(&["a.yaml"], 1_048_576)],
                limits()
            )
            .is_ok()
        );
    }

    #[test]
    fn zero_shard_size_blocks() {
        assert!(
            derive_faces(
                "oya",
                &[input("a.yaml", 1)],
                ExtractionDeclaration::FixedShards { shard_size: 0 }
            )
            .is_err()
        );
    }

    #[test]
    fn shard_size_513_blocks() {
        assert!(
            derive_faces(
                "oya",
                &[input("a.yaml", 1)],
                ExtractionDeclaration::FixedShards { shard_size: 513 }
            )
            .is_err()
        );
    }

    #[test]
    fn shuffled_inputs_derive_identical_shards() {
        let a = [input("b.yaml", 1), input("a.yaml", 1), input("c.yaml", 1)];
        let b = [input("c.yaml", 1), input("b.yaml", 1), input("a.yaml", 1)];
        assert_eq!(
            derive_faces(
                "oya",
                &a,
                ExtractionDeclaration::FixedShards { shard_size: 2 }
            ),
            derive_faces(
                "oya",
                &b,
                ExtractionDeclaration::FixedShards { shard_size: 2 }
            )
        );
    }

    #[test]
    fn package_local_globs_exclude_non_yaml_candidates() {
        let faces = derive_faces(
            "oya/example",
            &[
                input("oya/example/a.yaml", 1),
                input("oya/example/b.yml", 1),
                input("oya/example/notes.txt", 1),
            ],
            ExtractionDeclaration::FixedShards { shard_size: 256 },
        )
        .unwrap();
        assert_eq!(faces[0].paths, ["oya/example/a.yaml", "oya/example/b.yml"]);
    }

    #[test]
    fn ordinal_zero_is_unsuffixed_and_no_0000_exists() {
        let faces = derive_faces(
            "oya",
            &[input("a.yaml", 1), input("b.yaml", 1)],
            ExtractionDeclaration::FixedShards { shard_size: 1 },
        )
        .unwrap();
        assert_eq!(faces[0].label, "root//oya:corpus-yaml-facts");
        assert!(faces.iter().all(|face| !face.label.ends_with("-0000")));
        assert!(faces[1].label.ends_with("-0001"));
    }

    #[test]
    fn root_4082_derives_15x256_plus242() {
        let inputs: Vec<_> = (0..4_082)
            .map(|i| input(&format!("{i:04}.yaml"), 1))
            .collect();
        let faces = derive_faces(
            "oya",
            &inputs,
            ExtractionDeclaration::FixedShards { shard_size: 256 },
        )
        .unwrap();
        assert_eq!(faces.len(), 16);
        assert!(faces[..15].iter().all(|face| face.paths.len() == 256));
        assert_eq!(faces[15].paths.len(), 242);
    }

    fn declaration(text: &str) -> Result<ExtractionDeclaration, DeclarationError> {
        extraction_declaration("oya/ci-webhook-gateway", text)
    }

    const CANONICAL: &str = r#"
load("//governance/corpus/extract:yaml_facts.bzl", "corpus_yaml_facts_shards")
corpus_yaml_facts_shards(
    srcs = glob(["**/*.yaml", "**/*.yml"]),
    shard_size = 256,
)
"#;

    #[test]
    fn canonical_load_and_macro_call_passes() {
        assert_eq!(
            declaration(CANONICAL).unwrap(),
            ExtractionDeclaration::FixedShards { shard_size: 256 }
        );
    }

    #[test]
    fn harmless_formatting_and_comments_preserve_ast() {
        let text = CANONICAL.replace("load(", "# harmless\nload( ");
        assert_eq!(
            declaration(&text).unwrap(),
            ExtractionDeclaration::FixedShards { shard_size: 256 }
        );
    }

    const CANONICAL_LITERAL_CMD: &str = "$(exe //governance/corpus/extract:yaml-facts) --target root//oya/ci-webhook-gateway:corpus-yaml-facts --prefix oya/ci-webhook-gateway --out $OUT $SRCS";

    fn literal(cmd: &str, out: &str) -> String {
        format!(
            "genrule(name = \"corpus-yaml-facts\", srcs = glob([\"**/*.yaml\", \"**/*.yml\"]), out = \"{out}\", cmd = \"{cmd}\")"
        )
    }

    #[test]
    fn canonical_ci_webhook_literal_passes() {
        assert_eq!(
            declaration(&literal(CANONICAL_LITERAL_CMD, "yaml-facts.json")).unwrap(),
            ExtractionDeclaration::LiteralSingle
        );
    }

    #[test]
    fn literal_touch_command_spoof_blocks() {
        assert!(declaration(&literal("touch $OUT", "yaml-facts.json")).is_err());
    }

    #[test]
    fn literal_wrong_output_blocks() {
        assert!(declaration(&literal(CANONICAL_LITERAL_CMD, "other.json")).is_err());
    }

    #[test]
    fn literal_wrong_target_blocks() {
        assert!(
            declaration(&literal(
                &CANONICAL_LITERAL_CMD.replace(
                    "root//oya/ci-webhook-gateway:corpus-yaml-facts",
                    "root//oya/wrong:corpus-yaml-facts"
                ),
                "yaml-facts.json"
            ))
            .is_err()
        );
    }

    #[test]
    fn literal_wrong_prefix_blocks() {
        assert!(
            declaration(&literal(
                &CANONICAL_LITERAL_CMD
                    .replace("--prefix oya/ci-webhook-gateway", "--prefix oya/wrong"),
                "yaml-facts.json"
            ))
            .is_err()
        );
    }

    #[test]
    fn literal_wrong_extractor_blocks() {
        assert!(
            declaration(&literal(
                &CANONICAL_LITERAL_CMD.replace(
                    "//governance/corpus/extract:yaml-facts",
                    "//wrong:yaml-facts"
                ),
                "yaml-facts.json"
            ))
            .is_err()
        );
    }

    #[test]
    fn literal_and_macro_declarations_conflict() {
        assert!(declaration(&format!("{CANONICAL}\ngenrule(name = \"corpus-yaml-facts\", srcs = glob([\"**/*.yaml\", \"**/*.yml\"]), out = \"yaml-facts.json\", cmd = \"extract\")")).is_err());
    }

    #[test]
    fn load_without_macro_call_blocks() {
        assert!(
            declaration(
                "load(\"//governance/corpus/extract:yaml_facts.bzl\", \"corpus_yaml_facts_shards\")"
            )
            .is_err()
        );
    }

    #[test]
    fn macro_text_in_comment_or_string_blocks() {
        assert_eq!(
            declaration("# corpus_yaml_facts_shards(srcs = glob([\"**/*.yaml\", \"**/*.yml\"]), shard_size = 256)\nX = \"corpus_yaml_facts_shards\"").unwrap(),
            ExtractionDeclaration::None
        );
    }

    #[test]
    fn wrong_load_source_blocks() {
        assert!(
            declaration(&CANONICAL.replace(
                "//governance/corpus/extract:yaml_facts.bzl",
                "//wrong:yaml_facts.bzl"
            ))
            .is_err()
        );
    }

    #[test]
    fn imported_alias_blocks() {
        assert!(
            declaration(&CANONICAL.replace(
                "\"corpus_yaml_facts_shards\")",
                "corpus_yaml_facts_shards = \"alias\")"
            ))
            .is_err()
        );
    }

    #[test]
    fn partial_glob_blocks() {
        assert!(declaration(&CANONICAL.replace(", \"**/*.yml\"", "")).is_err());
    }

    #[test]
    fn glob_excludes_block() {
        assert!(
            declaration(&CANONICAL.replace(
                "glob([\"**/*.yaml\", \"**/*.yml\"])",
                "glob([\"**/*.yaml\", \"**/*.yml\"], exclude = [\"x.yaml\"])"
            ))
            .is_err()
        );
    }

    #[test]
    fn selected_or_concatenated_srcs_block() {
        assert!(
            declaration(&CANONICAL.replace(
                "glob([\"**/*.yaml\", \"**/*.yml\"])",
                "glob([\"**/*.yaml\"]) + glob([\"**/*.yml\"])"
            ))
            .is_err()
        );
        assert!(
            declaration(&CANONICAL.replace(
                "glob([\"**/*.yaml\", \"**/*.yml\"])",
                "select({\"DEFAULT\": []})"
            ))
            .is_err()
        );
    }

    #[test]
    fn duplicate_macro_calls_block() {
        assert!(declaration(&format!("{CANONICAL}{CANONICAL}")).is_err());
    }

    #[test]
    fn opaque_or_nonliteral_shard_size_blocks() {
        assert!(declaration(&CANONICAL.replace("256", "SIZE")).is_err());
        assert!(declaration(&CANONICAL.replace("256", "select({\"DEFAULT\": 256})")).is_err());
    }

    #[test]
    fn incomplete_literal_genrule_blocks() {
        assert!(declaration("genrule(name = \"corpus-yaml-facts\")").is_err());
    }
}
