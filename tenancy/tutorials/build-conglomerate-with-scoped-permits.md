---
doc_class: Tutorial
microservice: tenancy
persona: tenancy-engineer + platform-engineer
related_adrs: [ADR-TEN-001, ADR-0313]
date: 2026-05-20
doc_status: published
---

# Tutorial — Build a 3-level conglomerate hierarchy with scoped permits + sovereign child veto

You will: create a 3-level conglomerate (holding → operations → subsidiary), bind parent-child relationships with different relationship types, grant + revoke scoped permits, demonstrate sovereign-child pack veto, walk the full audit-chain trail. Total time ≤ 90 minutes.

## Pre-requisites

- A tenant with `tenant_class = paid`.
- `dev-cli` ≥ 1.42.0.
- Three tenant-admin principals (one per tenant level).
- KYB-clearance evidence files (`./kyb-evidence-*.json`).

## Step 1 — Create the holding company tenant (≤ 10 min)

```sh
# 1. Holding company (top of conglomerate)
oya tenancy tenant create \
    --cell prod-us-east-1 \
    --tenant-id acme-holdings \
    --kind regulated_b2b \
    --display-name "Acme Holdings LLC" \
    --requesting-principal u-ceo@acme-holdings.com \
    --requested-pack-set "default,sox,gdpr"
# Output: tenant_id=acme-holdings, state=requested

# 2. Fast-track to active (drill only; production has 7-d KYB)
oya tenancy tenant fast-track-active \
    --tenant acme-holdings \
    --kyb-evidence ./kyb-evidence-acme-holdings.json
# Output: state=active, home_cell=prod-us-east-1
```

## Step 2 — Create the operations subsidiary tenant (≤ 10 min)

```sh
# Operations division
oya tenancy tenant create \
    --cell prod-us-east-1 \
    --tenant-id acme-operations \
    --kind regulated_b2b \
    --display-name "Acme Operations Inc" \
    --requesting-principal u-coo@acme-operations.com \
    --requested-pack-set "default,sox,gdpr"

oya tenancy tenant fast-track-active \
    --tenant acme-operations \
    --kyb-evidence ./kyb-evidence-acme-operations.json
```

## Step 3 — Create the healthcare subsidiary tenant (sovereign child) (≤ 10 min)

```sh
# Healthcare arm — has HIPAA pack which is stricter
oya tenancy tenant create \
    --cell prod-us-east-1 \
    --tenant-id acme-pharma \
    --kind healthcare_provider \
    --display-name "Acme Pharma Inc" \
    --requesting-principal u-pharma-admin@acme-pharma.com \
    --requested-pack-set "default,sox,gdpr,hipaa"   # adds hipaa

oya tenancy tenant fast-track-active \
    --tenant acme-pharma \
    --kyb-evidence ./kyb-evidence-acme-pharma.json \
    --hipaa-baa-evidence ./hipaa-baa-acme-pharma.pdf
```

## Step 4 — Build the conglomerate hierarchy (≤ 15 min)

Create the relationships:

```sh
# Level 1: acme-holdings owns acme-operations
oya tenancy relationship create \
    --parent-tenant acme-holdings \
    --child-tenant acme-operations \
    --type owns \
    --starts-at 2024-01-01T00:00:00Z \
    --ends-at 2099-12-31T23:59:59Z \
    --pack-scope "default,sox,gdpr"
# Output: relationship_id=r_acme_holdings_ops

# Level 2: acme-operations owns acme-pharma
oya tenancy relationship create \
    --parent-tenant acme-operations \
    --child-tenant acme-pharma \
    --type owns \
    --starts-at 2024-01-01T00:00:00Z \
    --ends-at 2099-12-31T23:59:59Z \
    --pack-scope "default,sox,gdpr"   # NOT hipaa; pharma has stricter

# Optionally: cross-relationship (acme-holdings audits acme-pharma)
oya tenancy relationship create \
    --parent-tenant acme-holdings \
    --child-tenant acme-pharma \
    --type audits \
    --starts-at 2024-01-01T00:00:00Z \
    --ends-at 2099-12-31T23:59:59Z \
    --pack-scope "audit_only"
# Output: relationship_id=r_acme_holdings_pharma_audit
```

Verify the conglomerate:

```sh
oya tenancy conglomerate show --root-tenant acme-holdings
# Output:
#   conglomerate_root: acme-holdings
#   tree:
#     acme-holdings (holds: acme-operations, audits: acme-pharma)
#       └─ acme-operations (holds: acme-pharma)
#           └─ acme-pharma (sovereign: hipaa pack)
#   total_tenants: 3
#   max_depth: 2
```

## Step 5 — Grant scoped permit for billing access (≤ 10 min)

Holding company needs to see billing across the conglomerate but NOT raw transaction data:

```sh
# Grant: acme-holdings can read billing summaries from acme-operations
oya tenancy permit create \
    --relationship r_acme_holdings_ops \
    --action-namespace "cloud-billing::summary::read,cloud-billing::invoice::list" \
    --resource-scope "tenant=acme-operations" \
    --purpose "consolidated_financial_reporting" \
    --expires-at 2027-01-01T00:00:00Z \
    --approved-by u-cfo@acme-holdings.com
# Cedar evaluates:
#   - tenancy::relationship::permit_create ✓
#   - approver has cfo role at parent ✓
#   - action_namespace is in allowed list ✓
# Output:
#   grant_id: pg_billing_001
#   audit_event_id: ae_ten_permit_granted_billing

# Verify it works
oya cloud-billing summary fetch \
    --tenant acme-operations \
    --requesting-tenant acme-holdings \
    --requesting-user u-cfo@acme-holdings.com \
    --period 2026-Q2
# Cedar evaluates:
#   - cross-tenant: permit pg_billing_001 active ✓
#   - action in permit's allowed actions ✓
# Output: returns billing summary
```

Now try data access (NOT granted):

```sh
oya drive file list \
    --tenant acme-operations \
    --requesting-tenant acme-holdings \
    --requesting-user u-cfo@acme-holdings.com
# Cedar denies (drive::file::list not in permit's action_namespace)
# Output: 403 Forbidden
```

## Step 6 — Sovereign child pack veto (≤ 10 min)

Holding company tries to grant itself data access to the healthcare subsidiary's PHI:

```sh
oya tenancy permit create \
    --relationship r_acme_holdings_pharma_audit \
    --action-namespace "drive::file::decrypt" \
    --resource-scope "tenant=acme-pharma,data_class=PHI" \
    --purpose "audit_records_review" \
    --expires-at 2027-01-01T00:00:00Z
# Cedar denies:
#   - tenancy::relationship::parent_override_sovereign_child FORBIDDEN
#   - child pack (hipaa) denies parent PHI access
#   - higher-restriction-wins: hipaa pack rule blocks
# Output: 403 Forbidden
# Error: "child_pack_denies_parent_phi_access"
```

The HIPAA pack on acme-pharma denies parent data-plane access to PHI even though the parent administratively `owns` (via acme-operations) and `audits` it. Sovereign child veto in action.

Debug:

```sh
oya tenancy permit debug \
    --proposed-action "drive::file::decrypt" \
    --proposed-resource "tenant=acme-pharma,data_class=PHI" \
    --relationship r_acme_holdings_pharma_audit
# Output:
#   cedar_decision: deny
#   reason_code: child_pack_denies_parent_phi_access
#   child_pack_set: ["default", "sox", "gdpr", "hipaa"]
#   conflicting_pack: hipaa
#   pack_rule_id: hipaa-rule-phi-access-tenant-bound
#   winning_restriction_level: 9
#   override_path: child_tenant_explicit_grant_required AND hipaa_pack_does_not_permit
```

The pharma subsidiary's admin would need to explicitly grant this access AND the HIPAA pack must permit. Since HIPAA doesn't permit parent PHI access, the action is denied at construction.

## Step 7 — Permit revocation + workflow cancellation (≤ 10 min)

Revoke the billing permit (e.g., when the CFO changes):

```sh
oya tenancy permit revoke \
    --grant pg_billing_001 \
    --reason "CFO transitioned to advisory role" \
    --requesting-principal u-board-chair@acme-holdings.com
# Cedar evaluates:
#   - tenancy::permit::revoke ✓
# Output:
#   state: revoked
#   audit_event_id: ae_ten_permit_revoked_001
#   active_workflows_using_grant: 2 (cancellation events emitted)
```

Server emits `tenancy.permit.revoked.v1` to Kafka. Long-running workflows (consolidated financial reporting jobs) receive cancellation:

```sh
oya workflow status --workflow-class "consolidated-financial-reporting"
# Output:
#   - workflow_id: wf_001
#     state: cancelled
#     cancellation_reason: tenancy_permit_revoked
#     permit_id: pg_billing_001
```

## Step 8 — Spinoff/divestiture ceremony (paid tenant_class expanded deployment; ≤ 15 min)

Suppose acme-holdings sells acme-pharma to an external buyer (new-buyer-corp). The spinoff ceremony:

```sh
# 1. Initiate divestiture
oya tenancy divestiture initiate \
    --from-conglomerate-root acme-holdings \
    --child-tenant acme-pharma \
    --to-conglomerate-root new-buyer-corp \
    --transfer-effective-date 2026-08-01T00:00:00Z \
    --requesting-principal u-board-chair@acme-holdings.com \
    --legal-evidence ./divestiture-contract-2026-05-20.pdf
# Cedar evaluates:
#   - tenancy::divestiture::initiate ✓
#   - council approval present ✓
# Output:
#   divestiture_id: div_001
#   audit_event_id: ae_ten_divestiture_001
#   state: pending_data_transfer_evaluation

# 2. Pack residency evaluation (HIPAA stays with the entity)
oya tenancy divestiture evaluate \
    --divestiture div_001
# Output:
#   pack_set_transfers: ["default", "gdpr", "hipaa"]
#   pack_set_retained_by_origin: ["sox"]   # SOX bound to acme-holdings, not the entity
#   data_transfer_scope: full
#   estimated_wall_clock: 14 d
#   regulator_notification_required: true

# 3. Execute (server orchestrates cross-conglomerate transfer)
oya tenancy divestiture execute \
    --divestiture div_001 \
    --regulator-notification-evidence ./hhs-notification-2026-07-15.pdf
# After 14 d transfer wall-clock:
# Output:
#   state: completed
#   new_parent: new-buyer-corp
#   relationships_removed: [r_acme_operations_pharma, r_acme_holdings_pharma_audit]
#   relationships_created: [r_new_buyer_pharma]
#   audit_event_id: ae_ten_divestiture_completed_001
```

## Step 9 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant acme-holdings --event-class "tenancy.*" --since 90m
```

Expected events:

- `tenancy.tenant.created.v1` (× 3; holdings + operations + pharma)
- `tenancy.lifecycle.transitioned.v1` (multiple; each per transition)
- `tenancy.relationship.created.v1` (× 3; ops→pharma; holdings→ops; holdings→pharma audit)
- `tenancy.permit.granted.v1` (× 1; billing permit)
- `tenancy.permit.denied.v1` (× 1; sovereign-child-veto on PHI)
- `tenancy.permit.revoked.v1` (× 1; CFO transition)
- `tenancy.divestiture.initiated.v1`
- `tenancy.divestiture.completed.v1`

All Ed25519-signed; chain verifies:

```sh
oya audit verify-chain --tenant acme-holdings --since 90m
```

## What you've learned

- Three-level conglomerate hierarchy.
- Multiple relationship types between same tenant pair (owns + audits).
- Scoped permits with action_namespace + resource_scope.
- Sovereign-child pack veto (HIPAA denies parent PHI access).
- Permit revocation with workflow cancellation.
- Spinoff/divestiture ceremony with regulator notification.
- Audit-chain verification of the full conglomerate lifecycle.

Next tutorial: `tutorials/migrate-tenant-across-regions.md` — execute a council-approved cross-region tenant migration (paid tenant_class expanded deployment).
