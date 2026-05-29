---
id: ADR-SDK-0007
title: "Developer portal ships as a Backstage extension"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
related_oyatie_adrs:
  - ADR-0131
  - ADR-0173
  - ADR-0213
  - ADR-0243
  - ADR-0244
  - ADR-0258
  - ADR-0263
decision_owner: axis-ecosystem + council-architecture
---

# ADR-SDK-0007: Developer portal ships as a Backstage extension

## Context

- The named pressure is `developer-experience-without-a-second-portal-platform`.
- Developer-sdk needs a portal for API keys, SDK downloads, docs, sandbox tenants, webhook events, payout status, tax forms, KYC, marketplace submissions, and support cases.
- Prior incident class `standalone-dev-portal-sprawl` created a new web app for each developer surface.
- Prior incident class `catalog-drift-from-docs` let developer docs and service catalog entries diverge.
- Prior incident class `portal-auth-bypass` duplicated authentication and entitlement logic outside the standard platform shell.
- ADR-0213 defines Ecosystem-as-a-Service as a platform surface, not a marketing site.
- ADR-0173 allows adoption of open-source substrate when the platform retains ownership.
- ADR-0243 requires all portal actions to be Cedar-gated.
- ADR-0244 requires portal resources to be tenant and developer scoped.
- ADR-0258 requires developer APIs and SDK docs to expose version pinning.
- ADR-0263 requires portal actions to emit observability data.
- Backstage is the established open-source developer portal substrate.
- Backstage already provides software catalog, TechDocs, scaffolder workflows, plugin architecture, and RBAC extension points.
- Oyatie already needs service catalog and internal developer workflows; a separate developer portal would duplicate shell, plugin, catalog, and docs.
- External developers still need a branded public surface, but branding can be a Backstage app shell and plugin composition.
- A standalone Next.js or Astro portal would be easier for marketing pages but weaker for live developer operations.
- The portal must show live sandbox quota and status from tenancy.
- The portal must show SDK determinism proofs and signing keys.
- The portal must show KYC and payout status without exposing raw sensitive data.
- The portal must support tenant-admin and developer-account views.
- The portal must keep docs and API catalog under one navigation model.
- The portal must not become a hidden second policy engine.

## Decision

- We choose `Backstage 1.27.x LTS-compatible extension` as the developer portal substrate.
- The named pattern is `developer portal as plugin over service catalog`, following Spotify Backstage and CNCF internal developer platform practice.
- Developer-sdk ships a Backstage plugin package named `@oyatie/plugin-developer-sdk`.
- The plugin mounts in the canonical Backstage app at `/developers`.
- The public route is `https://developers.oyatie.com` through the same Backstage app shell.
- The plugin does not own authentication.
- The plugin does not own authorization.
- The plugin calls developer-sdk APIs through typed clients.
- Backstage identity maps to `DeveloperSdk::DeveloperAccount` principals.
- Tenant-admin identity maps to `Tenant::Admin` principals through the tenancy service.
- Every mutation button maps to a Cedar-gated backend action.
- The plugin uses TechDocs for API guides.
- The plugin uses the software catalog for SDK packages, APIs, and sample apps.
- The plugin uses scaffolder templates for sandbox app creation.
- The plugin uses the developer-sdk SDK to call backend APIs.
- The plugin shows SDK release manifests and determinism proofs.
- The plugin shows signing public keys and key rotation status.
- The plugin shows sandbox tenant status and cleanup timeline.
- The plugin shows KYC status without raw evidence.
- The plugin shows payout status and tax form versions.
- The plugin shows webhook delivery attempts and replay controls.
- The plugin shows marketplace submission review status.
- The plugin uses ADR-0258 API version pinning in every generated API client.
- The plugin surfaces pack simulation state for sandboxes.
- The plugin supports WCAG 2.2 AA.
- The plugin emits audit event `DeveloperPortalActionInvoked` for state-changing actions.
- The plugin p95 page data load target is 500 ms for dashboard summary.
- The plugin p95 mutation acknowledgement target is 1 second excluding downstream long-running jobs.
- The plugin bundle size budget is 250 KiB gzip for the developer-sdk plugin chunk.
- The plugin must work in self-hosted deployments where the Backstage app is tenant-owned.

## Alternatives Considered

### Standalone Next.js developer portal

- Pro: strong public web UX.
- Pro: easy brand control.
- Pro: common frontend hiring pool.
- Con: duplicates Backstage catalog and TechDocs.
- Con: risks second auth and policy surface.
- Con: scaffolder workflows need reimplementation.
- Con: internal and external developer docs drift.
- Tradeoff: marketing flexibility but platform sprawl.
- Rejected as canonical.

### Static documentation site only

- Pro: simple and fast.
- Pro: low runtime risk.
- Pro: good for API guides.
- Con: cannot manage sandboxes, webhooks, KYC, payouts, or tax forms.
- Con: no live service catalog.
- Con: no state-changing workflows.
- Tradeoff: docs clarity but insufficient product surface.
- Rejected.

### Fully custom Rust web app

- Pro: stack ownership and strong type safety.
- Pro: fewer Node dependencies.
- Pro: direct integration with backend crates.
- Con: would recreate portal primitives already solved by Backstage.
- Con: weaker plugin ecosystem.
- Con: slower to ship TechDocs, catalog, and scaffolder parity.
- Tradeoff: ownership but too much reinvention.
- Rejected for portal shell; Rust remains backend implementation language.

### Backstage fork

- Pro: maximum control over shell behavior.
- Pro: can remove unneeded plugins.
- Pro: branding can be deeply customized.
- Con: high maintenance cost.
- Con: upgrade path becomes harder.
- Con: security patches lag.
- Tradeoff: control but unnecessary fork debt.
- Rejected; extension/plugin approach preserves upgrade path.

## Consequences

- Positive: developer docs, API catalog, SDK packages, and scaffolder workflows share one portal substrate.
- Positive: portal authorization stays behind platform Cedar gates.
- Positive: internal and external developer workflows converge instead of diverging.
- Positive: Backstage plugin model keeps the developer-sdk surface modular.
- Positive: self-hosted deployments can install the same plugin into their Backstage app.
- Negative: Backstage brings Node/React dependency surface.
- Negative: public marketing flexibility is lower than a bespoke site.
- Negative: Backstage upgrade cadence becomes part of developer-sdk maintenance.
- Negative: plugin developers must understand Backstage APIs.
- Neutral: marketing pages may still exist outside this plugin, but operational developer workflows live here.
- Neutral: Backstage is substrate, not the source of truth; developer-sdk APIs remain authoritative.
- Follow-up work: implement `SDK-IP-012-backstage-plugin-shell`.
- Follow-up work: add plugin API client generated by ADR-SDK-0002.
- Follow-up work: add portal action audit event mapping.
- Follow-up work: add self-hosted Backstage installation guide.

## Implementation Notes

- Package name is `@oyatie/plugin-developer-sdk`.
- Backstage compatibility target is `1.27.x`.
- Plugin route ref is `developerSdkRootRouteRef`.
- Main route path is `/developers`.
- Dashboard component is `DeveloperSdkDashboardPage`.
- Sandbox component is `DeveloperSandboxesPage`.
- SDK releases component is `SdkReleasesPage`.
- Webhooks component is `WebhookDeliveriesPage`.
- KYC component is `DeveloperKycStatusPage`.
- Payout component is `DeveloperPayoutsPage`.
- Tax forms component is `DeveloperTaxFormsPage`.
- Marketplace component is `MarketplaceSubmissionsPage`.
- API client package is `@oyatie/developer-sdk-client`.
- API client pins request API version per ADR-0258.
- Data shape `DeveloperPortalSessionV1` binds Backstage identity to developer account.
- Field `backstage_identity_ref` stores Backstage entity ref.
- Field `developer_account_id` stores developer account id.
- Field `tenant_id` stores active tenant scope when applicable.
- Field `roles` stores resolved portal roles.
- Field `api_version` stores request version pin.
- Backend proxy endpoint `GET /v1/developer/portal/summary` returns dashboard summary.
- Backend proxy endpoint `GET /v1/developer/portal/catalog` returns SDK/API catalog projection.
- State-changing endpoints stay in their own backend domains; portal summary endpoint is read-only.
- Cedar principal is resolved from Backstage identity and developer account.
- Cedar action `developer-sdk.portal.view` applies to portal summary.
- Cedar action `developer-sdk.sandbox.create` applies to sandbox creation buttons.
- Cedar action `developer-sdk.webhook.replay` applies to webhook replay buttons.
- Cedar action `developer-sdk.payout.read` applies to payout views.
- Cedar action `developer-sdk.tax_form.read` applies to tax form views.
- Cedar action `developer-sdk.marketplace.submit` applies to marketplace submission.
- Cedar context field `backstage_entity_ref` is included.
- Cedar context field `developer_account_id` is included.
- Cedar context field `tenant_id` is included when tenant-scoped.
- Cedar context field `api_version` is included for every backend call.
- Example permit: principal `DeveloperSdk::DeveloperAccount::"dev_01HY"`, action `developer-sdk.portal.view`, resource `DeveloperSdk::Portal::"developers"`, context `{tenant_id:"tn_01HY", api_version:"2026-05-18"}`.
- Example forbid: same principal, action `developer-sdk.payout.read`, context `{developer_account_matches:false}`.
- TechDocs source path is `microservices/developer-sdk/tutorials/`.
- Catalog entity kind for SDK package is `Component`.
- Catalog entity kind for API is `API`.
- Scaffolder template `developer-sdk-sandbox-app-template` creates sample app wired to sandbox tenant.
- Plugin feature flag `developerSdk.payouts.enabled` is controlled by Cedar-backed backend config, not frontend-only config.
- Plugin emits frontend metric `oya_developer_sdk_portal_page_load_ms`.
- Backend emits metric `oya_developer_sdk_portal_api_latency_ms`.
- Backend emits audit event `DeveloperPortalActionInvoked` for state-changing actions.
- OpenTelemetry span `developer_sdk.portal.request` wraps backend proxy calls.
- Dashboard `developer-sdk-portal-ops.json` shows page load, API latency, Cedar denials, and route errors.
- SLO `developer-sdk-portal-dashboard-load.openslo.yaml` sets p95 <= 500 ms for summary data.
- SLO `developer-sdk-portal-mutation-ack.openslo.yaml` sets p95 <= 1 second for accepted mutation requests.
- Failure mode `backstage_identity_unmapped` returns 403 and prompts account linking.
- Failure mode `api_version_unsupported` returns 400 with supported versions.
- Failure mode `cedar_denied` returns 403 and emits denial event.
- Failure mode `downstream_unavailable` degrades the affected card only, not the entire dashboard.
- Failure mode `plugin_bundle_budget_exceeded` blocks CI.
- The plugin must not store KYC raw evidence in browser state.
- The plugin must not log payout bank details.
- The plugin must redact tax identifiers except last four characters.

## Verification

- Test `portal_maps_backstage_identity_to_developer_account` verifies identity binding.
- Test `portal_summary_is_read_only` verifies summary endpoint cannot mutate state.
- Test `portal_sandbox_create_uses_cedar` verifies mutation maps to Cedar action.
- Test `portal_webhook_replay_uses_cedar` verifies replay maps to Cedar action.
- Test `portal_payout_read_scope` verifies developers cannot read others' payouts.
- Test `portal_tax_form_redacts_identifiers` verifies last-four display.
- Test `portal_kyc_no_raw_evidence_in_browser_state` verifies sensitive state is absent.
- Test `portal_api_version_pin_sent` verifies ADR-0258 header on calls.
- Test `portal_degraded_card_when_downstream_unavailable` verifies partial failure behavior.
- Test `portal_a11y_wcag_aa` verifies accessibility scan.
- Metric `oya_developer_sdk_portal_page_load_ms` must meet p95 <= 500 ms for dashboard data.
- Metric `oya_developer_sdk_portal_api_latency_ms` must meet p95 <= 300 ms for summary backend.
- Metric `oya_developer_sdk_portal_cedar_denied_total` must be visible by route and action.
- Dashboard `developer-sdk-portal-ops.json` must include route error and downstream degradation panels.
- Dashboard `developer-experience-funnel.json` must link portal onboarding to sandbox, KYC, and first SDK download.
- CI check `developer-sdk-backstage-plugin-build` builds the plugin.
- CI check `developer-sdk-backstage-plugin-bundle-budget` enforces 250 KiB gzip budget.
- CI check `developer-sdk-portal-a11y` runs automated accessibility tests.
- CI check `developer-sdk-portal-api-version-pin` verifies generated clients send the version pin.
- CI check `developer-sdk-portal-no-sensitive-logs` scans browser and backend logs.
- CI check `oya-governance-cedar-action-coverage --microservice developer-sdk` validates portal actions.
- CI check `oya-governance-observability-emission --microservice developer-sdk` validates ADR-0263 telemetry.
- Browser smoke test opens `/developers`, `/developers/sandboxes`, `/developers/sdk-releases`, `/developers/payouts`, and `/developers/tax-forms`.
- Security test attempts route access with unmapped Backstage identity and expects 403.
- Self-hosted install test loads plugin into a clean Backstage app.
- Audit query verifies every state-changing portal action has `DeveloperPortalActionInvoked`.

## References

- ADR-0131: Per-microservice flat layout.
- ADR-0173: Vendor lock-in avoidance and stack ownership.
- ADR-0213: Ecosystem-as-a-Service architecture.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0258: API versioning model.
- ADR-0263: Observability emission contract.
- Backstage 1.27 documentation.
- Backstage plugin development documentation.
- Backstage TechDocs documentation.
- Backstage software catalog documentation.
- Backstage Scaffolder documentation.
- CNCF internal developer platform guidance.
- Spotify Backstage origin documentation.
- WCAG 2.2 AA.
- OpenTelemetry web instrumentation documentation.
- OWASP ASVS 4.0 session and access-control guidance.
- NIST SP 800-63B digital identity guidance.
