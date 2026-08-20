#!/usr/bin/env bash
# Regenerate every committed snapshot fixture from its Go corpus.
#
# The fixtures are GENERATED, and the corpus is the source. Editing a fixture by hand survives
# exactly until the next run of this script, and a hand edit has been lost that way before — put the
# change in the corpus Go instead.
#
# Run this whenever the extractor changes shape: a fixture produced by an older extractor still
# carries the node kinds that extractor emitted, so a construct that has since become a real kind
# stays recorded as `unsupported` and the transform refuses it. That failure looks like a transform
# regression and is not one.
#
# The corpus directory names and module ids are NOT interchangeable. `corpus-upstream-before` and
# `corpus-upstream-after` deliberately share ONE module id, because the drift pair is two ports of
# the SAME package: give them distinct ids and the classification comes back `Unchanged` instead of
# `Explained`, which is a green result for a broken test.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$root/adapters/frontend-go/gosrc"
out=../../snapshot/src

gen() { # <corpus dir> <module id> <module root> <fixture file>
  go run ./extractor -corpus "$1" -module "$2" -root "$3" -out "$out/$4"
  echo "regen $4"
}

gen ./corpus                 oyatie.example/portengine-fixture           . fixture-snapshot-v1.json
gen ./corpus-refused         oyatie.example/portengine-fixture-refused   . fixture-snapshot-refused-v1.json
gen ./corpus-failure         oyatie.example/portengine-fixture-failure   . fixture-snapshot-failure-v1.json
gen ./corpus-interface       oyatie.example/portengine-fixture-interface . fixture-snapshot-interface-v1.json
gen ./corpus-ownership       oyatie.example/portengine-fixture-ownership . fixture-snapshot-ownership-v1.json
gen ./corpus-unproven        oyatie.example/portengine-fixture-unproven  . fixture-snapshot-unproven-v1.json
gen ./corpus-sentinel        oyatie.example/portengine-fixture-sentinel  . fixture-snapshot-sentinel-v1.json
gen ./corpus-buildtags       oyatie.example/portengine-fixture-buildtags . fixture-snapshot-buildtags-v1.json
gen ./corpus-foreign         oyatie.example/portengine-fixture-foreign   . fixture-snapshot-foreign-v1.json
gen ./corpus-upstream-before oyatie.example/portengine-fixture ./corpus-upstream-before fixture-snapshot-drift-before-v1.json
gen ./corpus-upstream-after  oyatie.example/portengine-fixture ./corpus-upstream-after  fixture-snapshot-drift-after-v1.json
