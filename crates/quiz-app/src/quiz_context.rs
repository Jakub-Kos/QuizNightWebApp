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
        resolve_media(name, self.quiz.builtin, &self.media_urls.0)
    }
}
fn resolve_media(
    name: &str,
    builtin: bool,
    urls: &std::collections::BTreeMap<String, String>,
) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }
    urls.get(&quiz_core::quiz::parse::media_key(trimmed))
        .cloned()
        .or_else(|| {
            let lower = trimmed.to_ascii_lowercase();
            if builtin || lower.starts_with("https://") || lower.starts_with("http://") {
                media(trimmed)
            } else {
                None
            }
        })
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
    // Goes through the Google session when there is one (private Sheets, Drive quizzes) and
    // falls back to the public CSV export; on failure the stored copy is used.
    if !quiz.questions_sheet_url.trim().is_empty()
        && let Ok(csv) = crate::google::sheets::fetch_sheet_csv(&quiz.questions_sheet_url).await
    {
        quiz.questions_csv = csv;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_assets_use_quiz_media_before_builtin_fallback() {
        let urls = [(
            quiz_core::quiz::parse::media_key("theme-background.jpg"),
            "blob:theme".into(),
        )]
        .into_iter()
        .collect();
        assert_eq!(
            resolve_media("theme-background.jpg", false, &urls).as_deref(),
            Some("blob:theme")
        );
        assert_eq!(
            resolve_media("theme-background.jpg", true, &urls).as_deref(),
            Some("blob:theme")
        );
        assert_eq!(resolve_media("missing.jpg", false, &urls), None);
        assert_eq!(
            resolve_media("picture.jpg", true, &urls).as_deref(),
            Some("source/picture.jpg")
        );
        assert_eq!(
            resolve_media(" HTTPS://example.org/image.jpg ", false, &urls).as_deref(),
            Some("HTTPS://example.org/image.jpg")
        );
        assert_eq!(resolve_media("  ", true, &urls), None);
    }
}
