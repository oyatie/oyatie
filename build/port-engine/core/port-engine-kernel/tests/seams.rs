//! Seam tests for `port-engine-kernel`: every seam ADR-0637 D1 names is proven INHABITABLE by
//! in-memory fakes alone — no front end, no rule corpus, no filesystem, no clock — and every
//! fail-closed refusal is exercised.
//!
//! These live outside `src/lib.rs` because the kernel's compile-time neutrality rule refuses ANY
//! submodule declaration, inline ones included — that blunt rule is what makes its one-file scan a
//! complete scan. Everything under test is public API, so nothing is lost by testing from outside.
//!
//! This file is scanned by that same rule (`src/lib.rs` reads it with `include_str!`), so it too
//! carries no corpus vocabulary and no submodules. `tests/neutrality.rs` is the one file the scan
//! does not read, because it must spell the needles out to prove they go red.
//!
//! ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};

use port_engine_kernel::{
    Delta, Digest, LanguagePair, PlanStep, PortError, RECEIPT_AXES, Receipt, ReceiptAxis, RegionId,
    Renderer, RuleId, RulePack, SourceModel, TargetIr, UnitId, Verdict, Verification, emit, plan,
    verify,
};

// The in-memory fakes. Their only job is to prove each seam is inhabitable without a front end, a
// rule corpus, or a filesystem — which is the whole W0 claim.

struct FakeModel {
    language: String,
    units: Vec<UnitId>,
}

impl SourceModel for FakeModel {
    fn language(&self) -> &str {
        &self.language
    }
    fn snapshot_digest(&self) -> Digest {
        Digest("snapshot-0".into())
    }
    fn units(&self) -> Vec<UnitId> {
        self.units.clone()
    }
}

struct FakePack {
    pair: LanguagePair,
    rules: Vec<RuleId>,
    // unit -> rules the pack claims apply, in pack order.
    applies: BTreeMap<UnitId, Vec<RuleId>>,
}

impl RulePack for FakePack {
    fn pair(&self) -> &LanguagePair {
        &self.pair
    }
    fn digest(&self) -> Digest {
        Digest("pack-0".into())
    }
    fn rules(&self) -> Vec<RuleId> {
        self.rules.clone()
    }
    fn rules_for(&self, unit: &UnitId) -> Vec<RuleId> {
        self.applies.get(unit).cloned().unwrap_or_default()
    }
}

struct FakeIr {
    target_language: String,
    regions: Vec<RegionId>,
}

impl TargetIr for FakeIr {
    fn target_language(&self) -> &str {
        &self.target_language
    }
    fn regions(&self) -> Vec<RegionId> {
        self.regions.clone()
    }
}

struct FakeRenderer {
    target_language: String,
    // Regions this renderer will actually emit, independent of what the IR declares, so the
    // drop/invent defect is reachable in a test.
    emits: Vec<RegionId>,
}

impl Renderer for FakeRenderer {
    fn target_language(&self) -> &str {
        &self.target_language
    }
    fn formatter_digest(&self) -> Digest {
        Digest("fmt-0".into())
    }
    fn render(&self, _ir: &dyn TargetIr) -> Result<BTreeMap<RegionId, Vec<u8>>, PortError> {
        Ok(self
            .emits
            .iter()
            .map(|r| (r.clone(), r.0.as_bytes().to_vec()))
            .collect())
    }
}

fn unit(s: &str) -> UnitId {
    UnitId(s.into())
}
fn rule(s: &str) -> RuleId {
    RuleId(s.into())
}
fn region(s: &str) -> RegionId {
    RegionId(s.into())
}

// Two arbitrary slugs. Deliberately NOT any real language name: the kernel must be provably
// indifferent to which pair it is handed, and a test that only ever passes one real pair would
// not show that.
fn pair() -> LanguagePair {
    LanguagePair {
        source: "alpha".into(),
        target: "beta".into(),
    }
}

fn pack_with(applies: BTreeMap<UnitId, Vec<RuleId>>, rules: Vec<RuleId>) -> FakePack {
    FakePack {
        pair: pair(),
        rules,
        applies,
    }
}

fn receipt() -> Receipt {
    Receipt {
        pin: "pin-0".into(),
        snapshot_digest: Digest("snap-0".into()),
        engine_digest: Digest("engine-0".into()),
        rulepack_digest: Digest("pack-0".into()),
        toolchain_digest: Digest("tc-0".into()),
        formatter_digest: Digest("fmt-0".into()),
    }
}

#[test]
fn pair_slug_is_the_rule_namespace_segment() {
    assert_eq!(pair().slug(), "alpha-beta");
}

#[test]
fn the_model_seam_carries_the_snapshot_axis_of_the_receipt() {
    let model = FakeModel {
        language: "alpha".into(),
        units: vec![],
    };
    assert_eq!(model.snapshot_digest(), Digest("snapshot-0".into()));
}

#[test]
fn the_pack_seam_carries_the_rulepack_axis_of_the_receipt() {
    let pack = pack_with(BTreeMap::new(), vec![]);
    assert_eq!(pack.digest(), Digest("pack-0".into()));
}

#[test]
fn the_renderer_seam_carries_the_formatter_axis_of_the_receipt() {
    let renderer = FakeRenderer {
        target_language: "beta".into(),
        emits: vec![],
    };
    assert_eq!(renderer.formatter_digest(), Digest("fmt-0".into()));
}

#[test]
fn plan_orders_by_unit_then_pack_rule_order() {
    let model = FakeModel {
        language: "alpha".into(),
        units: vec![unit("u2"), unit("u1")],
    };
    let mut applies = BTreeMap::new();
    applies.insert(unit("u1"), vec![rule("r2"), rule("r1")]);
    applies.insert(unit("u2"), vec![rule("r1")]);
    let pack = pack_with(applies, vec![rule("r1"), rule("r2")]);

    let plan = plan(&model, &pack).expect("plan");

    // Model order (u2 before u1) is preserved, NOT sorted; within a unit the pack's returned
    // order is preserved. Both are the supplier's decision, and the engine must not reorder.
    assert_eq!(
        plan.steps,
        vec![
            PlanStep {
                unit: unit("u2"),
                rule: rule("r1")
            },
            PlanStep {
                unit: unit("u1"),
                rule: rule("r2")
            },
            PlanStep {
                unit: unit("u1"),
                rule: rule("r1")
            },
        ]
    );
    assert_eq!(plan.pair, pair());
}

#[test]
fn plan_is_stable_across_repeated_calls() {
    let model = FakeModel {
        language: "alpha".into(),
        units: vec![unit("u1"), unit("u2")],
    };
    let mut applies = BTreeMap::new();
    applies.insert(unit("u1"), vec![rule("r1")]);
    applies.insert(unit("u2"), vec![rule("r1")]);
    let pack = pack_with(applies, vec![rule("r1")]);

    assert_eq!(plan(&model, &pack), plan(&model, &pack));
}

#[test]
fn plan_refuses_a_pack_authored_for_another_source_language() {
    let model = FakeModel {
        language: "gamma".into(),
        units: vec![unit("u1")],
    };
    let pack = pack_with(BTreeMap::new(), vec![]);

    assert_eq!(
        plan(&model, &pack),
        Err(PortError::LanguageMismatch {
            expected: "alpha".into(),
            actual: "gamma".into(),
        })
    );
}

#[test]
fn plan_refuses_a_duplicate_unit_because_step_order_would_be_ambiguous() {
    let model = FakeModel {
        language: "alpha".into(),
        units: vec![unit("u1"), unit("u1")],
    };
    let mut applies = BTreeMap::new();
    applies.insert(unit("u1"), vec![rule("r1")]);
    let pack = pack_with(applies, vec![rule("r1")]);

    assert_eq!(
        plan(&model, &pack),
        Err(PortError::DuplicateUnit { unit: unit("u1") })
    );
}

#[test]
fn plan_refuses_a_rule_the_pack_never_declared() {
    let model = FakeModel {
        language: "alpha".into(),
        units: vec![unit("u1")],
    };
    let mut applies = BTreeMap::new();
    applies.insert(unit("u1"), vec![rule("ghost")]);
    let pack = pack_with(applies, vec![rule("r1")]);

    assert_eq!(
        plan(&model, &pack),
        Err(PortError::UndeclaredRule {
            unit: unit("u1"),
            rule: rule("ghost"),
        })
    );
}

#[test]
fn plan_over_an_empty_model_is_an_empty_plan_not_an_error() {
    let model = FakeModel {
        language: "alpha".into(),
        units: vec![],
    };
    let pack = pack_with(BTreeMap::new(), vec![rule("r1")]);

    assert_eq!(plan(&model, &pack).expect("plan").steps, vec![]);
}

#[test]
fn emit_returns_every_declared_region() {
    let ir = FakeIr {
        target_language: "beta".into(),
        regions: vec![region("a"), region("b")],
    };
    let renderer = FakeRenderer {
        target_language: "beta".into(),
        emits: vec![region("a"), region("b")],
    };

    let out = emit(&renderer, &ir).expect("emit");
    assert_eq!(
        out.keys().cloned().collect::<Vec<_>>(),
        vec![region("a"), region("b")]
    );
    assert_eq!(
        out.get(&region("a")).map(Vec::as_slice),
        Some(b"a".as_slice())
    );
}

#[test]
fn emit_refuses_a_renderer_for_another_target_language() {
    let ir = FakeIr {
        target_language: "beta".into(),
        regions: vec![region("a")],
    };
    let renderer = FakeRenderer {
        target_language: "delta".into(),
        emits: vec![region("a")],
    };

    assert_eq!(
        emit(&renderer, &ir),
        Err(PortError::LanguageMismatch {
            expected: "beta".into(),
            actual: "delta".into(),
        })
    );
}

#[test]
fn emit_refuses_a_renderer_that_drops_or_invents_a_region() {
    let ir = FakeIr {
        target_language: "beta".into(),
        regions: vec![region("a"), region("b")],
    };
    let renderer = FakeRenderer {
        target_language: "beta".into(),
        emits: vec![region("a"), region("c")],
    };

    assert_eq!(
        emit(&renderer, &ir),
        Err(PortError::RegionSetMismatch {
            missing: [region("b")].into_iter().collect(),
            unexpected: [region("c")].into_iter().collect(),
        })
    );
}

#[test]
fn no_change_is_unchanged_even_when_every_axis_moved() {
    let previous = receipt();
    let mut current = receipt();
    current.pin = "pin-1".into();

    assert_eq!(
        verify(&previous, &current, &BTreeSet::new()),
        Verification {
            verdict: Verdict::Green,
            delta: Delta::Unchanged,
        }
    );
}

#[test]
fn a_changed_region_with_a_moved_axis_is_explained() {
    let previous = receipt();
    let mut current = receipt();
    current.rulepack_digest = Digest("pack-1".into());
    let changed: BTreeSet<RegionId> = [region("a")].into_iter().collect();

    assert_eq!(
        verify(&previous, &current, &changed),
        Verification {
            verdict: Verdict::Green,
            delta: Delta::Explained {
                regions: changed,
                axes: [ReceiptAxis::RulePack].into_iter().collect(),
            },
        }
    );
}

#[test]
fn a_changed_region_with_every_axis_held_is_unexplained_and_red() {
    let previous = receipt();
    let current = receipt();
    let changed: BTreeSet<RegionId> = [region("a")].into_iter().collect();

    assert_eq!(
        verify(&previous, &current, &changed),
        Verification {
            verdict: Verdict::Red,
            delta: Delta::Unexplained { regions: changed },
        }
    );
}

#[test]
fn every_declared_axis_is_actually_compared() {
    // A seventh axis added to RECEIPT_AXES without a comparison arm would silently never differ;
    // this walks each axis and proves a change on it is detected.
    let base = receipt();
    for axis in RECEIPT_AXES {
        let mut moved = base.clone();
        match axis {
            ReceiptAxis::Pin => moved.pin = "other".into(),
            ReceiptAxis::Snapshot => moved.snapshot_digest = Digest("other".into()),
            ReceiptAxis::Engine => moved.engine_digest = Digest("other".into()),
            ReceiptAxis::RulePack => moved.rulepack_digest = Digest("other".into()),
            ReceiptAxis::Toolchain => moved.toolchain_digest = Digest("other".into()),
            ReceiptAxis::Formatter => moved.formatter_digest = Digest("other".into()),
        }
        assert_eq!(
            base.differing_axes(&moved),
            [axis].into_iter().collect::<BTreeSet<_>>(),
            "axis {axis:?} is declared but not compared"
        );
    }
}

#[test]
fn an_identical_receipt_differs_on_nothing() {
    assert!(receipt().differing_axes(&receipt()).is_empty());
}

#[test]
fn errors_render_their_subject() {
    let rendered = PortError::UndeclaredRule {
        unit: unit("u1"),
        rule: rule("ghost"),
    }
    .to_string();
    assert!(rendered.contains("ghost"), "{rendered}");
    assert!(rendered.contains("u1"), "{rendered}");
}
