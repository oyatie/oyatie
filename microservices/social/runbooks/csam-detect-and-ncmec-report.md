---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0292, ADR-SOC-0003]
companion_docs: [microservices/social/policy/content-policy.cedar]
inbound_citations: [microservices/social/ARCHITECTURE.md]
---

# Runbook: CSAM detect + NCMEC report

## A. Trigger conditions

- `oya-community-social-csam-classifier-adapter-photodna` emits a hash-match against the NCMEC list OR
- Content-moderation classifier emits `csam_classifier_score > 50` per `policy/content-policy.cedar` block.
- User report flagging content as CSAM.

## B. Pre-checks

1. Operator Cedar permit `oya.social.csam-incident-respond` + trust-and-safety role.
2. Hash-match must be authoritative (PhotoDNA / Microsoft hash-DB / Thorn Safer / Google CSAI Match).
3. Capture content-id + uploader-id + IP + device-fingerprint + timestamps.

## C. Procedure

1. **Hard remove.** Content immediately removed from all caches + CDN purge; emits `oya.social.csam-detect` + `oya.social.post-delete{reason=csam}`. Timing ≤30s.
2. **Account suspension.** Uploader account suspended pending review: `oya social account-suspend --user <id> --reason csam-detect`. Emits `oya.social.account-suspend{reason=csam-detect}`.
3. **Preserve evidence.** Original content + uploader metadata + audit chain preserved in secure evidence store; do NOT delete (legal hold per chain-of-custody).
4. **File NCMEC CyberTipline report.** Within 24h per 18 U.S.C. §2258A; submit via NCMEC API; emits `oya.social.csam-ncmec-report` with the CyberTipline report ID. Required fields: content hash, content type, uploader IP + timestamp, account info, EXIF if available.
5. **Notify TSI partners.** If content was federated outbound, notify receiving instances via standard ActivityPub abuse-report channel + dedicated CSAM-takedown channel.
6. **Notify LEA on subpoena.** If law enforcement subpoenas the user record, route to legal counsel; preserve via legal hold.
7. **Suspend related accounts.** If sock-puppet cluster detected (per `runbooks/sock-puppet-cluster-takedown.md`), suspend the cluster.
8. **Mute the user-facing surface during investigation.** No user-visible message details until investigation closes.
9. **Closure.** After NCMEC acknowledgement received + account-level review complete: emit `oya.social.csam-incident-close`.

## D. Verification

- Content unreachable across all surfaces (CDN, cache, federation, search index).
- NCMEC report ID logged + retrievable.
- Evidence preserved + accessible to authorized investigators.
- Account suspension persistent.

## E. Rollback

CSAM detection rollback is **forbidden**. False-positive recovery (extremely rare given PhotoDNA precision) goes through the trust-and-safety appeals channel; the content remains removed during appeal; account-suspension may be lifted only after manual review by ≥2 trust-and-safety reviewers.

## F. Post-incident

- File ADR amendment if Cedar gap surfaced.
- Update classifier corpus if new hash variant detected.
- Update CHANGELOG of trust-and-safety operations.

## G. References

- `policy/content-policy.cedar`
- `runbooks/sock-puppet-cluster-takedown.md`
- 18 U.S.C. §2258A (NCMEC mandatory reporting)
- EU CSAM Regulation
