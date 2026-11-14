//! GIS OAuth token model. No refresh tokens; localStorage is shared across windows.
use super::Error;
use super::*;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, closure::Closure};
use wasm_bindgen_futures::{JsFuture, spawn_local};
pub const STORAGE_KEY: &str = "quiznight.google";
pub const INVITE_KEY: &str = "quiznight.driveInvite";
pub const SCOPE: &str = "https://www.googleapis.com/auth/drive";
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub email: String,
    pub photo: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub token: Option<String>,
    pub expires: f64,
    pub user: User,
}
#[derive(Clone)]
pub struct GoogleAccount {
    pub session: RwSignal<Option<Session>>,
    pub signed_in: RwSignal<bool>,
}
thread_local! {
 static SESSION:RefCell<Option<Session>>=RefCell::new(read_session());
 static CLIENT:RefCell<JsValue>=const{RefCell::new(JsValue::UNDEFINED)};
 static LOAD:RefCell<Option<js_sys::Promise>>=const{RefCell::new(None)};
 static PENDING:RefCell<Option<(js_sys::Function,js_sys::Function)>>=const{RefCell::new(None)};
 static LISTENERS:RefCell<Vec<(u64,Rc<dyn Fn()>)>>=const{RefCell::new(vec![])};
 static NEXT:RefCell<u64>=const{RefCell::new(0)};
 static STORAGE_LISTENER:RefCell<Option<Closure<dyn FnMut(web_sys::StorageEvent)>>>=const{RefCell::new(None)};
}
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}
fn read_session() -> Option<Session> {
    serde_json::from_str(&storage()?.get_item(STORAGE_KEY).ok()??).ok()
}
pub fn google_enabled() -> bool {
    !config::GOOGLE_CLIENT_ID.is_empty()
}
pub fn has_drive_invite() -> bool {
    google_enabled()
        && (storage()
            .and_then(|s| s.get_item(INVITE_KEY).ok().flatten())
            .as_deref()
            == Some("1")
            || read_session().is_some())
}
pub fn accept_drive_invite() {
    if let Some(s) = storage() {
        let _ = s.set_item(INVITE_KEY, "1");
    }
}
pub fn get_session() -> Option<Session> {
    SESSION.with(|s| s.borrow().clone())
}
pub fn current_token() -> Option<String> {
    get_session()
        .filter(|s| s.expires - now() > 60_000.)
        .and_then(|s| s.token)
        .filter(|s| !s.is_empty())
}
fn notify() {
    let callbacks = LISTENERS.with(|s| {
        s.borrow()
            .iter()
            .map(|(_, f)| f.clone())
            .collect::<Vec<_>>()
    });
    for f in callbacks {
        f();
    }
}
pub(super) fn write_session(session: Option<Session>) {
    if let Some(s) = storage() {
        if let Some(v) = &session {
            let _ = s.set_item(STORAGE_KEY, &serde_json::to_string(v).unwrap());
        } else {
            let _ = s.remove_item(STORAGE_KEY);
        }
    }
    SESSION.with(|s| *s.borrow_mut() = session);
    notify();
}
pub struct Subscription(u64);
impl Drop for Subscription {
    fn drop(&mut self) {
        LISTENERS.with(|s| s.borrow_mut().retain(|(id, _)| *id != self.0));
    }
}
pub fn subscribe_google(f: impl Fn() + 'static) -> Subscription {
    STORAGE_LISTENER.with(|listener| {
        if listener.borrow().is_none() {
            let cb =
                Closure::<dyn FnMut(web_sys::StorageEvent)>::new(|e: web_sys::StorageEvent| {
                    if e.key().as_deref() == Some(STORAGE_KEY) {
                        SESSION.with(|s| *s.borrow_mut() = read_session());
                        notify();
                    }
                });
            if let Some(w) = web_sys::window() {
                let _ = w.add_event_listener_with_callback("storage", cb.as_ref().unchecked_ref());
            }
            *listener.borrow_mut() = Some(cb);
        }
    });
    let id = NEXT.with(|n| {
        *n.borrow_mut() += 1;
        *n.borrow()
    });
    LISTENERS.with(|s| s.borrow_mut().push((id, Rc::new(f))));
    Subscription(id)
}
fn member(v: &JsValue, k: &str) -> JsValue {
    js_sys::Reflect::get(v, &k.into()).unwrap_or_default()
}
fn oauth() -> JsValue {
    member(
        &member(&member(&web_sys::window().unwrap(), "google"), "accounts"),
        "oauth2",
    )
}
fn finish(result: Result<Session>) {
    if let Some((resolve, reject)) = PENDING.with(|p| p.borrow_mut().take()) {
        match result {
            Ok(s) => {
                let _ = resolve.call1(&JsValue::NULL, &js(&serde_json::to_value(s).unwrap()));
            }
            Err(e) => {
                let _ = reject.call1(&JsValue::NULL, &js(&serde_json::to_value(e).unwrap()));
            }
        }
    }
}
async fn receive_token(response: JsValue) -> Result<Session> {
    if let Some(error) = member(&response, "error").as_string() {
        return Err(Error::Auth(error));
    }
    let oauth = oauth();
    let granted = member(&oauth, "hasGrantedAllScopes")
        .dyn_into::<js_sys::Function>()?
        .call2(&oauth, &response, &SCOPE.into())?;
    if !granted.as_bool().unwrap_or(false) {
        return Err(Error::Auth("scope".into()));
    }
    let token = member(&response, "access_token")
        .as_string()
        .unwrap_or_default();
    let duration = member(&response, "expires_in");
    let duration = duration
        .as_f64()
        .or_else(|| duration.as_string().and_then(|s| s.parse().ok()))
        .filter(|n| *n != 0.)
        .unwrap_or(3600.);
    let expires = now() + duration * 1000.;
    let res = super::fetch::raw(
        "https://www.googleapis.com/drive/v3/about?fields=user(displayName,emailAddress,photoLink)",
        &super::fetch::Options::default(),
        Some(&token),
    )
    .await?;
    let profile = super::fetch::response_json(res).await?;
    let session = Session {
        token: Some(token),
        expires,
        user: User {
            name: string(&profile["user"], "displayName"),
            email: string(&profile["user"], "emailAddress"),
            photo: profile["user"]["photoLink"].as_str().map(str::to_owned),
        },
    };
    write_session(Some(session.clone()));
    Ok(session)
}
fn initialize_client() -> std::result::Result<(), JsValue> {
    let token = Closure::<dyn FnMut(JsValue)>::new(|response| {
        spawn_local(async move {
            finish(receive_token(response).await);
        });
    });
    let error = Closure::<dyn FnMut(JsValue)>::new(|error: JsValue| {
        finish(Err(Error::Auth(
            member(&error, "type")
                .as_string()
                .unwrap_or("popup_closed".into()),
        )));
    });
    let options = js(&json!({"client_id":config::GOOGLE_CLIENT_ID,"scope":SCOPE}));
    js_sys::Reflect::set(&options, &"callback".into(), token.as_ref())?;
    js_sys::Reflect::set(&options, &"error_callback".into(), error.as_ref())?;
    let oauth = oauth();
    let client = member(&oauth, "initTokenClient")
        .dyn_into::<js_sys::Function>()?
        .call1(&oauth, &options)?;
    CLIENT.with(|c| *c.borrow_mut() = client);
    token.forget();
    error.forget();
    Ok(())
}
pub async fn load_google() -> Result<bool> {
    if !google_enabled() {
        return Ok(false);
    }
    let cached = LOAD.with(|l| l.borrow().clone());
    let promise = cached.unwrap_or_else(|| {
        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            let setup_reject = reject.clone();
            let setup = (|| -> std::result::Result<(), JsValue> {
                let document = web_sys::window().unwrap().document().unwrap();
                let script = document
                    .create_element("script")?
                    .dyn_into::<web_sys::HtmlScriptElement>()?;
                script.set_src("https://accounts.google.com/gsi/client");
                script.set_async(true);
                let load_reject = reject.clone();
                let loaded = Closure::<dyn FnMut()>::new(move || match initialize_client() {
                    Ok(()) => {
                        let _ = resolve.call1(&JsValue::NULL, &JsValue::TRUE);
                    }
                    Err(e) => {
                        let _ = load_reject.call1(&JsValue::NULL, &e);
                    }
                });
                let failed = Closure::<dyn FnMut()>::new(move || {
                    let _ =
                        reject.call1(&JsValue::NULL, &"Google sign-in could not be loaded".into());
                });
                script.set_onload(Some(loaded.as_ref().unchecked_ref()));
                script.set_onerror(Some(failed.as_ref().unchecked_ref()));
                document.head().unwrap().append_child(&script)?;
                loaded.forget();
                failed.forget();
                Ok(())
            })();
            if let Err(e) = setup {
                let _ = setup_reject.call1(&JsValue::NULL, &e);
            }
        });
        LOAD.with(|l| *l.borrow_mut() = Some(promise.clone()));
        promise
    });
    match JsFuture::from(promise).await {
        Ok(_) => Ok(true),
        Err(e) => {
            LOAD.with(|l| *l.borrow_mut() = None);
            Err(Error::from(e))
        }
    }
}

/// Preload GIS before a click. When loaded, popup request executes synchronously on first poll.
pub async fn sign_in(choose_account: bool) -> Result<Session> {
    if CLIENT.with(|c| c.borrow().is_undefined()) {
        load_google().await?;
    }
    if let Some((_, reject)) = PENDING.with(|p| p.borrow_mut().take()) {
        let _ = reject.call1(&JsValue::NULL, &"superseded".into());
    }
    let p = js_sys::Promise::new(&mut |resolve, reject| {
        PENDING.with(|p| *p.borrow_mut() = Some((resolve, reject)));
        let hint = if choose_account {
            None
        } else {
            get_session().map(|s| s.user.email)
        };
        let mut options = json!({"prompt":if hint.is_some(){""}else{"select_account"}});
        if let Some(h) = hint {
            options["login_hint"] = json!(h);
        }
        let c = CLIENT.with(|c| c.borrow().clone());
        if let Err(e) = member(&c, "requestAccessToken")
            .dyn_into::<js_sys::Function>()
            .and_then(|f| f.call1(&c, &js(&options)))
        {
            finish(Err(Error::from(e)));
        }
    });
    let v = JsFuture::from(p).await.map_err(|e| {
        let fallback = e.as_string().unwrap_or_default();
        value(e)
            .ok()
            .and_then(|v| serde_json::from_value::<Error>(v).ok())
            .unwrap_or(Error::Auth(fallback))
    })?;
    serde_json::from_value(value(v)?).map_err(|e| Error::Other(e.to_string()))
}
pub fn sign_out() {
    if let Some(token) = get_session().and_then(|s| s.token) {
        let o = oauth();
        if let Ok(f) = member(&o, "revoke").dyn_into::<js_sys::Function>() {
            let cb = Closure::<dyn FnMut()>::once(|| {});
            let _ = f.call2(&o, &token.into(), cb.as_ref());
            cb.forget();
        }
    }
    write_session(None);
}
pub fn provide_google_account() -> GoogleAccount {
    let account = GoogleAccount {
        session: RwSignal::new(get_session()),
        signed_in: RwSignal::new(current_token().is_some()),
    };
    let update = account.clone();
    let timer = Rc::new(RefCell::new(None::<gloo_timers::callback::Timeout>));
    let refresh = {
        let timer = timer.clone();
        move || {
            update.session.set(get_session());
            update.signed_in.set(current_token().is_some());
            timer.borrow_mut().take();
            if let Some(s) = get_session().filter(|_| current_token().is_some()) {
                let signal = update.signed_in;
                *timer.borrow_mut() = Some(gloo_timers::callback::Timeout::new(
                    (s.expires - now() - 60_000. + 500.)
                        .max(0.)
                        .min(u32::MAX as f64) as u32,
                    move || signal.set(false),
                ));
            }
        }
    };
    refresh();
    let sub = subscribe_google(refresh);
    on_cleanup(move || drop(sub));
    let timer = StoredValue::new_local(timer);
    on_cleanup(move || {
        timer.update_value(|t| {
            t.borrow_mut().take();
        });
    });
    provide_context(account.clone());
    spawn_local(async {
        let _ = load_google().await;
    });
    account
}
pub fn use_google_account() -> GoogleAccount {
    expect_context()
}
