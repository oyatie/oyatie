use super::support::*;
use mail_api::{Store, SubmissionQueue};
use mail_kernel::Command;
use mail_service::OwnerPolicy;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn successful_submission_applies_exact_mailbox_and_draft_patch_with_a_separate_receipt() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    let account = f.db.account("a").unwrap();
    let account =
        f.db.execute(
            "a",
            account.revision,
            vec![Command::CreateMailbox {
                name: "Sent".into(),
            }],
        )
        .unwrap();
    let sent = account
        .mailboxes
        .iter()
        .find(|m| m.name == "Sent")
        .unwrap()
        .id
        .clone();
    let response = f.call("EmailSubmission/set", json!({"accountId":"a","create":{"s":{"identityId":"a","emailId":"e1"}},"onSuccessUpdateEmail":{"#s":{"mailboxIds":{&sent:true},"keywords/$draft":null}}})).await;
    id(&response);
    let calls = response["methodResponses"].as_array().unwrap();
    assert_eq!(calls.len(), 2, "{response}");
    assert_eq!(calls[1][0], "Email/set");
    assert!(
        calls[1][1]["updated"]
            .as_object()
            .unwrap()
            .contains_key("e1"),
        "{response}"
    );
    let after = f.db.account("a").unwrap();
    assert_eq!(
        after.messages[0].mailboxes.keys().collect::<Vec<_>>(),
        vec![&sent]
    );
    assert_eq!(after.messages[0].keywords, vec!["$seen"]);
    assert_eq!(f.db.blob("a", "e1").unwrap(), RAW);
    assert!(!f.db.claim_outbound(100).unwrap().is_empty());
}

#[tokio::test]
async fn refused_submission_does_not_apply_on_success_updates_or_destruction() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    let before = f.db.account("a").unwrap();
    let response = f.call("EmailSubmission/set", json!({"accountId":"a","create":{"s":{"identityId":"b","emailId":"e1"}},"onSuccessUpdateEmail":{"#s":{"keywords/$draft":null}},"onSuccessDestroyEmail":["#s"]})).await;
    assert!(
        response["methodResponses"][0][1]["notCreated"]["s"]["type"].is_string(),
        "{response}"
    );
    assert_eq!(f.db.account("a").unwrap(), before);
    assert!(f.db.claim_outbound(100).unwrap().is_empty());
}

#[tokio::test]
async fn on_success_destroy_removes_draft_but_retains_accepted_delivery_payloads() {
    let f = Fixture::new(":memory:", RAW, Arc::new(OwnerPolicy));
    let response = f.call("EmailSubmission/set", json!({"accountId":"a","create":{"s":{"identityId":"a","emailId":"e1"}},"onSuccessDestroyEmail":["#s"]})).await;
    id(&response);
    assert_eq!(response["methodResponses"][1][0], "Email/set");
    assert_eq!(
        response["methodResponses"][1][1]["destroyed"],
        json!(["e1"]),
        "{response}"
    );
    assert!(f.db.account("a").unwrap().messages.is_empty());
    let leases = f.db.claim_outbound(100).unwrap();
    assert!(!leases.is_empty());
    for lease in leases {
        assert!(
            f.db.outbound_message(&lease)
                .unwrap()
                .raw
                .ends_with(b"body\r\n")
        );
    }
}

struct NoMailboxWrite;
impl mail_api::Policy for NoMailboxWrite {
    fn authorize(
        &self,
        principal: &mail_api::Principal,
        action: mail_api::Action,
        account: &mail_api::AccountInfo,
    ) -> Result<(), mail_kernel::Error> {
        mail_api::Policy::authorize(&OwnerPolicy, principal, action, account)?;
        if action == mail_api::Action::Write {
            Err(mail_kernel::Error::Forbidden)
        } else {
            Ok(())
        }
    }
}
#[tokio::test]
async fn on_success_write_denial_keeps_acceptance_receipt_and_original_draft() {
    let f = Fixture::new(":memory:", RAW, Arc::new(NoMailboxWrite));
    let before = f.db.account("a").unwrap();
    let response = f.call("EmailSubmission/set", json!({"accountId":"a","create":{"s":{"identityId":"a","emailId":"e1"}},"onSuccessUpdateEmail":{"#s":{"keywords/$draft":null}}})).await;
    id(&response);
    assert_eq!(response["methodResponses"][1][0], "Email/set", "{response}");
    assert_eq!(
        response["methodResponses"][1][1]["notUpdated"]["e1"]["type"], "forbidden",
        "{response}"
    );
    assert_eq!(f.db.account("a").unwrap(), before);
    assert!(!f.db.claim_outbound(100).unwrap().is_empty());
}
