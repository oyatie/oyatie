# Cloud-CI as Product: the paved-road pipeline

Refined 2026-06-10 via /idea-refine with founder decisions in-session. Status: concept one-pager;
the governed artifact is a follow-up ADR lane. Founder directives bound here:
(a) "if any part of our pipeline is not reusable, it is not meeting our requirements";
(b) "cloud native, hyperscaler pattern, hermetic, universal — so anyone can build cloud-native,
hermetic applications if it goes through our pipeline."

## Problem Statement

How might we turn Oyatie's enforcement pipeline (ratchet gates + baselines + friction ledger +
exception decay) into a hermetic, repo-agnostic product — a paved road such that any application
that goes through it comes out cloud-native, hermetic, and production-grade by construction?

## Recommended Direction (founder-decided 2026-06-10)

**Neutral ratchet engine + policy packs, delivered as a K8s operator with CRDs, proven against
public GitHub repos, with the loop closed by a FRIC-total-accounting meta-gate.**

The product is two things layered:

1. **The ratchet engine (universal).** One kernel contract already latent in 15/18 of our gates:
   `collect(root, policy) → observed rows` + `evaluate(policy, observed) → findings → verdict`,
   with baseline semantics (new violations block, baselines only shrink, exceptions are decaying
   leases not permanent grants). Precedents: Google Tricorder (analysis-as-platform, criticism-
   driven feedback), OPA/Gatekeeper (neutral engine, ConstraintTemplate/Constraint = engine/pack
   split on CRDs), Betterer-class quality ratchets. We reimplement Rust-native per doctrine.
2. **The paved road (the promise).** Doctrine ships as *policy packs* (oyatie pack: Rust-first,
   zero-shell, buck2-hermetic, K8s-native presence — operator/CRD/PDB/SLO/runbook/OWNERS), plus
   scaffolding that emits the proven service shape (kernel/adapter/app crates + helm + SLOs +
   runbooks — exactly what PR #686 hand-built; the scaffold mechanizes it). Hermeticity evidence
   rides SLSA-style provenance + cosign (already in our values.yaml posture). Precedents:
   Netflix paved road, Backstage golden paths, Tekton Chains.

Surface sequencing (respects "K8s operator first" while de-risking the bootstrap): the engine
extraction is identical work for every surface, so (1) extract kernel + policy packs to `libs/`,
(2) wrap in operator CRDs (`GatePolicy`, `Baseline`, `Exception`, `GateRun`) using the cloud-kms
kernel/adapter/app pattern we just shipped, (3) the GitHub required-check becomes one adapter of
the same kernel. Conformance = a harness that points read-only collectors at public GitHub repos
and snapshots verdicts — proving universality without a distribution commitment.

## Acceptance criterion R0 (binding, from directive (a))

Every pipeline component must be consumable outside this repo. Engine = crates/binaries taking
policy-as-data; the ONLY repo-specific residue allowed is policy packs + baselines + ledgers.
Current audit: 18 gates; 15 kernel-shaped; **only 2 ship policy as data** — 16 bake policy into
code. Each is a reusability violation → friction rows → ratcheted down like any other debt.

## Key Assumptions to Validate

- [ ] Kernel contract covers ≥90% of existing gates — validate by per-gate audit (the 15/18 count
      is `fn evaluate` presence, not full contract conformance).
- [ ] Public-repo conformance runs are read-only and bounded — validate on 3 pilot repos.
- [ ] Operator surface reuses the kernel/adapter/app pattern cleanly — validate with a `GateRun`
      CRD walking skeleton before committing the CRD schema.
- [ ] Friction ledger rows are machine-parseable enough to enforce termination states (gate |
      automation | accepted-risk) — validate by schema-linting `.omc/ultragoal/friction-ledger.jsonl`;
      no gate consumes the ledger today, so the meta-gate is greenfield.

## MVP Scope

In: kernel extraction (`libs/oya-gate-kernel` + policy-pack format), 2 existing gates migrated to
packs as proof, FRIC-total-accounting meta-gate, public-repo conformance harness (3 repos),
GateRun/GatePolicy CRD walking skeleton. Out: everything in Not Doing.

## Not Doing (and Why)

- **SaaS console / multi-tenant control plane** — surface #3+; needs the operator stable first.
- **Open-sourcing now** — "test against public repos" is a conformance strategy, not distribution.
- **Bring-your-own-build hermeticity in v1** — hermetic-build claims require a hermetic builder;
  non-buck2 repos get the analysis/ratchet tier only. Honest tiering beats a false promise.
- **Big-bang migration of all 18 gates** — ratchet it: new gates must be pack-shaped (enforced by
  the automation-ratchet gate), existing gates migrate as touched.
- **Non-GitHub forges / non-git VCS in v1** — adapter seam reserved per ADR-0510 discipline.

## Open Questions

- Product name (working: cloud-ci; the operator + engine likely deserve a non-internal name).
- Where packs live: `registry/` vs dedicated `packs/` tree vs OCI artifacts (Gatekeeper precedent
  says versioned bundles; OPA bundle model maps to OCI).
- OSS posture timing (kernel-open/doctrine-commercial was deliberately NOT chosen yet).
