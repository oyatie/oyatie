---
doc_class: JudgmentNote
title: Clear #1646 Cargo.lock / controller members bridge
status: Accepted
date: 2026-08-11
ssot_todo: cargo-lock-sole-owner
---

Root `Cargo.lock` restored to `origin/dev` and `ci/controller/**` workspace
members removed. Forever owner is `integ/build` (#1662@`ce9735104` absorb).
Controller **sources** remain on this tip (`ci/**` envelope).
