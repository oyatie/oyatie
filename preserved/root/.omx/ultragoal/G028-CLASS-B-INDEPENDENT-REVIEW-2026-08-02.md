# G028 class-B independent architecture review — 2026-08-02

State: **REQUEST_CHANGES — BINDING — NO IMPLEMENTATION — NO LIVE MUTATION**  
Reviewed source baseline: `origin/dev` `0c1014b87f0d881a821faa6a872b309deba0cfbf`  
Reviewed packet: `.omc/ultragoal/G028-CLASS-B-PERMANENT-LAB-GITOPS-DESIGN-2026-08-02.md` before the post-verdict state annotation  
Class selection: **B remains correct** under the 2026-07-29 founder ruling; this verdict rejects the transition design, not the class.

## Verdict

`REQUEST_CHANGES`

The design states the desired end state but does not define a safe executable transition from live Helm ownership to Argo ownership. Implementation must not start from the reviewed packet.

## Findings

### Critical

1. **No executable CRD readiness barrier.** The proposed single multi-document bundle and single apply cannot guarantee `Application` CRDs are `Established=True` before a root `Application` is submitted. YAML document order is not readiness.
2. **First sync has full-platform blast radius.** Existing `infra/gitops/root-app.yaml` and `infra/gitops/values.yaml` instantiate roughly 18 Applications, including networking, policy, identity, storage, and observability. Root and children enable prune immediately (`infra/gitops/root-app.yaml:24-26`; `infra/gitops/templates/applications.yaml:56-60`), while the packet proves adoption only for ARC.
3. **ARC ownership identity is underspecified.** No immutable pre-adoption inventory binds both ARC Helm releases to admitted Argo renders by release name, namespace, chart/app version, values digest, resource tuple, selectors, service accounts, Helm ownership annotations, and proposed Argo tracking identity. The template does not set `helm.releaseName` (`infra/gitops/templates/applications.yaml:20-31`).

### High

4. **Argo and ARC render inputs are not content-addressed.** Existing Argo machinery uses ambient Helm repositories and raw GitHub CRD URLs (`infra/capi/crs/render.sh:12-20,37-38,49-53`; `infra/capi/clusters/values.yaml:17-26`). Version strings alone do not identify bytes.
5. **Initial ownership transfer follows mutable `dev`.** Root and values point at `dev` (`infra/gitops/root-app.yaml:14-17`; `infra/gitops/values.yaml:3-4`), contradicting the admitted immutable commit/digest boundary.
6. **Secret continuity and rollback are obligations, not procedures.** `githubConfigSecret: oyatie-arc-app` is declared (`infra/arc/runner-scale-set-arm64-values.yaml:9-14`), but expected namespace/type/key names, metadata-only fingerprint, restoration actor/source, exclusion from prune, last-known-good artifact/value/resource inventory, and ownership-metadata rollback are absent.
7. **Bootstrap and rollback identities are unnamed.** “platform/CI owner” is not an execution principal. Exact identity, target/API fingerprint, least-privilege RBAC, credential source/lifetime/revocation, digest-gated operation, audit sink, rollback owner, and escalation path are unresolved.
8. **Protected Buck2 fan-in is asserted, not designed.** No target label, Rust gate, workflow job, final `needs`/result check, affected-set roots, or born-blocking registration fixture is named. Repository authority requires real executable fan-in (`ci/facade/baseline-ratchet/tests/gate_registration.rs:6-11,1545-1550`; `docs/AGENTS.md:174-177,248-252`).

### Major

9. **Review was not bound to packet bytes.** The planning packets are not tree objects at source baseline `0c1014b87…`. A future approval must bind exact packet digest or committed blob/tree identity together with baseline SHA, reviewer identity, verdict, and timestamp.

## Required v2 architecture

```text
admitted content-addressed/vendored inputs
→ phase 1: Namespace + Argo CRDs only
→ wait: every required CRD Established=True and admitted version observed
→ phase 2: pinned Argo controllers only
→ wait: required deployments Available and admitted version observed
→ preflight: immutable live-vs-admitted ARC controller + scale-set inventory; secret metadata continuity; exact target/API identity
→ phase 3: ARC-only exact-SHA Applications, explicit release names, automated sync off, prune=false
→ controlled first sync and health/22Gi proof
→ separately admitted transition to protected dev tracking
→ separately admitted prune enablement only after ownership proof
→ separately reviewed expansion beyond ARC
```

Each phase needs a distinct deterministic artifact digest and abort boundary. The first adoption must not instantiate unrelated platform Applications.

## Required exact inputs before v2 can be approved

- Content-addressed/vendored Argo and ARC artifacts with one machine-readable pin authority.
- Exact Buck2 target, gate owner, workflow execution job, final protected fan-in, affected roots, and false-green fixture.
- ARC controller and scale-set live inventory plus deterministic admitted-render diff contract.
- Secret metadata-only preflight and executable restoration/rollback contract.
- Named bootstrap principal and named rollback owner with scoped authority and audit destination.
- Packet-byte attestation.

## Non-actions

- No implementation or implementation worktree from this rejected packet.
- No cluster mutation, Helm apply, CRS apply, or bootstrap.
- No #1526/#1528 rerun while live ARC remains 20Gi.
- No #1523 push and no #1524 mutation.
- No transport failure or partial revision treated as approval.
