---
doc_class: Program-Discovery-Decision-Record
doc_status: drafted
discovery_id: DDR-E0-20260810-ecosystem-coexistence-contract
judgment_class: ecosystem-coexistence-contract
recorded_at: 2026-08-10
owner: council-architecture
authority_tier: 3
---
# DDR-E0-20260810-ecosystem-coexistence-contract

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-10) |
|---|---|---|
| Repository baseline | `origin/dev` @ `9a56538c74b1fce4d474869956dd278f7fe1981e` | E0 discovery encode lane base. |
| Upstream Kubernetes pin | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Pinned program input; not consumed here. |
| Engine | `build/port-engine/*`, v0 | Not in force as a producer for this record. |
| Neutral rule pack | `specs/port-rules/**`, v0 | Not in force. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Not in force. |
| Go front end | Bootstrap extractor; strategy ruled | Not in force. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six required axes; not in force. This record emits no receipt. |
| Program authority | ADR-0704 (live apex) | Discovery record only. |

## Record identity

- **Stable ID:** `DDR-E0-20260810-ecosystem-coexistence-contract`.
- **Judgment class:** ecosystem coexistence — Go/any clients via API/wire; port-engine not required for operators.
- **Status:** `discovery` — record, **not** doctrine, **not** Accepted apex.
- **Recorded:** 2026-08-10.
- **Owner role:** `council-architecture`.

## Authority fence

This contract does not force PORT of third-party operators, CNI/CSI plugins, or Helm charts.
It does not flip scope rows. It does not require a dual forever Go+Rust control plane.

## Judgment

### Compatibility contract — what “support” means

Ecosystem success is **wire + API + behavior** compatibility, not “everything rewritten in Rust”
and not “everything passed through port-engine.”

| Surface | Must stay compatible so Go (and any language) clients keep working |
|---|---|
| kube-apiserver HTTP/JSON + Protobuf envelope | client-go, controller-runtime, kubectl, operators |
| Authn/authz hooks / TokenReview / SubjectAccessReview | existing IdP/webhook patterns; Cedar PEP is a **server-side** divergence with enumerated conformance IDs (future) |
| Aggregate APIs / CRDs / conversion / validation webhooks | language of webhook binary irrelevant |
| kubelet ↔ Device Plugin API | device plugins (not CRI); kubelet-local gRPC registration surface |
| External CRI Unix-socket face (versioned compatibility profile) | listed external consumers only (for example crictl, node-problem-detector); unlisted = REFUSE |
| CNI / CSI / cloud-provider interfaces | **CONSUME** plugin binaries (usually Go); we do not PORT every plugin |
| Helm / charts / YAML | unchanged — talk to the API |

Sonobuoy + CNCF conformance are the primary proof that ecosystem-facing behavior holds
(gate-class split: PR smoke subset vs promotion full suite — encode of gate classes is later;
this record only states the contract). Operator soak of common Go controllers is a secondary
evidence lane, not a rewrite mandate.

### What port-engine is for (and is not)

| In scope of port-engine PORT | Out of scope (stay upstream language) |
|---|---|
| First-party Kubernetes A-prime we ship (apiserver, kubelet, …) | Third-party operators, CRDs, Helm charts |
| Shared substrates (SMP, protobuf envelope) used by PORT output | Customer workloads inside pods (any language) |
| First-party owned runtime libraries are **not** a port-engine corpus | CNI/CSI plugin binaries (CONSUME unless a named first-party plugin is PORTed later) |

**Rule:** port-engine is an **implementation factory for components we own**. Ecosystem apps are
**clients**. Preferring Go for an app is fine; operators do **not** need Rust or port-engine.

Round-2 clarification: there is **no** containerd product PORT. The CRI external face and owned
runtime libraries are first-party node-supervisor work, not a second port-engine corpus. Dated
Go containerd CONSUME is bootstrap only.

### First-party language posture (fail-closed, not silent dual-stack)

| Posture | When | How |
|---|---|---|
| PORT (default for kubelet, apiserver) | TCB / surface / mechanical bump matter | port-engine + conformance |
| CONSUME Go binary (time-boxed only) | Bootstrap, canary, or not-yet-in-PORT | scope row + pin + digest + **calendar-dated fail-closed expiry** + named owned-Rust destination; same APIs |
| Customer/partner Go | Always | Unsupported to rewrite; supported as API clients |

**Ban:** first-party forever-Go product code (including a permanent “CONSUME Go forever”
exception row). Any first-party Go binary is a temporary adapter with an owned-Rust destination
and dated expiry — never a silent dual-stack forever posture. **Ban** applies a fortiori to
kubelet/apiserver product path.

**Do not require** a Go apiserver alongside Rust for ecosystem apps if wire parity holds. Optional
Go control-plane canary (CONSUME upstream binaries in a shadow cell) is a *validation* tactic,
not the product forever shape.

### Authz note for ecosystem

Cedar-as-apiserver-authorizer is a doctrine-first divergence: Go RBAC clients still call the same
APIs; some RBAC conformance cases may be expected-red with **ratified IDs** (not invented here).
Operators that only need ordinary create/update verbs keep working; exotic RBAC edge cases must
be listed, not discovered in production.

## Round-2 basis

Locks ecosystem coexistence: apps stay language-agnostic clients of K8s/CRI/CNI APIs; PORT only
first-party node/CP we own; optional Go CONSUME only with explicit scope rows (scope edits are
**not** part of E0).

## Alternatives

| Approach | Why not default |
|---|---|
| Rewrite popular operators into Rust | Scope explosion; customers will not |
| Mandate port-engine for all cluster addons | Wrong tool; addons are not our corpus |
| Keep Go kubelet forever “for ecosystem” | Ecosystem talks to **API/CRI**, not kubelet’s implementation language |
| Dual forever Go+Rust control planes | Dual-truth ops + 2× CVE surface |

## Downstream blockers

- Scope rows for any first-party Go CONSUME (including dated containerd bootstrap) — F1(b).
- Conformance artifact-class admission (Sonobuoy/CNCF) — K2.
- Enumerated expected-red IDs for Cedar divergence — W0 ledger work.

## Naming law

Uses Round-2 neutral nouns (**node supervisor**, **owned runtime**, CRI face). Does not adopt
`asterkube` or `kuberos` as product/public names. Prior-art forever Go userspace shapes are
referenced only as **not adopted**.
