# IP-002: Cedar Default Deny

Status: Reconciled
Date: 2026-05-21

## Goal

Enforce lab/pathology authorization through Cedar default-deny policies.

## Policies

- `hipaa-deny-default.cedar`
- `ordering-provider-can-view-own.cedar`
- `pathologist-can-sign-out.cedar`
- `medical-lab-technologist-can-result.cedar`

## Acceptance

- Cross-tenant reads are denied.
- Lab-result release requires authorized lab roles.
- Pathology sign-out requires assigned pathologist and verified electronic signature.
- No radiology or imaging actions exist in diagnostics policies.
