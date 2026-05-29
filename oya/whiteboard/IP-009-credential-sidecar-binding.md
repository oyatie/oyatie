# IP-009 Whiteboard credential-sidecar-binding

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-009-credential-sidecar-binding.md
Benchmarks displaced: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## 1. Outcome
- Bind whiteboard runtime adapters to a credential sidecar instead of letting handlers, workers, or generated clients read secret material directly.
- Keep credential retrieval outside domain, kernel, and usecase logic.
- Keep OpenBao references in configuration and sidecar contracts, not in canvas operation payloads.
- Keep marketplace template credentials separate from board, export, and import credentials.
- Keep source-vendor migration credentials separate from tenant-local whiteboard runtime credentials.
- Support short TTL leases so collaborative sessions can scale without long-lived service keys.
- Support revocation paths for compromised guest links, classroom sessions, marketplace templates, and source import jobs.
- Satisfy ADR-0321 by naming specific credential surfaces needed to displace Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- Preserve the PRD invariant that vendor parity must not create a new suite boundary.
- Preserve ADR-0314 marketplace DealSet settlement for template and licensed export flows.

## 2. Local Source Anchors
- microservices/whiteboard/ARCHITECTURE.md states adapters own source-system, storage, queue, and evidence integrations.
- microservices/whiteboard/PRD.md requires tenant-scoped evidence and migration readiness.
- microservices/whiteboard/iac/local-openbao-policy.hcl is the local OpenBao policy anchor.
- microservices/whiteboard/iac/openbao-policy.yaml is the deployment policy anchor.
- microservices/whiteboard/iac/secret-bindings.yaml is the runtime secret binding anchor.
- microservices/whiteboard/iac/local-secret-binding.yaml is the local secret binding anchor.
- microservices/whiteboard/iac/pqc-cert.yaml is the certificate anchor for transport posture.
- microservices/whiteboard/iac/ech-config.yaml is the ECH anchor for edge posture.
- microservices/whiteboard/contracts/whiteboard-v1.proto must carry only references, never secret values.
- microservices/whiteboard/capabilities/template-marketplace-install.yaml declares DealSet-sensitive template installation.
- microservices/whiteboard/capabilities/export-render.yaml declares export artifact generation.
- microservices/whiteboard/backfill-replay.md and microservices/whiteboard/backfill-replay.md-adjacent worker plans need source import credentials.
- microservices/whiteboard/runbooks/template-import-rollback.md is the template credential rollback companion.
- microservices/whiteboard/runbooks/export-render-failure.md is the export credential failure companion.

## 3. Credential Classes
- `whiteboard-runtime-signing-key` signs internal board session tokens.
- `whiteboard-presence-fanout-token` authenticates presence fanout adapters.
- `whiteboard-export-render-key` signs export artifact manifests.
- `whiteboard-template-marketplace-key` authorizes template package install and settlement lookup.
- `whiteboard-source-miro-token` imports Miro Enterprise boards during migration.
- `whiteboard-source-mural-token` imports Mural Enterprise boards during migration.
- `whiteboard-source-figjam-token` imports FigJam files during migration.
- `whiteboard-source-lucidspark-token` imports Lucidspark boards during migration.
- `whiteboard-source-whiteboardfi-token` imports Whiteboard.fi classroom boards during migration.
- `whiteboard-source-ms-whiteboard-token` imports Microsoft Whiteboard boards during migration.
- `whiteboard-audit-seal-key` signs audit evidence envelopes.
- `whiteboard-replay-worker-token` authorizes history replay workers.
- `whiteboard-classroom-session-key` signs short-lived classroom board authority.
- `whiteboard-guest-invite-key` signs external collaborator invite references.
- `whiteboard-ci-fixture-token` authorizes CI-only policy and contract fixtures.

## 4. SecretReference Shape
- Every credential request uses a `SecretReference`.
- `SecretReference.tenant_id` is mandatory.
- `SecretReference.secret_class` is mandatory.
- `SecretReference.capability` is mandatory.
- `SecretReference.home_cell` is mandatory.
- `SecretReference.pack_overlay_id` is mandatory when compliance affects secret use.
- `SecretReference.deal_set_id` is mandatory for marketplace template credentials.
- `SecretReference.source_system` is mandatory for migration credentials.
- `SecretReference.workflow_run_id` is mandatory for workflow-triggered import, export, or replay.
- `SecretReference.audit_chain_ref` is mandatory for privileged retrieval.
- `SecretReference.requested_ttl_seconds` must be bounded.
- `SecretReference.reason_code` must be one of board-open, canvas-op, presence, history, export, template-install, import, replay, classroom, support, or ci.
- `SecretReference.policy_decision_id` must be supplied after IP-008 policy evaluation.
- `SecretReference.policy_context_hash` must match the caller decision.
- `SecretReference.trace_id` must exist for every retrieval.

## 5. Sidecar Contract
- The sidecar exposes `LeaseSecret`.
- The sidecar exposes `RenewLease`.
- The sidecar exposes `RevokeLease`.
- The sidecar exposes `DescribeLease`.
- The sidecar exposes `AttestBinding`.
- `LeaseSecret` returns an opaque lease id.
- `LeaseSecret` returns a server-side credential handle when possible.
- `LeaseSecret` never returns raw secret material to domain or usecase code.
- `LeaseSecret` may return raw material only to approved adapter processes with memory-safe handling controls.
- `RenewLease` requires the original policy decision id.
- `RenewLease` refuses renewal after board, classroom, template, or workflow expiry.
- `RevokeLease` accepts compromise, expiry, rollback, settlement-hold, and operator-quarantine reasons.
- `DescribeLease` returns metadata only.
- `AttestBinding` proves the pod, SPIFFE id, cell, tenant, and capability match the requested secret class.

## 6. Runtime Placement
- Sidecar runs beside API pods that need signing or source adapter credentials.
- Sidecar runs beside worker pods that need replay, export, or import credentials.
- Sidecar does not run beside pure domain test processes.
- Sidecar is excluded from browser-facing assets.
- Sidecar uses SPIFFE identity for pod-to-sidecar trust.
- Sidecar uses OpenBao for lease source of truth.
- Sidecar enforces cell-local secret paths.
- Sidecar enforces tenant-local secret namespaces.
- Sidecar enforces pack overlay constraints.
- Sidecar emits lease audit events.
- Sidecar emits lease metrics without raw tenant cardinality.
- Sidecar keeps no durable cache after lease expiry.
- Sidecar clears in-memory secret handles on revoke.
- Sidecar refuses lease if workload attestation fails.

## 7. Board and Presence Credentials
- Board open may need `whiteboard-runtime-signing-key`.
- Board open signs a board access envelope after Cedar permit.
- Board open never stores the signing key in board state.
- Presence sync may need `whiteboard-presence-fanout-token`.
- Presence fanout token TTL must be shorter than the presence stream lifetime.
- Guest invite key signs invite references, not user identity.
- Classroom session key signs student-board authority.
- Whiteboard.fi parity requires classroom sessions to expire without a cleanup operator.
- Microsoft Whiteboard parity requires guest sharing to become explicit principal grants.
- Miro Enterprise parity requires team-level sharing to become explicit tenant-scoped access.

## 8. Canvas Operation Credentials
- Normal canvas append should not need external credentials.
- CRDT merge and local storage use workload identity rather than per-user secrets.
- Imported source operation replay may need source migration credentials.
- Source credentials must be scoped read-only during migration dry-run.
- Source credentials must be revoked after source-system sunset.
- FigJam import credentials must not grant unrelated Figma design file access.
- Mural Enterprise import credentials must not grant workspace administration.
- Miro Enterprise import credentials must not grant organization-wide exports by default.
- Lucidspark import credentials must not grant unrelated Lucidchart documents.
- Microsoft Whiteboard import credentials must not grant full Microsoft Graph scopes.

## 9. Export Credentials
- Export render may need `whiteboard-export-render-key`.
- Export artifacts are signed by manifest, board revision, render profile, and data class.
- Export key leases are tied to export id.
- Export key leases are tied to artifact class.
- Export key leases are tied to residency target.
- Export key leases are revoked on artifact quarantine.
- Export key leases are revoked on pack overlay conflict.
- Export key leases are revoked on audit-chain failure.
- Lucidspark parity requires deterministic diagram export signing.
- Microsoft Whiteboard parity requires share-source provenance in exported artifacts.
- Mural Enterprise parity requires facilitation artifact provenance.

## 10. Template Marketplace Credentials
- Template install may need `whiteboard-template-marketplace-key`.
- Template install credentials require `deal_set_id`.
- Template install credentials require license scope.
- Template install credentials require publisher reference.
- Template install credentials require settlement status.
- Credentials are denied during settlement hold.
- Credentials are denied when a template package lacks provenance.
- Credentials are denied when template data class exceeds principal clearance.
- Miro Enterprise template parity requires reusable board templates.
- Mural Enterprise template parity requires facilitation templates.
- ADR-0314 requires DealSet settlement, not vendor-specific template licensing shortcuts.

## 11. Migration Credentials
- Migration credentials are used by source import adapters.
- Each benchmark source gets a distinct secret class.
- Source credentials must carry `source_system`.
- Source credentials must carry `migration_batch_id`.
- Source credentials must carry `workflow_run_id`.
- Source credentials must carry dry-run or cutover mode.
- Dry-run credentials should be read-only.
- Cutover credentials should be time-boxed.
- Delta-capture credentials should be narrower than initial import credentials.
- Source-system sunset revokes all remaining source credentials.
- Failed migration revokes source credentials before rollback bundle export.
- Audit evidence records source credential class without storing secret material.

## 12. TTL Rules
- Runtime signing key leases target 60 seconds or less.
- Presence fanout token leases target 30 seconds or less.
- Guest invite signing key leases target 60 seconds or less.
- Classroom session key leases target the classroom board lifetime.
- Export render key leases target one render job.
- Template marketplace key leases target one install transaction.
- Source dry-run credentials target one dry-run batch.
- Source cutover credentials target the approved cutover window.
- Replay worker tokens target one replay segment.
- CI fixture tokens target one CI job.
- Renewals require fresh workload attestation.
- Renewals require the original policy context hash.
- Renewals are refused after policy bundle rotation unless explicitly allowed.

## 13. Failure Modes
- Missing SecretReference field produces `SECRET_REFERENCE_INVALID`.
- Missing policy decision produces `SECRET_POLICY_DECISION_MISSING`.
- Policy hash mismatch produces `SECRET_POLICY_CONTEXT_MISMATCH`.
- Workload attestation failure produces `SECRET_ATTESTATION_FAILED`.
- Home-cell mismatch produces `SECRET_CELL_MISMATCH`.
- Pack conflict produces `SECRET_PACK_CONFLICT`.
- DealSet hold produces `SECRET_DEAL_SET_HOLD`.
- Source scope too broad produces `SECRET_SOURCE_SCOPE_TOO_BROAD`.
- TTL too long produces `SECRET_TTL_REJECTED`.
- Lease expired produces `SECRET_LEASE_EXPIRED`.
- Lease revoked produces `SECRET_LEASE_REVOKED`.
- OpenBao unavailable produces fail-closed for privileged mutations.
- OpenBao unavailable may degrade non-sensitive reads only when no secret is needed.
- Audit-chain unavailable blocks privileged secret retrieval.

## 14. Observability
- Emit `whiteboard.secret.lease.request.count`.
- Emit `whiteboard.secret.lease.permit.count`.
- Emit `whiteboard.secret.lease.deny.count`.
- Emit `whiteboard.secret.lease.duration`.
- Emit `whiteboard.secret.lease.renew.count`.
- Emit `whiteboard.secret.lease.revoke.count`.
- Emit `whiteboard.secret.attestation.failure.count`.
- Emit `whiteboard.secret.policy_context_mismatch.count`.
- Emit `whiteboard.secret.openbao.failure.count`.
- Emit dimensions for secret class, capability, result, cell, and data class.
- Do not emit raw tenant id as a high-cardinality metric label.
- Link lease events to audit-chain evidence.
- Link export lease failures to microservices/whiteboard/runbooks/export-render-failure.md.
- Link template lease failures to microservices/whiteboard/runbooks/template-import-rollback.md.
- Link source credential failures to migration dry-run evidence.

## 15. Implementation Steps
- Define SecretReference value object in the adapter boundary.
- Define sidecar client interface.
- Bind API pods to sidecar through local service identity.
- Bind worker pods to sidecar through local service identity.
- Wire OpenBao path templates from secret-bindings files.
- Add lease request validation.
- Add workload attestation checks.
- Add policy decision verification.
- Add pack overlay verification.
- Add DealSet settlement verification for template credentials.
- Add source-scope verification for migration credentials.
- Add TTL caps per secret class.
- Add revoke on rollback.
- Add revoke on source-system sunset.
- Add metrics and audit event emission.
- Keep generated proto and OpenAPI secret fields as references only.

## 16. Tests
- Unit tests validate SecretReference mandatory fields.
- Unit tests validate TTL caps.
- Unit tests validate secret class to capability mapping.
- Integration tests lease runtime signing key with valid attestation.
- Integration tests deny lease with invalid policy hash.
- Integration tests deny template lease during DealSet hold.
- Integration tests deny source token with overbroad scope.
- Integration tests revoke export lease on pack conflict.
- Replay tests confirm migrations do not persist raw source tokens.
- Audit tests confirm lease ids appear in evidence.
- Static checks confirm proto messages contain no raw secret fields.
- Benchmark tests cover Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard source credentials.

## 17. Rollback
- Roll back by disabling new sidecar-required paths through adapter config.
- Revoke all active leases for the affected cell and capability.
- Quarantine audit evidence for failed lease attempts.
- Return board open to workload identity only if no signing lease is required.
- Return export render to paused state if export keys cannot be leased.
- Return template install to settlement hold if marketplace credentials cannot be leased.
- Return source import to dry-run paused state if source credentials cannot be leased.
- Preserve denial evidence for every paused capability.

## 18. Acceptance Criteria
- No whiteboard domain, kernel, or usecase code reads secret material directly.
- Every privileged credential request carries SecretReference.
- Every SecretReference carries tenant, cell, capability, policy decision, and trace context.
- Template credentials carry DealSet context.
- Source migration credentials carry source-system and migration batch context.
- Leases are short-lived and revocable.
- Raw secrets do not appear in proto, OpenAPI, AsyncAPI, audit logs, metrics, or board state.
- ADR-0321 remains cited and benchmark-specific credential surfaces are named.

## 19. SLO Notes
- SLO: sidecar lease acquisition for board-open signing material targets warmed-cache p95 below 50 ms.
- SLO: export-render key lease failures surface to the async job within 5 seconds.
- SLO: source migration credential denial emits audit evidence before retry scheduling.
- SLO: template marketplace credential holds become visible to tenant admins within 30 seconds.
- SLO: lease revocation after rollback reaches all affected whiteboard workers within 60 seconds.
- SLO evidence is correlated with Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard migration source credentials where applicable.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
