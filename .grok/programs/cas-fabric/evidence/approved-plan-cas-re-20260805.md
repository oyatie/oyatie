# DELIBERATE Ralplan Consensus Plan: CAS/RE Hyperscaler Pattern and Capability-First Reorganization

> **Status:** **CONSENSUS APPROVED** on 2026-08-05 after sequential Planner -> Architect -> Critic review. This is planning and execution-handoff authority only; it does not approve PR #1558, CAS/RE activation, bridge deletion, or production mutation.
> **Evidence baseline:** `origin/dev@b64eaaf4ab40f7428e3a27d7cd4b02930404eee9`, current GitHub state observed 2026-08-05, `.omx/context/cas-re-hyperscaler-20260805T051909Z.md`, `.omx/drafts/mac-oci-owned-ephemeral-ci-idea-20260805.md`, `.omx/context/mac-oci-owned-ci-analysis-20260805.md`, `.omx/plans/cas-re-hyperscaler-architect-review-final-approved-20260805.md`, and `.omx/plans/cas-re-hyperscaler-critic-review-approved-20260805.md`.

## 1. Desired Outcome

Deliver the smallest safe progression from today’s GitHub Actions/ARC bridge to an owned, ephemeral CI and remote-build fabric:

```text
Owned SCM/admission/event plane
  -> immutable global admission/routing
  -> cell-local CI queue/leases + scheduler
  -> ephemeral coordinators -> Buck2
  -> cell-local CAS + Action Cache
  -> optional distinct cell-local RE queue/leases + scheduler
  -> ephemeral hermetic ARM64/AMD64 workers
```

CAS must be proven and operated independently before any remote-execution activation. GitHub, GitHub Actions, and ARC are temporary proof/transition surfaces, not north-star components.

Every touched artifact must first pass a present-need challenge, then prefer:

1. delete;
2. derive;
3. merge;
4. reuse;
5. rehome only irreducible live behavior.

No lift-and-shift into new `cloud/`, `oya/`, `libs/`, `infra/`, or speculative canonical scaffolds.

---

## 2. Requirements Summary

### Required

- Preserve `oya-ci-required` as the current single admission context until owned admission reaches exact parity.
- Use ephemeral, one-job coordinators separated by trust class.
- Preserve cold/untrusted lanes with no CAS identity, no cache access, and no inherited warm configuration.
- Prove cache-only CAS/AC with `remote=0` before designing or activating RE.
- Separate owned CI scheduling/control, ephemeral coordinators, CAS/AC, RE scheduling, and per-architecture workers.
- Enforce the single normative §5.5 per-RPC authorization matrix for every identity-bearing CAS/AC/RE operation; no narrower authorization summary is authoritative.
- Keep action execution unable to access worker identities, coordinator credentials, GitHub tokens, control endpoints, or unrestricted egress.
- Support ARM64 and AMD64 as distinct execution platforms, images, capacity pools, and action-key dimensions.
- Align placement with Accepted ADR-0562/0615:
  - `build/`: crate-free Buck2 prelude, toolchains, execution-platform declarations, static build machinery;
  - `ci/{core,ports,adapters,facade}`: delivery-fabric runtime, scheduling/control, admission and provider adapters;
  - `storage/{core,ports,adapters}`: CAS semantics, REAPI storage contracts, cache providers;
  - Cilium/Talos/OpenBao/GitOps evaluated individually against `network/`, `os/` or `k8s/`, `secrets/`, and `iac/`.
- Use `tools/oya-reorg-codemod-app` and at most one active committed `specs/reorg/*-move-plan.json` per rehome lane. Derive the move manifest on demand; never re-track or hand-edit it.
- Use isolated worktrees with one temporal owner per touched path, no concurrent ownership, no consumption of an unpromoted predecessor, SSH-signed commits, protected PRs to `dev`, independent review, exact-head `oya-ci-required`, and squash merge.
- Include rollback, telemetry, audit evidence, browser/user-story evidence where applicable, release impact, and observation harvesting.

### Explicit Non-goals

- No implementation or production mutation from this plan.
- No merge approval of PR #1558 or any other PR.
- No RE activation under Proposed ADR-0612.
- No permanent GitHub Actions or ARC architecture.
- No whole-`infra/` relocation.
- No universal container image.
- No runner-count increase justified only by available CPUs.
- No production durability claim from one RWO `local-path` volume.
- No stretched Kubernetes cluster across Mac-to-OCI WAN.
- No hand-edited `*.generated.json`.
- No preservation of the archived RE/PDP/Envoy prototype.
- No production CAS vendor decision until license, durability, and recovery gates resolve.
- No assumption that OCI A1 provides a continuously free 4-OCPU/24-GB control plane.

---

## 3. Authority Audit

### Binding placement and governance authority

| ADR | Status | Planning consequence |
|---|---|---|
| ADR-0515 | Accepted | Current single canonical admission and owned CI/CD north star. |
| ADR-0562 | Accepted | Capability-first topology; supersedes `{oya,cloud}/service + libs` as destination doctrine. |
| ADR-0614 | Accepted | Reorg move manifest remains derived and de-committed. |
| ADR-0615 | Accepted | Capability boundaries and `build/` versus `ci/` distinction constrain placement. |
| ADR-0616 | Accepted | Frozen reference faces derive from immutable merge-base source. |
| ADR-0619 | Accepted | Retired external coordination brand cannot re-enter active context. |
| ADR-0624 | Accepted | Immutable ADR-census transition; does not activate other Proposed ADRs. |
| ADR-0632 | Accepted | Internal gRPC/TLS posture, OpenTelemetry, Cilium/Hubble, and provider-fabric limits. |
| ADR-0635 | Accepted | Face-aware dependency graph is authoritative only for its bounded 19-unit slice. |
| ADR-0636 | Accepted | Bounded cross-run affected baseline reuse; not CAS, RE, or capacity authority. |

### Design input only

| ADR | Status | Constraint |
|---|---|---|
| ADR-0556 | Proposed | Cache trust classification is useful design input, not activation authority. |
| ADR-0560 | Proposed | Existing NativeLink cache artifacts do not authorize live warm reads. |
| ADR-0600 | Proposed | Root-hygiene design does not constrain CAS activation. |
| ADR-0612 | Proposed | Does not authorize scheduler, workers, or `remote_enabled=true`. |
| ADR-0617 | Proposed | Does not activate a living governance graph. |
| ADR-0630 | Proposed | ARC may describe the current bridge but cannot make it permanent. |
| ADR-0631 | Proposed | A backlink may expose amendment history but does not amend Accepted topology until accepted. |

Before a warm-license flip, existing cache trust and NativeLink adoption decisions must gain Accepted cross-artifact authority or be replaced by a narrowly scoped Accepted decision. Legal and architecture approval of NativeLink FSL use is a separate hard gate.

PR #1559 improves decision discoverability only. It cannot promote Proposed ADRs by adding backlinks.

### Complete ADR-0600–ADR-0636 disposition

| ADR | Status on `origin/dev` | CAS/RE disposition |
|---|---|---|
| ADR-0600 | Proposed | Root-hygiene design input only; no activation or placement authority. |
| ADR-0601 | Absent/reserved | No decision file; supplies no authority. |
| ADR-0602 | Absent/reserved | No decision file; supplies no authority. |
| ADR-0603 | Proposed | CRM fail-closed authz precedent only; not a CAS/RE decision. |
| ADR-0604 | Proposed | De-committed SCM-facts design input; current derivation is constrained by later Accepted ADR-0616. |
| ADR-0605 | Proposed | Supply-chain audit design input for images/SBOM/provenance; not activation authority. |
| ADR-0606 | Proposed | Secret-bootstrap/RBAC negative-test input; #1541 and Accepted security controls remain the gate. |
| ADR-0607 | Proposed | Fail-closed Kubernetes authorization precedent; not RE sandbox authority. |
| ADR-0608 | Proposed | Cedar deploy-parity design input; useful for negative fixtures only. |
| ADR-0609 | Proposed | Masterplan design input amended by Accepted ADR-0619; retired-brand constraints win. |
| ADR-0610 | Proposed | Preregistered-rubric precedent reused for the workload/capacity corpus. |
| ADR-0611 | Proposed | Kernel real-boot design input; no CAS/RE activation effect. |
| ADR-0612 | Proposed | RE design input only; explicitly cannot authorize scheduler, workers, or `remote_enabled=true`. |
| ADR-0613 | Accepted | Controller projections remain derived/de-committed; do not mint a new committed projection for this plan. |
| ADR-0614 | Accepted | Move manifest is derived on demand and cannot be re-tracked without an Accepted reversal. |
| ADR-0615 | Accepted | Binding capability boundaries and `build/` versus registered `ci/` ownership. |
| ADR-0616 | Accepted | Immutable merge-base-source regeneration constrains verification evidence. |
| ADR-0617 | Proposed | Living-graph design input only; cannot create planning or runtime authority. |
| ADR-0618 | Proposed | Contract-slice scope precedent; tests must state their bounded claim. |
| ADR-0619 | Accepted | Retired external coordination brand remains absent from live context. |
| ADR-0620 | Proposed | History-only observation provenance design input; not current operational authority. |
| ADR-0621 | Proposed | Active-contract graph de-commit design input; no new committed derived graph. |
| ADR-0622 | Proposed | Friction-ledger successor design input; reuse current audit/mistakes surfaces. |
| ADR-0623 | Proposed | Evidence-epoch proposal remains nonbinding; ADR-0624 explicitly does not accept it. |
| ADR-0624 | Accepted | ADR-census epoch transition only; no implied acceptance of neighboring ADRs. |
| ADR-0625 | Proposed | OpenTofu lock design input for future OCI/IaC work only. |
| ADR-0626 | Proposed | Structural ledger merge-driver input; no relevance to activation. |
| ADR-0627 | Proposed | Facade/core gate design input; Accepted ADR-0562/0615 placement remains binding. |
| ADR-0628 | Proposed | Scan-root liveness precedent: zero observations are not success. |
| ADR-0629 | Proposed | Catalog coverage precedent for image/artifact inventory; no activation authority. |
| ADR-0630 | Proposed | ARC describes a current bridge and its parity deletion trigger; it is not north-star authority. |
| ADR-0631 | Proposed | Boundary-test input only; its backlink does not amend Accepted topology. |
| ADR-0632 | Accepted | Binding internal TLS/gRPC, telemetry, Cilium/Hubble, and provider-fabric posture. |
| ADR-0633 | Proposed | Design input: enforcement lives with the owner of the fact, population parity is required, and zero observations are RED rather than a clean result. |
| ADR-0634 | Proposed | Producer-bound approval design input; receipts must bind the producing identity and exact attempt. |
| ADR-0635 | Accepted | Binding only for the declared bounded face-aware graph; no false full-topology claim. |
| ADR-0636 | Accepted | Bounded immutable affected-baseline reuse; not runner-capacity, CAS, or RE authority. |

---

## 4. RALPLAN-DR

### Principles

1. **Causality before scale:** CAS correctness and isolation precede RE; measured queue pressure precedes concurrency or fleet scale-out.
2. **Ownership before vendor:** place behavior by capability and face, never by the fact that NativeLink, ARC, Talos, Cilium, OpenBao, or Argo currently implements it; when an active package crosses an Accepted ownership boundary, move the complete package atomically rather than leaving split authority.
3. **Shared nothing across trust and failure domains:** separate identities, instance names, worker pools, queues, cache permissions, and architecture-specific execution platforms.
4. **Deletion-first transition:** bridges carry explicit deletion triggers; no copied authority or permanent dual homes.
5. **Evidence defines maturity:** proof-cell success, production readiness, activation, and scale-out are distinct claims with distinct gates.

### Top Decision Drivers

1. Prevent arbitrary build actions from acquiring identities, mutating trusted cache state, or expanding blast radius.
2. Keep the required merge path available and reproducible while removing GitHub/ARC dependence.
3. Reach canonical ownership without turning reorganization into a risky, repository-wide lift-and-shift.

### Options

#### Option A — Mac-local proof cell, then staged owned-fabric cutover

Keep control, coordinators, CAS, and workers in one homogeneous Mac-hosted Talos proof cell. Use GitHub Actions/ARC only as the temporary ingress/coordinator bridge.

**Pros**
- Lowest latency and simplest failure model.
- Best environment for cache-only and first RE sandbox proof.
- No WAN-coupled Kubernetes control plane.
- Minimal infrastructure cost.

**Cons**
- Laptop outage removes CI, data, and execution.
- Single local storage and power/network domain.
- Cannot support a production-availability claim.

**Disposition:** Use as the proof cell only.

#### Option B — OCI Kubernetes control plane with Mac workers and data plane

Move only the Kubernetes/Talos control plane to OCI A1 while retaining workers, CAS, OpenBao, and data locally.

**Pros**
- Control-plane process survives a Mac reboot.
- Small apparent OCI footprint.

**Cons**
- CI and data still fail when the Mac is unavailable.
- Introduces WAN latency and partition sensitivity into Kubernetes quorum, heartbeats, reconciliation, secret delivery, and recovery.
- Expands certificate, routing, firewall, and recovery surface without improving workload availability.
- OCI Always Free A1 is currently bounded to 2 OCPU/12 GB total; continuous 4 OCPU/24 GB is paid, and capacity may be unavailable or reclaimed.

**Disposition:** Reject for production and normal proof work. Permit only as a deliberately disposable WAN-failure experiment.

#### Option C — Split at the owned CI/RE application protocol

Keep the Mac Talos execution/storage cell homogeneous. Place only owned SCM event ingestion plus immutable admission/routing on OCI, sized to measured capacity; keep mutable CI/RE queues, leases, schedulers, coordinators, AC/CAS and workers cell-local. Mac-side cell agents initiate outbound mTLS sessions. Add independent homogeneous execution cells later rather than stretching one Kubernetes cluster.

**Pros**
- Failure boundary aligns with stable application protocols, not Kubernetes internals.
- Mac remains a self-contained execution cell.
- Control services can scale and recover independently.
- Enables later shared-nothing Mac, OCI, or other cells.
- Removes inbound worker exposure.

**Cons**
- Global admission/routing availability remains limited if deployed within the free A1 allowance.
- WAN interruption delays work even though it does not corrupt cluster state.
- Requires owned control services before GitHub/ARC removal.

**Recommendation:** Option C as the post-proof target, with Option A as the immediate proof cell. Do not recommend Option C for merge authority until the discriminating proof passes.

#### Option D — Owned CI plus CAS/AC, with local execution as the durable stopping point

Complete owned SCM/admission, ephemeral coordinators, and correct CAS/AC, but retain local-on-coordinator execution and close the program without RE.

**Pros**
- Removes GitHub/ARC dependence without adding a second scheduler or arbitrary-code worker identity plane.
- Captures cache reuse and owned admission value with materially less security and operational surface.
- Preserves future REAPI compatibility if evidence later justifies RE.

**Cons**
- Coordinator resource limits continue to bound execution parallelism.
- Large hermetic action graphs may retain higher p95 wall time than RE could achieve.

**Disposition:** This is a fully viable success state, not a deferral. Reopen RE only when all of the following are measured on the preregistered corpus:

1. at least 20 consecutive owned-CI-plus-CAS green runs across at least 7 days;
2. cache-only end-to-end gate p95 remains above 15 minutes and local action execution remains at least 60% of wall time;
3. admitted-load queue age exceeds 120 seconds p95 at the proven physical concurrency cap in three separate 15-minute windows;
4. at least 90% of the proposed RE canary actions pass hermeticity and deterministic-output qualification;
5. a measured model predicts either at least 30% p95 wall-time reduction or at least 20% cost-per-successful-gate reduction after including scheduler, worker, storage, and control-plane cost.

If any trigger is absent, stop at owned CI plus CAS/AC and record the next measurement watermark.

### Critic-resolved placement and readiness decisions

- Promote three sequential PRs in order, each based on its predecessor's promoted head: NativeLink manifest/ownership to `storage/adapters/nativelink/` (3A), the complete active Buck2 cache package plus both overlays and every necessary active consumer to exactly `build/buck2/cache/` (3B), then behavior-only CI resolver/policy closure on canonical paths (3C). Ownership is exclusive in time, not permanently file-disjoint.
- Lane 3B creates no alias, forwarding target, copied source or dual home; root `.buckconfig` remains dark. Lane 3C performs daemon-lifecycle, policy-semantics and no-inheritance/regression work only. Any old path or label at 3C start is a hard RED proving 3B did not complete.
- Before Lane 5A, the owned SCM/event/status lane must prove only the bounded SCM/CI readiness slice mapped to `specs/bespoke-cloud-toolchain-services.json` T1→T3 and issue a current signed readiness receipt covering every in-scope replacement, rollback and deletion inventory row. Lane 5A performs only the SCM/CI portion of T4; cloud-cd/Argo T2–T4 remain retained, open and out of scope.

### Discriminating Topology Proof

Before choosing OCI as a persistent control location:

1. Deploy a non-authoritative immutable admission/routing slice within 2 OCPU/12 GB; the Mac retains the mutable cell-local queue/leases and scheduler.
2. Have the Mac cell establish outbound-only mTLS and execute bounded synthetic plus real Buck2 jobs.
3. Run for at least 72 hours and inject WAN loss and latency, OCI process restart, Mac sleep/reboot, duplicate delivery, expired identity, and queue overload.
4. Measure enqueue-to-start p50/p95/p99, reconnect and redelivery time, duplicate execution rate, lost-job rate, egress bytes and cost per successful gate, queue depth, scheduling delay, and Mac saturation.
5. Go only if work is never lost or falsely marked successful, partitions preserve bounded queued state, and recovery requires no Kubernetes cross-WAN repair.

A negative result records a verified no-op in the existing audit chain/mistakes ledger with scope, watermark, and next trigger; it does not create a new Markdown authority.

---

## 5. Architecture

### 5.1 Current Bridge Proof Architecture

```text
GitHub PR/push
  -> GitHub Actions / oya-ci-required
  -> ARC ephemeral coordinator pod
       - cold/untrusted: no CAS credentials, no participation
       - trusted reader: read-only identity
       - postmerge writer: job-scoped write identity
  -> Buck2 local execution
  -> NativeLink CAS/AC proof service
       - one replica
       - RWO local-path volume
       - cache-only execution platform
       - remote_enabled=false
```

This can prove cache behavior, not production availability or RE.

### 5.2 North-star Production Architecture

```text
Owned SCM
  -> immutable global admission decision + cell routing only
       -> cell A
            CI queue/leases -> cell-local CI scheduler -> ephemeral coordinators -> Buck2
            cell-local AC namespace + CAS partition/provider adapter
            [RE activated only if reopened]
              distinct RE queue/leases -> distinct cell-local RE scheduler
                -> linux-arm64-glibc workers
                -> linux-amd64-glibc workers
                -> separate musl/native pools only on proven demand
       -> cell B
            same isolated mutable components and per-cell quotas/failure budget

 Cross-cell: asynchronous content-verified immutable CAS blob replication only
```

### Required separations

- Global control admits immutable intent and routes it to one cell; it owns no mutable execution queue, lease, AC, CAS partition, coordinator or worker state.
- Within each cell, CI scheduling and RE scheduling use distinct queues, lease stores, schedulers and failure planes.
- ARC coordinators and RE workers are never the same pods or node pool.
- Scheduler loss may delay execution but must not make CAS unavailable.
- CAS hashes establish content identity, not provenance or authorization.
- AC retention cannot exceed the availability of referenced CAS outputs.
- Worker platform matching includes architecture, libc, toolchain/image digest, sandbox class, and trust class.
- Coordinators possess submission identity; workers possess only the minimum worker identity.
- Executed actions see neither identity.

### 5.3 Provider-neutral tenant/job envelope

Every admission, queue, cache, and execution transition carries one immutable envelope independent of GitHub, ARC, or a particular RE vendor:

| Field | Required meaning |
|---|---|
| `tenant_id` | Billing, policy, cache and fairness tenant boundary. |
| `cell_id` | Single shared-nothing execution/storage cell. |
| `pipeline_id` | Stable owned-CI pipeline identity. |
| `source` | Provider-neutral SCM repository/ref/revision plus trusted source event identity. |
| `work_area_digest` | ADR-0517/0520 content-addressed work-area identity. |
| `action_digest` | Canonical REAPI action key including input root and command. |
| `platform` | Architecture, OS, libc, toolchain and sandbox properties. |
| `image_digest` | Exact multi-arch child image used by coordinator or worker. |
| `trust_class` | Cold/untrusted, trusted-reader, postmerge-writer, or RE-eligible class. |
| `attempt` | Unique run and retry attempt; never inferred from a mutable name. |
| `expires_at` | Maximum authorization/lease lifetime. |
| `fencing_token` | Monotonic lease generation preventing stale worker completion. |
| `audit_event_id` | Stable link to admission, authorization, execution attestation and result. |

Unknown, expired, cross-tenant, cross-cell, unfenced, or incomplete envelopes are refused. Retries mint a new attempt and fencing token while preserving causal links.

### 5.4 Shared-nothing cell contract

```text
Owned global admission/routing (immutable intent and immutable verdict references only)
  -> cell A
       CI queue/leases -> CI scheduler -> ephemeral coordinators -> Buck2
       AC namespace + CAS partition/provider adapter
       optional RE queue/leases -> separate RE scheduler -> per-arch hermetic workers
       quotas + overload policy + failure budget
  -> cell B
       independently owned instance of the same mutable components

Cross-cell: asynchronous replication of immutable CAS blobs only
Forbidden: global/shared mutable CI or RE queue, lease table, AC, CAS metadata,
           synchronous cross-cell execution, or shared mutable runner farm
```

Each cell owns CI queue/lease recovery, CI scheduler capacity, ephemeral coordinators, AC namespace, CAS partition/provider adapter, and quotas/failure budget. If RE is activated, that cell separately owns its RE queue/leases, RE scheduler and per-architecture hermetic workers. An unavailable cell may return immutable unstarted intent to global routing for a new cell-specific attempt; another cell never mutates the first cell's queues, leases or AC. Blob replication is content-verified, asynchronous, immutable and safe to replay.

### 5.5 Authorization and execution attestation

This is the **single normative authorization reference** for CAS/AC/RE. Every other section cites this matrix and may not narrow it.

| RPC/operation class | Required authorization inputs | Required refusal cases |
|---|---|---|
| CAS discovery/capabilities and identity-bearing metadata reads | authenticated workload URI SAN, tenant/job envelope, cell and instance | anonymous or sibling identity, wrong tenant/cell/instance, expired attempt |
| CAS blob/missing-blob reads | matrix inputs plus digest and permitted trust/cache class | unauthorized digest/namespace, cross-cell mutable lookup, quarantined generation |
| CAS uploads/writes | matrix inputs plus writer role, size/digest verification and fencing token | reader/untrusted role, stale fence, hash/size mismatch, replay |
| AC lookup/read | matrix inputs plus action digest, platform and image/toolchain identity | cross-tenant/cell/platform/image lookup, quarantined action |
| AC update/write | matrix inputs plus authorized producer and complete execution attestation | unattested/mismatched result, stale fence, non-writer, unauthorized output digest |
| RE execute/submit | matrix inputs plus RE-eligible trust, action/input/platform/image/sandbox qualification | caller-controlled trust, unqualified platform/image/sandbox, wrong cell |
| RE cancel | original submitter or explicitly authorized control identity, exact attempt and fence | sibling attempt, stale fence, unrelated tenant/pipeline |
| RE result/status read | authorized submitter/control identity and exact envelope | anonymous, sibling identity, cross-tenant/cell attempt |
| CI/RE queue and worker lease acquire/renew/release | cell-local scheduler/worker identity, exact attempt, expiry and monotonic fence | global or cross-cell mutator, expired identity, stale/duplicate fence |

Discovery that is intentionally public must be a separately Accepted non-identity-bearing endpoint; it is never an implicit exemption from this matrix.
- Policy inputs come only from the trusted envelope and authenticated workload identity; caller headers cannot supply trust.
- Negative fixtures cover missing/expired identity, wrong tenant/cell/trust class, sibling URI SAN, stale fencing token, replayed attempt, cross-instance digest, unauthorized read, unauthorized write, PDP outage and unknown RPC method.
- Execution attestation binds `tenant_id`, `cell_id`, `pipeline_id`, source revision, work-area/input-root digest, action/command digest, platform, exact image child digest, worker URI SAN, sandbox profile/version, attempt, fencing token, output digests, timestamps and audit event.
- An unattested, partially attested or mismatched result cannot populate AC or satisfy admission.

---

## 6. Ranked Anti-patterns

### P0 — Stop immediately

1. Reusing any Talos, OpenBao, ARC, NativeLink, or cache credential implicated by #1541.
2. Accepting caller-controlled headers as build class, trust, identity, or policy evidence.
3. Any identity-bearing CAS/AC/RE RPC that does not conform to the complete §5.5 per-RPC authorization matrix.
4. Mounting worker SVIDs, tokens, control sockets, or authorized egress into the action sandbox.
5. Treating same-CA membership as authorization instead of exact URI-SAN workload identity.
6. Activating RE before cache-only parity, integrity, and rollback proof.
7. Merging or copying the archived RE/PDP/Envoy prototype.

### P1 — Must be removed before production claims

8. Single-replica RWO local-path CAS described as production.
9. CAS and scheduler sharing one availability or rollout unit.
10. ARC runner pods doubling as RE workers.
11. Shared identities, cache instances, or mutable workspaces across trust tiers or architectures.
12. CPU-only HPA, fake node autoscaling on one desktop, or scale-out without queue and I/O evidence.
13. Scaling runners while storage, workspace, and Buck2 parallelism remain the bottleneck.
14. One universal Linux image across coordinators, control services, and workers.
15. Stretched Kubernetes control plane across OCI-to-Mac WAN.

### P2 — Reorganization and operability debt

16. Moving `infra/*` wholesale rather than classifying each artifact by owner.
17. Copying artifacts to canonical paths while leaving live sources.
18. Re-tracking or hand-editing the generated reorg move manifest.
19. Treating Proposed ADRs or draft PRs as activation authority.
20. Retaining GitHub/ARC-specific workflows, OIDC roles, images, labels, or PowerShell/Node baggage after owned cutover.
21. Autoscaling without cost per successful gate/action, scheduling delay, saturation, cache latency, and disk/IO telemetry.

---

## 7. Adaptive Dependency-ordered Lanes

Each lane uses a fresh isolated worktree from its required predecessor's promoted head (`origin/dev` when it has no predecessor), one temporal owner for every touched path, one reviewable PR, and no downstream work consuming an unpromoted predecessor. Ownership may pass sequentially after promotion; it is never concurrent.

### Binding execution DAG

```text
A. source -> target -> owner -> dependency map
  -> B. independent map review
  -> C. one representative low-coupling #1558 rehome trial
  -> D. exact post-merge source-deletion/reverse/consumer proof
  -> E. fan out only the independently proven present-need rehomes

Parallel to A-D: #1541 non-mutating inventory, recovery and ceremony preparation only
Before any live CAS deployment: #1541 rotation/rebuild/purge completion
Off critical path: #1559 metadata correction
```

No second rehome begins before the representative trial is promoted and its post-merge proof is accepted. PR #1559 may land whenever independently green; it cannot block or authorize the DAG.

### Lane 0 — Authority, boundary map, and PR sequencing

**Anchors**
- `docs/decisions/ADR-0515-*.md`
- `docs/decisions/ADR-0562-*.md`
- `docs/decisions/ADR-0614-*.md`
- `docs/decisions/ADR-0615-*.md`
- `docs/decisions/ADR-0612-*.md`
- `docs/decisions/ADR-0630-*.md`
- `specs/capability-registry.json`
- `specs/substrate-dependency-dag.json`
- PR #1559

**Actions**
1. Produce the source-target-owner-dependency boundary map in implementation PR evidence, not a new Markdown authority.
2. Obtain independent review of that map before selecting the one representative trial.
3. Separate Accepted authority, Proposed design input, operational fact, and future decision.
4. Resolve NativeLink FSL legal/architecture status.
5. Treat PR #1559 as off-path metadata maintenance; merge only after its own review and green exact-head CI.
6. If cache activation remains desired, promote the existing cache decisions through the required Accepted propagation rather than creating a duplicate ADR.

**Go:** placement and activation authority are unambiguous; compliance traceability rows identify primary source, effective date, applicability, control, owner, evidence, and review date.

**Stop:** any required decision is still merely Proposed or contradictory.

### Lane 1 — Disposition PR #1558 as the low-coupling trial slice

**Current PR:** draft #1558, head `54b22d0c6470d8008012542eb37d0ff32b72e1b5`.

| Candidate | Present need | Disposition |
|---|---|---|
| `infra/gitops/local-path-storage.yaml` | Generic `local-path` is referenced by NativeLink, registry/SeaweedFS surfaces, and `infra/gitops/bootstrap-sync.yaml`; the existing ARC workspace provisioner is separate and insufficient. | **Keep behavior, rehome only if proof cell uses it.** Prefer `storage/adapters/local-path/` as the provider-owned proof adapter; point the GitOps Application at that path. |
| `infra/talos/qemu-cilium.patch.yaml` | Its two settings already exist in current Talos/CAPI declarations, but the QEMU proof path lacks a tracked consumer. | **Delete from #1558 unless an exact QEMU command/config consumer is added.** Reuse existing Talos declarations; do not keep a duplicate patch merely for a structural test. |
| additions to `infra/arc/tests/ci_workspace_capacity.rs` | Mix ARC capacity ownership with storage and Talos prerequisites. | **Split.** Keep only ARC capacity assertions here; validate storage/Talos behavior with their owning surface’s smallest runnable check. |

**Migration mechanism**
- Use `tools/oya-reorg-codemod-app`.
- Maintain one active committed move plan for this trial.
- Derive `specs/reorg/move-manifest.generated.json` on demand only.
- No branch path under `infra/gitops/local-path-storage.yaml` may merge and later become a second source.
- Before modification, execute the owner-approved population query and record `N_pre`, the number of governed live pilot artifacts and consumer edges the query is required to observe. `N_pre = 0` is RED and stops the pilot.
- Validate that query with an independent positive probe shaped differently from the pilot artifact (different path/document structure but the same governed classification). The query must detect exactly the expected additional population; removing the probe must restore `N_pre`.
- After the codemod, execute the same behavior-level query and require `N_post = N_pre`. Path names may change; consumer behavior, ownership, enforcement coverage and population may not.

**Source deletion criteria**
- All GitOps references resolve to the canonical adapter path.
- Owner-local validation runs.
- `git grep` finds no runtime reference to the rejected branch path.
- Codemod reverse proof is byte-stable.
- Tests are confirmed executed, not skipped.
- The PR receipt records `N_pre > 0`, successful differently shaped positive-probe detection, and `N_post = N_pre` with the exact query/version and immutable source/head SHAs.

**PR boundary:** revise #1558 to this single low-coupling storage prerequisite or close it and replace it with one narrower PR. It must not merge in its current three-concern form.

### Lane 2 — Close credential incident #1541

**Anchors:** issue #1541, `infra/talos/`, `infra/external-secrets/`, and current OpenBao, ARC, and NativeLink identities.

**Actions**
- During Lanes 0–1, perform only non-mutating inventory, recovery validation, consumer mapping, support-case preparation and ceremony rehearsal.
- Complete inventory without secret values.
- Verify encrypted recovery.
- Rotate or rebuild every exposed credential class.
- Obtain GitHub dangling-object purge evidence.
- Build a new proof cell from fresh trust roots.
- Start with an empty CAS generation.
- Prove old Talos, Kubernetes, bootstrap/join, ARC, OpenBao, and NativeLink credentials fail.

**Go:** independent security and operations review accepts the completion packet.

**Stop:** unknown consumer, incomplete backup, old credential still accepted, or secret output entering logs.

### Lane 3 — Three sequentially owned CAS/build/CI PRs

Strict order: **3A promoted and post-merge proven → 3B starts from the 3A promoted head, is promoted and post-merge proven → 3C starts from the 3B promoted head, is promoted and proven**. Each PR has one isolated worktree and one temporal owner for every file it touches; no path has concurrent owners and no lane consumes an unpromoted predecessor. Lanes may touch the same consumer files sequentially when required. Lanes 3A and 3B each bind exactly one active committed move plan and repeat the Lane-1 population-parity contract (`N_pre = N_post = N_promoted > 0` plus a differently shaped positive probe); Lane 3C creates no move plan because it is behavior-only.

#### Lane 3A — NativeLink CAS provider rehome

**Exact move**
- `infra/nativelink/nativelink-cas.k8s.yaml` → `storage/adapters/nativelink/nativelink-cas.k8s.yaml`
- `infra/nativelink/OWNERS` → `storage/adapters/nativelink/OWNERS`

**Ownership and move authority**
- Owner: storage/CAS adapter lane.
- Start point: the then-current promoted `dev` head after all declared predecessors.
- One isolated worktree and exactly one active committed move plan, `specs/reorg/nativelink-storage-move-plan.json`; derive the move manifest on demand per ADR-0614.

**Same-PR requirements**
- Update GitOps/reachability/ownership/validation consumers to the storage adapter path.
- Preserve cache-only behavior, identities, instance names, image digest and proof-only RWO topology byte-for-byte except path-owned metadata.
- Delete `infra/nativelink/` when empty; no symlink, copy, alias or compatibility path.

**Verification**
- YAML/embedded NativeLink JSON parse, provider-manifest structural tests, GitOps source resolution, full consumer grep, codemod reverse proof, targeted storage tests and population parity before and after promotion.

#### Lane 3B — Complete active Buck2 cache execution-platform package move

**Grounded decision:** move the complete active package and both overlays into exactly `build/buck2/cache/`, because Accepted ADR-0562 assigns Buck2 prelude/toolchains/static CI machinery to `build/`. Root `.buckconfig` stays dark.

**Exact move**
- `toolchains/cache/BUCK` → `build/buck2/cache/BUCK`
- `toolchains/cache/defs.bzl` → `build/buck2/cache/defs.bzl`
- `toolchains/cache/OWNERS` → `build/buck2/cache/OWNERS`
- `infra/ci/buckconfig/warm-cache-ro.buckconfig` → `build/buck2/cache/warm-cache-ro.buckconfig`
- `infra/ci/buckconfig/warm-cache-rw.buckconfig` → `build/buck2/cache/warm-cache-rw.buckconfig`

**Ownership and move authority**
- Owner: the sole temporal path/label migration owner across build, CI, workflow and test consumers for this PR.
- Start point: the exact promoted 3A head; an unpromoted or merely green 3A head is not consumable.
- One isolated worktree and exactly one active committed move plan, `specs/reorg/buck2-cache-move-plan.json`; derive the move manifest on demand.

**Atomic same-PR requirements**
- Rewrite every `toolchains//cache:cache-platform` label to the exact canonical `//build/buck2/cache:cache-platform` label.
- Update every necessary active consumer in the same PR, including resolver constants, CI runtime/policy references, bridge-workflow references, BUCK loads/inputs, reachability, OWNERS, comments, tests and conformance fixtures. Governed specs and historical prose remain in place and are changed only if they contain an active machine-consumed path/label.
- Delete `toolchains/cache/` and `infra/ci/buckconfig/` when empty.
- Prohibit compatibility aliases, symlinks, copies, forwarding BUCK targets and dual source paths.
- Root `.buckconfig` must still contain no active `[buck2_re_client]` or `[oya_cache]` configuration.

**Verification**
- `buck2 audit config` for both canonical overlays, execution-platform resolution, cold-root dark-config negative test, resolver/conformance tests, affected Buck2 target resolution, full old-label/path grep, codemod reverse proof and population parity before and after promotion.

#### Lane 3C — CI cache resolver/policy behavioral closure

**Scope and placement**
- `ci/facade/build-cache-policy/` remains the sole CI runtime/policy owner; it is not moved into `build/` or `storage/`.
- `.github/workflows/cache-integrity-canary*.yml` and `.github/workflows/oya-ci-required.yml` remain bounded bridge adapters in place.
- `specs/cache-warm-license.json` and `specs/cache-warmth-policy.json` remain governed data in place.
- Do not create scheduler or worker directories.

**Ownership and move authority**
- Owner: CI cache policy/control lane, taking temporal ownership only after 3B promotion and proof.
- Start point: the exact promoted 3B head. Before any behavior edit, run the old-path/label scan; any old runtime, CI, workflow or test reference is a hard RED that fails 3B and blocks 3C.
- Consume the promoted `specs/reorg/buck2-cache-move-plan.json` only as migration evidence. Create no active or reference-only move plan in 3C.

**Behavior-only actions and stop criteria**
- Fix the daemon-startup configuration path Buck2 actually consumes using only the canonical `build/buck2/cache/` overlays.
- Prove a cold invocation cannot inherit a warm overlay or daemon.
- Refine CI cache-policy semantics and regression coverage only on canonical paths; workflow and governed-spec files remain in their existing bridge/governance homes.
- Do not repair, delete or migrate an old `toolchains/cache`, `infra/ci/buckconfig` or `toolchains//cache` runtime/CI/workflow/test reference in 3C. Its presence at 3C start is hard RED: stop, return ownership to a corrective 3B migration PR based on the promoted head, re-promote and re-prove 3B, then restart 3C from that new promoted head. Historical ADR/prose remains classified separately and is not rewritten as live authority.

**Verification**
- Pre-edit hard-RED old-reference scan, CI resolver unit/CLI/conformance tests, real pinned-Buck2 daemon visibility and cleanup, policy-semantics tests, cold-no-inheritance negative test, bridge-workflow structural regression tests, generated-face materialization without hand edits, and canonical-consumer count consistency against the promoted 3B proof.

### Lane 4 — Commission cache-only proof cell for #1534

**Dependencies:** Lanes 0–3 and #1541 complete.

**Actions**
- Rebuild the homogeneous Mac Talos cell with Cilium/Hubble.
- Reconcile generic local-path and NativeLink through GitOps.
- Issue fresh exact-purpose reader and writer identities.
- Preserve `warm_reads_licensed=false`, `remote_enabled=false`, and cold/untrusted no participation.
- Use distinct fresh ARC pods only as bridge coordinators.
- Run first writer, then separate reader at the same promoted SHA.
- Compare byte-identical outputs and structured Buck2 records.

**Proof-cell claim only:** one NativeLink replica and 50-GiB RWO local storage remain disposable. No production durability, HA, or autoscaling claim.

### Lane 5 — CAS activation and SCM/CI bridge deletion ratchets

**Separate activation PR**
- Flip `warm_reads_licensed` only after the reviewed proof receipt and Accepted authority exist.
- Bind the receipt to exact SHA, writer pod, reader pod, identities, instance, cache generation, and workflow runs.
- Preserve `remote=0`.
- Exercise license-off rollback plus cache quarantine.

**GitHub/ARC deletion criteria**
- These criteria apply only to SCM/CI bridge surfaces. They do not authorize deletion or retirement of cloud-cd/Argo.
- `.github/workflows/*` CAS/CI authority is deleted only after the exact Lane 5A sequence completes through its green rollback window and independent retirement review.
- `infra/arc/*` is deleted only after §5A-qualified owned cloud-ci launches ephemeral coordinators under the receipt-matched protected producer, parity remains exact through the rollback window, and independent review approves retirement.
- GitHub OIDC-to-OpenBao roles are deleted when owned workload identity issues coordinator credentials.
- `oya-arm64` and `oya-live-postgres-arm64` labels disappear with ARC.
- The bridge runner image and GitHub/Pwsh/Node baggage are retired with the last bridge coordinator.

### Lane 5-pre — Bounded SCM/CI-slice readiness mapped to T1–T3

Lane 5A cannot start until this prerequisite produces a valid signed readiness receipt for the **SCM/CI subset only**. It maps evidence to the existing `specs/bespoke-cloud-toolchain-services.json` sequence without claiming that this plan completes any whole cross-product stage:

1. **T1 SCM/CI slice — Bridge product contracts:** provider-neutral cloud-scm/cloud-ci APIs, tenant isolation, SLOs, quotas, audit schemas and adapter boundaries are executable and tested while GitHub remains the bridge. The cloud-cd API/adapter portion of T1 is not delivered or closed here.
2. **T2 SCM/CI slice — Rust control-plane MVPs:** owned SCM admission/status façade and owned CI controller/status producer run for `tenant=oyatie-internal` and at least one sandbox tenant with isolation and replay tests. The oya-cd release-ledger/reconciler façade remains open.
3. **T3 SCM/CI slice — Tenant private-preview evidence:** an external sandbox tenant creates a repository and pipeline, receives isolated CI execution/evidence, observes status, and exercises SCM/CI rollback without internal or cross-tenant access. Tenant release, deployment through the CD bridge and CD rollback remain open.
4. **T4 SCM/CI slice only is Lane 5A:** the T1–T3 SCM/CI-slice evidence is prerequisite evidence, never protected-admission authority. Lane 5A may replace only the GitHub Actions/branch-protection/ARC SCM/CI bridge surfaces in its inventory; it does not complete T4 because Argo/cloud-cd cutover remains open.

**Required replacement/rollback/deletion inventory**

| Bridge/runtime dependency | Owned replacement readiness | Rollback evidence | Deletion criterion |
|---|---|---|---|
| GitHub SCM adapter and git/PR repository identity | Owned provider-neutral SCM adapter preserves repository/ref/revision/work-area identity and exact candidate bytes | Adapter switch restores exact GitHub source without changing an attempt | Delete runtime adapter only after Lane 5A window; retain historical docs/ADRs |
| GitHub webhook/event source | Owned authenticated event ingestion with durable replay, dedupe, ordering and expiry | Replay from last acknowledged immutable event without duplicate success | Delete webhook secrets/routes after cutover window |
| Commit-status callback and `oya-ci-required` context | Owned protected status producer binds branch/SHA/attempt/receipt and reconciles readback | Restore bridge callback and invalidate owned outstanding statuses | Delete callback credentials only after exact parity and branch-protection update |
| Repository URLs, including `infra/gitops/bootstrap-sync.yaml` | Owned SCM URLs/revision pins resolve in GitOps and every runtime consumer | Revert URL/config atomically to exact prior repository endpoint | Remove GitHub runtime URLs only when owned fetch/reconcile/rollback proof passes |
| Branch protection | Protected rule recognizes exactly one qualified owned producer/context after §5A | Versioned rule snapshot restores bridge-only producer | Remove GitHub rule only when the repository itself no longer uses GitHub; until then update rather than delete |
| GitHub OIDC roles and claims | Owned workload identity binds tenant/job/cell/pipeline/attempt and exact audience | Disable owned issuer and restore scoped bridge issuer | Revoke/delete bridge roles after rollback window and old-identity rejection |
| GitHub workflows | Owned controller executes the canonical gate set from trusted trunk definitions | Re-enable exact pinned workflows without accepting ambiguous attempts | Delete only after §5A and rollback window |
| ARC | Owned cell-local scheduler launches ephemeral coordinators with parity | Re-enable pinned ARC scale sets within rehearsed rollback | Delete manifests/controller identities after window |
| GitHub artifacts and external runner logs | Owned tenant/cell-partitioned artifact, evidence and external log retention with retrieval/expiry parity | Read-only bridge retention remains accessible through its declared retention period | Delete producers after cutover; retain required historical evidence until expiry/retention policy permits |
| Runner labels and bridge images | Owned coordinator class/image selection is provider-neutral, digest/SBOM/provenance bound | Restore exact labels and pinned bridge image | Remove labels/images after ARC retirement and no consumer references |
| cloud-cd and Argo CD/Rollouts/Events/Workflows bridge/reference surfaces | **Retained and out of scope:** this plan delivers no oya-cd T2 MVP, T3 tenant release/deploy/rollback preview or T4 Argo cutover | Existing Argo/CD behavior remains unchanged; no rollback action is created by this plan | **No deletion in this plan.** Keep every cloud-cd/Argo surface open until a separately approved CD plan proves the remaining T1–T4 scope and its own cutover gates |

Historical ADRs, decisions, audit packets and documentation that cite GitHub/Actions/ARC are classified **retain as provenance** unless a separate retirement policy says otherwise. They are not runtime cutover blockers and must not be path-rewritten into false history.

**Signed readiness receipt**

The receipt binds `specs/bespoke-cloud-toolchain-services.json` version/digest, the explicitly enumerated T1/T2/T3 **SCM/CI-slice** evidence, exact owned SCM/event/status release and config digests, tenant-isolation fixtures, every in-scope inventory-row replacement and rollback result, the retained cloud-cd/Argo disposition, unresolved exceptions, expiry, reviewer identities and audit event. Missing in-scope row, failing rollback, stale digest, SCM/CI-slice evidence gap or expired receipt blocks Lane 5A and leaves the SCM/CI bridge sole authority. This receipt cannot assert whole-stage T1, T2 or T3 completion.

### Lane 5A — Bounded SCM/CI qualification and T4-slice admission cutover

This lane completes production qualification for the bounded SCM/CI admission slice before owned control receives any protected-admission authority and is independently rollbackable. It performs only the GitHub Actions/branch-protection/ARC portion of T4; cloud-cd/Argo T2–T4 remain open, retained and out of scope.

1. **Shadow/non-authoritative canary:** owned SCM facts and immutable global admission/routing plus the selected cell's local CI queue/leases and scheduler ingest the same source events and run bounded canaries, but publish no protected verdict. The bridge remains the sole admission authority. Compare gate selection, routing, tenant/job envelopes, results, timing and audit rows to the bridge.
2. **Complete §5A production qualification and bind the receipt:** satisfy every §5A requirement below. Emit one immutable signed receipt bound to the exact owned-control release digest, runtime/configuration digest, trusted trunk revision containing gate definitions, cell-topology version, policy-pack versions, toolchain/image set, qualification corpus and observation window. The receipt has an explicit expiry and cannot be reused after any bound input changes.
3. **Freeze, reconcile and drain:** pause new merge admission through the still-authoritative bridge, drain or explicitly cancel in-flight attempts, pin exact bridge and owned heads, and reconcile every branch, status, verdict, envelope, attempt and output. Any mismatch, incomplete drain, changed bound input, expired receipt or stale receipt cancels cutover and returns to shadow qualification.
4. **Authoritative protected-producer cutover:** switch protected admission to owned control only while the exact §5A receipt is current, valid and matches the release/configuration/trusted-trunk/cell-topology/policy versions being activated. Start no different binary, config, topology or policy under that receipt.
5. **Rollback window:** retain the bridge disabled but operationally live for the full reviewed window. Rehearse one unambiguous rollback using the exact prior bridge configuration. On availability regression, restore the bridge and requeue incomplete attempts with new fencing tokens; on security/integrity failure, freeze admission and quarantine evidence rather than downgrading trust.
6. **Retire SCM/CI bridge surfaces only:** delete the in-scope GitHub/ARC surfaces only after the rollback window completes green and an independent reviewer confirms receipt validity, production evidence, rollback readiness and no unresolved mismatch. Do not delete or claim cutover of any cloud-cd/Argo surface.

**Qualification gates**
- zero gate-set, verdict, tenant, source revision or output-digest mismatches on the preregistered corpus;
- no lost, duplicated-success or stale-fencing attempt under restart/replay tests;
- owned-control p95/p99/error/queue-age budgets pass;
- exact audit linkage from source event through admission and result;
- rollback restores admission without accepting an ambiguous attempt.

#### §5A Authoritative production-admission qualification

All requirements are conjunctive:

1. **Trusted definitions, untrusted candidate bytes:** gate definitions, policy packs, toolchain/image pins and admission code come only from a trusted promoted trunk revision. Candidate PR/work-area bytes are treated as untrusted declared inputs and cannot replace or relax their own gates.
2. **Branch/status reconciliation:** the owned producer binds repository, target branch, exact candidate revision, merge-base/trunk revision, attempt and protected status context. A status for another SHA, branch, attempt or producer is refused.
3. **Explicit verdict semantics:** every required constituent produces typed `GREEN` or `RED`; missing, skipped, cancelled, timed-out, ambiguous, stale or unreconciled results are non-green and cannot authorize merge. There is no aggregate false-green path.
4. **Numeric SLO and error budget:** over the qualifying observation window, admission/control availability is at least 99.9%, internal control request errors are at most 0.1%, queue/scheduling p95 and p99 remain within §8.1 budgets, and there are zero false-green, lost-verdict, duplicate-success or cross-SHA reconciliation events. Exhausting the monthly error budget freezes expansion and triggers rollback review.
5. **Operations:** dashboards cover admission rate/verdict, reconciliation, queue age, scheduler delay, coordinator capacity, error budget, retries, fencing refusal and rollback state. Alerts have named owners, on-call escalation, actionable runbooks and tested alert delivery.
6. **Preproduction and canary:** the exact release passes preproduction fault/recovery tests, then a bounded non-authoritative production-environment canary while the bridge remains sole authority. Breaching verdict parity, security/integrity, reconciliation or numeric SLO thresholds automatically rolls back/freezes the owned candidate without waiting for manual approval.
7. **DR restore:** encrypted durable control/admission state has a tested proof budget of RPO ≤5 minutes and RTO ≤30 minutes. Restore must preserve exact attempts/fences and must not replay an already accepted or rejected merge as new authority.
8. **Observation window:** require 30 consecutive days of sustained green under real non-authoritative/shadow plus canary traffic before full authority. A genuinely new-service exception must be independently reviewed, time-bounded, define stricter canary caps and automatic rollback, and record why 30 days cannot be observed before limited activation; it cannot waive security, reconciliation, DR or false-green requirements.
9. **Proof limitation:** the 72-hour OCI/free-tier discriminating proof validates WAN/application-boundary behavior only. It never satisfies the 30-day production gate and never authorizes protected admission.
10. **Qualification receipt:** serialize every conjunctive result into the exact signed receipt described in Lane 5A step 2. Receipt validation fails closed on expiry, missing evidence, release/config/trusted-trunk/cell-topology/policy mismatch, altered corpus, or an observation window that no longer applies.

### Lane 6 — Production CAS design and qualification

**Do not promote the proof store.**

**Actions**
- Select an REAPI-compatible object-backed CAS adapter only after durability and license review.
- Define CAS/AC replication, repair, retention, corruption detection, backup, restore, and cell partitioning.
- Preserve provider substitutability through `storage/ports`.
- Run failover and recovery without coordinator or scheduler coupling.
- Establish sustained cache telemetry before scale-out.

**Go**
- Recovery point/time objectives are proven.
- AC references never outlive CAS outputs.
- Corruption and unavailable storage fail closed.
- A cell failure cannot corrupt or stall unrelated cells.

### Lane 7 — RE design and dark implementation for #1549

**Dependencies:** #1534 closed; CAS sustained proof green; production CAS qualified; Accepted RE authority exists.

**Canonical ownership**
- `ci/ports/`: execution submission/scheduling contracts.
- `ci/core/`: owned RE scheduling policy and queue behavior.
- `ci/adapters/`: NativeLink scheduler/worker provider adapters while used.
- `build/`: crate-free Buck2 execution-platform and toolchain declarations.
- `os/`/`k8s/`: sandbox/runtime integration.
- `network/`: Cilium policies.
- `secrets/`: worker/coordinator identity projection.
- `iac/`: GitOps reconciliation mechanics.

The sandbox contract is vendor-neutral: process/filesystem isolation, identity separation, controlled mounts, bounded network, resource accounting, deterministic teardown and attestation are required properties. Kata is the first candidate, not permanent authority. A different Talos-supported runtime may replace it if the same gates pass.

**Actions**
- Remove every caller-controlled trust header.
- Implement and test the complete §5.5 per-RPC authorization matrix without operation-class exceptions.
- Validate exact URI SAN per workload.
- Prove pinned scheduler and worker configs start.
- Run the pinned worker image startup, toolchain availability, namespace/filesystem/network isolation, teardown and attestation smoke gate under the candidate sandbox.
- Place worker identity outside the action sandbox boundary.
- Deny worker control egress from executed actions.
- Keep `remote_enabled=false`.

**Stop:** no accepted sandbox, sibling certificate succeeds, PDP outage permits, or action reaches credentials/control endpoints.

### Lane 8 — RE canary, autoscaling proof, and bounded activation

**Activation sequence**
1. Single hermetic target on ARM64.
2. Bounded target set on ARM64.
3. Same target set on AMD64.
4. Local-versus-remote digest parity.
5. Controlled failure injection.
6. Small trusted build-class canary.
7. Separate reviewed activation PR.

**Concurrency benchmark**
- Benchmark effective coordinator/worker concurrency at 2 → 4 → 6.
- Stop on defined CPU steal, memory pressure, disk/IO latency, queue delay, cache latency, eviction, thermal, or gate-duration thresholds.
- Do not infer safe concurrency from the Mac’s 18 CPUs/128 GiB RAM.
- Current two-job limit is shaped by two bounded 48-GiB workspaces and Buck2’s measured over-parallelization, not a proven host ceiling.

**Scaling planes**
- Coordinators: scale from queue depth and scheduling delay; scale to zero where cold-start is acceptable.
- RE scheduler/control: retain minimum capacity; never scale to zero while authoritative.
- Workers: separate ARM64/AMD64 pools; scale from compatible queue depth, delay, and saturation.
- CAS: scale from latency, throughput, disk/IO, replication, and recovery evidence.
- Mac cell: enforce hard physical caps, per-cell quotas, backpressure, and overload shedding.
- No fake node autoscaling or CPU-only HPA.
- Track cost per successful gate and remote action.

---

## 8. Role-based Image Taxonomy

| Role | Default | Required properties | Rejected shortcuts |
|---|---|---|---|
| Current ARC bridge coordinator | Existing digest-pinned Actions runner image | Exact child-manifest verification; deletion trigger tied to ARC cutover | Treating GitHub/Pwsh/Node baggage as permanent |
| Owned coordinator | Minimal glibc image appropriate to Buck2 coordination | Non-root, digest, SBOM, provenance, signature; no worker identity | Alpine/musl as universal default |
| Rust control services | ADR-0146 distroless `static-debian12:nonroot` | Scratch only by explicit carveout | Full toolchain image |
| RE workers | Ephemeral Debian/glibc toolchain images per arch/toolchain | Rust, shell, clang/lld/native tools inside a qualified sandbox; Kata is the first candidate; immutable digest | Distroless/scratch worker, shared mutable farm, or making Kata permanent without evidence |
| Musl target worker | Separate explicit platform | Separate action keys and toolchain image | Pretending glibc and musl are interchangeable |
| Talos nodes | Talos/Image Factory immutable host image | Version and schematic digest; host-only role | Treating Talos as workload image |
| Windows/macOS | Native pools only on proven target need | Native isolation, toolchain and signing | Emulating them through Linux containers |
| Product runtime | Per product contract | ADR-0146 defaults and independent lifecycle | Reusing CI worker image |

Inventory each live image with role, owner, architecture, libc, toolchain, security context, SBOM, digest, provenance/signature, consumers, and retirement criterion. Reuse existing image/registry governance; do not create a second inventory.

### 8.1 Preregistered workload and quantitative proof budgets

The evidence packet freezes the corpus, commit, image/toolchain digests, repetitions, concurrency, metrics and thresholds before the run. Tuning after observing results requires a reviewed revision and a new watermark.

**Workload corpus**

1. small control target: `//ci/facade/build-cache-policy:...` tests and binary;
2. medium CI graph: `buck2 test //ci/...`;
3. representative native-build targets containing Rust plus clang/lld/native dependencies;
4. binding required-lane affected set at an exact promoted SHA;
5. cold, warm-reader and postmerge-writer cache classes;
6. ARM64 first, then the identical qualified action set on AMD64;
7. at least 30 measured repetitions per steady-state cell/platform point after five discarded warm-up runs; fault tests are reported separately.

**Conservative proof budgets**

| Measure | Initial pass/stop budget |
|---|---|
| CAS read p95 / p99 | ≤100 ms / ≤250 ms cell-local |
| CAS write p95 / p99 | ≤250 ms / ≤500 ms cell-local |
| CAS/AC request error rate | ≤0.1%, excluding injected faults |
| admitted queue age p95 / p99 | ≤60 s / ≤180 s; hard oldest-job age ≤300 s |
| scheduling delay p95 / p99 | ≤30 s / ≤90 s |
| tenant fairness | normalized p95 wait for any admitted tenant ≤2× fleet median; zero starvation |
| scale-out hysteresis | queue delay >30 s for three consecutive 1-minute windows |
| scale-in hysteresis | queue delay <10 s and saturation <40% for 10 minutes; 10-minute cooldown |
| worker/action error rate | ≤0.5%; zero lost or duplicate-success attempts |
| memory headroom | allocatable capacity remains ≥30% above the measured peak of the preregistered workload at the admitted concurrency; review peak and headroom monthly and after image/toolchain/workload changes |
| disk headroom | ≥25% physical free space and every workspace/CAS volume <80% used |
| thermal stop | stop on macOS serious/critical thermal pressure or sustained throttling |
| Mac concurrency | step 2→4→6; stop at the first threshold violation and make the prior step the cap |
| OCI proof budget | ≤2 OCPU and ≤12 GB RAM; exceeding it is a reviewed paid-capacity decision |
| OCI egress | ≤1 GiB per successful full gate and no mutable cache/AC synchronization |
| cost | no more than 110% of measured bridge cost per successful gate unless p95 improves ≥20% |

These are explicit proof budgets where current repository authority supplies no number. Measured evidence may justify tighter or looser values only through a reviewed update; operators may not silently tune thresholds. Existing repository values remain binding where stronger: one runner per current 48-GiB workspace, measured Buck2 `j=2` knee, RWO single-replica proof limitation, and the current Mac/Talos physical allocations.

---

## 9. Stop/Go Gates

### Failure semantics: availability versus security/integrity

| Failure class | Required behavior | Forbidden behavior |
|---|---|---|
| Availability: endpoint unavailable, capacity exhausted, lease timeout, cell temporarily offline | Bounded retry with a new attempt/fencing token, requeue to an eligible cell, or explicitly policy-authorized cold-local execution without cache/RE. Preserve no-verdict until recomputation completes. | Marking skipped work successful, reusing stale leases, or silently changing trust class. |
| Security/integrity: identity/authz failure, digest mismatch, poisoned AC, attestation mismatch, cross-tenant/cell request, replay, unexpected method | Refuse, freeze the affected admission/action, quarantine cache namespace/blob/result/identity generation as applicable, emit audit evidence, and require reviewed recovery. | Retry through a weaker identity, fall back to local or another cache, publish AC, or convert refusal to a miss. |

Tests must distinguish these classes. An injected availability failure proves bounded retry/requeue/cold-local behavior; the same harness injects security/integrity failures and proves zero fallback, zero result publication and quarantine/audit emission.

### CAS Go Gate

All must be true:

- Accepted activation authority.
- NativeLink legal and architecture approval, or approved substitute.
- #1541 closed.
- Fresh homogeneous cell with Cilium/Hubble enforcement.
- Fresh identities and empty cache generation.
- GitOps reconciliation, health, drift, and rollback green.
- Cold lane proves zero CAS connection/upload attempts.
- Writer and reader are distinct fresh ephemeral coordinators at one SHA.
- Reader records `cached > 0`, `remote = 0`.
- Output digests match cold build byte-for-byte.
- Wrong identity, no identity, sibling identity, and expired identity fail.
- Authorized and unauthorized fixtures cover every §5.5 matrix row.
- Execution/cache results bind the full tenant/job envelope and producer attestation.
- License-off rollback and cache quarantine rehearsed.

### CAS Stop Gate

Stop on zero warm hits, `remote > 0`, inherited config, unknown identity, untrusted participation, cache corruption, missing receipt, Proposed-only authority, or unavailable rollback.

### RE Go Gate

All must be true:

- CAS production qualification and sustained healthy evidence.
- Accepted RE decision.
- #1549 closed.
- Scheduler and worker separated from CAS and coordinators.
- Exact URI-SAN identity and every §5.5 matrix row proven fail closed.
- Accepted sandbox prevents key/filesystem/control-network exposure.
- ARM64 and AMD64 platform selection proven.
- Local/remote digest parity.
- Scheduler/worker failure and `remote_enabled=false` rollback rehearsed.
- Independent security, operations, and exact-head code review complete.

### RE Stop Gate

Stop on credential visibility, sibling-cert acceptance, authz fail-open, worker startup mismatch, non-hermetic dependency, digest divergence, unbounded egress, cache regression, or ambiguous action ownership.

---

## 10. Testable Acceptance Criteria

1. No implementation PR branches from the stale primary checkout.
2. Every reorg candidate records present consumer, owner, enforcement, operational value, disposition, and source deletion criterion.
3. No source/destination duplicate remains after a rehome merge.
4. Rehomes use the codemod plus one active move plan; the derived move manifest remains untracked.
5. PR #1558 is revised/split/closed and is not merged in its current shape.
6. #1541 proves old credentials rejected and the new CAS generation empty.
7. Cold/untrusted CI cannot resolve, dial, read, or write CAS.
8. Distinct fresh writer and reader report a real cache hit with `remote=0`.
9. Cold and warm outputs are byte-identical.
10. Cache license rollback disables every warm class.
11. Proof-cell documentation contains no HA, production, or durability claim.
12. Every §5.5 per-RPC matrix row passes authorized-positive and unauthorized-negative tests.
13. Executed actions cannot read worker identities or access worker-authorized endpoints.
14. Sibling certificates from the same CA are rejected.
15. ARM64/AMD64 workers use distinct platform keys and pinned child manifests.
16. Local and remote outputs match for the bounded canary target set.
17. Autoscaling uses queue, delay, saturation, disk/IO, and cache metrics with hard cell caps.
18. No test stage is accepted without confirmation it executed and was not skipped.
19. Owned CI parity includes canonical gate-set parity, failure semantics and evidence during bridge-authoritative shadow; full §5A qualification and receipt validation must then complete before protected authority, followed by the rollback window before GitHub/ARC deletion.
20. Every merged lane receives exact-head `oya-ci-required`, independent review, resolved threads, no conflict, and the post-merge completion packet.
21. The representative #1558 pilot and promoted post-merge proof record `N_pre = N_post = N_promoted > 0`, pass a differently shaped independent positive probe, and complete before fanout; #1559 is not a dependency.
22. Every queued job and action carries the complete provider-neutral envelope with expiry and fencing.
23. Every §5.5 matrix operation class has an authorized-positive and unauthorized-negative fixture, including discovery, execute, cancel, result/status and leases.
24. Every accepted remote result has a matching complete execution attestation; mismatch cannot publish AC.
25. Global control holds immutable admission/routing only; each production cell owns distinct CI queue/leases and scheduler, ephemeral coordinators, AC namespace/CAS partition, and, only when RE is activated, a separate RE queue/leases and scheduler plus per-architecture workers; each cell owns quotas/failure budget and only immutable CAS blobs replicate asynchronously.
26. Availability faults requeue/retry or use explicit cold-local policy, while security/integrity faults refuse and quarantine with zero fallback.
27. The frozen workload corpus meets every preregistered numeric budget or records a reviewed threshold revision before rerun.
28. The bounded SCM/CI control slice follows the exact Lane 5A order: bridge-authoritative shadow/canary → complete §5A SCM/CI qualification and exact bound receipt → freeze/reconcile/drain with stale-or-mismatch cancellation → protected-producer cutover only under the current matching receipt → live/rehearsed rollback window → independent review and retirement of in-scope GitHub/ARC surfaces only. Qualification precedes authority, the 72-hour/free-tier proof grants no admission authority, and cloud-cd/Argo T2–T4 remain open.
29. Stopping at owned CI plus CAS/AC is accepted unless every quantitative RE reopening trigger passes.
30. Lane 3A promotes and proves `infra/nativelink/nativelink-cas.k8s.yaml` and `infra/nativelink/OWNERS` moved to `storage/adapters/nativelink/` under `specs/reorg/nativelink-storage-move-plan.json`, with both source paths deleted and no alias before Lane 3B starts.
31. Lane 3B starts from the promoted 3A head and, as the sole temporal migration owner, atomically moves `toolchains/cache/{BUCK,defs.bzl,OWNERS}` and both `infra/ci/buckconfig/warm-cache-*.buckconfig` files into exactly `build/buck2/cache/`, updates every necessary active build/CI/workflow/test label, resolver and consumer, deletes both old source directories when empty, creates no compatibility alias, leaves root `.buckconfig` dark, and records `N_pre = N_post = N_promoted > 0` before 3C starts.
32. Lane 3C starts from the promoted and post-merge-proven 3B head, creates no move plan, changes only canonical-path daemon lifecycle, policy semantics, no-inheritance and regression behavior, and leaves workflows/specs in their current bridge/governance homes. Any old runtime/CI/workflow/test path or label at its pre-edit scan is hard RED that blocks 3C and returns migration ownership to a corrective 3B PR.
33. Lanes 3A→3B→3C have no concurrent path ownership and no consumption of an unpromoted predecessor. 3A and 3B each use one active move plan, exact source deletion, targeted tests and promoted population parity; 3C is behavior-only and proves canonical-consumer count consistency plus targeted regression behavior.
34. Lane 5-pre proves only the enumerated SCM/CI subset mapped to `specs/bespoke-cloud-toolchain-services.json` T1→T3, completes every in-scope SCM/event/status replacement/rollback/deletion row, records cloud-cd/Argo as retained and out of scope, and emits a current signed readiness receipt before Lane 5A shadow begins. It does not close whole-stage T1, T2 or T3; Lane 5A performs only the SCM/CI portion of T4 and leaves cloud-cd/Argo T2–T4 open.
35. Wrong-SHA, branch/status mismatch or stale readiness/qualification receipt yields no-verdict, freezes admission and invalidates derived statuses.
36. Cell queue/lease restore proves monotonic fencing, orphan reconciliation, zero duplicate success and §5A RPO/RTO before the cell reopens.

---

## 11. Verification Plan

### Unit and static

- Cache classification and license lattice.
- Daemon configuration install/remove/no-inheritance.
- Exact URI-SAN authorization across the full §5.5 matrix.
- Default-deny dispatch for every §5.5 operation class and unknown RPC method.
- Action-key dimensions: arch, libc, toolchain/image digest, sandbox and trust class.
- AC-to-CAS retention/reference invariants.
- Reorg move-plan bijection and source deletion for 3A/3B; 3C rejects creation of a move plan.
- #1558 population query contract: `N_pre > 0`, differently shaped positive-probe delta, `N_post = N_pre`, and promoted `N_promoted = N_pre`.
- Image inventory digest/SBOM/provenance validation.
- Autoscaling threshold and hard-cap logic.
- Tenant/job envelope completeness, expiry, retry and fencing semantics.
- Complete §5.5 per-RPC authorization matrix and negative fixtures.
- Execution-attestation completeness and mismatch rejection.
- Availability-versus-security failure classifier with no downgrade path.
- Exact `build/buck2/cache/` package inventory and prohibition of old labels, forwarding targets, aliases, copies and non-dark root configuration.
- Owned SCM/event/status readiness-receipt schema, digest/expiry/version binding, complete in-scope inventory-row coverage, explicit SCM/CI-slice claim ceiling, and retained cloud-cd/Argo disposition.
- Wrong-SHA/stale-receipt invalidation and queue/lease fencing state-machine fixtures.

### Compile and configuration

- `cargo metadata`
- Buck2 target resolution for affected roots.
- Targeted `buck2 test` for changed `ci/`, `storage/`, `build/`, governance, and infrastructure contracts.
- Pinned NativeLink config startup validation.
- YAML/JSON/schema checks.
- Generated-output diff policy.
- Confirm all expected tests ran and none silently skipped.
- Resolve `//build/buck2/cache:cache-platform` through both canonical overlays and prove no `toolchains//cache`, `toolchains/cache/` or `infra/ci/buckconfig/` runtime consumer remains.

### Integration

- GitOps sync, health, self-heal, prune, and rollback.
- OpenBao identity issuance/rotation/revocation without secret logging.
- Cilium policy and Hubble flow proof.
- Separate writer/reader coordinator pods.
- CAS restart, disk exhaustion, corruption, and quarantine.
- Scheduler/worker startup with exact pinned images.
- Malicious action filesystem and network tests.
- Queue redelivery/idempotency across control-plane restart.
- Cell isolation: another cell cannot read or mutate queue leases or AC; asynchronous immutable blob replication verifies digest and replay safety.
- #1558 pilot and post-merge population parity rerun against immutable pre, candidate and promoted SHAs; `N=0`, missed independent probe or unequal population is RED.
- Availability injection proves bounded retry/requeue/authorized cold-local; security/integrity injection proves refusal, quarantine and zero fallback.
- Owned-control sequence test: bridge-authoritative shadow/canary completes before §5A qualification; the exact receipt binds release/configuration/trusted-trunk/cell-topology/policy versions before freeze; freeze/reconcile/drain cancels on mismatch or stale receipt; cutover accepts only the current matching receipt; the live bridge rollback is rehearsed before retirement.
- Trusted-trunk gate definitions remain effective against candidate attempts to edit/replace gates; branch/SHA/status/attempt mismatch fixtures refuse.
- Typed verdict tests prove missing, skipped, cancelled, timed-out, stale and ambiguous results are non-green.
- Non-authoritative production-environment canary threshold breach automatically rolls back/freezes the owned candidate while the bridge remains sole authority and emits the expected dashboard, alert, on-call and runbook evidence.
- DR backup/restore proves RPO ≤5 minutes and RTO ≤30 minutes without duplicate acceptance or stale-fence replay.
- Sequential 3A→3B→3C promoted-head validation: 3A proves NativeLink GitOps resolution and promotion; 3B starts from that promoted head, atomically resolves the cache-platform label/config and every active build/CI/workflow/test consumer, deletes old sources and proves promoted parity; 3C starts from that promoted head, fails hard before editing if any old reference remains, then proves canonical-path daemon lifecycle, policy semantics, no-inheritance and regression behavior. No lane consumes an unpromoted predecessor and path ownership is never concurrent.
- Owned SCM webhook replay/dedupe, status callback/readback, repository URL/GitOps reconciliation, OIDC claim isolation, artifact/log retention retrieval, branch-protection rollback and signed readiness receipt for only the T1/T2/T3 SCM/CI slice; assert that cloud-cd/Argo state is unchanged and its T2–T4 work remains open.
- Wrong-SHA/stale-receipt injection freezes admission and invalidates statuses; cell queue/lease loss produces no-verdict, fences the cell, restores within bounds and accepts no duplicate result.

### E2E

- GitHub/ARC bridge: exact SHA → writer → reader → output parity → `remote=0`.
- Owned-control proof: immutable global admission/routing → cell-local CI queue/leases and scheduler → ephemeral coordinator → Buck2 → cell-local AC/CAS.
- RE: coordinator → Buck2 → CAS/AC → scheduler → correct-arch worker → digest parity.
- WAN interruption, Mac reboot, OCI restart, scheduler outage, worker loss, CAS degraded mode.
- License-off and `remote_enabled=false` rollback.
- Owned CI E2E order is bridge-authoritative shadow/canary → §5A qualification receipt → freeze/reconcile/drain → receipt-validated authoritative cutover → rollback window → independent retirement review; no test shortcut may reorder these phases.
- Thirty-day production-admission qualification receipt, or an independently reviewed time-bounded new-service exception with stricter canary and auto-rollback, before authoritative cutover—not merely before retirement.
- T1 SCM/CI bridge-contract slice → T2 SCM/CI owned-MVP slice → T3 SCM/CI sandbox repo/pipeline/CI-evidence slice and signed readiness receipt before the Lane 5A shadow/canary E2E begins; Lane 5A exercises only the GitHub/ARC SCM/CI portion of T4. The test must prove no cloud-cd/Argo resource changed and must not report whole-stage T1–T4 completion.
- Full envelope and execution attestation remain byte/identity linked from SCM event through final output.
- The preregistered corpus runs at each 2→4→6 concurrency point without hidden threshold changes.

### Observability

Collect per cell, architecture, trust class, and image:

- queue depth;
- scheduling delay;
- coordinator cold-start;
- worker saturation;
- action success/retry/loss;
- cache hit/miss/upload;
- CAS/AC latency and error rates;
- disk capacity/IOPS/queue;
- scheduler latency;
- Hubble denies and unexpected flows;
- identity issuance/expiry/rejection;
- cost per successful gate/action;
- GitHub bridge runner logs retained externally until deletion.

### Security

- Threat-model coordinator, CAS, scheduler, worker, sandbox, identity issuer, and GitOps paths.
- Hostile headers, unknown methods, PDP outage, sibling certificates, expired identities, replay, duplicate action, poisoned AC, malformed digest, secret mount discovery, and egress escape.
- SBOM, signature, provenance, vulnerability policy, digest pin and multi-arch child verification.
- Independent security review on the exact head.

### Rollback

- CAS: license false, identity revoke, endpoint remove, cache quarantine/eviction, cold local execution.
- RE: `remote_enabled=false`, drain workers, stop scheduler, preserve CAS, cold/local execution.
- Reorg: codemod reverse proof and Git revert; never restore copied dual authority.
- Bridge: ARC remains until owned parity; no flag-day removal.
- Topology: fall back to Mac-local proof cell if OCI application-plane proof fails.

Verification ladder for every lane:

```text
static/schema
  -> compile/target resolution
  -> component smoke
  -> local proof
  -> all-platform or live proof
  -> exact-head required CI
  -> post-merge verification
```

---

## 12. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Proposed ADR treated as authority | Status matrix plus explicit Accepted activation gate. |
| Reorg expands scope | One low-coupling trial, one active move plan, deletion-first review. |
| Proof infrastructure becomes production by inertia | Proof/production diagrams, separate durability lane, explicit claim language. |
| Credentials leak into actions | Sandbox-host identity separation and malicious-action tests. |
| Cache poisoning | Writer isolation, read-only AC endpoint, fail-closed authorization, integrity canary. |
| Scheduler outage blocks cache | Separate deployments, identities, rollout and SLOs. |
| WAN split destabilizes Kubernetes | Reject stretched control plane; split at application protocol. |
| OCI “free” assumption drives design | Size to current 2-OCPU/12-GB allowance and measure actual cost/capacity. |
| Mac overload causes false CI instability | Physical caps, stepped 2→4→6 benchmark, backpressure and I/O thresholds. |
| Image sprawl | Role-based inventory; build only present ARM64/AMD64 glibc worker profiles first. |
| GitHub/ARC bridge persists | Per-artifact deletion triggers and owned parity gate. |
| Tests exist but do not run | Gate on execution evidence, not test-file presence. |

---

## 13. Deliberate Pre-mortem

### Scenario 1 — “Warm” CI is green but never used the cache

**Cause:** Buck2 daemon started before the overlay, inherited stale config, or report parsing observed the wrong process.

**Early signal:** zero-hit warm run, missing counters, no NativeLink traffic, or the same daemon ID across cold/warm phases.

**Mitigation:** fresh isolation directory/daemon, configuration visibility test against pinned Buck2, network evidence, hard failure on zero-hit warm proof.

### Scenario 2 — First RE action steals the worker identity

**Cause:** Kata/runtime boundary mounts the SVID or leaves worker-authorized sockets and egress visible.

**Early signal:** malicious canary reads identity paths or reaches scheduler/PDP/OpenBao endpoints.

**Mitigation:** identity terminates outside the guest/action environment, explicit mount and network deny tests, fail closed before any RE activation.

### Scenario 3 — Reorganization creates a second live authority

**Cause:** manifests/configs are copied to canonical roots while workflows/GitOps still consume `infra/*`.

**Early signal:** both paths appear in `git grep`, ownership differs, or reverse codemod is not byte-stable.

**Mitigation:** one move plan, source deletion in the same PR, reference scan and reverse proof as merge blockers.

### Scenario 4 — OCI control-plane move worsens availability

**Cause:** Kubernetes control-plane traffic is stretched over WAN while workers, CAS, OpenBao, and data remain on the Mac.

**Early signal:** node NotReady churn, delayed reconciliation, certificate/routing failures, CI still unavailable when Mac sleeps.

**Mitigation:** reject the topology; test only the application-protocol split with outbound mTLS and durable queue semantics.

### Scenario 5 — Scaling runners makes CI slower

**Cause:** workspace disk, CAS latency, Buck2 over-parallelization, or two worker VMs saturate before CPU count.

**Early signal:** scheduling delay falls while action latency, I/O queue, eviction, or failure rises.

**Mitigation:** stepped 2→4→6 benchmark, hard stop thresholds, per-cell quotas, scale-out only after measured improvement in cost and wall time.

### Scenario 6 — Availability fallback launders an integrity failure

**Cause:** a digest, identity or attestation refusal is treated as an ordinary cache miss and recomputed through a less-trusted local path.

**Early signal:** the same audit event changes failure class, a quarantined key is requested again, or an AC result appears after a refusal.

**Mitigation:** typed failure classes, refusal/quarantine state, zero-fallback negative fixtures, new reviewed recovery attempt rather than automatic retry.

### Scenario 7 — Wrong-SHA or stale-readiness receipt produces a false green

**Cause:** protected status is reconciled to a different branch/SHA/attempt, or Lane 5A activates a release/config/topology/policy set that no longer matches its signed readiness/qualification receipts.

**Early signal:** receipt digest mismatch, branch-protection readback naming another producer/context, status head differing from the candidate envelope, expired receipt, changed trusted-trunk gate revision, or a GREEN with no exact audit join.

**Mitigation:** freeze admission immediately; treat the result as no-verdict, invalidate the receipt and every derived status, retain candidate evidence, restore bridge authority if still inside the rollback contract, and restart from shadow with new attempts/fencing tokens. Never relabel the stale result or copy it to the new SHA.

### Scenario 8 — Cell queue/lease or DR loss creates orphaned or duplicate work

**Cause:** cell-local queue/lease state is lost or restored behind durable admission/result state, allowing an orphaned action, duplicate worker lease or replayed success.

**Early signal:** queue intent without a live lease, two workers holding one action fence, result without current attempt, oldest-job age discontinuity, restored lease generation below audit watermark, or duplicate-success audit events.

**Mitigation:** publish no verdict; fence the cell, stop new leases, recover from the last durable queue/audit watermark, restore within §5A RPO ≤5 minutes/RTO ≤30 minutes, mint new monotonic fencing tokens, reconcile every orphan as cancelled/requeued, and require zero duplicate acceptance before reopening the cell.

---

## 14. ADR

### Decision

Adopt a cache-first, capability-owned, shared-nothing build-fabric progression.

Reorganize the active CAS/build/CI surfaces through three sequentially owned promoted PRs, each starting from its predecessor's promoted head with no concurrent path ownership. Move the NativeLink manifest and ownership to `storage/adapters/nativelink/`; then make 3B the sole temporal migration owner that atomically moves `toolchains/cache/{BUCK,defs.bzl,OWNERS}` and both `infra/ci/buckconfig/warm-cache-*.buckconfig` overlays into exactly `build/buck2/cache/`, rewrites every necessary active build/CI/workflow/test consumer, deletes the old sources, creates no alias or dual home, and keeps root `.buckconfig` dark. Lane 3C is behavior-only on canonical paths; an old reference at its start is hard RED against 3B.

Use the homogeneous Mac Talos cell for disposable CAS and RE proof. Do not stretch its Kubernetes control plane to OCI. If OCI is used, split at the owned CI/RE application protocol: global services hold immutable admission/routing only, while every mutable queue, lease, scheduler, coordinator, AC, CAS partition and worker remains inside one homogeneous execution cell whose agents establish outbound mTLS.

GitHub Actions and ARC remain the sole protected-admission bridge while owned SCM/admission and cloud-ci run shadow/canary. Before Lane 5A begins, the owned SCM/event/status lane must prove the bounded SCM/CI subset mapped to `specs/bespoke-cloud-toolchain-services.json` T1→T3 and issue a current signed readiness receipt covering the in-scope SCM, event replay, status production/readback, repository URLs, branch protection, OIDC, workflows, ARC, retention, labels and images. Owned SCM/CI control receives authority only after full §5A qualification of that bounded slice yields a separate current exact cutover receipt, freeze/reconciliation/drain finds no mismatch, and the activated release/configuration/trusted-trunk/cell-topology/policy versions match that receipt. The disabled SCM/CI bridge remains live through the rehearsed rollback window and is retired only after independent review. Cloud-cd/Argo T2–T4 remain open, retained and out of scope; this decision neither modifies nor deletes them and claims no whole-stage T1–T4 completion. CAS/AC, CI scheduling, RE scheduling, and worker execution remain independently deployable and independently scalable.

Production execution is shared-nothing by cell: distinct cell-local CI queue/leases and scheduler feed ephemeral coordinators; each cell has its own AC namespace and CAS partition/provider adapter; when RE is activated, a separate cell-local RE queue/leases and scheduler feed per-architecture hermetic workers. Each cell owns quotas and failure budget. Global control retains immutable admission/routing only, and only immutable content-verified CAS blobs replicate asynchronously. A complete provider-neutral tenant/job envelope and producer execution attestation bind every transition and result.

Owned CI plus CAS/AC is an accepted terminal architecture. RE begins only after every preregistered reopening trigger demonstrates that cache-only local execution remains materially latency- or cost-bound.

### Drivers

1. Arbitrary-code execution and cache poisoning risk.
2. Availability and reproducibility of the merge-authoritative path.
3. Accepted capability-first topology and deletion-first migration.

### Alternatives Considered

- Mac-only permanent fabric: simplest, but cannot meet availability goals.
- OCI Kubernetes control plane with Mac workers/data: rejected due WAN coupling and no meaningful workload availability gain.
- One-shot CAS+RE rollout: rejected because it erases causal evidence and broadens rollback.
- Leaving `toolchains/cache/` in place or moving only the two overlays: rejected because it preserves split ownership contrary to Accepted ADR-0562 and permits label/resolver drift.
- Compatibility aliases or a dual-source cache package: rejected because they defeat source deletion, population parity and one-authority rollback.
- Absorbing cloud-cd/Argo delivery into the SCM/CI admission cutover: rejected because it would falsely close cross-product T1–T4 stages and expand this plan beyond its evidence and ownership.
- Managed remote build service: preserves REAPI possibility but conflicts with current owned-stack direction and requires separate cost, trust, residency, and dependency approval.
- Application-protocol split with independent execution cells: chosen staged target.
- Owned CI plus CAS-only: accepted stopping option unless the quantitative RE reopening gate passes.

### Why Chosen

It proves the highest-risk claims in the smallest reversible order, keeps proof shortcuts visibly non-production, aligns source ownership with Accepted ADR-0562/0615, makes the complete active Buck2 cache package one atomic build-owned unit, and creates explicit deletion triggers for every bridge.

### Consequences

- RE delivery waits for CAS evidence, Accepted authority, and sandbox proof.
- Initial throughput remains bounded.
- GitHub/ARC bridge code persists temporarily but gains mandatory deletion criteria.
- Lane 3 requires three sequential promotions from predecessor promoted heads; 3A/3B require post-merge population parity, while 3C is behavior-only. This trades short-term throughput for non-concurrent ownership and deterministic rollback.
- Lane 5A is blocked until the T1→T3 SCM/CI-slice readiness receipt is current, complete for its bounded inventory and independently signed; this does not advance or close cloud-cd/Argo T2–T4.
- Production qualification delays authoritative cutover: a shadow result, 72-hour topology proof, stale receipt or qualification for different release/configuration/topology/policy can never authorize protected admission.
- Production CAS selection remains open behind REAPI ports.
- Image and worker pools multiply only when target demand proves need.
- OCI may host limited control services, but a free-tier instance is not HA.

### Follow-ups

- Close #1559, revise/split #1558, promote Lane 3A then 3B then 3C with post-merge proofs, close #1541, execute #1534, then address #1549.
- Run the OCI application-plane discriminating proof.
- Qualify production CAS durability.
- Promote or replace Proposed CAS/RE decisions before activation.
- Complete and sign the SCM/event/status replacement/rollback/deletion readiness inventory mapped only to the T1→T3 SCM/CI slice before Lane 5A shadow begins; record cloud-cd/Argo as retained and open.
- Complete bridge-authoritative shadow parity, §5A qualification and exact receipt, freeze/reconcile/drain, receipt-validated protected cutover, live rollback window and independent review before deleting only the in-scope GitHub/ARC SCM/CI surfaces.
- Hand the remaining cloud-cd/Argo T1–T4 work to a separate approved CD plan; do not absorb it into this program.
- Add independent homogeneous execution cells only after measured capacity and availability need.

---

## 15. Current PR and Issue Sequence

1. **Boundary map and independent review** — no rehome PR until source, target, owner, consumers and dependencies are approved.
2. **PR #1558 representative trial** — keep draft; revise into one low-coupling storage prerequisite or close/supersede. Delete unsupported QEMU duplication, split ownership, and rehome only live local-path behavior.
3. **#1558 post-merge proof** — rerun the population query against promoted `dev`; require `N_promoted = N_pre = N_post > 0`, rerun the differently shaped independent positive probe, and verify source deletion, consumer resolution, derived manifest, reverse proof and tests actually executed before any fanout.
4. **Lane 3A NativeLink rehome** — one storage-owned worktree/PR starts from its required promoted predecessor, moves the manifest and OWNERS, deletes the source, and proves promoted population parity.
5. **Lane 3B complete build migration** — only after 3A promotion/proof, one migration-owned worktree/PR starts from the exact promoted 3A head, atomically moves the complete cache package and both overlays to `build/buck2/cache/`, rewrites every necessary active build/CI/workflow/test consumer, deletes old sources and proves promoted population parity with no alias or dual home.
6. **Lane 3C CI behavior closure** — only after 3B promotion/proof, one CI-owned worktree/PR starts from the exact promoted 3B head and changes canonical-path daemon lifecycle, policy semantics and no-inheritance/regression behavior only. Any old reference at pre-edit is hard RED against 3B; 3C does not repair it or create a move plan.
7. **Issue #1541** — non-mutating preparation may run parallel with steps 1–6; credential rotation/rebuild/purge must close before any live CAS proof.
8. **Issue #1534** — execute cache-only wiring, proof-cell commissioning, two-runner receipt, separate activation PR, and accept CAS-only as a possible terminal state.
9. **Lane 5-pre bounded SCM/CI readiness** — prove only the SCM/CI subset mapped to T1→T3, close every in-scope replacement/rollback/deletion inventory row, record cloud-cd/Argo as retained/open, and issue an independently signed current readiness receipt; no Lane 5A shadow starts without it.
10. **Bounded SCM/CI qualification and cutover (T4 slice)** — bridge-authoritative shadow/canary → full §5A qualification of the SCM/CI slice and exact bound cutover receipt → freeze/reconcile/drain with cancel-on-mismatch/staleness → receipt-validated protected-producer cutover → live/rehearsed rollback window → independent retirement review of in-scope GitHub/ARC only. Whole T4 and cloud-cd/Argo remain open.
11. **Issue #1549** — redesign and prove RE sandbox only if all RE reopening triggers pass after #1534 and production CAS gates.
12. **PR #1559** — off critical path metadata correctness; merge whenever independently reviewed and green. It does not activate ADR-0631.

Repository-only work on #1558 may proceed while #1541 operational closure runs, but no proof-cell deployment or credential issuance may start until both are complete.

### Execution preflight — must be rerun immediately before handoff

Requery, never rely on this snapshot:

```text
gh pr view 1558 --repo jason931225/oyatie \
  --json state,isDraft,headRefOid,mergeable,reviewDecision,statusCheckRollup,files
```

Planning-time snapshot on 2026-08-05:

- PR #1558 remains OPEN and draft at `54b22d0c6470d8008012542eb37d0ff32b72e1b5`.
- It still contains the three original paths.
- `buck2 (hermetic build + affected gate tests)` is **FAILURE**.
- `gate · affected-set (ADR-0554, binding workspace coverage)` is **FAILURE**.
- Other checks were queued/in progress; there is no complete exact-head green verdict or review decision.

Therefore #1558 is currently a hard **STOP**. Execution must capture the fresh failing logs, diagnose them without assuming they are unrelated, revise/split the PR as planned, and obtain a new complete exact-head `oya-ci-required` result before trial merge or post-merge proof.

---

## 16. Staffing and Handoff

### Available Agent Types

`explore`, `researcher`, `dependency-expert`, `planner`, `architect`, `critic`, `scholastic`, `executor`, `test-engineer`, `debugger`, `designer`, `code-reviewer`, `verifier`, `git-master`, `writer`, `code-simplifier`, `team-executor`.

### Recommended Staffing

| Lane | Role | Reasoning |
|---|---|---|
| Authority/boundary audit | architect + scholastic | high/xhigh |
| FSL and NativeLink qualification | dependency-expert + researcher | high |
| Reorg trial | executor + git-master | medium/high |
| Lane 3A NativeLink move | executor + verifier | medium/high |
| Lane 3B atomic migration across build/CI/workflow/test consumers | executor + git-master + verifier | medium/high/high |
| Lane 3C canonical-path behavior closure | executor + test-engineer + verifier | medium/medium/high |
| Credential closure | executor + code-reviewer | medium/high |
| CAS wiring | executor | medium |
| Sandbox/identity | architect + executor | xhigh/medium |
| Test design | test-engineer | medium |
| Exact-head completion | verifier + code-reviewer | high |
| Final simplification | code-simplifier | high |
| Envelope/cell contract | architect + security-focused code-reviewer | xhigh/high |
| Bounded SCM/CI T4-slice cutover | architect + executor + verifier | xhigh/medium/high |
| SCM/event/status T1→T3 slice readiness | architect + executor + test-engineer + verifier | xhigh/medium/medium/high |
| cloud-cd/Argo open-scope handoff (no implementation) | planner + architect | medium/xhigh |
| Cell queue/lease recovery and DR | architect + debugger + verifier | xhigh/high/high |
| Quantitative corpus/autoscaling | performance-goal evaluator + test-engineer | high/medium |

### Recommended Follow-up

Default:

```text
$ultragoal execute .omx/plans/cas-re-hyperscaler-capability-reorg-20260805.md
```

Use Ultragoal as the durable dependency ledger. Use Team only within a stage where active path ownership does not overlap concurrently and every consumed predecessor is already promoted, for example:

```text
$team 4 "Execute the approved CAS authority, PR #1558 disposition,
credential-prevention, and verification lanes with exclusive worktrees;
do not cross the CAS activation gate."
```

Suggested parallel Team lanes after approval:

1. authority/legal evidence;
2. #1558 low-coupling reorg trial;
3. #1541 preventive repository checks;
4. independent test/verification preparation.

Operational rotation, proof-cell mutation, CAS activation, and RE activation remain sequential.

### Team Verification Path

- Each worker owns one isolated worktree and no path has concurrent owners; sequential ownership transfer is allowed only after predecessor promotion and proof.
- Integrator checks source deletion and cross-lane references.
- Verifier reruns the full verification ladder on exact PR heads.
- Code reviewer inspects security-sensitive manifests/configuration manually.
- Merge only with resolved threads, no conflicts, exact `oya-ci-required` green, and branch protection satisfied.
- Post-merge verification records rollout, rollback, observability, user-story evidence, release impact, and observation harvest.

### Ralph Fallback

Use `$ralph` only if a single approved lane needs persistent fix/verify pressure, such as the Buck2 daemon-config regression. Do not use Ralph for the cross-stage CAS→RE program or as a substitute for Ultragoal’s dependency ledger.

---

## 17. Goal-Mode Follow-up Suggestions

- **`$ultragoal` — default:** convert this approved dependency-ordered plan into durable repo-native goals and preserve the CAS-before-RE gates.
- **`$ultragoal` + `$team` — parallel delivery:** keep Ultragoal as the leader-owned ledger while Team executes only dependency-independent lanes with isolated worktrees, non-overlapping concurrent path ownership, and verifier-ready evidence.
- **`$performance-goal` — measured tuning subprogram:** use for the 2→4→6 concurrency benchmark, cache/queue latency tuning, cold-start versus warm-pool evaluation, capacity caps, and cost-per-successful-gate/action optimization.
- **`$autoresearch-goal` — unresolved research only:** use only if NativeLink licensing, production object-backed CAS selection, or an accepted Talos sandbox remains an evidence question after bounded primary-source review.
- **`$ralph` — explicit narrow fallback only:** use for one approved single-owner fix/verify loop when durable multi-goal tracking and parallel execution are unnecessary.

---

## 18. Consensus Review Changelog

- Added CAS-only as a valid terminal architecture and made RE conditional on preregistered demand, hermeticity, security, and unit-cost evidence.
- Made the system tenant-aware and cell-local: immutable global routing only; separate per-cell CI and RE queues, leases, schedulers, AC/CAS partitions, quotas, identities, and failure budgets.
- Replaced mutation-only security language with one all-operation authorization matrix and separated availability fallback from integrity/security refusal and quarantine.
- Added a receipt-gated shadow -> qualification -> freeze -> cutover -> rollback-window -> retirement state machine for the bounded SCM/CI bridge slice.
- Made reorganization non-vacuous through map -> representative pilot -> promoted proof -> fan-out, positive population parity, atomic path migration, source deletion, and promoted-head temporal ownership.
- Split NativeLink, Buck2 cache-package, and CI behavior work into executable 3A/3B/3C lanes; chose exact `build/buck2/cache/` ownership without aliases or dual homes.
- Added the owned SCM/event/status readiness receipt and complete in-scope GitHub runtime replacement inventory while retaining cloud-cd/Argo as explicitly open, separate scope.
- Added quantitative autoscaling, fairness, hysteresis, thermal, storage, headroom, cost, and failure-shedding budgets plus wrong-SHA and queue/lease-loss pre-mortems.
- Final Architect verdict: **APPROVE** (`.omx/plans/cas-re-hyperscaler-architect-review-final-approved-20260805.md`).
- Final Critic verdict: **APPROVE** (`.omx/plans/cas-re-hyperscaler-critic-review-approved-20260805.md`).
