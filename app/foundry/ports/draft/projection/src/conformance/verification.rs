use super::*;
pub fn check_exact_entry_verification_is_read_only<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let entry = applied_with_links(
        "ten_a",
        1,
        vec![object("ten_a", "ent_a1", "ety_reading", vec![])],
        vec![edge("ent_a1", "ent_a2")],
    );
    let store = fixture.store();
    if !matches!(
        store.verify_entry(&entry),
        Err(ProjectionStoreError::DivergentReplay { ordinal: 1 })
    ) {
        return Err("missing entry verification must refuse".into());
    }
    if store.applied_head("ten_a").map_err(|e| format!("{e:?}"))? != 0 {
        return Err("verification wrote a missing entry".into());
    }
    store
        .apply(entry.clone(), &KeyDesignations::default())
        .map_err(|e| format!("{e:?}"))?;
    let poison = AppliedEntry {
        tenant_id: "ten_a".into(),
        ordinal: 2,
        outcome: EntryOutcome::Poisoned {
            reason: "malformed_payload".into(),
        },
    };
    store
        .apply(poison.clone(), &KeyDesignations::default())
        .map_err(|e| format!("{e:?}"))?;
    store.verify_entry(&entry).map_err(|e| format!("{e:?}"))?;
    store.verify_entry(&poison).map_err(|e| format!("{e:?}"))?;
    for mutation in 0..6 {
        let mut changed = entry.clone();
        if let EntryOutcome::Applied { objects, links } = &mut changed.outcome {
            match mutation {
                0 => objects[0].last_actor = "prn_other".into(),
                1 => objects[0].schema_revision = 2,
                2 => objects[0].last_ordinal = 2,
                3 => links[0].observed_at_epoch_ms += 1,
                4 => links[0].to_object_ref = "ent_other".into(),
                _ => objects.clear(),
            }
        }
        if !matches!(
            store.verify_entry(&changed),
            Err(ProjectionStoreError::DivergentReplay { ordinal: 1 })
        ) {
            return Err(format!("changed entry dimension {mutation} was accepted"));
        }
    }
    let mut changed = poison.clone();
    changed.outcome = EntryOutcome::Poisoned {
        reason: "unknown_revision".into(),
    };
    if !matches!(
        store.verify_entry(&changed),
        Err(ProjectionStoreError::DivergentReplay { ordinal: 2 })
    ) {
        return Err("changed poison accepted".into());
    }
    if store.applied_head("ten_a").map_err(|e| format!("{e:?}"))? != 2 {
        return Err("verification changed head".into());
    }
    store.verify_entry(&entry).map_err(|e| format!("{e:?}"))?;
    store.verify_entry(&poison).map_err(|e| format!("{e:?}"))?;
    if fixture.reopen() {
        fixture
            .store()
            .verify_entry(&entry)
            .map_err(|e| format!("{e:?}"))?;
    }
    Ok(())
}
