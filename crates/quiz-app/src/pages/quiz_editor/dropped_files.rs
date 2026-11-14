//! Capture entries synchronously: DataTransfer becomes unreadable after the drop handler returns.
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

pub(super) struct DropFiles {
    entries: Vec<JsValue>,
    fallback: Vec<web_sys::File>,
}
fn property(value: &JsValue, name: &str) -> JsValue {
    js_sys::Reflect::get(value, &name.into()).unwrap_or_default()
}
fn call(value: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    property(value, name)
        .dyn_into::<js_sys::Function>()?
        .call0(value)
}
pub(super) fn capture(transfer: &web_sys::DataTransfer) -> DropFiles {
    let items = property(transfer.as_ref(), "items");
    let length = property(&items, "length").as_f64().unwrap_or(0.) as u32;
    let entries = (0..length)
        .filter_map(|i| {
            let item = js_sys::Reflect::get(&items, &i.into()).ok()?;
            call(&item, "webkitGetAsEntry")
                .ok()
                .filter(|e| !e.is_null() && !e.is_undefined())
        })
        .collect();
    let fallback = transfer
        .files()
        .map(|files| (0..files.length()).filter_map(|i| files.get(i)).collect())
        .unwrap_or_default();
    DropFiles { entries, fallback }
}
async fn callback(value: &JsValue, method: &str) -> Result<JsValue, JsValue> {
    let function = property(value, method).dyn_into::<js_sys::Function>()?;
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        if let Err(error) = function.call2(value, &resolve, &reject) {
            let _ = reject.call1(&JsValue::NULL, &error);
        }
    });
    JsFuture::from(promise).await
}
pub(super) async fn collect(captured: DropFiles) -> Result<Vec<web_sys::File>, String> {
    if captured.entries.is_empty() {
        return Ok(captured.fallback);
    }
    let mut pending = captured.entries;
    let mut files = vec![];
    while let Some(entry) = pending.pop() {
        if property(&entry, "isFile").as_bool() == Some(true) {
            let file = callback(&entry, "file")
                .await
                .map_err(|e| format!("{e:?}"))?
                .dyn_into::<web_sys::File>()
                .map_err(|e| format!("{e:?}"))?;
            files.push(file);
        } else if property(&entry, "isDirectory").as_bool() == Some(true) {
            let reader = call(&entry, "createReader").map_err(|e| format!("{e:?}"))?;
            loop {
                let batch = callback(&reader, "readEntries")
                    .await
                    .map_err(|e| format!("{e:?}"))?;
                let batch = js_sys::Array::from(&batch);
                if batch.length() == 0 {
                    break;
                }
                pending.extend(batch.iter());
            }
        }
    }
    Ok(files)
}
