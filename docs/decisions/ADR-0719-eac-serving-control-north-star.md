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
amends: [ADR-0701, ADR-0702, ADR-0704, ADR-0705, ADR-0708, ADR-0716]
amended_by: []
depends_on: [ADR-0615, ADR-0701, ADR-0702, ADR-0704, ADR-0705]
related: [ADR-0243, ADR-0280, ADR-0354, ADR-0049]
milestone: W0
deliverables:
  - id: ADR-0719-D1
    description: "Record serving-path vs control-path split as live law: 10^8-class user/Check traffic is in-cell RAM snapshots; writes, IR apply, packs, and cluster objects are a journaled control plane. k8s/ports: etcd is the v1 adapter, owned journal is the destination adapter; cluster-object API is never the etcd API. Time is our TrueTime: interval API always; cell clock port; data consumes it; storage does not use clock as identity."
    exit_criteria: "This ADR is Accepted; CLAUDE.md live apex list cites it; no implement PR treats etcd or a Kubernetes object store as the Check/IR/tuple store."
    verified_by: "presubmit"
  - id: ADR-0719-D2
    description: "Record EaC as one protobuf IR plus per-plane reconcilers behind one Connect/H3 gateway contract (N cell frontends). No wrap language. JSON is not a product codec. Gateway TLS port crates: hybrid ML-KEM public default, classical dying, ECH in tree. Visibility is after trusted terminate + audit export, not on-path QUIC MITM. No Istio. No standing gRPC. No firewall/ cap."
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
    description: "The merge-blocking CI context is named presubmit (Google TAP-shaped). New workflow and required-context names do not use an oyatie- prefix. Today's presubmit string is a rename target, not the destination name."
    exit_criteria: "This ADR uses presubmit as verified_by; no new ADR or workflow is named ci-*; the live GitHub required context rename is a follow-through PR that updates branch protection in the same change."
    verified_by: "presubmit"
  - id: ADR-0719-D10
    description: "Hyperscaler pipeline names: presubmit (merge-blocking, graph-aware), postsubmit (on merge to dev), nightly, weekly, promotion rungs dev-staging-canary-production, release train bundling. One required context. No oyatie- prefix. No per-capability required GitHub checks."
    exit_criteria: "This ADR defines those cadences; new workflows use those names; presubmit remains a rename target with branch protection in the same follow-through change."
    verified_by: "presubmit"
  - id: ADR-0719-D11
    description: "Cloud-provider placement: the registered capabilities ARE the cloud. Repo root holds only directory names plus meta (build/third-party; base/ only when admitted) and app/. No kernel/ or os/ rungs — fleet node is Linux + compute agent, not Talos/kube. Each capability owns one engine (core), ports, adapters, facade. 2+ compose in app/. No cloud/ folder."
    exit_criteria: "This table is the placement reading; new engines go in an existing cap or a §7 registry split, not a new root dump; app/ is composition only."
    verified_by: "presubmit"
  - id: ADR-0719-D13
    description: "Fleet is stripped-minimum Linux on Cloud Hypervisor and/or Firecracker. Asterinas/Hermit are not plant today; reconsider only with a measured five-field ADR. Not Talos/kube as the cloud OS. Delete kernel/ and os/."
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
    description: "pipeline/ is one execute engine: TAP internally (tenant #0) and Cloud Build sold are two facades. GHA disjoint adapter. JSON/governance check fleets are not the product. ci/ is a retired path."
    exit_criteria: "ADR tables use pipeline/, bus/, notify/; workflow/ and comms/ trees absent; rust-first exclude_prefixes includes .github/scripts/; GHA YAML is not a face of pipeline/."
    verified_by: "presubmit"
  - id: ADR-0719-D19
    description: "Every repo-root name is DO or DON'T, and HAVE or HAVE NOT: DONE, BUILD, REMOVE, or STAY GONE. No new cloud-* crates. REMOVE is delete/rewrite in charter, not a move to another cap."
    exit_criteria: "New crates are DO+HAVE-NOT (BUILD) or DO+HAVE (DONE); PRs that add DON'T names or rehome REMOVE dumps fail review."
    verified_by: "presubmit"
  - id: ADR-0719-D20
    description: "Charter reconciliation (founder default A, 2026-08-22): two compute reconcilers not k8s-on-compute; ontology out of data/core; intelligence is Vertex not copilot; price is marketplace+billing not build/; iam consumes federation; Drive/PACS/Meet out of storage (Drive is app/drive over a storage adapter, D-23); marketplace plugins+SKU only; gateway is PEP; meters are usage events; port-engine frozen; quota split; DNS/CDN split."
    exit_criteria: "D-11/D-14/D-15/D-19 and registry charters match D-20; no new crate uses k8s-on-compute, ontology-in-data-core, gateway Cedar engine, or build/ price view."
    verified_by: "presubmit"
  - id: ADR-0719-D21
    description: "Palantir Foundry is the product (app/foundry). Ontology + Pages + Grid + Workshop + Manager + Pipeline Builder UX live there, not in data/ and not in intelligence/. Intelligence foundry/RAG is dead. D41 retired notes/slides/sites/office only — docs/sheets are Foundry primitives, not retired."
    exit_criteria: "data/ charter has no ontology kernel; no foundry/ capability root; no intelligence foundry surface; D41 list is notes/slides/sites/office/translate only."
    verified_by: "presubmit"
  - id: ADR-0719-D22
    description: "Apps 2x2: one launchpad; Foundry module; v1 People=hr+payroll; v1 Finance=accounting+payments+ledger; community shrunk; messenger dual-context one dir; no SAP ghost dirs."
    exit_criteria: "No empty app dirs for dropped modules or registry ghosts; app/ledger is the posting product not a cap; community has no SecureDrop v1; no app/social."
    verified_by: "presubmit"
  - id: ADR-0719-D23
    description: "Apps are tenants of the cloud. First-party apps consume cloud SKUs only through adapters (same as external tenants). Foundry v1 is the full suite and persists via data/storage/pipeline adapters. Console dumps deleted. Calendar is embeddable. Mailbox vs notify. Messenger Slack+Discord dual. app/drive over storage adapter. Payments lowest v1. Community Blind+Reddit dual. One pack engine; app overlay slices."
    exit_criteria: "No tenant-admin-console dump; no app crate depends on cloud core in-process; D-22 table lists drive; notify is multi-channel send not mailbox; packs schema allows app.<product> slices without a second pack reconciler."
    verified_by: "presubmit"
  - id: ADR-0719-D24
    description: "OVERRULE D-23 pack-as-one-file and Pipeline-Builder-to-TAP. Packs: per-cap/app overlay content + thin central install authority. Runtime state is not git plaintext; v1 SQLite adapters then data/storage/on-prem. Cloud pipeline/ is CI/CD only. Mailbox is a port (IMAP/JMAP/SMTP/Connect adapters). One blob port for Drive/Foundry/mail; on-prem storage is an adapter."
    exit_criteria: "D-23 Foundry settle table does not send Pipeline Builder to pipeline/; packs/ is install authority not overlay novels; app crates persist through ports with a SQLite adapter; Drive/Foundry bytes share one blob port."
    verified_by: "presubmit"
  - id: ADR-0719-D25
    description: "App business logic lives in core; IO only through ports. Cloud SKUs are one adapter family among others (SQLite v1, S3, Postgres, IMAP, Stripe, on-prem). End of a cloud capability must not end the app without a rewrite. Not HA during our outage."
    exit_criteria: "App crates: domain/use-case in core with no cloud/SQLite/HTTP types; one port per substrate need; adapters for our cloud and for commodity substitutes; no path-dep on cloud core/ports."
    verified_by: "presubmit"
  - id: ADR-0719-D26
    description: "REJECT a trusted-vs-untrusted tenant mode in apps. D-23/D-25 already prove the cloud (same facade) and portability (commodity adapters). First-party is an IAM principal like any other; not an app flag, VIP class, skip-PDP, or in-process core. Do not add a mechanism we will regret."
    exit_criteria: "No TrustedTenant/cfg(trusted)/first-party quota class in app cores; no skip-PDP; no second cloud API; adapter injection and IAM principals remain the only knobs."
    verified_by: "presubmit"
  - id: ADR-0719-D27
    description: "g3doc split: owner-local engineering docs live in <cap>/docs and app/<p>/docs (closed inner grammar). Repo-root docs/ is thin (operating contract, live 07xx ADRs, standards). No ADR copies, no catalog/IP/scorecard resurrection, no mass-move of the old wiki."
    exit_criteria: "D-8 allowlists admit docs/ as a cap/app extra with the inner grammar; root docs/decisions remains the unique ADR home; no IPs/scorecards under docs/."
    verified_by: "presubmit"
  - id: ADR-0719-D28
    description: "Cross-owner bindings are ports+adapters. Unagreed ports are path-visible (ports/draft/) and cheap to rename. A second owner depending on a shape forces reconcile onto one agreed name on the provider (owner-port grammar + proto v1) via escalated review. No contracts/ root and no libs/ports dump."
    exit_criteria: "Other owners cannot path-dep ports/draft/; agreed shared shapes live on the provider as owner-port; proto packages do not ship draft names; no new contracts/ or libs/ tree."
    verified_by: "presubmit"
  - id: ADR-0719-D29
    description: "Amendment jurisdiction: owner OWNERS may amend content inside their cap/app root. They must not change the canonical children, inner crate layout, or crate grammar. Shared contracts, sold facades, and repo-root law require escalated review."
    exit_criteria: "PRs that touch agreed ports/proto/facade or repo-root law name the other owners + architecture; local-only docs/draft/core changes stay on owner OWNERS; new children, faces, or plan/tasks stay rejected."
    verified_by: "presubmit"
  - id: ADR-0719-D30
    description: "Names and inner files follow established Cargo + google3 + AIP conventions: RFC 430/940, directory leaf = last grammar token, package name = full owner-port grammar, proto package directory = AIP-191. No invented domain/use_case taxonomy. Structure (D-8) does not change per team."
    exit_criteria: "New crates: kebab package, omitted [lib].name, dir leaf matches last token, src/lib.rs or src/main.rs, snake_case modules, proto under facade/proto matching package.v1; no -rs/-rust/oyatie-/cloud- prefixes; no domain/ or use_case/ as required folders."
    verified_by: "presubmit"
  - id: ADR-0719-D31
    description: "Default implement/review worker runs in an ephemeral out-of-tree git worktree. Writable surface is only the dispatched owner (one cap or one app/<p>/). OS write-jail when available. Sparse cone of that owner plus declared read-only inputs. Agent cannot expand its own sandbox. Not a full-repo hide (Cargo + D-28). Not a VCS ratchet product."
    exit_criteria: "Worker dispatches name the owner path; worktree is not the human clone; writes outside that owner fail or are review-blocking; D-29 escalated lanes name extra writable cones; worktree is removed when the lane ends; no new claim/verify ceremony."
    verified_by: "presubmit"
  - id: ADR-0719-D32
    description: "Parallelism unit is the leaf crate, not the cargo workspace and not the whole cap. Subagents get separate worktrees with disjoint crate cones. Local proof is buck2 on those targets (ADR-0716). Cargo.lock / root Cargo.toml are single-writer. CI cargo --workspace is the linearized merge proof, not N-way local cargo."
    exit_criteria: "Dispatches name crate paths when splitting an owner; two live lanes do not share a worktree or Cargo.lock writes; local agent verify is buck2; cargo update/generate-lockfile is not used in parallel owner sandboxes; path-only PRs do not touch Cargo.lock."
    verified_by: "presubmit"
  - id: ADR-0719-D33
    description: "Structural Mutation Separation: reorg (git mv/rm, D-8 children, crate grammar, workspace members, lockfile, faces) is a different class from behavioral edits. Do not mix in one lane. After the structure wave, implement lanes are content-only inside frozen crates."
    exit_criteria: "PRs are either structural (layout/lock/members/faces) or behavioral (crate src/tests/docs), not both; implement dispatches do not git mv trees or edit root Cargo.toml; D-8 shape stays frozen mid-feature."
    verified_by: "presubmit"
  - id: ADR-0719-D34
    description: "Local N-way uses shared read-only build cache (buck2 CAS/disk cache), cargo --offline --locked if cargo is used at all, and the existing buck2 + rust-analyzer graphs for dispatch. Reject shared CARGO_TARGET_DIR lock-bypass, per-agent lockfiles, and an Aggregated AST Patch product."
    exit_criteria: "Agent local verify does not rewrite Cargo.lock or take a shared cargo target lock; cache is content-addressed and trusted-writer; dispatcher consults build graph (buck2) and crate graph (metadata/r-a); no new AST-merge service."
    verified_by: "presubmit"
  - id: ADR-0719-D35
    description: "Hand-written non-exempt files are at most 300 lines. New or touched files over 300 fail. Existing over-budget files are split in dedicated lanes when that crate is worked, not one repo-wide dump. Exempt: live ADRs, PRD.md, AGENTS/CLAUDE, generated, lockfiles, third-party."
    exit_criteria: "Presubmit pattern check on touched non-exempt paths; no expected_total of file counts; splits stay inside the crate (D-32); generated proto/lock/vendor ignored."
    verified_by: "presubmit"
  - id: ADR-0719-D36
    description: "Live law is one monolithic ADR per apex topic (this file) plus app PRD.md. That document is the iterating checklist: PRD, spec, decisions, contradictions, staleness. Do not split D-n into files and do not recreate specs/ or plan/ trees."
    exit_criteria: "No new specs/ or plan/ roots; new decisions amend this ADR or a live 07xx apex in place; PRD.md remains the app-level monolith; owner docs/ stay under the 300-line cap."
    verified_by: "presubmit"
  - id: ADR-0719-D37
    description: "Shared docs/config/json/yaml/toml are not split like .rs. Keep them minimal. Implement agents must not in-place edit the denylist; additive changes are uuid-named fragments. Mechanical fold is one serial step on the receiving branch (pre-commit/merge_group), not per-worktree. Prose ADRs stay single-writer. Cargo.lock is regenerated once after fold, not fragment-merged."
    exit_criteria: "Implement PRs that touch root Cargo.toml/lock/toolchain/deny/rustfmt/AGENTS/live ADRs in place fail unless a structural lane; additive member/config lands as a unique fragment; fold engine is Rust; no specs/ yaml farm; lockfile diffs only from the fold step."
    verified_by: "presubmit"
  - id: ADR-0719-D38
    description: "Worktrees isolate indexes, not integration. Lanes integrate star-shaped onto dev via merge_group only. Never mesh-merge live worktrees. A path conflict quarantines writers to that identity only; other disjoint lanes continue. Do not merge origin/dev into the lane (rebase/queue replay)."
    exit_criteria: "No agent merges one implementation worktree into another; PRs target dev; conflict on a file/Item pauses only that identity; other files in the same crate continue; merge_group is the combination test."
    verified_by: "presubmit"
  - id: ADR-0719-D39
    description: "OVERRULE crate-lock and uuid-delta VCS. Commute identity is a unique git path at module/item (or markdown block-file) grain. Parent membership (mod, workspace members) is a pure function of the directory (Cargo/Buck globs + generated mod list), not a hand-edited list and not .delta files. N agents on one crate is allowed iff they own disjoint files/items. Same Item two writers is still refused at assign — not a crate mutex, not poll-until-unlock."
    exit_criteria: "Workspace members use closed globs over D-8 faces so adding a crate dir does not edit root Cargo.toml; crate roots do not grow hand-maintained mod lists for every sibling file; occupancy is open PR paths at file/item grain; no Cargo.toml.d uuid product required for the common add; no whole-crate lock."
    verified_by: "presubmit"
  - id: ADR-0719-D40
    description: "Occupancy is a path-set, not a cap/app session lock. Write/edit/delete occupy the path; git mv occupies {old,new}. Mixed N ops commute iff path-sets are disjoint. Cross-cap work is named extra paths (D-29), not a crate lock. Plan/ADR is one path and therefore commutes with all src. Mechanical LSC (same symbol, many files) is one lane or file-sharded, not N overlapping edits. Cap cone remains default sandbox blast-radius, not merge necessity."
    exit_criteria: "Dispatch names a path-set (including rename pairs); overlapping path-sets are not spawned; sessions may list paths in more than one cap when escalated; plan PRs touch ADR/PRD only; LSC does not share files with feature lanes; no poll-lock of a capability."
    verified_by: "presubmit"
  - id: ADR-0719-D41
    description: "YAGNI cut: git conflict is impossible iff commits do not share a path. Keep parent indexes STABLE (workspace member globs; crate module list from compile-time directory scan, not a committed generated file). Jail writes to the dispatched files. PR to dev; merge_group. No occupancy service, no uuid fragments, no crate/cap mutex. Uncoordinated same-path create is a tiny rebase, like TAP mid-air — do not invent unique-name VCS to prevent it."
    exit_criteria: "Implement PRs do not edit root members lists or hand-maintained mod inventories; new Item is a new unique file; lib.rs membership line is stable; merge_group is the combination test; no Cargo.toml.d product; no occupancy JSON."
    verified_by: "presubmit"
  - id: ADR-0719-D42
    description: "Cross-harness (Grok, Claude, Codex, Cursor, Antigravity, …): only git + draft PR on origin/dev + presubmit is portable. Do not rely on a vendor sandbox, worktree, or dispatcher. Instruction must live on every session-loaded hub. Draft PR is occupancy. Same-path create is rebase, not a lock."
    exit_criteria: "AGENTS.md and CLAUDE.md state the same sequence; implement PRs still fail denylist regardless of which harness authored them; no harness-specific occupancy tool required."
    verified_by: "presubmit"
  - id: ADR-0719-D43
    description: "N-parallel delivery loop is path-set PRs, not a task-board poll. Launcher (not the agent) derives unique output paths from ADR/PRD and spawns. Each PR: red tests on that path, implement, de-slop, coverage on that crate, pipeline review only if pipeline files are in the path-set. presubmit green is required; if red, process error on that PR not a factory stop. merge_group then squash. Stages commute across PRs."
    exit_criteria: "No tasks/ JSON board; no agent loop on gh pr list; overlapping path-sets fail presubmit; CI-metric review only when .github/ or pipeline/ is touched; local pre-push is fmt-on-touched not workspace nextest."
    verified_by: "presubmit"
  - id: ADR-0719-D44
    description: "Client need is received by the human operator plus orchestrator. Interview + research against existing docs/tree + verification produce an ephemeral artifact package. Ambiguous or wrong needs fail closed (NeedClarification / Rejected). The package hands off to Product (app/) XOR Program (capability). Raw client text never reaches implement. The package is not written under plan/ or tasks/."
    exit_criteria: "No implement hop is admitted from an unverified prompt; mixed app+capability packages fail; dump-root requests reject; Product vs Program is a function of target paths."
    verified_by: "presubmit"
  - id: ADR-0719-D45
    description: "OVERRULE D-43 single-agent walk of stages. Occupancy remains one draft PR path-set (D-42). Roles are a DAG with fan-out (implement complete unblocks review, coverage, security, docs together). Orchestrator publishes ready hops; it does not spawn agents and must not fold N ready hops of a role onto one worker. Each hop binds a fresh agent. Implementer finishing a slice is free for the next disjoint slice immediately."
    exit_criteria: "Ready-hop cardinality for a role equals disjoint schedulable slices in that role; reused agent ids fail; completing Implement does not wait for PrBabysit before another Implement hop elsewhere; no long-running implementer looping disjoint work."
    verified_by: "presubmit"
  - id: ADR-0719-D46
    description: "Cross-slice need (break, contract amend, agreement) is not a committee. Writer without occupancy of a path must not write it. Consumer files ports/draft/ or adapters/draft/ on their own path-set (commutes). Owner gets a ContractAmend hop when those owner paths are free. Presubmit/merge_group red quarantines that path-set only; other disjoint slices continue. No ticket board. No poll-lock."
    exit_criteria: "Cross-owner path writes without occupancy fail; draft ports are path-visible; owner amend is a hop on owner paths; one red PR does not stop other path-sets."
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

- **Serving path** (product RPC + `Check`, 10^8-class fleet): Maglev-class anycast
  **per cell**, then the **one** L7 gateway **contract** (Connect/H3; many
  frontends), then **in-process PEP**, then **in-process PDP** on a **cell-local
  RAM snapshot** (compiled Cedar + ReBAC shards + product cache). Lookups ≫ RPCs. RPC
  `Check` is miss / recent-zookie / cross-cell only. No sidecar hop.
- **Control path** (IR apply, pack sign, tuple write, schedule, SLO, billing close):
  journaled, sharded, 10^3–10^5 class. First-party IR may live in git. Tenant IR lives on
  the gateway API.
- **Scheduler** = kube-scheduler (pods) + cell placement (tenants). Not an EaC scheduler.
- **Orchestrator** = `workflow` for business sagas. Apply order is `iac`’s object graph.
- **Monitor** = `observability` + `audit` + `iac` drift. No `eac-monitor`.
- **Audit on serving:** two classes. Privileged / admin / payroll-approve / policy-publish
  persist evidence **before ACK**. Other `Check`s may be async/sampled. Silent drop under
  load on privileged class is **platform-forbidden**. DORA/NIS2/CSAP retention
  overlays live in packs (D-6): a pack may **raise** the privileged class, never
  lower it.

**MUST (serving store)**

- **achieves:** 10^8-class `Check` and product RPC without a consensus box on the hit path.
- **origin:** etcd/Raft and remote PDP-RPC were drawn as if they were Borg/Zanzibar.
- **rule:** serving state is cell-local memory snapshots; consensus/journal is off the hit path.
- **ensure:** no PR stores tuples, IR, or tenant documents as etcd keys or product CRDs.
- **overturn_when:** a measured serving path needs a different store AND a replacement ADR
  with five fields lands same-wave.

**Time — our TrueTime, two layers, one API.** Hyperscalers who **are** the
cloud install clocks in the DC. Software-only HLC is what you do when you
**rent** someone else’s DC. We are the operator, so the **destination plant
is GNSS + holdover atomic + PTP**. We do **not** wait for that hardware to
exist before the API exists. Google TrueTime, AWS ClockBound, and Meta
fbclock all return an **interval**; the antenna only **narrows** it.

- **API (always, from v1).** `Now() → Interval { earliest, latest }` plus
  a hybrid logical counter inside the interval (Cockroach HLC shape).
  Never a product `time.Now()` point. This **is** our TrueTime. Bound width
  is **measured from the wired adapter**, not a brochure number. Do not
  publish Spanner-class ε until GNSS/atomic is live **and** a measured
  bound exists.
- **Port (`cell/ports`).** Clock source is a port. Adapters (closed):
  `ntp` (chrony, v1 default, wide ε), `ptp_phc` (NIC / hypervisor PHC —
  AWS PHC, Azure VMIC), `gnss_atomic` (OCP Time Card + GNSS antenna +
  holdover atomic; Meta 2022 / Google GPS+atomic plant). Same API on
  every adapter. Without hardware the interval is wide; with hardware it
  is tight. Callers do not branch.
- **Not a product flag.** Adapter is **cell IR at bring-up** (like which
  NTP peers), not `flags/` and not a runtime kill-switch that silently
  shrinks ε under in-flight timestamps. Swapping NTP→PTP→GNSS is a
  **control-plane cell mutation**. In-flight intervals keep the bound they
  were issued with. `flags/` kill switches do not select the clock.
- **Physical plant (`cell/` core + adapters).** Google TrueTime = GPS +
  atomic in the DC. AWS Time Sync = satellite **and atomic clocks in each
  region**, PTP PHC, ClockBound. Azure = hypervisor PTP. Meta 2020 NTP;
  **2022 PTP** GNSS antenna + atomic Time Card (OCP) + NIC PHC + ptp4u;
  still Window of Uncertainty (~100× better commit-wait vs NTP). v1 plant
  is NTP/chrony. Skipping GNSS/PTP *forever* is not Meta/AWS/Google. Not
  a `time/` cap. Sold Time Sync is a later `cell` facade.
- **Use.** `data/` **consumes** cell `Now() → Interval`. It does not own
  a second clock. Linearizability is **cell-local** via Raft/Paxos.
  **FoundationDB versionstamps** are an OLTP **commit ordinal** inside an
  engine, not a second TrueTime and not a replacement for `Now()`.
  `storage/` identity is digest / generation / CAS key. `Last-Modified` is
  metadata, not a consistency primitive — TrueTime-on-PUT would kill the
  cheap object store. Cross-cell: no global commit timestamp. Serving
  `Check` does not commit-wait. **Commit-wait is not implied by the
  interval API.** v1 OLTP does not wait NTP ε. `data/` keeps a
  `commit_wait` **adapter crate** (IR off unless a measured ε SLO says
  wait is cheaper than restart). Deleting that crate is born-blocking.

**MUST (time: our TrueTime, ported plant)**

- **achieves:** one interval API with or without atomic clocks; plant can
  mature NTP→PTP→GNSS without a second clock product.
- **origin:** first draft treated “no hardware” as identity; Meta/AWS/Google
  as operators deploy GNSS/atomic + PTP and still expose WOU/ClockBound;
  a flags switch would change ε under live timestamps.
- **rule:** TrueTime API is always an interval; clock source is a `cell/`
  port with ntp / ptp_phc / gnss_atomic adapters; v1 is ntp; GNSS/atomic
  is destination plant not a missing API; no `time/` cap; no `flags/` clock
  switch; no global commit time; `data/` consumes the interval;
  versionstamps are commit ordinals not a second clock; `storage/` does
  not use wall time as identity; Check does not commit-wait; v1 OLTP
  does not commit-wait NTP ε; no ε claim without measured plant.
- **ensure:** new clock code implements the port; no PR requires GPS for
  v1 merge; no PR forbids the GNSS adapter; no PR adds `Now() → Instant`
  as the product clock; no PR wires clock selection through `flags/`;
  no PR puts TrueTime into object identity.
- **overturn_when:** a measured GNSS/atomic plant exists AND a five-field
  ADR publishes ε (the API does not wait for that ADR).

### D-2 — k8s / etcd / “AWS journal”

`k8s/` remains the managed-cluster product (ADR-0704 as scoped by D-13). Durable
store is **cluster objects in a cell**, served from an apiserver **watch cache**.
The store is a **port** (`k8s/ports`):

| Adapter | Role |
|---|---|
| `etcd` | v1 for **sold** kube SKU only (upstream). Cell-sized. Not the fleet store. |
| `owned_journal` | Destination. Log vs memory, no single Raft leader on the hot path, MIT/Apache, cell-local. |

The **cluster-object API is never the etcd API**. Product records, tuples, and
IR are never etcd keys (D-1). Do **not** adopt Amazon’s closed EKS journal.
Steal the split, not the binary. Do **not** rewrite etcd before cells exist
(opportunity cost: cells are the scale lever). Do **not** leave a door that
blesses etcd as destination. Deleting the `owned_journal` adapter name is
born-blocking.

Admission (VAP/CEL+PSA) remains the live rule cited from ADR-0704 / ADR-0700.
Proposed 0710-range ids are not `depends_on`.

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

**One contract, two transports.** Protobuf IDL is the SSOT. The RPC is
**Connect-class HTTP** (no gRPC trailers). Same contract north-south and
east-west. `Check` on the hit path is **in-process** (D-1) — no QUIC, no
TCP, no RPC. Cross-cell Check, control apply, and watches use Connect.

**Transport follows the plant.** DC NICs, kernel, Maglev, and in-cell
path are **TCP-optimized** (TSO/GRO/RSS, DCTCP-class). Hyperscalers put
QUIC on the **public door** (GFE/Cloudflare); east-west stays TCP
(gRPC/H2-over-TCP in their fossil; we still do **Connect**, not gRPC).

| Leg | Transport | Why |
|---|---|---|
| Hit-path Check | in-process | D-1 |
| East-west (in-cell / cross-cell RPC) | **TCP** + TLS (SPIFFE). Connect on H2 (or H1 where needed). | Hardware offload. UDP/QUIC east-west is a tax. |
| North-south public door | **HTTP/3 (QUIC)** + ECH/PQC. Connect. WebTransport watches. | Internet, ECH, one public contract. H2 same Connect if the path cannot UDP. |

Forcing H3 east-west because the public door is H3 is the same class as
forcing gRPC because a mesh “automates HTTP/2.” UEC/RDMA is a later
`network/` adapter, not a reason to skip TCP v1.

**Why DCs run gRPC east-west — and what that is not.** They wanted
**binary protobuf**, HTTP/2 multiplex/streaming, deadlines, generated
stubs on **TCP**. That is not “gRPC so we can skip TLS.” Google still
encrypts in-DC (**ALTS**, cheaper handshake than public TLS, still
identity-bound). Plaintext-on-the-trusted-LAN is the old model; we do
not take it. **Connect is already binary protobuf.** gRPC vs Connect is
the **envelope** (trailers, `grpc-status`), not the codec. A standing
gRPC east-west stack is a second SDK. Same-host hit path is in-process
(no TLS). Same-host cross-process may be unix/vsock. Cross-host: TCP +
SPIFFE mTLS. If handshake CPU is the issue, that is **kTLS / ALTS-class
adapter on the TLS port**, not gRPC and not plaintext.

**gRPC leftover still deletes.** No public gRPC-Web. No new east-west
gRPC services. Overturn only with a measured EW path where the **gRPC
envelope** (not protobuf, not TCP) beats Connect **and** a five-field
ADR same-wave.

**One door, many frontends.** One gateway **codebase and proto**. Maglev-class
anycast **per cell**. Not one global VIP. Not a second REST/gRPC connector.

**No mesh product.** Istio/Linkerd/sidecar is not how we run mTLS. SPIFFE mTLS
is a **library** plus `gateway/` terminate and `network/` dataplane. A sidecar
hop on serving or product RPC is forbidden. A sold “ASM-class” SKU, if it
exists, is a `network/` facade IR module — it must not become our own Check
path. Deleting the mesh-not-identity rule is born-blocking.

**gRPC is leftover, not a mode.** Public gRPC-Web is not an operator-UI
protocol. East-west gRPC is not the product because a mesh “automates HTTP/2.”
Existing gRPC surfaces are **reorg_now deletion**, same class as leftover REST.
No standing transcode. FlatBuffers remain a measured adapter, never a second
SSOT.

Identity on the channel: SPIFFE mTLS east-west; passkeys at L2; **step-up to
L3/L4** (KR 본인인증 / eIDAS EUDI / passport+liveness per interview D58) via
Cedar `acr_required`.

**TLS port on `gateway/` (v1, not prose).** Closed adapters live as crates.
Deleting an adapter without a five-field ADR is born-blocking (same anti-forget
as GNSS on `cell/`):

| Adapter | Role |
|---|---|
| `tls13_hybrid_mlkem` | **v1 default on the public door** (X25519 + ML-KEM). |
| `tls13_x25519` | Classical-only. Dying adapter, not the destination. |
| `ech` | Encrypted Client Hello on the public door. IR `ech=on` when the cell has a cover hostname; `ech=off` does not mean the crate is absent. |

East-west SPIFFE is **TCP TLS** (same port crates; hybrid ML-KEM when the
stack speaks it). ECH is a **public-door** adapter (SNI privacy), not an
in-cell SPIFFE feature. ADR-0354 remains the crypto apex this amends.

**Enterprise visibility = Zero Trust endpoint isolation, not a QUIC
firewall.** A 2019 NGFW gets “visibility” by blocking UDP/443 or MITMing
TLS (custom CA, read SNI). QUIC + ECH are designed so an **on-path** box
cannot do that. Disabling ECH, blocking H3, or decrypting QUIC in the
middle weakens the product so a Palo Alto can keep its business model.
We do not sell that.

Inspection happens at **endpoints**. The path is opaque.

```
[Zero Trust: endpoint isolation]

  Client  -- inspect here --========================>  Server
  (device agent / ZTNA      encrypted H3 + ECH + PQC   (we terminate:
   client the user chose)    no SNI steal, no DPI       Cedar, WAF, audit)
```

Two trusted ends, not a middlebox:

- **Client end:** device posture + pre-encrypt inspect (their agent), or
  an **explicit** ZTNA hop they chose (`gateway/` `explicit_proxy` crate).
  That hop is a client they authenticated to, then a new H3/PQC/ECH leg
  to origin. It is not transparent UDP MITM.
- **Server end:** we terminate. After decrypt: Cedar, WAF, quota,
  structured events into `audit/` (tenant-exportable). This is GFE, not
  a NGFW on the wire.

What they need is **who talked to what, whether the call was allowed,
and whether data left**. That is endpoint evidence + our audit export.
It is not a copy of ciphertext on the path.

| Observer | Visibility (does not weaken H3/ECH/PQC) | Forbidden |
|---|---|---|
| Us (operator) | We **terminate**. After decrypt: Cedar, WAF, quota, structured access events into `audit/`. | On-path QUIC DPI of tenant traffic we did not terminate. |
| Tenant admin (workloads in a cell) | `network/` flow logs + SPIFFE identity; `audit/` export to their SIEM; WAF events after **our** terminate. | A `firewall/` cap. Sidecar MITM. Security groups that **block** UDP/443. |
| Employee device / corp SASE | **Explicit-proxy / ZTNA adapter** on `gateway/`: they **choose** us as the hop; both legs are real TLS/H3 (PQC+ECH still on). Or **Connect on H2** (same contract, TCP) if their office blocks UDP. Or private connectivity so the NGFW is not on path. Endpoint agents they run see pre-encrypt — their device, not our wire. | Transparent QUIC MITM. “Enterprise mode” that turns ECH off so SNI is visible to a middlebox. A second public API for “inspected” clients. |

**How the three SASE patterns map (apply, do not become a NGFW).**

1. **Endpoint / browser inspect (DLP and malware before encrypt).** Applies as
   the **client end**, not as `gateway/` core. Chrome Enterprise / Island /
   CrowdStrike-class: scan on the device, then wrap in QUIC/TLS, path stays
   E2E. We compose with that; we do not intercept after encrypt.

   **We do not ship an Island-class browser as a cloud requirement.** AWS
   does not; Google can because they already own Chrome. Device health is
   a **port**, not a Chromium fork: `iam/` `device_attestation` crate
   (closed adapters: passkey/WebAuthn, MDM/Intune, Chrome-Enterprise
   attestation, SPIFFE workload). Cedar reads those as context
   (`acr_required`). First-party web apps (when the apps discussion
   adds them) run **in the customer’s browser** as tenant #0, same APIs.
   A sold enterprise-browser `app/` exists only if that discussion
   explicitly adds it — deleting the **attestation port** is
   born-blocking; not building Chromium is not a gap.

2. **Handshake metadata and behavior (JA4/JA4S, volume, timing, dest
   reputation).** Applies at **our terminate and dataplane** as abuse/bot
   **signal**, never as the PDP and never as DLP. `gateway/` `fingerprint`
   crate (JA4-class on the ClientHello we actually terminate);
   `network/` `quic_metadata` + `flow_log`; dest reputation is **egress**
   of tenant workloads, not SNI-steal on inbound. ECH makes outer-hello
   fingerprints weaker — do not “fix” that by turning ECH off. ML-on-flow
   is `observability/` + gateway WAF, not `intelligence/detection`
   (GuardDuty stays out). Authz remains Cedar.

3. **Identity-aware proxy for inbound workloads.** Applies: this **is**
   north-south `gateway/` + `iam/` + in-process `policy/` Check. Validate
   identity, device health, context, then the backend sees an already
   authorized call. It does **not** apply as a sidecar / ephemeral proxy
   per microservice (that is Istio). East-west stays SPIFFE library +
   in-process PEP. A sold “Cloud IAP” SKU is a `gateway/` **facade** on
   the same contract, not a second door and not a mesh.

**Connect transport:** one RPC contract. H3 is the door default. **H2 is
the same Connect framing** when the path cannot carry UDP — not a second
SDK, not REST, not “disable QUIC in the product.” Our `network/` default
**allows** UDP/443.

Closed adapter crates (delete = born-blocking):

| Crate | Cap | Role |
|---|---|---|
| `waf` | `gateway/` | L7 inspect **after** decrypt. |
| `explicit_proxy` | `gateway/` | ZTNA / explicit hop. Trusted terminate, not transparent MITM. |
| `fingerprint` | `gateway/` | JA4-class handshake signal. Not Cedar. Not DLP. |
| `device_attestation` | `iam/` | Device/workload posture into Cedar. Not a browser. |
| `flow_log` | `network/` | 5-tuple + identity. No payload. |
| `quic_metadata` | `network/` | Version / connection-ID. Not payload decrypt. |

No `firewall/` capability (ADR-0132). Marketplace NGFW appliances plug
into `network/` as plugins, they do not become our door.

`Check` is never a public method. Product RPCs are. PEPs call the in-cell PDP.

**MUST (one Connect wire; TLS adapters in tree; visibility at terminate)**

- **achieves:** one generated SDK; PQC/ECH cannot vanish into chat; no sidecar
  tax; enterprise audit without turning off cryptography.
- **origin:** “gRPC+mesh vs protobuf+HTTP for middleboxes” is a tenant-on-AWS
  playbook. “Inspect QUIC like a NGFW” is the same playbook applied to H3:
  visibility by weakening. Prose-only PQC/ECH/WAF is forgotten.
- **rule:** one Connect contract (protobuf **is** the binary); public
  door **H3/QUIC**; east-west **TCP** + SPIFFE mTLS (Connect on H2);
  Check in-process; no plaintext-LAN; no standing gRPC EW (envelope ≠
  codec); TLS CPU → kTLS/ALTS-class adapter not gRPC; no QUIC-EW because
  the door is H3; no Istio; ECH public-door only; no on-path QUIC MITM.
- **ensure:** new RPCs generate Connect; new TLS/ECH/WAF/proxy/fingerprint/flow-log
  code implements those ports; layout/registry cannot drop those adapter
  names without OVERRULE; no new gRPC service crates; no PR that turns ECH
  off as “enterprise visibility”; no PR adds a sidecar IAP.
- **overturn_when:** a measured serving path needs a different RPC **and** a
  five-field ADR lands same-wave; TLS suite change is a new adapter plus IR,
  not a second door.

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
- **rule:** jurisdiction law is **pack overlay** on the same IR/Cedar/ReBAC/cells.
  **EU is not a country.** v1 **namespaces:** `us`, `eu`, `jp`, `kr`.
  **Packages** inside a namespace are granular (instrument/program), not
  the whole namespace. Binding is a **union** of package ids. Structural
  controls that help many regimes live in the platform; the rest lives in
  packages.
- **ensure:** no PR encodes EU-only identity or worldwide ACL replica as default; cell
  placement refuses a pack that exceeds the cell’s certification (E18); no PR
  invents `packs/kr-eu` combinatoric ids; extra jurisdiction dirs beyond
  us/eu/jp/kr are not v1.
- **overturn_when:** a single jurisdiction is the only remaining market AND a replacement
  ADR says so.

EU next-decade instruments (GDPR tightening, DORA, NIS2, AI Act, Data Act, CRA, eIDAS 2)
are **inputs to `packs/eu`**, not a reason to delete KR-shaped L3 or legal-retention
classes. Data Act switching is a **control-plane dump** of the event log and proofs, not a
JSON public API. Member states (DE, FR, …) bind **`eu/*` packages**
(at least `eu/gdpr` when GDPR attaches) until a member-state package
exists. They are not `packs/eu` as one blob.

**Granular packages, projected, then unioned.** A namespace is not one
blob. Packages are the unit (`eu/gdpr`, `eu/dora`, `kr/pipa`,
`kr/csap`, `us/hipaa`). Selecting a package is **not** blanket
application to the tenant.

Each package **projects** onto dimensions of the request. IR declares
the projection; Cedar is scoped to it. Dimensions are **not a closed
product catalog**. If a regulation attaches to it, it must be
projectable. They **map onto Cedar’s tuple** so we do not grow a second
policy language:

| Dimension (examples) | Cedar | Example package |
|---|---|---|
| **client** / subject / device | Principal | `eu/gdpr` on customer C, not every principal |
| **action** / RPC / business transaction | Action | `eu/dora` on an operational posting, not a profile `Get` |
| **resource** / record / field | Resource | `kr/pipa` on RRN; `us/hipaa` on a clinical row |
| **routing** / cell / hop / home-cell | Context | `kr/csap` on the cell; cross-cell Check; no silent replica |
| **purpose, acr, sector, time, …** | Context | `acr_required`; DORA ICT third-party; retention window |

A Check **unions** packages whose projection **matches this**
(Principal, Action, Resource, Context). Selecting `packs/eu` does not
blanket DORA onto every dimension. HIPAA does not attach to an
anonymous US download. New dimension = pack IR + Cedar attribute, not a
new cap and not a markdown folder.

**Placement:** packages that project onto **cell** or **record** still
need `required ⊆ certified_for` for those bytes. Contradiction on the
same bytes fail-closes or splits. Never “strictest common subset.”
Never `packs/kr-eu`.

**A in B, customer C from D.** KR company in JP, German customer:
profile Check unions client-projected `{eu/gdpr, …}`; a JP payroll
posting unions transaction/record `{jp/appi}` only. Same tenant, different
projections.

Empty per-instrument directories are not v1. Projection lives in Cedar+IR.

**CaC in, CaS out.** Packs are **Compliance-as-Code**: versioned Cedar+IR
packages, not markdown. They are **consumed as Compliance-as-a-Service**
on the public contract (same Connect/H3 door): bind, project, evaluate
which packages apply to this client / transaction / record / cell,
export evidence. **First-party apps and third-party apps** (marketplace
plugins) consume **that same CaS** as tenant #0 — no private pack path,
no in-process PDP only we can call. CaS is a **facade of `compliance/`
+ `policy/` Check**, not a `packs/` capability and not a second door.
`packs/` remains data the engines load.

**Where it belongs, and what it must not do.** `packs/` is **not** a
capability. It is a **versioned CaC bundle** at repo root (like
`third-party/`: data engines load, no `core/`). It stays at root so
**one** bundle feeds three engines without those engines owning each
other’s data:

| Job | Owner | Packs do **not** |
|---|---|---|
| Evaluate Check | `policy/` (compiled Cedar in-cell) | No PDP, no pack-algebra, no fetch on the hit path |
| ReBAC tuples | `policy/` | No `eu/gdpr` as a Zanzibar group |
| Principals, attestation | `iam/` | No IdP, no passkeys |
| Cell certify / place | `cell/` (`certified_for`) | No tenant CRM, no second placer |
| Evidence, CaS catalog/bind | `compliance/` facade | No Merkle log (`audit/`) |
| Tenant lifecycle | `tenancy/` | No “the tenant is pack X” as the only law |
| Marketplace plugins | `marketplace/` (signed plugins) | Jurisdiction packages ≠ `vpack_` vertical plugin ids — different word |
| i18n FTL, sticker packs | apps / copy | Not `packs/` |

Folding the bundle under `policy/` makes `cell/` and `compliance/`
depend on the PDP’s tree. Folding under `compliance/` makes Check
compile from the evidence product. **Root `packs/` is the zip SSOT.**
Today’s markdown under `packs/` is KEEP+WORK, not the job.

**Realization (adversarial — do not build a second PDP).**

Hyperscalers do **not** recompute “GDPR ∪ APPI ∪ PIPA” as a custom
algebra on every RPC. They split:

| Plane | AWS / Google | Us |
|---|---|---|
| **Control** | Org policy, Assured Workloads, VPC-SC, region | Cell `certified_for` + pack IR placement. CSAP, DORA, FedRAMP-class packages live **here**. They are not a per-payment Check. |
| **Serving** | IAM / Cedar on the call | In-process `policy/` Check. Pack **Cedar fragments** are already in the cell snapshot. |
| **Evidence** | Artifact / Audit | `compliance/` CaS catalog, bind, export — **not** a DIY evaluator |

**Challenges that fail if we ignore them:**

1. **“Union” ≠ Cedar permit-OR.** Regulatory composition is **conjunction
   of forbids/obligations**. A custom unioner beside Cedar is a second
   PDP. Compile enabled fragments into the snapshot; Cedar’s forbid
   wins. Do not implement pack-algebra on the hit path.
2. **DORA/CSAP on a profile `Get` is a category error.** Package IR has
   `plane: serving | control`. Control packages do not enter every
   Check union.
3. **“Any dimension” without schema is README.** A projection dimension
   **is** a Cedar schema attribute (principal/action/resource/context)
   already on the request. New dimension = schema+IR version, not a
   free string.
4. **CaS “evaluate packs” for apps is a second Check.** First-party and
   third-party apps are **PEPs**: they call product RPCs that already
   Check. CaS is bind/list/preview/evidence. Plugins do not re-implement
   GDPR.
5. **Fetching `packs/` on Check** violates D-1. Snapshot is compiled at
   pack bind / cell certify. Request-time only supplies context.
6. **Zanzibar tuples are not pack ids.** Do not encode `eu/gdpr` as a
   ReBAC group. Packs overlay policy; tuples stay relationships.

**MUST (packs: granular union, not a country stamp)**

- **achieves:** A-in-B-with-C-from-D without GDPR-floor, N² ids, or DORA
  for every EU tenant.
- **origin:** markdown country trees; EU as a country; one pack per
  tenant; combinatoric `kr-eu`; namespace implied every instrument.
- **rule:** v1 namespaces `us`, `eu`, `jp`, `kr`; EU is Union law not a
  country; packages are granular CaC (Cedar+IR) in root `packs/` **data**;
  each **projects** via Cedar schema (P/A/R/Ctx); Check is `policy/`;
  placement is `cell/`; CaS is `compliance/` facade; `iam/` `tenancy/`
  `audit/` `marketplace/` are not pack engines; no pack-algebra, no pack
  fetch on Check; `plane: serving | control`; apps are PEPs; no
  `packs/<a>-<b>` ids; `packs/` is not a cap.
- **ensure:** no new namespace outside {us,eu,jp,kr} without a five-field
  pack ADR; no PR that blanket-applies a namespace; no PR that loads
  `packs/` on Check; no PR that adds a pack-unioner beside Cedar; no
  private pack API for `app/`; control-plane packages are not serving
  Check predicates; no PR that puts a PDP, placer, or CaS core under
  `packs/`; marketplace `vpack_` is not a jurisdiction package.
- **overturn_when:** a five-field ADR adds a namespace or member-state
  package with a loader same-wave.

### D-7 — 3-year no-regret / regret

**Keep even if implementations swap:** cells; serving RAM vs control journal; Cedar+ReBAC
in-cell; proto IDL; in-process `Check`; k8s store behind a port; apps as IR modules;
two-class evidence; shreddable principals; eIDAS/KR step-up; owned cell journal; one
gateway (one contract, N cell frontends); packs; TrueTime interval API with a
cell clock port (ntp/ptp/gnss adapters); Connect-class one wire; gateway TLS
port with hybrid-ML-KEM + ECH crates; Zero Trust endpoint isolation (not
on-path QUIC MITM); Connect contract with H3 north-south and TCP east-west.

**Regret:** etcd as product DB; AWS closed journal; JSON as product codec; Helm-as-source;
Kyverno/Kubewarden as default; one global cluster; worldwide ACL replica; silent drop of
privileged evidence; passkeys as L3; unpublished binary lock-in; EU-as-only-baseline;
cap-root census files; dual `cedar/`+`policy/` children; Istio/Linkerd as our
mTLS identity; standing gRPC east-west; PQC/ECH as ADR prose with no crates;
on-path QUIC MITM or ECH-off “enterprise mode”; a `firewall/` cap; `ci/` and
`messaging/` as a live capability name; new `cloud-*` crates or a `cloud/` root.

### D-8 — Repo root and capability / app root (amends ADR-0701)

A directory or file is allowed only if something that is **not a census gate** loads it,
or it is `OWNERS` / short `README.md` / `BUCK` / `app/<product>/PRD.md`. Git history is
the audit log. Do not invent a destination for leftovers; many must **not exist**.

**Repo root (closed).** Workspace: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
`rustfmt.toml`, `deny.toml`, `reindeer.toml`, `.buckconfig`, `.buckroot`, `.cargo/`.
GitHub: `.github/`, `.gitignore`, `.gitattributes`. Hubs: `README.md`, `LICENSE`,
`OWNERS`, `AGENTS.md`, `CLAUDE.md`. Meta: `build/`, `third-party/`. `base/` is **not**
pre-created; it appears only when the first crate admitted under the ≥3-caps-below-all
rule. `governance/` is gone (D-17). **No `kernel/` and no `os/` rungs** (D-13). Fleet is
stripped Linux on Cloud Hypervisor and/or Firecracker (`compute/`).
Composition: `app/`. One directory per capability (including BUILD `policy/`).
`packs/` = install authority (D-24). Repo-root `docs/` = thin operating contract
+ unique `docs/decisions/` ADR home + `docs/standards/` (D-27). Per-owner
`docs/` lives under each cap/app, not here. Thin `templates/`: ADR skeleton +
swarm ritual only. **No catch-all `specs/`.** Agent entry is `AGENTS.md` /
`CLAUDE.md`.

**Not repo-root (gone this wave):** `contracts/`, `plan/`, `tasks/`, `scripts/`,
`specs/`, `registry/`, `evidence/`, `governance/`, `oya/`, `cloud/`.
**Removed this wave (not shrink-only):** `libs/`, `tools/`, `infra/`, `kernel/`,
`os/`, `contracts/`, `plan/`, `tasks/`, `scripts/`. Last leg is **gone**, not
tolerated.

**Capability and `app/<product>/` — identical closed children.** Caps and apps
use the **same** four faces. Hyperscaler analog: one API (ports), one engine
(core), N backends (adapters), one serving process (facade). Google Stubby
service + impls; AWS control-plane vs data-plane as engines, not dump folders.

| Child | Belongs | Does not belong |
|---|---|---|
| `core/` | Domain + use cases. No IO. | sqlx, HTTP, S3, Helm, IPs |
| `ports/` | Traits: blob, records, mailbox, clock, pack-id, … | Adapter impls, proto dumps as SSOT |
| `adapters/` | One impl per backend: sqlite, postgres, s3, oyatie, imap, onprem, stripe | Business rules |
| `facade/` | The process / Connect surface you run | Cloud SKU implementation |
| `cedar/` | This cap/app’s Cedar only | Platform templates (those wait `policy/` cap) |
| `observability/slos/` | Generated from IR | Hand OpenSLO, dashboards JSON |
| `iac/` | IR the reconciler applies | Helm/Tofu/charts as source |
| `docs/` | This owner's g3doc (D-27): `README.md`, `concepts/`, `runbooks/`, `design/` | ADR copies, IPs, catalogs, scorecards, customer manuals, `plan/`, `tasks/` |
| `OWNERS`, short `README.md`, `BUCK` | Yes | — |
| `PRD.md` | **`app/<product>/` only** | Cap roots |

**This shape does not change.** Owner PRs fill these children with content.
They do **not** add faces, rename faces, insert `plan/`/`tasks/`/`crates/`/
`internal/`/`domain/`, or invent a per-team taxonomy. A new child is a D-8
five-field amendment (escalated, D-29), not an OWNERS courtesy.

**Inside each face (Cargo defaults; same for every cap and every `app/<product>/`).**
One crate per directory. Only `Cargo.toml`, `src/`, `tests/`, `OWNERS`, `BUCK`.
No nested IPs, catalog.yaml, Helm, `plan/`, `tasks/`. Do not flatten `src/`
(matklad / rust-analyzer). Do not require Clean-Architecture `domain/` or
`use_case/` — those are not Cargo or google3.

| Path | What is in it | Not in it |
|---|---|---|
| `core/<engine>/` | Lib crate: `src/lib.rs` + snake_case modules. No IO. Calls **port crates**. | sqlx, HTTP, proto, adapter types |
| `ports/<port>/` | **Agreed** traits (`owner-port`). Dir leaf = `<port>`. | Impl, proto SSOT |
| `ports/draft/<port>/` | **Unagreed** (`owner-port-draft`). Other owners must not `path =`. | Sold proto |
| `adapters/<port>-<backend>/` | One backend of an agreed port. Dir leaf = `<port>-<backend>`. | Domain rules |
| `adapters/draft/<port>-<backend>/` | Draft impl; `git mv` with the port | — |
| `facade/<surface>-app/` | Process: `src/main.rs` (and `lib.rs` if tests need it). Handlers call core. | sqlite, business novels |
| `facade/proto/<owner>/<api>/v1/` | Sold proto (AIP-191): dir **is** the proto package; files `snake_case.proto` | Draft names, `v1.proto` as filename |
| `cedar/` | This owner's Cedar only | Platform templates |
| `observability/slos/` | Generated from IR | Hand YAML novels |
| `iac/` | IR this engine needs in a cell | Helm/Tofu as source |
| `docs/README.md` | Landing (g3doc traffic cop) | Essays |
| `docs/concepts/` | How *this* owner works | ADR-07xx copies |
| `docs/runbooks/` | On-call for *this* owner | Fleet stamped books |
| `docs/design/` | Owner-local design notes | `plan/`, IPs, ADR forks |
| `PRD.md` | App product requirements only | Cap roots |

Do not add a fifth face. Do not pre-create empty module dirs.

Grammar (no `oyatie-`, no `cloud-` prefix):

```
owner     := <capability> | <product>          # kebab, registered / D-22
engine    := kebab                             # lifecycle, evaluate, journal
port      := kebab                             # blob, records, mailbox, clock
backend   := sqlite|postgres|s3|oyatie|imap|smtp|jmap|stripe|onprem|…
surface   := kebab                             # optional facade qualifier

core crate      := owner "-" engine
ports crate     := owner "-" port                 # AGREED only (D-28)
draft port crate:= owner "-" port "-draft"        # UNAGREED; path ports/draft/<port>/
adapters crate  := owner "-" port "-" backend     # agreed port
draft adapter   := owner "-" port "-" backend "-draft"
facade crate    := owner ["-" surface] "-app"
```

Examples: `storage-blob`, `storage-blob-sqlite`, `storage-blob-s3`,
`foundry-blob-draft` (local, rename freely), `foundry-ontology`,
`payroll-run-app`, `iam-pdp`. One primary `core/` engine per cap/app unless a
§7 split names a second. `app/foundry/{pages,grid}` fold into these faces on
the Foundry BUILD PR — they are not a third layout. Draft names are **illegal**
in sold proto packages. Promotion is `git mv` of `ports/draft/<port>/` onto the
provider's `ports/<port>/` after D-28/D-29 review — not a copy.

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
- **rule:** cap and `app/<product>/` share this child set; the set and inner Cargo
  layout **do not change** per owner (D-29/D-30); cap-root `cedar/` only; `policy/`
  is the capability; SLO source is IR; no `specs/` catch-all; `ports/` is the
  contract face (draft vs agreed: D-28); extras, REST/JSON product surfaces, and
  `HANDOFF.md` deleted, not grandfathered. Temporarily breaking live callers/gates
  is accepted. Leaving anti-pattern debt is not.
- **ensure:** layout allowlists match this set; no immortal `IPs/`; no both `cedar/`
  and `policy/` as cap children; owner PRs that add a new child or `domain/`/
  `use_case/`/`crates/` taxonomy fail.
- **overturn_when:** a child is loaded by a compiler/PDP/SLO/reconciler AND a
  five-field amendment lands same-wave.

## Consequences

- Implementers read this plus ADR-0701/0702/0704/0705/0615. Do not re-derive from chat.
- `policy/` extraction and IR proto are implementation follow-through, not optional sketch.
- Admission remains VAP/CEL+PSA as cited from ADR-0704 / ADR-0700. Proposed
  0710-range ids are not `depends_on`.
- Merge-blocking CI is **presubmit** only (D-10). No second protected
  `presubmit` context.
- Node OS/kernel: D-13. Do not re-create `kernel/` or `os/` as empty rungs.

### D-13 — Stripped Linux on Cloud Hypervisor / Firecracker

**Decision.** Hyperscalers do **not** run Talos or Kubernetes as the
cloud OS. Google: Linux + **Borglet**. Meta: Linux + **Twine**. AWS:
**Nitro** + **Firecracker** (Lambda/microVM) / KVM VMs — not Talos.
Talos is a **Kubernetes node** OS. GKE nodes are COS/Ubuntu, not Talos.

**Honest constraint today: Asterinas is not a working node kernel for
us.** Delete `kernel/`. Do not vendor it. Do not leave an evaluation
tree as a destination. **Reconsider** replacing Linux **only when**
Asterinas **or Hermit** (or an equally measured unikernel/kernel) is
mature **and** a five-field ADR cuts over with plant evidence. That is
not a vacant `kernel/` rung and not a pin “in case.”

**How the plant actually runs** (Talos is not in this picture):

```
cell/ places load
        │
        v
compute/ agent (Borglet analog)  ──host Linux (KVM)──
        │                              │
        ├─ Cloud Hypervisor  ──►  stripped Linux guest (VM)
        └─ Firecracker       ──►  stripped Linux guest (microVM / function)
```

| Piece | What we do | Who owns it |
|---|---|---|
| **Host** | Minimum Linux **with KVM**. Runs the agent + VMM processes. Not Talos, not kubelet. | `compute/` + `build/` (host image) |
| **VMM** | **Cloud Hypervisor** = VM class (virtio, full-ish machine). **Firecracker** = microVM/functions (jailer, dense). Both KVM. Not QEMU as identity. | `compute/` adapters |
| **Guest kernel** | **Stripped-to-minimum upstream Linux** (Firecracker-class Kconfig / tiny guest). One kernel family, two configs (host vs guest). We **consume** stable LTS; we do not fork a distro under `os/`. | `build/` produces the artifacts |
| **Guest userspace** | Bare min to run the workload + our agent/protocol (virtio-vsock). Not a distro, not Talos machined. | `compute/` + `build/` |
| **Agent** | Ours, Borglet analog: place, health, attach net/disk. Host-side talks CH/FC APIs. | `compute/` core |

**Sold kube SKU** (optional): kubelet runs **inside a guest**, as a tenant. It is not the host agent.

**gVisor and Kata — considered, not the fleet VMM.**

| Tech | What it is | Hyperscaler | Us |
|---|---|---|---|
| **gVisor** | Userspace kernel (Sentry) sandboxing **containers** | GKE gVisor, Cloud Run, App Engine — **not** Borg’s VMM | Optional **`compute/` adapter** for a container-isolation SKU (Cloud Run analog). Not the host. Not a replacement for CH/FC. |
| **Kata Containers** | VM-per-pod via CRI (QEMU/CH/FC underneath) | Kube **runtime class**, not Twine/Borg/Nitro | Allowed only as a **sold `k8s/` runtime class** (guest). Using Kata as our fleet scheduler **is** CRI-on-kube. |

Fleet plant stays **CH + Firecracker + stripped Linux**. Kata sitting on Firecracker is still kube CRI. gVisor is a **sandbox**, not a hypervisor. Empty `gvisor/` / `kata/` dirs are STAY GONE. New adapter crates under `compute/` only when that SKU is sold.

**What we build vs do not:**

| Build (Rust) | Do **not** build |
|---|---|
| **Fleet orchestration** — `cell/` placement + `compute/` agent + CH/FC reconcilers. This **is** our Borg/Twine. Own proto, own agent, own VMM APIs. | A **Kubernetes engine** (apiserver, kubelet-as-host, kube-rs as Borg, etcd as the fleet store). Google did not Borg-on-kube. |
| Sold **`k8s/`** SKU **if** customers want a kube API: wrap **upstream** kube (EKS/AKS pattern) — hosting, quota, SLA, CAPI. | A Go→Rust kubernetes.git port as a prerequisite to operate. |

Two products only if we **sell** kube. One **operations** plane either way: CH/FC + stripped Linux.

**kube-rs / k8s-openapi:** Kubernetes **client** libraries (OpenAPI types for
Pod/Job/…). They talk to a **kube-apiserver**. That is **not Borg.** Do
**not** use them as the fleet scheduler (`compute/` / `cell/`). Using
them for “our Borg” **is** running Kubernetes. Allowed only as
**`k8s/` adapters** talking to **upstream** kube for the **sold SKU**.
Pipeline must not spawn `batch/v1 Job`s as the worker plant (that is
Prow-on-kube). Workers are CH/FC guests via `compute/`.

`build/` ships: host image, guest kernel, CH/FC binaries (pinned). `storage/` holds images. `pipeline/` builds them when the graph says so. No `os/` farm. No Talos machine API.

- **Delete `kernel/`.** Asterinas is gone. No third-party pin “in case.”
- **Delete `os/`.** Talos farm is not the fleet OS.
- **`os/ports/kernel-abi` dies with `os/`.** We are not porting a
  second kernel ABI until we own one — and that is **not** Asterinas.
- **Talos** at most a **`k8s/` adapter** for a sold kube worker image —
  optional. `infra/talos/` burns or that adapter. Not operations.

**Hyperscale operation does not use Kubernetes.** Google’s fleet is **Borg**
(still; kube ~5k-node class is not Borg). Meta’s fleet is **Twine**. AWS’s
fleet is not EKS; they **sell** EKS by wrapping **upstream** kube and never
needed a Go→Rust apiserver. Our fleet scheduler is **`compute/` + `cell/`**
(Borg/Twine analog). A `build/port-engine` **kubernetes.git port is not
required to run the cloud.** It is a SKU choice: if we sell GKE-class, **EKS
pattern** (upstream apiserver + our hosting/quota/SLA) is enough. Porting
kube-apiserver does **not** give multi-tenant CP hosting, upgrades, quota,
SLA, CAPI, cell placement, or cluster billing. Google still has **GKE** after
writing Kubernetes. Collapsing “core = port, facade = managed” is the
dual-stack we killed for Talos.

`build/port-engine` stays as a **generic** generator for when we **own the
node OS** (D-13), not as a standing kubernetes-port program. Do not staff a
kube port because the tree once had one.

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
- **Not this cap:** fleet node OS (**compute/** agent on Linux), mesh/DNS
  (**network**), public door (**gateway**), cell topology (**cell**), SPIFFE
  (**secrets**). GKE-class **uses** those; it does not own the fleet.

**MUST (managed cluster ≠ k8s port)**

- **achieves:** porting Kubernetes cannot be mistaken for finishing the cloud
  product; no generated apiserver tree as `k8s/core`.
- **origin:** 0704 port-engine + 0562 coarse “k8s = owned CP + managed facade”
  mixed two jobs hyperscalers split (GKE vs kubernetes.git).
- **rule:** hyperscale **operation** is `compute/`+`cell/`, not Kubernetes;
  `k8s/` is a **sold** GKE/EKS-class SKU wrapping **upstream** kube unless
  a later ADR owns an apiserver; no kubernetes port-engine as an operations
  requirement.
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
| `infra/talos/**` | local kube bring-up | **`k8s/adapters`** only if we sell that worker image; else **delete**. Not fleet OS. |
| `kernel/**` Asterinas | nested workspace only | **Delete.** No pin. Reconsider only via D-13 overturn_when. |

Census JSON (`registry/graph`, manifests) **regenerates or dies** with the dirs. It is
not a destination.

**MUST (one kernel story)**

- **achieves:** fleet is Linux+agent like Borg/Twine/Nitro; Talos/kube are not
  the cloud OS; no empty rungs.
- **origin:** kuberos deleted; Talos port farm remained as if Sidero were the
  hyperscaler node story.
- **rule:** in-tree `kernel/` and `os/` are gone; fleet is **stripped
  Linux** on **Cloud Hypervisor and/or Firecracker**; agent is
  `compute/`; Asterinas/Hermit are **not** plant today and not a git
  pin; Talos/kube are not operations; sold `k8s/` may wrap upstream kube.
- **ensure:** registry has no `kernel/` or `os/`; no Asterinas/Hermit
  evaluation tree; no PR treats Talos as the fleet OS; new VM/microVM
  code is CH/Firecracker adapters under `compute/`; guest/host kernels
  are `build/` artifacts from upstream Linux, not an `os/` capability;
  no `kube`/`k8s-openapi` in `compute/` or `cell/` core (sold `k8s/`
  adapters only).
- **overturn_when:** Asterinas **or Hermit** (or equal) is measured
  mature **and** a five-field ADR replaces Linux as the guest kernel
  same-wave, with CH/Firecracker still the VMM unless that ADR also
  changes the VMM.

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

No new `scripts/check.sh` / pre-push product. Rust-first: a three-line git hook may call `cargo fmt --check` on staged `*.rs`. Do not resurrect retired `./bin/oya verify` / `dev-cli`.

**MUST (nextest is the proof)**

- **achieves:** one compile+test signal; no double compile; no libtest dual; PR hermetic.
- **origin:** blog four-tier put `cargo check` and network `cargo-audit` on the merge path; `cargo-dist` assumes a CLI product; mutants-on-everything is not a nightly.
- **rule:** nextest is the only compile+test proof in presubmit/postsubmit/nightly unit lanes; `cargo check` and `cargo test` are not CI; release binary is CD; advisory fetch on PR is vendored; one license/ban engine; no cargo-dist; no crates.io semver gate.
- **ensure:** required workflow invokes nextest, not libtest; no `cargo check` job; no win/mac per-PR smoke; deny/audit are not two network tools on the PR.
- **overturn_when:** a five-field ADR names a different runner that still compiles once and stays hermetic.

Do **not** add one required GitHub check per capability (skipped-check failures, queue
combinatorics). **One** protected context: `presubmit`. `presubmit`
is not a second protected check. Lane isolation is **worktrees + non-overlapping
paths**, not 24 contexts. Do not resurrect merge-base **count** baselines as
“affected set.”

New workflow and context names: `presubmit`, `postsubmit`, `nightly`, `weekly`,
`promotion-predecessor`, `release`. No `oyatie-` prefix. Today’s `presubmit` is
the **presubmit** rename target (branch protection in the same change).

### D-11 — What the cloud is, and what each capability holds

The cloud is **not** a root `cloud/` tree and **not** a JSON catalog. It is the
**closed capability set** (registry). Repo root only **names** those directories
(plus `base/` when admitted, `build/`, `third-party/`, `app/`, `packs/`, `docs/`,
`governance/` as already in D-8 — **not** `kernel/` or `os/`). Everything a
tenant or operator calls is a **facade** of one capability or an
**`app/<product>/`** that wires 2+.

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
| S3 / GCS / CAS | **storage** | core: **bytes** (object/CAS). Identity = digest/generation. Wall time is metadata, not TrueTime. Not SQL. |
| Spanner / Cockroach / Cloud SQL / BigQuery / Dataflow | **data** | core: **records**. Consumes cell TrueTime interval. Versionstamps = engine commit ordinal, not a second clock. **No `cloud-*` crates.** |
| EC2 / GCE / Functions | **compute** | **One** cap, **two reconcilers** (CH VM, Firecracker functions). GPU = VM SKU. gVisor = adapter. Sold kube is `k8s/`, not a third reconciler. |
| GKE / EKS control plane (sold) | **k8s** | core: managed cluster (lifecycle, CP host, quota, CAPI). Adapter: upstream or owned apiserver when we run it (D-13). Store: cluster objects only (D-2). |
| VPC / DNS / firewall / flow logs | **network** | core: dataplane, security groups (**allow** UDP/443), flow logs, QUIC metadata. Not Istio. Not a `firewall/` cap. |
| Front door / GFE / WAF / IAP | **gateway** | core: one Connect contract (H3 default, H2 same framing), Maglev **per cell**. TLS port + WAF-after-decrypt + explicit-proxy crates. Transcode is not a second API. |
| Pub/Sub / SQS | **bus** | owned queue + fan-out + seekable stream + outbox; per-key order. Kafka/Pulsar = adapters. Not `notify`. |
| Vertex / Bedrock | **intelligence** | core: inference + agent runtime + adapters. Not GuardDuty. Not a chat app. |
| Step Functions / Composer | **workflow** | core: engine. Studio is facade. Business sagas, **not** deploy orchestrator (D-1). |
| Cloud Build / TAP / CodePipeline | **pipeline** | **One** execute engine. Internal TAP = this repo as tenant #0. Sold Cloud Build = same engine, tenant graphs. GitHub is an adapter, not the product. JSON check fleets are not this cap. |
| CloudFormation / Config reconciler | **iac** | core: IR unifier + reconcilers. `<cap>/iac/` is **this** cap’s desired state; `iac/` the cap owns the **engine**. |
| Billing / Cost Explorer | **billing** | core: meter, rate, invoice, tax, FinOps. Sold-ness, not a drawer. |
| Marketplace | **marketplace** | core: signed plugins + Cedar install envelope + SKU **engine** (what exists). Rate/invoice is `billing/`. Not a `build/` price list. Not KYC/escrow. |
| Artifact / evidence packs | **compliance** | core: pack evidence, data-class registry. Consumes **audit**. Not the Merkle log. |
| SES / SNS / FCM (send) | **notify** | core: transactional email/SMS/push **send**. Not Gmail/Meet/Slack. |
| AppConfig / Feature flags | **flags** | core: flags, kill switches. Pack-gated overrides. |

**Meta (not sold as a tenant API, still in-repo):** `base/` only when admitted (≥3 caps, **non-domain** primitives; `TenantId`/`CellId` stay on their owner caps). `build/` toolchains/images/**frozen** port-engine (no destination corpus). `third-party/` vendored. `governance/` registry + off-ladder checks (D-17 default delete). No `kernel/` or `os/` rungs (D-13).

**`app/<product>/`:** composition only (hr, payroll, calendar, community, …). Wires 2+ of the table. **Does not** grow a cloud engine.

**`payments/` and `ledger/` are not this cloud set** (D-15). Do not park them in `billing/` or `oya/`. Product placement is a later discussion.

### D-14 — Each capability: is / is not / burns

Same split as `k8s/` (GKE product vs kube port). Nested leftover service dirs inside a cap **burn** (faces or `git rm`).

| Cap | **Is** (engine) | **Is not** | **Burns / move** |
|---|---|---|---|
| **cell** | Topology, physical **capacity**, which-cell router, rebalance, **clock port**. | GKE. Tenant CRM. Pack **loader**. `time/` cap. Schedulable remainder (`compute`). | Nested lifecycle/rebalancer dumps; `core/regional-pack`; `cloud-*` names. |
| **tenancy** | Tenant lifecycle, home-cell, org/account analog. Stores SKU **entitlement counts**; does **not** enforce them. | IdP (`iam`). PDP (`policy`). SKU catalog (`marketplace`). KYC/DSR/JWT. Numeric enforcer. | KYB/KYC, DSR cascade, nested PDP, JWT issuer, Citus/Helm, IP-journeys. |
| **iam** | Principals, passkeys, SCIM, role **store** (compiles to Cedar), federation **consume**, workload identity **consume**, **`device_attestation`**. Cognito-class user-token **issue** is a facade SKU if sold — not the kernel. | Cedar **eval** (`policy`). SVID **issue** (`secrets`). Zitadel-as-identity. SCIM creating tenants. Island browser. | PDP crates **move to `policy/`**. SVID issue **move to `secrets/`**. `consent-graph`, `cloud-iam/` dump, `tenant-rbac-*-evidence` farm. |
| **policy** | Cedar + ReBAC PDP, G-face + C0 snapshots. | IdP. Empty dir forever. | **Extract crates from iam now.** Cap-root `<other>/policy/*.cedar` → `<other>/cedar/`. |
| **secrets** | One cap, three facades: **KMS**, secret store, **SPIFFE/cert issue**. One crypto root. | PDP. OpenBao as identity. Nested `kms/` cap. | Nested `kms/` paperwork; OpenBao Helm as product; BYOK journey novels. |
| **audit** | Tamper-evident emit/seal/verify/query. Async on serving path. Chain TTL. CISO export. | Packs. Sync Merkle on Check. DPIA. Fifth retention store. | Journey `.cedar` novels; Helm HSM/postgres; scorecards. |
| **observability** | Telemetry + SLO **controller**. Not the billing meter. | Per-cap hand OpenSLO. SIEM as a 25th cap. Lab/diagnostics product. | Stamped OpenSLO. Nested `diagnostics/`. Helm Grafana/Loki/Mimir as identity. |
| **storage** | Durable **bytes**: object/CAS. Identity = digest/generation. S3 API + EBS-class **block** facade when sold. Pipeline CAS lives here. | SQL (`data`). Search. Drive/Meet/PACS **apps**. TrueTime as object identity. | `drive/`, `recordings/`, `imaging/` **REMOVE** (later `app/`). Census. `cloud-storage-*` OpenAPI. |
| **data** | Durable **records** engines: OLTP + OLAP + pipelines. **Consumes** cell TrueTime. Versionstamps = commit ordinal. `commit_wait` adapter (IR off on NTP ε). | S3/CAS (`storage`). **Ontology / Pages / Grid** (`app/foundry`). SERP. RAG (`intelligence` facade). BI **app**. `cloud-*`. A private `Now()`. Foundry-the-product. | Ontology kernel crates **move to `app/foundry`** when that product BUILDs (do not leave them in `data/core`). `search-*`; `data-cloud-*`. |
| **compute** | **One** cap, **two reconcilers**: CH **VM** + Firecracker **functions**. Agent is ours. GPU = VM SKU. gVisor = adapter. | GKE (`k8s/`). **`k8s-on-compute`**. Kata as Borg. Talos. QEMU as identity. One Raft. | Phrase `k8s-on-compute`. Splitting into 3 caps. |
| **k8s** | **Managed cluster product** (lifecycle, CP host, quota, SLO, CAPI). | kube-apiserver port (`build/port-engine` → adapter when we run it). Node OS. Mesh. | Dump + nested `managed-*`. |
| **network** | VPC, **private DNS**, L3/L4 dataplane, SG (allow UDP/443), `flow_log` + `quic_metadata`. Volumetric DDoS. TCP-optimized. DPU destination adapter. | Public door, public zone for the door, CDN, L7 WAF (`gateway`). Istio. `firewall/` cap. Cell picker. | Nested `dns/` dump; mesh as identity; `cloud-network-*`. |
| **gateway** | One Connect **contract**, Maglev **per cell**, TLS/ECH/WAF/IAP/fingerprint. **PEP only** (then `policy/` Check). Public DNS for the door + CDN/cache SKU. L7 bot/WAF. | Mesh. Second REST/gRPC API. Cedar **engine**. One global VIP. QUIC MITM. Connectors as the door. | Connector dump **REMOVE** then Connect door BUILD. `edge-cedar-eval`. |
| **bus** | Owned queue + fan-out bus + seekable stream + outbox; per-key order. Pub/Sub + SQS analog. | Sagas (`workflow`). Mailbox (`app/`). **Kafka/Pulsar as `core/`**. Human chat. SES (`notify`). | Crate names still `messaging-*` (KEEP+WORK rename). Kafka/Pulsar = **adapters** only. |
| **intelligence** | Vertex/Bedrock: **invoke**, endpoints, batch, quota. Hosted-agent **SKU**. GPUs **rented from `compute/`**. RAG = facade over `data/`. | Copilot UX. Chat CLI/SDK as core. GPU plant. GuardDuty. A vector store. | Claude/Codex/OpenAI-compat dumps; `authz-cedar-adapter`; CLIs. |
| **workflow** | Step Functions analog (rewrite). | Bus (`bus`). Forms/tasks/SaaS. Deploy (`pipeline`/`iac`). | **Purge current tree; rewrite.** Do not strangler. |
| **pipeline** | One execute engine (graph, queue, workers, controller). TAP / Cloud Build. | GHA as the product. **Foundry Pipeline Builder** (that UX is `app/foundry` calling `data/`). Prow/Tide as `core/`. JSON census. A root named `ci/`. | KEEP+WORK: today’s tree is not the product. Census/JSON gates REMOVE. Tide/webhook = GitHub adapter until cutover. |
| **iac** | IR unifier + reconcilers that call **cap Connect APIs** (`compute/`/`network/`/`storage/`). | Argo/Helm/Tofu **source**. kubectl as operations. Merge queue (`pipeline`). | `tofu/`, Argo app-of-apps, IP-GITOPS, `ports/rest`. |
| **billing** | **What you pay:** usage events from owning caps, rate, invoice, tax on platform SKUs, FOCUS/FinOps. Not scraped from observability. | Ledger (`ledger/`). Payments (`payments/`). SKU **catalog** (`marketplace`). | `accounting/`, `tax/` dump, `finops-portal/`, payment/subscription modules. |
| **marketplace** | **What exists:** signed plugins, Cedar install envelope, SKU attach. | List price / invoice (`billing/`). KYC/escrow/payout. App/dataset store. `build/` price view. | Escrow/deals/mediation dumps; `developer-sdk/` + `plugin-app-store/`. |
| **compliance** | **CaS** bind/project/export + evidence engine. Projects pack retention into `audit/`/`data/`/`storage/`. Does not store a fifth copy. | Merkle log. DLP core. Trust portal. eDiscovery product. Pack **data** (`packs/`). Second PDP. | `core/dlp`, `dsr`, `ediscovery`, `trust-portal`, DPIA clones. |
| **notify** | Transactional send (SES/SNS/FCM). | Email/SMS/push **send API**. | Mailbox/Meet/Messenger/contact-center (`app/` later). Current `comms/` dump **purged**. |
| **flags** | Deterministic **eval** (keep `evaluation-domain`), targeting, kill switch, pack-gated overrides. | Experiments product / p-value dashboards. OpenAPI+REST+gRPC dual. Clock adapter. OFREP as SSOT. | Cap-root dump (`catalog.yaml`, IPs, Helm, AUDIT-FINDINGS). REST/gRPC server dual. |
| **governance/** | Registry. Check crates **off ladder**; D-17 default **delete**. | Org JSON `specs/` corpus. Cloud product. | Census kernels (`no-template-stamping`, …). |
| **build/** | Toolchains, images, CH/FC+kernel **pins**. Port-engine **frozen** until a named owned corpus is Accepted. | Capability engines. Price list. Fleet agent. | `evidence/` essays. Staffing port-engine as Borg. |
| **third-party/** | Vendored pins when we need them. | Fake rungs (`kernel/`/`os/`). | Asterinas eval in `kernel/`. |
| **app/** | 2+ cap products. One shell `application`; Foundry is a **module**. V1: foundry, hr, payroll, accounting, **ledger**, payments, calendar, mail, messenger, community (shrunk). | A cloud engine. Ops console. D41 retirees. V1 treasury/FP&A/performance/learning. `ledger/` **cap**. `app/social`. | Absorbing D41; parking payments as a **cap**; `console/` pilot; empty SAP ghosts; community SecureDrop. |

**Missed before, now closed:** GKE vs kube-port (`k8s/`); Talos vs `os/`; PDP vs iam; trustd vs secrets; payments/ledger not billing; no empty `base/`/`kernel/`/`os/`/`k8s-port/`; census `ci` gates are not the delivery fabric. **D-20:** no `k8s-on-compute`; ontology not `data/core`; no gateway PDP; no `build/` price view.

### D-15 — Cloud-provider purpose and scope (not SaaS)

This set is what we **sell and run as a hyperscale cloud**. Analog: AWS/GCP/Azure **platforms**.  
**Out of every row below:** tenant SaaS products (HR, payroll, community, calendar, Slack-class UX, SAP-class accounting). Those **use** these engines via `app/<product>/`. A capability that ships a vertical product in `core/` is out of charter.

| Cap | Purpose | In scope | Out of scope |
|---|---|---|---|
| **cell** | Bound failure domains and place load. | Topology, physical capacity, which-cell router, rebalance, home-cell **admit**, clock port + adapters. TrueTime interval is the API. | GKE. Tenant CRM. Pack loader. Schedulable remainder (`compute`). Spanner ε as v1. `time/` cap. |
| **tenancy** | Tenant as the scoping primitive. | Create/suspend/delete, org/account tree, home-cell **bind**, store SKU entitlement **counts**. | IdP. PDP. Enforcing quotas. KYC. DSR execution. JWT. HR orgs. |
| **iam** | Prove **who** (and **device posture** as Cedar context). | Principals, passkeys, SCIM into an **existing** tenant, role **store**, federation **consume**, workload **consume**, `device_attestation`. User-token **issue** only as Cognito-class **facade SKU**. | Cedar **eval**. SVID **issue**. Creating tenants via SCIM. Zitadel kernel. Forking Chromium. |
| **policy** | Decide **may**. | Cedar PDP, ReBAC tuples, G-face distribute, C0 in-cell snapshot, in-process Check. | IdP. Writing every cap’s Cedar (caps own `<cap>/cedar/`). Global tuple replica. |
| **secrets** | Crypto root and issuance. | KMS, secret material, SPIFFE **issue**, cert **issue** when sold. | PDP. Embedding secrets in app products. |
| **audit** | Tamper-evident **record**. | Merkle log, seal of principal+tenant, privileged-path durability, **tenant-exportable** access events (the CISO feed). | Pack evidence (`compliance`). Sync seal on every Check. DPIA markdown. On-path packet capture as the audit product. |
| **observability** | See and SLO-gate the platform. | Metrics/logs/traces **substrate**, SLO **controller**, generated OpenSLO. | The **bill**. Hand OpenSLO. SIEM. Diagnostics/lab product. App analytics. |
| **storage** | Durable **bytes** (S3 / GCS / Colossus / CAS). | Object/CAS; S3 API; EBS-class block **when sold**. Pipeline CAS. Object DLP as SKU if sold. | Query engines. Drive/Meet/PACS apps. Search. Clock as identity. |
| **data** | Durable **records** infrastructure (sold **without** Foundry). | OLTP + OLAP + pipeline **engines**. Consumes cell `Now() → Interval`. Versionstamps = ordinal. `commit_wait` crate (IR off without measured ε). Vector search **facade SKU** if sold. | Bytes (`storage/`). Ontology / Pages / Grid / Workshop (`app/foundry`). TAP (`pipeline/`). SERP. RAG. BI app. `cloud-*`. Private `Now()`. |
| **compute** | Run **the fleet** (Borg/Twine/Nitro analog). | Two reconcilers: **CH VM** + **Firecracker functions**. Agent. GPU SKU. gVisor adapter. | GKE as fleet. **`k8s-on-compute`**. Talos. Kata as Borg. Asterinas today. QEMU as identity. GPU plant for intelligence. |
| **k8s** | **Sold** GKE/EKS/AKS-class SKU. | Cluster lifecycle, hosted CP, quota, SLA, CAPI, **upstream** kube adapter (EKS pattern). | Our Borg (`compute/`+`cell/`). A kubernetes.git port as operations. Node OS. Mesh. Public door. Empty `k8s-port/`. |
| **network** | Connect inside the cloud. | VPC, **private DNS**, TCP dataplane, SG, flow logs, volumetric DDoS, UDP/443 allowed. | Public door, public door DNS, CDN, L7 WAF (`gateway`). QUIC-EW. Istio. `firewall/`. Payload decrypt. Cell picker. |
| **gateway** | **One** north-south **contract**, many cell frontends. | Public H3/QUIC (H2 fallback). Maglev per cell. TLS/ECH/WAF/IAP. **PEP** then `policy/` Check. Public names + CDN SKU. L7 bot. | Mesh. Cedar engine. Connectors as door. REST/gRPC second API. One global VIP. QUIC MITM. Per-pod IAP. |
| **bus** | Move **events** (Pub/Sub / SQS / Service Bus). | Owned substrate: **queue** (competing consumers), **bus** (fan-out subscriptions), **stream** (seekable cursor); transactional **outbox**; at-least-once; per-key order. Serving `Check` never *is* a consume. | Sagas (`workflow`). Mailbox / chat (`app/`). **Kafka or Pulsar as `core/`**. SES send (`notify`). A root named `messaging/`. |
| **workflow** | Managed **sagas** (Step Functions / Cloud Workflows). | Rewrite: state machine, retries, timers, execution API; studio as authoring **facade**. | Bus (`bus`). Forms/tasks/SaaS. Deploy (`pipeline`/`iac`). Current tree (purged). |
| **intelligence** | Managed **inference** (Vertex / Bedrock). | Invoke, endpoints, batch, quota; hosted-agent **SKU**; adapters. GPUs from `compute/`. RAG facade over `data/`. | Copilot UX. Chat CLI/SDK core. GPU plant. GuardDuty. Vector store. Nested Cedar PDP. |
| **flags** | Dynamic config and kill switches. | Deterministic eval (`evaluation-domain`), targeting, kill switch. Pack gates via **C0 Cedar context**, not a pack fetch. Connect facade. | Experiment product. Clock adapter. REST/gRPC dual. OFREP as SSOT (adapter only). Helm. Cell topology. |
| **pipeline** | Productized execute (TAP internally, Cloud Build sold). | **One engine**, two facades: **polyglot** hermetic graph + queue (buck2 when CAS+RE live). Workers = `compute/`. Promotion graph execute. One required context `presubmit`. Tenant #0 is Rust-first; **customers are not**. | GHA as product. **Foundry Pipeline Builder** (`app/foundry` UX on `data/` engines). Cargo as sold runtime. JSON check product. Prow/Tide as core. A `ci/` root. |
| **iac** | Apply **desired state**. | IR unify/preview/apply/watch. Reconcilers call **cap APIs**, not kubectl. Helm/Tofu **adapters** only. | Merge queue. Sagas. Helm/Tofu as source. Argo as identity. |
| **billing** | Charge for **cloud use**. | Usage **events** from owning caps; rate; invoice; tax adapter; FOCUS. | Scraped metrics as the bill. Catalog of what exists (`marketplace`). Ledger. Card rails. |
| **marketplace** | Third-party **modules** on the cloud. | Signed plugins, Cedar install envelope, SKU **attach** (what exists). | Invoice (`billing`). KYC/escrow/payout. App/dataset store. `build/` price view. |
| **compliance** | Evidence **engine** + **CaS facade**. | Bind/project/export. Data-class registry. Projects pack retention into audit/data/storage. Apps are PEPs. | Merkle log. `packs/` data. DLP/eDiscovery/trust-portal products. Second PDP. Fetch-on-Check. |
| **notify** | Transactional **delivery** (SES / SNS / FCM). | Send email/SMS/push; bounce/complaint; DKIM/SPF/DMARC; optional inbound **to the bus** (SES-receive analog). | **Mailbox** (IMAP/JMAP/webmail), Meet, Messenger, calendar, contact-center — later `app/`. Emergency clinical. Current `comms/` tree (purged). |
| **packs/** (data, not a cap) | **CaC** — jurisdiction/program packages the engines load. | v1 namespaces `us`, `eu` (Union, **not a country**), `jp`, `kr`. Granular packages **projected** on any compliance dimension (Cedar Principal/Action/Resource/Context: client, action/txn, resource, routing/cell, purpose, acr, …). Check **unions** matching projections. CaS for first- and third-party apps. | A capability `core/`. Closed dimension catalog that cannot add routing/action. Blanket apply. Combinatoric ids. Markdown as CaC. Private pack path. |

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
- **origin:** `cloud/` was emptied; the cloud leaked into JSON specs and nested `oyatie-*` / `cloud-*` leftover dirs inside caps.
- **rule:** a cloud-provider engine occupies exactly one registered capability’s `core/`; sold single-cap surface is `facade/`; 2+ is `app/`; repo root does not hold IaaS dumps; **no new `cloud-*` crate, dir, or type name** (existing `CloudRegion` fossils burn with their dump, they are not a pattern).
- **ensure:** new engines get a registry row or a face, never `cloud/` or `libs/`; new crate names use the cap slug (`cell-clock-api`, not `cloud-clock`).
- **overturn_when:** a §7 split/merge ADR with five fields lands same-wave.

### D-19 — DO / DON'T × HAVE / HAVE NOT

Repo-root capabilities are the cloud provider. Every name is exactly one cell:

|  | **DO** (should exist) | **DON'T** (must never exist) |
|---|---|---|
| **HAVE** (on this branch) | **DONE** — keep. If dump/ports remain: **KEEP+WORK** | **REMOVE** — delete or rewrite **here**; do not move |
| **HAVE NOT** (absent) | **BUILD** — create in charter | **STAY GONE** — do not invent a home |

**DONE** is not “finished product.” KEEP+WORK = engine stays; nested dump is REMOVE; missing ports are BUILD. Law names the **DO** root (`bus/`, `pipeline/`), never the retired path.

No new `cloud-*` crates. REMOVE is not rehome.

**DONE** (DO + HAVE, engine matches charter)

- `cell/` engine + clock port (GNSS adapter still fail-closed — KEEP+WORK plant)
- `tenancy/` engine
- `iam/` who + `device_attestation` port
- `secrets/`, `audit/`, `observability/` engines
- `storage/` bytes; `data/` records (not `cloud-*`)
- `compute/` (one cap); `k8s/` managed-cluster **engine**; `network/` dataplane
- `bus/` kernels: queue + fan-out + stream + outbox (crate names `messaging-*` are KEEP+WORK rename, not a second slug)
- `intelligence/` inference
- `iac/` unifier; `billing/` platform meter; `marketplace/` plugin+SKU kernel; `compliance/`
- `flags/core/evaluation-domain`
- Meta: `docs/`, `build/`, `third-party/`, `packs/` (install authority), `templates/` (ADR + ritual only), `app/` (composition)

**KEEP+WORK** (DO + HAVE, additional REMOVE or BUILD inside)

- `iam/`: extract PDP → `policy/` (BUILD `policy/` same change); drop nested dumps; SVID issue → `secrets/`
- `k8s/`, `network/`: nested census REMOVE
- `data/`: ontology/Pages/Grid **product** trees REMOVE from core (OLTP/OLAP/pipeline engines stay); they BUILD under `app/foundry` — not `app/ontology`, not `data/`
- `intelligence/`: chat/CLI/SDK dumps REMOVE; invoke kernel stays
- `storage/`: `drive/` `recordings/` `imaging/` REMOVE
- `tenancy/`: KYC/DSR/JWT/nested PDP REMOVE
- `secrets/`: nested `kms/` dump REMOVE; one crypto root
- `audit/`: journey cedar novels REMOVE
- `observability/`: `diagnostics/` + Helm-as-identity REMOVE
- `billing/`: accounting/tax/portal/payment REMOVE
- `marketplace/`: escrow/deals/app-store dumps REMOVE
- `compliance/`: dlp/dsr/ediscovery/trust-portal REMOVE; BUILD CaS
- `iac/`: Argo/Tofu/Helm source REMOVE; reconcilers call cap APIs
- `flags/`: cap-root dump + REST/gRPC `server` REMOVE; eval stays
- `gateway/`: connector dump REMOVE; Connect door BUILD after; no Cedar engine
- `pipeline/`: path is `pipeline/`; contents are not the product (Prow/Tide/census). BUILD execute core; REMOVE JSON gates and Tide-as-core
- `bus/`: rename `messaging-*` crate ids; Connect facade BUILD
- `cell/`: PTP/GNSS bind when plant exists; `regional-pack` REMOVE
- `packs/`: KEEP us/eu/jp/kr; BUILD Cedar+IR loader; REMOVE markdown/OVH YAML and extra jurisdiction dirs (`au br cn in ksa mx`)
- `build/`: port-engine frozen; no price view
- `governance/check`: D-17 default delete

**BUILD** (DO + HAVE NOT)

- `policy/` — extract from `iam/` in the same change as the directory
- `gateway/` Connect/H3 door, TLS/WAF/IAP/fingerprint/ECH crates **after** connector REMOVE
- `workflow/` Step Functions — directory + `core/` in one PR
- `notify/` SES send — directory + `core/` in one PR
- `network/` `flow_log` + `quic_metadata`; `k8s/ports` `owned_journal`; `data/` `commit_wait`
- `base/` only when the ≥3-caps rule admits the first crate
- `app/foundry` — Palantir Foundry product (ontology + Pages + Grid + Workshop). No empty scaffold until that PR. Not a cap. Module of `app/application`.
- `app/application` — suite launchpad (from `oya/application`). Not console.
- `app/mail`, `app/messenger`, `app/payments`, `app/accounting`, `app/ledger` — v1 products; no empty scaffold until each PR

**REMOVE** (DON'T + HAVE)

- `gateway/` Workday/Slack/Salesforce/… connectors
- `flags/` dump (catalog.yaml, IPs, Helm, REST+gRPC server, experiment dashboards)
- Nested census; a root named `ci/` (retired; do not recreate)
- Repo-root leftovers **deleted:** `contracts/`, `plan/`, `tasks/`, `scripts/`, `libs/`, `infra/`, `tools/`, `kernel/`, `os/`, `oya/`, `evidence/`, `registry/`, `specs/`, `governance/`. Not shrink-only.
- A root named `messaging/` (retired; do not recreate)
- `cloud-*` crates; cap-root IPs, AUDIT-FINDINGS, Helm source, OpenAPI product, `catalog.yaml`

**STAY GONE** (DON'T + HAVE NOT)

- `cloud/`, `console/`, `comms/`, `time/`, `firewall/`, `k8s-port/`, empty `kernel/`/`os/`/`policy/`/`workflow/`/`notify/` scaffolds
- Island-class browser as a cloud root; `payments/` and `ledger/` as **caps** (products, §7); `foundry/` as a **cap** (Palantir Foundry is `app/foundry`)
- Kafka as `bus/` core; GHA as `pipeline/` core; Istio as identity; on-path QUIC MITM
- New `cloud-*` names; EU-as-world-floor; EU as a country; combinatoric
  pack ids; REST+gRPC as a standing product
- Search/detection/GPU/CDN as **roots** (vector = `data/` facade SKU; GPU = `compute/` SKU; CDN = `gateway/` SKU; DLP object = `storage/` SKU; client DLP = endpoint)

Apps: D-22/D-23. `app/sheets` → `app/foundry/grid`. `app/global-trade` deleted.

**MUST (DONE / KEEP+WORK / BUILD / REMOVE / STAY GONE; no cloud-* debt)**

- **achieves:** placement is a 2×2; DONE is not finished; law never treats a retired slug as live.
- **origin:** calling the bus `messaging/` in the ADR recreated the alias; dumps “moved” instead of dying.
- **rule:** DO+HAVE = DONE or KEEP+WORK; DO+HAVE NOT = BUILD in charter; DON'T+HAVE = REMOVE here; DON'T+HAVE NOT = STAY GONE; capability id is `bus/` not `messaging/`; no new `cloud-*`; REMOVE is not rehome.
- **ensure:** new crates land under the DO name (`bus/`, `pipeline/`); PRs that add STAY GONE names, a `messaging/` root, or rehome REMOVE dumps fail review.
- **overturn_when:** a §7 split/merge changes DO/DON'T with five fields same-wave.

### D-20 — Charter reconciliation (founder default A, 2026-08-22)

Interview on remaining collisions. Unanswered picker; **A** is the recorded default.

**Two reconcilers, not three.** `compute/` = Cloud Hypervisor VMs + Firecracker functions. Kill **`k8s-on-compute`**. GPU is a VM SKU. gVisor is an adapter. Sold kube is `k8s/` placing nodes **as** compute VMs (kubelet in the guest). AWS does not put EKS inside EC2.

**Ontology is not `data/core`.** `data/` = OLTP + OLAP + pipelines (records engines). Palantir Foundry placement is **D-21** (`app/foundry`, not a generic later app, not `data/`). Vector search, if sold, is a `data/` **facade SKU**, not a `search/` root.

**Intelligence is Vertex, not Copilot.** Invoke + endpoints + batch + quota. Hosted-agent **SKU** allowed. GPUs rented from `compute/`. RAG is a facade over `data/`, not a store. Chat CLI/SDK dumps are not core.

**Price has two owners.** `marketplace/` = **what exists** (plugin/SKU attach). `billing/` = **what you pay** (rate, invoice, FOCUS). `build/` is **not** a price list.

**Meters are usage events.** Owning caps emit VM-hour / GB-month / invoke onto billing ingest / `bus/`. Observability may display; it is not the bill.

**IAM consumes federation.** User-token issue is a Cognito-class **`iam/` facade SKU** if sold — not the kernel. Workload SVID **issue** stays `secrets/`. SCIM does not create tenants. Role store compiles to Cedar; `iam/` never Checks. No Zitadel-as-identity.

**Storage facades are S3 and EBS-class.** `drive/`, `recordings/`, `imaging/` REMOVE (later `app/`). Pipeline CAS lives in `storage/`.

**Marketplace is plugins + SKU attach.** No escrow, KYC, payout, app/dataset store.

**Gateway is a PEP.** Authn, WAF, quota, then `policy/` Check. No Cedar engine in `gateway/` (nor in intelligence/tenancy nested PDPs).

**Quota.** Cell = physical capacity. Compute = schedulable remainder. Marketplace SKU attach stored on tenancy; **owning cap enforces** (network refuses VPC 6). Billing = money. Tenancy does not enforce.

**DNS / CDN / DDoS.** Private DNS = `network/`. Public door names + CDN/cache = `gateway/` SKU. Volumetric DDoS = `network/`; L7 WAF/bot = `gateway/`. No `cdn/` root.

**Retention cascade.** `tenancy` sets deleting → `data` erases records → `storage` erases bytes → `audit` keeps what the pack requires → `compliance` **projects** via CaS. No fifth store.

**`base/`:** domain IDs stay on the owner cap (`TenantId`, `CellId`). `base/` only for non-domain primitives if three caps share one.

**`build/` port-engine:** frozen until a named owned corpus is Accepted. Not staffed as Borg.

**`packs/`:** consumed only via `compliance` CaS + `policy` C0. `cell/` and `flags/` do not load packs.

**MUST (D-20 reconciliation)**

- **achieves:** stop over-claiming products inside cloud caps; one owner per collision.
- **origin:** adversarial pass found k8s-on-compute vs EKS wrap, ontology-in-data, copilot-in-intelligence, three price lists, gateway PDP, meter-from-metrics.
- **rule:** D-20 defaults above are live reading on conflict with earlier D-11/D-14/D-15 slogans; no `k8s-on-compute`; no ontology in `data/core`; no gateway Cedar engine; no `build/` price view.
- **ensure:** new crates and registry charters match this section; PRs that reintroduce those phrases fail review.
- **overturn_when:** a five-field ADR same-wave names a different owner for any row.

### D-21 — Palantir Foundry is the product; ontology lives in Foundry, not in `data/`

Founder 2026-08-22: (1) Palantir Foundry ≠ the retired intelligence “foundry.” That intelligence/RAG Foundry is **dead**. (2) Ontology is **implemented in Foundry**, not inside `data/`. (3) Pages/Grid stay; D41 retirees stay dead.

**Three names, one live product.**

| Name | What it is | Fate |
|---|---|---|
| **Palantir Foundry** | Suite product: Ontology (heart) + Pages + Grid + Workshop/Manager UX. Sits **on** `data/` engines + `storage/` bytes. | **`app/foundry`** (BUILD; no empty scaffold until that PR). Not a cloud cap. Not `foundry/` at repo root. |
| Retired **`contracts/openapi/foundry`** / intelligence “foundry” | Old AI/RAG HTTP surface. | **Dead.** Do not revive. Not Vertex. Not this product. |
| Retired **“Foundry engineering platform” axis** | Agent DX vocabulary (ADR-0025 cluster). | **Dead.** Not Palantir Foundry. |

Palantir’s own docs: Ontology is *the heart of Foundry*, an operational layer **on** datasets/models — objects, properties, links, actions, functions. Ontology Manager is an app **inside** Foundry. AIP binds *to* ontology objects (= our `intelligence/` flag-off, D7) — it is **not** Foundry.

**OVERRULE (same-wave).**

- Interview **D6** “`data/` is the foundry-class root (datasets + pipelines + lineage + **ontology binding** + analytics); no rename to `foundry/`.” **Ontology binding leaves `data/`.** `data/` keeps datasets/pipelines/lineage/OLAP/OLTP **engines**. Foundry-the-product is `app/foundry`.
- Interview **D40** “Pages + Grid owned by `data/`.” **Pages + Grid are Foundry primitives**, implemented in `app/foundry`, consuming `data/` engines. Not Google Docs/Sheets as v1 standalone apps (wave 2 surfaces still allowed).
- D-20 “ontology is a later generic `app/`.” **It is `app/foundry`**, not `app/ontology` as a sibling product.

**D41 did not retire docs/sheets.** Retired: notes, slides, office, sites, translate (D42). `app/sheets` and `oya/docs` are early dumps of Grid/Pages — KEEP+WORK toward Foundry primitives, not D41 kills. Registry “sheets = ontology spine” is **false**: Ontology is the spine; Grid is a view on object-sets.

**Do not mix these five (founder 2026-08-22: `data/` is cloud records persistence, Foundry is the app).**

A tenant **can** buy `storage/` and `data/` **without** Foundry. Foundry **requires** them. Ontology is **not** implemented in `data/`.

| | Layer | Sells | Must not be |
|---|---|---|---|
| **`storage/`** | Cloud cap | **Bytes** (S3 / GCS / CAS). Identity = digest. | SQL. Ontology. Drive app. Foundry. |
| **`data/`** | Cloud cap | **Records infrastructure** (RDS / Spanner / BQ / Dataflow **engines**). Sold as IaaS/PaaS. | Bytes (`storage/`). Ontology kernel. Pages/Grid. Palantir Foundry. TAP (`pipeline/`). |
| **`app/foundry`** | **App product** | Ontology (heart) + Pages + Grid + Workshop / Ontology Manager. Pipeline Builder **UX** that *calls* `data/` engines. | A cap. A `foundry/` root. `data/core`. Intelligence. TAP. |
| **`pipeline/`** | Cloud cap | TAP / Cloud Build **execute**. | Foundry Pipeline Builder. Dataflow engine. |
| **`intelligence/`** | Cloud cap | Vertex invoke (AIP **on** Foundry objects, flag-off). | Foundry. Ontology store. Dead RAG “foundry.” |

Pipeline Builder ≠ `pipeline/`. Object Storage (Foundry) ≠ `storage/`. Google Vertex ≠ Palantir Vertex.

No empty `app/foundry/` until the BUILD PR. Ontology crates today under `data/` **move** in that PR — they do not stay as `data/core` and they do not become a `foundry/` capability.

**MUST (Foundry product, ontology in Foundry)**

- **achieves:** Palantir Foundry is one product; ontology is its heart; `data/` stays cloud records engines; intelligence Foundry stays dead.
- **origin:** D5 named Palantir Foundry as the suite spine; D6 parked ontology in `data/`; D-20 parked it in a generic app; founder 2026-08-22 put ontology **in Foundry**, not in `data/`.
- **rule:** `app/foundry` owns ontology + Pages + Grid + Workshop + Manager + Pipeline Builder UX; `data/` is sold records **engines** without Foundry; `storage/` is bytes; TAP is `pipeline/` not Foundry pipelines; AIP is `intelligence/` not Foundry; no `foundry/` cap; D41 list is notes/slides/sites/office only. Apps call those engines through adapters (D-23).
- **ensure:** new ontology crates land under `app/foundry` (when it exists); PRs that implement ontology in `data/core`, merge `data/` into `storage/`, put Pipeline Builder in `pipeline/`, or revive intelligence foundry fail review.
- **overturn_when:** a five-field ADR same-wave names a different Foundry home.

### D-22 — Apps: one shell; v1 People + Finance (shrunk)

Founder 2026-08-22: (1) **`app/application`** is the launchpad for the whole suite; **`app/foundry`** is a **module** in that shell (Palantir Workspace hosts Foundry — Foundry is not the only shell). D-16 stands: not `console/`, not `app/ops-console`. (2) Interview D1 Finance+People clusters stay the **shape**, but v1 **drops** FP&A, treasury, performance-management, learning-management. That is D1-A shrunk toward “B plus payments.”

**V1 product dirs** (BUILD when missing; no empty scaffolds; KEEP+WORK if HAVE):

| Dir | Role |
|---|---|
| `app/application` | **Shell / launchpad** (move from `oya/application`) |
| `app/foundry` | Foundry module **v1 full**: ontology + Pages + Grid + Workshop + Manager + Pipeline Builder **UX** — D-21/D-23 |
| `app/hr` | People |
| `app/payroll` | People |
| `app/accounting` | Finance **UI** (GL/close, statements, AR/AP). |
| `app/ledger` | Posting **engine** product (universal journal). Not a cap. Not `billing/`. |
| `app/payments` | Processor + execution (Stripe analog). **Lowest v1 priority.** Not `billing/`. Not a cap. |
| `app/calendar` | Events + availability. **Embeddable** in other modules (PTO, payroll run, …). |
| `app/mail` | **Mailbox**. Not notify. |
| `app/messenger` | **One dir**, dual-context: Slack-superset **professional** + Discord/WhatsApp-class **personal**. Meet inside. Not `app/social`. |
| `app/community` | **One dir**, dual-context: TeamBlind-class **professional** + Reddit-class **personal**. No SecureDrop v1. |
| `app/drive` | People files/folders/sharing. **Not** `storage/`. Maps to `storage/` via adapter. BUILD. |

**Not v1** (no dirs, not membership ghosts): `app/treasury`, `app/financial-planning`, `app/performance-management`, `app/learning-management`, and every registry SAP ghost (`crm`, `itsm`, `warehouse`, …). D1 can grow them later. **Do not create empty `app/<ghost>/`.**

**3A.** `app/ledger` = journal engine; `app/accounting` = accountant UI; cloud `billing/` invoices **cloud** SKUs (D37: Oyatie as tenant #0 later, then billing’s internal journal can die).

**4A.** Community KEEP+WORK **shrunk**: drop SecureDrop/whistleblower from v1.

**5A.** Personal network is the same `app/messenger` engine (deny-default dual-context). No second product dir. D32 stands (`app/social` dead).

**6A.** Drop SAP ghosts from membership. Live `oya/*` dumps that still exist stay mapped until REMOVE/move, then disappear — they are not a roadmap catalog.

**Dump map (founder 2026-08-22: refactor ≠ KEEP+WORK; REMOVE means delete).**

| Tree | Class | Action |
|---|---|---|
| `oya/application` → `app/application` | **REFACTOR** | `git mv` (done) |
| `oya/payments` → `app/payments` | **REFACTOR** | `git mv` (done) |
| `oya/docs` → `app/foundry/pages` | **REFACTOR** | `git mv` (done) |
| `app/sheets` → `app/foundry/grid` | **REFACTOR** | `git mv` (done) |
| `oya/intelligence` | **REMOVE** | deleted |
| `oya/governance` | **REMOVE** | deleted |
| `oya/global-trade`, `app/global-trade` | **REMOVE** | deleted |
| `storage/drive`, `storage/recordings`, `storage/imaging`, `storage/facade/{drive,recordings}` | **REMOVE** | deleted |
| `observability/diagnostics` | **REMOVE** | deleted |
| `iam/consent-graph` | **REMOVE** | deleted |
| `oya/` root | **REMOVE** | empty after moves; deleted |

**MUST (one shell; shrunk v1 People+Finance; ledger product; no ghosts)**

- **achieves:** one launchpad; Foundry is a module; v1 money/people set is small enough to staff; ledger is not billing and not a cap.
- **origin:** D23/D35 one shell vs D-16 no console; D1 vs founder drop of four modules; D15/D37 two ledgers; D38 personal messenger; census-like app membership lists.
- **rule:** `app/application` is the only shell; `app/foundry` is a module in it; v1 People = hr+payroll (employment + pay-run); v1 Finance = accounting + **ledger** + payments (payments last); community and messenger are each one dir dual-context; `app/drive` is an app; no empty ghost dirs; `ledger/` is not a cap.
- **ensure:** D-22 table + D-8 `app/` children are the roster (no JSON membership hub); no `console/` / `app/ops-console` / `app/social` / empty `app/crm`; Foundry PRs do not replace the launchpad.
- **overturn_when:** a five-field ADR same-wave changes the shell, v1 roster, or ledger home.

### D-23 — Apps are tenants; cloud is SKUs behind adapters

Founder 2026-08-22: dump **all** console (including application `tenant-admin-console`). Foundry v1 is the **full** suite. Identity pattern is the template: **cloud provider and tenant are separate**. First-party apps dogfood as tenant #0. They may consume cloud capabilities **only behind an adapter**. In-process `use <cloud>_core` from `app/` is vendor lock-in to ourselves.

**Proposal (adopted).** Every `app/*` is a tenant product. Every cloud cap is a sold SKU (`iam`, `storage`, `data`, `pipeline`, `notify`, `bus`, `secrets`, …). The only coupling is the **public** Connect/proto facade the cloud already sells. Cloud crates do not import `app/`. App crates do not import cloud `core/` / `ports/` as libraries. `app/*/adapters/` talk to cloud facades the way an external tenant would. Substituting an external S3/Stripe/IdP is the same port with a different adapter.

**Foundry (v1 all in).** Ontology, Pages, Grid, Workshop, Ontology Manager, Pipeline Builder **UX** do the product work **inside** `app/foundry`. They **settle** to cloud SKUs:

| Foundry work | Settles through adapter to |
|---|---|
| Object instances, datasets, OLAP, lineage facts | `data/` records engines |
| Files, attachments, workbook bytes | **blob port** → `storage/` or on-prem adapter (D-24) |
| Pipeline Builder *run this graph* | **`data/` dataset jobs** — not `pipeline/` TAP (D-24) |
| Who | `iam/` (federation; Foundry users are not cloud principals) |

Ontology kernels today under `data/core/` **move** in the Foundry BUILD PR and then call `data/` through the adapter — they do not stay as a cloud ontology engine and they do not become a `foundry/` cap.

**Console.** `console/` and tenant-admin-console dumps are **REMOVE**. Tenant policy/roles live in each product + cloud IAM, not an ops-dashboard cap and not a shell “control plane” product.

**Calendar.** `app/calendar` is the suite scheduling engine (events, availability, recurrence). Other modules **embed** it (HR PTO, payroll run calendar, Foundry due dates). They compose `app/calendar` as an app port — not `cell/` TrueTime, not a cloud cap. Meet stays in messenger.

**Mail vs `notify/`.** `app/mail` = **mailbox**. Cloud `notify/` = **notification delivery** (email send, SMS, push, messenger ping). Notify may use mail as a *channel* without being a mailbox. Apps raise notifications through the notify adapter.

**Messenger.** One dir. Professional = Slack-superset. Personal = Discord/WhatsApp-class. Deny-default dual-context (D38). Meet inside.

**Drive.** Cloud `storage/` = buckets/bytes (S3 analog). `app/drive` = people Drive (folders, sharing). Drive **maps** to storage through an adapter. Not a storage dump (`storage/drive` stays deleted). Not Foundry Pages (those are ontology documents).

**Payments (lowest v1).** `app/payments` is the processor + merchant product (Stripe analog) **and** a port for an *external* processor adapter. `billing/` remains cloud SKU invoices. Later: banking-API sync so payroll can payout through payments — still an **app** adapter, not `billing/core`. Do not staff payments ahead of ledger/hr/foundry.

**Community.** One dir. Professional = TeamBlind-class. Personal = Reddit-class. Dual-context. No SecureDrop v1.

**Packs.** OVERRULED by D-24: not one `packs/kr` novel. Overlay **content** per owner; `packs/` is thin **install** authority (pack-id).

**MUST (tenant apps; adapters; one pack engine)**

- **achieves:** first-party apps are customers of the cloud; Foundry is not a backdoor into `data/core`; packs are one CaC plane.
- **origin:** cloud/app fusion (ontology-in-data, Drive-in-storage, console-as-cap, payroll-law-in-iam).
- **rule:** `app/` ↔ cloud only via sold facades/adapters; no in-process cloud `core` from apps; Foundry v1 is the full module set and persists via data/blob/iam adapters; Foundry Pipeline Builder is **not** TAP; console dumps gone; calendar embeddable; mailbox ≠ notify; messenger and community each one dual-context dir; `app/drive` over the **blob port**; payments last in v1; packs = thin install authority + per-owner overlay content (D-24).
- **ensure:** new `app/` crates that `path =` a cloud `core/` or `ports/` fail review; tenant-admin-console files stay deleted; pack PRs do not add a second pack reconciler or a git JSON overlay SSOT.
- **overturn_when:** a five-field ADR same-wave allows in-process cloud cores for tenant #0 or a second pack engine.

### D-24 — Packs split; SQLite then cloud; TAP ≠ Foundry pipelines; mailbox + blob ports

Founder challenge 2026-08-22. D-23 stood except where this OVERRULES it.

#### Packs: per-capability content, central **install** authority

A single `packs/kr` novel with `cloud.*` and `app.*` slices **recouples** every cap PR (the JSON product we just burned). Per-cap overlays with **no** shared tenant→jurisdiction fact **split-brain** (storage US, payroll KR).

**Decouple.** Overlay **content** lives next to the owner that evaluates it: `storage/` KR residency Cedar, `app/payroll` KR filings, `iam/` proofing. Independent PRs. One overlay **format** (Cedar + typed proto config). Not N DSLs.

**Connect.** Thin cloud cap `packs/` is **install authority only**: tenant T (cell) has pack-id `kr@v3`. Runtime object, not a git census. Caps and apps ask `packs/` (adapter) “what pack is T on?” then load **their** overlay for that id. Default: one pack-id per tenant/cell (split-brain is explicit, not accidental).

IaaS-only tenants install a pack-id whose owners have only `cloud.*` overlays. App data-at-rest still needs the storage owner’s overlay (payroll cannot place bytes).

**Not:** resurrect `governance/capability-registry.json`. **Not:** `app/payroll/packs/` as a second reconciler.

#### Runtime state is not git plaintext

ADRs, README, proto, Cedar **as source** stay git. Instance data (objects, mail, drive files, pack **installs**, ledger rows) does **not**. v1 adapters: **SQLite**. Destination adapters: `data/` (records), blob port (bytes), on-prem. Ports exist on day 1 so SQLite is not a data model. SQLite is not the D-1 serving path (10^8 checks stay RAM snapshots).

#### `pipeline/` is CI/CD only

OVERRULE D-23 “Pipeline Builder → `pipeline/` TAP.” Cloud `pipeline/` = TAP / Cloud Build / automation **of software**. Foundry Pipeline Builder = **dataset transforms** → `data/` job engine. Two English “pipelines”; **one slug** (`pipeline/`) is CI. Foundry does not embed TAP. TAP is not ontology-aware.

#### Mailbox port (not a casual `mail/` root)

`app/mail` is the client (Gmail analog). Store + ingress/egress is a **Mailbox port**: adapters IMAP, JMAP, SMTP, and Connect/protobuf on **one** store. v1 = SQLite mailbox (self-contained). Destination = sell hosting as facade of `data/`+blob+`network/` (MX/spam is the only reason to add a cloud `mail/` engine later). `notify/` stays delivery, not IMAP.

#### One blob port (Drive, Foundry, mail attachments)

On-prem / off-cloud / customer MinIO is **why** the port exists. Drive, Foundry bytes, and mail attachments share **one** blob port. Placement is pack-id + tenant config, not a per-app side door to `storage/core`. Our `storage/` is one adapter.

#### Challenges that did not overturn the rest of D-23

- **Foundry “all in v1”** is a company. Charter stays full-suite; **success** is ports + ontology + Pages/Grid actually persisting through SQLite adapters — not feature-parity Palantir on day one. Workshop/Manager/Pipeline Builder UX may be thin, not absent from the charter.
- **Calendar embed** must not become `hr` importing `calendar` core. Same adapter rule **between apps** (Connect), or we built a second cloud.
- **Dual-context** one dir is a lie if professional/personal are two codepaths. Shared engine or two binaries is an implementation choice; one **dir** and one dual-context policy stay.
- **Notify** is delivery. It does not own messenger threads. A “messenger ping” is notify → messenger adapter.
- **Payments last** vs payroll payout: v1 payroll may mark paid without a processor. Do not fake Stripe.
- **SQLite everywhere** is a trap if Check/ReBAC tuples only live there (D-1).

**MUST**

- **achieves:** caps/apps ship overlays without a god pack file; tenant jurisdiction is one fact; apps persist without git; CI ≠ Foundry data jobs; mail/drive can leave our storage.
- **origin:** D-23 one-file packs; Pipeline Builder mis-settled to TAP; plaintext/JSON as databases; Drive assumed our S3.
- **rule:** `packs/` = install authority (pack-id); overlay content per owner; runtime state via ports (SQLite v1 → data/blob/on-prem); `pipeline/` is CI/CD; Foundry Pipeline Builder → `data/`; Mailbox port (IMAP/JMAP/SMTP/Connect); one blob port for Drive/Foundry/mail; on-prem is an adapter.
- **ensure:** D-23 settle table matches this; no git JSON pack SSOT; no `path =` storage core from Drive/Foundry.
- **overturn_when:** a five-field ADR same-wave restores a monolithic pack file, TAP-as-Foundry-pipelines, or git as the object store.

### D-25 — Clean architecture: app core is portable; cloud is an adapter

Founder 2026-08-22 (clarified): maximize **business logic** behind ports. Cloud and apps are separate. **“Instantly portable” is not HA during our downtime.** It is **end of service** of a cloud capability (we retire the SKU, the tenant leaves, or they never bought it): the **app must keep serving** by pointing the same ports at **existing technologies** (Postgres, S3/MinIO, IMAP, Stripe, on-prem) **without a significant rewrite**.

**Shape (ADR-0562 faces, app edition).**

| Face | Holds | Must not hold |
|---|---|---|
| `app/<p>/core` | Domain + use cases (ontology rules, payroll calc, ledger postings, calendar recurrence, mail labels) | SQLite, HTTP, S3, IAM, `storage::`, `data::` |
| `app/<p>/ports` | Traits: records, blob, mailbox, identity, notify, pack-id, payments, calendar-embed | Adapter impls |
| `app/<p>/adapters` | SQLite v1; our-cloud Connect client; Postgres/S3/IMAP/Stripe/on-prem | Business rules |
| `app/<p>/facade` | That product’s UX/API | Cloud SKU implementation |

**One active adapter per port per tenant.** Not dual-write. Between apps (hr → calendar): **ports**, not `use calendar_core`.

**Portable means.** Core source does not change when the substrate does. Proof: the same Foundry/hr/payroll tests run against SQLite and against a commodity adapter (Postgres, S3 API). Rewriting ontology/payroll because we stopped selling `storage/` is a D-25 fail. Live outage behavior is **not** this decision (that is SLO/cell failover of whatever adapter is selected).

**MUST (portable app cores)**

- **achieves:** retiring or unbundling a cloud SKU does not kill tenant apps; they keep running on commodity adapters.
- **origin:** in-process cloud cores; “just call storage/”; Foundry would die if we EOL’d a cap.
- **rule:** app business logic only in `core`; every IO through a port; v1 SQLite adapter per durable port; our cloud is one adapter among Postgres/S3/IMAP/Stripe/on-prem; no `path =` cloud `core/`/`ports/` from `app/`; between-app composition is ports too. Not an HA-during-outage claim.
- **ensure:** new `app/<p>/core` crate that imports sqlx/reqwest/`storage-*`/`data-*`/`iam-*` fails review; each durable port has a non-Oyatie adapter path in charter (SQLite and/or commodity).
- **overturn_when:** a five-field ADR same-wave allows in-process cloud cores or makes an app require a living Oyatie SKU with no substitute adapter.

### D-26 — Do not add a trusted-tenant *mode* (premise rejected)

Founder 2026-08-22: maybe privileged access as a trusted tenant vs untrusted, to prove the cloud works — flag or otherwise. **Do not weaken or add a regret.**

**The premise is surplus.** D-23 already proves the cloud (apps call the sold facade). D-25 already proves portability (commodity adapters at SKU EOL). IAM already issues service principals. A named trusted/untrusted **mode in the app** is a second path. Second paths become skip-PDP, unmetered, extra proto, or `cfg` linking `storage/core`. That is a product regression.

Hyperscalers do not ship “trusted GCS for Gmail.” One API, identities, ACLs. Console is not an enum inside S3.

**Do not add:** feature flag; `TrustedTenant` in `app/*/core`; first-party quota class; skip-PDP; second cloud API; in-process cloud cores.

**Already exists (do not rebrand as trusted mode):** adapter injection (sqlite | cloud-client | s3 | …); IAM principal including first-party; cell-local VIP as **network**, not an app bit; customer-data access uses **that customer’s** grant.

**MUST**

- **achieves:** no regret mechanism that splits dogfood from customers or bypasses D-23/D-25.
- **origin:** “prove it works” invited a second path; second paths become the only path.
- **rule:** no trusted-tenant mode, flag, or skip-PDP. Proofs remain sold facade + commodity adapters. First-party is an IAM principal. Adapter injection is substrate choice, not privilege.
- **ensure:** review rejects `TrustedTenant`, `cfg(trusted)` cloud-core path-deps, first-party unmetered classes, and privileged extra APIs.
- **overturn_when:** a five-field ADR same-wave shows a missing capability that IAM+adapters cannot express AND names a fail-closed alternative that is still one proto.

### D-27 — Docs live with the owner; root `docs/` is thin (g3doc)

Hyperscaler analog: Google **g3doc** next to the package; org-wide developer
guides stay central; customer manuals are a different product. Chromium: same
CL as the code. Not a 1,490-file root wiki (Google already failed that as
GooWiki).

**Per owner** (`<cap>/docs/`, `app/<p>/docs/`): engineering docs for *this*
tree — concepts, runbooks, design notes. Same inner grammar as the D-8 table.
Owner OWNERS may amend these without architecture review (D-29).

**Repo root `docs/`:** `AGENTS.md` operating contract pointer, unique
`docs/decisions/` (07xx live ADRs), `docs/standards/`. No per-cap copies of
ADRs. No catalogs, IPs, scorecards, census JSON.

**Do not:** mass-move the old wiki this wave; put Foundry user manuals here
(that is a later product site); resurrect `plan/` or `tasks/` as `docs/`.

**MUST (g3doc)**

- **achieves:** docs that describe a service change in the same PR; org law has
  one home; owner docs cannot fork ADRs.
- **origin:** root `docs/` became GooWiki; cap context was unfindable; founders
  asked whether per-owner docs match hyperscaler practice (they do: g3doc).
- **rule:** owner engineering docs under `<owner>/docs/` with the closed inner
  grammar; root `docs/` is thin law; unique ADR home; no IP/catalog resurrection;
  no same-wave 1,490-file move.
- **ensure:** D-8 allowlist includes `docs/` as a cap/app extra; layout rejects
  `plan/`, `tasks/`, `IPs/`, `decisions/` copies under owners.
- **overturn_when:** a five-field ADR names a different docs home that still
  co-locates service docs with code and keeps a thin org-law tree.

### D-28 — Shared contracts: draft vs agreed (ports + adapters)

Caps and apps bind **only** through ports and adapters (D-23/D-25). There is
**no** `contracts/` root and **no** `libs/ports`.

**Local / unagreed.** A team may invent a port under `ports/draft/<port>/`
(crate `owner-port-draft`). The word `draft` is in the **path and crate name**
so it cannot be mistaken for an agreed contract. Rename is `git mv`. Other
owners **must not** depend on it. Sold proto packages **must not** contain
`draft`.

**Shared / agreed.** The second owner that needs the same shape **cannot** copy
the draft. Reconcile onto **one** name on the **provider** (`storage-blob`,
`data-records`, mailbox, clock, …) using the `owner-port` grammar, plus the
sold proto `*.v1` if it is a facade. Promotion is `git mv` of the draft onto
the provider, not a fork. Google analog: AIP-181/185 — `v1alpha` is expected
to break; `v1` is the compatibility surface. We use `draft` internally so
rename stays cheap; customer-facing proto uses `v1` / `v2`, not `v1draft`.

**Escalated review** (D-29) is what turns draft → agreed. Observation that two
trees “look similar” is not agreement.

**MUST (contract stages)**

- **achieves:** unagreed shapes are grep-visible and cheap to rename; agreed
  names are the compatibility surface; no third contracts tree.
- **origin:** hexagonal ports proliferate incompatible `blob` traits; shared
  JSON/IDL dumps drifted; founder required distinguishable unagreed contracts.
- **rule:** bindings are ports+adapters; `ports/draft/` is local and not a
  dependency; a second consumer forces one agreed provider port + proto v1;
  no `contracts/` or `libs/`; draft illegal in sold proto.
- **ensure:** review rejects cross-owner `path =` into `ports/draft/`; agreed
  crate names match `owner-port`; layout rejects a `contracts/` root.
- **overturn_when:** a five-field ADR names a different staging that still
  makes unagreed contracts visually distinct and keeps shared names reviewed.

### D-29 — Amendment jurisdiction (owner-local vs external vs repo root)

**Same structure, different blast radius.** Every cap and every app uses the
D-8 children. Who may **amend** them is not the same.

| Blast | What | Who reviews | Analog |
|---|---|---|---|
| **Owner-local** | `core/`, `ports/draft/`, local `adapters/draft/`, `cedar/`, `docs/` (including `docs/design/`), `iac/` that only this engine consumes | That owner's `OWNERS` | Google package OWNERS / g3doc CL |
| **External contract** | Agreed `ports/<port>/`, adapters other owners consume, sold `facade/` proto, any crate another owner `path =`s | **This owner + every consuming owner + architecture** | Google API review / AIP; not a drive-by |
| **Repo root** | `AGENTS.md`, `CLAUDE.md`, `docs/decisions/ADR-07xx`, D-8 allowlist, `rust-toolchain.toml`, workspace membership policy, required `presubmit` | Architecture (+ founder on law) | Central developer guides; not a cap feature PR |

Owner-local **includes** amending this owner's docs, design notes, and
`ports/draft/` **content**. It is **not** a license to change the canonical
children, inner crate files, crate grammar, or to add `plan/`, `tasks/`,
`crates/`, `domain/`, or a private ADR tree. The structure is the same for
every owner and does not evolve per team. Repo-root law and agreed contracts
**do not** land as a side effect of a feature PR.

**MUST (jurisdiction)**

- **achieves:** teams move inside a frozen tree; they cannot silently bind the
  rest of the company, rewrite org law, or fork layout.
- **origin:** local iteration blocked by org review; conversely, shared ports
  and `AGENTS.md` edited from a cap dump; teams inventing `domain/`/`plan/`
  inside “their” root.
- **rule:** owner OWNERS for **content** in canonical children that do not leak;
  the D-8 shape and D-30 grammar are not owner-amendable; escalated review for
  agreed ports/proto/facade and for repo-root law; `plan/`/`tasks/` stay gone.
- **ensure:** PRs touching agreed ports, sold proto, or root law name the extra
  reviewers; layout rejects non-canonical children and invented inner taxonomies
  even on owner PRs.
- **overturn_when:** a five-field ADR replaces this split with an equally
  fail-closed OWNERS + API-review model.

### D-30 — Names and inner files: Cargo + google3 + AIP (not invented taxonomy)

The D-8 **tree is frozen**. Names *inside* that tree follow conventions that
already exist. Do not invent a per-repo dialect.

**Cargo package vs rustc crate (RFC 940, API guidelines C-CASE / RFC 430).**

| Thing | Convention | Source |
|---|---|---|
| `[package] name` | kebab-case, `[a-z0-9-]+`. Grammar above. No `-rs`, `-rust`, `oyatie-`, `cloud-` | RFC 940; C-CASE; AWS SDK `aws-sdk-s3`, tokio `tokio-util` |
| rustc crate / `use` path | hyphen → underscore. **Omit** `[lib] name` so Cargo does this | RFC 940 |
| Directory leaf | last grammar token: `blob`, `blob-sqlite`, `pdp-app` | google3 path identity; matklad: don't strip prefixes *from the Cargo name* |
| Cargo name | full `owner-port` / `owner-port-backend` / `owner-app` (Cargo's namespace is **flat**, so the prefix lives in the name) | matklad large workspaces; not a `crates/` dump — faces stay D-8 |
| Modules / files | `snake_case.rs`; `src/lib.rs` (lib), `src/main.rs` (facade bin) | RFC 430; Cargo book |
| Types / traits | `UpperCamelCase` (`BlobStore`, not `BLOBStore`) | RFC 430 |
| fns / methods | `snake_case`; getters are `blob()`, not `get_blob()` | C-GETTER |
| Consts / statics | `SCREAMING_SNAKE_CASE` | RFC 430 |
| Cargo features | additive; named `std` not `use-std` / `no-foo` | C-FEATURE |
| Tests | unit next to code; integration in `tests/` | Cargo book |
| Proto package | `owner.api.v1`; directory **matches** package; files `snake_case.proto` not `v1.proto` | AIP-191, AIP-185 |
| Proto messages / RPCs | `UpperCamelCase` | AIP-190 |

**Why not rust-analyzer's flat `crates/`.** That layout is right for a *single*
Cargo project (hir, hir_def, …). This repo is a google3-shaped capability
monorepo: path is identity (`storage/ports/blob`), Cargo names carry the
owner prefix because Cargo cannot express `storage::blob`. Fuchsia/GN and
google3 work the same way: hierarchical path + flat target/package name.

**Why not `domain/` / `use_case/`.** Those are Uncle Bob Clean Architecture
folders. They are not Cargo, not google3, not rustc, not Oxide/tokio. Core
is a lib crate; ports are sibling crates; adapters implement ports. Extra
module names are ordinary `snake_case` as the code needs them.

**MUST (established names)**

- **achieves:** one grammar humans and cargo already know; rename stays a
  `git mv` of a leaf that matches the last token; proto path = package.
- **origin:** D-8 needed inner files; an invented `domain/use_case/connect`
  taxonomy is not hyperscaler or Rust-workspace practice.
- **rule:** RFC 430/940 + dir leaf = last token + package = full grammar +
  AIP-191 proto under `facade/proto`; no `[lib] name` override; no
  `-rs`/`oyatie-`/`cloud-`; no required `domain/`/`use_case/`/`crates/`.
- **ensure:** new crate PRs match the table; proto dirs match `package`;
  layout still rejects extra faces.
- **overturn_when:** a five-field ADR cites a different *established*
  (Cargo or AIP or google3) convention and lands same-wave.

### D-31 — Ephemeral out-of-tree sandbox (owner-writable only)

D-29 is policy. This is **physical**. A default implement/review worker
must not be able to edit `iam/` while dispatched to `storage/`.

**Hyperscaler analogs (use, don't cargo-cult).**

| Practice | What it actually does | We take |
|---|---|---|
| Google CitC / Piper workspace | Full *view*, overlay only of dirty files; OWNERS on submit | Out-of-tree workspace; **not** “hide the rest of google3 from reads” — CitC does not |
| git cone sparse-checkout + worktree | Working tree materializes only named dirs ([GitHub](https://github.blog/open-source/git/bring-your-monorepo-down-to-size-with-sparse-checkout/), git-scm) | Default cone = one owner |
| Bazel action sandbox | Process sees **declared inputs**; Linux namespaces / macOS `sandbox-exec`; undeclared writes die ([Bazel sandboxing](https://bazel.build/docs/sandboxing)) | Write-jail + declared read set |
| Docker AI / agent sandboxes | VM or Landlock/Seatbelt; workspace-write vs full-trust | Landlock (Linux) / Seatbelt (macOS) when the runtime has it |
| Codex/Claude worktrees | Isolated checkout per agent, destroy later | Ephemeral; not the human clone |

**Default worker sandbox (one owner).**

1. **Out of tree.** `git worktree add` under a temp path (`/tmp`, `/private/tmp`, …). Never the human clone (`integ/audit` or `~/Developer/oyatie`). Already required; this names why.
2. **Ephemeral.** Create on dispatch. Remove on PR merge, lane abort, or idle expiry. Leftover worktrees are bugs, not inventory.
3. **Writable cone.** Maximum is one `<cap>/` or `app/<product>/` named in the dispatch (D-29). Parallel subagents **narrow** that to disjoint **leaf crate** dirs (D-32). Plus the worktree's own `.git` / index so git works. One git index per worktree — two writers in one worktree are forbidden.
4. **Read-only declared inputs** (not “the whole company” as writes). Root law the agent must load (`AGENTS.md`, `CLAUDE.md`, `rust-toolchain.toml`, workspace `Cargo.toml`/`Cargo.lock`, thin `docs/decisions/` for live 07xx). Agreed ports/facades this owner **already** `path =`s (D-28). Toolchain. `/tmp` for build scratch.
5. **OS write-jail** when the host can: Landlock path-beneath on Linux, Seatbelt `sandbox-exec` on macOS (Bazel Darwin sandbox, Anthropic/DeepSeek agent runtimes). Writable = owner cone + tmp + target dir. Everything else read-only or absent. The agent **cannot** widen this set.
6. **Local proof is buck2** (D-32 / ADR-0716). Not `cargo nextest --workspace`, not N cargo processes. Optional `cargo nextest -p <one crate>` is local feedback only. Full-graph cargo stays CI.

**Rejected (would regret).**

- **Hide other owners from all reads.** Cargo path-deps and D-28 reconciliation need to *see* the agreed provider port. Blind agents duplicate `blob` traits. CitC shows the repo; OWNERS bind submit. We jail **writes**.
- **A VCS ratchet product** (claim/verify/done). ADR-0363. Sandbox is dispatch plumbing, not a merge context.
- **JSON census of allowed paths** as a gate fleet. D-17. The cone is the dispatch argument, not a repo-wide list to freeze.
- **Agent self-service `sparse-checkout add iam/`.** Expanding the sandbox is a **new dispatch** (D-29 escalated), with extra writable cones named by the dispatcher.

**Escalated D-29 lane.** Writable cones listed explicitly (this owner + consuming owner, or repo-root files named). Still out-of-tree, still ephemeral, still not “the whole monorepo.” Coordinator/architecture that must survey many trees is a different profile — read-mostly, not a default implementer with a full write mount.

**MUST (owner sandbox)**

- **achieves:** a worker physically cannot mutate another owner's tree; D-29 is enforced by FS not by hope; parallel lanes do not share a checkout.
- **origin:** worktrees existed but were full clones; agents edited across caps; founder asked for ephemeral out-of-tree sandbox keyed to ownership.
- **rule:** default worker = ephemeral out-of-tree worktree; writable = dispatched owner only; OS write-jail when available; declared read-only inputs for law + existing path-deps; agent cannot expand the cone; D-29 extra cones are dispatcher-named; no ratchet product; no full-read hide.
- **ensure:** dispatch records the owner path; writes outside it fail or are review-blocking; worktree is deleted when the lane ends; human clone stays untouched.
- **overturn_when:** a five-field ADR names a stricter isolation (e.g. per-crate Bazel-style input set) that still lets Cargo and D-28 work, or a fail-closed alternative that still keeps writes owner-local.

### D-32 — Parallelism is a leaf crate + buck2 locally; cargo is the linearized merge

The cargo **workspace** is the merge graph (ADR-0716). It is not an isolation
mechanism. Two agents in one workspace still collide on:

| Shared file | Who may write it live | Why |
|---|---|---|
| A leaf crate's `src/`, `Cargo.toml` | **One** lane | Git merge of the same crate is the conflict you already feel |
| Root `Cargo.toml` members | **One** lane (D-29) | Adding a crate is org-graph |
| `Cargo.lock` | **One** lane, only when a crates.io / version pin changes | N cargo processes regenerate it; path-dep-only work must not touch it |
| One worktree `.git/index` | **One** process | Two subagents in one checkout serialize on `index.lock` even on disjoint files |

**Unit of parallel write (OVERRULED by D-39):** not the whole crate.
Crate → files/modules → one primary Item per file (D-35). N agents on
the **same crate** is the point. Same **file/Item** → assign disjoint
or stack that item; do **not** lock the crate.

**Local (N-way).** `buck2 build` / `buck2 test` of the dispatched targets.
Hermetic per action; no `Cargo.lock` rewrite; no workspace-wide cargo lock.
That is why buck2 exists here (ADR-0716). Reindeer still reads `Cargo.toml` as
manifest input — do not invent a second package graph.

**CI (1-way).** Merge queue / `presubmit` runs **one** `cargo nextest --workspace`.
By then lanes are linearized; one lockfile writer. Cargo at CI is therefore
**not** the N-parallelism problem. Dual cargo+buck2 merge proof stays forbidden.

**Do not:** per-agent `Cargo.lock`; N virtual workspaces; `cargo update` /
`generate-lockfile` inside owner sandboxes; `--workspace` cargo as the
subagent loop; two live writers on the same **file/Item** (D-39). Two
writers on the same **crate** (disjoint files) is the desired case.

**MUST (parallel crates, local buck2)**

- **achieves:** subagents actually run in parallel; lockfile and crate files
  have a single writer; local proof does not N-way cargo; CI cargo stays the
  one merge graph.
- **origin:** cargo workspace still shares `Cargo.lock` and crate files;
  splitting workspaces creates N lockfiles; founder: buck2 is the local
  resolution, CI cargo is fine once linearized.
- **rule:** parallel write unit = file/Item (D-39), not the crate;
  worktree per parallel process; local verify = buck2; `Cargo.lock` is
  single-writer; membership is directory glob/generated (D-39); CI cargo
  `--workspace` is merge-only. Do not crate-lock.
- **ensure:** dispatch names crate paths when an owner is split; review
  rejects lockfile diffs on path-dep-only changes; agents do not run
  workspace cargo locally as the parallel loop.
- **overturn_when:** pipeline **serves** buck2 as tenant #0 presubmit
  (ADR-0716 overturn_when) same-wave, or a five-field ADR names another
  single-writer lock story that still permits crate-parallel worktrees.

### D-33 — Structural Mutation Separation

The reorg (D-8 last-leg, dumps, `git mv` / `git rm`, crate grammar, debrand,
workspace membership) is **not** the same class of change as implementing
ontology or a blob adapter. Mixing them is how agents tangle layout with
behavior (empirical: majority of agent “refactors” land inside feature
commits and become unreviewable).

| Class | What moves | Lane |
|---|---|---|
| **Structural** | Faces, crate dirs, `git mv`/`git rm`, root `Cargo.toml` members, `Cargo.lock`, D-8 children, proto package path, agreed port promotion | One serialized (or explicitly stacked) reorg lane. Escalated (D-29). This is `#2221`. |
| **Behavioral** | `src/`, `tests/`, owner `docs/`, cedar, IR under an **existing** crate | Parallel leaf-crate worktrees (D-32). Frozen shape (D-8/D-30). |

`REFACTOR` = `git mv`. `REMOVE` = `git rm`. Neither is a feature PR. After
the structure wave, implement agents **do not** mutate the tree shape.
Draft→agreed port promotion is structural (D-28), not a side effect of a
Foundry feature.

**MUST (separate structure from behavior)**

- **achieves:** review can tell layout from logic; parallel agents do not
  fight the workspace graph; reorg can finish.
- **origin:** one PR that both moves `iam/` and changes PDP semantics;
  cargo lock + git mv + feature in the same index.
- **rule:** a lane is structural or behavioral, not both; implement
  dispatches do not add/rename/remove crates or edit `Cargo.lock` /
  root members; structure stays frozen while behavior fans out.
- **ensure:** review rejects mixed PRs; D-8 allowlist diffs are structural
  lanes only.
- **overturn_when:** a five-field ADR names a mechanical splitter that
  still keeps layout diffs independently revertible.

### D-34 — Cache and graphs (not a cargo lock-bypass, not an AST-merge product)

N worktrees need **shared compile reuse** without sharing cargo’s locks.

| Idea | Ruling | Why |
|---|---|---|
| Shared **read-only** cache (buck2 CAS / disk cache, Bazel remote/disk cache, cargo **registry** as fetch cache) | **Keep** | Hyperscaler default. Trusted writers (CI + local buck2); agents **read**. Do not let agents poison CAS. |
| Global target dir / `CARGO_TARGET_DIR` for all worktrees | **Reject as the parallel loop** | Cargo’s target lock serializes N agents; bypassing it corrupts artifacts ([cargo#16804](https://github.com/rust-lang/cargo/issues/16804)). |
| “Lock bypass” of cargo target-dir | **Reject** | The lock exists to prevent corruption. Bypass **git** `Cargo.lock` mutation instead (D-32). |
| `cargo --offline --locked` if cargo is used at all | **Keep** | No fetch, no lock rewrite. Offline mode is not a second workspace. |
| Path mapping so cache keys are not worktree-absolute (`SCCACHE_BASEDIRS`, Bazel execroot, buck2 action keys) | **Keep as cache impl** | Hits must be content-addressed, not `/tmp/wt-3/...` paths. |
| Aggregated AST patching (merge N agents’ edits in one file via AST) | **Reject as a product** | D-32 already forbids two writers on one crate. An AST-merge service is a new dual stack. Git merge of disjoint crates is enough. |
| **Build graph** (buck2) | **Keep — dispatcher** | Who depends on this target; local `buck2 test //owner/...`. |
| **Crate/code graph** (`cargo metadata --offline --locked`, rust-analyzer) | **Keep — dispatcher** | Concurrent-safe paths; D-28 consumers. Read-only. |
| **AST/HIR graph** (rust-analyzer) | **Keep as editor/query**, not a stored census | Do not freeze HIR dumps in git (D-17). |

Local N-way proof remains **buck2** (D-32). The cache sits **beside**
worktrees, not inside them. When CAS is served by `storage/` that is
pipeline/RE (ADR-0716 overturn), the same read-only rule holds.

**MUST (cache + graphs)**

- **achieves:** parallel worktrees reuse compiles; agents cannot take the
  cargo target lock or rewrite `Cargo.lock`; dispatch uses graphs we
  already have.
- **origin:** N cargo `target/` dirs; shared `CARGO_TARGET_DIR` deadlocks;
  founder listed cache, offline cargo, AST patch, graphs as candidates.
- **rule:** shared cache is content-addressed and read-only to agents;
  no shared cargo target-dir as the parallel engine; cargo if used is
  `--offline --locked`; dispatcher uses buck2 + metadata/r-a; no
  AST-merge product; no HIR/census files in git.
- **ensure:** path-only PRs have no lockfile diff; agent logs do not show
  `cargo update` / `generate-lockfile`; cache writers are not owner
  sandboxes.
- **overturn_when:** a five-field ADR shows a safe shared cargo cache that
  does not lock or corrupt, or an AST merge that is still crate-disjoint
  (then it is unnecessary).

### D-35 — File budget: 300 lines (agents), with a closed exempt set

Google does not hard-cap file length (C++ guide caps *functions* around
~40 lines as a smell, not files). For **agents**, a 2k-line `lib.rs` is a
context and conflict magnet (D-32: one writer per crate is useless if
that crate is one file). Cap **hand-written** files.

**Budget.** **300** physical lines is the born-blocking maximum for a
non-exempt file (the top of the 100–300 range). Prefer splitting before
that; **100 is not a gate** — it would explode module count and fight
RFC 430 (`lib.rs` as the crate root). Count is `wc -l`, no comment-stripping
cleverness (that becomes a census).

**Exempt (closed).** Live `docs/decisions/ADR-*.md`; `app/<product>/PRD.md`;
`AGENTS.md` / `CLAUDE.md`; `Cargo.lock`; `third-party/`; generated
(`*.generated.*`, prost/tonic/reindeer output, OpenSLO from IR); vendored
lock-step snapshots. Not exempt: tests, cedar, owner `docs/`, `*.rs` agents
write. Do **not** add `specs/` or `plan/` to the exempt list by recreating
those trees (D-8 / D-36).

**Existing over-budget files.** Split when that **crate** is next worked,
or in a dedicated file-budget lane for that crate (D-33: not mixed with a
feature, not one repo-wide 2221 dump). Split = more `snake_case.rs` modules
**inside the same leaf crate**, not a new crate (unless D-28/D-29).

**Enforcement.** Pattern gate on **touched** non-exempt paths (D-17 keep
set: fail the file, not `expected_total`). Agents must not emit a new file
over 300.

**MUST (file budget)**

- **achieves:** agent-local diffs stay reviewable; parallel crates stay
  small; monsters get split on purpose.
- **origin:** unbounded `lib.rs`; founder 100–300 range; census gates are
  forbidden.
- **rule:** 300-line cap on hand-written non-exempt files; 100 is not a
  gate; exempt set is closed; existing over-budget split per crate when
  touched; no repo-wide dump; no `expected_total`.
- **ensure:** touched-path check in presubmit; new agent files >300 fail;
  generated/lock/ADR/PRD ignored.
- **overturn_when:** a five-field ADR names a different number that still
  fits agent context and does not become a file-count freeze.

### D-36 — Live law is one monolithic checklist document

ADR-0719 (and each live 07xx apex) is **one file** on purpose. Splitting
D-1…D-n into a directory of mini-docs recreates `specs/` drift: N copies,
stale bullets, no single pass that sees contradictions.

Iterate **in place** on that monolith: PRD ↔ decisions ↔ owner docs
pointers. The D-n tables *are* the checklist. Recursive challenge is
editing this file, not adding `plan/iteration-47.md`.

App-level monolith is `PRD.md` (D-8). Owner `docs/` stays short (D-35)
and points at the ADR/PRD — it is not a second spec farm.

**Do not:** recreate `specs/`, `plan/`, `tasks/`; write a parallel “north
star” markdown; fork this ADR into per-D files.

**MUST (monolithic live law)**

- **achieves:** one place to see contradictions and staleness; PRD and
  decisions iterate together; no N+1 spec hubs.
- **origin:** JSON/spec hubs drifted; founder: ADR/plans as one document
  that behaves like a checklist.
- **rule:** live 07xx ADRs and app `PRD.md` are the long-form checklists;
  amend in place; no `specs/`/`plan/` resurrection; owner docs remain
  under D-35.
- **ensure:** layout still rejects `specs/` and `plan/`; new law lands in
  this file or another live apex, not a sidecar novel.
- **overturn_when:** PHASE-5 promotion of a different operating-contract
  home moves this monolith atomically with evidence (INV-DOC-9).

### D-37 — Shared docs/config are not 300-line splits; uuid fragments + one fold

N agents colliding on `Cargo.toml` / `Cargo.lock` / YAML / JSON is the
problem D-32 named. Splitting those files like `lib.rs` does not work
(Cargo has one manifest; lock is one graph). **Different resolution.**

**Prefer delete or generate.** File-based config stays the closed minimum:
root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`,
`deny.toml`, per-crate `Cargo.toml` (travels with that crate’s writer).
No new JSON/YAML product. OpenSLO/faces stay generated. Cedar is per-owner
policy, not a global yaml.

**Denylist (implement agents: no in-place edit).** Root workspace
`Cargo.toml`, `Cargo.lock`, toolchain/fmt/deny, `AGENTS.md`, `CLAUDE.md`,
live 07xx ADRs, generated `*.json`/`*.yaml`. Crate-local `Cargo.toml` is
**not** on this list — it is that crate’s file (D-32).

**Additive path: uuid fragments, not clones of the whole file.** Copying
`Cargo.toml.<uuid>` and hoping to 3-way merge is still a novel. Agents
**add** a unique file, e.g. `Cargo.toml.d/<uuid>.toml`, that states only
the addition (`members = ["storage/adapters/blob-sqlite"]`). Paths never
collide across worktrees. Ephemeral: the fragment is not the long-term
SSOT.

**Fold is serial, once, on the receiving branch.** If each worktree
pre-commit *already* folds into `Cargo.toml` and deletes the fragment,
merge recreates the `Cargo.toml` conflict. So:

| Stage | What happens |
|---|---|
| Implement lane commit | Fragment only. Hook **rejects** in-place denylist edits. Validates fragment schema. Does **not** rewrite the canonical file. |
| Restack / merge_group / integration pre-commit | **One** Rust fold engine: apply all fragments (set-union of members; fail on key conflict), write canonical, `git rm` fragments, refresh `Cargo.lock` **once** (`--locked` path if no pin change; regenerate only if the fold added a crates.io dep). |

That step is “pre-commit” in the TAP sense (before the linearized snapshot
is proven), not N local hooks racing.

**Cannot fragment-merge:** prose ADRs/PRDs (D-36 — single writer,
architecture lane); `Cargo.lock` body (derive after fold); YAML/JSON
novels (do not exist). Duplicate fragment keys with different values
**fail closed**.

**YAGNI bound (thought experiment).** After D-33 the common implement
lane never touches root `Cargo.toml`. Two humans on different laptops
already resolve via **merge queue + disjoint crate paths** (Google:
small packages so BUILD files rarely collide; TAP mid-air collisions
are semantic, caught on the linearized snapshot). UUID fragments are **YAGNI** if workspace `members` are **globs** over
D-8 faces (D-39). They are not a general clone-the-file protocol and
not a second VCS (`.delta`). Prefer glob; fragments only if a glob
cannot express the add.
Same-machine N agents: shared **read-only** buck2 cache (D-34),
separate worktrees, no shared cargo `target/` lock. Different
machines: git + merge_group is the bus; do not invent a fragment
message bus. Dispatcher assigns disjoint crates **before** spawn —
that is the waste-avoidance, not a smarter fold.

**MUST (fragments + one fold)**

- **achieves:** N agents add to shared config without touching the same
  git path; one mechanical fold; no yaml/json farm; lockfile has one
  writer at fold.
- **origin:** N-way `Cargo.toml`/`Cargo.lock`/docs; 300-line split does
  not apply; founder: uuid clones + pre-commit resolve.
- **rule:** keep file-config minimal; implement agents do not in-place
  edit the denylist; additions are uuid fragments; fold is one Rust
  serial step on the receiving branch; ADRs stay single-writer; lock is
  regenerated once after fold; no whole-file uuid clones.
- **ensure:** review/presubmit reject in-place denylist diffs on
  implement PRs; fragment schema is closed; leftover fragments after
  fold fail; no `specs/` resurrection.
- **overturn_when:** a five-field ADR names another single-writer
  merge-driver that still keeps additions on unique paths and fails
  closed on semantic conflict.

### D-38 — Worktrees are not integration; star into `dev`; quarantine the identity

Worktrees stop two processes sharing one `.git/index`. They do **not**
make two diffs commute. Mesh-merging live worktrees (A into B into C
while agents still write) is how conflicts take hours and then grow:
each resolution is a moving target (trunk-based: the cost lives at a
**stale** merge boundary, not inside the branch).

Google/CitC: no git worktree mesh. CLs are patches on Piper HEAD; TAP
tests the projected submit; mid-air collisions are **semantic** and
caught on a linearized snapshot. GitHub/Trunk merge queues: test PR
against `dev` **plus** what is ahead in the queue — not against a
sibling feature branch. Hyperscaler rule: **never merge trunk into the
lane; never merge lanes into each other.** Replay the lane onto trunk
(rebase / queue).

**Topology (star, not mesh).**

```
lane worktree ──PR──► merge_group ──► origin/dev
lane worktree ──PR──►      ▲
lane worktree ──PR──►      │
```

Forbidden: `git merge other-agent-worktree`, `git merge origin/dev` into
a live implement branch, a “integration worktree” that sucks in five
lanes while they keep coding.

**When a conflict appears.** Constant-work / bulkhead: **quarantine that
identity** (the **file/Item**, D-39 — not the whole crate). Writers to
*that* path stop or restack **one** lane onto current `dev`. Other files
in the same crate **continue**. Global stop is the N-tax. Continuing to
implement on the conflicted **item** is how you get a second hour of
conflicts. If a git conflict happened, assignment was not path-disjoint
(failure), not “need a better merge.”

Worktrees stay **optional isolation** (same-machine index, D-31). If a
lane is sequential on one crate, one worktree is enough. Parallelism
is disjoint PRs into the queue, not N checkouts that later have to be
hand-folded.

**MUST (star integration)**

- **achieves:** no hour-long mesh merges; implementation does not halt
  globally; conflict debt cannot compound on a path still being written.
- **origin:** worktrees treated as the integration strategy; resolving
  while agents kept writing the same files.
- **rule:** integrate only lane → `dev` via merge_group; no live
  worktree↔worktree merges; no merge-trunk-into-lane; on conflict,
  pause writers to that identity only; other disjoint lanes continue.
- **ensure:** dispatch/review reject inter-lane merges; overlapping
  crate assignment is a dispatcher fail, not a merge-hero task.
- **overturn_when:** a five-field ADR names another integration topology
  that still forbids mesh merges of live writers and still tests the
  combination before `dev` moves.

### D-39 — Identity is a unique git path (file/Item), not a crate lock or `.delta`

**Adversarial on D-32/D-37/D-38.** “One writer per crate, others wait” is
a mutex with polling. “`.delta` / uuid clones” is a second version
control. Neither meets “conflict is structurally impossible under N
writers,” including **N on the same crate**.

Git conflicts iff two commits mutate the **same path** (or the same
hunk of that path). Therefore the only structural impossibility is:
**each parallel mutation owns a unique path.** Crate directories are too
big; uuid sidecars invent paths that are not the code.

**Decomposition (this is the grain).**

```
crate → module files → one primary Item per file
        (fn / struct / impl / trait / enum — RFC 430 names)
markdown → one structural block per file
        (one H2 / one D-n section — not a .delta of the ADR)
```

D-35 (≤300 lines) exists so one file ≈ one Item. N agents on crate P
= N files under P (`blob.rs`, `quota.rs`, …). `git merge` of those
paths cannot conflict.

**Parent lists must not be hand-edited.** `mod blob;` in `lib.rs` and
`members = ["storage/adapters/blob-sqlite"]` in root `Cargo.toml` are
the remaining mutexes. Make membership a **pure function of the
directory**:

- Workspace: Cargo globs over closed D-8 faces
  (`storage/adapters/*`, `app/*/core/*`, …). Adding a crate is
  `git add storage/adapters/foo/{Cargo.toml,src/lib.rs}` — unique
  paths; root toml does not change.
- Crate modules: Buck `srcs = glob(["src/**/*.rs"])` already. Cargo
  still wants `mod`. **Generate** `src/mods.generated.rs` (sorted
  `mod x;`) from the directory at the **one** merge_group snapshot
  (same class as generated faces). Agents never edit it. `lib.rs` is
  `include!("mods.generated.rs");` plus the crate’s tiny public API
  (itself one Item file if it grows).

No `.delta`. No uuid object store. The new file **is** the commit.

**Honest limit.** Two agents editing the same `fn foo` in the same file
**will** conflict. That is not solved by OT, mergiraf, or hope. Occupancy
is **that file/Item** on open PRs. Agent 2 is dispatched to a **different
Item/file in the same crate**, immediately — not parked on a crate lock.

Markdown law (ADR/PRD) stays one writer (D-36): rare, checklist
iteration. Owner `docs/` splits like code: one topic file.

**MUST (path identity)**

- **achieves:** N-way work on one crate without crate mutex, without a
  sidecar VCS; git conflicts become assignment bugs.
- **origin:** crate-lock and uuid-delta both failed the founder bar;
  crates already decompose into files and Items.
- **rule:** commute identity = unique git path at module/Item (or md
  block-file); membership = directory glob + generated mod list; no
  `.delta`; no whole-crate lock; same-Item dual write refused at
  assign; N disjoint files in one crate is allowed and expected.
- **ensure:** workspace members globs match D-8; generated mod list not
  hand-edited; occupancy matches PR file paths; review rejects crate-
  wide locks and uuid-delta dumps.
- **overturn_when:** rustc/Cargo load modules by directory without
  generated `mod` (then drop the generator) or a five-field ADR names
  another unique-path membership that is not a second VCS.

### D-40 — Path-sets, mixed ops, plans, cross-owner; cap cone is sandbox not a lock

**Is a session locked to an app/capability?** For **merge: no.** Commute
is path-sets (D-39). For **blast radius / sandbox: default yes** (D-31
cone). For **contracts: escalate** (D-29). Those are three different
knobs. Conflating them produced crate-locks and “agent owns `storage/`.”

An agent’s dispatch is a **path-set** `S`. Spawn iff `S` is disjoint
from every open PR’s path-set (rename occupies `{old, new}`). The set
may list files in more than one cap **only** when the dispatcher names
them (found a correction; D-29). The agent still cannot self-widen.

**Mixed ops (all at once, N-way).** Git only cares about paths.

| A \ B | new unique file | edit F | delete F | `git mv` F→G |
|---|---|---|---|---|
| new unique file | commute | commute | commute | commute if dest ≠ new |
| edit F | | **no** (same Item) | **no** | **no** |
| delete F | | | idempotent / **no** | **no** |
| `git mv` F→G | | | | commute iff `{F,G}` ∩ `{F',G'}` = ∅ |

Refactor = `git mv` + edits of **callers**. Occupancy = old + new +
every caller file touched. If that set is huge, it is a **large-scale
change** (Google LSC / Rosie): **one** mechanical lane, or N agents
each given a **disjoint shard of caller files**. Do not N-way edit the
same caller. Do not run a feature lane on files an LSC already occupies.

Write + refactor + move + delete **in parallel** is allowed **and
expected** when those path-sets do not intersect. That is structurally
conflict-free. Same-path mix is an assignment bug, not a merge skill.

**Planning vs implementation.** ADR-0719 / `PRD.md` are **one path
each**. A plan-amendment lane occupies that path; every `*.rs` lane
**commutes** with it (different paths). “Needs more planning” does
**not** stop implementation. Two plan-amendment lanes on the **same**
ADR **do** conflict — serialize plan writers (D-36), or split only
owner `docs/` into block-files (not a `.delta` of the ADR). Corrections
found in another cap: **new dispatch** with those paths, original lane
keeps its set. Do not mutate the living ADR from ten implement agents.

**Scenarios (experiment).**

1. N writes of new Items in one crate — unique files, glob membership —
   commute.
2. N edits of disjoint files, plus one `git mv` of a third file —
   commute.
3. Edit F while another lane moves F — **forbidden at assign.**
4. Extract fn from F.rs into G.rs while another lane edits H.rs —
   commute; if they both edit F.rs — **no.**
5. Delete dead F.rs while someone still edits F.rs — **no.**
6. Mechanical rename of a trait across 40 files — LSC lane owns those
   40, or 40 shards with disjoint files; feature work on those files
   waits **on those files**, not on the capability.
7. Plan lane amends D-40 in this ADR while N crates implement — commute.
8. Two plan lanes both edit this ADR — serialize (single writer).
9. Foundry agent finds a `storage` bug — dispatcher opens a **storage
   path-set** lane; Foundry lane does not absorb `storage/` (D-31 no
   self-widen). If it is a **shared port** shape, that is D-28/D-29,
   not a silent extra file.
10. Session “locked” to `app/foundry` as sandbox, but occupancy is
    `app/foundry/core/ontology/src/property.rs` — another Foundry agent
    on `pages.rs` runs now.

**MUST (path-sets)**

- **achieves:** mixed N ops without crate/cap locks; plans commute with
  code; cross-owner corrections are named dispatches; LSC does not
  collide with features on the same files.
- **origin:** cap-session lock and crate-wait were mutexes; founder
  asked mixed write/refactor/move/delete, plan/amend/correct, and
  whether cap lock is necessary.
- **rule:** occupancy = path-set (mv occupies both ends); disjoint
  sets commute including across caps; cap cone is default sandbox not
  merge law; plan/ADR is one path (commutes with src); LSC is one lane
  or file-sharded; no poll-lock; no self-widen; no `.delta`.
- **ensure:** dispatch records the path-set; overlapping PR path-sets
  are not spawned; review rejects cap-wide locks and un-named cross-cap
  writes.
- **overturn_when:** a five-field ADR names another occupancy grain
  that still makes same-path dual write unspawnable without a sidecar
  VCS or a capability mutex.

### D-41 — Simplest commute: stable indexes + unique files + queue

**Challenge.** D-37 fragments, D-39 committed generated `mods`, D-40
occupancy-before-spawn are still machinery. Compile-time `mod` generation
**committed at merge_group** fails locally (agents need to compile) or
**conflicts** if they commit it. Occupancy against GitHub is not
structural (two laptops can still `add src/put.rs`). Cap cones do not
make git commute.

**First principle.** Two commits conflict iff they change the same git
path. Therefore:

1. **Never edit the indexes.** Workspace `members` are **globs** over
   D-8 faces (Cargo already supports `crates/*`). Crate `lib.rs` has
   **one stable line** that includes modules by **scanning `src/items/`
   at compile time** (tiny owned `build.rs` / buck2 genrule writing
   `OUT_DIR`, not a tracked file). Agents never touch `lib.rs` or root
   `Cargo.toml` for adds.
2. **New work is a new unique file** (`src/items/quota.rs`, or a new
   globbed crate dir). The file **is** the commit. No `.delta`.
3. **Edit / delete / `git mv`** only those files. `git mv` needs a free
   destination. Callers you actually rewrite are extra paths — if that
   set is large, it is one mechanical LSC PR, not N overlapping edits.
4. **Integrate** PR → `origin/dev` → merge_group. Star, not mesh (D-38).
5. **Jail** writes to the dispatched files (narrower than a cap). Cap
   is a default hint, not a lock.
6. **Plan** occupies the ADR path; implement occupies `*.rs`. They
   already commute. One plan writer (D-36).

**Do not build:** occupancy JSON; uuid fragments; committed generated
mod lists; crate/cap poll-locks; automod from crates.io unless owned.

**Uncoordinated same-name create** (`put.rs` vs `put.rs`): Google TAP
mid-air — cheap, one file. Merge queue rebases the loser; they rename.
Do not uuid-name every module to prevent a rare event.

**How we achieve it (instruction → automation → presubmit).**

- Convention: new Items only under `src/items/` (or a new globbed crate
  directory). `lib.rs` / root members lists are denylist for implement
  PRs (same class as D-37 denylist, without fragments).
- One owned compile-time scanner per crate (or shared tiny codegen in
  `build/`) when a crate has more than `lib.rs`. Until the second file,
  YAGNI — stay in `lib.rs` under 300 lines.
- Workspace glob in the structural wave (`#2221` or immediately after).
- Write-jail = path-set. merge_group as today.
- Dispatcher *looks* at open PR files (no store). If overlap, pick
  another filename **now**, do not wait on a lock.

**MUST (stable index)**

- **achieves:** N mixed ops with no sidecar VCS, no mutex, no generated
  file in git; git conflicts only when two lanes actually share a path.
- **origin:** fragments and occupancy were solving coordination with
  product; founder asked simplest first-principles.
- **rule:** membership is glob + compile-time directory scan; new work
  is a new unique file; implementers do not edit parent indexes; jail
  the path-set; PR to `dev`; no `.delta`; no crate/cap lock; same-path
  dual write is an assign rename, not a queue.
- **ensure:** admission rejects implement diffs to root members lists
  and hand-maintained `mod` inventories; `src/items/*.rs` adds do not
  change `lib.rs`.
- **overturn_when:** rustc modules-from-directory lands and drops the
  scanner, or a five-field ADR shows a smaller rule that still keeps
  parent indexes stable.

### D-42 — Cross-harness: only git, draft PR, and presubmit travel

50 agents on five vendors do **not** share a dispatcher, Landlock,
CitC, or even a worktree. Cursor often shares one checkout; Claude
may lazy-worktree; Codex sandboxes; Grok uses isolated worktrees.
Antigravity will differ again. **Worktrees and sandboxes are
harness-private. They are not Oyatie law.** D-31 is a *local recipe*
for a Grok-class isolation, not `required_sequence` for every tool.

**What still holds (harness-agnostic).**

- Commute = disjoint git paths (D-41).
- Parent indexes stable; new work is a new file.
- Every lane opens a **draft PR against `origin/dev` as soon as it
  has a path** — that is occupancy every other harness can `gh pr
  list` / GitHub files API. No JSON board.
- `presubmit` / merge_group is the only combination test. Denylist
  and 300-line cap fire on the PR, not in the vendor.
- Instruction on **every** session hub (`AGENTS.md`, `CLAUDE.md`;
  Cursor/Codex overlays must point here, not fork law).

**What does not hold unless you stop believing the vendor.**

- Physical write-jail — optional, per harness. Assume the agent *can*
  edit `iam/` anyway; admission + review catch it.
- “Dispatcher assigned disjoint names before spawn” — there is no
  one dispatcher across Grok+Cursor. Two uncoordinated `put.rs`
  creates: merge_group rebase, rename. Waste is **one file**, not a
  crate. Accept it (TAP mid-air). Do not uuid-name modules.
- Ten Cursors in one clone without worktrees: `index.lock` / mixed
  diffs. That is a **local** problem: those ten need worktrees or
  they are one writer. Other harnesses on other machines are fine.
- 50× `cargo nextest --workspace`: laptop tax. Instruction: buck2
  `-p`/target. CI still one cargo snapshot. You cannot make Codex
  run buck2 if it ignores AGENTS — that is operator, not law.

**50-wide merge queue.** Semantic collisions (two PRs, different
files, tests red together) grow with N. That is why the queue
exists. Do not serialise spawn to avoid it.

**MUST (portable protocol)**

- **achieves:** five harnesses, one merge story; no vendor lock-in
  of isolation.
- **origin:** N agents on Grok/Claude/Codex/Cursor/Antigravity do
  not share a sandbox.
- **rule:** portable surface is git + draft PR on `origin/dev` +
  `presubmit`; law on every hub; do not depend on vendor jail or a
  cross-harness dispatcher; same-path create is rebase.
- **ensure:** denylist/file-budget fail regardless of authoring
  tool; hubs do not fork D-41/D-42.
- **overturn_when:** a five-field ADR names another portable
  occupancy that is still git (not a new sidecar VCS).

### D-43 — N-parallel delivery: launcher assigns paths; PRs are the pipeline

This is the acquire → plan → red → green → slop → CI → queue loop,
**N-wide**. It is not a Kanban JSON and not “agents browse a list.”

**Honest limit.** Thought-experiments on commute (D-38…D-42) are
law. The four-harness collision lab was **not** executed to merge.
Do not claim that loop is empirically green.

**Acquire is the launcher, once.** ADR/PRD names work as **output
paths** (`storage/adapters/blob-sqlite/src/items/put.rs`). A spawn
script (human or one coordinator process) gives each harness **one
path-set**. The agent does not poll issues, `gh pr list`, or
`tasks/`. Grabbing is `argv`, not a mutex. Second spawn with an
overlapping path is a launcher bug; presubmit path-intersect is the
backstop (D-42).

**One PR per path-set** (usually one harness). Cross-harness =
separate PRs onto `dev`/`main`, never a Graphite stack across
vendors. Subagents inside one harness share that PR unless the
vendor invents branches by itself.

**Stages on that PR** (same path-set; do not open a second PR per
stage). Other PRs run the same stages **at the same time**.

| Stage | What | N-parallel? |
|---|---|---|
| Valid | Path in D-8 grammar; not denylist; not intersecting open PR files | Launcher + presubmit |
| Plan | PR body or `docs/design/<item>.md` ≤300, cites the D-n. Review on the PR | Yes, different items |
| Red | Tests for **this** item as unique files | Yes |
| Review tests | PR review (≠ merge APPROVE) | Yes |
| Implement | Fill the item file | Yes |
| Review / de-slop | Same PR | Yes |
| Coverage | More tests in **this** crate only | Yes |
| Pipeline/CI review | **Only if** `.github/` or `pipeline/` ∈ path-set. Metrics: wall-clock, queue depth, duplicate jobs vs last `dev` presubmit — skip if the PR is `src/items/` | Rare; serialize if they touch the same workflow file |
| Diff + merge review | Required APPROVE | Per PR |
| presubmit | fmt/clippy/nextest (touched + workspace as today). Red = **this PR’s** process error. Do not stop other PRs | N in the queue |
| merge_group | Combination on advancing `origin` (D-38). Agent does not rebase-loop | One snapshot at a time, many PRs waiting |
| squash | Lands | |

**Local pre-push** is `cargo fmt` on staged `*.rs` only. It cannot
catch workspace nextest; claiming it should is false on this graph.
presubmit catches. If presubmit is red on fmt/denylist, the agent
skipped the cheap local step. If red on full nextest, that is the
merge proof, not a pre-push miss.

**CI must be green to merge.** If it is not, that PR is wrong — not
the factory. Other path-sets stay in the queue.

**Forbidden:** `tasks/` board; agent wait-until-unlock; one global
plan review gate; every implement PR reviewing CI throughput;
mesh-merge of stages across PRs; new ceremony markdown per stage.

**MUST (N-parallel loop)**

- **achieves:** many PRs in plan/red/implement/review at once;
  acquire without polling; CI red is local to a PR.
- **origin:** a 19-step serial ritual plus a grab-list would recreate
  locks and GooWiki.
- **rule:** launcher assigns unique path-sets from ADR/PRD; one PR
  carries plan→tests→impl→slop→coverage; pipeline/CI review only
  when those files are touched; presubmit/merge_group/squash;
  pre-push is fmt-touched only; no task board; no factory-stop on
  one red PR.
- **ensure:** path-intersect check; denylist; no `tasks/` directory;
  workflow diffs require CI-metric look, src-only PRs do not.
- **overturn_when:** a five-field ADR names a smaller loop that still
  assigns unique paths without agent-side wait.

### D-44 — Operator interview is the only intake; Product xor Program

**Who receives client directions?** Not implementers. Not the merge
queue. The **human operator** working with the orchestrator.

Client needs arrive **ambiguous and sometimes wrong**. Passing them
down the factory is how you get N agents building the wrong shape.

**Gate (fail closed, before any slice).**

1. Interview until the need is unambiguous (no TBD / empty acceptance).
2. Research **existing** docs and the working tree; cite paths that
   exist.
3. Realistic evaluation: layout (D-8 faces), YAGNI, blast radius,
   what already exists. Dump-root asks (`plan/`, `libs/`, …) are
   **Rejected**, not clarified into existence.
4. Emit an **ephemeral artifact package** (not a `plan/` or `tasks/`
   in the product tree).
5. Handoff is a function of paths: `app/` → **Product**; capability
   root → **Program**. Mixed packages split. No owner → no package.

Product/Program decompose the package into path-sets. Only then do
role hops exist.

**MUST (intake)**

- **achieves:** wrong or vague client text cannot occupy implement
  capacity; Product vs Program is deterministic.
- **origin:** prompt-to-implement skipped clarification and built
  dump folders; founder required interview artifacts and a Product
  or Program handoff.
- **rule:** operator+orchestrator interview, cite, verify; fail
  closed on ambiguity or forbidden shape; ephemeral package; Product
  xor Program; never raw text to implement.
- **ensure:** admission of an implement hop requires a packaged
  handoff; mixed app+cap packages fail; dump-root packages reject.
- **overturn_when:** a five-field ADR names another intake that still
  stops wrong/ambiguous needs before path occupancy.

### D-45 — Lanes are deterministic; orchestrator does not spawn or fold

D-43's occupancy (one PR per path-set) stands. The implication that
**one agent walks plan→red→implement→coverage** on that PR is
**OVERRULED**. That is a serial mega-agent. Ten implement-ready
disjoint slices bound to one implementer is a fold. The whole
pipeline waiting on that implementer is a bottleneck.

**Ready hops are a pure function** of (slices, completed roles,
in-flight, path occupancy). Anvil publishes them. Harnesses bind a
**fresh agent id** per claimed hop. The orchestrator does not spawn
vendor CLIs for those hops.

Roles form a **DAG with fan-out**: after Implement, review, coverage,
security, docs, and box tests become ready **together**. Completing
Implement on slice A unblocks A's successors **and** frees the
implement lane for slice B.

**MUST (lanes)**

- **achieves:** N-wide role trains; no fold; no spawn bottleneck in
  the orchestrator.
- **origin:** one implementer looping disjoint work; serial stage
  enum; founder: tasks and lanes are deterministic; 1-of-10 is
  wrong.
- **rule:** ready-hop count for a role is the number of disjoint
  schedulable slices in that role; no agent-id reuse; orchestrator
  publishes, launchers bind; fan-out after Implement; not one
  sequential pipeline.
- **ensure:** fold (k < N bound agents for N ready hops) fails
  closed; reused agent ids fail; tests prove fan-out cardinality.
- **overturn_when:** a five-field ADR names another scheduler that
  still forbids folding a lane onto one long-running agent.

### D-46 — Foreign-path need is a draft on *your* files, not a committee

When something breaks, or a downstream team needs a contract change
on a slice they **do not occupy**: they **do not write those files**
(D-40). They do not open a ticket. They do not wait on a lock.

They add `ports/draft/<port>/` or `adapters/draft/` **on their own
path-set** (D-28). That commutes with the owner. The owner receives
a `ContractAmend` hop when **their** paths are free. Agreement is
`git mv` of the draft onto the provider (D-28), under D-29
escalated review — not a standing committee.

A red presubmit / merge_group **quarantines that path-set** (D-38).
Other disjoint slices **continue**. Global stop is the N-tax.

**MUST (amend without bureaucracy)**

- **achieves:** N-parallelism survives contract lag and CI red;
  unagreed shapes stay grep-visible.
- **origin:** escalation boards and "please change your API" waits
  serialized the factory; founder asked minimum bureaucracy.
- **rule:** no occupancy ⇒ no write; consumer drafts on owned
  paths; owner amends on owner paths when free; red is local to
  the path-set; no `tasks/` board.
- **ensure:** cross-owner writes without occupancy fail; draft
  paths contain `draft`; one red PR is not a factory stop.
- **overturn_when:** a five-field ADR names another settlement that
  still keeps foreign writes unspawnable and does not add a queue
  product.

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

### D-17 — Presubmit is the graph, not a JSON product

**Executed (this wave):** `git rm` of `specs/` as a living-law corpus (including `root-hub-pointers.json`, `integ-branch-envelopes.json`, `masterplan.json`, `cedar-policy-schema.json`), `registry/`, `evidence/`, `governance/` (including `capability-registry.json` and `check/`), `ci.toml`, `pipeline/facade` census crates, Tide/GateRun/process-kit/webhook-gateway, and the `libs/check-*` + `libs/governance-*` fitness farm (except library kernels `check-cost-budget` and `governance-eval-domain`). D-8 unknown-root names live in `pipeline/core/admission`. Agent entry is `AGENTS.md` + `CLAUDE.md` + owning ADRs. No replacement JSON hub.

`pipeline/facade/*` (was `ci/facade`) and `governance/check/*` grew a **second
product**: JSON policy files, frozen path lists, FNV signatures, Helm/OpenAPI
parity. Hyperscaler TAP is **build + test the graph**. It is not ten crates
that observe the tree. Trimming one stale JSON keeps the anti-pattern.

**Merge proof (the TAP).** `cargo fmt --all --check`; clippy `-D warnings`
when clean; **`cargo nextest`** (compile+test once). That is the product’s
presubmit for tenant #0. Not `cargo check`. Not libtest. Not a JSON census.

**Still allowed as graph steps** (code in the execute graph, **not** a
governance JSON product). Default is delete. Keep only if it is a **pattern**
(new unauth HTTP, new GraphQL crate, new license, new non-Rust automation,
hand-edit of generated faces, D-8 unknown root name):

| Step | Why it is not census |
|---|---|
| One license/ban engine (`deny.toml` + weekly `cargo deny`, not a crate) | Legal. |
| D-8 unknown-name (`pipeline/core/admission`) | Closed root set. Fails on **unknown names**, never `expected_total`. |
| D-35 file budget on **touched** non-exempt files | Pattern: this file is too long. Not a frozen count of files. |

**Not needed** as crates, JSON dirs, or merge predicates: corpus-census,
planning-projection as a required check, cross-artifact-agreement as a
required check, endpoint-authorization-coverage (that is tests + PDP
fail-closed), graphql/crypto as standalone census crates (encode in
clippy/tests), `governance/check` fleet, `governance/corpus`, cap-root
`*-policy.json` path lists, Helm/OpenAPI/OpenSLO parity, Tide/webhook as
**product core**, Prow `GateRun` as Cloud Build.

`governance/` **is** needed for `capability-registry.json`, packs, and
Cedar the PDP compiles. It is **not** a CI product. `check/` crates that
are not a D-8 unknown-name step **REMOVE**.

**MUST (graph, not JSON gates)**

- **achieves:** merge is TAP execute, not a policy-file farm.
- **origin:** census JSON and `governance/check` became the product; Cloud
  Build never shipped.
- **rule:** presubmit is fmt + clippy + nextest plus the short pattern-step
  table; path/count freeze JSON is not a gate; `governance/` is registry
  not CI; new check crates are born-blocking unless they are a pattern
  step in the `pipeline/` graph.
- **ensure:** no new `*-policy.json` freeze; no new `governance/check/*`
  census crate; GHA must not grow predicates this ADR deleted.
- **overturn_when:** a five-field ADR adds one engine that evaluates
  IR/Cedar/cargo graph without a frozen corpus.

### D-18 — `pipeline/` product vs GHA operator; purge `workflow/`; `.github/scripts` glue

**Product.** One execute engine: **graph + queue + schedule**. Analog:
Google **TAP** (internal) + **Cloud Build** (sold). Two **facades**, not
two codebases.

- **Internal:** this monorepo is **tenant #0**. D-10 cadences are that
  tenant’s graph. Same APIs a customer will call.
- **Sold:** tenant submits a graph; same schedule/queue.
- **Workers:** **`compute/` runs the work** (CH VM / Firecracker
  functions). `pipeline/` does not own a second cluster or a Prow job
  pool. Functions are one step type, not the only runner. Sold kube
  workers, if any, are `k8s/` nodes that **are** compute VMs.
- **Promotion / CD:** `pipeline/` **executes** the promotion graph
  (`dev` → `staging` → `canary` → `production`). `iac/` is desired
  state. `k8s/` is the cluster product. Images are `build/` artifacts in
  `storage/`. `cargo build --release` is a **CD graph step**, not
  presubmit.
- **Merge:** **one** required context: **presubmit**. GitHub merge queue
  is GitHub’s, via an adapter, then gone. No `presubmit`
  as a second protected check. No owned Gerrit/submit-queue in v1.
- **CAS + execute client:** When the cloud can **serve** `pipeline/`
  against `storage/` CAS and `compute/` RE, the execute client is
  **buck2** (hermetic **polyglot** action graph). The language set is
  **not closed**: C, C++, C#, Go, Java, Kotlin, Scala, Python, Ruby,
  JS/TS, Dart, Swift, Rust, Zig, Haskell, Assembly, R, SQL, Shell, and
  whatever else we need. Each is an **action + toolchain**, not a CI
  SKU. The sold graph IR is language-agnostic (D-3 protobuf). New
  language = `build/` toolchain (or tenant-provided tool in the graph),
  not a new cap and not a cargo/go/npm product. **Do not sell cargo.**
  **Do not** grow one SKU per language (the Actions-matrix anti-pattern).
  Tenant #0 is Rust-first **in this repo**; that is not the product
  contract. Cargo is not a second pipeline runtime. Keep cargo only
  where buck2 cannot for **this** repo (today: `Cargo.toml` + reindeer
  as crate **manifest** input to buck2; `cargo fmt` until buck2 owns
  format). rust-analyzer is not merge proof. Dual cargo+buck2 merge is
  forbidden.
  **v1** until that cloud is serving: cargo nextest is tenant #0
  presubmit (ADR-0716) because there is no live CAS+RE. **Overturn 0716
  same-wave** when pipeline **serves** the buck2 graph; cargo execute
  is then not needed. CAS up or weekly `buck2 build //...` alone does
  not overturn.
- **Not the product:** `.github/` GHA (adapter until the engine **runs**
  tenant #0); Tide/webhook-gateway as `core/`; Prow
  `GateRun` as `core/` (**REMOVE** — BUILD a clean graph+queue kernel;
  do not strangler GateRun into Cloud Build); JSON check fleets; a
  directory named `ci/`.

Today’s tree under `pipeline/` is **KEEP+WORK** at the **path** only.
Contents are not the product. BUILD `pipeline/core/` graph+queue.
REMOVE census, Tide-as-core, webhook-as-core, GateRun-as-core.

**MUST (one pipeline engine; GHA ≠ product)**

- **achieves:** sold TAP/Cloud Build cannot be Actions YAML or a JSON
  gate farm; internal and sold cannot drift into two pipelines.
- **origin:** `ci/` mixed census, GHA glue, Prow, and the name of a
  product we did not build.
- **rule:** capability is `pipeline/`; one execute engine (graph+queue);
  workers are `compute/`; promotion graphs are `pipeline/` execute, `iac/`
  desired state, `k8s/` the cluster; one required context `presubmit`;
  GateRun/Tide are not core; tenant #0 and customers share the engine;
  GHA is a temporary adapter; JSON/census/`governance/check` fleets are
  not the product; no root named `ci/`.
- **ensure:** no new crates under `ci/`; no new GHA glue outside
  `.github/scripts/` and workflow YAML; git mv of dumps does not make
  them `pipeline/core`.
- **overturn_when:** `pipeline/` runs tenant #0 and GHA is deleted
  same-wave, or a five-field ADR names a different sold slug.

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

**`bus/`.** Capability root is **`bus/`**. What goes here: owned **queue**
(competing consumers), **fan-out bus**, **seekable stream**, plus
transactional **outbox**. At-least-once; per-key order only. Analog:
Google Pub/Sub, AWS SQS+SNS, Azure Service Bus. Kafka/Pulsar are
**adapters** (or a sold protocol facade crate — not `core/`). Not the
serving path, not the outbox store, not mailbox, not sagas, not SES.
D-1: Check/IR/tuples are not a broker log. A directory named
`messaging/` is **REMOVE**, not an alias.

**MUST (owned bus, not Kafka)**

- **achieves:** async fabric matches cell-local serving; no consumer-group on Check.
- **origin:** Kafka-as-default is industry cargo-cult; hyperscalers sell MSK, they
  do not run Kafka as S3/IAM’s bus.
- **rule:** `bus/` core is the owned queue/bus/stream + outbox; Kafka/Pulsar only
  as `adapters/` or a sold facade crate; serving traffic is not a Kafka consume.
- **ensure:** no new `core/` crate named kafka/pulsar; workflow Kafka dumps stay
  deleted (D-18).
- **overturn_when:** a five-field ADR sells Kafka-protocol as `core/` with
  measured serving-path evidence.

**`notify/` not `comms/`.** Mailbox, Meet, messenger, calendar are **`app/`**
(D-22/D-23). Cloud `notify/` is **notification delivery** (email send, SMS,
push, messenger ping) — SES/SNS/FCM analog, not a mailbox. Apps raise
notifications through a notify adapter; mail as a *channel* is not `app/mail`.
**Purge** `comms/`. Rewrite `notify/` from that charter.

**`workflow/`.** Current tree is n8n/SaaS/bus/forms/tasks, not Step Functions.
**Purge** (`git rm -r workflow/`). Keep the **registry row** as the rewrite
destination. No empty scaffold. Do **not** strangler event-bus crates into
`bus/` from this junk — `bus/` already has kernels. Rewrite the
saga engine from the D-15 charter (proto/H3, studio as facade). Forms/tasks wait
for the apps discussion.

**MUST (GHA ≠ pipeline product)** — see D-18 one-engine MUST above. `workflow/`
implementation is gone pending rewrite; no dump resurrection.

## Rejected alternatives

- AWS EKS etcd journal as our store.
- A kubernetes.git port-engine as a hyperscale **operations** requirement.
  Google runs Borg, Meta Twine, AWS sells EKS without writing kube. Our
  fleet is `compute/`+`cell/`. Sold `k8s/` wraps **upstream**.
- Talos (Sidero) as the fleet node OS. Hyperscalers do not run Talos.
  Talos is a kube-node OS; at most a sold-SKU worker image.
- Asterinas or Hermit as **today’s** node kernel or a vacant `kernel/`
  rung “until it matures.” QEMU as the compute identity. Kubernetes as
  the cloud OS. Talos as the fleet OS.
- kube-rs / k8s-openapi as the fleet (Borg) control plane. Those
  libraries are a kube-apiserver client. Sold `k8s/` adapters only.
- Kata Containers as the fleet runtime (that is CRI/kube). gVisor as
  the fleet VMM (it is a container sandbox). Empty `kata/` / `gvisor/`
  roots. **`k8s-on-compute` as a compute reconciler** (sold kube is `k8s/`).
- Ontology kernel in `data/core`. Copilot/CLI as `intelligence/core`.
  Drive/Meet/PACS as storage facades. `build/` as a price list.
  Gateway (or nested cap) Cedar **engine**. Observability as the bill.
- Palantir Foundry as `data/` or as `intelligence/`. A `foundry/` **capability**
  root. Reviving intelligence/RAG “foundry.” `app/ontology` as a sibling of
  Foundry. D41-retiring docs/sheets/Pages/Grid.
- CUE+Timoni or Haskell as EaC wrap.
- Public JSON/REST as the destination codec.
- Standing gRPC (public or east-west) because a mesh automates HTTP/2,
  because middleboxes break HTTP/2, or because “binary/gRPC skips TLS.”
  Protobuf is already binary on Connect. TLS CPU is kTLS/ALTS-class, not
  plaintext and not a second RPC envelope. Leftover gRPC deletes.
- Istio/Linkerd/sidecar as SPIFFE identity.
- PQC/ECH only in prose (no `gateway/` TLS adapter crates). Classical TLS as
  the destination suite. ECH with no crate.
- On-path QUIC MITM, blocking UDP/443 on the **public** door, or turning
  ECH off so a NGFW can read SNI. Forcing QUIC/H3 **east-west** because
  the public door is H3 (DC plant is TCP-optimized). A `firewall/` cap.
- Forking Chromium / shipping Island-class browser as a cloud v1
  requirement so that endpoint DLP exists. Attestation is the port;
  their browser is the client.
- New `cloud-*` crates, `cloud/` root, or moving a dump into another cap
  because the current home is wrong. Wrong-home burns or rewrites; it
  does not change address.
- Zanzibar global tuple replica.
- EU GDPR as the sole compliance floor. EU as a country pack id. Combinatoric
  `packs/kr-eu` (or A×B×C×D pack ids) instead of a set on the record.
  Treating the tenant’s home country as the only pack.
- A pack-union engine beside Cedar. Fetching packs on Check. CaS as a
  DIY PDP for apps. DORA/CSAP as a per-RPC serving predicate. Encoding
  pack ids as ReBAC groups. Projection dimensions as free strings
  outside the Cedar schema.
- Mega EaC orchestrator / remote PDP on the hit path.
- Sync Merkle on every `Check`.
- Cap-root `catalog.yaml` / `manifest.json` as allowlist debt.
- Cap-root `policy/` for Cedar files (collides with the `policy/` engine) or dual `cedar/`+`policy/` children.
- Rehoming AUDIT-FINDINGS, IPs, scorecards, DPIA essays into `docs/` instead of deleting.
- Cap-root `contracts/` or root `contracts/*.yaml` as IDL SSOT.
- Hand-authored OpenSLO (clones or “unique”) as SLO source of truth.
- Standing REST/JSON transcode “until SDK is ready” (that is dual-stack debt).
- Hand-authored OpenSLO as a W0 source of truth.
- Branding the merge-blocking CI `ci-*` or adding one required check per capability.
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
- A second protected GitHub context (`presubmit`) or per-cap
  checks. Prow `GateRun` / Tide as `pipeline/core`. A pipeline-owned worker
  cluster beside `compute/`. `iac/` as the CD engine.
- Dual cargo+buck2 merge proof. Switching tenant #0 to buck2 because CAS
  exists but pipeline does not yet **serve** that graph. Cargo as a
  standing second pipeline runtime after cutover. Cargo-as-destination TAP.
  A sold per-language CI (cargo / go test / npm / mvn as products).
  Requiring tenant graphs to be Cargo.toml. A closed language enum that
  refuses Haskell/Zig/SQL/Shell/… without a new cap.
- Strangler-moving `workflow/` event-bus/saas/forms into `bus/` or `app/`
  instead of purge+rewrite.
- Keeping the slug `messaging/` as a live capability name (collides with
  `comms`). Law and registry are `bus/`.
- Allowing shell/Python/Go under `scripts/` or `tools/` because `.github/scripts`
  is allowed. The exception is prefix-exact.
- Keeping `comms/` as a cloud cap (mailbox/Meet/messenger/calendar). Those are
  apps. Cloud send is `notify/`.
- Kafka (or Pulsar) as the `bus/` engine / serving consume path.
- `intelligence/detection/` as this cap’s core (GuardDuty ≠ Vertex). Copilot UX in `intelligence/core`.
- Marketplace as KYC/escrow/app-store; strangler of `developer-sdk/` into billing.
- `data/core/cloud-*` crates or a `cloud-data/` nest. Search/SERP/RAG in `data/core`.
- Putting Spanner/Cockroach/RDS in `storage/`. Putting BigQuery in `storage/`. A `search/` capability for Google Search.
- Claiming Spanner TrueTime / zero-ε without a measured GNSS/atomic plant.
  Waiting for GNSS hardware before the interval API exists. A product
  `Now() → Instant`. Selecting the clock adapter through `flags/`.
  Forbidding PTP/GNSS adapters because v1 is NTP. A `time/` capability.
  Commit-wait on the `Check` path. Commit-wait of NTP ε on every v1
  OLTP write. Treating Cockroach-on-AWS as the operator plant. Two
  APIs (software clock vs hardware clock).

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
- **presubmit name:** TAP/presubmit vs postsubmit. Not `presubmit`.
- **One context vs per-cap CI:** Central **admission** is hyperscaler; central
  **full-repo JSON census** is the conflict source. Per-cap **required** checks are
  the skip-fail anti-pattern.
- **etcd / AWS journal:** EKS journal is closed, etcd-API-preserving, mega-cluster.
  We cell-shard; steal log-vs-memory, not the binary.
- **EU world-floor:** KR CSAP/본인인증/e-tax is not a GDPR subset. Packs overlay.
