# Migration Plan: ADR-0357 Vertical-Slice Monorepo Nesting

Status: READY-TO-EXECUTE (pending backbone PR #363 merge + ADR acceptance)
Authored: 2026-05-29
Author: council-architecture (executor agent)

## Prerequisites

- [x] Enterprise bundle landed in dev (PR #181)
- [x] Workflow bundle landed in dev
- [ ] Backbone bundle landed in dev (PR #363 open, compiles clean, 0 conflicts — pending merge)
- [ ] ADR-0357 status promoted from Proposed to Accepted (requires founder/council decision)

## Scope (as of 2026-05-29 dev HEAD)

- Crates in `crates/`: 504
- Microservice directories in `microservices/`: 100
- Estimated shared/libs crates: ~50 (oya-shared-*, oya-data-boundary-kernel, oya-check-*, tools)
- Estimated service-owned crates: ~454

## Classification Rules

A crate is **shared (→ libs/)** if ANY of the following:
1. Name starts with `oya-shared-`
2. Name starts with `oya-check-` (governance check crates — cross-cutting tooling)
3. Name is `oya-data-boundary-kernel` or `oya-data-class-*` (data classification primitives)
4. Name starts with `oya-governance-` (multi-service governance)
5. More than one microservice's crates depend on it (determined by Cargo.toml dep scanning)

A crate is **service-owned (→ microservices/<ms>/crates/)** if:
1. Its name prefix matches a microservice directory name in `microservices/`
2. It is not shared by the classification above

## Crate-to-Microservice Mapping Rules

For crate `oya-<service>-<layer>`, the owning microservice is determined by:
1. Exact prefix match: `oya-<service>` where `<service>` is a directory in `microservices/`
2. Longest prefix wins when multiple match (e.g. `oya-cloud-iam-*` → `cloud-iam`, not `cloud`)
3. `oya-connect-*` → `connect`
4. `oya-comms-*` → check if `comms` or specific sub-service directory exists
5. Unresolved crates → flag for manual classification

## Execution Steps

### Step 1: Verify pre-migration gate (before any moves)

```bash
cargo check --workspace --all-targets
cargo nextest run --workspace
```
Record output as baseline evidence.

### Step 2: Build classification manifest

Run the classifier script (see below) to generate:
- `tasks/adr-0357-crate-classification.json` — full crate→destination mapping
- `tasks/adr-0357-unresolved-crates.txt` — crates requiring manual classification

### Step 3: Create destination directories

```bash
# For each microservice that will receive crates:
mkdir -p microservices/<ms>/crates
# For shared libs:
mkdir -p libs/<lib-name>
```

### Step 4: Execute git mv for each crate

```bash
# For each service-owned crate:
git mv crates/oya-<service>-<layer> microservices/<ms>/crates/oya-<service>-<layer>
# For each shared crate:
git mv crates/oya-shared-<name> libs/oya-shared-<name>
```

### Step 5: Update root Cargo.toml workspace members

Replace all `"crates/oya-*"` entries with:
- `"microservices/<ms>/crates/oya-<service>-<layer>"` for service crates
- `"libs/oya-<shared-name>"` for shared crates

### Step 6: Update architecture-boundaries gate

In the `oya-check-architecture-boundaries` crate, flip the enforcement from:
- `crates/oya-*` → service-owned
- (no libs/ path)

To:
- `microservices/<ms>/crates/oya-*` → service-owned (validated against manifest)
- `libs/oya-*` → shared libs
- `crates/oya-*` → ERROR (migration not complete)

### Step 7: Update registry catalog path fields

Scan `registry/` JSON files for any `crates/oya-*` path references and update to new paths.

### Step 8: Verify post-migration gate

```bash
cargo check --workspace --all-targets
cargo nextest run --workspace
```
Both must pass before the migration PR is opened.

### Step 9: Open migration PR against dev

PR title: `chore(adr-0357): vertical-slice nesting — move 504 crates to microservices/<ms>/crates/ + libs/`

## Classifier Script (Python)

```python
#!/usr/bin/env python3
"""
ADR-0357 crate classifier.
Run from repo root after backbone PR #363 merges.
"""
import os, json, re, subprocess

CRATES_DIR = "crates"
MS_DIR = "microservices"
LIBS_DIR = "libs"

SHARED_PREFIXES = [
    "oya-shared-",
    "oya-check-",
    "oya-data-boundary-",
    "oya-data-class-",
    "oya-governance-",
]

def get_microservice_dirs():
    return sorted([
        d for d in os.listdir(MS_DIR)
        if os.path.isdir(os.path.join(MS_DIR, d)) and not d.startswith(".")
    ])

def is_shared(crate_name):
    return any(crate_name.startswith(p) for p in SHARED_PREFIXES)

def find_owning_ms(crate_name, ms_dirs):
    # Strip "oya-" prefix, find longest matching ms dir
    stem = crate_name.removeprefix("oya-")
    matches = [d for d in ms_dirs if stem.startswith(d.replace("-", "_")) or stem.startswith(d)]
    if not matches:
        # Try hyphen-normalised match
        matches = [d for d in ms_dirs if stem.startswith(d)]
    if not matches:
        return None
    return max(matches, key=len)  # longest prefix wins

ms_dirs = get_microservice_dirs()
crates = sorted(os.listdir(CRATES_DIR))

results = {"service_crates": {}, "shared_crates": [], "unresolved": []}

for crate in crates:
    if not os.path.isdir(os.path.join(CRATES_DIR, crate)):
        continue
    if is_shared(crate):
        lib_name = crate.removeprefix("oya-shared-")
        results["shared_crates"].append({
            "crate": crate,
            "source": f"crates/{crate}",
            "dest": f"libs/{crate}",
        })
    else:
        ms = find_owning_ms(crate, ms_dirs)
        if ms:
            results["service_crates"][crate] = {
                "source": f"crates/{crate}",
                "dest": f"microservices/{ms}/crates/{crate}",
                "microservice": ms,
            }
        else:
            results["unresolved"].append(crate)

print(f"Service crates: {len(results['service_crates'])}")
print(f"Shared crates: {len(results['shared_crates'])}")
print(f"Unresolved: {len(results['unresolved'])}")
if results["unresolved"]:
    print("Unresolved:", results["unresolved"][:10])

with open("tasks/adr-0357-crate-classification.json", "w") as f:
    json.dump(results, f, indent=2)
```

## Risk Mitigation

- Package names unchanged — all imports, `use` statements, and `Cargo.toml` dep names are stable.
- `Cargo.lock` is unchanged in content (only paths change).
- The migration is a single atomic commit to minimise bisect difficulty.
- `oya verify --ci-required` green before and after is the acceptance gate.
- If the gate fails post-migration, `git revert` restores the flat layout instantly (package names unchanged means downstream is unaffected).

## Notes

- The `oya-check-*` crates are governance check crates that validate the whole workspace — they are cross-cutting by design and belong in `libs/`.
- The `tools/` directory crates (e.g. `tools/oya-governance-purpose-audit-app/`) are already outside `crates/` and are unaffected by this migration.
- CODEOWNERS entries referencing `crates/oya-<service>-*` must be updated to `microservices/<ms>/crates/oya-<service>-*`.
