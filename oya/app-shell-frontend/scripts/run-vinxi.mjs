#!/usr/bin/env node
// ADR-0393 retirement fence: SolidJS/Vinxi is no longer the canonical app shell.
// The production shell is the Leptos/Rust-WASM frontend under
// oya/application/crates/oya-application-shell-frontend.

console.error("ADR-0393 retired the SolidJS/Vinxi app-shell bridge; use the Leptos/Rust-WASM app shell instead.");
process.exit(1);
