---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-012-branch-protection-and-release-pointers
status: pending
owner: ops-sre-reliability
acceptance_lanes: [governance-protection-context-match]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: branch-protection.yaml + release pointers for tenancy

## Intent

Update `.github/branch-protection.yaml` to add tenancy-specific BLOCKER lanes (rls-no-superuser-bypass, rls-force-on-tenant-tables, jwt-key-fingerprint-advertised); add pattern protection for `release/tenancy/{staging,production}` per ADR-0139; establish initial release pointers for tenancy.

## Concrete File Targets

| Path | Action |
|---|---|
| `.github/branch-protection.yaml` | update — add 3 lanes to dev + 2 to staging; pattern rules for release/tenancy/{staging,production} |
| `release/tenancy/staging` Git ref | create (initial pointer at first green dev SHA) |
| `release/tenancy/production` Git ref | create (initial pointer at staging pointer) |

## branch-protection.yaml diff (preview)

```yaml
branches:
  dev:
    required_status_checks:
      # existing...
      - governance-rls-no-superuser-bypass        # NEW
      - governance-rls-force-on-tenant-tables     # NEW
      - governance-jwt-key-fingerprint-advertised # NEW
      - governance-tenancy-residency-conformance  # NEW
      - governance-tenancy-cedar-coverage         # NEW

  ? release/tenancy/staging
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    required_status_checks:
      - governance-promotion-readiness
      - governance-rls-no-superuser-bypass
      - governance-rls-force-on-tenant-tables

  ? release/tenancy/production
  :
    # identical pattern
```

## Acceptance Gates

```bash
cargo run -p dev-cli -- gate validate protection-context-match
gh api repos/jason931225/oyatie/branches?protected=true | jq '.[].name'
```

## Test Plan

- Verify branch-protection.yaml schema valid.
- Verify GitHub-side rule application: push protection refuses force-push to release pointers.
- Verify tenancy-specific lanes registered with GitHub status-check API.

## Next IP

[`IP-013-canary-cohort-and-rollback-wiring.md`](IP-013-canary-cohort-and-rollback-wiring.md)
