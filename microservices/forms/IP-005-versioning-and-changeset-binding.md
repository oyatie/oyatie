---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-005-versioning-and-changeset-binding
status: pending
execution_unit: ChangeSet
owner: axis-forms
acceptance_lanes: [cargo-test, oya-forms-version-isolation, oya-forms-changeset-conformance]
---

# IP-005: Versioning + ChangeSet binding

## Intent

Bind form-definition lifecycle (draft → publish → archive) to the ChangeSet state machine (ADR-0110). Form publishes create a new major version; old responses always queryable against the version they were captured under (AC-05 version isolation).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/domain/version/lifecycle.rs` | create |
| `microservices/forms/src/domain/version/changeset.rs` | create — ChangeSet integration |
| `microservices/forms/src/domain/version/migration.rs` | create — non-breaking migration support |
| `microservices/forms/tests/version_isolation.rs` | create |

## Code Shape

```rust
pub struct FormVersion {
    pub form_id: FormId,
    pub version: u32,
    pub schema_hash: ShaHash,
    pub published_at: DateTime<Utc>,
    pub change_set_id: ChangeSetId,
}

pub fn publish_form(
    draft: FormDraft,
    cedar_policy_preview_acknowledged: bool,
    annex_iii_4_screening: bool,
) -> Result<(FormVersion, AuditChainSeal), PublishError> { /* … */ }
```

## Acceptance Gates

- Old response against old version still queryable; schema_hash matches.
- ChangeSet-based revert restores prior version.
- Breaking-change publish requires explicit ADR reference + sunset window.

## References

- ADR-0110 ChangeSet state machine.
- ADR-FORMS-0001.
- `feedback_no_silent_regression.md`.

## Next IP

[`IP-006-postgres-citus-adapter-with-column-encryption.md`](IP-006-postgres-citus-adapter-with-column-encryption.md)
