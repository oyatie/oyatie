# CI gate fleet audit — vacuous / stale / unwired / currency (2026-07-28)

15 agents · 1,345,359 subagent tokens · 498 tool calls · 51 gates + 111 libs kernels examined.
**30 findings → 11 materially verified → 6 CONFIRMED, 5 REFUTED.**

## CONFIRMED

### 1. [CRITICAL] 105 of 111 libs/ check kernels are unreachable from merge authority
`libs/oya-check-*` (72) + `libs/oya-governance-*` (39). Only 6 reach a `ci/facade/`
consumer. **This is worse than the 68-of-72 figure I reported earlier** — the
`oya-governance-*` family is in the same state and I had not counted it.

### 2. [CRITICAL] The gate that polices gate wiring cannot see the kernels
`ci/facade/automation-coverage` (GATE-4) + its producer `artifact-inventory-registry`
**cannot see any `oya-check-*` kernel at all.** So the one detector that would have
surfaced finding #1 is structurally blind to it. That is the reason this rotted
unnoticed rather than being caught on the first disconnected kernel.

### 3. [HIGH] `topology-manifest-contract` is structurally vacuous
`src/lib.rs` is **9 lines** and contains exactly one item (`pub const GATE_ID`). All
"enforcement" is 3 tests reading three static governance documents: its own spec, one
fixture authored to satisfy that spec, and the root-hub pointer. **Zero service
manifests are ever opened.**

The verifier made it worse than claimed: **0 of 90** `{oya,cloud}/*/manifest.json`
carry a `cell_topology` key — the finding's two examples were substring false
positives (`cell_topology_tier`, and file-path strings). So the spec's declared live
surface has **zero instances repo-wide**, and the proposed remedy ("scan manifests
declaring cell_topology") would itself be vacuous.

It also byte-asserts a validator string naming `cloud/cloud-ci/gates/oya-cloud-ci-cell-topology-manifest-contract-app` — **a directory that no longer exists.**

Mitigating: `gate-disposition.json` already records it as `stub-pending-hardening`,
`prod: false`. ADR-0009 (status: *proposed*) sanctions "spec-only" scope.

### 4. [MEDIUM] The only vulnerability gate runs on a frozen snapshot
`ci/facade/supply-chain-audit` matches `Cargo.lock` against a RustSec advisory mirror
frozen **2026-06-26** — a month stale. Every CVE published since is invisible.

### 5. [MEDIUM] SLSA-L3 evidence grounded in a deleted Jenkins pipeline
`libs/oya-check-slsa-l3-evidence-grounded` — `lib.rs:242-255` defines
`canonical_citations`, two of which point at
`infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy`. Jenkins is retired; the file
is gone. The supply-chain evidence chain cites a deleted artifact.

### 6. [MEDIUM] `dependency-automation` DATA contract names retired tooling
`oya-deps.toml` names `cargo-deny`/`cargo-vet` and a drift guard at a path that has
moved. Scale under-counted in the original finding, severity overstated.

## REFUTED — do NOT act on these

| Claim | Why it failed |
|---|---|
| 3 gates stale because **ADR-0532 retired the `oya-` prefix** | **ADR-0532 does NOT retire the `oya-` grammar.** Its ratified rename set says otherwise. Acting on this would have removed three *working* gates (`crate-name-prefix`, `crate-layer-suffix`, `package-manifest-hygiene`) on a misread ADR. |
| `automation-coverage` self-test structurally cannot fail | The remedy proposed already exists for that gate over that corpus |
| `feature-maturity-policy` misses 25 of 31 PRDs | The 25 `{oya,cloud}/<svc>/PRD.md` are a **different artifact class on a different template**; the scan root is correct |
| `gate-self-conformance` bounded by a path literal | Facts right, verdict wrong — property misclassified |
| `governance_crate_substr` finding | Counts re-derived differently |

## The two systemic defects worth productizing

**A. No root-liveness detector, fleet-wide.** The fleet has an anti-weakening ratchet
(you cannot *remove* a scan root) but **no detector for a declared root that no longer
resolves**. So every reorg move silently converts a live scan root into a skipped one,
and the ratchet actively prevents cleaning it up. Confirmed instances:
- `automation-language-policy`: **4 of 48** declared roots do not exist
- `layer-dependency-acyclicity`: 1 of 30 `crate_root_globs` matches zero paths
- `oya-ci.toml [enforcement].governance_lanes`: 1 of 2 lanes is a phantom file the
  producer silently skips
- `registry/hyperscaler-scorecards`: **32-for-32 phantom** — every per-service
  override path sits under a removed root

This directly threatens the ~250 remaining crate moves: each one can silently blind a
gate, and nothing reports it.

**B. `gate-self-conformance` has no vacuity detector.** The meta-gate enforcing the
"7-property bar over every gate" cannot detect a gate that cannot fail. That is
precisely why #3 survives in the required matrix, burning a runner leg while
asserting nothing about the tree.

## Recommended order

1. **Root-liveness detector** — every declared scan root/glob must resolve, or be
   explicitly marked retired-with-reason. Highest leverage: it is the class fix for
   four confirmed instances *and* the guard for the remaining reorg.
2. **Vacuity detector in `gate-self-conformance`** — require every gate to prove a
   non-empty live corpus and ship a RED fixture. Would have caught #3 at authoring.
3. **Point GATE-4 at `libs/`** — the wiring police must see the kernels it polices,
   else #1 recurs silently.
4. **Refresh the RustSec mirror + wire freshness** — a month-stale vulnerability
   snapshot is the highest real-risk item here.
5. `topology-manifest-contract` — build it or cut it; `gate-disposition.json` already
   says "build-or-cut per the move plan".

## Process note

45% of materially-verified findings were refuted. The single most expensive avoided
error: acting on a **misread of ADR-0532** would have deleted three functioning gates.
Consistent with the rest of this session — the adversarial pass is where the errors
surface, not where they are rubber-stamped.
