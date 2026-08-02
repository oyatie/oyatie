//! Foundry cohesion-fitness kernel.

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossAxisContract {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub owner_axis: String,            // data_class: INTERNAL_ONLY
    pub consumer_axes: Vec<String>,    // data_class: INTERNAL_ONLY
    pub location: String,              // data_class: INTERNAL_ONLY
    pub change_review: String,         // data_class: INTERNAL_ONLY
    pub source_crate_ids: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CohesionFitnessReport {
    pub contracts_checked: usize,           // data_class: INTERNAL_ONLY
    pub implemented_sources_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CohesionFitnessError {
    NoContracts,
    EmptyContractId,
    DuplicateContractId {
        contract_id: String,
    },
    MissingOwnerAxis {
        contract_id: String,
    },
    MissingConsumerAxes {
        contract_id: String,
    },
    MissingLocation {
        contract_id: String,
    },
    MissingChangeReview {
        contract_id: String,
    },
    ImplementedSourceMissingCatalog {
        contract_id: String,
        crate_id: String,
    },
}

pub fn validate_cohesion_fitness<C, W>(
    contracts: &[CrossAxisContract],
    catalog_crate_ids: C,
    workspace_crate_ids: W,
) -> Result<CohesionFitnessReport, CohesionFitnessError>
where
    C: IntoIterator,
    C::Item: AsRef<str>,
    W: IntoIterator,
    W::Item: AsRef<str>,
{
    if contracts.is_empty() {
        return Err(CohesionFitnessError::NoContracts);
    }
    let catalog_crate_ids = catalog_crate_ids
        .into_iter()
        .map(|crate_id| crate_id.as_ref().to_string())
        .collect::<BTreeSet<_>>();
    let workspace_crate_ids = workspace_crate_ids
        .into_iter()
        .map(|crate_id| crate_id.as_ref().to_string())
        .collect::<BTreeSet<_>>();

    let mut contract_ids = BTreeSet::new();
    let mut implemented_sources_checked = 0;
    for contract in contracts {
        if contract.id.trim().is_empty() {
            return Err(CohesionFitnessError::EmptyContractId);
        }
        if !contract_ids.insert(contract.id.clone()) {
            return Err(CohesionFitnessError::DuplicateContractId {
                contract_id: contract.id.clone(),
            });
        }
        if contract.owner_axis.trim().is_empty() {
            return Err(CohesionFitnessError::MissingOwnerAxis {
                contract_id: contract.id.clone(),
            });
        }
        if contract.consumer_axes.is_empty()
            || contract
                .consumer_axes
                .iter()
                .any(|axis| axis.trim().is_empty())
        {
            return Err(CohesionFitnessError::MissingConsumerAxes {
                contract_id: contract.id.clone(),
            });
        }
        if contract.location.trim().is_empty() {
            return Err(CohesionFitnessError::MissingLocation {
                contract_id: contract.id.clone(),
            });
        }
        if contract.change_review.trim().is_empty() {
            return Err(CohesionFitnessError::MissingChangeReview {
                contract_id: contract.id.clone(),
            });
        }
        for source_crate_id in &contract.source_crate_ids {
            if !workspace_crate_ids.contains(source_crate_id) {
                continue;
            }
            implemented_sources_checked += 1;
            if !catalog_crate_ids.contains(source_crate_id) {
                return Err(CohesionFitnessError::ImplementedSourceMissingCatalog {
                    contract_id: contract.id.clone(),
                    crate_id: source_crate_id.clone(),
                });
            }
        }
    }

    Ok(CohesionFitnessReport {
        contracts_checked: contracts.len(),
        implemented_sources_checked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_contract_ids() {
        let contract = contract("TENANT_KERNEL", &["oya-platform-tenant-kernel"]);

        assert_eq!(
            validate_cohesion_fitness(
                &[contract.clone(), contract],
                ["oya-platform-tenant-kernel"],
                ["oya-platform-tenant-kernel"]
            ),
            Err(CohesionFitnessError::DuplicateContractId {
                contract_id: "TENANT_KERNEL".into()
            })
        );
    }

    #[test]
    fn rejects_missing_change_review() {
        let mut contract = contract("TENANT_KERNEL", &["oya-platform-tenant-kernel"]);
        contract.change_review = " ".into();

        assert_eq!(
            validate_cohesion_fitness(
                &[contract],
                ["oya-platform-tenant-kernel"],
                ["oya-platform-tenant-kernel"]
            ),
            Err(CohesionFitnessError::MissingChangeReview {
                contract_id: "TENANT_KERNEL".into()
            })
        );
    }

    #[test]
    fn rejects_implemented_source_crate_missing_catalog_record() {
        assert_eq!(
            validate_cohesion_fitness(
                &[contract("TENANT_KERNEL", &["oya-platform-tenant-kernel"])],
                ["oya-platform-identity-kernel"],
                ["oya-platform-tenant-kernel"]
            ),
            Err(CohesionFitnessError::ImplementedSourceMissingCatalog {
                contract_id: "TENANT_KERNEL".into(),
                crate_id: "oya-platform-tenant-kernel".into()
            })
        );
    }

    #[test]
    fn accepts_implemented_sources_with_catalog_records_and_future_sources() {
        assert_eq!(
            validate_cohesion_fitness(
                &[
                    contract("TENANT_KERNEL", &["oya-platform-tenant-kernel"]),
                    contract("CLOUD_RESOURCE_TYPE", &["oya-cloud-resource-kernel"]),
                ],
                ["oya-platform-tenant-kernel"],
                ["oya-platform-tenant-kernel"]
            ),
            Ok(CohesionFitnessReport {
                contracts_checked: 2,
                implemented_sources_checked: 1,
            })
        );
    }

    fn contract(id: &str, source_crate_ids: &[&str]) -> CrossAxisContract {
        CrossAxisContract {
            id: id.into(),
            owner_axis: "saas".into(),
            consumer_axes: vec!["all".into()],
            location: "crates/oya-platform-tenant-kernel".into(),
            change_review: "cross-axis".into(),
            source_crate_ids: source_crate_ids
                .iter()
                .map(|crate_id| (*crate_id).into())
                .collect(),
        }
    }
}
