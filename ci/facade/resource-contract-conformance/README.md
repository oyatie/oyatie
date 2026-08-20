# cloud-ci-cloud-resource-contracts

`ci-resource-contract-conformance` is the Rust/API-shaped replacement for the first P0 cloud-resource Python validator slice:

- `scripts/tests/cloud_resource_contract_parity_catalog_check.py`
- `scripts/tests/cloud_control_plane_operation_contract_check.py`
- `scripts/tests/cloud_enforceability_facets_check.py`

Authority anchors: `/specs/root-hub-pointers.json` routes agents to the current operating contract, and accepted ADR-0515 makes `oya-ci-required` the single blocking CI context with Rust/Buck2 gates as merge authority. This crate follows that accepted lane: the legacy Python files are fenced local/provenance bridges only, not admission authority.

The Python files are no longer the primary validation path. The authoritative gate is:

```text
//ci/facade/resource-contract-conformance:ci-resource-contract-conformance-gate
```

## API surface

The production entrypoint is pure and side-effect free:

```text
evaluate_configured(policy_json, corpus_json) -> Report { verdict, violations, findings }
```

The policy is data in `cloud-resource-contracts-policy.json`. The corpus is a JSON object keyed by the configured `spec_inputs`, for example:

```json
{
  "cloud_resource_contract_parity_catalog": { "...": "specs/cloud-resource-contract-parity-catalog.json" },
  "cloud_control_plane_operation_contract": { "...": "specs/cloud-control-plane-operation-contract.json" },
  "cloud_enforceability_facets": { "...": "specs/cloud-enforceability-facets.json" },
  "cloud_hyperscaler_parity_taxonomy": { "...": "specs/cloud-hyperscaler-parity-taxonomy.json" },
  "cloud_resource_catalog_target": { "...": "specs/cloud-resource-catalog-target.json" },
  "cloud_control_plane_canonical": { "...": "specs/cloud-control-plane-canonical.json" }
}
```

A future service/controller can expose the same contract over the architecture-planned operation API without changing gate semantics:

```text
POST /v1/infra-automation/operations { source_item_id, target_surface, config_ref, idempotency_key }
GET  /v1/infra-automation/operations/{operation_id}
```

## Operator workflow

Local focused check:

```text
buck2 test //ci/facade/resource-contract-conformance:ci-resource-contract-conformance-unittest
```

Merge-authority check:

```text
buck2 test \
  //ci/facade/resource-contract-conformance:ci-resource-contract-conformance-unittest \
  //ci/facade/resource-contract-conformance:ci-resource-contract-conformance-gate
```

The GitHub Actions `oya-ci-required` matrix runs those Buck2 targets as the canonical CI surface. Any change to the six configured spec inputs should be validated by this gate instead of invoking the retired Python scripts.

## Configuration contract

`cloud-resource-contracts-policy.json` declares:

- `source_migration_slice`: the selected legacy Python sources and their Rust gate replacement target.
- `spec_inputs`: the repository-local JSON artifacts consumed by the evaluator.
- `claim_policy`: forbidden overclaim/actuation wording and the metadata-only boundary.

The evaluator is intentionally read-only: no subprocess, no network, no clock/randomness, and no repository scanning outside the caller-provided corpus.
