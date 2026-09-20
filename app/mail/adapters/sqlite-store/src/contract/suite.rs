//! The contract suite: numbered gates any `Store` implementation must pass,
//! run against a `Fixture` that opens a fresh store with the accounts and
//! consumers a gate asks for. The SQLite tier runs it under the merge
//! verdict; the hosted tier (S9) is accepted only when it runs green here.
use mail_api::{Consumer, DeliveryQueue, Store, SubmissionQueue};
use std::fmt;

/// An account the suite asks a fixture to provision: `(id, tenant, address)`.
pub type AccountSpec<'a> = (&'a str, &'a str, &'a str);

/// Opens a fresh, empty store with `consumers` enabled and `accounts`
/// provisioned. A closure `Fn(&[Consumer], &[AccountSpec]) -> S` is one.
pub trait Fixture {
    type Store: Store + DeliveryQueue + SubmissionQueue;
    fn open(&self, consumers: &[Consumer], accounts: &[AccountSpec<'_>]) -> Self::Store;
}
impl<F, S> Fixture for F
where
    F: Fn(&[Consumer], &[AccountSpec<'_>]) -> S,
    S: Store + DeliveryQueue + SubmissionQueue,
{
    type Store = S;
    fn open(&self, consumers: &[Consumer], accounts: &[AccountSpec<'_>]) -> S {
        self(consumers, accounts)
    }
}

/// One predicate of one gate that did not hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateFailure {
    pub gate: u8,
    pub case: &'static str,
    pub detail: String,
}
impl fmt::Display for GateFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "gate {} {}: {}", self.gate, self.case, self.detail)
    }
}

/// A measured value a gate reports beside its verdict (rows read, rounds).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Measure {
    pub gate: u8,
    pub case: &'static str,
    pub value: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub measures: Vec<Measure>,
}

/// Collects one gate's predicates: the first failing predicate ends the
/// gate, so a `Broken` store reports the violation it hit first.
pub struct Gate {
    pub number: u8,
    pub measures: Vec<Measure>,
}
impl Gate {
    pub fn check(
        &self,
        case: &'static str,
        ok: bool,
        detail: impl fmt::Display,
    ) -> Result<(), GateFailure> {
        if ok {
            Ok(())
        } else {
            Err(GateFailure {
                gate: self.number,
                case,
                detail: detail.to_string(),
            })
        }
    }
    /// A store error inside a gate is that gate's failure, never a panic.
    pub fn store(&self, case: &'static str, error: mail_kernel::Error) -> GateFailure {
        GateFailure {
            gate: self.number,
            case,
            detail: format!("store error {error:?}"),
        }
    }
    pub fn measure(&mut self, case: &'static str, value: u64) {
        self.measures.push(Measure {
            gate: self.number,
            case,
            value,
        });
    }
}

type Run<F> = fn(&F, &mut Gate) -> Result<(), GateFailure>;

/// Every gate is run even after one fails, so the self-check can assert the
/// exact set of gates a deliberately broken store trips.
pub fn run_suite<F: Fixture>(fixture: &F) -> Result<Report, Vec<GateFailure>> {
    let mut report = Report::default();
    let mut failures = Vec::new();
    let gates: [(u8, Run<F>); 5] = [
        (2, super::gates::cost::run),
        (3, super::gates::fence::run),
        (4, super::gates::replay::run),
        (5, super::gates::feed::run),
        (6, super::gates::sweep::run),
    ];
    for (number, run) in gates {
        let mut gate = Gate {
            number,
            measures: Vec::new(),
        };
        if let Err(failure) = run(fixture, &mut gate) {
            failures.push(failure);
        }
        report.measures.append(&mut gate.measures);
    }
    if failures.is_empty() {
        Ok(report)
    } else {
        Err(failures)
    }
}
