trait PublicationTransactionV1 {
    fn read_destination(&mut self) -> Result<Option<Vec<u8>>, FailureClassV1>;
    fn write_stage(&mut self, bytes: &[u8]) -> Result<(), FailureClassV1>;
    fn sync_stage(&mut self) -> Result<(), FailureClassV1>;
    fn ensure_lease(&mut self) -> Result<(), FailureClassV1>;
    fn replace(&mut self) -> Result<(), FailureClassV1>;
    fn sync_directory(&mut self) -> Result<(), FailureClassV1>;
    fn discard_stage(&mut self) -> Result<(), FailureClassV1>;
}

fn run_publication_transaction<T: PublicationTransactionV1>(
    transaction: &mut T,
    expected_preimage: Option<DigestV1>,
    bytes: &[u8],
) -> PublicationOutcomeV1 {
    let current = match transaction.read_destination() {
        Ok(current) => current,
        Err(failure) => return failed_publication(failure),
    };
    if !preimage_matches(current.as_deref(), expected_preimage) {
        return failed_publication(FailureClassV1::DestinationConflict);
    }
    if current.as_deref() == Some(bytes) {
        return unchanged_after_discard(transaction);
    }
    drop(current);

    if let Err(failure) = transaction.write_stage(bytes) {
        return fail_after_discard(transaction, failure);
    }
    if let Err(failure) = transaction.sync_stage() {
        return fail_after_discard(transaction, failure);
    }
    if let Err(failure) = transaction.ensure_lease() {
        return fail_after_discard(transaction, failure);
    }
    let current = match transaction.read_destination() {
        Ok(current) => current,
        Err(failure) => return fail_after_discard(transaction, failure),
    };
    if !preimage_matches(current.as_deref(), expected_preimage) {
        return fail_after_discard(transaction, FailureClassV1::DestinationConflict);
    }
    if current.as_deref() == Some(bytes) {
        return unchanged_after_discard(transaction);
    }
    if let Err(failure) = transaction.replace() {
        return fail_after_discard(transaction, failure);
    }
    if let Err(failure) = transaction.sync_directory() {
        return indeterminate_publication(failure);
    }
    PublicationOutcomeV1::Replaced
}

fn preimage_matches(current: Option<&[u8]>, expected: Option<DigestV1>) -> bool {
    match (current, expected) {
        (None, None) => true,
        (Some(bytes), Some(expected)) => DigestV1::of(bytes) == expected,
        (None, Some(_)) | (Some(_), None) => false,
    }
}

fn fail_after_discard<T: PublicationTransactionV1>(
    transaction: &mut T,
    failure: FailureClassV1,
) -> PublicationOutcomeV1 {
    match transaction.discard_stage() {
        Ok(()) => failed_publication(failure),
        Err(cleanup_failure) => failed_publication(cleanup_failure),
    }
}

fn unchanged_after_discard<T: PublicationTransactionV1>(
    transaction: &mut T,
) -> PublicationOutcomeV1 {
    match transaction.discard_stage() {
        Ok(()) => PublicationOutcomeV1::Unchanged,
        Err(failure) => failed_publication(failure),
    }
}

fn failed_publication(failure: FailureClassV1) -> PublicationOutcomeV1 {
    PublicationOutcomeV1::Failed {
        failure: FailureV1::new(failure),
        replacement: ReplacementStateV1::No,
    }
}

fn indeterminate_publication(failure: FailureClassV1) -> PublicationOutcomeV1 {
    PublicationOutcomeV1::Indeterminate {
        failure: FailureV1::new(failure),
        replacement: ReplacementStateV1::Maybe,
        durability: DurabilityStateV1::Unknown,
    }
}
