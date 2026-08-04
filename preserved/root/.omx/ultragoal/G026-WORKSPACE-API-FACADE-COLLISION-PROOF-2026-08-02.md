# G026 workspace API facade collision/importer proof — 2026-08-02

State: **PLANNING_ONLY — FOUR CAPABILITY OWNERS CONFIRMED; EXACT LEAF NAMES NOT YET APPROVED; NO MOVE PLAN**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G026-APPLICATION-CONE-OWNER-SPLIT-2026-08-02.md`.  
No path, package, workspace, policy, generated face, PR, GitOps declaration, or cluster state was changed.

## Result

The four workspace API crates are independently movable as capability-owned facade candidates after review. They are not one `app/workspace` product and must not be moved as one batch.

| Source | Owning capability | Existing dependency | Exact destination path collision | External Cargo/Buck importer |
|---|---|---|---|---|
| `oya-workspace-chat-api` | `comms` | `comms/core/messenger-domain` | none for `comms/facade/{chat-api,workspace-chat-api}` | none |
| `oya-workspace-meet-api` | `comms` | `comms/core/meet-domain` | none for `comms/facade/{meet-api,workspace-meet-api}` | none |
| `oya-workspace-drive-api` | `storage` | `storage/facade/drive` | none for `storage/facade/{drive-api,workspace-drive-api}` | none |
| `oya-workspace-forms-api` | `workflow` | `workflow/facade/forms-domain` | none for `workflow/facade/{forms-api,workspace-forms-api}` | none |

This proves destination-face availability and a bounded importer rewrite set. It does not choose the exact leaf spelling: that remains an independently reviewed capability-owner decision.

## Collision method and correction

Existence was tested against the immutable commit with exact destination paths. A first global basename probe found `comms/ports/meet-api`, but that is not a collision with `comms/facade/meet-api`: the closed face grammar permits the same leaf basename under different faces. Treating it as a collision would conflate a port seam with a sold facade.

All twelve tested exact destination candidates were absent:

- `comms/facade/{chat-api,meet-api,workspace-chat-api,workspace-meet-api}`;
- `storage/facade/{drive-api,workspace-drive-api}`;
- `workflow/facade/{forms-api,workspace-forms-api}`;
- plus the separately measured shell/plugin candidate paths under `console/facade` and `marketplace/{core,facade}`.

Package-path basename probes were also free for the four workspace candidates. This is naming evidence only; package rename policy still belongs to the codemod lane.

## Semantic overlap proof

The source crates are facade-shaped HTTP/REST boundaries, not ports or capability kernels:

- Chat owns REST normalization, idempotent message-send handling, authorization proof checks, membership checks, and request/response records around `comms-messenger-domain`.
- Meet owns REST normalization, idempotent session-start handling, authorization proof checks, and host-participant validation around `comms-meet-domain`.
- Drive owns PUT/GET boundary normalization, idempotent metadata creation, authorization, and ACL projection around `storage-drive-domain`.
- Forms owns REST normalization, idempotent submission ingest, authorization, submitter binding, and route projection around `workflow-forms-domain`.

Their data-boundary helper dependency supplies classification primitives and does not create a second product capability.

Existing destinations corroborate the split:

- `comms/facade/messenger-stream-rest` is already a framework-free REST boundary; `comms/ports/meet-api` is explicitly a port seam, so the meet REST boundary must not overwrite or absorb it mechanically.
- `storage/facade/drive` currently contains the Drive domain/kernel-shaped records consumed by the source API; moving the API beside it is a facade consolidation, not an app birth.
- `workflow/facade/forms-domain` currently contains Forms domain/kernel-shaped records consumed by the source API; moving the API beside it is likewise a facade consolidation.

No conclusion is made here about whether the existing `storage/facade/drive` and `workflow/facade/forms-domain` leaf names themselves are ideally face-classified. This slice avoids expanding into an unrelated reclassification.

## Importer and policy rewrite set

No Cargo or Buck target outside each source directory imports any of the four package names. The measured mechanical rewrite set is therefore bounded to:

1. the moved crate's own `Cargo.toml` relative paths;
2. its own `BUCK` labels and path-derived target location;
3. root glob membership, which already includes `oya/*/crates/oya-*` but requires the canonical workspace resolver to observe the new capability path;
4. module-membership policy rows for all four source paths;
5. caller-supplied-authorization frozen path/function hashes for all four crates;
6. the explicit chat `validate_authorization` path assertion in `ci/facade/caller-supplied-authorization/tests/dto_authz_trust.rs`;
7. catalogs, contracts, SLOs, dashboards, ownership, and other non-code artifacts assigned by semantic owner in the same move.

The four authz policy rows are load-bearing path identities, not docs. A codemod move that omits their producer-approved regeneration would fail closed or create stale-policy drift. Do not hand-edit generated faces.

## Safe serial sequence

After independent approval and a fresh immutable-tip repeat of these probes:

1. Chat may move alone into a `comms/facade/<owner-approved-leaf>` destination.
2. Meet may move alone into a distinct `comms/facade/<owner-approved-leaf>` destination; the existing `comms/ports/meet-api` stays a separate port seam.
3. Drive may move alone into `storage/facade/<owner-approved-leaf>`.
4. Forms may move alone into `workflow/facade/<owner-approved-leaf>`.

Each move is one capability-lane codemod transaction with Buck2-authoritative build/test evidence, policy/catalog producer regeneration, exact importer rewrites, and semantic non-code co-move. Do not batch all four merely because their source directory is shared.

## What remains unresolved

- Exact leaf spelling and whether `workspace-` remains in package names.
- Whether each capability owner wants the API boundary as a sibling leaf or a reviewed merge into an existing facade leaf.
- Independent approval of the owner split and this collision proof.
- Fresh collision/importer proof immediately before execution.

These do not justify an eight-row `application` move plan or a generic `app/workspace` root.

## Non-actions and non-claims

- No `app/workspace`, `app/application`, or shared workspace suite root.
- No exact destination leaf selected.
- No package renamed.
- No move-plan JSON.
- No policy, generated artifact, or frozen baseline edited.
- No deletion or source-directory cleanup.
- No independent APPROVE; transport failure remains non-approval.
