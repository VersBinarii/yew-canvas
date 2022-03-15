use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    window, Blob, ImageBitmap, Request, RequestInit, RequestMode, Response,
};

/// Something wrong has occurred while fetching an external resource.
#[derive(Debug, Clone, PartialEq)]
pub struct FetchError {
    err: JsValue,
}
impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}
impl Error for FetchError {}

impl From<JsValue> for FetchError {
    fn from(value: JsValue) -> Self {
        Self { err: value }
    }
}

pub async fn fetch_image(file_path: &str) -> Result<ImageBitmap, FetchError> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(file_path, &opts)?;
    let window = window().unwrap();
    let resp_value =
        JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    let blob: Blob = JsFuture::from(resp.blob()?).await?.dyn_into()?;

    let image_bitmap_promise = window.create_image_bitmap_with_blob(&blob)?;
    gloo::console::log!("{}", &image_bitmap_promise);
    let image_bitmap: ImageBitmap =
        JsFuture::from(image_bitmap_promise).await?.dyn_into()?;
    Ok(image_bitmap)
}
