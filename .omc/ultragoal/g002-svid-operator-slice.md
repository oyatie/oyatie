# G002 slice-1b-iii-c — live SVID-delivery operator (architect-scoped 2026-06-21)

Closes FRIC-1781490000 (single named OPEN friction). Unblocks G004 PDP slice-2 per-tenant policy.
The PDP cryptographic-tenant vertical is ~95% built+RED-proven on origin/dev; the ONLY gap is the in-cluster
producer of the `oya-cloud-iam-pdp-svid` Secret. ADR-0561 names this exact slice; two-way door.

## What already exists (consume, do NOT rebuild)
- iam/core/identity-workload-svid-kernel/src/lib.rs:306(bind_caller_tenant #717),430/442/456 (WorkloadIdentityIssuer/SvidVerifier/TrustBundleSource ports)
- iam/adapters/identity-workload-svid-trustd/src/{lib.rs,leaf_der.rs} — REAL ECDSA-P256 X.509 issue/verify (rcgen+x509-parser on aws-lc-rs; ring-free). TrustdSvidIssuer::issue = the issuance port the operator drives.
- iam/facade/cloud-pdp-app/src/{mtls.rs,client_cert_verifier.rs,mtls_transport.rs} — live rustls mTLS PEP; main.rs boot_from_config → MtlsContext::from_path + start_with_mtls, FAIL-CLOSED (exit 1, never plain TCP).
- cloud/cloud-iam/iac/k8s/helm/templates/deployment.yaml — already mounts the `oya-cloud-iam-pdp-svid` Secret RO at /etc/oya-cloud-iam-pdp/tls; comments the producer "is slice-1b-iii-c".

## CONSUMER CONTRACT (the byte-for-byte spec the operator output MUST satisfy)
MtlsContext::from_path (iam/facade/cloud-pdp-app/src/mtls_transport.rs:79-84,215) reads a kubernetes.io/tls Secret:
- tls.crt = PEM leaf chain (line 79)
- tls.key = PKCS#8 PEM (line 82)
- ca.crt  = one+ CA certs; real SPKI DER feeds TrustBundle::trusted_ca_spki_ders (line 84)
Secret name EXACTLY `oya-cloud-iam-pdp-svid`. Match these filenames exactly.

## Build (mirror the kms-operator split precedent)
New crates (axis-identity OWNERS; full born-accounting):
1. iam/core/identity-workload-svid-operator-kernel — PURE `reconcile<C: Clock>` (mirror secrets/core/kms-operator-kernel/src/lib.rs:207 + tests/reconcile.rs). DesiredState{spiffe id `spiffe://oyatie.cell-<id>/platform/cloud-iam-pdp`, ttl, secret name/ns}; ObservedState{secret present? leaf expiry vs now}; Action{Issue, Rotate(within window), Noop}. No I/O/clock/crypto. kernel-purity (zero transient deps).
2. iam/adapters/identity-workload-svid-operator-k8s — kube-rs adapter (ADR-0510 transient; mirror secrets/adapters/kms-operator-k8s/src/lib.rs Controller/watcher). Calls TrustdSvidIssuer::issue to mint leaf+key+CA bundle, projects the kubernetes.io/tls Secret (tls.crt/tls.key/ca.crt).
3. iam/facade/identity-workload-svid-operator-app — binary; *_from_env config validation + non-zero exit on failure; compose actuator+runtime + reconcile loop (mirror secrets/facade/kms-operator-app/{lib.rs,main.rs}). cell-id from env (mirror OYA_KMS_OPERATOR_NAMESPACE).
4. SLO: iam/observability/slos/identity-workload-svid-operator-kernel/svid-delivery-availability.openslo.yaml (mirror the svid-kernel SLO).
5. Helm: operator Deployment + RBAC (Secret create/update in cloud-iam ns) under cloud/cloud-iam/iac/k8s/helm/templates/ (mirror cloud/cloud-kms/iac/k8s/helm/templates/operator-{deployment,rbac}.yaml).

Clean-arch: kernel(pure reconcile)→port(existing WorkloadIdentityIssuer/TrustdSvidIssuer)→adapter(kube-rs Secret projection)→facade(env-config+loop). Issuance via the unchanged SigningBackend seam (real EcdsaP256Signer today; cloud-kms swap LATER, no kernel/port change).

## Tests (RED/GREEN, in-process, no live K8s)
- kernel unit: reconcile empty→Issue; within-window→Rotate; fresh→Noop (mirror kms reconcile.rs).
- adapter: issue via TrustdSvidIssuer; assert produced Secret tls.crt/tls.key/ca.crt round-trip through MtlsContext::from_path + verify vs trust bundle (closure proof).
- E2E keystone GREEN: extend iam/facade/cloud-pdp-app/tests/main_boot_closure.rs so the mount is OPERATOR-PRODUCED (not test-written) → PDP boot_from_config → real rustls handshake: trusted tenant SVID = ALLOW bound to SVID tenant; cross-tenant = 403/PermissionDenied. THIS is what flips FRIC-1781490000 closed.
- fail-closed RED: operator down/Secret absent ⇒ PDP boot refuses (keep as guard).

## Deps / done-bar
- aws-lc-rs/ring-free (ADR-0506, FRIC-1781520000): reuse existing rcgen/x509-parser/rustls aws-lc-rs; assert `buck2 cquery "deps(//...)" | grep -c ring-0.17` == 0 (or the repo's ring-free check).
- buck2 wiring (BUCK+reindeer) for the 3 new crates = part of done. Regen Cargo.lock + materialize faces + run freshness/affected-set gates (buck2-build-green≠CI-green).
- Born-accounting: OWNERS + reachability-registry + capability mapping + catalog + workspace member for each new crate, justified by an ADR-0561 amendment `## Governed surfaces` row. (Could dogfood register_crate later; do manually now.)

## SCOPE CAVEAT (do not overclaim, do not pull in)
Roots on the trustd in-memory/EcdsaP256 CA. cloud-kms signer swap (per-cell sealing-root) stays DEFERRED behind the unchanged SigningBackend seam (ADR-0561 D4/D5). PR states: cert-delivery dimension of FRIC-1781490000 CLOSES; cloud-kms-rooting remains a separately-tracked follow-up. NO full-G002-completion overclaim (twice-burned overclaim rule #722/#725).
