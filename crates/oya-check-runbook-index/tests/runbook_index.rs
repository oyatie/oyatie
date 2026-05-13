use oya_foundry_runbook_index_kernel::{validate_runbook_index_resolves, RunbookIndexError};

#[test]
fn runbook_index_resolves_when_every_indexed_path_exists() {
    let report = validate_runbook_index_resolves(
        &[
            "foundry/provider-quota-exhausted.md",
            "ops/sev-1-bridge-procedure.md",
        ],
        &[
            "foundry/provider-quota-exhausted.md",
            "ops/sev-1-bridge-procedure.md",
        ],
    )
    .expect("indexed runbooks exist");

    assert_eq!(report.indexed_count, 2);
}

#[test]
fn runbook_index_rejects_missing_and_duplicate_entries() {
    assert_eq!(
        validate_runbook_index_resolves(
            &["foundry/provider-quota-exhausted.md"],
            &["ops/sev-1-bridge-procedure.md"],
        ),
        Err(RunbookIndexError::MissingRunbook)
    );

    assert_eq!(
        validate_runbook_index_resolves(
            &[
                "foundry/provider-quota-exhausted.md",
                "foundry/provider-quota-exhausted.md"
            ],
            &["foundry/provider-quota-exhausted.md"],
        ),
        Err(RunbookIndexError::DuplicateRunbookEntry)
    );
}

#[test]
fn runbook_index_rejects_empty_index_paths() {
    assert_eq!(
        validate_runbook_index_resolves(&[""], &["foundry/provider-quota-exhausted.md"]),
        Err(RunbookIndexError::EmptyRunbookPath)
    );
}
