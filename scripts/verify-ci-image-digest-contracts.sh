#!/usr/bin/env bash
# Verify Jenkins Kubernetes agents fail closed on CI image digest injection.
set -euo pipefail

root="${1:-.}"

python3 - "$root" <<'PY'
from __future__ import annotations

import pathlib
import sys

root = pathlib.Path(sys.argv[1]).resolve()
jenkinsfiles = sorted((root / "microservices").glob("*/ci/Jenkinsfile"))
failures: list[str] = []

for path in jenkinsfiles:
    rel = path.relative_to(root).as_posix()
    if rel.startswith("microservices/llm-gateway/"):
        continue
    text = path.read_text(encoding="utf-8")
    if "registry.oyatie.dev/ci/rust:stable@" not in text:
        continue
    required = [
        "String requireRealCiImageDigest(String variableName)",
        "env[variableName]?.trim()",
        "/^sha256:[0-9a-f]{64}$/",
        "/^sha256:0{64}$/",
        "registry.oyatie.dev/ci/rust:stable@${requireRealCiImageDigest('OYA_CI_RUST_IMAGE_DIGEST')}",
    ]
    missing = [needle for needle in required if needle not in text]
    if missing:
        failures.append(f"{rel}: missing {', '.join(missing)}")
    if "/^sha256:0+$/" in text:
        failures.append(f"{rel}: accepts variable-length/all-zero digest with a permissive OR check")
    if "registry.oyatie.dev/ci/rust:stable@${env.OYA_CI_RUST_IMAGE_DIGEST}" in text:
        failures.append(f"{rel}: raw env digest interpolation bypasses fail-closed validator")

if failures:
    print("CI image digest contract gate failed:", file=sys.stderr)
    for failure in failures:
        print(f"  - {failure}", file=sys.stderr)
    sys.exit(1)

print(f"CI image digest contract gate passed ({len(jenkinsfiles)} Jenkinsfiles checked)")
PY
