#!/usr/bin/env bash
# Enforce Oyatie domain intent:
#   * .com hosts are public/customer-facing
#   * .dev hosts are allowed only for internal/developer infrastructure
set -euo pipefail

root="${1:-.}"

python3 - "$root" <<'PY'
from __future__ import annotations

import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1]).resolve()
host_atom = r"(?:[A-Za-z0-9.-]|\{[A-Za-z0-9_-]+\}|<[^\s/>{}\\\"']+>|\{\{[^\n}]+\}\})+"
dev_url_re = re.compile(r"https://(" + host_atom + r"[.]oyatie[.]dev)")
root_dev_url_re = re.compile(r"https://oyatie[.]dev(?=[:/\"'\s]|$)")
dev_email_re = re.compile(r"(?<![A-Za-z0-9._%+-])([A-Za-z0-9._%+-]+@oyatie[.]dev)(?![A-Za-z0-9.-])")
public_host_field_re = re.compile(r"(?im)^\s*(?:public_name|public_host|public_endpoint|external_host|customer_host|ExternalDomain|RPID|host|Host|vhost)\s*[:=]\s*[\"']?(" + host_atom + r"[.]oyatie[.]dev)(?::[0-9]+)?")
identity_pack_host_re = re.compile(r"(?<![A-Za-z0-9.<{_-])(identity-(?:<pack>|\{pack\}|\{\{[^\n}]+\}\})[.]oyatie[.]dev)(?![A-Za-z0-9_.}>-])")
identity_global_host_re = re.compile(r"(?<![A-Za-z0-9.-])(identity[.]oyatie[.]dev)(?::[0-9]+)?(?![A-Za-z0-9.-])")
gateway_hostname_item_re = re.compile(r"(?im)^\s*-\s*[\"']?(" + host_atom + r"[.]oyatie[.]dev)[\"']?\s*$")
non_owned_public_domain_re = re.compile(r"(?<![A-Za-z0-9.-])([A-Za-z0-9.-]+[.]oya[.]cloud|[A-Za-z0-9.*-]+[.]oyatie[.]app)(?![A-Za-z0-9.-])")
bare_public_docs_re = re.compile(r"(?<![A-Za-z0-9.-])(docs|schemas)[.]oyatie[.]dev")

ignored_names = {".git", "target", "node_modules", ".next", "dist", "build", ".omx"}
ignored_prefixes = (
    "docs/decisions/",
    "docs/raw/",
    "docs/archive/",
    ".omc/",
    "evidence/",
    "crates/oya-llm-gateway",
    "microservices/llm-gateway/",
)
ignored_suffixes = (
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".webp",
    ".ico",
    ".pdf",
    ".zip",
    ".gz",
    ".tar",
    ".tgz",
)
allowed_internal_dev_hosts = {
    "join.oyatie.dev",
    "sidero.oyatie.dev",
    "sidero-link.oyatie.dev",
    "pypi.oyatie.dev",
    "openbao.pack-kr.oyatie.dev",
    "argocd-eu.oyatie.dev",
    "argocd-kr.oyatie.dev",
    "argocd-<pack>.oyatie.dev",
    "grafana-<pack>.oyatie.dev",
    "k8s-api-<pack>.oyatie.dev",
}


def relpath(path: pathlib.Path) -> str:
    return path.relative_to(root).as_posix()


def ignored(path: pathlib.Path) -> bool:
    rel = relpath(path)
    if any(part in ignored_names for part in path.parts):
        return True
    if rel.endswith(ignored_suffixes):
        return True
    if any(rel.startswith(prefix) for prefix in ignored_prefixes):
        return True
    if rel in {
        "scripts/reject-public-dev-domains.sh",
        "scripts/tests/reject-public-dev-domains.test.sh",
    }:
        return True
    return False


failures: list[str] = []
for path in sorted(root.rglob("*")):
    if not path.is_file() or ignored(path):
        continue
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        continue
    rel = relpath(path)

    for match in dev_url_re.finditer(text):
        host = match.group(1)
        if ".internal." in host or host in allowed_internal_dev_hosts:
            continue
        failures.append(f"{rel}: public-looking .dev URL {match.group(0)!r}; use .com or an allowed internal .dev host")

    for match in root_dev_url_re.finditer(text):
        failures.append(f"{rel}: public-looking root .dev URL {match.group(0)!r}; use .com")

    for match in dev_email_re.finditer(text):
        failures.append(f"{rel}: public contact email uses .dev address {match.group(1)!r}; use .com")

    for match in public_host_field_re.finditer(text):
        host = match.group(1)
        if ".internal." in host or host in allowed_internal_dev_hosts:
            continue
        failures.append(f"{rel}: public-facing host field uses .dev host {host!r}; use .com or an allowed internal .dev host")

    for match in identity_pack_host_re.finditer(text):
        failures.append(f"{rel}: identity pack endpoint {match.group(1)!r} is public-facing; use .com")

    for match in identity_global_host_re.finditer(text):
        failures.append(f"{rel}: global identity endpoint {match.group(1)!r} is public-facing; use .com")

    if "/iac/" in rel:
        for match in gateway_hostname_item_re.finditer(text):
            host = match.group(1)
            if ".internal." in host or host in allowed_internal_dev_hosts:
                continue
            failures.append(f"{rel}: deployable hostnames entry uses .dev host {host!r}; use .com or an allowed internal .dev host")

        for match in non_owned_public_domain_re.finditer(text):
            failures.append(f"{rel}: deployable public host {match.group(1)!r} is outside owned .com/.net/.org/.dev domains; use .com for public surfaces")

    if bare_public_docs_re.search(text):
        failures.append(f"{rel}: docs/schemas.oyatie.dev is public-facing; use .com")

if failures:
    print("public .dev domain gate failed:", file=sys.stderr)
    for failure in failures:
        print(f"  - {failure}", file=sys.stderr)
    sys.exit(1)

print("public .dev domain gate passed")
PY
