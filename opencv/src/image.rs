
use failure::Error;

use std::ffi::{CString, CStr};
use libc::c_int;

use opencv_sys::image::Mat;

pub use opencv_sys::image::Size;

use crate::errors::CanNotCreateImage;
use crate::errors::CanNotOpenImage;
use crate::errors::CanNotSaveImage;
use crate::errors::CanNotResizeImage;

pub struct Image {
    mat:Mat
}

impl Image {
    pub fn create(size:Size, depth:i32, channels: i32) -> Result<Self, Error> {
        let mat = unsafe {
            opencv_sys::image::cvCreateImage ( size, depth, channels )
        };

        if mat==0 as Mat {
            let error = unsafe{opencv_sys::error::cvGetErrStatus()};

            //TODO

            bail!(CanNotCreateImage);
        }

        let image = Image {
            mat
        };

        Ok(image)
    }

    pub fn open(path:&str) -> Result<Self, Error> {//TODO options
        let mat = unsafe {
            opencv_sys::image::cvLoadImage(CString::new(path).unwrap().as_ptr(), 1)
        };

        if mat==0 as Mat {
            let error = unsafe{opencv_sys::error::cvGetErrStatus()};

            //TODO

            bail!(CanNotOpenImage);
        }

        let image = Image {
            mat
        };

        Ok(image)
    }

    pub fn save(&self, path:&str) -> Result<(), Error> {//TODO options
        let result = unsafe {
            opencv_sys::image::cvSaveImage(CString::new(path).unwrap().as_ptr(), self.mat, 0 as *const c_int)
        };

        if result==0 {
            let error = unsafe{opencv_sys::error::cvGetErrStatus()};

            //TODO

            bail!(CanNotSaveImage);
        }

        Ok(())
    }

    pub fn resize(&self, into:Self) -> Result<Self, Error> {//TODO options
        unsafe {
            opencv_sys::image::cvResize(self.mat, into.mat, 1);
        }

        Ok(into)
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        //TODO
    }
}