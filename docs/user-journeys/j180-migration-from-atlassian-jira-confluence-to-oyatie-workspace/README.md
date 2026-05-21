---
doc_class: User-Journey-README
journey_id: j180-migration-from-atlassian-jira-confluence-to-oyatie-workspace
slice: vendor-migration-journey-wave-3-j
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Nora Stein, VP Engineering at AtlasBridge Robotics
audience_type: B2B_ENGINEERING_VP
incumbent_system: Atlassian Jira Software plus Confluence
target_system: Oyatie workspace
source_system: atlassian-cloud-site-atlasbridge
related_adrs:
  - ADR-0131-per-microservice-flat-layout
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0263-observability-emission-contract
  - ADR-0317-role-based-projection-unified-ux-shell
microservice_count: 15
---

# j180-migration-from-atlassian-jira-confluence-to-oyatie-workspace - Atlassian Jira and Confluence to Oyatie Workspace cutover

## At a glance

Nora Stein, VP Engineering at AtlasBridge Robotics leads a migration from Atlassian Jira Software plus Confluence to Oyatie workspace. The journey is not a generic persona story; it is a vendor exit path where the protagonist must preserve operational continuity while replacing named incumbent objects, APIs, permissions, reports, dashboards, and audit evidence.

- Incumbent: Atlassian Jira Software plus Confluence.
- Target: Oyatie workspace.
- Company: AtlasBridge Robotics.
- Migration window: Jira project, workflow, permission, and Confluence knowledge-space cutover.
- Extract mechanism: Jira Cloud REST export plus Confluence space export with attachment manifest.
- Named projection: oyatie.workspace.delivery_graph_projection_v1.
- Parallel-run posture: one-sprint parallel-run with frozen workflow-scheme changes and Confluence read-only mirror.
- Stop condition: Oyatie is active, incumbent writes are frozen, rollback remains rehearsed, and all deltas are below go/no-go thresholds.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| README.md | Persona context, µservice roster, ADRs, regulatory anchors, acceptance summary | Names incumbent objects, target projection, and cutover gates |
| story.md | Full migration narrative with named milestones | Minute-by-minute migration texture, not a scaffold |
| handshake.md | Every cross-µservice and vendor-API interaction | Names caller, callee, payload, Cedar permit, audit event, and rollback |
| ux-flow.md | Migration-tool screens, progress dashboards, rollback options | Names operator controls, status states, accessibility, and failure surfaces |
| integration-test-plan.md | Verification and go/no-go plan | Parallel-run delta detection, phase gates, and rollback tests |
| schemas/cedar-policy.cedar | Authorization fragment | Principal/action/resource policy for cutover operations |
| schemas/journey-messages.proto | RPC/event contract | Migration commands, events, delta records, rollback requests |
| schemas/migration-state-machine.yaml | Lifecycle state machine | Phase transitions and terminal states |
| schemas/vendor-extract-schema.json | Source extract contract | Vendor object schema and row-hash expectations |
| schemas/cutover-runbook.json | Machine-readable cutover runbook | Hour-by-hour tasks, owners, commands, gates |

## Primary protagonist

Nora Stein, VP Engineering at AtlasBridge Robotics is accountable for the business outcome. The executive question is whether AtlasBridge Robotics can operate on Monday, produce defensible audit evidence, and explain the decision when Atlassian Jira Software plus Confluence becomes read-only.

## ADR anchors

| ADR | How it constrains this migration |
|---|---|
| ADR-0131-per-microservice-flat-layout | Requires tenant-scoped, Cedar-gated, auditable transitions. |
| ADR-0145-inter-microservice-communication-reform | Constrains µservice boundaries, event emission, and role-projected UX. |
| ADR-0243-cedar-as-universal-gate | Requires tenant-scoped, Cedar-gated, auditable transitions. |
| ADR-0244-tenant-as-universal-scoping-primitive | Constrains µservice boundaries, event emission, and role-projected UX. |
| ADR-0251-compliance-pack-cell-certification-levels | Requires tenant-scoped, Cedar-gated, auditable transitions. |
| ADR-0263-observability-emission-contract | Constrains µservice boundaries, event emission, and role-projected UX. |
| ADR-0317-role-based-projection-unified-ux-shell | Requires tenant-scoped, Cedar-gated, auditable transitions. |

## µservice roster

| µservice | Role | Migration responsibility |
|---|---|---|
| workspace | primary | Owns epic migration state for Issue types during jira-rest-export. |
| tasks | primary | Owns story migration state for Workflow schemes during workflow-permission-freeze. |
| notes | primary | Owns bug migration state for Permission schemes during confluence-space-load. |
| docs | primary | Owns task migration state for Confluence space during sprint-parallel-run. |
| drive | primary | Owns sprint migration state for Project board during workspace-cutover. |
| identity | supporting | Owns space migration state for Issue types during jira-rest-export. |
| tenancy | supporting | Owns page migration state for Workflow schemes during workflow-permission-freeze. |
| workflow-engine | supporting | Owns attachment migration state for Permission schemes during confluence-space-load. |
| audit-chain | supporting | Owns epic migration state for Confluence space during sprint-parallel-run. |
| observability | supporting | Owns story migration state for Project board during workspace-cutover. |
| search | supporting | Owns bug migration state for Issue types during jira-rest-export. |
| messenger | supporting | Owns task migration state for Workflow schemes during workflow-permission-freeze. |
| connect | supporting | Owns sprint migration state for Permission schemes during confluence-space-load. |
| compliance | supporting | Owns space migration state for Confluence space during sprint-parallel-run. |
| ops-dashboard-control-center | supporting | Owns page migration state for Project board during workspace-cutover. |

## Incumbent object roster

| Incumbent object/table | Purpose | Named fields | Oyatie landing projection |
|---|---|---|---|
| Issue types | Jira issue classification | Epic, Story, Task, Bug, Incident, Change, Spike | oyatie.workspace.delivery_graph_projection_v1 |
| Workflow schemes | Project workflow binding | To Do, In Progress, Code Review, QA, Blocked, Done | oyatie.workspace.delivery_graph_projection_v1 |
| Permission schemes | Project permission model | Browse, Create, Edit, Transition, Administer, Worklog | oyatie.workspace.delivery_graph_projection_v1 |
| Confluence space | Knowledge-space container | Space key, page tree, labels, restrictions, attachments | oyatie.workspace.delivery_graph_projection_v1 |
| Project board | Scrum/Kanban board | Board id, filter JQL, sprint, swimlane, quick filters | oyatie.workspace.delivery_graph_projection_v1 |

## Field-mapping table

| Source field | Oyatie field | Transform rule | Evidence |
|---|---|---|---|
| Jira issue key | tasks.source_issue_key | retain ABC-123 identifier in backlink | audit-chain source hash and row-count proof required |
| Issue type | tasks.work_item_type | map Epic/Story/Task/Bug/Spike | audit-chain source hash and row-count proof required |
| Workflow status | workflow-engine.delivery_state | map through signed workflow scheme | audit-chain source hash and row-count proof required |
| Permission scheme role | identity.project_role_grant | preserve admin/developer/viewer split | audit-chain source hash and row-count proof required |
| Sprint | tasks.iteration_id | map active and closed sprint history | audit-chain source hash and row-count proof required |
| Confluence space key | notes.space_id | space to notes µservice namespace | audit-chain source hash and row-count proof required |
| Confluence page id | notes.note_id | preserve page tree and backlinks | audit-chain source hash and row-count proof required |
| Attachment id | drive.attachment_id | hash and WORM where policy requires | audit-chain source hash and row-count proof required |

## Replacement surface map

- Jira Scrum Board -> Oyatie Sprint Board.
- Jira Issue View -> Oyatie Work Item Drawer.
- Jira Workflow Scheme Editor -> Oyatie Delivery Workflow Editor.
- Jira Permission Scheme -> Oyatie Project Access Matrix.
- Confluence Space -> Oyatie notes µservice space.
- Confluence Page Tree -> Oyatie Docs/Notes Graph.

## Named regulatory anchors

1. SOX Section 404 software change-control evidence for release approvals.
2. SOC 2 CC8.1 change management and CC6.6 access-control evidence.
3. GDPR Articles 30 and 32 for employee and customer data in work items and pages.
4. SEC Rule 17a-4(f) WORM retention for regulated engineering communications where enabled.
5. ISO/IEC 27001 Annex A 5.37 documented operating procedures and A 8.15 logging.

## Named milestones

- M1 Jira issue type, workflow scheme, and permission scheme inventory complete.
- M2 Confluence spaces exported with attachments.
- M3 notes µservice mapping signed for every Confluence space.
- M4 one-sprint parallel-run delta accepted.
- M5 Atlassian site read-only and Oyatie Workspace active.

## Acceptance summary

| AC | Required result | Evidence |
|---|---|---|
| AC-J180-001 | workspace proves Issue types migration during jira-rest-export; SOX Section 404 software change-control evidence for release approvals remains satisfied. | EVT-J180-WORKSPACE-001 plus row-count and hash proof. |
| AC-J180-002 | tasks proves Workflow schemes migration during workflow-permission-freeze; SOC 2 CC8.1 change management and CC6.6 access-control evidence remains satisfied. | EVT-J180-TASKS-002 plus row-count and hash proof. |
| AC-J180-003 | notes proves Permission schemes migration during confluence-space-load; GDPR Articles 30 and 32 for employee and customer data in work items and pages remains satisfied. | EVT-J180-NOTES-003 plus row-count and hash proof. |
| AC-J180-004 | docs proves Confluence space migration during sprint-parallel-run; SEC Rule 17a-4(f) WORM retention for regulated engineering communications where enabled remains satisfied. | EVT-J180-DOCS-004 plus row-count and hash proof. |
| AC-J180-005 | drive proves Project board migration during workspace-cutover; ISO/IEC 27001 Annex A 5.37 documented operating procedures and A 8.15 logging remains satisfied. | EVT-J180-DRIVE-005 plus row-count and hash proof. |
| AC-J180-006 | identity proves Issue types migration during jira-rest-export; SOX Section 404 software change-control evidence for release approvals remains satisfied. | EVT-J180-IDENTITY-006 plus row-count and hash proof. |
| AC-J180-007 | tenancy proves Workflow schemes migration during workflow-permission-freeze; SOC 2 CC8.1 change management and CC6.6 access-control evidence remains satisfied. | EVT-J180-TENANCY-007 plus row-count and hash proof. |
| AC-J180-008 | workflow-engine proves Permission schemes migration during confluence-space-load; GDPR Articles 30 and 32 for employee and customer data in work items and pages remains satisfied. | EVT-J180-WORKFLOW_ENGINE-008 plus row-count and hash proof. |
| AC-J180-009 | audit-chain proves Confluence space migration during sprint-parallel-run; SEC Rule 17a-4(f) WORM retention for regulated engineering communications where enabled remains satisfied. | EVT-J180-AUDIT_CHAIN-009 plus row-count and hash proof. |
| AC-J180-010 | observability proves Project board migration during workspace-cutover; ISO/IEC 27001 Annex A 5.37 documented operating procedures and A 8.15 logging remains satisfied. | EVT-J180-OBSERVABILITY-010 plus row-count and hash proof. |
| AC-J180-011 | search proves Issue types migration during jira-rest-export; SOX Section 404 software change-control evidence for release approvals remains satisfied. | EVT-J180-SEARCH-011 plus row-count and hash proof. |
| AC-J180-012 | messenger proves Workflow schemes migration during workflow-permission-freeze; SOC 2 CC8.1 change management and CC6.6 access-control evidence remains satisfied. | EVT-J180-MESSENGER-012 plus row-count and hash proof. |
| AC-J180-013 | connect proves Permission schemes migration during confluence-space-load; GDPR Articles 30 and 32 for employee and customer data in work items and pages remains satisfied. | EVT-J180-CONNECT-013 plus row-count and hash proof. |
| AC-J180-014 | compliance proves Confluence space migration during sprint-parallel-run; SEC Rule 17a-4(f) WORM retention for regulated engineering communications where enabled remains satisfied. | EVT-J180-COMPLIANCE-014 plus row-count and hash proof. |
| AC-J180-015 | ops-dashboard-control-center proves Project board migration during workspace-cutover; ISO/IEC 27001 Annex A 5.37 documented operating procedures and A 8.15 logging remains satisfied. | EVT-J180-OPS_DASHBOARD_CONTROL_CENTER-015 plus row-count and hash proof. |
| AC-J180-016 | workspace proves Issue types migration during jira-rest-export; SOX Section 404 software change-control evidence for release approvals remains satisfied. | EVT-J180-WORKSPACE-016 plus row-count and hash proof. |
| AC-J180-017 | tasks proves Workflow schemes migration during workflow-permission-freeze; SOC 2 CC8.1 change management and CC6.6 access-control evidence remains satisfied. | EVT-J180-TASKS-017 plus row-count and hash proof. |
| AC-J180-018 | notes proves Permission schemes migration during confluence-space-load; GDPR Articles 30 and 32 for employee and customer data in work items and pages remains satisfied. | EVT-J180-NOTES-018 plus row-count and hash proof. |
| AC-J180-019 | docs proves Confluence space migration during sprint-parallel-run; SEC Rule 17a-4(f) WORM retention for regulated engineering communications where enabled remains satisfied. | EVT-J180-DOCS-019 plus row-count and hash proof. |
| AC-J180-020 | drive proves Project board migration during workspace-cutover; ISO/IEC 27001 Annex A 5.37 documented operating procedures and A 8.15 logging remains satisfied. | EVT-J180-DRIVE-020 plus row-count and hash proof. |
| AC-J180-021 | identity proves Issue types migration during jira-rest-export; SOX Section 404 software change-control evidence for release approvals remains satisfied. | EVT-J180-IDENTITY-021 plus row-count and hash proof. |
| AC-J180-022 | tenancy proves Workflow schemes migration during workflow-permission-freeze; SOC 2 CC8.1 change management and CC6.6 access-control evidence remains satisfied. | EVT-J180-TENANCY-022 plus row-count and hash proof. |
| AC-J180-023 | workflow-engine proves Permission schemes migration during confluence-space-load; GDPR Articles 30 and 32 for employee and customer data in work items and pages remains satisfied. | EVT-J180-WORKFLOW_ENGINE-023 plus row-count and hash proof. |
| AC-J180-024 | audit-chain proves Confluence space migration during sprint-parallel-run; SEC Rule 17a-4(f) WORM retention for regulated engineering communications where enabled remains satisfied. | EVT-J180-AUDIT_CHAIN-024 plus row-count and hash proof. |
| AC-J180-025 | observability proves Project board migration during workspace-cutover; ISO/IEC 27001 Annex A 5.37 documented operating procedures and A 8.15 logging remains satisfied. | EVT-J180-OBSERVABILITY-025 plus row-count and hash proof. |
| AC-J180-026 | search proves Issue types migration during jira-rest-export; SOX Section 404 software change-control evidence for release approvals remains satisfied. | EVT-J180-SEARCH-026 plus row-count and hash proof. |
| AC-J180-027 | messenger proves Workflow schemes migration during workflow-permission-freeze; SOC 2 CC8.1 change management and CC6.6 access-control evidence remains satisfied. | EVT-J180-MESSENGER-027 plus row-count and hash proof. |
| AC-J180-028 | connect proves Permission schemes migration during confluence-space-load; GDPR Articles 30 and 32 for employee and customer data in work items and pages remains satisfied. | EVT-J180-CONNECT-028 plus row-count and hash proof. |
| AC-J180-029 | compliance proves Confluence space migration during sprint-parallel-run; SEC Rule 17a-4(f) WORM retention for regulated engineering communications where enabled remains satisfied. | EVT-J180-COMPLIANCE-029 plus row-count and hash proof. |
| AC-J180-030 | ops-dashboard-control-center proves Project board migration during workspace-cutover; ISO/IEC 27001 Annex A 5.37 documented operating procedures and A 8.15 logging remains satisfied. | EVT-J180-OPS_DASHBOARD_CONTROL_CENTER-030 plus row-count and hash proof. |
| AC-J180-031 | workspace proves Issue types migration during jira-rest-export; SOX Section 404 software change-control evidence for release approvals remains satisfied. | EVT-J180-WORKSPACE-031 plus row-count and hash proof. |
| AC-J180-032 | tasks proves Workflow schemes migration during workflow-permission-freeze; SOC 2 CC8.1 change management and CC6.6 access-control evidence remains satisfied. | EVT-J180-TASKS-032 plus row-count and hash proof. |
| AC-J180-033 | notes proves Permission schemes migration during confluence-space-load; GDPR Articles 30 and 32 for employee and customer data in work items and pages remains satisfied. | EVT-J180-NOTES-033 plus row-count and hash proof. |
| AC-J180-034 | docs proves Confluence space migration during sprint-parallel-run; SEC Rule 17a-4(f) WORM retention for regulated engineering communications where enabled remains satisfied. | EVT-J180-DOCS-034 plus row-count and hash proof. |
| AC-J180-035 | drive proves Project board migration during workspace-cutover; ISO/IEC 27001 Annex A 5.37 documented operating procedures and A 8.15 logging remains satisfied. | EVT-J180-DRIVE-035 plus row-count and hash proof. |
| AC-J180-036 | identity proves Issue types migration during jira-rest-export; SOX Section 404 software change-control evidence for release approvals remains satisfied. | EVT-J180-IDENTITY-036 plus row-count and hash proof. |

## Bespoke data packet and named failure modes

- Workspace scope: 142 Jira projects, 318 issue types across project schemes, 77 workflow schemes, 52 permission schemes, 41 Confluence spaces, and 1.8 TB attachments.
- Nora's materiality line: any release-blocking issue, restricted design page, or permission escalation blocks workspace cutover.
- Named failure mode ATL-FM-01: a Bug issue type maps to Task in a project-specific issue-type scheme.
- Named failure mode ATL-FM-02: workflow scheme loses the Blocked -> In Progress transition validator.
- Named failure mode ATL-FM-03: permission scheme grants Browse Projects to a contractor group after migration.
- Named failure mode ATL-FM-04: Confluence space restriction on robotics-safety pages is flattened into a public note.
- VP Engineering question: "Can we run Monday sprint planning and open the motor-safety spec without Atlassian?"
- Go branch: one sprint parallel-run preserves sprint board, workflow transitions, permission schemes, and Confluence page restrictions.
- No-go branch: Jira remains writable for release-blocking projects while Confluence spaces move read-only into notes.

- Operator dialogue: Nora says safety-severity disappearing means the workspace is not live.
- Concrete data value: sprint NAV-2026-22 has 41 issues and 128 story points in both systems.
- Evidence owner: tasks owns NAV-1187 issue-type proof; notes owns SAFE page restriction proof.
- Rollback owner: engineering operations can keep Jira writable for NAV and SAFE projects only.
- Business clock: sprint planning starts Monday 09:00 PST.

## Deliberately out of scope

- Rewriting j01-j175 user journeys.
- Inventing a new µservice suite or hiding ownership behind a bundle.
- Taking production credentials from the incumbent system.
- Treating vendor export success as business cutover success without parallel-run deltas.
