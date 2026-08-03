#!/usr/bin/env python3
"""Drafting-quality ratchet for the ~/.agents skill corpus.

The official validator (skills-ref) checks frontmatter validity and naming. It
says nothing about whether a skill will actually be *selected* or *executed*.
This checks that, and freezes today's failure count per assertion so the number
can only go down.

Governing rule: every assertion here FAILS on a concrete violating input, and
`--selftest` proves it. A rule that cannot fail does not ship.

Usage:
    skill_conformance.py                  # check against baseline, exit 1 on regression
    skill_conformance.py --freeze         # rewrite baseline.json from current corpus
    skill_conformance.py --selftest       # prove each assertion bites
    skill_conformance.py ROOT [ROOT ...]  # override corpus roots
"""

import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BASELINE = os.path.join(HERE, "baseline.json")
DEFAULT_ROOTS = [
    os.path.expanduser("~/.agents/skills"),
    os.path.expanduser("~/.agents/adapters/codex/skills"),
]

# ponytail: hand-rolled frontmatter reader instead of PyYAML. The corpus uses
# flat `key: value` plus folded (`>`/`|`) scalars and nothing else; a dependency
# for that is not worth it. Swap in PyYAML if nested structures ever appear.
_KEY = re.compile(r"^([A-Za-z_][\w-]*):\s*(.*)$")
_FOLD = {">", "|", ">-", "|-", ">+", "|+"}


def parse_frontmatter(text):
    """Return (frontmatter_dict, body_lines) or None if there is no frontmatter.

    Only the fenced header is parsed. Body lines that look like `description:`
    -- template examples inside a skill that documents skill-authoring -- are
    deliberately NOT treated as frontmatter. A naive grep gets this wrong.
    """
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    end = next((i for i in range(1, len(lines)) if lines[i].strip() == "---"), None)
    if end is None:
        return None
    fm, key = {}, None
    for ln in lines[1:end]:
        m = _KEY.match(ln)
        if m:
            key, val = m.group(1), m.group(2).strip()
            fm[key] = "" if val in _FOLD else val.strip("\"'")
        elif key and ln.strip():
            fm[key] = (fm[key] + " " + ln.strip()).strip()
    return fm, lines[end + 1:]


class Skill:
    def __init__(self, path):
        self.path = path
        self.dir = os.path.dirname(path)
        self.slug = os.path.basename(self.dir)
        text = open(path, encoding="utf-8").read()
        self.line_count = len(text.splitlines())
        self.text = text
        parsed = parse_frontmatter(text)
        self.frontmatter, self.body = parsed if parsed else ({}, [])

    @property
    def description(self):
        return self.frontmatter.get("description", "")


# --- assertions -------------------------------------------------------------
# Each returns True when the skill CONFORMS.

_TRIGGER = re.compile(r"\bwhen\b|\btriggers?\s+on\b", re.I)
_REF = re.compile(r"(?<![\w/.-])((?:references|scripts|assets|rules|agents)/[\w./-]+\.\w+)")

MAX_LINES = 500
MIN_DESCRIPTION = 80


def a_description_states_trigger(s):
    """Description says WHEN to use the skill, not only what it does.

    A description with no trigger clause gives the model nothing to match a user
    request against, so the skill is never selected.
    """
    return bool(_TRIGGER.search(s.description))


def a_description_min_length(s):
    """Description is long enough to disambiguate from sibling skills.

    A 33-character description cannot separate one skill from thirty others.
    """
    return len(s.description) >= MIN_DESCRIPTION


def a_body_within_line_budget(s):
    """SKILL.md fits the context budget; overflow belongs in references/."""
    return s.line_count <= MAX_LINES


def a_body_has_example(s):
    """Body contains at least one fenced block: a concrete thing to run or emit.

    Pure prose gives the model no executable form of the instruction.
    """
    return any(ln.startswith("```") for ln in s.body)


def a_refs_resolve(s):
    """Every references//scripts/ path named in the body exists on disk.

    A skill that tells the model to read a file that is not there fails at use
    time, not at load time, so nothing else catches it.
    """
    return not any(
        not os.path.exists(os.path.join(s.dir, m)) for m in set(_REF.findall(s.text))
    )


ASSERTIONS = [
    ("description_states_trigger", a_description_states_trigger),
    ("description_min_length", a_description_min_length),
    ("body_within_line_budget", a_body_within_line_budget),
    ("body_has_example", a_body_has_example),
    ("refs_resolve", a_refs_resolve),
]


# --- runner -----------------------------------------------------------------


def collect(roots):
    out = []
    for root in roots:
        if not os.path.isdir(root):
            sys.exit(f"error: corpus root not found: {root}")
        for slug in sorted(os.listdir(root)):
            path = os.path.join(root, slug, "SKILL.md")
            if os.path.isfile(path):
                out.append(Skill(path))
    if not out:
        sys.exit(f"error: no SKILL.md found under {roots}")
    return out


def evaluate(skills):
    return {
        name: sorted(s.slug for s in skills if not fn(s)) for name, fn in ASSERTIONS
    }


def main(argv):
    roots = [a for a in argv if not a.startswith("--")] or DEFAULT_ROOTS
    skills = collect(roots)
    failures = evaluate(skills)

    if "--freeze" in argv:
        limits = {k: len(v) for k, v in failures.items()}
        with open(BASELINE, "w", encoding="utf-8") as fh:
            json.dump({"corpus_size": len(skills), "max_failures": limits}, fh,
                      indent=2, sort_keys=True)
            fh.write("\n")
        print(f"froze baseline over {len(skills)} skills -> {BASELINE}")
        return 0

    if not os.path.exists(BASELINE):
        sys.exit(f"error: no baseline at {BASELINE}; run --freeze once")
    limits = json.load(open(BASELINE, encoding="utf-8"))["max_failures"]

    print(f"corpus: {len(skills)} SKILL.md under {', '.join(roots)}\n")
    print(f"{'assertion':30s} {'fail':>5s} {'max':>5s}  status")
    regressed = False
    for name, _ in ASSERTIONS:
        got, cap = len(failures[name]), limits.get(name, 0)
        bad = got > cap
        regressed |= bad
        print(f"{name:30s} {got:5d} {cap:5d}  {'REGRESSED' if bad else 'ok'}")
        if bad:
            # The baseline stores counts, not names, so the delta is not
            # recoverable here -- print the whole set and let the author diff it.
            print(f"    failing set ({got}): {failures[name]}")
    if regressed:
        print("\nratchet broken: an assertion has more failures than the frozen baseline")
    return 1 if regressed else 0


# --- selftest ---------------------------------------------------------------

GOOD = """---
name: demo
description: Runs the widget audit and reports drift. Use when the user asks to audit widgets or mentions widget drift in a review.
---

# Demo

```bash
demo --audit
```
"""

# (assertion name, mutation of GOOD that must make it FAIL)
VIOLATIONS = [
    ("description_states_trigger",
     GOOD.replace("Runs the widget audit and reports drift. Use when the user asks to audit widgets or mentions widget drift in a review.",
                  "Runs the widget audit and reports drift across every configured widget namespace.")),
    ("description_min_length", GOOD.replace(
        "Runs the widget audit and reports drift. Use when the user asks to audit widgets or mentions widget drift in a review.",
        "Use when auditing.")),
    ("body_within_line_budget", GOOD + "\nfiller\n" * (MAX_LINES + 10)),
    ("body_has_example", GOOD.replace("```bash\ndemo --audit\n```", "Just run it.")),
    ("refs_resolve", GOOD.replace("# Demo", "# Demo\n\nSee references/missing.md")),
]


def selftest():
    import tempfile
    ok = True
    with tempfile.TemporaryDirectory() as tmp:
        def write(slug, text):
            d = os.path.join(tmp, slug)
            os.makedirs(d, exist_ok=True)
            p = os.path.join(d, "SKILL.md")
            open(p, "w", encoding="utf-8").write(text)
            return Skill(p)

        baseline_skill = write("good", GOOD)
        for name, fn in ASSERTIONS:
            if not fn(baseline_skill):
                print(f"FAIL {name}: rejects the compliant fixture")
                ok = False

        for name, text in VIOLATIONS:
            fn = dict(ASSERTIONS)[name]
            if fn(write(f"bad-{name}", text)):
                print(f"FAIL {name}: passes a violating input -- the rule does not bite")
                ok = False

        covered = {n for n, _ in VIOLATIONS}
        for name, _ in ASSERTIONS:
            if name not in covered:
                print(f"FAIL {name}: no violating fixture, so the rule is unproven")
                ok = False

    print(f"selftest: {len(ASSERTIONS)} assertions, "
          f"{len(VIOLATIONS)} violating fixtures -> {'PASS' if ok else 'FAIL'}")
    return 0 if ok else 2


if __name__ == "__main__":
    sys.exit(selftest() if "--selftest" in sys.argv else main(sys.argv[1:]))
