
#[fail(display = "Can not create an image")]
#[derive(Debug, Fail, Clone)]
pub struct CanNotCreateImage;

#[fail(display = "Can not open an image  \"{}\"", path)]
#[derive(Debug, Fail, Clone)]
pub struct CanNotOpenImage {
    path:String
}

impl CanNotOpenImage {
    pub fn new(path:String) -> Self {
        CanNotOpenImage {
            path
        }
    }
}

#[fail(display = "Can not save an image  \"{}\"", path)]
#[derive(Debug, Fail, Clone)]
pub struct CanNotSaveImage {
    path:String
}

impl CanNotSaveImage {
    pub fn new(path:String) -> Self {
        CanNotSaveImage {
            path
        }
    }
}

#[fail(display = "Can not resize an image")]
#[derive(Debug, Fail, Clone)]
pub struct CanNotResizeImage;