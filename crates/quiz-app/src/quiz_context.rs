use leptos::prelude::*;
use quiz_core::quiz::{
    demo_teams::demo_teams,
    model::{Quiz, ResolvedTeam, resolve_teams},
    parse::{Round, parse_questions_csv},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct QuizContext {
    pub quiz: Arc<Quiz>,
    pub rounds: Arc<Vec<Round>>,
    pub teams: Arc<Vec<ResolvedTeam>>,
    pub media_urls: Arc<crate::store::MediaUrls>,
}
impl QuizContext {
    pub fn resolve_media(&self, name: &str) -> Option<String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return None;
        }
        if trimmed.to_ascii_lowercase().starts_with("https://")
            || trimmed.to_ascii_lowercase().starts_with("http://")
        {
            return Some(trimmed.into());
        }
        if self.quiz.builtin {
            media(name)
        } else {
            self.media_urls
                .0
                .get(&quiz_core::quiz::parse::media_key(name))
                .cloned()
        }
    }
}
fn media(name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    let trimmed = name.trim();
    let lower = trimmed.to_ascii_lowercase();
    Some(
        if lower.starts_with("https://") || lower.starts_with("http://") {
            trimmed.into()
        } else {
            format!("source/{trimmed}")
        },
    )
}
pub fn use_quiz() -> QuizContext {
    expect_context()
}
/// A query parameter of the page's address.
pub fn query_param(name: &str) -> Option<String> {
    let search = window().location().search().unwrap_or_default();
    web_sys::UrlSearchParams::new_with_str(&search)
        .ok()?
        .get(name)
}
pub async fn load_demo() -> Result<QuizContext, String> {
    // Dev only: `?csv=<url>` shows another sheet, e.g. `?csv=assets/dev-questions.csv` (README)
    let url = query_param("csv").unwrap_or_else(|| "questions.csv".into());
    load_demo_url(&url).await
}
pub async fn load_demo_seed() -> Result<QuizContext, String> {
    load_demo_url("questions.csv").await
}
async fn load_demo_url(url: &str) -> Result<QuizContext, String> {
    let response = gloo_net::http::Request::get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(format!("Loading {url} failed: HTTP {}", response.status()));
    }
    let csv = response.text().await.map_err(|e| e.to_string())?;
    let quiz = Quiz {
        id: "demo".into(),
        title: "Quiz Night (demo)".into(),
        builtin: true,
        questions_csv: csv,
        teams: demo_teams(),
        ..Quiz::default()
    };
    let rounds = Arc::new(parse_questions_csv(&quiz.questions_csv));
    let teams = Arc::new(resolve_teams(&quiz.teams, media));
    let context = QuizContext {
        quiz: Arc::new(quiz),
        rounds,
        teams,
        media_urls: Default::default(),
    };
    let _ = context.resolve_media("");
    Ok(context)
}

pub async fn load_stored(store: crate::store::Store, id: String) -> Result<QuizContext, String> {
    use quiz_store::QuizStore;
    let mut quiz = store
        .get_quiz(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Quiz not found".to_string())?;
    if !quiz.questions_sheet_url.trim().is_empty() {
        let url = quiz_core::quiz::sheets::to_csv_url(&quiz.questions_sheet_url);
        if let Ok(response) = gloo_net::http::Request::get(&url).send().await
            && response.ok()
            && let Ok(csv) = response.text().await
        {
            quiz.questions_csv = csv;
        }
    }
    let urls = Arc::new(crate::store::media_urls(&store, &id).await?);
    let mut context = QuizContext {
        rounds: Arc::new(parse_questions_csv(&quiz.questions_csv)),
        teams: Arc::new(Vec::new()),
        quiz: Arc::new(quiz),
        media_urls: urls,
    };
    context.teams = Arc::new(resolve_teams(&context.quiz.teams, |name| {
        context.resolve_media(name)
    }));
    Ok(context)
}
