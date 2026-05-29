---
doc_class: FAQ
microservice: plugin-app-store
persona: marketplace-publisher
related_adrs: [ADR-0316, ADR-0249, ADR-0251]
date: 2026-05-20
doc_status: published
---

# plugin-app-store — Publisher FAQ

## Q1: What's the revenue share? Can I negotiate it?

Default: 70/30 (you keep 70%, marketplace keeps 30%). Configurable per publisher at paid billing_components tier — successful publishers (high listings, high retention, strategic partners) can negotiate up to 85/15. The 30% covers: payment processing (Stripe/Adyen fees ~ 2.9%), security review + scanning costs, hosting + bandwidth, marketing surface, tax compliance, refund pool. Compare to Apple App Store (30%) or Salesforce AppExchange (15-25%). For free listings: no revenue share. For listings with usage-based pricing: same 70/30 default applied to your gross revenue.

## Q2: Why do I need an SBOM (Software Bill of Materials)?

Two reasons: (1) Security — when a CVE drops on a transitive dependency (e.g. Log4Shell), the marketplace can query the SBOM index and notify all affected listing publishers + their customers within minutes. Without SBOMs, we'd discover affected listings days/weeks later. (2) Compliance — many enterprise customers (FedRAMP, EU NIS2, ISO 27001) require SBOM for procurement; without one, your listing isn't installable by those customers. CycloneDX 1.6 + SPDX 2.3 are the supported formats; either is acceptable.

## Q3: My listing requires payment by my tenants. How does that work?

Use Stripe (default) or Adyen MarketPay (alternative). At publish time, you provide your Stripe account ID. When a customer installs your paid listing, the substrate creates the Stripe Checkout session, the customer pays, Stripe transfers your 70% share to your account (after the 30% platform fee + Stripe's processing fee). For subscriptions: Stripe handles recurring billing; the substrate tracks active subscriptions per tenant. For usage-based: you emit usage events to the marketplace; daily aggregation generates the Stripe invoice. Refunds (within 14 days for most products) are processed through Stripe with the platform fee partially refunded.

## Q4: What about taxes (US sales tax, EU VAT, KR VAT, JP consumption)?

At tenant_class paid tier, the substrate integrates with TaxJar (US) + Avalara (US + global). For each sale, the substrate calculates the appropriate tax based on the customer's tax-domicile, adds it to the checkout, collects + remits to the jurisdiction. Your 70% share is calculated on the pre-tax amount; tax is paid to the relevant authority by the substrate. For VAT-MOSS, EU OSS, and equivalent simplified-tax regimes, the substrate handles VAT invoice generation + quarterly returns.

## Q5: Can I sell to government / regulated industries?

Yes via pack-specific listings (compliance_pack-bound paid tier). To list in:
- US-FedRAMP-Moderate marketplace: complete the FedRAMP PMO Authorization to Operate + provide SBOM + STIG-compliance evidence.
- KR-CSAP marketplace: get a CSAP-가 (highest grade) or CSAP-나 (cloud-suitable) certification from the Korean Internet & Security Agency.
- EU-NIS2 marketplace: provide NIS2 conformance evidence per pack-specific requirements.

These require additional review cycles (3-6 months) but unlock access to enterprise + government customers who can't buy from a non-certified marketplace.

## Q6: My listing got rejected. What recourse do I have?

Review rejections come with a structured reason. Common rejection classes:
- **Security**: failed scan or excessive permissions. Fix + resubmit.
- **Privacy**: insufficient privacy policy or undisclosed data collection. Fix + resubmit.
- **Quality**: poor screenshots, vague description, broken install. Improve + resubmit.
- **IP**: trademark infringement, unauthorized use of brand assets. Remove infringing content.
- **Legal**: violates marketplace TOS (e.g. selling something illegal, missing required disclosures).

Each rejection has an appeal path: portal → Listings → Rejected → "Appeal". Appeals reach a senior reviewer (different from the original) within 5 business days. Repeated bad-faith submissions can result in publisher account suspension (rare).

## Q7: How do I version my listing? When should I deprecate?

Semver. `1.0.0` for the initial production release; bump patch for fixes (`1.0.1`), minor for new features (`1.1.0`), major for breaking changes (`2.0.0`). Substrate retains all versions; customers can pin to a specific version or accept auto-updates within the minor.

Deprecation: when you no longer support a version, mark it `deprecated` in the manifest. Customers on that version get a portal notification. They can upgrade or contact you for extended support. After 1 year of deprecation, the substrate hides the version from search (but existing installations remain functional). After 2 years, the version is moved to archive (not installable; existing installs may continue but lose substrate support).

## Q8: My listing depends on a third-party API (e.g. OpenAI). How does that work?

Declare it in your manifest:
```json
"external_dependencies": [
  {
    "name": "OpenAI API",
    "url": "https://api.openai.com",
    "purpose": "LLM inference for the AI features",
    "data_sent": ["user query text", "document content"],
    "data_returned": ["LLM completion"],
    "privacy_policy": "https://openai.com/privacy",
    "billing": "tenant-supplied API key required"
  }
]
```

Reviewers verify the privacy + billing disclosures are accurate. Customers see this in the install flow + must accept before installing.

## Q9: Can I publish AI agents (per ADR-0249's "agents" category)? What's different?

Yes. Agent listings have stricter review:
- Required: model card describing the underlying LLM(s) used + safety evaluations.
- Required: tool-permissions enumeration (what tools the agent can invoke; substrate enforces via Cedar permits).
- Required: human-in-the-loop policy (which actions require approval).
- Required: hallucination/error rate disclosure on a standard test set.
- Required: termination conditions (when does the agent stop?).
- Manual review: every agent listing reviewed by an AI-safety reviewer.

EU AI Act conformance: agents that meet "high-risk" criteria per Annex III require additional documentation; the substrate provides a template + flags listings that need this.

## Q10: How do I handle support? Can I require customer-paid support?

Free listings: support is your responsibility but you set the SLA. Paid listings: you can offer tiered support (e.g. "community support free; email support included; priority support +$50/mo"). The marketplace provides a customer-support channel (portal → My Listings → Support Tickets) that routes customer questions to your team via the `contact-center` or `community` µservice.

## Q11: I want to discontinue my listing entirely. What happens to existing customers?

Two paths:
1. **Graceful sunset**: mark deprecated → notify customers → wait 90 days → hide from search → wait 1 year → archive. Customers have time to migrate.
2. **Immediate removal**: only for: legal mandate, security emergency (zero-day), TOS violation. Substrate notifies customers immediately + offers refund (substrate covers the cost; you forfeit unpaid revenue).

For subscription-based listings: existing subscriptions can continue to bill if you want (some publishers keep legacy customers in maintenance mode); or you can refund proportionally to remaining subscription period.

## Q12: My pack is KR-PIPA. Can I publish to global marketplace + Korean marketplace simultaneously?

Yes if your listing complies with KR-PIPA requirements. To publish to the Korean (compliance_pack-bound paid) marketplace:
- Korean translation of all marketing copy.
- KR-PIPA pack overlay-compliant privacy policy.
- Pack-resident customer data handling.
- KR-resident customer support (per 정보통신망법 § 32).
- Submit through the KR-pack review process (different reviewer pool than global).

Approval takes 4-8 weeks for the first Korean listing; subsequent listings 2-3 weeks. Korean revenue settles in KRW (Stripe handles currency conversion).
