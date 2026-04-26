use crate::validate::validate_sql_query_schema;

#[test]
fn accepts_valid_sql() {
    assert_eq!(validate_sql_query_schema("SELECT 1", None), None);
}

#[test]
fn rejects_invalid_sql() {
    assert!(validate_sql_query_schema("SELECT FROM", None).is_some());
}

#[test]
fn rejects_unknown_dialect() {
    assert_eq!(
        validate_sql_query_schema("SELECT 1", Some("unknown")),
        Some(vec![String::from(
            "sql parser error: unsupported SQL dialect: unknown"
        )])
    );
}
