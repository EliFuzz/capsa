#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::doc_markdown)]

#[cfg(feature = "gen")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(not(feature = "gen"))]
include!("bindings.rs");

mod validate;

pub use validate::validate_sql_query_rules;

pub mod rules;
