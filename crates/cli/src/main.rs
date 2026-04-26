use core_regex::validate_sql_query_rules;
use core_sql::validate::validate_sql_query_schema;
use std::env::args;
use std::process::exit;
use std::thread::scope;

const USAGE: &str = "usage: capsa -q <query> [-d <dialect>]";

struct Args {
    query: String,
    dialect: Option<String>,
}

fn main() {
    let args = parse_args(args().skip(1)).unwrap_or_else(|error| {
        eprintln!("{error}");
        exit(2);
    });

    println!("{:?}", validate(&args.query, args.dialect.as_deref()));
}

fn validate(query: &str, dialect: Option<&str>) -> Vec<String> {
    scope(|scope| {
        let schema = scope.spawn(|| validate_sql_query_schema(query, dialect));
        let rules = scope.spawn(|| validate_sql_query_rules(query));

        let mut results = schema
            .join()
            .expect("schema validation thread panicked")
            .unwrap_or_default();
        if let Some(errors) = rules.join().expect("rules validation thread panicked") {
            results.extend(errors.into_iter().map(str::to_owned));
        }
        results
    })
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut args = args.into_iter();
    let mut query = String::new();
    let mut dialect = None;

    while let Some(arg) = args.next() {
        let Some(value) = args.next() else {
            return Err(USAGE.to_owned());
        };

        match arg.as_str() {
            "-q" | "--query" => query = value,
            "-d" | "--dialect" => dialect = Some(value),
            _ => return Err(USAGE.to_owned()),
        }
    }

    if query.is_empty() {
        return Err(USAGE.to_owned());
    }

    Ok(Args { query, dialect })
}
