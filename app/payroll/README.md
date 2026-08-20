# Payroll microservice

Service: `payroll`
Owner: `axis-enterprise`
Status: foundation-slice-in-progress

This flat microservice owns payroll-run close invariants, payee payroll classification, wage-ledger evidence, statutory-export evidence envelopes, rollback-first close promotion, and payroll-to-accounting journal drafts. It is separate from HR and Accounting and composes by metadata-only refs/events.

## Current landed slice

- `crates/oya-payroll-run-domain` (`oya-payroll-run-domain`): pure Rust domain invariants for trial close, legal-entity group rollup, statutory export evidence, payroll journal balancing, and rollback-first promotion decisions.

## Does not own

- HR employment records; those belong to `hr`.
- Persisted double-entry ledger and financial close; those belong to `accounting`.
- Tax-rate calculation, statutory filing transport, disbursement rails, storage, REST/gRPC, Workflow execution, or cloud adapters.
