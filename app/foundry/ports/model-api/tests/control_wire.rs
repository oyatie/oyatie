use foundry_model_api::{conformance::*, control::*, *};
use serde::{Serialize, de::DeserializeOwned};

#[path = "control_wire/lifecycle.rs"]
mod lifecycle;
#[path = "control_wire/maximum.rs"]
mod maximum;
use maximum::maximum_document;

fn number(value: u64) -> ModelNumber {
    ModelNumber::new(value).unwrap()
}

fn assert_closed<T: Serialize + DeserializeOwned>(row: &T) {
    let value = serde_json::to_value(row).unwrap();
    let object = value.as_object().unwrap();
    let bytes = serde_json::to_string(row).unwrap();
    assert!(serde_json::from_str::<T>("[]").is_err());
    let unknown = format!("{{\"unexpected\":null,{}", &bytes[1..]);
    assert!(serde_json::from_str::<T>(&unknown).is_err());
    for (field, member) in object {
        let duplicate = format!(
            "{{{}:{},{}",
            serde_json::to_string(field).unwrap(),
            member,
            &bytes[1..]
        );
        assert!(
            serde_json::from_str::<T>(&duplicate).is_err(),
            "duplicate {field}"
        );
        let mut missing = object.clone();
        missing.remove(field);
        assert!(
            serde_json::from_value::<T>(serde_json::Value::Object(missing)).is_err(),
            "missing {field}"
        );
    }
}

#[test]
fn decimal_strings_keep_integer_identity_across_browser_precision_boundary() {
    for value in [
        1,
        9_007_199_254_740_991,
        9_007_199_254_740_992,
        9_007_199_254_740_993,
        MAX_MODEL_VERSION,
    ] {
        let row = number(value);
        let encoded = serde_json::to_string(&row).unwrap();
        assert_eq!(encoded, format!("\"{value}\""));
        assert_eq!(
            serde_json::from_str::<ModelNumber>(&encoded).unwrap().get(),
            value
        );
        assert_eq!(ModelNumber::parse(&value.to_string()).unwrap(), row);
        assert!(serde_json::from_value::<ModelNumber>(serde_json::json!(value)).is_err());
    }
    for value in [
        "",
        "0",
        "01",
        "+1",
        "-1",
        " 1",
        "1 ",
        "1e3",
        "1.0",
        "1\n",
        "9223372036854775808",
        "18446744073709551615",
        "١",
        "１",
    ] {
        assert!(ModelNumber::parse(value).is_err(), "{value:?}");
        assert!(serde_json::from_value::<ModelNumber>(serde_json::json!(value)).is_err());
    }
    for value in [0, MAX_MODEL_VERSION + 1, u64::MAX] {
        assert!(ModelNumber::new(value).is_err());
    }
}

#[test]
fn every_control_object_refuses_missing_duplicate_unknown_or_positional_fields() {
    let capabilities = ModelCapabilities {
        author: false,
        publish: true,
    };
    let workspace = ModelWorkspace {
        generation: number(1),
        document: baseline().document,
        capabilities: capabilities.clone(),
    };
    let summary = ModelDraftSummary {
        id: number(1),
        version: number(1),
        base_generation: number(1),
        published_generation: None,
    };
    let page = ModelDraftPage {
        drafts: vec![summary.clone()],
        next: None,
    };
    let draft = ModelDraftWire {
        id: number(1),
        tenant_id: "ten_model".into(),
        owner: "usr_author".into(),
        base_generation: number(1),
        version: number(1),
        document: baseline().document,
        publication: None,
    };
    let receipt = ModelReceiptWire {
        generation: number(2),
        prior_generation: number(1),
        draft_id: number(1),
        draft_version: number(1),
        owner: "usr_author".into(),
    };
    let create = CreateModelDraft {
        expected_generation: number(1),
        document: baseline().document,
    };
    let save = SaveModelDraft {
        expected_version: number(1),
        expected_base: number(1),
        document: baseline().document,
    };
    let prepare = PrepareModelDraft {
        expected_version: number(1),
        expected_base: number(1),
    };
    let diagnostic = ModelDiagnostic {
        code: "native_invalid".into(),
        path: "entities[0]".into(),
        message: "Correct this declaration.".into(),
    };
    let validation = ModelValidation {
        draft_id: number(1),
        draft_version: number(1),
        base_generation: number(1),
        valid: false,
        diagnostics: vec![diagnostic.clone()],
    };
    assert_closed(&capabilities);
    assert_closed(&workspace);
    assert_closed(&summary);
    assert_closed(&page);
    assert_closed(&draft);
    assert_closed(&receipt);
    assert_closed(&create);
    assert_closed(&save);
    assert_closed(&prepare);
    assert_closed(&diagnostic);
    assert_closed(&validation);
    assert_closed(&ModelRefusal {
        error: ModelRefusalCode::Forbidden,
    });
    let bytes = draft.encode().unwrap();
    assert_eq!(ModelDraftWire::decode(&bytes).unwrap(), draft);
    let bytes = page.encode().unwrap();
    assert_eq!(ModelDraftPage::decode(&bytes).unwrap(), page);
}

#[test]
fn full_document_budget_survives_escaped_closed_draft_and_each_envelope() {
    let limits = model_wire_limits().unwrap();
    let document = maximum_document();
    let number = number(MAX_MODEL_VERSION);
    let owner = "\0".repeat(MAX_MODEL_CONTEXT_BYTES);
    let receipt = ModelReceiptWire {
        generation: number,
        prior_generation: ModelNumber::new(MAX_MODEL_VERSION - 1).unwrap(),
        draft_id: number,
        draft_version: number,
        owner: owner.clone(),
    };
    let draft = ModelDraftWire {
        id: number,
        tenant_id: format!("ten_{}", "\0".repeat(MAX_MODEL_CONTEXT_BYTES - 4)),
        owner,
        base_generation: receipt.prior_generation,
        version: number,
        document: document.clone(),
        publication: Some(receipt),
    };
    let bytes = draft.encode().unwrap();
    assert!(bytes.len() > MAX_MODEL_BYTES + 4096);
    assert!(bytes.len() <= limits.draft);
    assert_eq!(ModelDraftWire::decode(&bytes).unwrap(), draft);
    let create = CreateModelDraft {
        expected_generation: number,
        document: document.clone(),
    };
    let save = SaveModelDraft {
        expected_version: number,
        expected_base: number,
        document: document.clone(),
    };
    let workspace = ModelWorkspace {
        generation: number,
        document: document.clone(),
        capabilities: ModelCapabilities {
            author: false,
            publish: false,
        },
    };
    assert_eq!(create.encode().unwrap().len(), limits.create);
    assert_eq!(save.encode().unwrap().len(), limits.save);
    assert_eq!(workspace.encode().unwrap().len(), limits.workspace);
    assert_eq!(
        CreateModelDraft::decode(&create.encode().unwrap()).unwrap(),
        create
    );
    assert_eq!(
        SaveModelDraft::decode(&save.encode().unwrap()).unwrap(),
        save
    );
    assert_eq!(
        ModelWorkspace::decode(&workspace.encode().unwrap()).unwrap(),
        workspace
    );
    let mut too_large = create.clone();
    too_large.document.entities[0].display_name.push('x');
    assert!(too_large.encode().is_err());
    assert!(CreateModelDraft::decode(&serde_json::to_vec(&too_large).unwrap()).is_err());
    assert!(CreateModelDraft::decode(&vec![b' '; limits.create + 1]).is_err());
    assert!(SaveModelDraft::decode(&vec![b' '; limits.save + 1]).is_err());
    assert!(ModelWorkspace::decode(&vec![b' '; limits.workspace + 1]).is_err());
    assert!(ModelDraftWire::decode(&vec![b' '; limits.draft + 1]).is_err());
}

#[test]
fn page_validation_and_receipt_metadata_refuse_incoherent_or_oversized_wire() {
    let summary = ModelDraftSummary {
        id: number(1),
        version: number(1),
        base_generation: number(1),
        published_generation: None,
    };
    let mut page = ModelDraftPage {
        drafts: vec![summary.clone()],
        next: Some(number(1)),
    };
    assert!(page.encode().is_ok());
    page.next = Some(number(2));
    assert!(page.encode().is_err());
    page.next = None;
    page.drafts.push(summary);
    assert!(page.encode().is_err());
    page.drafts.truncate(1);
    page.drafts[0].published_generation = Some(number(1));
    assert!(page.encode().is_err());
    page.drafts[0].published_generation = None;
    page.drafts = vec![page.drafts[0].clone(); MAX_MODEL_ITEMS + 1];
    assert!(page.encode().is_err());
    let mut validation = ModelValidation {
        draft_id: number(1),
        draft_version: number(1),
        base_generation: number(1),
        valid: true,
        diagnostics: vec![],
    };
    assert!(validation.encode().is_ok());
    validation.valid = false;
    assert!(validation.encode().is_err());
    validation.diagnostics.push(ModelDiagnostic {
        code: "native_invalid".into(),
        path: "entities[0]".into(),
        message: "Correct this declaration.".into(),
    });
    assert!(validation.encode().is_ok());
    validation.diagnostics[0].path = "x".repeat(MAX_MODEL_DIAGNOSTIC_PATH_BYTES + 1);
    assert!(validation.encode().is_err());
}
