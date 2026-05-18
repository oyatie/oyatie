# feedback_oya_git_canonical_2026_05_18

Canonical as of 2026-05-18:

- Git drop-in operations use `oya git <git-subcommand>`.
- `oya git` shells out to git, preserves stdout / stderr / exit status, and emits a local ledger side channel under `.git/oya/ledger/audit-chain.jsonl`.
- Ledger rows must not record raw arguments or absolute local paths.
- `oya vcs <claim|work|verify|done|status|symbols|queue|watch|promote>` remains the compatibility policy-ratchet surface until explicit policy verbs split out.
- Do not infer claim / work / done state from git verbs.
- Do not auto-create PRs from `oya git push`.
- Do not add conflict-radar behavior to `oya git` v1.

Authority:

- ADR-0223
- evidence/pr-159-adr-0223-doubt-driven-design-checkpoint-2026-05-18.json
- tools/hooks/_canonical-primitives.md
