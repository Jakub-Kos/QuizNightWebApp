// Remove this allowance when scenes consume the translation API.
#![allow(dead_code)]

use leptos::prelude::*;
use std::{collections::BTreeMap, sync::LazyLock};
type Table = BTreeMap<String, String>;
static EN: LazyLock<Table> =
    LazyLock::new(|| serde_json::from_str(include_str!("../i18n/en.json")).unwrap());
static CS: LazyLock<Table> =
    LazyLock::new(|| serde_json::from_str(include_str!("../i18n/cs.json")).unwrap());
static SK: LazyLock<Table> =
    LazyLock::new(|| serde_json::from_str(include_str!("../i18n/sk.json")).unwrap());

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Cs,
    Sk,
}
impl Lang {
    pub fn from_code(code: &str) -> Self {
        match code {
            "cs" => Self::Cs,
            "sk" => Self::Sk,
            _ => Self::En,
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Cs => "cs",
            Self::Sk => "sk",
        }
    }
}
fn table(lang: Lang) -> &'static Table {
    match lang {
        Lang::En => &EN,
        Lang::Cs => &CS,
        Lang::Sk => &SK,
    }
}
fn lookup(lang: Lang, key: &str) -> String {
    table(lang)
        .get(key)
        .or_else(|| EN.get(key))
        .cloned()
        .unwrap_or_else(|| key.to_owned())
}
fn format_text(text: &str, vars: &[(&str, &str)]) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        if let Some(end) = rest.find('}') {
            let key = &rest[1..end];
            if !key.is_empty() && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
                out.push_str(
                    vars.iter()
                        .find(|(name, _)| *name == key)
                        .map(|(_, v)| *v)
                        .unwrap_or(""),
                );
                rest = &rest[end + 1..];
                continue;
            }
        }
        out.push('{');
        rest = &rest[1..];
    }
    out.push_str(rest);
    out
}
#[derive(Clone, Copy)]
pub struct T(Signal<Lang>);
impl T {
    pub fn get(self, key: &str) -> String {
        lookup(self.lang(), key)
    }
    pub fn fmt(self, key: &str, vars: &[(&str, &str)]) -> String {
        format_text(&self.get(key), vars)
    }
    pub fn lang(self) -> Lang {
        self.0.get()
    }
}
pub fn provide_translations(lang: Signal<Lang>) {
    provide_context(T(lang));
}
pub fn use_t() -> T {
    expect_context::<T>()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tables() {
        for lang in [Lang::En, Lang::Cs, Lang::Sk] {
            let entries = table(lang);
            let missing: Vec<_> = EN.keys().filter(|k| !entries.contains_key(*k)).collect();
            println!("{} missing: {:?}", lang.code(), missing);
            for key in entries.keys() {
                assert!(
                    EN.contains_key(key)
                        || matches!(
                            (lang, key.as_str()),
                            (
                                Lang::Cs,
                                "waiting_for_answer" | "correct_order" | "back_to_menu"
                            ) | (Lang::Sk, "click_to_enter")
                        ),
                    "{}: {}",
                    lang.code(),
                    key
                );
            }
        }
    }
    #[test]
    fn formatting() {
        assert_eq!(format_text("Hi {name}", &[("name", "Ada")]), "Hi Ada");
        assert_eq!(
            format_text("{a} {b}", &[("a", "one"), ("b", "two")]),
            "one two"
        );
        assert_eq!(
            format_text("{name}/{name}/{other}/{bad-name}/{}", &[("name", "Ada")]),
            "Ada/Ada//{bad-name}/{}"
        );
    }
    #[test]
    fn fallbacks() {
        assert_eq!(lookup(Lang::Cs, "missing_key"), "missing_key");
        assert_eq!(Lang::from_code("unknown"), Lang::En);
        for lang in [Lang::En, Lang::Cs, Lang::Sk] {
            assert_eq!(Lang::from_code(lang.code()), lang);
        }
        let empty = Table::new();
        assert_eq!(
            empty.get("title").or_else(|| EN.get("title")).unwrap(),
            "Quiz Night"
        );
    }
}

// Extra local-first question importer labels; shared JSON dictionaries remain compatible with React.
impl T {
    pub fn questions_editor(self, key: &str) -> String {
        let (en, cs) = match key {
            "qe_csv_import" => ("Import questions from CSV", "Importovat otázky z CSV"),
            "qe_csv_replace" => ("Import replaces all questions. Local edits become the saved copy; import the public Sheet again to refresh it.", "Import nahradí všechny otázky. Místní úpravy se uloží do kvízu; veřejný list aktualizujete dalším importem."),
            "qe_csv_url" => ("Public Google Sheet or CSV URL", "Odkaz na veřejný Google list nebo CSV"),
            "qe_csv_paste" => ("Paste CSV", "Vložit CSV"),
            "qe_csv_load" => ("Load and replace questions", "Načíst a nahradit otázky"),
            "qe_csv_invalid" => ("No question rows found. Use the CSV template and a public link.", "Nebyly nalezeny žádné otázky. Použijte šablonu CSV a veřejný odkaz."),
            _ => return self.get(key),
        };
        if self.lang() == Lang::Cs { cs } else { en }.into()
    }
}
