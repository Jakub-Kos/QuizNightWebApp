use super::format::*;
use std::io::{Cursor, Read, Write};
#[derive(Debug, thiserror::Error)]
pub enum ThemePackError {
    #[error("no_theme")]
    NoTheme,
    #[error("bad_json")]
    BadJson,
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
#[derive(Debug, Clone)]
pub struct PackAsset {
    pub name: String,
    pub bytes: Vec<u8>,
}
#[derive(Debug, Clone)]
pub struct ThemePack {
    pub theme: Theme,
    pub problems: Vec<Problem>,
    pub assets: Vec<PackAsset>,
}
fn basename(s: &str) -> &str {
    s.rsplit('/').next().unwrap_or(s)
}
pub fn read_theme_pack(bytes: &[u8]) -> Result<ThemePack, ThemePackError> {
    let mut entries = Vec::new();
    if bytes.starts_with(b"PK") {
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;
        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let name = file.name()?.into_owned();
            if file.is_dir()
                || name.starts_with("__MACOSX/")
                || name.contains("/__MACOSX/")
                || basename(&name).starts_with('.')
            {
                continue;
            }
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            entries.push(PackAsset { name, bytes });
        }
    } else {
        entries.push(PackAsset {
            name: "theme.json".into(),
            bytes: bytes.to_vec(),
        });
    }
    let index = entries
        .iter()
        .position(|e| basename(&e.name).eq_ignore_ascii_case("theme.json"))
        .or_else(|| {
            (entries.len() == 1 && entries[0].name.to_lowercase().ends_with(".json")).then_some(0)
        })
        .ok_or(ThemePackError::NoTheme)?;
    let meta = entries.remove(index);
    let raw = serde_json::from_slice(&meta.bytes).map_err(|_| ThemePackError::BadJson)?;
    let root = meta
        .name
        .rsplit_once('/')
        .map(|(r, _)| format!("{r}/"))
        .unwrap_or_default();
    for e in &mut entries {
        e.name = e.name.strip_prefix(&root).unwrap_or(&e.name).to_owned();
    }
    let result = normalize_theme_with_options(
        &raw,
        &NormalizeOptions {
            stored: false,
            assets: entries
                .iter()
                .map(|e| Asset {
                    name: e.name.clone(),
                    size: e.bytes.len() as f64,
                })
                .collect(),
        },
    );
    let theme = result.theme.ok_or(ThemePackError::NoTheme)?;
    let used = theme_files(&theme);
    entries.retain(|e| used.contains(&e.name));
    Ok(ThemePack {
        theme,
        problems: result.problems,
        assets: entries,
    })
}
pub fn theme_for_quiz_media(pack: &ThemePack) -> (Theme, Vec<PackAsset>) {
    let mut names = std::collections::BTreeMap::new();
    let mut taken = Vec::new();
    let assets = pack
        .assets
        .iter()
        .map(|a| {
            let plain = basename(&a.name)
                .strip_prefix("theme-")
                .unwrap_or(basename(&a.name));
            let mut name = format!("theme-{plain}");
            let mut i = 2;
            while taken.contains(&name.to_lowercase()) {
                name = format!("theme-{i}-{plain}");
                i += 1;
            }
            taken.push(name.to_lowercase());
            names.insert(a.name.clone(), name.clone());
            PackAsset {
                name,
                bytes: a.bytes.clone(),
            }
        })
        .collect();
    (
        rename_theme_files(&pack.theme, |p| {
            Some(names.get(p).cloned().unwrap_or_else(|| p.into()))
        }),
        assets,
    )
}
pub fn export_theme_pack(theme: &Theme, assets: &[PackAsset]) -> Result<Vec<u8>, ThemePackError> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut names = std::collections::BTreeMap::new();
    let mut files = std::collections::BTreeMap::new();
    for name in theme_files(theme) {
        if let Some(a) = assets.iter().find(|a| a.name == name) {
            let base = basename(&name);
            let base = base.strip_prefix("theme-").unwrap_or(base);
            let path = format!("assets/{base}");
            names.insert(name, path.clone());
            files.insert(path, a.bytes.clone());
        }
    }
    let packed = rename_theme_files(theme, |n| names.get(n).cloned());
    files.insert(
        "theme.json".into(),
        serde_json::to_vec_pretty(&packed).unwrap(),
    );
    for (name, bytes) in files {
        zip.start_file(name, zip::write::SimpleFileOptions::default())?;
        zip.write_all(&bytes)?;
    }
    Ok(zip.finish()?.into_inner())
}
pub fn theme_pack_name(theme: &Theme) -> String {
    let id = slug(&serde_json::json!(theme.id));
    format!("theme-{}.zip", if id.is_empty() { "theme" } else { &id })
}
