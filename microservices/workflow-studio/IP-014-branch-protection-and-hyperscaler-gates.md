---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-014-branch-protection-and-hyperscaler-gates
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow + ops-sre-reliability
acceptance_lanes: [oya-governance-branch-protection-conformance]
depends_on: [IP-013]
---

# IP-014: branch-protection.yaml + hyperscaler-gates.json updates

## Intent

Update `.github/branch-protection.yaml` to add Studio-specific required status checks on `dev` + `staging` branches and add pattern-based protection for `release/workflow-studio/{staging,production}`. Add HG-WORKFLOW-STUDIO entry to `/specs/hyperscaler-gates.json`.

## ChangeSet boundary

Two files updated:
- `.github/branch-protection.yaml` — additive only (does not remove existing checks).
- `/specs/hyperscaler-gates.json` — append HG-WORKFLOW-STUDIO entry.

## Concrete File Targets

| Path | Action |
|---|---|
| `.github/branch-protection.yaml` | update | add 5 Studio governance lanes to dev + staging required_status_checks; add 2 pattern protections for release/workflow-studio/{staging,production} |
| `/specs/hyperscaler-gates.json` | update | append HG-WORKFLOW-STUDIO gate entry |

## Code Shape

`.github/branch-protection.yaml` (additive section):

```yaml
branches:
  dev:
    required_status_checks:
      # existing checks plus:
      - oya-governance-workflow-spec-roundtrip
      - oya-governance-cedar-preview-required
      - oya-governance-editor-execution-forbidden
      - oya-governance-node-library-determinism
      - oya-governance-node-library-signature-verification
      - oya-governance-wasm-bundle-sri
      - oya-governance-xss-vector-scan
      - oya-governance-citus-rls-enforced
      - oya-governance-cdn-cache-key-tenant-isolated
      - oya-governance-no-tenant-branding-mid-render
      - oya-governance-llm-assist-validation-required
  staging:
    required_status_checks:
      - oya-governance-workflow-spec-roundtrip
      - oya-vcs-promotion-readiness

  ? release/workflow-studio/staging
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-vcs-promotion-readiness

  ? release/workflow-studio/production
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-vcs-promotion-readiness
```

`/specs/hyperscaler-gates.json` (append):

```json
{
  "gate_id": "HG-WORKFLOW-STUDIO",
  "microservice": "workflow-studio",
  "claim_class": "hero-product-editor-substrate",
  "verifying_lanes": [
    "oya-governance-workflow-spec-roundtrip",
    "oya-governance-cedar-preview-required",
    "oya-governance-editor-execution-forbidden",
    "oya-governance-node-library-determinism",
    "oya-governance-node-library-signature-verification",
    "oya-governance-wasm-bundle-sri",
    "oya-governance-xss-vector-scan",
    "oya-governance-citus-rls-enforced",
    "oya-governance-cdn-cache-key-tenant-isolated",
    "oya-governance-no-tenant-branding-mid-render",
    "oya-governance-llm-assist-validation-required"
  ],
  "competitor_parity_doc": "microservices/workflow-studio/competitor-parity-matrix.md",
  "claim_boundary": "No availability, latency, connector breadth, or superiority claim until measured by Oyatie gates and backed by source evidence per ADR-0123.",
  "registered_at": "2026-05-17",
  "registered_by": "axis-workflow + council-architecture"
}
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate branch-protection-conformance
cargo run -p oya-dev-cli -- gate validate hyperscaler-gates-registry
cargo run -p oya-dev-cli -- gate validate authority-cohesion
```

## Test Plan

| Test | Verifies |
|---|---|
| branch-protection.yaml schema valid | gh branch-protection-rule-validator passes |
| no removal of existing checks | diff verifies additive-only |
| HG-WORKFLOW-STUDIO present | hyperscaler-gates.json schema valid |
| release pattern protection live | `gh api repos/.../branches/release/workflow-studio/staging/protection` returns 200 |

## Halt Conditions

- branch-protection.yaml schema invalid — STOP.
- Existing checks removed — STOP. additive-only invariant breach.

## Next IP

[`IP-015-hg-workflow-studio-registration-final.md`](IP-015-hg-workflow-studio-registration-final.md)

## References

- ADR-0123 Hyperscaler maturity claim gate.
- ADR-0130 Agentic SLO-gated promotion (release pointer pattern).
- docs/standards/git-workflow.md.
