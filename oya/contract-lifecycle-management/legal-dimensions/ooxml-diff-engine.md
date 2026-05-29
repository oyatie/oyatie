---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: Migrated-from-tier-matrix
related_packs: []
date: 2026-05-21
---

# OOXML Diff Engine

Many corporate contracts are authored in Microsoft Word (.docx, OOXML per ISO/IEC 29500). CLM ingests OOXML documents and produces a server-side diff visualization. This is the OOXML diff engine — Rust-native (no JVM dependency, replacing docx4j 11.5 from the retired retired-standard-tier capability).

## Architecture

The OOXML diff kernel lives in `crates/oya-clm-ooxml-diff-kernel/`. It:

1. Opens the OOXML ZIP container.
2. Parses the `word/document.xml` (or per-section XML) into a typed AST.
3. Tokenizes paragraphs, runs, and embedded objects.
4. Computes a Myers-diff or Patience-diff over the token sequence.
5. Emits a unified diff annotated with formatting changes (track-changes style).

## Implementation notes

- **Pure Rust** dependencies: `quick-xml` (XML parsing), `zip` (container), `serde` (typed AST), `similar` (diff algorithm), `unicode-segmentation` (grapheme cluster splitting for accurate text positions).
- **No JVM**, no docx4j, no Apache POI.
- **Streaming**: large contracts (>50 MB OOXML) parsed in streaming mode to avoid full materialization.
- **Determinism**: same input always yields same diff (no clock-dependent state).

## Token representation

```
enum OoxmlToken {
  Paragraph { properties: ParagraphProperties, runs: Vec<Run> },
  Run { properties: RunProperties, text: String },
  Table { properties: TableProperties, rows: Vec<TableRow> },
  Hyperlink { url: String, runs: Vec<Run> },
  EmbeddedImage { relationship_id: String, alt_text: String },
  TrackChange { author: String, timestamp: Timestamp, change_type: ChangeType, content: Vec<OoxmlToken> },
  Comment { author: String, timestamp: Timestamp, content: Vec<OoxmlToken> },
  // ... more variants
}
```

## Diff output

```
struct OoxmlDiff {
  source_hash: BLAKE3Hash,
  target_hash: BLAKE3Hash,
  hunks: Vec<DiffHunk>,
  formatting_changes: Vec<FormattingChange>,
  semantic_changes: Vec<SemanticChange>,    // detected substantive changes (e.g. amount changed)
}

struct DiffHunk {
  before_span: TextSpan,
  after_span: TextSpan,
  kind: HunkKind,                            // insert | delete | replace
  text_before: String,
  text_after: String,
  formatting_delta: FormattingDelta?,
}

struct SemanticChange {
  category: SemanticCategory,                // pricing | term | jurisdiction | party_name | etc.
  source_value: String,
  target_value: String,
  confidence: f64,
}
```

## Semantic-change detection

The diff engine detects substantive changes:

- **Pricing change**: regex + currency-parser detects amount changes.
- **Term change**: date-parser detects effective/expiration date changes.
- **Jurisdiction change**: "governing law" clause changes.
- **Party-name change**: counterparty legal name changes.
- **Clause-family change**: addition or removal of a clause family.

Semantic changes are surfaced separately from formatting noise.

## Track-changes preservation

If the source OOXML contains track-changes (Word's reviewer mode), the diff engine preserves the track-change metadata. Each track-change becomes a `TrackChange` token; the diff distinguishes:

- Author-pending changes (not yet accepted/rejected by the receiver).
- Author-accepted changes (committed to the document).
- Author-rejected changes (excluded from the document).

## Performance

Per IP-029 redline-turnaround SLO:

- p95 ≤ 2s for documents ≤ 200 KB.
- p95 ≤ 10s for documents 200 KB - 5 MB.
- p95 ≤ 60s for documents > 5 MB.

Throughput on a 4-core node: ≈ 200 documents/sec at average 50 KB size.

## Cedar gate

```cedar
permit (
  principal,
  action == Action::"OoxmlDiff",
  resource is Contract
) when {
  resource.tenant_id == principal.tenant_id &&
  resource.state in ["Draft", "Review", "InternalReview", "CounterpartyEdited"]
};
```

## Audit events

- `oya.contract.lifecycle.management.ooxml.diff_computed`
- `oya.contract.lifecycle.management.ooxml.semantic_change_detected`
- `oya.contract.lifecycle.management.ooxml.track_change_preserved`

## Standards references

- ISO/IEC 29500 (OOXML).
- Open Packaging Conventions (OPC).
- Myers, "An O(ND) Difference Algorithm and Its Variations" (1986).
- Patience Diff (Bram Cohen).
