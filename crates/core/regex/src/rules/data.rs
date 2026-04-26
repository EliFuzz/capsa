use std::ffi::CStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rule {
    pub id: u32,
    pub message: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/rules.rs"));
