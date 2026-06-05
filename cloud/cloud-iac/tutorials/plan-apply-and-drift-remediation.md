# Tutorial — Plan, apply, and remediate drift on a multi-provider module

Goal: take a sample inputs file, generate a plan that touches AWS + Cloudflare, apply it, induce drift on Cloudflare,
detect and remediate. End-to-end on a loopback cloud-iac cell.

Pre-reqs:
- Loopback iac cell: `make dev-cell.up CELL=iac-loopback-1 PROFILE=cloud-iac-dev`
- Mock provider creds wired by `make dev-cell.up` (no real AWS/Cloudflare account needed)
- Tenant: `make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid`

## Step 1 — declare the inputs

`samples/paid/acme-website-stack.yaml`:
```yaml
module_set: oya-iac-modules-paid-per-usage-v1
modules:
  - name: aws-s3-static-site
    inputs:
      bucket_name: acme-software-website-prod
      region: us-east-2
      acl: private
  - name: aws-cloudfront-distribution
    inputs:
      origin: ${module.aws-s3-static-site.bucket_regional_domain_name}
      aliases: ["www.acme-software.io"]
      certificate_arn: ${ref:cloud_secrets.acme_software_io_cert_arn}
  - name: cloudflare-zone
    inputs:
      zone_name: acme-software.io
      plan: free
  - name: cloudflare-record
    inputs:
      zone_id: ${module.cloudflare-zone.zone_id}
      name: www
      type: CNAME
      value: ${module.aws-cloudfront-distribution.domain_name}
      proxied: true
```

## Step 2 — plan

```bash
cloud-iac native operator API action; verify through release-conveyor reconciliation evidence.
  --tenant oyatie.b2b.smb.acme-software \
  --inputs samples/paid/acme-website-stack.yaml \
  --module-set oya-iac-modules-paid-per-usage-v1
```

Expected:
```
plan_id        : plan-2026-05-20-acme-001
resources_to_add: 4
resources_to_change: 0
resources_to_destroy: 0
permits_required: 4 × cloud_iac::Action::Plan
permits_granted : 4 (all)
graph_signature : blake3-256:5b4a…
review_status   : NeedsReviewerAgent
```

The plan output is in `last-plan.json`; inspect with `jq`:
```bash
jq '.diff[] | {address, action, provider}' last-plan.json
```

## Step 3 — wait for reviewer-agent verdict

```bash
cloud-iac native operator API action; verify through release-conveyor reconciliation evidence.
```
Expect (within ~30 s on loopback):
```
reviewer_agent_decision: APPROVE
multispectrum_facets_passed: 11/11
```

## Step 4 — apply

```bash
cloud-iac native operator API action; verify through release-conveyor reconciliation evidence.
```

Watch the structured progress:
```
applying… 1/4 aws-s3-static-site                     ok in 3.1 s
applying… 2/4 aws-cloudfront-distribution            ok in 11.4 s
applying… 3/4 cloudflare-zone                        ok in 0.8 s
applying… 4/4 cloudflare-record                      ok in 0.4 s
apply_id        : apply-2026-05-20-acme-001
duration        : 15.9 s (under p95 = 4 min)
audit_chain_event_id: ce-2026-05-20T08:21:13.214Z-…
```

## Step 5 — induce drift via mock provider

```bash
cloud-iac native operator API action; verify through release-conveyor reconciliation evidence.
  --resource cloudflare_record.www_acme_software_io \
  --field proxied \
  --new-value false
```

## Step 6 — detect drift

```bash
cloud-iac native operator API action; verify through release-conveyor reconciliation evidence.
```
Expected:
```
drift_detected: 1 resource
  cloudflare_record.www_acme_software_io
    declared : proxied=true
    actual   : proxied=false
    severity : Warn (security-impacting; CDN bypass risk)
recommended_action: RemediateDrift
```

## Step 7 — remediate

```bash
cloud-iac native operator API action; verify through release-conveyor reconciliation evidence.
  --tenant oyatie.b2b.smb.acme-software \
  --resource cloudflare_record.www_acme_software_io
```
Expected:
```
remediate_id  : rem-2026-05-20-acme-001
permits_used  : 1 × cloud_iac::Action::RemediateDrift
outcome       : Success
duration      : 0.6 s
audit_chain_event_id: ce-2026-05-20T08:23:01.117Z-…
```

## Step 8 — verify state is clean

```bash
cloud-iac native operator API action; verify through release-conveyor reconciliation evidence.
```
Expect `drift_detected: 0 resources`.

## Step 9 — cleanup (optional)

```bash
cloud-iac native operator API action; verify through release-conveyor reconciliation evidence.
  --tenant oyatie.b2b.smb.acme-software \
  --inputs samples/paid/acme-website-stack.yaml
```

The `Destroy` action is Cedar-gated separately from `Apply`; paid tenants get an extra confirmation prompt and a reviewer-agent
checkpoint for governed operations.

## What you just demonstrated

- Multi-provider plans compose AWS + Cloudflare under one `plan_id` with a graph signature.
- The reviewer-agent gate happens before apply, not after — matches the Foundry admission-gate doctrine.
- Drift detection produces a structured diff with severity that translates to a remediation action.
- Remediation uses a separate Cedar action — auditable distinct from user-initiated `Apply`.
- Every step writes to `audit-chain`; reproducible by replaying the chain.
