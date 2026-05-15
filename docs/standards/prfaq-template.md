---
purpose: Auto-backfilled purpose for prfaq-template.md
---

---
doc_class: Template
purpose: Working-backwards PR/FAQ template (Amazon-style) for launching new capabilities; force-clarifies customer value before any engineering investment.
---

# PR/FAQ Template

## How to use

Write the press release **before** writing the design doc. If you cannot
write a compelling press release in plain language for a non-engineer
customer, the feature isn't ready to design — let alone ship.

Sections marked `(required)` cannot be empty when the PR/FAQ enters review.

---

## 1. Press Release `(required)`

Imagine the day this ships. Write the announcement now.

- **Heading**: One-sentence headline a customer would care about.
- **Subheading**: Who this is for, and what it lets them do they couldn't before.
- **Location & date**: Where the launch happens; ship date (target).
- **Opening paragraph**: Restate the headline in plain language. Why now.
- **Problem paragraph**: The specific pain a real customer feels today.
- **Solution paragraph**: What we're shipping. Concrete, not hand-wavy.
- **Quote from a leader at our company**: Why we built this.
- **How it works**: 1–3 sentences a non-engineer can follow.
- **Quote from a customer**: A real or representative voice of the user.
- **Call to action**: How a reader becomes a user.

## 2. FAQ `(required)`

Anticipate the questions a skeptical reviewer or hostile customer will
ask. Answer each in 2–4 sentences. Minimum: cover every item below.

### Customer questions
- What does this do for me, today?
- What does it replace? What does it cost?
- How do I get started in <5 minutes?
- What are the limits? When should I **not** use this?

### Internal questions
- Why now? What changed that makes this possible/necessary?
- What is the smallest shippable version, and what does it deliberately not do?
- What are we betting on (assumptions that could falsify the plan)?
- Who is the single accountable owner? Who else must say yes?
- What's the rollback story if the launch goes badly?
- What metric will tell us this is succeeding or failing 30 days post-launch?

## 3. Tenets `(required)`

3–7 short principles that will resolve future disagreements about scope.
Each tenet is a sentence: "We will prefer X over Y because Z."

## 4. Out of scope `(required)`

What this PR/FAQ explicitly **does not** commit to. Listing this here
prevents scope creep masquerading as "of course we'd also do that."

## 5. Open questions

Unresolved decisions, in priority order. Note who is expected to resolve
each and by when.

---

## Review checklist (reviewer adds initials)

- [ ] Press release readable by a non-engineer.
- [ ] FAQ covers cost, limits, rollback, success metric.
- [ ] Tenets resolve foreseeable scope arguments.
- [ ] Out-of-scope section non-empty.
- [ ] Open questions name an owner and a deadline.
