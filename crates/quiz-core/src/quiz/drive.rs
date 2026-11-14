//! Drive identifiers, links, and record merging shared with browser clients.
use serde_json::Value;
pub const GOOGLE_CLIENT_ID: &str =
    "321755554883-pegnabl76bbm71d58akvtaqhq2hm2ik0.apps.googleusercontent.com";
pub const GOOGLE_SHEET_TEMPLATE: &str = "https://docs.google.com/spreadsheets/d/1p1-bxa2EBWcRvRE_bWu3AbHaSHkgjD7jgBcvu1tWNvw/edit?usp=sharing";
pub const DRIVE_API: &str = "https://www.googleapis.com/drive/v3";
pub const UPLOAD_API: &str = "https://www.googleapis.com/upload/drive/v3/files";
pub const SHEETS_API: &str = "https://sheets.googleapis.com/v4/spreadsheets";
pub const FOLDER: &str = "application/vnd.google-apps.folder";
pub const SPREADSHEET: &str = "application/vnd.google-apps.spreadsheet";
pub fn google_copy_url(url: &str) -> String {
    regex::Regex::new(r"docs\.google\.com/spreadsheets/d/([A-Za-z0-9_-]+)")
        .unwrap()
        .captures(url)
        .map_or_else(
            || url.into(),
            |c| format!("https://docs.google.com/spreadsheets/d/{}/copy", &c[1]),
        )
}
pub fn parse_drive_link(link: &str) -> Option<String> {
    let s = link.trim();
    for p in [
        r"/folders/([A-Za-z0-9_-]{10,})",
        r"/d/([A-Za-z0-9_-]{10,})",
        r"[?&]id=([A-Za-z0-9_-]{10,})",
        r"^([A-Za-z0-9_-]{20,})$",
    ] {
        if let Some(c) = regex::Regex::new(p).unwrap().captures(s) {
            return Some(c[1].into());
        }
    }
    None
}
pub fn folder_url(id: &str) -> String {
    format!("https://drive.google.com/drive/folders/{id}")
}
pub fn sheet_url(id: &str, gid: Option<i64>) -> String {
    format!(
        "https://docs.google.com/spreadsheets/d/{id}/edit{}",
        gid.map(|g| format!("#gid={g}")).unwrap_or_default()
    )
}
pub fn is_media_folder_name(name: &str) -> bool {
    regex::Regex::new("(?i)^(zdroje?|sources?|media)$")
        .unwrap()
        .is_match(name.trim())
}
pub fn same_drive_quiz(a: &Value, b: &Value) -> bool {
    !a.is_null()
        && !b.is_null()
        && a.get("folderId")
            .filter(|v| !v.is_null())
            .or(a.get("spreadsheetId"))
            == b.get("folderId")
                .filter(|v| !v.is_null())
                .or(b.get("spreadsheetId"))
}
pub fn merge_drive(quiz: &mut super::model::Quiz, fields: &Value, extra: &Value) {
    quiz.questions_csv = fields["questionsCsv"].as_str().unwrap_or_default().into();
    quiz.questions_sheet_url = fields["questionsSheetUrl"]
        .as_str()
        .unwrap_or_default()
        .into();
    if let Some(s) = fields["scoresSheetUrl"].as_str().filter(|s| !s.is_empty()) {
        quiz.scores_sheet_url = s.into();
    }
    let mut d = quiz.drive.clone().unwrap_or(serde_json::json!({}));
    for v in [&fields["drive"], extra] {
        if let Some(m) = v.as_object() {
            for (k, v) in m {
                d[k] = v.clone();
            }
        }
    }
    quiz.drive = Some(d);
}
/// JavaScript encodeURIComponent, used for Sheets range path segments.
pub fn encode_component(s: &str) -> String {
    encode(s, false)
}
/// URLSearchParams encoding, including '+' for spaces.
pub fn form_query(pairs: &[(&str, String)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k, true), encode(v, true)))
        .collect::<Vec<_>>()
        .join("&")
}
fn encode(s: &str, form: bool) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"*-._".contains(&b) || (!form && b"!~'()".contains(&b)) {
            out.push(b as char);
        } else if form && b == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}
