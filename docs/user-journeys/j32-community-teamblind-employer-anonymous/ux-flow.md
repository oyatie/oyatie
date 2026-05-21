---
doc_class: UserJourneyUXFlow
shape: HowTo
journey_id: j32
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0244
  - ADR-0263
  - ADR-0273
  - ADR-0297
  - ADR-0299
---

# UX Flow - Community TeamBlind employer-anonymous post

## A. Assumptions
- Persona: Yejin Park.
- Locale: `ko-KR`.
- Tenant mode: `verified-employer-anonymous`.
- Input: touch, keyboard, screen reader, voice, reduced motion.
- Accessibility target: WCAG 2.2 AAA on critical paths.

## B. Screen-by-screen flow

| # | Screen | Service | User action | System response |
|---:|---|---|---|---|
| 1 | `j32-screen-01` | community | Continue `community-teamblind-employer-anonymous` step 1. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 2 | `j32-screen-02` | identity | Continue `community-teamblind-employer-anonymous` step 2. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 3 | `j32-screen-03` | audit-chain | Continue `community-teamblind-employer-anonymous` step 3. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 4 | `j32-screen-04` | observability | Continue `community-teamblind-employer-anonymous` step 4. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 5 | `j32-screen-05` | community | Continue `community-teamblind-employer-anonymous` step 5. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 6 | `j32-screen-06` | identity | Continue `community-teamblind-employer-anonymous` step 6. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 7 | `j32-screen-07` | audit-chain | Continue `community-teamblind-employer-anonymous` step 7. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 8 | `j32-screen-08` | observability | Continue `community-teamblind-employer-anonymous` step 8. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 9 | `j32-screen-09` | community | Continue `community-teamblind-employer-anonymous` step 9. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 10 | `j32-screen-10` | identity | Continue `community-teamblind-employer-anonymous` step 10. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 11 | `j32-screen-11` | audit-chain | Continue `community-teamblind-employer-anonymous` step 11. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 12 | `j32-screen-12` | observability | Continue `community-teamblind-employer-anonymous` step 12. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 13 | `j32-screen-13` | community | Continue `community-teamblind-employer-anonymous` step 13. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 14 | `j32-screen-14` | identity | Continue `community-teamblind-employer-anonymous` step 14. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 15 | `j32-screen-15` | audit-chain | Continue `community-teamblind-employer-anonymous` step 15. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 16 | `j32-screen-16` | observability | Continue `community-teamblind-employer-anonymous` step 16. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 17 | `j32-screen-17` | community | Continue `community-teamblind-employer-anonymous` step 17. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 18 | `j32-screen-18` | identity | Continue `community-teamblind-employer-anonymous` step 18. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 19 | `j32-screen-19` | audit-chain | Continue `community-teamblind-employer-anonymous` step 19. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 20 | `j32-screen-20` | observability | Continue `community-teamblind-employer-anonymous` step 20. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 21 | `j32-screen-21` | community | Continue `community-teamblind-employer-anonymous` step 21. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 22 | `j32-screen-22` | identity | Continue `community-teamblind-employer-anonymous` step 22. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 23 | `j32-screen-23` | audit-chain | Continue `community-teamblind-employer-anonymous` step 23. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 24 | `j32-screen-24` | observability | Continue `community-teamblind-employer-anonymous` step 24. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 25 | `j32-screen-25` | community | Continue `community-teamblind-employer-anonymous` step 25. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 26 | `j32-screen-26` | identity | Continue `community-teamblind-employer-anonymous` step 26. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 27 | `j32-screen-27` | audit-chain | Continue `community-teamblind-employer-anonymous` step 27. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 28 | `j32-screen-28` | observability | Continue `community-teamblind-employer-anonymous` step 28. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 29 | `j32-screen-29` | community | Continue `community-teamblind-employer-anonymous` step 29. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 30 | `j32-screen-30` | identity | Continue `community-teamblind-employer-anonymous` step 30. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 31 | `j32-screen-31` | audit-chain | Continue `community-teamblind-employer-anonymous` step 31. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 32 | `j32-screen-32` | observability | Continue `community-teamblind-employer-anonymous` step 32. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 33 | `j32-screen-33` | community | Continue `community-teamblind-employer-anonymous` step 33. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 34 | `j32-screen-34` | identity | Continue `community-teamblind-employer-anonymous` step 34. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 35 | `j32-screen-35` | audit-chain | Continue `community-teamblind-employer-anonymous` step 35. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 36 | `j32-screen-36` | observability | Continue `community-teamblind-employer-anonymous` step 36. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 37 | `j32-screen-37` | community | Continue `community-teamblind-employer-anonymous` step 37. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 38 | `j32-screen-38` | identity | Continue `community-teamblind-employer-anonymous` step 38. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 39 | `j32-screen-39` | audit-chain | Continue `community-teamblind-employer-anonymous` step 39. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 40 | `j32-screen-40` | observability | Continue `community-teamblind-employer-anonymous` step 40. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 41 | `j32-screen-41` | community | Continue `community-teamblind-employer-anonymous` step 41. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 42 | `j32-screen-42` | identity | Continue `community-teamblind-employer-anonymous` step 42. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 43 | `j32-screen-43` | audit-chain | Continue `community-teamblind-employer-anonymous` step 43. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 44 | `j32-screen-44` | observability | Continue `community-teamblind-employer-anonymous` step 44. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 45 | `j32-screen-45` | community | Continue `community-teamblind-employer-anonymous` step 45. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 46 | `j32-screen-46` | identity | Continue `community-teamblind-employer-anonymous` step 46. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 47 | `j32-screen-47` | audit-chain | Continue `community-teamblind-employer-anonymous` step 47. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 48 | `j32-screen-48` | observability | Continue `community-teamblind-employer-anonymous` step 48. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 49 | `j32-screen-49` | community | Continue `community-teamblind-employer-anonymous` step 49. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 50 | `j32-screen-50` | identity | Continue `community-teamblind-employer-anonymous` step 50. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 51 | `j32-screen-51` | audit-chain | Continue `community-teamblind-employer-anonymous` step 51. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 52 | `j32-screen-52` | observability | Continue `community-teamblind-employer-anonymous` step 52. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 53 | `j32-screen-53` | community | Continue `community-teamblind-employer-anonymous` step 53. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 54 | `j32-screen-54` | identity | Continue `community-teamblind-employer-anonymous` step 54. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 55 | `j32-screen-55` | audit-chain | Continue `community-teamblind-employer-anonymous` step 55. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 56 | `j32-screen-56` | observability | Continue `community-teamblind-employer-anonymous` step 56. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 57 | `j32-screen-57` | community | Continue `community-teamblind-employer-anonymous` step 57. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 58 | `j32-screen-58` | identity | Continue `community-teamblind-employer-anonymous` step 58. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 59 | `j32-screen-59` | audit-chain | Continue `community-teamblind-employer-anonymous` step 59. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 60 | `j32-screen-60` | observability | Continue `community-teamblind-employer-anonymous` step 60. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 61 | `j32-screen-61` | community | Continue `community-teamblind-employer-anonymous` step 61. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 62 | `j32-screen-62` | identity | Continue `community-teamblind-employer-anonymous` step 62. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 63 | `j32-screen-63` | audit-chain | Continue `community-teamblind-employer-anonymous` step 63. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 64 | `j32-screen-64` | observability | Continue `community-teamblind-employer-anonymous` step 64. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 65 | `j32-screen-65` | community | Continue `community-teamblind-employer-anonymous` step 65. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 66 | `j32-screen-66` | identity | Continue `community-teamblind-employer-anonymous` step 66. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 67 | `j32-screen-67` | audit-chain | Continue `community-teamblind-employer-anonymous` step 67. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 68 | `j32-screen-68` | observability | Continue `community-teamblind-employer-anonymous` step 68. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 69 | `j32-screen-69` | community | Continue `community-teamblind-employer-anonymous` step 69. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 70 | `j32-screen-70` | identity | Continue `community-teamblind-employer-anonymous` step 70. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 71 | `j32-screen-71` | audit-chain | Continue `community-teamblind-employer-anonymous` step 71. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |
| 72 | `j32-screen-72` | observability | Continue `community-teamblind-employer-anonymous` step 72. | Runs `moderation-slo`, preserves tenant context, and shows localized recovery if needed. |
| 73 | `j32-screen-73` | community | Continue `community-teamblind-employer-anonymous` step 73. | Runs `teamblind-anonymous-post`, preserves tenant context, and shows localized recovery if needed. |
| 74 | `j32-screen-74` | identity | Continue `community-teamblind-employer-anonymous` step 74. | Runs `employer-attestation`, preserves tenant context, and shows localized recovery if needed. |
| 75 | `j32-screen-75` | audit-chain | Continue `community-teamblind-employer-anonymous` step 75. | Runs `anonymous-proof-seal`, preserves tenant context, and shows localized recovery if needed. |

## C. Interaction states

| State | Behavior | Invariant |
|---|---|---|
| Default | One clear next action | User completes the job without doctrine text |
| Loading | Stable skeleton | No layout jump |
| Recoverable error | Safe retry | No duplicate writes |
| Policy denial | Tenant policy and appeal path | No vague security copy |
| Offline | Draft or intent preserved | No silent data loss |
| Low bandwidth | Core action remains | Nonessential media deferred |
| Screen reader | Name role value present | No icon-only mystery action |
| Reduced motion | No motion-dependent meaning | User preference wins |

## D. Accessibility and i18n matrix

| # | Requirement | Verification |
|---:|---|---|
| 1 | Critical-path row 2 `account recovery` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 2 | Critical-path row 3 `financial dispute` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 3 | Critical-path row 4 `elder financial abuse` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 4 | Critical-path row 6 `whistleblower` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 5 | Critical-path row 7 `press freedom` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 6 | Critical-path row 8 `survivor shelter` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 7 | Critical-path row 9 `child safety` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 8 | Critical-path row 12 `accessibility accommodations` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 9 | Critical-path row 13 `non native language` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 10 | Critical-path row 14 `offline low bandwidth` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 11 | Critical-path row 15 `financial inclusion` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 12 | Critical-path row 16 `activist privacy` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 13 | Critical-path row 18 `regulator access` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 14 | Critical-path row 21 `pseudonymity` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 15 | Critical-path row 23 `cross jurisdiction` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 16 | Critical-path row 24 `hijack recovery` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 17 | Critical-path row 25 `mistaken action` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 18 | Critical-path row 28 `delegated agent` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 19 | Critical-path row 29 `high value transaction` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 20 | Critical-path row 30 `regional outage` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 21 | Critical-path row 2 `account recovery` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 22 | Critical-path row 3 `financial dispute` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 23 | Critical-path row 4 `elder financial abuse` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 24 | Critical-path row 6 `whistleblower` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 25 | Critical-path row 7 `press freedom` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 26 | Critical-path row 8 `survivor shelter` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 27 | Critical-path row 9 `child safety` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 28 | Critical-path row 12 `accessibility accommodations` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 29 | Critical-path row 13 `non native language` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 30 | Critical-path row 14 `offline low bandwidth` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 31 | Critical-path row 15 `financial inclusion` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 32 | Critical-path row 16 `activist privacy` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 33 | Critical-path row 18 `regulator access` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 34 | Critical-path row 21 `pseudonymity` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 35 | Critical-path row 23 `cross jurisdiction` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 36 | Critical-path row 24 `hijack recovery` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 37 | Critical-path row 25 `mistaken action` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 38 | Critical-path row 28 `delegated agent` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 39 | Critical-path row 29 `high value transaction` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 40 | Critical-path row 30 `regional outage` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 41 | Critical-path row 2 `account recovery` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 42 | Critical-path row 3 `financial dispute` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 43 | Critical-path row 4 `elder financial abuse` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 44 | Critical-path row 6 `whistleblower` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 45 | Critical-path row 7 `press freedom` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 46 | Critical-path row 8 `survivor shelter` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 47 | Critical-path row 9 `child safety` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 48 | Critical-path row 12 `accessibility accommodations` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 49 | Critical-path row 13 `non native language` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 50 | Critical-path row 14 `offline low bandwidth` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 51 | Critical-path row 15 `financial inclusion` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 52 | Critical-path row 16 `activist privacy` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 53 | Critical-path row 18 `regulator access` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 54 | Critical-path row 21 `pseudonymity` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 55 | Critical-path row 23 `cross jurisdiction` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 56 | Critical-path row 24 `hijack recovery` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 57 | Critical-path row 25 `mistaken action` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 58 | Critical-path row 28 `delegated agent` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 59 | Critical-path row 29 `high value transaction` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |
| 60 | Critical-path row 30 `regional outage` has locale-aware and assistive-tech-safe copy. | a11y smoke plus translation key coverage |

## E. Negative UX cases

| # | Case | Expected behavior |
|---:|---|---|
| 1 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 2 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |
| 3 | `audit-chain` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.audit-chain.ux_recovery`. |
| 4 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.observability.ux_recovery`. |
| 5 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 6 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |
| 7 | `audit-chain` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.audit-chain.ux_recovery`. |
| 8 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.observability.ux_recovery`. |
| 9 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 10 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |
| 11 | `audit-chain` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.audit-chain.ux_recovery`. |
| 12 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.observability.ux_recovery`. |
| 13 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 14 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |
| 15 | `audit-chain` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.audit-chain.ux_recovery`. |
| 16 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.observability.ux_recovery`. |
| 17 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 18 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |
| 19 | `audit-chain` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.audit-chain.ux_recovery`. |
| 20 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.observability.ux_recovery`. |
| 21 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 22 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |
| 23 | `audit-chain` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.audit-chain.ux_recovery`. |
| 24 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.observability.ux_recovery`. |
| 25 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 26 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |
| 27 | `audit-chain` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.audit-chain.ux_recovery`. |
| 28 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.observability.ux_recovery`. |
| 29 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 30 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |
| 31 | `audit-chain` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.audit-chain.ux_recovery`. |
| 32 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.observability.ux_recovery`. |
| 33 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 34 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |
| 35 | `audit-chain` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.audit-chain.ux_recovery`. |
| 36 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.observability.ux_recovery`. |
| 37 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 38 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |
| 39 | `audit-chain` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.audit-chain.ux_recovery`. |
| 40 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.observability.ux_recovery`. |
| 41 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 42 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |
| 43 | `audit-chain` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.audit-chain.ux_recovery`. |
| 44 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.observability.ux_recovery`. |
| 45 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 46 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |
| 47 | `audit-chain` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.audit-chain.ux_recovery`. |
| 48 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.observability.ux_recovery`. |
| 49 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.community.ux_recovery`. |
| 50 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j32.identity.ux_recovery`. |

## F. Done-state UX
The flow ends on the completed object or action, not a landing page. Reversible next steps are visible where policy allows, and audit transparency remains available without interrupting the job.

## Appendix A. Responsive recovery matrix

| UX-A001 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A002 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A003 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A004 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A005 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A006 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A007 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A008 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A009 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A010 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A011 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A012 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A013 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A014 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A015 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A016 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A017 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A018 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A019 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A020 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A021 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A022 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A023 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A024 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A025 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A026 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A027 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A028 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A029 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A030 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A031 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A032 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A033 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A034 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A035 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A036 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A037 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A038 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A039 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A040 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A041 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A042 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A043 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A044 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A045 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A046 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A047 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A048 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A049 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A050 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A051 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A052 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A053 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A054 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A055 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A056 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A057 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A058 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A059 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A060 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A061 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A062 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A063 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A064 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A065 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A066 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A067 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A068 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A069 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A070 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A071 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A072 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A073 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A074 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A075 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A076 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A077 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A078 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A079 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A080 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A081 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A082 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A083 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A084 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A085 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A086 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A087 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A088 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A089 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A090 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A091 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A092 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A093 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A094 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A095 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A096 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A097 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A098 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A099 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A100 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A101 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A102 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A103 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A104 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A105 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A106 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A107 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A108 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A109 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A110 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A111 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A112 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A113 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A114 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A115 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A116 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A117 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A118 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A119 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A120 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A121 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A122 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A123 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A124 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A125 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A126 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A127 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A128 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A129 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A130 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A131 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A132 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A133 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A134 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A135 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A136 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A137 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A138 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A139 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A140 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A141 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A142 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A143 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A144 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A145 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A146 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A147 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A148 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A149 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A150 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A151 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A152 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A153 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A154 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A155 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A156 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A157 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A158 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A159 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A160 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A161 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A162 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A163 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A164 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A165 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A166 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A167 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A168 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A169 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A170 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A171 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A172 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A173 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A174 | `community` `teamblind-anonymous-post` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A175 | `identity` `employer-attestation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A176 | `observability` `moderation-slo` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A177 | `audit-chain` `anonymous-proof-seal` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
