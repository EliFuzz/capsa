use core_regex::rules::{
    RULES, VECTORSCAN_PATTERN_FLAGS, VECTORSCAN_PATTERN_IDS, VECTORSCAN_PATTERNS,
};
use core_regex::{
    HS_MODE_BLOCK, HS_SUCCESS, hs_compile_multi, hs_database_t, hs_free_compile_error,
    hs_free_database,
};
use std::ffi::{CStr, c_char};
use std::ptr;

#[test]
fn sql_rule_patterns_compile_with_vectorscan() {
    let expressions = VECTORSCAN_PATTERNS.map(|pattern| pattern.as_ptr().cast::<c_char>());
    let mut db: *mut hs_database_t = ptr::null_mut();
    let mut compile_err: *mut core_regex::hs_compile_error_t = ptr::null_mut();

    let rc = unsafe {
        hs_compile_multi(
            expressions.as_ptr(),
            VECTORSCAN_PATTERN_FLAGS.as_ptr(),
            VECTORSCAN_PATTERN_IDS.as_ptr(),
            expressions.len() as u32,
            HS_MODE_BLOCK,
            ptr::null(),
            &mut db,
            &mut compile_err,
        )
    };

    if rc as u32 != HS_SUCCESS {
        let message = compile_error_message(compile_err);
        let expression = compile_error_expression(compile_err);
        unsafe {
            hs_free_compile_error(compile_err);
        }
        panic!("{}: {}", rule_name(expression), message);
    }

    unsafe {
        hs_free_database(db);
    }
}

fn compile_error_message(error: *mut core_regex::hs_compile_error_t) -> String {
    if error.is_null() {
        return String::from("unknown compile error");
    }

    unsafe {
        CStr::from_ptr((*error).message)
            .to_string_lossy()
            .into_owned()
    }
}

fn compile_error_expression(error: *mut core_regex::hs_compile_error_t) -> Option<usize> {
    if error.is_null() {
        return None;
    }

    usize::try_from(unsafe { (*error).expression }).ok()
}

fn rule_name(expression: Option<usize>) -> String {
    let Some(index) = expression else {
        return String::from("unknown rule");
    };

    let Some(rule) = RULES.get(index) else {
        return format!("unknown rule {index}");
    };

    format!("{}:{}", rule.id, rule.message)
}
