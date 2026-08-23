---
doc_class: Spec
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Define external-facing API versioning across all six axes: hybrid resource-name
  segment (v1 / v1beta1 / v1alpha1 / v2) plus optional date-based fine-grain via
  `?api-version=YYYY-MM-DD`, with OpenAPI 3.1 as source of truth and
  governance-api-version-stability as the BLOCKER gate.
planned_enforcement_ref: governance-api-version-stability
related_adrs: [ADR-0040, ADR-0041, ADR-0050]
doc_status: published
---

# External API Versioning Spec — oyatie

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Hybrid model: stability + date

Two orthogonal axes, both visible to clients:

1. **Stability segment** in the URL path: `v1` / `v1beta1` / `v1alpha1` / `v2`.
   Source: Google AIP-181. Kubernetes uses the same shape. A `v2` exists only
   when `v1` cannot satisfy the new contract without breakage.
2. **Date fine-grain** via query parameter: `?api-version=2026-05-12`.
   Source: Microsoft Azure (`api-version`) and AWS service date versions.
   Default = latest stable date within the path-version's stability tier.

Stripe-style header pinning (`Stripe-Version`) is **rejected** for oyatie:
header-only versioning is invisible to URL-based observability and inhibits
caching layers.

## 2. URL shape

```
https://api.oyatie.com/{axis}/{path-version}/{resource}[/{sub-resource}]?api-version=YYYY-MM-DD
```

| Axis | Example |
|---|---|
| Foundry | `/foundry/v1/capability/invoke?api-version=2026-05-12` |
| Cloud | `/cloud/v1/compute/instance/{id}` |
| SaaS | `/saas/v1beta1/workflow` |
| Workspace | `/workspace/v1/document/{id}` |
| Search | `/search/v1/index/query` |
| Ads | `/ads/v1/campaign` |

Per axis-prefix already established in `contracts/openapi/`.

## 3. Stability tiers (per AIP-181 + `docs/SPEC.md §10`)

| Tier | Path segment | Promise | SLA |
|---|---|---|---|
| **alpha** | `v1alpha1`, `v1alphaN` | breaking changes allowed any release | none |
| **beta** | `v1beta1`, `v1betaN` | minimised breakage; 90-d sunset on breaks | 99.0% |
| **stable / GA** | `v1`, `v2` | no breaking changes within major; 180-d sunset | 99.9% |

Promotion path: `v1alpha1` → `v1alpha2` → ... → `v1beta1` → ... → `v1`.

When `v1` cannot represent the new contract additively, mint `v2alpha1` and
follow the chain. Run BOTH `v1` and `v2` simultaneously for at least 12 months.

## 4. OpenAPI 3.1 source of truth

Every endpoint MUST appear in `contracts/openapi/{axis}/{path-version}.yaml`
with these extension fields:

```yaml
x-stability: stable          # alpha | beta | stable | ga
x-introduced: "2026-01-01"   # first api-version that exposes this op
x-deprecated: null           # YYYY-MM-DD; null if still current
x-sunset:     null           # YYYY-MM-DD; 180 d after deprecated
x-major:      1              # path-version major
```

The lane reads these fields to drive enforcement.

## 5. Date-based fine-grain (`api-version=YYYY-MM-DD`)

Per Azure + AWS pattern:

- Default (no header / no query): latest stable date in path-version's tier.
- Explicit date: server pins request to the operation contract as it existed on
  that date.
- Available dates published in `contracts/openapi/{axis}/{path-version}/dates.json`.
- Range: server keeps the last 24 months of date snapshots within a path-version.

Use the date axis ONLY for additive changes that benefit from explicit pinning
(new optional fields, new enum variants, new response envelopes). Breaking
changes ALWAYS bump the path-version, never just the date.

## 6. Backwards compatibility (per AIP-180)

Disallowed inside a path-version:

- Removing a field, method, resource.
- Renaming a field.
- Changing a field's type (even to a wire-compatible type).
- Tightening a validation rule (e.g. shrinking a max-length).
- Moving a field into or out of a oneOf.

Allowed inside a path-version:

- Adding optional fields.
- Adding new resources / methods.
- Adding new enum variants (provided the field is documented as open-enum).
- Loosening validation rules.

## 7. Pre-release labels

| Stripe-style date suffix | Use |
|---|---|
| `?api-version=2026-05-12-preview` | preview-only operations behind a tenant flag |
| `?api-version=2026-05-12` | stable date within current GA path-version |

Preview dates have the same 90-day churn rule as Azure preview.

## 8. Enforcement lane

`governance-api-version-stability` (BLOCKER) checks every PR that
touches `contracts/openapi/**`:

1. No field removed from a stable path-version.
2. No type change in a stable path-version.
3. Every new operation carries `x-introduced` ≥ today.
4. Every deprecated op carries `x-sunset` exactly 180 days after `x-deprecated`.
5. Stability promotion (alpha → beta → stable) requires an ADR.
6. Mint of a new major path-version (`v2`) requires the breaking-change ADR.

## 9. Client SDK alignment

Generated SDKs (Rust / TS / Python / Go per Directive 4) MUST:
- Expose the path-version as a constant on each client (`Client::V1`).
- Default `api-version` query parameter to the SDK build's pin date.
- Surface deprecation warnings emitted in the response header
  `api-deprecation: <sunset-date>` (per Kubernetes pattern).

## 10. Multi-axis contract bridges

Cross-axis contracts (e.g. Foundry → Search) are versioned by the producing
axis. The consuming axis pins to a specific path-version + date; bridge breaks
trigger the breaking-change process in BOTH axes.

## 11. Lift target

`oyatie/docs/release/api-versioning-spec.md` on approval.
