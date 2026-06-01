# K8s Admission/Policy Substrate — Layered (VAP+CEL default, bespoke-Rust signing webhook)

> idea-refine + best-practice-research output, 2026-05-31. CONSENSUS (founder-locked): adopt the
> layered model; in-tree VAP+CEL+MAP as default; bespoke-Rust cosign-verify webhook as the
> signature-verification destination. SUPERSEDES ADR-0379 (Kubewarden-as-default).

## Problem Statement
How might we choose oyatie's Kubernetes admission/policy substrate so it is maximally
hyperscaler-native AND dogfood-aligned — without depending on the least-mature engine
(ADR-0379 picked Kubewarden = CNCF Sandbox, single-vendor) for cases that Kubernetes itself
now handles natively?

## Recommended Direction (consensus, founder-locked 2026-05-31)
**Layered admission, superseding ADR-0379's "Kubewarden as default":**
- **Tier 1 — DEFAULT = in-tree ValidatingAdmissionPolicy + MutatingAdmissionPolicy + CEL.**
  Both GA on our cluster (k8s **v1.36.1**; VAP GA 1.30 / MAP GA 1.36). Carries ALL declarative
  structural + mutation policy (registry allowlist, digest-pin / no `:latest`, PSS restricted,
  nonroot UID 65532, label/annotation discipline). Zero dependency, zero vendor, upstream-
  maintained — the strongest hyperscaler-lens fit, and the emerging industry best practice
  (Gatekeeper AND Kyverno both now *generate* VAPs and keep the webhook only as backup/residue).
- **Tier 2 — webhook ONLY for the residue CEL cannot express.** Flagship case: container image
  **signature verification (cosign)** — architecturally impossible in pure CEL (needs an outbound
  registry fetch + crypto; the k8s docs use exactly this as the canonical "needs a webhook" case).
- **Tier 3 — DESTINATION = a bespoke-Rust cosign-verify admission webhook** for that residue,
  sharing code with the **bespoke OCI-registry + signer product** (the product surface already
  designated). **key-based cosign** (static public key, ctlog/tlog disabled — correct for a
  self-hosted cluster with no public Fulcio/Rekor).
- **Kubewarden: DEMOTED** default → optional Rust/WASM escalation (honest about Sandbox/single-
  vendor maturity; keep only if a genuine CEL-can't-express case appears). **Kyverno: removed
  from the base overlay** (ADR-0379 already said adapter-only; live cluster still runs it = drift).

## Evidence (best-practice-research, cited 2026-05-31)
- VAP GA k8s 1.30 (2024-04-24); MutatingAdmissionPolicy GA k8s 1.36 (2026-04-22). [kubernetes.io]
- CNCF tiers: **Kyverno GRADUATED 2026-03-24**; **OPA Graduated 2021**; **Kubewarden STILL Sandbox**
  (since 2022, single-vendor SUSE/Rancher, ~230⭐, thin adoption). [cncf.io]
- Pure VAP+CEL CANNOT verify cosign signatures (in-process, no network/crypto). [kubernetes.io/docs/concepts/policy]
- Layered (VAP-first + webhook-residue) is the recommended 2026 pattern, baked into Gatekeeper &
  Kyverno (`use-vap`), endorsed upstream. GKE Policy Controller = Gatekeeper; AKS auto-generates
  VAPs from CEL. [k8s admission-webhook-good-practices; Gatekeeper/Kyverno docs; cloud docs]
- A bespoke admission WEBHOOK is fine for the narrow residue; a bespoke admission ENGINE is an
  anti-pattern (re-implements cert-rotation/HA/fail-modes/policy-CRDs). [k8s good-practices]

## Key Assumptions to Validate
- [ ] VAP+MAP cover all current admission needs except signing. Test: express ADR-0183's 8
      canonical policies (PSS-restricted, workload-labels, digest-pin, etc.) as VAP/CEL; confirm
      only signature verification needs a webhook.
- [ ] A bespoke-Rust cosign-verify webhook is bounded. Test: spike `sigstore-rs` key-based verify
      + a minimal axum/hyper validating webhook with cert-manager TLS; measure latency budget.
- [x] key-based cosign (ctlog disabled) is correct for the self-hosted registry. [confirmed by research]

## MVP Scope
- **Gate loop (now):** UNBLOCKED — admission enforces nothing today, signing deferred. Proceed.
- **This decision (now):** the ADR superseding ADR-0379 + this one-pager + memory.
- **Fast-follow (when signing is turned on):** the bespoke-Rust cosign-verify webhook (Tier 3).
  Kyverno-interim was explicitly NOT chosen — go bespoke when ready.
- **Later:** author the 8 canonical policies as VAP/CEL; remove Kyverno from the base overlay;
  optional Kubewarden escalation only if a real CEL-can't-express case appears.

## Not Doing (and Why)
- Kubewarden as default — Sandbox-tier, single-vendor; superseded by more-native, more-mature in-tree VAP+CEL.
- Any general-purpose webhook engine as the PRIMARY substrate — layered pattern uses in-tree for the common cases.
- A bespoke admission ENGINE — only a bespoke webhook for the narrow signing residue; never reinvent the engine.
- Keyless/Fulcio cosign — needs infra we don't self-host; key-based is correct.

## Open Questions
- Reciprocal supersede of ADR-0379 (status → Superseded, superseded_by → new ADR); does ADR-0246
  (policy-engine-substrate-promotion) or ADR-0181 (image promotion) need an amendment too?
- Does the bespoke-Rust signing webhook live in the OCI-registry product crate or a standalone
  admission crate? (It shares the cosign-verify code path.)

→ becomes an ADR superseding ADR-0379 (number TBD), amending ADR-0246/0181 as needed.
