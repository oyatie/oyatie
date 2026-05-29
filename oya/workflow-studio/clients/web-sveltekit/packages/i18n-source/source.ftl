# Workflow Studio — Fluent i18n source (per ADR-0206).
# Source locale: en-US.
# Per-locale overlays land under ./<locale>.ftl.
# Translation memory tool (Pontoon / Localazy / Crowdin) consumes this file.

## Brand + chrome

brand-workflow-studio = Workflow Studio
nav-canvas = Canvas
nav-runs = Runs
nav-library = Library
nav-settings = Settings

## Canvas

workflow-studio-canvas-title = Workflow Studio canvas
workflow-studio-canvas-empty-state = Drop a node from the library to begin.
workflow-studio-canvas-node-count =
    { $count ->
        [0] No nodes yet
        [one] 1 node
       *[other] { $count } nodes
    }
workflow-studio-canvas-edge-count =
    { $count ->
        [0] No edges
        [one] 1 edge
       *[other] { $count } edges
    }
workflow-studio-canvas-zoom-percent = { $zoom }%
workflow-studio-canvas-aria-label = Workflow editor canvas. Use arrow keys to navigate nodes; press Space to grab; press arrow keys to move; press Space to drop; press Escape to cancel.

## Collab

collab-status-syncing = Syncing…
collab-status-synced = Up to date
collab-status-offline = Offline — your changes will sync when you reconnect.
collab-participant-count =
    { $count ->
        [0] No collaborators
        [one] 1 collaborator
       *[other] { $count } collaborators
    }
collab-grabbed-node = Grabbed { $node-label }
collab-moved-node = Moved { $node-label } to position { $x }, { $y }
collab-dropped-node = Dropped { $node-label }
collab-cancelled-grab = Cancelled grab

## Run

run-status-pending = Pending
run-status-running = Running
run-status-succeeded = Succeeded
run-status-failed = Failed
run-status-cancelled = Cancelled
run-step-duration-seconds = { $seconds }s
run-replay-button = Replay
run-cancel-button = Cancel run

## Library

library-search-placeholder = Search nodes…
library-empty = No nodes match your search.
library-tag-trigger = Trigger
library-tag-transform = Transform
library-tag-sink = Sink
library-tag-conditional = Conditional
library-tag-iterator = Iterator

## A11y

a11y-skip-to-content = Skip to main content
a11y-focus-trap-warning = Press Escape to exit this dialog.
a11y-error-prefix = Error:
a11y-success-prefix = Success:

## Errors

error-canvas-load-failed = We couldn't load the canvas. { $details }
error-collab-disconnected = Lost connection to collaboration server.
error-run-failed = Run failed at step { $step-name }: { $message }
error-permission-denied-canvas = You don't have permission to edit this workflow.
error-permission-denied-run = You don't have permission to start a run.

## DSAR (per ADR-0209)

dsar-request-export = Export my data
dsar-request-delete = Delete my data
dsar-request-rectify = Correct my data
dsar-eta-days = Estimated completion: { $days } days
dsar-statutory-sla = GDPR statutory SLA: 30 days
dsar-target-sla = Internal target: 5 days

## Compliance audit-portal

auditor-portal-title = Compliance auditor portal
auditor-framework-soc2 = SOC 2 Type II
auditor-framework-gdpr = GDPR
auditor-framework-hipaa = HIPAA
auditor-framework-pci = PCI-DSS
auditor-artifact-count = { $count } artifacts
auditor-seal-verify = Verify audit-chain seal
