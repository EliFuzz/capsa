use crate::rules::{RULES, VECTORSCAN_PATTERN_FLAGS, VECTORSCAN_PATTERN_IDS, VECTORSCAN_PATTERNS};
use crate::{
    HS_MODE_BLOCK, HS_SUCCESS, hs_alloc_scratch, hs_compile_multi, hs_database_t,
    hs_free_compile_error, hs_free_database, hs_free_scratch, hs_scan, hs_scratch_t,
};
use std::cell::RefCell;
use std::ffi::{CStr, c_char, c_uint, c_ulonglong, c_void};
use std::ptr;
use std::sync::OnceLock;

struct RuleDatabase(*mut hs_database_t);

unsafe impl Send for RuleDatabase {}
unsafe impl Sync for RuleDatabase {}

impl Drop for RuleDatabase {
    fn drop(&mut self) {
        unsafe {
            hs_free_database(self.0);
        }
    }
}

struct Scratch(*mut hs_scratch_t);

impl Drop for Scratch {
    fn drop(&mut self) {
        unsafe {
            hs_free_scratch(self.0);
        }
    }
}

struct MatchContext {
    errors: Option<Vec<&'static str>>,
}

static DATABASE: OnceLock<Result<RuleDatabase, String>> = OnceLock::new();

thread_local! {
    static SCRATCH: RefCell<Option<Scratch>> = const { RefCell::new(None) };
}

pub fn validate_sql_query_rules(sql: &str) -> Option<Vec<&'static str>> {
    if sql.is_empty() {
        return None;
    }

    let database = rule_database();
    let length =
        c_uint::try_from(sql.len()).expect("SQL query length exceeds Vectorscan block scan limit");
    let mut context = MatchContext { errors: None };

    let rc = with_scratch(database.0, |scratch| unsafe {
        hs_scan(
            database.0,
            sql.as_ptr().cast::<c_char>(),
            length,
            0,
            scratch,
            Some(on_match),
            ptr::from_mut(&mut context).cast::<c_void>(),
        )
    });

    assert_eq!(rc, HS_SUCCESS as i32, "hs_scan failed");
    context.errors
}

fn rule_database() -> &'static RuleDatabase {
    DATABASE
        .get_or_init(compile_database)
        .as_ref()
        .unwrap_or_else(|error| panic!("failed to compile SQL rule database: {error}"))
}

fn with_scratch<T>(database: *const hs_database_t, scan: impl FnOnce(*mut hs_scratch_t) -> T) -> T {
    SCRATCH.with(|scratch| {
        let mut scratch = scratch.borrow_mut();
        if scratch.is_none() {
            *scratch = Some(alloc_scratch(database));
        }

        scan(scratch.as_ref().expect("scratch was initialized").0)
    })
}

fn alloc_scratch(database: *const hs_database_t) -> Scratch {
    let mut scratch: *mut hs_scratch_t = ptr::null_mut();
    let rc = unsafe { hs_alloc_scratch(database, &mut scratch) };
    assert_eq!(rc, HS_SUCCESS as i32, "hs_alloc_scratch failed");
    assert!(!scratch.is_null(), "hs_alloc_scratch returned null scratch");
    Scratch(scratch)
}

fn compile_database() -> Result<RuleDatabase, String> {
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

    if rc == HS_SUCCESS as i32 && !database.is_null() {
        return Ok(RuleDatabase(database));
    }

    let message = compile_error_message(compile_error);
    if !compile_error.is_null() {
        unsafe {
            hs_free_compile_error(compile_error);
        }
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
    if let Some(rule) = RULES.get(id as usize) {
        context
            .errors
            .get_or_insert_with(Vec::new)
            .push(rule.message);
    }
    0
}
