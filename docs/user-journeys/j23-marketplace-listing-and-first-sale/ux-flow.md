---
doc_class: UserJourneyUXFlow
shape: HowTo
journey_id: j23
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0244
  - ADR-0249
  - ADR-0263
  - ADR-0273
  - ADR-0297
  - ADR-0299
---

# UX Flow - Marketplace listing and first seller payout

## A. Assumptions
- Persona: Yejin Park.
- Locale: `ko-KR`.
- Tenant mode: `personal-seller`.
- Input: touch, keyboard, screen reader, voice, reduced motion.
- Accessibility target: WCAG 2.2 AAA on critical paths.

## B. Screen-by-screen flow

| # | Screen | Service | User action | System response |
|---:|---|---|---|---|
| 1 | `j23-screen-01` | marketplace | Continue `marketplace-listing-and-first-sale` step 1. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 2 | `j23-screen-02` | payments | Continue `marketplace-listing-and-first-sale` step 2. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 3 | `j23-screen-03` | identity | Continue `marketplace-listing-and-first-sale` step 3. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 4 | `j23-screen-04` | mail | Continue `marketplace-listing-and-first-sale` step 4. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 5 | `j23-screen-05` | community | Continue `marketplace-listing-and-first-sale` step 5. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 6 | `j23-screen-06` | marketplace | Continue `marketplace-listing-and-first-sale` step 6. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 7 | `j23-screen-07` | payments | Continue `marketplace-listing-and-first-sale` step 7. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 8 | `j23-screen-08` | identity | Continue `marketplace-listing-and-first-sale` step 8. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 9 | `j23-screen-09` | mail | Continue `marketplace-listing-and-first-sale` step 9. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 10 | `j23-screen-10` | community | Continue `marketplace-listing-and-first-sale` step 10. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 11 | `j23-screen-11` | marketplace | Continue `marketplace-listing-and-first-sale` step 11. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 12 | `j23-screen-12` | payments | Continue `marketplace-listing-and-first-sale` step 12. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 13 | `j23-screen-13` | identity | Continue `marketplace-listing-and-first-sale` step 13. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 14 | `j23-screen-14` | mail | Continue `marketplace-listing-and-first-sale` step 14. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 15 | `j23-screen-15` | community | Continue `marketplace-listing-and-first-sale` step 15. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 16 | `j23-screen-16` | marketplace | Continue `marketplace-listing-and-first-sale` step 16. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 17 | `j23-screen-17` | payments | Continue `marketplace-listing-and-first-sale` step 17. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 18 | `j23-screen-18` | identity | Continue `marketplace-listing-and-first-sale` step 18. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 19 | `j23-screen-19` | mail | Continue `marketplace-listing-and-first-sale` step 19. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 20 | `j23-screen-20` | community | Continue `marketplace-listing-and-first-sale` step 20. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 21 | `j23-screen-21` | marketplace | Continue `marketplace-listing-and-first-sale` step 21. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 22 | `j23-screen-22` | payments | Continue `marketplace-listing-and-first-sale` step 22. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 23 | `j23-screen-23` | identity | Continue `marketplace-listing-and-first-sale` step 23. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 24 | `j23-screen-24` | mail | Continue `marketplace-listing-and-first-sale` step 24. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 25 | `j23-screen-25` | community | Continue `marketplace-listing-and-first-sale` step 25. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 26 | `j23-screen-26` | marketplace | Continue `marketplace-listing-and-first-sale` step 26. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 27 | `j23-screen-27` | payments | Continue `marketplace-listing-and-first-sale` step 27. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 28 | `j23-screen-28` | identity | Continue `marketplace-listing-and-first-sale` step 28. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 29 | `j23-screen-29` | mail | Continue `marketplace-listing-and-first-sale` step 29. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 30 | `j23-screen-30` | community | Continue `marketplace-listing-and-first-sale` step 30. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 31 | `j23-screen-31` | marketplace | Continue `marketplace-listing-and-first-sale` step 31. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 32 | `j23-screen-32` | payments | Continue `marketplace-listing-and-first-sale` step 32. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 33 | `j23-screen-33` | identity | Continue `marketplace-listing-and-first-sale` step 33. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 34 | `j23-screen-34` | mail | Continue `marketplace-listing-and-first-sale` step 34. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 35 | `j23-screen-35` | community | Continue `marketplace-listing-and-first-sale` step 35. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 36 | `j23-screen-36` | marketplace | Continue `marketplace-listing-and-first-sale` step 36. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 37 | `j23-screen-37` | payments | Continue `marketplace-listing-and-first-sale` step 37. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 38 | `j23-screen-38` | identity | Continue `marketplace-listing-and-first-sale` step 38. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 39 | `j23-screen-39` | mail | Continue `marketplace-listing-and-first-sale` step 39. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 40 | `j23-screen-40` | community | Continue `marketplace-listing-and-first-sale` step 40. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 41 | `j23-screen-41` | marketplace | Continue `marketplace-listing-and-first-sale` step 41. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 42 | `j23-screen-42` | payments | Continue `marketplace-listing-and-first-sale` step 42. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 43 | `j23-screen-43` | identity | Continue `marketplace-listing-and-first-sale` step 43. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 44 | `j23-screen-44` | mail | Continue `marketplace-listing-and-first-sale` step 44. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 45 | `j23-screen-45` | community | Continue `marketplace-listing-and-first-sale` step 45. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 46 | `j23-screen-46` | marketplace | Continue `marketplace-listing-and-first-sale` step 46. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 47 | `j23-screen-47` | payments | Continue `marketplace-listing-and-first-sale` step 47. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 48 | `j23-screen-48` | identity | Continue `marketplace-listing-and-first-sale` step 48. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 49 | `j23-screen-49` | mail | Continue `marketplace-listing-and-first-sale` step 49. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 50 | `j23-screen-50` | community | Continue `marketplace-listing-and-first-sale` step 50. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 51 | `j23-screen-51` | marketplace | Continue `marketplace-listing-and-first-sale` step 51. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 52 | `j23-screen-52` | payments | Continue `marketplace-listing-and-first-sale` step 52. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 53 | `j23-screen-53` | identity | Continue `marketplace-listing-and-first-sale` step 53. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 54 | `j23-screen-54` | mail | Continue `marketplace-listing-and-first-sale` step 54. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 55 | `j23-screen-55` | community | Continue `marketplace-listing-and-first-sale` step 55. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 56 | `j23-screen-56` | marketplace | Continue `marketplace-listing-and-first-sale` step 56. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 57 | `j23-screen-57` | payments | Continue `marketplace-listing-and-first-sale` step 57. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 58 | `j23-screen-58` | identity | Continue `marketplace-listing-and-first-sale` step 58. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 59 | `j23-screen-59` | mail | Continue `marketplace-listing-and-first-sale` step 59. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 60 | `j23-screen-60` | community | Continue `marketplace-listing-and-first-sale` step 60. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 61 | `j23-screen-61` | marketplace | Continue `marketplace-listing-and-first-sale` step 61. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 62 | `j23-screen-62` | payments | Continue `marketplace-listing-and-first-sale` step 62. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 63 | `j23-screen-63` | identity | Continue `marketplace-listing-and-first-sale` step 63. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 64 | `j23-screen-64` | mail | Continue `marketplace-listing-and-first-sale` step 64. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 65 | `j23-screen-65` | community | Continue `marketplace-listing-and-first-sale` step 65. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 66 | `j23-screen-66` | marketplace | Continue `marketplace-listing-and-first-sale` step 66. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 67 | `j23-screen-67` | payments | Continue `marketplace-listing-and-first-sale` step 67. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 68 | `j23-screen-68` | identity | Continue `marketplace-listing-and-first-sale` step 68. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 69 | `j23-screen-69` | mail | Continue `marketplace-listing-and-first-sale` step 69. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 70 | `j23-screen-70` | community | Continue `marketplace-listing-and-first-sale` step 70. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |
| 71 | `j23-screen-71` | marketplace | Continue `marketplace-listing-and-first-sale` step 71. | Runs `seller-listing`, preserves tenant context, and shows localized recovery if needed. |
| 72 | `j23-screen-72` | payments | Continue `marketplace-listing-and-first-sale` step 72. | Runs `stripe-connect-payout`, preserves tenant context, and shows localized recovery if needed. |
| 73 | `j23-screen-73` | identity | Continue `marketplace-listing-and-first-sale` step 73. | Runs `seller-kyc-lite`, preserves tenant context, and shows localized recovery if needed. |
| 74 | `j23-screen-74` | mail | Continue `marketplace-listing-and-first-sale` step 74. | Runs `sale-receipt`, preserves tenant context, and shows localized recovery if needed. |
| 75 | `j23-screen-75` | community | Continue `marketplace-listing-and-first-sale` step 75. | Runs `seller-reputation`, preserves tenant context, and shows localized recovery if needed. |

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
| 1 | `marketplace` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.marketplace.ux_recovery`. |
| 2 | `payments` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.payments.ux_recovery`. |
| 3 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.identity.ux_recovery`. |
| 4 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.mail.ux_recovery`. |
| 5 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.community.ux_recovery`. |
| 6 | `marketplace` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.marketplace.ux_recovery`. |
| 7 | `payments` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.payments.ux_recovery`. |
| 8 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.identity.ux_recovery`. |
| 9 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.mail.ux_recovery`. |
| 10 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.community.ux_recovery`. |
| 11 | `marketplace` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.marketplace.ux_recovery`. |
| 12 | `payments` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.payments.ux_recovery`. |
| 13 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.identity.ux_recovery`. |
| 14 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.mail.ux_recovery`. |
| 15 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.community.ux_recovery`. |
| 16 | `marketplace` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.marketplace.ux_recovery`. |
| 17 | `payments` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.payments.ux_recovery`. |
| 18 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.identity.ux_recovery`. |
| 19 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.mail.ux_recovery`. |
| 20 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.community.ux_recovery`. |
| 21 | `marketplace` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.marketplace.ux_recovery`. |
| 22 | `payments` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.payments.ux_recovery`. |
| 23 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.identity.ux_recovery`. |
| 24 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.mail.ux_recovery`. |
| 25 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.community.ux_recovery`. |
| 26 | `marketplace` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.marketplace.ux_recovery`. |
| 27 | `payments` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.payments.ux_recovery`. |
| 28 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.identity.ux_recovery`. |
| 29 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.mail.ux_recovery`. |
| 30 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.community.ux_recovery`. |
| 31 | `marketplace` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.marketplace.ux_recovery`. |
| 32 | `payments` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.payments.ux_recovery`. |
| 33 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.identity.ux_recovery`. |
| 34 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.mail.ux_recovery`. |
| 35 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.community.ux_recovery`. |
| 36 | `marketplace` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.marketplace.ux_recovery`. |
| 37 | `payments` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.payments.ux_recovery`. |
| 38 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.identity.ux_recovery`. |
| 39 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.mail.ux_recovery`. |
| 40 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.community.ux_recovery`. |
| 41 | `marketplace` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.marketplace.ux_recovery`. |
| 42 | `payments` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.payments.ux_recovery`. |
| 43 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.identity.ux_recovery`. |
| 44 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.mail.ux_recovery`. |
| 45 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.community.ux_recovery`. |
| 46 | `marketplace` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.marketplace.ux_recovery`. |
| 47 | `payments` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.payments.ux_recovery`. |
| 48 | `identity` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.identity.ux_recovery`. |
| 49 | `mail` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.mail.ux_recovery`. |
| 50 | `community` rejects stale, replayed, or over-scoped request. | Safe retry or appeal path; event `j23.community.ux_recovery`. |

## F. Done-state UX
The flow ends on the completed object or action, not a landing page. Reversible next steps are visible where policy allows, and audit transparency remains available without interrupting the job.

## Appendix A. Responsive recovery matrix

| UX-A001 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A002 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A003 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A004 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A005 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A006 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A007 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A008 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A009 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A010 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A011 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A012 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A013 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A014 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A015 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A016 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A017 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A018 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A019 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A020 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A021 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A022 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A023 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A024 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A025 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A026 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A027 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A028 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A029 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A030 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A031 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A032 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A033 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A034 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A035 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A036 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A037 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A038 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A039 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A040 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A041 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A042 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A043 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A044 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A045 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A046 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A047 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A048 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A049 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A050 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A051 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A052 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A053 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A054 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A055 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A056 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A057 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A058 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A059 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A060 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A061 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A062 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A063 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A064 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A065 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A066 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A067 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A068 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A069 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A070 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A071 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A072 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A073 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A074 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A075 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A076 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A077 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A078 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A079 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A080 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A081 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A082 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A083 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A084 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A085 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A086 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A087 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A088 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A089 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A090 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A091 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A092 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A093 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A094 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A095 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A096 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A097 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A098 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A099 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A100 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A101 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A102 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A103 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A104 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A105 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A106 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A107 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A108 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A109 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A110 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A111 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A112 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A113 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A114 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A115 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A116 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A117 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A118 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A119 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A120 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A121 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A122 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A123 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A124 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A125 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A126 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A127 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A128 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A129 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A130 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A131 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A132 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A133 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A134 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A135 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A136 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A137 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A138 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A139 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A140 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A141 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A142 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A143 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A144 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A145 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A146 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A147 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A148 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A149 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A150 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A151 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A152 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A153 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A154 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A155 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A156 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A157 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A158 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A159 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A160 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A161 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A162 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A163 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A164 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A165 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A166 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A167 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A168 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A169 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A170 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A171 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A172 | `identity` `seller-kyc-lite` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A173 | `mail` `sale-receipt` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A174 | `marketplace` `seller-listing` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A175 | `payments` `stripe-connect-payout` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
| UX-A176 | `community` `seller-reputation` has stable layout, keyboard path, screen-reader label, localized recovery copy, and no default abuse-defence friction. |
