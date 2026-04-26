use crate::parser::parse_sql;

#[test]
fn parses_with_the_default_generic_dialect() {
    let statements = parse_sql("SELECT 1", None).expect("default generic dialect should parse");

    assert_eq!(statements.len(), 1);
}

#[test]
fn rejects_unknown_dialects() {
    let error = parse_sql("SELECT 1", Some("unknown")).expect_err("unknown dialect should fail");

    assert!(error.to_string().contains("unsupported SQL dialect"));
}

#[cfg(feature = "postgres")]
#[test]
fn parses_with_the_postgres_plugin() {
    let statements = parse_sql("SELECT $1", Some("postgres"))
        .expect("postgres dialect plugin should parse postgres placeholders");

    assert_eq!(statements.len(), 1);
}

#[cfg(feature = "mysql")]
#[test]
fn parses_with_the_mysql_plugin() {
    let statements =
        parse_sql("SELECT * FROM `users`", Some("mysql")).expect("mysql dialect should parse");

    assert_eq!(statements.len(), 1);
}

#[cfg(feature = "snowflake")]
#[test]
fn parses_with_the_snowflake_plugin() {
    let statements = parse_sql("SELECT $1, identifier($2)", Some("snowflake"))
        .expect("snowflake dialect should parse snowflake placeholders");

    assert_eq!(statements.len(), 1);
}
