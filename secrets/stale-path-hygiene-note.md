---
doc_class: JudgmentNote
title: Stale microservices/{cloud-secrets,cloud-kms} path hygiene (wave-4 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - secrets/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`secrets/**` — Seat A wave-4)

## Scope

Retarget only **verified** in-tree destinations under `secrets/**` / `secrets/kms/**`.
Do not invent missing homes. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- `microservices/cloud-secrets/**` → `secrets/**` when present
- `microservices/cloud-kms/**` → `secrets/kms/**` when present (PRD/README/manifest/faqs/benchmarks/runbooks/contracts)

## Deferred

- Journey-local contract/policy/test paths without in-tree counterparts; `legal/**`; missing `byok-ceremony.md`.


## Wave-5 Seat A follow-through (2026-08-10)

Retargeted verified remaps:

- microservices/cloud-kms/slos/ -> secrets/observability/slos/cloud-kms/
- Bare microservices/cloud-secrets -> secrets (reorg-unit judgments)

### Deferred

- Journey-local policy/eval/asyncapi fixtures without in-tree counterparts; missing KMS ARCHITECTURE/src; control-plane.openslo.yaml filename not present under observability alias
- No hubs, no Cargo.lock, no merge.


## Wave-6 Seat A forever-shape (2026-08-10)

Retargeted verified remaps:

- Burned `crates/oya-cloud-kms-domain/**` → `secrets/core/kms-domain/**`
- Burned `crates/oya-cloud-kms-api/**` → `secrets/ports/kms-api/**`
- Burned `crates/oya-cloud-kms-adapter-oci/**` → `secrets/adapters/kms-oci/**`
- Burned `crates/oya-cloud-kms-adapter-openbao/**` → `secrets/adapters/kms-openbao/**`
- Absolute `.../microservices/cloud-kms/capability-tenant_class-deltas-...` → `secrets/kms/tenant-class-adoption-deltas-vs-counterparts-2026-05-20.md`
- Reader-facing retired tenant_class purpose/matrix cites → `secrets/kms/PRD.md` + `secrets/kms/tenant-class-adoption-deltas-vs-counterparts-2026-05-20.md`
- `ARCH.md` ownership: `microservices/{cloud-secrets,cloud-kms}/src/` → forever faces `secrets/{core,ports,adapters,facade}/`

### Deferred (still missing — do not invent)

- Journey-local jNN OpenAPI/AsyncAPI/proto/Cedar/runbook/test fixtures; `legal/**`; `byok-ceremony.md`; `policy/openbao*.hcl`; `policy/pack-routing.cedar`; `policy/schema.cedarschema`; `capabilities/eval/*paiden*`; `tests/bench/**`; `src/crates/**` create targets (IP-003); `secrets/kms/ARCHITECTURE.md`; `observability/slos/cloud-kms/control-plane.openslo.yaml`; `kms/runbooks/README.md`
- Burned `crates/oya-cloud-secrets-domain` / rotator crate cites without matching forever files (`secret_kind.rs`, rotator crates)
- `crates/oya-cloud-kms-adapter-vault-enterprise/` (no `secrets/adapters/kms-vault*` home)
- Repo-root `contracts/openapi/cloud/cloud-kms-v1.yaml` retained (secrets/kms stub is not the full contract)
- Historical coherence-audit finding labels that still say `retired tenant_class adoption artifact` as inventory rows (not reader file pointers)
- No hubs, no Cargo.lock, no merge.
