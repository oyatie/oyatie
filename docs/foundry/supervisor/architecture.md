---
doc_status: published
---

# Foundry Supervisor Architecture

The supervisor follows the project's 12-layer hexagonal architecture.

## 4-Crate Decomposition
1. **oya-intelligence-supervisor-kernel** (Layer 1): Pure value types and decision logic. Zero I/O.
2. **oya-intelligence-supervisor-app** (Layer 4): Orchestrates ports, owns the Tokio runtime, and implements the `tick_once` call chain.
3. **intelligence-jsonl-supervisor-adapter** (Layer 5): Implements `InboxStore` for JSONL file I/O.
4. **oya-intelligence-supervisor-conformance** (Layer 11): Measure and verify driver capabilities at build-time.

## Design Patterns
- **Port-in-Kernel (ADR-0056):** Ports like `SessionDriver` and `InboxStore` are defined in the kernel.
- **Driver-not-Kernel:** Provider-specific CLI logic is encapsulated in adapters; the kernel is provider-agnostic.
- **Stateless Subprocesses:** Every message spawns a fresh CLI process to ensure isolation.
