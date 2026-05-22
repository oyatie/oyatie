---
doc_class: UserJourneyUXFlow
shape: HowTo
journey_id: j27
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
  - ADR-0311
---

# UX Flow - Calendar cross-context family and work

## A. Assumptions
- Persona: Yejin Park.
- Locale: `ko-KR`.
- Tenant mode: `dual-context`.
- Input: touch, keyboard, screen reader, voice, reduced motion.
- Accessibility target: WCAG 2.2 AAA on critical paths.

## B. Screen-by-screen flow

| # | Screen | Service | User action | System response |
|---:|---|---|---|---|
| 1 | `j27-screen-01` | calendar | Continue `calendar-cross-context-family-and-work` step 1. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 2 | `j27-screen-02` | identity | Continue `calendar-cross-context-family-and-work` step 2. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 3 | `j27-screen-03` | mail | Continue `calendar-cross-context-family-and-work` step 3. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 4 | `j27-screen-04` | observability | Continue `calendar-cross-context-family-and-work` step 4. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 5 | `j27-screen-05` | calendar | Continue `calendar-cross-context-family-and-work` step 5. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 6 | `j27-screen-06` | identity | Continue `calendar-cross-context-family-and-work` step 6. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 7 | `j27-screen-07` | mail | Continue `calendar-cross-context-family-and-work` step 7. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 8 | `j27-screen-08` | observability | Continue `calendar-cross-context-family-and-work` step 8. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 9 | `j27-screen-09` | calendar | Continue `calendar-cross-context-family-and-work` step 9. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 10 | `j27-screen-10` | identity | Continue `calendar-cross-context-family-and-work` step 10. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 11 | `j27-screen-11` | mail | Continue `calendar-cross-context-family-and-work` step 11. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 12 | `j27-screen-12` | observability | Continue `calendar-cross-context-family-and-work` step 12. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 13 | `j27-screen-13` | calendar | Continue `calendar-cross-context-family-and-work` step 13. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 14 | `j27-screen-14` | identity | Continue `calendar-cross-context-family-and-work` step 14. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 15 | `j27-screen-15` | mail | Continue `calendar-cross-context-family-and-work` step 15. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 16 | `j27-screen-16` | observability | Continue `calendar-cross-context-family-and-work` step 16. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 17 | `j27-screen-17` | calendar | Continue `calendar-cross-context-family-and-work` step 17. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 18 | `j27-screen-18` | identity | Continue `calendar-cross-context-family-and-work` step 18. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 19 | `j27-screen-19` | mail | Continue `calendar-cross-context-family-and-work` step 19. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 20 | `j27-screen-20` | observability | Continue `calendar-cross-context-family-and-work` step 20. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 21 | `j27-screen-21` | calendar | Continue `calendar-cross-context-family-and-work` step 21. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 22 | `j27-screen-22` | identity | Continue `calendar-cross-context-family-and-work` step 22. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 23 | `j27-screen-23` | mail | Continue `calendar-cross-context-family-and-work` step 23. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 24 | `j27-screen-24` | observability | Continue `calendar-cross-context-family-and-work` step 24. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 25 | `j27-screen-25` | calendar | Continue `calendar-cross-context-family-and-work` step 25. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 26 | `j27-screen-26` | identity | Continue `calendar-cross-context-family-and-work` step 26. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 27 | `j27-screen-27` | mail | Continue `calendar-cross-context-family-and-work` step 27. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 28 | `j27-screen-28` | observability | Continue `calendar-cross-context-family-and-work` step 28. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 29 | `j27-screen-29` | calendar | Continue `calendar-cross-context-family-and-work` step 29. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 30 | `j27-screen-30` | identity | Continue `calendar-cross-context-family-and-work` step 30. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 31 | `j27-screen-31` | mail | Continue `calendar-cross-context-family-and-work` step 31. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 32 | `j27-screen-32` | observability | Continue `calendar-cross-context-family-and-work` step 32. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 33 | `j27-screen-33` | calendar | Continue `calendar-cross-context-family-and-work` step 33. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 34 | `j27-screen-34` | identity | Continue `calendar-cross-context-family-and-work` step 34. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 35 | `j27-screen-35` | mail | Continue `calendar-cross-context-family-and-work` step 35. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 36 | `j27-screen-36` | observability | Continue `calendar-cross-context-family-and-work` step 36. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 37 | `j27-screen-37` | calendar | Continue `calendar-cross-context-family-and-work` step 37. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 38 | `j27-screen-38` | identity | Continue `calendar-cross-context-family-and-work` step 38. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 39 | `j27-screen-39` | mail | Continue `calendar-cross-context-family-and-work` step 39. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 40 | `j27-screen-40` | observability | Continue `calendar-cross-context-family-and-work` step 40. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 41 | `j27-screen-41` | calendar | Continue `calendar-cross-context-family-and-work` step 41. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 42 | `j27-screen-42` | identity | Continue `calendar-cross-context-family-and-work` step 42. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 43 | `j27-screen-43` | mail | Continue `calendar-cross-context-family-and-work` step 43. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 44 | `j27-screen-44` | observability | Continue `calendar-cross-context-family-and-work` step 44. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 45 | `j27-screen-45` | calendar | Continue `calendar-cross-context-family-and-work` step 45. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 46 | `j27-screen-46` | identity | Continue `calendar-cross-context-family-and-work` step 46. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 47 | `j27-screen-47` | mail | Continue `calendar-cross-context-family-and-work` step 47. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 48 | `j27-screen-48` | observability | Continue `calendar-cross-context-family-and-work` step 48. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 49 | `j27-screen-49` | calendar | Continue `calendar-cross-context-family-and-work` step 49. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 50 | `j27-screen-50` | identity | Continue `calendar-cross-context-family-and-work` step 50. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 51 | `j27-screen-51` | mail | Continue `calendar-cross-context-family-and-work` step 51. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 52 | `j27-screen-52` | observability | Continue `calendar-cross-context-family-and-work` step 52. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 53 | `j27-screen-53` | calendar | Continue `calendar-cross-context-family-and-work` step 53. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 54 | `j27-screen-54` | identity | Continue `calendar-cross-context-family-and-work` step 54. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 55 | `j27-screen-55` | mail | Continue `calendar-cross-context-family-and-work` step 55. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 56 | `j27-screen-56` | observability | Continue `calendar-cross-context-family-and-work` step 56. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 57 | `j27-screen-57` | calendar | Continue `calendar-cross-context-family-and-work` step 57. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 58 | `j27-screen-58` | identity | Continue `calendar-cross-context-family-and-work` step 58. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 59 | `j27-screen-59` | mail | Continue `calendar-cross-context-family-and-work` step 59. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 60 | `j27-screen-60` | observability | Continue `calendar-cross-context-family-and-work` step 60. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 61 | `j27-screen-61` | calendar | Continue `calendar-cross-context-family-and-work` step 61. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 62 | `j27-screen-62` | identity | Continue `calendar-cross-context-family-and-work` step 62. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 63 | `j27-screen-63` | mail | Continue `calendar-cross-context-family-and-work` step 63. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 64 | `j27-screen-64` | observability | Continue `calendar-cross-context-family-and-work` step 64. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 65 | `j27-screen-65` | calendar | Continue `calendar-cross-context-family-and-work` step 65. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 66 | `j27-screen-66` | identity | Continue `calendar-cross-context-family-and-work` step 66. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 67 | `j27-screen-67` | mail | Continue `calendar-cross-context-family-and-work` step 67. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 68 | `j27-screen-68` | observability | Continue `calendar-cross-context-family-and-work` step 68. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 69 | `j27-screen-69` | calendar | Continue `calendar-cross-context-family-and-work` step 69. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 70 | `j27-screen-70` | identity | Continue `calendar-cross-context-family-and-work` step 70. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 71 | `j27-screen-71` | mail | Continue `calendar-cross-context-family-and-work` step 71. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |
| 72 | `j27-screen-72` | observability | Continue `calendar-cross-context-family-and-work` step 72. | Runs `schedule-conflict-metrics`, preserves tenant context, and shows localized recovery if needed. |
| 73 | `j27-screen-73` | calendar | Continue `calendar-cross-context-family-and-work` step 73. | Runs `dual-context-freebusy`, preserves tenant context, and shows localized recovery if needed. |
| 74 | `j27-screen-74` | identity | Continue `calendar-cross-context-family-and-work` step 74. | Runs `context-switch-claims`, preserves tenant context, and shows localized recovery if needed. |
| 75 | `j27-screen-75` | mail | Continue `calendar-cross-context-family-and-work` step 75. | Runs `imip-invite-bridge`, preserves tenant context, and shows localized recovery if needed. |

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
| 1 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 2 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |
| 3 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.mail.ux_recovery`. |
| 4 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.observability.ux_recovery`. |
| 5 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 6 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |
| 7 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.mail.ux_recovery`. |
| 8 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.observability.ux_recovery`. |
| 9 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 10 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |
| 11 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.mail.ux_recovery`. |
| 12 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.observability.ux_recovery`. |
| 13 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 14 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |
| 15 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.mail.ux_recovery`. |
| 16 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.observability.ux_recovery`. |
| 17 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 18 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |
| 19 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.mail.ux_recovery`. |
| 20 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.observability.ux_recovery`. |
| 21 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 22 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |
| 23 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.mail.ux_recovery`. |
| 24 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.observability.ux_recovery`. |
| 25 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 26 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |
| 27 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.mail.ux_recovery`. |
| 28 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.observability.ux_recovery`. |
| 29 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 30 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |
| 31 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.mail.ux_recovery`. |
| 32 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.observability.ux_recovery`. |
| 33 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 34 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |
| 35 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.mail.ux_recovery`. |
| 36 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.observability.ux_recovery`. |
| 37 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 38 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |
| 39 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.mail.ux_recovery`. |
| 40 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.observability.ux_recovery`. |
| 41 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 42 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |
| 43 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.mail.ux_recovery`. |
| 44 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.observability.ux_recovery`. |
| 45 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 46 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |
| 47 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.mail.ux_recovery`. |
| 48 | `observability` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.observability.ux_recovery`. |
| 49 | `calendar` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.calendar.ux_recovery`. |
| 50 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j27.identity.ux_recovery`. |

## F. Done-state UX
The flow ends on the completed object or action, not a landing page. Reversible next steps are visible where policy allows, and audit transparency remains available without interrupting the job.

## Appendix A. Responsive recovery matrix

| UX-A001 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A002 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A003 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A004 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A005 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A006 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A007 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A008 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A009 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A010 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A011 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A012 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A013 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A014 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A015 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A016 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A017 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A018 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A019 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A020 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A021 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A022 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A023 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A024 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A025 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A026 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A027 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A028 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A029 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A030 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A031 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A032 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A033 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A034 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A035 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A036 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A037 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A038 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A039 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A040 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A041 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A042 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A043 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A044 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A045 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A046 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A047 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A048 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A049 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A050 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A051 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A052 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A053 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A054 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A055 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A056 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A057 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A058 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A059 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A060 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A061 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A062 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A063 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A064 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A065 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A066 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A067 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A068 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A069 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A070 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A071 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A072 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A073 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A074 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A075 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A076 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A077 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A078 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A079 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A080 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A081 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A082 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A083 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A084 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A085 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A086 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A087 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A088 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A089 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A090 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A091 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A092 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A093 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A094 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A095 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A096 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A097 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A098 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A099 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A100 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A101 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A102 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A103 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A104 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A105 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A106 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A107 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A108 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A109 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A110 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A111 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A112 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A113 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A114 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A115 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A116 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A117 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A118 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A119 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A120 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A121 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A122 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A123 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A124 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A125 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A126 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A127 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A128 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A129 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A130 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A131 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A132 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A133 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A134 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A135 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A136 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A137 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A138 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A139 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A140 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A141 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A142 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A143 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A144 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A145 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A146 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A147 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A148 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A149 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A150 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A151 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A152 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A153 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A154 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A155 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A156 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A157 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A158 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A159 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A160 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A161 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A162 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A163 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A164 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A165 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A166 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A167 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A168 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A169 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A170 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A171 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A172 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A173 | `calendar` `dual-context-freebusy` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A174 | `identity` `context-switch-claims` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A175 | `mail` `imip-invite-bridge` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A176 | `observability` `schedule-conflict-metrics` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
