#!/usr/bin/env bash
set -euo pipefail
python3 - "$@" <<'PY'
import json
import pathlib
import subprocess
import sys

LEGACY_IMPLEMENTATION_DIRS = ("modules", "services", "platform")

ALLOWED_DEPENDENCY_ROLES = {
    "kernel": {"kernel"},
    "domain": {"kernel", "domain"},
    "app": {"kernel", "domain", "adapter"},
    "api": {"kernel", "domain", "app"},
    "worker": {"kernel", "domain", "app"},
    "adapter": {"kernel", "domain"},
    "runtime": {"kernel", "domain", "app", "api", "worker", "adapter", "runtime"},
}


def load_metadata():
    return json.loads(
        subprocess.check_output(
            ["cargo", "metadata", "--format-version", "1", "--no-deps"], text=True
        )
    )


def parse_catalog_record(contents):
    record = {}
    for line in contents.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#") or ":" not in stripped:
            continue
        key, value = stripped.split(":", 1)
        record[key.strip()] = value.strip()
    return record


def load_catalog_records(root, package_names):
    records = {}
    for name in package_names:
        path = root / "registry" / "catalog" / f"{name}.yaml"
        if path.exists():
            records[name] = parse_catalog_record(path.read_text())
    return records


def workspace_packages(metadata):
    workspace = set(metadata.get("workspace_members", []))
    return [package for package in metadata.get("packages", []) if package.get("id") in workspace]


def relative_path(path, root):
    path = pathlib.Path(path)
    try:
        return path.resolve().relative_to(root.resolve())
    except ValueError:
        return None


def validate_architecture(metadata, catalog_records, root, legacy_dirs_present=None):
    root = pathlib.Path(root)
    errors = []

    if legacy_dirs_present is None:
        legacy_dirs_present = {
            name for name in LEGACY_IMPLEMENTATION_DIRS if (root / name).exists()
        }

    for dirname in sorted(legacy_dirs_present):
        if dirname in LEGACY_IMPLEMENTATION_DIRS:
            errors.append(
                f"legacy implementation directory is forbidden by ADR-0015/PRD: {dirname}/"
            )

    packages = workspace_packages(metadata)
    by_name = {package["name"]: package for package in packages}

    for package in packages:
        name = package["name"]
        if not name.startswith("oya-"):
            errors.append(f"workspace package must use oya- prefix: {name}")

        manifest_parent = pathlib.Path(package["manifest_path"]).parent
        relative_manifest_parent = relative_path(manifest_parent, root)
        expected_parent = pathlib.Path("crates") / name
        if relative_manifest_parent != expected_parent:
            actual = str(relative_manifest_parent) if relative_manifest_parent else str(manifest_parent)
            errors.append(
                f"workspace package {name} must live at {expected_parent}, found {actual}"
            )

        if name not in catalog_records:
            errors.append(f"missing catalog record for {name}: registry/catalog/{name}.yaml")

    for package in packages:
        name = package["name"]
        record = catalog_records.get(name)
        if record is None:
            continue

        dependent_role = record.get("role")
        if dependent_role not in ALLOWED_DEPENDENCY_ROLES:
            errors.append(f"unknown role for {name}: {dependent_role}")
            continue

        for dep in package.get("dependencies", []):
            dep_name = dep.get("name")
            if dep_name not in by_name:
                continue
            dependency_record = catalog_records.get(dep_name)
            dependency_role = dependency_record.get("role") if dependency_record else None
            if dependency_role not in ALLOWED_DEPENDENCY_ROLES[dependent_role]:
                errors.append(
                    f"forbidden dependency edge: {name} ({dependent_role}) -> "
                    f"{dep_name} ({dependency_role})"
                )

    return errors


def package(name, role, deps=(), root="/__oya_fixture__", path=None):
    manifest_dir = pathlib.Path(path) if path else pathlib.Path(root) / "crates" / name
    return {
        "id": name,
        "name": name,
        "manifest_path": str(manifest_dir / "Cargo.toml"),
        "dependencies": [{"name": dep} for dep in deps],
        "role": role,
    }


def fixture(packages):
    return {
        "workspace_members": [item["id"] for item in packages],
        "packages": packages,
    }


def catalog_for(packages, extra=None):
    records = {item["name"]: {"role": item["role"]} for item in packages}
    if extra:
        records.update(extra)
    return records


def expect_self_test(label, metadata, catalog_records, expected_fragment, legacy_dirs_present=None):
    errors = validate_architecture(
        metadata,
        catalog_records,
        pathlib.Path("/__oya_fixture__"),
        legacy_dirs_present=legacy_dirs_present or set(),
    )
    if expected_fragment is None:
        if errors:
            raise AssertionError(f"{label}: expected success, got {errors}")
        return
    if not any(expected_fragment in error for error in errors):
        raise AssertionError(
            f"{label}: expected error containing {expected_fragment!r}, got {errors}"
        )


def run_self_test():
    kernel = package("oya-platform-tenant-kernel", "kernel")
    app = package("oya-foundation-app", "app", deps=["oya-platform-tenant-kernel"])
    metadata = fixture([kernel, app])
    records = catalog_for([kernel, app])

    expect_self_test("happy path", metadata, records, None)

    missing_catalog = dict(records)
    missing_catalog.pop("oya-foundation-app")
    expect_self_test("missing catalog", metadata, missing_catalog, "missing catalog record")

    forbidden_kernel = package(
        "oya-platform-tenant-kernel", "kernel", deps=["oya-foundation-app"]
    )
    forbidden_metadata = fixture([forbidden_kernel, app])
    forbidden_records = catalog_for([forbidden_kernel, app])
    expect_self_test(
        "forbidden role edge",
        forbidden_metadata,
        forbidden_records,
        "forbidden dependency edge",
    )

    bad_prefix = package("platform-tenant-kernel", "kernel")
    expect_self_test(
        "bad prefix",
        fixture([bad_prefix]),
        catalog_for([bad_prefix]),
        "oya- prefix",
    )

    wrong_path = package(
        "oya-foundry-api",
        "api",
        path=pathlib.Path("/__oya_fixture__") / "services" / "oya-foundry-api",
    )
    expect_self_test(
        "wrong workspace path",
        fixture([wrong_path]),
        catalog_for([wrong_path]),
        "must live at crates/oya-foundry-api",
    )

    expect_self_test(
        "legacy top-level dir",
        metadata,
        records,
        "legacy implementation directory",
        legacy_dirs_present={"services"},
    )

    extra_catalog = catalog_for(
        [kernel, app], extra={"oya-retired-placeholder-kernel": {"role": "kernel"}}
    )
    expect_self_test("extra catalog remains allowed", metadata, extra_catalog, None)

    print("architecture boundary self-test passed: 7 cases")


def main():
    args = sys.argv[1:]
    if args == ["--self-test"]:
        run_self_test()
        return
    if args:
        print("usage: scripts/check-architecture-boundaries.sh [--self-test]", file=sys.stderr)
        sys.exit(2)

    root = pathlib.Path.cwd()
    metadata = load_metadata()
    packages = workspace_packages(metadata)
    catalog_records = load_catalog_records(root, [package["name"] for package in packages])
    errors = validate_architecture(metadata, catalog_records, root)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        sys.exit(1)
    print(f"architecture boundary check passed for {len(packages)} workspace crates")


if __name__ == "__main__":
    main()
PY
