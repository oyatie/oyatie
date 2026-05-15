#!/usr/bin/env bash
# Failing fixture for check_rust_default_language.
# .sh extension violates P15 (Rust default) — counter increments.
# Resolution per audit: F-PORT-CHECK-SH-TO-RUST or similar Rust port task.
echo "this script should be a Rust binary per P15"
