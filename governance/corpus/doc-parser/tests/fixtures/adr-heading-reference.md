---
id: ADR-0517
status: Accepted
---
# ADR-0517: One owned AST substrate read by every consumer

This fixture cites [ADR-0541](ADR-0541-corpus-liveness-graph.md) and the [root hub](../../specs/root-hub-pointers.json).

## Decision

Own the parser and produce content-addressed node identity.

### Threat model

Markdown content is data only.
