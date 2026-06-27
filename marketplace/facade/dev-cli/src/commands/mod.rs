pub(crate) mod catalog;
pub(crate) mod check;
pub(crate) mod cleanup;
pub(crate) mod demo;
pub(crate) mod doc;
pub(crate) mod gate;
// ADR-0364 D3: `oya gen masterplan` — generative projections from the ADR log.
// Module is named `generate` because `gen` is a reserved keyword in edition
// 2024; the CLI surface verb stays `gen` (a runtime dispatch string).
pub(crate) mod generate;
pub(crate) mod lint;
pub(crate) mod onprem;
pub(crate) mod ops;
pub(crate) mod plan;
pub(crate) mod submit;
pub(crate) mod supply_chain;
pub(crate) mod verify;
pub(crate) mod verify_affected;
// O6 (ADR-0360): speculative merge-queue algorithm (ADR-0111), wired as
// `oya merge-queue simulate`.
pub(crate) mod merge_queue;
// O2 (ADR-0360): nextest JUnit results-ingest, wired as `oya verify --from-results`.
pub(crate) mod verify_results;
