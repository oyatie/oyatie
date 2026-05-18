//! WASM runtime kernel — canonical Wasmtime substrate per ADR-0200.
//!
//! # Why this crate exists
//!
//! ADR-0200 makes Wasmtime (BytecodeAlliance / CNCF; Fastly
//! Compute@Edge reference deployment) the only sanctioned WASM
//! runtime for oyatie. Every WASM call site — Envoy north-south
//! filters (ADR-0182), Workflow Studio user-supplied node logic
//! (ADR-0185), Foundry tool sandbox (ADR-0136) — instantiates
//! bytecode through this trait so that:
//!
//! 1. Sandbox class invariants (fuel + memory + import allowlist)
//!    are enforced uniformly.
//! 2. The discipline gate `oya-check-wasm-runtime-discipline`
//!    can prove no µservice imports `wasmtime` (or `wasmer` /
//!    `wasmedge`) directly.
//! 3. Per-tenant capability tokens (no ambient authority — the
//!    Cloudflare Workers / Fastly Compute@Edge security model)
//!    flow through a single chokepoint for audit + chargeback
//!    (ADR-0174 FinOps).
//!
//! # Layer
//!
//! `kernel` (port-in-kernel per ADR-0056).
//!
//! # Naming justification
//!
//! `oya-shared-wasm-runtime-kernel` follows BNF v4.1:
//! `oya-<axis:shared>-<topic:wasm-runtime>-<layer:kernel>`.
//!
//! # References
//!
//! - ADR-0200 — WASM runtime canonical (Wasmtime).
//! - ADR-0147 — container sandboxing runtime ladder.
//! - ADR-0182 — north-south vs east-west separation.
//! - ADR-0185 — Workflow Studio client stack.
//! - docs/standards/wasm-runtime-canonical.md.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]
#![allow(clippy::result_large_err)]

use std::collections::BTreeSet;
use std::fmt;
use std::time::Duration;

/// Canonical sandbox classes, fixed at ADR-0200 cadence.
///
/// Each variant carries an immutable fuel + memory ceiling and an
/// import allowlist that the runtime enforces at instantiation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum SandboxClass {
    /// Envoy north-south WASM filter (ADR-0182). Highest QPS;
    /// strict fuel cap; tiny memory ceiling.
    EnvoyFilter,
    /// Workflow Studio user-supplied node (ADR-0185). User-authored
    /// expressions / mappers / transformers. Medium fuel; medium
    /// memory; no network imports.
    WorkflowStudioNode,
    /// Foundry tool execution sandbox (ADR-0136). LLM-driven tool
    /// calls. Larger fuel + memory; outbound HTTP only when the
    /// capability registry says so (deferred decision per ADR-0200
    /// open questions).
    FoundryTool,
}

impl SandboxClass {
    /// Class-level fuel ceiling. "Fuel" is Wasmtime's deterministic
    /// instruction-count gas meter.
    #[must_use]
    pub fn fuel_ceiling(self) -> u64 {
        match self {
            SandboxClass::EnvoyFilter => 1_000_000,
            SandboxClass::WorkflowStudioNode => 50_000_000,
            SandboxClass::FoundryTool => 500_000_000,
        }
    }

    /// Class-level memory ceiling in bytes.
    #[must_use]
    pub fn memory_ceiling_bytes(self) -> u64 {
        match self {
            // 16 MiB — Envoy filter must stay tiny.
            SandboxClass::EnvoyFilter => 16 * 1024 * 1024,
            // 128 MiB — Workflow Studio user node.
            SandboxClass::WorkflowStudioNode => 128 * 1024 * 1024,
            // 512 MiB — Foundry tool sandbox.
            SandboxClass::FoundryTool => 512 * 1024 * 1024,
        }
    }

    /// Class-level wall-clock ceiling (defense-in-depth on top of
    /// fuel exhaustion).
    #[must_use]
    pub fn wall_clock_ceiling(self) -> Duration {
        match self {
            SandboxClass::EnvoyFilter => Duration::from_millis(50),
            SandboxClass::WorkflowStudioNode => Duration::from_secs(5),
            SandboxClass::FoundryTool => Duration::from_secs(60),
        }
    }

    /// Class-level import allowlist (capability identifiers). Any
    /// import outside this set fails at instantiation.
    #[must_use]
    pub fn import_allowlist(self) -> BTreeSet<&'static str> {
        let mut s = BTreeSet::new();
        match self {
            SandboxClass::EnvoyFilter => {
                s.insert("oya:envoy/header_get");
                s.insert("oya:envoy/header_set");
                s.insert("oya:envoy/body_read");
                s.insert("oya:envoy/log");
            }
            SandboxClass::WorkflowStudioNode => {
                s.insert("oya:workflow/input_read");
                s.insert("oya:workflow/output_write");
                s.insert("oya:workflow/log");
            }
            SandboxClass::FoundryTool => {
                s.insert("oya:foundry/argv_read");
                s.insert("oya:foundry/stdout_write");
                s.insert("oya:foundry/log");
                s.insert("oya:foundry/capability_token_check");
            }
        }
        s
    }
}

impl fmt::Display for SandboxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SandboxClass::EnvoyFilter => f.write_str("envoy-filter"),
            SandboxClass::WorkflowStudioNode => f.write_str("workflow-studio-node"),
            SandboxClass::FoundryTool => f.write_str("foundry-tool"),
        }
    }
}

/// Per-tenant capability token bound to a (tenant, sandbox-class,
/// call-site) tuple. There is no ambient authority — every host
/// call resolves through a token check.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CapabilityToken {
    pub tenant_id: String,
    pub sandbox_class: SandboxClass,
    pub call_site: String,
    /// Opaque token material. Rotated per tenant key cycle.
    pub token: String,
}

/// Bytecode to instantiate. Owned bytes so the caller is decoupled
/// from any specific source (filesystem, blob store, capability
/// pack, etc.).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmModuleBytes(pub Vec<u8>);

/// Per-invocation request — what to call, what input to pass, and
/// the capability token authorizing it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmInvocation {
    pub module: WasmModuleBytes,
    pub entrypoint: String,
    pub input: Vec<u8>,
    pub token: CapabilityToken,
}

/// Per-invocation result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmInvocationOutcome {
    pub output: Vec<u8>,
    pub fuel_consumed: u64,
    pub peak_memory_bytes: u64,
    pub wall_clock: Duration,
}

/// Failure surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmRuntimeError {
    /// Bytecode rejected at validation (not WASI Preview 2
    /// component-model, or import outside allowlist, etc.).
    ValidationFailed { reason: String },
    /// Module imports something not in the sandbox class allowlist.
    ImportNotAllowed { import: String, class: SandboxClass },
    /// Fuel exhausted before reaching halt.
    FuelExhausted { class: SandboxClass, ceiling: u64 },
    /// Memory ceiling tripped.
    MemoryCeilingTripped { class: SandboxClass, ceiling: u64 },
    /// Wall-clock ceiling tripped.
    WallClockExceeded {
        class: SandboxClass,
        ceiling: Duration,
    },
    /// Capability token invalid for (tenant, class, call-site).
    CapabilityTokenInvalid,
    /// Entrypoint not exported by the module.
    EntrypointMissing { entrypoint: String },
    /// Real adapter not configured (feature flag absent).
    AdapterNotConfigured(&'static str),
}

impl fmt::Display for WasmRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmRuntimeError::ValidationFailed { reason } => {
                write!(f, "wasm validation failed: {reason}")
            }
            WasmRuntimeError::ImportNotAllowed { import, class } => {
                write!(f, "import {import} not allowed in sandbox class {class}")
            }
            WasmRuntimeError::FuelExhausted { class, ceiling } => {
                write!(f, "fuel exhausted in {class} (ceiling {ceiling})")
            }
            WasmRuntimeError::MemoryCeilingTripped { class, ceiling } => {
                write!(
                    f,
                    "memory ceiling tripped in {class} (ceiling {ceiling} bytes)"
                )
            }
            WasmRuntimeError::WallClockExceeded { class, ceiling } => {
                write!(f, "wall-clock exceeded in {class} (ceiling {ceiling:?})")
            }
            WasmRuntimeError::CapabilityTokenInvalid => {
                write!(f, "capability token invalid")
            }
            WasmRuntimeError::EntrypointMissing { entrypoint } => {
                write!(f, "entrypoint {entrypoint} missing from module")
            }
            WasmRuntimeError::AdapterNotConfigured(name) => {
                write!(f, "wasm runtime adapter {name} not configured")
            }
        }
    }
}

impl std::error::Error for WasmRuntimeError {}

/// The canonical trait every µservice integrates against. Real
/// implementation lives behind the `wasmtime-real` feature in a
/// follow-up adapter crate (parent wires it).
pub trait WasmRuntime: Send + Sync {
    /// Validate a module against a sandbox class' import allowlist
    /// and structural rules. Returns Ok if the module would
    /// instantiate cleanly.
    fn validate(
        &self,
        class: SandboxClass,
        module: &WasmModuleBytes,
    ) -> Result<(), WasmRuntimeError>;

    /// Run a single invocation under the named sandbox class.
    fn invoke(
        &self,
        class: SandboxClass,
        invocation: &WasmInvocation,
    ) -> Result<WasmInvocationOutcome, WasmRuntimeError>;
}

/// Minimal in-kernel validator + reference runtime used by tests
/// and by µservices that do not yet wire the real Wasmtime
/// adapter. It performs all ADR-0200 invariants except actual
/// bytecode execution.
#[derive(Clone, Debug, Default)]
pub struct ReferenceWasmRuntime;

impl ReferenceWasmRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl WasmRuntime for ReferenceWasmRuntime {
    fn validate(
        &self,
        class: SandboxClass,
        module: &WasmModuleBytes,
    ) -> Result<(), WasmRuntimeError> {
        // Reference validator: enforce that the module body is
        // non-empty and that any "ambient" import marker
        // ("wasi:filesystem", "wasi:sockets", etc. — outside the
        // sandbox class allowlist) is rejected. This is intentionally
        // a substring scan; the real Wasmtime adapter does full
        // section parsing.
        if module.0.is_empty() {
            return Err(WasmRuntimeError::ValidationFailed {
                reason: "module is empty".into(),
            });
        }
        let allowlist = class.import_allowlist();
        let body = String::from_utf8_lossy(&module.0);
        for ambient in ["wasi:filesystem", "wasi:sockets", "wasi:cli/environ"] {
            if body.contains(ambient) && !allowlist.iter().any(|s| s.contains(ambient)) {
                return Err(WasmRuntimeError::ImportNotAllowed {
                    import: ambient.to_string(),
                    class,
                });
            }
        }
        Ok(())
    }

    fn invoke(
        &self,
        class: SandboxClass,
        invocation: &WasmInvocation,
    ) -> Result<WasmInvocationOutcome, WasmRuntimeError> {
        if invocation.token.sandbox_class != class {
            return Err(WasmRuntimeError::CapabilityTokenInvalid);
        }
        if invocation.token.tenant_id.is_empty()
            || invocation.token.token.is_empty()
            || invocation.token.call_site.is_empty()
        {
            return Err(WasmRuntimeError::CapabilityTokenInvalid);
        }
        self.validate(class, &invocation.module)?;
        if invocation.entrypoint.is_empty() {
            return Err(WasmRuntimeError::EntrypointMissing {
                entrypoint: String::new(),
            });
        }
        // Reference adapter does not execute bytecode; it returns
        // a deterministic shape so callers (and tests) can prove
        // their integration. The real Wasmtime adapter replaces
        // this with actual execution.
        Err(WasmRuntimeError::AdapterNotConfigured(
            "wasmtime-real feature flag not enabled in this build",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(class: SandboxClass) -> CapabilityToken {
        CapabilityToken {
            tenant_id: "tenant-acme".into(),
            sandbox_class: class,
            call_site: "calls/sample".into(),
            token: "tok-deadbeefcafe1234".into(),
        }
    }

    #[test]
    fn sandbox_classes_have_distinct_fuel_ceilings() {
        let e = SandboxClass::EnvoyFilter.fuel_ceiling();
        let w = SandboxClass::WorkflowStudioNode.fuel_ceiling();
        let f = SandboxClass::FoundryTool.fuel_ceiling();
        assert!(e < w);
        assert!(w < f);
    }

    #[test]
    fn sandbox_classes_have_distinct_memory_ceilings() {
        let e = SandboxClass::EnvoyFilter.memory_ceiling_bytes();
        let w = SandboxClass::WorkflowStudioNode.memory_ceiling_bytes();
        let f = SandboxClass::FoundryTool.memory_ceiling_bytes();
        assert!(e < w);
        assert!(w < f);
        // Envoy filter is intentionally tiny.
        assert!(e <= 32 * 1024 * 1024);
    }

    #[test]
    fn import_allowlist_excludes_ambient_wasi_filesystem() {
        for class in [
            SandboxClass::EnvoyFilter,
            SandboxClass::WorkflowStudioNode,
            SandboxClass::FoundryTool,
        ] {
            let allow = class.import_allowlist();
            assert!(
                !allow.iter().any(|s| s.contains("wasi:filesystem")),
                "sandbox class {class} must not allow ambient filesystem",
            );
            assert!(
                !allow.iter().any(|s| s.contains("wasi:sockets")),
                "sandbox class {class} must not allow ambient sockets",
            );
        }
    }

    #[test]
    fn reference_validator_rejects_empty_module() {
        let rt = ReferenceWasmRuntime::new();
        let err = rt
            .validate(SandboxClass::EnvoyFilter, &WasmModuleBytes(vec![]))
            .unwrap_err();
        match err {
            WasmRuntimeError::ValidationFailed { reason } => {
                assert!(reason.contains("empty"));
            }
            other => panic!("expected ValidationFailed, got {other:?}"),
        }
    }

    #[test]
    fn reference_validator_rejects_ambient_filesystem_import() {
        let rt = ReferenceWasmRuntime::new();
        let body = b"\0asm\x01\x00\x00\x00 import wasi:filesystem/preopens".to_vec();
        let err = rt
            .validate(SandboxClass::WorkflowStudioNode, &WasmModuleBytes(body))
            .unwrap_err();
        match err {
            WasmRuntimeError::ImportNotAllowed { import, class } => {
                assert_eq!(import, "wasi:filesystem");
                assert_eq!(class, SandboxClass::WorkflowStudioNode);
            }
            other => panic!("expected ImportNotAllowed, got {other:?}"),
        }
    }

    #[test]
    fn invoke_rejects_token_with_wrong_class() {
        let rt = ReferenceWasmRuntime::new();
        let inv = WasmInvocation {
            module: WasmModuleBytes(b"\0asm\x01\x00\x00\x00 entry".to_vec()),
            entrypoint: "run".into(),
            input: vec![],
            // Token bound to FoundryTool but caller invokes EnvoyFilter.
            token: token(SandboxClass::FoundryTool),
        };
        let err = rt.invoke(SandboxClass::EnvoyFilter, &inv).unwrap_err();
        assert_eq!(err, WasmRuntimeError::CapabilityTokenInvalid);
    }

    #[test]
    fn invoke_rejects_empty_entrypoint() {
        let rt = ReferenceWasmRuntime::new();
        let inv = WasmInvocation {
            module: WasmModuleBytes(b"\0asm\x01\x00\x00\x00 entry".to_vec()),
            entrypoint: String::new(),
            input: vec![],
            token: token(SandboxClass::FoundryTool),
        };
        let err = rt.invoke(SandboxClass::FoundryTool, &inv).unwrap_err();
        match err {
            WasmRuntimeError::EntrypointMissing { entrypoint } => assert!(entrypoint.is_empty()),
            other => panic!("expected EntrypointMissing, got {other:?}"),
        }
    }

    #[test]
    fn invoke_reports_adapter_not_configured_when_validation_passes() {
        let rt = ReferenceWasmRuntime::new();
        let inv = WasmInvocation {
            module: WasmModuleBytes(b"\0asm\x01\x00\x00\x00 minimal".to_vec()),
            entrypoint: "run".into(),
            input: vec![],
            token: token(SandboxClass::EnvoyFilter),
        };
        let err = rt.invoke(SandboxClass::EnvoyFilter, &inv).unwrap_err();
        match err {
            WasmRuntimeError::AdapterNotConfigured(name) => {
                assert!(name.contains("wasmtime-real"));
            }
            other => panic!("expected AdapterNotConfigured, got {other:?}"),
        }
    }

    #[test]
    fn capability_token_must_have_non_empty_call_site() {
        let rt = ReferenceWasmRuntime::new();
        let mut t = token(SandboxClass::FoundryTool);
        t.call_site = String::new();
        let inv = WasmInvocation {
            module: WasmModuleBytes(b"\0asm\x01\x00\x00\x00".to_vec()),
            entrypoint: "run".into(),
            input: vec![],
            token: t,
        };
        let err = rt.invoke(SandboxClass::FoundryTool, &inv).unwrap_err();
        assert_eq!(err, WasmRuntimeError::CapabilityTokenInvalid);
    }

    #[test]
    fn display_renders_sandbox_classes() {
        assert_eq!(SandboxClass::EnvoyFilter.to_string(), "envoy-filter");
        assert_eq!(
            SandboxClass::WorkflowStudioNode.to_string(),
            "workflow-studio-node"
        );
        assert_eq!(SandboxClass::FoundryTool.to_string(), "foundry-tool");
    }
}
