---
doc_class: Tutorial
tutorial_id: capability-tier-upgrade-bronze-to-platinum
persona: Evelyn Hart, Customer Success Platform Administrator for Contoso Research
prerequisite_packs:
  - bronze-business-baseline
  - platinum-enterprise-expansion
  - soc2-type-ii-baseline
  - gdpr-baseline
related_oyatie_adrs:
  - ADR-0316
  - ADR-0311
  - ADR-0314
status: draft
date: 2026-05-20
owner: docs/tutorial-library
estimated_completion_time: 95 minutes
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.


## Goal


## Prerequisites

- Named operator account: `evelyn.hart@contoso.example`.
- Named approver account: `marcus.reed@contoso.example`.
- Named finance account: `li.na@contoso.example`.
- Named support witness account: `amira.patel@oyatie.example`.
- Tenant id: `tenant-contoso-research`.
- Current tier grant id: `grant-contoso-bronze-2026`.
- Target tier grant id: `grant-contoso-platinum-2026`.
- Upgrade bundle id: `tier-upgrade-contoso-bronze-platinum-2026-05-20`.
- Subscription pack `bronze-business-baseline` must already be active.
- Subscription pack `platinum-enterprise-expansion` must be available in tenant catalog.
- Subscription pack `soc2-type-ii-baseline` must be active or scheduled for the tenant.
- Subscription pack `gdpr-baseline` must be active because this tenant contains EU research contacts.
- Named microservice subscribed: `tenancy`.
- Named microservice subscribed: `policy-engine`.
- Named microservice subscribed: `capability-registry`.
- Named microservice subscribed: `ontology`.
- Named microservice subscribed: `workflow-engine`.
- Named microservice subscribed: `audit-chain`.
- Named microservice subscribed: `observability`.
- Named microservice subscribed: `finops-portal`.
- Named microservice subscribed: `drive`.
- Named microservice subscribed: `mail`.
- Named microservice subscribed: `messenger`.
- Named microservice subscribed: `intelligence`.
- Named Cedar permit: `capability_tier.grant.read`.
- Named Cedar permit: `capability_tier.grant.create`.
- Named Cedar permit: `capability_tier.grant.activate`.
- Named Cedar permit: `capability_tier.grant.supersede`.
- Named Cedar permit: `capability_tier.projection.preview`.
- Named Cedar permit: `capability_tier.projection.apply`.
- Named Cedar permit: `capability_tier.workflow.load`.
- Named Cedar permit: `capability_tier.ux.render`.
- Named Cedar permit: `capability_tier.compliance.override`.
- Named Cedar permit: `billing.subscription.change`.
- Named Cedar permit: `audit.capability_tier.read`.
- Named Cedar permit: `audit.capability_tier.export`.
- Named Cedar permit: `observability.tenant_health.read`.
- Named Cedar permit: `finops.tier_cost.preview`.
- Browser requirement: Chrome 124 or later with passkey support enabled.
- Test data requirement: no real PHI, payment card data, or export-controlled data in screenshots.
- Recovery authority: Evelyn may pause the upgrade, but only Marcus may approve activation.
- Screenshot naming prefix: `tier-upgrade-contoso-`.
- UI route used in this tutorial: `Tenant Admin -> Capability Tiers -> Upgrade`.
- Command palette namespace used in this tutorial: `tier upgrade`.
- Verification query name: `tutorial.capability_tier_upgrade_status`.
- Expected output label: `contoso_platinum_ready`.

## Step-by-Step

1. Sign in to the Contoso work tenant as Evelyn.

   Open `https://app.oyatie.example/sign-in`.

   Enter `evelyn.hart@contoso.example`.

   Choose `Use passkey`.

   Confirm the browser passkey prompt.

   Select tenant card `Contoso Research`.

   Confirm the tenant badge in the top-left reads `tenant-contoso-research`.

   Do not continue if the badge reads `Personal`.

   Screenshot to capture: `tier-upgrade-contoso-01-tenant-badge.png`.

   The visible page title should be `Contoso Research Home`.

   The left navigation should show `Tenant Admin`, `Workflow Studio`, `Audit`, and `FinOps`.

   If `Tenant Admin` is missing, Evelyn does not have the correct admin group.

   Use `Profile -> Active roles`.

   Confirm role chip `Customer Success Platform Admin`.

   Keep this browser tab open for the rest of the tutorial.

2. Open the capability tier upgrade workspace.

   In the left navigation, select `Tenant Admin`.

   Select `Capability Tiers`.

   The page heading should read `Capability Tiers for Contoso Research`.

   Select the `Current tier` tab.


   Confirm the grant id line reads `grant-contoso-bronze-2026`.

   Confirm the status pill reads `Active`.

   Select `Upgrade tenant`.


   Set `Upgrade bundle id` to `tier-upgrade-contoso-bronze-platinum-2026-05-20`.

   Set `Requested by` to `evelyn.hart@contoso.example`.

   Select `Create upgrade workspace`.

   Screenshot to capture: `tier-upgrade-contoso-02-upgrade-workspace.png`.



   In the upgrade workspace, select `Source tier`.

   Expand `Grant details`.

   Confirm `Current grant id` is `grant-contoso-bronze-2026`.

   Confirm `Grant state` is `active`.

   Confirm `Superseded by` is empty.

   Confirm `Permit bundle` is `bronze-business-baseline-permits`.

   Confirm `UX shell` is `business-basic-shell`.

   Confirm `Workflow template set` is `bronze-business-template-set`.

   Select `Export source snapshot`.


   Open the command palette with `Command-K`.

   Run `tier upgrade compare source target`.

   Confirm the result drawer says `Source snapshot captured`.

   The important evidence value is `bronze_state_before=active`.

   Screenshot to capture: `tier-upgrade-contoso-03-source-grant.png`.


   Select `Target tier`.


   The preview panel should list `enterprise-analytics`, `advanced-workflow-studio`, `intelligence-document-assist`, and `marketplace-settlement-plus`.

   Confirm `New UX shell` is `enterprise-platinum-shell`.

   Confirm `New permit bundle` is `platinum-enterprise-permits`.

   Confirm `New workflow template set` is `platinum-enterprise-template-set`.

   Confirm the panel says `No product fork will be created`.

   The line matters because ADR-0316 requires tiering through grants, overlays, and projections.

   Select `Preview entitlement delta`.

   Filter by `Added capabilities`.

   Confirm `workflow.studio.advanced_branching` is listed.

   Confirm `intelligence.summary.contract_200p` is listed.

   Confirm `finops.platinum_cost_center_report` is listed.

   Screenshot to capture: `tier-upgrade-contoso-04-entitlement-delta.png`.

5. Run the Cedar permit preview.

   Select `Policy`.

   Select `Run Cedar preview`.

   Set `Principal` to `group:contoso-platform-admins`.

   Set `Action set` to `platinum-enterprise-permits`.

   Set `Resource tenant` to `tenant-contoso-research`.

   Select `Evaluate`.

   The expected result is `allow` for `capability_tier.grant.activate`.

   The expected result is `allow` for `capability_tier.workflow.load`.

   The expected result is `allow` for `capability_tier.projection.apply`.

   The expected result is `deny` for any personal tenant resource.

   Expand `Decision log`.


   Confirm the context includes `upgrade_bundle=tier-upgrade-contoso-bronze-platinum-2026-05-20`.


   Screenshot to capture: `tier-upgrade-contoso-05-cedar-preview.png`.

6. Preview the ontology projection changes.

   Select `Ontology`.

   Select `Preview projections`.

   Set `Projection mode` to `Preview only`.

   Set `Object set` to `tenant objects affected by tier`.

   Confirm `Workspace` gains projection view `EnterpriseWorkspaceView`.

   Confirm `Document` gains projection view `DocumentRiskSummaryView`.

   Confirm `DealSet` gains projection view `DealSettlementPlusView`.

   Confirm `WorkflowDefinition` gains projection view `AdvancedWorkflowControlView`.

   Confirm `ConsentRecord` remains filtered by tenant and purpose.

   Confirm `PersonalTenantObject` count remains `0`.

   Select `Run projection diff`.

   The diff summary should show `added_views=4`.

   The diff summary should show `removed_views=0`.

   The diff summary should show `cross_tenant_leaks=0`.

   Screenshot to capture: `tier-upgrade-contoso-06-ontology-preview.png`.

7. Preview the workflow template load.

   Select `Workflow templates`.

   Choose template set `platinum-enterprise-template-set`.

   Confirm template `employee-onboarding-enterprise-v2` appears.

   Confirm template `contract-review-with-intelligence-v1` appears.

   Confirm template `cross-tenant-deal-approval-v1` appears.

   Confirm template `gdpr-dsr-response-managed-v1` appears.

   Select `Load preview`.

   The preview should say `No running workflows will be modified`.

   Confirm `workflow-engine` health is `green`.

   Confirm `workflow-engine` event backlog is `0`.

   Confirm `workflow-engine` can emit audit events.

   Select `Save preview evidence`.


   Screenshot to capture: `tier-upgrade-contoso-07-workflow-templates.png`.

8. Check microservice readiness before finance approval.

   Select `Readiness`.

   The readiness matrix must show rows for `tenancy`, `policy-engine`, `capability-registry`, `ontology`, `workflow-engine`, `audit-chain`, `observability`, `finops-portal`, `drive`, `mail`, `messenger`, and `intelligence`.

   Each row should show `Ready`.

   Expand `intelligence`.

   Confirm capability `contract_200p_summary` is `available`.

   Expand `finops-portal`.

   Confirm meter `platinum_enterprise_active_tenant_month` is `configured`.

   Expand `audit-chain`.

   Confirm seal profile `capability-tier-upgrade-v1` is `available`.

   Select `Run readiness scan`.

   Expected scan result: `12 ready, 0 blocked, 0 degraded`.

   Save the readiness scan to Drive as `microservice-readiness.json`.

   Screenshot to capture: `tier-upgrade-contoso-08-readiness.png`.

9. Generate the cost preview for Li.

   Select `FinOps`.

   Select `Preview cost impact`.

   Set `Billing account` to `contoso-research-main`.

   Set `Effective date` to `2026-06-01`.

   Set `Currency` to `USD`.



   Confirm one-time upgrade charge is `USD 0.00`.

   Confirm projected delta is `USD 1,600.00`.

   Set cost center to `RND-PLATFORM-2026`.

   Select `Send for finance approval`.

   Address the approval to `li.na@contoso.example`.


   Screenshot to capture: `tier-upgrade-contoso-09-finops-preview.png`.

10. Record finance approval.

   Open `Approvals`.

   Wait for Li's approval row.

   For this tutorial fixture, select `Simulate approval response`.

   Choose approver `li.na@contoso.example`.

   Choose decision `Approved`.

   Enter approval reference `FIN-APPROVAL-2026-05-20-PLATINUM`.

   Select `Record approval`.

   Confirm approval status changes to `Approved`.

   Confirm `billing.subscription.change` was evaluated.

   Confirm the audit event `TierUpgradeFinanceApproved` appears in the activity rail.

   Save the approval evidence to Drive as `finance-approval.json`.

   Screenshot to capture: `tier-upgrade-contoso-10-finance-approval.png`.

   Do not activate the tier yet.

   Finance approval is necessary but not sufficient.

11. Send the activation request to Marcus.

   Select `Activation`.

   Confirm all checks above `Executive approval` are green.

   Select `Request activation approval`.

   Set approver to `marcus.reed@contoso.example`.


   Set effective time to `Immediate after approval`.

   Select `Send approval request`.

   Open the approval drawer.

   Confirm request id `TIER-ACTIVATE-CONTOSO-2026-05-20`.

   For the tutorial fixture, select `Simulate approval response`.

   Choose decision `Approved`.


   Select `Record executive approval`.

   Screenshot to capture: `tier-upgrade-contoso-11-executive-approval.png`.




   Review the confirmation checklist.



   Check `No product fork will be created`.

   Check `Audit-chain evidence will be sealed`.

   Type confirmation phrase `ACTIVATE PLATINUM`.

   Select `Activate`.

   Wait until the progress step `Capability grant active` is complete.


   Wait until the progress step `Policy bundle published` is complete.

   Wait until the progress step `Ontology projections applied` is complete.

   Wait until the progress step `Workflow templates loaded` is complete.

   Screenshot to capture: `tier-upgrade-contoso-12-activation-progress.png`.


   Select `Current tier`.


   Confirm the active grant id reads `grant-contoso-platinum-2026`.

   Expand `Previous grants`.

   Confirm `grant-contoso-bronze-2026` appears with status `superseded`.

   Confirm `Superseded by` reads `grant-contoso-platinum-2026`.

   Confirm the supersede reason reads `customer_success_tier_upgrade`.

   Select `View audit`.

   Confirm audit event `CapabilityTierGrantActivated`.

   Confirm audit event `CapabilityTierGrantSuperseded`.

   Confirm both events share bundle id `tier-upgrade-contoso-bronze-platinum-2026-05-20`.

   Screenshot to capture: `tier-upgrade-contoso-13-current-tier.png`.


   Open a new tab in the same tenant.

   Navigate to `Home`.

   The sidebar should now show `Enterprise Dashboard`.

   Select `Enterprise Dashboard`.

   Confirm the dashboard title reads `Contoso Research Enterprise Overview`.

   Confirm widget `Workflow throughput` appears.

   Confirm widget `Contract intelligence` appears.

   Confirm widget `Marketplace settlement health` appears.

   Confirm widget `Compliance evidence status` appears.

   Open `Command-K`.

   Search `contract intelligence`.

   Confirm command `Open contract intelligence workspace` appears.

   Search `advanced workflow`.

   Confirm command `Create advanced workflow` appears.

   Screenshot to capture: `tier-upgrade-contoso-14-platinum-shell.png`.

15. Smoke-test a newly unlocked capability without changing production data.

   Navigate to `Intelligence`.

   Select `Contract summaries`.

   Select `New summary`.

   Choose fixture file `sample-northwind-msa-20p-redacted.pdf`.

   Set mode to `Preview only`.

   Set purpose to `productivity_ai_summary`.

   Select `Run preview`.

   Confirm the job starts.

   Confirm the job result says `Preview summary generated`.

   Confirm no final document is written to Drive.

   Confirm audit event `PlatinumCapabilityPreviewed`.

   Cancel the preview job if it remains running after the result is visible.

   Screenshot to capture: `tier-upgrade-contoso-15-intelligence-preview.png`.

16. Export the sealed audit evidence packet.

   Navigate back to `Tenant Admin -> Capability Tiers -> Upgrade`.

   Select `Evidence`.

   Select `Build evidence packet`.


   Include `Cedar preview`.

   Include `Ontology projection diff`.

   Include `Workflow template preview`.

   Include `Microservice readiness`.

   Include `Finance approval`.

   Include `Executive approval`.

   Include `Activation events`.

   Include `Post-activation smoke test`.

   Set export name to `contoso-platinum-tier-upgrade-evidence-2026-05-20`.

   Select `Seal and export`.

   Expected seal status: `sealed`.


   Screenshot to capture: `tier-upgrade-contoso-16-evidence-packet.png`.

17. Run the tutorial verification query from the UI.

   Open `Audit`.

   Select `Queries`.

   Search for `tutorial.capability_tier_upgrade_status`.

   Select the query.

   Set `tenant_id` to `tenant-contoso-research`.

   Set `source_grant_id` to `grant-contoso-bronze-2026`.

   Set `target_grant_id` to `grant-contoso-platinum-2026`.

   Set `bundle_id` to `tier-upgrade-contoso-bronze-platinum-2026-05-20`.

   Select `Run query`.

   Expected output label: `contoso_platinum_ready`.

   Expected `source_state`: `superseded`.

   Expected `target_state`: `active`.

   Expected `product_fork_created`: `false`.

   Expected `audit_packet_sealed`: `true`.

   Screenshot to capture: `tier-upgrade-contoso-17-ui-verification.png`.

18. Run the same verification query from the command line.

   Open the integrated tenant shell from `Developer Tools -> Tenant Shell`.

   Confirm the shell header reads `tenant-contoso-research`.

   Run this command.

   ```bash
   oya query run tutorial.capability_tier_upgrade_status \
     --tenant tenant-contoso-research \
     --param source_grant_id=grant-contoso-bronze-2026 \
     --param target_grant_id=grant-contoso-platinum-2026 \
     --param bundle_id=tier-upgrade-contoso-bronze-platinum-2026-05-20
   ```

   The command output should include `result_label: contoso_platinum_ready`.

   The command output should include `cedar_policy_status: active`.

   The command output should include `ontology_projection_status: applied`.

   The command output should include `workflow_template_status: loaded`.

   The command output should include `billing_status: approved`.

   The command output should include `bronze_superseded: true`.

   The command output should include `platinum_active: true`.

   Screenshot to capture: `tier-upgrade-contoso-18-shell-verification.png`.

## Verification

Run the named query `tutorial.capability_tier_upgrade_status`.

Use the UI query runner for a human-readable status.

Use the tenant shell for copyable evidence.

The query parameters are:

- `tenant_id=tenant-contoso-research`
- `source_grant_id=grant-contoso-bronze-2026`
- `target_grant_id=grant-contoso-platinum-2026`
- `bundle_id=tier-upgrade-contoso-bronze-platinum-2026-05-20`

Expected output:

```text
result_label: contoso_platinum_ready
tenant_id: tenant-contoso-research
source_grant_id: grant-contoso-bronze-2026
target_grant_id: grant-contoso-platinum-2026
source_state: superseded
target_state: active
bronze_superseded: true
platinum_active: true
cedar_policy_status: active
cedar_personal_tenant_denies: 1
ontology_projection_status: applied
ontology_added_views: 4
ontology_cross_tenant_leaks: 0
workflow_template_status: loaded
workflow_template_count: 4
ux_shell: enterprise-platinum-shell
billing_status: approved
finance_reference: FIN-APPROVAL-2026-05-20-PLATINUM
executive_approval: approved
product_fork_created: false
audit_packet_sealed: true
```

The success condition is not just `target_state: active`.

The success condition also requires `source_state: superseded`.

The success condition also requires `product_fork_created: false`.

The success condition also requires `audit_packet_sealed: true`.

The success condition also requires `ontology_cross_tenant_leaks: 0`.

The success condition also requires `cedar_policy_status: active`.

If any value differs, the upgrade is not complete for tutorial purposes.

Keep screenshots 01 through 18 in the evidence folder.

Keep JSON exports in the same evidence folder.

Do not close the customer-success case until the query returns the expected output.

## Common Pitfalls + Recovery

- Pitfall: Evelyn signs in to her personal tenant.
- Recovery: stop immediately, switch to `tenant-contoso-research`, and repeat step 1.
- Pitfall: `grant-contoso-bronze-2026` is missing.
- Recovery: inspect tenant history; do not infer source state from billing alone.
- Pitfall: the preview says `product fork will be created`.
- Recovery: stop and reject the template; ADR-0316 requires capability-tier grants, not service forks.
- Pitfall: Cedar preview permits a personal tenant resource.
- Recovery: repair the policy context and rerun the preview before finance approval.
- Pitfall: ontology projection diff shows `cross_tenant_leaks=1`.
- Recovery: block activation and inspect projection filters on `ConsentRecord` and `DealSet`.
- Pitfall: workflow templates try to modify running workflows.
- Recovery: change load mode to `add templates only`; running workflow mutation is out of scope.
- Pitfall: `intelligence` health is degraded.
- Pitfall: finance approval is simulated before cost preview is generated.
- Recovery: delete the approval fixture and regenerate the cost preview with Li as approver.
- Pitfall: Marcus approves from the wrong tenant.
- Recovery: void the approval and resend from `tenant-contoso-research`.
- Recovery: use `Capability Tiers -> Current tier -> Resolve grant conflict` and rerun verification.
- Pitfall: the UX shell does not show `Enterprise Dashboard`.
- Recovery: refresh tenant shell cache and confirm `enterprise-platinum-shell` is bound.
- Pitfall: smoke test writes a final AI summary.
- Recovery: delete the test artifact and rerun with `Preview only`.
- Pitfall: evidence packet is exported without a seal.
- Recovery: rebuild the packet with `Seal and export`; screenshots alone are not enough.
- Pitfall: billing status is `pending`.
- Recovery: do not mark upgrade complete; finance approval must be in the query output.
- Pitfall: query output says `product_fork_created: true`.
- Recovery: halt and file an architecture defect against the capability registry implementation.
- Pitfall: the upgrade creates separate `platinum-drive` or `platinum-mail` services.
- Recovery: disable the forked services; tier behavior belongs in grants, projections, UX shell, and workflow templates.
- Pitfall: audit packet path is outside tenant evidence.
- Pitfall: a screenshot contains confidential customer records.
- Recovery: delete it, retake the screenshot with only tutorial fixture rows visible, and update the evidence packet.
- Pitfall: the command-line query runs against a local developer tenant.
- Recovery: confirm the shell header reads `tenant-contoso-research` before running the command.
- Pitfall: the tutorial ends after UI activation without command-line verification.
- Recovery: run step 18 and attach the output to the evidence folder.

## Next Tutorials

- [First-day-on-Oyatie quickstart](quickstart-new-user-day-one.md).
- [Build an employee-onboarding workflow](workflow-studio-build-employee-onboarding.md).
- [Use intelligence to summarize a 200-page contract](ai-assisted-document-summarization.md).
- [List, sell, buy, and settle a marketplace deal](marketplace-list-sell-buy.md).
- [Activate HIPAA, SOC 2, GDPR, and KR-PIPA on a single tenant](multi-pack-tenant-activation.md).

## References

- [Capability Tier Over Product Fragmentation ADR](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md).
- [ADR-0311 Dual-Tenant Identity Personal vs Work Boundary](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md).
- [ADR-0314 Marketplace as Universal Deal Settlement](../decisions/ADR-0314-marketplace-as-universal-deal-settlement.md).
- [Capability tier matrix standard](../standards/capability-tier-matrix.md).
- [Tenant lifecycle standard](../standards/tenant-lifecycle.md).
- [Cedar Policy Authoring Standard](../standards/cedar-policy-authoring.md).
- [Ontology Projection Substrate Standard](../standards/ontology-projection-substrate.md).
- [Workflow Substrate Engine Standard](../standards/workflow-substrate-engine.md).
- [Compliance Evidence Automation Standard](../standards/compliance-evidence-automation.md).
- [Documentation Rigor](../standards/documentation-rigor.md).
- [Doc Style](../standards/doc-style.md).
