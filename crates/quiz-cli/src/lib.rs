//! The theme workshop command, a port of scripts/workshop.mjs.
//!
//! `index` checks every theme in the workshop folder and writes index.json, `add` adds a submitted
//! theme pack after review, `check` prints the Markdown review report the theme-check workflow
//! comments on a submission. Output, exit codes and written files follow the Node script; the
//! differences that remain are listed in README.md.
use quiz_core::theme::format::{
    Asset, Font, NormalizeOptions, Problem, normalize_theme_with_options, readability_issues,
    theme_files,
};
use quiz_core::theme::pack::{PackAsset, ThemePack};
use quiz_core::theme::workshop::index_entry;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path};

pub type Pack = ThemePack;

/// Downloads stop at this size, and a pack may not unpack to more (a zip can claim huge contents).
pub const MAX_PACK: u64 = 30 * 1024 * 1024;

/// What fetches a URL: its bytes, or the text the Node script shows for the failure (a pack error
/// such as "the file is too big", or what `String(err)` gives for other errors).
pub type Fetch<'a> = &'a dyn Fn(&str) -> Result<Vec<u8>, String>;

/// A file that is not a theme pack, with the reason shown to the submitter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackError(pub String);
impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for PackError {}

/// Text from a submission, safe in a comment: no Markdown or HTML, no table breaks, no @-mentions.
pub fn md(text: &str) -> String {
    let mut result = String::new();
    let mut whitespace = false;
    for c in text.chars() {
        if js_space(c) {
            if !whitespace {
                result.push(' ');
            }
            whitespace = true;
            continue;
        }
        whitespace = false;
        if "\\`*_[]<>|#~".contains(c) {
            result.push('\\');
        }
        result.push(c);
        if c == '@' {
            result.push('\u{200b}');
        }
    }
    result
}

/// JavaScript's `\s`: WhiteSpace and LineTerminator (U+FEFF but not U+0085, unlike Unicode's).
fn js_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

/// The .zip attached to a submission: GitHub puts attachments in the body as links to
/// user-attachments. The same pattern as the Node script, which matches ASCII letters in any case.
pub fn find_pack_url(body: &str) -> Option<String> {
    static PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let pattern = PATTERN.get_or_init(|| {
        regex::Regex::new(concat!(
            r"https://(?:github\.com/user-attachments/files|(?:[a-z0-9_-]+\.)?githubusercontent\.com)/",
            r"[^\t\n\x0B\x0C\r \u{a0}\u{1680}\u{2000}-\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff})",
            "\"'<>`]+\\.zip"
        ))
        .unwrap()
    });
    // ASCII lowercasing keeps byte offsets, so the match can be cut from the original text
    let lower = body.to_ascii_lowercase();
    let found = pattern.find(&lower)?;
    Some(body[found.range()].to_owned())
}

/// A zip entry's name as fflate decodes it: UTF-8 when the entry says so, Latin-1 otherwise.
fn entry_name(bytes: &[u8], file: &zip::read::ZipFile<'_, Cursor<&[u8]>>) -> String {
    let raw = file.name_raw();
    let at = file.central_header_start() as usize + 8;
    let utf8 = bytes
        .get(at..at + 2)
        .is_some_and(|f| u16::from_le_bytes([f[0], f[1]]) & 0x800 != 0);
    if utf8 {
        let name = String::from_utf8_lossy(raw);
        name.strip_prefix('\u{feff}').unwrap_or(&name).to_owned()
    } else {
        raw.iter().map(|&b| char::from(b)).collect()
    }
}

/// A key JavaScript orders before all others in an object: a canonical array index.
fn array_index(key: &str) -> Option<u32> {
    let n: u32 = key.parse().ok()?;
    (n != u32::MAX && n.to_string() == key).then_some(n)
}

/// A theme pack's bytes -> theme, problems and assets; PackError when it is not one.
///
/// Entries are kept as in the Node script, where fflate returns them as a JavaScript object: a
/// name that comes twice keeps its first place and its last contents, names that are array indexes
/// come first, and `__proto__` is lost.
pub fn read_pack(bytes: &[u8]) -> Result<Pack, PackError> {
    let not_zip = || PackError("this is not a .zip file".into());
    let too_much = || PackError("the pack unpacks to too much data".into());
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| not_zip())?;
    // The sizes the archive declares, as the Node script counts them
    let mut declared = 0u64;
    for i in 0..zip.len() {
        declared = declared.saturating_add(zip.by_index_data(i).map_err(|_| not_zip())?.size());
        if declared > MAX_PACK {
            return Err(too_much());
        }
    }
    // What the entries really unpack to, which can be more than declared
    let mut total = 0u64;
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for i in 0..zip.len() {
        let file = zip.by_index(i).map_err(|_| not_zip())?;
        let name = entry_name(bytes, &file);
        let mut data = Vec::new();
        file.take(MAX_PACK - total + 1)
            .read_to_end(&mut data)
            .map_err(|_| not_zip())?;
        total += data.len() as u64;
        if total > MAX_PACK {
            return Err(too_much());
        }
        if name == "__proto__" {
            continue;
        }
        match files.iter_mut().find(|(n, _)| *n == name) {
            Some(existing) => existing.1 = data,
            None => files.push((name, data)),
        }
    }
    files.sort_by_key(|(name, _)| array_index(name).map_or((1, 0), |n| (0, n)));
    let mut entries: Vec<PackAsset> = files
        .into_iter()
        .filter(|(path, _)| {
            !path.ends_with('/')
                && !path.contains("__MACOSX/")
                && !path.rsplit('/').next().unwrap_or("").starts_with('.')
        })
        .map(|(name, bytes)| PackAsset { name, bytes })
        .collect();
    let index = entries
        .iter()
        .position(|e| {
            e.name
                .rsplit('/')
                .next()
                .unwrap_or("")
                .to_lowercase()
                .eq("theme.json")
        })
        .ok_or_else(|| PackError("the pack has no theme.json".into()))?;
    let meta = entries.remove(index);
    let base = meta
        .name
        .rsplit_once('/')
        .map(|(p, _)| format!("{p}/"))
        .unwrap_or_default();
    for e in &mut entries {
        if let Some(rest) = e.name.strip_prefix(&base) {
            e.name = rest.to_owned();
        }
    }
    // TextDecoder: invalid UTF-8 becomes U+FFFD and a byte order mark is dropped
    let text = String::from_utf8_lossy(&meta.bytes);
    let raw = serde_json::from_str(text.strip_prefix('\u{feff}').unwrap_or(&text))
        .map_err(|_| PackError("theme.json is not valid JSON".into()))?;
    let normalized = normalize_theme_with_options(
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
    Ok(Pack {
        theme: normalized
            .theme
            .ok_or_else(|| PackError("theme.json is not a theme".into()))?,
        problems: normalized.problems,
        assets: entries,
    })
}

/// The problems that leave part of a theme out (an unused file only stays out of the workshop).
fn blocking(problems: &[Problem]) -> Vec<&Problem> {
    problems
        .iter()
        .filter(|p| p.code != "unused_file")
        .collect()
}

const HEADER: &str = "<!-- workshop-check -->\n### Theme check\n\n";

/// The report when the submission has no theme file.
pub fn report_missing() -> String {
    format!(
        "{HEADER}❌ No theme file found. Drag the downloaded `theme-….zip` into the issue (edit it), and the check runs again."
    )
}

/// The report when the attached file could not be read as a theme pack.
pub fn report_error(error: &str) -> String {
    format!(
        "{HEADER}❌ The attached file could not be checked: {error}.\n\nDownload the theme again from the theme editor (Share to the workshop) and attach the new file."
    )
}

/// The review report for a submission, as Markdown (the marker lets the workflow update its own
/// comment). `exists`: a theme with this id is already in the workshop.
pub fn report(source: &str, pack: &Pack, exists: bool) -> String {
    let t = &pack.theme;
    let dropped = blocking(&pack.problems);
    let readability = readability_issues(t);
    let mut lines = vec![
        "<!-- workshop-check -->".to_owned(),
        "### Theme check".into(),
        String::new(),
    ];
    lines.push(
        if !dropped.is_empty() {
            "❌ **Does not pass.** These parts would be left out:"
        } else if !readability.is_empty() {
            "⚠️ **Passes, but may be hard to read on a TV.**"
        } else {
            "✅ **Passes the check.**"
        }
        .into(),
    );
    lines.push(String::new());
    for p in &dropped {
        lines.push(format!("- `{}`{}", p.code, detail(&md(&p.detail))));
    }
    if !dropped.is_empty() {
        lines.push(String::new());
    }
    let or_dash = |s: String| if s.is_empty() { "—".into() } else { s };
    let fonts = ["title", "body"]
        .iter()
        .filter_map(|r| {
            t.fonts.get(*r).map(|f| match f {
                Font::Family { family } => format!("{r}: {family}"),
                Font::File { file } => format!("{r}: file {}", md(file)),
            })
        })
        .collect::<Vec<_>>()
        .join(", ");
    let used = theme_files(t);
    let files = pack
        .assets
        .iter()
        .filter(|a| used.contains(&a.name))
        .map(|a| format!("{} ({})", md(&a.name), kb(a.bytes.len())))
        .collect::<Vec<_>>()
        .join(", ");
    let name = ["en", "cs", "sk"]
        .into_iter()
        .filter_map(|l| t.name.get(l).map(|n| format!("{} ({l})", md(n))))
        .collect::<Vec<_>>()
        .join(", ");
    let tags = t
        .tags
        .iter()
        .map(|s| format!("#{s}"))
        .collect::<Vec<_>>()
        .join(" ");
    let word = |v: serde_json::Value| v.as_str().unwrap_or("").to_owned();
    lines.extend([
        "| | |".into(),
        "| --- | --- |".into(),
        format!("| Name | {name} |"),
        format!("| Author | {} |", or_dash(md(&t.author))),
        format!(
            "| Id | `{}`{} |",
            t.id,
            if exists {
                " ⚠️ already in the workshop: adding it replaces that theme"
            } else {
                ""
            }
        ),
        format!("| Tags | {} |", or_dash(tags)),
        format!(
            "| Fonts | {} |",
            if fonts.is_empty() {
                "Classic".into()
            } else {
                fonts
            }
        ),
        format!(
            "| Scenery, decoration | {}, {} |",
            or_dash(word(serde_json::json!(t.background.ornament))),
            or_dash(word(serde_json::json!(t.decoration.r#type)))
        ),
        format!("| Files | {} |", or_dash(files)),
        String::new(),
    ]);
    if !readability.is_empty() {
        let descriptions = readability
            .iter()
            .map(|c| match *c {
                "text_ground" => "white text is hard to read on the background",
                "text_panel" => "white text is hard to read on the cards",
                "accent_text" => "black text is hard to read on highlight buttons",
                "primary_text" => "white text is hard to read on main-colour buttons",
                "image" => "the background picture is very visible behind text",
                _ => "undefined",
            })
            .collect::<Vec<_>>()
            .join("; ");
        lines.extend([format!("Readability: {descriptions}."), String::new()]);
    }
    let unused = pack
        .problems
        .iter()
        .filter(|p| p.code == "unused_file")
        .map(|p| md(&p.detail))
        .collect::<Vec<_>>();
    if !unused.is_empty() {
        lines.extend([
            format!(
                "Not used by the theme (left out when added): {}.",
                unused.join(", ")
            ),
            String::new(),
        ]);
    }
    if dropped.is_empty() {
        lines.extend([
            "To review it, download the file and import it in a quiz (Look → Import theme). To add it:".into(),
            String::new(),
            "```".into(),
            format!("cargo run -p quiz-cli -- add {source}{}", if exists { " --replace" } else { "" }),
            "```".into(),
        ]);
    }
    lines.join("\n")
}

fn detail(text: &str) -> String {
    if text.is_empty() {
        String::new()
    } else {
        format!(": {text}")
    }
}

/// Math.round(n / 1024), at least 1.
fn kb(n: usize) -> String {
    format!("{} KB", ((n as f64 / 1024.0 + 0.5).floor() as u64).max(1))
}

fn describe(problems: &[&Problem]) -> String {
    problems
        .iter()
        .map(|p| format!("  - {}{}", p.code, detail(&p.detail)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every file below a folder, with paths relative to it, in the order Node's readdirSync gives.
fn list_files(dir: &Path, prefix: &str, out: &mut Vec<(String, u64)>) -> std::io::Result<()> {
    for name in sorted_names(dir)? {
        let path = dir.join(&name);
        let meta = fs::metadata(&path)?;
        if meta.is_dir() {
            list_files(&path, &format!("{prefix}{name}/"), out)?;
        } else {
            out.push((format!("{prefix}{name}"), meta.len()));
        }
    }
    Ok(())
}

fn sorted_names(dir: &Path) -> std::io::Result<Vec<String>> {
    let mut names = fs::read_dir(dir)?
        .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    Ok(names)
}

/// A filesystem error worded as Node words it in `String(err)`, for the common cases.
fn node_error(err: &std::io::Error, syscall: &str, path: &Path) -> String {
    use std::io::ErrorKind::*;
    let path = path.display();
    match err.kind() {
        NotFound => format!("ENOENT: no such file or directory, {syscall} '{path}'"),
        PermissionDenied => format!("EACCES: permission denied, {syscall} '{path}'"),
        IsADirectory => "EISDIR: illegal operation on a directory, read".into(),
        _ => format!("{err}, {syscall} '{path}'"),
    }
}

/// The checked theme of one workshop folder.
fn check_folder(
    themes: &Path,
    id: &str,
    err: &mut dyn Write,
) -> Result<quiz_core::theme::format::Theme, String> {
    let dir = themes.join(id);
    let mut files = Vec::new();
    list_files(&dir, "", &mut files).map_err(|e| format!("{id}: {e}"))?;
    files.retain(|(name, _)| name != "theme.json");
    let path = dir.join("theme.json");
    let raw: serde_json::Value = fs::read(&path)
        .map_err(|e| node_error(&e, "open", &path))
        .and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string()))
        .map_err(|e| format!("{id}: theme.json cannot be read ({e})"))?;
    let normalized = normalize_theme_with_options(
        &raw,
        &NormalizeOptions {
            stored: false,
            assets: files
                .into_iter()
                .map(|(name, size)| Asset {
                    name,
                    size: size as f64,
                })
                .collect(),
        },
    );
    let theme = normalized
        .theme
        .ok_or_else(|| format!("{id}: theme.json is not a theme"))?;
    if !normalized.problems.is_empty() {
        let all: Vec<_> = normalized.problems.iter().collect();
        return Err(format!(
            "{id}: the theme does not pass the check:\n{}",
            describe(&all)
        ));
    }
    if theme.id != id {
        return Err(format!(
            "{id}: the folder must be named after the theme's id ({})",
            theme.id
        ));
    }
    let readability = readability_issues(&theme);
    if !readability.is_empty() {
        let _ = writeln!(err, "! {id}: hard to read: {}", readability.join(", "));
    }
    Ok(theme)
}

/// Checks every theme in the workshop folder and writes index.json. New themes are dated `today`
/// (YYYY-MM-DD); the others keep the date in the previous index. Returns the number of themes.
pub fn write_index(
    workshop: &Path,
    today: &str,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<usize, String> {
    let themes_dir = workshop.join("themes");
    let index = workshop.join("index.json");
    fs::create_dir_all(&themes_dir).map_err(|e| e.to_string())?;
    let previous: serde_json::Value = if index.exists() {
        let bytes = fs::read(&index).map_err(|e| node_error(&e, "open", &index))?;
        serde_json::from_slice(&bytes).map_err(|e| format!("index.json: {e}"))?
    } else {
        serde_json::json!({ "themes": [] })
    };
    let previous = previous["themes"]
        .as_array()
        .ok_or("index.json: themes is not a list")?;
    let mut entries = Vec::new();
    for id in sorted_names(&themes_dir).map_err(|e| e.to_string())? {
        if !fs::metadata(themes_dir.join(&id)).is_ok_and(|m| m.is_dir()) {
            continue;
        }
        let theme = check_folder(&themes_dir, &id, err)?;
        // a Map from the previous entries: the last one with an id wins
        let added_at = previous
            .iter()
            .rev()
            .find(|e| e["theme"]["id"] == id.as_str())
            .and_then(|e| e["addedAt"].as_str())
            .unwrap_or(today);
        entries.push(index_entry(&theme, added_at));
    }
    entries.sort_by(|a, b| {
        b.added_at
            .cmp(&a.added_at)
            .then(a.theme.id.cmp(&b.theme.id))
    });
    let mut json = js_json::Json::from(&serde_json::json!({ "version": 1, "themes": entries }));
    json.order_keys(&["version", "themes"]);
    if let Some(list) = json.get_mut("themes") {
        for entry in list.items_mut() {
            entry.order_keys(&["path", "addedAt", "theme"]);
            if let Some(theme) = entry.get_mut("theme") {
                theme.order_theme();
            }
        }
    }
    fs::write(&index, json.pretty()).map_err(|e| node_error(&e, "open", &index))?;
    let count = entries.len();
    let _ = writeln!(
        out,
        "✓ {count} workshop theme{} checked, index written",
        if count == 1 { "" } else { "s" }
    );
    Ok(count)
}

/// Reads a pack from a file or an http(s) URL; the error text is what the Node script shows.
pub fn load_pack(source: &str, fetch: Fetch<'_>) -> Result<Pack, String> {
    let bytes = if source.starts_with("http://") || source.starts_with("https://") {
        fetch(source)?
    } else {
        fs::read(source)
            .map_err(|e| format!("Error: {}", node_error(&e, "open", Path::new(source))))?
    };
    read_pack(&bytes).map_err(|e| e.0)
}

/// A path from a pack that stays inside the folder it is written to: relative, no `..`.
pub fn stays_inside(name: &str) -> bool {
    !name.is_empty()
        && Path::new(name)
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
}

/// Adds a submitted theme pack to the workshop and writes the index again. The theme must pass the
/// check without dropping anything; `replace` overwrites a theme with the same id.
pub fn add(
    workshop: &Path,
    source: Option<&str>,
    replace: bool,
    today: &str,
    fetch: Fetch<'_>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<(), String> {
    let source = source.ok_or("give the theme pack: a .zip file or its URL")?;
    let pack = load_pack(source, fetch)?;
    let theme = &pack.theme;
    let dropped = blocking(&pack.problems);
    if !dropped.is_empty() {
        return Err(format!(
            "the theme does not pass the check (ask the author to fix it, or fix theme.json yourself):\n{}",
            describe(&dropped)
        ));
    }
    let used = theme_files(theme);
    let assets: Vec<_> = pack
        .assets
        .iter()
        .filter(|a| used.contains(&a.name))
        .collect();
    // Not in the Node script, which would write such a file outside the theme's folder
    if let Some(a) = assets.iter().find(|a| !stays_inside(&a.name)) {
        return Err(format!(
            "the theme refers to a file outside its folder ({}); fix the pack before adding it",
            a.name
        ));
    }
    let dir = workshop.join("themes").join(&theme.id);
    if dir.exists() {
        if !replace {
            return Err(format!(
                "a theme with the id \"{}\" is already in the workshop (add --replace to overwrite it)",
                theme.id
            ));
        }
        let removed = match fs::symlink_metadata(&dir) {
            Ok(m) if m.is_dir() => fs::remove_dir_all(&dir),
            _ => fs::remove_file(&dir),
        };
        removed.map_err(|e| node_error(&e, "rm", &dir))?;
    }
    for a in assets {
        let path = dir.join(&a.name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| node_error(&e, "mkdir", parent))?;
        }
        fs::write(&path, &a.bytes).map_err(|e| node_error(&e, "open", &path))?;
    }
    fs::create_dir_all(&dir).map_err(|e| node_error(&e, "mkdir", &dir))?;
    let path = dir.join("theme.json");
    fs::write(&path, theme_json(theme)).map_err(|e| node_error(&e, "open", &path))?;
    let _ = writeln!(
        out,
        "✓ added \"{}\" by {} as public/workshop/themes/{}/",
        theme.name.get("en").unwrap_or(&theme.id),
        if theme.author.is_empty() {
            "(no author)"
        } else {
            &theme.author
        },
        theme.id
    );
    write_index(workshop, today, out, err).map(|_| ())
}

/// theme.json as the Node script writes it: `JSON.stringify(theme, null, 2)` and a newline.
pub fn theme_json(theme: &quiz_core::theme::format::Theme) -> String {
    let mut json = js_json::Json::from(&serde_json::to_value(theme).unwrap_or_default());
    json.order_theme();
    json.pretty()
}

/// The review report for a pack (`None`: no file was found in the submission).
pub fn check(workshop: &Path, source: Option<&str>, fetch: Fetch<'_>) -> String {
    let Some(source) = source else {
        return report_missing();
    };
    match load_pack(source, fetch) {
        Ok(pack) => {
            let exists = workshop.join("themes").join(&pack.theme.id).exists();
            report(source, &pack, exists)
        }
        Err(e) => report_error(&e),
    }
}

/// Downloads a pack as the Node script does with fetch: redirects followed (up to 20), the
/// download failed unless the final status is 2xx, too big over MAX_PACK bytes (by its declared
/// length, or by what arrives). Like the Node script it takes any http(s) URL: the host check is
/// `find_pack_url`, which picks the URL from a submission.
pub fn download(url: &str) -> Result<Vec<u8>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .max_redirects(20)
        .http_status_as_error(false)
        .build()
        .into();
    let mut response = agent.get(url).call().map_err(|e| match e {
        ureq::Error::Http(_) | ureq::Error::BadUri(_) => {
            format!("TypeError: Failed to parse URL from {url}")
        }
        _ => "TypeError: fetch failed".to_owned(),
    })?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("the download failed (HTTP {})", status.as_u16()));
    }
    let length = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<f64>().ok())
        .unwrap_or(0.0);
    if length > MAX_PACK as f64 {
        return Err("the file is too big".into());
    }
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_PACK + 1)
        .read_to_vec()
        .map_err(|e| match e {
            ureq::Error::BodyExceedsLimit(_) => "the file is too big".to_owned(),
            _ => "TypeError: terminated".to_owned(),
        })?;
    if bytes.len() as u64 > MAX_PACK {
        return Err("the file is too big".into());
    }
    Ok(bytes)
}

const USAGE: &str = "usage: quiz-cli index | add <zip or URL> [--replace] | check <zip or URL> | check --from-issue";

/// The command line, as `quiz-cli <args>`: returns the exit code. Arguments are
/// read as the Node script reads them (the source is the first that does not start with `--`,
/// unknown flags are ignored). `--dir <folder>` (Rust only) is the workshop folder, by default
/// public/workshop below the current folder. `issue_body`: $ISSUE_BODY.
pub fn run(
    args: &[String],
    issue_body: Option<&str>,
    today: &str,
    fetch: Fetch<'_>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    let mut workshop = std::path::PathBuf::from("public/workshop");
    let mut rest: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if let Some(dir) = args[i].strip_prefix("--dir=") {
            workshop = dir.into();
        } else if args[i] == "--dir" && i + 1 < args.len() {
            workshop = args[i + 1].as_str().into();
            i += 1;
        } else {
            rest.push(&args[i]);
        }
        i += 1;
    }
    // Node's folder is absolute (it is found from the script), and so are paths in its messages
    let workshop = std::path::absolute(&workshop).unwrap_or(workshop);
    let command = rest.first().copied();
    let args = rest.get(1..).unwrap_or(&[]);
    let source = args.iter().copied().find(|a| !a.starts_with("--"));
    let result = match command {
        Some("index") => write_index(&workshop, today, out, err).map(|_| ()),
        Some("add") => add(
            &workshop,
            source,
            args.contains(&"--replace"),
            today,
            fetch,
            out,
            err,
        ),
        Some("check") => {
            let source = if args.contains(&"--from-issue") {
                find_pack_url(issue_body.unwrap_or(""))
            } else {
                source.map(str::to_owned)
            };
            let _ = writeln!(out, "{}", check(&workshop, source.as_deref(), fetch));
            Ok(())
        }
        _ => Err(USAGE.into()),
    };
    match result {
        Ok(()) => 0,
        Err(message) => {
            let _ = writeln!(err, "✗ {message}");
            1
        }
    }
}

/// Today's date in UTC as YYYY-MM-DD, as `new Date().toISOString().slice(0, 10)`.
pub fn utc_today() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() / 86400) as i64;
    // civil-from-days (Howard Hinnant)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// JSON with its keys in a chosen order, printed as `JSON.stringify(value, null, 2)`.
///
/// quiz-core keeps a theme's names, descriptions, colours and fonts in sorted maps, and serde_json
/// sorts object keys, while the Node script writes them in the order normalizeTheme adds them.
mod js_json {
    use serde::ser::{SerializeMap, SerializeSeq};
    use serde_json::Value;

    pub enum Json {
        Plain(Value),
        List(Vec<Json>),
        Object(Vec<(String, Json)>),
    }

    impl From<&Value> for Json {
        fn from(v: &Value) -> Self {
            match v {
                Value::Array(a) => Json::List(a.iter().map(Json::from).collect()),
                Value::Object(o) => {
                    Json::Object(o.iter().map(|(k, v)| (k.clone(), Json::from(v))).collect())
                }
                other => Json::Plain(other.clone()),
            }
        }
    }

    impl Json {
        pub fn get_mut(&mut self, key: &str) -> Option<&mut Json> {
            match self {
                Json::Object(o) => o.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
                _ => None,
            }
        }
        pub fn items_mut(&mut self) -> std::slice::IterMut<'_, Json> {
            match self {
                Json::List(l) => l.iter_mut(),
                _ => [].iter_mut(),
            }
        }
        /// Puts the keys in this order; keys not listed keep their order after them.
        pub fn order_keys(&mut self, order: &[&str]) {
            if let Json::Object(o) = self {
                o.sort_by_key(|(k, _)| order.iter().position(|o| o == k).unwrap_or(order.len()));
            }
        }
        /// A theme's keys in the order normalizeTheme builds it.
        pub fn order_theme(&mut self) {
            use quiz_core::theme::format::COLOR_KEYS;
            self.order_keys(&[
                "format",
                "version",
                "id",
                "name",
                "author",
                "description",
                "tags",
                "colors",
                "fonts",
                "background",
                "decoration",
                "style",
                "celebration",
                "sounds",
                "scenes",
            ]);
            let langs = ["en", "cs", "sk"];
            let colors: Vec<&str> = COLOR_KEYS.to_vec();
            for (key, order) in [
                ("name", &langs[..]),
                ("description", &langs[..]),
                ("colors", &colors[..]),
                ("fonts", &["title", "body"][..]),
                (
                    "background",
                    &["image", "imageOpacity", "imageBlur", "glows", "ornament"][..],
                ),
                ("style", &["corners", "cards", "headings", "effects"][..]),
                (
                    "scenes",
                    &["leaderboard", "dashboard", "round", "welcome"][..],
                ),
            ] {
                if let Some(v) = self.get_mut(key) {
                    v.order_keys(order);
                }
            }
            if let Some(glows) = self.get_mut("background").and_then(|b| b.get_mut("glows")) {
                for glow in glows.items_mut() {
                    glow.order_keys(&["color", "x", "y", "size", "opacity"]);
                }
            }
        }
        /// Two-space indented, with a final newline.
        pub fn pretty(&self) -> String {
            let mut text = serde_json::to_string_pretty(self).unwrap_or_default();
            text.push('\n');
            text
        }
    }

    impl serde::Serialize for Json {
        fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            match self {
                Json::Plain(v) => v.serialize(s),
                Json::List(l) => {
                    let mut seq = s.serialize_seq(Some(l.len()))?;
                    for item in l {
                        seq.serialize_element(item)?;
                    }
                    seq.end()
                }
                Json::Object(o) => {
                    let mut map = s.serialize_map(Some(o.len()))?;
                    for (k, v) in o {
                        map.serialize_entry(k, v)?;
                    }
                    map.end()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escapes_hostile_text() {
        assert_eq!(
            md("@owner <img src=x> | **bold** [link](u)"),
            "@\u{200b}owner \\<img src=x\\> \\| \\*\\*bold\\*\\* \\[link\\](u)"
        );
        // JavaScript's \s has U+FEFF but not U+0085
        assert_eq!(md("a\u{feff}\u{85}b"), "a \u{85}b");
    }
    #[test]
    fn attachments_only() {
        assert_eq!(find_pack_url("https://evil.example.com/x.zip"), None);
        assert_eq!(find_pack_url(""), None);
        assert_eq!(find_pack_url("https://githubusercontent.com/.zip"), None);
        assert_eq!(
            find_pack_url("https://a.b.githubusercontent.com/x.zip"),
            None
        );
        let url = "https://github.com/user-attachments/files/123456/theme-neon.zip";
        assert_eq!(find_pack_url(&format!("[theme]({url})")), Some(url.into()));
        assert_eq!(
            find_pack_url("HTTPS://GitHub.com/user-attachments/files/1/A.ZIP.zip x"),
            Some("HTTPS://GitHub.com/user-attachments/files/1/A.ZIP.zip".into())
        );
    }
    #[test]
    fn rejects_non_zip() {
        assert!(read_pack(&[1, 2, 3]).unwrap_err().0.contains("not a .zip"));
    }
    #[test]
    fn javascript_rounding() {
        assert_eq!(kb(0), "1 KB");
        assert_eq!(kb(1535), "1 KB");
        assert_eq!(kb(1536), "2 KB");
    }
    #[test]
    fn paths_inside() {
        for ok in ["a.png", "assets/a.png", "./a.png", "a/./b.png", "a\\b.png"] {
            assert!(stays_inside(ok), "{ok}");
        }
        for bad in ["", "../a.png", "a/../../b.png", "/etc/a.png", "a/.."] {
            assert!(!stays_inside(bad), "{bad}");
        }
    }
    #[test]
    fn dates() {
        let today = utc_today();
        assert_eq!(today.len(), 10);
        assert!(today.as_str() > "2026-01-01");
    }
}
