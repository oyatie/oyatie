---
id: ADR-0638
title: "Mechanically maintained Kubernetes Rust port"
status: Superseded
doc_status: published
planning_impact: true
deciders: founder
owner: council-architecture
date: 2026-08-05
door: one-way
supersedes: []
superseded_by: [ADR-0704]
depends_on: [ADR-0013, ADR-0092, ADR-0515, ADR-0597, ADR-0605, ADR-0606, ADR-0608, ADR-0613, ADR-0614, ADR-0615, ADR-0627, ADR-0633, ADR-0634, ADR-0635, ADR-0637]
amends: []
related: [ADR-0243, ADR-0607, ADR-0609, ADR-0619, ADR-0623, ADR-0628, ADR-0629, ADR-0631, ADR-0632, ADR-0636]
related_specs:
  - /specs/k8s-port/upstream-pin.json
  - /specs/k8s-port/scope.json
  - /specs/k8s-port/divergence-ledger.json
  - /specs/k8s-port/licensing.json
milestone: W0-A
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0638: Mechanically maintained Kubernetes Rust port
## Baseline version header


| Authority | Version this ADR is authored against | Status at authoring (2026-08-05) |
|---|---|---|
| Repository baseline | `origin/dev@b64eaaf4a` | Pinned baseline; local `dev` is not pin evidence. |
| Kubernetes upstream | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67`; annotated tag object `5b824a493a7ca248b726b6ea09d53842b9b992c2` | Pinned for this decision. Evidence source: `git ls-remote` plus fleet `infra/talos/installation-media/presets.yaml`. |
| Port output | `k8s/` capability | W0 design decision only; no generated corpus exists or is in force. |
| Engine | `oya-port` at `build/port-engine/*` | W0-B deliverable; unbuilt and not in force. |
| Rules and corpus policy | `specs/port-rules/**` and `specs/k8s-port/**` | W0-A pin, scope, divergence, and licensing policy land with this admission; neutral/corpus rules and later registries remain unauthored and not in force. |
| Source-model front end | Out-of-band bootstrap followed by owned Rust front end | W0-B artifact and measurement work; no front end is in force. |
| Receipt axes | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Binding acceptance invariant; schema and verification implementation remain pending. |
| Relevant ADRs | ADR-0013, ADR-0092, ADR-0515, ADR-0597, ADR-0605/0606/0608, ADR-0613/0614/0615, ADR-0633/0634/0635/0637 | ADR-0634 has not reached Accepted status and cannot provide class-G auto-approval before its stated prerequisites close. |

## Status

**Accepted — 2026-08-05.** The founder authorizes the W0 Kubernetes-port design and its required evidence. This ADR binds future producers and acceptance criteria. It does not claim that source snapshots, registries, rules, output, gates, divergence rows, external artifacts, or conformance results already exist. W1+ execution remains unapproved pending W0 exit evidence.

## Context

Kubernetes is a moving Go corpus whose control plane, clients, generated families, kubelet, kube-proxy, and command surfaces cannot safely become a manually maintained Rust fork. The port must be a deterministic, receipt-bound projection produced by ADR-0637's neutral engine. Its correctness authority must be set-based and auditable even where upstream Go tests or host-specific behavior cannot translate directly.

The canonical destination is `k8s/`, not `os/`: `os/` and managed-Kubernetes facades consume the port through approved seams. A port under `os/` would repeat the boundary defect identified by ADR-0631. ADR-0635's missing `k8s.bootstrap` face and the `os → k8s/ports` direction remain W0-G topology work, not facts asserted by this ADR.

## Drivers

1. Upstream maintenance MUST be regeneration from a complete, inspectable input set, never hand-patch accumulation.
2. Kubernetes-specific policy MUST preserve Oyatie doctrine while making every deliberate divergence explicit, bounded, and reviewable.
3. Determinism gates MUST distinguish a clean scan from an unwired probe.
4. The port MUST prove reusability beyond Kubernetes before W1 authorization.

## Decision

### D1 — destination, scope, and doctrine-first divergence

Generated Kubernetes Rust output SHALL live under `k8s/`. `os/`, `cloud/cloud-k8s`, and managed-Kubernetes facades MUST consume it only through approved `k8s/ports/**` seams; they MUST NOT become alternate homes for generated upstream code.

The program adopts full A-prime scope from the start: apimachinery, API types, client machinery, component base, apiserver, scheduler, controller manager, kubelet, kube-proxy, kubectl, and applicable generated families are PORT scope subject to the future scope registry. Kubelet and kube-proxy are not deferred from the target; a control-plane-only outcome is only the separately governed G3 fallback. `etcd`, containerd/CRI runtimes, CNI binaries, CSI drivers, and CoreDNS are consumed dependencies rather than port output; exclusions require a registry row and rationale.

Divergence policy is doctrine-first. The Cedar authorization seam, owned audit-chain emission, and owned observability emission are retained where they conflict with bit-compatible upstream behavior. Each divergence MUST have a ledger row with rationale, ADR citation, owner, expiry, review cadence, conformance impact, and enumerated expected-red test identifiers. A divergence MUST NOT produce a skipped test: an unenumerated red is a defect. The ledger begins with the Cedar, audit, observability, removed in-tree provider/volume-plugin, and bootstrap-front-end rows; the C-prime client row is added only if its separately authorized transition occurs. Ledger growth is limited to two new rows per wave.

### D2 — six-axis reproducibility acceptance invariant

The following predicate is an explicit acceptance invariant, not illustrative prose:

```text
verify(pin,
       snapshot_digest,
       engine_digest,
       rulepack_digest,
       toolchain_digest,
       formatter_digest,
       tree) -> { byte-identical | diff }
```

Every regenerable byte and its receipt MUST be derivable from all six named axes. `snapshot_digest` is REQUIRED: moving the semantic input without binding it to the receipt is a soundness failure. A receipt mismatch on any axis, an unregistered regenerable path, an unexplained byte change, or a failed clean-room comparison is RED. ADR-0638 could not be Accepted with a shorter tuple; its accepted state binds this condition while implementation remains W0 pending.

### D3 — source-model snapshot firewall and front-end replacement

During W0–W1, `go/packages` plus `go/types` SHALL run only out of band as the bootstrap extractor. It produces a canonical, content-addressed SourceModel snapshot of the pinned upstream corpus. `oya-port` consumes that snapshot and MUST NEVER invoke a Go toolchain in its producer or `verify()` path.

The canonical snapshot producer identities are `bootstrap-go-packages-go-types` and, after it is introduced, `owned-rust-go-front-end`. A composed snapshot MUST record exactly one of those identities for every package inside the snapshot; each package MUST have exactly one producer. The composed artifact, its package-to-producer mapping, and its schema are covered by `snapshot_digest`.

The owned Rust front end MAY begin only after W2 authorization. It has no authority merely because it can parse a subset: replacement occurs only when the model-equivalence gate demonstrates byte-identical SourceModel output for the full pinned corpus. Each increment changes the recorded producer identity and `snapshot_digest`, then MUST fully regenerate and re-verify all affected output. The bootstrap extractor is a ledger divergence with expiry and MUST be retired only after one complete full-corpus absorption cycle is green.

### D4 — determinism gates and canary population rules

W0-C MUST implement five determinism gates: regenerate-twice, boundary partition, detached control, manual-edit refusal, and rule-mutation canary. Each gate MUST declare independent scanned-population and finding counters, include at least one registered always-present canary region carrying a planted defect, and detect that defect.

Zero scanned population is RED unconditionally. A canary makes scanned population structurally nonzero whenever a gate is wired; therefore a true zero proves an unwired or broken probe. Zero findings with nonzero scanned population is GREEN. The gates MUST refuse, respectively, nondeterministic bytes or receipt mismatch; unclassified or overlapping regions; unauthorized or growing detached identities; accepted manual edits in regenerable regions; and rules whose semantics-changing mutants do not alter selected-fixture behavior. No gate MAY use an empty-corpus or existing-output qualifier.

**CRIT-P5-01 — canary denominator exclusion.** Canary regions remain included only in gate-liveness and scanned-population counters. They MUST be excluded from the upstream-to-Rust correspondence bijection, `R_detached`, coverage, catalog and born-accounting, and performance denominators. A canary cannot represent upstream source coverage, a ported catalog unit, detached production surface, or measured product performance.

### D5 — corpus proof, external artifacts, and upstream maintenance

W0 MUST NOT exit until the neutral engine and neutral rules port a bounded Talos second corpus and pass its landed `os/harness/difftest-app` vectors. Before W1 exits, they also MUST port a third unrelated Go corpus, such as a CNI plugin, under the same neutral rules. Kubernetes-specific rules MUST NOT be introduced to make either proof pass.

External artifacts, including the bootstrap extractor, Kubernetes source, Ginkgo or any conformance test artifact, and rule seeds, MUST have a pinned source/version/digest, SBOM, signature and provenance verification, sandbox policy, recorded license status, and owner. Kubernetes Apache-2.0 attribution, MIT rule-seed attribution, and per-file generated provenance MUST be carried through the licensing lane. An absent or failing external-artifact control is RED, not a reason to relax hermeticity.

Upstream maintenance is a semantic delta loop between pinned SourceModels: classify add, delete, move, split, signature, body, type-fact, build-constraint, and test-population changes; compute impact closure; update rules or corpus policy; regenerate; and verify the six-axis receipt. Every output change MUST be explained by source, rule/policy, pin, toolchain/formatter, or snapshot/extractor delta. Anything else is RED.

### D6 — deferred W0-G direction ruling and governance caveat

The Q7 `os/` shared-types direction is deliberately deferred to W0-G measured dependency evidence. The only candidates are moving irreducible config/version types below both consumers or making `k8s` consume an OS-owned HostOps seam. Neither remedy is chosen, and no runtime edge is authorized, before W0-G topology ratification.

ADR-0634 has not reached Accepted status. Consequently regenerated output MUST NOT receive class-G auto-approval until its D6–D8 prerequisites are closed and W0-G records live branch-protection readback. Until then, all changes use the ordinary reviewed PR path and the single protected `oya-ci-required` context.

## Alternatives considered

- **Host the port under `os/`.** Rejected: it violates the `k8s` capability boundary and strands managed-Kubernetes facades.
- **Bit-compatible upstream divergence policy.** Rejected: it abandons the Cedar, audit, and observability doctrine; doctrine-first differences are safer when explicitly ledgered and tested.
- **Run Go tooling inside `verify()`.** Rejected: a foreign toolchain in the producer predicate breaks the snapshot firewall and hermetic class-G basis.
- **Build an owned Rust `go/types` equivalent before W0.** Rejected: it serializes the program behind an unsized compiler front end before rule and corpus evidence exist.
- **Treat canaries as normal ported production surface.** Rejected: it inflates correspondence, coverage, detachment, catalog, born-accounting, and performance evidence.
- **Limit the program to the control plane.** Rejected as the selected scope; it is retained only as the G3 fallback.

## Why chosen

A snapshot-bound deterministic projection permits an owned Rust engine to use complete Go semantics without contaminating its verification path with a Go toolchain. Doctrine-first divergences preserve established security and observability obligations while refusing invisible conformance exceptions. Canary-backed liveness ensures a green gate always means that it scanned something meaningful, without allowing test canaries to falsify product denominators.

## Consequences

- `k8s/` becomes the only generated-port home; consumers must honor `k8s/ports/**` seams.
- Full Kubernetes scope includes node-plane components, increasing W0 design and W1+ delivery obligations.
- Every source-model, toolchain, formatter, rule, or pin movement changes a receipt axis and triggers regeneration evidence.
- The bootstrap Go extractor is permitted only outside the producer path and carries a tracked expiry, supply-chain controls, and a future full-corpus replacement gate.
- Talos and a third unrelated Go corpus are non-negotiable W1-exit evidence, not optional demonstrations.
- Canary regions provide liveness evidence only and cannot improve production coverage, accounting, detachment, or performance ratios.
- W0-G must resolve topology facts and Q7 before activation-oriented runtime claims; ADR-0634 does not yet permit auto-approval.

## Follow-ups

1. In W0-A, create the pin, scope, divergence, and ownership records with the exact baseline above and seed ledger rows.
2. In W0-B, produce and govern the bootstrap snapshot artifact, define the six-axis receipt schema, size the front end, and create neutral rule-pack fixtures.
3. In W0-C, wire all five gates, canary registry entries, manual-edit refusal, and zero-scan RED demonstrations.
4. In W0-D, complete the Talos proof without Kubernetes-specific neutral rules; before W1 exits, complete the third-corpus proof under the same neutrality constraint.
5. In W0-E, ratify the measured detached ceiling, derived maintenance SLA, performance baseline, and process-fix list; founder ratifies activation-blocking budgets from owned-fleet evidence.
6. In W0-G, ratify `k8s.bootstrap` topology, decide Q7 from measured evidence, rule the composition root, and record live branch-protection readback before requesting class-G authority.

## References

- Approved Kubernetes Go-to-Rust port program, revision 5, reconciled 2026-08-05
- ADR-0637; ADR-0597; ADR-0605, ADR-0606, ADR-0608; ADR-0613, ADR-0614, ADR-0615; ADR-0633, ADR-0634, ADR-0635
- Kubernetes upstream pin evidence: `git ls-remote` and `infra/talos/installation-media/presets.yaml`
