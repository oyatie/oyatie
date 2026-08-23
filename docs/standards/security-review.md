---
purpose: "Cross-cutting security-review standard. Names the OWASP control surfaces oyatie inherits, the supply-chain triad (`cargo-deny` + `cargo-audit` + `cargo-vet`), Sigstore signing + SBOM emission."
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Cross-cutting security-review standard. Names the OWASP control surfaces oyatie
  inherits, the supply-chain triad (`cargo-deny` + `cargo-audit` + `cargo-vet`),
  Sigstore signing + SBOM emission, the per-change-class threat-modeling
  obligation, the data-class boundary enforcement (per `data-class.md`), and the
  autonomy-ceiling guardrails (per `autonomy-ceiling.md`).
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
enforced_by: governance-supply-chain
companion_docs:
  - docs/security-program/security-program.json
  - docs/standards/data-class.md
  - docs/standards/autonomy-ceiling.md
  - docs/standards/dependency-policy.md
  - docs/standards/image-discipline.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# Security Review

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

This standard governs every PR that touches an auth, secret, payment,
privacy, capability, or supply-chain surface. The change-class reviewer
agents (`security-reviewer`, `privacy-reviewer`, `capability-reviewer`)
operate against this standard; the gate is named in
[`docs/AGENTS.md`](../AGENTS.md) §Per-change-class reviewer agents.

The program-level scope is in [`docs/security-program/security-program.json`](../security-program/security-program.json);
this standard supplies the per-PR review recipe.

## 1. OWASP control surfaces inherited

Oyatie inherits the OWASP Top 10 (Web, 2024) and the OWASP ASVS v4.0.3 control
catalog for every public surface in [`docs/SPEC.md`](../SPEC.md). The
[`compliance-matrix-coverage`](../DOC-CATALOG.md) lane validates
control-by-control mapping.

Per-PR reviewer-agent checklist (security-class changes):

1. **A01 Broken Access Control** — every public endpoint has a Cedar policy
   binding and a per-tenant authorization check; no `if user.is_admin` magic.
2. **A02 Cryptographic Failures** — only Ring / Rustls / RustCrypto crates
   approved by `dependency-policy.md`; no homegrown crypto; key material via
   `platform-secrets-kernel` + OpenBao.
3. **A03 Injection** — typed query builders (`sqlx`, `sea-orm`); no
   `format!` into SQL or shell.
4. **A04 Insecure Design** — every new capability has a threat model (§3).
5. **A05 Security Misconfiguration** — distroless base (per
   `image-discipline.md`); no debug endpoints exposed; secrets via OpenBao.
6. **A06 Vulnerable Components** — `cargo-deny` + `cargo-audit` + `cargo-vet`
   triad (§2); SBOM emission per artifact.
7. **A07 Identification & Authentication Failures** — federated identity via
   the platform identity surface; no per-app password storage.
8. **A08 Software & Data Integrity Failures** — Sigstore-signed artifacts;
   SLSA L2+ provenance attestation; admission control via policy-controller.
9. **A09 Logging & Monitoring Failures** — OTel emission per
   [`observability.md`](observability.md); audit-chain emission per
   [`data-class.md`](data-class.md).
10. **A10 SSRF** — outbound HTTP via vetted clients with explicit allow-list
    of upstream hostnames; provider SDKs go through `ProviderAdapter`.

## 2. Supply-chain triad

| Tool | Owner | Scope | CI lane |
|---|---|---|---|
| `cargo-audit` ([RustSec](https://rustsec.org/)) | Rust Secure Code WG | RustSec advisory DB scan | `governance-cargo-audit` |
| `cargo-deny` ([Embark](https://embarkstudios.github.io/cargo-deny/)) | Embark | license + advisory + source + duplicate-version | `governance-license` |
| `cargo-vet` ([Mozilla](https://mozilla.github.io/cargo-vet/)) | Mozilla | human-audit trail for third-party crates | `governance-cargo-vet` |
| `cargo-auditable` | Rust Secure Code | SBOM embedded in binary | release pipeline |

Rules:

1. All three (cargo-audit + cargo-deny + cargo-vet) MUST pass on every PR.
2. The `deny.toml` allow-list is the authoritative license posture; the
   forbidden tiers (AGPL / GPL / SSPL / BUSL / RSAL) are enumerated per
   [`forbidden-operations.json`](../../specs/forbidden-operations.json) FO-09.
3. `cargo-vet` audits live under `supply-chain/audits.toml`; share-points
   for AWS / Mozilla-published audits are configured.
4. New crate dependencies require a `cargo-vet` certification row OR an
   ADR-tracked exemption.

Source: [`.omc/scratch/hyperscaler-best-practices-2026-05-12.md`](../../.omc/scratch/hyperscaler-best-practices-2026-05-12.md)
Domain 4 "Supply-chain: signing + SBOM + provenance".

## 3. Threat modeling per change class

| Change class | Threat-model artifact | Reviewer |
|---|---|---|
| New capability (T2 / T3 / T4) | STRIDE table in capability record; explicit data-class flows | `capability-reviewer` |
| Auth / identity / session surface | STRIDE + token-lifecycle diagram | `security-reviewer` |
| Cross-axis contract | Data-class transition matrix; cross-tenant probe coverage | `security-reviewer` + `privacy-reviewer` |
| New external integration (provider SDK, webhook, OAuth) | SSRF + secret-exfiltration risk row in RISK-REGISTER | `security-reviewer` |
| Schema migration (PII / PHI fields) | DSR-cascade impact (per `data-class.md` §5) | `privacy-reviewer` |
| Privilege-tier uplift (T1→T2 / T2→T3 / T3→T4) | Cedar policy diff + runtime gate diff | `capability-reviewer` + Council-Privacy |

The threat-model artifact lives **in the same PR** as the change. The
reviewer-agent verdict (APPROVE / REQUEST CHANGES) is pasted to
`## Code Review` per [`docs/AGENTS.md`](../AGENTS.md) §PR shape.

## 4. Sigstore signing + SBOM emission

Per [`image-discipline.md`](image-discipline.md) and
[`release-management.md`](release-management.md):

1. Every CI artifact (binary, container image, crate publish) is signed via
   **Cosign keyless OIDC** against [Fulcio](https://www.sigstore.dev/) and
   logged in [Rekor](https://docs.sigstore.dev/logging/overview/).
2. An SBOM is generated via **Syft** (CycloneDX format) and attested
   alongside the artifact.
3. SLSA Level 2 provenance is emitted per
   [SLSA Provenance v0.1](https://slsa.dev/spec/v0.1/provenance).
4. Cluster-side admission control (Kyverno / policy-controller) verifies
   the signature + provenance before scheduling.
5. Cosign pin: **≥ v3.0.6** (per
   [`.omc/scratch/lts-versions-verified-2026-05-12.md`](../../.omc/scratch/lts-versions-verified-2026-05-12.md));
   the v3 `--bundle` contract is mandatory.

Lane: `governance-supply-chain` validates signed + SBOM-attached +
provenance-attested on every artifact emit.

Sources: [Wiz — SLSA Framework](https://www.wiz.io/academy/application-security/slsa-framework),
[Chainguard — Sign SBOM with Cosign](https://edu.chainguard.dev/open-source/sigstore/cosign/how-to-sign-an-sbom-with-cosign/),
[InfoQ — Provenance Tools Standard](https://www.infoq.com/news/2025/08/provenance/).

## 5. Data-class boundary enforcement

Every cross-pillar flow is gated by data-class checks per
[`data-class.md`](data-class.md). Security review specifically verifies:

1. Every new kernel-struct field declares `oyatie.data_class`.
2. Every cross-axis flow respects the transition matrix (e.g., PHI MUST NOT
   flow into Search/Ads pillars; see `privacy-class-taxonomy-coverage` in
   DOC-CATALOG.md §4).
3. DSR-cascade hooks are wired for any new PII/PHI surface.

## 6. Autonomy-ceiling guardrails

Every capability binding declares T1 / T2 / T3 / T4 per
[`autonomy-ceiling.md`](autonomy-ceiling.md). Security review verifies:

1. Tier declaration is present in the capability record.
2. Tier uplift carries a Cedar policy + runtime gate (NEVER a config flag).
3. T3/T4 capabilities have an out-of-band human-approval surface.
4. The audit-chain emission carries the tier, the approver, and the
   trace context.

## 7. Secret handling

- All secrets MUST be retrieved via the `SecretProvider` trait
  (`platform-secrets-kernel`); never `std::env::var` for secret values
  in product code.
- OpenBao is the primary store (per
  [`dependency-policy.md`](dependency-policy.md) §5).
- Secret rotation: ≤ 90 days for symmetric keys; ≤ 30 days for service
  account tokens. The lane `governance-secret-rotation` opens an
  issue when a stored secret exceeds its TTL.
- No secrets in logs, traces, or audit-chain payloads; the
  `silent-failure-hunter` reviewer agent + `gitleaks` + `trufflehog` scan
  every PR.

## 8. Container & image hardening

Per [`image-discipline.md`](image-discipline.md): distroless base, no
shells, no package managers, no `latest` tags, digest pinning at release.
Image-level vulnerability scan via **Trivy ≥ v0.70.0** (v0.69.4 forbidden
per the 2026-03-19 supply-chain incident).

## 9. Risk acceptance protocol

When a finding cannot be fixed in the PR:

1. File a row in `docs/RISK-REGISTER.md` with severity, owner, target date.
2. Add a row to `MISTAKES-LEDGER.md` only if a mechanical prevention is
   shippable.
3. Council-Architecture + Council-Privacy sign the risk acceptance if it
   crosses the catastrophic line.

## 10. Anti-patterns

1. **Disabling `cargo-deny` for a "convenience" crate** — refused.
2. **Inlining secrets in test fixtures** — use ephemeral fakes.
3. **Bypassing the autonomy ceiling with a config flag** — refused.
4. **Skipping SBOM emission for a "one-off" binary** — every artifact
   has provenance.
5. **Skipping `cargo-vet` certification for a new crate** — file the row
   or an ADR-tracked extension.

## 11. Sources scanned

- [`docs/security-program/security-program.json`](../security-program/security-program.json) (program scope).
- [OWASP Top 10 (2024)](https://owasp.org/Top10/), [OWASP ASVS v4.0.3](https://owasp.org/www-project-application-security-verification-standard/).
- [RustSec](https://rustsec.org/), [cargo-deny config](https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html).
- [Mozilla — cargo-vet](https://mozilla.github.io/cargo-vet/).
- [Sigstore](https://www.sigstore.dev/), [SLSA](https://slsa.dev/),
  [Syft](https://github.com/anchore/syft).
- [Kyverno](https://kyverno.io/), [policy-controller](https://github.com/sigstore/policy-controller).
- [`.omc/scratch/hyperscaler-best-practices-2026-05-12.md`](../../.omc/scratch/hyperscaler-best-practices-2026-05-12.md)
  Domain 4 "Supply-chain".
