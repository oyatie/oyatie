# oya-governance-audit-event-emission

Scaffolds the ADR-0263 CI gate for state-changing endpoint audit emission.

## Rule

Every state-changing endpoint must emit an ADR-0263 registered audit event class.

## Trigger

The gate triggers when mutating endpoints or audit event class registrations are added or changed.

## Compliant

A compliant endpoint emits a registered event class for each state-changing operation and keeps that event discoverable by the audit-chain registry.
