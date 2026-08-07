// audit-chain local import shim
// Cedar v4
//
// Purpose: local policy references resolve here while the canonical import
// authority remains microservices/governance/policy/cedar-canonical-imports.cedar.
// This file is intentionally policy-neutral; concrete permits live in sibling
// audit-chain policy fragments.

forbid (principal, action, resource) when { false };
