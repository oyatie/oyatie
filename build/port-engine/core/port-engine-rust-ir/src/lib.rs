//! # port-engine-rust-ir — Rust IR + syn/quote renderer (W0-B Slice 5).
//!
//! ADR-0637 D1 core face: holds `TargetIr` rendering with stable ordering and normalized
//! formatting. Slice 5 lands the syn/quote emit path and leakage-forbidden architecture fences;
//! CLI + six-axis end-to-end receipts remain Slice 6.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use port_engine_api::{Digest, PortError, RegionId, Renderer, TargetIr};
use quote::ToTokens;
use syn::File;

/// Fail-closed readiness gate. `true` once Slice 5 syn/quote path is wired.
pub const fn w0_ready() -> bool {
    true
}

/// Minimal in-memory Rust IR: ordered regions with optional syn ASTs (Slice 5).
///
/// Regions without an AST still participate in [`TargetIr::regions`] so empty/stub wiring and the
/// syn path share one IR type. [`EmptyRenderer`] emits zero bytes for every region;
/// [`SynQuoteRenderer`] refuses a region that lacks an AST rather than inventing empty Rust.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustIr {
    /// Deterministic region order (insertion order preserved).
    regions: Vec<RegionId>,
    /// Optional syn AST per region. Absent ⇒ empty/stub region.
    files: BTreeMap<RegionId, File>,
}

impl RustIr {
    /// Construct an IR declaring `region_ids` in deterministic order (no ASTs yet).
    #[must_use]
    pub fn new(region_ids: &[&str]) -> Self {
        Self {
            regions: region_ids
                .iter()
                .map(|id| RegionId((*id).to_owned()))
                .collect(),
            files: BTreeMap::new(),
        }
    }

    /// Attach a syn [`File`] to an already-declared region.
    ///
    /// # Errors
    /// [`PortError::Render`] when `region` was not declared in [`Self::new`].
    pub fn set_file(&mut self, region: &str, file: File) -> Result<(), PortError> {
        let id = RegionId(region.to_owned());
        if !self.regions.iter().any(|r| r == &id) {
            return Err(PortError::Render {
                detail: format!("region `{region}` is not declared on this RustIr"),
            });
        }
        self.files.insert(id, file);
        Ok(())
    }

    /// Parse `src` as a syn [`File`] and attach it to `region`.
    ///
    /// # Errors
    /// [`PortError::Render`] on undeclared region or syn parse failure.
    pub fn set_file_from_str(&mut self, region: &str, src: &str) -> Result<(), PortError> {
        let file = syn::parse_file(src).map_err(|err| PortError::Render {
            detail: format!("syn parse failed for region `{region}`: {err}"),
        })?;
        self.set_file(region, file)
    }

    /// Borrow the syn AST for `region`, if present.
    #[must_use]
    pub fn file(&self, region: &RegionId) -> Option<&File> {
        self.files.get(region)
    }
}

impl TargetIr for RustIr {
    fn target_language(&self) -> &str {
        "rust"
    }

    fn regions(&self) -> Vec<RegionId> {
        self.regions.clone()
    }
}

/// Deterministic empty renderer: emits zero-byte blobs for every declared region.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmptyRenderer {
    formatter_digest: Digest,
}

impl EmptyRenderer {
    /// Renderer with a fixed formatter digest for receipt wiring tests.
    #[must_use]
    pub fn new(formatter_digest: impl Into<String>) -> Self {
        Self {
            formatter_digest: Digest(formatter_digest.into()),
        }
    }
}

impl Renderer for EmptyRenderer {
    fn target_language(&self) -> &str {
        "rust"
    }

    fn formatter_digest(&self) -> Digest {
        self.formatter_digest.clone()
    }

    fn render(&self, ir: &dyn TargetIr) -> Result<BTreeMap<RegionId, Vec<u8>>, PortError> {
        let mut out = BTreeMap::new();
        for region in ir.regions() {
            out.insert(region, Vec::new());
        }
        Ok(out)
    }
}

/// Syn/quote renderer: emits Rust source bytes from [`RustIr`] region ASTs (Slice 5).
///
/// [`Renderer::render`] is intentionally fail-closed for bare [`TargetIr`] trait objects — the
/// syn path requires typed [`RustIr`] access via [`Self::render_rust_ir`]. Emitting empty blobs
/// through this type would lie about having run syn/quote.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SynQuoteRenderer {
    formatter_digest: Digest,
}

impl SynQuoteRenderer {
    /// Renderer with a fixed formatter digest for receipt wiring tests.
    #[must_use]
    pub fn new(formatter_digest: impl Into<String>) -> Self {
        Self {
            formatter_digest: Digest(formatter_digest.into()),
        }
    }

    /// Typed syn/quote emit path. Region order follows [`RustIr`]'s declaration order; bytes are
    /// the `TokenStream` spelling of each region's [`File`] (stable for a given AST).
    ///
    /// # Errors
    /// [`PortError::Render`] when any declared region lacks a syn AST, or when emitted UTF-8
    /// would carry a forbidden Go/corpus leakage needle (architecture fence).
    pub fn render_rust_ir(&self, ir: &RustIr) -> Result<BTreeMap<RegionId, Vec<u8>>, PortError> {
        let mut out = BTreeMap::new();
        for region in ir.regions() {
            let file = ir.file(&region).ok_or_else(|| PortError::Render {
                detail: format!(
                    "SynQuoteRenderer requires a syn AST for region `{}`",
                    region.0
                ),
            })?;
            let tokens = file.to_token_stream();
            let text = tokens.to_string();
            refuse_corpus_leakage(&text)?;
            out.insert(region, text.into_bytes());
        }
        Ok(out)
    }
}

impl Renderer for SynQuoteRenderer {
    fn target_language(&self) -> &str {
        "rust"
    }

    fn formatter_digest(&self) -> Digest {
        self.formatter_digest.clone()
    }

    fn render(&self, _ir: &dyn TargetIr) -> Result<BTreeMap<RegionId, Vec<u8>>, PortError> {
        Err(PortError::Render {
            detail: "SynQuoteRenderer::render requires typed RustIr via render_rust_ir".into(),
        })
    }
}

/// Fail closed if rendered Rust carries Go/corpus identity that must never leak through the
/// neutral syn/quote path (ADR-0637 D1: corpus vocabulary is not engine vocabulary).
fn refuse_corpus_leakage(text: &str) -> Result<(), PortError> {
    // Needles built piecewise so this source file's own fence text cannot self-match when the
    // architecture test scans production sources (mirrors frontend-go firewall style).
    let needles = [
        ["k", "8", "s", ".", "i", "o"].concat(),
        ["k", "ube", "rnete", "s"].concat(),
        ["k", "ube", "let"].concat(),
        ["api", "machin", "ery"].concat(),
        ["package ", "main"].concat(),
        ["func "].concat(),
    ];
    for needle in &needles {
        if text.contains(needle) {
            return Err(PortError::Render {
                detail: format!(
                    "syn/quote emit refused: corpus/Go leakage needle `{needle}` in rendered bytes"
                ),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_renderer_matches_declared_regions() {
        let ir = RustIr::new(&["root"]);
        let renderer = EmptyRenderer::new("fmt-stub-v0");
        let out = renderer.render(&ir).expect("empty stub must succeed");
        assert_eq!(out.len(), 1);
        assert!(out.contains_key(&RegionId("root".to_owned())));
        assert!(out.get(&RegionId("root".to_owned())).unwrap().is_empty());
    }

    #[test]
    fn syn_quote_renders_declared_fn() {
        let mut ir = RustIr::new(&["root"]);
        ir.set_file_from_str("root", "pub fn stub() {}")
            .expect("fixture must parse");
        let renderer = SynQuoteRenderer::new("fmt-syn-quote-v0");
        let out = renderer
            .render_rust_ir(&ir)
            .expect("syn/quote path must succeed");
        let text = String::from_utf8(out[&RegionId("root".into())].clone()).expect("utf-8");
        assert!(text.contains("stub"), "rendered bytes must carry fn name: {text}");
        assert!(text.contains("fn"), "rendered bytes must carry fn keyword: {text}");
    }

    #[test]
    fn syn_quote_refuses_region_without_ast() {
        let ir = RustIr::new(&["root"]);
        let renderer = SynQuoteRenderer::new("fmt-syn-quote-v0");
        let err = renderer
            .render_rust_ir(&ir)
            .expect_err("missing AST must refuse");
        assert!(matches!(err, PortError::Render { .. }));
    }

    #[test]
    fn syn_quote_trait_object_render_is_fail_closed() {
        let ir = RustIr::new(&["root"]);
        let renderer = SynQuoteRenderer::new("fmt-syn-quote-v0");
        let err = renderer.render(&ir).expect_err("dyn TargetIr must refuse");
        assert!(matches!(err, PortError::Render { .. }));
    }

    #[test]
    fn syn_quote_refuses_go_func_leakage() {
        let mut ir = RustIr::new(&["root"]);
        // A Rust string literal is still emitted into the token stream — leakage fence must fire.
        ir.set_file_from_str("root", r#"pub fn stub() { let _ = "func main"; }"#)
            .expect("fixture must parse");
        let renderer = SynQuoteRenderer::new("fmt-syn-quote-v0");
        let err = renderer
            .render_rust_ir(&ir)
            .expect_err("Go func leakage must refuse");
        assert!(matches!(err, PortError::Render { .. }));
    }

    /// ADR-0637 D1 architecture fence: rust-ir production sources must not carry corpus tokens
    /// or spawn a host toolchain (formatter is a receipt axis, not an in-process shell-out).
    #[test]
    fn production_source_forbids_corpus_and_toolchain_leakage() {
        let src = include_str!("lib.rs");
        let production = src
            .split("#[cfg(test)]")
            .next()
            .expect("lib.rs must have a production section");

        let corpus_needles = [
            ["k", "8", "s", ".", "i", "o"].concat(),
            ["k", "ube", "rnete", "s"].concat(),
            ["k", "ube", "let"].concat(),
            ["api", "machin", "ery"].concat(),
        ];
        for needle in &corpus_needles {
            assert!(
                !production.contains(needle),
                "port-engine-rust-ir production sources must not embed corpus needle `{needle}`"
            );
        }

        let cmd_new = ["Command", "::", "new"].concat();
        let process_cmd = ["std", "::", "process", "::", "Command"].concat();
        assert!(
            !production.contains(&cmd_new),
            "port-engine-rust-ir production sources must not call {cmd_new}"
        );
        assert!(
            !production.contains(&process_cmd),
            "port-engine-rust-ir production sources must not import {process_cmd}"
        );
    }
}
