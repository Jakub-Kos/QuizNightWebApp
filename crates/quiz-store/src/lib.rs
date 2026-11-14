//! Local-first persistence interfaces. No browser types or Send bounds cross these interfaces.
#![allow(async_fn_in_trait)]

use quiz_core::{quiz::model::Quiz, theme::format::Theme};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap};

#[cfg(target_arch = "wasm32")]
mod indexed_db;
pub mod package;
#[cfg(target_arch = "wasm32")]
pub use indexed_db::IndexedDbStore;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("storage: {0}")]
    Storage(String),
    #[error("invalid package: {0}")]
    Package(String),
    #[error("quiz id already exists: {0}")]
    Conflict(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
}
pub type Result<T> = std::result::Result<T, Error>;

/// Extra fields preserve React's optional driveId/driveModified (and future metadata).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub name: String,
    pub bytes: Vec<u8>,
    pub content_type: String,
    #[serde(default)]
    pub fields: BTreeMap<String, serde_json::Value>,
}
#[derive(Debug, Clone)]
pub struct LibraryTheme {
    pub id: String,
    pub theme: Theme,
    pub files: Vec<Media>,
    pub saved_at: i64,
}

pub trait QuizStore {
    async fn list_quizzes(&self) -> Result<Vec<Quiz>>;
    async fn get_quiz(&self, id: &str) -> Result<Option<Quiz>>;
    /// Upsert, stamping updatedAt, and return the saved record.
    async fn put_quiz(&self, quiz: &Quiz) -> Result<Quiz>;
    /// Atomic read-modify-write; missing records return None. The id is immutable.
    async fn update_quiz<F: FnOnce(&mut Quiz)>(&self, id: &str, change: F) -> Result<Option<Quiz>>;
    /// Idempotent; also removes all media belonging to this quiz.
    async fn delete_quiz(&self, id: &str) -> Result<()>;
}
pub trait MediaStore {
    async fn list_media(&self, quiz_id: &str) -> Result<Vec<Media>>;
    async fn get_media(&self, quiz_id: &str, name: &str) -> Result<Option<Media>>;
    /// Exact names are upsert keys; a batch is atomic. No quiz record is required.
    async fn put_media(&self, quiz_id: &str, files: &[Media]) -> Result<()>;
    async fn delete_media(&self, quiz_id: &str, name: &str) -> Result<()>;
}
pub trait ThemeStore {
    /// Newest savedAt first.
    async fn list_themes(&self) -> Result<Vec<LibraryTheme>>;
    async fn get_theme(&self, id: &str) -> Result<Option<LibraryTheme>>;
    async fn put_theme(&self, theme: &LibraryTheme) -> Result<LibraryTheme>;
    async fn delete_theme(&self, id: &str) -> Result<()>;
}

pub(crate) fn now_ms() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as i64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }
}

#[derive(Default)]
struct Data {
    quizzes: BTreeMap<String, Quiz>,
    media: BTreeMap<(String, String), Media>,
    themes: BTreeMap<String, LibraryTheme>,
}
/// Single-threaded, owned fallback. Retain this instance for the lifetime of the app.
#[derive(Default)]
pub struct MemoryStore(RefCell<Data>);
impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}
impl QuizStore for MemoryStore {
    async fn list_quizzes(&self) -> Result<Vec<Quiz>> {
        Ok(self.0.borrow().quizzes.values().cloned().collect())
    }
    async fn get_quiz(&self, id: &str) -> Result<Option<Quiz>> {
        Ok(self.0.borrow().quizzes.get(id).cloned())
    }
    async fn put_quiz(&self, quiz: &Quiz) -> Result<Quiz> {
        let mut next = quiz.clone();
        next.updated_at = now_ms();
        self.0
            .borrow_mut()
            .quizzes
            .insert(next.id.clone(), next.clone());
        Ok(next)
    }
    async fn update_quiz<F: FnOnce(&mut Quiz)>(&self, id: &str, change: F) -> Result<Option<Quiz>> {
        let mut data = self.0.borrow_mut();
        let Some(quiz) = data.quizzes.get_mut(id) else {
            return Ok(None);
        };
        change(quiz);
        quiz.id = id.into();
        quiz.updated_at = now_ms();
        Ok(Some(quiz.clone()))
    }
    async fn delete_quiz(&self, id: &str) -> Result<()> {
        let mut data = self.0.borrow_mut();
        data.quizzes.remove(id);
        data.media.retain(|(q, _), _| q != id);
        Ok(())
    }
}
impl MediaStore for MemoryStore {
    async fn list_media(&self, quiz_id: &str) -> Result<Vec<Media>> {
        Ok(self
            .0
            .borrow()
            .media
            .iter()
            .filter(|((q, _), _)| q == quiz_id)
            .map(|(_, m)| m.clone())
            .collect())
    }
    async fn get_media(&self, quiz_id: &str, name: &str) -> Result<Option<Media>> {
        Ok(self
            .0
            .borrow()
            .media
            .get(&(quiz_id.into(), name.into()))
            .cloned())
    }
    async fn put_media(&self, quiz_id: &str, files: &[Media]) -> Result<()> {
        let mut data = self.0.borrow_mut();
        for file in files {
            data.media
                .insert((quiz_id.into(), file.name.clone()), file.clone());
        }
        Ok(())
    }
    async fn delete_media(&self, quiz_id: &str, name: &str) -> Result<()> {
        self.0
            .borrow_mut()
            .media
            .remove(&(quiz_id.into(), name.into()));
        Ok(())
    }
}
impl ThemeStore for MemoryStore {
    async fn list_themes(&self) -> Result<Vec<LibraryTheme>> {
        let mut themes: Vec<_> = self.0.borrow().themes.values().cloned().collect();
        themes.sort_by_key(|t| std::cmp::Reverse(t.saved_at));
        Ok(themes)
    }
    async fn get_theme(&self, id: &str) -> Result<Option<LibraryTheme>> {
        Ok(self.0.borrow().themes.get(id).cloned())
    }
    async fn put_theme(&self, theme: &LibraryTheme) -> Result<LibraryTheme> {
        let mut theme = theme.clone();
        theme.saved_at = now_ms();
        self.0
            .borrow_mut()
            .themes
            .insert(theme.id.clone(), theme.clone());
        Ok(theme)
    }
    async fn delete_theme(&self, id: &str) -> Result<()> {
        self.0.borrow_mut().themes.remove(id);
        Ok(())
    }
}
