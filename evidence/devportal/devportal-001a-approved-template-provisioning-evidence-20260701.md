# DEVPORTAL-001A approved-template provisioning evidence

Generated at: 2026-07-01T14:05:50Z
Kanban task: t_e43dadd0
Claim boundary: fixture-backed Developer Portal / Leptos shell slice only. No provider-live provisioning, OpenTofu/Kubernetes apply, Backstage destination commitment, generated JSON hand edits, operator CLI surface, billing runtime, Cedar runtime, audit persistence, or production API readiness is claimed.

## Browser/user-story transcript

Local browser surface: `http://127.0.0.1:3123/#developer-portal-provisioning` served from `oya-application-shell-frontend`.

Observed story:
1. Application developer lands on `Approved service template provisioning` under the Service catalog / Developer Portal fixture surface.
2. The view shows the approved template `Rust API + Leptos Shell Service` with template id `svc-rust-axum-leptos`, version `1.0.0-fixture`, owner `platform engineer`, and catalog entity `ServiceCatalogEntity · orders-api`.
3. Required resources are visible: service, database, topic, bucket, secret reference, SLO, runbook, deploy pipeline, and preview env.
4. Parameter inputs are visible and labelled: `service_slug=orders-api`, `owning_team=axis-developer-experience`, and `cell=cell-us-east-2`.
5. The user can see `Quota and cost preview` and `Submit provisioning request` actions. The copy states that the UI stages no service admission, deploy, billing, IAM, database, or cloud mutation.
6. Durable operation/evidence result is visible: request `prq-devportal-001a`, idempotency key `idem-prq-devportal-001a`, operation `op-devportal-001a`, state `accepted_fixture`, preview environment `prv-orders-api-001a · preview-ready-fixture`, and receipt `REC-DEVPORTAL-001A`.
7. Admission sequence is visible from identity/tenant/project binding through audit/event emission.
8. Generated artifacts are visible: OpenSLO, runbook live-doc entry, progressive-delivery policy, preview environment lifecycle, and backing resource descriptor.
9. Policy-denial/audit fixture is visible: `deny · policy denied fixture: unapproved template version or quota over budget`, events `developer_portal.provisioning.requested`, `developer_portal.provisioning.policy_denied`, and `developer_portal.generated_artifacts.registered` with receipts.
10. Role coverage is visible for Platform engineer, Security reviewer, and Tenant admin.
11. Resource facet evidence is visible for service/database/topic/bucket across lifecycle, identity, policy, quota, billing, audit, observability, rollback, and reconciliation; `facet_contract_fail_closed=true`.

## Accessibility check

Added and verified explicit static landmarks for the fixture paths:
- `aria-labelledby="developer-portal-provisioning-title"`
- `aria-label="Template parameters"`
- `aria-label="Generated artifacts"`
- `aria-label="Policy denial and audit evidence"`
- `aria-label="Developer Portal role coverage"`
- `aria-label="Developer Portal resource facet evidence"`

Browser DOM spot-check found no icon-only buttons and no unlabeled inputs in the Developer Portal section. The current source contains all expected labels; the focused regression test `app::tests::developer_portal_story_has_accessible_landmarks_for_fixture_paths` covers them.

## API/contract fixture coverage

The `TenantRenderEnvelope.developer_portal` fixture models the required entities:
- `ServiceCatalogEntity`
- `ApprovedTemplate`
- `TemplateVersion`
- `TemplateParameter`
- `ProvisioningRequest`
- `ProvisioningOperation`
- `GeneratedArtifact`
- `PreviewEnvironment`
- `ResourceFacetEvidence`

The fixture records the required admission sequence:
identity/tenant/project binding -> template allowlist -> parameter validation -> Cedar/policy decision -> quota reservation and cost preview -> approval if required -> idempotent operation ledger mutation -> reconciler-owned actuation -> generated artifact registration -> audit/event emission.

## Verification commands and results

RED check:
- `cargo test -p oya-application-shell-frontend developer_portal_story_has_accessible_landmarks_for_fixture_paths --lib --features ssr --no-default-features`
- Result before implementation: failed with `missing accessibility marker aria-label="Template parameters"`.

Focused/full crate checks after implementation:
- `cargo test -p oya-application-shell-frontend developer_portal_story_has_accessible_landmarks_for_fixture_paths --lib --features ssr --no-default-features`
- Result: 1 passed, 0 failed.
- `cargo fmt -p oya-application-shell-frontend && cargo test -p oya-application-shell-frontend --lib --features ssr --no-default-features`
- Result: 41 passed, 0 failed.

Browser transcript and DOM checks:
- `#developer-portal-provisioning` found in the local browser.
- Visible buttons: `Quota and cost preview`, `Submit provisioning request`.
- Visible labelled inputs: `Search ORN`, `Template parameter service_slug`, `Template parameter owning_team`, `Template parameter cell`.
- Visible policy denial, audit events, role coverage, generated artifact list, and fail-closed resource facet table.

No-new-operator-CLI check:
- `git diff -- oya/application/crates/oya-application-shell-frontend/src/app.rs` contains DEVPORTAL fixture markers and contains no `operator CLI`, `dev-cli`, or `oya-dev-cli` strings.
- Scope of intended changes is the existing Leptos shell/frontend fixture surface and evidence files, not the operator CLI.

## D19/D20 closeout notes

Rollback/no-deploy rationale: local fixture-only Leptos shell changes; revert the changed shell/frontend files and this evidence artifact if the slice needs rollback. No deployment or production provider actuation occurred.

Observability: audit/evidence fixture rows are visible in the UI and in the static fixture; no live telemetry pipeline is claimed.

Release-governance impact: no release automation or release publication; this creates a fixture-backed first slice requiring independent review before merge readiness.

Observation harvest: dependent production/API/runtime graduation remains with API-001/SHELL-005/OPSDOC-001/NETWORK-001/FINOPS-001/RELEASE-001 owners; this card does not ratify API-001 subjects or Proposed ADR authority beyond fixture-backed UX/API evidence.
