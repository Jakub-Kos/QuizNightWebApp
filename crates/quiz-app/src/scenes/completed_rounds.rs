use leptos::prelude::*;

fn key(id: &str) -> String {
    format!("quiznight.completed.{id}")
}
fn decode(json: &str) -> Vec<String> {
    serde_json::from_str(json).unwrap_or_default()
}
pub fn toggle(ids: &mut Vec<String>, id: &str) {
    if ids.iter().any(|v| v == id) {
        ids.retain(|v| v != id);
    } else {
        ids.push(id.into());
    }
}
pub fn use_completed(quiz: &str) -> RwSignal<Vec<String>> {
    let key = StoredValue::new(key(quiz));
    let read = move || {
        window()
            .local_storage()
            .ok()
            .flatten()
            .and_then(|s| s.get_item(&key.get_value()).ok().flatten())
            .map(|v| decode(&v))
            .unwrap_or_default()
    };
    let ids = RwSignal::new(read());
    if let Ok(Some(storage)) = window().local_storage() {
        let _ = storage.remove_item("quizCompletedRounds");
    }
    // StorageEvent is not enabled in web-sys; read its key through JS reflection.
    let listener = window_event_listener(leptos::ev::storage, move |event| {
        let value = js_sys::Reflect::get(event.as_ref(), &"key".into()).ok();
        if value.and_then(|v| v.as_string()).as_deref() == Some(&key.get_value()) {
            ids.set(read());
        }
    });
    on_cleanup(move || listener.remove());
    Effect::new(move |_| {
        let value = serde_json::to_string(&ids.get()).unwrap_or_else(|_| "[]".into());
        if let Ok(Some(storage)) = window().local_storage() {
            let _ = storage.set_item(&key.get_value(), &value);
        }
    });
    ids
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn storage_format_and_toggle() {
        assert_eq!(key("quiz-a"), "quiznight.completed.quiz-a");
        assert!(decode("broken").is_empty());
        let mut ids = decode(r#"["round-1","round-2"]"#);
        toggle(&mut ids, "round-1");
        toggle(&mut ids, "round-3");
        assert_eq!(
            serde_json::to_string(&ids).unwrap(),
            r#"["round-2","round-3"]"#
        );
    }
}
