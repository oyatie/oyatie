---
rule_id: NEUTRALITY-CANARY-000
rule_kind: neutral
operations_journal_ref: W0-B-20260809-neutral-split-gate
---
# NEUTRALITY-CANARY-000 — the neutral pack's own liveness canary

## What this file is

A permanent, deliberately unremarkable file under the neutral rule root. It declares no
translation and matches no source shape. Its only job is to exist so that the split check has a
subject in the rule pack even when the pack is otherwise empty, and so the check's red proof can be
demonstrated against a file that is not a real rule.

Its identifier is deliberately OUTSIDE the `GO-RUST-<FAMILY>-<NNN>` grammar the language rules are
allocated from. It is not a language rule, it must never be mistaken for one, and it must never
collide with an identifier a real rule wants.

## Why a canary rather than an assertion

The split between language rules and corpus rules is not the kind of thing convention holds. A rule
justified by "the source repository we sized this from does it this way" compiles, reads well,
reviews clean, and stays invisible until a second repository arrives. So the split is a check that
fires, and this file is the fixture that lets the check be proven to fire.

Note the shape of the guarantee, because it is narrower than it looks. A green check is NOT proof of
neutrality: it rejects a fixed set of five vocabulary needles, and no finite list of needles can
decide "is this specific to one source repository". A branch on some domain noun no needle
anticipates passes the check, and review is what catches that. The check is a canary set, not a
decision procedure. The neutral engine's own module documentation makes the same admission about the
same five needles, and the two lists are kept identical on purpose.

## What may be written here

Nothing that names the source repository this program ports, in any of its vocabulary — not in the
body, not in the front matter, and not in the filename, because the filename is also the rule
identifier and a body review never looks at it.

## Ordering

None. This file constrains no other rule and is constrained by none.

## Residue

This file proves the scan has a subject. It does not prove the scan REACHED the rule root on the
live tree — a check handed its own inputs cannot answer that. That assertion lives with the gate,
against the real tree, and is the one a narrowed scan cannot survive.
