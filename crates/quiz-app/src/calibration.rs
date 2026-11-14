//! Device-local calibration and the fullscreen reminder, outside the scaled stage.
use crate::settings::SettingsIcon;
use crate::stage::{Display, StageLayout, stage_layout};
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

fn bounded(value: f64, default: f64, min: f64, max: f64) -> f64 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        default
    }
}
pub fn normalized(display: Display) -> Display {
    Display {
        size: (bounded(display.size, 1.0, 0.6, 1.5) * 100.0).round() / 100.0,
        margin: bounded(display.margin, 0.0, 0.0, 10.0),
    }
}
fn decode(value: &str) -> Display {
    let v: serde_json::Value = serde_json::from_str(value).unwrap_or_default();
    normalized(Display {
        size: v["size"].as_f64().unwrap_or(1.0),
        margin: v["margin"].as_f64().unwrap_or(0.0),
    })
}
pub fn display_settings(role: quiz_core::show::Role) -> RwSignal<Display> {
    let key = match role {
        quiz_core::show::Role::Main => "quiznight.display.main",
        _ => "quiznight.display.presenter",
    };
    let initial = window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item(key).ok().flatten())
        .map(|s| decode(&s))
        .unwrap_or_default();
    let display = RwSignal::new(initial);
    Effect::new(move |_| {
        let d = normalized(display.get());
        if let Ok(Some(storage)) = window().local_storage() {
            let _ = storage.set_item(
                key,
                &serde_json::json!({"size":d.size,"margin":d.margin}).to_string(),
            );
        }
    });
    display
}
fn viewport() -> (f64, f64) {
    (
        window()
            .inner_width()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        window()
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
    )
}
fn fullscreen_geometry(outer: (f64, f64), inner: (f64, f64)) -> bool {
    outer.0 > 0.0 && inner.0 > 0.0 && outer.1 - inner.1 * (outer.0 / inner.0) < 8.0
}
fn fullscreen() -> bool {
    document().fullscreen_element().is_some()
        || fullscreen_geometry(
            (
                window()
                    .outer_width()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                window()
                    .outer_height()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            ),
            viewport(),
        )
}
fn call_method(target: &JsValue, method: &str) -> Option<JsValue> {
    js_sys::Reflect::get(target, &method.into())
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?
        .call0(target)
        .ok()
}
pub fn enter_fullscreen() {
    // Optional Keyboard Lock is not in web-sys; use feature detection just as React does.
    let Some(element) = document().document_element() else {
        return;
    };
    let Some(promise) = call_method(element.as_ref(), "requestFullscreen") else {
        return;
    };
    wasm_bindgen_futures::spawn_local(async move {
        if wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&promise))
            .await
            .is_err()
        {
            return;
        }
        let navigator: JsValue = window().navigator().into();
        if let Ok(keyboard) = js_sys::Reflect::get(&navigator, &"keyboard".into())
            && let Ok(lock) = js_sys::Reflect::get(&keyboard, &"lock".into())
            && let Some(lock) = lock.dyn_ref::<js_sys::Function>()
        {
            let keys = js_sys::Array::of1(&"Escape".into());
            if let Ok(result) = lock.call1(&keyboard, &keys) {
                let _ =
                    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&result)).await;
            }
        }
    });
}
fn toggle_fullscreen() {
    if document().fullscreen_element().is_some() {
        let _ = call_method(document().as_ref(), "exitFullscreen");
    } else {
        enter_fullscreen();
    }
}
#[component]
pub fn CalibrationPattern() -> impl IntoView {
    let t = crate::i18n::use_t();
    view! {
        <div class="absolute inset-0 z-[200] bg-[#050505] text-white font-['League_Spartan'] overflow-hidden">
            <div class="absolute inset-0 opacity-20" style="background-image:linear-gradient(to right,#fff 1px,transparent 1px),linear-gradient(to bottom,#fff 1px,transparent 1px);background-size:10% 10%"/>
            <div class="absolute inset-0 border-4 border-yellow-400"/>
            <div class="absolute w-24 h-24 border-yellow-400 top-0 left-0 border-t-[12px] border-l-[12px]"/>
            <div class="absolute w-24 h-24 border-yellow-400 top-0 right-0 border-t-[12px] border-r-[12px]"/>
            <div class="absolute w-24 h-24 border-yellow-400 bottom-0 left-0 border-b-[12px] border-l-[12px]"/>
            <div class="absolute w-24 h-24 border-yellow-400 bottom-0 right-0 border-b-[12px] border-r-[12px]"/>
            <div class="absolute inset-[5%] border-2 border-dashed border-white/40"/>
            <div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[400px] h-[400px] rounded-full border-4 border-blue-400"/>
            <div class="absolute left-1/2 top-0 bottom-0 w-px bg-blue-400/50"/>
            <div class="absolute top-1/2 left-0 right-0 h-px bg-blue-400/50"/>
            <div class="absolute top-[8%] left-0 right-0 text-center"><h1 class="text-7xl font-black uppercase tracking-widest text-yellow-400">{move || t.get("calib_title")}</h1><p class="text-3xl text-gray-300 mt-4">{move || t.get("calib_corners")}</p><p class="text-3xl text-gray-300 mt-2">{move || t.get("calib_circle")}</p></div>
            <div class="absolute bottom-[8%] left-[7%] space-y-2 text-left">{["text-2xl", "text-4xl", "text-6xl", "text-8xl"].into_iter().map(|class| view! { <div class=format!("{class} font-bold")>{move || t.get("calib_text_sample")}<span class="text-gray-500 text-xl font-mono">" "{class}</span></div> }).collect_view()}</div>
        </div>
    }
}
#[component]
pub fn CalibrationControls(display: RwSignal<Display>) -> impl IntoView {
    let t = crate::i18n::use_t();
    let ui = crate::show::use_show_ui();
    let vp = RwSignal::new(viewport());
    let _ = window_event_listener(leptos::ev::resize, move |_| vp.set(viewport()));
    let layout = move || stage_layout(vp.get(), display.get());
    let keys =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
            e.stop_propagation();
            let mut next = display.get_untracked();
            match e.key().as_str() {
                "Escape" | "Enter" => ui.calibrating.set(false),
                "+" | "=" | "ArrowUp" => next.size += 0.01,
                "-" | "ArrowDown" => next.size -= 0.01,
                "]" | "ArrowRight" => next.margin += 0.5,
                "[" | "ArrowLeft" => next.margin -= 0.5,
                "0" => next = Display::default(),
                "f" | "F" => toggle_fullscreen(),
                _ => return,
            }
            e.prevent_default();
            display.set(normalized(next));
        });
    let _ = window().add_event_listener_with_callback_and_bool(
        "keydown",
        keys.as_ref().unchecked_ref(),
        true,
    );
    let keys = StoredValue::new_local(Some(keys));
    on_cleanup(move || {
        if let Some(Some(keys)) = keys.try_update_value(Option::take) {
            let _ = window().remove_event_listener_with_callback_and_bool(
                "keydown",
                keys.as_ref().unchecked_ref(),
                true,
            );
        }
    });
    view! { <div class="fixed inset-0 z-[300] flex items-center justify-center pointer-events-none">
        <div class="pointer-events-auto w-[420px] max-w-[90vw] bg-[#053049]/95 backdrop-blur-md border border-gray-700 rounded-2xl shadow-2xl p-6 space-y-5 text-white font-sans text-sm">
            <div class="flex justify-between font-mono text-xs text-gray-400">
                <span>{move || { let (w,h) = vp.get(); format!("{}: {w}×{h} @{}x", t.get("calib_window"), window().device_pixel_ratio()) }}</span>
                <span>{move || { let StageLayout {w,h,..} = layout(); format!("{}: {w}×{h}", t.get("calib_stage")) }}</span>
            </div>
            <label class="block"><div class="flex justify-between mb-2"><span>{move || t.get("calib_scale")}</span><span class="font-mono text-yellow-400">{move || format!("{}%", (display.get().size * 100.0).round())}</span></div>
                <input type="range" min="60" max="150" prop:value=move || (display.get().size * 100.0).round() on:input=move |e| { if let Ok(v) = event_target_value(&e).parse::<f64>() { display.update(|d| *d = normalized(Display { size:v / 100.0, ..*d })); } } class="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-yellow-500"/>
            </label>
            <label class="block"><div class="flex justify-between mb-2"><span>{move || t.get("calib_margin")}</span><span class="font-mono text-blue-400">{move || format!("{}%", display.get().margin)}</span></div>
                <input type="range" min="0" max="10" step="0.5" prop:value=move || display.get().margin on:input=move |e| { if let Ok(v) = event_target_value(&e).parse::<f64>() { display.update(|d| *d = normalized(Display { margin:v, ..*d })); } } class="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-500"/>
            </label>
            <p class="text-xs text-gray-400 leading-relaxed">{move || t.get("calib_keys")}</p>
            <div class="flex gap-2"><button on:click=move |_| display.set(Display::default()) class="flex-1 py-3 rounded-xl bg-white/10 hover:bg-white/20 flex items-center justify-center gap-2"><SettingsIcon name="reset" size=16/>{move || t.get("calib_reset")}</button><button on:click=move |_| toggle_fullscreen() class="flex-1 py-3 rounded-xl bg-white/10 hover:bg-white/20 flex items-center justify-center gap-2"><SettingsIcon name="fullscreen" size=16/>{move || t.get("calib_fullscreen")}</button><button on:click=move |_| ui.calibrating.set(false) class="flex-1 py-3 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold flex items-center justify-center gap-2"><SettingsIcon name="check" size=16/>{move || t.get("calib_done")}</button></div>
        </div>
    </div> }
}
#[component]
pub fn FullscreenHint() -> impl IntoView {
    let t = crate::i18n::use_t();
    let ui = crate::show::use_show_ui();
    let full = RwSignal::new(fullscreen());
    let dismissed = RwSignal::new(false);
    let _ = window_event_listener(leptos::ev::resize, move |_| full.set(fullscreen()));
    let change = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| full.set(fullscreen()));
    let _ = document()
        .add_event_listener_with_callback("fullscreenchange", change.as_ref().unchecked_ref());
    let change = StoredValue::new_local(Some(change));
    on_cleanup(move || {
        if let Some(Some(change)) = change.try_update_value(Option::take) {
            let _ = document().remove_event_listener_with_callback(
                "fullscreenchange",
                change.as_ref().unchecked_ref(),
            );
        }
    });
    let agent = window().navigator().user_agent().unwrap_or_default();
    let key = if ["Mac", "iPhone", "iPad"].iter().any(|s| agent.contains(s)) {
        "⌃⌘F"
    } else {
        "F11"
    };
    view! { <crate::presence::Presence when=Signal::derive(move || !full.get() && !dismissed.get() && !ui.calibrating.get()) enter="fullscreen-enter" exit="fullscreen-exit"
        class="fullscreen-hint fixed top-4 left-1/2 z-[250] flex items-center gap-4 pl-5 pr-2 py-2 rounded-full bg-[#053049]/95 backdrop-blur-md border border-yellow-500/40 shadow-2xl text-white font-sans text-sm">
        <span>{move || t.get("fs_hint_before")}<kbd class="mx-1 px-2 py-0.5 rounded bg-white/15 border border-white/20 font-mono font-bold text-yellow-400">{key}</kbd>{move || t.get("fs_hint_after")}</span>
        <button on:click=move |_| enter_fullscreen() class="flex items-center gap-2 px-4 py-2 rounded-full bg-yellow-500 hover:bg-yellow-400 text-black font-bold transition-colors"><SettingsIcon name="fullscreen" size=16/>{move || t.get("calib_fullscreen")}</button>
        <button on:click=move |_| dismissed.set(true) aria-label="Close" class="p-2 rounded-full text-gray-400 hover:text-white hover:bg-white/10 transition-colors"><SettingsIcon name="close" size=16/></button>
    </crate::presence::Presence> }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_shared_format_and_handles_bad_storage() {
        assert_eq!(
            decode(r#"{"size":1.25,"margin":4.5}"#),
            Display {
                size: 1.25,
                margin: 4.5
            }
        );
        for value in ["bad", "{}", "null", r#"{"size":null,"margin":"bad"}"#] {
            assert_eq!(decode(value), Display::default());
        }
        assert_eq!(
            decode(r#"{"size":0,"margin":50}"#),
            Display {
                size: 0.6,
                margin: 10.0
            }
        );
        assert_eq!(
            normalized(Display {
                size: 1.239,
                margin: -1.0
            }),
            Display {
                size: 1.24,
                margin: 0.0
            }
        );
        assert_eq!(
            normalized(Display {
                size: f64::NAN,
                margin: f64::INFINITY
            }),
            Display::default()
        );
    }
    #[test]
    fn fullscreen_detection_cancels_zoom() {
        assert!(fullscreen_geometry((1920.0, 1080.0), (2400.0, 1350.0)));
        assert!(!fullscreen_geometry((1920.0, 1080.0), (2400.0, 1200.0)));
        assert!(!fullscreen_geometry((0.0, 0.0), (0.0, 0.0)));
    }
}
