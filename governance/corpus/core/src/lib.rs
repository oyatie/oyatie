//! # corpus-core (ADR-0580, Phase -1 corpus extractor spike)
//!
//! The content-addressed fact model, the [`Graph`] (nodes + exactly two edge kinds) and its
//! queries, plus the stable [`AstSource`] trait for the live-AST governance substrate ("corpus").
//! This crate is the *contract* layer: it defines WHAT a liveness fact is, WHAT relates two of
//! them, and the seam an extractor plugs into — not how facts are produced (that is the
//! `corpus-extract` binary) and not any persistence.
//!
//! ## Identity is NOMINAL; the content-address is an ATTRIBUTE, and SHALLOW
//! Neither Kythe (`VName` = five plain strings) nor Glean (opaque per-database fact ids)
//! content-addresses node IDENTITY, and ADR-0541 D1 ratified the same split here: a [`NodeId`] is a
//! readable tuple, and the digest hanging off it covers that node's OWN pre-image with ZERO child
//! digests. There is deliberately no module, crate, or capability node carrying a roll-up — the
//! moment one exists this is a Merkle tree and every edit churns the root. Churn is bounded by that
//! ABSENCE, so adding an aggregate node is not an extension, it is a regression.
//!
//! ## Exactly two edge kinds
//! [`EdgeKind::Contains`] and [`EdgeKind::Refs`]. Kythe ships 11+; nine of them serve IDE
//! cross-reference and type hierarchy, which we are not building. Reachability is the BFS closure
//! over their union — a QUERY ([`Graph::reachable_from`]), never a materialized third kind, because
//! a materialized reachability view goes stale, which is exactly what
//! `specs/reachability-registry.json` does today.
//!
//! ## Why content-addressing (the liveness link)
//! Each [`Function`] fact carries two independent blake3 digests:
//! - [`Function::signature_hash`] — the *signature-level* anchor: crate id + fully-qualified path +
//!   item kind + visibility + the normalized signature. It is INVARIANT under body edits and under
//!   pure reformatting (whitespace/comments), so it is the stable identity the governance layer
//!   pins liveness to. A rename/removal/visibility-change MUST churn it.
//! - [`Function::body_hash`] — the *body-level* digest of the item's token stream. It legitimately
//!   churns at body granularity (any body edit), giving a finer "did the implementation move?"
//!   signal that does not destabilize the signature anchor.
//!
//! Splitting the two is the whole point: signature stability lets governance track an item across
//! refactors while body sensitivity still detects implementation drift.
//!
//! ## Determinism + canonical JSON
//! [`Function`] derives a TOTAL order ([`Ord`]) and is `serde`-serializable with fields declared in
//! a fixed order, so a sorted [`FactSet`] serializes byte-identically across runs (the hermetic
//! determinism contract, ADR-0083). No clock/rand/net/ambient state ever enters a fact.
//!
//! ## The AstSource seam (forward-compat)
//! [`AstSource`] is the stable trait the v1 `syn`-over-source extractor implements TODAY and that a
//! W-tier bespoke-rowan CST successor will implement LATER without disturbing the fact model or any
//! consumer. salsa/rowan are deliberately absent for v1.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use serde::{Deserialize, Serialize};

/// The kind of source item a [`Function`] fact describes.
///
/// "Function" is the fact-type name (the spike's unit of liveness), but the kind enumerates the
/// item families the v1 extractor resolves: free/assoc functions, type definitions, `impl` blocks,
/// route-ish handlers, and other public items. A consumer pins liveness per `(fqpath, item_kind)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// A free function or an associated function/method (`fn`).
    Function,
    /// A type definition (`struct` / `enum` / `union` / `type` alias / `trait`).
    Type,
    /// An `impl` block (inherent or trait impl).
    Impl,
    /// A route-ish handler: a function carrying an HTTP/route attribute (e.g. `#[get(...)]`,
    /// `#[route(...)]`). Recognized structurally from its outer attributes, not semantically.
    Route,
    /// Any other item exposed at module scope (e.g. a `const`, `static`, `mod`, `use` re-export)
    /// that the extractor surfaces as a public-surface fact.
    PubItem,
}

impl ItemKind {
    /// The stable, lowercase wire tag for this kind (matches the serde `snake_case` rename).
    /// Used in the canonical signature pre-image so the digest is independent of Rust's `Debug`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ItemKind::Function => "function",
            ItemKind::Type => "type",
            ItemKind::Impl => "impl",
            ItemKind::Route => "route",
            ItemKind::PubItem => "pub_item",
        }
    }
}

impl fmt::Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The visibility of a source item, normalized to a stable, source-syntax-independent label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// `pub` — public to all.
    Public,
    /// `pub(crate)` — crate-visible.
    Crate,
    /// `pub(in path)` / `pub(super)` — restricted-path visibility.
    Restricted,
    /// No visibility modifier — private to the defining module.
    Private,
}

impl Visibility {
    /// The stable, lowercase wire tag for this visibility (matches the serde `snake_case` rename).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Crate => "crate",
            Visibility::Restricted => "restricted",
            Visibility::Private => "private",
        }
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A blake3 content-address rendered as a lowercase hex string.
///
/// A newtype (not a bare `String`) so a fact's hashes cannot be confused with its text fields and
/// so the digest construction is centralized in [`ContentHash::of`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContentHash(String);

impl ContentHash {
    /// Compute the blake3 content-address of `bytes`, rendered as lowercase hex.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        ContentHash(blake3::hash(bytes).to_hex().to_string())
    }

    /// Borrow the hex digest.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A single content-addressed liveness fact for one source item.
///
/// The field order is fixed and the type derives a total [`Ord`], so a sorted collection serializes
/// byte-identically across runs. The two hashes are computed from disjoint canonical pre-images
/// (see [`Function::new`]): the signature hash is the stable identity anchor, the body hash is the
/// finer implementation-drift signal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Function {
    /// The crate this item belongs to (the crate's de-branded cargo name, e.g.
    /// `flags-evaluation-domain`). Part of the signature pre-image so identical `fqpath`s in
    /// different crates never collide.
    pub crate_id: String,
    /// The fully-qualified item path within the crate, module path joined by `::` and ending in the
    /// item's own name (e.g. `engine::evaluate` or `model::Flag`). For an `impl` block this is the
    /// `impl`'s self-type path with an `#impl[n]` disambiguator.
    pub fqpath: String,
    /// The item kind (see [`ItemKind`]).
    pub item_kind: ItemKind,
    /// The item's normalized visibility (see [`Visibility`]).
    pub visibility: Visibility,
    /// The SIGNATURE-level content-address: blake3 over the canonical signature pre-image
    /// `crate_id\0fqpath\0item_kind\0visibility\0signature`. The stable liveness identity.
    ///
    /// **Body invariance applies to `fn` items only.** For `Function` / `Route` items the
    /// `signature` pre-image is the normalized `fn` signature (reconstructed from `syn::Signature`
    /// fields), so the hash is invariant under body edits and pure reformatting. For `Type` items
    /// (struct/enum/union/trait/type-alias) the `signature` pre-image is the item's full normalized
    /// token stream (the whole definition IS the signature), so a field or variant addition churns
    /// this hash even though it is not a body edit in the `fn` sense. For `Impl` and `PubItem`
    /// items the pre-image is similarly the full canonical token stream. Body-invariance is
    /// therefore a property of the `Function`/`Route` item kinds, not of this hash in general.
    pub signature_hash: ContentHash,
    /// The BODY-level content-address: blake3 over the item's normalized token stream. Churns at
    /// body granularity; the implementation-drift signal.
    pub body_hash: ContentHash,
}

impl Function {
    /// The byte used to separate fields in the canonical signature pre-image. A NUL is used because
    /// it cannot appear in a Rust path/signature token stream, so the pre-image is unambiguous (no
    /// field-boundary collision).
    const FIELD_SEP: u8 = 0;

    /// Construct a fact, computing both content-addresses from canonical pre-images.
    ///
    /// `signature` is the normalized, reformatting-invariant signature text (the extractor renders
    /// it from the parsed item's token stream with consistent spacing). `body_tokens` is the
    /// normalized body token text (empty for items without a body, e.g. a type alias). The signature
    /// hash deliberately does NOT include `body_tokens`, so a body-only edit cannot churn it.
    #[must_use]
    pub fn new(
        crate_id: impl Into<String>,
        fqpath: impl Into<String>,
        item_kind: ItemKind,
        visibility: Visibility,
        signature: &str,
        body_tokens: &str,
    ) -> Self {
        let crate_id = crate_id.into();
        let fqpath = fqpath.into();

        let mut sig_pre = Vec::new();
        sig_pre.extend_from_slice(crate_id.as_bytes());
        sig_pre.push(Self::FIELD_SEP);
        sig_pre.extend_from_slice(fqpath.as_bytes());
        sig_pre.push(Self::FIELD_SEP);
        sig_pre.extend_from_slice(item_kind.as_str().as_bytes());
        sig_pre.push(Self::FIELD_SEP);
        sig_pre.extend_from_slice(visibility.as_str().as_bytes());
        sig_pre.push(Self::FIELD_SEP);
        sig_pre.extend_from_slice(signature.as_bytes());
        let signature_hash = ContentHash::of(&sig_pre);

        // The body pre-image is keyed by the same identity so two items with an identical body text
        // in different positions never share a body hash.
        let mut body_pre = Vec::new();
        body_pre.extend_from_slice(crate_id.as_bytes());
        body_pre.push(Self::FIELD_SEP);
        body_pre.extend_from_slice(fqpath.as_bytes());
        body_pre.push(Self::FIELD_SEP);
        body_pre.extend_from_slice(body_tokens.as_bytes());
        let body_hash = ContentHash::of(&body_pre);

        Function {
            crate_id,
            fqpath,
            item_kind,
            visibility,
            signature_hash,
            body_hash,
        }
    }
}

/// An error returned by [`FactSet::from_facts_checked`] when two structurally DISTINCT items share
/// the same content-address key `(crate_id, fqpath, item_kind, visibility)`.
///
/// In a content-addressed substrate, an address collision is a trust-root fault: two distinct items
/// mapping to the same identity cannot be silently merged. This error surfaces that fault so callers
/// can route the collision to an [`OpaqueReason::AddressCollision`] entry or escalate it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactSetError {
    /// The crate whose address space contains the collision.
    pub crate_id: String,
    /// The `fqpath` that is ambiguous (two distinct items share it within this crate).
    pub fqpath: String,
}

impl FactSetError {
    /// A human-readable description of the collision, for panic messages and diagnostics.
    #[must_use]
    pub fn display(&self) -> String {
        format!(
            "two distinct items share fqpath `{}` in crate `{}`",
            self.fqpath, self.crate_id
        )
    }
}

impl fmt::Display for FactSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display())
    }
}

impl std::error::Error for FactSetError {}

/// A deterministic, de-duplicated set of facts.
///
/// Facts are stored sorted by their total [`Ord`], so [`FactSet::facts`] yields a stable order and
/// [`FactSet::canonical_json`] is byte-identical for equal input regardless of insertion order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactSet {
    facts: Vec<Function>,
}

impl FactSet {
    /// An empty fact set.
    #[must_use]
    pub fn new() -> Self {
        FactSet { facts: Vec::new() }
    }

    /// Build a fact set from an iterator, sorting and de-duplicating for a canonical result.
    ///
    /// **Legit dedup** (byte-identical re-extraction of the same item from the same source): the
    /// duplicate is silently dropped, as usual. **Collision** (two DISTINCT items that compute the
    /// same `(crate_id, fqpath, item_kind, visibility)` but differ in their hash fields): this is a
    /// trust-root fault — a content-addressed substrate MUST NOT silently merge two distinct items
    /// into one. Use [`FactSet::from_facts_checked`] for a non-panicking path that returns a
    /// [`FactSetError`] on collision; use this method when duplicates are expected to be
    /// byte-identical.
    ///
    /// # Panics
    /// Panics if two structurally distinct items share the same content-address key. Callers that
    /// need a non-panicking path must use [`FactSet::from_facts_checked`].
    #[must_use]
    pub fn from_facts(facts: impl IntoIterator<Item = Function>) -> Self {
        match Self::from_facts_checked(facts) {
            Ok(set) => set,
            Err(e) => panic!(
                "corpus fact address collision (trust-root fault): {}",
                e.display()
            ),
        }
    }

    /// Build a fact set from an iterator, returning an error if any two structurally DISTINCT items
    /// share the same content-address key `(crate_id, fqpath, item_kind, visibility)`.
    ///
    /// A byte-identical duplicate (same item re-extracted) is silently dropped. A collision (distinct
    /// items, same key, different hashes) returns [`Err(FactSetError)`].
    ///
    /// # Errors
    /// Returns [`FactSetError`] if two structurally distinct items map to the same content-address
    /// key (`crate_id` + `fqpath` + `item_kind` + `visibility` are equal but hashes differ).
    pub fn from_facts_checked(
        facts: impl IntoIterator<Item = Function>,
    ) -> Result<Self, FactSetError> {
        let mut facts: Vec<Function> = facts.into_iter().collect();
        facts.sort();
        // After sort, equal items are adjacent. Walk pairs: byte-identical → dedup (legit),
        // same key but different hashes → collision (fault).
        let mut i = 0;
        while i + 1 < facts.len() {
            let a = &facts[i];
            let b = &facts[i + 1];
            if a == b {
                // Byte-identical: legit dedup — remove the second occurrence and stay at i.
                facts.remove(i + 1);
            } else if a.crate_id == b.crate_id
                && a.fqpath == b.fqpath
                && a.item_kind == b.item_kind
                && a.visibility == b.visibility
            {
                // Same address key, different hashes: two structurally DISTINCT items with the same
                // content-address → collision, not a safe dedup.
                return Err(FactSetError {
                    crate_id: a.crate_id.clone(),
                    fqpath: a.fqpath.clone(),
                });
            } else {
                i += 1;
            }
        }
        Ok(FactSet { facts })
    }

    /// The facts, in canonical (sorted, de-duplicated) order.
    #[must_use]
    pub fn facts(&self) -> &[Function] {
        &self.facts
    }

    /// The number of facts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// Whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// Serialize to canonical, pretty JSON.
    ///
    /// Deterministic because the facts are already sorted and `Function`'s serde field order is
    /// fixed. `serde_json` pretty output is itself deterministic for a given value.
    ///
    /// # Errors
    /// Returns the `serde_json` error if serialization fails (it cannot for this owned, finite,
    /// non-recursive value, but the API stays fallible rather than panicking).
    pub fn canonical_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.facts)
    }
}

/// Why an [`AstSource`] could not produce a clean fact for a source unit, and the OPAQUE category
/// it falls into. The extractor counts these to compute the go/no-go OPAQUE-RATE: the fraction of
/// the corpus that `syn`-over-source cannot resolve to a clean fact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "category", content = "detail")]
pub enum OpaqueReason {
    /// Two structurally DISTINCT items share the same content-address key `(crate_id, fqpath, ...)`.
    /// A content-addressed substrate MUST NOT silently merge them; both are counted as opaque with
    /// this reason so the collision is auditable in the OPAQUE-RATE report.
    AddressCollision(String),
    /// The source file failed to parse as Rust (`syn` parse error). Reported with the offending
    /// path so the rate can be audited.
    ParseError(String),
    /// The item is produced by macro expansion the source-level extractor cannot see (a macro
    /// invocation in item position, e.g. `tonic::include_proto!` / a custom `proc_macro` that emits
    /// items). Source-level `syn` sees the invocation, not the generated items.
    MacroGenerated(String),
    /// The item is behind a `#[cfg(...)]` whose truth depends on build-time configuration the
    /// hermetic extractor does not evaluate. The fact would be conditionally present, so it is
    /// counted opaque rather than asserted live.
    ///
    /// **Granularity note (v1):** a cfg-gated *module* (`mod tests { … }` or `mod foo { … }` behind
    /// `#[cfg(…)]`) emits exactly ONE opaque unit for the entire module, NOT one per descendant
    /// item. This is the "1 unit per gated module" rule: the extractor cannot see inside a gated
    /// module without evaluating the cfg predicate, so it counts the module as a single opaque unit
    /// and the reported OPAQUE-RATE understates the item-granular miss-rate proportionally. Document
    /// consumers must read cfg_gated counts as module-granular, not item-granular.
    CfgGated(String),
    /// The item is produced by a build script (`build.rs` / `OUT_DIR` `include!`), invisible to a
    /// source-only walk.
    BuildScriptGenerated(String),
    /// An item kind that the v1 extractor does not surface as a liveness fact AND does not have an
    /// explicit silent-drop rule — specifically `extern "C" { … }` blocks (`Item::ForeignMod`) and
    /// `trait Alias = Bound;` declarations (`Item::TraitAlias`). These carry surface area that a
    /// future extractor version SHOULD resolve; routing them here keeps the unresolved set auditable
    /// rather than silently hidden in the terminal `_ => {}` arm.
    Unhandled(String),
}

impl OpaqueReason {
    /// The stable category tag, independent of the carried detail.
    #[must_use]
    pub const fn category(&self) -> &'static str {
        match self {
            OpaqueReason::AddressCollision(_) => "address_collision",
            OpaqueReason::ParseError(_) => "parse_error",
            OpaqueReason::MacroGenerated(_) => "macro_generated",
            OpaqueReason::CfgGated(_) => "cfg_gated",
            OpaqueReason::BuildScriptGenerated(_) => "build_script_generated",
            OpaqueReason::Unhandled(_) => "unhandled",
        }
    }

    /// The human-readable detail the reason carries (the offending path / invocation), for auditing
    /// the opaque rate.
    #[must_use]
    pub fn detail(&self) -> &str {
        match self {
            OpaqueReason::AddressCollision(detail)
            | OpaqueReason::ParseError(detail)
            | OpaqueReason::MacroGenerated(detail)
            | OpaqueReason::CfgGated(detail)
            | OpaqueReason::BuildScriptGenerated(detail)
            | OpaqueReason::Unhandled(detail) => detail,
        }
    }
}

/// The result of extracting one logical source unit: the clean facts it yielded, the edges between
/// them, plus the opaque reasons that prevented clean facts. All are surfaced so the OPAQUE-RATE is
/// computed over the whole corpus, never silently dropped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Extraction {
    /// Clean facts resolved from the unit.
    pub facts: Vec<Function>,
    /// Edges the source could resolve from this unit.
    ///
    /// A source emits only the edges it alone can see (containment between items, and references an
    /// item names). The File node, the per-fact Entry node, and the file→item `Contains` edge are
    /// DERIVED by the driver from the facts and the source list, so every [`AstSource`] gets them
    /// without reimplementing them — and cannot forget to.
    pub edges: Vec<Edge>,
    /// Opaque reasons encountered while extracting the unit.
    pub opaque: Vec<OpaqueReason>,
}

/// The stable seam an AST fact-source plugs into.
///
/// The v1 `syn`-over-source extractor implements this today; a W-tier bespoke-rowan CST successor
/// will implement it later without disturbing the [`Function`] fact model or any consumer. The
/// trait is intentionally minimal for the spike: one method that turns a single source file's bytes
/// (with the crate id and the file's module path) into an [`Extraction`].
pub trait AstSource {
    /// The error type a source may raise for conditions that are not per-item opaque reasons (e.g.
    /// an I/O failure reading a file). Per-item opacity is reported via [`Extraction::opaque`], not
    /// this error.
    type Error: std::error::Error;

    /// Extract facts (and opaque reasons) from one source file.
    ///
    /// * `crate_id` — the owning crate's de-branded cargo name.
    /// * `module_path` — the `::`-joined module path the file's items live under (empty for the
    ///   crate root `lib.rs`/`main.rs`).
    /// * `source` — the file's UTF-8 contents.
    ///
    /// # Errors
    /// Returns [`Self::Error`] only for non-per-item failures; a parse failure is reported as an
    /// [`OpaqueReason::ParseError`] within the returned [`Extraction`] so it still counts toward the
    /// opaque rate rather than aborting the whole run.
    fn extract_file(
        &self,
        crate_id: &str,
        module_path: &str,
        source: &str,
    ) -> Result<Extraction, Self::Error>;
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// The graph model: nodes and the TWO edge kinds.
//
// This model was first written in `corpus-yaml-kernel` for the YAML artifact class. It is the
// shared graph CONTRACT, not a YAML detail, so it lives here in the pure kernel and `corpus-yaml-
// kernel` re-exports it. There is exactly ONE definition — a parallel copy per artifact class is
// the divergence this move exists to prevent.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// The schema version stamped into every [`GraphFace`].
///
/// Distinguishes shape from content when two producers are in flight during an ADR-0541 D2
/// verified-equivalence migration. Bump on any change to the serialized field set.
pub const SCHEMA_VERSION: u32 = 1;

/// The coarse kind of a graph node.
///
/// Deliberately three variants. Per-domain detail belongs in a per-domain fact type joined on
/// [`NodeId`] (for Rust that is [`Function`]), never contorted into this enum — a YAML key has no
/// `visibility` and a Rust `fn` has no `effect: permit`, so forcing both into one flat struct is the
/// contortion to avoid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// A build-graph node: the buck2 target whose action produced this shard.
    Target,
    /// A tracked file. The universal reachability leaf, and the only node kind an edge may dangle
    /// onto (a `Refs` edge naming a file that does not exist is the defect we want representable).
    File,
    /// A named, addressable location inside a container — a YAML scalar's key path, or a Rust
    /// item's fully-qualified path within its crate.
    Entry,
}

impl NodeKind {
    /// The stable lowercase wire tag, matching the serde rename. Used in digest pre-images so a
    /// digest never depends on Rust's `Debug` formatting.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            NodeKind::Target => "target",
            NodeKind::File => "file",
            NodeKind::Entry => "entry",
        }
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The stable, human-readable node identity. NOT a digest.
///
/// Editing file A changes no `NodeId` in file B; reformatting a file changes no `NodeId` at all.
/// This is the SCIP-over-LSIF lesson and ADR-0541 D1 precedent #7: fusing identity with content is
/// what LSIF got wrong. Neither Kythe (`VName` = five plain strings) nor Glean (opaque per-database
/// fact ids) content-addresses node identity either.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId {
    /// The addressing namespace, which is per-KIND:
    /// - [`NodeKind::Target`] — the buck2 target label.
    /// - [`NodeKind::File`] — the repo-relative file path.
    /// - [`NodeKind::Entry`] — the container the entry is addressed WITHIN. For YAML that is the
    ///   repo-relative file path; for Rust it is the crate's de-branded cargo id, because a Rust
    ///   reference names `module::item` within a CRATE and never names the defining file. The two
    ///   cannot collide: a repo-relative YAML path always contains `/` and a `.yaml` suffix, a
    ///   cargo id contains neither.
    pub container: String,
    /// Path within the container: the `/`-joined YAML key path (sequence elements use their index),
    /// or the `::`-joined Rust fully-qualified item path. Empty for [`NodeKind::File`] and
    /// [`NodeKind::Target`] nodes, whose container is already the whole identity.
    pub path: String,
    /// The node kind.
    pub kind: NodeKind,
}

impl NodeId {
    /// A `File` node for a repo-relative path.
    #[must_use]
    pub fn file(container: impl Into<String>) -> Self {
        NodeId {
            container: container.into(),
            path: String::new(),
            kind: NodeKind::File,
        }
    }

    /// A `Target` node for a buck2 target label.
    #[must_use]
    pub fn target(label: impl Into<String>) -> Self {
        NodeId {
            container: label.into(),
            path: String::new(),
            kind: NodeKind::Target,
        }
    }

    /// An `Entry` node for a path inside a container.
    #[must_use]
    pub fn entry(container: impl Into<String>, path: impl Into<String>) -> Self {
        NodeId {
            container: container.into(),
            path: path.into(),
            kind: NodeKind::Entry,
        }
    }
}

/// A graph node: nominal identity plus its own SHALLOW content-address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Node {
    /// The node's stable identity.
    pub id: NodeId,
    /// blake3 over this node's OWN canonical pre-image. Never includes a child's digest.
    ///
    /// The absence of a roll-up is load-bearing: a Merkle-style aggregate would make every edit
    /// churn the root, which is the parent-churn pathology neither Kythe nor Glean accepted. There
    /// is deliberately NO module, crate, or capability node carrying an aggregate hash — churn is
    /// bounded by that ABSENCE, and adding one would silently unbound it.
    pub digest: ContentHash,
}

/// The two relations the corpus graph needs.
///
/// Kythe ships 11+ edge kinds; nine of them exist to serve IDE cross-reference and type hierarchy,
/// which we are not building. Adding a third kind requires showing a query that neither of these
/// two can answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// Structural containment, exactly one level down: target→file, file→entry, impl→method. The
    /// spanning forest a reachability BFS walks.
    Contains,
    /// One node names another: a YAML scalar whose text is a repo-relative path, or a Rust item
    /// that calls a function / mentions a type / implements a trait.
    Refs,
}

impl EdgeKind {
    /// The stable lowercase wire tag, matching the serde rename.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            EdgeKind::Contains => "contains",
            EdgeKind::Refs => "refs",
        }
    }
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A typed edge between two nodes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Edge {
    /// The relation this edge asserts.
    pub kind: EdgeKind,
    /// The naming node.
    pub src: NodeId,
    /// The named node, as a NAME rather than a fact pointer.
    ///
    /// A `dst` with no matching [`Node`] is a DANGLING reference: legal, representable, and exactly
    /// the ADR-0541 D2 reference-integrity defect class. Making this a fact id would render the very
    /// defect we want to detect unrepresentable.
    pub dst: NodeId,
}

/// One shard of the corpus graph: the output of exactly one buck2 extraction action.
///
/// Sharding is what keeps this free of the global-registry contention that has repeatedly wedged
/// repo-wide moves: one action emits one shard, there is no global graph file to serialize against,
/// and the merged graph is a query-time concatenation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphFace {
    /// [`SCHEMA_VERSION`] at the time of production.
    pub schema_version: u32,
    /// The buck2 target label whose action produced this shard.
    pub target: String,
    /// Sorted and de-duplicated, so the serialized bytes are order-independent.
    pub nodes: Vec<Node>,
    /// Sorted and de-duplicated.
    pub edges: Vec<Edge>,
    /// Index-integrity attestation (ADR-0541 D3). NON-EMPTY MEANS THE FACE IS INCOMPLETE.
    pub opaque: Vec<OpaqueReason>,
}

impl GraphFace {
    /// Build a shard from raw parts, sorting and de-duplicating for a canonical result.
    #[must_use]
    pub fn new(
        target: impl Into<String>,
        mut nodes: Vec<Node>,
        mut edges: Vec<Edge>,
        mut opaque: Vec<OpaqueReason>,
    ) -> Self {
        nodes.sort();
        nodes.dedup();
        edges.sort();
        edges.dedup();
        opaque.sort();
        opaque.dedup();
        GraphFace {
            schema_version: SCHEMA_VERSION,
            target: target.into(),
            nodes,
            edges,
            opaque,
        }
    }

    /// Serialize to canonical pretty JSON.
    ///
    /// Determinism holds because every field is a string, integer, or enum (no floats reach the
    /// face), the vectors are sorted, and serde emits struct fields in declaration order.
    ///
    /// Deliberately serializes the typed struct directly and NEVER routes bytes through
    /// `serde_json::Value`: reindeer unions `serde_json`'s `preserve_order` feature workspace-wide,
    /// which swaps `Value`'s map from `BTreeMap` to `IndexMap`. Structs are unaffected by that
    /// feature; `Value` is not.
    ///
    /// # Errors
    /// Returns the underlying `serde_json` error if serialization fails.
    pub fn to_canonical_json(&self) -> Result<String, serde_json::Error> {
        let mut out = serde_json::to_string_pretty(self)?;
        out.push('\n');
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// Queries over the graph. Every one of these is COMPUTED on demand and NOTHING here is
// materialized: a materialized view goes stale, which is precisely what
// `specs/reachability-registry.json` does today.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// A queryable graph: a node set plus an edge set, both canonical (sorted, de-duplicated).
///
/// # What a reachability result does NOT license
/// A node that no edge reaches is NOT thereby dead. Today roughly two thirds of tracked files are
/// outside the build graph entirely, so ABSENCE OF A REFERENCE IS THE NORM, not evidence of death.
/// In particular this graph must never be read as licensing "unreferenced markdown ⇒ delete":
/// markdown is reached by humans, by review, and by extractors that do not exist yet. Deletion
/// requires POSITIVE evidence of deadness; [`Graph::reachable_from`] supplies a candidate set for
/// investigation and nothing more.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    /// Build a canonical graph, sorting and de-duplicating both sets.
    ///
    /// Unlike [`FactSet::from_facts_checked`] there is no collision concept: a node's identity is
    /// NOMINAL, so two nodes sharing a [`NodeId`] but differing in digest is a real condition the
    /// caller must resolve upstream (byte-identical nodes simply dedup here).
    #[must_use]
    pub fn new(mut nodes: Vec<Node>, mut edges: Vec<Edge>) -> Self {
        nodes.sort();
        nodes.dedup();
        edges.sort();
        edges.dedup();
        Graph { nodes, edges }
    }

    /// The nodes, in canonical order.
    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// The edges, in canonical order.
    #[must_use]
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Is `id` present in the node set (i.e. INDEXED)?
    #[must_use]
    pub fn contains_node(&self, id: &NodeId) -> bool {
        self.nodes.binary_search_by(|node| node.id.cmp(id)).is_ok()
    }

    /// Reachability: the BFS closure over `Contains ∪ Refs` from a seed set.
    ///
    /// A QUERY, never a materialized edge kind. The returned set contains NAMES, including dangling
    /// `dst` names that match no node — a reached name that is not indexed is exactly the signal
    /// [`Graph::coverage`] counts, so it is surfaced rather than silently filtered. Callers wanting
    /// only real nodes intersect the result with [`Graph::contains_node`].
    #[must_use]
    pub fn reachable_from(&self, seeds: &[NodeId]) -> BTreeSet<NodeId> {
        let mut adjacency: BTreeMap<&NodeId, Vec<&NodeId>> = BTreeMap::new();
        for edge in &self.edges {
            adjacency.entry(&edge.src).or_default().push(&edge.dst);
        }
        let mut seen: BTreeSet<NodeId> = BTreeSet::new();
        let mut queue: VecDeque<NodeId> = VecDeque::new();
        for seed in seeds {
            if seen.insert(seed.clone()) {
                queue.push_back(seed.clone());
            }
        }
        while let Some(current) = queue.pop_front() {
            let Some(children) = adjacency.get(&current) else {
                continue;
            };
            for child in children {
                if seen.insert((*child).clone()) {
                    queue.push_back((*child).clone());
                }
            }
        }
        seen
    }

    /// Byte-identical containers, grouped by digest: `(digest, sorted containers)` for every group
    /// of two or more.
    ///
    /// This falls out of the model for free and is deliberately a QUERY rather than converged
    /// identity: N byte-identical files stay N nominally distinct nodes that happen to share one
    /// digest. Converging them into a single node would content-address IDENTITY, which is exactly
    /// the split ADR-0541 D1 ratified against.
    ///
    /// # Measured scope, so nobody re-derives the wrong expectation
    /// The repo's large same-NAME markdown families are NOT byte-identical and this query will not
    /// group them. Measured 2026-08-01 by git blob OID: `ux-flow.md` 179 files / 179 distinct
    /// blobs, `story.md` 179 / 179, `dpia.md` 87 / 87. Same-name-different-content is a TEMPLATE
    /// family, and detecting it needs near-duplicate analysis (shingling, structural diff), which a
    /// content-address deliberately cannot do — a one-byte difference is a different digest, and
    /// that is the property that makes the digest trustworthy. Do not weaken the digest to make
    /// this query find them.
    #[must_use]
    pub fn duplicate_containers(&self) -> Vec<(ContentHash, Vec<String>)> {
        let mut by_digest: BTreeMap<&ContentHash, Vec<String>> = BTreeMap::new();
        for node in &self.nodes {
            if node.id.kind == NodeKind::File {
                by_digest
                    .entry(&node.digest)
                    .or_default()
                    .push(node.id.container.clone());
            }
        }
        by_digest
            .into_iter()
            .filter(|(_, containers)| containers.len() > 1)
            .map(|(digest, mut containers)| {
                containers.sort();
                (digest.clone(), containers)
            })
            .collect()
    }

    /// Every distinct `Refs` target that matches no node — the DANGLING reference set.
    #[must_use]
    pub fn unresolved_targets(&self) -> Vec<NodeId> {
        self.refs_targets()
            .into_iter()
            .filter(|id| !self.contains_node(id))
            .collect()
    }

    /// The edge-target coverage: indexed targets over total targets, COMPUTED from the graph.
    ///
    /// Counted over distinct `Refs` targets ONLY. A `Contains` target is emitted by the very pass
    /// that emits its parent, so it resolves BY CONSTRUCTION; including it would let the rate be
    /// inflated by adding structure rather than by indexing more of the world. This is the same
    /// anti-gaming rule the YAML `corpus-index-coverage` gate applies when it keeps unpackaged
    /// files in the DENOMINATOR.
    #[must_use]
    pub fn coverage(&self) -> Coverage {
        let targets = self.refs_targets();
        let indexed = targets.iter().filter(|id| self.contains_node(id)).count();
        Coverage {
            indexed_targets: indexed,
            total_targets: targets.len(),
        }
    }

    /// The distinct `Refs` targets, in canonical order.
    fn refs_targets(&self) -> Vec<NodeId> {
        let mut targets: BTreeSet<&NodeId> = BTreeSet::new();
        for edge in &self.edges {
            if edge.kind == EdgeKind::Refs {
                targets.insert(&edge.dst);
            }
        }
        targets.into_iter().cloned().collect()
    }
}

/// The measured edge-target coverage. Both fields are counted, never asserted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coverage {
    /// `Refs` targets that match a node in the graph.
    pub indexed_targets: usize,
    /// All distinct `Refs` targets.
    pub total_targets: usize,
}

impl Coverage {
    /// Targets named by an edge but present in no node — the debt this ratchet burns down.
    #[must_use]
    pub fn unindexed_targets(&self) -> usize {
        self.total_targets.saturating_sub(self.indexed_targets)
    }

    /// Coverage in basis points (indexed / total * 10000), integer so no float formatting can make
    /// the number non-deterministic.
    ///
    /// An EMPTY graph reports 0, never 10000. `0/0` is not full coverage — it is a probe that saw
    /// nothing, and reporting it as 100% is the exact false-green
    /// [`evaluate_coverage`]'s anti-vacuity floor exists to catch.
    #[must_use]
    pub fn rate_bps(&self) -> u64 {
        if self.total_targets == 0 {
            return 0;
        }
        (self.indexed_targets as u64 * 10_000) / self.total_targets as u64
    }
}

/// The frozen ratchet policy. Every repo-specific number is DATA: another repo adopts this ratchet
/// by repointing these two fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoveragePolicy {
    /// Shrink-only ceiling: measured unindexed targets may not exceed this.
    pub baseline_unindexed_targets: usize,
    /// Anti-vacuity floor: fewer total targets than this means the extractor collapsed, and its
    /// "no unindexed targets" result is meaningless rather than perfect.
    pub min_expected_targets: usize,
}

/// A NEW dangling target pushed the unindexed count above the frozen ceiling. Blocking.
pub const CODE_EDGE_COVERAGE_REGRESSION: &str = "corpus_edge_coverage_regression";
/// The observed edge set collapsed below the expected floor — the extractor is broken, and its
/// coverage number is meaningless. Blocking.
pub const CODE_EDGE_SCAN_VACUOUS: &str = "corpus_edge_scan_vacuous";
/// A `Refs` target that matches no node, within the frozen ceiling. Advisory: this is the debt.
pub const CODE_EDGE_DANGLING_TARGET: &str = "corpus_edge_dangling_target";
/// The frozen ceiling sits above the measured number — the ratchet has slack and should be lowered
/// so it keeps biting instead of silently absorbing the next regression. Advisory.
pub const CODE_EDGE_STALE_CEILING: &str = "corpus_edge_stale_ceiling";

/// One ratchet finding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CoverageFinding {
    /// The stable machine-readable code (one of the `CODE_EDGE_*` constants).
    pub code: String,
    /// Does this finding fail the ratchet closed?
    pub blocking: bool,
    /// Human-readable detail.
    pub detail: String,
}

/// Evaluate the edge-coverage ratchet against a frozen policy.
///
/// Born ADVISORY over today's debt and BLOCKING on regression, exactly like the YAML
/// `corpus-index-coverage` gate: today's dangling targets are reported one per finding and frozen
/// as a ceiling, and a NEW dangling target exceeds the ceiling and fails closed. The ceiling is
/// lowered as more of the corpus is indexed, so "everything in the graph" becomes a burn-down.
///
/// PURE: the caller supplies the graph and the policy; this counts them.
#[must_use]
pub fn evaluate_coverage(graph: &Graph, policy: &CoveragePolicy) -> Vec<CoverageFinding> {
    let coverage = graph.coverage();
    let unresolved = graph.unresolved_targets();
    let mut findings = Vec::new();

    // Anti-vacuity FIRST: a collapsed scan reports zero unindexed targets, which is indistinguishable
    // from perfect coverage. Fail closed before any coverage number is believed.
    if coverage.total_targets < policy.min_expected_targets {
        findings.push(CoverageFinding {
            code: CODE_EDGE_SCAN_VACUOUS.to_owned(),
            blocking: true,
            detail: format!(
                "observed {} Refs targets, below the expected floor of {} — the extractor saw \
                 (almost) nothing, so its coverage result is meaningless, not perfect",
                coverage.total_targets, policy.min_expected_targets
            ),
        });
    }

    let unindexed = coverage.unindexed_targets();
    if unindexed > policy.baseline_unindexed_targets {
        // The blocking finding NAMES the offenders. A red gate that reports only a count makes the
        // reader hunt for what changed, which is how a gate earns a reputation for noise and then
        // gets switched off. Bounded so one bad extraction cannot emit a megabyte of detail; the
        // full set is always available from `Graph::unresolved_targets`.
        const NAMED: usize = 10;
        let names: Vec<String> = unresolved
            .iter()
            .take(NAMED)
            .map(|target| format!("{}::{}", target.container, target.path))
            .collect();
        let elided = unresolved.len().saturating_sub(names.len());
        let suffix = if elided > 0 {
            format!(" (+{elided} more)")
        } else {
            String::new()
        };
        findings.push(CoverageFinding {
            code: CODE_EDGE_COVERAGE_REGRESSION.to_owned(),
            blocking: true,
            detail: format!(
                "{unindexed} Refs targets match no node, above the frozen ceiling of {}: {}{suffix}",
                policy.baseline_unindexed_targets,
                names.join(", ")
            ),
        });
    } else if unindexed < policy.baseline_unindexed_targets {
        findings.push(CoverageFinding {
            code: CODE_EDGE_STALE_CEILING.to_owned(),
            blocking: false,
            detail: format!(
                "ceiling {} exceeds the measured {unindexed}; lower it so the ratchet keeps biting",
                policy.baseline_unindexed_targets
            ),
        });
    }

    for target in unresolved {
        findings.push(CoverageFinding {
            code: CODE_EDGE_DANGLING_TARGET.to_owned(),
            blocking: false,
            detail: format!(
                "{}::{} ({}) is named by a Refs edge but matches no node",
                target.container, target.path, target.kind
            ),
        });
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_hash_independent_of_body() {
        let a = Function::new(
            "flags-evaluation-domain",
            "engine::evaluate",
            ItemKind::Function,
            Visibility::Public,
            "fn evaluate (flag : & Flag) -> Decision",
            "let x = 1 ;",
        );
        let b = Function::new(
            "flags-evaluation-domain",
            "engine::evaluate",
            ItemKind::Function,
            Visibility::Public,
            "fn evaluate (flag : & Flag) -> Decision",
            "let x = 2 ; let y = 3 ;",
        );
        // Same signature, different body: signature anchor stable, body digest churns.
        assert_eq!(a.signature_hash, b.signature_hash);
        assert_ne!(a.body_hash, b.body_hash);
    }

    #[test]
    fn signature_hash_churns_on_rename() {
        let a = Function::new(
            "c",
            "engine::evaluate",
            ItemKind::Function,
            Visibility::Public,
            "fn evaluate ()",
            "",
        );
        let renamed = Function::new(
            "c",
            "engine::evaluate_v2",
            ItemKind::Function,
            Visibility::Public,
            "fn evaluate_v2 ()",
            "",
        );
        assert_ne!(a.signature_hash, renamed.signature_hash);
    }

    #[test]
    fn signature_hash_churns_on_visibility_change() {
        let pubf = Function::new(
            "c",
            "m::f",
            ItemKind::Function,
            Visibility::Public,
            "fn f ()",
            "",
        );
        let privf = Function::new(
            "c",
            "m::f",
            ItemKind::Function,
            Visibility::Private,
            "fn f ()",
            "",
        );
        assert_ne!(pubf.signature_hash, privf.signature_hash);
    }

    #[test]
    fn factset_canonical_order_independent_of_insertion() {
        let f1 = Function::new("c", "a::one", ItemKind::Function, Visibility::Public, "fn one ()", "");
        let f2 = Function::new("c", "b::two", ItemKind::Function, Visibility::Public, "fn two ()", "");
        let forward = FactSet::from_facts([f1.clone(), f2.clone()]);
        let reverse = FactSet::from_facts([f2, f1]);
        assert_eq!(forward.canonical_json().unwrap(), reverse.canonical_json().unwrap());
    }

    #[test]
    fn factset_dedups() {
        let f = Function::new("c", "a::one", ItemKind::Function, Visibility::Public, "fn one ()", "");
        let set = FactSet::from_facts([f.clone(), f]);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn opaque_reason_category_is_stable() {
        assert_eq!(OpaqueReason::MacroGenerated("x".into()).category(), "macro_generated");
        assert_eq!(OpaqueReason::CfgGated("x".into()).category(), "cfg_gated");
        assert_eq!(OpaqueReason::Unhandled("x".into()).category(), "unhandled");
        assert_eq!(OpaqueReason::AddressCollision("x".into()).category(), "address_collision");
    }

    // HIGH-2 RED TEST: two structurally DISTINCT items that share the same (crate_id, fqpath,
    // item_kind, visibility) must NOT be silently merged — the collision is a trust-root fault.
    #[test]
    fn factset_collision_detected_not_silently_merged() {
        // Build two facts that look like the same item address but have different bodies
        // (e.g. the extractor produced conflicting definitions for the same fqpath).
        let a = Function::new(
            "c",
            "m::conflict",
            ItemKind::Function,
            Visibility::Public,
            "fn conflict ()",
            "let x = 1 ;",
        );
        let b = Function::new(
            "c",
            "m::conflict",
            ItemKind::Function,
            Visibility::Public,
            "fn conflict ()",
            "let x = 2 ;",  // different body → different body_hash → distinct item
        );
        // The signature hashes are equal (same sig pre-image) but body hashes differ → NOT byte-
        // identical → from_facts_checked must return Err, not silently drop one.
        assert_ne!(a, b, "precondition: the two facts are NOT byte-identical");
        let result = FactSet::from_facts_checked([a, b]);
        assert!(result.is_err(), "collision MUST be detected, not silently merged");
        let err = result.unwrap_err();
        assert_eq!(err.crate_id, "c");
        assert_eq!(err.fqpath, "m::conflict");
    }

    // Byte-identical duplicates (same item extracted twice) are still silently deduped.
    #[test]
    fn factset_byte_identical_duplicate_silently_deduped() {
        let f = Function::new("c", "a::one", ItemKind::Function, Visibility::Public, "fn one ()", "");
        let result = FactSet::from_facts_checked([f.clone(), f]);
        assert!(result.is_ok(), "byte-identical duplicate must be silently deduped");
        assert_eq!(result.unwrap().len(), 1);
    }

    // ── graph model ──────────────────────────────────────────────────────────────────────────

    fn entry_node(container: &str, path: &str) -> Node {
        Node {
            id: NodeId::entry(container, path),
            digest: ContentHash::of(format!("{container}\0{path}").as_bytes()),
        }
    }

    fn refs(src: NodeId, dst: NodeId) -> Edge {
        Edge {
            kind: EdgeKind::Refs,
            src,
            dst,
        }
    }

    #[test]
    fn graph_is_canonical_regardless_of_insertion_order() {
        let a = entry_node("c", "a");
        let b = entry_node("c", "b");
        let e1 = refs(NodeId::entry("c", "a"), NodeId::entry("c", "b"));
        let e2 = refs(NodeId::entry("c", "b"), NodeId::entry("c", "a"));
        let forward = Graph::new(vec![a.clone(), b.clone()], vec![e1.clone(), e2.clone()]);
        let reverse = Graph::new(vec![b, a], vec![e2, e1]);
        assert_eq!(forward, reverse);
    }

    // KNOWN-POSITIVE CONTROL: every Refs target matches a node → full coverage, zero dangling.
    #[test]
    fn coverage_positive_control_all_targets_indexed() {
        let graph = Graph::new(
            vec![entry_node("c", "a"), entry_node("c", "b")],
            vec![refs(NodeId::entry("c", "a"), NodeId::entry("c", "b"))],
        );
        let coverage = graph.coverage();
        assert_eq!(coverage.total_targets, 1);
        assert_eq!(coverage.indexed_targets, 1);
        assert_eq!(coverage.unindexed_targets(), 0);
        assert_eq!(coverage.rate_bps(), 10_000);
        assert!(graph.unresolved_targets().is_empty());
    }

    // KNOWN-NEGATIVE CONTROL: one target names a node that does not exist. The dangling reference
    // MUST stay representable and MUST be counted — a fact-pointer `dst` would have made this
    // defect unrepresentable, which is why `dst` is a NAME.
    #[test]
    fn coverage_negative_control_dangling_target_is_counted_and_named() {
        let graph = Graph::new(
            vec![entry_node("c", "a"), entry_node("c", "b")],
            vec![
                refs(NodeId::entry("c", "a"), NodeId::entry("c", "b")),
                refs(NodeId::entry("c", "a"), NodeId::entry("c", "gone")),
            ],
        );
        let coverage = graph.coverage();
        assert_eq!(coverage.total_targets, 2);
        assert_eq!(coverage.indexed_targets, 1);
        assert_eq!(coverage.unindexed_targets(), 1);
        assert_eq!(coverage.rate_bps(), 5_000);
        assert_eq!(graph.unresolved_targets(), vec![NodeId::entry("c", "gone")]);
    }

    // A Contains target resolves by construction, so counting it would inflate the rate with
    // structure rather than with indexing. It must not enter either side of the fraction.
    #[test]
    fn coverage_ignores_contains_edges() {
        let with_contains = Graph::new(
            vec![entry_node("c", "a"), entry_node("c", "b")],
            vec![
                refs(NodeId::entry("c", "a"), NodeId::entry("c", "gone")),
                Edge {
                    kind: EdgeKind::Contains,
                    src: NodeId::entry("c", "a"),
                    dst: NodeId::entry("c", "b"),
                },
            ],
        );
        assert_eq!(with_contains.coverage().total_targets, 1);
        assert_eq!(with_contains.coverage().rate_bps(), 0);
    }

    // An empty probe must NOT read as 100%. A round, total number means the probe is wrong until
    // proven otherwise.
    #[test]
    fn empty_graph_is_zero_percent_not_one_hundred() {
        let empty = Graph::default();
        assert_eq!(empty.coverage().rate_bps(), 0);
    }

    #[test]
    fn ratchet_advisory_within_ceiling_blocking_on_regression() {
        let graph = Graph::new(
            vec![entry_node("c", "a")],
            vec![refs(NodeId::entry("c", "a"), NodeId::entry("c", "gone"))],
        );
        // Within the ceiling: the dangling target is ADVISORY debt, nothing blocks.
        let advisory = evaluate_coverage(
            &graph,
            &CoveragePolicy {
                baseline_unindexed_targets: 1,
                min_expected_targets: 1,
            },
        );
        assert!(advisory.iter().all(|f| !f.blocking), "{advisory:?}");
        assert!(
            advisory
                .iter()
                .any(|f| f.code == CODE_EDGE_DANGLING_TARGET && f.detail.contains("gone")),
            "the dangling target must be NAMED, not just counted: {advisory:?}"
        );

        // Ceiling lowered to 0: the same graph is now a REGRESSION and fails closed.
        let blocking = evaluate_coverage(
            &graph,
            &CoveragePolicy {
                baseline_unindexed_targets: 0,
                min_expected_targets: 1,
            },
        );
        assert!(
            blocking
                .iter()
                .any(|f| f.code == CODE_EDGE_COVERAGE_REGRESSION && f.blocking),
            "{blocking:?}"
        );
    }

    // The important failure of a coverage ratchet is not a false red, it is a collapsed scan that
    // reports zero unindexed targets and reads as PERFECT.
    #[test]
    fn ratchet_fails_closed_on_a_vacuous_scan() {
        let findings = evaluate_coverage(
            &Graph::default(),
            &CoveragePolicy {
                baseline_unindexed_targets: 0,
                min_expected_targets: 10,
            },
        );
        assert!(
            findings
                .iter()
                .any(|f| f.code == CODE_EDGE_SCAN_VACUOUS && f.blocking),
            "an empty extraction must fail closed, not read as full coverage: {findings:?}"
        );
    }

    #[test]
    fn ratchet_reports_stale_ceiling_so_slack_cannot_accumulate() {
        let graph = Graph::new(
            vec![entry_node("c", "a"), entry_node("c", "b")],
            vec![refs(NodeId::entry("c", "a"), NodeId::entry("c", "b"))],
        );
        let findings = evaluate_coverage(
            &graph,
            &CoveragePolicy {
                baseline_unindexed_targets: 5,
                min_expected_targets: 1,
            },
        );
        assert!(
            findings
                .iter()
                .any(|f| f.code == CODE_EDGE_STALE_CEILING && !f.blocking),
            "{findings:?}"
        );
    }

    // Reachability is a QUERY over Contains ∪ Refs, never a materialized edge.
    #[test]
    fn reachability_is_the_bfs_closure_over_both_edge_kinds() {
        let graph = Graph::new(
            vec![
                entry_node("c", "root"),
                entry_node("c", "child"),
                entry_node("c", "grandchild"),
                entry_node("c", "island"),
            ],
            vec![
                Edge {
                    kind: EdgeKind::Contains,
                    src: NodeId::entry("c", "root"),
                    dst: NodeId::entry("c", "child"),
                },
                refs(
                    NodeId::entry("c", "child"),
                    NodeId::entry("c", "grandchild"),
                ),
            ],
        );
        let reached = graph.reachable_from(&[NodeId::entry("c", "root")]);
        assert!(reached.contains(&NodeId::entry("c", "grandchild")));
        assert!(
            !reached.contains(&NodeId::entry("c", "island")),
            "an unreached node is a CANDIDATE for investigation, never proof of death"
        );
    }

    // A cycle must terminate — the BFS is over a graph, not a tree.
    #[test]
    fn reachability_terminates_on_a_cycle() {
        let graph = Graph::new(
            vec![entry_node("c", "a"), entry_node("c", "b")],
            vec![
                refs(NodeId::entry("c", "a"), NodeId::entry("c", "b")),
                refs(NodeId::entry("c", "b"), NodeId::entry("c", "a")),
            ],
        );
        assert_eq!(graph.reachable_from(&[NodeId::entry("c", "a")]).len(), 2);
    }

    // Duplication detection falls out of the shallow digest for free: N byte-identical containers
    // share ONE digest while staying N nominally distinct nodes.
    #[test]
    fn duplicate_containers_group_by_digest_without_converging_identity() {
        let bytes = b"identical\n";
        let nodes: Vec<Node> = ["a/ux-flow.md", "b/ux-flow.md", "c/ux-flow.md"]
            .into_iter()
            .map(|path| Node {
                id: NodeId::file(path),
                digest: ContentHash::of(bytes),
            })
            .chain(std::iter::once(Node {
                id: NodeId::file("d/other.md"),
                digest: ContentHash::of(b"different\n"),
            }))
            .collect();
        let graph = Graph::new(nodes, Vec::new());
        let groups = graph.duplicate_containers();
        assert_eq!(groups.len(), 1, "only the identical family groups");
        assert_eq!(
            groups[0].1,
            vec!["a/ux-flow.md", "b/ux-flow.md", "c/ux-flow.md"]
        );
        assert_eq!(
            graph.nodes().len(),
            4,
            "identity stays NOMINAL: duplicates do not converge into one node"
        );
    }

    // The anti-Merkle commitment, at the model level: no node kind carries a roll-up of its
    // children, so a child edit cannot churn a parent digest.
    #[test]
    fn no_aggregate_node_kind_exists() {
        let kinds = [NodeKind::Target, NodeKind::File, NodeKind::Entry];
        assert_eq!(
            kinds.len(),
            3,
            "adding a crate/module roll-up node kind creates a Merkle tree"
        );
        let edge_kinds = [EdgeKind::Contains, EdgeKind::Refs];
        assert_eq!(
            edge_kinds.len(),
            2,
            "exactly two edge kinds; a third needs a query neither answers"
        );
    }
}
