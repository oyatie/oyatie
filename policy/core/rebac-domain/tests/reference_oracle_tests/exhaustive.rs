use std::collections::BTreeSet;

use crate::reference_oracle_support::cases::{generated_cases, queries, tuple_universe};
use crate::reference_oracle_support::harness::{Execution, oracle_outcome, production_outcome};
use crate::reference_oracle_support::{Outcome, Tuple};

#[test]
fn mutation_controls_cover_every_generated_rewrite_comparison() {
    let cases = generated_cases();
    let universe = tuple_universe();
    let queries = queries();
    let names: BTreeSet<&str> = cases.iter().map(|case| case.name.as_str()).collect();
    let mut operators = BTreeSet::new();
    let mut comparisons = 0usize;
    let mut allows = 0usize;
    let mut denies = 0usize;

    assert_eq!(cases.len(), 52, "the bounded namespace grammar changed");
    assert_eq!(names.len(), cases.len(), "generated namespaces are unique");
    for case in &cases {
        operators.extend(&case.operators);
        for mask in 0..(1usize << universe.len()) {
            let tuples: Vec<Tuple> = universe
                .iter()
                .enumerate()
                .filter(|(index, _)| mask & (1usize << index) != 0)
                .map(|(_, tuple)| tuple.clone())
                .collect();
            for page_size in [1, 2] {
                let execution = Execution {
                    page_size,
                    ..Execution::default()
                };
                for query in &queries {
                    let reference = oracle_outcome(&case.model, &tuples, query);
                    let production = production_outcome(&case.model, &tuples, query, &execution);
                    assert_eq!(
                        production, reference,
                        "{} disagreed for tuple mask {mask:#x}, page size {page_size}, query {query:?}",
                        case.name
                    );
                    match reference {
                        Outcome::Allow => allows += 1,
                        Outcome::Deny => denies += 1,
                        Outcome::Refuse(refusal) => {
                            panic!("bounded valid grammar unexpectedly refused: {refusal:?}")
                        }
                    }
                    comparisons += 1;
                }
            }
        }
    }

    assert_eq!(comparisons, 6_656, "the Cartesian corpus changed");
    assert!(
        allows > 100 && denies > 100,
        "the corpus must carry substantial positive and negative values"
    );
    assert_eq!(
        operators,
        BTreeSet::from([
            "this",
            "computed_userset",
            "tuple_to_userset",
            "union",
            "intersection",
            "difference",
        ]),
        "every rewrite operator must remain generated"
    );
}
