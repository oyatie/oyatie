use super::*;

#[test]
fn retained_history_has_no_arbitrary_record_ceiling_and_remains_publishable() {
    let database = Database::new();
    let mut store = database.initialize();
    let mut history = store.history(&caller()).unwrap();
    let mut drafts = Vec::new();
    for generation in 2..=1026 {
        let mut document = baseline().document;
        document.entities[0].revision = generation as u32;
        document.entities[0].display_name = format!("Record {generation}");
        let receipt = PublicationReceipt {
            generation,
            prior_generation: generation - 1,
            draft_id: generation - 1,
            draft_version: 1,
            owner: caller().owner,
        };
        drafts.push(ModelDraft {
            id: generation - 1,
            tenant_id: caller().tenant_id,
            owner: caller().owner,
            base_generation: generation - 1,
            version: 1,
            document: document.clone(),
            publication: Some(receipt.clone()),
        });
        history.publications.push(ModelPublication {
            generation,
            document,
            receipt: Some(receipt),
        });
    }
    ModelLedger::restore(history.clone(), 1026, drafts.clone()).unwrap();
    let transaction = store.connection.transaction().unwrap();
    for (draft, publication) in drafts.iter().zip(history.publications.iter().skip(1)) {
        transaction
            .execute(
                "INSERT INTO ontology_model_draft VALUES (?1, ?2)",
                params![draft.id, encode(draft).unwrap()],
            )
            .unwrap();
        transaction
            .execute(
                "INSERT INTO ontology_model_publication VALUES (?1, ?2, ?3)",
                params![
                    publication.generation,
                    draft.id,
                    encode(publication).unwrap()
                ],
            )
            .unwrap();
    }
    transaction
        .execute(
            "UPDATE ontology_model_binding SET head = 1026, next_draft = 1026",
            [],
        )
        .unwrap();
    transaction.commit().unwrap();
    drop(store);
    let mut reopened = database.open();
    assert_eq!(reopened.history(&caller()).unwrap(), history);
    let mut next = history.head().unwrap().document.clone();
    next.entities[0].revision += 1;
    next.entities[0].display_name = "Next".into();
    let draft = reopened.create_draft(&caller(), 1026, next).unwrap();
    let prepared = reopened.prepare(&caller(), draft.id, 1, 1026).unwrap();
    assert_eq!(
        reopened.publish(&caller(), &prepared).unwrap().generation,
        1027
    );
    assert_eq!(
        reopened.history(&caller()).unwrap().publications.len(),
        1027
    );
}

#[test]
fn many_owned_drafts_do_not_limit_tenant_lifetime_to_sixteen_mebibytes() {
    let database = Database::new();
    let mut store = database.initialize();
    let mut document = baseline().document;
    let text = "x".repeat(MAX_MODEL_TEXT_BYTES);
    document.entities[0].display = Some(ModelDisplay {
        display_name: Some(text.clone()),
        description: Some(text.clone()),
        icon: Some(text.clone()),
        color: Some(text.clone()),
        plural_name: Some(text),
    });
    for _ in 0..1025 {
        store.create_draft(&caller(), 1, document.clone()).unwrap();
    }
    let bytes: u64 = store
        .connection
        .query_row(
            "SELECT SUM(length(record)) FROM ontology_model_draft",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(bytes > 16_777_216);
    drop(store);
    let reopened = database.open();
    assert_eq!(
        reopened.open_draft(&caller(), 1025).unwrap().document,
        document
    );
    assert_eq!(
        reopened
            .history(&caller())
            .unwrap()
            .head()
            .unwrap()
            .generation,
        1
    );
}
