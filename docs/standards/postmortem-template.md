---
purpose: Blameless postmortem template (Google SRE-style) for any incident or near-miss; forces causal analysis and actionable follow-up over individual blame.
---

---
doc_class: Template
purpose: Blameless postmortem template (Google SRE-style) for any incident or near-miss; forces causal analysis and actionable follow-up over individual blame.
---

# Postmortem Template

## Ground rule

This is a **blameless** postmortem. The goal is to make the system
safer, not to identify whose fingerprints are on the trigger. Wording
to use: "the change passed review and reached production"; wording to
avoid: "X merged a broken change."

If anyone in the room hears blame, they say so out loud and the room
rewords.

---

## 1. Metadata `(required)`

- **Incident ID**:
- **Severity**: `SEV-1` | `SEV-2` | `SEV-3` | `near-miss`
- **Service / component affected**:
- **Author**:
- **Reviewers**:
- **Status**: `draft` | `in-review` | `closed`
- **Last updated**:

## 2. Summary `(required)`

≤ 4 sentences answering: what happened, who/what was impacted, how long,
how was it stopped. A reader skimming the wiki should learn the gist
without scrolling.

## 3. Impact `(required)`

- **Time range**: First detection → final mitigation. UTC.
- **Customer impact**: Number of users / requests / dollars / SLO budget
  consumed. Use concrete numbers, not "many."
- **Internal impact**: Engineer-hours, escalations, missed deadlines.

## 4. Timeline `(required)`

Bulleted, timestamped (UTC), in causal order. Mark the first detection,
each major decision point, and the final mitigation.

```
HH:MM  what happened (who/system observed it)
HH:MM  …
```

Resist the urge to summarize — verbatim Slack/PagerDuty snippets are
fine and often the most useful primary source.

## 5. Root cause analysis `(required)`

Use the **5 Whys** (or a richer causal-chain technique) to trace from
the user-visible symptom back through every contributing factor. A
single "root cause" is almost always wrong; expect 2–4 contributing
causes spanning code, process, and environment.

For each contributing cause, classify as:
- **Trigger**: the specific change/event that caused this *now*.
- **Latent flaw**: a property of the system that allowed the trigger
  to cause harm.
- **Detection gap**: why we noticed late (or didn't).
- **Recovery gap**: why mitigation took as long as it did.

## 6. What went well `(required)`

Concrete things that helped. Naming them anchors them as part of "how
we respond" so they don't decay.

## 7. What went poorly `(required)`

Honest list. Tie each item to a recovery / detection / latent-flaw
classification from §5.

## 8. Action items `(required)`

Each action item has:
- **Owner** (single name, not a team).
- **Due date** (concrete, ≤ 6 weeks where possible).
- **Acceptance** (how we'll know it's done).
- **Category**: `prevent` (latent flaw fix) | `detect` (faster signal)
  | `respond` (faster recovery) | `documentation`.

Action items live in the project tracker; this section is the canonical
list. If an item ages out without resolution, it gets re-discussed at
the next review.

## 9. Open questions

Things we still don't understand. Each should have an owner and a
deadline; if it stays "open" too long it becomes its own incident
candidate.

## 10. Lessons + system-level patterns

Two paragraphs. What did this teach us about the *shape* of our system,
not just this one bug. If the same lesson has appeared in two prior
postmortems, that's a meta-incident — flag it for SRE review.

---

## Review checklist (reviewer adds initials)

- [ ] Impact uses concrete numbers, not adjectives.
- [ ] Timeline is in UTC with verbatim sources where useful.
- [ ] Root-cause section has ≥ 2 contributing causes.
- [ ] Each action item has owner, due date, and acceptance.
- [ ] No blame-language; reworded if any reviewer flagged it.
