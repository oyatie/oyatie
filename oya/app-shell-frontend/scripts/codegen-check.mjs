#!/usr/bin/env node
// ADR-0393 retirement fence: the archived SolidJS app-shell is not a merge gate.
// Run `pnpm --dir oya/app-shell-frontend codegen`, then run the Rust-native
// `app_shell_frontend_retirement.rs` check used by CI.

console.error("ADR-0393 retired this SolidJS codegen-check MJS bridge; use the Rust app-shell retirement check instead.");
process.exit(1);
