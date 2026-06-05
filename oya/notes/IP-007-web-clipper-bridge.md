---
doc_class: ImplementationPlan
impl_plan_id: IP-007-web-clipper-bridge
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes + ops-security
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location, web-extension-security-review]
---


# IP-007: web-clipper-bridge + browser extensions (Chrome MV3 + Firefox MV3 + Safari Web Extensions + Edge Add-ons)

## Intent

Land `oya-notes-web-clipper-bridge-*` (server-side ingest) + browser extension code (Chrome MV3 + Firefox MV3 + Safari Web Extensions + Edge Add-ons).

Per-installation token rotation 90d. MV3 isolated-world execution. Minimum-permission manifest (no broad host_permissions; `activeTab` only).

## Extension Manifest (MV3)

```json
{
  "manifest_version": 3,
  "name": "oyatie notes — Web Clipper",
  "version": "1.0.0",
  "permissions": ["activeTab", "storage", "notifications"],
  "host_permissions": [],
  "background": {"service_worker": "service-worker.js"},
  "action": {"default_popup": "popup.html"},
  "content_security_policy": {"extension_pages": "default-src 'self'; connect-src https://*.oyatie.dev"}
}
```

## Test Plan

- Capture latency p95 ≤ 500ms.
- Installation token rotation 90d enforced.
- Per-installation token never exposed via DOM (MV3 isolated world).
- Local-queue mode (offline) replays on reconnect.

## Acceptance Gates

```bash
cargo check -p oya-notes-web-clipper-bridge-kernel
npm run lint --prefix extensions/chrome
npm run test --prefix extensions/chrome
buck2 build //:quality-lane-registry-authority-check # lane=web-extension-security
```

## Halt Conditions

- Extension fails security review (XSS via clipped HTML; token leakage via DOM) — block.
- Manifest grows beyond minimum-permission spec — review.

## Next IP

[`IP-008-share-link-and-embed.md`](IP-008-share-link-and-embed.md)


## A. Problem
`IP-007: web-clipper-bridge + browser extensions (Chrome MV3 + Firefox MV3 + Safari Web Extensions + Edge Add-ons)` is not a generic implementation packet; it closes the `007 web clipper bridge` gap for `notes` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Note, PersonalNoteRef, ProfessionalNoteRef, tag-graph, backlink graph, Loro CRDT, MLS key package, share-link, E2E refusal.

## B. Approach
Capture/share paths keep notes personal-by-default: clipper tokens are per-installation, share-links are read-only/revocable, and Personal sharing requires client-side re-encryption. The implementation must keep the µservice boundary intact: contracts remain under `microservices/notes/contracts/openapi/notes.yaml` / `microservices/notes/contracts/proto/notes.proto`, policy decisions remain in `microservices/notes/policy/tenant-scope.cedar`, operational proof remains in `microservices/notes/slos/note-open-latency.openslo.yaml`, and the parity claim is checked against `microservices/notes/competitor-parity-matrix.md`.

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
- `microservices/notes/catalog/oya-notes-web-clipper-bridge-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-share-link-kernel.yaml` — verify/update as the authoritative artifact for this IP.
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
| Apple Notes, Google Keep, OneNote, Notion, Bear, Obsidian, Standard Notes, Evernote, Roam, Logseq, Joplin, Reflect, Tana, Mem, and Heptabase | Notion and OneNote define workspace/collab parity; Obsidian/Roam/Logseq define backlink and graph parity; Standard Notes and Apple Notes define privacy pressure; Evernote/Bear/Google Keep define capture/import expectations. This IP closes the relevant gap by binding `007 web clipper bridge` to concrete `notes` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
