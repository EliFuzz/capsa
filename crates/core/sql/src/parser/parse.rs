use sqlparser::ast::Statement;
#[cfg(feature = "ansi")]
use sqlparser::dialect::AnsiDialect;
#[cfg(feature = "bigquery")]
use sqlparser::dialect::BigQueryDialect;
#[cfg(feature = "clickhouse")]
use sqlparser::dialect::ClickHouseDialect;
#[cfg(feature = "databricks")]
use sqlparser::dialect::DatabricksDialect;
#[cfg(feature = "duckdb")]
use sqlparser::dialect::DuckDbDialect;
#[cfg(feature = "hive")]
use sqlparser::dialect::HiveDialect;
#[cfg(feature = "mssql")]
use sqlparser::dialect::MsSqlDialect;
#[cfg(feature = "mysql")]
use sqlparser::dialect::MySqlDialect;
#[cfg(feature = "postgres")]
use sqlparser::dialect::PostgreSqlDialect;
#[cfg(feature = "redshift")]
use sqlparser::dialect::RedshiftSqlDialect;
#[cfg(feature = "sqlite")]
use sqlparser::dialect::SQLiteDialect;
#[cfg(feature = "snowflake")]
use sqlparser::dialect::SnowflakeDialect;
use sqlparser::dialect::{Dialect, GenericDialect};
use sqlparser::parser::{Parser, ParserError};

trait SqlDialectPlugin: Sync {
    fn name(&self) -> &'static str;
    fn parse(&self, sql: &str) -> Result<Vec<Statement>, ParserError>;
}

pub fn parse_sql(sql: &str, dialect: Option<&str>) -> Result<Vec<Statement>, ParserError> {
    let plugin = dialect
        .map(find_plugin)
        .transpose()?
        .unwrap_or(&GENERIC_PLUGIN);

    plugin.parse(sql)
}

fn find_plugin(dialect: &str) -> Result<&'static dyn SqlDialectPlugin, ParserError> {
    SQL_DIALECT_PLUGINS
        .iter()
        .copied()
        .find(|plugin| plugin.name().eq_ignore_ascii_case(dialect))
        .ok_or_else(|| ParserError::ParserError(format!("unsupported SQL dialect: {dialect}")))
}

struct ParserPlugin<T> {
    name: &'static str,
    dialect: T,
}

impl<T> SqlDialectPlugin for ParserPlugin<T>
where
    T: Dialect + Sync,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn parse(&self, sql: &str) -> Result<Vec<Statement>, ParserError> {
        Parser::parse_sql(&self.dialect, sql)
    }
}

static GENERIC_PLUGIN: ParserPlugin<GenericDialect> = ParserPlugin {
    name: "generic",
    dialect: GenericDialect {},
};

static SQL_DIALECT_PLUGINS: &[&dyn SqlDialectPlugin] = &[
    &GENERIC_PLUGIN,
    #[cfg(feature = "ansi")]
    &ANSI_PLUGIN,
    #[cfg(feature = "bigquery")]
    &BIGQUERY_PLUGIN,
    #[cfg(feature = "clickhouse")]
    &CLICKHOUSE_PLUGIN,
    #[cfg(feature = "databricks")]
    &DATABRICKS_PLUGIN,
    #[cfg(feature = "duckdb")]
    &DUCKDB_PLUGIN,
    #[cfg(feature = "hive")]
    &HIVE_PLUGIN,
    #[cfg(feature = "mssql")]
    &MSSQL_PLUGIN,
    #[cfg(feature = "mysql")]
    &MYSQL_PLUGIN,
    #[cfg(feature = "postgres")]
    &POSTGRES_PLUGIN,
    #[cfg(feature = "redshift")]
    &REDSHIFT_PLUGIN,
    #[cfg(feature = "snowflake")]
    &SNOWFLAKE_PLUGIN,
    #[cfg(feature = "sqlite")]
    &SQLITE_PLUGIN,
];

#[cfg(feature = "ansi")]
static ANSI_PLUGIN: ParserPlugin<AnsiDialect> = ParserPlugin {
    name: "ansi",
    dialect: AnsiDialect {},
};

#[cfg(feature = "bigquery")]
static BIGQUERY_PLUGIN: ParserPlugin<BigQueryDialect> = ParserPlugin {
    name: "bigquery",
    dialect: BigQueryDialect {},
};

#[cfg(feature = "clickhouse")]
static CLICKHOUSE_PLUGIN: ParserPlugin<ClickHouseDialect> = ParserPlugin {
    name: "clickhouse",
    dialect: ClickHouseDialect {},
};

#[cfg(feature = "databricks")]
static DATABRICKS_PLUGIN: ParserPlugin<DatabricksDialect> = ParserPlugin {
    name: "databricks",
    dialect: DatabricksDialect {},
};

#[cfg(feature = "duckdb")]
static DUCKDB_PLUGIN: ParserPlugin<DuckDbDialect> = ParserPlugin {
    name: "duckdb",
    dialect: DuckDbDialect {},
};

#[cfg(feature = "hive")]
static HIVE_PLUGIN: ParserPlugin<HiveDialect> = ParserPlugin {
    name: "hive",
    dialect: HiveDialect {},
};

#[cfg(feature = "mssql")]
static MSSQL_PLUGIN: ParserPlugin<MsSqlDialect> = ParserPlugin {
    name: "mssql",
    dialect: MsSqlDialect {},
};

#[cfg(feature = "mysql")]
static MYSQL_PLUGIN: ParserPlugin<MySqlDialect> = ParserPlugin {
    name: "mysql",
    dialect: MySqlDialect {},
};

#[cfg(feature = "postgres")]
static POSTGRES_PLUGIN: ParserPlugin<PostgreSqlDialect> = ParserPlugin {
    name: "postgres",
    dialect: PostgreSqlDialect {},
};

#[cfg(feature = "redshift")]
static REDSHIFT_PLUGIN: ParserPlugin<RedshiftSqlDialect> = ParserPlugin {
    name: "redshift",
    dialect: RedshiftSqlDialect {},
};

#[cfg(feature = "snowflake")]
static SNOWFLAKE_PLUGIN: ParserPlugin<SnowflakeDialect> = ParserPlugin {
    name: "snowflake",
    dialect: SnowflakeDialect {},
};

#[cfg(feature = "sqlite")]
static SQLITE_PLUGIN: ParserPlugin<SQLiteDialect> = ParserPlugin {
    name: "sqlite",
    dialect: SQLiteDialect {},
};
