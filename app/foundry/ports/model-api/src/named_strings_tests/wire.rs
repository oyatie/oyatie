use super::*;

#[test]
fn document_codec_retains_exact_legacy_shape_and_refuses_structural_unknowns_duplicate_fields_and_wrong_widths()
 {
    let empty = ModelDocument {
        entities: vec![],
        links: vec![],
        named_string_types: vec![],
    };
    assert_eq!(empty.encode().unwrap(), br#"{"entities":[],"links":[]}"#);
    for bytes in [
        r#"[[],[]]"#,
        r#"{"entities":[],"links":[],"named_string_types":null}"#,
        r#"{"entities":[],"links":[],"unknown":[]}"#,
        r#"{"entities":[],"entities":[],"links":[]}"#,
        r#"{"links":[]}"#,
        r#"{"entities":[],"links":[],"named_string_types":[{"id":"vty_test","revision":4294967296,"display_name":"Test","members":[] }]}"#,
        r#"{"entities":[],"links":[],"named_string_types":[["vty_test",1,"Test",[]]]}"#,
        r#"{"entities":[],"links":[],"named_string_types":[{"id":"vty_test","revision":1,"display_name":"Test","members":[],"tenant":"ten_foreign"}]}"#,
    ] {
        assert_eq!(
            ModelDocument::decode(bytes.as_bytes()).unwrap_err(),
            ModelError::MalformedDocument,
            "{bytes}"
        );
    }
    let encoded = named_string_model().encode().unwrap();
    assert_eq!(
        ModelDocument::decode(&encoded).unwrap(),
        named_string_model()
    );
    let mut bad: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    bad["entities"][0]["properties"][2]["value_type"]["tenant_id"] = "ten_foreign".into();
    assert!(ModelDocument::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
}
