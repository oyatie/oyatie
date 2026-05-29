---
doc_class: PolicyArtifact
template_id: TPL-POLICY-DOC
microservice: drive
status: Accepted
date: 2026-05-17
owner_team: ops-security + axis-drive
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0135, ADR-0140 (retired per ADR-0145)]
doc_status: published
---

# Dual-Context Isolation — drive µservice

## Purpose

Define the structural invariant that **Personal-context** and **Professional-context** file data are isolated at code-level and policy-level, never co-mingled or cross-referenced.

This artifact carries Bominal ADR-0208 inheritance (dual-context unified-channel hub) forward to the drive µservice per `feedback_bominal_inheritance_precedence.md`. Per ADR-0135 + ADR-0140 the invariant is preserved + strengthened in the new flat layout.

## Invariant (binding contract)

For every file `f` stored in drive:
1. `f.context_kind ∈ {Personal, Professional}` — non-nullable, immutable post-creation.
2. There exists NO code path in drive that reads or writes both contexts in the same usecase invocation (modulo the narrow context-bridge exception below).
3. Cross-context queries (folder list, search, preview render, sync, share-link generation) refuse cross-context resource binding with `403 DriveCrossContextRefused`.
4. Audit-chain seal emitted on every refusal.

## Layered enforcement

The invariant is enforced at **three** layers (defence-in-depth):

### Layer 1 — Rust type system

`oya-drive-file-store-kernel` declares two distinct types:
- `PersonalFile { /* … */ }`
- `ProfessionalFile { /* … */ }`

There is **no** shared parent struct. A function that operates on personal files cannot accept a professional file (and vice-versa) without an explicit cast through the context-bridge port. The LEAN check `oya-check-context-isolation` validates at build-time that no usecase reads both types.

### Layer 2 — Cedar policy

`policy/tenant-scope.cedar` carries forbid clauses:

```cedar
forbid (
  principal in DriveRole::"professional_reader",
  action in [Action::"file_read", Action::"folder_list", Action::"search_query"],
  resource in FileContext::"Personal"
);

forbid (
  principal in DriveRole::"personal_reader",
  action in [Action::"file_read", Action::"folder_list", Action::"search_query"],
  resource in FileContext::"Professional"
);
```

Permit for explicit context-bridge:

```cedar
permit (
  principal in DriveRole::"context_bridge_reader",
  action == Action::"file_read",
  resource
)
when {
  context.personal_to_professional_grant.file_id == resource.id &&
  context.personal_to_professional_grant.consent_at <= context.now &&
  context.personal_to_professional_grant.revoked_at == null
};
```

### Layer 3 — Storage isolation

- Per-context Postgres schemas (`drive_personal_<tenant>` vs `drive_professional_<tenant>`).
- Per-context Meilisearch index (search cross-context query refused server-side).
- Per-context object-store prefix.

## Context-bridge exception (narrow, audited)

The user MAY explicitly grant Personal → Professional file read for a specific file via the Application Shell consent flow. The grant:
- Records `personal_to_professional_grant{file_id, principal_user_id, consent_at, revoked_at?}`.
- Emits audit-chain seal at grant + revoke.
- Cedar policy admits read only for the granted file id; no other files.
- Grant is revocable at any time; revocation cascade evicts from per-tenant cache.

## Hyrum's-Law surface preservation

Per the Strangler migration in ADR-0134, the legacy `oya-drive-domain` dual-context invariant is preserved in the new layout verbatim. The `FileContextBoundaryGuard` port trait signature is identical; the refusal error variant `DriveCrossContextRefused` is identical. Consumers wrapping the boundary guard via the legacy import path see identical refusal behaviour after migration.

## CI lanes

- `oya-check-context-isolation` (BLOCKER) — refuses build if any usecase reads both contexts.
- `oya-check-cedar-context-policy-coverage` (BLOCKER) — refuses if the forbid clauses above are removed / weakened.
- `oya-check-per-context-storage-prefix` (BLOCKER) — refuses if Postgres schema / Meilisearch index / object-store prefix lacks `personal_` / `professional_` qualifier.

## Verification

```bash
# Build-time
cargo run -p oya-dev-cli -- gate validate context-isolation --microservice drive

# Cedar policy unit tests
cargo nextest run -p oya-drive-permissions-domain -- cedar_dual_context_refusal

# E2E: confirm Personal-context file does NOT appear in Professional-context list/search/preview
cargo nextest run --test e2e_dual_context_isolation
```

## References

- ADR-0028 (Bominal): audit chain.
- ADR-0056: BNF v4.1.
- ADR-0105: 13-layer enum.
- ADR-0135: unbundle dual-context inheritance.
- ADR-0140: Cedar policy enforcement.
- Bominal ADR-0208: dual-context unified-channel hub.
- `policy/tenant-scope.cedar`.
- `microservices/drive/threat-model.md` (T-I-02).
- `microservices/drive/PRD.md` (AC-07).
- `feedback_bominal_inheritance_precedence.md`.
