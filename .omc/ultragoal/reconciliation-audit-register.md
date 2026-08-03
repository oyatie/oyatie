## 1. Executive summary

- **31 directories audited** across the monorepo (26 code-bearing + 5 non-code/data/ephemeral roots); scatter dirs (cloud/, oya/, data/, infra/, scripts/, tools/) carry the bulk of the debt.
- **6 confirmed cross-directory duplicates** survived verification (billing, IAM, IaC-module-registry, cell-lifecycle-runbooks, feature-flags, cloud-os k8s-control) plus 3 confirmed same-dir dupes (libs/ outbox 3-way, tenancy dual Tenant kernel, oya/ billing-cost-meter stubs); **4 headline duplicate claims were VERIFIED FALSE** (cloud-intelligence already relocated by PR #767, messaging-vs-workflow event-bus, infra-vs-capabilities, secrets-vs-cloud-os) — do NOT action those.
- **~7 confirmed cross-dir name collisions**: gateway/network, comms/messaging, contracts/specs, plus the audit/compliance/governance triad which verification RULED must stay separate (ADR-0615) — rename gateway/ and messaging/, keep the triad.
- **~40+ dead/empty dirs**: 19 spec-only cloud/ subdirs, ~60 spec-only oya/ subdirs, 6 empty oya/oya-* stubs (reap-now), plus scattered dead port-kernels (audit query/retention, billing-kernel [FALSE-dead — intentional], observability-kernel [truly dead]).
- **Visibility posture is uniformly broken**: every code dir sampled uses `visibility = ["PUBLIC"]` on every target — the ports/adapters boundary is drawn in directory layout everywhere and enforced by buck2 nowhere. Third-party/ (reindeer-generated) is the sole exception.
- **Highest-leverage finding**: the single missing invariant family — an anti-dupe + fan-in-ratchet + visibility-tightness + cross-cap-port + hermeticity gate — is what lets every duplicate, PUBLIC-everywhere leak, and ambient `Command::new("git"/"cargo"/"buck2")` shellout above regenerate. `oya-data-boundary-kernel` (fan-in 128) and PUBLIC-everywhere are the two structural roots; ship the gate family in §5.

## 2. Confirmed conflict register (ranked by severity)

| # | type | concern | locations | recommended resolution | severity |
|---|------|---------|-----------|------------------------|----------|
| 1 | cross-dir-dupe | IAM runtime crates split from their own contracts/IaC; 4-way identity name cluster | `iam/` (65 crates, iam-cloud-* prefix), `cloud/cloud-iam/` (doc-only, holds the proto/OpenAPI/helm), `oya/identity/` (docs), `oya/oya-identity/` (dead stub) | Founder-gated migration IP: move iam-cloud-* slice → `cloud/cloud-iam/`; reap `oya/oya-identity/`; fold `oya/identity/` docs into survivor | high |
| 2 | cross-dir-dupe | billing/accounting/tax: working code in one dir, doc-only shells in two | `billing/` (17 crates), `cloud/cloud-billing/` (0 crates), `cloud/cloud-billing-tax/` (0 crates) | `billing/` wins; drain cloud/cloud-billing{,-tax} docs into it; implement stranded tax port under billing/core+adapters | high |
| 3 | cross-dir-dupe | Two authoritative OpenTofu module registries, one pointing at a dead `microservices/` path | `iac/modules/catalog.json` (ADR-0339, live), `cloud/cloud-iac/tofu/modules/catalog.json` (stale) | `iac/modules/` wins; drain the 6 cloud/cloud-iac modules in, delete stale catalog | high |
| 4 | same-dir-dupe | 3-way outbox abstraction, explicitly transitional but unconsolidated | `libs/oya-shared-outbox-pattern-kernel`, `libs/oya-shared-transactional-outbox-*` (7 crates), `libs/oya-data-outbox-*` | Consolidate under ADR-0536/0537 W5 owned-store direction the code already points to; mark transitional crates with retirement anchor | high |
| 5 | same-dir-dupe | cloud-os: two crates independently implement static-pod + control-plane bootstrap, no dep edge | `cloud/cloud-os/crates/oya-cloud-os-k8s-control-domain`, `.../oya-cloud-os-kubernetes-domain` | Dedupe the overlapping static-pod/bootstrap/secrets scope; establish one owner + port seam | high |
| 6 | cross-dir-dupe | cell-lifecycle: runtime in cell/, two cloud/ doc dirs with verbatim-templated runbooks | `cell/` (8 crates), `cloud/cell-lifecycle/` (0 crates), `cloud/cell-rebalancer/` (0 crates) | cell/ owns runtime; dedupe the two identical runbook sets; cross-link docs to cell/ | medium |
| 7 | cross-dir-dupe | feature-flags spread across real dir + two empty shells with 3 naming schemes | `flags/` (2 crates), `oya/feature-flags/` (0 crates), `oya/oya-flags/` (0 crates) | flags/ wins; reap oya/oya-flags/; fold oya/feature-flags/ specs into flags/ | medium |
| 8 | same-dir-dupe | tenancy: two Tenant kernel generations coexist, no retirement marker | `tenancy/core/kernel` (id: TenantId), `tenancy/core/domain` (id: String) — both live | Confirm which is superseded; drain the ADR-0002 pair if the lifecycle-* stack replaces it | medium |
| 9 | same-dir-dupe | oya/: billing/cost/meter concern split across 3 empty stubs while real impl orphaned outside workspace | `oya/oya-billing`, `oya/oya-cost`, `oya/oya-meter`, `oya/accounting` (all 0 crates); real code at `billing/facade/*` not in workspace globs | Reap the empty oya/ stubs; wire billing/ into workspace per ADR-0562 | medium |
| 10 | placement | marketplace-dev-cli is the repo-wide CI gate-runner (~90 oya-check-* deps, bin `oya`), zero marketplace deps | `marketplace/facade/dev-cli` | Move → `ci/facade/`; keep ADR-0562 SKU framing as doc-note | medium |
| 11 | placement | network-residency is a platform-wide ADR-0049 kernel (32 cross-cap consumers) shelved as a network/ leaf | `network/core/residency` | Extract → `libs/` (shared substrate); shrinks network/ blast radius for ~30 caps | medium |
| 12 | dead-code | observability-kernel: zero consumers repo-wide, duplicates cardinality/budget modeling already live in observability-domain | `observability/core/kernel` | Reap (truly dead, verified) | medium |
| 13 | placement | data-cloud-domain is a cell+compute+secrets+network aggregation mis-shelved next to analytics/ontology | `data/core/cloud-domain` (+ empty-dep `data/core/cloud-kernel`) | Move → `cloud/` (cloud-data capability) or reap | medium |
| 14 | placement | 4 libs/ crates path-depend into a single owning capability (not neutral kernels) | `oya-bus-boundary-kernel`, `oya-queue-boundary-kernel` → messaging/; `oya-shared-backbone-grpc-generated-adapter` → comms/; `oya-http-tenant-middleware-infrastructure` → tenancy/ | Move each to its owning capability | low |
| 15 | placement | billing-saas-bench has 100% non-billing deps (workflow/marketplace SaaS harness) | `billing/facade/saas-bench` | Move → workflow/ or marketplace/ | low |
| 16 | placement | friction-ledger merge-driver's runtime+test deps are all ci/facade-owned | `tools/oya-friction-ledger-merge-driver-app` | Move → `ci/facade/` | low |
| 17 | dead-code | two port-kernels with zero dependents anywhere | `audit/ports/query-kernel`, `audit/ports/retention-cascade-kernel` | Reap | low |
| 18 | dead-code | 6 empty placeholder stubs duplicating real sibling names | `oya/oya-identity`, `oya/oya-flags`, `oya/oya-billing`, `oya/oya-cost`, `oya/oya-meter`, `oya/oya-authn-device-firmware` | Reap (0 crates, commented-out BUCK) | low |

## 3. Per-directory ruling sheet

| dir | kind | crates | ruling | one-line why |
|-----|------|--------|--------|--------------|
| audit/ | capability | 17 | keep + reap 2 + tighten visibility | clean charter; reap dead query/retention port-kernels; PUBLIC lets iam/marketplace reach past ports |
| benchmarks/ | dead-empty | 0 | merge-into-docs | one .md SLO-baseline record, already strangled to a single file |
| billing/ | capability | 17 | keep + move saas-bench + wire billing-kernel | real home for domain; drain doc-only cloud/cloud-billing{,-tax} in |
| cell/ | capability | 8 | keep (rename siblings) | small hermetic capability; the collision is cloud/cell-* doc dirs, not cell/ |
| ci/ | cross-cutting-infra | 49 | keep-as-is | legitimate flat single-concern gate fleet; naming nit only (cloud-ci brand at top-level ci/) |
| cloud/ | scatter | 62 | keep flat shape; resolve collisions before code lands | 19/21 subdirs spec-only; dedupe cloud-os k8s-control; tighten visibility before fan-out |
| comms/ | capability | 22 | keep-as-is | clean per-sub-product layering; only rename the messaging/ sibling |
| compliance/ | capability | 7 | keep-as-is | clean single-concern; ADR-0615 confirms stays separate from audit/governance |
| compute/ | capability | 7 | keep-as-is | clean core/adapters/facade; compute-resource is a legit cross-cap hub |
| console/ | capability | 9 | keep-as-is | leaf, 0 external fan-in, correct dep direction |
| contracts/ | cross-cutting-infra | 0 | keep as global registry; rename per-cap contracts/ subdir | ~90 capability-local contracts/ mirrors collide with the root registry name |
| data/ | scatter | 22 | split | drain dead search-* cluster + move cloud-domain/kernel to cloud/; leaves analytics+ontology |
| docs/ | cross-cutting-infra | 0 | keep + reap stray artifacts | delete committed .omc/state runtime files; merge docs/audits→audit; fold docs/ci→docs/oya-ci |
| evidence/ | ephemeral | 0 | keep (append-only sink) + reap 1-file subdirs | subdir sprawl; outside buck2 graph so no blast-radius cost |
| flags/ | capability | 2 | keep-as-is | clean 2-crate capability; reap the two oya/ shells |
| gateway/ | capability | 10 | rename → connectors/ | holds zero edge code, only SaaS connectors; name over-promises + collides with network/ |
| governance/ | cross-cutting-infra | 5 | keep-as-is | early ADR-0580 substrate, ADR-registry-mapped future; reconcile dual identity-kernel |
| iac/ | capability | 5 | keep + drain stale cloud/cloud-iac catalog | textbook clean-arch; the defect is the second stale module registry |
| iam/ | capability | 65 | keep core; founder-gated move of iam-cloud-* → cloud/cloud-iam/ | collapses 4-way identity name collision into one discoverable home |
| infra/ | scatter | 0 | drain(strangler) — CONTESTED | verification says infra=legit ops-substrate layer READ by ci/ gates; NOT a duplicate — needs-human |
| intelligence/ | capability | 17 | keep-as-is (relocation ALREADY DONE) | PR #767 already relocated; only stale cloud/cloud-intelligence README/manifest needs cleanup |
| k8s/ | capability | 18 | keep-as-is | textbook 4-slice capability; only nit is one facade→kernel port-bypass |
| kernel/ | capability | 3 | keep-as-is | ADR-sanctioned nested no_std workspace; core/harness IS the rebuild firewall |
| libs/ | cross-cutting-infra | 189 | split | 3 cohesive families (check/governance/shared) accidentally colocated; move capability-coupled crates out |
| marketplace/ | capability | 5 | keep core; split dev-cli → ci/ | dev-cli is the global gate-runner, defeats crate-as-firewall inside marketplace |
| messaging/ | capability | 3 | keep + rename → eventbus/ | clean eventing substrate; one letter from comms/messenger, real misfiling risk |
| network/ | capability | 8 | keep + move residency out | residency is platform-wide (32 consumers), not a network leaf |
| observability/ | capability | 5 | keep chain + reap kernel | observability-kernel is truly dead (verified); fix stale CLAUDE.md pointer |
| oya/ | scatter | 246 | reap 6 empty stubs; per-cap audit for rest | ~60 spec-only subdirs; empty oya-* stubs duplicate real sibling names |
| packs/ | cross-cutting-infra | 0 | keep (rename docs pointer) | legit content substrate; fix docs/AGENTS.md regional-packs/ forward-ref |
| plan/ | ephemeral | 0 | merge-into evidence/ or docs/plans | 5 loose .md + 1 JSON, no crates, collides with tasks/ |
| registry/ | cross-cutting-infra | 0 | keep + close ambient-fs-read gap | 31 crates read registry/** via string-literal paths, not buck2-declared inputs |
| scripts/ | scatter | 0 | drain(strangler) | legacy shell/Python CI glue; ADR-0515/0523 already name Rust successors |
| secrets/ | capability | 10 | keep-as-is | clean capability; cloud-os secrets-domain is OS-PKI, NOT a dup (verified) |
| specs/ | cross-cutting-infra | 0 | keep + reap products/ tombstone + de-Python fixture | canonical registry; drain calendar_prd_replay_check.py |
| storage/ | capability | 8 | keep + wire object-store-kernel | textbook shape; unconsumed "MUST call" port is a dead-code risk |
| tasks/ | ephemeral | 0 | merge-into plan/ | 0 crates, name-collides with plan/, plan.md/todo.md already retired stubs |
| templates/ | cross-cutting-infra | 0 | merge-into-docs (lift + reap) | frontmatter-tagged pending-lift dups of docs/templates + docs/checklists |
| tenancy/ | capability | 21 | keep + reconcile dual Tenant kernel | two Tenant generations coexist with no retirement marker (confirmed) |
| third-party/ | cross-cutting-infra | 0 | keep-as-is | correct reindeer-generated vendored-dep layer; fan-in is inherent, not a defect |
| toolchains/ | cross-cutting-infra | 0 | keep-as-is | canonical buck2 toolchain/platform home; minimal + referenced |
| tools/ | scatter | 29 | split | keep governance-*-app wrappers; move friction-ledger→ci/; reap Python scatter |
| workflow/ | capability | 48 | keep core; split facade tier | engine tiers clean; 4 unrelated product facades bundled under one facade/ label |

## 4. Needs-human (unverified / judgment calls)

- **infra/ drain (CONTESTED)**: The dir-audit ruled `drain(strangler)`, but verification returned **false-positive** — infra/ is a legitimate two-layer ops-substrate (YAML/Terraform/manifests) that ci/ gate crates *read and verify* (e.g. `ci/facade/build-cache-policy` reads `infra/ci/buckconfig/`, `ci/facade/supply-chain-audit` verifies `infra/kyverno/`), anchored by ADR-0117/0120. Draining it would embed non-hermetic config as string literals inside Rust crates. **Founder call**: keep infra/ as the ops-substrate layer vs. drain into per-capability adapter-config dirs. The non-hermetic *shell scripts* (network-fetching install-buck2.sh, talos curl-to-factory.dev) should be de-shelled regardless.
- **contracts/ vs specs/ + ~94 per-capability contracts/ renames**: verification verdict = **needs-human**. Navigational confusion is real (463 path-grep hits across 4 semantic contexts) but the dirs serve distinct purposes (catalog vs source vs policy vs tasks). Whether that justifies ~94 directory renames + thousands of reference updates is a judgment call, not a correctness fix.
- **data/core search-* + cloud-* clusters**: verification = **needs-human**. Structurally confirmed dead (zero external refs, no facade/ports wiring, cloud-kernel has empty deps) but they're documented under the data capability per MOVE-16/ADR-0562 as intentional. Reap-vs-keep-as-planned-scaffolding needs ADR-0562 intent review. (Note: 8 search crates, not 7.)
- **tenancy dual Tenant kernel**: confirmed both live with no retirement marker — but which of (ADR-0002 kernel/domain) vs (G001 lifecycle-* stack) is authoritative is not decidable from the tree. Needs owner ruling before draining either.
- **iam-cloud-* and intelligence relocations**: both are founder-gated migration IPs per reorg doctrine, not silent moves. iam-cloud-* → cloud/cloud-iam/ requires the migration IP. intelligence/ is already relocated (PR #767) — but choosing which kernel/rest is authoritative vs. any residue still needs confirmation.
- **libs/ 3-way split (check/governance/shared)**: high-value but 189 crates; the split must ship *with* tightened visibility or it's cosmetic. Sequencing + blast-radius (oya-data-boundary-kernel touches 128 dependents) needs a founder-approved migration plan.

## 5. Enforcement gaps → proposed invariant family (the productized pipeline)

Every finding above is a *symptom* of one absent capability: nothing mechanically stops a duplicate, a PUBLIC-everywhere leak, a new fan-in hub, a cross-capability reach-past-the-port, or an ambient `Command::new`. Ship **one invariant family** — call it the **reconciliation gate** — with five invariants over the buck2 target/dep graph.

**5.1 duplicate-implementation detector (anti-dupe)**
- *Checks*: content-addressed crate fingerprint (public trait/type surface + charter token) across dirs; flags two crates claiming the same concern (e.g. two Tenant structs, two outbox kernels, two module-registry catalog.json) and doc-only capability shells whose named crates live under a different top-level dir.
- *Rung*: automation (advisory report) → blocking (fail on a *new* duplicate pair introduced by the candidate diff) → ratchet (existing-dupe count may only decrease, merge-base anchored).
- *Portable*: concern-equivalence thresholds + known-intentional-multi-backend allowlist (e.g. the 6 event-bus adapters, s3/oci adapters) live in a policy manifest; stranger's-repo fixture = a 2-crate repo with one real + one shell dir asserts one finding.

**5.2 fan-in ratchet / hub detector (anti-unmaintainable)**
- *Checks*: reverse-dep count per crate from the buck2 graph; a crate crossing the hub threshold (e.g. >40) that is NOT a sanctioned substrate kernel is a finding; existing hubs (oya-data-boundary-kernel@128, network-residency@32, serde_json@567) are baselined and may only shrink.
- *Rung*: automation (fan-in report) → ratchet (per-crate fan-in ceiling from merge-base, no new hubs).
- *Portable*: threshold + sanctioned-hub allowlist as data; fixture = a repo where crate X gains a 41st dependent → one finding.

**5.3 visibility-tightness gate (buck2 visibility must name consumers)**
- *Checks*: every `rust_library`/`rust_binary` target's `visibility`; `["PUBLIC"]` on a non-port crate is a violation — a crate may be PUBLIC only if it is a `*-api`/`*-kernel` port OR names its consumers explicitly. (Third-party reindeer output is exempt by generator tag.)
- *Rung*: automation (count PUBLIC leaks) → ratchet (PUBLIC-target count may only decrease — the whole repo is PUBLIC today, so a hard block is infeasible until drained) → blocking on new non-port PUBLIC targets.
- *Portable*: port-suffix list (`-api`, `-kernel`) + generator-exempt tag as data; fixture = a repo with one PUBLIC core crate → one finding, one PUBLIC `-api` crate → zero.

**5.4 cross-capability-must-use-port lint (the rustc rebuild firewall)**
- *Checks*: any Cargo/BUCK dep edge whose source and target are in different top-level capabilities MUST target a `*-api` port crate; a cross-cap edge into `core/` or `adapters/` (e.g. iam/pdp-cedar → audit-chain-domain, k8s facade → tenant-quota-kernel, storage consumer → adapters/s3) is a violation.
- *Rung*: automation → ratchet (cross-cap non-port edge count from merge-base only decreases) → blocking on new violations.
- *Portable*: capability-map (top-dir → capability) + port-suffix + intentional-cross-cap allowlist as data; fixture = repo with cap-A/core → cap-B/core edge → one finding.

**5.5 hermeticity gate (anti-non-hermetic)**
- *Checks*: no `build.rs` network/undeclared-fs reads; no `Command::new`/`std::process::Command` in prod (non-test, non-sanctioned) code — flags the confirmed ambient shellouts (ci/adapters/path-resolver git, intelligence claude-agent-sdk/codex subprocess, tools/lane-supervisor codex/gh/kill, marketplace dev-cli bash/cargo/tofu, libs governance-lifecycle git); buck2-declared inputs only (no cross-crate-dir manifest reads like iam identity-workload-rest build.rs); reproducible outputs.
- *Rung*: automation (inventory ambient calls) → ratchet (count only decreases) → blocking on new prod `Command::new`.
- *Portable*: sanctioned-exception list (ci/adapters/path-resolver "single git boundary", kernel/harness QEMU) as data with ADR anchors; fixture = repo with a prod `Command::new("git")` → one finding, same call under `#[cfg(test)]` → zero.

### 5b. Product properties (mandatory)

- **CONFIGURABLE** — every threshold (hub fan-in ceiling, PUBLIC budget), allowlist (sanctioned hubs, intentional multi-backend adapters, hermeticity exceptions), port-suffix set (`-api`/`-kernel`), and the capability-map (top-dir → capability) live in **policy-as-data manifests** (checked into `specs/`/`registry/`, read at eval time). Engine code carries **zero repo-specific constants** — no hardcoded "oya-data-boundary-kernel", no hardcoded "128", no hardcoded dir names. Swapping the manifest re-targets the whole family at any repo.
- **API-DRIVEN** — every invariant is queryable and invokable over an owned API (REST + gRPC + streaming, **no GraphQL**): `GET /reconciliation/findings?invariant=anti-dupe`, `POST /reconciliation/evaluate` returns structured findings (crate, edge, rung, verdict, ADR-anchor) a console/operator renders — not CLI-only. The same structured finding shape feeds the gate and the ops console.
- **CLOUD-NATIVE** — ships as a **continuous reconciler** (CRD: `ReconciliationPolicy` + operator watching the dep graph), GitOps-reconciled, fail-closed at the admission gate (a PDP fault or missing manifest denies, never defaults-green). Not a one-shot script — the reconciler re-evaluates on every graph change and holds the ratchet baseline.
- **BUCK2-NATIVE** — consumes the **buck2 target/dep graph** as its primary input (deps, visibility, srcs globs — the same graph §5.1–5.4 read), runs as buck2 gate targets, is **hermetic** (declared inputs only) and **affected-set-aware** (only re-evaluates capabilities touched by the candidate diff, per the affected-target-set machinery already in ci/facade/), so it scales as a required `oya-ci-required` context without full-graph recompute per PR.
