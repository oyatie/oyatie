# Accounting microservice

Service: `accounting`
Owner: `axis-enterprise`
Status: foundation-slice-in-progress

This flat microservice owns journal-voucher posting invariants, source-document evidence, payroll posting intake, VAT workflow evidence, AP approval gates, and financial-close evidence refusal. It is separate from HR and Payroll and composes by metadata-only refs/events.

## Current landed slice

- `crates/oya-accounting-journal-domain` (`oya-accounting-journal-domain`): pure Rust domain invariants for balanced open-period posting, payroll digest intake, KR VAT evidence workflow, AP approval checks, and close-evidence refusal.

## Does not own

- Payroll gross-to-net or statutory payroll export; those belong to `payroll`.
- HR employment records; those belong to `hr`.
- Storage, REST/gRPC, bank rails, tax filing transport, Workflow execution, or cloud adapters.
