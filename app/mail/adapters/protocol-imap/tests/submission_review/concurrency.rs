use super::support::*;
use mail_api::{DeliveryQueue, SubmissionFailure, SubmissionQueue, SubmissionStore};
use mail_kernel::{Error, UndoStatus};
use mail_service::{OwnerPolicy, SubmitEmail, SubmitEmailError};
use serde_json::Value;
use std::sync::{Arc, Barrier};

#[test]
fn concurrent_acceptance_at_one_revision_cannot_duplicate_jobs_or_history() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    let barrier = Barrier::new(3);
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..2)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    f.service.submit_email(
                        TOKEN,
                        "a",
                        0,
                        SubmitEmail {
                            identity_id: "a".into(),
                            email_id: "e1".into(),
                            envelope: None,
                        },
                    )
                })
            })
            .collect();
        barrier.wait();
        let results: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|r| matches!(r, Err(SubmitEmailError::Storage(Error::Conflict))))
                .count(),
            1
        );
    });
    let selection = f.db.submissions("a", None).unwrap();
    assert_eq!(selection.revision, 1);
    assert_eq!(selection.records.len(), 1);
    assert_eq!(f.db.claim_outbound(100).unwrap().len(), 3);
    assert!(f.db.claim(100).unwrap().is_empty());
}

#[tokio::test]
async fn cancellation_racing_a_delivery_claim_never_reports_success_after_a_claim() {
    for _ in 0..8 {
        let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
        let response = f.create(Value::Null).await;
        let id = id(&response);
        let revision = f.db.submissions("a", None).unwrap().revision;
        let barrier = Barrier::new(3);
        std::thread::scope(|scope| {
            let cancel = scope.spawn(|| {
                barrier.wait();
                f.db.cancel_submission("a", revision, &id)
            });
            let claim = scope.spawn(|| {
                barrier.wait();
                f.db.claim_outbound(100)
            });
            barrier.wait();
            let canceled = cancel.join().unwrap();
            let leases = claim.join().unwrap().unwrap();
            match canceled {
                Ok(result) => {
                    assert!(leases.is_empty());
                    assert_eq!(result.records[0].undo_status, UndoStatus::Canceled);
                }
                Err(SubmissionFailure::CannotUnsend) => assert_eq!(leases.len(), 3),
                Err(error) => panic!("unexpected cancellation failure: {error:?}"),
            }
        });
    }
}
