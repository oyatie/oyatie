use super::*;

#[test]
fn handles_are_bounded_and_unambiguously_bound_to_canonical_accounts() {
    let long_user = format!("usr_{}", "a".repeat(100));
    assert_eq!(user_handle("ten_acme", &long_user).len(), 32);
    assert_eq!(
        user_handle("ten_acme", &long_user),
        user_handle("ten_acme", &long_user)
    );
    assert_ne!(
        user_handle("ten_acme", &long_user),
        user_handle("ten_other", &long_user)
    );
    assert_ne!(
        user_handle("ten_a/usr_b", "usr_c"),
        user_handle("ten_a", "usr_b/usr_c")
    );
}

#[test]
fn attestation_rejects_duplicate_keys_wrong_types_and_trailing_bytes() {
    let valid = vec![
        (
            CborValue::Text("fmt".into()),
            CborValue::Text("none".into()),
        ),
        (CborValue::Text("attStmt".into()), CborValue::Map(vec![])),
        (
            CborValue::Text("authData".into()),
            CborValue::Bytes(vec![0; 37]),
        ),
    ];
    let encode = |fields: Vec<(CborValue, CborValue)>, trailing: bool| {
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(&CborValue::Map(fields), &mut bytes).expect("CBOR");
        if trailing {
            bytes.push(0);
        }
        URL_SAFE_NO_PAD.encode(bytes)
    };
    assert!(checked_none_attestation(&encode(valid.clone(), false)).is_ok());
    for key in ["fmt", "attStmt", "authData"] {
        let mut fields = valid.clone();
        fields.push((
            CborValue::Text(key.into()),
            CborValue::Text("duplicate".into()),
        ));
        assert!(checked_none_attestation(&encode(fields, false)).is_err());
    }
    let mut wrong = valid.clone();
    wrong[2].1 = CborValue::Text("not bytes".into());
    assert!(checked_none_attestation(&encode(wrong, false)).is_err());
    assert!(checked_none_attestation(&encode(valid, true)).is_err());
}

#[test]
fn origin_and_session_token_are_exact() {
    assert!(PasskeyOrigin::parse("https://identity.example.com").is_ok());
    assert!(PasskeyOrigin::parse("http://localhost:8080").is_ok());
    for value in [
        "http://identity.example.com",
        "https://identity.example.com/path",
        "https://identity.example.com?x=1",
    ] {
        assert!(PasskeyOrigin::parse(value).is_err());
    }
    let token = new_session_token().expect("OS randomness");
    assert!(session_hash(&token).is_some());
    assert!(session_hash("v1.bad").is_none());
}

#[test]
fn client_data_rejects_cross_origin_metadata() {
    let good = URL_SAFE_NO_PAD.encode(br#"{"crossOrigin":false}"#);
    assert!(checked_client_data(&good).is_ok());
    for raw in [
        br#"{"crossOrigin":true}"#.as_slice(),
        br#"{"topOrigin":"https://evil.test"}"#.as_slice(),
    ] {
        assert!(checked_client_data(&URL_SAFE_NO_PAD.encode(raw)).is_err());
    }
}
