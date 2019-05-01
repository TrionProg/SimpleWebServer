#[macro_use]
extern crate failure_derive;

pub mod image_api;

pub mod errors;

pub use crate::image_api::{ImageApi, ImageApiRef};
pub use crate::image_api::PutImageInput;