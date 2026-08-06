# `cloud-billing` µservice — Billing Engineer FAQ

20 real questions raised against `cloud-billing` (the µservice that owns Oyatie's usage-attribution + invoicing substrate).

---

**Q1. Does `cloud-billing` replace AWS Cost & Usage Reports + GCP Billing + Azure Cost Management?**

It **wraps and unifies**. For Oyatie-tenant workloads running across AWS/GCP/Azure, `cloud-billing` ingests vendor CUR/billing
exports, normalizes to FOCUS 1.1, attributes to tenants + cost centers, and presents a unified invoice. Tenants never see raw
AWS line items — they see Oyatie-priced services, with vendor pass-through clearly broken out.

---

**Q2. Why FOCUS 1.1 as the canonical schema?**

FinOps Foundation FOCUS is the industry-standard cost+usage schema, ratified by AWS / GCP / Azure / Oracle / IBM. Adopting it
makes Oyatie's billing exports drop-in compatible with downstream FinOps tools (Vantage, CloudZero, Apptio Cloudability,
Anodot Cost Management). Per ADR-0245, substrate µservices conform to industry-standard schemas where they exist.

---

**Q3. What's the metering bus?**

A Kafka topic (`cloud_billing.metering.v1`) with strict schema enforcement. Each µservice that generates billable activity
publishes events conforming to the `MeteringEvent` schema. The bus deduplicates by `event_id` within a 5-minute window and
fans out to: (1) per-tenant raw ledger, (2) rolling aggregate streams, (3) anomaly detection. Throughput target: 5 M events/sec
sustained at scale.

---

**Q4. What's the schema of a `MeteringEvent`?**

```protobuf
message MeteringEvent {
  string event_id = 1;        // UUID v7
  string tenant_id = 2;
  string resource_kind = 3;   // e.g. "cloud_compute_k8s.pod_minute"
  string resource_id = 4;     // canonical resource URI
  double quantity = 5;
  string unit = 6;            // "minute", "byte", "ops", "gb", ...
  google.protobuf.Timestamp timestamp = 7;
  map<string, string> dimensions = 8;  // region, pack, cost_center, ...
  string emitter_principal = 9;        // Cedar principal that emitted this
  bytes signature = 10;                // emitter HMAC for tamper detection
}
```

---

**Q5. How is multi-currency handled?**

Each tenant has a `billing_currency` (USD default). For tenants billed in non-USD, the invoice locks the ECB reference rate at
the issuance timestamp. The FX rate, source (`ECB-reference-rates-daily`), and lock timestamp are persisted on the invoice. No
silent revaluation: an invoice issued in EUR at 1.0850 stays 1.0850 forever; FX corrections require explicit credit memos.

---

**Q6. How do reservations work?**

A reservation is a tenant commitment to use ≥ X quantity of a resource over a period (1y/3y/5y). Reservations pre-pay the
discounted amount upfront (or per-month-with-commitment fee). Usage up to X is billed at the reserved rate; overage at the
on-demand rate. Convertible reservations (paid) can be re-pointed to a different resource class within the same family.

---

**Q7. What's the chargeback / showback distinction?**

- **Showback**: visible attribution; no internal funds movement. Used by SMB+.
- **Chargeback**: visible attribution + internal funds movement (cost center A is debited; substrate-provider is credited).
  Requires ERP integration via `cloud-billing-erp-adapter-*` crates.

---

**Q8. How does transfer pricing work for cross-entity charges?**

paid tenant_class supports OECD BEPS-compliant transfer pricing. Define a transfer-pricing rule via
`cloud_billing::Action::ManageTransferPricing` — e.g. "Entity A in DE charges Entity B in US at cost + 7 %". The invoice splits
into per-entity invoices; the markup is recorded in the OECD GloBE report (Pillar Two).

---

**Q9. What anomaly detection runs?**

- **DemoTrial**: weekly z-score on monthly spend vs trailing 13 months.
- **Paid**: hourly z-score + DBSCAN clustering on per-resource-kind spend.
- **Paid**: streaming Bayesian model + tenant-wide baseline; threshold breach → reviewer-agent thread.
- **Paid**: continuous; > 3σ breach escalates to governance within 5 min.

---

**Q10. How are credits + refunds issued?**

Only through `cloud_billing::Action::IssueCreditMemo` (Cedar-gated). The memo:
1. Creates a credit ledger entry (negative line item).
2. Applies to the next invoice (or refunds immediately if requested + payment-method supports it).
3. Writes a `cloud_billing.credit_memo.issued` audit event.
4. Requires a reason code (`error`, `goodwill`, `sla_breach`, `governance_remediation`, ...).

Direct `UPDATE` on the ledger is forbidden and refused by the database trigger.

---

**Q11. How does the invoice integrate with `cloud-billing-tax`?**

`cloud-billing` produces a tax-naive invoice (`subtotal` only). It then calls `cloud-billing-tax` per line item, which returns
per-jurisdiction tax amounts. The final invoice carries `tax_lines` per jurisdiction. The two µservices are co-deployed in the
same cell for latency (≤ 5 ms p95 round-trip).

---

**Q12. What's the SLO for invoice issuance?**

DemoTrial: month-end + 24 h. Paid: month-end + 12 h. Paid: month-end + 4 h. Paid: month-end + 1 h. Late invoices fire a
P1 incident; SOX-404 controls require timely issuance for revenue recognition.

---

**Q13. How are vendor pass-through costs (AWS, GCP, Azure) attributed?**

Vendor CUR/billing exports are ingested via `cloud-billing-ingestor-{aws,gcp,azure}-*` adapters. Each line is tagged with the
Oyatie tenant via Cedar tag mapping (every Oyatie-managed cloud resource carries an `oyatie.tenant_id` tag). Untagged lines
go to `oyatie.platform.shared-overhead` and are amortised across all tenants per the substrate allocation policy.

---

**Q14. Can a tenant export raw usage to their own warehouse?**

Yes. paid supports daily Parquet exports to a tenant-controlled S3-compatible bucket (via `cloud-storage`). paid supports
real-time Kafka streaming via the `cloud_billing.focus.v1` topic. Both are Cedar-gated (`cloud_billing::Action::ExportFocusStream`).

---

**Q15. How does this work for cells in air-gapped / sovereign deployments?**

Air-gapped Paid cells maintain a local metering bus; usage syncs to the sovereign control plane via a one-way replicator
that never sends usage data outside the sovereign boundary. Invoices are issued by a per-tenant `cloud-billing` instance running
inside the sovereign perimeter. KR K-FSI requires this for financial-sector tenants.

---

**Q16. How is the rate card structured?**

A rate card is a versioned YAML document with per-resource-kind pricing:
```yaml
rate_card_id: oya-rate-card-smb-paid-v1
effective_from: 2026-01-01
currency: USD
items:
  - resource_kind: cloud_compute_k8s.pod_minute
    unit: minute
    rate: 0.0142
    tiers:
      - from: 0
        to: 100000
        rate: 0.0142
      - from: 100000
        to: ~
        rate: 0.0118  # volume discount
```

Tenants attach to a rate card; rate card changes require a 30/60-day notice per tenant_class.

---

**Q17. How are unused reservations refunded?**

Unused reservations are **not refunded** (commitment is the whole point). At reservation expiry:
- Unused capacity → discarded.
- Used capacity → reserved-rate; overage → on-demand-rate.

Convertible reservations (paid) can be **re-pointed** mid-term to a different resource within the family, preserving the
discount. This is the only "refund-like" path; documented under `cloud_billing::Action::ConvertReservation`.

---

**Q18. How does the Kafka metering bus survive a region outage?**

Topic `cloud_billing.metering.v1` has 5× replication factor + min-in-sync-replicas=3. Each producer (`cloud-*` µservice) has a
client-side outbox (BadgerDB embedded) that buffers events when the bus is unreachable. On reconnection, the outbox drains in
order. Worst-case retention: 7 d on the outbox before alarming.

---

**Q19. How is fraud / abuse detected?**

`cloud-billing` runs a daily fraud sweep:
- Sudden spike in usage on a DemoTrial tenant (≥ 10× weekly baseline) → reviewer-agent + auto-throttle.
- Stripe payment method declined ≥ 3× in 24 h → tenant freeze (Cedar denies `cloud_billing::Action::IncurUsage`).
- Reservation purchase + immediate cancellation pattern → governance escalation.

Fraud events anchor to `audit-chain` with severity `Suspicious`.

---

**Q20. Where does Foundry hook in?**

Foundry pipelines themselves consume `cloud-compute-k8s` minutes for their runners; those costs are attributed to the
`oyatie.foundry.<pipeline-id>` tenant + cost center `infra-substrate`. Foundry's substrate cost is amortised across all
oyatie-tenants per the substrate allocation policy (see `microservices/finops-portal/`).
