use super::fetch::{Options, google_json};
use super::*;
pub use quiz_core::quiz::sheets::{rows_to_csv, tab_range, to_csv_url};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap};
use wasm_bindgen_futures::JsFuture;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub sheet_id: i64,
    pub title: String,
    pub index: i64,
}
thread_local! {static CACHE:RefCell<HashMap<String,(f64,Vec<Tab>)>>=RefCell::new(HashMap::new());}
pub async fn sheet_tabs(id: &str) -> Result<Vec<Tab>> {
    if let Some(t) = CACHE.with(|c| {
        c.borrow()
            .get(id)
            .filter(|(at, _)| now() - at < 300_000.)
            .map(|(_, t)| t.clone())
    }) {
        return Ok(t);
    }
    let meta = google_json(
        &format!(
            "{}/{id}?fields=sheets.properties(sheetId,title,index)",
            config::SHEETS_API
        ),
        Options::default(),
    )
    .await?;
    let mut tabs: Vec<Tab> = meta["sheets"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|s| serde_json::from_value(s["properties"].clone()).ok())
        .collect();
    tabs.sort_by_key(|t| t.index);
    CACHE.with(|c| c.borrow_mut().insert(id.into(), (now(), tabs.clone())));
    Ok(tabs)
}
pub fn rows(v: &Value) -> Vec<Vec<String>> {
    v.as_array()
        .into_iter()
        .flatten()
        .map(|r| {
            r.as_array()
                .into_iter()
                .flatten()
                .map(|c| {
                    c.as_str().map(str::to_owned).unwrap_or_else(|| {
                        if c.is_null() {
                            String::new()
                        } else {
                            c.to_string()
                        }
                    })
                })
                .collect()
        })
        .collect()
}
pub async fn tab_values(id: &str, title: &str) -> Result<Vec<Vec<String>>> {
    let v = google_json(
        &format!(
            "{}/{id}/values/{}",
            config::SHEETS_API,
            enc(&tab_range(title))
        ),
        Options::default(),
    )
    .await?;
    Ok(rows(&v["values"]))
}
pub fn gid(url: &str) -> Option<i64> {
    url.split(['#', '&', '?'])
        .find_map(|s| s.strip_prefix("gid=").and_then(|s| s.parse().ok()))
}
pub async fn fetch_sheet_csv(url: &str) -> Result<String> {
    let id = quiz_core::quiz::drive::parse_drive_link(url)
        .filter(|_| url.contains("docs.google.com/spreadsheets/d/"));
    if let Some(id) = id.filter(|_| auth::current_token().is_some()) {
        let result = async {
            let tabs = sheet_tabs(&id).await?;
            let tab = tabs
                .iter()
                .find(|t| gid(url).is_none_or(|g| g == t.sheet_id))
                .ok_or(Error::Sheet("http_404".into()))?;
            Ok::<_, Error>(rows_to_csv(&tab_values(&id, &tab.title).await?))
        }
        .await;
        if let Ok(csv) = result {
            return Ok(csv);
        }
        CACHE.with(|c| c.borrow_mut().remove(&id));
    }
    let url = to_csv_url(url);
    let url = url.split('#').next().unwrap_or(&url);
    let sep = if url.contains('?') { "&" } else { "?" };
    let res = fetch::raw(
        &format!("{url}{sep}_cb={}", now() as i64),
        &Options {
            no_store: true,
            ..Default::default()
        },
        None,
    )
    .await
    .map_err(|_| Error::Sheet("network".into()))?;
    if !res.ok() {
        return Err(Error::Sheet(format!("http_{}", res.status())));
    }
    let text = JsFuture::from(res.text()?)
        .await?
        .as_string()
        .unwrap_or_default();
    if text.trim_start().starts_with('<') {
        return Err(Error::Sheet("not_shared".into()));
    }
    Ok(text)
}
pub fn sheet_error_text(error: &Error, t: &Value) -> String {
    match error {
        Error::Sheet(s) => string(
            t,
            if s.starts_with("http_") {
                "ed_err_http"
            } else {
                "ed_err_network"
            },
        ),
        e => e.to_string(),
    }
}
