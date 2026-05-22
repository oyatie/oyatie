---
doc_class: ImplementationPlan
status: Accepted
date: 2026-05-20
related_adrs: [ADR-NOTES-0003, ADR-NOTES-0001]
acceptance_status: draft
companion_docs: [microservices/notes/ARCHITECTURE.md]
inbound_citations: [microservices/notes/manifest.json]
---

# IP-016: Collab-edit MLS + Loro hardening

## A. Goal
Harden the collaborative-editing pipeline (Loro CRDT + MLS group key) to handle network partitions, divergence, and member-rotation gracefully. Hyperscaler precedent: Figma's CRDT recovery (multi-replica convergence) + Google Docs OT-fallback playbook + Apple Notes' MLS-backed group sync.

## B. Acceptance criteria
- Loro divergence resolves automatically within 30s post-reconnect for 99% of cases.
- MLS member-add / member-remove rotates group key with ≤200ms p99 latency.
- E2E invariant holds: server never sees plaintext for `B2C_PERSONAL_E2E` notes.
- Runbook `runbooks/crdt-divergence-recovery.md` covers the residual 1% manual path.
- SLO `notes-collab-edit-merge-latency` green for 14 consecutive days.

## C. Tasks
1. Upgrade `oya-notes-collab-edit-adapter-loro` to Loro 1.x stable.
2. MLS group rotation on share-link revocation (per `runbooks/notes-share-link-revocation.md`).
3. Divergence detector + auto-rebroadcast.
4. Server-side force-merge as last-resort.
5. SLO + dashboard.
6. Runbook `runbooks/crdt-divergence-recovery.md` (done).

## D. Dependencies
- IP-011 collab-edit Loro GA.
- IP-014 e2e-key-management.

## E. Risks
- Loro library bugs; mitigated by upstream contribution + fallback to CRDT-Y.
- MLS group thrash on high-churn share lists; mitigated by batching member-rotations.

## F. References
- ADR-NOTES-0003
- ADR-NOTES-0001
- MLS RFC 9420


## A. Problem
`IP-016: Collab-edit MLS + Loro hardening` is not a generic implementation packet; it closes the `016 collab edit mls loro hardening` gap for `notes` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Note, PersonalNoteRef, ProfessionalNoteRef, tag-graph, backlink graph, Loro CRDT, MLS key package, share-link, E2E refusal.

## B. Approach
Personal-tier plaintext exclusion is enforced before collaboration or AI: MLS/OpenMLS key material stays client-side, Loro CRDT is Professional-only unless an encrypted client flow exists. The implementation must keep the µservice boundary intact: contracts remain under `microservices/notes/contracts/openapi/notes.yaml` / `microservices/notes/contracts/proto/notes.proto`, policy decisions remain in `microservices/notes/policy/tenant-scope.cedar`, operational proof remains in `microservices/notes/slos/note-open-latency.openslo.yaml`, and the parity claim is checked against `microservices/notes/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/notes/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/openapi/notes.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/proto/notes.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/asyncapi/notes-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/policy/tenant-scope.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/slos/note-open-latency.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/runbooks/sync-conflict-resolution.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/policy/e2e-personal-tier-default.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-e2e-key-management-adapter-mls.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/notes/PRD.md` and `microservices/notes/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `notes`.
2. Diff the declared contract in `microservices/notes/contracts/openapi/notes.yaml` and `microservices/notes/contracts/proto/notes.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/notes/policy/tenant-scope.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/notes/slos/note-open-latency.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/notes/PRD.md`, `microservices/notes/ARCHITECTURE.md`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/policy/tenant-scope.cedar`, `microservices/notes/slos/note-open-latency.openslo.yaml`, and `microservices/notes/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/notes/PRD.md`
- `microservices/notes/ARCHITECTURE.md`
- `microservices/notes/contracts/openapi/notes.yaml`
- `microservices/notes/contracts/proto/notes.proto`
- `microservices/notes/contracts/asyncapi/notes-events.yaml`
- `microservices/notes/policy/tenant-scope.cedar`
- `microservices/notes/slos/note-open-latency.openslo.yaml`
- `microservices/notes/runbooks/sync-conflict-resolution.md`
- `microservices/notes/catalog/oya-notes-note-store-kernel.yaml`
- `microservices/notes/competitor-parity-matrix.md`
- `microservices/notes/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Apple Notes, Google Keep, OneNote, Notion, Bear, Obsidian, Standard Notes, Evernote, Roam, Logseq, Joplin, Reflect, Tana, Mem, and Heptabase | Notion and OneNote define workspace/collab parity; Obsidian/Roam/Logseq define backlink and graph parity; Standard Notes and Apple Notes define privacy pressure; Evernote/Bear/Google Keep define capture/import expectations. This IP closes the relevant gap by binding `016 collab edit mls loro hardening` to concrete `notes` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
