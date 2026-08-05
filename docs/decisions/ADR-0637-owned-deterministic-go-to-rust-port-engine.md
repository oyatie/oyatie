---
id: ADR-0637
title: "Owned deterministic Go-to-Rust port engine"
status: Accepted
doc_status: published
planning_impact: true
deciders: founder
owner: council-architecture
date: 2026-08-05
door: one-way
supersedes: []
superseded_by: []
depends_on: [ADR-0013, ADR-0092, ADR-0515, ADR-0538, ADR-0597, ADR-0613, ADR-0614, ADR-0633, ADR-0634, ADR-0635, ADR-0636]
amends: [ADR-0538]
related: [ADR-0548, ADR-0554, ADR-0555, ADR-0562, ADR-0605, ADR-0606, ADR-0608, ADR-0609, ADR-0615, ADR-0619, ADR-0623, ADR-0627, ADR-0628, ADR-0629, ADR-0632, ADR-0638]
related_specs:
  - /specs/k8s-port/upstream-pin.json
  - /specs/k8s-port/scope.json
  - /specs/k8s-port/divergence-ledger.json
  - /specs/k8s-port/licensing.json
milestone: W0-A
---

# ADR-0637: Owned deterministic Go-to-Rust port engine
## Baseline version header

| Authority | Version this ADR is authored against | Status at authoring (2026-08-05) |
|---|---|---|
| Repository baseline | `origin/dev@b64eaaf4a` | Pinned baseline; local `dev` is not pin evidence. |
| Kubernetes upstream | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67`; annotated tag object `5b824a493a7ca248b726b6ea09d53842b9b992c2` | Pinned for this decision. Evidence source: `git ls-remote` plus fleet `infra/talos/installation-media/presets.yaml`. |
| Engine | `oya-port` at `build/port-engine/*` | W0-B deliverable; unbuilt and not in force. |
| Neutral rules | `specs/port-rules/**` | W0-B deliverable; unauthored and not in force. |
| Kubernetes corpus policy | `specs/k8s-port/**` | Pin, scope, divergence, and licensing registries land and become active with W0-A; rules, boundary, and detachment registries remain W0-B/W0-C deliverables. |
| Source-model front end | Bootstrap artifact followed by owned Rust front end | W0-B measurement and artifact work; neither implementation exists or is in force. |
| Receipt axes | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Decision contract; receipt schema and gate wiring remain W0-B/W0-C pending. |
| Relevant ADRs | ADR-0013, ADR-0092, ADR-0515, ADR-0538, ADR-0597, ADR-0613/0614, ADR-0633/0634/0635/0636 | ADR-0634 has not reached Accepted status and grants no current auto-approval authority. |

## Status

**Accepted — 2026-08-05.** The founder authorizes the W0-only engine and governance work described here. Acceptance binds the architecture and procedures; it does not assert that an engine, rule pack, front end, gate, registry, receipt, or generated output already exists. W1 and later corpus work remain unapproved until W0 exit evidence is accepted.

## Context

A hand-maintained Kubernetes fork makes upstream maintenance proportional to translated output and invites unreviewable divergence. The maintained product therefore MUST be a reusable transformer and a policy-as-data rule corpus, with Kubernetes as its first corpus. This decision resolves the approved program's engine home, governance mechanics, and W0 procedures without authorizing a ported corpus.

ADR-0538 chose globbed root membership to reduce recurring shared-members-array conflicts. The `build/port-engine/*` home is intentionally a narrowly reviewed exception: it requires one root `Cargo.toml` members-line amendment because no existing glob admits this new engine root. The amendment is justified as a one-time, explicit merge-surface decision under ADR-0538, not by placing the engine in `ci/` merely to avoid that amendment.

## Drivers

1. Upstream changes MUST be absorbed by changing rules and re-rendering, rather than by patching generated Rust.
2. The engine MUST remain reusable across Go corpora and MUST NOT acquire Kubernetes vocabulary or capability ownership.
3. Parallel delivery MUST preserve the single protected merge authority and make review pressure improve rules rather than conceal output defects.
4. Every W0 result MUST have an executable procedure and durable record, not only session knowledge.

## Decision

### D1 — reusable engine, explicit home, and neutral policy split

The program SHALL build `oya-port`, an owned, Kubernetes-agnostic deterministic Go-to-Rust port engine at `build/port-engine/*`. The one root Cargo workspace members-line amendment needed to admit that root is authorized by this ADR as the ADR-0538 exception described above. It MUST be reviewed as a root-membership change and MUST NOT be avoided by homing the engine in `ci/`.

`build/` is a build-meta directory and owns **zero capability crates**. The engine's path and its build-meta ownership MUST NOT create a `build` capability, transfer Kubernetes facts to build metadata, or create a new capability claim. Capability facts remain with their owning capability; `k8s` owns corpus facts and counters, while the engine remains neutral build infrastructure.

The engine/policy boundary is mandatory:

- Engine code is neutral and MAY define SourceModel, RulePack, TransformPlan, TargetIr, Renderer, receipt, delta, and verification seams.
- `specs/port-rules/lang/go-rust/**`, `specs/port-rules/idiom/**`, and `specs/port-rules/canary/**` are neutral rule data.
- `specs/k8s-port/**` is corpus policy, including pin, scope, boundary, detachment, and Kubernetes-specific rules.
- A Kubernetes token or corpus-specific branch in the neutral kernel is a defect and MUST be rejected by the future W0 enforcement; corpus-specific behavior belongs in corpus policy.

### D2 — generated output and mechanical upstream maintenance

All output in a registered regenerable region is generated output. It MUST be emitted by the registered producer, committed only through the ADR-0597 materializer relationship, and MUST NEVER be hand-edited. A red output, compile, correspondence, or determinism result is an engine, rule, policy, model, or declared-detachment defect; it MUST NOT be repaired by editing generated Rust.

Maintenance is a mechanical delta loop: pinned upstream change → canonical semantic SourceModel delta → impact closure → rule or corpus-policy work → regeneration → receipt-bound verification. Textual upstream diff MAY inform discovery but MUST NOT be the authoritative classifier. An unexplained emitted-byte change is RED.

### D3 — review and fan-out model

Rule work MUST use one implementer, two adversarial reviewers with split context windows, and one fixer. Review object is the rule corpus and its fixtures, not generated output. A repeated output failure is evidence of a missing or insufficient rule, fixture, policy datum, or gate.

The approved delivery shape is Bun-style, all-at-once full A-prime scope: all Kubernetes domains, including kubelet and kube-proxy, are in the target program from the start. Control-plane-only operation is not the default scope; it is solely the separately governed G3 fallback.

Parallel fan-out MAY use four isolated worktrees with up to sixteen agents per worktree only when owned-runner capacity evidence demonstrates that the work can run without starving required gates or violating assigned CPU, memory, disk, and IOPS limits. Until that evidence exists, the cap is not authorization to oversubscribe runners. Each lane MUST have non-overlapping ownership and a named integration order.

### D4 — W0 authorization, merge path, and detachment ratchet

Only W0 work is authorized by this ADR. W1+ implementation, corpus expansion, activation, and release claims are expressly outside this authorization.

There is one required sequence: isolated worktree lane → SSH-signed commit and push on that lane → pull request into `dev` → independent review and all review threads resolved → no merge conflict and branch-protection requirements satisfied → the single protected `oya-ci-required` context green → squash merge → post-merge completion packet. There is no long-lived merge-authority branch, no alternate protected status, and no advisory determinism gate. Legacy CLI output is optional local evidence and MUST NOT be merge authority.

Detachment is an identity-set ratchet, not a line-count waiver. Every detached identity MUST record identity, owner, rationale, expiry, and allocation receipt. From the first populated wave gate the identity set MUST NOT grow relative to merge base. Exactly one ADR-authorized C-prime transition receipt MAY add the measured client-lane identities; it is a one-time exception, then shrink-only. The detached magnitude ceiling is not ratified until W0-E measurement.

### D5 — procedures, record lanes, and documentation gate

The program MUST maintain three standing record lanes under `docs/programs/k8s-port/`: an operations journal, reusable prescriptions, and doctrine. The operations journal records each run and every rule change or the reason no rule changed. Repeated operational entry classes MUST be promoted to executable prescriptions at every wave gate; binding judgments MUST graduate to ADRs.

`R-DOC` MUST reject a program document without a baseline version header, a completed wave without a non-empty journal entry, a rule change without a journal reference, ungraduated doctrine older than one wave, and prescription-lane starvation across two consecutive gates. These lanes and the gate are W0 deliverables; this ADR authorizes them but does not claim that they exist yet.

## Alternatives considered

- **Place the engine in `ci/*`.** Rejected. It avoids a members-line change but misstates the engine as CI capability work and evades the explicit ADR-0538 merge-surface decision.
- **Hand-maintain a Kubernetes Rust fork.** Rejected. Upstream absorption would scale with output size and destroy repeatable provenance.
- **Adopt kube-rs as the primary implementation.** Rejected. It introduces another upstream clock and does not provide the owned, mechanically maintained product. The bounded C-prime client fallback remains separately governed.
- **Begin control-plane-only.** Rejected for the target scope. It contradicts the founder's full A-prime ruling; it remains only the G3-bound fallback.
- **Use a merge-authority branch or advisory gates to increase throughput.** Rejected. Both bypass the protected `oya-ci-required` authority and make red conditions negotiable.

## Why chosen

This decision is the smallest architecture that preserves neutral-engine reuse, mechanical upstream maintenance, deterministic generated output, and the singleton governance path at the same time. The explicit root-membership amendment is safer than a semantically false `ci` placement, while the W0-only boundary prevents a decision record from being mistaken for implementation evidence.

## Consequences

- A root Cargo members-line amendment is required before engine crates can be introduced, and its conflict surface is intentional, bounded, and reviewed under ADR-0538.
- `build/port-engine/*` becomes the engine home, but `build/` remains a zero-capability build-meta directory.
- Generated output has no hand-edit escape hatch; rule and gate quality become the primary engineering work.
- Delivery capacity is conditional on owned-runner evidence, not an assumed agent count.
- The identity-set ratchet limits detached creep before any numerical ceiling is ratified.
- W0 must deliver durable procedures and records before W1 can rely on accumulated program knowledge.

## Follow-ups

1. Complete W0-A admission: pin record, scope and divergence registries, licensing lane, and the R-DOC record-lane setup.
2. Complete W0-B engine crates, neutral rule-pack v0, bootstrap front-end artifact governance, and the six-axis receipt schema.
3. Complete W0-C five determinism gates and registered canary regions; each gate must prove zero-scan RED and planted-defect detection.
4. Complete W0-D Talos second-corpus proof and W0-E measured trial, detached ceiling, throughput measurements, and process-fix list.
5. Complete W0-F seam contracts and crate map, W0-G topology and composition-root rulings with branch-protection readback, and W0-H threat-model and benchmark methodology before proposing W1 authorization.

## References

- Approved Kubernetes Go-to-Rust port program, revision 5, reconciled 2026-08-05
- ADR-0538, ADR-0597, ADR-0613, ADR-0614, ADR-0633, ADR-0634, ADR-0635, ADR-0636
- Kubernetes upstream pin evidence: `git ls-remote` and `infra/talos/installation-media/presets.yaml`
