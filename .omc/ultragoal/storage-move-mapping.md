# Storage capability — SOUND face-mapping (move-5, from workflow wpfvj2fw4, critic verdict SOUND)

Dispatch the move-5 executor with this once #739 post-merge oya-ci-required is GREEN (serial discipline). Worktree: /Users/jasonlee/oyatie-worktrees/p7-storage (branch agent/p7-storage @ dev 8a0c9dbde — REBASE/recreate onto the post-#739 dev tip before executing).

## Final mapping (7 crates, 3 source dirs; collision-free, dep-legal)
| old_path | new_path | cargo | face |
|---|---|---|---|
| cloud/cloud-storage/crates/oya-cloud-storage-domain | storage/core/domain | storage-domain | core |
| cloud/cloud-storage/crates/oya-cloud-storage-object-api | storage/ports/object-api | storage-object-api | ports |
| cloud/cloud-storage/crates/oya-cloud-storage-block-api | storage/ports/block-api | storage-block-api | ports |
| cloud/cloud-storage/crates/oya-cloud-storage-adapter-s3 | storage/adapters/s3 | storage-s3-adapter | adapters |
| cloud/cloud-storage/crates/oya-cloud-storage-adapter-oci | storage/adapters/oci | storage-oci-adapter | adapters |
| oya/drive/crates/oya-drive-domain | storage/facade/drive | storage-drive-domain | facade |
| oya/recordings/crates/oya-recordings-domain | storage/facade/recordings | storage-recordings-domain | facade |

## Face reasoning
cloud-storage = CORE substrate (storage-domain DEFINES the outbound provider port traits StorageProviderObjectPort/StorageProviderBlockPort + StorageRepo; s3/oci adapters implement them — the engine-we-RUN). object-api/block-api = ports (inbound capability boundary surfaces; ports->core legal, iac-rest precedent). s3/oci = adapters (transient provider infra). oya/drive + oya/recordings = facade (oya PRODUCTS consuming the storage substrate — §2 iam-pattern; they don't depend on cloud-storage at Cargo level but their product charter sits on the object/blob substrate). Collision: the 'domain' triple-clash disambiguated by face+leaf (storage-domain=core; storage-drive-domain/storage-recordings-domain=facades). storage/ top-level dir confirmed absent.

## Critic notes (handle, no mapping change)
1. block-api -> oya-residency-domain is a RUNTIME cross-cap dep (cloud-network), NOT dev-only — codemod preserves cross-cap deps; just the moved crate's own path changes.
2. oya-drive-domain/src/lib.rs:6 doc comment carries the retired-brand "Foundry" stem ("...consumed by Search and Foundry."). SCRUB it (comment-only de-brand: drop the retired brand, e.g. "...consumed by Search and intelligence consumers." — NO identifier/behavior change; removing residue is always gate-allowed). This keeps forbidden_foundry clean at the new path. (oya/drive/manifest.json residue is OUTSIDE the crate -> phase-2 de-brand lane #63.)

## External dependent to rewrite (codemod)
oya/application/crates/oya-workspace-drive-api — depends on oya-drive-domain via Cargo.toml path + BUCK lib+test deps -> retarget to //storage/facade/drive:storage-drive-domain. (Sole code dependent; non-code refs in Cargo.lock/docs/registry/specs are metadata, phase-2.)

## Move protocol (per playbook): commit ONE specs/reorg/storage-move-plan.json, regenerate manifest, hard-gate buck2 dry-run, contract interactions (members glob storage/*/*, registry absorbs_current_dirs->storage, membership scan_roots+allowed_top_level_dirs+=storage, acyclicity crate_root_globs+=storage/*/* + unclassified_roots+=storage), born-accounting (storage/OWNERS + ADR §10.x verbatim paths + reachability seeds), full gate suite GREEN vs merge-base, grep-clean, forbidden_* 0 regression.
