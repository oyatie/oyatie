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
//!    [`FactSet`](corpus_core::FactSet) + an [`OpaqueReport`].
//!
//! ## Hermeticity + determinism
//! No clock, no rand, no network, no ambient env. The only inputs are the committed manifest, the
//! caller-supplied source list, and file contents. Output is sorted/canonical, so the same input
//! yields a byte-identical fact set.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use corpus_core::{
    AstSource, Extraction, FactSet, Function, ItemKind, OpaqueReason, Visibility,
};
use oya_workspace_members_kernel::{ResolveError, resolve_member_dirs};
use quote::ToTokens;

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
    /// The `::`-joined module path the file's items live under (empty for `lib.rs`/`main.rs`).
    pub module_path: String,
    /// The file's UTF-8 source.
    pub source: String,
}

/// A canonically-ordered set of source files to extract. Ordering is by `(crate_id, module_path)`
/// so the corpus walk — and thus any incidental ordering effect — is deterministic.
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

/// The full result of extracting a corpus slice: the canonical fact set + the opaque report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CorpusExtraction {
    /// The canonical, sorted, de-duplicated facts.
    pub facts: FactSet,
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

    for file in set.files() {
        let extraction: Extraction =
            source.extract_file(&file.crate_id, &file.module_path, &file.source)?;
        facts.extend(extraction.facts);
        opaque.extend(extraction.opaque);
    }

    opaque.sort();
    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    for reason in &opaque {
        *by_category.entry(reason.category().to_owned()).or_insert(0) += 1;
    }

    let fact_set = FactSet::from_facts(facts);
    let report = OpaqueReport {
        clean_facts: fact_set.len(),
        opaque,
        by_category,
    };

    Ok(CorpusExtraction {
        facts: fact_set,
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

/// Per-file walk state: the accumulating [`Extraction`] plus a stable, reformatting-invariant
/// disambiguator counter for `impl` blocks.
///
/// The counter is keyed by the impl's identity (module path + trait + self-type) and increments in
/// DOCUMENT ORDER. Document order is invariant under pure whitespace/comment reformatting (it does
/// not move items), so two `impl Foo` blocks on the same type get stable ordinals `#0`, `#1` that —
/// unlike a source LINE number — do not shift when blank lines are inserted. This is the fix for the
/// spike-discovered defect that a line-based impl anchor churned under reformatting.
#[derive(Debug, Default)]
struct WalkState {
    extraction: Extraction,
    impl_ordinals: BTreeMap<String, usize>,
}

impl WalkState {
    /// The next stable ordinal for an impl identity key, incrementing in document order.
    fn next_impl_ordinal(&mut self, key: &str) -> usize {
        let slot = self.impl_ordinals.entry(key.to_owned()).or_insert(0);
        let ordinal = *slot;
        *slot += 1;
        ordinal
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
    let generic_params: Vec<String> = sig
        .generics
        .params
        .iter()
        .map(normalize_tokens)
        .collect();
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
        let predicates: Vec<String> = where_clause.predicates.iter().map(normalize_tokens).collect();
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
                state.extraction.opaque.push(OpaqueReason::CfgGated(join_path(
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
                fqpath,
                kind,
                normalize_visibility(&item_fn.vis),
                &signature,
                &body,
            ));
        }
        syn::Item::Struct(s) => {
            push_type(crate_id, module_path, &s.attrs, &s.vis, &s.ident, item, &mut state.extraction);
        }
        syn::Item::Enum(e) => {
            push_type(crate_id, module_path, &e.attrs, &e.vis, &e.ident, item, &mut state.extraction);
        }
        syn::Item::Union(u) => {
            push_type(crate_id, module_path, &u.attrs, &u.vis, &u.ident, item, &mut state.extraction);
        }
        syn::Item::Trait(t) => {
            push_type(crate_id, module_path, &t.attrs, &t.vis, &t.ident, item, &mut state.extraction);
        }
        syn::Item::Type(t) => {
            push_type(crate_id, module_path, &t.attrs, &t.vis, &t.ident, item, &mut state.extraction);
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
            state.extraction.opaque.push(OpaqueReason::MacroGenerated(join_path(
                module_path,
                &format!("{name}!"),
            )));
        }
        syn::Item::Const(c) => {
            push_pub_item(crate_id, module_path, &c.attrs, &c.vis, &c.ident.to_string(), item, &mut state.extraction);
        }
        syn::Item::Static(s) => {
            push_pub_item(crate_id, module_path, &s.attrs, &s.vis, &s.ident.to_string(), item, &mut state.extraction);
        }
        // Other item kinds (use/extern crate/foreign mod/trait-alias/verbatim) are not surfaced as
        // liveness facts in v1; they carry no stable callable/type surface the governance layer pins.
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
        fqpath,
        ItemKind::Type,
        normalize_visibility(vis),
        &tokens,
        "",
    ));
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
        fqpath,
        ItemKind::PubItem,
        normalize_visibility(vis),
        &tokens,
        "",
    ));
}

/// Walk an `impl` block: emit one `Impl` fact for the block and recurse its methods as
/// `Function`/`Route` facts under the self-type path.
///
/// The impl is anchored to its `(trait, self-type)` identity plus a stable document-order ordinal
/// (`#impl[n]`) from [`WalkState::next_impl_ordinal`]. The ordinal — NOT a source line number — is
/// what keeps two `impl Foo` blocks distinct WHILE staying invariant under whitespace/comment
/// reformatting (inserting blank lines moves line numbers but not document order). This is the fix
/// for the spike-discovered defect where a line-based anchor churned the signature hash on reformat.
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
    // The implemented trait (if any) is part of the impl identity so `impl Foo` and `impl Bar for
    // Foo` never share an ordinal sequence.
    let trait_key = item_impl
        .trait_
        .as_ref()
        .map(|(_, path, _)| normalize_tokens(path).replace(' ', ""))
        .unwrap_or_default();
    let identity = join_path(module_path, &format!("{trait_key}@{self_key}"));
    let ordinal = state.next_impl_ordinal(&identity);
    let impl_fqpath = join_path(module_path, &format!("{self_key}#impl[{ordinal}]"));
    // The impl fact's signature pre-image is the trait+self-type identity, NOT the ordinal, so the
    // anchor depends only on what the impl IS (trait for type), invariant under reordering siblings.
    let impl_sig = format!("impl {trait_key} for {self_key}");
    state.extraction.facts.push(Function::new(
        crate_id,
        impl_fqpath,
        ItemKind::Impl,
        // An impl block has no visibility modifier; treat as Private (its methods carry their own).
        Visibility::Private,
        &impl_sig,
        "",
    ));

    let method_base = join_path(module_path, &self_key);
    for impl_item in &item_impl.items {
        if let syn::ImplItem::Fn(method) = impl_item {
            if has_cfg_attr(&method.attrs) {
                state.extraction.opaque.push(OpaqueReason::CfgGated(join_path(
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
                fqpath,
                kind,
                normalize_visibility(&method.vis),
                &signature,
                &body,
            ));
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
        state.extraction.opaque.push(OpaqueReason::CfgGated(join_path(
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
