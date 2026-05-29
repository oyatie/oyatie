# Tutorial — Create, collaborate, branch, merge, and sign a document

Goal: end-to-end document lifecycle in one session. Create a doc, edit collaboratively with two users, branch for review, merge,
and apply a qualified e-signature. All on a loopback cell.

Pre-reqs:
- Loopback cell: `make dev-cell.up CELL=docs-loopback-1 PROFILE=docs-dev`
- Two dev tenants:
  ```bash
  make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid
  make dev-user.create T=oyatie.b2b.smb.acme-software USER=alice EMAIL=alice@acme-software.io
  make dev-user.create T=oyatie.b2b.smb.acme-software USER=bob EMAIL=bob@acme-software.io
  ```

## Step 1 — Alice creates a doc

```bash
ALICE_KEY=$(./bin/oya creds dev-token --tenant oyatie.b2b.smb.acme-software --user alice)
DOC_ID=$(./bin/oya docs create \
  --auth $ALICE_KEY \
  --title "Q3 2026 Engineering Plan" \
  --template blank \
  --json | jq -r .id)
echo "doc_id=$DOC_ID"
```

Expected: `doc_id=doc-…`

## Step 2 — Alice writes the first draft

```bash
./bin/oya docs blocks append \
  --auth $ALICE_KEY \
  --doc $DOC_ID \
  --block-kind heading-1 \
  --content "Q3 2026 Engineering Plan"

./bin/oya docs blocks append \
  --auth $ALICE_KEY \
  --doc $DOC_ID \
  --block-kind paragraph \
  --content "This document outlines the engineering team's commitments for Q3."

./bin/oya docs blocks append \
  --auth $ALICE_KEY \
  --doc $DOC_ID \
  --block-kind heading-2 \
  --content "Goals"

./bin/oya docs blocks append \
  --auth $ALICE_KEY \
  --doc $DOC_ID \
  --block-kind bullet-list \
  --content "Ship workflow-studio GA||Land 3 customer pilots||Reduce P95 latency by 30%"
```

## Step 3 — Alice grants Bob editor access

```bash
./bin/oya docs share \
  --auth $ALICE_KEY \
  --doc $DOC_ID \
  --principal "oyatie.b2b.smb.acme-software::User::bob" \
  --role editor
```

## Step 4 — Bob edits collaboratively

In another shell:
```bash
BOB_KEY=$(./bin/oya creds dev-token --tenant oyatie.b2b.smb.acme-software --user bob)

./bin/oya docs blocks insert-after \
  --auth $BOB_KEY \
  --doc $DOC_ID \
  --after-block-kind heading-2 \
  --after-block-content "Goals" \
  --block-kind bullet-item \
  --content "Migrate 100 % of customers to HTTP/3"
```

Both users see the merged document. Inspect:
```bash
./bin/oya docs render --doc $DOC_ID --format markdown
```

## Step 5 — Bob branches for legal review

```bash
BRANCH_ID=$(./bin/oya docs branch \
  --auth $BOB_KEY \
  --doc $DOC_ID \
  --branch-name "legal-review-2026-Q3" \
  --json | jq -r .branch_id)
```

Bob shares the branch with legal:
```bash
./bin/oya docs share \
  --auth $BOB_KEY \
  --doc $BRANCH_ID \
  --principal "oyatie.b2b.smb.acme-software::Group::legal" \
  --role commenter
```

## Step 6 — Legal adds comments

```bash
LEGAL_KEY=$(./bin/oya creds dev-token --tenant oyatie.b2b.smb.acme-software --user legal-eve)

./bin/oya docs comment \
  --auth $LEGAL_KEY \
  --doc $BRANCH_ID \
  --target-block-text "100 % of customers" \
  --content "Add 'subject to customer consent' to comply with the customer-master-agreement clause 3.2."
```

## Step 7 — Bob addresses the comment + merges branch

```bash
./bin/oya docs blocks update \
  --auth $BOB_KEY \
  --doc $BRANCH_ID \
  --block-text "100 % of customers" \
  --new-content "100 % of customers (subject to customer consent)"

./bin/oya docs comment resolve \
  --auth $BOB_KEY \
  --doc $BRANCH_ID \
  --comment-id <comment_id>

./bin/oya docs branch merge \
  --auth $BOB_KEY \
  --doc $DOC_ID \
  --branch $BRANCH_ID
```

## Step 8 — Alice publishes + signs

```bash
./bin/oya docs publish-snapshot \
  --auth $ALICE_KEY \
  --doc $DOC_ID \
  --label "Q3-2026-final"

./bin/oya docs digital-sign \
  --auth $ALICE_KEY \
  --doc $DOC_ID \
  --signature-level eidas-advanced
```

Alice (in dev cell auto-signs) provides the signature; the doc is now immutable.

## Step 9 — Verify the audit chain

```bash
./bin/oya audit query \
  --tenant oyatie.b2b.smb.acme-software \
  --resource "docs/$DOC_ID" \
  --window 1h
```

You should see events: doc create, 4 block-append, share-bob, edit-bob, branch, share-legal, comment, comment-resolve, edit-resolution,
branch-merge, snapshot, digital-sign. Each event chains via BLAKE3-256.

## Step 10 — Export the signed PDF

```bash
./bin/oya docs export \
  --auth $ALICE_KEY \
  --doc $DOC_ID \
  --format pdf-signed \
  --output q3-plan-signed.pdf
```

Verify with any PDF reader:
```bash
gpg --verify q3-plan-signed.pdf   # (or use Adobe Reader's signature validation)
```

## What you proved

- Multi-user collaboration converges via CRDT.
- Branching is a first-class workflow for review.
- Comments + resolution are part of the audit chain.
- E-signatures use the right level for the document type.
- Every event is chain-anchored and verifiable.
