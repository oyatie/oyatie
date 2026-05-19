# Connect Tenant Isolation

The retiring umbrella carries only retirement status by tenant context. It does not own sub-service user data or cross-tenant runtime behavior.

Tenant isolation is enforced by keeping product data in the eight first-class replacement microservices and by rejecting new runtime scope under `connect`.
