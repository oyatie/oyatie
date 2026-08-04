# G024 contract self-review — 2026-08-02

Candidate: PR #1528 head `da46906d02408cef255f3a678ff5e047fe8a3d44`
File: `specs/reorg/intelligence-remainder-move-plan.json` only
State: `WRITE_COMPLETE_SELF_REVIEWED_NOT_INDEPENDENTLY_REVIEWED_NOT_ADMITTED`

## Contract checks (coordinator; NOT independent approval)

- moves=78 artifacts=78 capability=intelligence
- live oya/intelligence/crates census equality: true
- prior plan old_path overlap: 0
- destination package collision: 0
- faces: core=60 adapters=18; no ports/facade destinations invented
- catalog ArtifactMove rows are registry/catalog/*.yaml with stems 1:1 to cargo names (matches landed intelligence plans)
- debrand: only leading oya-/cloud- forbidden; `intelligence/core/cloud-mutation-domain` permitted as non-prefix descriptor (model unit test `validate_allows_debranded_targets_that_keep_cloud_as_a_non_prefix_descriptor`)
- redundant adapter leaf names normalized (no `intelligence/adapters/adapter-*`)
- Buck2 codemod unittest previously 99/0; manifest load rc0 at authoring

## Independent review

- Multiple transport failures (`encrypted_content` decrypt / connection closed)
- No APPROVE from an independent reviewer agent yet
- Self-review is not admission authority

## Admission blockers remaining

1. Independent exact-object APPROVE
2. Candidate protected CI green including oya-ci-required
3. No merge while corpus repair #1526 is still not promoted-green and Stage A promoted tip is binding-red
