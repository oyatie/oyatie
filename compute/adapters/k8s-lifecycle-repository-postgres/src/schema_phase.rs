use crate::schema_contract::{
    EXPECTED_COLUMNS, EXPECTED_CONSTRAINTS, EXPECTED_GRANTS, EXPECTED_INDEXES, EXPECTED_POLICIES,
    PENDING_COLUMNS, PENDING_CONSTRAINTS,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SchemaPhase {
    Empty,
    RoleBoundary,
    LegacyRepository,
    LegacyAdoption { ledger: bool, ledger_read: bool },
    PendingIntentRepository,
}

impl SchemaPhase {
    pub(crate) fn has_runtime(self) -> bool {
        self != Self::Empty
    }

    fn has_repository(self) -> bool {
        matches!(
            self,
            Self::LegacyRepository | Self::LegacyAdoption { .. } | Self::PendingIntentRepository
        )
    }

    fn has_ledger(self) -> bool {
        !matches!(
            self,
            Self::Empty | Self::LegacyAdoption { ledger: false, .. }
        )
    }

    pub(crate) fn relations(self) -> Vec<&'static str> {
        let mut expected = Vec::new();
        if self.has_repository() {
            expected.extend([
                "clusters|r|p|true|true|false|heap|true|true",
                "operations|r|p|true|true|false|heap|true|true",
            ]);
        }
        if self.has_ledger() {
            expected.push("schema_migrations|r|p|false|false|false|heap|true|true");
        }
        expected
    }

    fn native(self, source: &[&'static str]) -> Vec<&'static str> {
        source
            .iter()
            .copied()
            .filter(|fact| match fact.split('|').next() {
                Some("clusters" | "operations") => self.has_repository(),
                Some("schema_migrations") => self.has_ledger(),
                _ => false,
            })
            .collect()
    }

    pub(crate) fn columns(self) -> Vec<&'static str> {
        let mut expected = self.native(EXPECTED_COLUMNS);
        if self == Self::PendingIntentRepository {
            expected.extend_from_slice(PENDING_COLUMNS);
        }
        expected
    }

    pub(crate) fn constraints(self) -> Vec<&'static str> {
        let mut expected = self.native(EXPECTED_CONSTRAINTS);
        if self == Self::PendingIntentRepository {
            expected.extend_from_slice(PENDING_CONSTRAINTS);
        }
        expected
    }

    pub(crate) fn indexes(self) -> Vec<&'static str> {
        self.native(EXPECTED_INDEXES)
    }
    pub(crate) fn policies(self) -> Vec<&'static str> {
        self.native(EXPECTED_POLICIES)
    }

    pub(crate) fn grants(self) -> Vec<&'static str> {
        EXPECTED_GRANTS
            .iter()
            .copied()
            .filter(|fact| {
                if fact.starts_with("schema|") {
                    return self.has_runtime();
                }
                if fact.starts_with("table|schema_migrations|") {
                    return self.has_ledger()
                        && !matches!(
                            self,
                            Self::LegacyAdoption {
                                ledger_read: false,
                                ..
                            }
                        );
                }
                self.has_repository()
            })
            .collect()
    }
}
