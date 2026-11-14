use super::fetch::{Options, google_json, post};
use super::*;
use quiz_core::quiz::{model::Quiz, sheet_model::*};
use serde_json::json;
#[derive(Debug, Clone)]
pub struct QuestionSheet {
    pub spreadsheet_id: String,
    pub title: String,
    pub tab: WriteTab,
    pub sheet: Sheet,
}
#[derive(Debug, Clone)]
pub struct ConflictError {
    pub data: QuestionSheet,
}
#[derive(Debug)]
pub enum EditError {
    Conflict(Box<ConflictError>),
    Google(Error),
}
impl From<Error> for EditError {
    fn from(e: Error) -> Self {
        Self::Google(e)
    }
}
pub async fn load_question_sheet(q: &Quiz) -> Result<QuestionSheet> {
    async {
        let d = q
            .drive
            .as_ref()
            .ok_or(Error::Drive("link".into(), "link".into()))?;
        let id = string(d, "spreadsheetId");
        let tabs = sheets::sheet_tabs(&id).await?;
        let tab = tabs
            .iter()
            .find(|t| Some(t.sheet_id) == sheets::gid(&q.questions_sheet_url))
            .or_else(|| tabs.iter().find(|t| d["questionsTab"] == t.title))
            .or(tabs.first())
            .ok_or(Error::Drive("no_sheet".into(), "no_sheet".into()))?;
        let values = sheets::tab_values(&id, &tab.title).await?;
        let stamps = stamp_columns(&values);
        let sheet = read_sheet(&to_model_rows(&values, &stamps));
        Ok(QuestionSheet {
            spreadsheet_id: id,
            title: tab.title.clone(),
            tab: WriteTab {
                sheet_id: tab.sheet_id,
                stamps,
            },
            sheet,
        })
    }
    .await
    .map_err(drive::to_drive_error)
}
pub async fn can_edit_sheet(id: &str) -> Result<bool> {
    google_json(
        &format!(
            "{}/files/{id}?fields=capabilities(canEdit)&supportsAllDrives=true",
            config::DRIVE_API
        ),
        Options::default(),
    )
    .await
    .map(|v| v["capabilities"]["canEdit"] == true)
    .map_err(drive::to_drive_error)
}
/// Planner returns None to report a conflict, with the fresh sheet attached to the error.
pub async fn commit_edit(
    q: &Quiz,
    make_plan: impl FnOnce(&Sheet) -> Option<Plan>,
) -> std::result::Result<QuestionSheet, EditError> {
    let data = load_question_sheet(q).await?;
    let plan = make_plan(&data.sheet)
        .ok_or_else(|| EditError::Conflict(Box::new(ConflictError { data: data.clone() })))?;
    let writes = sheet_writes(&data.tab, &sheets::tab_range(&data.title), &plan);
    let result = async {
        if !writes.structure.is_empty() {
            post(
                &format!("{}/{}:batchUpdate", config::SHEETS_API, data.spreadsheet_id),
                &json!({"requests":writes.structure}),
            )
            .await?;
        }
        for (mode, cells) in [("RAW", writes.raw), ("USER_ENTERED", writes.entered)] {
            if !cells.is_empty() {
                post(
                    &format!(
                        "{}/{}/values:batchUpdate",
                        config::SHEETS_API,
                        data.spreadsheet_id
                    ),
                    &json!({"valueInputOption":mode,"data":cells}),
                )
                .await?;
            }
        }
        Ok::<_, Error>(())
    }
    .await;
    result.map_err(|e| {
        EditError::Google(if matches!(e, Error::Api(403, _)) {
            Error::Drive("readonly".into(), "readonly".into())
        } else {
            drive::to_drive_error(e)
        })
    })?;
    Ok(load_question_sheet(q).await?)
}
pub use quiz_core::quiz::sheet_presence::Presence;
pub fn presence_key(email: &str) -> String {
    quiz_core::quiz::sheet_presence::presence_key(email)
}
pub async fn read_presence(id: &str) -> Result<Vec<Presence>> {
    let file = google_json(
        &format!(
            "{}/files/{id}?fields=appProperties&supportsAllDrives=true",
            config::DRIVE_API
        ),
        Options::default(),
    )
    .await?;
    Ok(quiz_core::quiz::sheet_presence::read_presence(
        &file["appProperties"],
        now() / 1000.,
    ))
}
pub async fn write_presence(
    id: &str,
    key: &str,
    entry: Option<&Presence>,
    keepalive: bool,
) -> Result<()> {
    let existing = if entry.is_some() {
        google_json(
            &format!(
                "{}/files/{id}?fields=appProperties&supportsAllDrives=true",
                config::DRIVE_API
            ),
            Options::default(),
        )
        .await
        .ok()
        .map(|v| v["appProperties"].clone())
        .unwrap_or_default()
    } else {
        Value::Null
    };
    let props =
        quiz_core::quiz::sheet_presence::presence_properties(key, entry, &existing, now() / 1000.);
    let mut options = Options::json("PATCH", &json!({"appProperties":props}));
    options.keepalive = keepalive;
    google_json(
        &format!(
            "{}/files/{id}?fields=id&supportsAllDrives=true",
            config::DRIVE_API
        ),
        options,
    )
    .await?;
    Ok(())
}
