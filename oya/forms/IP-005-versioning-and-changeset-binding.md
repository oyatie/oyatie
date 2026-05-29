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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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
- PRD FR-04 and AC-05.
- `microservices/forms/manifest.json` service registration.
- `microservices/forms/catalog/oya-forms-version-domain.yaml`.
- `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml` form-published event.
- `microservices/forms/runbooks/response-store-corruption.md` for version mismatch rollback.
- `microservices/forms/decisions/ADR-FORMS-0001-form-definition-schema.md`.

## Foundation A-G Substance

- A. Product scope: every published form is immutable for response interpretation and export replay.
- B. Domain model: `FormDraft`, `FormVersion`, `SchemaHash`, `VersionLifecycle`, and `ChangeSetBinding` are first-class.
- C. Contracts: `FormPublished` includes version, schema hash, pack, and audit-chain seal in REST, AsyncAPI, and proto paths.
- D. Policy: breaking publish requires ADR reference, sunset window, Cedar preview acknowledgement, and pack-specific compliance check.
- E. Operations: revert restores the prior published version pointer without rewriting historical responses.
- F. Observability: emit publish latency, revert count, version lookup misses, and schema-hash mismatch alerts.
- G. Promotion: version-isolation tests, catalog entry, manifest binding, and runbook link must all resolve.

## Counterpart Benchmark

- Counterpart: Notion Forms/Databases schema evolution, Airtable Forms field changes, and GitHub issue form YAML revisions.
- Defensible parity claim: Oyatie must preserve historical response readability after a field rename, removal, or type change.
- Differentiator: ChangeSet binding makes form publish/revert auditable rather than a UI-only save operation.
- Grep counterpart names: Notion Forms/Databases; GitHub issue forms; Airtable Forms.

## Remediation Notes

- Added artifact-backed lifecycle criteria tied to manifest, catalog, contracts, and runbooks.
- Added A-G substance to make versioning defensible as a foundation capability instead of a small helper.
- Added counterpart names for mechanical parity review.

## Next IP

[`IP-006-postgres-citus-adapter-with-column-encryption.md`](IP-006-postgres-citus-adapter-with-column-encryption.md)
