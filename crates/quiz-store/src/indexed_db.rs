// Exact React schema (src/quiz/storage.js), no version changes:
// quiznight v1: quizzes keyPath "id", raw camelCase Quiz records;
// media keyPath "key", nonunique index "quizId" on "quizId";
// {key: `${quizId}/${name}`, quizId, name, blob: Blob, ...extra (driveId/driveModified)}.
// quiznight-library v1: themes keyPath "id";
// {id, theme, files: [{name, blob: Blob}], savedAt: milliseconds}.
use crate::*;
use js_sys::{Array, JSON, Object, Promise, Reflect, Uint8Array};
use serde::{Serialize, de::DeserializeOwned};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Blob, BlobPropertyBag, Event, IdbDatabase, IdbObjectStore, IdbObjectStoreParameters,
    IdbRequest, IdbTransaction, IdbTransactionMode,
};

fn err(value: JsValue) -> Error {
    Error::Storage(format!("{value:?}"))
}
fn get(object: &JsValue, key: &str) -> Result<JsValue> {
    Reflect::get(object, &key.into()).map_err(err)
}
fn set(object: &JsValue, key: &str, value: &JsValue) -> Result<()> {
    Reflect::set(object, &key.into(), value).map_err(err)?;
    Ok(())
}
fn encode<T: Serialize>(value: &T) -> Result<JsValue> {
    JSON::parse(&serde_json::to_string(value)?).map_err(err)
}
fn decode<T: DeserializeOwned>(value: &JsValue) -> Result<T> {
    let text = JSON::stringify(value)
        .map_err(err)?
        .as_string()
        .ok_or_else(|| Error::Storage("record is not JSON".into()))?;
    Ok(serde_json::from_str(&text)?)
}

// Keep callbacks alive only until completion/cancellation, then detach them. The Promise is
// registered immediately, before a request can finish or a transaction can auto-commit.
type EventCallback = Closure<dyn FnMut(Event)>;
struct Pending {
    promise: Promise,
    target: JsValue,
    callbacks: Vec<(&'static str, EventCallback)>,
}
impl Drop for Pending {
    fn drop(&mut self) {
        for (name, _) in &self.callbacks {
            let _ = Reflect::set(&self.target, &(*name).into(), &JsValue::NULL);
        }
    }
}
impl Pending {
    fn new(target: JsValue, transaction: bool) -> Self {
        let mut callbacks = Vec::new();
        let promise = Promise::new(&mut |resolve, reject| {
            for name in if transaction {
                vec!["oncomplete", "onerror", "onabort"]
            } else {
                vec!["onsuccess", "onerror"]
            } {
                let resolve = resolve.clone();
                let reject = reject.clone();
                let object = target.clone();
                let callback = Closure::wrap(Box::new(move |_: Event| {
                    if name == "onsuccess" || name == "oncomplete" {
                        let value = if name == "onsuccess" {
                            Reflect::get(&object, &"result".into()).unwrap_or(JsValue::UNDEFINED)
                        } else {
                            JsValue::UNDEFINED
                        };
                        let _ = resolve.call1(&JsValue::UNDEFINED, &value);
                    } else {
                        let reason =
                            Reflect::get(&object, &"error".into()).unwrap_or(JsValue::NULL);
                        let _ = reject.call1(&JsValue::UNDEFINED, &reason);
                    }
                }) as Box<dyn FnMut(Event)>);
                Reflect::set(&target, &name.into(), callback.as_ref())
                    .expect("IDB event handler property");
                callbacks.push((name, callback));
            }
        });
        Self {
            promise,
            target,
            callbacks,
        }
    }
    async fn wait(self) -> Result<JsValue> {
        JsFuture::from(self.promise.clone()).await.map_err(err)
    }
}
fn request(request: IdbRequest) -> Pending {
    Pending::new(request.into(), false)
}
struct Transaction {
    tx: IdbTransaction,
    done: Pending,
}
impl Transaction {
    fn new(db: &IdbDatabase, names: &[&str], write: bool) -> Result<Self> {
        let stores = Array::new();
        for name in names {
            stores.push(&(*name).into());
        }
        let tx = db
            .transaction_with_str_sequence_and_mode(
                &stores,
                if write {
                    IdbTransactionMode::Readwrite
                } else {
                    IdbTransactionMode::Readonly
                },
            )
            .map_err(err)?;
        let done = Pending::new(tx.clone().into(), true);
        Ok(Self { tx, done })
    }
    fn store(&self, name: &str) -> Result<IdbObjectStore> {
        self.tx.object_store(name).map_err(err)
    }
    // Abort on synchronous conversion/request errors as well as asynchronous request failures.
    async fn finish<T>(self, result: Result<T>) -> Result<T> {
        if result.is_err() {
            let _ = self.tx.abort();
        }
        let completed = self.done.wait().await;
        let value = result?;
        completed?;
        Ok(value)
    }
}

pub struct IndexedDbStore {
    db: IdbDatabase,
    library: IdbDatabase,
}
impl Drop for IndexedDbStore {
    fn drop(&mut self) {
        self.db.close();
        self.library.close();
    }
}
impl IndexedDbStore {
    /// Failure is returned to the caller, who may select MemoryStore. Never silently hides data.
    pub async fn open() -> Result<Self> {
        let db = open("quiznight", false).await?;
        match open("quiznight-library", true).await {
            Ok(library) => Ok(Self { db, library }),
            Err(error) => {
                db.close();
                Err(error)
            }
        }
    }
}
async fn open(name: &str, library: bool) -> Result<IdbDatabase> {
    let factory = web_sys::window()
        .ok_or_else(|| Error::Storage("window unavailable".into()))?
        .indexed_db()
        .map_err(err)?
        .ok_or_else(|| Error::Storage("IndexedDB unavailable".into()))?;
    let req = factory.open_with_u32(name, 1).map_err(err)?;
    let upgrade_req = req.clone();
    let upgrade = Closure::wrap(Box::new(move |_: Event| {
        let result = (|| -> Result<()> {
            let db: IdbDatabase = upgrade_req.result().map_err(err)?.dyn_into().map_err(err)?;
            let options = IdbObjectStoreParameters::new();
            options.set_key_path(&"id".into());
            if library {
                db.create_object_store_with_optional_parameters("themes", &options)
                    .map_err(err)?;
            } else {
                db.create_object_store_with_optional_parameters("quizzes", &options)
                    .map_err(err)?;
                options.set_key_path(&"key".into());
                let media = db
                    .create_object_store_with_optional_parameters("media", &options)
                    .map_err(err)?;
                media
                    .create_index_with_str("quizId", "quizId")
                    .map_err(err)?;
            }
            Ok(())
        })();
        if result.is_err()
            && let Some(tx) = upgrade_req.transaction()
        {
            let _ = tx.abort();
        }
    }) as Box<dyn FnMut(Event)>);
    req.set_onupgradeneeded(Some(upgrade.as_ref().unchecked_ref()));
    let value = request(req.clone().unchecked_into()).wait().await;
    req.set_onupgradeneeded(None);
    value?.dyn_into().map_err(err)
}

fn media_record(quiz_id: &str, media: &Media) -> Result<JsValue> {
    let value = encode(&media.fields)?;
    set(&value, "key", &format!("{quiz_id}/{}", media.name).into())?;
    set(&value, "quizId", &quiz_id.into())?;
    set(&value, "name", &media.name.clone().into())?;
    set(&value, "blob", &blob(media)?.into())?;
    Ok(value)
}
fn blob(media: &Media) -> Result<Blob> {
    let parts = Array::new();
    parts.push(&Uint8Array::from(media.bytes.as_slice()));
    let options = BlobPropertyBag::new();
    options.set_type(&media.content_type);
    Blob::new_with_u8_array_sequence_and_options(&parts, &options).map_err(err)
}
async fn read_media(value: JsValue) -> Result<Media> {
    let name = get(&value, "name")?
        .as_string()
        .ok_or_else(|| Error::Storage("media name missing".into()))?;
    let blob: Blob = get(&value, "blob")?.dyn_into().map_err(err)?;
    let bytes = Uint8Array::new(&JsFuture::from(blob.array_buffer()).await.map_err(err)?).to_vec();
    let extra = Object::new();
    for key in Object::keys(&Object::from(value.clone())).iter() {
        let name = key.as_string().unwrap_or_default();
        if !["key", "quizId", "name", "blob"].contains(&name.as_str()) {
            set(&extra, &name, &get(&value, &name)?)?;
        }
    }
    Ok(Media {
        name,
        bytes,
        content_type: blob.type_(),
        fields: decode(&extra)?,
    })
}
impl QuizStore for IndexedDbStore {
    async fn list_quizzes(&self) -> Result<Vec<Quiz>> {
        let tx = Transaction::new(&self.db, &["quizzes"], false)?;
        let result = async {
            let value = request(tx.store("quizzes")?.get_all().map_err(err)?)
                .wait()
                .await?;
            Array::from(&value).iter().map(|v| decode(&v)).collect()
        }
        .await;
        tx.finish(result).await
    }
    async fn get_quiz(&self, id: &str) -> Result<Option<Quiz>> {
        let tx = Transaction::new(&self.db, &["quizzes"], false)?;
        let result = async {
            let v = request(tx.store("quizzes")?.get(&id.into()).map_err(err)?)
                .wait()
                .await?;
            if v.is_undefined() {
                Ok(None)
            } else {
                decode(&v).map(Some)
            }
        }
        .await;
        tx.finish(result).await
    }
    async fn put_quiz(&self, quiz: &Quiz) -> Result<Quiz> {
        let mut next = quiz.clone();
        next.updated_at = now_ms();
        let tx = Transaction::new(&self.db, &["quizzes"], true)?;
        let result = async {
            request(tx.store("quizzes")?.put(&encode(&next)?).map_err(err)?)
                .wait()
                .await?;
            Ok(next)
        }
        .await;
        tx.finish(result).await
    }
    async fn update_quiz<F: FnOnce(&mut Quiz)>(&self, id: &str, change: F) -> Result<Option<Quiz>> {
        let tx = Transaction::new(&self.db, &["quizzes"], true)?;
        let result = async {
            let store = tx.store("quizzes")?;
            let value = request(store.get(&id.into()).map_err(err)?).wait().await?;
            if value.is_undefined() {
                return Ok(None);
            }
            let mut quiz: Quiz = decode(&value)?;
            change(&mut quiz);
            quiz.id = id.into();
            quiz.updated_at = now_ms();
            request(store.put(&encode(&quiz)?).map_err(err)?)
                .wait()
                .await?;
            Ok(Some(quiz))
        }
        .await;
        tx.finish(result).await
    }
    async fn delete_quiz(&self, id: &str) -> Result<()> {
        let tx = Transaction::new(&self.db, &["quizzes", "media"], true)?;
        let result = async {
            tx.store("quizzes")?.delete(&id.into()).map_err(err)?;
            let media = tx.store("media")?;
            let keys = request(
                media
                    .index("quizId")
                    .map_err(err)?
                    .get_all_keys_with_key(&id.into())
                    .map_err(err)?,
            )
            .wait()
            .await?;
            for key in Array::from(&keys).iter() {
                media.delete(&key).map_err(err)?;
            }
            Ok(())
        }
        .await;
        tx.finish(result).await
    }
}
impl MediaStore for IndexedDbStore {
    async fn list_media(&self, quiz_id: &str) -> Result<Vec<Media>> {
        let tx = Transaction::new(&self.db, &["media"], false)?;
        let result = async {
            request(
                tx.store("media")?
                    .index("quizId")
                    .map_err(err)?
                    .get_all_with_key(&quiz_id.into())
                    .map_err(err)?,
            )
            .wait()
            .await
        }
        .await;
        let records = tx.finish(result).await?;
        // Blob.arrayBuffer may yield outside IDB's transaction lifetime. Read only after completion.
        let mut files = Vec::new();
        for value in Array::from(&records).iter() {
            files.push(read_media(value).await?);
        }
        Ok(files)
    }
    async fn get_media(&self, quiz_id: &str, name: &str) -> Result<Option<Media>> {
        let tx = Transaction::new(&self.db, &["media"], false)?;
        let result = async {
            request(
                tx.store("media")?
                    .get(&format!("{quiz_id}/{name}").into())
                    .map_err(err)?,
            )
            .wait()
            .await
        }
        .await;
        let value = tx.finish(result).await?;
        if value.is_undefined() {
            Ok(None)
        } else {
            read_media(value).await.map(Some)
        }
    }
    async fn put_media(&self, quiz_id: &str, files: &[Media]) -> Result<()> {
        let tx = Transaction::new(&self.db, &["media"], true)?;
        let result = (|| {
            let store = tx.store("media")?;
            for file in files {
                store.put(&media_record(quiz_id, file)?).map_err(err)?;
            }
            Ok(())
        })();
        tx.finish(result).await
    }
    async fn delete_media(&self, quiz_id: &str, name: &str) -> Result<()> {
        let tx = Transaction::new(&self.db, &["media"], true)?;
        let result = tx
            .store("media")?
            .delete(&format!("{quiz_id}/{name}").into())
            .map(|_| ())
            .map_err(err);
        tx.finish(result).await
    }
}
fn theme_record(theme: &LibraryTheme) -> Result<JsValue> {
    let value = encode(
        &serde_json::json!({"id": theme.id, "theme": theme.theme, "savedAt": theme.saved_at}),
    )?;
    let files = Array::new();
    for file in &theme.files {
        let record = Object::new();
        set(&record, "name", &file.name.clone().into())?;
        set(&record, "blob", &blob(file)?.into())?;
        files.push(&record);
    }
    set(&value, "files", &files)?;
    Ok(value)
}
async fn read_theme(value: JsValue) -> Result<LibraryTheme> {
    let mut files = Vec::new();
    for file in Array::from(&get(&value, "files")?).iter() {
        files.push(read_media(file).await?);
    }
    Ok(LibraryTheme {
        id: decode(&get(&value, "id")?)?,
        theme: decode(&get(&value, "theme")?)?,
        saved_at: decode(&get(&value, "savedAt")?)?,
        files,
    })
}
impl ThemeStore for IndexedDbStore {
    async fn list_themes(&self) -> Result<Vec<LibraryTheme>> {
        let tx = Transaction::new(&self.library, &["themes"], false)?;
        let result = async {
            request(tx.store("themes")?.get_all().map_err(err)?)
                .wait()
                .await
        }
        .await;
        let records = tx.finish(result).await?;
        let mut themes = Vec::new();
        for value in Array::from(&records).iter() {
            themes.push(read_theme(value).await?);
        }
        themes.sort_by_key(|t| std::cmp::Reverse(t.saved_at));
        Ok(themes)
    }
    async fn get_theme(&self, id: &str) -> Result<Option<LibraryTheme>> {
        let tx = Transaction::new(&self.library, &["themes"], false)?;
        let result = async {
            request(tx.store("themes")?.get(&id.into()).map_err(err)?)
                .wait()
                .await
        }
        .await;
        let value = tx.finish(result).await?;
        if value.is_undefined() {
            Ok(None)
        } else {
            read_theme(value).await.map(Some)
        }
    }
    async fn put_theme(&self, theme: &LibraryTheme) -> Result<LibraryTheme> {
        let mut next = theme.clone();
        next.saved_at = now_ms();
        let tx = Transaction::new(&self.library, &["themes"], true)?;
        let result = async {
            request(
                tx.store("themes")?
                    .put(&theme_record(&next)?)
                    .map_err(err)?,
            )
            .wait()
            .await?;
            Ok(next)
        }
        .await;
        tx.finish(result).await
    }
    async fn delete_theme(&self, id: &str) -> Result<()> {
        let tx = Transaction::new(&self.library, &["themes"], true)?;
        let result = tx
            .store("themes")?
            .delete(&id.into())
            .map(|_| ())
            .map_err(err);
        tx.finish(result).await
    }
}
