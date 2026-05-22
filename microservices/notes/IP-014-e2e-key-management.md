---
doc_class: ImplementationPlan
impl_plan_id: IP-014-e2e-key-management
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes + council-privacy
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location, openmls-version-pin]
---


# IP-014: e2e-key-management (openmls 0.6) + recovery seed UX

## Intent

Land `oya-notes-e2e-key-management-{kernel,domain,usecase,api,adapter,adapter-mls,sdk,app}`. Implements:
- Server: public KeyPackage distribution + commit message routing + revocation list (TTL 1m).
- Client SDK: MLS RFC 9420 key derivation via openmls 0.6 (TS via Wasm; Swift via openmls Swift bridge; Kotlin via openmls JVM; Rust native).
- Recovery seed UX: 24-word BIP39-style; double-confirmation at onboarding.

## Server-Side Constraints

- NEVER stores private key material.
- NEVER decrypts ciphertext.
- Stores `key_package_bytes` (public) + `commit_bytes` (opaque).

## Client SDK Recovery UX

- Recovery seed presented at onboarding; user must explicitly confirm receipt ("I have stored the seed safely") + acknowledge tradeoff ("Loss = permanent destruction").
- Recovery flow on new device: prompt for seed → derive root key → register new device-bound KeyPackage.

## Acceptance Gates

```bash
cargo check -p oya-notes-e2e-key-management-kernel
cargo check -p oya-notes-e2e-key-management-adapter-mls
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Next IP

[`IP-015-hg-notes-conformance.md`](IP-015-hg-notes-conformance.md)


## A. Problem
`IP-014: e2e-key-management (openmls 0.6) + recovery seed UX` is not a generic implementation packet; it closes the `014 e2e key management` gap for `notes` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Note, PersonalNoteRef, ProfessionalNoteRef, tag-graph, backlink graph, Loro CRDT, MLS key package, share-link, E2E refusal.

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
| Apple Notes, Google Keep, OneNote, Notion, Bear, Obsidian, Standard Notes, Evernote, Roam, Logseq, Joplin, Reflect, Tana, Mem, and Heptabase | Notion and OneNote define workspace/collab parity; Obsidian/Roam/Logseq define backlink and graph parity; Standard Notes and Apple Notes define privacy pressure; Evernote/Bear/Google Keep define capture/import expectations. This IP closes the relevant gap by binding `014 e2e key management` to concrete `notes` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
