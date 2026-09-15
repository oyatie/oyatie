use messenger_domain::{
    AdmitCommand, ban_allowed, join_allowed, member_can_leave, member_can_send, send_endpoint,
    valid_room, valid_txn, valid_user, validate_admit,
};
use serde_json::json;

fn command() -> AdmitCommand {
    AdmitCommand {
        room: "!room:messenger.test".into(),
        sender: "@alice:messenger.test".into(),
        device: "DEVICE".into(),
        event_type: "m.room.message".into(),
        state_key: None,
        content: json!({"body":"hello","msgtype":"m.text"}),
        txn: Some("txn".into()),
        endpoint: send_endpoint("!room:messenger.test", "m.room.message"),
    }
}

#[test]
fn membership_rules_match_joined_send_and_closed_bans() {
    assert!(member_can_send("join"));
    assert!(!member_can_send("leave"));
    assert!(!member_can_send("ban"));
    assert!(member_can_leave("join"));
    assert!(member_can_leave("invite"));
    assert!(!member_can_leave("ban"));
    assert!(join_allowed(None, "public").is_ok());
    assert!(join_allowed(Some("invite"), "invite").is_ok());
    assert!(join_allowed(None, "invite").is_err());
    assert!(join_allowed(Some("ban"), "public").is_err());
    assert!(ban_allowed(true, false).is_ok());
    assert!(ban_allowed(false, false).is_err());
    assert!(ban_allowed(true, true).is_err());
}

#[test]
fn admission_identity_rejects_malformed_ids_and_oversize_content() {
    assert!(valid_user("@alice:messenger.test"));
    assert!(!valid_user("alice"));
    assert!(valid_room("!room:messenger.test"));
    assert!(!valid_room("#room:messenger.test"));
    assert!(valid_txn("txn-1"));
    assert!(!valid_txn(""));
    assert!(validate_admit(&command()).is_ok());
    let mut bad = command();
    bad.sender = "alice".into();
    assert!(validate_admit(&bad).is_err());
    let mut huge = command();
    huge.content = json!({"body":"x".repeat(70_000)});
    assert!(validate_admit(&huge).is_err());
}
