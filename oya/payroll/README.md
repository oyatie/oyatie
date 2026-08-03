# Payroll microservice

Service: `payroll`
Owner: `axis-enterprise`
Status: foundation-slice-in-progress

This flat microservice owns payroll-run close invariants, payee payroll classification, wage-ledger evidence, statutory-export evidence envelopes, rollback-first close promotion, and payroll-to-accounting journal drafts. It is separate from HR and Accounting and composes by metadata-only refs/events.

## Current landed slice

- `crates/oya-payroll-run-domain` (`oya-payroll-run-domain`): pure Rust domain invariants for trial close, legal-entity group rollup, statutory export evidence, payroll journal balancing, rollback-first promotion decisions, HR leave-impact intake, variance/retro-adjustment evidence, and statutory rulepack source provenance.
- `crates/oya-payroll-run-app` (`oya-payroll-run-app`): metadata-only audit, accounting dispatch, and HR leave-impact envelopes over the pure domain layer.
- `crates/oya-payroll-run-api` (`oya-payroll-run-api`) plus `contracts/openapi-v1.yaml`: transport-neutral DTO and preview OpenAPI wire contracts.
- `crates/oya-payroll-run-infrastructure` (`oya-payroll-run-infrastructure`): testable Hyper-router foundation with explicit non-deployment/non-storage health claims.
- `crates/oya-payroll-run-storage-adapter-inmemory` (`oya-payroll-run-storage-adapter-inmemory`): volatile in-memory reference adapter for metadata records only.

## Does not own

- HR employment records; those belong to `hr`.
- Persisted double-entry ledger and financial close; those belong to `accounting`.
- Tax-rate calculation, statutory filing transport, disbursement rails, durable storage/Postgres RLS, deployed HTTP/gRPC listeners, Workflow execution, HR/accounting network calls, runtime audit-chain emission, cloud adapters, or GA/production-close readiness.
