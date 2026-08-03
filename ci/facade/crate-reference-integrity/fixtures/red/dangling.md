# RED fixture — do not fix these references

This file exists to be WRONG. It is the proof that cloud-ci-crate-reference-integrity can
fail; a gate only ever observed passing has not been shown capable of failing. The live
scan never reads it (the policy excludes `ci/facade/crate-reference-integrity/fixtures/**`),
so it is proof without being debt.

    cargo check -p oya-definitely-not-a-live-crate-red-fixture

Target: //cloud/cloud-ci/gates/oya-red-fixture-app:oya-red-fixture-app-gate
