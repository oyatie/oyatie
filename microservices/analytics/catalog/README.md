# Analytics µservice — Catalog

This directory holds BNF v4.1 (Backus-Naur Form, v4.1 convention) catalog entries for every named entity that the analytics µservice introduces. Per the "naming justification" doctrine (CLAUDE.md), every new name carries a one-line justification proving v4.1 BNF + 12-layer-enum conformance.

## BNF v4.1 syntax recap

```
<entity> ::= <product>-<layer>-<concern>-<kind>
<product>    ::= "oya"           // canonical fleet prefix
<layer>      ::= <12-layer-enum> // per ADR-0083 layer enum
<concern>    ::= <microservice>  // canonical microservice name
<kind>       ::= adapter | kernel | domain | usecase | api | app | controller | policy | runbook | dashboard | slo | scorecard | capability
```

The 12-layer enum is canonical per ADR-0083 (post-ADR-0105 amendment to 13 layers; analytics catalog uses the canonical 12-of-13 active subset).

## Files

Each `*.json` in this directory is one catalog entry. Naming: `<entity-name>.json`.

## Reference

- CLAUDE.md naming justification doctrine.
- ADR-0083 12-layer enum (post-amendment ADR-0105 → 13-layer canonical).
- `specs/master-plan-sequencing.json` glossary.
