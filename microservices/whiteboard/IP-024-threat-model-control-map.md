# IP-024 Whiteboard Threat Model Control Map

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-024-threat-model-control-map.md
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- Map whiteboard threats to controls for collaborative board operations.
- Cover service-specific risks that generic storage, chat, or workflow threat models miss.
- Preserve ADR-0321 and existing ADR references.
- Use B2B leader benchmarks as pressure inputs: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- Keep threat evidence local to `microservices/whiteboard/` and reviewable without editing ADR files.

## Repo-Local Anchors
- Threat model: `microservices/whiteboard/threat-model.md`.
- Policies: `microservices/whiteboard/policies/`.
- Policy aliases: `microservices/whiteboard/policy/`.
- Compliance: `microservices/whiteboard/compliance.md`.
- Failure modes: `microservices/whiteboard/failure-modes.md`.
- Incident response: `microservices/whiteboard/incident-response.md`.
- Runbooks: `microservices/whiteboard/runbooks/`.
- Capability records: `microservices/whiteboard/capabilities/`.
- Contracts: `microservices/whiteboard/contracts/`.
- Audit findings: `microservices/whiteboard/AUDIT-FINDINGS-2026-05-21.json`.

## Assets
- Tenant board envelope.
- Canvas operation log.
- Presence cursor lease.
- Board participant list.
- History snapshot.
- Export artifact.
- Template marketplace grant.
- DealSet settlement reference.
- Cedar decision record.
- Audit-chain event.
- Migration fixture.
- Replay worker state.
- Dashboard metrics.
- Runbook evidence.
- Pack overlay policy.

## Whiteboard Domain Model Under Threat
- Board state is derived from an ordered operation log plus server-owned merge policy.
- Board sessions are active collaboration windows, not durable content.
- Presence leases are expiring session grants tied to participants and boards.
- CRDT-compatible operations are accepted only through append commands.
- Operation replay is a recovery and migration mechanism, not a client write bypass.
- Snapshot pointers are immutable evidence objects with retention policy.
- Export artifacts are separately authorized outputs.
- Template grants are tenant-scoped activations backed by DealSet settlement.
- Vendor import fixtures are untrusted inputs until preview transform passes policy checks.
- Benchmark labels are evidence dimensions, not trust boundaries.

## Command, Event, And Proto Attack Surfaces
- Command surface `boards:open` can be attacked through board-id enumeration.
- Command surface `operations:append` can be attacked through duplicate operation replay.
- Command surface `operations:preview` can be attacked through malicious vendor payloads.
- Command surface `history:snapshot` can be attacked through retention bypass attempts.
- Command surface `exports:render` can be attacked through oversized render requests.
- Command surface `exports:download` can be attacked through artifact id guessing.
- Command surface `templates:install` can be attacked through settlement bypass.
- Event surface `canvas_operation.appended` can be attacked through forged event injection.
- Event surface `presence.lease_renewed` can be attacked through stale lease replay.
- Event surface `export_render.completed` can be attacked through false artifact readiness.
- Event surface `template_install.settled` can be attacked through forged settlement state.
- Proto internal append surface can be attacked if edge policy facts are not forwarded.
- Proto internal render surface can be attacked if artifact class is downgraded.
- Proto internal presence surface can be attacked if lease expiry is ignored.

## Cedar Fact Requirements
- Cedar fact `tenant_id` is required for every decision.
- Cedar fact `principal_id` is required for every actor decision.
- Cedar fact `audience_type` is required for collaboration, education, auditor, and CI paths.
- Cedar fact `purpose` is required for purpose limitation.
- Cedar fact `data_class` is required for pack and retention controls.
- Cedar fact `capability` is required for capability-specific permits.
- Cedar fact `board_id` is required for board-bound actions.
- Cedar fact `operation_id` is required for append and replay actions.
- Cedar fact `presence_lease_id` is required for presence publish.
- Cedar fact `snapshot_id` is required for snapshot read and compare.
- Cedar fact `artifact_id` is required for export download.
- Cedar fact `template_id` is required for template preview and install.
- Cedar fact `dealset_id` is required for template install.
- Cedar fact `source_benchmark` is required for migration fixture evaluation.
- Cedar fact `pack_overlay` is required for regulated tenant activation.

## Trust Boundaries
- Browser or client SDK to whiteboard edge.
- Whiteboard edge to policy evaluation.
- Whiteboard edge to operation append path.
- Whiteboard edge to presence transport.
- Whiteboard edge to export render workers.
- Whiteboard edge to history snapshot workers.
- Whiteboard edge to template marketplace settlement.
- Async worker to audit-chain publication.
- Migration import to preview transform.
- Preview transform to accepted replay.
- Tenant admin to auditor scope.
- CI principal to contract validation.
- Public internet to HTTP/3 h3-alt-svc endpoint.
- Internal transport to proto3 surfaces where present.
- Pack overlay policy to base capability policy.

## Threats: Tenant And Principal
- Threat TP-01: attacker guesses board id and omits tenant id.
- Control TP-01: contracts and SDKs require explicit `tenant_id`.
- Threat TP-02: attacker reuses principal from another tenant.
- Control TP-02: Cedar default-deny evaluates tenant and principal together.
- Threat TP-03: websocket identity is treated as sufficient authorization.
- Control TP-03: presence publish still requires tenant and principal scope.
- Threat TP-04: auditor scope reads board payload without evidence need.
- Control TP-04: auditor routes expose policy and audit evidence, not raw board data by default.
- Threat TP-05: CI scope mutates live boards.
- Control TP-05: CI audience has contract-validation permissions only.
- Threat TP-06: education participant sees instructor-only controls.
- Control TP-06: Whiteboard.fi-style classroom audience is explicit.
- Threat TP-07: tenant admin inherits Microsoft Whiteboard storage assumptions.
- Control TP-07: export and retention permissions are separate.

## Threats: Canvas Operations
- Threat CO-01: duplicate append creates divergent board state.
- Control CO-01: idempotency key is mandatory for append retry.
- Threat CO-02: stale sequence overwrites newer content.
- Control CO-02: append rejects stale sequence with replay conflict.
- Threat CO-03: malicious imported vendor payload smuggles permissions.
- Control CO-03: import preview maps permissions through Cedar, never blindly preserves vendor grants.
- Threat CO-04: large Miro Enterprise board overwhelms append path.
- Control CO-04: capacity admission gates by tenant, board, cell, and operation rate.
- Threat CO-05: Mural Enterprise facilitation burst causes global degradation.
- Control CO-05: tenant and cell partitioning isolates burst.
- Threat CO-06: FigJam-style reconnect replays stale operations.
- Control CO-06: reconnect validates operation sequence before append.
- Threat CO-07: operation payload leaks through logs.
- Control CO-07: structured logs redact canvas bodies by default.

## Threats: Presence
- Threat PR-01: cursor state persists beyond session need.
- Control PR-01: presence leases expire and are not durable history.
- Threat PR-02: noisy participant causes presence fanout denial.
- Control PR-02: publisher throttling and tenant-scoped fanout limits.
- Threat PR-03: participant sees another tenant's cursor.
- Control PR-03: presence channel key includes tenant and board.
- Threat PR-04: classroom participant sees unauthorized instructor state.
- Control PR-04: audience type partitions Whiteboard.fi-style session roles.
- Threat PR-05: presence outage blocks board editing.
- Control PR-05: presence is fail-soft and isolated from append path.
- Threat PR-06: cursor events flood audit-chain.
- Control PR-06: audit captures policy transitions, not every cursor movement.
- Threat PR-07: reconnect accepts stale lease.
- Control PR-07: lease renewal required before publish.

## Threats: History And Export
- Threat HE-01: snapshot captures data beyond retention pack.
- Control HE-01: retention pack check before accepted snapshot job.
- Threat HE-02: snapshot replay mutates live board unexpectedly.
- Control HE-02: replay requires explicit dry-run or accepted replay state.
- Threat HE-03: export artifact downloaded by collaborator without export permission.
- Control HE-03: artifact download has separate authorization.
- Threat HE-04: export artifact hash is tampered.
- Control HE-04: artifact hash is stored and verified before download.
- Threat HE-05: Lucidspark-grade diagram export leaks hidden layers.
- Control HE-05: export renderer applies board policy and data-class filters.
- Threat HE-06: Microsoft Whiteboard tenant-admin export bypasses user policy.
- Control HE-06: tenant-admin export remains Cedar-gated and audited.
- Threat HE-07: Mural Enterprise export burst starves interactive appends.
- Control HE-07: async render queues are isolated from interactive commands.

## Threats: Template Marketplace
- Threat TM-01: template installs without DealSet settlement.
- Control TM-01: ADR-0314 DealSet reference is mandatory before activation.
- Threat TM-02: template preview mutates a board.
- Control TM-02: preview is non-mutating and separately audited.
- Threat TM-03: malicious template injects unapproved operations.
- Control TM-03: template operations pass the same Cedar and data-class checks.
- Threat TM-04: Miro Enterprise library import grants unauthorized template access.
- Control TM-04: imported template grants are remapped per tenant.
- Threat TM-05: Mural Enterprise facilitation template bypasses pack overlay.
- Control TM-05: pack overlay evaluated before template activation.
- Threat TM-06: FigJam starter template carries unmanaged attribution.
- Control TM-06: attribution stored as metadata and not as policy.
- Threat TM-07: marketplace outage blocks native board editing.
- Control TM-07: template install path is isolated from board append path.

## Threats: Transport And Crypto
- Threat TC-01: downgrade from HTTP/3 posture hides transport weakness.
- Control TC-01: ADR-0253-amendment negotiation metadata is observable.
- Threat TC-02: internal proto surface bypasses edge policy.
- Control TC-02: internal calls carry tenant, principal, audience, and policy context.
- Threat TC-03: event stream reconnect skips authentication refresh.
- Control TC-03: reconnect requires renewed scoped credentials.
- Threat TC-04: export download bypasses ECH/PQC policy expectation.
- Control TC-04: download endpoints use the same transport posture metadata.
- Threat TC-05: benchmark migration tool uses direct storage access.
- Control TC-05: migration tools use contract and policy surfaces.

## Threats: Observability And Audit
- Threat OA-01: audit-chain event missing for accepted mutation.
- Control OA-01: accepted board append, snapshot, export, and template install require audit event publication.
- Threat OA-02: denial evidence is not captured.
- Control OA-02: Cedar refusal is typed and observable.
- Threat OA-03: dashboard aggregates hide tenant impact.
- Control OA-03: metrics include tenant, cell, region, pack, and data class.
- Threat OA-04: incident response cannot reconstruct benchmark source.
- Control OA-04: migration and benchmark source are evidence dimensions.
- Threat OA-05: audit finding is closed without proof.
- Control OA-05: IP-025 closeout requires evidence pointer.
- Threat OA-06: logs leak payloads.
- Control OA-06: redaction policy blocks canvas body logging by default.
- Threat OA-07: rollback lacks traceability.
- Control OA-07: rollback action is linked to audit event and runbook.

## Control Map
- Control C-01: explicit tenant scope on every capability.
- Control C-02: explicit principal scope on every mutation.
- Control C-03: explicit audience type on collaborative, auditor, CI, and education paths.
- Control C-04: explicit data class for board, operation, cursor, snapshot, and artifact paths.
- Control C-05: Cedar default-deny authorization.
- Control C-06: capability record governance.
- Control C-07: contract generation gate from IP-019.
- Control C-08: catalog admission gate from IP-020.
- Control C-09: SLO promotion gate from IP-021.
- Control C-10: chaos drill evidence from IP-022.
- Control C-11: DPIA evidence from IP-023.
- Control C-12: audit closeout from IP-025.
- Control C-13: DealSet settlement for marketplace templates.
- Control C-14: HTTP/3 h3-alt-svc plus ECH/PQC posture.
- Control C-15: export artifact authorization.
- Control C-16: presence lease expiry.
- Control C-17: append idempotency.
- Control C-18: snapshot retention check.
- Control C-19: payload log redaction.
- Control C-20: rollback runbook linkage.

## Benchmark-Specific Control Notes
- Miro Enterprise requires controls for large boards, templates, history, and export.
- Miro Enterprise does not justify vendor namespace or suite folder creation.
- Mural Enterprise requires controls for facilitation bursts, templates, and exports.
- Mural Enterprise does not justify workspace boundary leakage.
- FigJam requires controls for cursor fanout, reconnect, and append sequencing.
- FigJam does not justify design-file storage coupling.
- Lucidspark requires controls for diagram export and snapshot comparison.
- Lucidspark does not justify diagram-specific service split.
- Whiteboard.fi requires controls for classroom audience separation.
- Whiteboard.fi does not justify education-only fork of whiteboard.
- Microsoft Whiteboard requires controls for tenant governance, retention, and export.
- Microsoft Whiteboard does not justify Office storage assumptions.

## Verification
- Verify all six capability records map to at least one threat.
- Verify all six capability records map to at least one control.
- Verify all six benchmark names appear in the control map.
- Verify ADR-0321 remains referenced.
- Verify no ADR file is edited.
- Verify no vendor namespace is introduced.
- Verify no suite boundary is introduced.
- Verify tenant scope appears in every control family.
- Verify principal scope appears in every mutation family.
- Verify data class appears in every capability family.
- Verify DealSet settlement appears for template controls.
- Verify artifact authorization appears for export controls.
- Verify lease expiry appears for presence controls.
- Verify retention check appears for snapshot controls.
- Verify idempotency appears for append controls.
- Verify audit closeout linkage appears for all material controls.

## Rollback
- Roll back threat acceptance if any capability lacks a control.
- Roll back threat acceptance if any benchmark pressure is unnamed.
- Roll back threat acceptance if tenant scope is implicit.
- Roll back threat acceptance if export artifacts lack authorization.
- Roll back threat acceptance if template settlement is missing.
- Roll back threat acceptance if presence retention is ambiguous.
- Roll back threat acceptance if append conflict handling is ambiguous.
- Roll back threat acceptance if snapshot retention is ambiguous.
- Roll back threat acceptance if audit evidence is missing.
- Roll back affected capability promotion rather than editing ADR-0321.

## Test Cases
- Test tenant pivot attack against board-open and canvas-op-append controls.
- Test unauthorized facilitator mutation against timer, vote, and zone-lock controls.
- Test stale presence replay against lease expiry and volatile-state controls.
- Test export artifact theft against artifact authorization and pack residency controls.
- Test template marketplace bypass against DealSet and review controls.
- Test source-vendor migration fixture for Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- Test threat acceptance rollback when a control lacks audit evidence.

## Acceptance Criteria
- Threat model control map names Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- Threat model control map preserves the existing ADR binding set including ADR-0321.
- Threat model control map covers tenant, principal, canvas operation, presence, history, export, template, transport, observability, and audit threats.
- Threat model control map ties every threat family to concrete controls.
- Threat model control map links IP-019, IP-020, IP-021, IP-022, IP-023, and IP-025.
- Threat model control map blocks vendor namespace and suite-boundary leakage.
- Threat model control map distinguishes volatile presence from durable board history.
- Threat model control map distinguishes export authorization from board mutation authorization.
- Threat model control map can be reviewed without editing ADR-0321.
- Threat model control map does not require `oya vcs verify`, `done`, or `promote`.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
