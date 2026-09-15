//! Both sides of every boundary the filter grammar states: the shapes it
//! refuses, each matched to the cause naming the rule it broke, and the shapes
//! it serves with the rows each one selects.

mod filter_support;

use axum::http::StatusCode;
use filter_support::{TYPE, list, refs, three_tenants};

/// Both sides of every stated boundary of the filter grammar. The served
/// rows prove the refusals are the value under test and not the route.
#[tokio::test]
async fn the_filter_grammar_holds_at_both_edges() {
    let (fixture, session) = three_tenants("filter-grammar").await;
    let token = fixture.operator_token();
    let base = format!("type={TYPE}&revision=1");

    let refused: &[(String, &str)] = &[
        (format!("{base}&equals=text:Ada"), "a filter is ?property="),
        (
            format!("{base}&from=int:1&to=int:2"),
            "a filter is ?property=",
        ),
        (format!("{base}&property=name"), "a filter is ?property="),
        (
            format!("{base}&property=name&equals=text:Ada&from=int:1&to=int:2"),
            "not both",
        ),
        (
            format!("{base}&property=name&from=int:1"),
            "either ?equals= or both",
        ),
        (
            format!("{base}&property=name&to=int:2"),
            "either ?equals= or both",
        ),
        (
            format!("{base}&property=&equals=text:Ada"),
            "must name the property to filter on",
        ),
        (
            format!("{base}&property=&from=int:1&to=int:2"),
            "must name the property to filter on",
        ),
        (
            format!("{base}&property=name&equals=Ada"),
            "text:, int: or bool:",
        ),
        (
            format!("{base}&property=name&equals=date:2020-01-01"),
            "text:, int: or bool:",
        ),
        (
            format!("{base}&property=name&equals=int:notanumber"),
            "text:, int: or bool:",
        ),
        (
            format!("{base}&property=name&equals=bool:yes"),
            "text:, int: or bool:",
        ),
        (
            format!("{base}&property=name&from=int:5&to=int:1"),
            "may not sort after",
        ),
        (
            format!("{base}&property=name&from=int:1&to=text:z"),
            "same kind",
        ),
        (
            format!("{base}&property=absent&equals=text:Ada"),
            "declares no property by that name",
        ),
        (
            format!("{base}&property=nam&equals=text:Ada"),
            "declares no property by that name",
        ),
        (
            format!("{base}&property=names&equals=text:Ada"),
            "declares no property by that name",
        ),
        (
            format!("{base}&property=NAME&equals=text:Ada"),
            "declares no property by that name",
        ),
        (
            format!("{base}&property=absent&from=text:A&to=text:z"),
            "declares no property by that name",
        ),
    ];
    for (query, expected) in refused {
        let (status, body) = list(&session, token, query).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}: {body}");
        assert_eq!(body["gate"], "surface", "{query}");
        let cause = body["cause"].as_str().expect("a cause");
        assert!(cause.contains(expected), "{query} answered {cause}");
    }

    // Each served shape is judged by the rows it selects: a status alone
    // would pass while the filter was ignored and every row came back.
    let served: &[(String, Vec<&str>)] = &[
        (
            format!("{base}&property=name&equals=text:Ada"),
            vec!["ent_mine_a"],
        ),
        (format!("{base}&property=name&equals=text:"), vec![]),
        (
            format!("{base}&property=name&from=text:A&to=text:z"),
            vec!["ent_mine_a", "ent_mine_b"],
        ),
        (
            format!("{base}&property=name&from=text:A&to=text:A"),
            vec![],
        ),
        (format!("{base}&property=name&equals=bool:true"), vec![]),
        (format!("{base}&property=name&equals=bool:false"), vec![]),
        // The third cell of the guard's truth table: `note` is declared at the
        // pin, so the filter is answerable, and no object carries it, so this
        // store's property index has no row to offer. These rows witness the
        // ANSWER this route gives, not the port rule behind it: that an absent
        // property is no match belongs to `PropertyPredicate`, and a page over
        // this store never reaches it. The refused rows above are the cell
        // where the pin declares no such property at all.
        (format!("{base}&property=note&equals=text:x"), vec![]),
        (
            format!("{base}&property=note&from=text:A&to=text:z"),
            vec![],
        ),
    ];
    for (query, expected) in served {
        let (status, body) = list(&session, token, query).await;
        assert_eq!(status, StatusCode::OK, "{query}: {body}");
        assert_eq!(&refs(&body), expected, "{query}");
    }
}

/// A RANGE whose bounds are of one kind over stored values of another is a
/// conflict, not a store outage and not an empty page — for each kind of bound
/// that can mismatch a text-valued property, which is every kind the grammar
/// spells except text itself.
///
/// An EQUALITY under the wrong kind is the served empty page below. Over this
/// store the empty page comes from the property index, which is keyed by value
/// kind, so these rows witness the answer and not the port's law that exact
/// typed equality makes a differently-kinded value simply unequal.
#[tokio::test]
async fn a_range_over_values_of_another_kind_is_a_conflict() {
    let (fixture, session) = three_tenants("filter-kind").await;
    let token = fixture.operator_token();
    for bounds in ["from=int:1&to=int:9", "from=bool:false&to=bool:true"] {
        let (status, body) = list(
            &session,
            token,
            &format!("type={TYPE}&revision=1&property=name&{bounds}"),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "{bounds}: {body}");
        assert_eq!(body["gate"], "surface", "{bounds}");
        assert!(
            body["cause"]
                .as_str()
                .expect("a cause")
                .contains("another kind"),
            "{bounds}: {body}"
        );
    }
    for equals in ["equals=int:1", "equals=bool:true"] {
        let (status, body) = list(
            &session,
            token,
            &format!("type={TYPE}&revision=1&property=name&{equals}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{equals}: {body}");
        assert_eq!(refs(&body), Vec::<String>::new(), "{equals}");
    }
}
