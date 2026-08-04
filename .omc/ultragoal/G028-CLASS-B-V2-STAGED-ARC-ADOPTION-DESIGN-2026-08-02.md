# G028 class B v2 — permanent-lab ARC-only staged GitOps adoption — 2026-08-02

State: **DRAFT REVISION — FACT COLLECTION IN PROGRESS — NOT REVIEWED — NOT IMPLEMENTABLE — NO LIVE MUTATION**  
Source baseline: `origin/dev` `0c1014b87f0d881a821faa6a872b309deba0cfbf`  
Supersedes for future review only: rejected v1 `G028-CLASS-B-PERMANENT-LAB-GITOPS-DESIGN-2026-08-02.md`  
Binding review: `G028-CLASS-B-INDEPENDENT-REVIEW-2026-08-02.md`

## Fixed decision

The 2026-07-29 founder ruling fixes `KEEP_CURRENT_LAB=true → class=B`. It authorizes class selection only. It does not authorize implementation, credentials, or cluster mutation.

## Unresolved approval inputs

```text
bootstrap_principal=UNRESOLVED
rollback_owner=UNRESOLVED
target_cluster_context=OBSERVED_NOT_ADMITTED: admin@oya-talos
target_api_endpoint=OBSERVED_NOT_ADMITTED: https://10.5.0.1:6443
target_api_certificate_sha256=OBSERVED_NOT_ADMITTED: F7:6E:65:AE:02:F5:F4:B9:73:86:2C:20:B8:14:FE:D5:01:34:7B:EF:B4:ED:FB:39:5F:71:C7:00:B4:AA:20:A1
credential_source_lifetime_revocation=UNRESOLVED
audit_sink=UNRESOLVED
buck2_target=PROPOSED: root//ci/facade/gitops-bootstrap-contract:ci-gitops-bootstrap-contract-gate
rust_gate_owner=PROPOSED: ci/facade/gitops-bootstrap-contract following existing protected-fleet contract
workflow_job_and_final_fanin=FACT: existing buck2 job executes root//ci/... and oya-ci-required checks needs.buck2.result
affected_roots=SPECIFIED_BELOW
argo_pin_authority_and_digests=ABSENT_REQUIRES_VENDORED_PIN_AUTHORITY
arc_chart_versions=FACT: controller+scale-set OCI charts both 0.14.2
arc_chart_content_digests=UNRESOLVED
arc_application_namespaces=FACT: argocd/{arc,oya-arm64} → {arc-systems,arc-runners}
arc_explicit_helm_release_names=FACT_REQUIRED_CHANGE: controller=arc; scale_set=oya-arm64
live_arc_controller_release_inventory=FACT: arc/arc-systems revision=1 chart=gha-runner-scale-set-controller-0.14.2 app=0.14.2
live_arc_scale_set_release_inventory=FACT: oya-arm64/arc-runners revision=12 chart=gha-runner-scale-set-0.14.2 app=0.14.2
live_arc_scale_set_safe_values=FACT: min=0 max=3 request_cpu=2 request_memory=4Gi request_ephemeral_storage=20Gi limits_memory=8Gi limits_ephemeral_storage=60Gi
live_arc_workload_at_observation=FACT: ARS+ERS Running; current/pending/running=0/0/0; no EphemeralRunner or runner pod
live_arc_ownership_drift=UNRESOLVED: three oya-arm64-gha-rs-kube-mode SA/Role/RoleBinding are HELM_LIVE_NOT_RENDERED
secret_reference=FACT: arc-runners/oyatie-arc-app external Opaque GitHub-App Secret
secret_metadata=FACT_SAFE_ONLY: owner_refs=[] manager=kubectl-create key_names={github_app_id,github_app_installation_id,github_app_private_key}; no value bytes captured
secret_restoration_contract=UNRESOLVED: durable provisioner/source/restoration actor/lifetime/revocation authority absent
packet_digest=COMPUTE_ONLY_AFTER_V2_FREEZE
```

No placeholder may be interpreted as an implementation choice.

## Minimal transition boundary

V2 adopts **only** the existing ARC controller and `oya-arm64` runner scale set. It does not submit the existing global root Application and does not instantiate unrelated platform Applications.

A separately reviewed future expansion may reuse the app-of-apps topology after each added workload has ownership, secret, stateful-data, health, and rollback proof. That expansion is not G028.

## Staged immutable protocol

Every phase is a distinct deterministic Buck2-produced artifact with a recorded digest. The execution surface accepts only the admitted digests and exact target identity.

### Phase A1 — CRD establishment

Artifact contents:

- `argocd` Namespace;
- only content-addressed/vendored Argo CRDs selected by the single pin authority;
- no controllers;
- no `Application` custom resources;
- no ARC resources.

Fail closed until every required CRD reports `Established=True` and its observed version/schema digest matches the admitted input. Timeout or mismatch aborts without Phase A2.

### Phase A2 — controller establishment

Artifact contents:

- only pinned Argo controller/RBAC/service resources;
- no `Application` custom resources;
- no ARC resources.

Fail closed until every required deployment reports `Available` and the observed controller image/version equals the admitted pin. Timeout or mismatch aborts without adoption.

### Phase B0 — immutable pre-adoption proof

Capture an immutable metadata-only inventory for both live Helm releases:

```text
release_name
release_namespace
chart_name + chart_version + application_version + chart_digest
values_digest
(apiVersion, kind, namespace, name) for every rendered/live object
immutable selectors
service-account names
Helm ownership labels/annotations
proposed Argo tracking identity
```

Produce a deterministic diff against the admitted ARC-only Argo render. Fail closed on any extra, missing, renamed, shared, or immutable-field-different object unless an exact accepted delta is enumerated in a reviewed packet.

Secret preflight records no credential bytes. It must bind secret namespace/name/type, expected key-name set, metadata-only fingerprint, restoration source/actor, and proof that neither desired Application manages or prunes the Secret.

### Phase B1 — inert ARC-only Applications

Submit exactly two `Application` resources:

- ARC controller;
- `oya-arm64` runner scale set.

Both must:

- target the admitted immutable source SHA, not mutable `dev`;
- use explicit `helm.releaseName` matching the accepted live inventory;
- have automated sync disabled;
- have prune disabled;
- use only content-addressed/vendored chart inputs and the single admitted ARC values source;
- not own or prune credential Secrets.

Before submission, resolve protected `dev` and require it to equal the admitted SHA. If it advances, abort and readmit rather than adopting newer bytes.

### Phase B2 — controlled first synchronization

The digest-gated bootstrap operation synchronizes only the two inert ARC Applications. Acceptance requires:

- Argo observed revision equals the admitted SHA;
- deterministic live resource identity diff satisfies the accepted inventory;
- controller/listener health;
- AutoscalingRunnerSet request `22Gi`;
- EphemeralRunnerSet request `22Gi`;
- a newly created runner pod requests `22Gi`;
- runner registration and one representative job complete;
- no node DiskPressure and no unproven OOM/eviction claim.

Failure triggers the admitted rollback contract; it never weakens the request to 20Gi.

### Phase C — separately admitted steady-state transition

Only after B2 acceptance may a separate protected declaration:

1. transition source tracking from the admitted SHA to protected `dev`;
2. enable automated sync;
3. later enable prune in another separately admitted step after ownership identity remains proven.

No broad app-of-apps expansion is implied.

## Content-addressed supply chain contract

Repository fact collection proves no content-addressed Argo source exists today: `infra/capi/crs/render.sh:13,18-20,37-53` uses chart `9.5.15` through ambient Helm repositories and remote metadata; `infra/capi/clusters/values.yaml:17-26` names three raw GitHub v3.4.2 CRD URLs; no lock, vendored chart/CRD bytes, or digest authority exists under `infra/capi` or `infra/gitops`. Therefore this is an implementation prerequisite, not a reusable current surface.

The implementation design must add the minimum single machine-readable pin authority deriving:

- Argo chart version, application version, CRD version, source location, and digest;
- ARC controller and runner-scale-set chart versions, source locations, and digests;
- controller image digests where the chart permits immutable image identity.

All source bytes must be repository-vendored or materialized by an existing hermetic Buck2 mechanism with verified digest. Render and test forbid network access, `helm repo add/update`, mutable tags, and raw unverified URLs.

Exact vendored paths/digests remain unresolved; v2 will not invent them. Repository source facts are recorded in `G028-BOOTSTRAP-REPOSITORY-FACTS-2026-08-02.md`, ARC declaration facts in `G028-ARC-ADOPTION-REPOSITORY-FACTS-2026-08-02.md`, and the read-only safe live projection in `G028-LIVE-ARC-ADOPTION-INVENTORY-2026-08-03.md`.

## Frozen live inventory facts and remaining disposition gap

The read-only observation window `2026-08-03T00:41:35Z..00:46:44Z` established the observed target coordinates and actual release names without reading Secret values or mutating the cluster. The coordinates remain `OBSERVED_NOT_ADMITTED` until accountable authority binds them to the frozen packet:

- target `admin@oya-talos`, API `https://10.5.0.1:6443`, public certificate SHA-256 `F7:6E:65:AE:02:F5:F4:B9:73:86:2C:20:B8:14:FE:D5:01:34:7B:EF:B4:ED:FB:39:5F:71:C7:00:B4:AA:20:A1`;
- controller release `arc/arc-systems`, revision 1, chart/application `0.14.2`;
- scale-set release `oya-arm64/arc-runners`, revision 12, chart/application `0.14.2`;
- ARS and ERS Running but request still `20Gi`; no runner object/pod during the point-in-time quiescence observation;
- external Secret `arc-runners/oyatie-arc-app`, metadata/key names only, no owner references, not chart-rendered;
- three Helm-owned `oya-arm64-gha-rs-kube-mode` SA/Role/RoleBinding objects absent from the current render.

The three absent-from-render objects remain a blocking owner decision. Each must be classified as exactly `KEEP_EXTERNAL`, `ADOPT_WITH_ENUMERATED_DELTA`, or `RETIRE_WITH_PROOF` before packet freeze; no default and no deletion are inferred. Secret provisioning/restoration ownership and all named execution principals also remain unresolved. The point-in-time empty workload must be rechecked immediately before any separately authorized transition.

## Protected admission contract

The protected topology will reuse the existing `ci/facade` fleet contract rather than create a parallel lane:

```text
proposed target: root//ci/facade/gitops-bootstrap-contract:ci-gitops-bootstrap-contract-gate
crate shape: existing ci/facade library + unittest + gate pattern
execution: .github/workflows/oya-ci-required.yml Buck2 job recursively tests root//ci/...
final fan-in: oya-ci-required requires needs.buck2.result == success
```

Anchors: `.github/workflows/oya-ci-required.yml:187-197,578-586,1205-1220,1241-1253`; `ci/facade/baseline-ratchet/tests/gate_registration.rs:685-754`. No `infra/BUCK` exists, so placing an unregistered check under `infra/` would be a dark gate.

Required affected-set ownership roots for the proposed gate:

```text
infra/gitops/**
infra/arc/**
<new vendored Argo+ARC pin/input paths>
<new hermetic renderer/executor-contract paths>
<new inventory+rollback contract paths>
.github/workflows/oya-ci-required.yml
ci/facade/gitops-bootstrap-contract/**
```

The gate must use existing affected-set registration mechanics so every listed source change executes the gate; its born-blocking meta-test must fail if the gate target, recursive Buck2 execution, or final checked fan-in disappears. Local render success alone is never admission.

## Identity and rollback contract

Before v2 can receive design approval, accountable authority must name:

- exact human or workload bootstrap identity;
- exact target cluster context and API fingerprint;
- minimum namespace/cluster RBAC for admitted objects only;
- credential source, lifetime, revocation rule;
- immutable audit destination;
- distinct rollback owner and escalation path.

Rollback must be executable and content-addressed:

1. Disable Argo automation without deleting ARC resources.
2. Preserve or deliberately restore accepted ownership metadata.
3. Use exact last-known-good chart artifacts, values digest, and resource inventory.
4. Restore secret only through the named durable secret source/actor; never record credential bytes.
5. Prove listener and runner registration health after rollback.
6. Record phase digests, target fingerprint, before/after inventory, and outcome.

## Packet-byte review identity

After all unresolved facts are filled and before independent re-review, freeze these packet bytes and compute a digest set covering:

- this v2 packet;
- the binding independent review;
- the class-B selection packet;
- source baseline SHA.

The review attestation must bind digest set, baseline SHA, reviewer identity, verdict, and timestamp. Any byte change invalidates the verdict and requires re-review. A protected committed blob/tree identity is preferred when implementation begins.

## Required RED/GREEN proof

### RED

- `Application` appears before required CRDs are Established → abort.
- Controller phase starts before CRD establishment or adoption starts before controller availability → abort.
- Initial scope renders any Application besides ARC controller and `oya-arm64` → fail.
- Automated sync or prune enabled during adoption → fail.
- Initial revision is mutable or `dev != admitted SHA` → abort.
- Missing/extra/renamed resource, selector mismatch, release-name mismatch, secret ownership, or unmanaged accepted delta → fail.
- Input lacks verified content digest or render accesses network → fail.
- Protected job/fan-in/affected coverage removed → fail admission.
- Packet digest differs from reviewed attestation → verdict invalid.

### GREEN

- Clean Buck2 render is byte-identical twice without network.
- A1/A2/B1 artifacts contain only their permitted resource classes.
- Live inventory equals admitted ARC-only render modulo enumerated accepted deltas.
- Secret metadata preflight passes without exposing bytes and desired objects do not own/prune it.
- Controlled sync observes exact admitted SHA and live 22Gi across ARS/ERS/new pod with healthy registration.
- Rollback dry contract resolves every artifact, identity, and step without mutable lookup.

## Train boundary

```text
fill exact repository + live inventory + identity facts
→ freeze v2 packet digest
→ real independent APPROVE on exact digest
→ implementation worktree from fresh origin/dev
→ exact-head code APPROVE + protected PR green
→ separately authorized digest-gated staged bootstrap
→ live 22Gi acceptance
→ #1526 cold FULL
→ #1523 restack/admission
→ G023/W0-C/D only afterward
```

## Non-actions

- No implementation from this draft.
- No live cluster mutation, direct Helm, CRS, or scratchpad apply.
- No global root Application during G028 adoption.
- No prune or automated sync during initial adoption.
- No mutable revision during ownership transfer.
- No #1526/#1528 rerun while live ARC remains 20Gi.
- No #1523 push; no #1524 mutation; no canonical dirty-checkout mutation.
- No generated JSON or multispectrum evidence file.
- No failed transport or partial fact collection treated as approval.
