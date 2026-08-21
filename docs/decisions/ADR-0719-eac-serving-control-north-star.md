---
doc_status: published
id: ADR-0719
title: "EaC north star: serving vs control, proto IR, cell-local authz, packs not an EU world-floor"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-21
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0701, ADR-0702, ADR-0704, ADR-0705, ADR-0708]
amended_by: []
depends_on: [ADR-0615, ADR-0701, ADR-0702, ADR-0704, ADR-0705, ADR-0710]
related: [ADR-0243, ADR-0280, ADR-0354, ADR-0049]
milestone: W0
deliverables:
  - id: ADR-0719-D1
    description: "Record serving-path vs control-path split as live law: 10^8-class user/Check traffic is in-cell RAM snapshots; writes, IR apply, packs, and cluster objects are a journaled control plane. etcd is fenced to cell cluster objects behind k8s/ports. Product records, tuples, and IR are never etcd/CRDs."
    exit_criteria: "This ADR is Accepted; CLAUDE.md live apex list cites it; no implement PR treats etcd or a Kubernetes object store as the Check/IR/tuple store."
    verified_by: "presubmit"
  - id: ADR-0719-D2
    description: "Record EaC as one protobuf IR plus per-plane reconcilers behind one gateway. No wrap language (CUE/Timoni/Haskell/Helm-as-source). JSON is not a product codec."
    exit_criteria: "New public product surfaces and IR apply/preview/watch are authored from the Rust/proto contract SSOT; Helm/Tofu/CUE are not sources of desired state."
    verified_by: "presubmit"
  - id: ADR-0719-D3
    description: "Record compliance as jurisdiction packs. EU is not the world baseline. KR (and others) are not GDPR subsets. ReBAC and snapshots stay in the certified cell."
    exit_criteria: "Pack overlays are the only place jurisdiction law is specialized; no implement PR assumes EU-only identity, retention, or global ACL replication."
    verified_by: "presubmit"
  - id: ADR-0719-D8
    description: "Closed directory set for repo root and capability/app/<product>/ roots. A name exists only if a compiler, test, PDP, SLO controller, or reconciler loads it (or it is OWNERS/README/BUCK/app PRD). Census files and wrap languages are not children."
    exit_criteria: "ADR-0701 Status cites this D-8; new cap/app children outside the set are born-blocking without grandfathering catalog.yaml or dual cedar+policy; layout-allowlist PRs match this set."
    verified_by: "presubmit"
  - id: ADR-0719-D9
    description: "The merge-blocking CI context is named presubmit (Google TAP-shaped). New workflow and required-context names do not use an oya- prefix. Today's oya-ci-required string is a rename target, not the destination name."
    exit_criteria: "This ADR uses presubmit as verified_by; no new ADR or workflow is named oya-ci-*; the live GitHub required context rename is a follow-through PR that updates branch protection in the same change."
    verified_by: "presubmit"
  - id: ADR-0719-D10
    description: "Hyperscaler pipeline names: presubmit (merge-blocking, graph-aware), postsubmit (on merge to dev), nightly, weekly, promotion rungs dev-staging-canary-production, release train bundling. One required context. No oya- prefix. No per-capability required GitHub checks."
    exit_criteria: "This ADR defines those cadences; new workflows use those names; oya-ci-required remains a rename target with branch protection in the same follow-through change."
    verified_by: "presubmit"
  - id: ADR-0719-D11
    description: "Cloud-provider placement: the registered capabilities ARE the cloud. Repo root holds only directory names plus meta (build/third-party; base/ only when admitted) and app/. No kernel/ or os/ rungs — node is upstream Talos/Linux; port-engine regenerates a port when we own the node. Each capability owns one engine (core), ports, adapters, facade. 2+ compose in app/. No cloud/ folder."
    exit_criteria: "This table is the placement reading; new engines go in an existing cap or a §7 registry split, not a new root dump; app/ is composition only."
    verified_by: "presubmit"
  - id: ADR-0719-D13
    description: "Node is upstream Linux + Talos (Sidero). Delete in-tree kernel/ (Asterinas eval) and os/ (generated Talos port). Keep build/port-engine; regenerate a port only when we own the node. No empty kernel/os scaffolds. A higher Accepted ADR overrides an earlier one only with explicit amends or supersedes."
    exit_criteria: "kernel/ and os/ are absent from the tree and from the capability-registry meta_directories; AGENTS.md/CLAUDE.md no longer list them as production rungs; port-engine remains under build/."
    verified_by: "presubmit"
  - id: ADR-0719-D14
    description: "Per-capability is/is-not/burn for the cloud-provider set. Nested leftover service trees burn. Product engines (payments, ledger, SaaS apps) are out of this set — later discussion."
    exit_criteria: "This table is the reading for reorg; no PR parks a second engine inside a cap or treats a k8s/Talos port as that cap's core."
    verified_by: "presubmit"
  - id: ADR-0719-D15
    description: "Cloud-provider purpose, in-scope, and out-of-scope for each registered engine. This set is IaaS/PaaS/control plane only. Tenant SaaS (HR, payroll, community, Slack-superset, SAP-class ledger/payments products) is app/, not a capability charter."
    exit_criteria: "Reorg and new crates match these in/out lists; no cap charter absorbs an app product; no app/ grows a cloud engine."
    verified_by: "presubmit"
  - id: ADR-0719-D16
    description: "console/ is not a cloud-provider capability. Discard the ops-dashboard-control-center pilot. git rm; no empty scaffold; do not park in app/ops-console. Token broker is iam. Operator actions stay on each cap facade. A future UI is app/ after the apps discussion."
    exit_criteria: "console/ is absent from the tree and from the closed capability registry; ADR-0701 Status cites D-16; layout allowlists do not re-admit console/."
    verified_by: "presubmit"
  - id: ADR-0719-D17
    description: "Presubmit is cargo fmt/clippy/test plus a short closed set of admission engines. Census gates, Helm/OpenAPI/OpenSLO parity, docs-coverage, frozen counts, min_expected_*, and expected_total pins are deleted, not trimmed."
    exit_criteria: "ci/facade and governance/check contain only the D-17 keep set; cedar-deploy-parity and scan-root-liveness are gone; no new gate is a path/count freeze."
    verified_by: "presubmit"
  - id: ADR-0719-D18
    description: "pipeline/ is TAP+Cloud Build; GHA disjoint; workflow/ and comms/ purged; rewrite workflow and notify; messaging→bus; .github/scripts any-language glue."
    exit_criteria: "ADR tables use pipeline/, bus/, notify/; workflow/ and comms/ trees absent; rust-first exclude_prefixes includes .github/scripts/; GHA YAML is not a face of pipeline/."
    verified_by: "presubmit"
---

# ADR-0719: EaC north star — serving vs control, proto IR, packs

## Status

**Accepted** (founder 2026-08-21). Amends the live identity/authz, k8s-port, product-protocol,
and platform-foundations apexes. Does not archive them.

Chat/session architecture is not law. This file is.

## Context

The platform was sketched as Everything-as-Code (EaC) plus Cedar and Zanzibar-pattern ReBAC,
then challenged at hundreds of millions of RPS, against AWS’s closed etcd-journal, and against
Eurozone (and KR) law. Three mistakes were available: (1) put serving state in etcd/CRDs or a
remote PDP; (2) treat JSON/REST/Helm as the product because they are familiar; (3) copy GDPR
as a global floor and copy Zanzibar’s worldwide ACL replica.

Hyperscalers split **serving** (memory, local, constant-work) from **control** (journal,
lower RPS). Zanzibar 2019: >10M client QPS, ~200M cache lookups/s, Spanner behind the
watch, not etcd on `Check`. EKS rebuilt etcd (Raft → internal journal, BoltDB → tmpfs) to
sell **one 100k-node Kubernetes**. GKE used Spanner behind the apiserver watch cache. We
sell a **cloud** sharded by **cells**, not a 100k-node logo.

## Decision

### D-1 — Serving vs control

- **Serving path** (product RPC + `Check`, 10^8-class fleet): Maglev-class anycast, then
  one logical L7 gateway, then **in-process PEP**, then **in-process PDP** on a **cell-local
  RAM snapshot** (compiled Cedar + ReBAC shards + product cache). Lookups ≫ RPCs. RPC
  `Check` is miss / recent-zookie / cross-cell only.
- **Control path** (IR apply, pack sign, tuple write, schedule, SLO, billing close):
  journaled, sharded, 10^3–10^5 class. First-party IR may live in git. Tenant IR lives on
  the gateway API.
- **Scheduler** = kube-scheduler (pods) + cell placement (tenants). Not an EaC scheduler.
- **Orchestrator** = `workflow` for business sagas. Apply order is `iac`’s object graph.
- **Monitor** = `observability` + `audit` + `iac` drift. No `eac-monitor`.
- **Audit on serving:** two classes. Privileged / admin / payroll-approve / policy-publish
  persist evidence **before ACK**. Other `Check`s may be async/sampled. Silent drop under
  load on privileged class is forbidden (DORA/NIS2-shaped).

**MUST (serving store)**

- **achieves:** 10^8-class `Check` and product RPC without a consensus box on the hit path.
- **origin:** etcd/Raft and remote PDP-RPC were drawn as if they were Borg/Zanzibar.
- **rule:** serving state is cell-local memory snapshots; consensus/journal is off the hit path.
- **ensure:** no PR stores tuples, IR, or tenant documents as etcd keys or product CRDs.
- **overturn_when:** a measured serving path needs a different store AND a replacement ADR
  with five fields lands same-wave.

### D-2 — k8s / etcd / “AWS journal”

`k8s/` remains the owned control plane + managed-k8s facade (ADR-0704). Its durable store
is **cluster objects in a cell** (pods, nodes, VAP params), served from an apiserver
**watch cache**. Vanilla Talos etcd Raft is accepted **only** until `k8s/ports` grows an
owned journal+memory adapter.

Do **not** adopt Amazon’s EKS journal. It is closed, not OSS, clock/hardware-shaped, and
exists to preserve the **etcd API** for a Kubernetes vendor. Steal the split (log vs
memory, no single Raft leader on the hot path). Implement the log ourselves behind the
port, cell-local, MIT/Apache. Cells, not one mega-cluster, are the scale lever.

### D-3 — EaC shape

EaC is **one protobuf IR** (Rust contract → Protobuf Editions) plus **per-plane
reconcilers**. Unifier: platform ∧ pack ∧ app ∧ tenant. Helm, Tofu, CUE, Timoni, Haskell,
and Kyverno/Kubewarden are **not** sources of desired state. Helm may exist only as an
`iac` **adapter** that renders third-party charts into objects **validated against the IR**.

There is no mega `EaService` that owns policy + infra + SLOs. Plane APIs stay plane APIs.
A thin IR `validate` / `unify` / `preview` / `apply` / `watch` fans out. The SDK is
generated from the IDL. Console is a client of the same methods. Apps (`app/<product>/`)
are **IR modules** on the cloud, not private Helm worlds.

**MUST (no wrap language, no JSON product codec)**

- **achieves:** one evolution story; JSON/YAML/Helm dual stacks cannot become the product.
- **origin:** public REST+JSON and Helm-as-source were compatibility fossils and corpus.
- **rule:** product and IR contracts are protobuf; JSON is not a product codec; wrap
  languages are not EaC.
- **ensure:** new surfaces take types from the Rust/proto SSOT; gates do not require JSON
  censuses (`manifest.json`, scorecard dumps) as existence proofs.
- **overturn_when:** a pack or Data-Act export format is separately Accepted as
  **control-plane dump**, not as the serving API.

Leftover REST/JSON public shapes are **reorg_now deletion**, not a supported codec.
Temporarily breaking callers is accepted; a dual REST+proto product API is not.

### D-4 — Protocol (no pre-2016 compatibility as destination)

North star wire: HTTP/3 + protobuf, Connect-class HTTP (no gRPC trailers), WebTransport for
watches. Identity on the channel: SPIFFE mTLS east-west; passkeys at L2; **step-up to L3/L4**
(KR 본인인증 / eIDAS EUDI / passport+liveness per interview D58) via Cedar `acr_required`.
PQC + ECH at the edge (ADR-0354). Public gRPC-Web is not an operator-UI protocol. FlatBuffers
remain a measured adapter, never a second SSOT.

`Check` is never a public method. Product RPCs are. PEPs call the in-cell PDP.

### D-5 — Authz placement (ADR-0615 stands, topology corrected)

`policy/` is the Cedar PDP + ReBAC tuple plane (G-face distribute, C0 in-cell snapshot).
`iam/` emits the principal. Stale snapshot denies or routes; never silent allow.

**ReBAC is cell-local.** Zanzibar’s “replicate all ACLs worldwide” is rejected: tuples and
group memberships are personal data; global replica fails residency (E18, ADR-0049, GDPR
transfers, KR CSAP). Cross-cell `Check` is an explicit Cedar’d hop.

Subject IDs in tuples, logs, and meters are **opaque handles** under the subject DEK
(interview D10). Erasure is crypto-shred + unlink. Legal retention (D11) is pack data and
blocks shred for that class only. Merkle+TSA (D86) hashes the control log; it does not
store PII and does not sync-append every `Check`.

### D-6 — Packs, not “EU = world”

- **achieves:** sell KR and EU (and later US/others) without a fake global GDPR floor.
- **origin:** “strictest common subset = EU” fails KR (CSAP, 본인인증, RRN, e-tax, K-GAAP
  retention) and fails CN localization; DORA/AI Act/eIDAS are not universal.
- **rule:** jurisdiction law is **pack overlay** on the same IR/Cedar/ReBAC/cells. EU is
  one pack (`packs/eu`). KR is one pack (`packs/kr`). Structural controls that help many
  regimes (crypto-shred, certified cells, purpose in Cedar, two-class evidence, published
  proto + event-log export) live in the platform; the rest lives in the pack.
- **ensure:** no PR encodes EU-only identity or worldwide ACL replica as default; cell
  placement refuses a pack that exceeds the cell’s certification (E18).
- **overturn_when:** a single jurisdiction is the only remaining market AND a replacement
  ADR says so.

EU next-decade instruments (GDPR tightening, DORA, NIS2, AI Act, Data Act, CRA, eIDAS 2)
are **inputs to `packs/eu`**, not a reason to delete KR-shaped L3 or legal-retention
classes. Data Act switching is a **control-plane dump** of the event log and proofs, not a
JSON public API.

### D-7 — 3-year no-regret / regret

**Keep even if implementations swap:** cells; serving RAM vs control journal; Cedar+ReBAC
in-cell; proto IDL; in-process `Check`; k8s store behind a port; apps as IR modules;
two-class evidence; shreddable principals; eIDAS/KR step-up; owned cell journal; one
gateway; packs.

**Regret:** etcd as product DB; AWS closed journal; JSON as product codec; Helm-as-source;
Kyverno/Kubewarden as default; one global cluster; worldwide ACL replica; silent drop of
privileged evidence; passkeys as L3; unpublished binary lock-in; EU-as-only-baseline;
cap-root census files; dual `cedar/`+`policy/` children.

### D-8 — Repo root and capability / app root (amends ADR-0701)

A directory or file is allowed only if something that is **not a census gate** loads it,
or it is `OWNERS` / short `README.md` / `BUCK` / `app/<product>/PRD.md`. Git history is
the audit log. Do not invent a destination for leftovers; many must **not exist**.

**Repo root (closed).** Workspace: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
`rustfmt.toml`, `deny.toml`, `reindeer.toml`, `.buckconfig`, `.buckroot`, `.cargo/`.
GitHub: `.github/`, `.gitignore`, `.gitattributes`. Hubs: `README.md`, `LICENSE`,
`OWNERS`, `AGENTS.md`, `CLAUDE.md`. Meta: `build/`, `third-party/`. `base/` is **not**
pre-created; it appears only when the first crate admitted under the ≥3-caps-below-all
rule. `governance/` is a **capability** (checks + `capability-registry.json`).
**No `kernel/` and no `os/` rungs.** Node is **upstream Linux + Talos (Sidero)**.
Asterinas evaluation is **removed**. In-tree Talos/Asterinas port output is **deleted**;
`build/port-engine` remains so a port can be **regenerated when we own the node**, not
kept as a second OS in git. `os/ports/kernel-abi` dies with `os/`.
Composition: `app/`. One directory per **registered** capability (including `policy/`
the **engine**). Jurisdiction: `packs/<id>/` one versioned bundle the engines load.
`docs/` = ADRs + operating contract. **No catch-all `specs/`.** Machine contracts live
next to the evaluator (gate `*-policy.json`, Cedar, IR proto). Agent entry is
`AGENTS.md` / `CLAUDE.md`. `HANDOFF.md` is **deleted** (founder 2026-06-08 exception
withdrawn); it was a redirect, not law.

**Not repo-root:** `oya/`, `cloud/`, `libs/`, `infra/`, `tools/`, `toolchains/`
(reorg_now); root `Makefile`; root `Dockerfile*` (→ `build/`); `oya-*.toml`; tracked
agent dirs; `evidence/` dumps; `benchmarks/`; `scripts/`; `tasks/`/`plan/` as required
corpora; `catalog.yaml` trees; root `contracts/` YAML as IDL; root `registry/`;
root `specs/` as a JSON law corpus (delete with the gates that load contradicting
files; do not scatter into cap/app). `HANDOFF.md`. `kernel/`. `os/`.

**Capability root and `app/<product>/` (closed children):** `core/`, `ports/`,
`adapters/`, `facade/`, `cedar/` (**.cedar unique to this cap** — schema and resource
policies the PDP compiles; platform templates live under the `policy/` **capability**,
not stamped here), `observability/slos/` (**generated OpenSLO from IR only** — hand-authored OpenSLO,
including “unique” files, is debt; clones already forbidden), `iac/` (IR the reconciler
applies, not Helm/Tofu source), `OWNERS`,
short `README.md`, `BUCK`. `app/<product>/` may add `PRD.md`. No cap-root `contracts/`
(IDL is Rust → proto on `ports/` / generated faces). No cap-root `policy/` (that name
is the engine at repo root). Nested leftover service trees become faces or go.

**Must not exist at cap/app root:** `manifest.json` census, `catalog.yaml`, `IPs/`,
`IP-journey-*.md`, `AUDIT-FINDINGS-*.json`, `REMEDIATION-NOTES-*.md`, `scorecards/`,
`dashboards/*.json`, `dpia/`, `decisions/` copies, `capabilities/*.yaml` essays,
stamped runbooks, stamped `tenant-scope.cedar` copies, dual ARCH files, placeholder
READMEs. Delete the gate with the files.

**Public door:** proto/H3 is the product. REST/JSON leftover is **deleted**, not
transcoded as a standing codec. No new public REST shapes. Console/SDK/gates that
still speak REST may go red until they speak proto — that break is in-scope hygiene.

**MUST (closed children)**

- **achieves:** engine vs data names do not collide; N copies of platform Cedar/SLO
  cannot reappear; org law has one tree gates already load.
- **origin:** naming `policy/` for both the PDP and per-cap Cedar followed the live
  tree; OpenSLO-as-authoring and REST transcode are Helm-shaped dual stacks; dual
  `cedar/`+`policy/` allowlists encoded the collision.
- **rule:** cap-root `cedar/` only; `policy/` is the capability; SLO source is IR;
  no `specs/` catch-all; `ports/` is the contract face; extras, REST/JSON product
  surfaces, and `HANDOFF.md` deleted, not grandfathered. Temporarily breaking live
  callers/gates is accepted. Leaving anti-pattern debt is not.
- **ensure:** layout allowlists match this set; no immortal `IPs/`; no both `cedar/` and
  `policy/` as cap children.
- **overturn_when:** a child is loaded by a compiler/PDP/SLO/reconciler AND a five-field
  amendment lands same-wave.

## Consequences

- Implementers read this plus ADR-0701/0702/0704/0705/0615. Do not re-derive from chat.
- `policy/` extraction and IR proto are implementation follow-through, not optional sketch.
- Admission remains ADR-0710 (VAP/CEL+PSA); this ADR does not re-open ADR-0710's D-8.
- Merge-blocking CI is **presubmit**. Pair with **merge-admission**. See D-10.
- Node OS/kernel: D-13. Do not re-create `kernel/` or `os/` as empty rungs.

### D-13 — Node is upstream Talos/Linux; generated ports are not kept

**Decision.** Production node = **upstream Linux via Talos (Sidero)**. There is **one**
kernel story.

- **Delete `kernel/`.** Asterinas evaluation is removed. Upstream is pre-release; we
  **vendor into `third-party/` when we need it**, not a rung-0 theater.
- **Delete `os/`.** The in-tree Talos-shaped domain farm is **port-engine output** (or
  a shadow of Sidero). Keeping it is a second OS. The generator stays at
  `build/port-engine`. We **run the port engine again** when we own the node OS, not
  merge a permanent generated tree.
- **`os/ports/kernel-abi` is deleted with `os/`.** A kernel-ABI seam is created when
  we own a kernel, not as a vacant port.
- Consume Talos as **upstream** through **`k8s/adapters`** / **`iac/`**, not a
  parallel `os/` engine.

**`k8s/` is GKE/EKS/AKS, not kubernetes/kubernetes.** A successful Go→Rust **port**
of kube-apiserver does **not** give multi-tenant CP hosting, upgrades, quota, SLA,
CAPI, cell placement, or cluster billing. Google still has **GKE** after writing
Kubernetes. AWS/Azure sell **EKS/AKS** without writing Kubernetes. Collapsing those
into one “core = port, facade = managed” charter is the dual-stack we just killed
for Talos.

- **Do not add a second empty capability** for “the port.” Empty `k8s-port/` is a
  magnet. `build/port-engine` **is** the port (build meta, ADR-0704). Generated
  apiserver output is **not** checked in (same as `os/`). When we **run** an owned
  apiserver, it plugs in as **`k8s/adapters`** (alongside CAPI / upstream), not as
  a replacement for the product `core/`.
- **`k8s/` capability = managed cluster product only.** Today’s 18 crates already
  are that: cluster lifecycle, control-plane **hosting**, tenant quota, SLO, CAPI
  adapter. Registry charter “owned CP core + managed facade” is **scoped-superseded**
  by this paragraph. Nested leftover `k8s/managed-*` dirs **burn** — they duplicate
  faces.
- **`k8s/` dump burns (D-8).** IPs, AUDIT-FINDINGS, dual ARCH, dashboards, DPIA,
  `manifest.json`, scorecards, capability `PRD.md`, Helm/Tofu/kustomize **source**,
  cap-root `policy/` vs `cedar/`, extra `slos/`.
- **Not this cap:** node OS (Talos upstream), mesh/DNS (**network**), public door
  (**gateway**), cell topology (**cell**), SPIFFE issuance (**secrets**). GKE-class
  **uses** those; it does not own them.

**MUST (managed cluster ≠ k8s port)**

- **achieves:** porting Kubernetes cannot be mistaken for finishing the cloud
  product; no generated apiserver tree as `k8s/core`.
- **origin:** 0704 port-engine + 0562 coarse “k8s = owned CP + managed facade”
  mixed two jobs hyperscalers split (GKE vs kubernetes.git).
- **rule:** `k8s/` is the managed-cluster product; the port lives in
  `build/port-engine` and lands as an **adapter** only when we run it.
- **ensure:** no in-tree generated kube API farm; no empty port capability dir;
  nested `managed-*` deleted in favor of existing faces.
- **overturn_when:** we cut over production to an owned apiserver **and** a
  five-field ADR moves implementation from adapter to `core/` same-wave.

**Destinations (so delete is not an orphan):** nothing in `infra/` **needs** `os/`.
`infra/talos/` is **upstream Talos machine state** — it moves with the `infra/`
`reorg_now` burn into **`k8s/adapters`** (cluster consume) / **`iac/`** (desired
state), not into an `os/` keep.

| In `os/` today | Outside consumers | Destination |
|---|---|---|
| Talos domain farm (`machined`, `siderolink`, `kubelet-domain`, …) | none (self-only) | **Delete.** Port-engine regenerates if we own the node. |
| `os-trustd-domain` | `iam` SVID adapters + `iam/facade/cloud-pdp-app` | **`secrets/`** (trustd = SPIFFE issuance). Rehome those three crates’ deps **in the same change as `os/` delete**. |
| `os/core/kubernetes-domain` templates (PSA/namespace emit) | `governance/psa-exception-registry.json` | **`k8s/`** (it is apiserver/namespace law, not an OS). |
| `os/ports/kernel-abi` | none worth keeping | **Delete** with `os/`. |
| `os/harness/*` | workspace glob | **Delete** with `os/` or fold into `k8s/` only if it tests k8s, not Talos-port. |
| `infra/talos/**` | local cluster bring-up | **`k8s/adapters` / `iac/`** when `infra/` burns. Not a reason to keep `os/`. |
| `kernel/**` Asterinas | nested workspace only | **Delete.** Vendor to `third-party/` if we ever need the pin. |

Census JSON (`registry/graph`, manifests) **regenerates or dies** with the dirs. It is
not a destination.

**MUST (one kernel story)**

- **achieves:** no dual kernel/OS in git; no empty rungs; no generated-output debt.
- **origin:** kuberos deleted; Asterinas pin + Talos port farm remained as two more
  OS stories while production already runs upstream Talos.
- **rule:** in-tree `kernel/` and `os/` are gone; node is upstream Talos/Linux;
  port-engine may regenerate a port later; no empty scaffolds.
- **ensure:** registry `meta_directories` has no `kernel/` or `os/`; tree has no
  those dirs; AGENTS.md/CLAUDE.md match.
- **overturn_when:** we own a node kernel/OS and land it as port-engine output in a
  new rung with five fields same-wave.

**MUST (ADR override)**

- **achieves:** agents cannot treat a higher number as silent OVERRULE.
- **origin:** `HANDOFF.md` carried this; the file is deleted.
- **rule:** a newer ADR controls an earlier Accepted ADR only when it is Accepted
  and carries explicit `amends` or `supersedes` (reciprocal lifecycle edge).
- **ensure:** 0701/this ADR `amends` edges exist; number-only citations are not
  implement authority.
- **overturn_when:** a replacement lifecycle ADR with five fields lands same-wave.

### D-10 — Pipeline cadences (names, not brand)

Hyperscaler shape: Google **presubmit / postsubmit / continuous / release**; one
admission, graph-aware work; promotion is not “the test job.”

| Cadence | When | What | Blocks merge? |
|---|---|---|---|
| **presubmit** | every PR / merge-queue group | `cargo fmt --all --check`; clippy `-D warnings` (blocking once the tree is clippy-clean; advisory is debt); **`cargo nextest run --locked --workspace --no-fail-fast`** (this **is** compile proof — do not also `cargo check`); D-8 path-set; license; rust-first; generated-not-hand-edited; Cedar compile of **touched** `cedar/`. **linux/amd64 only.** | **Yes.** One required context. |
| **postsubmit** | merge to `dev` | Same nextest workspace (or remainder not proven on an affected set); start of promotion **into staging** only via the promotion pipeline | No (already merged). Failure is a **revert/block-next** signal, not a second required PR check. |
| **nightly** | schedule | arm64 nextest (D88-amend), fuzz, long E2E, soak | No |
| **weekly** | schedule | buck2 `build //...` honesty smoke (ADR-0716); hermetic graph | No |
| **promotion** | explicit rung | `dev` → `staging` → `canary` → `production`; predecessor check | N/A (branch protection on the rung) |
| **release / CD** | train (interview D63) | Bundle what’s on the promotion rung; **`cargo build --release`** (and buck2 if still local-honest) **here, not presubmit** | N/A |

**Cargo verbs (closed).** Hyperscaler TAP: cheap local, hermetic blocking presubmit, slow on a schedule, artifacts only on CD. `cargo nextest` is the only compile+test proof. `cargo check` is a local optional shortcut, never CI. `cargo test` (libtest) is not CI. `rustfmt` CLI is not CI — `cargo fmt` is. Windows/macOS per-PR smoke is not a cadence.

The “four-tier cargo blog” (fmt+clippy+check locally in <5s; nextest+doc-tests+deny+audit+semver on PR; mutants+machete+update+asan nightly; dist+bloat on CD) is the right **shape** and the wrong **tool list** for this repo.

| Tool | Local | Presubmit | Nightly / weekly | CD | Ruling |
|---|---|---|---|---|---|
| `cargo fmt` | touched `--check` | `--all --check` | — | — | **Keep.** Merge bar is `--all`. Local is not `--all` (this workspace is not <5s). |
| `cargo clippy -D warnings` | optional, touched | `--workspace --all-targets` (blocking when clean) | — | — | **Keep.** Local `--all-targets` is a lie on an 800-crate graph; it gets skipped. |
| `cargo check` | optional | **never** | **never** | **never** | **Drop from CI.** Nextest already compiles. A check job is a second compile. |
| `cargo nextest` | `-p` affected | `--workspace --locked --no-fail-fast` linux/amd64 | arm64 workspace; long / soak | — | **The proof.** Developers run affected; CI runs the workspace. |
| `cargo test` / `--doc` | **never** | **never** | **never** | **never** | **Drop.** libtest is not the runner. Doctests are `doctest = false` here; an empty `--doc` job is vacant. If rustdoc tests return, they are nightly until nextest covers them. |
| License / bans (`deny.toml`) | — | hermetic `cargo deny check licenses bans sources` **or** the owned license-policy — **one**, not both | — | — | **Keep one.** Dual deny+license-policy is bloat. Advisories are not this row. |
| Advisories | — | **vendored** snapshot (owned supply-chain-audit). **No network.** | Network `cargo audit` / refresh snapshot as a **report**, not a lockfile mutate | — | **PR is hermetic.** TAP does not fetch rustsec.org to merge. |
| `cargo-semver-checks` | — | **never** | — | — | **Drop.** Public surface is proto/H3 facades, not crates.io semver of internal crates. |
| `cargo-mutants` | — | **never** | **sampled** / per-crate, not 800 crates | — | **Not whole-workspace nightly.** That is weeks. |
| `cargo-machete` | — | **never** | weekly advisory | — | **Keep as signal.** Unused-dep cleanup is not merge-blocking. |
| `cargo update` | — | **never** | scratch report; human PR | — | **Cron must not rewrite Cargo.lock.** |
| ASan / TSan | — | **never** | subset of concurrent crates | — | **Keep as nightly subset.** Not the workspace. |
| `cargo build --release` | — | **never** | **never** | **yes** | **Keep.** |
| `cargo-dist` | — | — | — | **never** | **Drop.** CLI retirement; we ship images + promotion, not GitHub Release binaries. |
| `cargo-bloat` | — | — | optional size log | not a ship gate | **Metric, not CD.** |
| `buck2 build //...` | local hermeticity | **never** | weekly honesty | optional | Unchanged (ADR-0716). |
| Live Postgres nextest | — | dedicated jobs (service) | — | — | **Keep.** Not a laptop default. |
| Win/mac | — | **never** | — | — | **Drop.** D88-amend: amd64 per-PR, arm64 nightly. |

Local pre-push is **fmt on touched files**, not a 5-second fantasy that also runs clippy `--all-targets` and `cargo check`. A hook that takes minutes is a hook people `--no-verify`. Tests the author cares about are `cargo nextest run -p <crate>`.

No new `scripts/check.sh` / pre-push product. Rust-first: a three-line git hook may call `cargo fmt --check` on staged `*.rs`. Do not resurrect `oya verify` / `dev-cli`.

**MUST (nextest is the proof)**

- **achieves:** one compile+test signal; no double compile; no libtest dual; PR hermetic.
- **origin:** blog four-tier put `cargo check` and network `cargo-audit` on the merge path; `cargo-dist` assumes a CLI product; mutants-on-everything is not a nightly.
- **rule:** nextest is the only compile+test proof in presubmit/postsubmit/nightly unit lanes; `cargo check` and `cargo test` are not CI; release binary is CD; advisory fetch on PR is vendored; one license/ban engine; no cargo-dist; no crates.io semver gate.
- **ensure:** required workflow invokes nextest, not libtest; no `cargo check` job; no win/mac per-PR smoke; deny/audit are not two network tools on the PR.
- **overturn_when:** a five-field ADR names a different runner that still compiles once and stays hermetic.

Do **not** add one required GitHub check per capability (skipped-check failures, queue
combinatorics). Lane isolation is **worktrees + non-overlapping paths**, not 24
contexts. Do not resurrect merge-base **count** baselines as “affected set.”

New workflow and context names: `presubmit`, `postsubmit`, `nightly`, `weekly`,
`promotion-predecessor`, `release`. No `oya-` prefix. Today’s `oya-ci-required` is
the **presubmit** rename target (branch protection in the same change).

### D-11 — What the cloud is, and what each capability holds

The cloud is **not** a root `cloud/` tree and **not** a JSON catalog. It is the
**closed capability set** (registry). Repo root only **names** those directories
(plus `kernel/` `os/` `base/` `build/` `third-party/` `app/` `packs/` `docs/`
`governance/` as already in D-8). Everything a tenant or operator calls is a
**facade** of one capability or an **`app/<product>/`** that wires 2+.

**Inside every capability (D-8 children only):** `core/` = engine we run;
`ports/` = traits + IDL; `adapters/` = transient AWS/OCI/etc.; `facade/` = sold
API; `cedar/` = this cap’s unique Cedar; `observability/slos/` generated from IR;
`iac/` = this cap’s IR desired state (what *this* engine needs in a cell). PEP
calls `policy/` in-process; it does not embed a second PDP.

**AWS/GCP analog → our cap** (coarse, for placement — not a product-mapping SKU list):

| Cloud-provider concern | Capability | core / facade split |
|---|---|---|
| Regions / cells / hard caps | **cell** | core: topology, router, rebalance. Not tenant CRM. |
| Accounts / orgs | **tenancy** | core: tenant lifecycle, home-cell. Enablement SKUs are IR apply, not a side console. |
| IAM users/roles/IdP | **iam** | core: principals, passkeys, SCIM, tenant-RBAC **store**, workload identity **consumption**. Does **not** evaluate Cedar. |
| IAM Access Analyzer / Zanzibar / Verified Permissions | **policy** | core: PDP + ReBAC tuple store (G + C0). Every other cap is a PEP. |
| KMS / Secrets Manager / SPIFFE | **secrets** | core: keys, secrets, **SVID issuance**. |
| CloudTrail | **audit** | core: Merkle log. Always on. Async seal on serving path (D-1). |
| CloudWatch / Monarch | **observability** | core: telemetry + SLO **controller**. Per-cap SLOs are IR → generated OpenSLO, not this cap’s YAML novel. |
| S3 / GCS / CAS | **storage** | core: blob/CAS. Drive/recordings = **facade**. Imaging waits a product, not a nest. |
| RDS / Spanner / BigQuery / ontology | **data** | core: ontology, OLTP/OLAP, pipelines. Foundry-shaped data plane. Not S3. |
| EC2 / GCE / Functions / GKE-on-VMs | **compute** | **One** engine: VM + k8s-on-compute + functions. Facades, not three caps. |
| GKE / EKS control plane (sold) | **k8s** | core: owned apiserver. Facade: managed cluster. Store: cluster objects only (D-2). |
| VPC / DNS / mesh | **network** | core: mesh, signed DNS snapshots, cell dataplane. |
| Front door / GFE / API Gateway | **gateway** | core: the **one** public door (D-3/D-4). Rate/quota. Transcode is not a second API. |
| Pub/Sub / Service Bus | **bus** (today’s dir `messaging/` until git mv) | core: bus, outbox, per-key order. Not `comms`. |
| SageMaker / internal AI | **intelligence** | core: model/agent substrate. AI Act registry is pack + this cap, not `capabilities/*.yaml` essays. |
| Step Functions / Composer | **workflow** | core: engine. Studio is facade. Business sagas, **not** deploy orchestrator (D-1). |
| Cloud Build / TAP / CodePipeline | **pipeline** (today’s dir `ci/` until git mv) | core: graph-aware execution, queue, controller. GitHub is an **adapter**, not the product. |
| CloudFormation / Config reconciler | **iac** | core: IR unifier + reconcilers. `<cap>/iac/` is **this** cap’s desired state; `iac/` the cap owns the **engine**. |
| Billing / Cost Explorer | **billing** | core: meter, rate, invoice, tax, FinOps. Sold-ness, not a drawer. |
| Marketplace | **marketplace** | core: plugins, signed modules, SKU **engine**. Generated sell-catalog view is `build/`. |
| Artifact / evidence packs | **compliance** | core: pack evidence, data-class registry. Consumes **audit**. Not the Merkle log. |
| SES / SNS / FCM (send) | **notify** | core: transactional email/SMS/push **send**. Not Gmail/Meet/Slack. |
| AppConfig / Feature flags | **flags** | core: flags, kill switches. Pack-gated overrides. |

**Meta (not sold as a tenant API, still in-repo):** `kernel/` rung 0; `os/` node OS; `base/` (≥3 caps, below all); `build/` toolchains/images; `third-party/` vendored; `governance/` registry + check crates (off the runtime ladder).

**`app/<product>/`:** composition only (hr, payroll, calendar, community, …). Wires 2+ of the table. **Does not** grow a cloud engine.

**`payments/` and `ledger/` are not this cloud set** (D-15). Do not park them in `billing/` or `oya/`. Product placement is a later discussion.

### D-14 — Each capability: is / is not / burns

Same split as `k8s/` (GKE product vs kube port). Nested leftover service dirs inside a cap **burn** (faces or `git rm`).

| Cap | **Is** (engine) | **Is not** | **Burns / move** |
|---|---|---|---|
| **cell** | Topology, hard caps, router, rebalance. Borg/GCP-zone analog. | GKE product (`k8s/`). Tenant CRM. | Census, leftover lifecycle dirs once faces exist. |
| **tenancy** | Tenant lifecycle, home-cell, org/account analog. | IdP (`iam`). PDP (`policy`). SKU catalog (`marketplace`). | Enablement side-effects; JSON tenant novels. |
| **iam** | Principals, passkeys, SCIM, role **store**, workload identity **consume**. | Cedar **eval** (`policy`). SVID **issue** (`secrets`). | PDP crates **move to `policy/`**. trustd deps **move to `secrets/`** with `os/` delete. |
| **policy** | Cedar + ReBAC PDP, G-face + C0 snapshots. | IdP. Empty dir forever. | **Extract crates from iam now.** Cap-root `<other>/policy/*.cedar` → `<other>/cedar/`. |
| **secrets** | KMS, secret material, **SPIFFE issue**. | PDP. Cert spam as YAML. | Absorb `os-trustd-domain` consumers. |
| **audit** | Tamper-evident log. Always on. | Compliance packs (`compliance`). Sync Merkle on every Check. | DPIA essays, scorecards. |
| **observability** | Telemetry + SLO **controller**. | Per-cap hand OpenSLO. SIEM as a 25th cap. | Stamped OpenSLO; detection stays **intelligence** facade if sold. |
| **storage** | Object/CAS. Drive/recordings **facade**. | Ontology DB (`data`). Block/file = **facades when sold**, no empty dirs. | Nested imaging leftover; census. |
| **data** | Ontology, OLTP/OLAP, pipelines. | S3 (`storage`). Cache/search = facades **when sold**. | Nested ontology/analytics leftover trees; Postgres is **adapter**. |
| **compute** | **One** engine: VM + k8s-on-compute + functions. | GKE product (`k8s/`). GPU = facade when sold. | Splitting into 3 caps. |
| **k8s** | **Managed cluster product** (lifecycle, CP host, quota, SLO, CAPI). | kube-apiserver port (`build/port-engine` → adapter when we run it). Node OS. Mesh. | Dump + nested `managed-*`. |
| **network** | VPC/DNS/mesh dataplane. | Public API door (`gateway`). Direct Connect/CDN = facade when sold. | Census. |
| **gateway** | **One** public door, quota, proto/H3. | Mesh (`network`). Second REST API. | REST dual-stack; connector leftover if it’s a second door. |
| **bus** | Outbox, order, pub/sub, queues. | Sagas (`workflow`). Human notify (`comms`). Kafka-as-source. | git mv `messaging/` later. |
| **intelligence** | Model/agent substrate. | Console. Workflow studio. AI Act essays as cap-root YAML. | `capabilities/*.yaml`; detection is a **facade** of this cap if sold. |
| **workflow** | Step Functions analog (rewrite). | Bus (`bus`). Forms/tasks/SaaS. Deploy (`pipeline`/`iac`). | **Purge current tree; rewrite.** Do not strangler. |
| **pipeline** | TAP + Cloud Build engines, queue, controller. | This repo’s `.github/` GHA. Census gates. | GHA stays disjoint; census already D-17. git mv `ci/` → `pipeline/` later. |
| **iac** | IR unifier + reconcilers. | Argo-SHA observer as the engine. Helm/Tofu **source**. | Observer; `<cap>/iac` Helm dumps. |
| **billing** | Meter, rate, invoice, tax, FinOps. | Ledger books (`ledger/`). Payments rails (`payments/`). | Nested accounting/tax leftover dirs. |
| **marketplace** | Plugins, signed modules, SKU **engine**. | Generated sell-catalog (**`build/` view**). | Dev-cli as merge authority. |
| **compliance** | Pack evidence, data-class registry. | Merkle log (`audit`). Cloned `dpia.md`. | Those clones. |
| **notify** | Transactional send (SES/SNS/FCM). | Email/SMS/push **send API**. | Mailbox/Meet/Messenger/contact-center (`app/` later). Current `comms/` dump **purged**. |
| **flags** | Flags, kill switches. | Census `catalog.yaml` / IPs. | Cap-root junk. |
| **governance/** | Registry + check **crates** (off ladder). | Org JSON `specs/` corpus. | Specs catch-all. |
| **build/** | Toolchains, images, **port-engine**, SKU **view**. | Capability engines. | — |
| **third-party/** | Vendored pins when we need them. | Fake rungs (`kernel/`/`os/`). | Asterinas eval in `kernel/`. |
| **app/** | 2+ cap products. | A cloud engine. | Absorbing D41 retirees; parking payments; **do not absorb the `console/` pilot**. |

**Missed before, now closed:** GKE vs kube-port (`k8s/`); Talos vs `os/`; PDP vs iam; trustd vs secrets; payments/ledger not billing; no empty `base/`/`kernel/`/`os/`/`k8s-port/`; census `ci` gates are not the delivery fabric.

### D-15 — Cloud-provider purpose and scope (not SaaS)

This set is what we **sell and run as a hyperscale cloud**. Analog: AWS/GCP/Azure **platforms**.  
**Out of every row below:** tenant SaaS products (HR, payroll, community, calendar, Slack-class UX, SAP-class accounting). Those **use** these engines via `app/<product>/`. A capability that ships a vertical product in `core/` is out of charter.

| Cap | Purpose | In scope | Out of scope |
|---|---|---|---|
| **cell** | Bound failure domains and place load. | Cell topology, hard caps, router, rebalance, home-cell **placement**. | Selling a k8s cluster (`k8s`). Tenant directory (`tenancy`). Mesh (`network`). |
| **tenancy** | Tenant as the scoping primitive. | Create/suspend/delete tenant, home-cell binding, org/account tree. | IdP (`iam`). Authz eval (`policy`). Marketplace SKUs. HR orgs. |
| **iam** | Prove **who**. | Principals, credentials, passkeys, SCIM, role **store**, workload identity **consume**. | Cedar **eval** (`policy`). SVID **issue** (`secrets`). Tenant lifecycle (`tenancy`). |
| **policy** | Decide **may**. | Cedar PDP, ReBAC tuples, G-face distribute, C0 in-cell snapshot, in-process Check. | IdP. Writing every cap’s Cedar (caps own `<cap>/cedar/`). Global tuple replica. |
| **secrets** | Crypto root and issuance. | KMS, secret material, SPIFFE **issue**, cert **issue** when sold. | PDP. Embedding secrets in app products. |
| **audit** | Tamper-evident **record**. | Merkle log, seal of principal+tenant, privileged-path durability. | Pack evidence (`compliance`). Sync seal on every Check. DPIA markdown. |
| **observability** | See and SLO-gate the platform. | Metrics/logs/traces **substrate**, SLO **controller**, generated OpenSLO apply. | Hand OpenSLO novels. SIEM as a 25th cap. App product analytics. |
| **storage** | Durable **bytes**. | Object/CAS; drive/recordings as **byte facades**. | Tables/ontology (`data`). Block/file until sold as facades. Imaging product. |
| **data** | Durable **records** and analytics engines. | Ontology/OLTP/OLAP/pipelines as **platform**. Postgres as adapter. | Object store (`storage`). Foundry **app**. Cache/search until sold as facades. |
| **compute** | Run **workloads**. | One engine: VM + k8s-on-compute + functions. GPU as facade when sold. | GKE product (`k8s`). Cell topology (`cell`). |
| **k8s** | **Managed Kubernetes product** (GKE/EKS/AKS). | Cluster lifecycle, hosted CP, quota, SLA, CAPI, **adapter** to upstream or owned apiserver. | kube-apiserver **port** (`build/port-engine`). Node OS. Mesh. Public door. |
| **network** | Connect inside the cloud. | VPC, DNS snapshots, mesh dataplane. | Public API door (`gateway`). CDN/Interconnect until sold as facades. |
| **gateway** | **One** north-south door. | Proto/H3 edge, authn terminate, quota, Cedar on the call. | Mesh (`network`). Second connector door. Tenant SaaS APIs implemented here. |
| **bus** | Move **events** (Pub/Sub / Service Bus). | Outbox, idempotency, per-key order, queues, streams. | Sagas (`workflow`). Mail/chat (`comms`). Kafka-as-source. |
| **workflow** | Managed **sagas** (Step Functions / Cloud Workflows). | Rewrite: state machine, retries, timers, execution API; studio as authoring **facade**. | Bus (`bus`). Forms/tasks/SaaS. Deploy (`pipeline`/`iac`). Current tree (purged). |
| **intelligence** | Managed **model/agent** substrate. | Inference/agent runtime, adapters to models, platform proof layer. | Console. Workflow studio. Vertical AI products. Cap-root autonomy YAML essays. |
| **flags** | Dynamic config and kill switches. | Flag eval, targeting, kill switch. | App feature roadmaps. Census catalogs. |
| **pipeline** | Sold TAP + Cloud Build. | Graph-aware execute, queue, controller, SCM **adapter**. Tenant graphs. | `.github/` GHA. Census gates. Desired-state apply (`iac`). |
| **iac** | Apply **desired state**. | IR unify/preview/apply/watch, reconcilers, Helm **adapter only**. | Merge queue (`pipeline`). Business sagas (`workflow`). Helm/Tofu as source. |
| **billing** | Charge for **cloud use**. | Meter, rate, invoice, tax on **platform SKUs**, FinOps attribution. | Card rails as a bank (`payments` product). Universal accounting books (`ledger` product). |
| **marketplace** | Third-party **modules** on the cloud. | Signed plugins, permission envelope, SKU **engine**. | Generated price list (**`build/` view**). The apps themselves. |
| **compliance** | Evidence **engine** for the platform. | Load packs, data-class registry, evidence export. | Merkle log (`audit`). Jurisdiction **data** (`packs/`). Cloned DPIA files. |
| **notify** | Transactional **send** (SES / SNS / FCM). | Send email/SMS/push; DKIM/SPF as this engine. | Inbox/Meet/Messenger/calendar/contact-center (apps later). Emergency clinical. Current `comms/` tree (purged). |
| **packs/** (data, not a cap) | Jurisdiction overlay on the cloud. | Cedar+ontology+constraints per region. | Copied into each cap. EU as world floor. |

**Not cloud-provider capabilities:** `payments` (money movement **product**), `ledger` (books **product**), `app/*` (SaaS), **`console/`** (D-16 — discarded pilot, not a shell engine). They must not live in `billing/` or in a cloud cap `core/`. If they ship, they are **product** placement (`app/` if 2+ cloud caps, or a later §7 **product** engine — not this cloud set).

**Dogfood.** First-party `app/<product>/` is a **tenant of this cloud** (Oyatie as tenant #0). It calls the **same** gateway, iam, policy Check, cells, storage, data, bus, workflow, billing meters, and packs as any customer. No private Helm tree, no in-process PDP that customers cannot call, no `iam/**` shortcut around `policy/`, no cap `core/` that exists only for our SaaS.

**MUST (cloud vs SaaS)**

- **achieves:** capability charters cannot absorb HR/Slack/SAP products; apps cannot grow a shadow cloud.
- **origin:** registry mixed `cloud/` + `oya/` seeds; drafts delivered IPs as if each cap were a SaaS; private paths would make dogfood fake.
- **rule:** D-15 in/out is the cloud-provider charter; apps only in `app/`; apps consume public/cloud APIs only; no cap `core/` owns a vertical product.
- **ensure:** new crates match in-scope; PRs that put payroll/studio-SaaS in `workflow/core` or add a private control plane for `app/` fail review.
- **overturn_when:** a §7 ADR explicitly adds a product engine **outside** this cloud set, with five fields.

**MUST (cloud lives in caps)**

- **achieves:** one place for each cloud concern; `cloud/` and `specs/cloud-*` cannot return.
- **origin:** `cloud/` was emptied; the cloud leaked into JSON specs and nested `oya-*` / `cloud-*` leftover dirs inside caps.
- **rule:** a cloud-provider engine occupies exactly one registered capability’s `core/`; sold single-cap surface is `facade/`; 2+ is `app/`; repo root does not hold IaaS dumps.
- **ensure:** new engines get a registry row or a face, never `cloud/` or `libs/`.
- **overturn_when:** a §7 split/merge ADR with five fields lands same-wave.

### D-16 — `console/` is not a capability; discard the pilot

The tree at `console/` is **ops-dashboard-control-center**: a Wave-15 internal
ops dashboard (incident-command, tenant-admin, pack-author, on-call-handoff, …).
That is a **pilot product**, not a cloud engine.

Hyperscaler analog: AWS Console / GCP Console is a **first-party web app** that
dogfoods IAM and the public APIs. It is not EC2. It is not a closed-registry row.
ADR-0615 “console SHELL is the substrate; ops leaves → `app/ops-console/`” mixed
the shell with the pilot and invented a parking lot.

**Burn.** `git rm -r console/`. No empty `console/` scaffold. Do **not** park the
pilot in `app/ops-console/` or `app/console/`. Token broker is **iam**. Operator
mutations stay on each capability’s **facade** (Cedar). A future tenant/operator
UI is `app/` **after** the apps discussion, as tenant #0, same public APIs.

**MUST (no console capability)**

- **achieves:** a dashboard pilot cannot occupy a cloud-provider root.
- **origin:** `console/` README is ops-dashboard-control-center; registry called it
  the Leptos shell / token broker.
- **rule:** `console/` is absent from the tree and the closed registry; no empty
  scaffold; no `app/ops-console` rehome of this pilot.
- **ensure:** membership/hygiene allowlists do not re-admit `console/`; new UI
  waits for the apps discussion.
- **overturn_when:** a §7 ADR adds a real sold shell as `app/` (or a new cap) with
  five fields same-wave — not a resurrection of this directory.

### D-17 — Presubmit is cargo plus a short admission set; census gates delete

`ci/facade/*` and `governance/check/*` grew a second product: JSON policy files
with `min_expected_*`, `expected_total`, frozen path lists, FNV signatures, and
Helm/OpenAPI/OpenSLO corpora. That is observation-hardcode, not TAP. Trimming one
stale path (e.g. `cedar-deploy-parity-policy.json`) keeps the anti-pattern.

**Keep (admission).** Workflow cargo fmt + nextest. Then only:

| Engine | Why it is TAP |
|---|---|
| `generated-artifact-freshness` / `generated-artifact-policy` | Generated faces are not merge surfaces. |
| `license-policy` / `supply-chain-audit` | Legal + advisory activation. |
| `automation-language-policy` | Rust-first automation. |
| `repo-root-hygiene` | D-8 closed root names. |
| `module-membership` | Closed capability registry. |
| `endpoint-authorization-coverage` | New HTTP control plane is fail-closed authz. |
| `graphql-usage-policy` | No GraphQL without a reversing ADR. |
| `crypto-backend-policy` | No `ring` activation. |
| `affected-target-set` | Graph + live hub-exclusivity binary the workflow runs. |
| `no-template-stamping` | D-8 stamped docs. |

Support crates those engines import (`planning-projection`,
`cross-artifact-agreement`, `path-resolver`, `corpus-census`, Tide/webhook,
`dependency-automation`, toolchain proposer) stay as **libraries/tools**, not as
extra required predicates. The accounting-registry / scm-facts producer is
**deleted with the census fleet**; generated-face materialize is cargo-lock +
diff-policy until a producer that does not fan-in census gates exists.

**Delete.** Every other `ci/facade/*` and `governance/check/*` crate, including
`policy-deploy-parity` (Helm Cedar census), `scan-root-liveness` (`min_expected_roots`),
`corpus-index-coverage`, `product-protocol-policy` (`expected_total`),
`crate-catalog-coverage`, `slo-coverage`, `helm-chart-shape`, `gitops-chart-license`,
`baseline-ratchet`, `gate-self-conformance`, docs/glossary/runbook/RACI/OpenAPI
coverage, `authz-tier-discipline` frozen leak **counts**, `event-schema-versioning`
`min_*` floors. Do not re-freeze numbers to make a delete green.

**MUST (no census gates)**

- **achieves:** merge admission cannot be a hand-maintained observation of the tree.
- **origin:** GH #16 Helm Cedar parity; `min_expected_roots`; `expected_total` pins;
  two-sided frozen path lists that must be edited on every `git rm`.
- **rule:** a gate is born-blocking on a **pattern** (new path, new unauth route,
  new license, new GraphQL crate) or it does not exist. Path/count freeze JSON is
  not a gate.
- **ensure:** new `ci/facade/*` crates match the keep table or they are not merged.
- **overturn_when:** a five-field ADR adds one engine that evaluates IR/Cedar/cargo
  graph without a frozen corpus.

### D-18 — `pipeline/` product vs GHA operator; purge `workflow/`; `.github/scripts` glue

**Product.** Analog: Google **TAP** (internal graph-aware execute) + **Cloud Build**
(sold). Destination slug is **`pipeline/`** (rename from `ci/` — git mv later, not
this change). Core: execute a graph, queue, controller. GitHub is an **adapter**,
not the engine. Tenant pipelines (including first-party apps as tenant #0) run
**here**. D-17 crates still under `ci/facade/` are **operator admission**, not
`pipeline/` core — they move to `governance/` or die with GHA; they do not become
the TAP product.

**Operator GHA.** `.github/workflows` is a **temporary** merge path for *this*
monorepo. Completely **disjoint** from `pipeline/`: no YAML copied into cap
`core/`; no claim that GHA **is** Cloud Build. Cutover is when `pipeline/` can
run this repo’s nextest graph; until then GHA stays.

**Glue languages.** Rust-first still owns `scripts/`, `tools/`, `infra/`, and every
capability. **Exception:** self-contained files under **`.github/scripts/`** may be
shell, JS/MJS, Python, **Go, or any other language**. That keeps GHA glue out of
the product tree.

Self-contained means: no repo-root `go.mod` / `package.json` / `requirements.txt`;
if a module file exists it lives **only** under `.github/scripts/`; not a Cargo
workspace member; no shared crate with a capability; no `npm install` / `pip install`
/ `go get` on the presubmit path (pin a binary or a file in that directory). The
exception **dies with GHA**. It is not a license to grow `scripts/` or `tools/`.

**`bus/`.** `messaging/` collides with human chat. Destination slug is **`bus/`**.
git mv later.

**`notify/` not `comms/`.** `comms/core` is mailbox, Meet, messenger, calendar,
address book — Workspace/Slack, not a cloud send API. Those are **later `app/`**
products (dogfood tenant #0). The cloud primitive is transactional **send**
(SES/SNS/FCM): destination slug **`notify/`**. **Purge** `comms/`. Do not
strangler mail/meet/messenger into `app/` from this dump. Rewrite `notify/` from
the send charter.

**`workflow/`.** Current tree is n8n/SaaS/bus/forms/tasks, not Step Functions.
**Purge** (`git rm -r workflow/`). Keep the **registry row** as the rewrite
destination. No empty scaffold. Do **not** strangler event-bus crates into
`bus/` from this junk — `messaging/` (→ `bus/`) already has kernels. Rewrite the
saga engine from the D-15 charter (proto/H3, studio as facade). Forms/tasks wait
for the apps discussion.

**MUST (GHA ≠ pipeline product)**

- **achieves:** sold TAP/Cloud Build cannot be this repo’s Actions YAML.
- **origin:** `ci/` mixed census gates, GHA glue, and the delivery-fabric product.
- **rule:** product slug is `pipeline/`; `.github/` is temporary operator merge,
  disjoint; rust-first does not scan `.github/scripts/`; `workflow/` implementation
  is gone pending rewrite.
- **ensure:** no new GHA glue outside `.github/scripts/` and workflow YAML; no
  `workflow/` dump resurrection; git mv `ci/` → `pipeline/` does not carry D-17
  gates into product `core/`.
- **overturn_when:** `pipeline/` runs this repo and GHA is deleted same-wave, or a
  five-field ADR names a different sold slug.

## Rejected alternatives

- AWS EKS etcd journal as our store.
- CUE+Timoni or Haskell as EaC wrap.
- Public JSON/REST as the destination codec.
- Zanzibar global tuple replica.
- EU GDPR as the sole compliance floor.
- Mega EaC orchestrator / remote PDP on the hit path.
- Sync Merkle on every `Check`.
- Cap-root `catalog.yaml` / `manifest.json` as allowlist debt.
- Cap-root `policy/` for Cedar files (collides with the `policy/` engine) or dual `cedar/`+`policy/` children.
- Rehoming AUDIT-FINDINGS, IPs, scorecards, DPIA essays into `docs/` instead of deleting.
- Cap-root `contracts/` or root `contracts/*.yaml` as IDL SSOT.
- Hand-authored OpenSLO (clones or “unique”) as SLO source of truth.
- Standing REST/JSON transcode “until SDK is ready” (that is dual-stack debt).
- Hand-authored OpenSLO as a W0 source of truth.
- Branding the merge-blocking CI `oya-ci-*` or adding one required check per capability.
- Keeping `specs/` as a JSON org-law tree (even “thin”).
- Keeping `HANDOFF.md` as a fourth root hub.
- Keeping `console/` as a shell/token-broker capability, or rehoming the
  ops-dashboard pilot to `app/ops-console/`.
- Trimming `cedar-deploy-parity-policy.json` (or any census JSON) instead of
  deleting the gate. Hand-maintained path lists, FNV signatures, `min_expected_*`,
  and `expected_total` are the same anti-pattern.
- `cargo check` as a CI job (nextest already compiles). `cargo test` (libtest) as
  the merge runner. Windows/macOS per-PR smoke. `rustfmt` CLI instead of `cargo fmt`.
- `cargo test --doc` as a vacant merge job; `cargo-semver-checks` on internal crates;
  network `cargo-audit` on presubmit; `cargo-dist` GitHub Release CLIs; whole-repo
  `cargo-mutants` nightly; a cron that runs `cargo update` into `Cargo.lock`; a
  local hook that runs clippy `--all-targets` and claims “<5 seconds.”
- **mold** on GitHub presubmit. Rust 1.90+ already uses bundled `rust-lld` on
  `x86_64-unknown-linux-gnu`; CI already has debuginfo off. Mold is a local Linux
  optional, not a merge-path pin.
- Treating `.github/workflows` as the `pipeline/` product, or copying GHA YAML
  into a capability `core/`.
- Strangler-moving `workflow/` event-bus/saas/forms into `bus/` or `app/`
  instead of purge+rewrite.
- Keeping the slug `messaging/` (collides with `comms`).
- Allowing shell/Python/Go under `scripts/` or `tools/` because `.github/scripts`
  is allowed. The exception is prefix-exact.
- Keeping `comms/` as a cloud cap (mailbox/Meet/messenger/calendar). Those are
  apps. Cloud send is `notify/`.

## Appendix — considerations (not implement authority)

Record of why D-8/D-9/D-10 landed. Not additional MUST.

- **`specs/`:** Live tree ~376 files / ~359 JSON. Gates load `product-protocol-contract.json`
  (REST required, Connect forbidden) and `per-microservice-flat-layout.json`
  (`microservices/` + required PRDs). That contradicts D-3/D-4/D-8. A catch-all
  `specs/` refilled once; shrinking it still leaves a magnet. Hyperscaler: policy
  next to the evaluator; ADRs for decisions. **Do not copy `specs/` into cap/app.**
- **`cedar/` vs cap-root `policy/`:** Inventory had ~359 files under `<cap>/policy/`.
  That collides with the `policy/` **capability** (PDP). Engine vs data names must
  not collide (`observability/` vs `observability/slos/`). Platform Cedar lives in
  the engine; caps hold unique fragments only. Stamped `tenant-scope.cedar` is the
  runbook-clone bug.
- **OpenSLO:** “Controller eats YAML” is how Helm survived. SLO source is IR;
  OpenSLO YAML is generated. Hand files (clone or unique) are debt.
- **REST transcode:** Google ESP still JSON-encodes proto. We still reject a
  **standing** dual API. Leftover REST is deleted; temporary caller red is hygiene.
- **`HANDOFF.md`:** Thin redirect (2026-06-08 exception). Content is a read-order
  already in `AGENTS.md`/`CLAUDE.md`. Deleted so a fourth hub cannot accrete status.
  **Pending founder:** the sentence “a higher ADR number does not override an earlier
  Accepted ADR by itself; only explicit `amends`/`supersedes`” — admit into this ADR
  if that is still wanted as law (it is not copied here as MUST).
- **presubmit name:** TAP/presubmit vs postsubmit. Not `oya-ci-required`.
- **One context vs per-cap CI:** Central **admission** is hyperscaler; central
  **full-repo JSON census** is the conflict source. Per-cap **required** checks are
  the skip-fail anti-pattern.
- **etcd / AWS journal:** EKS journal is closed, etcd-API-preserving, mega-cluster.
  We cell-shard; steal log-vs-memory, not the binary.
- **EU world-floor:** KR CSAP/본인인증/e-tax is not a GDPR subset. Packs overlay.
