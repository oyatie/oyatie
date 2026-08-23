---
purpose: Oyatie — Risk Register
doc_status: published
---

# Oyatie — Risk Register

> **Status:** Draft v0.1 — 2026-05-09. Aggregates project-level risks from PRD §7, DESIGN §11, PRIVACY-PROGRAM, the 77 contradictions found by the rename+contradiction agent, and the 9 recon-file findings.
> **Owner:** `council-architecture`. Updates per [DOC-CATALOG.md `doc.risk_register`](DOC-CATALOG.md) (weekly cadence + per EVT-RISK-MATERIALIZED / EVT-INCIDENT-CLOSED / EVT-AUDIT-FINDING).
> **Companion:** [`machine-readable/risks.json`](machine-readable/risks.json) for agent consumption.

---

## 1. Severity × Likelihood × Velocity scoring

We score each risk on three dimensions:

- **Severity**: 1 (annoying) → 5 (existential / unrecoverable failure mode)
- **Likelihood**: 1 (rare) → 5 (probable absent active mitigation)
- **Velocity**: 1 (slow-moving, weeks of warning) → 5 (sudden, no warning)

**Risk score** = Severity × Likelihood × Velocity. Risks ≥ 50 are tracked weekly in council; ≥ 25 monthly; < 25 quarterly.

---

## 2. Top risks (sorted by score, then severity)

### 2.1 Catastrophic + structural (score ≥ 75)

| # | Risk | Sev | Like | Vel | Score | Owner | Mitigation | Monitoring |
|---|---|---|---|---|---|---|---|---|
| R-001 | Tenant data leak into search index or as ad-targeting feature (PHI / PII / PCI / KR-신용정보 / KR-PIPA Art-23) | 5 | 4 | 5 | **100** | `council-privacy` | Data Use Boundary ADR (P0 prereq); 6-layer structural enforcement per [PRIVACY-PROGRAM §2.2.4](PRIVACY-PROGRAM.md); per-class compile-time annotation; singleton `platform-ads-gate` source; runtime guard at auction boundary | Audit-chain emission rate; rejected cross-axis call alerts; quarterly re-identification red team |
| R-002 | Cross-axis contract drift (an axis evolves a contract another axis has frozen) | 5 | 4 | 4 | **80** | `council-architecture` | DESIGN §10 contract surface as audit point; cross-axis PR class label; CI fitness function checks every contract against every consumer per [DESIGN §11](DESIGN.md); cohesion fitness function (top-20 #13 from Foundry-improvements research) | PR class label compliance; fitness-function block rate |
| R-003 | Audit-chain emission gap (some regulated capability path bypasses ADR-0003 emission) | 5 | 3 | 5 | **75** | `platform-audit-evidence` | Mandatory emission per capability registration; CI gate `governance-audit-emit`; per-quarter chain-integrity check + replay drill | Emission completeness % per capability; chain-integrity check exit code |

### 2.2 High structural (score 50-74)

| # | Risk | Sev | Like | Vel | Score | Owner | Mitigation | Monitoring |
|---|---|---|---|---|---|---|---|---|
| R-004 | Foundry agent escapes autonomy ceiling for a regulated capability | 5 | 3 | 4 | **60** | `axis-foundry` + `council-privacy` | Cedar policy + runtime enforcement (top-20 #1 from Foundry-improvements); per-capability break-glass; automated revoke; M-of-N break-glass approval per [PRIVACY-PROGRAM §2.2.8](PRIVACY-PROGRAM.md) | Autonomy-tier breach alerts; break-glass invocation audit |
| R-005 | Regional pack regulatory drift (a regulator updates rule; pack out-of-date) | 4 | 4 | 3 | **48** (~50) | `regional-packs` + `ops-compliance` | Per-pack regulatory-change watch lane; quarterly per-pack regulatory refresh; ADR amendment per change | Regulatory-watch alert volume per region; pack refresh recency |
| R-006 | Architectural-flattening migration breaks `main` (per ADR-0015 plan PM-1/2/3/4) | 5 | 2 | 5 | **50** | `council-architecture` + `axis-foundry` (foundry surface) | Per-crate move PR shape (ADR-0015 plan §7); workspace-stays-green invariant; one-PR-at-a-time `members =` modification; `cargo check --all-features` + `cargo nextest --all-features` per move | Post-merge `main` build status; member-list integrity check |
| R-007 | Foundry preview slips beyond Foundation (force multiplier delayed) | 5 | 3 | 3 | **45** (~50) | `axis-foundry` | Foundry preview is sequenced *second* in the optimal-path waves; Foundation is the *only* gate; resource over-allocation if needed (no time/resource constraint per drawing-board re-framing) | Foundry preview gate criteria checklist completion % |

### 2.3 High operational (score 25-49)

| # | Risk | Sev | Like | Vel | Score | Owner | Mitigation | Monitoring |
|---|---|---|---|---|---|---|---|---|
| R-008 | Brand consolidation to Oyatie touches public APIs incorrectly | 4 | 3 | 3 | **36** | `axis-foundry` + `council-architecture` | Standalone PG-0a precursor PR per bounded context (ADR-0017); aliases retire in Phase 7 sweep; per-batch rename plan in 17 batches per recon | DNS / package / API consumer count regression |
| R-009 | License drift (an external dep changes from Apache-2 to AGPL/SSPL/BUSL) | 4 | 3 | 3 | **36** | `ops-security` + `axis-foundry` (foundry) | License policy ADR (planned P0); `governance-license` CI lane; per-quarter license audit; Vendor-Partner-Ledger | cargo-deny exit; license-watch alerts |
| R-010 | Multi-provider quota exhaustion (one provider rate-limits Foundry) | 3 | 4 | 3 | **36** | `axis-foundry` | Multi-provider router with cost-aware + latency-aware fallback (top-20 #15); per-tenant per-capability budget ceilings; provider equivalence map; subscription↔API failover | Provider error-rate dashboards; quota-exhaustion alerts |
| R-011 | Korea-locale regulatory shift mid-build (PIPA / FSC / MFDS / KCC) | 4 | 3 | 3 | **36** | `regional-packs/pack-kr` + `ops-compliance` | Quarterly regulatory-change watch lane per ADR-0050; KR-pack maintainer engages with regulator-relations | KR regulator publication monitoring |
| R-012 | Cell-isolation failure (one tenant's data accessible to another) | 5 | 2 | 4 | **40** | `axis-cloud` + `platform-tenancy-identity` | Cell architecture ADR (planned P0); per-cell isolation evidence; quarterly cross-tenant access fuzz test (#129) | Cross-tenant access negative tests; cell membership integrity |
| R-013 | Subscription-mode session breakage (Claude/OpenAI/Gemini change auth flow) | 3 | 4 | 3 | **36** | `axis-foundry` | Per-provider subscription contract tests (top-20 #16); subscription↔API failover; PTY warm pool; idle refresh | Per-provider subscription health probe |
| R-014 | KCMVP HSM procurement lead-time (6-9 months) blocks KR cloud GA | 4 | 3 | 2 | **24** (~25) | `axis-cloud` + `regional-packs/pack-kr` | Order at month 0 of W-Cloud-Preview per recon; secondary supplier identified | HSM delivery tracking |
| R-015 | Plugin substrate trust gate failure allows malicious plugin install | 5 | 2 | 3 | **30** | `axis-foundry` (foundry surface — plugin substrate) + `ops-security` | Cosign keyless signing per ADR-0039; Wasmtime sandbox per ADR-0023; plugin trust tiers per ADR-0036; sigstore + Rekor mirror (top-20 D-1) | Plugin signature verification audit; sandbox escape attempts |
| R-016 | Autonomy-ceiling implementation gap (Cedar policy authored but no runtime enforcement) | 5 | 2 | 3 | **30** | `axis-foundry` | Top-20 #1 — runtime gate, not docs; `intelligence-policy` runtime check on every capability invocation | Capability-invocation policy-check log |
| R-017 | Per-capability eval set regression on a model upgrade | 3 | 4 | 2 | **24** (~25) | `axis-foundry` | Per-capability eval contract (PRD-shaping #10); golden tasks + nightly run + regression gating | Per-capability eval pass rate trend |

### 2.4 Medium (score 10-24)

| # | Risk | Sev | Like | Vel | Score | Owner | Mitigation | Monitoring |
|---|---|---|---|---|---|---|---|---|
| R-018 | Vertical pilot fails to prove cohesion thesis | 4 | 3 | 2 | 24 | `tactical-first-vertical-pilot` | Pre-flight check that pilot exercises ≥ 6 cross-axis contracts; design-partner co-author of acceptance | Per-pilot acceptance test pass rate |
| R-019 | M3/MVP vocab leakage from legacy docs into new docs | 2 | 4 | 2 | 16 | `council-architecture` | `governance-glossary` checks for retired terms; banner in PRD §3.1; ADR-0050/0185/0191 explicitly amended | Grep for retired terms in PR diffs |
| R-020 | ADR backlog (71 Proposed) creates governance drift | 3 | 4 | 1 | 12 | `crew-adr-promotion` | Monthly burndown target; promotion validator; per-ADR Owner+Status check | Proposed:Accepted ratio per quarter |
| R-021 | Catalog projection drift (`goals.json` ↔ `batch-manifest.json`) | 2 | 4 | 2 | 16 | `axis-foundry` (foundry) | Autosync per Issue #1486; CI check | Drift-detection alerts |
| R-022 | Worktree leakage / branch-name collision | 3 | 3 | 2 | 18 | `axis-foundry` (foundry) | Worktree-isolation guardrails; branch-name collision detection at spawn (#58) | Per-spawn collision-check log |
| R-023 | Foreign data localization law shift (e.g., new India / Russia / KSA mandate) | 4 | 2 | 2 | 16 | `regional-packs` + `ops-compliance` | Per-pack residency declaration; cross-region replication opt-in only; legal watch | Per-region legal-watch reports |
| R-024 | OSS dependency vulnerability (CVE on a runtime dep) | 3 | 3 | 2 | 18 | `ops-security` | Trivy 4-layer per ADR-0039; weekly cargo-audit; auto-ChangeSet via the in-house deps bump-bot | RUSTSEC backlog; Trivy alerts |

### 2.5 Watching (score < 10)

| # | Risk | Sev | Like | Vel | Score | Owner | Mitigation |
|---|---|---|---|---|---|---|---|
| R-025 | Cosmetic doc drift (e.g., outdated diagrams) | 1 | 4 | 1 | 4 | per-doc owner | DOC-CATALOG cadence + agent-authored refresh |
| R-026 | Glossary term drift across docs | 2 | 3 | 1 | 6 | `council-architecture` | `governance-glossary` |
| R-027 | Internal CRM ↔ tenant catalog sync drift | 2 | 3 | 1 | 6 | `gtm-customer-success` | Monthly sync check |

---

## 3. Anti-risks (worth flagging)

These are *not* risks per se but are *common organizational pathologies* worth pre-emptively naming.

| # | Pathology | Why it matters | Counter-pattern |
|---|---|---|---|
| AR-1 | "Let's launch the M3 wave on schedule even if foundation isn't done" — date-driven scope reduction | The drawing-board re-framing explicitly retired this; without active resistance the org reverts to date-driven scope cuts | No date commitment in any wave gate; only state-of-correctness gate criteria |
| AR-2 | "Just one foundation bypass, we'll renew it later" → permanent bypasses | Foundation bypasses are tracked + expirable + ledgered, but cultural drift can normalize them | Quarterly bypass-expiry review at council; auto-revert PR after expiry |
| AR-3 | "Customer X is special, let's give them a one-off integration" → fork the architecture | Cohesion thesis depends on no per-customer forks | All customer-extension goes through plugin substrate or regional pack; no other path |
| AR-4 | "Privacy is a regulatory tax, let's minimize" | Privacy is the moat. Minimizing it cedes the moat | Privacy posture stricter than any single competitor; structural enforcement, not policy text |
| AR-5 | "Agents are unreliable; we'll keep humans in the loop forever" | Agents WILL be reliable; investing in Foundry early is the force multiplier | Foundry preview second after Foundation; agent-driven optimization loops baked into every axis |
| AR-6 | "Korea is hard; let's launch US first" | Korea-pack is one regional pack; US is another. Parallel onboarding. Korea has the design-partner gravity already | Canonical + regional-pack architecture per [DESIGN §12](DESIGN.md) — no locale special-case |

---

## 4. Risk-velocity dashboard (proposed)

A real-time risk dashboard tracks for each risk:
- Current score
- Trend (last quarter)
- Mitigation status (% complete)
- Last review date

Powered by `intelligence-risk-tracker` (a capability that aggregates audit-chain events + CI lane status + manual council inputs). Surfaced on `dev.oyatie.com/risks` for internal staff and on `trust.oyatie.com/risks` (filtered) for customers.

---

## 5. Per-axis risk slice

Each axis has its own slice of this register. See [`products/<axis>/PRD.md` §10](products/) for axis-specific risks. The cross-axis risks above (R-001 ... R-027) are aggregated here.

| Axis | Owning team | Slice review cadence |
|---|---|---|
| SaaS | `axis-saas` | weekly |
| Vertical | per-vertical | monthly |
| Foundry | `axis-foundry` | weekly |
| Cloud | `axis-cloud` | weekly |
| Search | `axis-search` | monthly |
| Ads + Analytics | `axis-ads-analytics` | monthly (escalates to weekly when ads-stable wave nears) |

---

## 6. Risk → Mitigation → Acceptance flow

Every risk goes through a 4-state lifecycle:

```
Identified → Mitigated → Tested → Accepted (or → Re-identified)
```

- **Identified**: someone names the risk and registers a row.
- **Mitigated**: a control is implemented; the row gets a `mitigation:` link.
- **Tested**: a verification proves the control works (red team, fuzz, audit).
- **Accepted**: the council ratifies the residual risk; row moves to the "Watching" tier.

If at any stage new evidence raises the score, the row goes back to Identified.

---

## 7. Sources scanned

- [`PRD.md`](PRD.md) §7 (top-10 risks)
- [`DESIGN.md`](DESIGN.md) §11 (cross-axis contradiction risks)
- [`PRIVACY-PROGRAM.md`](PRIVACY-PROGRAM.md) §2 (Data Use Boundary)
- `/Users/jasonlee/oyatie/docs/raw/rename-and-contradiction.md` (77 contradictions)
- `/Users/jasonlee/oyatie/docs/raw/foundry-improvements.md` (top-20 + 10 PRD-shaping)
- `/Users/jasonlee/oyatie/docs/raw/gap-issues.md` (387 gap issues — top-of-the-list highest-risk subset)
- ADR-0015 plan PM-1/2/3/4 pre-mortems
- ADR-0050 master plan (legacy milestones)
- `docs/MISTAKES-LEDGER.md`

*Footer regenerated whenever this doc is edited.*
