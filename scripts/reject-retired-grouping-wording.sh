#!/usr/bin/env bash
set -euo pipefail

root="${1:-.}"

python3 - "$root" <<'PY'
from __future__ import annotations

import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1]).resolve()

# Retired grouping vocabulary is intentionally narrow. It catches active
# product/module/suite/platform wrappers while allowing unrelated English words
# (cipher suite, C-suite, platform as generic infra, Rust modules, vendor names,
# historical migration notes, and connector/integration terminology).
pattern = r'''oya-enterprise-suite|enterprise-suite|connect-suite|enterprise suite|connect suite|Enterprise Suite|Connect Suite|Productivity Suite|Documentation Suite|Doc Suite|DocSuite|doc-suite|docsuite|test-suite|test_suite|TestSuite|eval_suite|EvalSuite|suite_id|suite_boundary|suite_shell|suite_perimeter|suite_activation|suite-governance|suite-storage|suite-workflow|suite_policy|suite policy|suite gateway|suite shell|suite perimeter|product_class"[[:space:]]*:[[:space:]]*"suite|oya-enterprise-platform|enterprise-platform|connect-platform|enterprise platform|connect platform|Enterprise Platform|Connect Platform|EnterprisePlatform|ConnectPlatform|oya_enterprise_platform|oya_connect_platform|enterprise_platform|connect_platform|platform_id|platform_boundary|platform_shell|platform_perimeter|platform_activation|platform-governance|platform-storage|platform-workflow|product_class"[[:space:]]*:[[:space:]]*"platform"|product_class"[[:space:]]*:[[:space:]]*"platform-app"|oya-enterprise-module|enterprise-module|connect-module|healthcare-module|enterprise module|connect module|healthcare module|Enterprise Module|Connect Module|Healthcare Module|EnterpriseModule|ConnectModule|HealthcareModule|enterprise_module|connect_module|healthcare_module|module_id"[[:space:]]*:[[:space:]]*"(connect|enterprise|healthcare)"|module_boundary|module_shell|module_perimeter|module_activation|module-governance|module-storage|module-workflow|product_class"[[:space:]]*:[[:space:]]*"module"|connect-product|enterprise-product|healthcare-product|connect_product|enterprise_product|healthcare_product|connect product|enterprise product|healthcare product|Connect Product|Enterprise Product|Healthcare Product|ProductPrd:Connect|ProductPrd:Enterprise|PRD-CONNECT|PRD-ENTERPRISE|product"[[:space:]]*:[[:space:]]*"(connect|enterprise|healthcare)"|specs/products/connect|docs/products/connect|microservices/connect($|/)|name:[[:space:]]*oya-connect(?!or|[a-z0-9-])|repository:[[:space:]]*oya-connect(?!or|[a-z0-9-])|serviceAccount:[[:space:]]*\n[[:space:]]*name:[[:space:]]*oya-connect(?!or|[a-z0-9-])|connect[.]oyatie[.](dev|com|app)|[.]connect[.]oyatie[.](dev|com|app)|ns/connect/sa/connect-[a-z0-9-]+|connect[.]svc[.]cluster[.]local|oya-connect-[a-z0-9-]+-'''
compiled = re.compile(pattern.replace('[[:space:]]', r'\s'))

ignored_dir_names = {'.git', '.omc', 'target', 'node_modules'}
ignored_prefixes = (
    'evidence/',
    'registry/stub-audit/',
    'crates/oya-llm-gateway-',
    'microservices/llm-gateway/',
    'docs/decisions/ADR-0373',
    'docs/decisions/ADR-0384',
)
ignored_exact = {
    'scripts/reject-retired-grouping-wording.sh',
    'scripts/tests/reject-retired-grouping-wording.test.sh',
}
allowed = re.compile(
    r'(^|/)('
    r'gateway/adapters/netsuite-connector|'
    r'registry/catalog/gateway-netsuite-connector\.yaml|'
    r'scripts/reject-retired-grouping-wording\.sh|'
    r'scripts/tests/reject-retired-grouping-wording\.test\.sh|'
    r'specs/products/RETIREMENT\.md|'
    r'crates/oya-check-no-grouping/src/lib\.rs|'
    r'ADR-INVENTORY\.tsv|'
    r'docs/decisions/ADR-.*\.md|'
    r'microservices/.*/migration-from-connect\.md|'
    r'microservices/.*/deprecation-notice\.md|'
    r'microservices/connector/RETIREMENT-PLAN\.md|'
    r'docs/ADR-LEGACY-REGRESSION-MAPPING\.md|'
    r'docs/plans/rename-plan-v4-clean-arch-2026-05-13\.md|'
    r'docs/architecture/corpus-rigor-audit-.*\.md|'
    r'registry/milestone-audit/index\.json|'
    r'registry/graph/architecture-map\.json|'
    r'registry/stub-audit/.*'
    r')'
)

def relpath(path: pathlib.Path) -> str:
    return path.relative_to(root).as_posix()

def ignored(path: pathlib.Path) -> bool:
    rel = relpath(path)
    if rel in ignored_exact:
        return True
    if any(part in ignored_dir_names for part in path.relative_to(root).parts):
        return True
    if any(rel.startswith(prefix) for prefix in ignored_prefixes):
        return True
    return False

matches: list[str] = []
for dirpath, dirnames, filenames in __import__('os').walk(root):
    current = pathlib.Path(dirpath)
    rel_dir = '.' if current == root else current.relative_to(root).as_posix()
    dirnames[:] = [
        d for d in dirnames
        if d not in ignored_dir_names
        and not any(((rel_dir + '/' + d + '/') if rel_dir != '.' else (d + '/')).startswith(prefix) for prefix in ignored_prefixes)
    ]
    for name in sorted(filenames):
        path = current / name
        if ignored(path):
            continue
        rel = relpath(path)
        if allowed.search(rel):
            continue
        try:
            text = path.read_text(encoding='utf-8')
        except (UnicodeDecodeError, OSError):
            continue
        for lineno, line in enumerate(text.splitlines(), start=1):
            if compiled.search(line):
                matches.append(f'{rel}:{lineno}:{line}')

if matches:
    print(
        'Retired grouping wording found in active files. Use flat service/lib/kernel/application/infrastructure/doc-set/test-set/eval-set naming and tenant/RBAC packaging instead.',
        file=sys.stderr,
    )
    print('\n'.join(matches), file=sys.stderr)
    sys.exit(1)

print('retired grouping wording gate passed')
PY
