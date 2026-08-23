---
doc_status: published
---

# Fitness Lane: schema-migration

- status: Accepted
- date: 2026-05-12
- purpose: Verify every schema migration file has matching forward + rollback steps and is referenced from the migration index.
- enforces: STANDARD/schema-migration; AGENTS.md fitness-lane `governance-schema-migration`.
- kernel_crate: `governance-schema-migration-kernel` — `MigrationFile { id, has_forward, has_rollback, indexed }`, verdict `SchemaMigrationFitnessReport { migrations_checked }`.
- runner_path: `tools/governance-schema-migration`
- inputs: `db/migrations/**/*`, `db/migrations/INDEX.md`.
- failure_modes:
  - migration file lacks rollback section
  - migration not referenced in INDEX
  - duplicate migration id
- ci_invocation: `cargo run -p governance-schema-migration`
- runtime_budget: 200 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct MigrationFile {
    pub id: String,           // data_class: INTERNAL_ONLY
    pub has_forward: bool,    // data_class: INTERNAL_ONLY
    pub has_rollback: bool,   // data_class: INTERNAL_ONLY
    pub indexed: bool,        // data_class: INTERNAL_ONLY
}

pub struct SchemaMigrationFitnessReport { pub migrations_checked: usize }

pub enum SchemaMigrationFitnessError {
    MissingForward { id: String },
    MissingRollback { id: String },
    Unindexed { id: String },
    DuplicateId { id: String },
}

pub fn validate_schema_migration_fitness(
    migrations: &[MigrationFile],
) -> Result<SchemaMigrationFitnessReport, SchemaMigrationFitnessError> {
    let mut ids = std::collections::BTreeSet::new();
    for m in migrations {
        if !ids.insert(m.id.clone()) {
            return Err(SchemaMigrationFitnessError::DuplicateId { id: m.id.clone() });
        }
        if !m.has_forward { return Err(SchemaMigrationFitnessError::MissingForward { id: m.id.clone() }); }
        if !m.has_rollback { return Err(SchemaMigrationFitnessError::MissingRollback { id: m.id.clone() }); }
        if !m.indexed { return Err(SchemaMigrationFitnessError::Unindexed { id: m.id.clone() }); }
    }
    Ok(SchemaMigrationFitnessReport { migrations_checked: migrations.len() })
}
```
