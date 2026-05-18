#!/usr/bin/env bash
# ADR-0221 governance gate harness.
#
# Purpose: CI-owned non-vacuous checks for the guidance hooks introduced by
# ADR-0221. Each gate creates a local fixture that must trigger the hook and a
# clean fixture that must not trigger the same failure class.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

gate="${1:-}"

fail() {
    local title="$1"
    local message="$2"
    echo "::error title=${title}::${message}" >&2
    exit 1
}

run_hook() {
    local script="$1"
    local path="$2"
    TOOL_INPUT="{\"path\":\"$path\"}" bash "$REPO_ROOT/$script" 2>&1
}

require_contains() {
    local output="$1"
    local needle="$2"
    local title="$3"
    if [[ "$output" != *"$needle"* ]]; then
        fail "$title" "expected hook output to contain: $needle"
    fi
}

require_not_contains() {
    local output="$1"
    local needle="$2"
    local title="$3"
    if [[ "$output" == *"$needle"* ]]; then
        fail "$title" "clean fixture unexpectedly contained: $needle"
    fi
}

case "$gate" in
    vacuous-green)
        fixture_dir="$REPO_ROOT/crates/oya-check-governance-fixture-$$"
        fixture_rel="crates/oya-check-governance-fixture-$$/src/lib.rs"
        trap 'rm -rf "$fixture_dir"' EXIT
        mkdir -p "$fixture_dir/src"
        printf '#[test]\nfn vacuous() {\n    assert!(true);\n}\n' > "$REPO_ROOT/$fixture_rel"
        output="$(run_hook tools/hooks/vacuous-green-gate-detect.sh "$fixture_rel")"
        require_contains "$output" "Possible vacuous-green pattern" "oya-governance-vacuous-green"

        printf '#[test]\nfn real_assertion() {\n    assert_eq!(2 + 2, 4);\n}\n' > "$REPO_ROOT/$fixture_rel"
        output="$(run_hook tools/hooks/vacuous-green-gate-detect.sh "$fixture_rel")"
        require_not_contains "$output" "Possible vacuous-green pattern" "oya-governance-vacuous-green"
        echo "oya-governance-vacuous-green passed"
        ;;

    orphan-citation)
        fixture_dir="$REPO_ROOT/target/adr-0221-governance-fixtures/$$"
        fixture_rel="target/adr-0221-governance-fixtures/$$/orphan.md"
        trap 'rm -rf "$fixture_dir"' EXIT
        mkdir -p "$fixture_dir"
        printf 'References missing ADR-7777.\n' > "$REPO_ROOT/$fixture_rel"
        output="$(run_hook tools/hooks/adr-orphan-detect.sh "$fixture_rel")"
        require_contains "$output" "ADR-7777" "oya-governance-adr-orphan-citation"

        printf 'References existing ADR-0001.\n' > "$REPO_ROOT/$fixture_rel"
        output="$(run_hook tools/hooks/adr-orphan-detect.sh "$fixture_rel")"
        require_not_contains "$output" "Orphan ADR references" "oya-governance-adr-orphan-citation"
        echo "oya-governance-adr-orphan-citation passed"
        ;;

    version-pin)
        fixture_dir="$REPO_ROOT/target/adr-0221-governance-fixtures/$$"
        fixture_rel="target/adr-0221-governance-fixtures/$$/contract.yaml"
        trap 'rm -rf "$fixture_dir"' EXIT
        mkdir -p "$fixture_dir"
        printf 'openapi: 3.1.0\ninfo:\n  title: fixture\n  version: 1.0.0\npaths: {}\n' > "$REPO_ROOT/$fixture_rel"
        output="$(run_hook tools/hooks/spec-version-pin-suggester.sh "$fixture_rel")"
        require_contains "$output" "canonical version is 3.2.0" "oya-governance-version-pin-source-citation"

        printf 'asyncapi: 3.0.0\ninfo:\n  title: fixture\n  version: 1.0.0\nchannels: {}\n' > "$REPO_ROOT/$fixture_rel"
        output="$(run_hook tools/hooks/spec-version-pin-suggester.sh "$fixture_rel")"
        require_contains "$output" "canonical version is 3.1.0" "oya-governance-version-pin-source-citation"

        printf 'openapi: 3.2.0\ninfo:\n  title: fixture\n  version: 1.0.0\npaths: {}\n' > "$REPO_ROOT/$fixture_rel"
        output="$(run_hook tools/hooks/spec-version-pin-suggester.sh "$fixture_rel")"
        require_not_contains "$output" "Detected openapi" "oya-governance-version-pin-source-citation"
        echo "oya-governance-version-pin-source-citation passed"
        ;;

    buildability-line-count)
        fixture_dir="$REPO_ROOT/microservices/governance-fixture-$$"
        short_rel="microservices/governance-fixture-$$/docs/short.md"
        long_rel="microservices/governance-fixture-$$/docs/long.md"
        trap 'rm -rf "$fixture_dir"' EXIT
        mkdir -p "$fixture_dir/docs"
        printf 'one substantive line\n' > "$REPO_ROOT/$short_rel"
        output="$(run_hook tools/hooks/buildability-line-count.sh "$short_rel")"
        require_contains "$output" "has 1 substantive lines" "oya-governance-buildability-line-count"

        : > "$REPO_ROOT/$long_rel"
        for index in $(seq 1 50); do
            printf 'substantive line %s\n' "$index" >> "$REPO_ROOT/$long_rel"
        done
        output="$(run_hook tools/hooks/buildability-line-count.sh "$long_rel")"
        require_contains "$output" "Buildability bar met" "oya-governance-buildability-line-count"
        echo "oya-governance-buildability-line-count passed"
        ;;

    *)
        echo "Usage: $0 <vacuous-green|orphan-citation|version-pin|buildability-line-count>" >&2
        exit 2
        ;;
esac
