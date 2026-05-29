---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0297]
companion_docs: [microservices/social/catalog/oya-community-social-sock-puppet-detector-kernel.yaml]
inbound_citations: [microservices/social/ARCHITECTURE.md]
---

# Runbook: Sock-puppet cluster takedown

## A. Trigger conditions

- `oya-community-social-sock-puppet-detector-kernel` emits a cluster verdict with confidence > 90.
- External tip-off / news report of coordinated inauthentic behavior on the platform.
- Pattern-anomaly alert: ≥10 new accounts within 1h sharing device fingerprint + IP-range + email-domain.

## B. Pre-checks

1. Operator Cedar permit `oya.social.sock-puppet-takedown` + trust-and-safety role.
2. Pull the cluster: `oya social sock-puppet-cluster-list --cluster-id <id>`; expect account roster + linkage evidence (shared fingerprint, IP, email domain, behavior).
3. Confirm cluster verdict reviewed by ≥2 trust-and-safety reviewers (avoid single-point-of-error).

## C. Procedure

1. **Snapshot evidence.** `oya social cluster-snapshot --cluster-id <id> --evidence-dir evidence/sock-puppet/<date>/`; persists fingerprints, post history, engagement edges. Emits `oya.social.cluster-snapshot`. Timing ≤120s.
2. **Suspend accounts.** Batch suspend the cluster: `oya social account-suspend --cluster <id> --reason coordinated-inauthentic-behavior`. Emits `oya.social.account-suspend` per account.
3. **Reverse engagements.** Roll back the cluster's likes / follows / reposts / amplifications on legitimate-content visibility metrics (so attacker doesn't profit). Emits `oya.social.engagement-reverse`.
4. **Quarantine content.** Cluster's posts → not deleted (evidence) but withdrawn from public surfaces; visible only in trust-and-safety review console.
5. **Notify affected legitimate users** whose accounts were targets of the cluster (e.g., dogpiled victims of coordinated harassment); per ADR-0263 emit `oya.social.user-notify{reason=cluster-takedown}`.
6. **Update sock-puppet detector** training corpus with the cluster's pattern; emits `oya.social.detector-corpus-update`.
7. **Publish transparency report entry** if cluster meets DSA threshold (>1000 accounts or coordinated harassment campaign); add to `runbooks/dsa-transparency-report-generation.md` queue.
8. **External notification** (if federated): notify receiving instances of suspended actor IDs via standard ActivityPub abuse-report channel.
9. **LEA referral** if cluster shows criminal coordination (fraud, election interference, coordinated harassment crossing into stalking).
10. **Closure.** `oya.social.cluster-takedown-complete`.

## D. Verification

- Cluster accounts all suspended; new posts blocked.
- Engagement rollback applied to legitimate-content metrics.
- Evidence preserved.
- Detector training updated.

## E. Rollback

Individual-account false-positive recovery via standard appeals (each affected account can appeal); cluster-level verdict reversal requires ≥3 reviewer consensus.

## F. Post-incident

DSA transparency-report entry + ADR amendment if doctrinal gap.

## G. References

- `policy/abuse-defence.cedar`
- `runbooks/coordinated-inauthentic-behavior-response.md`
- ADR-0297
