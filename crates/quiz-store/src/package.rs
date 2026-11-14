//! React-compatible quiz archives. Caller supplies a fresh id (normally a UUID) and timestamp.
use crate::{Error, Media, MediaStore, QuizStore, Result};
use quiz_core::{
    quiz::{model::Quiz, parse::media_key},
    theme::format::{NormalizeOptions, normalize_theme_with_options},
};
use std::io::{Cursor, Read, Write};

// theme::pack currently has no archive-level caps/path checks to share. These protect allocations
// before decompression; theme validation itself remains in quiz-core, with its existing limits.
pub const MAX_ARCHIVE_BYTES: usize = 512 * 1024 * 1024;
pub const MAX_ENTRY_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 10_000;

#[derive(Debug, Clone)]
pub struct Import {
    pub quiz: Quiz,
    pub media: Vec<Media>,
}
fn base(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}
fn safe(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains(['\\', ':', '\0'])
        && path.split('/').all(|p| p != ".." && p != ".")
}
fn junk(path: &str) -> bool {
    path.starts_with("__MACOSX/") || path.contains("/__MACOSX/") || base(path).starts_with('.')
}
pub fn media_type(name: &str) -> Option<&'static str> {
    Some(match name.rsplit('.').next()?.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        _ => return None,
    })
}
fn invalid(message: &str) -> Error {
    Error::Package(message.into())
}

pub async fn export_zip<S: MediaStore>(store: &S, quiz: &Quiz) -> Result<Vec<u8>> {
    let mut meta = serde_json::to_value(quiz)?;
    for key in ["id", "createdAt", "updatedAt", "builtin", "questionsCsv"] {
        meta.as_object_mut().unwrap().remove(key);
    }
    let media = store.list_media(&quiz.id).await?;
    let mut entries = vec![
        ("quiz.json".into(), serde_json::to_vec_pretty(&meta)?),
        (
            "questions.csv".into(),
            quiz.questions_csv.as_bytes().to_vec(),
        ),
    ];
    for file in media {
        entries.push((format!("media/{}", file.name), file.bytes));
    }
    if entries.len() > MAX_ENTRIES {
        return Err(invalid("too many entries"));
    }
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut total = 0;
    for (path, bytes) in entries {
        total += bytes.len();
        if !safe(&path) || bytes.len() > MAX_ENTRY_BYTES || total > MAX_ARCHIVE_BYTES {
            return Err(invalid("unsafe path or archive too large"));
        }
        zip.start_file(
            path,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored),
        )?;
        zip.write_all(&bytes)?;
    }
    Ok(zip.finish()?.into_inner())
}

pub fn read_zip(bytes: &[u8], fresh_id: impl Into<String>, now_ms: i64) -> Result<Import> {
    if bytes.len() > MAX_ARCHIVE_BYTES {
        return Err(invalid("archive too large"));
    }
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;
    if zip.len() > MAX_ENTRIES {
        return Err(invalid("too many entries"));
    }
    let mut entries = Vec::new();
    let mut total = 0usize;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let path = file.name()?.into_owned();
        if !safe(&path) {
            return Err(invalid("unsafe archive path"));
        }
        if file.is_dir() || junk(&path) {
            continue;
        }
        if file.size() > MAX_ENTRY_BYTES as u64 {
            return Err(invalid("entry too large"));
        }
        let mut data = Vec::new();
        (&mut file)
            .take(MAX_ENTRY_BYTES as u64 + 1)
            .read_to_end(&mut data)?;
        total += data.len();
        if data.len() > MAX_ENTRY_BYTES || total > MAX_ARCHIVE_BYTES {
            return Err(invalid("archive too large"));
        }
        entries.push((path, data));
    }
    read_entries(entries, fresh_id, now_ms)
}
/// Also accepts loose files/folder entries; paths use '/' as in webkitRelativePath.
pub fn read_entries(
    entries: Vec<(String, Vec<u8>)>,
    fresh_id: impl Into<String>,
    now_ms: i64,
) -> Result<Import> {
    if entries.len() > MAX_ENTRIES {
        return Err(invalid("too many entries"));
    }
    let mut total = 0usize;
    for (path, bytes) in &entries {
        total = total.saturating_add(bytes.len());
        if !safe(path) || bytes.len() > MAX_ENTRY_BYTES || total > MAX_ARCHIVE_BYTES {
            return Err(invalid("unsafe path or archive too large"));
        }
    }
    let entries: Vec<_> = entries
        .into_iter()
        .filter(|(p, _)| !junk(p) && !p.ends_with('/'))
        .collect();
    let meta = entries
        .iter()
        .find(|(p, _)| base(p).eq_ignore_ascii_case("quiz.json"));
    let csv = entries
        .iter()
        .find(|(p, _)| base(p).eq_ignore_ascii_case("questions.csv"))
        .or_else(|| {
            entries
                .iter()
                .find(|(p, _)| p.to_lowercase().ends_with(".csv"))
        });
    let raw: serde_json::Value = match meta {
        Some((_, b)) => serde_json::from_slice(b)?,
        None => serde_json::json!({}),
    };
    let mut selected = serde_json::Map::new();
    // createQuiz deliberately does not import id, timestamps, builtin, drive or unknown metadata.
    for key in [
        "title",
        "questionsSheetUrl",
        "scoresSheetUrl",
        "teams",
        "rules",
        "prizes",
        "manualScores",
        "settings",
    ] {
        if let Some(value) = raw.get(key) {
            if value.is_null()
                || (key == "teams" && !value.is_array())
                || (key == "manualScores" && !value.is_object())
            {
                continue;
            }
            selected.insert(key.into(), value.clone());
        }
    }
    if let Some(raw) = raw.get("customTheme")
        && let Some(theme) = normalize_theme_with_options(
            raw,
            &NormalizeOptions {
                stored: true,
                assets: vec![],
            },
        )
        .theme
    {
        selected.insert("customTheme".into(), serde_json::to_value(theme)?);
    }
    let mut quiz: Quiz = serde_json::from_value(selected.into())?;
    quiz.id = fresh_id.into();
    if quiz.id.is_empty() {
        return Err(invalid("fresh id is empty"));
    }
    quiz.created_at = now_ms;
    quiz.updated_at = now_ms;
    if let Some((path, bytes)) = csv {
        quiz.questions_csv = String::from_utf8_lossy(bytes).into_owned();
        if quiz.title.is_empty() {
            quiz.title = base(path)[..base(path).len() - 4].into();
        }
    }
    // JS Map preserves first insertion position but replaces the value on a normalized-name clash.
    let mut media: Vec<Media> = Vec::new();
    for (path, bytes) in entries {
        if let Some(kind) = media_type(&path) {
            let file = Media {
                name: base(&path).into(),
                bytes,
                content_type: kind.into(),
                fields: Default::default(),
            };
            if let Some(i) = media
                .iter()
                .position(|m| media_key(&m.name) == media_key(&path))
            {
                media[i] = file;
            } else {
                media.push(file);
            }
        }
    }
    Ok(Import { quiz, media })
}
/// The caller must allocate a fresh id for every import. This check guards accidental reuse;
/// it is not a cross-client uniqueness transaction. Saving quiz+media spans two transactions, as in JS.
pub async fn save_import<S: QuizStore + MediaStore>(store: &S, import: &Import) -> Result<String> {
    if store.get_quiz(&import.quiz.id).await?.is_some() {
        return Err(Error::Conflict(import.quiz.id.clone()));
    }
    store.put_quiz(&import.quiz).await?;
    store.put_media(&import.quiz.id, &import.media).await?;
    Ok(import.quiz.id.clone())
}

/// Local file references used by the editor's missing-media display and demo asset loading.
pub fn referenced_media(quiz: &Quiz) -> Vec<String> {
    use quiz_core::quiz::parse::{KnownQuestionData, QuestionData, is_url, parse_questions_csv};
    let mut names = Vec::new();
    let mut add = |name: &str| {
        if !name.is_empty() && !is_url(name) && !names.iter().any(|n| n == name) {
            names.push(name.to_owned());
        }
    };
    for round in parse_questions_csv(&quiz.questions_csv) {
        for question in round.questions {
            if let Some(name) = question.media {
                add(&name);
            }
            match question.data {
                QuestionData::Known(KnownQuestionData::Sort { items, .. }) => {
                    for item in items {
                        if let Some(name) = item.media {
                            add(&name);
                        }
                    }
                }
                QuestionData::Known(KnownQuestionData::Handout { answer }) => add(answer.trim()),
                _ => {}
            }
        }
    }
    for team in &quiz.teams {
        if let Some(name) = &team.image {
            add(name);
        }
    }
    if let Some(raw) = quiz.fields.get("customTheme")
        && let Some(theme) = normalize_theme_with_options(
            raw,
            &NormalizeOptions {
                stored: true,
                assets: vec![],
            },
        )
        .theme
    {
        for name in quiz_core::theme::format::theme_files(&theme) {
            add(&name);
        }
    }
    names
}

pub async fn duplicate_quiz<S: QuizStore + MediaStore>(
    store: &S,
    quiz: &Quiz,
    title: &str,
    fresh_id: &str,
    now_ms: i64,
) -> Result<String> {
    let mut import = read_zip(&export_zip(store, quiz).await?, fresh_id, now_ms)?;
    import.quiz.title = title.into();
    save_import(store, &import).await
}
