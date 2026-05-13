//! Vendor contract recency fitness kernel.
//!
//! The vendor ledger is allowed to bootstrap with an explicit no-signed-contracts
//! declaration, but contracted vendors must carry expiry dates and renewal tasks
//! when they are within the 90-day renewal window.

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VendorContractRecord {
    pub contract_id: String,            // data_class: INTERNAL_ONLY
    pub vendor: String,                 // data_class: INTERNAL_ONLY
    pub status: String,                 // data_class: INTERNAL_ONLY
    pub expiry_epoch_days: Option<i64>, // data_class: INTERNAL_ONLY
    pub renewal_task: Option<String>,   // data_class: INTERNAL_ONLY
    pub owner_team: String,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorContractRecencyPolicy {
    pub renewal_window_days: i64, // data_class: INTERNAL_ONLY
}

impl VendorContractRecencyPolicy {
    pub fn default_sla() -> Self {
        Self {
            renewal_window_days: 90,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorContractRecencyReport {
    pub records_checked: usize,                 // data_class: INTERNAL_ONLY
    pub contracted_records_checked: usize,      // data_class: INTERNAL_ONLY
    pub renewal_tasks_required_checked: usize,  // data_class: INTERNAL_ONLY
    pub no_signed_contract_declarations: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VendorContractRecencyError {
    NoRecords,
    DuplicateContractId {
        contract_id: String,
    },
    MissingField {
        contract_id: String,
        field: &'static str,
    },
    UnknownStatus {
        contract_id: String,
        status: String,
    },
    ContractMissingExpiry {
        contract_id: String,
    },
    NonContractRecordHasExpiry {
        contract_id: String,
    },
    RenewalTaskRequired {
        contract_id: String,
        days_until_expiry: i64,
    },
    RenewalTaskShapeInvalid {
        contract_id: String,
        renewal_task: String,
    },
    InvalidPolicy,
}

pub fn validate_vendor_contract_recency<R>(
    records: R,
    today_epoch_days: i64,
    policy: VendorContractRecencyPolicy,
) -> Result<VendorContractRecencyReport, VendorContractRecencyError>
where
    R: IntoIterator<Item = VendorContractRecord>,
{
    if policy.renewal_window_days < 0 {
        return Err(VendorContractRecencyError::InvalidPolicy);
    }

    let mut seen = BTreeSet::new();
    let mut records_checked = 0usize;
    let mut contracted_records_checked = 0usize;
    let mut renewal_tasks_required_checked = 0usize;
    let mut no_signed_contract_declarations = 0usize;

    for record in records {
        records_checked += 1;
        validate_required_fields(&record)?;
        if !seen.insert(record.contract_id.clone()) {
            return Err(VendorContractRecencyError::DuplicateContractId {
                contract_id: record.contract_id,
            });
        }

        match record.status.as_str() {
            "contracted" => {
                contracted_records_checked += 1;
                let Some(expiry_epoch_days) = record.expiry_epoch_days else {
                    return Err(VendorContractRecencyError::ContractMissingExpiry {
                        contract_id: record.contract_id,
                    });
                };
                let days_until_expiry = expiry_epoch_days - today_epoch_days;
                if days_until_expiry <= policy.renewal_window_days {
                    let Some(renewal_task) = usable_task(record.renewal_task.as_deref()) else {
                        return Err(VendorContractRecencyError::RenewalTaskRequired {
                            contract_id: record.contract_id,
                            days_until_expiry,
                        });
                    };
                    if !valid_renewal_task_ref(renewal_task) {
                        return Err(VendorContractRecencyError::RenewalTaskShapeInvalid {
                            contract_id: record.contract_id,
                            renewal_task: renewal_task.to_string(),
                        });
                    }
                    renewal_tasks_required_checked += 1;
                }
            }
            "no-signed-contracts" | "non-contracting-terms" | "retired" => {
                if record.expiry_epoch_days.is_some() {
                    return Err(VendorContractRecencyError::NonContractRecordHasExpiry {
                        contract_id: record.contract_id,
                    });
                }
                if record.status == "no-signed-contracts" {
                    no_signed_contract_declarations += 1;
                }
            }
            status => {
                return Err(VendorContractRecencyError::UnknownStatus {
                    contract_id: record.contract_id,
                    status: status.to_string(),
                });
            }
        }
    }

    if records_checked == 0 {
        return Err(VendorContractRecencyError::NoRecords);
    }

    Ok(VendorContractRecencyReport {
        records_checked,
        contracted_records_checked,
        renewal_tasks_required_checked,
        no_signed_contract_declarations,
    })
}

fn validate_required_fields(
    record: &VendorContractRecord,
) -> Result<(), VendorContractRecencyError> {
    for (field, value) in [
        ("contract_id", &record.contract_id),
        ("vendor", &record.vendor),
        ("status", &record.status),
        ("owner_team", &record.owner_team),
    ] {
        if value.trim().is_empty() {
            return Err(VendorContractRecencyError::MissingField {
                contract_id: record.contract_id.clone(),
                field,
            });
        }
    }
    Ok(())
}

fn usable_task(value: Option<&str>) -> Option<&str> {
    let value = value?.trim();
    if value.is_empty() || matches!(value, "n/a" | "N/A" | "none" | "None" | "-") {
        None
    } else {
        Some(value)
    }
}

fn valid_renewal_task_ref(value: &str) -> bool {
    value.starts_with("gh:")
        || value.starts_with("jira:")
        || value.starts_with("linear:")
        || value.starts_with("task:")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TODAY: i64 = 20_000;

    #[test]
    fn accepts_explicit_no_signed_contract_declaration() {
        assert_eq!(
            validate_vendor_contract_recency(
                [VendorContractRecord {
                    contract_id: "vcr-none-2026-05-10".into(),
                    vendor: "All listed vendors and partners".into(),
                    status: "no-signed-contracts".into(),
                    expiry_epoch_days: None,
                    renewal_task: None,
                    owner_team: "gtm-partnerships + ops-security".into(),
                }],
                TODAY,
                VendorContractRecencyPolicy::default_sla(),
            ),
            Ok(VendorContractRecencyReport {
                records_checked: 1,
                contracted_records_checked: 0,
                renewal_tasks_required_checked: 0,
                no_signed_contract_declarations: 1,
            })
        );
    }

    #[test]
    fn accepts_contract_outside_renewal_window_without_task() {
        assert_eq!(
            validate_vendor_contract_recency(
                [contract(Some(TODAY + 120), None)],
                TODAY,
                VendorContractRecencyPolicy::default_sla(),
            ),
            Ok(VendorContractRecencyReport {
                records_checked: 1,
                contracted_records_checked: 1,
                renewal_tasks_required_checked: 0,
                no_signed_contract_declarations: 0,
            })
        );
    }

    #[test]
    fn accepts_contract_inside_renewal_window_with_task() {
        assert_eq!(
            validate_vendor_contract_recency(
                [contract(Some(TODAY + 30), Some("gh:oyatie/oyatie#123"))],
                TODAY,
                VendorContractRecencyPolicy::default_sla(),
            ),
            Ok(VendorContractRecencyReport {
                records_checked: 1,
                contracted_records_checked: 1,
                renewal_tasks_required_checked: 1,
                no_signed_contract_declarations: 0,
            })
        );
    }

    #[test]
    fn rejects_contract_inside_renewal_window_without_task() {
        assert_eq!(
            validate_vendor_contract_recency(
                [contract(Some(TODAY + 30), None)],
                TODAY,
                VendorContractRecencyPolicy::default_sla(),
            ),
            Err(VendorContractRecencyError::RenewalTaskRequired {
                contract_id: "ctr-oci-001".into(),
                days_until_expiry: 30,
            })
        );
    }

    #[test]
    fn rejects_contract_missing_expiry() {
        assert_eq!(
            validate_vendor_contract_recency(
                [contract(None, Some("gh:oyatie/oyatie#123"))],
                TODAY,
                VendorContractRecencyPolicy::default_sla(),
            ),
            Err(VendorContractRecencyError::ContractMissingExpiry {
                contract_id: "ctr-oci-001".into(),
            })
        );
    }

    #[test]
    fn rejects_duplicate_contract_ids_and_empty_sets() {
        assert_eq!(
            validate_vendor_contract_recency([], TODAY, VendorContractRecencyPolicy::default_sla()),
            Err(VendorContractRecencyError::NoRecords)
        );
        assert_eq!(
            validate_vendor_contract_recency(
                [
                    contract(Some(TODAY + 120), None),
                    contract(Some(TODAY + 120), None)
                ],
                TODAY,
                VendorContractRecencyPolicy::default_sla(),
            ),
            Err(VendorContractRecencyError::DuplicateContractId {
                contract_id: "ctr-oci-001".into(),
            })
        );
    }

    fn contract(
        expiry_epoch_days: Option<i64>,
        renewal_task: Option<&str>,
    ) -> VendorContractRecord {
        VendorContractRecord {
            contract_id: "ctr-oci-001".into(),
            vendor: "OCI".into(),
            status: "contracted".into(),
            expiry_epoch_days,
            renewal_task: renewal_task.map(str::to_string),
            owner_team: "gtm-partnerships".into(),
        }
    }
}
