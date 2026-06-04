#!/usr/bin/env python3
"""Run a Buck2-owned Rust LLVM source-coverage fixture smoke.

This is narrow local fixture evidence. It proves that a Buck2 target can invoke
rustc source-based coverage instrumentation plus llvm-profdata/llvm-cov from
the active rustup sysroot. It does not prove production coverage budgets, live
cloud-ci authority, protected branch readiness, or Phase-0 exit.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

DEFAULT_SOURCE = Path("specs/fixtures/rust-llvm-coverage-smoke/branchy.rs")
FALSE_CLAIMS = {
    "production_coverage_report_generated": False,
    "coverage_budget_enforced": False,
    "status_mutation_performed": False,
    "protected_branch_authority_proven": False,
    "live_required_context_execution_proven": False,
    "p0_0_green": False,
    "phase0_complete": False,
    "production_ready": False,
    "hyperscaler_grade": False,
}
OFFICIAL_SOURCES = [
    {
        "name": "rustc Book — Instrumentation-based Code Coverage",
        "url": "https://doc.rust-lang.org/rustc/instrument-coverage.html",
        "used_for": "rustc -C instrument-coverage and LLVM_PROFILE_FILE .profraw emission.",
    },
    {
        "name": "LLVM — Source-based Code Coverage",
        "url": "https://clang.llvm.org/docs/SourceBasedCodeCoverage.html",
        "used_for": "llvm-profdata merge -sparse and llvm-cov report/export.",
    },
    {
        "name": "Buck2 — Commands",
        "url": "https://buck2.build/docs/users/commands/",
        "used_for": "Buck2 target-based build evidence and Build ID capture.",
    },
]


def run(cmd: list[str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, text=True, capture_output=True, check=False, **kwargs)


def rustc_host(rustc: str) -> tuple[str | None, str]:
    completed = run([rustc, "-vV"])
    host = None
    for line in completed.stdout.splitlines():
        if line.startswith("host: "):
            host = line.split(": ", 1)[1]
            break
    return host, completed.stdout.strip()


def base_result(source: Path) -> dict[str, Any]:
    return {
        "authority_boundary": "Buck2 local fixture LLVM source-coverage smoke only; no production coverage budget or live required-context authority proven",
        "fixture_coverage_smoke_generated": False,
        **FALSE_CLAIMS,
        "smoke_source": source.as_posix(),
        "official_sources": OFFICIAL_SOURCES,
        "verdict": "FAIL",
        "failures": [],
    }


def emit(result: dict[str, Any], out: Path | None) -> None:
    rendered = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if out is None:
        print(rendered, end="")
    else:
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(rendered)
        print(rendered, end="")


def fail(result: dict[str, Any], failure: str, out: Path | None) -> int:
    result["failures"].append(failure)
    result["failures"] = sorted(set(result["failures"]))
    result["verdict"] = "FAIL"
    emit(result, out)
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", default=str(DEFAULT_SOURCE), help="Rust fixture source path")
    parser.add_argument("--out", help="JSON output path; stdout is still emitted")
    args = parser.parse_args()

    source = Path(args.source)
    out = Path(args.out) if args.out else None
    result = base_result(source)

    if not source.is_file():
        return fail(result, "missing_source_file", out)

    rustc = os.environ.get("OYA_RUSTC") or shutil.which("rustc")
    if not rustc:
        return fail(result, "missing_rustc", out)
    # Keep rustup proxy symlinks unresolved. Resolving ~/.cargo/bin/rustc to the
    # rustup binary changes argv[0], so rustup no longer dispatches as rustc.
    rustc = str(Path(rustc))

    version = run([rustc, "--version"])
    if version.returncode != 0:
        result["rustc_error"] = version.stderr
        return fail(result, "rustc_version_failed", out)
    host, verbose = rustc_host(rustc)
    if not host:
        result["rustc_verbose"] = verbose
        return fail(result, "missing_rustc_host", out)
    sysroot = run([rustc, "--print", "sysroot"])
    if sysroot.returncode != 0:
        result["rustc_sysroot_error"] = sysroot.stderr
        return fail(result, "rustc_sysroot_failed", out)

    llvm_bin = Path(os.environ.get("OYA_LLVM_BIN") or Path(sysroot.stdout.strip()) / "lib" / "rustlib" / host / "bin")
    llvm_profdata = llvm_bin / "llvm-profdata"
    llvm_cov = llvm_bin / "llvm-cov"
    result.update(
        {
            "rustc_path": rustc,
            "rustc_version": version.stdout.strip(),
            "rustc_host": host,
            "rustc_sysroot": sysroot.stdout.strip(),
            "llvm_tools_source": "rustup sysroot rustlib host bin",
            "llvm_bin": llvm_bin.as_posix(),
            "ambient_path_llvm_tools_required": False,
            "rustc_flag": "rustc -C instrument-coverage",
            "profile_env_var": "LLVM_PROFILE_FILE",
            "profile_collision_guard": "%m-%p",
        }
    )
    if not llvm_profdata.is_file() or not os.access(llvm_profdata, os.X_OK):
        return fail(result, "missing_llvm_profdata", out)
    if not llvm_cov.is_file() or not os.access(llvm_cov, os.X_OK):
        return fail(result, "missing_llvm_cov", out)

    with tempfile.TemporaryDirectory(prefix="oya-rust-llvm-coverage-smoke-") as tmp:
        tmp_path = Path(tmp)
        profraw_dir = tmp_path / "profraw"
        profraw_dir.mkdir()
        binary = tmp_path / "branchy"
        profdata = tmp_path / "default.profdata"
        coverage_json_path = tmp_path / "coverage.json"
        report_path = tmp_path / "coverage.txt"
        profile_template = profraw_dir / "%m-%p.profraw"

        compile_result = run([rustc, "-C", "instrument-coverage", source.as_posix(), "-o", binary.as_posix()])
        if compile_result.returncode != 0:
            result["compile_stdout"] = compile_result.stdout
            result["compile_stderr"] = compile_result.stderr
            return fail(result, "rustc_instrumented_compile_failed", out)

        env = os.environ.copy()
        env["LLVM_PROFILE_FILE"] = profile_template.as_posix()
        run_outputs: list[dict[str, Any]] = []
        for argument in ("2", "3"):
            run_result = run([binary.as_posix(), argument], env=env)
            run_outputs.append(
                {
                    "argument": argument,
                    "returncode": run_result.returncode,
                    "stdout": run_result.stdout.strip(),
                    "stderr": run_result.stderr.strip(),
                }
            )
            if run_result.returncode != 0:
                result["run_outputs"] = run_outputs
                return fail(result, "instrumented_fixture_run_failed", out)

        profraws = sorted(profraw_dir.glob("*.profraw"))
        if not profraws:
            result["run_outputs"] = run_outputs
            return fail(result, "missing_profraw_output", out)

        merge_result = run([llvm_profdata.as_posix(), "merge", "-sparse", *[p.as_posix() for p in profraws], "-o", profdata.as_posix()])
        if merge_result.returncode != 0:
            result["llvm_profdata_stdout"] = merge_result.stdout
            result["llvm_profdata_stderr"] = merge_result.stderr
            return fail(result, "llvm_profdata_merge_failed", out)

        export_result = run([llvm_cov.as_posix(), "export", binary.as_posix(), "--instr-profile", profdata.as_posix(), "--format=text"])
        if export_result.returncode != 0:
            result["llvm_cov_export_stdout"] = export_result.stdout
            result["llvm_cov_export_stderr"] = export_result.stderr
            return fail(result, "llvm_cov_export_failed", out)
        coverage_json_path.write_text(export_result.stdout)

        report_result = run([llvm_cov.as_posix(), "report", binary.as_posix(), "--instr-profile", profdata.as_posix()])
        if report_result.returncode != 0:
            result["llvm_cov_report_stdout"] = report_result.stdout
            result["llvm_cov_report_stderr"] = report_result.stderr
            return fail(result, "llvm_cov_report_failed", out)
        report_path.write_text(report_result.stdout)

        coverage = json.loads(coverage_json_path.read_text())
        totals = coverage["data"][0]["totals"]
        line_percent = totals["lines"]["percent"]
        region_percent = totals["regions"]["percent"]
        if line_percent < 100 or region_percent < 100:
            result["coverage_totals"] = totals
            return fail(result, "fixture_coverage_below_100_percent", out)

        result.update(
            {
                "verdict": "PASS",
                "failures": [],
                "fixture_coverage_smoke_generated": True,
                "coverage_report_format": ["json", "text"],
                "coverage_totals": totals,
                "fixture_line_coverage_percent": line_percent,
                "fixture_region_coverage_percent": region_percent,
                "profraw_count": len(profraws),
                "profile_template": profile_template.as_posix(),
                "profdata_operation": "llvm-profdata merge -sparse",
                "llvm_cov_operations": ["export --format=text", "report"],
                "run_outputs": run_outputs,
                "text_report": report_path.read_text(),
            }
        )

    emit(result, out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
