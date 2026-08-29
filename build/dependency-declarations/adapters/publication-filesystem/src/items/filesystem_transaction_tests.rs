#[cfg(test)]
mod filesystem_transaction_tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Step {
        StageWrite,
        StageSync,
        LeaseCheck,
        RecheckRead,
        Replace,
        DirectorySync,
        Discard,
    }

    struct MemoryTransaction {
        current: Option<Vec<u8>>,
        staged: Option<Vec<u8>>,
        failures: Vec<Step>,
        reads: usize,
        calls: Vec<&'static str>,
    }

    impl MemoryTransaction {
        fn new(current: Option<&[u8]>, failures: &[Step]) -> Self {
            Self {
                current: current.map(<[u8]>::to_vec),
                staged: None,
                failures: failures.to_vec(),
                reads: 0,
                calls: Vec::new(),
            }
        }

        fn fails(&self, step: Step) -> bool {
            self.failures.contains(&step)
        }
    }

    impl PublicationTransactionV1 for MemoryTransaction {
        fn read_destination(&mut self) -> Result<Option<Vec<u8>>, FailureClassV1> {
            self.calls.push("read");
            self.reads += 1;
            if self.reads == 2 && self.fails(Step::RecheckRead) {
                return Err(FailureClassV1::DestinationConflict);
            }
            Ok(self.current.clone())
        }

        fn write_stage(&mut self, bytes: &[u8]) -> Result<(), FailureClassV1> {
            self.calls.push("write");
            self.staged = Some(bytes.to_vec());
            if self.fails(Step::StageWrite) {
                Err(FailureClassV1::StageWriteFailed)
            } else {
                Ok(())
            }
        }

        fn sync_stage(&mut self) -> Result<(), FailureClassV1> {
            self.calls.push("stage-sync");
            if self.fails(Step::StageSync) {
                Err(FailureClassV1::StageSyncFailed)
            } else {
                Ok(())
            }
        }

        fn ensure_lease(&mut self) -> Result<(), FailureClassV1> {
            self.calls.push("lease");
            if self.fails(Step::LeaseCheck) {
                Err(FailureClassV1::LeaseLost)
            } else {
                Ok(())
            }
        }

        fn replace(&mut self) -> Result<(), FailureClassV1> {
            self.calls.push("replace");
            if self.fails(Step::Replace) {
                return Err(FailureClassV1::ReplaceFailed);
            }
            self.current = self.staged.take();
            Ok(())
        }

        fn sync_directory(&mut self) -> Result<(), FailureClassV1> {
            self.calls.push("directory-sync");
            if self.fails(Step::DirectorySync) {
                Err(FailureClassV1::DirectorySyncFailed)
            } else {
                Ok(())
            }
        }

        fn discard_stage(&mut self) -> Result<(), FailureClassV1> {
            self.calls.push("discard");
            if self.fails(Step::Discard) {
                return Err(FailureClassV1::StageCleanupFailed);
            }
            self.staged = None;
            Ok(())
        }
    }

    #[test]
    fn transaction_rechecks_preimage_then_replaces_and_syncs() {
        let old = b"old";
        let mut transaction = MemoryTransaction::new(Some(old), &[]);
        let outcome = run_publication_transaction(
            &mut transaction,
            Some(DigestV1::of(old)),
            b"new",
        );

        assert_eq!(outcome, PublicationOutcomeV1::Replaced);
        assert_eq!(transaction.current.as_deref(), Some(b"new".as_slice()));
        assert_eq!(
            transaction.calls,
            [
                "read",
                "write",
                "stage-sync",
                "lease",
                "read",
                "replace",
                "directory-sync",
            ]
        );
    }

    #[test]
    fn unchanged_and_conflicting_preimages_never_stage() {
        let current = b"current";
        let mut unchanged = MemoryTransaction::new(Some(current), &[]);
        assert_eq!(
            run_publication_transaction(
                &mut unchanged,
                Some(DigestV1::of(current)),
                current,
            ),
            PublicationOutcomeV1::Unchanged
        );
        assert_eq!(unchanged.calls, ["read", "discard"]);

        let mut conflict = MemoryTransaction::new(Some(current), &[]);
        assert_failed(
            run_publication_transaction(
                &mut conflict,
                Some(DigestV1::of(b"different")),
                b"new",
            ),
            FailureClassV1::DestinationConflict,
        );
        assert_eq!(conflict.calls, ["read"]);
    }

    #[test]
    fn every_pre_replace_failure_preserves_destination_and_discards_stage() {
        let cases = [
            (Step::StageWrite, FailureClassV1::StageWriteFailed),
            (Step::StageSync, FailureClassV1::StageSyncFailed),
            (Step::LeaseCheck, FailureClassV1::LeaseLost),
            (Step::RecheckRead, FailureClassV1::DestinationConflict),
            (Step::Replace, FailureClassV1::ReplaceFailed),
        ];
        for (step, expected) in cases {
            let mut transaction = MemoryTransaction::new(Some(b"old"), &[step]);
            assert_failed(
                run_publication_transaction(
                    &mut transaction,
                    Some(DigestV1::of(b"old")),
                    b"new",
                ),
                expected,
            );
            assert_eq!(transaction.current.as_deref(), Some(b"old".as_slice()));
            assert!(transaction.staged.is_none());
            assert!(!transaction.calls.contains(&"directory-sync"));
        }
    }

    #[test]
    fn cleanup_failure_is_explicit_without_claiming_replacement() {
        let mut transaction =
            MemoryTransaction::new(Some(b"old"), &[Step::StageSync, Step::Discard]);
        assert_failed(
            run_publication_transaction(
                &mut transaction,
                Some(DigestV1::of(b"old")),
                b"new",
            ),
            FailureClassV1::StageCleanupFailed,
        );
        assert_eq!(transaction.current.as_deref(), Some(b"old".as_slice()));
        assert_eq!(transaction.staged.as_deref(), Some(b"new".as_slice()));
    }

    #[test]
    fn directory_sync_failure_is_indeterminate_after_replacement() {
        let mut transaction = MemoryTransaction::new(Some(b"old"), &[Step::DirectorySync]);
        let outcome = run_publication_transaction(
            &mut transaction,
            Some(DigestV1::of(b"old")),
            b"new",
        );
        assert!(matches!(
            outcome,
            PublicationOutcomeV1::Indeterminate {
                failure,
                replacement: ReplacementStateV1::Maybe,
                durability: DurabilityStateV1::Unknown,
            } if failure.class() == FailureClassV1::DirectorySyncFailed
        ));
        assert_eq!(transaction.current.as_deref(), Some(b"new".as_slice()));
    }

    fn assert_failed(outcome: PublicationOutcomeV1, expected: FailureClassV1) {
        assert!(matches!(
            outcome,
            PublicationOutcomeV1::Failed {
                failure,
                replacement: ReplacementStateV1::No,
            } if failure.class() == expected
        ));
    }
}
