# DPIA: cell-lifecycle

cell-lifecycle stores operational cell metadata, tenant-class scope, resident-count proofs, evidence pack references, Cedar decision ids, and audit-chain event ids. It does not store tenant payloads, user content, secrets, or raw evidence pack contents.

## Data Inventory
DATA-001: cell_id and region_id identify platform topology rather than a natural person.
DATA-002: tenant_class_scope records demo_trial and paid applicability but not tenant secrets.
DATA-003: resident_count_proof records aggregate counts and snapshot ids from tenancy.
DATA-004: evidence_pack_id and hashes refer to evidence held by the evidence substrate.
DATA-005: principal records operator or automation identity and may be personal data for human operators.
DATA-006: audit_chain_event_id and cedar_decision_id are operational accountability records.

## Privacy Controls
CTRL-001: Minimize raw data by storing evidence references and hashes instead of evidence payloads.
CTRL-002: Restrict lifecycle list output by Cedar action ReadLifecycle and pack-specific visibility.
CTRL-003: Keep operator identity retention aligned with audit-chain retention and compliance pack floors.
CTRL-004: Avoid tenant payloads in logs, spans, metrics, errors, and runbook evidence snippets.
CTRL-005: Use HLC timestamps for ordering, not human-local timestamps that leak location beyond region metadata.
CTRL-006: Support deletion only for non-audit operational cache entries; history remains immutable under compliance retention.
