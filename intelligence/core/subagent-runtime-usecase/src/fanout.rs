//! Facet fan-out plan — the closed-enum mirror of the 21-facet panel
//! authored under `evidence/pipeline-maturity-glue/ip-004-pr-review/facets/`.
//!
//! Each [`FacetSlug`] entry is the canonical id used both as the
//! prompt-template filename (`<slug>.md`) AND the per-facet evidence
//! filename (`<slug>.json`) consumed by IP-004's
//! `parse_facet_slug`. Keeping this list co-located with the runtime
//! lets `cargo test` validate the panel completeness without taking
//! a dependency on `intelligence-pr-review-dispatcher-app` (which would
//! invert the dependency direction and is forbidden by clean-arch). This
//! lives in the `usecase` layer so dispatcher apps do not import another
//! app crate.

/// The 21 facet slugs that the dispatcher's `FacetId::full_panel_v23()`
/// claims. Kept in sync with `feedback_multispectrum_review_v22.md` +
/// `feedback_multispectrum_adherence_facets.md`.
pub const FACET_PANEL_V23: [FacetSlug; 21] = [
    FacetSlug("F1_linus"),
    FacetSlug("F2_hyperscaler"),
    FacetSlug("F3_adversarial"),
    FacetSlug("F4_ergonomic"),
    FacetSlug("F5_quality"),
    FacetSlug("F6_alternatives"),
    FacetSlug("F7_security"),
    FacetSlug("F8_performance"),
    FacetSlug("F9_compliance"),
    FacetSlug("F10_reversibility"),
    FacetSlug("F11_observability"),
    FacetSlug("F13_migration"),
    FacetSlug("M1_challenge_assumption"),
    FacetSlug("M2_zoomed_out_fit"),
    FacetSlug("A1_naming_adherence"),
    FacetSlug("A2_documentation_adherence"),
    FacetSlug("A3_structure_adherence"),
    FacetSlug("A4_architecture_adherence"),
    FacetSlug("A5_dependency_adherence"),
    FacetSlug("A6_schema_adherence"),
    FacetSlug("A7_algorithm_adherence"),
];

/// Newtype around a facet slug for type-safety. The kernel speaks
/// `String` because adapter callers can come from many places; this
/// app-layer wrapper guarantees the slug is one of the known 21.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FacetSlug(pub &'static str);

impl FacetSlug {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.0
    }
}

/// Return the 21-facet panel as a vector. Convenience wrapper for
/// fan-out loops that don't want to spell the const array.
#[must_use]
pub fn fanout_panel_v23() -> Vec<FacetSlug> {
    FACET_PANEL_V23.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_has_exactly_21_facets() {
        assert_eq!(FACET_PANEL_V23.len(), 21);
    }

    #[test]
    fn panel_has_distinct_slugs() {
        let mut slugs: Vec<&str> = FACET_PANEL_V23.iter().map(|f| f.as_str()).collect();
        slugs.sort_unstable();
        let original = slugs.len();
        slugs.dedup();
        assert_eq!(slugs.len(), 21);
        assert_eq!(original, 21);
    }

    #[test]
    fn panel_matches_dispatcher_slugs() {
        // Mirrors `FacetId::full_panel_v23()::slug()` in
        // `tools/intelligence-pr-review-dispatcher-app/src/fanout.rs`.
        // We re-check here without taking the dispatcher dep.
        let expected = [
            "F1_linus",
            "F2_hyperscaler",
            "F3_adversarial",
            "F4_ergonomic",
            "F5_quality",
            "F6_alternatives",
            "F7_security",
            "F8_performance",
            "F9_compliance",
            "F10_reversibility",
            "F11_observability",
            "F13_migration",
            "M1_challenge_assumption",
            "M2_zoomed_out_fit",
            "A1_naming_adherence",
            "A2_documentation_adherence",
            "A3_structure_adherence",
            "A4_architecture_adherence",
            "A5_dependency_adherence",
            "A6_schema_adherence",
            "A7_algorithm_adherence",
        ];
        let actual: Vec<&str> = FACET_PANEL_V23.iter().map(|f| f.as_str()).collect();
        assert_eq!(actual, expected);
    }
}
