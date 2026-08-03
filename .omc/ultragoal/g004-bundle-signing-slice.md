# G004 slice — Cedar policy-bundle SIGNING + fail-closed verify-on-load — architect-scoped 2026-06-22 (dev dd91cd742)

Closes the SIGNING dimension of G004 (PARTIAL near-done: real cedar PDP + mTLS + retired hand-rolled evaluator already done). Net-new (NO signature exists today; verify-on-load is only closed-schema parse + version-token re-validation). DECOMPOSED: this slice = sign + verify-on-load + trust-config + tests ONLY. DEFERRED to separate slices: content-addressed push, CRD/operator delivery, removing the plain-TCP `start()` residual, the production signing PRODUCER.

## FOUNDER-GATED sub-step (scope AROUND, do NOT decide it)
Production PRIVATE signing-key custody root (human KMS/OpenBao vs owned-KMS ADR-0536 D5) is a custody decision — DEFER it. This slice ships: verify-against-a-configured-SET-of-trusted-PUBLIC-keys + a TEST-SIDE signer for fixtures. Production signing rides the CRD/operator slice. Verify-on-load is fully completable + meaningful without deciding custody.

## The seam (file:line)
- `iam/adapters/cloud-pdp-bundle-file/src/lib.rs:62-85` — `parse_bundle(raw)` (closed schema + `PolicyVersion::new` re-validate at :71) + `load()`. The verify MUST happen here (the store owns raw bytes) BEFORE the bundle reaches the engine.
- `iam/facade/cloud-pdp-app/src/server.rs:152-171` — `build_state`: `store.load()` → `CedarPdp::load`; boot-refusal at :153 (`StartError::Bundle`).
- `libs/oya-shared-pdp-adapter-cedar/src/lib.rs:109/132/142` — `CedarPdp::load/swap_bundle/compile` take a PARSED bundle, never raw bytes → verify belongs upstream at the store.
- `libs/oya-shared-pdp-kernel/src/lib.rs:115-156` — `PolicyBundle` closed schema (`deny_unknown_fields`, BTreeMap deterministic; round-trip proven at test :540).

## Design (fail-closed)
- **Outer envelope `SignedPolicyBundleDoc { bundle: <exact stored bytes>, signatures: [{key_id, public_key_hex, signature_hex}, …≥1] }`** — detached signature over the EXACT stored inner bytes (sign==verify by construction; the version-token re-validation stays INSIDE the verified region; no field-exclusion fragility). Mirror the existing `Ed25519Signature` shape (`audit/core/chain-domain/src/lib.rs:178-186`). Do NOT add a signature field inside `PolicyBundle`.
- **Reuse the OWNED aws-lc-rs signer/verifier (ADR-0506, ring-free):** `Ed25519ChainVerifier` + `Ed25519ChainSigner` in `libs/oya-shared-audit-digest-adapter-awslc/src/lib.rs:43-120`; ports `ChainSigner`/`ChainVerifier` in `libs/oya-shared-audit-event-kernel/src/lib.rs:405-420`. Do NOT use `audit/core/chain-domain` (ed25519-dalek = wrong backend). Do NOT add a new crypto dep.
- **verify-on-load (in the file-store adapter):** parse envelope → verify signatures against the TRUSTED key set → on success parse the inner bundle (existing schema + version-token path) → else `BundleStoreError` fail-closed (add a `SignatureRejected{detail}` variant at `iam/core/cloud-pdp-kernel/src/lib.rs:48-52`; still maps to `StartError::Bundle` boot-refusal). A bundle verifies if ANY trusted key whose key_id matches a signature validates the bytes (rotation = multiple trusted keys).
- **Trust anchor config:** add `OYA_CLOUD_IAM_PDP_BUNDLE_TRUST_DIR` to `PdpConfig` (mounted dir of trusted public-key files; ConfigMap projection) — mirror the `ENV_MTLS_CERT_DIR`/`from_lookup` pattern. ABSENT/EMPTY trust anchor = BOOT REFUSAL (a PDP that can't prove which keys to trust must not serve; mirror the mTLS `from_path` fail-closed precedent).
- **Canonical bytes:** sign over the EXACT bytes embedded in the envelope `bundle` field (serialize once deterministically — BTreeMap-sorted, proven — sign those, embed verbatim; verifier verifies the stored bytes then parses the SAME bytes). No serialize-then-compare trap.

## Clean-arch
`PolicyBundleStore` port UNCHANGED (still returns `PolicyBundle`; signed-bundle is a store-side obligation per the port doc). Kernel stays CRYPTO-FREE (sign/verify execution only in the awslc adapter + the file-store adapter; cloud-pdp-kernel change is config/error only). Cutover litmus: the W5 CRD/operator store swaps the file-store adapter; load()→PolicyBundle + the envelope/signature contract + trust-config seam unchanged.

## Tests (RED/GREEN; mirror cedar_pdp_conformance.rs + main_boot_closure.rs)
1. GREEN: envelope signed by a trusted key → load() returns bundle; ALLOW path works.
2. RED unsigned (empty signatures) → SignatureRejected/Malformed (fail-closed).
3. RED tampered (flip one inner byte) → rejected.
4. RED wrong-key (signer not in trust set) → rejected.
5. version-token still enforced INSIDE the verified region (signed bundle w/ malformed version token → still rejected by the inner check); unknown envelope field → deny_unknown_fields rejects.
6. key rotation: trust set {A,B}; bundle signed by B only → loads.
7. boot-closure RED: tampered/unsigned bundle file → boot refusal, NO socket serves (mirror main_boot_closure.rs fixture-2).
8. boot RED: absent/empty trust-anchor dir → boot refusal.

## born-accounting / buck2 / doctrine / done-bar
- NO new crate. Extend `iam/adapters/cloud-pdp-bundle-file` (verify + envelope) + `iam/core/cloud-pdp-kernel` (env const + PdpConfig field + SignatureRejected variant). Add deps to the adapter Cargo.toml + BUCK (rust_library + rust_test): `//libs/oya-shared-audit-digest-adapter-awslc` + `//libs/oya-shared-audit-event-kernel`. NOTE: first `iam/ → oya-shared-audit-*` cross-capability edge (legal lib→lib, NO inversion) — FLAG for adversarial review.
- aws-lc-rs/ring-free (crypto-backend-purity passes — zero ring). kernel-purity passes (kernel stays crypto-free). Tier-3 no unwrap/expect/panic in prod. Update any seed bundle fixture to the new envelope shape (cloud/cloud-iam helm + the test seeds + seed_parity if it covers the file-store seed).
- Regen lock + faces; firewall GO-LIVE + freshness + affected-set + crypto-backend-purity + kernel-purity + the live-PG/conformance suites; the NEW gate-live-postgres lane (does identity/iam touch it? the cedar conformance is hermetic, not live-PG — confirm no live-PG impact). face-settle --verify. Fresh worktree off origin/dev; never touch canonical; trailer EXACTLY `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.
- After green: STOP, do NOT self-approve — orchestrator runs adversarial security review (probe: empty-signatures truly fail-closed? version-token stays inside verified region? trust anchor boot-refuses when empty? canonical-bytes sign==verify? the new cross-capability dep sound? PR honestly states production key-custody is DEFERRED — no overclaim that "bundles are signed in prod").

## PR honesty
State: closes the verify-on-load SIGNING dimension of G004 (fail-closed verify against trusted public keys + test-side signing); production private-key custody + content-addressed-push/CRD delivery are tracked follow-ups (founder-gated custody). Do NOT claim G004 fully done.
