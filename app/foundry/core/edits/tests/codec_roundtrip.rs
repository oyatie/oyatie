//! Round-trip and determinism pins over the full value vocabulary, and
//! the canonical-struct / canonical-double refusals that byte-equality
//! dedup depends on.

use std::collections::BTreeMap;

use foundry_edits::{
    ActionRecord, DecodeError, EditSet, OntologyEdit, WireDataClass, WireDate, WireDouble,
    WireProperty, WireTier, WireValue, decode_action_record, encode_action_record,
};

fn prop(name: &str, tier: WireTier, value: WireValue) -> WireProperty {
    WireProperty::new(name, tier, WireDataClass::PiiIdentifying, value).unwrap()
}

/// Every WireValue variant, nested struct-in-array-in-struct included.
fn kitchen_sink() -> ActionRecord {
    let mut inner = BTreeMap::new();
    inner.insert("count".to_string(), WireValue::Integer(-7));
    inner.insert(
        "ratio".to_string(),
        WireValue::Double(WireDouble::new(-1000.25).unwrap()),
    );
    let mut outer = BTreeMap::new();
    outer.insert(
        "metrics".to_string(),
        WireValue::Array(vec![WireValue::Struct(inner)]),
    );
    outer.insert("open".to_string(), WireValue::Boolean(true));
    outer.insert(
        "since".to_string(),
        WireValue::Date(WireDate::new(2024, 2, 29).unwrap()),
    );
    let parameters = vec![
        prop("name", WireTier::Scalar, WireValue::String("Ada".into())),
        prop(
            "samples",
            WireTier::Vector,
            WireValue::Array(vec![
                WireValue::Timestamp { epoch_millis: -1 },
                WireValue::Timestamp {
                    epoch_millis: i64::MAX,
                },
            ]),
        ),
        prop("config", WireTier::Struct, WireValue::Struct(outer)),
    ];
    let edits = EditSet::new(vec![
        OntologyEdit::create_object(
            "ety_reading",
            vec![prop("seed", WireTier::Scalar, WireValue::Integer(i64::MIN))],
        )
        .unwrap(),
        OntologyEdit::create_link("lty_measures", "ent_target_1").unwrap(),
    ])
    .unwrap();
    ActionRecord::new(
        "prn_alice",
        "dec_0001",
        "reading.calibrated",
        "idem_2718",
        1_700_000_000_000,
        parameters,
        edits,
    )
    .unwrap()
}

#[test]
fn round_trip_preserves_every_variant() {
    let record = kitchen_sink();
    let bytes = encode_action_record(&record);
    assert_eq!(decode_action_record(&bytes).unwrap(), record);
}

#[test]
fn encoding_is_deterministic() {
    assert_eq!(
        encode_action_record(&kitchen_sink()),
        encode_action_record(&kitchen_sink())
    );
}

fn single_value_record_bytes(build: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut bytes = vec![0x01, 0x00];
    for field in ["p", "d", "e", "k"] {
        bytes.extend((field.len() as u32).to_le_bytes());
        bytes.extend(field.as_bytes());
    }
    bytes.extend(1u64.to_le_bytes());
    bytes.extend(0u32.to_le_bytes()); // no parameters
    bytes.extend(1u32.to_le_bytes()); // one edit
    bytes.push(0x01); // UpsertProperties
    bytes.extend(1u32.to_le_bytes()); // one property
    bytes.extend(1u32.to_le_bytes());
    bytes.push(b'n');
    bytes.push(0x00); // Scalar
    bytes.push(0x01); // INTERNAL_ONLY
    build(&mut bytes); // the value under test
    bytes
}

fn key_string(out: &mut Vec<u8>, s: &str) {
    out.extend((s.len() as u32).to_le_bytes());
    out.extend(s.as_bytes());
}

#[test]
fn negative_zero_double_key_is_refused() {
    // The key !(-0.0f64).to_bits() is finite but never constructible —
    // construction folds -0.0 into the +0.0 key. A second spelling of
    // zero would break byte-equality dedup.
    let folded = WireDouble::new(-0.0).unwrap();
    let noncanonical_key = !(-0.0f64).to_bits();
    assert_ne!(folded.sort_key(), noncanonical_key);
    assert_eq!(
        WireDouble::from_sort_key(folded.sort_key()).unwrap(),
        folded
    );
    assert!(WireDouble::from_sort_key(noncanonical_key).is_err());
    assert!(WireDouble::from_sort_key(f64::NAN.to_bits() ^ (1 << 63)).is_err());

    let bytes = single_value_record_bytes(|out| {
        out.push(0x02);
        out.extend(noncanonical_key.to_le_bytes());
    });
    assert_eq!(
        decode_action_record(&bytes),
        Err(DecodeError::NonCanonicalDouble)
    );
}

#[test]
fn unsorted_struct_keys_are_refused() {
    let bytes = single_value_record_bytes(|out| {
        out.push(0x07);
        out.extend(2u32.to_le_bytes());
        key_string(out, "b");
        out.push(0x01);
        out.extend(1i64.to_le_bytes());
        key_string(out, "a"); // descends: non-canonical
        out.push(0x01);
        out.extend(2i64.to_le_bytes());
    });
    assert_eq!(
        decode_action_record(&bytes),
        Err(DecodeError::NonCanonicalStructOrder)
    );
}

#[test]
fn duplicate_struct_keys_are_refused() {
    let bytes = single_value_record_bytes(|out| {
        out.push(0x07);
        out.extend(2u32.to_le_bytes());
        for _ in 0..2 {
            key_string(out, "a");
            out.push(0x01);
            out.extend(1i64.to_le_bytes());
        }
    });
    assert_eq!(
        decode_action_record(&bytes),
        Err(DecodeError::DuplicateStructKey)
    );
}

#[test]
fn hostile_length_prefix_is_refused_before_allocation() {
    let bytes = single_value_record_bytes(|out| {
        out.push(0x00); // String value
        out.extend(u32::MAX.to_le_bytes()); // hostile length
        out.push(b'v');
    });
    assert_eq!(decode_action_record(&bytes), Err(DecodeError::Truncated));
}

#[test]
fn unknown_value_and_tag_bytes_are_refused() {
    let bytes = single_value_record_bytes(|out| out.push(0x08));
    assert_eq!(
        decode_action_record(&bytes),
        Err(DecodeError::UnknownValueTag { tag: 8 })
    );
    let bytes = single_value_record_bytes(|out| {
        out.push(0x04); // Date
        out.extend(2023i32.to_le_bytes());
        out.push(2);
        out.push(29); // not a leap year
    });
    assert_eq!(decode_action_record(&bytes), Err(DecodeError::InvalidDate));
}
