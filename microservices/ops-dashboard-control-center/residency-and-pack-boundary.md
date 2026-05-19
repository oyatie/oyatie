# Ops Dashboard / Control Center Residency and Regional Pack Boundary

## Data residency

The service supports all standard regulatory packs declared in `manifest.json#regulatory_packs`. Residency-sensitive runtime data remains in owning services and pack-specific evidence stores. The control center stores command records, audit seal refs, and evidence refs needed for operator accountability.

## KR pack boundary

KR localization hooks are operational runbook and evidence-export hooks. They must not introduce jurisdiction-specific logic into canonical-base code paths. KR regulatory content belongs under the KR localization pack and signed evidence directories.

## Acceptance criteria

- Pack-specific escalation uses pack evidence refs, not hard-coded canonical-base behavior.
- Evidence exports are scoped by tenant, pack, and time window.
- Regional pack controls remain separable from control-plane command semantics.
