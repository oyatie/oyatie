use crate::{
    codec::{decode, row_bounds},
    encode, storage,
};
use foundry_model_api::*;
use rusqlite::{Connection, OptionalExtension, params};

pub(crate) struct State {
    pub history: ModelHistory,
    pub next_draft: u64,
}

pub(crate) fn state(
    connection: &Connection,
    expected_binding: &ModelBinding,
    expected_baseline: &ModelBaseline,
) -> Result<State, ModelError> {
    let (count, draft_count, max_draft): (u64, u64, u64) = connection
        .query_row(
            "SELECT (SELECT COUNT(*) FROM ontology_model_binding),
            (SELECT COUNT(*) FROM ontology_model_draft),
            COALESCE((SELECT MAX(id) FROM ontology_model_draft), 0)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(storage)?;
    if count != 1 || draft_count != max_draft {
        return Err(ModelError::CorruptState);
    }
    let expected_baseline_bytes = encode(expected_baseline)?;
    let expected_binding_bytes = encode(expected_binding)?;
    let bounds = row_bounds()?;
    let (version, binding, baseline, head, next_draft): (
        u64,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
        u64,
        u64,
    ) = connection
        .query_row(
            "SELECT schema_version,
            CASE WHEN length(binding) = ?2 AND length(binding) <= ?3 THEN binding ELSE NULL END,
            CASE WHEN length(baseline) = ?1 THEN baseline ELSE NULL END,
            head, next_draft FROM ontology_model_binding WHERE singleton = 1",
            params![
                expected_baseline_bytes.len() as u64,
                expected_binding_bytes.len() as u64,
                bounds.binding as u64
            ],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(storage)?;
    if version != 1
        || next_draft != next_model_version(max_draft).map_err(|_| ModelError::CorruptState)?
    {
        return Err(ModelError::CorruptState);
    }
    let binding_bytes = binding.ok_or(ModelError::CorruptState)?;
    let binding: ModelBinding = decode(&binding_bytes, bounds.binding)?;
    if binding_bytes != expected_binding_bytes
        || &binding != expected_binding
        || baseline.as_ref() != Some(&expected_baseline_bytes)
    {
        return Err(ModelError::BindingMismatch);
    }
    let mut statement = connection
        .prepare(
            "SELECT p.generation, p.draft_id,
            CASE WHEN length(p.record) <= ?1 THEN p.record ELSE NULL END,
            CASE WHEN length(d.record) <= ?2 THEN d.record ELSE NULL END
         FROM ontology_model_publication p LEFT JOIN ontology_model_draft d ON d.id = p.draft_id
         ORDER BY p.generation",
        )
        .map_err(storage)?;
    let rows = statement
        .query_map(
            params![bounds.publication as u64, bounds.draft as u64],
            |row| {
                Ok((
                    row.get::<_, u64>(0)?,
                    row.get::<_, Option<u64>>(1)?,
                    row.get::<_, Option<Vec<u8>>>(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?,
                ))
            },
        )
        .map_err(storage)?;
    let mut publications = Vec::new();
    for row in rows {
        let (generation, id, bytes, draft_bytes) = row.map_err(storage)?;
        let publication: ModelPublication =
            decode(&bytes.ok_or(ModelError::CorruptState)?, bounds.publication)?;
        if publication.generation != generation {
            return Err(ModelError::CorruptState);
        }
        match (&publication.receipt, id, draft_bytes) {
            (Some(receipt), Some(id), Some(bytes)) => {
                let draft: ModelDraft = decode(&bytes, bounds.draft)?;
                draft
                    .document
                    .check_bounds()
                    .map_err(|_| ModelError::CorruptState)?;
                if id != draft.id
                    || id != receipt.draft_id
                    || draft.publication.as_ref() != Some(receipt)
                    || draft.document != publication.document
                    || draft.owner != receipt.owner
                    || draft.tenant_id != binding.tenant_id
                    || draft.version != receipt.draft_version
                    || draft.base_generation != receipt.prior_generation
                {
                    return Err(ModelError::CorruptState);
                }
            }
            (None, None, None) if generation == 1 => {}
            _ => return Err(ModelError::CorruptState),
        }
        publications.push(publication);
    }
    if publications.last().is_none_or(|row| row.generation != head) {
        return Err(ModelError::CorruptState);
    }
    let history = ModelHistory {
        binding,
        baseline: expected_baseline.clone(),
        publications,
    };
    history.registry().map_err(|_| ModelError::CorruptState)?;
    Ok(State {
        history,
        next_draft,
    })
}

pub(crate) fn draft(
    connection: &Connection,
    history: &ModelHistory,
    id: u64,
) -> Result<ModelDraft, ModelError> {
    let max_bytes = row_bounds()?.draft;
    let bytes: Option<Option<Vec<u8>>> = connection.query_row(
        "SELECT CASE WHEN length(record) <= ?1 THEN record ELSE NULL END FROM ontology_model_draft WHERE id = ?2",
        params![max_bytes as u64, id], |row| row.get(0)).optional().map_err(storage)?;
    let draft: ModelDraft = decode(
        &bytes
            .ok_or(ModelError::MissingDraft)?
            .ok_or(ModelError::CorruptState)?,
        max_bytes,
    )?;
    if draft.id != id {
        return Err(ModelError::CorruptState);
    }
    draft
        .check_record(history)
        .map_err(|_| ModelError::CorruptState)?;
    Ok(draft)
}

pub(crate) fn audit_drafts(
    connection: &Connection,
    history: &ModelHistory,
) -> Result<(), ModelError> {
    let mut statement = connection
        .prepare("SELECT id FROM ontology_model_draft ORDER BY id")
        .map_err(storage)?;
    let rows = statement
        .query_map([], |row| row.get::<_, u64>(0))
        .map_err(storage)?;
    for row in rows {
        draft(connection, history, row.map_err(storage)?)?;
    }
    Ok(())
}
