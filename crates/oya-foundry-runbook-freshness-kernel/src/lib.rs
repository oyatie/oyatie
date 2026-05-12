//! Foundry runbook freshness fitness kernel.
//!
//! RUNBOOKS-INDEX §3 defines freshness SLAs by severity: Sev-1 within 90 days,
//! Sev-2 within 180 days, and Sev-3/4 within 365 days. Deferred stubs that do
//! not yet carry a severity scope are treated as Sev-4-equivalent for freshness:
//! they must still carry a verifiable date and cannot silently rot, but this
//! bootstrap gate does not claim the full procedure has landed.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunbookFreshnessRecord {
    pub path: String,                          // data_class: INTERNAL_ONLY
    pub status: Option<String>,                // data_class: INTERNAL_ONLY
    pub severity_scope: Option<String>,        // data_class: INTERNAL_ONLY
    pub last_verified_epoch_days: Option<i64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunbookFreshnessReport {
    pub runbooks_checked: usize,         // data_class: INTERNAL_ONLY
    pub severity_scoped_runbooks: usize, // data_class: INTERNAL_ONLY
    pub unscoped_runbooks: usize,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunbookFreshnessError {
    NoRunbooks,
    MissingStatus {
        path: String,
    },
    MissingLastVerified {
        path: String,
    },
    FutureLastVerified {
        path: String,
    },
    UnknownSeverity {
        path: String,
        severity: String,
    },
    StaleRunbook {
        path: String,
        severity: String,
        age_days: i64,
        max_age_days: i64,
    },
}

pub fn validate_runbook_freshness<R>(
    records: R,
    today_epoch_days: i64,
) -> Result<RunbookFreshnessReport, RunbookFreshnessError>
where
    R: IntoIterator<Item = RunbookFreshnessRecord>,
{
    let mut runbooks_checked = 0usize;
    let mut severity_scoped_runbooks = 0usize;
    let mut unscoped_runbooks = 0usize;

    for record in records {
        runbooks_checked += 1;
        if record
            .status
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
        {
            return Err(RunbookFreshnessError::MissingStatus { path: record.path });
        }
        let Some(last_verified_epoch_days) = record.last_verified_epoch_days else {
            return Err(RunbookFreshnessError::MissingLastVerified { path: record.path });
        };
        if last_verified_epoch_days > today_epoch_days {
            return Err(RunbookFreshnessError::FutureLastVerified { path: record.path });
        }

        let (severity_label, max_age_days) = match record.severity_scope.as_deref().map(str::trim) {
            Some("Sev 1") | Some("Sev-1") => {
                severity_scoped_runbooks += 1;
                ("Sev 1", 90)
            }
            Some("Sev 2") | Some("Sev-2") => {
                severity_scoped_runbooks += 1;
                ("Sev 2", 180)
            }
            Some("Sev 3") | Some("Sev-3") => {
                severity_scoped_runbooks += 1;
                ("Sev 3", 365)
            }
            Some("Sev 4") | Some("Sev-4") => {
                severity_scoped_runbooks += 1;
                ("Sev 4", 365)
            }
            Some("") | None => {
                unscoped_runbooks += 1;
                ("unscoped", 365)
            }
            Some(severity) => {
                return Err(RunbookFreshnessError::UnknownSeverity {
                    path: record.path,
                    severity: severity.into(),
                });
            }
        };

        let age_days = today_epoch_days - last_verified_epoch_days;
        if age_days > max_age_days {
            return Err(RunbookFreshnessError::StaleRunbook {
                path: record.path,
                severity: severity_label.into(),
                age_days,
                max_age_days,
            });
        }
    }

    if runbooks_checked == 0 {
        Err(RunbookFreshnessError::NoRunbooks)
    } else {
        Ok(RunbookFreshnessReport {
            runbooks_checked,
            severity_scoped_runbooks,
            unscoped_runbooks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_fresh_severity_scoped_and_unscoped_runbooks() {
        assert_eq!(
            validate_runbook_freshness(
                [
                    record("sev1.md", Some("Sev 1"), Some(10)),
                    record("stub.md", None, Some(1)),
                ],
                100,
            ),
            Ok(RunbookFreshnessReport {
                runbooks_checked: 2,
                severity_scoped_runbooks: 1,
                unscoped_runbooks: 1,
            })
        );
    }

    #[test]
    fn rejects_stale_sev1_and_unscoped_runbooks() {
        assert_eq!(
            validate_runbook_freshness([record("sev1.md", Some("Sev 1"), Some(9))], 100),
            Err(RunbookFreshnessError::StaleRunbook {
                path: "sev1.md".into(),
                severity: "Sev 1".into(),
                age_days: 91,
                max_age_days: 90,
            })
        );

        assert_eq!(
            validate_runbook_freshness([record("stub.md", None, Some(0))], 366),
            Err(RunbookFreshnessError::StaleRunbook {
                path: "stub.md".into(),
                severity: "unscoped".into(),
                age_days: 366,
                max_age_days: 365,
            })
        );
    }

    #[test]
    fn rejects_missing_status_and_missing_date() {
        assert_eq!(
            validate_runbook_freshness(
                [RunbookFreshnessRecord {
                    path: "missing-status.md".into(),
                    status: None,
                    severity_scope: Some("Sev 2".into()),
                    last_verified_epoch_days: Some(10),
                }],
                100,
            ),
            Err(RunbookFreshnessError::MissingStatus {
                path: "missing-status.md".into(),
            })
        );

        assert_eq!(
            validate_runbook_freshness(
                [RunbookFreshnessRecord {
                    path: "missing-date.md".into(),
                    status: Some("Stub".into()),
                    severity_scope: Some("Sev 2".into()),
                    last_verified_epoch_days: None,
                }],
                100,
            ),
            Err(RunbookFreshnessError::MissingLastVerified {
                path: "missing-date.md".into(),
            })
        );
    }

    #[test]
    fn rejects_future_date_unknown_severity_and_empty_set() {
        assert_eq!(
            validate_runbook_freshness([record("future.md", Some("Sev 3"), Some(101))], 100),
            Err(RunbookFreshnessError::FutureLastVerified {
                path: "future.md".into(),
            })
        );

        assert_eq!(
            validate_runbook_freshness([record("bad.md", Some("Critical"), Some(10))], 100),
            Err(RunbookFreshnessError::UnknownSeverity {
                path: "bad.md".into(),
                severity: "Critical".into(),
            })
        );

        assert_eq!(
            validate_runbook_freshness([], 100),
            Err(RunbookFreshnessError::NoRunbooks)
        );
    }

    fn record(
        path: &str,
        severity_scope: Option<&str>,
        last_verified_epoch_days: Option<i64>,
    ) -> RunbookFreshnessRecord {
        RunbookFreshnessRecord {
            path: path.into(),
            status: Some("Stub".into()),
            severity_scope: severity_scope.map(str::to_string),
            last_verified_epoch_days,
        }
    }
}
