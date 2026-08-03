# Fresh-context adversarial review — PR #686 (G002 slice 2: cloud-kms K8s operator)

- Repo/branch: jason931225/oyatie @ `agent/g002-kms-operator`, head `a24a52728`, base `dev`, merge-base `430ee02ee`.
- Reviewer: Fable (Claude) fresh-context reviewer of record. No worker self-review exists; this verdict gates the merge.
- Standard applied: RUBRIC-torvalds-review.md (Torvalds §1–5, hyperscaler §6, owned-architecture §7) + BRIEF-g002-kms-operator-slice2.md. TRUST SUBSTRATE — elevated security severity floor.
- All verification run FOREGROUND with buck2 from `/Users/jasonlee/oyatie-worktrees/g002-kms-operator`.

## VERDICT: APPROVE

No CRITICAL or HIGH defect at HIGH confidence survives hostile inspection. Kernel purity, fail-closed actuation, crypto posture (no key material in operator), least-privilege RBAC, idempotent reconcile, and accounting attribution all verify in code and by running buck2. Findings below are MEDIUM/LOW hardening + accounting-hygiene items; none gate the trust substrate. Two MEDIUM items (ADR governed-surfaces undercount; PDB-wedges-singleton) are worth fixing pre- or fast-follow-merge but do not block: they are operational/accounting, not correctness or security regressions.

---

## Evidence — commands + exact outputs (buck2, leader-env daemon)

1. `buck2 build //…/oya-cloud-kms-operator-kernel:oya-cloud-kms-operator-kernel` → `BUILD SUCCEEDED` (7 local cmds).
2. `buck2 test //…/oya-cloud-kms-operator-kernel:oya-cloud-kms-operator-kernel-tests` → `test result: ok. 8 passed; 0 failed` (incl. `applying_create_rotate_demote_and_quarantine_actions_is_idempotent`, `partial_or_ambiguous_observation_only_emits_fail_closed_quarantine`).
3. `buck2 build //…/oya-cloud-kms-operator-k8s-adapter:…` → `BUILD SUCCEEDED` (291 local cmds; compiled `k8s-openapi-0.27`, kube-rs chain). **FRIC-1781113000 resolved in leader env — adapter compiles.**
4. `buck2 build //…/oya-cloud-kms-operator-app:…` → `BUILD SUCCEEDED` (5 cmds).
5. `buck2 test //…/oya-cloud-kms-operator-k8s-adapter:…-tests` → `test result: ok. 25 passed; 0 failed` (incl. `partial_observed_state_fails_closed_without_side_effects` asserting `actuator.actions == []`; `domain_actuator_fails_closed_for_quarantined_observed_state`; 4 stale-retry idempotency tests).
6. `buck2 test //…/oya-cloud-kms-operator-app:…-tests` → `test result: ok. 4 passed; 0 failed`.
7. `buck2 test //…/oya-cloud-kms-domain:oya-cloud-kms-domain-unittest` → `test result: ok. 30 passed; 0 failed` (canonical generator target; incl. `directory_operator_lifecycle_ports_record_sealing_root_demote_and_quarantine`, `provider_encrypt_request_validates_refs_without_plaintext_material`). NOTE: the harden commit's `oya-cloud-kms-domain-tests` stanza is correctly retired by the leader's conflict resolution toward the canonical `-unittest` stanza — `Unknown target oya-cloud-kms-domain-tests` confirms removal.
8. Total: **67 tests pass, 0 fail** across the 4 owned targets + domain.

Static checks:
- `grep -rnE '\.unwrap\(\)|\.expect\(|panic!|todo!|unimplemented!|unreachable!'` over operator src (non-test) → **NONE**.
- `#![forbid(unsafe_code)]` present in kernel/adapter/app lib.rs + app main.rs.
- Operator crates grep for `plaintext|ciphertext|key_material|secret_bytes|raw_key|private_key|[u8]|Vec<u8>|wrapping_key|dek|kek` → **NO MATCHES**. Operator actuates through domain refs (`root_ref`, `key_id`) only; never touches key bytes.
- Kernel Cargo deps = `serde` only; kernel BUCK deps = `third-party//:serde` only. **ZERO kube/k8s-openapi/tokio.** Injected `Clock` trait; no `SystemTime`/fs/process in kernel decision paths (`SystemClock` lives in the app, injected).

Mechanics:
- Settle protocol: HEAD `a24a52728` touches ONLY `*.generated.json` (faces-only settle last). Content commits first. **Compliant.**
- SSH signatures: all content + recovered commits show `%G? = U` (valid signature, signer not in reviewer allow-list) — signed, expected in fresh env.
- Key-diff both ways vs origin/dev (`gate-baseline`, `accounting-registry`, `scm-facts`): see Finding 6 / accounting note below.

---

## Numbered findings

### Finding 1 — Top-level relist consistency signal is unwired in the live adapter — MEDIUM, HIGH confidence
`oya-cloud-kms-operator-k8s-adapter/src/lib.rs:194` — `project_observed_state` hardcodes `read_consistency: ReadConsistency::Complete`. The adapter NEVER emits `Partial`/`Ambiguous`, so the kernel's top-level fail-closed branch (`oya-cloud-kms-operator-kernel/src/lib.rs:213` → `QuarantineObservedState`) is dead on the live path. The runbook (line 67) and SLO assert `read_consistency=Partial/Ambiguous` as observable conditions that can never appear.
Why it (mostly) doesn't bite: partial reads ARE caught by two other fail-closed paths — (a) `list()` error → propagated → error-policy requeue with backoff (`lib.rs:582`, `568-575`); (b) per-object missing `status.versions`/`status.health`/`observedVersion` → `PartialObservedState` error → fail-closed (`lib.rs:1489-1500,1553-1564`, tested at adapter.rs:131/152/173). And the missing-CR case is safe-by-construction: `desired` is derived from observed (`desired_state_from_observed`), so a dropped CR drops from desired and produces no action (no spurious delete — there is no delete action). Per-resource ambiguity IS wired via CRD `status.health: Ambiguous|Compromised` → kernel quarantine (`lib.rs:236,276`).
Minimal fix: either (i) have `observe_current_state` set `read_consistency` from an actual relist-consistency signal (resourceVersion continuity / watch-desync detection), or (ii) delete the `Partial`/`Ambiguous` enum variants + the runbook/SLO references so the documented contract matches the wiring. Today the operator's "fail-closed on ambiguous OBSERVED-STATE consistency" is a documented property the adapter does not actually produce.

### Finding 2 — ADR-0543 "Governed surfaces" undercounts the real diff — MEDIUM, HIGH confidence
`docs/decisions/ADR-0543-cloud-kms-operator-commissioning.md:46-68` lists 23 governed surfaces but OMITS 2 (3) genuinely-changed files: `cloud/cloud-kms/crates/oya-cloud-kms-domain/src/lib.rs` (the substantial harden-commit domain change adding `SealingRootRef`, `KmsKeyVersionLifecycle`, `KmsSealingRoot`, demotion/quarantine request types — +328/−? lines) and `cloud/cloud-kms/iac/k8s/helm/values.yaml` (the Helm values the deployment/PVC/PDB/RBAC templates all interpolate). Req #6 requires the governed-surfaces list to MATCH the diff file set. The domain change is the most security-relevant code in the PR (it is the actuation port surface) and it is unlisted in the commissioning ADR that exists specifically to account for these surfaces.
Verified via set-diff: `comm -23 <diff-non-generated> <ADR-backticked-paths>` → `oya-cloud-kms-domain/src/lib.rs`, `helm/values.yaml`, (ADR self). (The 4 apparent "phantom" ADR entries — `helm/`, the 3 crate names, the `reconcile(...)` signature — are prose code-spans in the Decision section, not list entries; not phantoms.)
Minimal fix: add `cloud/cloud-kms/crates/oya-cloud-kms-domain/src/lib.rs` and `cloud/cloud-kms/iac/k8s/helm/values.yaml` to the Governed surfaces list.

### Finding 3 — PDB minAvailable:1 on a single-replica operator wedges node drains — MEDIUM, HIGH confidence
`iac/k8s/helm/values.yaml:30` `operator.replicaCount: 1` + `:55` `pdb.minAvailable: 1` (template `operator-pdb.yaml:11`). With 1 desired replica and `minAvailable: 1`, the PDB permits ZERO voluntary disruptions → `kubectl drain`/node-maintenance/cluster-autoscaler eviction of the operator pod will block indefinitely. Reconcile is level-triggered and idempotent (verified), so a brief single-pod gap during eviction is safe.
Minimal fix: use `maxUnavailable: 1` for the singleton (allows the pod to be evicted and rescheduled), or raise `operator.replicaCount` ≥ 2 with leader-election. As written, the PDB protects availability the deployment doesn't provide and trades it for a maintenance deadlock.

### Finding 4 — Persisted-state round-trip uses a quarantine workaround for Disabled keys — MEDIUM, LOW confidence
`oya-cloud-kms-operator-k8s-adapter/src/lib.rs:984-1064` (`into_directory`) — a persisted key with `state=Disabled` and `current_version>1` is force-created as `Enabled` (line 1001-1003) then re-quarantined (1018-1027) because the domain `create_key` won't accept a disabled initial state. The final converged state is correct (quarantined), and `persistent_domain_repo_reloads_operator_mutations_from_state_path` (adapter.rs:712) covers the basic round-trip, but the multi-version Disabled restore path with interleaved version lifecycle is NOT directly asserted byte-for-byte after reload. Risk: a subtle drift in restored `updated_at`/version-lifecycle timestamps on operator restart.
Minimal fix: add a golden round-trip test that persists a 3-version key in `DecryptOnly`+`Quarantined` mix, reloads, and asserts the directory is byte-identical. (Low confidence it actually misbehaves; the logic reads correct.)

### Finding 5 — PerPack residency persistence fails the whole snapshot — LOW, HIGH confidence
`oya-cloud-kms-operator-k8s-adapter/src/lib.rs:2157` — `domain_residency_label(ResidencyClass::PerPack(_))` returns `Err`, which `from_directory`→`from_key` propagates, failing the ENTIRE `persist()`. The operator can't originate PerPack keys (CRD enum is only the 3 modes), so this only triggers if the shared `CloudKmsDirectory` ever contains a PerPack key from another writer. It fails closed (error, not silent loss), so it's safe, but it's a latent coupling fragility: one foreign residency class wedges all operator persistence.
Minimal fix: either persist PerPack faithfully (carry the pack id) or scope the persistent snapshot to operator-owned keys so a foreign key class can't block the operator's own state write.

### Finding 6 — Accounting: 24 adds are fully attributed, ZERO laundering — NOT A DEFECT (positive verification)
Key-diff both ways vs `origin/dev` on the three generated faces:
- `gate-baseline.generated.json`: member keys dev=133, head=133 → **0 adds / 0 removes** (no gate-member churn).
- `accounting-registry.generated.json` `rows`: **+24 / −1**; all 24 adds are EXACTLY this PR's new files (operator crates, helm CRDs/templates/PDB/PVC, runbook, SLO, OWNERS, ADR-0543, evidence). `stale(merge-base − dev) = 0`.
- `scm-facts.generated.json` `tracked_paths`: identical **+24 / −1**, same file set.
- `decision-crosswalk.generated.json`: `decision_count 362→363` + one ADR-0543 record (`in_spec:true, status:Proposed`) — producer-mechanical.
The brief's anticipated "~62 target-parity stale-base member keys" do NOT appear as unexplained adds: the branch's merge `fcf31152d` ("Merge origin/dev") already reconciled the stale base (`stale(mb−dev)=0` on all faces), so target-parity netted out. **ZERO accounting-class (unjustified/unowned/unreachable) adds remain** — every add is an owned (`OWNERS: axis-cloud-platform`), ADR-governed operator file. This is full attribution, the opposite of laundering.

---

## Axis scorecard (rubric)

- §1 Intent: solves the commissioned problem (G002 missing operator) — CRDs + pure kernel + transient adapter + GitOps + SLO + runbook all present and wired. Right fix, not a perfect impl of the wrong fix. PASS.
- §2 Cited-test reality: every claimed test exists and asserts what's claimed; idempotency/fail-closed/stale-retry use value/byte equality (`assert_eq!`), not `.contains` (the only `.contains` are on CRD-manifest string presence — appropriate). 67/67 pass via buck2. PASS.
- §3 Silent failure modes: fail-closed honored on list-error + partial-status + quarantined-observed-state (with the Finding-1 caveat that the *top-level consistency* signal specifically is unwired); no fail-open; no data-drop on the desired-from-observed derivation. PASS (with Finding 1).
- §4 No weakening: no gate/baseline/assertion relaxed; generated faces are producer-mechanical; the only "removed" target (`oya-cloud-kms-domain-tests`) is a duplicate retired toward the canonical `-unittest` stanza, not a coverage relaxation (domain still 30/30). PASS.
- §5 Repo doctrine: buck2-green, no unwrap/expect/panic in prod, `#![forbid(unsafe_code)]`, settle protocol faces-last, SSH-signed, single-concern crates (ADR-0132), no new merge authority. PASS.
- §6 Hyperscaler lens: kernel/adapter split = level-triggered reconcile + pure decision core (Borg/operator-pattern precedent), injected clock, wide-event-per-cycle (observability precedent), exponential backoff capped. Idempotent actuation guards (`is_some`/`sealing_root_exists`) match controller-runtime convergence semantics. No bespoke mechanism where a proven pattern fits. PASS.
- §7 Owned-architecture: kernel trait shapes (KeyRing/SealingRoot/Action/ObservedState) model the cloud-k8s destination, not kube-rs idiosyncrasy; kube-rs/k8s-openapi confined to the ADR-0510-marked adapter crate; cutover litmus holds (swap adapter only). Trait would NOT change at cutover. PASS.

## Positive observations (reinforce)
- Crypto posture is exemplary: zero key material in any operator crate; actuation strictly through domain refs.
- RBAC is genuinely least-privilege: namespaced `Role` (not ClusterRole), no `secrets` verbs, no cluster-admin; only `kms.oyatie.com` CRDs + `/status` subresources + `events` create/patch. mTLS via read-only Secret volume.
- Deployment is hardened: distroless command (no shell), `runAsNonRoot`, `readOnlyRootFilesystem`, `drop: ALL`, seccomp RuntimeDefault, cosign digest gate that `fail`s on a fake/zero digest.
- CRDs declare `subresources.status: {}` + strong field validation (tenantId regex, enums, required status.versions) — the schema actively drives the fail-closed projection.
- Honesty in artifacts: SLO + runbook explicitly mark the Prometheus exporter `target-only`/`exporter_gap` rather than over-claiming a live histogram.
- The two reflog-recovered hardening commits ("harden actuation", "idempotent sealing-root retries"), read hostilely with no prior review, are coherent and test-backed: they introduce the domain-repo trait + persistence + idempotency guards and add real value-equality tests (`sealing_roots().count()==1` after double-cycle). No smell.

## Residual risk
The single most likely production failure even after merge: a Kubernetes watch desync / API-server relist that returns a SUBSET of `KmsKeyRing` CRs WITHOUT an error and WITHOUT a per-object status gap (i.e. the surviving CRs are individually complete). Because `read_consistency` is hardcoded `Complete` (Finding 1) and `desired` is derived from observed, the operator would silently treat the truncated set as ground truth and simply not reconcile the dropped key rings — a stuck/under-actuation gap (NOT mis-actuation: it won't rotate/quarantine the wrong key, and it won't delete). For a trust substrate this is a convergence-availability risk, not a key-safety risk, which is why it's MEDIUM not blocking — but it is the failure the current fail-closed wiring does not catch. Recommend Finding 1's fix (real relist-consistency signal) as the first G002 fast-follow.
