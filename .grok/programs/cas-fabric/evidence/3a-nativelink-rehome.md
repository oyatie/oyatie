# CAS Lane 3A — NativeLink CAS provider rehome

**Status:** draft PR open (path move only; no warm activation)  
**Date:** 2026-08-05  
**Authority:** `approved-plan-cas-re-20260805.md` Lane 3A; ADR-0560 (Proposed, cold); ADR-0562; ADR-0614  
**Hard stops honored:** no `warm_reads_licensed` flip; no RE (ADR-0612); no credentials; no cluster apply; no runner-scale coupling; no #1561/#1562

## Delivery

| Field | Value |
|-------|-------|
| PR | https://github.com/jason931225/oyatie/pull/1563 (draft) |
| Base | `dev` |
| Head branch | `agent/cas-3a-nativelink-rehome-20260805` |
| Head SHA | `e8495b28fdfdbaf96a1b9d53909f68d985a60005` |
| Parent | `a1bd1f14a` (#1558 G039 pilot packaging, origin/dev at lane start) |
| Worktree | `/Users/jasonlee/Developer/oyatie-cas-3a-nativelink-20260805` |
| Signature | SSH-signed (`ssh-ed25519`); local principal match unverified (allowed_signers path) |

## Exact move

| From | To | Notes |
|------|----|-------|
| `infra/nativelink/nativelink-cas.k8s.yaml` | `storage/adapters/nativelink/nativelink-cas.k8s.yaml` | 100% rename (content byte-identical) |
| `infra/nativelink/OWNERS` | `storage/adapters/nativelink/OWNERS` | 100% rename (`cloud-ci-platform`) |
| `infra/nativelink/` | *(deleted empty)* | no symlink / copy / alias |

**Active move plan (exactly one):** `specs/reorg/nativelink-storage-move-plan.json`  
- capability: `storage`  
- `moves: []` (artifact-only)  
- artifact: `infra/nativelink` → `storage/adapters/nativelink`

## Consumer list updated

| Consumer | Change |
|----------|--------|
| `specs/reachability-registry.json` | prefix `storage/adapters/nativelink/`; ownership seed path; rehome note in anchor |
| `ci/facade/operator-secret-rbac/operator-secret-bootstrap-policy.json` | `external_secret_scan_roots` + openbao-oya `manifest_paths` |
| `infra/arc/tests/ci_workspace_capacity.rs` | structural read path for nativelink manifest |
| `infra/external-secrets/RUNBOOK.md` | future GitOps Application path citation |
| `infra/arc/runner-scale-set-arm64-values.yaml` | comment path only (labels/endpoints unchanged) |
| `registry/fixuptasks.jsonl` | `named_in` observation path |
| `Cargo.toml` | `exclude += "storage/adapters/nativelink"` — non-crate leaf matches `*/adapters/*` but has no `Cargo.toml` |

## Preserved byte-for-byte (content)

- Cache-only CAS/AC tier (no scheduler/worker)
- Deployment / Service / ExternalSecret / PVC / NetworkPolicy names
- Instance endpoints: `nativelink-cas-writer…:50051`, `nativelink-cas-reader…:50052`
- Image: `ghcr.io/tracemachina/nativelink:v1.6.2@sha256:6750ab337eb1835ebe8452ddb76786641a80e23de71d8a5e630469399219b6ea`
- RWO PVC topology / Recreate strategy
- `specs/cache-warm-license.json` → `warm_reads_licensed: false` (untouched)

## Verification

| Check | Result |
|-------|--------|
| Multi-doc YAML structural parse (8 docs; Deployment `nativelink-cas`, etc.) | OK |
| Embedded `cas.json` JSON parse (`stores`/`servers`/`global`) | OK |
| `cargo test -p ci-operator-secret-rbac` | 23 passed |
| `buck2 test //infra/arc:ci-workspace-capacity-test` | 7 passed |
| `cargo metadata --no-deps` after exclude | OK |
| Live `rg infra/nativelink` residual | only move-plan `old_path`, rehome-anchor history; historical ADR prose intentionally retained |

## Residual / follow-ons

1. **Historical ADR prose** still cites `infra/nativelink/` (ADR-0560, ADR-0612, ADR-0606, ADR-0630, etc.) — intentional; not machine-consumed activation paths.
2. **No GitOps Application** yet points at the new path (still dark; reachability row grants no deployment claim). Activation remains a later reviewed PR.
3. **Lane 3B** must start from the **promoted** 3A head after post-merge proof (not this draft alone).
4. Population-parity (`N_pre = N_post = N_promoted > 0`) is a post-merge / promotion contract item per the approved plan verification block; this PR is path rehome only (no live apply).
5. Commit SSH signature present; local allowed_signers principal match not configured in this worktree (`.git/omx-local/allowed_signers` missing) — GitHub may still accept if org trust is configured.

## Files in PR (10)

```
Cargo.toml
ci/facade/operator-secret-rbac/operator-secret-bootstrap-policy.json
infra/arc/runner-scale-set-arm64-values.yaml
infra/arc/tests/ci_workspace_capacity.rs
infra/external-secrets/RUNBOOK.md
registry/fixuptasks.jsonl
specs/reachability-registry.json
specs/reorg/nativelink-storage-move-plan.json
storage/adapters/nativelink/OWNERS          (renamed)
storage/adapters/nativelink/nativelink-cas.k8s.yaml  (renamed)
```
