use super::format::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexEntry {
    pub path: String,
    pub added_at: String,
    pub theme: Theme,
}
pub fn index_entry(theme: &Theme, added_at: &str) -> IndexEntry {
    IndexEntry {
        path: format!("themes/{}/", theme.id),
        added_at: added_at.into(),
        theme: theme.clone(),
    }
}
pub fn parse_index(raw: &serde_json::Value) -> Vec<IndexEntry> {
    let re = regex::Regex::new(r"^themes/[a-z0-9-]+/$").unwrap();
    raw["themes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| {
            let path = e["path"].as_str()?;
            if !re.is_match(path) {
                return None;
            }
            let theme = normalize_theme_with_options(
                &e["theme"],
                &NormalizeOptions {
                    stored: true,
                    ..Default::default()
                },
            )
            .theme?;
            Some(IndexEntry {
                path: path.into(),
                added_at: e["addedAt"].as_str().unwrap_or("").into(),
                theme,
            })
        })
        .collect()
}
pub fn workshop_file_url(base_url: &str, entry: &IndexEntry, name: &str) -> String {
    format!("{base_url}workshop/{}{name}", entry.path)
}
