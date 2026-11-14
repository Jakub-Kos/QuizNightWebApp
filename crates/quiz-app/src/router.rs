//! React-compatible hash routing. Encoding is deliberately independent of browser APIs.
use leptos::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Library,
    Drive(Option<String>),
    Help(Option<String>),
    Themes,
    Show(String),
    Presenter(String),
    Edit(String),
    Questions(String),
    Theme { id: String, fresh: bool },
    Check(String),
    Map(String),
    Attendance(String),
    Scores(String),
}
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
fn decode(s: &str) -> String {
    let mut bytes = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if s.as_bytes()[i] == b'%'
            && i + 2 < s.len()
            && let Ok(b) = std::str::from_utf8(&s.as_bytes()[i + 1..i + 3])
                .ok()
                .and_then(|s| u8::from_str_radix(s, 16).ok())
                .ok_or(())
        {
            bytes.push(b);
            i += 3;
            continue;
        }
        bytes.push(s.as_bytes()[i]);
        i += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}
pub fn parse(hash: &str) -> Route {
    let parts: Vec<_> = hash
        .trim_start_matches('#')
        .split('/')
        .filter(|s| !s.is_empty())
        .map(decode)
        .collect();
    let second = || parts.get(1).cloned();
    match parts.first().map(String::as_str) {
        Some("drive") => Route::Drive(second()),
        Some("help") => Route::Help(second()),
        Some("themes") => Route::Themes,
        Some("quiz") if parts.len() > 1 => {
            let id = parts[1].clone();
            match parts.get(2).map(String::as_str) {
                Some("presenter") => Route::Presenter(id),
                Some("edit") => Route::Edit(id),
                Some("questions") => Route::Questions(id),
                Some("check") => Route::Check(id),
                Some("map") => Route::Map(id),
                Some("attendance") => Route::Attendance(id),
                Some("scores") => Route::Scores(id),
                Some("theme") => Route::Theme {
                    id,
                    fresh: parts.get(3).is_some_and(|s| s == "new"),
                },
                _ => Route::Show(id),
            }
        }
        _ => Route::Library,
    }
}
impl Route {
    pub fn hash(&self) -> String {
        let (id, suffix) = match self {
            Self::Library => return "#/".into(),
            Self::Drive(folder) => {
                return format!(
                    "#/drive{}",
                    folder
                        .as_ref()
                        .map(|s| format!("/{}", encode(s)))
                        .unwrap_or_default()
                );
            }
            Self::Help(section) => {
                return format!(
                    "#/help{}",
                    section
                        .as_ref()
                        .map(|s| format!("/{}", encode(s)))
                        .unwrap_or_default()
                );
            }
            Self::Themes => return "#/themes".into(),
            Self::Show(id) => (id, ""),
            Self::Presenter(id) => (id, "/presenter"),
            Self::Edit(id) => (id, "/edit"),
            Self::Questions(id) => (id, "/questions"),
            Self::Theme { id, fresh } => (id, if *fresh { "/theme/new" } else { "/theme" }),
            Self::Check(id) => (id, "/check"),
            Self::Map(id) => (id, "/map"),
            Self::Attendance(id) => (id, "/attendance"),
            Self::Scores(id) => (id, "/scores"),
        };
        format!("#/quiz/{}{suffix}", encode(id))
    }
}
pub fn navigate(route: Route) {
    let _ = window().location().set_hash(&route.hash());
}
pub fn current_route() -> RwSignal<Route> {
    let route = RwSignal::new(parse(&window().location().hash().unwrap_or_default()));
    let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
        route.set(parse(&window().location().hash().unwrap_or_default()))
    });
    let _ =
        window().add_event_listener_with_callback("hashchange", listener.as_ref().unchecked_ref());
    let listener = StoredValue::new_local(Some(listener));
    on_cleanup(move || {
        if let Some(Some(listener)) = listener.try_update_value(Option::take) {
            let _ = window().remove_event_listener_with_callback(
                "hashchange",
                listener.as_ref().unchecked_ref(),
            );
        }
    });
    route
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn routes_round_trip() {
        let id = "a/b % ž?".to_string();
        for route in [
            Route::Library,
            Route::Drive(None),
            Route::Drive(Some(id.clone())),
            Route::Help(None),
            Route::Help(Some(id.clone())),
            Route::Themes,
            Route::Show(id.clone()),
            Route::Presenter(id.clone()),
            Route::Edit(id.clone()),
            Route::Questions(id.clone()),
            Route::Theme {
                id: id.clone(),
                fresh: true,
            },
            Route::Theme {
                id: id.clone(),
                fresh: false,
            },
            Route::Check(id.clone()),
            Route::Map(id.clone()),
            Route::Attendance(id.clone()),
            Route::Scores(id),
        ] {
            assert_eq!(parse(&route.hash()), route);
        }
    }
    #[test]
    fn malformed_encoding() {
        assert_eq!(parse("#/quiz/%ž"), Route::Show("%ž".into()));
    }
    #[test]
    fn react_defaults() {
        assert_eq!(parse("#/quiz/demo/unknown"), Route::Show("demo".into()));
        assert_eq!(parse("#/unknown"), Route::Library);
        assert_eq!(
            parse("#quiz/a%2Fb/presenter"),
            Route::Presenter("a/b".into())
        );
        assert_eq!(
            parse("#/help/start/ignored"),
            Route::Help(Some("start".into()))
        );
    }
}
