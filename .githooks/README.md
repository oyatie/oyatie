# Tracked git hooks

These hooks are the reviewed, clone-visible enforcement surface. They are not
enabled by anything in the tree: a fresh clone has no `core.hooksPath` until
you set it locally.

```
git config core.hooksPath .githooks
```

`pre-commit` refuses staged paths under gitignored agent directories
(`.claude/`, `.codex/`, `.cursor/`, `.grok/`, …). `pre-push` delegates to any
local untracked hook; it does not run `cargo` (PORTABLE hard-ban 11 / ADR-0711
D-9: worker lanes must not compile).

Local, untracked hooks still run via the delegation at the end of each script
(`.beads/hooks/<name>` or `.git/hooks/<name>.local`).
