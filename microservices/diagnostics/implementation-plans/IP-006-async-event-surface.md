# IP-006: Async Event Surface

Status: Reconciled
Date: 2026-05-21

## Goal

Publish lab/pathology events and explicit cross-service correlation requests.

## Events

- `diagnostics.lab-order.accepted`
- `diagnostics.lab-result.released`
- `diagnostics.lab-result.corrected`
- `diagnostics.pathology-case.signed-out`
- `diagnostics.critical-result.notified`
- `diagnostics.critical-result.acknowledged`
- `diagnostics.reflex-test.triggered`
- `diagnostics.quality-control.failed`
- `diagnostics.lab-result.image-correlation-requested`

## Acceptance

- AsyncAPI defines diagnostics-owned lab/pathology events only.
- The correlation event requests imaging context but does not define imaging-owned payloads.
