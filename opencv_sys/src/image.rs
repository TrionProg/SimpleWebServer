

use libc::{c_void, c_char, size_t, c_int};

#[repr(C)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

pub type Mat = *mut c_void;

#[repr(C)]
pub struct cv_return_value_void_X {
    pub error_code: i32,
    pub error_msg: *const c_char,
    pub result: *mut c_void
}

//TODO cvGetErrStatus();

//#[link(name = "opencv_world346", kind = "static")]
#[link(name = "target/debug/opencv_world340")]
extern "C"{
    pub fn cvLoadImage(filename: *const c_char, flags: i32) -> Mat;
    //pub fn cv_imgcodecs_cv_imwrite_String_filename_Mat_img_VectorOfint_params(filename: *const c_char, flags: i32) -> Mat;// *mut c_void;
    //pub fn imread(filename: *const c_char, flags: i32) -> Mat;// *mut c_void;
    pub fn cvSaveImage(filename: *const c_char, arr: *const c_void, param: *const c_int) -> c_int;
    //pub fn cv_imgcodecs_cv_imread_String_filename_int_flags(filename: *const c_char, flags: i32) -> cv_return_value_void_X;
    //fn snappy_max_compressed_length(source_length: size_t) -> size_t;
    pub fn cvResize(src: *const c_void, dst: *const c_void, interpolation:c_int) -> c_void;
    pub fn cvCreateImage(size:Size, depth:c_int, channels:c_int) -> Mat;
}