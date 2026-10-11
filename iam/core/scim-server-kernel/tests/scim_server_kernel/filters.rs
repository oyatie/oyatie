use super::*;

#[tokio::test]
async fn filter_eq_matches_username() {
    let s = srv();
    s.create_user(&tenant(), new_user("alice"), 1_700_000_000)
        .await
        .expect("c");
    s.create_user(&tenant(), new_user("bob"), 1_700_000_001)
        .await
        .expect("c");
    let r = s
        .list_users(
            &tenant(),
            &ListQuery {
                start_index: 1,
                items_per_page: 100,
                filter: Some(r#"userName eq "alice""#.into()),
            },
        )
        .await
        .expect("list");
    assert_eq!(r.total_results, 1);
    assert_eq!(r.resources[0].user_name, "alice");
}

#[tokio::test]
async fn filter_co_matches_substring() {
    let s = srv();
    s.create_user(&tenant(), new_user("alice"), 1_700_000_000)
        .await
        .expect("c");
    s.create_user(&tenant(), new_user("bob"), 1_700_000_001)
        .await
        .expect("c");
    let r = s
        .list_users(
            &tenant(),
            &ListQuery {
                start_index: 1,
                items_per_page: 100,
                filter: Some(r#"userName co "li""#.into()),
            },
        )
        .await
        .expect("list");
    assert_eq!(r.total_results, 1);
}

#[tokio::test]
async fn filter_and_combines_conditions() {
    let s = srv();
    s.create_user(&tenant(), new_user("alice"), 1_700_000_000)
        .await
        .expect("c");
    s.create_user(&tenant(), new_user("alex"), 1_700_000_001)
        .await
        .expect("c");
    s.create_user(&tenant(), new_user("bob"), 1_700_000_002)
        .await
        .expect("c");
    let r = s
        .list_users(
            &tenant(),
            &ListQuery {
                start_index: 1,
                items_per_page: 100,
                filter: Some(r#"userName sw "al" and active eq "true""#.into()),
            },
        )
        .await
        .expect("list");
    assert_eq!(r.total_results, 2);
}

#[tokio::test]
async fn filter_pr_present() {
    let s = srv();
    s.create_user(&tenant(), new_user("alice"), 1_700_000_000)
        .await
        .expect("c");
    let r = s
        .list_users(
            &tenant(),
            &ListQuery {
                start_index: 1,
                items_per_page: 100,
                filter: Some("userName pr".into()),
            },
        )
        .await
        .expect("list");
    assert_eq!(r.total_results, 1);
}

#[test]
fn filter_parser_rejects_garbage() {
    let err = parse_filter(r#"userName == "alice""#).unwrap_err();
    assert_eq!(err.scim_type, Some(ScimType::InvalidFilter));
}

#[test]
fn filter_parser_handles_or() {
    let e = parse_filter(r#"userName eq "a" or userName eq "b""#).expect("parse");
    assert!(matches!(e, FilterExpr::Or(_, _)));
}
