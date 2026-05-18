---
doc_class: FailureModeAnalysis
template_id: TPL-FMA
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + ops-sre-reliability
doc_status: published
---

# Failure mode analysis — slides µservice

| FM | Surface | Trigger | Impact | Detection | Mitigation | Recovery |
|---|---|---|---|---|---|---|
| FM-01 | editor-rest | Postgres primary down | save fails | health probe + SLO burn | DR failover; freeze writes; tenant banner | RTO 30s; RPO 5s |
| FM-02 | editor-rest | Redis cluster split-brain | CRDT lease confusion → silent loss risk | lease verification + Sev-1 alarm | force lease reacquire; reconcile from Postgres CRDT snapshot | 5min; data preserved |
| FM-03 | real-time-collaboration-worker | WS connection storm > cap | new connections refused | WS connection count + 503 | per-tenant session cap + queue; HPA scale-out | 10min |
| FM-04 | real-time-collaboration-worker | Loro library merge bug (regression) | conflict surfacer mis-fires OR silent loss | proptest + RustSec advisory | pin previous Loro; rollback pods; freeze T2 authoring | 30min |
| FM-05 | real-time-collaboration-worker | HMAC key compromise | op-tampering possible | HMAC mismatch counter + Sev-1 | rotate per-session keys; force reconnect | 10min |
| FM-06 | broadcast-mode-worker | LiveKit signaling drop | broadcast viewers disconnect | signal health SLO | runbook: graceful degrade to present-mode (no broadcast); audience reconnect on signal recovery | 15min |
| FM-07 | broadcast-mode-worker | LiveKit cluster overload | new broadcast sessions refused | viewer-cap + cluster-saturation alarm | admission throttle + cascade SFU | 20min |
| FM-08 | export-workers (PPTX) | malicious OOXML crashes parser | export-job fail; gVisor OOM-kill | OOM + worker-restart alarm | gVisor sandbox holds; rotate worker | 5min per job |
| FM-09 | export-workers (PDF) | WeasyPrint OOM on complex deck | export-job fail | OOM + Sev-3 | retry on Chromium-headless fallback; alarm if both fail | 10min |
| FM-10 | export-workers (MP4) | ffmpeg deterministic-mode regression | non-deterministic MP4 output | deterministic-output hash check | pin previous ffmpeg version; retry job | 20min |
| FM-11 | import-workers | malicious PPTX with macro/embed | parser crash + potential CVE | gVisor isolation + ClamAV/OPSWAT detection | quarantine + alert + sandbox limit holds | 5min per job |
| FM-12 | chart bridge | sheets µservice down | chart shows stale or unavailable | sheets-SDK health probe | render stale chart with timestamp; SLI degraded mark | tenant-visible until sheets recovers |
| FM-13 | chart bridge | sheets ACL revoked | live-link must revoke | revocation cascade audit | revocation flow ≤ 5s; chart shows revoked marker | 5s p95 |
| FM-14 | embed-bridge (docs quote) | docs µservice down | quote stale | docs-SDK health probe | render stale quote with timestamp | tenant-visible until docs recovers |
| FM-15 | embed-bridge (forms poll) | forms µservice down | poll stale | forms-SDK health probe | hide poll widget; reactivate on recovery | tenant-visible |
| FM-16 | ai-design / ai-content-generation | foundry-runtime degraded | AI assist unavailable | foundry-SDK health | Studio works without AI; banner notifies | tenant-visible |
| FM-17 | ai-content-generation | T2 risk-class mis-evaluation | high-risk T2 invocation slipped past | ai-act-risk-class-stamp lane + runtime check | freeze T2 capability; manual triage | 1h |
| FM-18 | acl | Cedar evaluator crash | per-slide ACL fails | crash + Sev-1 | fail-closed (helm cedar.failClosed: true); 503 to client | 5min |
| FM-19 | acl | per-slide ACL state drift between cache + Postgres | wrong allow/deny | drift detector cron + per-decision audit | invalidate cache; refresh from Postgres | 5min |
| FM-20 | animations | reduced-motion fallback skipped | accessibility violation | reduced-motion-fallback-mandatory lane | rollback; per-pack policy reset | 30min |
| FM-21 | version-history | restore corrupts deck | data corruption | restore-test + diff verification | freeze deck; manual diff; rollback to last-known-good | 30min |
| FM-22 | CDN | per-tenant cache pollution | cross-tenant leak risk | per-tenant cache key audit + Sev-1 if violated | purge affected keys; rotate keys | 30min |
| FM-23 | OpenBao | secret retrieval down | new pod start fails | OpenBao health | wait for OpenBao recovery; existing pods unaffected | 5min |
| FM-24 | CRDT WS gateway | single-region failover during active session | active sessions disconnect | per-region health | clients auto-reconnect after failover; CRDT state reconstructed from Postgres | RTO 30s |
| FM-25 | Postgres async replica lag | DR replica falls behind | RPO breached | replica lag SLO | alarm + manual investigation; throttle writes if extreme | tunable |
| FM-26 | gVisor sandbox escape (hypothetical) | CVE in gVisor | export worker compromise | gVisor advisory feed + node detection | per-pod isolation; immediate gVisor upgrade; SLSA L3 attest fail | hours |
| FM-27 | T2 deck-from-prompt hallucinates PII | T2 model regression | privacy + audit issue | watermark check + per-pack PHI redaction | refuse T2 invocations until validated; per ADR-SLIDES-0006 | 1h |
| FM-28 | broadcast presenter token leak | LiveKit one-time token compromised | unauthorized presenter | messenger token rotation + audit | rotate; revoke session; audit | 5min |
| FM-29 | per-pack theme/template signing key compromise | external CA / OpenBao compromise | tampered themes accepted | revocation list + Sev-1 | revoke key; force-load latest CRL; tenant comms | 1h |
| FM-30 | WASM bundle SRI mismatch (CDN tamper) | WASM-bundle-sri lane red OR runtime SRI fail | WASM refuse load | SRI mismatch counter + Sev-1 | rebuild + republish; CDN purge; tenant banner | 30min |

## Cross-µservice failure propagation

- sheets down → chart bridge degrades + tenant banner
- docs down → embed-bridge quotes stale + tenant banner
- forms down → poll widget hidden + tenant banner
- foundry-runtime down → AI capabilities disabled + tenant banner
- messenger LiveKit down → broadcast unavailable; non-broadcast present-mode unaffected
- tenancy down → per-seat ACL evaluation falls back to deny (fail-closed); editor opens refuse
- audit-chain down → save path queued + retry; eventually consistent

## Failure-mode test suite

Each FM mapped to a chaos-engineering test under `tests/chaos/`:

| FM | Test |
|---|---|
| FM-01 | `tests/chaos/postgres-primary-down.rs` |
| FM-02 | `tests/chaos/redis-split-brain.rs` |
| FM-06 | `tests/chaos/livekit-signal-drop.rs` |
| FM-08 | `tests/chaos/pptx-malicious-import.rs` |
| FM-12 | `tests/chaos/sheets-down-chart-stale.rs` |
| FM-13 | `tests/chaos/sheets-acl-revoke-cascade.rs` |
| FM-18 | `tests/chaos/cedar-evaluator-crash.rs` |
| FM-30 | `tests/chaos/wasm-bundle-sri-tamper.rs` |

## References

- ADR-0139 SLO-gated promotion.
- ADR-SLIDES-0005 broadcast-mode LiveKit reuse.
- ADR-SLIDES-0008 chart-live-link.
- `runbooks/`.
