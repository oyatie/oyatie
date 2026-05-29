# Payments Tenant-Class Counterpart Deltas vs Counterparts — 2026-05-20

## Citation Anchor Block
1. Canonical deployment/IaC/OS/Rust/OCI constraints: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-4235`.
2. Machine-readable tier constraints: `specs/master-plan-sequencing.json:704-868`.
3. Payments tenant_class source: `microservices/payments/tenant_class adoption record:1-155`.
4. Payments product and architecture sources: `microservices/payments/PRD.md:86-125`, `microservices/payments/PRD.md:1381-1499`, `microservices/payments/ARCHITECTURE.md:44-135`, `microservices/payments/ARCHITECTURE.md:1597-1720`.
5. Substance and contradiction rules: `docs/standards/documentation-rigor.md:133-156` and `docs/standards/brief-template.md:1727-1855`.

## External Source Set
- Stripe Payments: https://docs.stripe.com/payments
- Stripe Connect: https://docs.stripe.com/connect
- Stripe rate limits: https://docs.stripe.com/rate-limits
- Adyen Online Payments: https://docs.adyen.com/online-payments/
- Adyen Platforms split transactions: https://docs.adyen.com/platforms/online-payments/split-transactions/
- Adyen Platforms payouts: https://docs.adyen.com/platforms/quickstart-guide/payouts
- Braintree docs home: https://developer.paypal.com/braintree/docs/
- Braintree recurring billing: https://developer.paypal.com/braintree/articles/guides/recurring-billing/overview
- Braintree webhooks: https://developer.paypal.com/braintree/articles/control-panel/webhooks

## §1 Tier Definitions in Oyatie — Payments-Specific
| Axis | tenant_class |
|---|---|---|---|---|
| T-01 Purpose | Sandbox/evaluation payment substrate. | Paid production baseline. | High-scale multi-region production. | Hyperscaler/single-tenant-capable substrate. |
| T-02 Current local source | `tenant_class adoption record:13-39`. | `tenant_class adoption record:41-71`. | `tenant_class adoption record:73-102`. | `tenant_class adoption record:104-128`. |
| T-03 Canonical correction | OCI Always Free demo_trial must equal Always Free when context is `guest-on-oci`. | Paid context can exceed Always Free. | Paid context with stronger SLOs. | Dedicated/hyperscaler context. |
| T-04 PSP mode | Current local DemoTrial says Stripe sandbox only. | Stripe, Adyen, Checkout.com, PayPal Braintree per local tenant_class doc. | Multi-PSP with higher scale. | Full multi-PSP plus dedicated tenant. |
| T-05 Braintree status | Not present in contracts. | Named in tier doc but absent in contracts. | Should be present by parity bar. | Required for top-3 union. |
| T-06 Cardholder data | Current DemoTrial says no real PAN. | PCI DSS L1 production PAN allowed. | PCI DSS L1 plus multi-region controls. | PCI DSS L1 plus single-tenant isolation. |
| T-07 Currency | Current DemoTrial USD only. | Multi-currency baseline. | Multi-currency with FX/settlement. | Global multi-currency with sovereign cell controls. |
| T-08 Throughput | Current local DemoTrial says 50 TPS. | Current local Paid says 5k TPS. | Higher target not fully formalized. | Hyperscaler target not measured. |
| T-09 Tenant ceiling | Current DemoTrial says 10 tenants. | Current Paid says 5k tenants. | Larger marketplace targets. | Single tenant or massive multi-tenant. |
| T-10 OCI compute | Must fit 4 OCPU/24GB. | Paid OCI allowed. | Paid OCI allowed. | Paid OCI/dedicated substrate required. |
| T-11 OCI storage | Must fit 200GB block plus 10GB object/archive. | Paid storage allowed. | Paid storage allowed. | Dedicated storage allowed. |
| T-12 OCI LB | Must fit 10 Mbps Always Free LB. | Paid LB allowed. | Paid LB allowed. | Dedicated edge allowed. |
| T-13 API surface | Charge/refund/payout/subscription/sub-merchant minimal. | Full production core APIs. | Advanced marketplace settlement. | Dedicated tenant, cell, and hyperscaler APIs. |
| T-14 Hosted checkout | Missing today. | Should exist for parity. | Should exist. | Should exist plus tenant customization. |
| T-15 Embedded components | Missing today. | Should exist. | Should exist. | Should exist. |
| T-16 Vault/tokenization | Missing today. | Required for production parity. | Required. | Required at scale. |
| T-17 Account updater | Missing today. | Desirable. | Required. | Required. |
| T-18 Subscriptions | Partial API today. | Production subscriptions. | Proration/dunning. | Large-scale billing. |
| T-19 Marketplace split | Journey docs today. | Production splits. | Complex waterfalls. | Dedicated account sharding. |
| T-20 Payouts | API present. | Production payout scheduling. | Instant/staged payouts. | Dedicated payout rails. |
| T-21 Fraud/risk | Runbooks/dashboards. | Rule authoring expected. | Adaptive risk expected. | Dedicated risk controls. |
| T-22 Compliance | Docs/runbooks. | PCI L1 production. | Regional overlays. | Sovereign cell/compliance. |
| T-23 Observability | Dashboards. | SLO-backed alerts. | Multi-region tracing. | Dedicated tenant telemetry. |
| T-24 OpenTofu | Missing. | Missing. | Missing. | Missing. |
| T-25 OS matrix | Missing. | Missing. | Missing. | Missing. |
| T-26 Rust build proof | Missing source tree. | Missing source tree. | Missing source tree. | Missing source tree. |
| T-27 Current readiness | Design-only sandbox partial. | Not deployable to canonical bar. | Not deployable to canonical bar. | Not deployable to canonical bar. |
| T-28 Tier delta headline | DemoTrial local doc conflicts with OCI Always Free. | Paid product claims exceed current contracts. | Paid needs measured scale and parity features. | Paid is aspiration without deployability proof. |

## §2 Counterpart Tier Mapping
| Counterpart | Tier/map | Axis emphasized | Oyatie mapping |
|---|---|---|---|
| Stripe | Standard account/sandbox | Start accepting payments, hosted checkout, Payment Links, core APIs. | Oyatie DemoTrial/Paid should match core payment acceptance. |
| Stripe | platform | Marketplace, connected accounts, onboarding, payouts, balances. | Oyatie Paid/Paid should map to parity. |
| Stripe | Billing/Tax/Radar add-ons | Subscriptions, invoices, tax, fraud. | Oyatie paid missing several add-on equivalents. |
| Stripe | Enterprise/custom limits | Higher rate limits, support, account-specific scaling. | Oyatie Paid/Paid target. |
| Stripe | Dedicated/special arrangements | Account-specific approval and support. | Oyatie Paid analog. |
| Adyen | Test account | Integration and sandbox validation. | Oyatie DemoTrial analog. |
| Adyen | Online Payments | Checkout API, sessions, components, payment methods. | Oyatie Paid core acceptance analog. |
| Adyen | Adyen for Platforms | Split transactions, balance accounts, onboarding, payouts. | Oyatie Paid/Paid marketplace analog. |
| Adyen | Risk/compliance features | 3DS/SCA, risk management, PCI guidance. | Oyatie paid risk/compliance analog. |
| Adyen | Enterprise global processing | Multi-region/global merchant processing. | Oyatie Paid/Paid analog. |
| Braintree | Sandbox | Test payment methods and recurring flows. | Oyatie DemoTrial analog. |
| Braintree | Standard gateway | Transactions, payment methods, vault, webhooks. | Oyatie Paid core gateway analog. |
| Braintree | Recurring Billing | Plans, customers, subscriptions, vault-backed billing. | Oyatie Paid/Paid subscription analog. |
| Braintree | Marketplace | Sub-merchants, disbursements, marketplace webhooks. | Oyatie Paid/Paid platform analog. |
| Braintree | Enterprise/PayPal ecosystem | PayPal/Venmo, fraud tools, support. | Oyatie Paid/Paid analog, currently missing. |
| Cross-counterpart | Free/sandbox | Controlled testing, low or no production exposure. | Oyatie DemoTrial, but OCI Always Free demo_trial has stricter Always Free resource cap. |
| Cross-counterpart | Production baseline | Core payment acceptance and webhooks. | Oyatie Paid. |
| Cross-counterpart | Platform scale | Marketplace, payouts, account verification, disputes. | Oyatie Paid. |
| Cross-counterpart | Enterprise/dedicated | Custom limits, support, isolation, advanced compliance. | Oyatie Paid. |
| Oyatie-specific | Canonical deployability | Six contexts, OpenTofu, OS matrix, Rust, OCI. | Required across all tiers, currently missing/partial. |

## §3 Per-Oyatie-Tier Delta Tables
### §3.1 DemoTrial Tier
| Feature | Oyatie DemoTrial | Stripe equivalent | Adyen equivalent | Braintree equivalent | Gap classification |
|---|---|---|---|---|---|
| BR-01 Sandbox payments | Stripe sandbox only in local tenant_class doc | Stripe sandbox | Adyen test | Braintree sandbox | partial; too Stripe-specific |
| BR-02 OCI Always Free | missing in tier doc | no equivalent | no equivalent | no equivalent | Oyatie canonical gap |
| BR-03 Charge API | present | PaymentIntent/Checkout | Sessions/payments | Transaction sale | parity partial |
| BR-04 Hosted checkout | missing | Checkout | Drop-in | Drop-in | gap |
| BR-05 Embedded fields | missing | Elements | Components | Hosted Fields | gap |
| BR-06 Payment links | journey-only | Payment Links | partial | partial | gap |
| BR-07 Authorize/capture | present | present | present | present | parity |
| BR-08 Void/cancel | present | present | present | present | parity |
| BR-09 Refund | present | present | present | present | parity |
| BR-10 Partial refund | not explicit | present | present | present | partial |
| BR-11 Disputes | present | present | present | present | parity |
| BR-12 3DS/SCA | mentioned | present | present | present | partial |
| BR-13 Fraud dashboard | dashboards/runbooks | Radar | Risk tools | fraud tools | partial |
| BR-14 Fraud rule authoring | missing | Radar rules | risk rules | premium tools | gap |
| BR-15 Vault | missing | payment methods | tokenization | Vault | gap |
| BR-16 Token payment | missing | saved PMs | token payments | vaulted PM | gap |
| BR-17 Network tokenization | missing | present | present | partial | gap |
| BR-18 Account updater | missing | present | present | partial | gap |
| BR-19 Subscriptions | partial | Billing | recurring | recurring billing | partial |
| BR-20 Billing plan | missing | prices/plans | partial | plans | gap |
| BR-21 Invoices | missing | Invoicing | partial | partial | gap |
| BR-22 Marketplace onboarding | partial | onboarding | platform onboarding | marketplace | partial |
| BR-23 Braintree adapter | missing | not applicable | not applicable | required | gap |
| BR-24 Payouts | present | payouts | payouts | disbursements | parity partial |
| BR-25 Payout schedule | missing | present | present | present | gap |
| BR-26 Multi-currency | USD only in local tenant_class | global | global | multi-currency | DemoTrial deliberately below |
| BR-27 Bank debit | missing | bank debits | SEPA | ACH | gap |
| BR-28 Wallets | missing | wallets | wallets | PayPal/Venmo | gap |
| BR-29 POS | missing | Terminal | POS | in-store | gap |
| BR-30 Webhooks | present | webhooks | webhooks | webhooks | parity |
| BR-31 SDKs | plan only | SDKs | SDKs | SDKs | partial |
| BR-32 PCI posture | no real PAN | PCI docs | PCI docs | PCI-safe tools | acceptable for sandbox |
| BR-33 OpenTofu context | missing | no equivalent | no equivalent | no equivalent | Oyatie canonical gap |
| BR-34 OS manifest | missing | no equivalent | no equivalent | no equivalent | Oyatie canonical gap |
| BR-35 Rust build proof | missing | no equivalent | no equivalent | no equivalent | Oyatie canonical gap |
| BR-36 Tenant ceiling | 10 local doc, 5 target for OCI free | account-specific | account-specific | account-specific | needs context split |
| BR-37 Throughput | 50 TPS local doc, 15 rps OCI free target | sandbox 25 ops/s Stripe | account-specific | account-specific | context conflict |
| BR-38 Cost cap | missing | no equivalent | no equivalent | no equivalent | must add zero-cost cap |
| BR-39 Regulator runbooks | present | support docs | support docs | support docs | ahead |
| BR-40 DemoTrial verdict | product partial, deployability failing | stronger hosted/vault | stronger components/tokenization | stronger vault/PayPal | catch-up |

### §3.2 Paid Tier
| Feature | Oyatie Paid | Stripe equivalent | Adyen equivalent | Braintree equivalent | Gap classification |
|---|---|---|---|---|---|
| SI-01 Production card charges | intended | standard payments | online payments | gateway transactions | parity target |
| SI-02 Hosted checkout | missing | Checkout | Drop-in | Drop-in | gap |
| SI-03 Embedded checkout | missing | Elements | Components | Hosted Fields | gap |
| SI-04 Payment links | missing | Payment Links | partial | partial | gap |
| SI-05 Capture/cancel/refund | present | present | present | present | parity |
| SI-06 Disputes/evidence | present | present | present | present | parity |
| SI-07 3DS/SCA | partial | present | present | present | partial |
| SI-08 Fraud rules | missing | Radar | Risk | Premium Fraud | gap |
| SI-09 Token vault | missing | payment methods | tokenization | Vault | gap |
| SI-10 Account updater | missing | present | present | partial | gap |
| SI-11 Subscriptions | partial | Billing | recurring | recurring billing | partial |
| SI-12 Plans/proration | missing | present | partial | plans | gap |
| SI-13 Dunning/rescue | missing | billing recovery | Auto Rescue | recurring workflows | gap |
| SI-14 Invoicing | missing | present | partial | partial | gap |
| SI-15 Tax calculation | partial | Stripe Tax | partial | partial | partial |
| SI-16 Marketplace onboarding | partial | | Platforms | Marketplace | partial |
| SI-17 Connected account capability | missing | capabilities | account holder checks | sub-merchant checks | gap |
| SI-18 Split payments | partial | charges/transfers | split transactions | marketplace | partial |
| SI-19 Balance accounts | partial ledger | balances | balance accounts | merchant accounts | partial |
| SI-20 Payouts | present | payouts | payouts | disbursements | parity |
| SI-21 Instant payout | missing | present | present | partial | gap |
| SI-22 Payout schedule | missing | present | present | present | gap |
| SI-23 Multi-currency | intended | present | present | present | parity target |
| SI-24 PayPal/Venmo | missing | partial | partial | strong | gap |
| SI-25 Braintree adapter | named in tier doc but absent | n/a | n/a | required | hard gap |
| SI-26 ACH/SEPA | missing | present | present | present | gap |
| SI-27 POS | missing | Terminal | POS | in-store | gap |
| SI-28 Webhooks | present | present | present | present | parity |
| SI-29 Webhook replay | present | present | present | present | parity |
| SI-30 Reporting | dashboards only | reports/export | analytics | reports | partial |
| SI-31 Admin console | missing | dashboard | customer area | control panel | gap |
| SI-32 SDKs | plan only | present | present | present | partial |
| SI-33 PCI L1 | intended | PCI docs | PCI docs | PCI tooling | parity target |
| SI-34 KYC/KYB | partial | verification | platform onboarding | marketplace | partial |
| SI-35 OpenTofu | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| SI-36 OS manifest | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| SI-37 Context IaC | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| SI-38 Rust build | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| SI-39 Paid verdict | core APIs useful, counterpart platform features incomplete | strong | strong | vault/PayPal strong | catch-up |
| SI-40 Required before Paid claim | Braintree, vault, hosted checkout, OpenTofu, OS matrix | source aggregate | source aggregate | source aggregate | blocked |

### §3.3 Paid Tier
| Feature | Oyatie Paid | Stripe equivalent | Adyen equivalent | Braintree equivalent | Gap classification |
|---|---|---|---|---|---|
| GO-01 High-scale charge API | target only | custom limits | enterprise processing | enterprise gateway | unmeasured |
| GO-02 Multi-region active-active | doc partial | opaque | enterprise | opaque | partial |
| GO-03 PSP failover cascade | present in ADR/runbook | multi-acquirer internal | Adyen routing | limited | parity/ahead |
| GO-04 Cost-aware routing | partial | platform pricing | split fees | merchant routing | partial |
| GO-05 Hosted checkout at scale | missing | Checkout | Drop-in | Drop-in | gap |
| GO-06 Component SDKs | missing | Elements | Components | Hosted Fields | gap |
| GO-07 Vault/token lifecycle | missing | strong | strong | strong | gap |
| GO-08 Network tokenization | missing | strong | strong | partial | gap |
| GO-09 Account updater | missing | strong | strong | partial | gap |
| GO-10 Advanced subscriptions | partial | strong | partial | strong | gap |
| GO-11 Invoicing/rev-rec | missing | strong | partial | partial | gap |
| GO-12 Tax automation | partial | strong | partial | partial | partial |
| GO-13 Platform onboarding scale | partial | | Platforms | Marketplace | partial |
| GO-14 Split settlement waterfall | journey docs | | Platforms split | Marketplace | partial |
| GO-15 Reserves/holdbacks | partial | platform controls | platform controls | marketplace | partial |
| GO-16 Instant/staged payout | missing | present | present | partial | gap |
| GO-17 Fraud adaptive controls | dashboard only | Radar | Risk | Premium Fraud | gap |
| GO-18 Regulator exports | runbooks | reports | reports | reports | partial/ahead |
| GO-19 Tenant cell isolation | architecture | no direct exposed | no direct exposed | no direct exposed | ahead |
| GO-20 Audit-chain | architecture/events | no direct exposed | no direct exposed | no direct exposed | ahead |
| GO-21 Cedar policy gates | present | no direct exposed | no direct exposed | no direct exposed | ahead |
| GO-22 SDK ecosystem | plan only | mature | mature | mature | catch-up |
| GO-23 Dashboard/admin UI | missing | mature | mature | mature | gap |
| GO-24 Reporting/search API | missing | mature | analytics | reports | gap |
| GO-25 POS/in-person | missing | Terminal | POS | in-store | gap |
| GO-26 PayPal/Venmo | missing | partial | partial | strong | gap |
| GO-27 Braintree adapter | missing | n/a | n/a | required | hard gap |
| GO-28 OpenTofu six contexts | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| GO-29 OS matrix | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| GO-30 OCI paid/Always Free split | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| GO-31 Measured benchmarks | missing | published limits | account-specific | account-specific | unmeasured |
| GO-32 Load-test harness | missing | guidance exists | not public universal | not public universal | gap |
| GO-33 P99 200ms claim | conflicting docs | no public universal | no public universal | no public universal | local conflict |
| GO-34 Webhook 25k rps | target only | no public universal | no public universal | no public universal | unmeasured |
| GO-35 Tenant 50k target | target only | enterprise | platform | marketplace | unmeasured |
| GO-36 Multi-currency/FX | journey docs | present | present | present | partial |
| GO-37 Bank debits | missing | present | present | present | gap |
| GO-38 Compliance regional overlays | strong docs | support | support | support | parity/ahead |
| GO-39 Paid verdict | ahead in governance, behind in product/API parity and deployability | mature | mature | mature | catch-up |
| GO-40 Required before Paid claim | measured scale plus Paid gaps fixed | aggregate | aggregate | aggregate | blocked |

### §3.4 Paid Tier
| Feature | Oyatie Paid | Stripe equivalent | Adyen equivalent | Braintree equivalent | Gap classification |
|---|---|---|---|---|---|
| PL-01 Dedicated tenant/cell | architecture target | enterprise/custom | enterprise | enterprise | parity/ahead target |
| PL-02 Hyperscaler p99 150ms | target only | no public universal | no public universal | no public universal | unmeasured |
| PL-03 25k-40k sustained rps | target only | custom limit required | enterprise | enterprise | unmeasured |
| PL-04 100k+ webhook rps | target only | no public universal | no public universal | no public universal | unmeasured |
| PL-05 Single-tenant compliance | architecture target | enterprise support | enterprise | enterprise | partial |
| PL-06 Sovereign cell controls | architecture/policy | not directly exposed | not directly exposed | not directly exposed | ahead target |
| PL-07 Audit-chain proof | planned/current events | not directly exposed | not directly exposed | not directly exposed | ahead |
| PL-08 Cedar governance | present | not direct | not direct | not direct | ahead |
| PL-09 PSP BYOK | PRD | partial | partial | partial | ahead/partial |
| PL-10 Multi-PSP routing | adapter trait | no | no | no | additive |
| PL-11 Braintree adapter | missing | n/a | n/a | required | hard gap |
| PL-12 Hosted checkout | missing | mature | mature | mature | gap |
| PL-13 Embedded checkout | missing | mature | mature | mature | gap |
| PL-14 Vault | missing | mature | mature | mature | gap |
| PL-15 Tokenization/account updater | missing | mature | mature | partial | gap |
| PL-16 Billing/invoice platform | partial/missing | mature | partial | recurring | gap |
| PL-17 Tax automation | partial | mature | partial | partial | partial |
| PL-18 Risk platform | dashboards only | Radar | Risk | Premium Fraud | gap |
| PL-19 Admin portal | missing | dashboard | customer area | control panel | gap |
| PL-20 Reporting/export | partial | mature | mature | mature | gap |
| PL-21 POS | missing | Terminal | POS | in-store | gap |
| PL-22 PayPal/Venmo | missing | partial | partial | strong | gap |
| PL-23 Bank rails | missing | present | present | ACH | gap |
| PL-24 Account capabilities | missing | mature | mature | marketplace | gap |
| PL-25 Payout scheduling/instant | missing | mature | mature | partial | gap |
| PL-26 Settlement file ingestion | missing | mature | mature | reports | gap |
| PL-27 Multi-region proof | doc only | enterprise | enterprise | enterprise | partial |
| PL-28 RTO proof | target only | account-specific | account-specific | account-specific | unmeasured |
| PL-29 RPO proof | ledger invariant only | account-specific | account-specific | account-specific | unmeasured |
| PL-30 OpenTofu | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| PL-31 OS matrix | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| PL-32 Rust build proof | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| PL-33 Sigstore IaC modules | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| PL-34 OCI paid/free tier split | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| PL-35 Per-context state backend | missing | no equivalent | no equivalent | no equivalent | Oyatie gap |
| PL-36 Paid support model | missing | enterprise support | enterprise support | support | gap |
| PL-37 Pricing model | partial cost budget | pricing tools | pricing | pricing | partial |
| PL-38 SLA contract | SLO docs only | enterprise SLA | enterprise SLA | enterprise | partial |
| PL-39 Paid verdict | governance-heavy aspiration, product/deployment proof missing | mature platform | mature platform | mature gateway | catch-up |
| PL-40 Required before Paid claim | all P1/P2 gaps fixed plus measured scale evidence | aggregate | aggregate | aggregate | blocked |

## §4 OCI Always Free demo_trial = Always Free Reconciliation
| OCI Always Free demo_trial item | Required by ADR-0328 | Current payments tier | Reconciliation |
|---|---|---|---|
| O-01 Context identity | `guest-on-oci` subprofile, not separate strategy. | No `guest-on-oci` tier split. | Add explicit `guest-on-oci/always-free` tier overlay. |
| O-02 IaC path | `iac/oci-guest/always-free/`. | Missing. | Create OpenTofu Always Free root. |
| O-03 Compute | 4 OCPU / 24GB Ampere A1 plus two E2.1.Micro. | DemoTrial says 3 nodes, 8 vCPU, 32GB each. | Current DemoTrial exceeds Always Free; split OCI Always Free demo_trial from generic DemoTrial. |
| O-04 Storage | 200GB block, 10GB object, 10GB archive. | DemoTrial says PostgreSQL 1.92TiB class and 500GB node storage. | Reduce OCI Always Free demo_trial storage or make paid required. |
| O-05 Database | Two Autonomous DBs 20GB each. | Local tier doc uses PostgreSQL class. | Map sandbox ledger/idempotency to ADB or document paid DB requirement. |
| O-06 Load balancer | 10 Mbps. | Local tier no LB cap. | Add traffic and webhook rate limits. |
| O-07 Egress | 10TB/month with 8TB alert. | No egress alert. | Add cloud-billing zero-cost alerts. |
| O-08 Cost events | cloud-billing emits zero-cost/quota events. | Cost budget lacks OCI free events. | Add DemoTrial OCI cost telemetry. |
| O-09 PSP mode | Evaluation/sandbox/dev. | DemoTrial says sandbox-only Stripe. | Align; prohibit real PAN and production PSP. |
| O-10 Tenant count | Must fit free compute/storage. | DemoTrial says 10 tenants. | Set OCI Always Free demo_trial to 5 tenants until measured. |
| O-11 Throughput | Must fit 10Mbps/LB/4 OCPU. | DemoTrial says 50 TPS. | Set OCI Always Free demo_trial to 15 rps target until measured. |
| O-12 Webhooks | Must fit free compute. | No OCI-specific target. | Cap at 75 rps target. |
| O-13 Observability | Must not spill to paid monitoring. | Dashboards not context-costed. | Use free-compatible telemetry profile. |
| O-14 Secrets | Must fit free profile. | OpenBao policy generic. | Context-bind OpenBao or OCI Vault free-safe equivalent through cloud-secrets. |
| O-15 State backend | OCI Object Storage + ADB state. | Terraform GCS backend. | Replace with OCI free-safe state backend. |
| O-16 Module signing | Sigstore required. | Missing. | Add signed module metadata. |
| O-17 Fail plan | Must fail before paid spillover. | Missing. | Add hard quota guard. |
| O-18 DemoTrial naming | DemoTrial equals Always Free for OCI. | DemoTrial currently generic paid-ish sandbox. | Rename generic sandbox tier or add context overlay. |
| O-19 Paid boundary | Any real PAN, production PSP, >free compute/storage, or >10Mbps LB requires paid. | Not explicit. | Add paid escalation criteria. |
| O-20 Verdict | OCI Always Free demo_trial is not coherent today. | Evidence above. | P1 canonical gap. |

## §5 Findings — Per-Tier Ahead / Parity / Catch-Up
| Tier | Ahead areas | Parity areas | Catch-up areas | Final classification |
|---|---|---|---|---|
| DemoTrial | Cedar/audit-chain/runbooks exceed counterpart sandbox docs. | Charge/refund/dispute basics partially match. | Hosted checkout, vault, Braintree, OCI Always Free, OpenTofu, OS matrix. | catch-up |
| Paid | Multi-PSP abstraction and policy gates are additive. | Core charges/refunds/payouts/disputes align. | Hosted UI, vault, account updater, billing depth, Braintree adapter, deployability. | catch-up |
| Paid | Cell isolation, regulated journeys, audit-chain evidence are ahead if implemented. | Marketplace/split settlement intent matches Stripe/Adyen/Braintree categories. | Measured scale, admin/reporting UI, POS, wallets, bank rails, risk rules, OpenTofu/OS. | catch-up |
| Paid | Sovereign cell and policy-governed dedicated tenant story can exceed counterpart public docs. | Enterprise/dedicated support intent maps to counterpart enterprise tiers. | Almost all performance, support, deployment, and Braintree/product-surface evidence is unproven. | catch-up |

## §6 Tier-Specific Remediation Queue
| Priority | Tier | Remediation | Evidence trigger |
|---:|---|---|---|
| 1 | DemoTrial | Add OCI Always Free overlay and `iac/oci-guest/always-free/`. | `ADR-0328:3491-3697`; missing path. |
| 2 | DemoTrial | Cap OCI Always Free demo_trial compute/storage/LB/egress to Always Free. | `tenant_class adoption record:13-39` conflicts with OCI limits. |
| 3 | DemoTrial | Add hosted checkout or explicitly defer with approved scope. | Stripe/Adyen/Braintree docs. |
| 4 | DemoTrial | Add vault/token minimal sandbox API or mark unsupported. | all counterparts. |
| 5 | Paid | Add Braintree adapter and PSP enum entries. | `PRD.md:1381-1410`; `manifest.json:94-102`. |
| 6 | Paid | Add production payment-method taxonomy for PayPal/Venmo/ACH/SEPA/wallets. | union matrix gaps. |
| 7 | Paid | Add payout schedules and instant payout eligibility. | counterpart platform docs. |
| 8 | Paid | Add account capabilities and verification-state API. | Stripe Connect/Adyen Platforms/Braintree Marketplace. |
| 9 | Paid | Add measured benchmark harness and per-context target evidence. | performance report target-only disclosure. |
| 10 | Paid | Add reporting/export API, not only dashboards. | counterpart reports/search features. |
| 11 | Paid | Add fraud rule authoring surface. | Stripe Radar/Adyen Risk/Braintree Fraud. |
| 12 | Paid | Add POS/in-person decision. | all three counterparts have in-person path. |
| 13 | Paid | Add single-tenant support contract and SLO. | architecture cell claims. |
| 14 | Paid | Add dedicated context modules and state backends. | missing OpenTofu context roots. |
| 15 | All | Add `supported-oses.json` and Tier-1 CI lanes. | ADR-0328 §D-17. |
| 16 | All | Reconcile PRD 14 contexts vs architecture 7 contexts. | product contradiction. |
| 17 | All | Reconcile p99 200ms vs 242ms vs 500ms. | PRD/capacity/OpenSLO contradiction. |
| 18 | All | Fix broken paths in tenant_class adoption matrix. | `tenant_class adoption record:130-137`. |
| 19 | All | Document generated SDK boundary for non-Rust client outputs. | `sdk-plan.md:20`; proto options. |
| 20 | All | Remove or replace content-pass scaffolding. | `ARCHITECTURE.md:174-267`; `compliance.md:64-158`. |

## §7 Final Tier Delta Verdict
- DemoTrial is not currently coherent with canonical OCI Always Free doctrine.
- Paid is not currently coherent with its own Braintree and production PSP claims.
- Paid is not currently coherent with counterpart parity because several union features are absent and no measured scale evidence exists.
- Paid is not currently coherent with hyperscaler maturity because deployment context, OpenTofu, OS, Rust build, and benchmark gates are incomplete.
- Oyatie payments is ahead in policy/audit/cell doctrine, but those additive strengths do not offset missing counterpart basics.
- The next correct action is a repair pass that makes tier contracts machine-readable and context-aware before implementation teams treat the tiers as build-ready.

## §8 Cross-Tier Acceptance Gates
| Gate | tenant_class |
|---|---|---|---|---|
| G-01 PSP adapter truth | Stripe sandbox only or explicit Braintree absence. | Stripe, Adyen, and Braintree production adapters declared. | Multi-PSP failover tested. | Dedicated PSP/account sharding tested. |
| G-02 Hosted checkout | Sandbox checkout session or approved exclusion. | Production hosted checkout. | Themed and localized checkout. | Tenant-custom checkout isolation. |
| G-03 Embedded components | No production component required if sandbox-only. | PCI-minimizing component boundary. | Mobile/web component parity. | Dedicated tenant component governance. |
| G-04 Vault | Sandbox token lifecycle. | Production vault with revoke/rotate. | Network tokenization and updater. | Dedicated vault isolation. |
| G-05 Subscriptions | Minimal sandbox renewal. | Plans, prices, billing periods. | Proration, dunning, rescue. | Massive subscription sharding. |
| G-06 Marketplace onboarding | Test sub-merchant only. | Production KYC/KYB onboarding. | Capability and reserve controls. | Dedicated onboarding queues and SLAs. |
| G-07 Split settlement | Example split only. | Production platform fees. | Waterfalls, reserves, withholding. | Tenant-dedicated settlement engines. |
| G-08 Payouts | Sandbox payout simulation. | Manual and scheduled payouts. | Instant/staged payout policy. | Dedicated payout rail routing. |
| G-09 Fraud/risk | Alert-only dashboards. | Rule authoring and review workflow. | Adaptive risk and velocity controls. | Tenant-specific risk model isolation. |
| G-10 Disputes | Evidence sandbox flow. | Production evidence submission. | Bulk dispute operations. | Dedicated dispute SLA and regulator export. |
| G-11 Reporting | Basic dashboards. | Exportable reports. | Search and reconciliation APIs. | Dedicated data pipeline. |
| G-12 Webhooks | Signature/dedupe sandbox. | Production replay and delivery SLO. | High-volume fanout. | Tenant-dedicated event partitions. |
| G-13 OpenTofu | OCI Always Free root plus all context declarations. | All six paid/free context roots. | Context roots with scale variables. | Dedicated/signed modules and state isolation. |
| G-14 OS support | Manifest includes supported/excluded OSes. | Tier-1 CI lanes green. | Tier-1 packaging and upgrade tests. | Tenant-dedicated OS/package evidence. |
| G-15 Rust build | Cargo build proof or explicit design-only status. | Release build proof. | Perf build proof. | Reproducible signed build proof. |
| G-16 Benchmarking | Sandbox smoke numbers only. | Measured p50/p95/p99 and RPS. | Saturation and failover runs. | Dedicated tenant hyperscale run. |
| G-17 OCI cost | Always Free hard fail before paid spill. | Paid OCI budget allowed. | Paid OCI scale budget. | Dedicated OCI/private-region profile. |
| G-18 Compliance | No real PAN unless PCI path complete. | PCI L1 production controls. | Regional overlays tested. | Sovereign cell controls tested. |
| G-19 Product docs | No contradictions in DemoTrial subset. | No contradictions in production API. | No contradictions in platform scale docs. | No contradictions in enterprise/dedicated docs. |
| G-20 Counterpart parity | Sandbox parity with top-3 basics. | Production parity with top-3 core. | Platform parity with top-3 union. | Enterprise parity plus Oyatie additive controls. |
| G-21 Migration | Stripe sandbox migration. | Stripe/Adyen/Braintree production migration. | Bulk migration/reconciliation. | Dedicated migration runbooks and SLAs. |
| G-22 Error semantics | Basic PSP failure codes. | Full PSP result normalization. | Cross-PSP retry and failover codes. | Tenant-specific policy/result overlays. |
| G-23 Rate limits | Sandbox throttles. | PSP/account throttles documented. | Adaptive multi-PSP throttling. | Dedicated limit-increase runbooks. |
| G-24 Admin controls | Minimal internal operations. | Production support actions audited. | Bulk operational controls. | Tenant-dedicated control plane. |
| G-25 Release claim | Evaluation only. | Production baseline only after P1 closure. | Scale claim only after measured evidence. | Hyperscaler claim only after all gates are green. |

## §9 Non-Negotiable Tier Claim Blocks
- DemoTrial cannot be advertised for `guest-on-oci` until the Always Free OpenTofu root exists and the tenant_class adoption matrix stops exceeding free quotas.
- Paid cannot be advertised as production counterpart-parity until Braintree, vaulting, hosted checkout, account capabilities, and payout schedules are in contracts.
- Paid cannot be advertised as platform-scale until measured benchmark artifacts replace target-only numbers and settlement/reconciliation APIs exist.
- Paid cannot be advertised as hyperscaler-grade until single-tenant cell isolation, signed context modules, OS lanes, and RTO/RPO drills are proven.
- Every tier claim must name deployment context, OS/arch, PSP mode, tenant class, and whether real cardholder data is permitted.
- Any tier claim that relies on generated client SDKs must state that non-Rust outputs are generated/distribution artifacts, not backend implementation.
- Any tier claim that references Braintree must include the actual Braintree adapter, enum, migration, webhook, and vault semantics.
- Any tier claim that references OpenTofu must show `tofu init` and `tofu plan` evidence for the relevant context.
- Any tier claim that references OCI Always Free demo_trial must include zero-cost telemetry and a fail-closed paid-spillover guard.
- Until those gates exist, the safest external wording is "payments design artifacts exist; tier contracts are not release-ready."
