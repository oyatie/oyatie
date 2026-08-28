# Payroll

Owner: `app/payroll`

Status: portable-app migration; validation foundation only

Payroll owns deterministic gross-to-net calculation, pay-run gates and close,
balanced accounting intent, and its jurisdiction overlay content.

The landed Rust packages validate supplied wage ledgers and evidence; they do
not yet calculate statutory payroll or provide durable lifecycle, installed
pack integration, encrypted SQLite, downstream delivery, a sold generated-
Connect process, deployment, or measured production SLO. Current cloud-core,
volatile-storage, and REST/JSON links are migration debt.

Canonical owner law:

- [ADR.md](ADR.md) — decisions and portability boundaries
- [PRD.md](PRD.md) — requirements, failures, and SLO objectives
- [SPEC.md](SPEC.md) — current behavior and target contracts
- [PLAN.md](PLAN.md) — remaining semantic lanes and exact path sets
