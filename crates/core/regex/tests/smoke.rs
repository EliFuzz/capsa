use core_regex::{
    HS_FLAG_DOTALL, HS_MODE_BLOCK, HS_SUCCESS, hs_alloc_scratch, hs_compile, hs_database_t,
    hs_free_database, hs_free_scratch, hs_scan, hs_scratch_t,
};
use std::ffi::{CString, c_char, c_uint, c_ulonglong, c_void};
use std::ptr;

unsafe extern "C" fn on_match(
    _id: c_uint,
    _from: c_ulonglong,
    _to: c_ulonglong,
    _flags: c_uint,
    ctx: *mut c_void,
) -> std::os::raw::c_int {
    unsafe {
        *ctx.cast::<u32>() += 1;
    }
    0
}

#[test]
fn compile_and_scan_block() {
    let pattern = CString::new("fo+").unwrap();
    let mut db: *mut hs_database_t = ptr::null_mut();
    let mut compile_err: *mut core_regex::hs_compile_error_t = ptr::null_mut();

    let rc = unsafe {
        hs_compile(
            pattern.as_ptr().cast::<c_char>(),
            HS_FLAG_DOTALL,
            HS_MODE_BLOCK,
            ptr::null(),
            &mut db,
            &mut compile_err,
        )
    };
    assert_eq!(rc as u32, HS_SUCCESS, "hs_compile failed");
    assert!(!db.is_null());

    let mut scratch: *mut hs_scratch_t = ptr::null_mut();
    let rc = unsafe { hs_alloc_scratch(db, &mut scratch) };
    assert_eq!(rc as u32, HS_SUCCESS);

    let haystack = b"bar foo foo baz";
    let mut matches: u32 = 0;
    let rc = unsafe {
        hs_scan(
            db,
            haystack.as_ptr().cast::<c_char>(),
            haystack.len() as c_uint,
            0,
            scratch,
            Some(on_match),
            ptr::from_mut(&mut matches).cast::<c_void>(),
        )
    };
    assert_eq!(rc as u32, HS_SUCCESS);
    assert!(matches >= 2, "expected at least 2 matches, got {matches}");

    unsafe {
        hs_free_scratch(scratch);
        hs_free_database(db);
    }
}
