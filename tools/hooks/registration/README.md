# Hook registration, tracked

The hook *scripts* live in `tools/hooks/`. These files are the per-tool
**registration** that wires those scripts into an agent runtime.

They used to live at `.claude/settings.json` and `.codex/hooks.json`. Those are
agent working directories, which must not be tracked — but the
enforcement-liveness gate reads the registration to prove the hooks are actually
wired, and a CI checkout lacking it cannot tell a wired hook from an unwired
one. The producer errors on an absent corpus file rather than reporting that it
could not measure, so untracking without moving first turns five
`baseline-ratchet` firewall tests red.

The canonical copy is therefore here, tracked and reviewed. Claude Code still
loads the gitignored `.claude/settings.json` locally; Codex still loads
`.codex/hooks.json`. Those local copies can drift — they are not CI corpus.
There is no `hook_registration_copies_agree` test.

`.cargo/config.toml` sets `OYA_CI_ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS` and
`..._CODEX_HOOKS` to these paths. That env var **overrides** the constants in
the enforcement-liveness consumers.

To enable the tracked git hooks in a clone:

```
git config core.hooksPath .githooks
```
