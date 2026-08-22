//! Foundry SLO coverage kernel.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SloCatalogRecord {
    pub crate_id: String,    // data_class: INTERNAL_ONLY
    pub slo: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SloCoverageReport {
    pub records_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SloCoverageError {
    EmptyCrateId,
    MissingSlo { crate_id: String },
}

pub fn validate_slo_coverage(
    records: &[SloCatalogRecord],
) -> Result<SloCoverageReport, SloCoverageError> {
    for record in records {
        if record.crate_id.trim().is_empty() {
            return Err(SloCoverageError::EmptyCrateId);
        }
        if record
            .slo
            .as_ref()
            .map(|slo| slo.trim().is_empty())
            .unwrap_or(true)
        {
            return Err(SloCoverageError::MissingSlo {
                crate_id: record.crate_id.clone(),
            });
        }
    }
    Ok(SloCoverageReport {
        records_checked: records.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_catalog_record_missing_slo_row() {
        assert_eq!(
            validate_slo_coverage(&[SloCatalogRecord {
                crate_id: "intelligence-capability-kernel".into(),
                slo: None,
            }]),
            Err(SloCoverageError::MissingSlo {
                crate_id: "intelligence-capability-kernel".into()
            })
        );
    }

    #[test]
    fn rejects_blank_slo_row() {
        assert_eq!(
            validate_slo_coverage(&[SloCatalogRecord {
                crate_id: "intelligence-capability-kernel".into(),
                slo: Some(" ".into()),
            }]),
            Err(SloCoverageError::MissingSlo {
                crate_id: "intelligence-capability-kernel".into()
            })
        );
    }

    #[test]
    fn accepts_records_with_slo_rows() {
        assert_eq!(
            validate_slo_coverage(&[
                SloCatalogRecord {
                    crate_id: "intelligence-capability-kernel".into(),
                    slo: Some("preview-control-plane".into()),
                },
                SloCatalogRecord {
                    crate_id: "intelligence-run-kernel".into(),
                    slo: Some("preview-data-plane".into()),
                },
            ]),
            Ok(SloCoverageReport { records_checked: 2 })
        );
    }
}
