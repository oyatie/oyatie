---
doc_class: User-Journey-UX-Flow
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
---

# j180-migration-from-atlassian-jira-confluence-to-oyatie-workspace UX flow - migration tool screens, dashboards, rollback

## UX principle

The migration UI is an operator tool for executives and migration owners. It translates vendor object state into business cutover state, while keeping exact source object names one click away for audit and engineering review.

## Navigation model

- Left rail: Overview, Extracts, Field Mapping, Parallel Run, Exceptions, Rollback, Evidence, Go/No-Go.
- Top bar: tenant atlasbridge-robotics, source atlassian-cloud-site-atlasbridge, target Oyatie workspace, current phase, and rollback readiness.
- Dashboard primary metric: records clean, records in exception, material deltas, rollback ceiling, and next gate owner.

## Screen inventory

### Screen 001 - Migration Overview - Issue types

- Primary view: Migration Overview shows Issue types during jira-rest-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Scrum Board -> Oyatie Sprint Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workspace publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected epic records.

### Screen 002 - Source Extract Monitor - Workflow schemes

- Primary view: Source Extract Monitor shows Workflow schemes during workflow-permission-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Issue View -> Oyatie Work Item Drawer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tasks publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected story records.

### Screen 003 - Vendor Object Inspector - Permission schemes

- Primary view: Vendor Object Inspector shows Permission schemes during confluence-space-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Workflow Scheme Editor -> Oyatie Delivery Workflow Editor; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: notes publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected bug records.

### Screen 004 - Field Mapping Workbench - Confluence space

- Primary view: Field Mapping Workbench shows Confluence space during sprint-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Permission Scheme -> Oyatie Project Access Matrix; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: docs publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected task records.

### Screen 005 - Projection Load Console - notes µservice mapping

- Primary view: Projection Load Console shows notes µservice mapping during workspace-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Space -> Oyatie notes µservice space; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: drive publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected sprint records.

### Screen 006 - Parallel-Run Delta Dashboard - Jira sprint

- Primary view: Parallel-Run Delta Dashboard shows Jira sprint during jira-rest-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Page Tree -> Oyatie Docs/Notes Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected space records.

### Screen 007 - Exception Queue - page tree

- Primary view: Exception Queue shows page tree during workflow-permission-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Scrum Board -> Oyatie Sprint Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected page records.

### Screen 008 - Rollback Rehearsal - project board

- Primary view: Rollback Rehearsal shows project board during confluence-space-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Issue View -> Oyatie Work Item Drawer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected attachment records.

### Screen 009 - Executive Go/No-Go Card - Issue types

- Primary view: Executive Go/No-Go Card shows Issue types during sprint-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Workflow Scheme Editor -> Oyatie Delivery Workflow Editor; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected epic records.

### Screen 010 - Evidence Vault - Workflow schemes

- Primary view: Evidence Vault shows Workflow schemes during workspace-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Permission Scheme -> Oyatie Project Access Matrix; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: observability publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected story records.

### Screen 011 - Migration Overview - Permission schemes

- Primary view: Migration Overview shows Permission schemes during jira-rest-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Space -> Oyatie notes µservice space; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: search publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected bug records.

### Screen 012 - Source Extract Monitor - Confluence space

- Primary view: Source Extract Monitor shows Confluence space during workflow-permission-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Page Tree -> Oyatie Docs/Notes Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: messenger publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected task records.

### Screen 013 - Vendor Object Inspector - notes µservice mapping

- Primary view: Vendor Object Inspector shows notes µservice mapping during confluence-space-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Scrum Board -> Oyatie Sprint Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: connect publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected sprint records.

### Screen 014 - Field Mapping Workbench - Jira sprint

- Primary view: Field Mapping Workbench shows Jira sprint during sprint-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Issue View -> Oyatie Work Item Drawer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compliance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected space records.

### Screen 015 - Projection Load Console - page tree

- Primary view: Projection Load Console shows page tree during workspace-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Workflow Scheme Editor -> Oyatie Delivery Workflow Editor; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: ops-dashboard-control-center publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected page records.

### Screen 016 - Parallel-Run Delta Dashboard - project board

- Primary view: Parallel-Run Delta Dashboard shows project board during jira-rest-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Permission Scheme -> Oyatie Project Access Matrix; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workspace publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected attachment records.

### Screen 017 - Exception Queue - Issue types

- Primary view: Exception Queue shows Issue types during workflow-permission-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Space -> Oyatie notes µservice space; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tasks publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected epic records.

### Screen 018 - Rollback Rehearsal - Workflow schemes

- Primary view: Rollback Rehearsal shows Workflow schemes during confluence-space-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Page Tree -> Oyatie Docs/Notes Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: notes publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected story records.

### Screen 019 - Executive Go/No-Go Card - Permission schemes

- Primary view: Executive Go/No-Go Card shows Permission schemes during sprint-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Scrum Board -> Oyatie Sprint Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: docs publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected bug records.

### Screen 020 - Evidence Vault - Confluence space

- Primary view: Evidence Vault shows Confluence space during workspace-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Issue View -> Oyatie Work Item Drawer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: drive publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected task records.

### Screen 021 - Migration Overview - notes µservice mapping

- Primary view: Migration Overview shows notes µservice mapping during jira-rest-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Workflow Scheme Editor -> Oyatie Delivery Workflow Editor; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected sprint records.

### Screen 022 - Source Extract Monitor - Jira sprint

- Primary view: Source Extract Monitor shows Jira sprint during workflow-permission-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Permission Scheme -> Oyatie Project Access Matrix; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected space records.

### Screen 023 - Vendor Object Inspector - page tree

- Primary view: Vendor Object Inspector shows page tree during confluence-space-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Space -> Oyatie notes µservice space; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected page records.

### Screen 024 - Field Mapping Workbench - project board

- Primary view: Field Mapping Workbench shows project board during sprint-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Page Tree -> Oyatie Docs/Notes Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected attachment records.

### Screen 025 - Projection Load Console - Issue types

- Primary view: Projection Load Console shows Issue types during workspace-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Scrum Board -> Oyatie Sprint Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: observability publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected epic records.

### Screen 026 - Parallel-Run Delta Dashboard - Workflow schemes

- Primary view: Parallel-Run Delta Dashboard shows Workflow schemes during jira-rest-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Issue View -> Oyatie Work Item Drawer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: search publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected story records.

### Screen 027 - Exception Queue - Permission schemes

- Primary view: Exception Queue shows Permission schemes during workflow-permission-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Workflow Scheme Editor -> Oyatie Delivery Workflow Editor; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: messenger publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected bug records.

### Screen 028 - Rollback Rehearsal - Confluence space

- Primary view: Rollback Rehearsal shows Confluence space during confluence-space-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Permission Scheme -> Oyatie Project Access Matrix; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: connect publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected task records.

### Screen 029 - Executive Go/No-Go Card - notes µservice mapping

- Primary view: Executive Go/No-Go Card shows notes µservice mapping during sprint-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Space -> Oyatie notes µservice space; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compliance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected sprint records.

### Screen 030 - Evidence Vault - Jira sprint

- Primary view: Evidence Vault shows Jira sprint during workspace-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Page Tree -> Oyatie Docs/Notes Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: ops-dashboard-control-center publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected space records.

### Screen 031 - Migration Overview - page tree

- Primary view: Migration Overview shows page tree during jira-rest-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Scrum Board -> Oyatie Sprint Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workspace publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected page records.

### Screen 032 - Source Extract Monitor - project board

- Primary view: Source Extract Monitor shows project board during workflow-permission-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Issue View -> Oyatie Work Item Drawer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tasks publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected attachment records.

### Screen 033 - Vendor Object Inspector - Issue types

- Primary view: Vendor Object Inspector shows Issue types during confluence-space-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Workflow Scheme Editor -> Oyatie Delivery Workflow Editor; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: notes publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected epic records.

### Screen 034 - Field Mapping Workbench - Workflow schemes

- Primary view: Field Mapping Workbench shows Workflow schemes during sprint-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Permission Scheme -> Oyatie Project Access Matrix; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: docs publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected story records.

### Screen 035 - Projection Load Console - Permission schemes

- Primary view: Projection Load Console shows Permission schemes during workspace-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Space -> Oyatie notes µservice space; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: drive publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected bug records.

### Screen 036 - Parallel-Run Delta Dashboard - Confluence space

- Primary view: Parallel-Run Delta Dashboard shows Confluence space during jira-rest-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Page Tree -> Oyatie Docs/Notes Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected task records.

### Screen 037 - Exception Queue - notes µservice mapping

- Primary view: Exception Queue shows notes µservice mapping during workflow-permission-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Scrum Board -> Oyatie Sprint Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected sprint records.

### Screen 038 - Rollback Rehearsal - Jira sprint

- Primary view: Rollback Rehearsal shows Jira sprint during confluence-space-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Issue View -> Oyatie Work Item Drawer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected space records.

### Screen 039 - Executive Go/No-Go Card - page tree

- Primary view: Executive Go/No-Go Card shows page tree during sprint-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Workflow Scheme Editor -> Oyatie Delivery Workflow Editor; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected page records.

### Screen 040 - Evidence Vault - project board

- Primary view: Evidence Vault shows project board during workspace-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Jira Permission Scheme -> Oyatie Project Access Matrix; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: observability publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected attachment records.

### Screen 041 - Migration Overview - Issue types

- Primary view: Migration Overview shows Issue types during jira-rest-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Space -> Oyatie notes µservice space; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: search publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected epic records.

### Screen 042 - Source Extract Monitor - Workflow schemes

- Primary view: Source Extract Monitor shows Workflow schemes during workflow-permission-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Nora Stein can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Confluence Page Tree -> Oyatie Docs/Notes Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: messenger publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected story records.
