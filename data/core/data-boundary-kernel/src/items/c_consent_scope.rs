use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConsentScope {
    grants: BTreeSet<(Purpose, PrivacyDataClass)>,
}

impl ConsentScope {
    /// Record a purpose-bound privacy-program grant.
    pub fn allow_privacy_data_class(
        mut self,
        purpose: Purpose,
        data_class: PrivacyDataClass,
    ) -> Self {
        self.grants.insert((purpose, data_class));
        self
    }

    /// Record a purpose-bound privacy-program grant.
    pub fn allow(self, purpose: Purpose, data_class: PrivacyDataClass) -> Self {
        self.allow_privacy_data_class(purpose, data_class)
    }

    /// Compatibility constructor for bootstrap/import seams that still carry
    /// raw `DataClass` labels. Canonical consent grants take
    /// `PrivacyDataClass`; this path fails closed for operational markers and
    /// subject markers by returning [`NonPrivacyDataClass`].
    pub fn try_allow_legacy_data_class(
        self,
        purpose: Purpose,
        data_class: DataClass,
    ) -> Result<Self, NonPrivacyDataClass> {
        let data_class = PrivacyDataClass::try_from(data_class)?;
        Ok(self.allow_privacy_data_class(purpose, data_class))
    }

    pub fn allows_privacy_data_class(
        &self,
        purpose: Purpose,
        data_class: PrivacyDataClass,
    ) -> bool {
        if is_hard_denied_classification(purpose, DataClassification::Privacy(data_class)) {
            return false;
        }
        self.grants.contains(&(purpose, data_class))
    }

    pub fn allows(&self, purpose: Purpose, data_class: PrivacyDataClass) -> bool {
        self.allows_privacy_data_class(purpose, data_class)
    }

    pub fn allows_legacy_data_class(&self, purpose: Purpose, data_class: DataClass) -> bool {
        let Ok(data_class) = PrivacyDataClass::try_from(data_class) else {
            return false;
        };
        self.allows_privacy_data_class(purpose, data_class)
    }

    pub fn allows_all(&self, purpose: Purpose, classes: &[PrivacyDataClass]) -> bool {
        classes.iter().all(|class| self.allows(purpose, *class))
    }

    pub fn allows_all_legacy_data_classes(&self, purpose: Purpose, classes: &[DataClass]) -> bool {
        classes
            .iter()
            .all(|class| self.allows_legacy_data_class(purpose, *class))
    }

    pub fn allows_classification(
        &self,
        purpose: Purpose,
        classification: impl Into<DataClassification>,
    ) -> bool {
        let classification = normalize_classification(classification.into());
        if is_hard_denied_classification(purpose, classification) {
            return false;
        }
        classification
            .privacy_data_class()
            .is_some_and(|data_class| self.allows_privacy_data_class(purpose, data_class))
    }

    pub fn allows_all_classifications(
        &self,
        purpose: Purpose,
        classifications: &[DataClassification],
    ) -> bool {
        classifications
            .iter()
            .all(|classification| self.allows_classification(purpose, *classification))
    }
}
