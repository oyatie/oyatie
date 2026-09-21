/// Derived feature lineage carries the source classifications that must be
/// inherited by a model feature or computed attribute. Policy evaluation keeps
/// every source classification live so equal-severity classes cannot erase
/// each other's purpose-bound consent or override-pack rows.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DerivedFeatureLineage {
    sources: Vec<DataClassification>, // data_class: INTERNAL_ONLY
}

impl DerivedFeatureLineage {
    pub fn from_sources(sources: impl IntoIterator<Item = DataClassification>) -> Self {
        Self {
            sources: sources.into_iter().collect(),
        }
    }

    pub fn effective_classification(&self) -> Option<DataClassification> {
        most_restrictive_policy_classification(self.source_classifications())
    }

    fn source_classifications(&self) -> impl Iterator<Item = DataClassification> + '_ {
        self.sources
            .iter()
            .copied()
            .map(canonical_policy_classification)
    }

    fn hard_denied_source(&self, purpose: Purpose) -> Option<DataClassification> {
        self.source_classifications().find(|classification| {
            DataUseBoundaryMatrix::default().is_hard_denied(purpose, *classification)
        })
    }
}

fn classification_level(classification: DataClassification) -> ClassificationLevel {
    ClassificationLevel::from_data_class(classification.compatibility_data_class())
}

fn most_restrictive_policy_classification(
    classifications: impl IntoIterator<Item = DataClassification>,
) -> Option<DataClassification> {
    classifications.into_iter().max_by_key(|classification| {
        (
            classification_level(*classification),
            classification.compatibility_data_class(),
        )
    })
}

/// ADR-0144 graduated EU AI Act risk tier.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum EuAiRiskTier {
    Minimal,
    Limited,
    GeneralPurpose,
    HighRisk,
    Unacceptable,
}

impl EuAiRiskTier {
    pub const fn blocks_deployment(self) -> bool {
        matches!(self, Self::HighRisk | Self::Unacceptable)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct EuAiRiskRegistryEntry {
    archetype: &'static str, // data_class: INTERNAL_ONLY
    tier: EuAiRiskTier,      // data_class: INTERNAL_ONLY
}

impl EuAiRiskRegistryEntry {
    pub const fn new(archetype: &'static str, tier: EuAiRiskTier) -> Self {
        Self { archetype, tier }
    }
}
