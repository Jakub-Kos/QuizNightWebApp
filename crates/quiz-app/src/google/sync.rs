//! Serialized per-quiz config synchronization. notify_quiz_saved is the storage/UI save hook.
use super::Error;
use super::fetch::{Options, google_fetch, google_json, post};
use super::*;
use leptos::prelude::*;
use quiz_core::quiz::{drive_config::*, model::Quiz};
use quiz_store::QuizStore;
use serde_json::json;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen_futures::{JsFuture, spawn_local};
#[derive(Debug, Clone, Default)]
pub struct SyncState {
    pub phase: Phase,
    pub at: Option<f64>,
    pub error: Option<Error>,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Phase {
    #[default]
    Idle,
    Pending,
    Syncing,
    Done,
    Error,
}
thread_local! {static STATES:RefCell<HashMap<String,SyncState>>=RefCell::new(HashMap::new());static LISTENERS:RefCell<Vec<(u64,Rc<dyn Fn(&str,SyncState)>)>>=const{RefCell::new(vec![])};static NEXT:RefCell<u64>=const{RefCell::new(0)};static RUNNING:RefCell<HashMap<String,js_sys::Promise>>=RefCell::new(HashMap::new());static TIMERS:RefCell<HashMap<String,gloo_timers::callback::Timeout>>=RefCell::new(HashMap::new());static ACCESS:RefCell<HashMap<String,(f64,bool)>>=RefCell::new(HashMap::new());}
pub fn get_sync_state(id: &str) -> SyncState {
    STATES.with(|s| s.borrow().get(id).cloned().unwrap_or_default())
}
fn set(id: &str, phase: Phase, error: Option<Error>) {
    let mut state = get_sync_state(id);
    state.phase = phase.clone();
    state.error = error;
    if phase == Phase::Done {
        state.at = Some(now());
    }
    STATES.with(|s| s.borrow_mut().insert(id.into(), state.clone()));
    let listeners = LISTENERS.with(|l| {
        l.borrow()
            .iter()
            .map(|(_, f)| f.clone())
            .collect::<Vec<_>>()
    });
    for f in listeners {
        f(id, state.clone());
    }
}
pub struct SyncSubscription(u64);
impl Drop for SyncSubscription {
    fn drop(&mut self) {
        LISTENERS.with(|l| l.borrow_mut().retain(|(id, _)| *id != self.0));
    }
}
pub fn on_sync_state(f: impl Fn(&str, SyncState) + 'static) -> SyncSubscription {
    let id = NEXT.with(|n| {
        *n.borrow_mut() += 1;
        *n.borrow()
    });
    LISTENERS.with(|l| l.borrow_mut().push((id, Rc::new(f))));
    SyncSubscription(id)
}
pub fn use_drive_sync(id: &str) -> RwSignal<SyncState> {
    let state = RwSignal::new(get_sync_state(id));
    let id = id.to_owned();
    let sub = on_sync_state(move |q, s| {
        if q == id {
            state.set(s)
        }
    });
    on_cleanup(move || drop(sub));
    state
}
pub fn notify_quiz_saved(q: &Quiz, source: &str) {
    if source == "drive"
        || q.drive.as_ref().is_none_or(|d| !d["folderId"].is_string())
        || auth::current_token().is_none()
    {
        return;
    }
    if q.drive.as_ref().and_then(|d| d.get("configBase")) == Some(&config_from_quiz(q)) {
        return;
    }
    let id = q.id.clone();
    set(&id, Phase::Pending, None);
    let saved = id.clone();
    TIMERS.with(|t| {
        t.borrow_mut().insert(
            id,
            gloo_timers::callback::Timeout::new(1500, move || {
                spawn_local(async move {
                    let _ = sync_drive_config(&saved).await;
                })
            }),
        )
    });
}
/// React-compatible t + eight base-36 random digits, independent of team names.
pub fn new_team_id() -> String {
    let n = js_sys::Number::from(js_sys::Math::random());
    let digits = n
        .to_string_with_radix(36)
        .ok()
        .and_then(|s| s.as_string())
        .unwrap_or_default();
    format!("t{}", digits.chars().skip(2).take(8).collect::<String>())
}
async fn find_config(d: &Value) -> Result<Option<String>> {
    if let Some(id) = d["configFileId"].as_str() {
        if let Ok(f) = google_json(
            &format!(
                "{}/files/{id}?fields=id,trashed&supportsAllDrives=true",
                config::DRIVE_API
            ),
            Options::default(),
        )
        .await
        {
            if f["trashed"] != true {
                return Ok(Some(id.into()));
            }
        }
    }
    let q = enc(&format!(
        "'{}' in parents and name = '{CONFIG_NAME}' and trashed = false",
        string(d, "folderId")
    ));
    let f=google_json(&format!("{}/files?q={q}&fields=files(id)&orderBy=modifiedTime%20desc&supportsAllDrives=true&includeItemsFromAllDrives=true",config::DRIVE_API),Options::default()).await?;
    Ok(f["files"][0]["id"].as_str().map(str::to_owned))
}
async fn write_config(folder: &str, file: Option<&str>, config_value: &Value) -> Result<String> {
    let body = serde_json::to_string_pretty(config_value).unwrap();
    if let Some(id) = file {
        google_fetch(
            &format!(
                "{}/{id}?uploadType=media&supportsAllDrives=true",
                config::UPLOAD_API
            ),
            Options {
                method: "PATCH".into(),
                body: Some(body),
                headers: vec![("Content-Type".into(), "application/json".into())],
                ..Default::default()
            },
        )
        .await?;
        return Ok(id.into());
    }
    let boundary = format!("quiznight{}", crate::store::new_id());
    let meta = json!({"name":CONFIG_NAME,"parents":[folder],"mimeType":"application/json"});
    let multipart = format!(
        "--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{meta}\r\n--{boundary}\r\nContent-Type: application/json\r\n\r\n{body}\r\n--{boundary}--"
    );
    let f = google_json(
        &format!(
            "{}?uploadType=multipart&supportsAllDrives=true&fields=id",
            config::UPLOAD_API
        ),
        Options {
            method: "POST".into(),
            body: Some(multipart),
            headers: vec![(
                "Content-Type".into(),
                format!("multipart/related; boundary={boundary}"),
            )],
            ..Default::default()
        },
    )
    .await?;
    Ok(string(&f, "id"))
}
pub async fn write_scores_tab(
    id: &str,
    tab: &sheets::Tab,
    table: &ScoresTable,
    ops: &Ops,
) -> Result<()> {
    let w = scores_writes(table, tab.sheet_id, &sheets::tab_range(&tab.title), ops);
    if !w["structure"].as_array().unwrap().is_empty() {
        post(
            &format!("{}/{id}:batchUpdate", config::SHEETS_API),
            &json!({"requests":w["structure"]}),
        )
        .await?;
    }
    if !w["cells"].as_array().unwrap().is_empty() {
        post(
            &format!("{}/{id}/values:batchUpdate", config::SHEETS_API),
            &json!({"valueInputOption":"RAW","data":w["cells"]}),
        )
        .await?;
    }
    Ok(())
}
async fn can_edit(id: &str) -> bool {
    if let Some(b) = ACCESS.with(|a| {
        a.borrow()
            .get(id)
            .filter(|(at, _)| now() - at < 300_000.)
            .map(|(_, v)| *v)
    }) {
        return b;
    }
    let v = sheet_edit::can_edit_sheet(id).await.unwrap_or(false);
    ACCESS.with(|a| a.borrow_mut().insert(id.into(), (now(), v)));
    v
}
async fn run(qid: &str) -> Result<Option<Quiz>> {
    let store = fetch::store()?;
    let Some(q) = store.get_quiz(qid).await? else {
        return Ok(None);
    };
    let Some(d) = q
        .drive
        .as_ref()
        .filter(|d| d["folderId"].is_string() && auth::current_token().is_some())
    else {
        return Ok(None);
    };
    let file = find_config(d).await?;
    let remote = if let Some(id) = &file {
        let res = google_fetch(
            &format!(
                "{}/files/{id}?alt=media&supportsAllDrives=true",
                config::DRIVE_API
            ),
            Options::default(),
        )
        .await?;
        let text = JsFuture::from(res.text()?)
            .await?
            .as_string()
            .unwrap_or_default();
        upgrade(serde_json::from_str(&text).unwrap_or(Value::Null))
    } else {
        Value::Null
    };
    let local = config_from_quiz(&q);
    let base = d.get("configBase").filter(|b| b["version"] == 2);
    let mut merged = if remote.is_null() {
        local.clone()
    } else if let Some(b) = base {
        merge3(Some(b), Some(&local), Some(&remote)).unwrap()
    } else {
        let mut m = overlay(&local, &remote);
        m["teams"] = remote.get("teams").cloned().unwrap_or(json!({}));
        m
    };
    let mut order = None;
    if !q.scores_sheet_url.is_empty() {
        let id = string(d, "spreadsheetId");
        let tabs = sheets::sheet_tabs(&id).await?;
        if let Some(tab) = tabs
            .iter()
            .find(|t| Some(t.sheet_id) == sheets::gid(&q.scores_sheet_url))
            .or(tabs.first())
        {
            if let Some(table) = scores_table(&sheets::tab_values(&id, &tab.title).await?) {
                let result = reconcile(
                    &table.rows,
                    &merged["teams"],
                    base.map(|b| &b["teams"]),
                    &local["teams"],
                    can_edit(&id).await,
                    &mut || new_team_id(),
                );
                let has_ops = result.ops != Ops::default();
                let result = if has_ops
                    && write_scores_tab(&id, tab, &table, &result.ops)
                        .await
                        .is_err()
                {
                    reconcile(
                        &table.rows,
                        &merged["teams"],
                        None,
                        &json!({}),
                        false,
                        &mut || new_team_id(),
                    )
                } else {
                    result
                };
                merged["teams"] = result.entries;
                order = Some(result.order);
            }
        }
    }
    let mut saved = file.clone();
    if merged != remote {
        match write_config(&string(d, "folderId"), file.as_deref(), &merged).await {
            Ok(id) => saved = Some(id),
            Err(Error::Api(403, _)) => {}
            Err(e) => return Err(e),
        }
    }
    let next = store
        .update_quiz_source(
            qid,
            move |q| {
                let mut next = apply_config(q, &merged, order.as_deref());
                let base = config_from_quiz(&next);
                if let Some(d) = next.drive.as_mut() {
                    d["configFileId"] = json!(saved);
                    d["configBase"] = base;
                }
                *q = next;
            },
            "drive",
        )
        .await?;
    Ok(next)
}
/// A promise chain preserves FIFO order even if an awaiting UI task is disposed.
pub async fn sync_drive_config(id: &str) -> Result<Option<Quiz>> {
    let previous = RUNNING.with(|r| r.borrow().get(id).cloned());
    let result = Rc::new(RefCell::new(None));
    let output = result.clone();
    let quiz_id = id.to_owned();
    let next = wasm_bindgen_futures::future_to_promise(async move {
        if let Some(previous) = previous {
            let _ = JsFuture::from(previous).await;
        }
        set(&quiz_id, Phase::Syncing, None);
        let value = run(&quiz_id).await;
        match &value {
            Ok(Some(_)) => set(&quiz_id, Phase::Done, None),
            Ok(None) => set(&quiz_id, Phase::Idle, None),
            Err(e) => set(&quiz_id, Phase::Error, Some(e.clone())),
        }
        *output.borrow_mut() = Some(value);
        Ok(wasm_bindgen::JsValue::UNDEFINED)
    });
    RUNNING.with(|r| r.borrow_mut().insert(id.into(), next.clone()));
    JsFuture::from(next).await?;
    let value = result.borrow_mut().take().expect("sync promise completed");
    value
}

/// Locate one writable score by stable team ID, falling back to a unique name for older tabs.
/// Never guess a row or column: a missing/ambiguous cell leaves the Sheet untouched.
pub fn score_cell(rows: &[Vec<String>], team_id: &str, name: &str, round: &str) -> Option<String> {
    let table = scores_table(rows)?;
    let by_id: Vec<_> = table
        .rows
        .iter()
        .filter(|r| r.id.as_deref() == Some(team_id))
        .collect();
    let candidates = if by_id.is_empty() {
        table
            .rows
            .iter()
            .filter(|r| r.name.trim() == name.trim() && r.id.is_none())
            .collect::<Vec<_>>()
    } else {
        by_id
    };
    if candidates.len() != 1 {
        return None;
    }
    let columns = rows[table.header]
        .iter()
        .enumerate()
        .filter(|(_, heading)| {
            let lower = heading.to_lowercase();
            (lower.contains("round") || lower.contains("kolo"))
                && heading
                    .split(|c: char| !c.is_ascii_digit() && c != '.')
                    .find(|s| !s.is_empty())
                    == Some(round)
        })
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if columns.len() != 1 {
        return None;
    }
    Some(format!("{}{}", column(columns[0]), candidates[0].row + 1))
}

pub async fn write_score_cell(q: &Quiz, team_id: &str, round: &str, value: &str) -> Result<()> {
    let d = q
        .drive
        .as_ref()
        .ok_or_else(|| Error::Sheet("not_drive".into()))?;
    let id = string(d, "spreadsheetId");
    let tabs = sheets::sheet_tabs(&id).await?;
    let tab = tabs
        .iter()
        .find(|t| Some(t.sheet_id) == sheets::gid(&q.scores_sheet_url))
        .or_else(|| tabs.iter().find(|t| d["scoresTab"] == t.title))
        .ok_or_else(|| Error::Sheet("missing_scores_tab".into()))?;
    let values = sheets::tab_values(&id, &tab.title).await?;
    let name = q
        .teams
        .iter()
        .find(|t| match &t.id {
            Some(quiz_core::quiz::model::TeamId::Text(s)) => s == team_id,
            Some(quiz_core::quiz::model::TeamId::Number(n)) => n.to_string() == team_id,
            None => false,
        })
        .map(|t| t.name.as_str())
        .ok_or_else(|| Error::Sheet("missing_team".into()))?;
    let cell = score_cell(&values, team_id, name, round)
        .ok_or_else(|| Error::Sheet("missing_score_cell".into()))?;
    // RAW numbers are independent of the spreadsheet's decimal separator. Empty clears the cell.
    let value = if value.is_empty() {
        json!("")
    } else {
        json!(
            value
                .parse::<f64>()
                .ok()
                .filter(|n| n.is_finite())
                .unwrap_or(0.)
        )
    };
    post(&format!("{}/{id}/values:batchUpdate",config::SHEETS_API),
        &json!({"valueInputOption":"RAW","data":[{"range":format!("{}!{cell}",sheets::tab_range(&tab.title)),"values":[[value]]}]})).await?;
    Ok(())
}

#[cfg(test)]
mod score_tests {
    use super::*;
    #[test]
    fn score_targets_follow_ids_and_refuse_ambiguous_or_missing_cells() {
        let rows = vec![
            vec![
                "Team name".into(),
                "Round 1".into(),
                "Kolo 2".into(),
                quiz_core::quiz::parse::TEAM_ID_HEADER.into(),
            ],
            vec!["Renamed".into(), "".into(), "".into(), "t1".into()],
            vec!["Legacy".into()],
        ];
        assert_eq!(score_cell(&rows, "t1", "Old name", "2"), Some("C2".into()));
        assert_eq!(score_cell(&rows, "t2", "Legacy", "1"), Some("B3".into()));
        assert_eq!(score_cell(&rows, "t3", "Renamed", "1"), None);
        assert_eq!(score_cell(&rows, "t1", "Renamed", "3"), None);
        let mut duplicate = rows.clone();
        duplicate.push(rows[1].clone());
        assert_eq!(score_cell(&duplicate, "t1", "Renamed", "1"), None);
        let mut duplicate = rows.clone();
        duplicate[0].push("Round 1".into());
        assert_eq!(score_cell(&duplicate, "t1", "Renamed", "1"), None);
    }
}
