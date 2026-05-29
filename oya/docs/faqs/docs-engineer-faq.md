# `docs` µservice — Docs Engineer FAQ

20 real questions raised against the µservice that owns Oyatie's collaborative-document surface.

---

**Q1. What's the difference between `docs`, `notes`, `sites`, and `sheets`?**

- `docs`: rich-text collaborative documents (Google Docs-class).
- `notes`: personal note-taking (Notion-personal / Obsidian / Roam-class).
- `sites`: publishing (Webflow / Wordpress / Notion-as-CMS-class).
- `sheets`: spreadsheets (Google Sheets / Excel-class).

They share `ontology::Person` and embed each other (`docs` can embed `sheets`; `sites` can publish a `docs`), but each owns its
domain primitives.

---

**Q2. Which CRDT do we use?**

A Y.js-compatible CRDT family. The choice was driven by: maturity (Y.js has been in production for 7+ years at Notion, Confluence,
Linear, etc), wire interop (we can interop with existing Y.js clients), and convergence-property tested. We adapted the rust port
`yrs` (yjs-rs) as the kernel.

---

**Q3. What's the transport?**

CRDT ops flow over QUIC streams (HTTP/3) primarily; WebSocket-over-HTTP/2 fallback for environments without QUIC. Per ADR-0145 +
ADR-0253. The transport multiplexes presence (cursors, selections) + ops + comments + auth challenges over a single connection.

---

**Q4. How is permission enforced at block level?**

Every CRDT op carries a `principal` + `action`. The kernel runs Cedar before applying the op; rejected ops are returned to the
client with `OpRejected { reason }`. The client surfaces the rejection as a UI hint (e.g. "you don't have permission to edit this
table").

---

**Q5. What about embed permissions?**

Embeds (Figma, YouTube, Spotify, Loom) are inert from the µservice's perspective — we render an iframe with a URL. The permission
check is at view-time on the embed provider's side. Rich embeds with data (e.g. embedded `sheets`) cross-check permission via the
embedded µservice's Cedar.

---

**Q6. How does branching work?**

`docs::Action::Branch` creates a copy-on-write branch of a document. Edits in the branch don't affect the trunk; merging is a
manual operation (similar to git). Used for review workflows (legal review, regulator review).

---

**Q7. Can multiple branches edit concurrently?**

Yes, but conflicts on merge are CRDT-merged. The merge is non-destructive; both versions are preserved + diffed visually.

---

**Q8. How is AI co-author gated?**

Per-tenant policy (Cedar). demo_trial/paid default-on per-document opt-in; paid tenants can policy-enforce default-off; compliance_pack
tenants restrict to tenant-only models. AI prompts are sent to `intelligence` µservice with `document_id` + `block_id` context;
responses streamed back as block-level suggestions the user accepts or rejects.

---

**Q9. What's the offline story?**

The CRDT runs offline (Y.js + IndexedDB on the web client; SQLite cache on native). Ops queue offline + sync on reconnect; the
CRDT guarantees convergence even with weeks-long offline.

---

**Q10. How do you handle 10,000 simultaneous editors per doc (compliance_pack)?**

Sharded CRDT — large documents are partitioned by block range; each shard has its own awareness channel; cross-shard ops use a
super-CRDT layer. Awareness (cursors) is aggregated to ≤ 100 "presence beacons" visible at once (configurable).

---

**Q11. What's the audit-chain granularity?**

Per-block-mutation. Inserting a paragraph writes one event; editing a paragraph writes one event per coalesced edit (every ≤ 5 s
or ≤ 100 chars). Coarse enough to keep ledger size reasonable; fine enough for compliance trace.

---

**Q12. How are signatures handled?**

`docs::Action::DigitalSignature` invokes the `workplace-integration` e-sign engine to sign the document at the chosen signature
level (eIDAS simple/advanced/qualified, ESIGN, FDA 21 CFR Part 11). The signed PDF is chain-anchored.

---

**Q13. Can a document be retracted after signature?**

Signed documents are immutable. A retraction creates a new "retracted" version with an audit link; the original signature is
preserved per regulatory requirements.

---

**Q14. How does export work?**

Exporters are per-target-format:
- HTML, Markdown, PDF, DOCX (demo_trial)
- EPUB, ODT, LaTeX, JSON-Schema-defined custom (paid)
- DITA-XML, S1000D structured content, custom transform pipelines (paid)
- eCTD, regulator-cleared structured content (compliance_pack)

Exporters are signed + tested against reference fixtures.

---

**Q15. What's the latency budget for keystroke → server ack?**

demo_trial p95: 60 ms. paid: 35 ms. paid: 18 ms in-region. compliance_pack: 12 ms. Above budget, the CRDT engine raises `latency_breach`
and the client surfaces a "connection slow" hint.

---

**Q16. Can a block extend across documents?**

No. Blocks are document-scoped. Cross-document references use **embeds** (which fetch + render the other doc's content) or
**transclusions** (which display another doc's block read-only). Transclusion is paid; demo_trial + paid use embeds.

---

**Q17. How are images + attachments stored?**

Per-document blob store in `drive` µservice; the doc holds a reference. Inline images get a thumbnail in the doc, full content in
`drive`. Permissions cascade: if you can read the doc, you can read its inline blobs.

---

**Q18. How does collaboration scale globally?**

Cells are region-sharded; CRDT ops are routed to the home cell, then mirrored to other cells where the doc is active. Mirror lag
is sub-cell ≤ 50 ms (paid) / ≤ 15 ms (compliance_pack). Reads always come from the local cell; writes route to home.

---

**Q19. What's the cold-load time for a 100-page doc?**

demo_trial p95: 1.4 s. paid: 0.9 s. paid: 0.5 s. compliance_pack: 0.3 s. Cold-load is dominated by CRDT state hydration; we use a snapshot
+ delta-compress + lazy block hydration to keep this fast.

---

**Q20. How does the µservice integrate with `workflow-engine`?**

Workflow embeds: a block in a doc can embed a workflow. Triggering the workflow from the doc runs it under the doc's principal +
the user's principal. Workflow output can stream into the doc as a new block (e.g. "summarise PR #1234" → an AI block populated
by the workflow result).
