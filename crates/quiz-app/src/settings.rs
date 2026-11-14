//! The unscaled operator settings window. Changes apply immediately, including over CONFIG sync.
use leptos::prelude::*;
use quiz_core::{
    quiz::model::Settings,
    theme::{Theme, builtin},
};
use serde_json::{Value, json};

/// Small Lucide-compatible icons kept here so the parallel scene icon port is independent.
#[component]
pub fn SettingsIcon(name: &'static str, #[prop(default = 18)] size: u32) -> impl IntoView {
    let path = match name {
        "close" => "m18 6-12 12M6 6l12 12",
        "up" => "m18 15-6-6-6 6",
        "down" => "m6 9 6 6 6-6",
        "check" => "m20 6-11 11-5-5",
        "reset" => "M3 11a9 9 0 1 1 2.6 6.4M3 4v7h7",
        "fullscreen" => {
            "M8 3H5a2 2 0 0 0-2 2v3m13-5h3a2 2 0 0 1 2 2v3M3 16v3a2 2 0 0 0 2 2h3m8 0h3a2 2 0 0 0 2-2v-3"
        }
        "save" => {
            "M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2ZM17 21v-7H7v7M7 3v5h8"
        }
        "library" => "M4 6v14M8 6v14M12 6v14m4-14 4 14",
        "clock" => "M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0M12 7v5l3 2",
        "globe" => {
            "M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0M3 12h18M12 3a18 18 0 0 0 0 18 18 18 0 0 0 0-18"
        }
        _ => "m13 2-3 8H3l8 12 3-8h7Z",
    };
    view! { <svg xmlns="http://www.w3.org/2000/svg" width=size height=size viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d=path/></svg> }
}

#[derive(Clone, Copy)]
pub struct SettingsContext {
    pub settings: RwSignal<Settings>,
    pub update: Callback<Settings>,
}
pub fn use_settings() -> SettingsContext {
    expect_context()
}

/// Use serde's existing camelCase mapping so extension fields survive every edit.
fn changed(settings: &Settings, key: &str, value: Value) -> Settings {
    let mut object = serde_json::to_value(settings).expect("settings serialize");
    object[key] = value;
    serde_json::from_value(object).expect("valid settings edit")
}
fn set(context: SettingsContext, key: &str, value: Value) {
    context
        .update
        .run(changed(&context.settings.get_untracked(), key, value));
}
fn time_parts(time: &str) -> (i32, i32) {
    let Some((h, m)) = time.split_once(':') else {
        return (20, 0);
    };
    match (h.parse::<i32>(), m.parse::<i32>()) {
        (Ok(h), Ok(m)) if (0..24).contains(&h) && (0..60).contains(&m) => (h, m),
        _ => (20, 0),
    }
}
fn edited_time(time: &str, hours: bool, value: i32, wrap: bool) -> String {
    let (mut h, mut m) = time_parts(time);
    let limit = if hours { 24 } else { 60 };
    let value = if wrap {
        value.rem_euclid(limit)
    } else {
        value.clamp(0, limit - 1)
    };
    if hours {
        h = value
    } else {
        m = value
    }
    format!("{h:02}:{m:02}")
}
pub fn custom_theme() -> Option<Theme> {
    crate::quiz_context::use_quiz()
        .quiz
        .fields
        .get("customTheme")
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}
pub fn selected_theme(settings: &Settings, custom: Option<&Theme>) -> Theme {
    builtin::theme_for_quiz(
        settings
            .fields
            .get("theme")
            .and_then(Value::as_str)
            .unwrap_or("default"),
        custom,
    )
    .clone()
}
const INACTIVE: &str = "flex-1 py-2 rounded-lg border text-sm font-bold transition-all bg-black/20 text-gray-400 border-white/5 hover:bg-white/5";
const ACTIVE: &str = "flex-1 py-2 rounded-lg border text-sm font-bold transition-all bg-yellow-500 text-black border-yellow-500";
const BOX: &str = "bg-black/20 p-4 rounded-xl border border-white/5 space-y-3";

#[component]
fn Choices(field: &'static str, choices: Vec<(String, String)>) -> impl IntoView {
    let settings = use_settings().settings;
    let context = use_settings();
    let set = move |key: &str, value| set(context, key, value);
    view! { <div class="flex gap-2">{choices.into_iter().map(|(key, label)| {
        let selected = key.clone();
        view! { <button class=move || {
            let s = settings.get();
            let value = match field { "language" => s.language.as_str(), "welcomeLayout" => s.welcome_layout.as_str(), _ => s.fields.get(field).and_then(Value::as_str).unwrap_or("default") };
            if value != selected { if field == "language" { "flex-1 py-3 rounded-xl border font-bold transition-all bg-black/20 text-gray-400 border-white/5 hover:bg-white/5" } else { INACTIVE } } else if field == "language" {
                match selected.as_str() {
                    "sk" => "flex-1 py-3 rounded-xl border font-bold transition-all bg-red-600 text-white border-red-500",
                    "en" => "flex-1 py-3 rounded-xl border font-bold transition-all bg-blue-600 text-white border-blue-500",
                    _ => "flex-1 py-3 rounded-xl border font-bold transition-all bg-yellow-500 text-black border-yellow-500",
                }
            } else { ACTIVE }
        } on:click=move |_| set(field, json!(key))>{label}</button> }
    }).collect_view()}</div> }
}
#[component]
fn Toggle(field: &'static str, label: &'static str) -> impl IntoView {
    let t = crate::i18n::use_t();
    let s = use_settings().settings;
    let context = use_settings();
    let set = move |key: &str, value| set(context, key, value);
    let enabled = move || {
        let s = s.get();
        if field == "showTime" {
            s.show_time
        } else {
            s.fields.get(field).and_then(Value::as_bool).unwrap_or(true)
        }
    };
    view! { <div class="flex items-center justify-between bg-black/20 p-4 rounded-xl border border-white/5">
        <span class="text-gray-200 font-medium">{move || t.get(label)}</span>
        <button role="switch" aria-label=move || t.get(label) aria-checked=move || enabled().to_string()
            class=move || if enabled() { "w-14 h-8 rounded-full p-1 transition-all duration-300 bg-green-500 shadow-[0_0_10px_rgba(34,197,94,0.4)]" } else { "w-14 h-8 rounded-full p-1 transition-all duration-300 bg-gray-700" }
            on:click=move |_| set(field, json!(!enabled()))>
            <div class=move || if enabled() { "w-6 h-6 bg-white rounded-full shadow-md transition-transform duration-300 translate-x-6" } else { "w-6 h-6 bg-white rounded-full shadow-md transition-transform duration-300 translate-x-0" }/>
        </button>
    </div> }
}
#[component]
fn Slider(
    field: &'static str,
    label: &'static str,
    min: u64,
    max: u64,
    step: u64,
) -> impl IntoView {
    let s = use_settings().settings;
    let context = use_settings();
    let set = move |key: &str, value| set(context, key, value);
    let t = crate::i18n::use_t();
    let value = move || {
        let s = s.get();
        match field {
            "introPause" => s.intro_pause,
            "headerInterval" => {
                if s.header_interval == 0 {
                    5000
                } else {
                    s.header_interval
                }
            }
            "splitDelay" => s.split_delay,
            _ => s.cycle_duration,
        }
    };
    let color = match field {
        "headerInterval" => "font-mono text-pink-400",
        "splitDelay" => "font-mono text-yellow-400",
        "cycleDuration" => "font-mono text-blue-400",
        _ => "font-mono text-green-400",
    };
    let range_class = match field {
        "headerInterval" => {
            "w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-pink-500"
        }
        "splitDelay" => {
            "w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-yellow-500"
        }
        "cycleDuration" => {
            "w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-500"
        }
        _ => "w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-green-500",
    };
    view! { <div class=BOX><div class="flex justify-between text-gray-300 text-sm mb-3">
        <span>{move || t.get(label)}</span><span class=color>{move || format!("{}s", value() as f64 / 1000.0)}</span>
        </div><input type="range" min=min max=max step=step prop:value=value aria-label=move || t.get(label)
            on:input=move |event| { if let Ok(v) = event_target_value(&event).parse::<u64>() { set(field, json!(v)); } }
            class=range_class/>
    </div> }
}
#[component]
fn TimePart(hours: bool) -> impl IntoView {
    let s = use_settings().settings;
    let context = use_settings();
    let set = move |key: &str, value| set(context, key, value);
    let value = move || {
        let (h, m) = time_parts(&s.get().start_time);
        if hours { h } else { m }
    };
    let adjust = move |delta| {
        set(
            "startTime",
            json!(edited_time(
                &s.get_untracked().start_time,
                hours,
                value() + delta,
                true
            )),
        );
    };
    view! { <div class="flex flex-col items-center gap-1">
        <button aria-label=if hours { "Increase hour" } else { "Increase minute" } class="p-2 text-yellow-500/50 hover:text-yellow-400 hover:bg-white/5 rounded-full transition-all" on:click=move |_| adjust(1)><SettingsIcon name="up" size=24/></button>
        <input type="number" min="0" max=if hours { "23" } else { "59" } aria-label=if hours { "Hours" } else { "Minutes" }
            prop:value=move || format!("{:02}", value())
            on:input=move |event| { if let Ok(v) = event_target_value(&event).parse() { set("startTime", json!(edited_time(&s.get_untracked().start_time, hours, v, false))); } }
            on:blur=move |_| set("startTime", json!(edited_time(&s.get_untracked().start_time, hours, value(), false)))
            class="settings-time w-20 bg-transparent text-center font-mono text-5xl font-bold text-white focus:outline-none focus:text-yellow-400 appearance-none"/>
        <button aria-label=if hours { "Decrease hour" } else { "Decrease minute" } class="p-2 text-yellow-500/50 hover:text-yellow-400 hover:bg-white/5 rounded-full transition-all" on:click=move |_| adjust(-1)><SettingsIcon name="down" size=24/></button>
    </div> }
}
#[component]
pub fn SettingsModal() -> impl IntoView {
    let ui = crate::show::use_show_ui();
    let s = use_settings().settings;
    let context = use_settings();
    let set = move |key: &str, value| set(context, key, value);
    let t = crate::i18n::use_t();
    let custom = StoredValue::new(custom_theme());
    view! {
        <crate::presence::Presence when=Signal::derive(move || ui.settings_open.get()) enter="settings-enter" exit="settings-exit"
            class="settings-overlay fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm">
            <div role="dialog" aria-modal="true" aria-label=move || t.get("settings_title") class="settings-panel bg-[#053049] w-full max-w-lg rounded-2xl border border-gray-700 shadow-2xl overflow-hidden flex flex-col max-h-[90vh] text-white font-sans">
                <div class="p-6 border-b border-gray-700 flex justify-between items-center bg-black/20">
                    <h2 class="text-xl font-bold text-white flex items-center gap-2"><span class="text-yellow-400"><SettingsIcon name="zap" size=20/></span>{move || t.get("settings_title")}</h2>
                    <button aria-label="Close" on:click=move |_| ui.settings_open.set(false) class="text-gray-400 hover:text-white transition-colors bg-white/5 hover:bg-white/10 p-2 rounded-full"><SettingsIcon name="close" size=20/></button>
                </div>
                <div class="p-6 space-y-8 overflow-y-auto custom-scrollbar">
                    <div class="space-y-4"><h3 class="text-xs font-bold text-gray-400 uppercase tracking-widest flex items-center gap-2"><SettingsIcon name="globe" size=14/>{move || t.get("language")}</h3>
                        <Choices field="language" choices=vec![("cs".into(), "Čeština 🇨🇿".into()), ("sk".into(), "Slovenčina 🇸🇰".into()), ("en".into(), "English 🇺🇸".into())]/>
                    </div>
                    <div class="space-y-4 pt-4 border-t border-gray-700">
                        <h3 class="text-xs font-bold text-gray-400 uppercase tracking-widest flex items-center gap-2"><SettingsIcon name="clock" size=14/>{move || t.get("display_options")}</h3>
                        <div class=BOX><span class="text-gray-200 font-medium">{move || t.get("set_theme")}</span>
                            {move || { let language = s.get().language; custom.with_value(|custom| view! { <Choices field="theme" choices=builtin::theme_choices(custom.as_ref()).into_iter().map(|(key, theme)| (key.to_string(), builtin::theme_name(theme, &language).to_string())).collect()/> }) }}
                        </div>
                        <div class=BOX><span class="text-gray-200 font-medium">{move || t.get("set_layout")}</span>
                            {move || view! { <Choices field="welcomeLayout" choices=["auto", "cards", "wall"].into_iter().map(|key| (key.to_string(), t.get(&format!("set_layout_{key}")))).collect()/> }}
                            <p class="text-xs text-gray-500">{move || t.get("set_layout_help")}</p>
                        </div>
                        <Show when=move || s.get().welcome_layout != "cards"><Slider field="introPause" label="set_intro_pause" min=1000 max=10000 step=500/></Show>
                        <Toggle field="sounds" label="set_sounds"/>
                        <Show when=move || custom.with_value(|c| selected_theme(&s.get(), c.as_ref()).sounds.pack.is_some()) fallback=move || view! { <p class="text-xs text-gray-500">{move || t.get("set_sounds_none")}</p> }>
                            <Show when=move || s.get().fields.get("sounds").and_then(Value::as_bool) != Some(false)>
                                <div class="flex items-center gap-3">
                                    <input type="range" min="0" max="1" step="0.05" aria-label=move || t.get("set_volume") prop:value=move || volume(&s.get())
                                        on:input=move |event| { if let Ok(v) = event_target_value(&event).parse::<f64>() { set("soundVolume", json!(v)); } }
                                        class="flex-1 h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-green-500"/>
                                    <span class="font-mono text-green-400 text-sm w-10 text-right">{move || format!("{}%", (volume(&s.get()) * 100.0).round())}</span>
                                    <button on:click=move |_| {
                                        let settings = s.get_untracked();
                                        if let Some(pack) = custom.with_value(|c| selected_theme(&settings, c.as_ref()).sounds.pack) { crate::sounds::preview(pack, quiz_core::sounds::Sound::Answer, volume(&settings)); }
                                    } class="px-3 py-1.5 rounded-lg bg-white/10 hover:bg-white/20 text-sm">{move || t.get("set_sound_test")}</button>
                                </div>
                            </Show>
                        </Show>
                        <Toggle field="celebration" label="set_celebration"/>
                        <Toggle field="showTime" label="show_clock"/>
                        <button on:click=move |_| { ui.settings_open.set(false); ui.calibrating.set(true); } class="w-full flex items-center justify-center gap-2 bg-black/20 hover:bg-white/10 p-4 rounded-xl border border-white/5 text-gray-200 font-medium transition-all"><span class="text-yellow-400"><SettingsIcon name="fullscreen"/></span>{move || t.get("calib_open")}</button>
                        <div class="bg-black/20 p-4 rounded-xl border border-white/5 flex flex-col items-center">
                            <label class="text-xs font-bold text-gray-400 uppercase tracking-widest mb-4 self-start">{move || t.get("start_time")}</label>
                            <div class="flex items-center gap-4 bg-black/40 px-6 py-4 rounded-2xl border border-white/10 shadow-inner"><TimePart hours=true/><span class="text-4xl font-bold text-yellow-500/50 pb-2 animate-pulse">":"</span><TimePart hours=false/></div>
                        </div>
                    </div>
                    <div class="space-y-4 pt-4 border-t border-gray-700"><h3 class="text-xs font-bold text-gray-400 uppercase tracking-widest flex items-center gap-2"><SettingsIcon name="zap" size=14/>{move || t.get("anim_speed")}</h3>
                        <Slider field="headerInterval" label="header_rot" min=3000 max=10000 step=1000/>
                        <Slider field="splitDelay" label="read_time" min=1000 max=8000 step=500/>
                        <Slider field="cycleDuration" label="cycle_time" min=4000 max=15000 step=1000/>
                    </div>
                </div>
                <div class="p-4 bg-black/40 border-t border-gray-700 flex items-center justify-between gap-2">
                    <button on:click=move |_| { crate::router::navigate(crate::router::Route::Library); } class="px-4 py-3 text-gray-300 hover:text-white hover:bg-white/10 rounded-xl flex items-center gap-2 transition-colors"><SettingsIcon name="library"/>{move || t.get("settings_exit")}</button>
                    <button on:click=move |_| ui.settings_open.set(false) class="px-6 py-3 bg-yellow-500 hover:bg-yellow-400 text-black font-bold rounded-xl flex items-center gap-2 hover:scale-105 transition-transform"><SettingsIcon name="save"/>{move || t.get("save_close")}</button>
                </div>
            </div>
        </crate::presence::Presence>
    }
}
fn volume(s: &Settings) -> f64 {
    s.fields
        .get("soundVolume")
        .and_then(Value::as_f64)
        .unwrap_or(0.6)
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn time_wraps_and_typed_values_clamp() {
        assert_eq!(edited_time("23:59", true, 24, true), "00:59");
        assert_eq!(edited_time("00:00", false, -1, true), "00:59");
        assert_eq!(edited_time("20:15", true, 99, false), "23:15");
        assert_eq!(edited_time("20:15", false, -9, false), "20:00");
        assert_eq!(time_parts("bad"), (20, 0));
    }
    #[test]
    fn edits_preserve_extensions_and_typed_wire_fields() {
        let s = changed(&Settings::default(), "soundVolume", json!(0.35));
        let s = changed(&s, "language", json!("en"));
        let s = changed(&s, "showTime", json!(false));
        assert_eq!(s.language, "en");
        assert!(!s.show_time);
        assert_eq!(s.fields["soundVolume"], json!(0.35));
    }
}
