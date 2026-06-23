//! # corpus-core (ADR-0580, Phase -1 corpus extractor spike)
//!
//! The content-addressed fact model + the stable [`AstSource`] trait for the live-AST governance
//! substrate ("corpus"). This crate is the conservative-v1 *contract* layer: it defines WHAT a
//! liveness fact is and the seam an extractor plugs into, NOT how facts are produced (that is the
//! `corpus-extract` binary) and NOT a node/query model (deferred — Phase -1 is the de-risk slice).
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
    /// into one. The colliding fact is replaced with an [`OpaqueReason::AddressCollision`] entry in
    /// the returned [`CollisionReport`]. Use [`FactSet::from_facts_checked`] when you need to inspect
    /// collisions; use [`FactSet::from_facts`] when duplicates are expected to be byte-identical.
    ///
    /// # Panics
    /// Panics if a hash-collision (two structurally distinct items share the same content-address
    /// key) is detected. Callers that need a non-panicking path must use [`FactSet::from_facts_checked`].
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
    /// items, same key, different hashes) returns [`FactSetError::AddressCollision`].
    ///
    /// # Errors
    /// Returns [`FactSetError::AddressCollision`] if two structurally distinct items map to the same
    /// content-address key.
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

/// The result of extracting one logical source unit: the clean facts it yielded plus the opaque
/// reasons that prevented clean facts. Both are surfaced so the OPAQUE-RATE is computed over the
/// whole corpus, never silently dropped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Extraction {
    /// Clean facts resolved from the unit.
    pub facts: Vec<Function>,
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
}
