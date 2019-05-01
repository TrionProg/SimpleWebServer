
#[fail(display = "Can not create an image")] //TODO
#[derive(Debug, Fail, Clone)]
pub struct CanNotCreateImage;

#[fail(display = "Can not open an image")] //TODO
#[derive(Debug, Fail, Clone)]
pub struct CanNotOpenImage;

#[fail(display = "Can not save an image")] //TODO
#[derive(Debug, Fail, Clone)]
pub struct CanNotSaveImage;

#[fail(display = "Can not resize an image")] //TODO
#[derive(Debug, Fail, Clone)]
pub struct CanNotResizeImage;