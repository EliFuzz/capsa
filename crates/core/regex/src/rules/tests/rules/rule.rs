use crate::{
    hs_alloc_scratch, hs_compile_multi, hs_database_t, hs_free_compile_error, hs_free_scratch,
    hs_scan, hs_scratch_t, HS_MODE_BLOCK, HS_SUCCESS,
};
use std::ffi::{c_char, c_uint, c_ulonglong, c_void, CStr};
use std::ptr;
use std::sync::OnceLock;

use super::super::super::{
    RULES, VECTORSCAN_PATTERNS, VECTORSCAN_PATTERN_FLAGS, VECTORSCAN_PATTERN_IDS,
};

struct CompiledRuleDatabase(*mut hs_database_t);

unsafe impl Send for CompiledRuleDatabase {}
unsafe impl Sync for CompiledRuleDatabase {}

struct MatchContext {
    rule_id: u32,
    matched: bool,
}

static DATABASE: OnceLock<Result<CompiledRuleDatabase, String>> = OnceLock::new();

pub fn assert_rule_matches(rule_id: u32, input: &str) {
    assert!(
        rule_matches(rule_id, input),
        "expected rule {rule_id} to match {input:?}"
    );
}

pub fn assert_rule_rejects(rule_id: u32, input: &str) {
    assert!(
        !rule_matches(rule_id, input),
        "expected rule {rule_id} to reject {input:?}"
    );
}

fn rule_matches(rule_id: u32, input: &str) -> bool {
    assert!(
        RULES.get(rule_id as usize).is_some(),
        "unknown rule {rule_id}"
    );

    let database = DATABASE
        .get_or_init(compile_database)
        .as_ref()
        .unwrap_or_else(|error| panic!("failed to compile rule database: {error}"));

    let mut scratch: *mut hs_scratch_t = ptr::null_mut();
    let rc = unsafe { hs_alloc_scratch(database.0, &mut scratch) };
    assert_eq!(rc as u32, HS_SUCCESS, "hs_alloc_scratch failed");

    let mut context = MatchContext {
        rule_id,
        matched: false,
    };
    let rc = unsafe {
        hs_scan(
            database.0,
            input.as_ptr().cast::<c_char>(),
            input.len() as c_uint,
            0,
            scratch,
            Some(on_match),
            ptr::from_mut(&mut context).cast::<c_void>(),
        )
    };

    unsafe {
        hs_free_scratch(scratch);
    }

    assert_eq!(rc as u32, HS_SUCCESS, "hs_scan failed for rule {rule_id}");
    context.matched
}

fn compile_database() -> Result<CompiledRuleDatabase, String> {
    let expressions = VECTORSCAN_PATTERNS.map(|pattern| pattern.as_ptr().cast::<c_char>());
    let mut database: *mut hs_database_t = ptr::null_mut();
    let mut compile_error: *mut crate::hs_compile_error_t = ptr::null_mut();

    let rc = unsafe {
        hs_compile_multi(
            expressions.as_ptr(),
            VECTORSCAN_PATTERN_FLAGS.as_ptr(),
            VECTORSCAN_PATTERN_IDS.as_ptr(),
            expressions.len() as u32,
            HS_MODE_BLOCK,
            ptr::null(),
            &mut database,
            &mut compile_error,
        )
    };

    if rc as u32 == HS_SUCCESS {
        return Ok(CompiledRuleDatabase(database));
    }

    let message = compile_error_message(compile_error);
    unsafe {
        hs_free_compile_error(compile_error);
    }
    Err(message)
}

fn compile_error_message(error: *mut crate::hs_compile_error_t) -> String {
    if error.is_null() {
        return String::from("unknown compile error");
    }

    unsafe {
        CStr::from_ptr((*error).message)
            .to_string_lossy()
            .into_owned()
    }
}

unsafe extern "C" fn on_match(
    id: c_uint,
    _from: c_ulonglong,
    _to: c_ulonglong,
    _flags: c_uint,
    ctx: *mut c_void,
) -> std::os::raw::c_int {
    let context = unsafe { &mut *ctx.cast::<MatchContext>() };
    if id == context.rule_id {
        context.matched = true;
    }
    0
}

#[test]
fn simple_successful() {
    assert_rule_matches(0, r###"CREATE TABLE users (password VARCHAR(255));"###);
}

#[test]
fn simple_failed() {
    assert_rule_rejects(
        0,
        r###"SELECT order_id, customer_id, total_amount FROM orders WHERE status = 'open' AND total_amount > 100;"###,
    );
}

#[test]
fn complex_detects_rule() {
    assert_rule_matches(
        0,
        r###"CREATE TABLE public.accounts (id INT, secret TEXT, created_at TIMESTAMP);"###,
    );
}
