//! Foundry portfolio-citation fitness kernel.
//!
//! Verifies the bidirectional citation between `oyatie/docs/PRD.md` and
//! `bominal/docs/consolidated/PRD.md`. The kernel is I/O-free: runners parse
//! Markdown citation blocks and pass value objects into [`verify`].

pub const OYATIE_PRD_PATH: &str = "oyatie/docs/PRD.md";
pub const BOMINAL_PRD_PATH: &str = "bominal/docs/consolidated/PRD.md";
pub const REQUIRED_FOUNDRY_CORPUS_SOURCES: [&str; 3] = [
    "bominal/agents/ultragoal/2026-05-12-foundry-ultragoal-mega-plan.md",
    "bominal/agents/ultragoal/foundry-agentic-substrate-master.md",
    "bominal/agents/ultragoal/oyatie-product-delivery-implementation-plan.md",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CitationBlock {
    pub target_path: String,    // data_class: INTERNAL_ONLY
    pub role: CitationRole,     // data_class: INTERNAL_ONLY
    pub anchor: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CitationRole {
    PortfolioParent,
    CanonicalImplHome,
    FoundryCorpusSource,
}

impl CitationRole {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "PortfolioParent" | "portfolio-parent" | "portfolio_parent" => {
                Some(Self::PortfolioParent)
            }
            "CanonicalImplHome" | "canonical-implementation-home" | "canonical_impl_home" => {
                Some(Self::CanonicalImplHome)
            }
            "FoundryCorpusSource" | "foundry-corpus-source" | "foundry_corpus_source" => {
                Some(Self::FoundryCorpusSource)
            }
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PortfolioParent => "PortfolioParent",
            Self::CanonicalImplHome => "CanonicalImplHome",
            Self::FoundryCorpusSource => "FoundryCorpusSource",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioCitationVerdict {
    pub oyatie_cites_bominal: bool, // data_class: INTERNAL_ONLY
    pub bominal_cites_oyatie: bool, // data_class: INTERNAL_ONLY
    pub citations_checked: usize,   // data_class: INTERNAL_ONLY
}

impl PortfolioCitationVerdict {
    pub fn is_complete(&self) -> bool {
        self.oyatie_cites_bominal && self.bominal_cites_oyatie
    }
}

pub fn verify(
    oyatie_prd_citations: &[CitationBlock],
    bominal_prd_citations: &[CitationBlock],
) -> PortfolioCitationVerdict {
    let oyatie_cites_bominal = oyatie_prd_citations.iter().any(|citation| {
        citation.role == CitationRole::PortfolioParent && citation.target_path == BOMINAL_PRD_PATH
    });
    let bominal_cites_oyatie = bominal_prd_citations.iter().any(|citation| {
        citation.role == CitationRole::CanonicalImplHome && citation.target_path == OYATIE_PRD_PATH
    });

    PortfolioCitationVerdict {
        oyatie_cites_bominal,
        bominal_cites_oyatie,
        citations_checked: oyatie_prd_citations.len() + bominal_prd_citations.len(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCorpusCitationVerdict {
    pub required_sources_total: usize, // data_class: INTERNAL_ONLY
    pub present_sources: Vec<String>,  // data_class: INTERNAL_ONLY
    pub missing_sources: Vec<String>,  // data_class: INTERNAL_ONLY
    pub citations_checked: usize,      // data_class: INTERNAL_ONLY
}

impl FoundryCorpusCitationVerdict {
    pub fn is_complete(&self) -> bool {
        self.missing_sources.is_empty()
    }
}

pub fn verify_foundry_corpus(
    foundry_prd_citations: &[CitationBlock],
) -> FoundryCorpusCitationVerdict {
    let mut present_sources = Vec::new();
    let mut missing_sources = Vec::new();

    for required_source in REQUIRED_FOUNDRY_CORPUS_SOURCES {
        let present = foundry_prd_citations.iter().any(|citation| {
            citation.role == CitationRole::FoundryCorpusSource
                && citation.target_path == required_source
        });
        if present {
            present_sources.push(required_source.to_string());
        } else {
            missing_sources.push(required_source.to_string());
        }
    }

    FoundryCorpusCitationVerdict {
        required_sources_total: REQUIRED_FOUNDRY_CORPUS_SOURCES.len(),
        present_sources,
        missing_sources,
        citations_checked: foundry_prd_citations.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_bidirectional_prd_citations() {
        let verdict = verify(
            &[block(BOMINAL_PRD_PATH, CitationRole::PortfolioParent)],
            &[block(OYATIE_PRD_PATH, CitationRole::CanonicalImplHome)],
        );

        assert!(verdict.is_complete());
        assert_eq!(verdict.citations_checked, 2);
    }

    #[test]
    fn rejects_missing_bominal_backcite() {
        let verdict = verify(
            &[block(BOMINAL_PRD_PATH, CitationRole::PortfolioParent)],
            &[],
        );

        assert!(verdict.oyatie_cites_bominal);
        assert!(!verdict.bominal_cites_oyatie);
        assert!(!verdict.is_complete());
    }

    #[test]
    fn requires_expected_role_for_each_direction() {
        let verdict = verify(
            &[block(BOMINAL_PRD_PATH, CitationRole::CanonicalImplHome)],
            &[block(OYATIE_PRD_PATH, CitationRole::PortfolioParent)],
        );

        assert!(!verdict.oyatie_cites_bominal);
        assert!(!verdict.bominal_cites_oyatie);
    }

    #[test]
    fn accepts_foundry_corpus_source_citations() {
        let citations = REQUIRED_FOUNDRY_CORPUS_SOURCES
            .iter()
            .map(|source| block(source, CitationRole::FoundryCorpusSource))
            .collect::<Vec<_>>();

        let verdict = verify_foundry_corpus(&citations);

        assert!(verdict.is_complete());
        assert_eq!(verdict.required_sources_total, 3);
        assert_eq!(verdict.present_sources.len(), 3);
        assert!(verdict.missing_sources.is_empty());
    }

    #[test]
    fn rejects_missing_foundry_corpus_source_citation() {
        let citations = [block(
            REQUIRED_FOUNDRY_CORPUS_SOURCES[0],
            CitationRole::FoundryCorpusSource,
        )];

        let verdict = verify_foundry_corpus(&citations);

        assert!(!verdict.is_complete());
        assert_eq!(verdict.present_sources.len(), 1);
        assert_eq!(verdict.missing_sources.len(), 2);
    }

    fn block(target_path: &str, role: CitationRole) -> CitationBlock {
        CitationBlock {
            target_path: target_path.into(),
            role,
            anchor: Some("product-requirements-document".into()),
        }
    }
}
