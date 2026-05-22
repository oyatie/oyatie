---
doc_class: UserJourneyUXFlow
shape: HowTo
journey_id: j31
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

# UX Flow - Social broadcast versus DM

## A. Assumptions
- Persona: Yejin Park.
- Locale: `ko-KR`.
- Tenant mode: `personal`.
- Input: touch, keyboard, screen reader, voice, reduced motion.
- Accessibility target: WCAG 2.2 AAA on critical paths.

## B. Screen-by-screen flow

| # | Screen | Service | User action | System response |
|---:|---|---|---|---|
| 1 | `j31-screen-01` | social | Continue `social-broadcast-vs-DM` step 1. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 2 | `j31-screen-02` | identity | Continue `social-broadcast-vs-DM` step 2. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 3 | `j31-screen-03` | community | Continue `social-broadcast-vs-DM` step 3. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 4 | `j31-screen-04` | intelligence | Continue `social-broadcast-vs-DM` step 4. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 5 | `j31-screen-05` | social | Continue `social-broadcast-vs-DM` step 5. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 6 | `j31-screen-06` | identity | Continue `social-broadcast-vs-DM` step 6. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 7 | `j31-screen-07` | community | Continue `social-broadcast-vs-DM` step 7. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 8 | `j31-screen-08` | intelligence | Continue `social-broadcast-vs-DM` step 8. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 9 | `j31-screen-09` | social | Continue `social-broadcast-vs-DM` step 9. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 10 | `j31-screen-10` | identity | Continue `social-broadcast-vs-DM` step 10. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 11 | `j31-screen-11` | community | Continue `social-broadcast-vs-DM` step 11. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 12 | `j31-screen-12` | intelligence | Continue `social-broadcast-vs-DM` step 12. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 13 | `j31-screen-13` | social | Continue `social-broadcast-vs-DM` step 13. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 14 | `j31-screen-14` | identity | Continue `social-broadcast-vs-DM` step 14. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 15 | `j31-screen-15` | community | Continue `social-broadcast-vs-DM` step 15. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 16 | `j31-screen-16` | intelligence | Continue `social-broadcast-vs-DM` step 16. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 17 | `j31-screen-17` | social | Continue `social-broadcast-vs-DM` step 17. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 18 | `j31-screen-18` | identity | Continue `social-broadcast-vs-DM` step 18. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 19 | `j31-screen-19` | community | Continue `social-broadcast-vs-DM` step 19. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 20 | `j31-screen-20` | intelligence | Continue `social-broadcast-vs-DM` step 20. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 21 | `j31-screen-21` | social | Continue `social-broadcast-vs-DM` step 21. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 22 | `j31-screen-22` | identity | Continue `social-broadcast-vs-DM` step 22. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 23 | `j31-screen-23` | community | Continue `social-broadcast-vs-DM` step 23. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 24 | `j31-screen-24` | intelligence | Continue `social-broadcast-vs-DM` step 24. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 25 | `j31-screen-25` | social | Continue `social-broadcast-vs-DM` step 25. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 26 | `j31-screen-26` | identity | Continue `social-broadcast-vs-DM` step 26. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 27 | `j31-screen-27` | community | Continue `social-broadcast-vs-DM` step 27. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 28 | `j31-screen-28` | intelligence | Continue `social-broadcast-vs-DM` step 28. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 29 | `j31-screen-29` | social | Continue `social-broadcast-vs-DM` step 29. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 30 | `j31-screen-30` | identity | Continue `social-broadcast-vs-DM` step 30. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 31 | `j31-screen-31` | community | Continue `social-broadcast-vs-DM` step 31. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 32 | `j31-screen-32` | intelligence | Continue `social-broadcast-vs-DM` step 32. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 33 | `j31-screen-33` | social | Continue `social-broadcast-vs-DM` step 33. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 34 | `j31-screen-34` | identity | Continue `social-broadcast-vs-DM` step 34. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 35 | `j31-screen-35` | community | Continue `social-broadcast-vs-DM` step 35. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 36 | `j31-screen-36` | intelligence | Continue `social-broadcast-vs-DM` step 36. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 37 | `j31-screen-37` | social | Continue `social-broadcast-vs-DM` step 37. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 38 | `j31-screen-38` | identity | Continue `social-broadcast-vs-DM` step 38. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 39 | `j31-screen-39` | community | Continue `social-broadcast-vs-DM` step 39. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 40 | `j31-screen-40` | intelligence | Continue `social-broadcast-vs-DM` step 40. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 41 | `j31-screen-41` | social | Continue `social-broadcast-vs-DM` step 41. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 42 | `j31-screen-42` | identity | Continue `social-broadcast-vs-DM` step 42. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 43 | `j31-screen-43` | community | Continue `social-broadcast-vs-DM` step 43. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 44 | `j31-screen-44` | intelligence | Continue `social-broadcast-vs-DM` step 44. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 45 | `j31-screen-45` | social | Continue `social-broadcast-vs-DM` step 45. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 46 | `j31-screen-46` | identity | Continue `social-broadcast-vs-DM` step 46. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 47 | `j31-screen-47` | community | Continue `social-broadcast-vs-DM` step 47. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 48 | `j31-screen-48` | intelligence | Continue `social-broadcast-vs-DM` step 48. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 49 | `j31-screen-49` | social | Continue `social-broadcast-vs-DM` step 49. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 50 | `j31-screen-50` | identity | Continue `social-broadcast-vs-DM` step 50. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 51 | `j31-screen-51` | community | Continue `social-broadcast-vs-DM` step 51. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 52 | `j31-screen-52` | intelligence | Continue `social-broadcast-vs-DM` step 52. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 53 | `j31-screen-53` | social | Continue `social-broadcast-vs-DM` step 53. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 54 | `j31-screen-54` | identity | Continue `social-broadcast-vs-DM` step 54. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 55 | `j31-screen-55` | community | Continue `social-broadcast-vs-DM` step 55. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 56 | `j31-screen-56` | intelligence | Continue `social-broadcast-vs-DM` step 56. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 57 | `j31-screen-57` | social | Continue `social-broadcast-vs-DM` step 57. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 58 | `j31-screen-58` | identity | Continue `social-broadcast-vs-DM` step 58. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 59 | `j31-screen-59` | community | Continue `social-broadcast-vs-DM` step 59. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 60 | `j31-screen-60` | intelligence | Continue `social-broadcast-vs-DM` step 60. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 61 | `j31-screen-61` | social | Continue `social-broadcast-vs-DM` step 61. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 62 | `j31-screen-62` | identity | Continue `social-broadcast-vs-DM` step 62. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 63 | `j31-screen-63` | community | Continue `social-broadcast-vs-DM` step 63. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 64 | `j31-screen-64` | intelligence | Continue `social-broadcast-vs-DM` step 64. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 65 | `j31-screen-65` | social | Continue `social-broadcast-vs-DM` step 65. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 66 | `j31-screen-66` | identity | Continue `social-broadcast-vs-DM` step 66. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 67 | `j31-screen-67` | community | Continue `social-broadcast-vs-DM` step 67. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 68 | `j31-screen-68` | intelligence | Continue `social-broadcast-vs-DM` step 68. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 69 | `j31-screen-69` | social | Continue `social-broadcast-vs-DM` step 69. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 70 | `j31-screen-70` | identity | Continue `social-broadcast-vs-DM` step 70. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 71 | `j31-screen-71` | community | Continue `social-broadcast-vs-DM` step 71. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 72 | `j31-screen-72` | intelligence | Continue `social-broadcast-vs-DM` step 72. | Runs `spam-cib-signals`, preserves tenant context, and shows localized recovery if needed. |
| 73 | `j31-screen-73` | social | Continue `social-broadcast-vs-DM` step 73. | Runs `broadcast-context`, preserves tenant context, and shows localized recovery if needed. |
| 74 | `j31-screen-74` | identity | Continue `social-broadcast-vs-DM` step 74. | Runs `same-human-mode-claims`, preserves tenant context, and shows localized recovery if needed. |
| 75 | `j31-screen-75` | community | Continue `social-broadcast-vs-DM` step 75. | Runs `reply-thread-bridge`, preserves tenant context, and shows localized recovery if needed. |

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
| 1 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 2 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |
| 3 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.community.ux_recovery`. |
| 4 | `intelligence` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.intelligence.ux_recovery`. |
| 5 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 6 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |
| 7 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.community.ux_recovery`. |
| 8 | `intelligence` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.intelligence.ux_recovery`. |
| 9 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 10 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |
| 11 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.community.ux_recovery`. |
| 12 | `intelligence` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.intelligence.ux_recovery`. |
| 13 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 14 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |
| 15 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.community.ux_recovery`. |
| 16 | `intelligence` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.intelligence.ux_recovery`. |
| 17 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 18 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |
| 19 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.community.ux_recovery`. |
| 20 | `intelligence` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.intelligence.ux_recovery`. |
| 21 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 22 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |
| 23 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.community.ux_recovery`. |
| 24 | `intelligence` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.intelligence.ux_recovery`. |
| 25 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 26 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |
| 27 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.community.ux_recovery`. |
| 28 | `intelligence` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.intelligence.ux_recovery`. |
| 29 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 30 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |
| 31 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.community.ux_recovery`. |
| 32 | `intelligence` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.intelligence.ux_recovery`. |
| 33 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 34 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |
| 35 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.community.ux_recovery`. |
| 36 | `intelligence` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.intelligence.ux_recovery`. |
| 37 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 38 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |
| 39 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.community.ux_recovery`. |
| 40 | `intelligence` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.intelligence.ux_recovery`. |
| 41 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 42 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |
| 43 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.community.ux_recovery`. |
| 44 | `intelligence` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.intelligence.ux_recovery`. |
| 45 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 46 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |
| 47 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.community.ux_recovery`. |
| 48 | `intelligence` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.intelligence.ux_recovery`. |
| 49 | `social` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.social.ux_recovery`. |
| 50 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j31.identity.ux_recovery`. |

## F. Done-state UX
The flow ends on the completed object or action, not a landing page. Reversible next steps are visible where policy allows, and audit transparency remains available without interrupting the job.

## Appendix A. Responsive recovery matrix

| UX-A001 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A002 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A003 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A004 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A005 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A006 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A007 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A008 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A009 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A010 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A011 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A012 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A013 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A014 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A015 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A016 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A017 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A018 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A019 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A020 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A021 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A022 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A023 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A024 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A025 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A026 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A027 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A028 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A029 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A030 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A031 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A032 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A033 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A034 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A035 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A036 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A037 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A038 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A039 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A040 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A041 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A042 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A043 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A044 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A045 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A046 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A047 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A048 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A049 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A050 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A051 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A052 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A053 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A054 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A055 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A056 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A057 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A058 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A059 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A060 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A061 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A062 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A063 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A064 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A065 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A066 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A067 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A068 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A069 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A070 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A071 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A072 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A073 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A074 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A075 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A076 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A077 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A078 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A079 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A080 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A081 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A082 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A083 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A084 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A085 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A086 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A087 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A088 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A089 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A090 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A091 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A092 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A093 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A094 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A095 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A096 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A097 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A098 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A099 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A100 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A101 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A102 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A103 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A104 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A105 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A106 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A107 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A108 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A109 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A110 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A111 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A112 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A113 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A114 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A115 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A116 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A117 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A118 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A119 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A120 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A121 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A122 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A123 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A124 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A125 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A126 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A127 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A128 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A129 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A130 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A131 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A132 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A133 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A134 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A135 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A136 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A137 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A138 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A139 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A140 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A141 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A142 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A143 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A144 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A145 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A146 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A147 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A148 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A149 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A150 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A151 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A152 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A153 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A154 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A155 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A156 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A157 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A158 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A159 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A160 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A161 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A162 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A163 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A164 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A165 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A166 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A167 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A168 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A169 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A170 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A171 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A172 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A173 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A174 | `identity` `same-human-mode-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A175 | `intelligence` `spam-cib-signals` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A176 | `social` `broadcast-context` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A177 | `community` `reply-thread-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
