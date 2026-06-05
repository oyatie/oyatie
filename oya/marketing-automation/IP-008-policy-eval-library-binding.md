---
doc_class: ImplementationPlan
ip_id: IP-008-policy-eval-library-binding
microservice: marketing-automation
bounded_contexts: [campaign, journey, segment, suppression, deliverability, attribution]
related_adrs: [ADR-0243, ADR-0244, ADR-0263, ADR-0297, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + axis-policy-engine
tenant_class_aware: true
---

# IP-008: Policy Evaluation Library Binding

## A. Problem

The usecase layer has a `PolicyPort`, and the service owns Cedar files under both `policy/` and `policies/`, but the stamped IP never identified which Cedar actions protect marketing operations. Marketing sends need fail-closed policy before segment materialization, journey launch, suppression checks, deliverability inspection, attribution export, and marketplace audience licensing. Otherwise HubSpot/Marketo/Mailchimp parity is shallow: the service would have APIs without enforceable tenant, consent, and pack boundaries.

## B. Approach

Bind `PolicyPort::authorize()` to caller-side Cedar evaluation using the existing Marketing Automation entity vocabulary: `CampaignJourney`, `MarketingSegment`, suppression ledger entries, deliverability report, attribution export, and marketplace audience-license resource. The policy library loads `policy/campaign-journey-authorization.cedar`, `policy/abuse-defence.cedar`, `policy/emergency-services-bypass.cedar`, and the local policies under `policies/` for consent, suppression, attribution egress, and deliverability inspection.

## C. Deliverables

| Artifact | Change |
|---|---|
| `src/usecase/mod.rs` | Implement `PolicyPort` adapter call sites with explicit action names per command variant. |
| `policy/campaign-journey-authorization.cedar` | Keep same-tenant, non-anonymous, caller-side-library-first permit as the baseline journey gate. |
| `policies/local-consent-gated-automation.cedar` | Extend context tests to cover `tenant_class` and paid/demo caps without trusting client-supplied values. |
| `policies/local-attribution-export-egress.cedar` | Gate attribution exports by purpose and data class. |
| `tests/integration.rs` | Replace ignored Cedar cross-tenant test with a policy fixture that denies mismatched tenant journey launch. |

## D. Implementation

1. Define the action map: `LaunchJourney -> campaign.journey.use`, `SyncSegment -> campaign.segment.sync`, `EnforceSuppression -> campaign.suppression.enforce`, `RollupAttribution -> campaign.attribution.rollup`, `ExportConsent -> campaign.consent.export`, `LicenseMarketplaceAudience -> campaign.marketplace_audience.license`.
2. Build Cedar context from `UsecaseContext` and `UsecaseActor`; include `tenant_id`, `principal_id`, `audience_type`, `cell_tier`, `purpose`, `data_class`, `tenant_class`, and `audit_event_class`.
3. Add fixtures for same-tenant allow, cross-tenant deny, HIPAA purpose deny, and breakglass allow.
4. Fail closed when policy files are missing or when `context.data_class` is absent.
5. Record the Cedar decision id in `CommandReceipt` or a companion audit payload.
6. Keep policy-engine internals out of this µservice; this IP binds the library, not a new policy engine.

## E. Acceptance

- `cargo test -p oya-marketing-automation-campaign-journey-app cedar_policy_denies_cross_tenant_journey_launch`
- `buck2 build //:quality-lane-registry-authority-check # lane=policy-authorization --microservice marketing-automation`
- `cedar validate` or the repo Cedar validation lane over `policy/*.cedar` and `policies/*.cedar`.
- Manual evidence: every `MarketingAutomationCommand` variant maps to exactly one Cedar action.

## F. Evidence

- Local source: `src/usecase/mod.rs` `PolicyPort`.
- Local policy: `policy/campaign-journey-authorization.cedar`.
- Local policy: `policies/local-consent-gated-automation.cedar`.
- Doctrine: ADR-0244 tenant scoping; ADR-0328 Big-8 P0 elevation.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Replaces commercial permission-set assumptions with tenant, purpose, and data-class policy checks. |
| Adobe Marketo Engage | Smart Campaign execution receives explicit policy gates before activation. |
| Mailchimp | Audience and campaign operations inherit consent and regional-pack enforcement before send/export. |

## H. Local Traceability

- Port source: `PolicyPort::authorize`.
- Context source: `UsecaseContext`.
- Actor source: `UsecaseActor`.
- Cedar file: `policy/campaign-journey-authorization.cedar`.
- Cedar file: `policy/abuse-defence.cedar`.
- Cedar file: `policy/emergency-services-bypass.cedar`.
- Cedar file: `policies/local-consent-gated-automation.cedar`.
- Cedar file: `policies/local-attribution-export-egress.cedar`.
- Action target: `campaign.journey.use`.
- Action target: `campaign.segment.sync`.
- Action target: `campaign.suppression.enforce`.
- Action target: `campaign.attribution.rollup`.
- Action target: `campaign.consent.export`.
- Action target: `campaign.marketplace_audience.license`.
- Deny fixture: cross-tenant journey launch.
- Deny fixture: missing data class.
- Allow fixture: same-tenant policy-library call.
- Failure state: policy file load failure is fail-closed.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-008-policy-eval-library-binding.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-008-policy-eval-library-binding.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
