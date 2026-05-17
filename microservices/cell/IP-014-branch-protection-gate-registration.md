---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-014-branch-protection-gate-registration
status: pending
owner: axis-foundry
acceptance_lanes: [oya-governance-branch-protection-conformance]
---

# IP-014: Register cell-boundary lane in branch-protection.yaml

## Intent

Wire the new `oya-cell-boundary` BLOCKER lane (from IP-006) into `.github/branch-protection.yaml` as a required status check on `dev` and `staging`. PRs that fail the lane cannot merge.

## Concrete File Targets

| Path | Action |
|---|---|
| `.github/branch-protection.yaml` | update (add `oya-cell-boundary` to required_status_checks) |
| `.github/workflows/cell-boundary-lane.yml` | create (GitHub Actions workflow running the lane) |
| `/specs/quality/lanes.yaml` | update (declare BLOCKER class) |

## Code Shape

```yaml
# .github/branch-protection.yaml (excerpt)
branches:
  dev:
    required_status_checks:
      strict: true
      contexts:
        - oya-vcs-promotion-readiness
        - oya-cell-boundary           # NEW (this IP)
        - oya-cell-rls-conformance    # also added; from IP-002
        - ...
  staging:
    required_status_checks:
      contexts:
        - oya-cell-boundary
        - ...
```

```yaml
# .github/workflows/cell-boundary-lane.yml
name: oya-cell-boundary
on:
  pull_request:
    paths:
      - 'microservices/**/sql/**'
      - 'microservices/**/migrations/**'
      - 'microservices/**/k8s/**'
      - 'microservices/cell/**'
      - 'crates/oya-check-cell-boundary/**'

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo run -p oya-dev-cli -- gate validate cell-boundary --microservice ${{ matrix.microservice }}
    strategy:
      matrix:
        microservice: [tenancy, ontology, workflow, cell, mail, observability]
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate branch-protection-conformance
gh api repos/oyatie/oya/branches/dev/protection | jq '.required_status_checks.contexts | map(select(. == "oya-cell-boundary"))'
```

## Test Plan

- Smoke: branch-protection.yaml passes schema validation.
- E2E: author a PR violating cell-boundary; verify GitHub blocks merge.

## Halt Conditions

- Lane registered but not actually wired to GitHub branch-protection — fix.

## Next IP

[`IP-015-hyperscaler-claim-gate.md`](IP-015-hyperscaler-claim-gate.md)

## References

- ADR-0130 (promotion-readiness lane pattern).
- `microservices/cell/IP-006-cell-boundary-gate-lane.md`.
- GitHub branch-protection API.
