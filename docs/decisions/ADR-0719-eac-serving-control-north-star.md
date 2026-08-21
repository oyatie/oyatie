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
PQC + ECH at the edge (ADR-0354). Public gRPC-Web is not the console protocol. FlatBuffers
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
| **presubmit** | every PR / merge-queue group | Affected cargo graph: fmt, clippy, tests + dependents; D-8 path-set (new junk red); license; rust-first; generated-not-hand-edited; Cedar compile of **touched** `cedar/` | **Yes.** One required context. |
| **postsubmit** | merge to `dev` | Full workspace (or remainder not proven on the affected set); start of promotion **into staging** only via the promotion pipeline | No (already merged). Failure is a **revert/block-next** signal, not a second required PR check. |
| **nightly** | schedule | arm64 (D88-amend), fuzz, long E2E, soak | No |
| **weekly** | schedule | buck2 `build //...` honesty smoke (ADR-0716); hermetic graph | No |
| **promotion** | explicit rung | `dev` → `staging` → `canary` → `production`; predecessor check | N/A (branch protection on the rung) |
| **release** | train (interview D63) | Bundle what’s on the promotion rung; **release builds** (`cargo build --release` / buck2) on **CD**, not presubmit | N/A |

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
| Pub/Sub / Pulsar | **messaging** | core: bus, idempotency, outbox. Schema with the bus, not `specs/proto/`. |
| SageMaker / internal AI | **intelligence** | core: model/agent substrate. AI Act registry is pack + this cap, not `capabilities/*.yaml` essays. |
| Step Functions / Composer | **workflow** | core: engine. Studio is facade. Business sagas, **not** deploy orchestrator (D-1). |
| CodeBuild / TAP / merge queue | **ci** | core: presubmit engines, controller, queue. Delivery fabric as product (ADR-0548). Gate **policy next to the gate**, not `specs/`. |
| CloudFormation / Config reconciler | **iac** | core: IR unifier + reconcilers. `<cap>/iac/` is **this** cap’s desired state; `iac/` the cap owns the **engine**. |
| Billing / Cost Explorer | **billing** | core: meter, rate, invoice, tax, FinOps. Sold-ness, not a drawer. |
| Marketplace | **marketplace** | core: plugins, signed modules, SKU **engine**. Generated sell-catalog view is `build/`. |
| Console | **console** | core: shell, token broker, nav. 2+ cap dashboards → `app/`. |
| Artifact / evidence packs | **compliance** | core: pack evidence, data-class registry. Consumes **audit**. Not the Merkle log. |
| SES / Chat / Meet | **comms** | core: mail, messenger, meet, notify, contact-center **engines**. End-user “Workspace” product → `app/` when it wires 2+. |
| AppConfig / Feature flags | **flags** | core: flags, kill switches. Pack-gated overrides. |

**Meta (not sold as a tenant API, still in-repo):** `kernel/` rung 0; `os/` node OS; `base/` (≥3 caps, below all); `build/` toolchains/images; `third-party/` vendored; `governance/` registry + check crates (off the runtime ladder).

**`app/<product>/`:** composition only (hr, payroll, calendar, community, …). Wires 2+ of the table. **Does not** grow a 25th cloud engine. Interview `payments/` / `ledger/` become capabilities only via ADR-0562 §7 — they are **not** sneaked into `app/` or `billing/` as a junk drawer.

**MUST (cloud lives in caps)**

- **achieves:** one place for each cloud concern; `cloud/` and `specs/cloud-*` cannot return.
- **origin:** `cloud/` was emptied; the cloud leaked into JSON specs and nested `oya-*` / `cloud-*` leftover dirs inside caps.
- **rule:** a cloud-provider engine occupies exactly one registered capability’s `core/`; sold single-cap surface is `facade/`; 2+ is `app/`; repo root does not hold IaaS dumps.
- **ensure:** new engines get a registry row or a face, never `cloud/` or `libs/`.
- **overturn_when:** a §7 split/merge ADR with five fields lands same-wave.

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
