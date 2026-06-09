---
doc_status: published
---

# Checklist: Wave-gate passing

> **When:** When a wave-tactical team believes the wave's exit criteria are met. The gate is BLOCKING; no later wave starts until this passes.
> **Owner:** Per-wave tactical team (e.g. `tactical-first-vertical-pilot` for W-Vertical-Pilot) + architecture-governance co-sign.
> **Validator:** protected `oya-ci-required` status + per-wave exit-criteria evidence pack

---

## Universal gate (every wave)

1. ☐ **All exit-criteria items checked** — per-wave criteria from [ROADMAP.md §2](../ROADMAP.md) for this wave.
2. ☐ **Cross-axis contract integrity** — `oya-governance-cohesion` reports zero violations for the wave's surfaces.
3. ☐ **Audit-chain integrity** — chain-replay drill passed in last 30 days for affected axes.
4. ☐ **DSR cascade tested** — at least one synthetic DSR cascaded across all wave surfaces; proof-of-erasure verified.
5. ☐ **License gate** — no AGPL/GPL/SSPL/BUSL in any product crate touched by the wave.
6. ☐ **No open BLOCKER contradictions** — [CONTRADICTION-LEDGER.md](../CONTRADICTION-LEDGER.md) shows zero open BLOCKERs that affect this wave.
7. ☐ **Foundation-bypass freshness** — every active bypass affecting wave surfaces is within its expiry window.
8. ☐ **Risk-register review** — every risk affecting wave surfaces has either a Mitigated or Accepted status.
9. ☐ **Evidence pack regenerated** — per-wave evidence pack generated within 7 days, per [COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md) cadence.
10. ☐ **Per-affected-team sign-off** — every team listed in wave's `responsible_teams:` has approved.
11. ☐ **Per-pack regulator-watch** — for every regional pack the wave activates, regulator-change feed checked in last 14 days.
12. ☐ **SLO baseline** — per-surface SLOs in [SLO-CATALOG.md](../SLO-CATALOG.md) updated; burn-rate within budget for the wave's surfaces.
13. ☐ **Runbook coverage** — every alert in the wave's surfaces has a runbook in [RUNBOOKS-INDEX.md](../RUNBOOKS-INDEX.md).
14. ☐ **Threat-model refresh** — per-service threat models updated within the quarter for affected surfaces.
15. ☐ **DR drill** — quarterly DR drill passed for affected surfaces.

## Wave-specific (extends universal)

| Wave | Additional gate items |
|---|---|
| W-Foundation | Data Use Boundary ADR Accepted; License Policy ADR Accepted; Build-vs-Buy ADR Accepted; Cell architecture ADR Accepted; Regional-pack architecture ADR Accepted; tenant kernel + identity + audit chain + Cedar policy + plane separation + Ontology property tiers (legacy: Object Graph) all live |
| W-Intelligence-Preview | SecretProvider live + KMS HSM (Korean cryptographic module validation for KR) live; all 6 provider adapters (Anthropic / OpenAI / Gemini × api / subscription) operational; capability registry online with ≥ N capabilities; autonomy ceiling enforcement live; evidence-chain emission per capability; ≥ 1 live pilot capability end-to-end |
| W-Cloud-Preview | ≥ 2 regional packs onboarded; cell-isolation evidence; cloud control-plane API frozen at v1; CSAP / equivalent regulator-watch active per pack |
| W-SaaS-Preview | Workflow engine live; Ontology properties (vector/timeseries/geo/ciphertext/struct; legacy: Object Graph) Accepted ADRs implemented; plugin substrate trust gates live; public REST stability tier declared |
| W-Workspace-Preview | Mail deliverability ≥ 99% across ≥ 2 regions; Doc edit-prop p99 < 200ms intra-region; Drive sync conflict < 0.5%; ≥ 1 migration from Google/M365/Naver Works completed end-to-end |
| W-Search-Preview | pgroonga day-1 + KR/JP/EN morphology; per-tenant private indexes operational; RAG endpoint exposed to Intelligence; per-class data boundary enforcement |
| W-Vertical-Pilot | One vertical end-to-end on Foundation+axes preview; design-partner tenant live; ≥ 6 cross-axis contracts exercised |
| W-Vertical-Fan-Out | All 14 verticals running with regulatory-pack adoption + control evidence |
| W-Cloud-Stable | Public cloud GA; CSAP / ISMAP / FedRAMP / GAIA-X / etc. evidence; SLA committed (99.99% control plane) |
| W-Search-Stable | Public web search + crawler + freshness + KG + SERP; sponsored-result infra ready (ad serving still off) |
| W-Ads-Preview | Internal-tenant ads only; Data Use Boundary ADR satisfied; per-tenant auction quality ≥ baseline |
| W-Ads-Stable | External advertisers; cross-tenant aggregate consent flows; KR adtech compliance evidence |
| W-AI-Model-Substrate | GPU fleet provisioned; ≥ 1 in-house model trained + outperforms or cost-matches external on its eval set |
| W-Region-Fan-Out | Per-region regulator-equivalent + residency contracts; per-pack identity / payment / tax / language pack live |

## After passing

- ☐ Emit `EVT-WAVE-GATE-PASSED` (per [DOC-CATALOG.md §1](../DOC-CATALOG.md))
- ☐ Update [ROADMAP.md](../ROADMAP.md) to mark wave passed; next wave's start condition met
- ☐ Update [trust portal](https://trust.oyatie.com) per affected packs
- ☐ Annual report row added if commercial-impact wave
