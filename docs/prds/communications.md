---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-communications-services
microservice: communications-services
status: Retired-redirect
sales_segment: tenant-rbac-packaged
tier: service-set
milestone_first_ship: M03-first-paying-tenant
doc_status: published
successor_prds:
  - /specs/microservices/messenger.json
  - /specs/microservices/mail.json
  - /microservices/community/PRD.md
---

# Communications services redirect

The former communications grouping is not an active product, module, platform,
or service. Active engineering authority is the concrete flat microservices:

- **Messenger** — `/specs/microservices/messenger.json`
- **Mail** — `/specs/microservices/mail.json`
- **Community** — `/microservices/community/PRD.md`

Packaging is computed later from tenant entitlements, RBAC roles, regulatory
packs, residency, and feature flags; packaging does not create an engineering
boundary.

Messenger and mail keep personal-life and professional-life contexts under
strict tenant/RBAC separation. The default cross-context decision is **deny**;
any exception requires explicit user action, policy decision id, data-class
check, tenant/RBAC scope, and audit-chain evidence.
