# Payments Feature-Parity Matrix — Stripe / Adyen / Braintree — 2026-05-20

## Citation Anchor Block
1. Canonical audit sequence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3829-4235`.
2. Canonical machine constraints: `specs/master-plan-sequencing.json:704-868`.
3. Payments product source: `microservices/payments/PRD.md:86-125`, `microservices/payments/PRD.md:681-777`, `microservices/payments/PRD.md:1381-1499`.
4. Payments architecture source: `microservices/payments/ARCHITECTURE.md:44-135`, `microservices/payments/ARCHITECTURE.md:607-719`, `microservices/payments/ARCHITECTURE.md:1597-1720`.
5. Documentation-rigor source: `docs/standards/documentation-rigor.md:133-156` and `docs/standards/brief-template.md:1727-1855`.

## External Source Set
- Stripe Payments docs: https://docs.stripe.com/payments
- Stripe docs: https://docs.stripe.com/connect
- Stripe rate limits docs: https://docs.stripe.com/rate-limits
- Adyen Online Payments docs: https://docs.adyen.com/online-payments/
- Adyen Platforms split transaction docs: https://docs.adyen.com/platforms/online-payments/split-transactions/
- Adyen Platforms payouts docs: https://docs.adyen.com/platforms/quickstart-guide/payouts
- Adyen Platforms process-payments docs: https://docs.adyen.com/platforms/process-payments
- Braintree docs home: https://developer.paypal.com/braintree/docs/
- Braintree recurring billing overview: https://developer.paypal.com/braintree/articles/guides/recurring-billing/overview
- Braintree webhooks guide: https://developer.paypal.com/braintree/articles/control-panel/webhooks
- Braintree 3D Secure guide: https://developer.paypal.com/braintree/docs/guides/3d-secure/applying-3ds-to-transactions-and-verifications

## §1 Counterpart 1 — Stripe Capability Surface
| # | Stripe capability | Source cue | Oyatie parity hook |
|---:|---|---|---|
| S-01 | Online card acceptance. | Stripe Payments docs list online payments and Checkout. | `contracts/openapi-v1.yaml:44-79` charge API. |
| S-02 | Hosted Checkout. | Stripe Checkout quickstart linked from Payments docs. | Missing hosted checkout product surface. |
| S-03 | Embedded payment form/components. | Stripe Elements/Web Elements. | Missing frontend component contract. |
| S-04 | Advanced custom payment flows. | Stripe advanced integration and Payment Intents. | Partial charge state model. |
| S-05 | Payment Links. | Stripe no-code payment links. | Journey pay-link content exists but no API. |
| S-06 | Payment methods catalog. | Stripe payment-method docs. | PSP enum is narrow and provider-oriented. |
| S-07 | Dynamic payment methods. | Stripe dynamic payment method docs. | No payment-method ranking API. |
| S-08 | Link accelerated checkout. | Stripe Link docs. | No wallet/accelerated checkout feature. |
| S-09 | Terminal/in-person payments. | Stripe Terminal docs. | No POS/terminal contract. |
| S-10 | Subscriptions. | Stripe Billing quickstart. | `contracts/openapi-v1.yaml:280-300`. |
| S-11 | Subscription changes and proration. | Stripe subscription modification docs. | Not explicit in local subscription API. |
| S-12 | Recurring invoices. | Stripe recurring invoice docs. | No invoice contract. |
| S-13 | Hosted invoice page. | Stripe Invoicing docs. | No hosted invoice page. |
| S-14 | Custom invoicing API. | Stripe Invoicing integration. | No invoice API. |
| S-15 | SaaS platform support. | Stripe SaaS docs. | Sub-merchant onboarding partial. |
| S-16 | Marketplace support. | Stripe marketplace docs. | Strong journey and settlement docs. |
| S-17 | Connected account onboarding. | Stripe onboarding. | `contracts/openapi-v1.yaml:301-321` partial. |
| S-18 | Connected account capabilities. | Stripe account capabilities. | No capability management API. |
| S-19 | Connected account balances. | Stripe balance docs. | Ledger docs partial. |
| S-20 | Connected account payouts. | Stripe payouts. | `contracts/openapi-v1.yaml:204-242`. |
| S-21 | Split charges and transfers. | Stripe charges and transfers. | Settlement journeys partial; transfer naming conflict. |
| S-22 | Platform pricing tools. | Stripe platform pricing. | `cost-budget.md` partial, no admin API. |
| S-23 | Embedded connected-account components. | Stripe embedded components. | No embedded dashboard. |
| S-24 | Tax for Connect. | Stripe Tax with Connect. | PRD tax ledger only. |
| S-25 | Radar fraud for Connect. | Stripe Radar with Connect. | Fraud dashboards/runbooks partial. |
| S-26 | Disputes. | Stripe disputes docs. | `contracts/openapi-v1.yaml:243-279`. |
| S-27 | 3D Secure. | Stripe 3DS docs. | SCA mentioned, no explicit flow contract. |
| S-28 | Cards. | Stripe card payment docs. | Present through charge API, card details not exposed. |
| S-29 | Bank debits. | Stripe bank debit docs. | No ACH/SEPA debit contract. |
| S-30 | Bank redirects. | Stripe bank redirect docs. | No redirect payment-method flow. |
| S-31 | Wallets. | Stripe wallet docs. | No wallet abstraction. |
| S-32 | Crypto/onramp. | Stripe crypto docs. | Not present. |
| S-33 | Financial Connections. | Stripe Financial Connections docs. | No bank-account permission API. |
| S-34 | Global payouts. | Stripe global payouts. | Payout API present, global payout rails not explicit. |
| S-35 | API request IDs. | Stripe API request infrastructure. | Oyatie audit IDs present via event headers. |
| S-36 | Rate limiting and concurrency headers. | Stripe rate-limit docs. | Capacity model partial; no API response contract. |
| S-37 | Meter events. | Stripe Billing rate-limit docs mention meter events. | No usage/metering API. |
| S-38 | Search API and read allocation guidance. | Stripe rate-limit docs. | No search/reporting API. |
| S-39 | Data products/export. | Stripe Data Pipeline reference. | Audit-chain/dashboards partial, no customer export API. |
| S-40 | SDK ecosystem. | Stripe SDK docs linked. | `sdk-plan.md` partial, Rust-strict caveat. |
| S-41 | Webhooks. | Stripe webhooks docs linked. | `contracts/openapi-v1.yaml:322-349`; AsyncAPI. |
| S-42 | Refunds and partial refunds. | Stripe refunds docs. | refund API present. |
| S-43 | Payout rate constraints. | Stripe Create Payout API rate limit. | capacity model uses PSP ceilings. |
| S-44 | Platform account creation rates. | Stripe rate-limit docs. | no onboarding throughput contract. |
| S-45 | Load-test guidance. | Stripe rate-limit docs discourage direct sandbox load tests. | local benchmark docs need measured/mock disclosure. |
| S-46 | Atlas company incorporation. | Stripe Atlas linked from Payments docs. | not payments-core; no gap for current scope. |
| S-47 | Revenue recognition and billing analytics. | Stripe Billing ecosystem. | no explicit rev-rec feature. |
| S-48 | Identity/verification integration. | Stripe verification. | KYC/KYB bounded context partial. |
| S-49 | Fraud/risk rules. | Stripe Radar. | no first-class rule authoring API. |
| S-50 | Multi-party marketplace compliance. | Stripe Connect. | compliance docs strong but partner API incomplete. |

## §2 Counterpart 2 — Adyen Capability Surface
| # | Adyen capability | Source cue | Oyatie parity hook |
|---:|---|---|---|
| A-01 | Online payments overview. | Adyen Online Payments docs. | charge API present. |
| A-02 | Payment methods. | Adyen payment methods menu. | local payment rails partial. |
| A-03 | Integration checklist. | Adyen get-started docs. | onboarding guide partial. |
| A-04 | Sessions flow. | Adyen sessions flow. | no session API. |
| A-05 | Advanced flow. | Adyen advanced flow. | charge state partial. |
| A-06 | Result codes. | Adyen result codes. | no canonical PSP result enum. |
| A-07 | Checkout settings. | Adyen checkout settings. | no checkout configuration API. |
| A-08 | Go-live checklist. | Adyen go-live checklist. | missing six-context/OS/IaC gate. |
| A-09 | Capture. | Adyen capture docs. | `contracts/openapi-v1.yaml:122-146`. |
| A-10 | Capture failure reasons. | Adyen CAPTURE_FAILED reasons. | failure modes partial. |
| A-11 | Cancel. | Adyen cancel docs. | `contracts/openapi-v1.yaml:147-165`. |
| A-12 | Refund. | Adyen refund docs. | refund API present. |
| A-13 | Reversal. | Adyen reversal docs. | no separate reversal API. |
| A-14 | Authorization adjustment. | Adyen auth adjustment docs. | no auth-adjustment API. |
| A-15 | 3D Secure 2. | Adyen 3DS2 docs. | SCA partial. |
| A-16 | Native integration. | Adyen native docs. | no frontend/mobile SDK surface. |
| A-17 | Redirect integration. | Adyen redirect docs. | no redirect result flow. |
| A-18 | Web Drop-in. | Adyen Web Drop-in. | no hosted/drop-in UI. |
| A-19 | Web Component. | Adyen Web Components. | no component UI. |
| A-20 | iOS Drop-in. | Adyen iOS Drop-in. | no Swift frontend. |
| A-21 | iOS Component. | Adyen iOS Component. | no Swift frontend. |
| A-22 | Android Drop-in. | Adyen Android Drop-in. | no Kotlin frontend. |
| A-23 | Android Component. | Adyen Android Component. | no Kotlin frontend. |
| A-24 | Third-party authentication. | Adyen auth options. | no authentication delegation flow. |
| A-25 | Data-only 3DS. | Adyen data-only 3DS. | not explicit. |
| A-26 | Standalone authentication. | Adyen standalone authentication. | not explicit. |
| A-27 | Tokenization. | Adyen tokenization. | no vault/token API. |
| A-28 | Create tokens. | Adyen create token docs. | missing. |
| A-29 | Make token payments. | Adyen token payments. | missing. |
| A-30 | Manage tokens. | Adyen manage tokens. | missing. |
| A-31 | Forward payment details. | Adyen forwarding. | no forwarding API. |
| A-32 | Network tokenization. | Adyen network tokenization. | missing. |
| A-33 | Account Updater. | Adyen Account Updater. | missing. |
| A-34 | Real Time Account Updater. | Adyen RTAU. | missing. |
| A-35 | Batch Account Updater. | Adyen BAU. | missing. |
| A-36 | Auto Rescue. | Adyen Auto Rescue. | no retry/rescue product. |
| A-37 | SEPA direct debit. | Adyen SEPA. | no bank debit contract. |
| A-38 | Donations. | Adyen donations. | not scoped. |
| A-39 | Instant card funding. | Adyen instant funding. | no instant funding API. |
| A-40 | Online payouts. | Adyen payouts. | payout API present. |
| A-41 | Instant card payouts. | Adyen instant payouts. | no instant payout contract. |
| A-42 | Payouts in stages. | Adyen staged payouts. | settlement waterfall partial. |
| A-43 | Payout webhook. | Adyen payout webhook. | AsyncAPI partial. |
| A-44 | Payout to bank account. | Adyen payout docs. | payout API partial. |
| A-45 | Partial authorizations. | Adyen partial auth. | missing. |
| A-46 | Partial payments. | Adyen partial payments. | missing. |
| A-47 | Two-step checkout. | Adyen two-step checkout. | capture API partial. |
| A-48 | Surcharge. | Adyen surcharge. | no surcharge contract. |
| A-49 | Accessibility. | Adyen accessibility docs. | no hosted UI. |
| A-50 | Analytics/data tracking. | Adyen analytics. | dashboards partial. |
| A-51 | Card encryption with JWE. | Adyen JWE docs. | no client encryption contract. |
| A-52 | PCI DSS compliance. | Adyen PCI docs. | compliance docs present. |
| A-53 | PSD2 SCA compliance. | Adyen PSD2/SCA docs. | compliance partial. |
| A-54 | Platform split transactions. | Adyen split transaction docs. | settlement docs partial. |
| A-55 | Balance accounts. | Adyen platforms docs. | ledger docs partial. |
| A-56 | Payout schedules. | Adyen payouts docs. | no payout schedule API. |
| A-57 | Store-linked split profiles. | Adyen automatic split config. | missing. |
| A-58 | Account holder verification. | Adyen platforms onboarding. | sub-merchant onboarding partial. |
| A-59 | In-person split transactions. | Adyen in-person platform docs. | no POS contract. |
| A-60 | Sales-day settlement. | Adyen payouts docs. | no sales-day settlement config. |

## §3 Counterpart 3 — Braintree Capability Surface
| # | Braintree capability | Source cue | Oyatie parity hook |
|---:|---|---|---|
| B-01 | Integration guide. | Braintree docs home. | onboarding guide partial. |
| B-02 | Checkout UI Drop-in. | Braintree docs home lists Drop-in UI. | no hosted/drop-in UI. |
| B-03 | Hosted Fields. | Braintree docs home references hosted fields. | no component UI. |
| B-04 | Client authorization. | Braintree docs menu. | no client token API. |
| B-05 | Single-use token. | Braintree docs menu. | no nonce/token API. |
| B-06 | Customers. | Braintree docs menu. | no customer vault API. |
| B-07 | Payment methods. | Braintree docs menu. | no payment-method vault API. |
| B-08 | Transactions. | Braintree docs menu. | charge API present. |
| B-09 | ACH Direct Debit. | Braintree docs menu. | no ACH contract. |
| B-10 | Apple Pay. | Braintree docs menu. | no wallet API. |
| B-11 | Credit Cards. | Braintree docs menu. | charge API partial. |
| B-12 | Google Pay. | Braintree docs menu. | no wallet API. |
| B-13 | PayPal. | Braintree docs menu. | no PayPal enum/adapter. |
| B-14 | Venmo. | Braintree docs menu. | no Venmo enum/adapter. |
| B-15 | Secure Remote Commerce. | Braintree docs menu. | no SRC feature. |
| B-16 | 3D Secure. | Braintree 3D Secure docs. | SCA partial. |
| B-17 | Premium Fraud Management Tools. | Braintree docs menu. | fraud dashboards partial. |
| B-18 | Client SDK. | Braintree docs menu. | SDK plan partial. |
| B-19 | Disputes. | Braintree docs home says API dispute management. | dispute API present. |
| B-20 | Reports. | Braintree docs home reports. | dashboards partial, no report API. |
| B-21 | Webhooks. | Braintree webhook guide. | webhook receiver and AsyncAPI present. |
| B-22 | In-store payments. | Braintree docs home. | no POS contract. |
| B-23 | OAuth. | Braintree docs home. | no PSP OAuth/account linking API. |
| B-24 | Grant API. | Braintree docs home. | no grant/shared-vault API. |
| B-25 | Forward API. | Braintree docs home. | no forwarding API. |
| B-26 | Braintree Marketplace. | Braintree docs menu. | sub-merchant API partial, adapter absent. |
| B-27 | Recurring Billing. | Braintree recurring billing overview. | subscription API partial. |
| B-28 | Plans. | Recurring billing docs. | no plan API. |
| B-29 | Billing cycles. | Recurring billing docs. | not explicit. |
| B-30 | Subscription expiration. | Recurring billing docs. | not explicit. |
| B-31 | Vault-backed subscriptions. | Recurring billing docs. | vault API missing. |
| B-32 | Flexible subscription elements. | Recurring billing docs. | not explicit. |
| B-33 | Webhook subscription status changes. | Braintree webhook guide. | AsyncAPI subscription events partial. |
| B-34 | Disbursement webhooks. | Braintree webhook guide. | payout events partial. |
| B-35 | Dispute webhooks. | Braintree webhook guide. | dispute events partial. |
| B-36 | Marketplace sub-merchant status webhooks. | Braintree webhook guide. | sub-merchant events partial. |
| B-37 | Grant API webhooks. | Braintree webhook guide. | missing. |
| B-38 | Control Panel webhook management. | Braintree webhook guide. | no admin UI. |
| B-39 | Sandbox and production accounts. | Braintree docs/support text. | DemoTrial sandbox partial. |
| B-40 | Data migration. | Braintree recurring docs menu. | migration playbook present. |
| B-41 | Transaction lifecycle docs. | Braintree recurring docs menu. | charge lifecycle partial. |
| B-42 | Get paid guide. | Braintree docs menu. | payout API present. |
| B-43 | Currency support. | Braintree recurring docs menu. | multi-currency present in PRD. |
| B-44 | Payment method sharing. | Grant API docs cue. | missing. |
| B-45 | PCI-safe hosted fields/vault. | Braintree hosted fields/vault. | missing. |
| B-46 | PayPal ecosystem integration. | Braintree is PayPal service. | missing. |
| B-47 | Merchant account routing. | Braintree marketplace/merchant accounts. | PSP routing partial. |
| B-48 | Settlement/disbursement reporting. | Braintree reports/disbursements. | dashboards partial. |
| B-49 | Enterprise support paths. | Braintree support docs. | no support-tier product contract. |
| B-50 | Braintree adapter itself. | assignment top-3 counterpart. | missing in manifest/contracts/catalog. |

## §4 UNION-Coverage Matrix
| Capability | Stripe | Adyen | Braintree | UNION required | Oyatie payments has | Gap classification |
|---|---|---|---|---|---|---|
| U-001 Online card charges | yes | yes | yes | yes | yes, charge API | present |
| U-002 Hosted checkout | yes | yes | yes | yes | no | gap |
| U-003 Embedded card components | yes | yes | yes | yes | no | gap |
| U-004 Payment links | yes | partial | partial | yes | journey-only | gap |
| U-005 Payment sessions/intents | yes | yes | partial | yes | partial charge state | partial |
| U-006 Authorize and capture | yes | yes | yes | yes | yes | present |
| U-007 Void/cancel | yes | yes | yes | yes | yes | present |
| U-008 Refund | yes | yes | yes | yes | yes | present |
| U-009 Partial refund | yes | yes | yes | yes | amount semantics partial | partial |
| U-010 Disputes | yes | yes | yes | yes | yes | present |
| U-011 Evidence upload | yes | yes | yes | yes | yes | present |
| U-012 Chargeback workflow | yes | yes | yes | yes | runbooks | present |
| U-013 3D Secure 2 | yes | yes | yes | yes | mentioned, not fully contracted | partial |
| U-014 PSD2/SCA compliance | yes | yes | yes | yes | compliance partial | partial |
| U-015 Fraud scoring | yes | yes | yes | yes | dashboards/runbooks only | partial |
| U-016 Fraud rule authoring | yes | yes | yes | yes | no | gap |
| U-017 Token vault | yes | yes | yes | yes | no | gap |
| U-018 Network tokens | yes | yes | partial | yes | no | gap |
| U-019 Account updater | yes | yes | partial | yes | no | gap |
| U-020 Stored credential framework | yes | yes | yes | yes | no explicit | gap |
| U-021 Subscriptions | yes | yes | yes | yes | yes, partial | partial |
| U-022 Billing plans | yes | partial | yes | yes | no | gap |
| U-023 Proration | yes | partial | partial | yes | no | gap |
| U-024 Dunning/retry rescue | yes | yes | partial | yes | no | gap |
| U-025 Invoicing | yes | partial | partial | yes | no | gap |
| U-026 Hosted invoice payment | yes | partial | partial | yes | no | gap |
| U-027 Metered usage billing | yes | partial | no | yes | no | gap |
| U-028 Tax calculation | yes | partial | partial | yes | ledger-only | partial |
| U-029 Tax remittance/reporting | yes | partial | partial | yes | no | gap |
| U-030 Marketplace accounts | yes | yes | yes | yes | sub-merchant partial | partial |
| U-031 Account onboarding | yes | yes | yes | yes | partial | partial |
| U-032 Account verification | yes | yes | yes | yes | partial | partial |
| U-033 Account capabilities | yes | yes | partial | yes | no | gap |
| U-034 Split payments | yes | yes | yes | yes | settlement docs | partial |
| U-035 Platform fee allocation | yes | yes | yes | yes | cost docs partial | partial |
| U-036 Balance accounts | yes | yes | partial | yes | ledger partial | partial |
| U-037 Payouts | yes | yes | yes | yes | yes | present |
| U-038 Instant payouts | yes | yes | partial | yes | no | gap |
| U-039 Payout schedules | yes | yes | yes | yes | no | gap |
| U-040 Bank account verification | yes | yes | yes | yes | partial | partial |
| U-041 Multi-currency | yes | yes | yes | yes | yes | present |
| U-042 FX conversion | yes | yes | partial | yes | journey docs | partial |
| U-043 Local wallets | yes | yes | yes | yes | PSP list partial | partial |
| U-044 PayPal acceptance | partial | partial | yes | yes | no | gap |
| U-045 Venmo acceptance | no | no | yes | yes | no | gap |
| U-046 Apple Pay | yes | yes | yes | yes | no | gap |
| U-047 Google Pay | yes | yes | yes | yes | no | gap |
| U-048 ACH direct debit | yes | partial | yes | yes | no | gap |
| U-049 SEPA direct debit | yes | yes | partial | yes | no | gap |
| U-050 Bank redirects | yes | yes | partial | yes | no | gap |
| U-051 In-person POS | yes | yes | yes | yes | no | gap |
| U-052 Card present settlement | yes | yes | yes | yes | no | gap |
| U-053 Reader/device SDK | yes | yes | partial | yes | no | gap |
| U-054 Webhooks | yes | yes | yes | yes | yes | present |
| U-055 Webhook signature verification | yes | yes | yes | yes | partial | partial |
| U-056 Webhook replay | yes | yes | yes | yes | `backfill-replay.md` | present |
| U-057 Event contract | yes | yes | yes | yes | AsyncAPI | present |
| U-058 API idempotency | yes | yes | yes | yes | mentioned, not explicit | partial |
| U-059 API rate-limit response contract | yes | partial | partial | yes | capacity only | gap |
| U-060 Client SDKs | yes | yes | yes | yes | plan only | partial |
| U-061 Rust SDK | no official first-class client focus | no | no | Oyatie-specific | yes | additive |
| U-062 TypeScript SDK | yes | yes | yes | yes | plan only, policy caveat | partial |
| U-063 iOS SDK | yes | yes | yes | yes | plan only | partial |
| U-064 Android SDK | yes | yes | yes | yes | plan only | partial |
| U-065 Python SDK | yes | yes | yes | yes | plan conflicts with strict policy | gap/policy conflict |
| U-066 Reporting | yes | yes | yes | yes | dashboards only | partial |
| U-067 Search/export API | yes | yes | yes | yes | no | gap |
| U-068 Dashboard/admin console | yes | yes | yes | yes | no app UI | gap |
| U-069 PCI scope guidance | yes | yes | yes | yes | yes | present |
| U-070 PCI incident runbook | yes | yes | yes | yes | yes | present |
| U-071 KYC/KYB | yes | yes | yes | yes | partial | partial |
| U-072 AML/sanctions | yes | yes | partial | yes | runbooks/compliance | partial |
| U-073 Regulator export | partial | yes | partial | yes | runbooks | partial |
| U-074 Data residency | partial | yes | partial | yes | policy present | present |
| U-075 DSAR support | partial | partial | partial | yes for Oyatie privacy posture | PRD/dpia partial | partial |
| U-076 Audit log | yes | yes | yes | yes | audit-chain | present |
| U-077 Immutable evidence | no | no | no | Oyatie additive | audit-chain Merkle | additive |
| U-078 Tenant cell isolation | no | no | no | Oyatie additive | architecture cell section | additive |
| U-079 Cedar authorization | no | no | no | Oyatie additive | Cedar policies | additive |
| U-080 Multi-region failover | opaque | yes-ish | opaque | yes | multi-region doc | partial |
| U-081 PSP failover cascade | yes | yes | partial | yes | ADR-PAY-001/runbook | present |
| U-082 Smart routing | yes | yes | partial | yes | architecture/ADR | present |
| U-083 Cost-aware routing | partial | partial | no | yes | cost budget partial | partial |
| U-084 Health-based routing | yes | yes | partial | yes | failure modes/runbook | present |
| U-085 Surcharge support | partial | yes | partial | yes | no | gap |
| U-086 Partial authorization | partial | yes | partial | yes | no | gap |
| U-087 Partial payments | partial | yes | partial | yes | no | gap |
| U-088 Incremental auth | yes | yes | partial | yes | no | gap |
| U-089 Card-on-file mandates | yes | yes | yes | yes | no explicit | gap |
| U-090 Merchant category controls | yes | yes | yes | yes | no explicit | gap |
| U-091 Payout negative balance handling | yes | yes | yes | yes | partial in failure modes | partial |
| U-092 Reserve/holdback | yes | yes | yes | yes | escrow docs partial | partial |
| U-093 Escrow | partial | marketplace | marketplace | yes | journey docs | partial |
| U-094 Split settlement waterfall | yes | yes | yes | yes | journey docs | partial |
| U-095 Withholding ledger | partial | yes | partial | yes | journey docs | partial |
| U-096 Taxable transaction ledger | partial | partial | partial | yes | journey docs | present |
| U-097 Marketplace sub-merchant webhooks | yes | yes | yes | yes | AsyncAPI partial | partial |
| U-098 Disbursement webhooks | yes | yes | yes | yes | AsyncAPI partial | partial |
| U-099 Subscription webhooks | yes | yes | yes | yes | AsyncAPI partial | partial |
| U-100 Dispute webhooks | yes | yes | yes | yes | AsyncAPI partial | partial |
| U-101 Grant/shared vault | no | forwarding | yes | yes | no | gap |
| U-102 Forward API | no | yes | yes | yes | no | gap |
| U-103 OAuth account linking | yes | partial | yes | yes | no | gap |
| U-104 Hosted account dashboard | yes | yes | control panel | yes | no | gap |
| U-105 Account balance API | yes | yes | yes | yes | ledger partial | partial |
| U-106 Reconciliation reports | yes | yes | yes | yes | settlement dashboard | partial |
| U-107 Settlement file ingestion | yes | yes | yes | yes | not explicit | gap |
| U-108 Charge retry lifecycle | yes | yes | partial | yes | no explicit rescue | gap |
| U-109 Network retry/backoff guidance | yes | yes | yes | yes | runbooks partial | partial |
| U-110 Sandbox mode | yes | yes | yes | yes | DemoTrial sandbox | present |
| U-111 Production go-live checklist | yes | yes | yes | yes | missing canonical deployability | gap |
| U-112 Load test guidance | yes | partial | partial | yes | benchmark doc lacks measured proof | partial |
| U-113 Rate-limit increase process | yes | account-managed | account-managed | yes | not present | gap |
| U-114 Enterprise support | yes | yes | yes | yes | not present | gap |
| U-115 Dedicated tenant option | enterprise | enterprise | enterprise | yes | Paid intent partial | partial |
| U-116 Public status page integration | yes | yes | yes | yes | observability partial | partial |
| U-117 Multi-PSP adapter API | no, own rails | no, own rails | no, own rails | Oyatie additive | adapter trait | additive |
| U-118 Provider-BYOK | no direct counterpart | partial | partial | Oyatie additive | PRD mentions | additive |
| U-119 OpenTofu six-context IaC | no counterpart | no counterpart | no counterpart | Oyatie required | missing | Oyatie canonical gap |
| U-120 OCI Always Free DemoTrial | no counterpart | no counterpart | no counterpart | Oyatie required | missing | Oyatie canonical gap |

## §5 Capability Families Summary
| Family | UNION required count | Oyatie present | Oyatie partial | Oyatie missing | Notes |
|---|---:|---:|---:|---:|---|
| Core authorization/capture/refund/dispute | 12 | 8 | 4 | 0 | Core API and runbooks are strong. |
| Hosted and embedded checkout | 8 | 0 | 1 | 7 | No UI/drop-in/component surface. |
| Payment methods and wallets | 12 | 1 | 4 | 7 | PSP list exists; method-level API missing. |
| Vault/tokenization/account updater | 8 | 0 | 0 | 8 | Largest counterpart parity gap. |
| Billing/subscriptions/invoicing | 10 | 1 | 3 | 6 | Subscription exists; billing ecosystem thin. |
| Marketplace/platforms | 14 | 2 | 8 | 4 | Strong thesis, incomplete APIs. |
| Payouts/settlement/escrow | 14 | 3 | 9 | 2 | Journey docs strong; contracts incomplete. |
| Fraud/risk/compliance | 12 | 4 | 7 | 1 | Good runbooks; weak rule/product APIs. |
| Reporting/dashboard/export | 8 | 1 | 4 | 3 | Dashboards exist; user APIs missing. |
| SDK/developer experience | 7 | 1 | 4 | 2 | Rust reference strong; language-policy conflict. |
| Deployment/IaC/OS/Ops | 8 | 0 | 2 | 6 | Canonical Oyatie gaps are material. |
| Additive Oyatie governance | 7 | 5 | 2 | 0 | Cedar/audit-chain/cells are ahead. |

## §6 Headline Gap Analysis — Top 15 Missing Capabilities
| Rank | Missing capability | Why it matters | Suggested Oyatie hook |
|---:|---|---|---|
| 1 | Braintree adapter and PSP enum support. | The assigned union bar names Braintree, and local PRD/tier/migration docs already promise Braintree. | Add `catalog/oya-payments-adapter-braintree.yaml`, OpenAPI/proto enum entries, adapter trait mapping, and IP plan. |
| 2 | Hosted checkout. | All three counterparts provide a low-friction hosted collection path. | Add `checkout-session` bounded context or payments-owned hosted page contract. |
| 3 | Embedded components/Hosted Fields/Drop-in. | Braintree/Adyen/Stripe all reduce PCI exposure through controlled UI surfaces. | Add frontend/platform contracts under approved frontend lanes or route through developer-sdk. |
| 4 | Vault/token API. | Stored credentials are central to subscriptions, marketplace payouts, and SCA exemptions. | Add `payment-method-vault` BC with Cedar-guarded token lifecycle. |
| 5 | Network tokenization/account updater. | Counterparts reduce failed renewals and stale cards through card lifecycle maintenance. | Add token-refresh worker and PSP capability map. |
| 6 | PayPal/Venmo support. | Braintree's differentiator is PayPal/Venmo acceptance. | Add Braintree rails and payment-method enum family. |
| 7 | ACH/SEPA/bank debit support. | All counterpart sets include bank rails for lower-cost recurring/payment flows. | Extend payment-method model beyond card/wallet PSP names. |
| 8 | In-person/POS surface. | Stripe Terminal, Adyen POS, and Braintree in-store form the union bar. | Define POS as out-of-scope with canonical approval or add terminal adapter lane. |
| 9 | Account capability management. | Platforms need explicit merchant capability activation and verification states. | Extend sub-merchant API with capabilities and verification requirements. |
| 10 | Payout schedules and instant payout controls. | Platform sellers expect payout timing controls. | Add payout schedule API and policy-governed instant payout eligibility. |
| 11 | Billing plans/proration/dunning. | Subscriptions without plan/proration/rescue are not Stripe/Braintree-grade. | Add billing-plan and dunning state machine. |
| 12 | Invoicing. | Stripe has first-class invoicing and hosted invoice collection. | Add invoice issuance and hosted invoice payment link contracts. |
| 13 | Fraud rule authoring. | Dashboards alone do not equal Radar/Adyen Risk/Braintree fraud tools. | Add risk-rule policy surface backed by Cedar and model signals. |
| 14 | Reporting/search/export APIs. | Counterparts expose operational reporting beyond dashboards. | Add read-side reporting API with audit-chain export proofs. |
| 15 | Production go-live/deployment matrix. | Counterpart parity is meaningless if Oyatie cannot deploy in its own required contexts. | Fix OpenTofu, OS, OCI Always Free demo_trial, and context manifests before claiming maturity. |

## §7 Additive Surface — Oyatie Capabilities Not Directly Present in Counterparts
| Additive capability | Local evidence | Rationale |
|---|---|---|
| A-OYA-01 Cedar policy gates for charge/refund/payout/dispute/sub-merchant flows. | `policy/*.cedar` | Stronger explicit authorization provenance than counterpart docs expose. |
| A-OYA-02 Audit-chain Merkle evidence for regulated payment events. | `failure-modes.md:157-168`; `contracts/asyncapi-v1.yaml:153-164` | Adds tamper-evident control-plane posture. |
| A-OYA-03 Tenant cell eligibility and isolation. | `ARCHITECTURE.md:1597-1657` | Makes single-tenant/hyperscaler isolation explicit. |
| A-OYA-04 Provider-BYOK PSP posture. | `PRD.md:681-777` | Lets tenants bring PSP credentials under Oyatie governance. |
| A-OYA-05 Multi-PSP adapter abstraction. | `contracts/psp-adapter-trait.md:1-209` | Oyatie owns cross-PSP routing, while counterparts are mostly single-provider platforms. |
| A-OYA-06 Regulated journey pack depth. | `IP-journey-*.md` inventory | Strong domain journey coverage across payroll, marketplace, audit, and compliance. |
| A-OYA-07 Data-residency policy-as-code posture. | `policy/data-residency.md:1-154` | Makes jurisdiction rules explicit in service docs. |
| A-OYA-08 OpenSLO files per payment family. | `slos/*.openslo.yaml` | Gives structured service-level objectives in-repo. |
| A-OYA-09 OpenBao policy binding. | `iac/openbao/payments-policy.hcl:1-78` | Adds secret-governance detail outside counterpart API docs. |
| A-OYA-10 Regulator-pull runbooks. | `runbooks/kr-fss-audit-pull.md:1-282` | Better operational regulator posture than generic counterpart docs. |
| A-OYA-11 Cross-tenant settlement journey library. | `IP-journey-j142-cross-tenant-severance-payable.md`; `IP-journey-j145-cross-tenant-employer-pay-link.md` | Shows deep platform flows beyond checkout. |
| A-OYA-12 OCI Always Free requirement. | `ADR-0328:3491-3697` | Additive requirement, currently missing in payments implementation docs. |
| A-OYA-13 Six deployment contexts. | `master-plan-sequencing.json:704-746` | Additive deployability bar, currently missing locally. |
| A-OYA-14 Rust-strict backend rule. | `master-plan-sequencing.json:817-856` | Additive governance rule, mostly satisfied by current file scan. |
| A-OYA-15 Intern-buildability doctrine. | `documentation-rigor.md:133-156` | Stronger documentation bar than counterpart API docs alone. |

## §8 Parity Verdict
- Union-coverage result: partial.
- Oyatie payments is strong in domain-specific regulated-money-movement design, runbooks, failure modes, event contracts, and settlement journeys.
- Oyatie payments is weak against the union bar for hosted checkout, embedded PCI-minimizing UI, vault/token lifecycle, account updater, PayPal/Venmo/Braintree rails, bank debits, POS, invoice/billing depth, reporting/export, and production deployment evidence.
- Braintree is the highest-signal gap because it is named by the assignment, PRD/tier/migration docs, and counterpart bar, but omitted from manifest and contracts.
- Canonical Oyatie gaps are independent of counterpart gaps: OpenTofu context roots, supported OS manifest, and OCI Always Free DemoTrial must be fixed even though Stripe/Adyen/Braintree do not expose those as product features.

## §9 Implementation Hook Coverage Map
| Gap | Local artifact to update first | Acceptance evidence after repair |
|---|---|---|
| H-01 Braintree adapter | `manifest.json`, `contracts/openapi-v1.yaml`, `contracts/payments-v1.proto`, new catalog entry | Braintree appears in PSP enum, adapter catalog, migration playbook, and tenant_class adoption matrix. |
| H-02 Hosted checkout | PRD API section and OpenAPI contract | A checkout-session endpoint exists with idempotency, expiry, PSP selection, and audit events. |
| H-03 Embedded components | developer-sdk handoff and frontend-approved platform plan | Component boundary is owned outside backend or explicitly generated from contracts. |
| H-04 Vault/token lifecycle | new payment-method-vault bounded context | Create, attach, detach, rotate, revoke, and audit token actions are specified. |
| H-05 Account updater | token lifecycle worker plan | Card refresh and failed-renewal recovery states are documented. |
| H-06 PayPal/Venmo | Braintree adapter plan | PayPal/Venmo payment methods are distinct rails, not generic card aliases. |
| H-07 ACH/SEPA | payment-method taxonomy | Bank debit mandates, settlement delay, return codes, and retries are explicit. |
| H-08 POS/in-person | scope decision or terminal adapter plan | Either excluded with approval or added with reader/device and card-present settlement contracts. |
| H-09 Account capabilities | sub-merchant onboarding API | Verification, capability request, capability active, and disabled states are explicit. |
| H-10 Payout schedules | payout API and settlement worker plan | Manual, scheduled, instant, and held payout policies are represented. |
| H-11 Billing plans | subscription bounded context | Plan, price, period, proration, and expiration semantics are specified. |
| H-12 Dunning/retry | subscription renewal SLO and failure modes | Retry cadence, rescue outcome, and customer notification hooks are documented. |
| H-13 Invoicing | new invoice contract | Draft, issue, hosted pay, void, refund, and audit events are specified. |
| H-14 Fraud rules | policy/risk contract | Cedar or risk-rule authoring boundary is explicit and testable. |
| H-15 Reporting/export | OpenAPI read-side contract | Settlement, dispute, payout, fee, and audit export endpoints exist. |
| H-16 Admin console | ops-dashboard/developer portal handoff | Human operational actions are mapped to APIs and audit events. |
| H-17 Rate limit responses | OpenAPI error schema | Throttle headers, retry-after semantics, and PSP-limit reason codes are documented. |
| H-18 SDK provenance | `sdk-plan.md` and generated artifact policy | Non-Rust clients are generated/distributed artifacts, not backend implementation. |
| H-19 OpenTofu context roots | `iac/<context>/` directories | `tofu init` and `tofu plan` evidence exists for all six contexts. |
| H-20 OS support | `supported-oses.json` | Tier-1 OS/package/CI matrix is machine-readable. |
| H-21 OCI Always Free demo_trial | `iac/oci-guest/always-free/` and tenant_class adoption matrix | DemoTrial resource limits match Always Free quotas and fail before paid spillover. |
| H-22 Performance proof | benchmark harness | Raw p50/p95/p99/RPS artifacts replace planning targets. |
| H-23 Bounded-context truth | PRD, architecture, README, manifest | One count and one crate map are canonical. |
| H-24 API terminology | PRD and OpenAPI | Transfer/payout naming is reconciled with aliases or one canonical term. |
| H-25 Public/internal split | architecture classification and contract docs | Internal control surfaces and partner/public APIs have explicit boundaries. |
| H-26 Counterpart baseline | parity and benchmark docs | Stripe, Adyen, and Braintree remain first-class rows across feature, benchmark, and tier docs. |
| H-27 Compliance packs | compliance doc cleanup | Content-pass scaffold blocks are removed and replaced by payments-specific controls. |
| H-28 Cost model | cost-budget and cloud-billing handoff | PSP fees, infra cost, OCI free budget, and per-tenant attribution are connected. |
| H-29 Webhook semantics | AsyncAPI and webhook receiver API | Signature verification, replay, dedupe, and consumer delivery are explicit. |
| H-30 Settlement files | settlement domain docs | External PSP settlement file ingestion and reconciliation states are specified. |
