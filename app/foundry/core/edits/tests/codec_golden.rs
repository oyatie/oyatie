//! Golden byte vectors freezing wire_format_version 1, hand-derived
//! byte-by-byte and independent of the encoder — plus the refusal family
//! proving no non-canonical byte string decodes at all.

use foundry_edits::{
    ActionRecord, DecodeError, DenialRecord, EditSet, OntologyEdit, WireDataClass, WireProperty,
    WireTier, WireValue, decode_action_record, decode_denial_record, encode_action_record,
    encode_denial_record,
};

fn str_bytes(s: &str) -> Vec<u8> {
    let mut out = (s.len() as u32).to_le_bytes().to_vec();
    out.extend(s.as_bytes());
    out
}

fn minimal_action() -> ActionRecord {
    let property = WireProperty::new(
        "n",
        WireTier::Scalar,
        WireDataClass::InternalOnly,
        WireValue::String("v".into()),
    )
    .unwrap();
    ActionRecord::new(
        "p",
        "d",
        "e",
        "k",
        1,
        vec![],
        EditSet::new(vec![
            OntologyEdit::upsert_properties(vec![property]).unwrap(),
        ])
        .unwrap(),
    )
    .unwrap()
}

/// The layout of version 1, spelled out literally. Any change to these
/// bytes is a wire-format break, not a refactor.
fn minimal_action_golden() -> Vec<u8> {
    let mut golden = vec![0x01, 0x00]; // wire_format_version 1, u16 LE
    golden.extend(str_bytes("p")); // principal_id
    golden.extend(str_bytes("d")); // decision_id
    golden.extend(str_bytes("e")); // audit_event_type
    golden.extend(str_bytes("k")); // idempotency_key
    golden.extend(1u64.to_le_bytes()); // occurred_at_epoch_ms
    golden.extend(0u32.to_le_bytes()); // parameter count
    golden.extend(1u32.to_le_bytes()); // edit count
    golden.push(0x01); // edit tag: UpsertProperties
    golden.extend(1u32.to_le_bytes()); // property count
    golden.extend(str_bytes("n")); // property name
    golden.push(0x00); // tier tag: Scalar
    golden.push(0x01); // data-class tag: INTERNAL_ONLY
    golden.push(0x00); // value tag: String
    golden.extend(str_bytes("v")); // value
    golden
}

#[test]
fn action_record_golden_vector_is_frozen() {
    assert_eq!(
        encode_action_record(&minimal_action()),
        minimal_action_golden()
    );
    assert_eq!(
        decode_action_record(&minimal_action_golden()).unwrap(),
        minimal_action()
    );
}

#[test]
fn denial_record_golden_vector_is_frozen() {
    let record = DenialRecord::new("g", "c", "p", "d", "a", "o", 2).unwrap();
    let mut golden = vec![0x01, 0x00];
    for field in ["g", "c", "p", "d", "a", "o"] {
        golden.extend(str_bytes(field));
    }
    golden.extend(2u64.to_le_bytes());
    assert_eq!(encode_denial_record(&record), golden);
    assert_eq!(decode_denial_record(&golden).unwrap(), record);
}

#[test]
fn unsupported_version_is_refused() {
    let mut bytes = minimal_action_golden();
    bytes[0] = 0x02;
    assert_eq!(
        decode_action_record(&bytes),
        Err(DecodeError::UnsupportedWireVersion { found: 2 })
    );
}

#[test]
fn truncation_at_every_boundary_is_refused() {
    let golden = minimal_action_golden();
    for cut in 0..golden.len() {
        assert_eq!(
            decode_action_record(&golden[..cut]),
            Err(DecodeError::Truncated),
            "prefix of {cut} bytes must refuse as truncated",
        );
    }
}

#[test]
fn trailing_bytes_are_refused() {
    let mut bytes = minimal_action_golden();
    bytes.push(0x00);
    assert_eq!(
        decode_action_record(&bytes),
        Err(DecodeError::TrailingBytes)
    );
}

#[test]
fn reserved_and_unknown_edit_tags_are_distinct_refusals() {
    for (tag, expected) in [
        (0x02, DecodeError::ReservedEditKind { tag: 2 }),
        (0x03, DecodeError::ReservedEditKind { tag: 3 }),
        (0x05, DecodeError::ReservedEditKind { tag: 5 }),
        (0x06, DecodeError::UnknownEditTag { tag: 6 }),
    ] {
        let mut bytes = minimal_action_golden();
        let edit_tag_at = bytes.len() - str_bytes("v").len() - 3 - str_bytes("n").len() - 4 - 1;
        assert_eq!(bytes[edit_tag_at], 0x01, "self-check: found the edit tag");
        bytes[edit_tag_at] = tag;
        assert_eq!(decode_action_record(&bytes), Err(expected));
    }
}

#[test]
fn invalid_boolean_byte_is_refused() {
    let mut bytes = minimal_action_golden();
    let value_tag_at = bytes.len() - str_bytes("v").len() - 1;
    assert_eq!(bytes[value_tag_at], 0x00, "self-check: found the value tag");
    // Rewrite the tail: boolean tag + byte 2, dropping the string payload.
    bytes.truncate(value_tag_at);
    bytes.push(0x03);
    bytes.push(0x02);
    assert_eq!(
        decode_action_record(&bytes),
        Err(DecodeError::InvalidBoolean { byte: 2 })
    );
}

#[test]
fn blank_decoded_identity_fields_fail_closed() {
    // A structurally valid byte string whose principal_id is empty must
    // refuse through the validating constructor, not materialize.
    let mut bytes = vec![0x01, 0x00];
    bytes.extend(str_bytes("")); // principal_id: blank
    bytes.extend(str_bytes("d"));
    bytes.extend(str_bytes("e"));
    bytes.extend(str_bytes("k"));
    bytes.extend(1u64.to_le_bytes());
    bytes.extend(0u32.to_le_bytes()); // no parameters
    bytes.extend(1u32.to_le_bytes()); // one edit, itself valid
    bytes.push(0x01); // UpsertProperties
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(str_bytes("n"));
    bytes.push(0x00);
    bytes.push(0x01);
    bytes.push(0x00);
    bytes.extend(str_bytes("v"));
    assert!(matches!(
        decode_action_record(&bytes),
        Err(DecodeError::InvalidRecord(_))
    ));
}
