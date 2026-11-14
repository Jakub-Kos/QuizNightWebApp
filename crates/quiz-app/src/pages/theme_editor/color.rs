use super::{BUTTON, Draft, INPUT, value_at};
use leptos::prelude::*;
use serde_json::json;

const BRIGHTS: &[&str] = &[
    "#3b82f6", "#00b4d8", "#14b8a6", "#22c55e", "#84cc16", "#eab308", "#f5c542", "#ff7a1a",
    "#ef4444", "#e0303a", "#ec4899", "#a855f7", "#8b3dff", "#6366f1", "#94a3b8", "#ffffff",
];
const DARKS: &[&str] = &[
    "#050505", "#111111", "#0b0614", "#1d0c30", "#06150e", "#0f2a1d", "#06101d", "#0b1d33",
    "#1a0a0a", "#2a1410",
];

#[component]
pub(super) fn ColorField(
    draft: Draft,
    path: String,
    label: String,
    fallback: &'static str,
    #[prop(optional)] dark: bool,
) -> impl IntoView {
    let t = crate::i18n::use_t();
    let path = StoredValue::new(path);
    let value = Signal::derive(move || {
        path.with_value(|p| {
            value_at(
                &serde_json::to_value(super::normalized(&draft.raw.get())).unwrap(),
                p,
            )
            .as_str()
            .unwrap_or(fallback)
            .to_owned()
        })
    });
    let open = RwSignal::new(false);
    let box_ref = NodeRef::<leptos::html::Div>::new();
    let pointer = window_event_listener(leptos::ev::pointerdown, move |ev| {
        if let Some(element) = box_ref.get() {
            use wasm_bindgen::JsCast;
            if !ev
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Node>().ok())
                .is_some_and(|target| element.contains(Some(&target)))
            {
                open.set(false);
            }
        }
    });
    let key = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            open.set(false);
        }
    });
    on_cleanup(move || {
        pointer.remove();
        key.remove();
    });
    let set = Callback::new(move |color: String| {
        let color = if color.starts_with('#') {
            color
        } else {
            format!("#{color}")
        };
        if matches!(color.len(), 4 | 7) && color[1..].bytes().all(|b| b.is_ascii_hexdigit()) {
            path.with_value(|p| draft.set(p, json!(color.to_lowercase())));
        }
    });
    view! {
        <div node_ref=box_ref class="relative flex items-center gap-3">
            <button type="button" class="w-11 h-11 rounded-lg ring-1 ring-white/20 shrink-0" style=move || format!("background:{}", value.get()) aria-label=label.clone() aria-expanded=move || open.get() on:click=move |_| open.update(|v| *v = !*v)/>
            <span class="text-sm flex-1">{label.clone()}</span>
            <button type="button" class=BUTTON title=t.get("te_reset_color") on:click=move |_| path.with_value(|p| draft.set(p, serde_json::Value::Null))>"↺"</button>
            <Show when=move || open.get()>
                <div class="absolute left-0 top-full mt-2 z-50 w-64 p-3 rounded-2xl bg-[#16161a] border border-white/15 shadow-2xl space-y-3">
                    <button type="button" class=BUTTON on:click=move |_| open.set(false)>{t.get("chk_close")}</button>
                    <input type="color" class="w-full h-24" aria-label=label.clone() prop:value=move || value.get() on:input=move |ev| set.run(event_target_value(&ev))/>
                    <input type="text" class=INPUT aria-label=label.clone() maxlength="7" prop:value=move || value.get() on:change=move |ev| set.run(event_target_value(&ev))/>
                    <button type="button" class=BUTTON on:click=move |_| {
                        leptos::task::spawn_local(async move {
                            let picker = js_sys::Function::new_no_args("return typeof EyeDropper === 'function' ? new EyeDropper().open().then(r => r.sRGBHex) : Promise.resolve(null)");
                            if let Ok(result) = picker.call0(&wasm_bindgen::JsValue::NULL) {
                                if let Ok(result) = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&result)).await {
                                    if let Some(color) = result.as_string() { set.run(color); }
                                }
                            }
                        });
                    }>{t.get("cp_eyedropper")}</button>
                    <span class="text-xs text-gray-500">{t.get("cp_theme")}</span>
                    <div class="theme-color-swatches">{move || {
                        let theme = super::normalized(&draft.raw.get());
                        let mut colors = theme.colors.values().cloned().chain(theme.background.glows.into_iter().map(|g| g.color)).collect::<Vec<_>>();
                        colors.sort(); colors.dedup();
                        colors.into_iter().take(8).map(|color| { let chosen = color.clone(); view! { <button type="button" style=format!("background:{color}") title=color.clone() aria-label=color on:click=move |_| set.run(chosen.clone())/> } }).collect_view()
                    }}</div>
                    <span class="text-xs text-gray-500">{t.get("cp_suggested")}</span>
                    <div class="theme-color-swatches">{(if dark { DARKS } else { BRIGHTS }).iter().map(|&color| view! { <button type="button" style=format!("background:{color}") title=color aria-label=color on:click=move |_| set.run(color.into())/> }).collect_view()}</div>
                </div>
            </Show>
        </div>
    }
}
