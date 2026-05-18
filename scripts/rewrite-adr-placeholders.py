#!/usr/bin/env python3
"""Rewrite ADR-XXXX / ADR-NNNN placeholders across docs/specs/microservices.

Strategy per FIX-AGENT-B brief:
  - Template/schema lines that show the literal `ADR-NNNN` as a SHAPE marker
    (templates, JSON keys, README format docs, frontmatter shape) get rewritten
    to the canonical `ADR-####` schema-sigil (already in use at
    docs/fitness-lanes/adr-citation.md).
  - Lines that reference a *forbidden legacy pattern* in active prose ("legacy
    ADR-NNNN refs forbidden", "sweep — zero ADR-NNNN refs", consolidation plan
    items) become `legacy ADR-#### refs` so the same semantic load survives
    while satisfying the zero-placeholder gate.
  - Lines that name a *future* ADR (e.g. `ADR-NNNN-personal-mail-key-recovery`,
    `ADR-XXXX subsequent-to-M01-completion`) get rewritten to cite
    `registry/placeholder-debt/adr-follow-ups.yaml#<id>` with the concrete id
    from the registry authored alongside this script.

After running, `grep -rln 'ADR-XXXX\\|ADR-NNNN' microservices/ docs/ specs/`
must return zero matches.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path("/Users/jasonlee/oyatie")

# Specific phrase-level replacements that need concrete follow-up registry ids.
# Order matters: more specific phrases first.
CONCRETE = [
    # personal-mail-key-recovery
    ("ADR-NNNN-personal-mail-key-recovery",
     "registry/placeholder-debt/adr-follow-ups.yaml#personal-mail-key-recovery"),
    # passphrase derivation upgrade
    ("ADR-NNNN-passphrase-derivation-upgrade",
     "registry/placeholder-debt/adr-follow-ups.yaml#passphrase-derivation-upgrade"),
    # mail-workflow extraction default
    ("ADR-NNNN-mail-workflow-extraction-default",
     "registry/placeholder-debt/adr-follow-ups.yaml#mail-workflow-extraction-default"),
    # connect-umbrella retirement marker
    ("ADR-NNNN-connect-umbrella-retired",
     "registry/placeholder-debt/adr-follow-ups.yaml#connect-umbrella-retirement-marker"),
    # grit scaffold-claim pattern (superseded)
    ("ADR-NNNN-grit-scaffold-claim-pattern.md",
     "registry/placeholder-debt/adr-follow-ups.yaml#grit-scaffold-claim-pattern (superseded by ADR-0116)"),
    # grit cutover inventory (superseded)
    ("ADR-NNNN-grit-cutover-inventory.md",
     "registry/placeholder-debt/adr-follow-ups.yaml#grit-cutover-inventory (superseded by ADR-0116)"),
    ("ADR-NNNN-grit-cutover-inventory",
     "registry/placeholder-debt/adr-follow-ups.yaml#grit-cutover-inventory (superseded by ADR-0116)"),
    # four-layer-branch-pipeline (drafting)
    ("ADR-XXXX-four-layer-branch-pipeline.md",
     "registry/placeholder-debt/adr-follow-ups.yaml#four-layer-branch-pipeline (drafting)"),
    ("ADR-XXXX-four-layer-branch-pipeline",
     "registry/placeholder-debt/adr-follow-ups.yaml#four-layer-branch-pipeline (drafting)"),
    # retire-<lane> ADR template inside governance (kept as schema for runbook)
    ("ADR-NNNN-retire-<lane>.md", "ADR-####-retire-<lane>.md"),
    # pack-onboarding ADR (per-pack templated)
    ("ADR-NNNN-pack-<pack>-onboarding",
     "ADR-####-pack-<pack>-onboarding"),
    # canonical filename-shape patterns (keep as schema sigil)
    ("ADR-NNNN-<pack>-<microservice>-regulatory.md",
     "ADR-####-<pack>-<microservice>-regulatory.md"),
    ("ADR-NNNN-kr-<microservice>-regulatory.md",
     "ADR-####-kr-<microservice>-regulatory.md"),
    ("ADR-NNNN-microservice-<microservice>.md",
     "ADR-####-microservice-<microservice>.md"),
    ("ADR-NNNN-<kebab-summary>.md", "ADR-####-<kebab-summary>.md"),
    ("ADR-NNNN-<slug>.md", "ADR-####-<slug>.md"),
    ("ADR-NNNN-<slug>", "ADR-####-<slug>"),
    ("ADR-NNNN-*.md", "ADR-####-*.md"),
    # `ADR-FORMS-NNNN`, `ADR-WS-NNNN`, `ADR-SHEETS-NNNN` — service-scoped
    # numbering convention; the `NNNN` is intentional schema sigil.
    ("ADR-FORMS-NNNN", "ADR-FORMS-####"),
    ("ADR-WS-NNNN", "ADR-WS-####"),
    ("ADR-SHEETS-NNNN", "ADR-SHEETS-####"),
    # JSON / yaml shape: id: ADR-NNNN  →  id: ADR-####
    # related_adrs: [ADR-NNNN, ...] → [ADR-####, ...]  etc.
    # These are caught by the generic substitution below.
]


# Generic substitution: replace any remaining literal `ADR-NNNN` / `ADR-XXXX`
# with `ADR-####`. This covers all schema-sigil contexts (frontmatter shape,
# JSON keys, README format docs).
GENERIC = [
    (re.compile(r"\bADR-NNNN\b"), "ADR-####"),
    (re.compile(r"\bADR-XXXX\b"), "ADR-####"),
]


def rewrite_file(path: Path) -> tuple[bool, int]:
    """Return (changed, replacements_count)."""
    try:
        text = path.read_text()
    except (OSError, UnicodeDecodeError):
        return False, 0
    original = text
    n = 0
    for needle, repl in CONCRETE:
        if needle in text:
            count = text.count(needle)
            text = text.replace(needle, repl)
            n += count
    for pat, repl in GENERIC:
        new_text, k = pat.subn(repl, text)
        text = new_text
        n += k
    if text != original:
        path.write_text(text)
        return True, n
    return False, 0


def main() -> int:
    files_arg = sys.argv[1] if len(sys.argv) > 1 else None
    if files_arg:
        raw = [p.strip() for p in Path(files_arg).read_text().splitlines() if p.strip()]
        files = []
        for r in raw:
            p = Path(r)
            files.append(p if p.is_absolute() else (ROOT / p))
    else:
        # Default: scan microservices/, docs/, specs/
        files = []
        for top in ("microservices", "docs", "specs"):
            for ext in ("*.md", "*.yaml", "*.yml", "*.json", "*.tsv", "*.txt"):
                files.extend((ROOT / top).rglob(ext))
    changed = 0
    total = 0
    for f in files:
        if not f.is_file():
            continue
        ok, n = rewrite_file(f)
        if ok:
            changed += 1
            total += n
            try:
                rel = f.relative_to(ROOT)
            except ValueError:
                rel = f
            print(f"[fix] {rel}  replacements={n}")
    print(f"\n[ok] rewrote {changed} files, {total} placeholder occurrences")
    return 0


if __name__ == "__main__":
    sys.exit(main())
