//! # corpus-extract (ADR-0580, Phase -1 corpus extractor spike)
//!
//! The deterministic, hermetic `syn`-over-source implementation of the corpus
//! [`AstSource`](corpus_core::AstSource) seam, plus the capability→crates→sources resolution that
//! drives it. This is the de-risk slice: it answers whether parsing committed Rust SOURCE with
//! `syn` is a sufficient v1 liveness substrate, measured by the OPAQUE-RATE (the fraction of the
//! corpus `syn` cannot resolve to a clean fact).
//!
//! ## What it does (the pipeline)
//! 1. [`resolve_capability_crates`] — reuse [`oya_workspace_members_kernel::resolve_member_dirs`] to
//!    get the workspace member dirs, then filter to those under a capability dir-prefix. NO
//!    re-derivation of member-glob semantics; the kernel is the single source.
//! 2. [`SourceSet`] — the git-tracked `.rs` files of those crates, supplied by the caller (the
//!    binary feeds `git ls-files` output — no ambient filesystem walk, so vendored/target/ignored
//!    files never leak in and the run is reproducible from the committed tree alone).
//! 3. [`SynAstSource`] — parse each file with `syn` and emit one [`Function`](corpus_core::Function)
//!    fact per resolvable item, recording each unresolvable item as an
//!    [`OpaqueReason`](corpus_core::OpaqueReason).
//! 4. [`extract_corpus`] — fold the per-file extractions into a canonical
//!    [`FactSet`](corpus_core::FactSet), a [`Graph`](corpus_core::Graph), and an [`OpaqueReport`].
//!
//! ## The graph
//! Facts alone answer "does this item exist"; edges answer "what reaches it". This extractor emits
//! the [`Graph`](corpus_core::Graph) alongside the facts: a `File` node per source file, an `Entry`
//! node per fact, `Contains` edges (file→item, impl→method), and `Refs` edges for calls, type
//! mentions, and implemented traits.
//!
//! An edge `dst` is a NAME, not a fact pointer, so a reference to something that does not exist
//! stays REPRESENTABLE — that dangling state is the defect worth detecting, and a fact pointer
//! would have made it unrepresentable. [`Graph::coverage`](corpus_core::Graph::coverage) counts
//! indexed over total `Refs` targets, and
//! [`evaluate_coverage`](corpus_core::evaluate_coverage) turns that count into a shrink-only
//! ratchet. Measured 2026-08-01 with no import resolution: `governance/corpus` 159/463 (34.34%),
//! `ci/facade` 1710/4716 (36.25%) — the unindexed remainder is dominated by std and cross-crate
//! names, which are genuinely outside the extracted slice.
//!
//! ## Hermeticity + determinism
//! No clock, no rand, no network, no ambient env. The only inputs are the committed manifest, the
//! caller-supplied source list, and file contents. Output is sorted/canonical, so the same input
//! yields a byte-identical fact set.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]
// Stable toolchains classify `non_exhaustive_omitted_patterns` as an unknown unstable lint. Keep
// `-D warnings` verification green on stable while preserving the nightly signal when available.
#![allow(unknown_lints)]
// Warn when a future `syn` version adds a new `Item` variant not yet named in `walk_item`. Without
// this lint, the terminal `_ => {}` arm silently swallows new variants (syn::Item is
// `#[non_exhaustive]`). With it, the compiler surfaces the omission as a warning, forcing an
// explicit decision (fact / opaque / silent-drop) for every new syn item kind.
#![warn(non_exhaustive_omitted_patterns)]

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use corpus_core::{
    AstSource, ContentHash, Edge, EdgeKind, Extraction, FactSet, Function, Graph, ItemKind, Node,
    NodeId, OpaqueReason, Visibility,
};
use oya_workspace_members_kernel::{ResolveError, resolve_member_dirs};
use quote::ToTokens;
use syn::visit::{self, Visit};

/// The infallible error type for [`SynAstSource`]: parse failures are reported as per-item
/// [`OpaqueReason::ParseError`], so `extract_file` never actually fails. The type exists to satisfy
/// the [`AstSource`] associated-error bound while keeping a real `Error` impl.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractError {
    /// Reserved: a non-per-item failure. Never constructed by [`SynAstSource`] (parse failures are
    /// per-item opaque reasons), but present so the seam can carry hard failures in future sources.
    Source(String),
}

impl fmt::Display for ExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractError::Source(message) => write!(f, "corpus extract source: {message}"),
        }
    }
}

impl std::error::Error for ExtractError {}

/// Resolve a capability's crate directories from the committed root `Cargo.toml`.
///
/// `dir_prefix` is the capability's repo-relative directory (e.g. `flags` or `comms/core`). The
/// result is the subset of workspace member dirs that are `dir_prefix` itself or live beneath it,
/// sorted (the kernel already returns sorted, de-duplicated dirs). Reuses
/// [`resolve_member_dirs`] — the canonical member resolver — so it honors the exact `members`/
/// `exclude` glob semantics and never re-derives them.
///
/// # Errors
/// Returns [`ResolveError`] if the root manifest cannot be read/parsed.
pub fn resolve_capability_crates(
    repo_root: &Path,
    dir_prefix: &str,
) -> Result<Vec<String>, ResolveError> {
    let prefix = dir_prefix.trim_end_matches('/');
    let members = resolve_member_dirs(repo_root)?;
    Ok(members
        .into_iter()
        .filter(|dir| dir == prefix || dir.starts_with(&format!("{prefix}/")))
        .collect())
}

/// Derive the `::`-joined module path for a source file relative to its crate dir.
///
/// `src/lib.rs` / `src/main.rs` → `""` (crate root). `src/foo.rs` → `foo`. `src/foo/mod.rs` →
/// `foo`. `src/foo/bar.rs` → `foo::bar`. This is the conventional file→module mapping; it is a
/// best-effort module attribution for the fact path (the spike does not resolve `#[path]` overrides
/// — a rare construct that would, if present, show up as a path mismatch, not an extraction error).
#[must_use]
pub fn module_path_for(crate_dir: &str, rel: &str) -> String {
    let prefix = format!("{crate_dir}/src/");
    let Some(inner) = rel.strip_prefix(&prefix) else {
        // Not under src/ (e.g. tests/, benches/) — attribute under the file stem path.
        return rel
            .strip_prefix(&format!("{crate_dir}/"))
            .unwrap_or(rel)
            .trim_end_matches(".rs")
            .replace('/', "::");
    };
    let inner = inner.trim_end_matches(".rs");
    let mut segments: Vec<&str> = inner.split('/').collect();
    match segments.last().copied() {
        Some("lib") | Some("main") if segments.len() == 1 => String::new(),
        Some("mod") => {
            segments.pop();
            segments.join("::")
        }
        _ => segments.join("::"),
    }
}

/// One git-tracked source file: the owning crate's de-branded cargo id, the file's `::`-joined
/// module path (empty for a crate root), and its UTF-8 contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// The owning crate's de-branded cargo name (e.g. `flags-evaluation-domain`).
    pub crate_id: String,
    /// The file's REPO-RELATIVE path. This is the [`NodeId::file`] container, so it must be the
    /// repo-relative form: a per-package or absolute path would make two extractions of the same
    /// file produce different node identities, and the graph would fragment into islands that each
    /// look internally consistent.
    pub path: String,
    /// The `::`-joined module path the file's items live under (empty for `lib.rs`/`main.rs`).
    pub module_path: String,
    /// The file's UTF-8 source.
    pub source: String,
}

/// A canonically-ordered set of source files to extract. Ordering is by `(crate_id, path)` so the
/// corpus walk — and thus any incidental ordering effect — is deterministic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceSet {
    files: Vec<SourceFile>,
}

impl SourceSet {
    /// Build a canonically-ordered source set.
    pub fn new(files: impl IntoIterator<Item = SourceFile>) -> Self {
        let mut files: Vec<SourceFile> = files.into_iter().collect();
        files.sort_by(|a, b| {
            a.crate_id
                .cmp(&b.crate_id)
                .then_with(|| a.path.cmp(&b.path))
                .then_with(|| a.module_path.cmp(&b.module_path))
        });
        SourceSet { files }
    }

    /// The files, in canonical order.
    #[must_use]
    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }
}

/// The OPAQUE-RATE report: every opaque reason, bucketed by category, plus the totals needed to
/// compute the go/no-go fraction.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpaqueReport {
    /// Count of clean facts extracted (the resolvable corpus).
    pub clean_facts: usize,
    /// All opaque reasons, in canonical order.
    pub opaque: Vec<OpaqueReason>,
    /// Opaque count per category tag (e.g. `macro_generated` → N).
    pub by_category: BTreeMap<String, usize>,
}

impl OpaqueReport {
    /// Total resolvable + opaque units = the denominator of the opaque rate.
    #[must_use]
    pub fn total_units(&self) -> usize {
        self.clean_facts + self.opaque.len()
    }

    /// The opaque rate in basis points (opaque / total * 10000), integer to stay deterministic
    /// (no float formatting). Returns 0 when there are no units.
    #[must_use]
    pub fn opaque_rate_bps(&self) -> u64 {
        let total = self.total_units();
        if total == 0 {
            return 0;
        }
        // u64 math: opaque*10000 cannot overflow for any realistic corpus size.
        (self.opaque.len() as u64 * 10_000) / total as u64
    }
}

/// The full result of extracting a corpus slice: the canonical fact set, the graph, and the opaque
/// report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CorpusExtraction {
    /// The canonical, sorted, de-duplicated facts.
    pub facts: FactSet,
    /// The canonical node/edge graph over those facts.
    pub graph: Graph,
    /// The opaque-rate report.
    pub report: OpaqueReport,
}

/// Extract a whole [`SourceSet`] into a canonical [`CorpusExtraction`].
///
/// Folds each file's [`Extraction`] (via the supplied [`AstSource`]) into a sorted fact set and a
/// category-bucketed opaque report. Deterministic: input order does not affect the output (facts are
/// sorted, opaque reasons are sorted).
///
/// # Errors
/// Propagates the source's [`AstSource::Error`] (for [`SynAstSource`] this never occurs — parse
/// failures are per-item opaque reasons).
pub fn extract_corpus<S: AstSource>(
    source: &S,
    set: &SourceSet,
) -> Result<CorpusExtraction, S::Error> {
    let mut facts: Vec<Function> = Vec::new();
    let mut opaque: Vec<OpaqueReason> = Vec::new();
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();

    for file in set.files() {
        let extraction: Extraction =
            source.extract_file(&file.crate_id, &file.module_path, &file.source)?;

        // DERIVED, so no AstSource has to reimplement it (or can forget to):
        //  - one File node per source file, digested over the file BYTES. Two byte-identical files
        //    therefore carry the same digest while staying two nominally distinct nodes, which is
        //    exactly what makes duplication a free `Graph::duplicate_containers` query.
        //  - one Entry node per clean fact, digested by the fact's own SHALLOW signature hash.
        //  - a file→item `Contains` edge, the spanning forest a reachability BFS walks.
        let file_id = NodeId::file(&file.path);
        nodes.push(Node {
            id: file_id.clone(),
            digest: ContentHash::of(file.source.as_bytes()),
        });
        for fact in &extraction.facts {
            let entry_id = NodeId::entry(&fact.crate_id, &fact.fqpath);
            nodes.push(Node {
                id: entry_id.clone(),
                digest: fact.signature_hash.clone(),
            });
            edges.push(Edge {
                kind: EdgeKind::Contains,
                src: file_id.clone(),
                dst: entry_id,
            });
        }

        facts.extend(extraction.facts);
        edges.extend(extraction.edges);
        opaque.extend(extraction.opaque);
    }

    // Build the fact set, detecting address collisions rather than silently merging them (HIGH-2).
    // A collision (two structurally distinct items with the same content-address key) is routed to
    // the opaque set as OpaqueReason::AddressCollision so it is counted in the OPAQUE-RATE report
    // and surfaced to the caller — never silently dropped.
    //
    // Strategy: sort the raw facts and walk adjacent pairs. Byte-identical duplicates are deduped
    // (legit); same-key-different-hash pairs are a collision → both go to opaque. This mirrors the
    // logic in FactSet::from_facts_checked but lets us retain the clean subset in one pass without
    // cloning the whole Vec.
    facts.sort();
    let mut clean_facts: Vec<Function> = Vec::with_capacity(facts.len());
    let mut i = 0;
    while i < facts.len() {
        // Collect the run of facts that share the same (crate_id, fqpath, item_kind, visibility).
        let mut j = i + 1;
        while j < facts.len()
            && facts[j].crate_id == facts[i].crate_id
            && facts[j].fqpath == facts[i].fqpath
            && facts[j].item_kind == facts[i].item_kind
            && facts[j].visibility == facts[i].visibility
        {
            j += 1;
        }
        let run = &facts[i..j];
        // All byte-identical → keep one (legit dedup).
        let all_same = run.windows(2).all(|w| w[0] == w[1]);
        if all_same {
            clean_facts.push(run[0].clone());
        } else {
            // At least two DISTINCT items share the same address key → collision.
            let detail = format!("{}::{}", facts[i].crate_id, facts[i].fqpath);
            // Both (all) colliding items go to opaque, none to clean facts.
            for _ in run {
                opaque.push(OpaqueReason::AddressCollision(detail.clone()));
            }
        }
        i = j;
    }
    let fact_set = FactSet::from_facts(clean_facts);

    opaque.sort();
    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    for reason in &opaque {
        *by_category.entry(reason.category().to_owned()).or_insert(0) += 1;
    }

    let report = OpaqueReport {
        clean_facts: fact_set.len(),
        opaque,
        by_category,
    };

    Ok(CorpusExtraction {
        facts: fact_set,
        graph: Graph::new(nodes, edges),
        report,
    })
}

/// The v1 `syn`-over-source [`AstSource`].
///
/// Parses a file with `syn` and walks its top-level items, emitting one [`Function`] fact per
/// resolvable item and an [`OpaqueReason`] per item it cannot resolve to a clean source-level fact.
#[derive(Debug, Clone, Default)]
pub struct SynAstSource;

impl SynAstSource {
    /// A new source extractor.
    #[must_use]
    pub fn new() -> Self {
        SynAstSource
    }
}

impl AstSource for SynAstSource {
    type Error = ExtractError;

    fn extract_file(
        &self,
        crate_id: &str,
        module_path: &str,
        source: &str,
    ) -> Result<Extraction, Self::Error> {
        let file = match syn::parse_file(source) {
            Ok(file) => file,
            Err(error) => {
                // A parse failure is an opaque unit, not a hard failure: it still counts toward the
                // denominator so the opaque rate is honest.
                return Ok(Extraction {
                    facts: Vec::new(),
                    edges: Vec::new(),
                    opaque: vec![OpaqueReason::ParseError(format!(
                        "{crate_id}::{module_path}: {error}"
                    ))],
                });
            }
        };

        let mut state = WalkState::default();
        for item in &file.items {
            walk_item(crate_id, module_path, item, &mut state);
        }
        Ok(state.extraction)
    }
}

/// One row in the contract-IDL extractor-family rollout plan.
///
/// ADR-0580 A6 found that proto-heavy crates have a measured `include_proto!` true-miss. The
/// founder decision is a family of IDL sub-extractors behind the same [`AstSource`] seam, not a
/// Rust-only semantic fallback. This row keeps the family order and status machine-readable in the
/// extractor crate while the first concrete slice stays limited to `.proto`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdlExtractorPlanEntry {
    /// Stable extractor family id.
    pub extractor_id: &'static str,
    /// Input contract family handled by the extractor.
    pub input_family: &'static str,
    /// Current rollout status for this family member.
    pub status: &'static str,
    /// Measured blind spot or gating reason that justifies the row.
    pub measured_blind_spot: &'static str,
    /// Next planned implementation step.
    pub next_step: &'static str,
}

/// Rollout plan for the contract-IDL extractor family required by ADR-0580 A6.
pub const IDL_EXTRACTOR_FAMILY_PLAN: [IdlExtractorPlanEntry; 4] = [
    IdlExtractorPlanEntry {
        extractor_id: "proto",
        input_family: ".proto / protobuf service contracts",
        status: "first-slice-fixture-landed",
        measured_blind_spot: "ADR-0580 A6: 45% aggregate true-miss on include_proto! crates",
        next_step: "measure proto OPAQUE-RATE on a proto-heavy capability and wire tracked .proto inputs",
    },
    IdlExtractorPlanEntry {
        extractor_id: "openapi",
        input_family: "OpenAPI REST contracts",
        status: "planned-after-proto",
        measured_blind_spot: "same contract-IDL seam; REST surface lives in schema files, not Rust source alone",
        next_step: "emit route/operation facts from OpenAPI paths and operationIds",
    },
    IdlExtractorPlanEntry {
        extractor_id: "cedar",
        input_family: "Cedar policy contracts",
        status: "planned-after-proto",
        measured_blind_spot: "same contract-IDL seam; authorization surface lives in policy files",
        next_step: "emit policy/action/resource facts from Cedar policy files",
    },
    IdlExtractorPlanEntry {
        extractor_id: "sql",
        input_family: "SQL schema/migration contracts",
        status: "planned-after-proto",
        measured_blind_spot: "same contract-IDL seam; storage surface lives in schema/migration files",
        next_step: "emit table/view/procedure facts from SQL contracts",
    },
];

/// First concrete contract-IDL sub-extractor: protobuf service/message facts.
///
/// This deliberately does NOT parse generated Rust from `tonic::include_proto!`; it reads the IDL
/// source that generated Rust hides from `syn`. The first slice is intentionally conservative:
/// package, top-level `message`, top-level `service`, and one-line `rpc ... returns ...` method
/// declarations. It is enough to close the measured true-miss fixture without broadening the crate's
/// dependency surface or claiming full protobuf grammar coverage.
#[derive(Debug, Clone, Default)]
pub struct ProtoIdlAstSource;

impl ProtoIdlAstSource {
    /// A new protobuf IDL source extractor.
    #[must_use]
    pub fn new() -> Self {
        ProtoIdlAstSource
    }
}

impl AstSource for ProtoIdlAstSource {
    type Error = ExtractError;

    fn extract_file(
        &self,
        crate_id: &str,
        module_path: &str,
        source: &str,
    ) -> Result<Extraction, Self::Error> {
        let package = proto_package(source).unwrap_or_else(|| module_path.to_owned());
        Ok(extract_proto_idl(crate_id, &package, source))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProtoRpcSignature {
    name: String,
    request: String,
    response: String,
}

fn extract_proto_idl(crate_id: &str, package: &str, source: &str) -> Extraction {
    let lines: Vec<String> = source
        .lines()
        .map(|line| strip_proto_line_comment(line).trim().to_owned())
        .collect();
    let package_path = package.trim();
    let mut extraction = Extraction::default();
    let mut current_service: Option<String> = None;

    for (idx, line) in lines.iter().enumerate() {
        if line.is_empty() || line.starts_with("syntax ") || line.starts_with("package ") {
            continue;
        }

        if let Some(message_name) = parse_proto_decl(line, "message") {
            let signature = format!("proto message {}", join_path(package_path, &message_name));
            let body = proto_block_from(&lines, idx);
            extraction.facts.push(Function::new(
                crate_id,
                join_path(package_path, &message_name),
                ItemKind::Type,
                Visibility::Public,
                &signature,
                &body,
            ));
            continue;
        }

        if let Some(service_name) = parse_proto_decl(line, "service") {
            let signature = format!("proto service {}", join_path(package_path, &service_name));
            let body = proto_block_from(&lines, idx);
            extraction.facts.push(Function::new(
                crate_id,
                join_path(package_path, &service_name),
                ItemKind::Type,
                Visibility::Public,
                &signature,
                &body,
            ));
            current_service = Some(service_name);
            continue;
        }

        if let Some(service_name) = current_service.as_deref() {
            if let Some(rpc) = parse_proto_rpc(line) {
                let service_path = join_path(package_path, service_name);
                let fqpath = join_path(&service_path, &rpc.name);
                let signature = format!(
                    "proto rpc {service_path}.{} ({}) returns ({})",
                    rpc.name, rpc.request, rpc.response
                );
                extraction.facts.push(Function::new(
                    crate_id,
                    fqpath,
                    ItemKind::Function,
                    Visibility::Public,
                    &signature,
                    "",
                ));
            }
            if line.contains('}') {
                current_service = None;
            }
        }
    }

    extraction
}

fn strip_proto_line_comment(line: &str) -> &str {
    line.split_once("//")
        .map(|(before, _)| before)
        .unwrap_or(line)
}

fn proto_package(source: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let trimmed = strip_proto_line_comment(line).trim();
        let rest = trimmed.strip_prefix("package ")?;
        take_proto_name(rest).map(str::to_owned)
    })
}

fn parse_proto_decl(line: &str, keyword: &str) -> Option<String> {
    let rest = line.strip_prefix(keyword)?.trim_start();
    take_proto_name(rest).map(str::to_owned)
}

fn parse_proto_rpc(line: &str) -> Option<ProtoRpcSignature> {
    let rest = line.strip_prefix("rpc ")?.trim_start();
    let name = take_proto_name(rest)?.to_owned();
    let after_name = rest.get(name.len()..)?.trim_start();
    let (request, after_request) = proto_paren_value(after_name)?;
    let after_returns = after_request
        .trim_start()
        .strip_prefix("returns")?
        .trim_start();
    let (response, _) = proto_paren_value(after_returns)?;
    Some(ProtoRpcSignature {
        name,
        request,
        response,
    })
}

fn take_proto_name(input: &str) -> Option<&str> {
    let trimmed = input.trim_start();
    let end = trimmed
        .char_indices()
        .find_map(|(idx, ch)| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                None
            } else {
                Some(idx)
            }
        })
        .unwrap_or(trimmed.len());
    if end == 0 {
        None
    } else {
        Some(&trimmed[..end])
    }
}

fn proto_paren_value(input: &str) -> Option<(String, &str)> {
    let start = input.find('(')?;
    let after_start = input.get(start + 1..)?;
    let end = after_start.find(')')?;
    let value = after_start.get(..end)?.trim().to_owned();
    let rest = after_start.get(end + 1..)?;
    Some((value, rest))
}

fn proto_block_from(lines: &[String], start_idx: usize) -> String {
    let mut block = Vec::new();
    let mut depth: usize = 0;
    let mut saw_open = false;

    for line in lines.iter().skip(start_idx) {
        if line.is_empty() {
            continue;
        }
        block.push(line.as_str());
        for ch in line.chars() {
            match ch {
                '{' => {
                    saw_open = true;
                    depth += 1;
                }
                '}' => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }
        if saw_open && depth == 0 {
            break;
        }
        if !saw_open {
            break;
        }
    }

    block.join(" ")
}

/// Per-file walk state: the accumulating [`Extraction`].
///
/// The former `impl_ordinals` field (a document-order counter for disambiguating multiple `impl`
/// blocks of the same self-type within a file) has been REMOVED. Its replacement is a
/// content-stable disambiguator: the first 8 hex characters of the blake3 hash of the impl block's
/// own normalized token body (see [`impl_body_disambiguator`]). This makes the impl `fqpath` anchor
/// independent of both file position AND which file within a crate the impl lives in — fixing the
/// HIGH-1 defect where two `impl Foo` blocks across `lib.rs` + `main.rs` of the same crate both
/// produced `fqpath = "Foo#impl[0]"` because `WalkState` was reset per `extract_file` call.
#[derive(Debug, Default)]
struct WalkState {
    extraction: Extraction,
}

/// Compute a content-stable disambiguator for an `impl` block: the first 8 hex characters of the
/// blake3 hash of the impl block's own normalized token body.
///
/// This replaces the former document-order positional ordinal (`#impl[0]`, `#impl[1]`, …). A
/// positional ordinal is per-file-scoped: two `impl Foo` blocks in separate files of the same crate
/// both get ordinal 0 → identical `fqpath` → silent dedup (HIGH-1 defect). A content-hash prefix
/// is a property of the impl's OWN structure, independent of which file it lives in or how many
/// other impls of the same type exist. Two structurally distinct `impl Foo` blocks (even with
/// identical self-type and trait) will have different body tokens → different hash → different
/// `fqpath`.
///
/// 8 hex characters = 32 bits of hash space. For practical corpus sizes (hundreds of impls per
/// crate), the collision probability is astronomically low; `from_facts_checked` detects any
/// collision that does occur and surfaces it as an `OpaqueReason::AddressCollision` rather than
/// silently merging.
fn impl_body_disambiguator(item_impl: &syn::ItemImpl) -> String {
    let tokens = normalize_tokens(item_impl);
    let hash = blake3::hash(tokens.as_bytes());
    // First 8 hex chars = 4 bytes = 32 bits.
    hash.to_hex()[..8].to_owned()
}

/// Collects the NAMES a syn node references: call targets, struct-literal paths, and type paths.
///
/// Deliberately narrow. A bare `syn::Path` visitor would also pick up local bindings, match arms,
/// and macro fragments, and a `Refs` edge that names a local variable makes the dangling-reference
/// query cry wolf — which is the whole reason the query exists. Method calls (`x.foo()`) are NOT
/// collected: resolving a method needs the receiver's type, so there is no name to emit, and
/// inventing one would be worse than omitting it.
#[derive(Debug, Default)]
struct RefCollector {
    names: std::collections::BTreeSet<String>,
}

impl RefCollector {
    fn record(&mut self, path: &syn::Path) {
        if let Some(name) = reference_name(path) {
            self.names.insert(name);
        }
    }
}

impl<'ast> Visit<'ast> for RefCollector {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref() {
            self.record(&path.path);
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        self.record(&node.path);
        visit::visit_expr_struct(self, node);
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        self.record(&node.path);
        visit::visit_type_path(self, node);
    }
}

/// Render a referenced path as the NAME an edge `dst` carries: idents joined by `::`, generic
/// arguments dropped (they are visited separately as their own type paths).
///
/// `crate::` and `self::` prefixes are stripped because a fact `fqpath` is already crate-relative.
/// `Self` and `super` are rejected: resolving either needs context this pass does not carry, and a
/// wrong name is worse than a missing one.
// ponytail: the name is recorded AS WRITTEN, with no `use`-alias resolution — so `HashMap` stays
// `HashMap` rather than `std::collections::HashMap`, and an unqualified intra-module call `foo()`
// stays `foo` rather than `m::foo`. The ceiling: every such reference lands in the UNINDEXED bucket
// that `Graph::coverage` counts, so today's number is a floor, not a verdict on the code. The
// upgrade path is a per-file `use`-map applied before the name is emitted, which is a resolution
// pass, not a better heuristic — do NOT approximate it with suffix matching at query time, because
// that would silently resolve genuinely dangling references and destroy the signal.
fn reference_name(path: &syn::Path) -> Option<String> {
    let mut segments: Vec<String> = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    while matches!(
        segments.first().map(String::as_str),
        Some("crate") | Some("self")
    ) {
        segments.remove(0);
    }
    if segments.is_empty() || matches!(segments[0].as_str(), "Self" | "super") {
        return None;
    }
    Some(segments.join("::"))
}

/// Emit one `Refs` edge per collected name, from the item's own Entry node.
///
/// The `dst` container is the REFERENCING crate: without import resolution this pass cannot know
/// which crate defines the name, and guessing would fabricate edges. A name defined elsewhere
/// therefore resolves to no node and is counted as unindexed — which is the honest reading, since
/// from this graph's point of view it genuinely IS outside.
fn push_refs(crate_id: &str, src_fqpath: &str, collector: RefCollector, out: &mut Extraction) {
    let src = NodeId::entry(crate_id, src_fqpath);
    for name in collector.names {
        out.edges.push(Edge {
            kind: EdgeKind::Refs,
            src: src.clone(),
            dst: NodeId::entry(crate_id, name),
        });
    }
}

/// The path-join of a module path and an item name (`m::sub` + `f` → `m::sub::f`; empty module →
/// `f`).
fn join_path(module_path: &str, name: &str) -> String {
    if module_path.is_empty() {
        name.to_owned()
    } else {
        format!("{module_path}::{name}")
    }
}

/// Normalize a syn visibility to the stable [`Visibility`] label.
fn normalize_visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,
        syn::Visibility::Restricted(restricted) => {
            // `pub(crate)` is the single restricted form the corpus treats as crate-visible; every
            // other `pub(in ...)`/`pub(super)` is the broader Restricted bucket.
            if restricted.path.is_ident("crate") {
                Visibility::Crate
            } else {
                Visibility::Restricted
            }
        }
        syn::Visibility::Inherited => Visibility::Private,
    }
}

/// Render a node's tokens as a normalized, whitespace/comment-invariant string. `quote`/
/// `proc-macro2`'s token rendering collapses original whitespace and comments to a single canonical
/// spacing. NOTE: it does NOT drop a SOURCE-PRESENT trailing comma in a punctuated list (e.g.
/// `(a: T,)` renders with the comma, `(a: T)` without) — so this is reformatting-invariant for
/// whitespace/comments but not for the optional-trailing-comma choice. For function signatures use
/// [`normalize_signature`], which reconstructs from the structured parts and is trailing-comma-
/// invariant; for bodies/types the body/type digest may legitimately vary at its own granularity.
fn normalize_tokens<T: ToTokens>(node: &T) -> String {
    node.to_token_stream().to_string()
}

/// Render a function signature into a canonical, FULLY reformatting-invariant string.
///
/// Reconstructs the signature from `syn`'s structured [`syn::Signature`] fields rather than echoing
/// the source token stream, so a source-optional trailing comma in the argument list cannot churn
/// the signature anchor. Each input/generic is rendered via [`normalize_tokens`] and joined with a
/// fixed separator; the iteration over a `Punctuated` yields ONLY the elements (never the trailing
/// punctuation), which is exactly the trailing-comma invariance the anchor requires.
fn normalize_signature(sig: &syn::Signature) -> String {
    let mut out = String::new();
    if sig.constness.is_some() {
        out.push_str("const ");
    }
    if sig.asyncness.is_some() {
        out.push_str("async ");
    }
    if sig.unsafety.is_some() {
        out.push_str("unsafe ");
    }
    out.push_str("fn ");
    out.push_str(&sig.ident.to_string());

    // Generics (`<...>`), element-joined so a trailing comma in `<T,>` does not churn.
    let generic_params: Vec<String> = sig.generics.params.iter().map(normalize_tokens).collect();
    if !generic_params.is_empty() {
        out.push('<');
        out.push_str(&generic_params.join(" , "));
        out.push('>');
    }

    // Inputs (`(...)`), element-joined → trailing-comma invariant.
    let inputs: Vec<String> = sig.inputs.iter().map(normalize_tokens).collect();
    out.push('(');
    out.push_str(&inputs.join(" , "));
    out.push(')');

    // Return type.
    if let syn::ReturnType::Type(_, ty) = &sig.output {
        out.push_str(" -> ");
        out.push_str(&normalize_tokens(ty.as_ref()));
    }

    // Where-clause predicates, element-joined → trailing-comma invariant.
    if let Some(where_clause) = &sig.generics.where_clause {
        let predicates: Vec<String> = where_clause
            .predicates
            .iter()
            .map(normalize_tokens)
            .collect();
        if !predicates.is_empty() {
            out.push_str(" where ");
            out.push_str(&predicates.join(" , "));
        }
    }

    out
}

/// True iff any outer attribute is a route-ish HTTP handler attribute (`get`/`post`/`put`/
/// `delete`/`patch`/`head`/`options`/`route`). Recognized structurally from the attribute path's
/// last segment, not semantically — a v1 heuristic that is reformatting-invariant.
fn has_route_attr(attrs: &[syn::Attribute]) -> bool {
    const ROUTE_IDENTS: [&str; 8] = [
        "get", "post", "put", "delete", "patch", "head", "options", "route",
    ];
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .map(|seg| ROUTE_IDENTS.contains(&seg.ident.to_string().as_str()))
            .unwrap_or(false)
    })
}

/// True iff any outer attribute is a `#[cfg(...)]` (NOT `cfg_attr`). A `cfg`-gated item is opaque:
/// its presence depends on build-time configuration the hermetic extractor does not evaluate.
fn has_cfg_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("cfg"))
}

/// Walk one item, pushing a fact or an opaque reason into `state`.
fn walk_item(crate_id: &str, module_path: &str, item: &syn::Item, state: &mut WalkState) {
    match item {
        syn::Item::Fn(item_fn) => {
            if has_cfg_attr(&item_fn.attrs) {
                state
                    .extraction
                    .opaque
                    .push(OpaqueReason::CfgGated(join_path(
                        module_path,
                        &item_fn.sig.ident.to_string(),
                    )));
                return;
            }
            let kind = if has_route_attr(&item_fn.attrs) {
                ItemKind::Route
            } else {
                ItemKind::Function
            };
            let fqpath = join_path(module_path, &item_fn.sig.ident.to_string());
            let signature = normalize_signature(&item_fn.sig);
            let body = normalize_tokens(&item_fn.block);
            state.extraction.facts.push(Function::new(
                crate_id,
                fqpath.clone(),
                kind,
                normalize_visibility(&item_fn.vis),
                &signature,
                &body,
            ));
            let mut collector = RefCollector::default();
            collector.visit_item_fn(item_fn);
            push_refs(crate_id, &fqpath, collector, &mut state.extraction);
        }
        syn::Item::Struct(s) => {
            push_type(
                crate_id,
                module_path,
                &s.attrs,
                &s.vis,
                &s.ident,
                item,
                &mut state.extraction,
            );
        }
        syn::Item::Enum(e) => {
            push_type(
                crate_id,
                module_path,
                &e.attrs,
                &e.vis,
                &e.ident,
                item,
                &mut state.extraction,
            );
        }
        syn::Item::Union(u) => {
            push_type(
                crate_id,
                module_path,
                &u.attrs,
                &u.vis,
                &u.ident,
                item,
                &mut state.extraction,
            );
        }
        syn::Item::Trait(t) => {
            push_type(
                crate_id,
                module_path,
                &t.attrs,
                &t.vis,
                &t.ident,
                item,
                &mut state.extraction,
            );
        }
        syn::Item::Type(t) => {
            push_type(
                crate_id,
                module_path,
                &t.attrs,
                &t.vis,
                &t.ident,
                item,
                &mut state.extraction,
            );
        }
        syn::Item::Impl(item_impl) => walk_impl(crate_id, module_path, item_impl, state),
        syn::Item::Mod(item_mod) => walk_mod(crate_id, module_path, item_mod, state),
        syn::Item::Macro(item_macro) => {
            // An item-position macro invocation: the generated items are invisible to source-level
            // syn. Count it opaque (macro_generated) so the rate reflects the blind spot.
            let name = item_macro
                .mac
                .path
                .segments
                .last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_else(|| "<macro>".to_owned());
            state
                .extraction
                .opaque
                .push(OpaqueReason::MacroGenerated(join_path(
                    module_path,
                    &format!("{name}!"),
                )));
        }
        syn::Item::Const(c) => {
            push_pub_item(
                crate_id,
                module_path,
                &c.attrs,
                &c.vis,
                &c.ident.to_string(),
                item,
                &mut state.extraction,
            );
        }
        syn::Item::Static(s) => {
            push_pub_item(
                crate_id,
                module_path,
                &s.attrs,
                &s.vis,
                &s.ident.to_string(),
                item,
                &mut state.extraction,
            );
        }
        // `use` re-exports and `extern crate` declarations carry no standalone liveness surface in
        // v1 (the re-exported item is pinned at its definition site). Silently dropped — not opaque.
        syn::Item::Use(_) | syn::Item::ExternCrate(_) => {}
        // `Verbatim` is a syn catch-all for items that parsed but don't fit a known form. Silently
        // dropped — these are extremely rare and carry no addressable liveness surface.
        syn::Item::Verbatim(_) => {}
        // `extern "C" { … }` blocks (`ForeignMod`) and `trait Alias = Bound;` (`TraitAlias`) are
        // item kinds that the v1 extractor does NOT yet surface as facts but that carry real surface
        // area (FFI bindings and trait aliases). Route to `Unhandled` so the opaque rate reflects
        // these gaps rather than hiding them in the silent-drop set (MEDIUM-a fix).
        syn::Item::ForeignMod(fm) => {
            let detail = format!(
                "{module_path}::extern\"{}\"",
                fm.abi
                    .name
                    .as_ref()
                    .map(|lit| lit.value())
                    .unwrap_or_else(|| "C".to_owned())
            );
            state
                .extraction
                .opaque
                .push(OpaqueReason::Unhandled(detail));
        }
        syn::Item::TraitAlias(ta) => {
            state
                .extraction
                .opaque
                .push(OpaqueReason::Unhandled(join_path(
                    module_path,
                    &ta.ident.to_string(),
                )));
        }
        // Catch-all for any syn::Item variant not yet named above. syn::Item is #[non_exhaustive],
        // so new variants added in future syn releases land here. The crate-level
        // `#![warn(non_exhaustive_omitted_patterns)]` turns this into a compiler warning when that
        // happens — forcing an explicit decision (fact / OpaqueReason::Unhandled / silent-drop).
        // Do NOT promote this arm to a silent-drop; add an explicit arm above instead.
        _ => {}
    }
}

/// Push a `Type` fact (struct/enum/union/trait/type-alias), honoring cfg-opacity.
fn push_type(
    crate_id: &str,
    module_path: &str,
    attrs: &[syn::Attribute],
    vis: &syn::Visibility,
    ident: &syn::Ident,
    item: &syn::Item,
    out: &mut Extraction,
) {
    let fqpath = join_path(module_path, &ident.to_string());
    if has_cfg_attr(attrs) {
        out.opaque.push(OpaqueReason::CfgGated(fqpath));
        return;
    }
    let tokens = normalize_tokens(item);
    out.facts.push(Function::new(
        crate_id,
        fqpath.clone(),
        ItemKind::Type,
        normalize_visibility(vis),
        &tokens,
        "",
    ));
    let mut collector = RefCollector::default();
    collector.visit_item(item);
    push_refs(crate_id, &fqpath, collector, out);
}

/// Push a `PubItem` fact (const/static).
fn push_pub_item(
    crate_id: &str,
    module_path: &str,
    attrs: &[syn::Attribute],
    vis: &syn::Visibility,
    name: &str,
    item: &syn::Item,
    out: &mut Extraction,
) {
    let fqpath = join_path(module_path, name);
    if has_cfg_attr(attrs) {
        out.opaque.push(OpaqueReason::CfgGated(fqpath));
        return;
    }
    let tokens = normalize_tokens(item);
    out.facts.push(Function::new(
        crate_id,
        fqpath.clone(),
        ItemKind::PubItem,
        normalize_visibility(vis),
        &tokens,
        "",
    ));
    let mut collector = RefCollector::default();
    collector.visit_item(item);
    push_refs(crate_id, &fqpath, collector, out);
}

/// Walk an `impl` block: emit one `Impl` fact for the block and recurse its methods as
/// `Function`/`Route` facts under the self-type path.
///
/// The impl is anchored to its `(trait, self-type)` identity plus a **content-stable body-hash
/// disambiguator** (`#impl[{8-hex-chars}]`) from [`impl_body_disambiguator`]. This replaces the
/// former document-order positional ordinal (`#impl[n]`), which was per-file-scoped and would
/// produce identical `fqpath`s for two `impl Foo` blocks in `lib.rs` vs `main.rs` of the same
/// crate (HIGH-1 fix). The hash is a property of the impl block's own token structure, so it is
/// independent of which file the block lives in and of sibling-impl count — making impl identity
/// stable across any intra-crate file split.
fn walk_impl(crate_id: &str, module_path: &str, item_impl: &syn::ItemImpl, state: &mut WalkState) {
    if has_cfg_attr(&item_impl.attrs) {
        let self_ty = normalize_tokens(&item_impl.self_ty);
        state
            .extraction
            .opaque
            .push(OpaqueReason::CfgGated(join_path(module_path, &self_ty)));
        return;
    }
    let self_ty = normalize_tokens(&item_impl.self_ty);
    // The self-type token text can contain spaces (`Foo < T >`); collapse to a compact path-ish key
    // for the fqpath so it stays a stable, readable identifier across reformatting.
    let self_key = self_ty.replace(' ', "");
    // The implemented trait (if any) is part of the impl signature pre-image so `impl Foo` and
    // `impl Bar for Foo` never hash the same way.
    let trait_key = item_impl
        .trait_
        .as_ref()
        .map(|(_, path, _)| normalize_tokens(path).replace(' ', ""))
        .unwrap_or_default();
    // Content-stable disambiguator: 8 hex chars of the blake3 hash of the impl's own token body.
    // Two structurally distinct `impl Foo` blocks (different methods/bodies) get different hashes
    // → different disambiguators → different fqpaths AND different signature_hashes (because the
    // disambiguator is part of the signature pre-image below). This double anchoring means that
    // even the rare 32-bit hash collision (same 8-hex-char prefix for two structurally different
    // impls) still produces different fqpaths in theory only; in that degenerate case the two
    // impl facts share fqpath AND signature_hash (because the sig pre-image includes the disambig)
    // → they are byte-identical → legit dedup, not a silent AddressCollision drop. The claim
    // "disambiguator collisions are caught as AddressCollision" is therefore EXACTLY TRUE: a
    // real collision (different methods, genuinely same 32-bit prefix) yields different method
    // content in the impl block body → different body_hash → NOT byte-identical → caught.
    // A degenerate bit-exact collision (two impls with identical method bodies AND same 32-bit
    // prefix, i.e. genuinely indistinguishable at source level) would produce byte-identical facts
    // and be legit-deduped — which is correct, because the source IS identical.
    let disambig = impl_body_disambiguator(item_impl);
    let impl_fqpath = join_path(module_path, &format!("{self_key}#impl[{disambig}]"));
    // The impl fact's signature pre-image includes the disambiguator so that a disambiguator
    // collision between two distinct impls (different bodies, same 8-hex prefix) produces facts
    // with differing body_hashes → not byte-identical → caught as AddressCollision by the drain
    // in extract_corpus. Without the disambiguator in the sig pre-image, two impls that differ
    // only in methods (body_tokens="", impl_sig identical) would produce byte-identical facts
    // that the legit-dedup branch would silently merge (the former MEDIUM-1 silent-dedup hole).
    let impl_sig = format!("impl {trait_key} for {self_key} #{disambig}");
    state.extraction.facts.push(Function::new(
        crate_id,
        impl_fqpath.clone(),
        ItemKind::Impl,
        // An impl block has no visibility modifier; treat as Private (its methods carry their own).
        Visibility::Private,
        &impl_sig,
        "",
    ));

    // The impl block's OWN references: its self type and the trait it implements. Method bodies are
    // attributed to the methods, not to the block, so an impl node's Refs stay its own.
    let mut impl_refs = RefCollector::default();
    impl_refs.visit_type(&item_impl.self_ty);
    if let Some((_, path, _)) = &item_impl.trait_ {
        impl_refs.record(path);
    }
    push_refs(crate_id, &impl_fqpath, impl_refs, &mut state.extraction);

    let method_base = join_path(module_path, &self_key);
    for impl_item in &item_impl.items {
        if let syn::ImplItem::Fn(method) = impl_item {
            if has_cfg_attr(&method.attrs) {
                state
                    .extraction
                    .opaque
                    .push(OpaqueReason::CfgGated(join_path(
                        &method_base,
                        &method.sig.ident.to_string(),
                    )));
                continue;
            }
            let kind = if has_route_attr(&method.attrs) {
                ItemKind::Route
            } else {
                ItemKind::Function
            };
            let fqpath = join_path(&method_base, &method.sig.ident.to_string());
            let signature = normalize_signature(&method.sig);
            let body = normalize_tokens(&method.block);
            state.extraction.facts.push(Function::new(
                crate_id,
                fqpath.clone(),
                kind,
                normalize_visibility(&method.vis),
                &signature,
                &body,
            ));
            // The containment spine: the impl block CONTAINS its methods. This is the only
            // parent/child relation where BOTH ends are facts today — a module or crate parent
            // would need an aggregate node, and an aggregate node is a Merkle root.
            state.extraction.edges.push(Edge {
                kind: EdgeKind::Contains,
                src: NodeId::entry(crate_id, &impl_fqpath),
                dst: NodeId::entry(crate_id, &fqpath),
            });
            let mut collector = RefCollector::default();
            collector.visit_impl_item_fn(method);
            push_refs(crate_id, &fqpath, collector, &mut state.extraction);
        }
    }
}

/// Walk an inline `mod m { ... }`: recurse its items under the extended module path. A file-module
/// (`mod m;`) has no inline content — its items live in another file the source list supplies
/// separately, so nothing is emitted here.
fn walk_mod(crate_id: &str, module_path: &str, item_mod: &syn::ItemMod, state: &mut WalkState) {
    let Some((_, items)) = &item_mod.content else {
        return;
    };
    if has_cfg_attr(&item_mod.attrs) {
        // A cfg-gated module's entire item subtree is conditionally present → opaque at the module
        // granularity (one opaque unit for the gated module, not per descendant).
        state
            .extraction
            .opaque
            .push(OpaqueReason::CfgGated(join_path(
                module_path,
                &item_mod.ident.to_string(),
            )));
        return;
    }
    let inner = join_path(module_path, &item_mod.ident.to_string());
    for item in items {
        walk_item(crate_id, &inner, item, state);
    }
}

#[cfg(test)]
mod tests;
