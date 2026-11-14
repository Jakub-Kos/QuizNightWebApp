use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ornament {
    #[serde(rename = "graveyard")]
    Graveyard,
    #[serde(rename = "lights")]
    Lights,
    #[serde(rename = "skyline")]
    Skyline,
    #[serde(rename = "waves")]
    Waves,
    #[serde(rename = "meadow")]
    Meadow,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decoration {
    #[serde(rename = "bats")]
    Bats,
    #[serde(rename = "snow")]
    Snow,
    #[serde(rename = "leaves")]
    Leaves,
    #[serde(rename = "petals")]
    Petals,
    #[serde(rename = "hearts")]
    Hearts,
    #[serde(rename = "confetti")]
    Confetti,
    #[serde(rename = "bubbles")]
    Bubbles,
    #[serde(rename = "stars")]
    Stars,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Celebration {
    #[serde(rename = "confetti")]
    Confetti,
    #[serde(rename = "fireworks")]
    Fireworks,
    #[serde(rename = "decoration")]
    Decoration,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundPack {
    #[serde(rename = "chimes")]
    Chimes,
    #[serde(rename = "spooky")]
    Spooky,
    #[serde(rename = "festive")]
    Festive,
    #[serde(rename = "arcade")]
    Arcade,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderboardLayout {
    #[serde(rename = "table")]
    Table,
    #[serde(rename = "podium")]
    Podium,
    #[serde(rename = "race")]
    Race,
    #[serde(rename = "spotlight")]
    Spotlight,
    #[serde(rename = "medals")]
    Medals,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardLayout {
    #[serde(rename = "columns")]
    Columns,
    #[serde(rename = "tiles")]
    Tiles,
    #[serde(rename = "timeline")]
    Timeline,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoundLayout {
    #[serde(rename = "classic")]
    Classic,
    #[serde(rename = "curtain")]
    Curtain,
    #[serde(rename = "slam")]
    Slam,
    #[serde(rename = "marquee")]
    Marquee,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WelcomeIntro {
    #[serde(rename = "flip")]
    Flip,
    #[serde(rename = "broadcast")]
    Broadcast,
    #[serde(rename = "spotlight")]
    Spotlight,
    #[serde(rename = "polaroid")]
    Polaroid,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorKey {
    #[serde(rename = "primary")]
    Primary,
    #[serde(rename = "accent")]
    Accent,
    #[serde(rename = "groundTop")]
    GroundTop,
    #[serde(rename = "ground")]
    Ground,
    #[serde(rename = "groundBottom")]
    GroundBottom,
    #[serde(rename = "panel")]
    Panel,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "en")]
    En,
    #[serde(rename = "cs")]
    Cs,
    #[serde(rename = "sk")]
    Sk,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Corners {
    #[serde(rename = "rounded")]
    Rounded,
    #[serde(rename = "sharp")]
    Sharp,
    #[serde(rename = "round")]
    Round,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cards {
    #[serde(rename = "solid")]
    Solid,
    #[serde(rename = "glass")]
    Glass,
    #[serde(rename = "outline")]
    Outline,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Headings {
    #[serde(rename = "spaced")]
    Spaced,
    #[serde(rename = "plain")]
    Plain,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effects {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "neon")]
    Neon,
    #[serde(rename = "flat")]
    Flat,
}
pub const THEME_FORMAT: &str = "quiznight-theme";
pub const THEME_VERSION: u32 = 1;

pub const DISPLAY_FONTS: &[&str] = &[
    "Rubik Wet Paint",
    "Berkshire Swash",
    "Bebas Neue",
    "Pacifico",
    "Bungee",
    "Monoton",
    "Special Elite",
    "Audiowide",
];
pub const TEXT_FONTS: &[&str] = &[
    "Grenze",
    "Playfair Display",
    "Montserrat",
    "Nunito",
    "Oswald",
];

/// WCAG contrast ratio for six-digit hexadecimal theme colours.
/// Invalid colours produce NaN, matching JavaScript's numeric propagation.
pub fn contrast(a: &str, b: &str) -> f64 {
    fn luminance(color: &str) -> f64 {
        [1, 3, 5]
            .into_iter()
            .zip([0.2126, 0.7152, 0.0722])
            .map(|(i, weight)| {
                let channel = color
                    .get(i..i + 2)
                    .and_then(|s| u8::from_str_radix(s, 16).ok())
                    .map_or(f64::NAN, |v| f64::from(v) / 255.0);
                weight
                    * if channel <= 0.03928 {
                        channel / 12.92
                    } else {
                        ((channel + 0.055) / 1.055).powf(2.4)
                    }
            })
            .sum()
    }
    let (a, b) = (luminance(a), luminance(b));
    if a >= b {
        (a + 0.05) / (b + 0.05)
    } else {
        (b + 0.05) / (a + 0.05)
    }
}

/// Readability warnings in the same order as the editor's warnings.
fn readability_value(theme: &serde_json::Value) -> Vec<&'static str> {
    let color =
        |key: &str, fallback: &str| theme["colors"][key].as_str().unwrap_or(fallback).to_owned();
    let mut issues = Vec::new();
    if [
        ("groundTop", "#111111"),
        ("ground", "#050505"),
        ("groundBottom", "#000000"),
    ]
    .into_iter()
    .any(|(key, fallback)| contrast("#ffffff", &color(key, fallback)) < 7.0)
    {
        issues.push("text_ground");
    }
    if contrast("#ffffff", &color("panel", "#0a0a0a")) < 7.0 {
        issues.push("text_panel");
    }
    if contrast("#000000", &color("accent", "#eab308")) < 4.5 {
        issues.push("accent_text");
    }
    let primary = color("primary", "#3b82f6");
    let shaded: Option<Vec<_>> = [1, 3, 5]
        .into_iter()
        .map(|i| {
            primary
                .get(i..i + 2)
                .and_then(|s| u8::from_str_radix(s, 16).ok())
                .map(|v| format!("{:02x}", (f64::from(v) * 0.84).round() as u8))
        })
        .collect();
    if let Some(channels) = shaded
        && contrast("#ffffff", &format!("#{}", channels.join(""))) < 3.0
    {
        issues.push("primary_text");
    }
    let bg = &theme["background"];
    if bg["image"].as_str().is_some_and(|s| !s.is_empty())
        && bg["imageOpacity"].as_f64().unwrap_or(0.0) > 0.6
        && bg["imageBlur"].as_f64().unwrap_or(f64::NAN) < 4.0
    {
        issues.push("image");
    }
    issues
}

use serde_json::{Value, json};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub format: String,
    pub version: u32,
    pub id: String,
    pub name: BTreeMap<String, String>,
    pub author: String,
    pub description: BTreeMap<String, String>,
    pub tags: Vec<String>,
    pub colors: BTreeMap<String, String>,
    pub fonts: BTreeMap<String, Font>,
    pub background: Background,
    pub decoration: DecorationSettings,
    pub style: Style,
    pub celebration: CelebrationSettings,
    pub sounds: Sounds,
    pub scenes: Scenes,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Font {
    File { file: String },
    Family { family: String },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Background {
    pub image: Option<String>,
    #[serde(serialize_with = "serialize_number")]
    pub image_opacity: f64,
    #[serde(serialize_with = "serialize_number")]
    pub image_blur: f64,
    pub glows: Vec<Glow>,
    pub ornament: Option<Ornament>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glow {
    pub color: String,
    #[serde(serialize_with = "serialize_number")]
    pub x: f64,
    #[serde(serialize_with = "serialize_number")]
    pub y: f64,
    #[serde(serialize_with = "serialize_number")]
    pub size: f64,
    #[serde(serialize_with = "serialize_number")]
    pub opacity: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecorationSettings {
    pub r#type: Option<Decoration>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelebrationSettings {
    pub r#type: Option<Celebration>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sounds {
    pub pack: Option<SoundPack>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corners: Option<Corners>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards: Option<Cards>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headings: Option<Headings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<Effects>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenes {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leaderboard: Option<Layout<LeaderboardLayout>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dashboard: Option<Layout<DashboardLayout>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round: Option<Layout<RoundLayout>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub welcome: Option<Intro>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout<T> {
    pub layout: T,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intro {
    pub intro: WelcomeIntro,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub code: String,
    pub detail: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Normalized {
    pub theme: Option<Theme>,
    pub problems: Vec<Problem>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    #[serde(serialize_with = "serialize_number")]
    pub size: f64,
}
#[derive(Debug, Clone, Default)]
pub struct NormalizeOptions {
    pub assets: Vec<Asset>,
    pub stored: bool,
}
fn js(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::String(s) => s.clone(),
        Value::Array(a) => a
            .iter()
            .map(|v| if v.is_null() { String::new() } else { js(v) })
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".into(),
        _ => v.to_string(),
    }
}
fn text(v: &Value, max: usize) -> String {
    if v.is_null() {
        return String::new();
    }
    js(v)
        .chars()
        .map(|c| if c < ' ' { ' ' } else { c })
        .collect::<String>()
        .trim()
        .encode_utf16()
        .take(max)
        .collect::<Vec<_>>()
        .as_slice()
        .pipe_utf16()
}
trait Utf16Text {
    fn pipe_utf16(&self) -> String;
}
impl Utf16Text for [u16] {
    fn pipe_utf16(&self) -> String {
        // JS preserves a split surrogate in JSON; Rust strings cannot, so use U+FFFD.
        String::from_utf16_lossy(self)
    }
}
pub fn slug(v: &Value) -> String {
    let mut out = String::new();
    let input = if v.is_null() { String::new() } else { js(v) };
    for c in input.to_lowercase().nfd() {
        if ('\u{300}'..='\u{36f}').contains(&c) {
            continue;
        }
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c)
        } else if !out.ends_with('-') {
            out.push('-')
        }
    }
    out.trim_matches('-').chars().take(40).collect()
}
fn texts(v: &Value, max: usize) -> Value {
    let mut o = json!({});
    for l in LANGS {
        let t = text(if v.is_string() { v } else { &v[l] }, max);
        if !t.is_empty() {
            o[l] = json!(t)
        }
    }
    o
}
fn number(v: &Value) -> f64 {
    match v {
        Value::Null => 0.,
        Value::Bool(b) => {
            if *b {
                1.
            } else {
                0.
            }
        }
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::Object(_) => f64::NAN,
        _ => {
            let s = js(v);
            let s = s.trim();
            if s.is_empty() {
                0.
            } else if s.starts_with("0x") || s.starts_with("0X") {
                u64::from_str_radix(&s[2..], 16).map_or(f64::NAN, |n| n as f64)
            } else {
                s.parse().unwrap_or(f64::NAN)
            }
        }
    }
}
fn clamp(v: &Value, min: f64, max: f64, default: f64) -> Value {
    let n = number(v);
    json!(if !v.is_null() && v != &json!("") && n.is_finite() {
        n.clamp(min, max)
    } else {
        default
    })
}
fn hex(v: &Value) -> Option<String> {
    let s = js(v).trim().to_lowercase();
    let h = s.strip_prefix('#')?;
    if !h.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    match h.len() {
        3 => Some(format!(
            "#{}",
            h.chars().flat_map(|c| [c, c]).collect::<String>()
        )),
        6 => Some(s),
        _ => None,
    }
}
fn problem(p: &mut Vec<Problem>, code: &str, v: &Value) {
    p.push(Problem {
        code: code.into(),
        detail: js(v),
    })
}
fn pick(v: &Value, options: &[&str], code: &str, p: &mut Vec<Problem>) -> Option<Value> {
    if v.is_null() {
        None
    } else if v.as_str().is_some_and(|s| options.contains(&s)) {
        Some(v.clone())
    } else {
        problem(p, code, v);
        None
    }
}
fn file(
    v: &Value,
    types: &[&str],
    limit: f64,
    o: &NormalizeOptions,
    used: &mut Vec<String>,
    p: &mut Vec<Problem>,
) -> Value {
    if v.is_null() || v == &json!("") {
        return Value::Null;
    }
    let path = js(v);
    let asset = if o.stored {
        Some(Asset {
            name: text(v, 200),
            size: 0.,
        })
    } else {
        o.assets
            .iter()
            .rev()
            .find(|a| a.name.to_lowercase() == path.to_lowercase())
            .cloned()
    };
    let code = match &asset {
        None => Some("missing_file"),
        Some(_)
            if !types.contains(
                &path
                    .rsplit('.')
                    .next()
                    .unwrap_or("")
                    .to_lowercase()
                    .as_str(),
            ) =>
        {
            Some("file_type")
        }
        Some(a) if a.size > limit => Some("file_size"),
        _ => None,
    };
    if let Some(c) = code {
        problem(p, c, v);
        Value::Null
    } else {
        let a = asset.unwrap();
        used.push(a.name.clone());
        json!(a.name)
    }
}
pub fn normalize_theme(raw: &Value) -> Normalized {
    normalize_theme_with_options(raw, &NormalizeOptions::default())
}
pub fn normalize_theme_with_options(raw: &Value, o: &NormalizeOptions) -> Normalized {
    let mut p = vec![];
    if !raw.is_object() || raw["format"] != THEME_FORMAT {
        return Normalized {
            theme: None,
            problems: vec![Problem {
                code: "format".into(),
                detail: String::new(),
            }],
        };
    }
    if number(&raw["version"]) > 1. {
        problem(&mut p, "version", &raw["version"])
    }
    let mut name = texts(&raw["name"], 80);
    let mut id = slug(&raw["id"]);
    if id.is_empty() {
        for l in LANGS {
            if let Some(n) = name[l].as_str() {
                id = slug(&json!(n));
                break;
            }
        }
    }
    if id.is_empty() {
        id = "theme".into()
    }
    if name.as_object().unwrap().is_empty() {
        name = json!({"en":"Theme","cs":"Motiv","sk":"Téma"})
    }
    let tags = raw["tags"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(slug)
                .filter(|s| !s.is_empty())
                .take(8)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut t = json!({"format":THEME_FORMAT,"version":1,"id":id,"name":name,"author":text(&raw["author"],80),"description":texts(&raw["description"],300),"tags":tags,"colors":{},"fonts":{},"background":{"image":null,"imageOpacity":0.5,"imageBlur":0,"glows":[],"ornament":null},"decoration":{"type":null},"style":{},"celebration":{"type":null},"sounds":{"pack":null},"scenes":{}});
    for k in COLOR_KEYS {
        let v = &raw["colors"][k];
        if !v.is_null() {
            if let Some(h) = hex(v) {
                t["colors"][k] = json!(h)
            } else {
                problem(&mut p, "color", &json!(k))
            }
        }
    }
    let mut used = vec![];
    for role in ["title", "body"] {
        let f = &raw["fonts"][role];
        if !f.is_object() {
            continue;
        }
        if !f["file"].is_null() {
            let v = file(&f["file"], FONT_TYPES, 2097152., o, &mut used, &mut p);
            if v.as_str().is_some_and(|s| !s.is_empty()) {
                t["fonts"][role] = json!({"file":v})
            }
        } else if !f["family"].is_null() {
            let family = js(&f["family"]).trim().to_lowercase();
            if let Some(b) = BUNDLED_FONTS.iter().find(|b| b.to_lowercase() == family) {
                t["fonts"][role] = json!({"family":b})
            } else {
                problem(&mut p, "font", &f["family"])
            }
        }
    }
    let bg = &raw["background"];
    t["background"]["image"] = file(&bg["image"], IMAGE_TYPES, 8388608., o, &mut used, &mut p);
    t["background"]["imageOpacity"] = clamp(&bg["imageOpacity"], 0., 1., 0.5);
    t["background"]["imageBlur"] = clamp(&bg["imageBlur"], 0., 20., 0.);
    if let Some(gs) = bg["glows"].as_array() {
        if gs.len() > 6 {
            problem(&mut p, "glows", &json!(gs.len()))
        }
        for g in gs.iter().take(6).filter(|g| g.is_object()) {
            if let Some(c) = hex(&g["color"]) {
                t["background"]["glows"].as_array_mut().unwrap().push(json!({"color":c,"x":clamp(&g["x"],0.,100.,50.),"y":clamp(&g["y"],0.,100.,50.),"size":clamp(&g["size"],5.,150.,50.),"opacity":clamp(&g["opacity"],0.,1.,0.2)}))
            } else {
                problem(&mut p, "color", &json!("glow"))
            }
        }
    }
    if let Some(v) = pick(
        &raw["background"]["ornament"],
        ORNAMENTS,
        "ornament",
        &mut p,
    ) {
        t["background"]["ornament"] = v;
    }
    if let Some(v) = pick(
        &raw["decoration"]["type"],
        DECORATIONS,
        "decoration",
        &mut p,
    ) {
        t["decoration"]["type"] = v;
    }
    if let Some(v) = pick(&raw["style"]["corners"], STYLE_CORNERS, "style", &mut p) {
        t["style"]["corners"] = v;
    }
    if let Some(v) = pick(&raw["style"]["cards"], STYLE_CARDS, "style", &mut p) {
        t["style"]["cards"] = v;
    }
    if let Some(v) = pick(&raw["style"]["headings"], STYLE_HEADINGS, "style", &mut p) {
        t["style"]["headings"] = v;
    }
    if let Some(v) = pick(&raw["style"]["effects"], STYLE_EFFECTS, "style", &mut p) {
        t["style"]["effects"] = v;
    }
    if let Some(v) = pick(
        &raw["celebration"]["type"],
        CELEBRATIONS,
        "celebration",
        &mut p,
    ) {
        t["celebration"]["type"] = v;
    }
    if let Some(v) = pick(&raw["sounds"]["pack"], SOUND_PACKS, "sounds", &mut p) {
        t["sounds"]["pack"] = v;
    }
    if let Some(v) = pick(
        &raw["scenes"]["leaderboard"]["layout"],
        LEADERBOARD_LAYOUTS,
        "layout",
        &mut p,
    ) {
        t["scenes"]["leaderboard"] = json!({"layout":v});
    }
    if let Some(v) = pick(
        &raw["scenes"]["dashboard"]["layout"],
        DASHBOARD_LAYOUTS,
        "layout",
        &mut p,
    ) {
        t["scenes"]["dashboard"] = json!({"layout":v});
    }
    if let Some(v) = pick(
        &raw["scenes"]["round"]["layout"],
        ROUND_LAYOUTS,
        "layout",
        &mut p,
    ) {
        t["scenes"]["round"] = json!({"layout":v});
    }
    if let Some(v) = pick(
        &raw["scenes"]["welcome"]["intro"],
        WELCOME_INTROS,
        "layout",
        &mut p,
    ) {
        t["scenes"]["welcome"] = json!({"intro":v});
    }
    if !o.stored {
        for a in &o.assets {
            if !used.contains(&a.name) {
                problem(&mut p, "unused_file", &json!(a.name))
            }
        }
    }
    Normalized {
        theme: Some(serde_json::from_value(t).expect("normalized theme")),
        problems: p,
    }
}
pub fn readability_issues(theme: &Theme) -> Vec<&'static str> {
    readability_value(&serde_json::to_value(theme).unwrap())
}
pub fn theme_files(theme: &Theme) -> Vec<String> {
    let mut files = vec![];
    for role in ["title", "body"] {
        if let Some(Font::File { file }) = theme.fonts.get(role) {
            files.push(file.clone())
        }
    }
    if let Some(f) = &theme.background.image {
        files.push(f.clone())
    }
    files
}
pub fn rename_theme_files(theme: &Theme, mut rename: impl FnMut(&str) -> Option<String>) -> Theme {
    let mut t = theme.clone();
    t.fonts.retain(|_, f| match f {
        Font::File { file } => {
            if let Some(n) = rename(file).filter(|s| !s.is_empty()) {
                *file = n;
                true
            } else {
                false
            }
        }
        _ => true,
    });
    t.background.image = t
        .background
        .image
        .as_deref()
        .and_then(&mut rename)
        .filter(|s| !s.is_empty());
    t
}
pub const ORNAMENTS: &[&str] = &["graveyard", "lights", "skyline", "waves", "meadow"];
pub const DECORATIONS: &[&str] = &[
    "bats", "snow", "leaves", "petals", "hearts", "confetti", "bubbles", "stars",
];
pub const CELEBRATIONS: &[&str] = &["confetti", "fireworks", "decoration"];
pub const SOUND_PACKS: &[&str] = &["chimes", "spooky", "festive", "arcade"];
pub const LEADERBOARD_LAYOUTS: &[&str] = &["table", "podium", "race", "spotlight", "medals"];
pub const DASHBOARD_LAYOUTS: &[&str] = &["columns", "tiles", "timeline"];
pub const ROUND_LAYOUTS: &[&str] = &["classic", "curtain", "slam", "marquee"];
pub const WELCOME_INTROS: &[&str] = &["flip", "broadcast", "spotlight", "polaroid"];
pub const COLOR_KEYS: &[&str] = &[
    "primary",
    "accent",
    "groundTop",
    "ground",
    "groundBottom",
    "panel",
];
pub const LANGS: &[&str] = &["en", "cs", "sk"];
pub const IMAGE_TYPES: &[&str] = &["jpg", "jpeg", "png", "webp", "avif"];
pub const FONT_TYPES: &[&str] = &["woff2", "woff", "ttf", "otf"];
pub const STYLE_CORNERS: &[&str] = &["rounded", "sharp", "round"];
pub const STYLE_CARDS: &[&str] = &["solid", "glass", "outline"];
pub const STYLE_HEADINGS: &[&str] = &["spaced", "plain"];
pub const STYLE_EFFECTS: &[&str] = &["normal", "neon", "flat"];
pub const BUNDLED_FONTS: &[&str] = &[
    "Rubik Wet Paint",
    "Berkshire Swash",
    "Bebas Neue",
    "Pacifico",
    "Bungee",
    "Monoton",
    "Special Elite",
    "Audiowide",
    "Grenze",
    "Playfair Display",
    "Montserrat",
    "Nunito",
    "Oswald",
];

fn serialize_number<S: serde::Serializer>(n: &f64, s: S) -> Result<S::Ok, S::Error> {
    if n.fract() == 0. && n.abs() < 9e15 {
        s.serialize_i64(*n as i64)
    } else {
        s.serialize_f64(*n)
    }
}

pub const FONT_GROUPS: &[(&str, &[&str])] = &[("display", DISPLAY_FONTS), ("text", TEXT_FONTS)];
pub const STYLE_OPTIONS: &[(&str, &[&str])] = &[
    ("corners", STYLE_CORNERS),
    ("cards", STYLE_CARDS),
    ("headings", STYLE_HEADINGS),
    ("effects", STYLE_EFFECTS),
];
pub struct Limits {
    pub image: usize,
    pub font: usize,
    pub glows: usize,
    pub tags: usize,
    pub text: usize,
    pub description: usize,
}
pub const LIMITS: Limits = Limits {
    image: 8388608,
    font: 2097152,
    glows: 6,
    tags: 8,
    text: 80,
    description: 300,
};
