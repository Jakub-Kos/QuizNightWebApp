//! Cheap app-wide storage handle, notifications and browser media leases.
use leptos::prelude::*;
use quiz_core::quiz::model::Quiz;
use quiz_store::{LibraryTheme, Media, MediaStore, MemoryStore, QuizStore, Result, ThemeStore};
use std::rc::Rc;
use wasm_bindgen::{JsCast, closure::Closure};

enum Backend {
    #[cfg(target_arch = "wasm32")]
    Indexed(quiz_store::IndexedDbStore),
    Memory(MemoryStore),
}
struct Inner {
    backend: Backend,
    revision: RwSignal<u64>,
    channel: Option<web_sys::BroadcastChannel>,
    _listener: Closure<dyn FnMut(web_sys::MessageEvent)>,
    focus: Closure<dyn FnMut(web_sys::Event)>,
}
impl Drop for Inner {
    fn drop(&mut self) {
        let _ = window()
            .remove_event_listener_with_callback("focus", self.focus.as_ref().unchecked_ref());
        if let Some(c) = &self.channel {
            c.set_onmessage(None);
            c.close();
        }
    }
}
#[derive(Clone)]
pub struct Store(Rc<Inner>);
impl Store {
    pub fn is_persistent(&self) -> bool {
        !matches!(self.0.backend, Backend::Memory(_))
    }
    pub fn revision(&self) -> RwSignal<u64> {
        self.0.revision
    }
    fn changed(&self) {
        self.0.revision.update(|n| *n += 1);
        if let Some(c) = &self.0.channel {
            let _ = c.post_message(&wasm_bindgen::JsValue::TRUE);
        }
    }
}
pub async fn open() -> Store {
    #[cfg(target_arch = "wasm32")]
    let backend = match quiz_store::IndexedDbStore::open().await {
        Ok(db) => Backend::Indexed(db),
        Err(e) => {
            web_sys::console::warn_1(&e.to_string().into());
            Backend::Memory(MemoryStore::new())
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    let backend = Backend::Memory(MemoryStore::new());
    let revision = RwSignal::new(0);
    let channel = web_sys::BroadcastChannel::new("quiznight-storage").ok();
    let listener =
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |_| revision.update(|n| *n += 1));
    if let Some(c) = &channel {
        c.set_onmessage(Some(listener.as_ref().unchecked_ref()));
    }
    let focus = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| revision.update(|n| *n += 1));
    let _ = window().add_event_listener_with_callback("focus", focus.as_ref().unchecked_ref());
    let store = Store(Rc::new(Inner {
        backend,
        revision,
        channel,
        _listener: listener,
        focus,
    }));
    request_persistence();
    // Seed only once; never replace settings saved on an existing demo record.
    if matches!(store.get_quiz("demo").await, Ok(None))
        && let Ok(context) = crate::quiz_context::load_demo_seed().await
    {
        let mut quiz = (*context.quiz).clone();
        quiz.created_at = now_ms();
        let _ = store.put_quiz(&quiz).await;
    }
    store
}
pub fn provide_store(store: Store) {
    provide_context(StoredValue::new_local(store));
}
pub fn use_store() -> Store {
    expect_context::<StoredValue<Store, leptos::prelude::LocalStorage>>().get_value()
}
pub fn now_ms() -> i64 {
    js_sys::Date::now() as i64
}
pub fn new_id() -> String {
    // crypto.randomUUID where available; getRandomValues also works on insecure LAN origins.
    let crypto = js_sys::Reflect::get(&window(), &"crypto".into()).unwrap_or_default();
    let bytes = js_sys::Uint8Array::new_with_length(16);
    if let Ok(f) = js_sys::Reflect::get(&crypto, &"getRandomValues".into())
        .and_then(|v| v.dyn_into::<js_sys::Function>())
    {
        let _ = f.call1(&crypto, &bytes);
    }
    let mut b = bytes.to_vec();
    if b.iter().all(|b| *b == 0) {
        for byte in &mut b {
            *byte = (js_sys::Math::random() * 256.0) as u8;
        }
    }
    b[6] = (b[6] & 15) | 64;
    b[8] = (b[8] & 63) | 128;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0],
        b[1],
        b[2],
        b[3],
        b[4],
        b[5],
        b[6],
        b[7],
        b[8],
        b[9],
        b[10],
        b[11],
        b[12],
        b[13],
        b[14],
        b[15]
    )
}
impl QuizStore for Store {
    async fn list_quizzes(&self) -> Result<Vec<Quiz>> {
        match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.list_quizzes().await,
            Backend::Memory(s) => s.list_quizzes().await,
        }
    }
    async fn get_quiz(&self, id: &str) -> Result<Option<Quiz>> {
        match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.get_quiz(id).await,
            Backend::Memory(s) => s.get_quiz(id).await,
        }
    }
    async fn put_quiz(&self, quiz: &Quiz) -> Result<Quiz> {
        let result = match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.put_quiz(quiz).await,
            Backend::Memory(s) => s.put_quiz(quiz).await,
        };
        if result.is_ok() {
            self.changed();
        }
        result
    }
    async fn update_quiz<F: FnOnce(&mut Quiz)>(&self, id: &str, change: F) -> Result<Option<Quiz>> {
        let result = match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.update_quiz(id, change).await,
            Backend::Memory(s) => s.update_quiz(id, change).await,
        };
        if result.is_ok() {
            self.changed();
        }
        result
    }
    async fn delete_quiz(&self, id: &str) -> Result<()> {
        let result = match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.delete_quiz(id).await,
            Backend::Memory(s) => s.delete_quiz(id).await,
        };
        if result.is_ok() {
            self.changed();
        }
        result
    }
}
impl MediaStore for Store {
    async fn list_media(&self, quiz_id: &str) -> Result<Vec<Media>> {
        match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.list_media(quiz_id).await,
            Backend::Memory(s) => s.list_media(quiz_id).await,
        }
    }
    async fn get_media(&self, quiz_id: &str, name: &str) -> Result<Option<Media>> {
        match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.get_media(quiz_id, name).await,
            Backend::Memory(s) => s.get_media(quiz_id, name).await,
        }
    }
    async fn put_media(&self, quiz_id: &str, files: &[Media]) -> Result<()> {
        let result = match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.put_media(quiz_id, files).await,
            Backend::Memory(s) => s.put_media(quiz_id, files).await,
        };
        if result.is_ok() {
            self.changed();
        }
        result
    }
    async fn delete_media(&self, quiz_id: &str, name: &str) -> Result<()> {
        let result = match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.delete_media(quiz_id, name).await,
            Backend::Memory(s) => s.delete_media(quiz_id, name).await,
        };
        if result.is_ok() {
            self.changed();
        }
        result
    }
}
impl ThemeStore for Store {
    async fn list_themes(&self) -> Result<Vec<LibraryTheme>> {
        match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.list_themes().await,
            Backend::Memory(s) => s.list_themes().await,
        }
    }
    async fn get_theme(&self, id: &str) -> Result<Option<LibraryTheme>> {
        match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.get_theme(id).await,
            Backend::Memory(s) => s.get_theme(id).await,
        }
    }
    async fn put_theme(&self, theme: &LibraryTheme) -> Result<LibraryTheme> {
        let result = match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.put_theme(theme).await,
            Backend::Memory(s) => s.put_theme(theme).await,
        };
        if result.is_ok() {
            self.changed();
        }
        result
    }
    async fn delete_theme(&self, id: &str) -> Result<()> {
        let result = match &self.0.backend {
            #[cfg(target_arch = "wasm32")]
            Backend::Indexed(s) => s.delete_theme(id).await,
            Backend::Memory(s) => s.delete_theme(id).await,
        };
        if result.is_ok() {
            self.changed();
        }
        result
    }
}

/// Object URLs live as long as the loaded quiz (including scene copies).
#[derive(Debug, Default)]
pub struct MediaUrls(pub std::collections::BTreeMap<String, String>);
impl Drop for MediaUrls {
    fn drop(&mut self) {
        for url in self.0.values() {
            let _ = web_sys::Url::revoke_object_url(url);
        }
    }
}
pub fn blob(bytes: &[u8], mime: &str) -> std::result::Result<web_sys::Blob, String> {
    let parts = js_sys::Array::new();
    parts.push(&js_sys::Uint8Array::from(bytes));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(mime);
    web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options)
        .map_err(|e| format!("{e:?}"))
}
pub async fn media_urls(store: &Store, id: &str) -> std::result::Result<MediaUrls, String> {
    let mut urls = MediaUrls::default();
    for media in store.list_media(id).await.map_err(|e| e.to_string())? {
        let url =
            web_sys::Url::create_object_url_with_blob(&blob(&media.bytes, &media.content_type)?)
                .map_err(|e| format!("{e:?}"))?;
        if let Some(old) = urls
            .0
            .insert(quiz_core::quiz::parse::media_key(&media.name), url)
        {
            let _ = web_sys::Url::revoke_object_url(&old);
        }
    }
    Ok(urls)
}
/// Demo media is shipped separately; fetch it before package export/copy.
pub async fn export_quiz(store: &Store, quiz: &Quiz) -> std::result::Result<Vec<u8>, String> {
    if quiz.builtin {
        let temp = MemoryStore::new();
        let mut media = Vec::new();
        for name in quiz_store::package::referenced_media(quiz) {
            let response = gloo_net::http::Request::get(&format!("source/{name}"))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if !response.ok()
                || response
                    .headers()
                    .get("content-type")
                    .is_some_and(|t| t.contains("text/html"))
            {
                return Err(format!("Missing demo media: {name}"));
            }
            media.push(Media {
                content_type: response.headers().get("content-type").unwrap_or_default(),
                bytes: response.binary().await.map_err(|e| e.to_string())?,
                name,
                ..Default::default()
            });
        }
        temp.put_media(&quiz.id, &media)
            .await
            .map_err(|e| e.to_string())?;
        quiz_store::package::export_zip(&temp, quiz)
            .await
            .map_err(|e| e.to_string())
    } else {
        quiz_store::package::export_zip(store, quiz)
            .await
            .map_err(|e| e.to_string())
    }
}

fn request_persistence() {
    // Best effort, including browsers which do not expose StorageManager.
    let _ = js_sys::Function::new_no_args(
        "try { navigator.storage?.persist?.().catch(() => {}); } catch {}",
    )
    .call0(&wasm_bindgen::JsValue::NULL);
}
