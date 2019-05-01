
use libc::{c_void, c_char, size_t, c_int};

#[link(name = "target/debug/opencv_world340")]
extern "C"{
    pub fn cvGetErrStatus() -> c_int;
}