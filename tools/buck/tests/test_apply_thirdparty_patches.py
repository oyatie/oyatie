"""Unit tests for the semantic Reindeer post-generation overlay."""

import importlib.util
from pathlib import Path
import unittest


MODULE_PATH = Path(__file__).parents[1] / "apply-thirdparty-patches.py"
SPEC = importlib.util.spec_from_file_location("thirdparty_overlay", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
OVERLAY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(OVERLAY)


def buildscript_rule(name: str) -> str:
    return f'''buildscript_run(
    name = "{name}",
    env = {{
        "CARGO_PKG_VERSION_PRE": "",
    }},
)
'''


def psm_rule() -> str:
    return '''cxx_library(
    name = "psm-0.1-psm_asm",
    preprocessor_flags = [
        "-DCFG_TARGET_OS_darwin",
        "-DCFG_TARGET_ARCH_aarch64",
        "-DCFG_TARGET_ENV_",
    ],
)
'''


class OverlayTests(unittest.TestCase):
    def test_applies_once_and_is_idempotent(self) -> None:
        source = buildscript_rule("aws-lc-rs-1-build-script-run") + psm_rule()

        patched, changes = OVERLAY.apply(source)

        self.assertEqual(changes, 2)
        self.assertIn("DEP_AWS_LC_0_41_0_INCLUDE", patched)
        self.assertIn('"prelude//os:linux": [', patched)
        self.assertEqual(OVERLAY.apply(patched), (patched, 0))

    def test_refuses_a_missing_generated_anchor(self) -> None:
        with self.assertRaisesRegex(ValueError, "aws-lc-rs-1-build-script-run"):
            OVERLAY.apply(psm_rule())

    def test_refuses_a_corrupt_aws_dep_overlay(self) -> None:
        source = buildscript_rule("aws-lc-rs-1-build-script-run").replace(
            '        "CARGO_PKG_VERSION_PRE": "",',
            '''        "CARGO_PKG_VERSION_PRE": "",
        "DEP_AWS_LC_0_41_0_INCLUDE": "wrong",''',
        ) + psm_rule()

        with self.assertRaisesRegex(ValueError, "incomplete or corrupt DEP overlay"):
            OVERLAY.apply(source)

    def test_refuses_a_partial_aws_dep_overlay_without_include(self) -> None:
        source = buildscript_rule("aws-lc-rs-1-build-script-run").replace(
            '        "CARGO_PKG_VERSION_PRE": "",',
            '''        "CARGO_PKG_VERSION_PRE": "",
        "DEP_AWS_LC_0_41_0_LIBCRYPTO": "aws_lc_0_41_0_crypto",''',
        ) + psm_rule()

        with self.assertRaisesRegex(ValueError, "incomplete or corrupt DEP overlay"):
            OVERLAY.apply(source)

    def test_refuses_unexpected_psm_shape(self) -> None:
        source = buildscript_rule("aws-lc-rs-1-build-script-run") + '''cxx_library(
    name = "psm-0.1-psm_asm",
    preprocessor_flags = ["unexpected"],
)
'''

        with self.assertRaisesRegex(ValueError, "unexpected generated preprocessor flags"):
            OVERLAY.apply(source)

    def test_refuses_a_partial_psm_platform_overlay(self) -> None:
        source = buildscript_rule("aws-lc-rs-1-build-script-run") + '''cxx_library(
    name = "psm-0.1-psm_asm",
    preprocessor_flags = select({
        "prelude//os:linux": [],
        "DEFAULT": [],
    }),
)
'''

        with self.assertRaisesRegex(ValueError, "incomplete or corrupt platform overlay"):
            OVERLAY.apply(source)


if __name__ == "__main__":
    unittest.main()
