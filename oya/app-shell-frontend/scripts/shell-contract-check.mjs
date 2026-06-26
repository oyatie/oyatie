#!/usr/bin/env node
// ADR-0393 retirement fence: the package-level MJS lint/test bridge is inert.
// The deterministic retirement contract lives in the Rust-native
// app_shell_frontend_retirement.rs check.

console.error("ADR-0393 retired the SolidJS shell-contract MJS bridge; use the Rust app-shell retirement check instead.");
process.exit(1);
