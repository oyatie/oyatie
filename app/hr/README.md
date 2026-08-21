# HR microservice

Service: `hr`
Owner: `axis-enterprise`
Status: foundation-slice-in-progress

This flat microservice owns employee/employment records, organizational reporting seams, labor-compliance obligation detection, and HR evidence carriers. It is not an Tenant RBAC boundary; Tenant RBAC view and Tenant RBAC view are tenant/product-surface metadata layered over this HR service.

## Current landed slice

- `core/employment-domain` (`hr-employment-domain`): pure Rust domain invariants for legal-entity-scoped employment records, audit-backed lifecycle events, and Korea-first rules-of-employment / labor-management-council threshold obligations.

## Does not own

- Payroll gross-to-net, statutory payroll export, or disbursement; those belong to `payroll`.
- Double-entry posting, VAT workflow, AP/AR, or financial close; those belong to `accounting`.
- Workflow execution, audit-chain persistence, tenant identity, storage, REST/gRPC, or cloud adapters.
