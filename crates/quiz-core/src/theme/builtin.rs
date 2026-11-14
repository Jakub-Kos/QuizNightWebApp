use super::format::{Font, STYLE_OPTIONS, Theme};
use std::{collections::BTreeMap, sync::OnceLock};
pub const DEFAULT_THEME_ID: &str = "default";
pub const CUSTOM_THEME_ID: &str = "custom";
pub fn themes() -> &'static [Theme] {
    static THEMES: OnceLock<Vec<Theme>> = OnceLock::new();
    THEMES.get_or_init(|| serde_json::from_str(include_str!("builtin.json")).unwrap())
}
pub fn theme_by_id(id: &str) -> &'static Theme {
    themes().iter().find(|t| t.id == id).unwrap_or(&themes()[0])
}
pub fn theme_for_quiz<'a>(id: &str, custom: Option<&'a Theme>) -> &'a Theme {
    if id == CUSTOM_THEME_ID {
        custom.unwrap_or_else(|| theme_by_id(id))
    } else {
        theme_by_id(id)
    }
}
pub fn theme_choices(custom: Option<&Theme>) -> Vec<(&str, &Theme)> {
    let mut choices: Vec<_> = themes().iter().map(|t| (t.id.as_str(), t)).collect();
    if let Some(t) = custom {
        choices.push((CUSTOM_THEME_ID, t));
    }
    choices
}
pub fn theme_name<'a>(theme: &'a Theme, lang: &str) -> &'a str {
    theme
        .name
        .get(lang)
        .or_else(|| theme.name.get("en"))
        .or_else(|| theme.name.values().next())
        .map(String::as_str)
        .unwrap_or("")
}
pub fn font_family(font: &Font) -> String {
    match font {
        Font::Family { family } => family.clone(),
        Font::File { file } => {
            let hash = file
                .chars()
                .fold(2166136261u32, |h, c| (h ^ c as u32).wrapping_mul(16777619));
            let mut n = hash;
            let mut digits = Vec::new();
            loop {
                digits.push(char::from_digit(n % 36, 36).unwrap());
                n /= 36;
                if n == 0 {
                    break;
                }
            }
            format!("qn-font-{}", digits.into_iter().rev().collect::<String>())
        }
    }
}
pub fn theme_style(theme: &Theme) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for name in ["primary", "accent"] {
        if let Some(base) = theme.colors.get(name) {
            out.insert(format!("--color-{name}-500"), base.clone());
            for (step, amount) in [(50, 10), (100, 20), (200, 36), (300, 56), (400, 78)] {
                out.insert(
                    format!("--color-{name}-{step}"),
                    format!("color-mix(in oklab, {base} {amount}%, white)"),
                );
            }
            for (step, amount) in [(600, 16), (700, 32), (800, 46), (900, 58), (950, 72)] {
                out.insert(
                    format!("--color-{name}-{step}"),
                    format!("color-mix(in oklab, {base}, black {amount}%)"),
                );
            }
            let rgb: Vec<_> = [1, 3, 5]
                .map(|i| u8::from_str_radix(&base[i..i + 2], 16).unwrap())
                .into();
            out.insert(
                format!("--glow-{name}"),
                rgb.iter().map(u8::to_string).collect::<Vec<_>>().join(" "),
            );
            if name == "primary" {
                out.insert(
                    "--glow-primary-deep".into(),
                    rgb.iter()
                        .map(|v| ((*v as f64 * 0.45).round() as u8).to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }
    }
    for (key, css) in [
        ("ground", "ground"),
        ("groundTop", "ground-top"),
        ("groundBottom", "ground-bottom"),
        ("panel", "panel"),
    ] {
        if let Some(v) = theme.colors.get(key) {
            out.insert(format!("--color-{css}"), v.clone());
        }
    }
    for (role, key, suffix) in [
        ("body", "--theme-font", "\"League Spartan\""),
        (
            "title",
            "--theme-title-font",
            "var(--theme-font, \"League Spartan\")",
        ),
    ] {
        if let Some(font) = theme.fonts.get(role) {
            out.insert(key.into(), format!("\"{}\", {suffix}", font_family(font)));
        }
    }
    out
}
pub fn style_classes(theme: &Theme) -> String {
    let value = serde_json::to_value(&theme.style).unwrap();
    STYLE_OPTIONS
        .iter()
        .filter_map(|(key, options)| {
            value[key]
                .as_str()
                .filter(|v| *v != options[0])
                .map(|v| format!("qn-{key}-{v}"))
        })
        .collect::<Vec<_>>()
        .join(" ")
}
