# capsa

SQL validation for teams that want bad queries caught before they ship.

`capsa` parses SQL, validates dialect syntax, and scans the query against rule sets for security, performance, reliability, and data-handling risk. It does not execute SQL. It gives you fast feedback on the query you are about to run, review, or deploy.

## What It Does

- Parses SQL into statements for validation
- Supports dialect-aware validation through parser plugins
- Checks query against compiled rules
- Runs schema validation and rule validation in parallel
- Returns a compact list of findings

## Usage

1. Build: `cargo build --profile release -p cli`
2. Run: `capsa -q "SELECT * FROM users"` - ouputs: `["SELECT *; list required columns", "Unbounded SELECT; add WHERE, LIMIT, or pagination"]`
3. Dialect (optional): `capsa -q "SELECT * FROM users" -d postgres`. Supported dialects: ansi, bigquery, clickhouse, databricks, duckdb, generic, hive, mssql, mysql, postgres, redshift, snowflake, sqlite.

## Build

Dialects are implemented as plugins. To build with a specific dialect: `cargo build -p core-sql --features postgres`.
