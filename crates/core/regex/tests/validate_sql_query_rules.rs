use core_regex::validate_sql_query_rules;

fn errors_for(sql: &str) -> Vec<&'static str> {
    validate_sql_query_rules(sql).unwrap_or_default()
}

#[test]
fn returns_matching_rule_errors() {
    let errors =
        validate_sql_query_rules("SELECT * FROM orders; DROP DATABASE prod;").expect("rule errors");

    assert!(errors.contains(&"SELECT *; list required columns"));
    assert!(errors.contains(&"DROP DATABASE; require backup and approval"));
}

#[test]
fn returns_all_errors_reported_by_vectorscan() {
    let errors = validate_sql_query_rules("SELECT * FROM orders; SELECT * FROM customers;")
        .expect("rule errors");
    let count = errors
        .iter()
        .filter(|error| {
            **error == "SELECT *; list required columns"
        })
        .count();

    assert!(count >= 2);
}

#[test]
fn returns_no_errors_when_no_rules_match() {
    assert_eq!(validate_sql_query_rules(" "), None);
    assert_eq!(validate_sql_query_rules(""), None);
}

#[test]
fn update_delete_row_limit_rules_are_statement_local() {
    let filtered_update = errors_for("UPDATE t SET c = 1 WHERE id = 7 LIMIT 1;");
    assert!(!filtered_update.contains(
        &"UPDATE without WHERE; add filter"
    ));
    assert!(!filtered_update.contains(
        &"Limited DML without WHERE; add key filter"
    ));

    let unfiltered_update = errors_for("UPDATE t SET c = 1 LIMIT 1;");
    assert!(unfiltered_update.contains(
        &"UPDATE without WHERE; add filter"
    ));
    assert!(unfiltered_update.contains(
        &"Limited DML without WHERE; add key filter"
    ));

    let separate_select_limit = errors_for("UPDATE t SET c = 1; SELECT * FROM t LIMIT 10;");
    assert!(!separate_select_limit.contains(
        &"Limited DML without WHERE; add key filter"
    ));
}

#[test]
fn select_row_limit_rules_respect_order_by() {
    let ordered_limit = errors_for("SELECT * FROM t ORDER BY id LIMIT 10;");
    assert!(!ordered_limit.contains(
        &"LIMIT/OFFSET unordered; add stable ORDER BY"
    ));

    let unordered_limit = errors_for("SELECT * FROM t LIMIT 10;");
    assert!(unordered_limit.contains(
        &"LIMIT/OFFSET unordered; add stable ORDER BY"
    ));

    let ordered_top = errors_for("SELECT TOP 10 * FROM t ORDER BY id;");
    assert!(!ordered_top.contains(
        &"TOP without order; add stable ORDER BY"
    ));

    let unordered_top = errors_for("SELECT TOP 10 * FROM t;");
    assert!(unordered_top.contains(
        &"TOP without order; add stable ORDER BY"
    ));
}
