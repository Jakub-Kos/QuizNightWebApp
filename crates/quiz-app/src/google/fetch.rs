//! All authenticated writes pass the guard. Raw fetch is for GIS profile/public CSV only.
use super::*;
use quiz_core::quiz::write_guard::{Kind, WriteGuard};
use quiz_store::QuizStore;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
#[derive(Default, Clone)]
pub struct Options {
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub keepalive: bool,
    pub no_store: bool,
}
impl Options {
    pub fn json(method: &str, body: &Value) -> Self {
        Self {
            method: method.into(),
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: Some(body.to_string()),
            ..Default::default()
        }
    }
}
thread_local! {static GUARD:Rc<RefCell<WriteGuard>>=Rc::new(RefCell::new(WriteGuard::default()));static CREATED:RefCell<Vec<String>>=const{RefCell::new(vec![])};static BUSY:RefCell<bool>=const{RefCell::new(false)};static STORE:RefCell<Option<crate::store::Store>>=const{RefCell::new(None)};}
/// Required once after opening the browser store. Roots are reloaded for every unknown write target.
pub fn configure_store(store: crate::store::Store) {
    STORE.with(|s| *s.borrow_mut() = Some(store));
}
pub(crate) fn store() -> Result<crate::store::Store> {
    STORE
        .with(|s| s.borrow().clone())
        .ok_or(Error::Blocked("Google store is not configured".into()))
}
pub(super) async fn raw(
    url: &str,
    options: &Options,
    token: Option<&str>,
) -> Result<web_sys::Response> {
    let init = web_sys::RequestInit::new();
    init.set_method(if options.method.is_empty() {
        "GET"
    } else {
        &options.method
    });
    js_sys::Reflect::set(
        &init,
        &"keepalive".into(),
        &JsValue::from_bool(options.keepalive),
    )?;
    if options.no_store {
        init.set_cache(web_sys::RequestCache::NoStore);
    }
    if let Some(b) = &options.body {
        init.set_body(&JsValue::from_str(b));
    }
    let headers = web_sys::Headers::new()?;
    for (k, v) in &options.headers {
        headers.set(k, v)?;
    }
    if let Some(t) = token {
        headers.set("Authorization", &format!("Bearer {t}"))?;
    }
    init.set_headers(&headers);
    let req = web_sys::Request::new_with_str_and_init(url, &init)?;
    JsFuture::from(web_sys::window().unwrap().fetch_with_request(&req))
        .await
        .map_err(|_| Error::Api(0, "network".into()))?
        .dyn_into()
        .map_err(Error::from)
}
pub(super) async fn response_json(res: web_sys::Response) -> Result<Value> {
    value(JsFuture::from(res.json()?).await?)
}
async fn unchecked_json(url: &str, token: &str) -> Result<Value> {
    response_json(checked(raw(url, &Options::default(), Some(token)).await?).await?).await
}
async fn checked(res: web_sys::Response) -> Result<web_sys::Response> {
    if res.status() == 401 {
        if let Some(mut s) = auth::get_session() {
            s.token = None;
            auth::write_session(Some(s));
        }
        return Err(Error::Auth("expired".into()));
    }
    if !res.ok() {
        let status = res.status();
        let v = response_json(res).await.unwrap_or_default();
        return Err(Error::Api(
            status,
            v["error"]["message"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| format!("HTTP {status}")),
        ));
    }
    Ok(res)
}
struct Lock;
impl Drop for Lock {
    fn drop(&mut self) {
        BUSY.with(|b| *b.borrow_mut() = false);
    }
}
pub async fn google_fetch(url: &str, options: Options) -> Result<web_sys::Response> {
    let token = auth::current_token().ok_or(Error::Auth("signed_out".into()))?;
    let method = if options.method.is_empty() {
        "GET"
    } else {
        &options.method
    };
    let target = if method.eq_ignore_ascii_case("GET") || method.eq_ignore_ascii_case("HEAD") {
        None
    } else {
        // Serialize the short authorization walk; no RefCell borrow competes with another check.
        while BUSY.with(|b| *b.borrow()) {
            gloo_timers::future::TimeoutFuture::new(10).await;
        }
        BUSY.with(|b| *b.borrow_mut() = true);
        let _lock = Lock;
        let guard = GUARD.with(Clone::clone);
        let mut g = std::mem::take(&mut *guard.borrow_mut());
        let result = g
            .check(
                method,
                url,
                options.body.as_deref().unwrap_or(""),
                || async {
                    match store() {
                        Ok(s) => s
                            .list_quizzes()
                            .await
                            .unwrap_or_default()
                            .iter()
                            .flat_map(|q| {
                                quiz_core::quiz::write_guard::quiz_file_ids(
                                    &serde_json::to_value(q).unwrap(),
                                )
                            })
                            .collect(),
                        Err(_) => vec![],
                    }
                },
                |id| {
                    let token = token.clone();
                    async move {
                        unchecked_json(
                            &format!(
                                "{}/files/{id}?fields=parents&supportsAllDrives=true",
                                config::DRIVE_API
                            ),
                            &token,
                        )
                        .await
                        .ok()
                        .and_then(|v| v["parents"].as_array().cloned())
                        .unwrap_or_default()
                        .iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect()
                    }
                },
                || async {
                    unchecked_json(
                        &format!("{}/files/root?fields=id", config::DRIVE_API),
                        &token,
                    )
                    .await
                    .ok()
                    .and_then(|v| v["id"].as_str().map(str::to_owned))
                },
            )
            .await;
        CREATED.with(|ids| {
            for id in ids.borrow_mut().drain(..) {
                g.created(&id);
            }
        });
        *guard.borrow_mut() = g;
        result.map_err(Error::Blocked)?
    };
    let res = checked(raw(url, &options, Some(&token)).await?).await?;
    if target.is_some_and(|t| matches!(t.kind, Kind::Create | Kind::Folder)) {
        if let Ok(copy) = res.clone() {
            if let Ok(v) = response_json(copy).await {
                if let Some(id) = v["id"].as_str() {
                    if BUSY.with(|b| *b.borrow()) {
                        CREATED.with(|ids| ids.borrow_mut().push(id.into()));
                    } else {
                        GUARD.with(|g| g.borrow_mut().created(id));
                    }
                }
            }
        }
    }
    Ok(res)
}
pub async fn google_json(url: &str, options: Options) -> Result<Value> {
    response_json(google_fetch(url, options).await?).await
}
pub async fn post(url: &str, body: &Value) -> Result<Value> {
    google_json(url, Options::json("POST", body)).await
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn json_request() {
        let o = Options::json("PATCH", &json!({"appProperties":{"qn_editing_a":null}}));
        assert_eq!(o.method, "PATCH");
        assert_eq!(
            o.headers,
            vec![("Content-Type".into(), "application/json".into())]
        );
        assert_eq!(
            serde_json::from_str::<Value>(o.body.as_ref().unwrap()).unwrap(),
            json!({"appProperties":{"qn_editing_a":null}})
        );
        assert!(!o.keepalive);
    }
}
