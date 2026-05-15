---
doc_status: published
---

# Foundry Supervisor

The Foundry Supervisor is a daemon that orchestrates session execution across multiple provider accounts (Claude, Codex, Gemini). It manages inbox/outbox JSONL files, enforces usage limits and cost ceilings, and ensures settings consistency via canonical templates.

## Architecture
See [Architecture](./architecture.md) for the 4-crate decomposition and port-based design.

## Operations
See [Operations](./operations.md) for daemon management, signal handling, and crash recovery.

## Security
See [Security](./security.md) for secret management (OpenBao), Cedar policy enforcement, and autonomy ceilings.
