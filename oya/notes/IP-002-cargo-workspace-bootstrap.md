---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-notes-foundation
impl_plan_id: IP-002-cargo-workspace-bootstrap
status: pending
execution_unit: ChangeSet
owner: axis-notes
acceptance_lanes: [cargo-check, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---


# IP-002: Cargo workspace bootstrap

## Intent

Register all 111 crates in the workspace `Cargo.toml`. Crate templates per BC × layer per ADR-0105 + ADR-0106 + ADR-0131. Skeleton crates compile (empty kernel; empty domain stubs) so subsequent IPs add port traits + impls incrementally.

## Concrete File Targets

- `Cargo.toml` — add `members = [ "microservices/notes/src/oya-notes-*-*" ]` glob.
- `microservices/notes/src/<crate>/Cargo.toml` × 111 (workspace-aware deps; LTS pins).
- `microservices/notes/src/<crate>/src/lib.rs` × 111 (empty `pub mod` skeleton).

## Crate Inventory

Per BC × layer (17 BCs, layers vary):

| BC | Layers | Crates |
|---|---|---|
| note-store | kernel, domain, usecase, api, adapter, adapter-postgres, adapter-valkey, adapter-s3, rest, sdk, app | 11 |
| tag-graph | kernel, domain, usecase, api, adapter, adapter-postgres, sdk, app | 8 |
| backlink-graph | kernel, domain, usecase, api, adapter, adapter-postgres, worker, sdk, app | 9 |
| daily-note | kernel, domain, usecase, api, adapter, sdk, app | 7 |
| template-gallery | kernel, domain, usecase, api, adapter, adapter-postgres, sdk, app | 8 |
| web-clipper-bridge | kernel, domain, usecase, api, adapter, rest, sdk | 7 |
| share-link | kernel, domain, usecase, api, adapter, adapter-postgres, rest, sdk, app | 9 |
| embed | kernel, domain, usecase, api, adapter, adapter-s3, sdk, app | 8 |
| checklist | kernel, domain, usecase, api, adapter, worker, sdk, app | 8 |
| version-history | kernel, domain, usecase, api, adapter, adapter-postgres, worker, sdk, app | 9 |
| search-index | kernel, domain, usecase, api, adapter-meilisearch, worker, sdk, app | 8 |
| graph-view-data | kernel, domain, usecase, api, adapter, sdk, app | 7 |
| collab-edit | kernel, domain, usecase, api, adapter, adapter-loro, worker, sdk, app | 9 |
| import-pipeline | kernel, domain, usecase, api, adapter, worker, sdk, app | 8 |
| export-pipeline | kernel, domain, usecase, api, adapter, worker, sdk, app | 8 |
| ai-assist | kernel, domain, usecase, api, adapter, worker, sdk, app | 8 |
| e2e-key-management | kernel, domain, usecase, api, adapter, adapter-mls, sdk, app | 8 |

Total: 111 crates.

## Acceptance Gates

```bash
cargo check --workspace
buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice notes
buck2 build //:quality-lane-registry-authority-check # lane=version-pinning-conformance
```

## Halt Conditions

- Workspace `Cargo.toml` grows beyond agreed budget — escalate.
- Crate naming fails ADR-0056 BNF — fix.

## Next IP

[`IP-003-note-store-kernel-domain.md`](IP-003-note-store-kernel-domain.md)


## A. Problem
`IP-002: Cargo workspace bootstrap` is not a generic implementation packet; it closes the `002 cargo workspace bootstrap` gap for `notes` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Note, PersonalNoteRef, ProfessionalNoteRef, tag-graph, backlink graph, Loro CRDT, MLS key package, share-link, E2E refusal.

## B. Approach
The foundation slice proves deployability and crate registration for notes-specific kernels before product behavior claims move to HG-NOTES. The implementation must keep the µservice boundary intact: contracts remain under `microservices/notes/contracts/openapi/notes.yaml` / `microservices/notes/contracts/proto/notes.proto`, policy decisions remain in `microservices/notes/policy/tenant-scope.cedar`, operational proof remains in `microservices/notes/slos/note-open-latency.openslo.yaml`, and the parity claim is checked against `microservices/notes/competitor-parity-matrix.md`.

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
- `microservices/notes/iac/openbao-policy.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/iac/edge-waf.yaml` — verify/update as the authoritative artifact for this IP.
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
| Apple Notes, Google Keep, OneNote, Notion, Bear, Obsidian, Standard Notes, Evernote, Roam, Logseq, Joplin, Reflect, Tana, Mem, and Heptabase | Notion and OneNote define workspace/collab parity; Obsidian/Roam/Logseq define backlink and graph parity; Standard Notes and Apple Notes define privacy pressure; Evernote/Bear/Google Keep define capture/import expectations. This IP closes the relevant gap by binding `002 cargo workspace bootstrap` to concrete `notes` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
