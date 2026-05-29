---
doc_class: FailureModes
title: "Failure Modes Catalog"
microservice: developer-sdk
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Failure Modes Catalog


## FM-01 — Cedar evaluator outage

**Symptom**: All install requests fail with 503.
**Cause**: governance µservice Cedar evaluator unreachable.
**Mitigation**: Circuit breaker; 503 with Retry-After; on-call paged.

## FM-02 — Postgres replica lag > 5s

**Symptom**: Stale catalog data; recent installs invisible.
**Cause**: Replication lag spike.
**Mitigation**: Cilium routes traffic to primary; alert on lag > 5s.

## FM-03 — Valkey Sentinel split-brain

**Symptom**: Rate-limit bucket inconsistency.
**Cause**: Network partition.
**Mitigation**: Halt all writes; fail-closed on bucket reads; on-call.

## FM-04 — Wasmtime engine pool exhaustion

**Symptom**: Plugin install latency degrades; new installations queued.
**Cause**: Idle teardown not aggressive enough; sustained high install rate.
**Mitigation**: Scale engine pool; tune idle teardown to 30s; alert.

## FM-05 — Cosign verifier shell timeout

**Symptom**: Vetting pipeline stuck at signature-verification stage.
**Cause**: Cosign binary hangs (rare; usually DNS for rekor).
**Mitigation**: 30s timeout; fail vetting; alert + retry.

## FM-06 — Trivy DB stale

**Symptom**: Trivy passes a known CVE.
**Cause**: DB > 7d old.
**Mitigation**: Daily DB refresh job; alert on staleness; force-refresh runbook.

## FM-07 — OpenBao seal

**Symptom**: Signing key issuance fails.
**Cause**: OpenBao restarted without auto-unseal.
**Mitigation**: cloud-secrets µservice handoff for auto-unseal; alert.

## FM-08 — Bank rail rejection

**Symptom**: Payout settlement fails for a subset of developers.
**Cause**: Bank-side validation (e.g., closed account, name mismatch).
**Mitigation**: Defer payout; flag developer for re-verification.

## FM-09 — KYC false-positive

**Symptom**: Legitimate developer rejected at KYC.
**Cause**: OFAC name collision; liveness model false-negative.
**Mitigation**: Manual review queue; appeal path.

## FM-10 — Audit chain seal lost

**Symptom**: Audit chain integrity fails on daily verification.
**Cause**: audit-chain µservice outage during seal emission.
**Mitigation**: Local buffer; replay on recovery; chain-integrity gate BLOCKER.

