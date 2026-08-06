---
id: ADR-0039
status: Accepted
doc_status: published
---

# ADR-0039: Supply chain security — Trivy 4-layer scan, Cosign keyless signing, SBOM dual-format, signed commits and tags, Kyverno admission

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0036, ADR-0037, ADR-0038, ADR-0040, ADR-0041, ADR-0043, ADR-0050

---

## Context

The supply chain is the single most-exploited attack surface in enterprise software (SolarWinds, Codecov, log4shell, xz-utils backdoor). For Oyatie — which ships all microservices, dozens of vertical packs, and a third-party plugin marketplace — supply-chain integrity is not a feature but a precondition. The pack-of-19 foundation ADRs named supply chain as a concern but did not pin the scanner topology, the signing chain, the SBOM dual-format, the commit-signing requirement, or the admission policy.

This ADR pins the discipline so that every artifact landing in production carries verifiable provenance, every dep is scanned at multiple layers, every commit is cryptographically attributable, and every cluster admission decision can refuse unsigned artifacts.

---

## Decision

We adopt **Trivy 4-layer scanning** (filesystem + container + IaC + dep) on every PR + nightly; **Cosign keyless signing** for every release artifact; **Rekor transparency log** for signature inclusion; **SBOM in SPDX 2.3 + CycloneDX 1.5** per artifact; **signed commits and tags** repo-wide; **merge-governance ruleset** at the GitHub level; **Kyverno (or OPA-equivalent) admission policy** at every cluster; and a dedicated CI lane `oya-governance-supply-chain` that gates merge.

### Trivy 4-layer scan

```yaml
# .github/workflows/oya-governance-supply-chain.yml (canonical CI lane)
- name: Trivy filesystem scan
  run: trivy fs --severity HIGH,CRITICAL --exit-code 1 .

- name: Trivy container scan
  run: |
    for img in $(yq '.images[]' registry/release/images.yaml); do
      trivy image --severity HIGH,CRITICAL --exit-code 1 "$img"
    done

- name: Trivy IaC scan
  run: trivy config --severity HIGH,CRITICAL --exit-code 1 infra/

- name: Trivy dep / SBOM scan
  run: trivy fs --scanners vuln,secret,license --format sarif --output trivy.sarif .
```

| Layer | Scope | Severity gate |
|---|---|---|
| **Filesystem** | Working tree | HIGH/CRITICAL = fail |
| **Container** | Every release image (per ADR-0028) | HIGH/CRITICAL = fail |
| **IaC** | Terraform / Helm / Kubernetes manifests | HIGH/CRITICAL = fail |
| **Dep / SBOM** | Cargo / npm / pip / Go mod | HIGH/CRITICAL = fail; license-policy violations = fail |

Cadence: every PR + nightly main + weekly full-history rescan.

### Cosign keyless signing

```bash
# crates/oya-intelligence-release/scripts/sign.sh
COSIGN_EXPERIMENTAL=1 cosign sign \
  --identity-token "${OIDC_TOKEN}" \
  --output-signature "${ARTIFACT}.sig" \
  --output-certificate "${ARTIFACT}.cert" \
  "${ARTIFACT}"
```

- **Keyless.** OIDC identity from GitHub Actions / per-cell CI runner; no long-lived signing keys to leak.
- **Sigstore Fulcio** issues short-lived certificates per signing event.
- **Rekor transparency log** records every signature; `cosign verify --rekor-url <url>` confirms inclusion.
- Every release artifact (binary, container image, WASM plugin, IaC bundle, doc bundle) is signed.

### SBOM dual-format

Per artifact:

- **SPDX 2.3** (industry-standard; required for US Executive Order 14028 compliance equivalents).
- **CycloneDX 1.5** (richer dependency relationship model; preferred for vulnerability correlation).

SBOMs are signed via Cosign and attached to the artifact (`cosign attest`). Per-tenant trust portal (ADR-0038) exposes SBOMs for tenant-deployed artifacts.

### Signed commits + signed tags

Repo-wide enforcement:

- **Commit signing.** SSH-key-signing or GPG-signing required; unsigned commits rejected by branch-protection.
- **Tag signing.** All release tags signed; release pipeline verifies tag signature before publishing.
- **Per-author identity verification.** GitHub-level identity tied to per-author key set in `.github/CODEOWNERS-equivalent` registry.

### Merge-governance ruleset

GitHub branch-protection ruleset (codified in `.github/branch-protection.yaml`):

- Require PR before merge.
- Require status checks: `oya-governance-supply-chain`, `oya-governance-cohesion` (per ADR-0001), `oya-governance-api-semver` (per ADR-0037), per-microservice fitness lanes.
- Require signed commits.
- Require linear history (squash or rebase merge only).
- Require ≥ 1 reviewer (≥ 2 for substrate kernel changes per ADR-0001).
- Disallow force-push to main.
- Disallow branch deletion of main + release branches.

### Admission policy via Kyverno

```yaml
# infra/kyverno/policies/require-signed-images.yaml
apiVersion: kyverno.io/v1
kind: ClusterPolicy
metadata:
  name: require-signed-images
spec:
  validationFailureAction: Enforce
  rules:
  - name: verify-cosign-signature
    match:
      any:
      - resources:
          kinds: ["Pod"]
    verifyImages:
    - imageReferences:
      - "registry.oya.run/*"
      attestors:
      - entries:
        - keyless:
            issuer: "https://token.actions.githubusercontent.com"
            subject: "https://github.com/<org>/<repo>/.github/workflows/release.yml@refs/tags/v*"
```

- Every cluster (per-cell, per ADR-0028) runs Kyverno.
- Every Pod admission verifies (a) image signature, (b) Rekor transparency log inclusion, (c) image SBOM presence, (d) image not in known-CVE quarantine.
- Admission denial is audit-chained per ADR-0003.

### CI lane: `oya-governance-supply-chain`

Aggregates:

- Trivy 4-layer scan (above).
- Cosign signature presence on outgoing artifacts.
- SBOM dual-format presence + signature.
- Signed-commit verification on PR commits.
- License-policy compliance (per Product License Policy ADR; reject SSPL/AGPL outside legal isolation).
- Per-PR provenance attestation (SLSA Build L3 target).

### SLSA alignment

Target: **SLSA Build L3**.

- **Source.** Signed commits + version-controlled.
- **Build.** Hermetic + parameterless + per-build provenance.
- **Provenance.** Cosign-signed attestations covering build steps + inputs + outputs.
- **Distribution.** Rekor-logged signatures on every distributed artifact.

### Anti-scope

This ADR does not own the per-cell HSM partition (per ADR-0043). Does not own per-plugin signing (per ADR-0036, but ADR-0036 references this signing chain). Does not own runtime intrusion detection (per ADR-0042 observability stack).

### REGSEC-001 planning-contract governed surface

PR #1136 adds `specs/vulnerability-intelligence-sbom-vex-pipeline.json` as the planning-only vulnerability-intelligence, SBOM, VEX, prioritization, exception, evidence, and admission-decision contract for this ADR's supply-chain security posture. This exact path is governed by ADR-0039 for accounting-registration purposes only; it does not promote scanner CLI output, a production ingestion service, a live admission webhook, or tenant readiness/certification authority.

---

## Consequences

### Positive

- Every artifact landing in production is verifiable end-to-end: source → build → distribution → admission.
- Keyless signing eliminates the long-lived signing key as a leak target.
- SBOM dual-format satisfies both US EO 14028-equivalent and tooling-rich compliance audiences.
- Kyverno admission means a forged or unsigned artifact cannot run, even if it bypasses the registry.
- The 4-layer Trivy posture catches dep CVEs, container CVEs, IaC misconfigurations, and filesystem secrets all in one PR-time check.

### Negative

- Cosign keyless requires functional Sigstore + Rekor infrastructure; outage of either blocks releases. We mirror Rekor for verification but cannot mirror signing.
- SBOM generation has overhead at build time; large monorepo SBOMs are large files (multi-MB per artifact).
- Signed-commit enforcement excludes contributors without GPG/SSH-key setup — onboarding friction.
- Trivy 4-layer at every PR adds CI latency.

### Operational

- Per-cell Kyverno admission alarms wired to on-call.
- Cosign + Rekor outage runbook.
- Per-quarter SLSA Build L3 attestation review.
- Per-PR supply-chain lane status visible in PR UI.
- Per-month CVE quarantine review; quarantined images cannot be promoted to GA.
- Per-author key rotation cadence: annual for production-write authors; per-departure revocation.

---

## Alternatives considered

### Alternative A — Cosign with long-lived keys (not keyless)

- **Pros:** simpler offline signing; no Sigstore dependency.
- **Cons:** key leak is catastrophic; key rotation is manual.
- **Rejected because:** keyless is the durable posture; Sigstore mirroring covers the dep risk.

### Alternative B — Single SBOM format (SPDX only or CycloneDX only)

- **Pros:** less tooling.
- **Cons:** loses one audience (SPDX is standard for compliance; CycloneDX is preferred for dep tooling).
- **Rejected because:** dual-format is one extra build step but covers both audiences.

### Alternative C — OPA Gatekeeper instead of Kyverno

- **Pros:** also a credible admission controller.
- **Cons:** Rego is less ergonomic than Kyverno's YAML for the admission-policy use case; team familiarity matters.
- **Rejected because:** Kyverno covers the use case with less complexity.

### Alternative D — Nightly scan only (no per-PR)

- **Pros:** lower CI cost.
- **Cons:** vulnerability lands in main, gets reviewed N hours later; the failure mode this ADR exists to prevent.
- **Rejected because:** PR-time gating is the moat.

---

## Open questions

1. **Q1.** Per-cell private Rekor mirror at Phase 2 or Phase 3? Default: Phase 2 (KR colo); reduces public-Sigstore dependency for KR sovereignty. → ADR-0028.
2. **Q2.** SLSA Build L4 target — at GA or W+24? Default: L3 at GA; L4 at W+24 if commercial buyers require. → owner: `foundry`.
3. **Q3.** Per-PR Trivy budget — fail at HIGH/CRITICAL or also MEDIUM? Default: HIGH/CRITICAL; MEDIUM advisory only. → owner: `foundry`.
4. **Q4.** Signed-commit enforcement — SSH or GPG primary? Default: SSH (lower friction, GitHub native). → owner: `foundry`.
5. **Q5.** SBOM exposure on trust portal — per artifact or per axis aggregate? Default: per artifact downloadable; per-microservice aggregate dashboard. → ADR-0038.

---

## References

- `docs/PRD.md` §10 (security program)
- `docs/DESIGN.md` §11 (supply chain), §10 (cross-microservice contracts)
- US Executive Order 14028; CISA Secure Software Self-Attestation
- KR 「소프트웨어 진흥법」, KISA 시큐어코딩 가이드
- SLSA framework v1.0; SPDX 2.3; CycloneDX 1.5; SARIF 2.1.0
- Sigstore Cosign + Fulcio + Rekor specs
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0036 (plugin substrate), ADR-0037 (API stability), ADR-0038 (trust portal), ADR-0040 (progressive delivery), ADR-0041 (gitops), ADR-0043 (HSM + KMS), ADR-0050 (automation pipeline)
