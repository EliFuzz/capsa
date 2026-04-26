use crate::parser::parse_sql;

pub fn validate_sql_query_schema(sql: &str, dialect: Option<&str>) -> Option<Vec<String>> {
    parse_sql(sql, dialect)
        .err()
        .map(|error| vec![error.to_string()])
}
