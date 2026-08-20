---
doc_class: Runbook
title: Replay-Divergence Investigation
microservice: foundry-eval
severity: "Sev-2 (replay divergence ≥ 100ms) / Sev-1 (replay divergence ≥ 100ms persistent on critical capability)"
status: Accepted
owner_team: axis-foundry + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-04 replay divergence)
  - microservices/intelligence/incident-response.md
  - ADR-0024 §"Replay against past traces"
doc_status: published
---

# Runbook: Replay-Divergence Investigation

## Trigger

`oya_foundry_eval_replay_divergence_ms{capability="<cap>"} > 100` p99 observed in:
- Pre-promote replay (model-upgrade gate): the upgrade is held; on-call investigates.
- Nightly replay sample (drift detection): on-call alarmed.

## Severity

- Critical capability (tier-A; tenant-impactful): **Sev-1**.
- Non-critical capability: **Sev-2**.

## Pre-checks

1. Confirm the replay-engine worker is alive: `oya_foundry_eval_replay_engine_alive{pack="<pack>"} == 1` for ≥ 5 min.
2. Confirm the replay-trace store is reachable: S3 PUT/GET succeed; per-subject DEK availability.
3. Confirm the candidate route is reachable (provider model API or in-house variant).

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>`; assign IC; declare severity | ≤ 5 min |
| 2 | Pre-checks above | ≤ 3 min |
| 3 | Identify the divergent cohort: `oya-intelligence-eval-replay-engine-rest divergence --capability <cap> --top 10 --order divergence_ms desc` lists top divergent replay-samples | ≤ 5 min |
| 4 | Categorise: (a) non-deterministic case (seed not present / seed value differs); (b) provider response drift (provider model changed); (c) tokeniser drift; (d) capability prompt template changed; (e) baseline-output mismatch | ≤ 15 min |
| 5 | For (a) — non-deterministic case: tag for exclusion from determinism cohort + filed as eval-set authoring gap | per category |
| 6 | For (b) — provider drift: open provider's release-notes; if breaking, hold model upgrade + notify capability owner | per category |
| 7 | For (c) — tokeniser drift: confirm tokeniser version pin; if violated, revert to prior tokeniser version | per category |
| 8 | For (d) — prompt template changed: check capability registry version pin; if mismatch, revert capability to prior version | per category |
| 9 | For (e) — baseline mismatch: re-verify baseline Cosign signature; if signature valid, re-evaluate baseline (may need refresh) | per category |
| 10 | Re-run replay against affected cohort: `oya-intelligence-eval-replay-engine-rest replay --capability <cap> --cohort <cohort-id>` | ≤ 30 min |
| 11 | If divergence persists: escalate to ExecSponsor + axis-foundry weekly | — |
| 12 | If divergence resolves: emit `ReplayDivergenceResolved` event; release the held model-upgrade gate | ≤ 5 min |
| 13 | Postmortem within 5 business days for Sev-1 cases | — |

## Special-case: deterministic-seed cohort failure

If divergence occurs on deterministic-seed cases (where seed is present + value matches), this is a structural failure of replay-engine determinism — likely:
- Provider non-determinism leaking through despite seed setting.
- Sandbox CUDA context bleed (gVisor / Kata escape investigation needed).
- Replay-engine adapter-s3 race condition.

Engage ops-security if sandbox escape suspected. Trace under threat-model T-E-01.

## Verification

After completion:
- `oya_foundry_eval_replay_divergence_ms{capability="<cap>"} <= 100` p99 sustained ≥ 2 nightly cycles.
- Affected replay-traces re-replayed clean.
- `ReplayDivergenceResolved` event in audit-chain.
- Held model-upgrade gates released (if any).

## References

- ADR-0024 §"Replay against past traces".
- `microservices/intelligence/failure-modes.md` FM-04.
- `microservices/intelligence/threat-model.md` T-E-01 (sandbox escape).
- `microservices/intelligence/PRD.md` §"Performance Targets" (replay determinism ≤ 100ms).
