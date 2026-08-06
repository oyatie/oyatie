---
id: ADR-0608
title: "Cedar deploy-parity gate (deployed ConfigMap ⊆ authored policy; no action-agnostic blanket permit)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-28
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
amended_by: []
depends_on: [ADR-0083, ADR-0183, ADR-0243, ADR-0510, ADR-0515, ADR-0535, ADR-0547, ADR-0548, ADR-0566]
amends: []
related: [ADR-0535, ADR-0547, ADR-0548, ADR-0566, ADR-0605, ADR-0606]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0608: Cedar deploy-parity gate (deployed ConfigMap ⊆ authored policy; no action-agnostic blanket permit)

## Status

**Proposed - 2026-06-28 (authored for founder sign-off; door: one-way).**

## Context

GH #16 (deployed-vs-authored Cedar policy parity): every `<cap>/iac/k8s/helm/templates/cedar.yaml`
ConfigMap embeds the Cedar policy the in-cluster PDP loads at runtime. All such ConfigMaps were
stamped from a single byte-identical Helm template carrying a BLANKET, action-agnostic grant —
`permit(principal, action, resource) when { resource.microservice == "{{ .Values.microservice.id }}"
&& principal.tenant_class == "{{ .Values.microservice.tenantClass }}" }`. An unconstrained action
head authorizes EVERY action for whatever the `when` clause admits, and the policy bears no relation
to each capability's rich AUTHORED policy. Under Cedar union semantics a deployed blanket permit
can only widen authorization, never narrow it — so a deployed ConfigMap is the authorization surface
that actually runs, regardless of what was authored.

A one-time disarm of the blanket (re-pointing each LIVE, non-deprecated service at its real authored
policy — for the platform PBAC core, `iam/adapters/pdp-cedar/cedar/platform-policies.cedar`)
does not prevent recurrence: the next chart can re-stamp the blanket. Per the
pipeline-as-product / friction-is-a-process-failure doctrine (ADR-0548), the recurrence CLASS must be
closed by a born-blocking, hermetic gate, not a manual review. The disarm itself is a SEPARATE,
sequenced follow-up IP (it must determine the correct live target per service and is out of this
lane); this ADR ships only the gate.

## Decision

Ship a **self-contained cloud-ci gate**, `cloud-ci-cedar-deploy-parity`
(`ci/facade/policy-deploy-parity`), mirroring the registration footprint of
the supply-chain-audit (ADR-0605) and operator-secret-bootstrap (ADR-0606) gates: own crate, own
policy JSON, one appended matrix line in `.github/workflows/oya-ci-required.yml`, no
`libs/oya-ci-config` edit, no producer-face binding.

### D1 — Pure, policy-driven, hermetic

The gate is owned pure-Rust (`#![forbid(unsafe_code)]`, no `Command::new`, no shell-out, no network,
no clock, no VCS). The only dependency is the workspace-inherited `serde_json` — ZERO new crate
enters `Cargo.lock`. The deployed ConfigMaps are Helm templates (not valid YAML — the `{{- if … }}`
wrapper breaks a YAML parse), so the collector extracts the embedded Cedar `|` block scalar as text
and analyses it with an owned, string-aware Cedar statement splitter; no `cedar-policy` engine
dependency is pulled into the gate. The only I/O is a hermetic, read-only fs walk
([`collect`]); the verdict is a pure `evaluate_keyed(policy, observed)`. This satisfies the
hermetic-gate bar (ADR-0548) and rust-purity (ADR-0547).

### D2 — The two invariants (the verdict)

For each deployed Cedar ConfigMap the policy DATA selects (every file whose repo-relative path ends
with `deployed_suffix`), `evaluate_keyed` asserts:

- **CHECK-A — no unconstrained-head permit.** A deployed `permit` whose HEAD leaves the action
  unconstrained (a bare `action`, not `action == Action::"…"` or `action in [ … ]`) is over-broad by
  construction → `CDP-UNCONSTRAINED-PERMIT`. A deployed `permit` whose HEAD leaves bare `resource`
  and whose `when` clause has no resource/scope predicate is also over-broad for production policy →
  `CDP-UNCONSTRAINED-RESOURCE`. Authored PBAC (RBAC + ABAC) policy must name the action and constrain
  resource/scope; a predicate cannot redeem an unconstrained action head.
- **CHECK-B — deployed allows ⊆ authored allows.** Cedar returns allow only when at least one
  `permit` applies, no `forbid` applies, and otherwise defaults to deny. The gate therefore checks a
  conservative text-level sufficient condition: every deployed `permit`, after
  comment/annotation/whitespace normalization, must be present in the capability's AUTHORED permit set
  (`<cap>/<authored_subdir>/*.cedar`) → otherwise `CDP-DEPLOYED-NOT-SUBSET`; and when deployed permits
  are present, every authored `forbid` must also be present in the deployed policy → otherwise
  `CDP-DEPLOYED-NOT-SUBSET`. A deployed policy with no permits grants nothing under Cedar default-deny.
  A deployed ConfigMap with NO resolvable authored policy still fails closed →
  `CDP-NO-AUTHORED-BASELINE` (parity cannot be proven against nothing).

The gate is **fail-closed**: an un-extractable/un-parseable deployed Cedar policy
(`CDP-CEDAR-EXTRACT-FAILED`), an unknown `gate_id` (`CDP-POLICY-GATE-ID-MISMATCH`), a structurally
invalid observed view, or a scan that finds ZERO deployed ConfigMaps (`CDP-POLICY-MALFORMED`, a
guard against a vacuously-green run) all return RED.

### D3 — Born-blocking baseline, shrink-only (mirrors ADR-0605 `ignore[]`)

The GH #16 byte-identical blanket ConfigMaps predate this gate and their disarm is a sequenced
follow-up. Their repo-relative paths are recorded in `policy.baseline.paths` and the exact known
blanket Cedar authorization signature is recorded in `policy.baseline.policy_signatures`: an exception
is GRANDFATHERED (skipped by CHECK-A/CHECK-B) only when both the path and signature match. A
grandfathered entry remains DOCUMENTED, time-boxed (`remove_by`), and SHRINK-ONLY —
`CDP-STALE-BASELINE` flags a baseline path that is no longer present or whose signature changed, so it
must be dropped, after which the ConfigMap is fully checked. The baseline NEVER grows by automation: a
NEW or CHANGED deployed ConfigMap is checked in full. So the gate is born-blocking against regressions
from the first commit, the live tree is GREEN against the documented exact baseline (mergeable now),
and the blanket-disarm follow-up shrinks the baseline toward empty. This is the standard known-issue
ratchet (no rule is weakened — no new over-broad grant is ever admitted); it is NOT a path-only
exemption.

2026-06-29 GH #987 shrinks this baseline for the Cloud control-plane subset: the fourteen
`cloud/*/iac/k8s/helm/templates/cedar.yaml` files named in the audit finding now embed their authored
action/resource-specific PBAC policies and are no longer grandfathered. The remaining blanket baseline
is product-surface/deprecation-sensitive and must keep shrinking; it must not grow.

### D4 — Policy-as-data, nothing oyatie-specific in Rust

The scan suffix (`deployed_suffix`), the authored-policy subdirs (`authored_subdirs`), and the
shrink-only baseline live in `cedar-deploy-parity-policy.json`. The Rust carries no hardcoded path,
so the gate is reusable across any repo following the embedded-Cedar-ConfigMap pattern. It maps to
the existing `cloud-ci` capability (ADR-0562), so no new capability is registered, and (per ADR-0605/
ADR-0606) the in-crate policy JSON is reachable via the gate crate's `BUCK` `srcs` glob — no
`specs/reachability-registry.json` entry is required.

## Governed surfaces

The exact tracked paths this decision introduces and governs (born-accounting justification per
ADR-0568; one verbatim repo-relative path per line):

```
ci/facade/policy-deploy-parity/BUCK
ci/facade/policy-deploy-parity/Cargo.toml
ci/facade/policy-deploy-parity/OWNERS
ci/facade/policy-deploy-parity/cedar-deploy-parity-policy.json
ci/facade/policy-deploy-parity/src/lib.rs
ci/facade/policy-deploy-parity/tests/cedar_deploy_parity.rs
```

## Consequences

- **Born-blocking against regressions**: a new or changed deployed Cedar ConfigMap that carries an
  action-agnostic permit (`CDP-UNCONSTRAINED-PERMIT`), carries a bare-resource permit without a
  resource/scope predicate (`CDP-UNCONSTRAINED-RESOURCE`), grants more than its capability authored
  after Cedar forbid/default-deny semantics are accounted for (`CDP-DEPLOYED-NOT-SUBSET` /
  `CDP-NO-AUTHORED-BASELINE`), or no longer matches the exact grandfathered authorization signature
  (`CDP-STALE-BASELINE`) fails the `cloud-ci-cedar-deploy-parity` lane from the first commit.
- **Sequenced disarm**: the remaining 67 known-blanket product-surface ConfigMaps are grandfathered
  by path + signature in the documented baseline and remain RUNTIME over-broad until their
  deprecation/reorg-safe disarm IP re-points each live service at its real authored policy and shrinks
  the baseline. GH #987 removed the fourteen Cloud control-plane templates from the baseline by
  deploying their authored action/resource-specific PBAC policies, making the Cloud slice checked by
  `CDP-UNCONSTRAINED-PERMIT`, `CDP-UNCONSTRAINED-RESOURCE`, and subset parity like any new template.
  AUTHZ-004 removed the unused `oya/analytics` Helm Cedar ConfigMap template instead of deploying a
  replacement: the chart's Deployment did not mount the ConfigMap, so deleting the dead blanket avoids
  preserving an executable over-broad policy surface while shrinking the baseline by one.
- **Owned + hermetic**: `serde_json` only, no shell/network/clock/VCS, no `cedar-policy` engine in the
  gate — clears the AWS/Google "would they ship this as their gate" bar (consistent with ADR-0605/0606).
- **Accounting**: the new crate is owned by
  `ci/facade/policy-deploy-parity/OWNERS` (`cloud-ci-platform`) and reachable
  via its `BUCK` `rust_test` targets + the `oya-ci-required` matrix entry; it maps to the existing
  `cloud-ci` capability, so no new capability is registered.

## Alternatives considered

- **Parse the deployed ConfigMaps with `serde_yaml` + the `cedar-policy` engine**: rejected for the
  gate — the deployed files are Helm templates (`{{- if … }}` is not valid YAML) and pulling the
  Cedar engine in to evaluate untyped template fragments adds weight without buying soundness over a
  string-level over-broad-permit + statement-subset check. The owned text analyser is sufficient and
  fail-closed.
- **Disarm-then-empty-baseline in the original gate lane**: rejected — determining the correct LIVE,
  non-deprecated authz target per service (oya/identity is a deprecate target; cloud-iam is the IdP
  substrate; the PBAC core is `iam/adapters/pdp-cedar`) needed a separate decision + IP.
  GH #987 performs the scoped Cloud control-plane disarm only; remaining product/deprecation-target
  templates stay in the shrink-only baseline until their own reorg-safe migration lands.
- **A producer-face binding** (emit findings into the accounting registry): rejected — same R0
  portability rationale as ADR-0605 D-alternatives / ADR-0566.
