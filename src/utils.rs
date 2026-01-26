use base64::prelude::*;
use js_sys::wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Blob, CompressionFormat, CompressionStream, DecompressionStream, HtmlAnchorElement,
    ReadableWritablePair, Response, Url, window,
};

pub fn download_text_file(filename: &str, content: &str) {
    let document = window().unwrap().document().unwrap();
    let body = document.body().unwrap();

    let parts = js_sys::Array::of1(&JsValue::from_str(content));
    let blob = Blob::new_with_str_sequence(parts.as_ref()).unwrap();

    let url = Url::create_object_url_with_blob(&blob).unwrap();

    let a = document
        .create_element("a")
        .unwrap()
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();

    a.set_href(&url);
    a.set_download(filename);
    body.append_child(&a).unwrap();
    a.click();
    body.remove_child(&a).unwrap();
    Url::revoke_object_url(&url).unwrap();
}

pub fn blob_from_bytes(data: &[u8]) -> Blob {
    let arr = js_sys::Uint8Array::from(data);
    let parts = js_sys::Array::of1(&arr);
    Blob::new_with_u8_array_sequence(parts.as_ref()).unwrap()
}
pub fn blob_from_str(s: &str) -> Blob {
    let parts = js_sys::Array::of1(&JsValue::from_str(s));
    Blob::new_with_str_sequence(parts.as_ref()).unwrap()
}

// WARN: unstable api
pub async fn compress_gz(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    let stream = CompressionStream::new(CompressionFormat::Gzip)?;
    let rw_pair = ReadableWritablePair::new(&stream.readable(), &stream.writable());
    let blob = blob_from_bytes(data);
    let compressed = blob.stream().pipe_through(&rw_pair);
    let response = Response::new_with_opt_readable_stream(Some(&compressed))?;
    let buffer = JsFuture::from(response.array_buffer()?).await?;
    let arr = js_sys::Uint8Array::new(&buffer);
    Ok(arr.to_vec())
}
pub async fn decompress_gz(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    let stream = DecompressionStream::new(CompressionFormat::Gzip)?;
    let rw_pair = ReadableWritablePair::new(&stream.readable(), &stream.writable());
    let blob = blob_from_bytes(data);
    let decompressed = blob.stream().pipe_through(&rw_pair);
    let response = Response::new_with_opt_readable_stream(Some(&decompressed))?;
    let buffer = JsFuture::from(response.array_buffer()?).await?;
    let arr = js_sys::Uint8Array::new(&buffer);
    Ok(arr.to_vec())
}

pub async fn base64_gz_from_str(s: &str) -> Result<String, JsValue> {
    let bytes = s.as_bytes();
    let compressed = compress_gz(bytes).await?;
    Ok(BASE64_URL_SAFE.encode(compressed))
}
pub async fn str_from_base64_gz(base64: &str) -> Result<String, JsValue> {
    let bytes = BASE64_URL_SAFE.decode(base64).unwrap();
    let decompressed = decompress_gz(&bytes).await?;
    Ok(String::from_utf8_lossy(&decompressed).to_string())
}
