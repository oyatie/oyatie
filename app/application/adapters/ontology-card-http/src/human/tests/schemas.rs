use application_ontology_read::{OntologyReadError, OntologyReadSource};
use serde_json::json;

use super::support::{cookie, source, stub};

fn wire() -> serde_json::Value {
    json!([{"entity_type":"ety_custom","revision":0,"properties":[{
        "name":"name","tier":"scalar","data_class":"INTERNAL_ONLY","required":true,"value_type":null
    },{
        "name":"tags","tier":"vector","data_class":"PII_QUASI_IDENTIFIER","required":false,
        "value_type":{"kind":"array","element":{"kind":"struct","fields":[{
            "name":"label","required":true,"value_type":{"kind":"string"}
        }]}}
    }],"primary_key_property":"name","title_property":"name"}])
}

#[tokio::test]
async fn declaration_reads_preserve_native_metadata_and_send_only_fresh_human_cookie() {
    let expected = wire();
    let (base, seen) = stub(vec![(200, expected.to_string(), String::new())]);
    let schemas = source(&base, &cookie('A'))
        .schemas("ten_acme")
        .await
        .unwrap();
    assert_eq!(serde_json::to_value(schemas).unwrap(), expected);
    let request = seen.recv().unwrap();
    assert!(request.starts_with("GET /v1/tenants/ten_acme/schemas HTTP/1.1\r\n"));
    assert!(request.contains(&format!("cookie: {}\r\n", cookie('A'))));
    assert!(!request.to_ascii_lowercase().contains("authorization:"));
}

#[tokio::test]
async fn missing_nullable_fields_ambiguous_schemas_and_non_native_metadata_are_unavailable() {
    let mut malformed = Vec::new();
    for field in ["primary_key_property", "title_property"] {
        let mut missing = wire();
        missing[0].as_object_mut().unwrap().remove(field);
        malformed.push(missing);
    }
    let mut missing = wire();
    missing[0]["properties"][0]
        .as_object_mut()
        .unwrap()
        .remove("value_type");
    malformed.push(missing);
    for (field, value) in [
        ("data_class", json!("SECRET")),
        ("tier", json!("scalar")),
        (
            "value_type",
            json!({"kind":"array","element":{"kind":"string","extra":true}}),
        ),
    ] {
        let mut invalid = wire();
        invalid[0]["properties"][1][field] = value;
        malformed.push(invalid);
    }
    let mut duplicate = wire();
    let row = duplicate[0].clone();
    duplicate.as_array_mut().unwrap().push(row);
    malformed.push(duplicate);
    let mut duplicate_property = wire();
    let property = duplicate_property[0]["properties"][0].clone();
    duplicate_property[0]["properties"]
        .as_array_mut()
        .unwrap()
        .push(property);
    malformed.push(duplicate_property);
    let mut designation = wire();
    designation[0]["primary_key_property"] = json!("tags");
    malformed.push(designation);
    for invalid in malformed {
        let (base, _) = stub(vec![(200, invalid.to_string(), String::new())]);
        assert_eq!(
            source(&base, &cookie('A')).schemas("ten_acme").await,
            Err(OntologyReadError::SourceUnavailable)
        );
    }
}
