//! Theme editor: all persisted theme data is normalized by quiz-core.
use crate::{i18n::use_t, router::Route, store};
use leptos::prelude::*;
use quiz_core::theme::{builtin::*, format::*, pack::*};
use quiz_store::{LibraryTheme, Media, MediaStore, QuizStore, ThemeStore};
use serde_json::{Value, json};
use wasm_bindgen::JsCast;

mod preview;

pub(super) const BUTTON: &str = "inline-flex items-center justify-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm disabled:opacity-40";
const INPUT: &str = "w-full bg-black/40 border border-white/10 rounded-xl px-3 py-2 text-white focus:outline-none focus:border-yellow-500";
fn normalized(raw: &Value) -> Theme {
    normalize_theme_with_options(
        raw,
        &NormalizeOptions {
            stored: true,
            ..Default::default()
        },
    )
    .theme
    .unwrap_or_else(|| theme_by_id("default").clone())
}
fn set_path(raw: &mut Value, path: &str, value: Value) {
    let (key, rest) = path
        .split_once('.')
        .map_or((path, None), |(key, rest)| (key, Some(rest)));
    if let Some(array) = raw.as_array_mut() {
        if let Ok(index) = key.parse::<usize>() {
            if let Some(item) = array.get_mut(index) {
                if let Some(rest) = rest {
                    set_path(item, rest, value);
                } else {
                    *item = value;
                }
            }
        }
        return;
    }
    if let Some(rest) = rest {
        if !raw[key].is_object() && !raw[key].is_array() {
            raw[key] = json!({});
        }
        set_path(&mut raw[key], rest, value);
    } else if value.is_null() {
        if let Some(object) = raw.as_object_mut() {
            object.remove(key);
        }
    } else {
        raw[key] = value;
    }
}
fn value_at(raw: &Value, path: &str) -> Value {
    path.split('.')
        .fold(raw, |value, key| {
            if value.is_array() {
                key.parse::<usize>()
                    .ok()
                    .and_then(|index| value.get(index))
                    .unwrap_or(&Value::Null)
            } else {
                &value[key]
            }
        })
        .clone()
}
fn own_copy(mut theme: Theme, t: crate::i18n::T) -> Theme {
    if themes().iter().any(|b| b.id == theme.id) {
        theme.name = LANGS
            .iter()
            .map(|l| {
                (
                    (*l).into(),
                    t.fmt("te_copy_name", &[("name", theme_name(&theme, l))]),
                )
            })
            .collect();
        theme.id = slug(&json!(format!("{}-own", theme.id)));
        theme.author.clear();
    }
    theme
}
fn upload_name(name: &str, font: bool, size: usize) -> Result<String, String> {
    let (base, ext) = name.rsplit_once('.').unwrap_or((name, ""));
    let ext = ext.to_lowercase();
    if !(if font { FONT_TYPES } else { IMAGE_TYPES }).contains(&ext.as_str()) {
        return Err("file_type".into());
    }
    if size
        > if font {
            2 * 1024 * 1024
        } else {
            8 * 1024 * 1024
        }
    {
        return Err("file_size".into());
    }
    let base = slug(&json!(base));
    Ok(format!(
        "theme-{}.{}",
        if base.is_empty() {
            if font { "font" } else { "image" }
        } else {
            &base
        },
        ext
    ))
}
fn mime(name: &str) -> &'static str {
    match name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "avif" => "image/avif",
        _ => "application/octet-stream",
    }
}
pub(super) fn media_from_assets(assets: Vec<PackAsset>) -> Vec<Media> {
    assets
        .into_iter()
        .map(|a| Media {
            content_type: mime(&a.name).into(),
            name: a.name,
            bytes: a.bytes,
            ..Default::default()
        })
        .collect()
}
pub(super) async fn save_quiz_theme(
    store: &store::Store,
    id: &str,
    theme: Theme,
) -> Result<(), String> {
    let raw = serde_json::to_value(&theme).map_err(|e| e.to_string())?;
    let saved = store
        .update_quiz(id, move |q| {
            q.fields.insert("customTheme".into(), raw);
            q.settings
                .fields
                .insert("theme".into(), json!(CUSTOM_THEME_ID));
        })
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Quiz not found")?;
    if saved.drive.is_none() {
        let used = theme_files(&theme);
        for m in store.list_media(id).await.map_err(|e| e.to_string())? {
            if m.name
                .rsplit('/')
                .next()
                .unwrap_or("")
                .starts_with("theme-")
                && !used.iter().any(|n| n.eq_ignore_ascii_case(&m.name))
            {
                store
                    .delete_media(id, &m.name)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
pub(super) fn download(theme: &Theme, files: &[Media]) -> Result<(), String> {
    let assets = files
        .iter()
        .map(|m| PackAsset {
            name: m.name.clone(),
            bytes: m.bytes.clone(),
        })
        .collect::<Vec<_>>();
    let bytes = export_theme_pack(theme, &assets).map_err(|e| e.to_string())?;
    let url = web_sys::Url::create_object_url_with_blob(&store::blob(&bytes, "application/zip")?)
        .map_err(|e| format!("{e:?}"))?;
    let a = document()
        .create_element("a")
        .map_err(|e| format!("{e:?}"))?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|e| format!("{e:?}"))?;
    a.set_href(&url);
    a.set_download(&theme_pack_name(theme));
    a.click();
    gloo_timers::callback::Timeout::new(1000, move || {
        let _ = web_sys::Url::revoke_object_url(&url);
    })
    .forget();
    Ok(())
}

#[component]
pub(super) fn Preview(
    #[prop(into)] theme: Signal<Theme>,
    #[prop(into)] files: Signal<Vec<Media>>,
    #[prop(optional)] base: String,
    #[prop(optional)] quiz: Option<quiz_core::quiz::model::Quiz>,
    #[prop(optional)] compact: bool,
    #[prop(optional)] selected_scene: Option<RwSignal<String>>,
    #[prop(optional)] selected_wall: Option<RwSignal<bool>>,
    #[prop(optional)] celebrate: Option<RwSignal<u64>>,
) -> impl IntoView {
    let urls = StoredValue::new_local(std::cell::RefCell::new(Vec::<String>::new()));
    let assets = Memo::new(move |_| {
        urls.with_value(|u| {
            for url in u.borrow_mut().drain(..) {
                let _ = web_sys::Url::revoke_object_url(&url);
            }
        });
        let mut map = std::collections::BTreeMap::new();
        for m in files.get() {
            if let Ok(blob) = store::blob(&m.bytes, &m.content_type)
                && let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob)
            {
                urls.with_value(|u| u.borrow_mut().push(url.clone()));
                map.insert(quiz_core::quiz::parse::media_key(&m.name), url);
            }
        }
        map
    });
    on_cleanup(move || {
        urls.with_value(|u| {
            for url in u.borrow_mut().drain(..) {
                let _ = web_sys::Url::revoke_object_url(&url);
            }
        })
    });
    let resolve = StoredValue::new(move |name: &str| {
        assets
            .get()
            .get(&quiz_core::quiz::parse::media_key(name))
            .cloned()
            .or_else(|| (!base.is_empty()).then(|| format!("{base}{name}")))
    });
    let font_css = move || {
        theme
            .get()
            .fonts
            .values()
            .filter_map(|f| match f {
                Font::File { file } => resolve.with_value(|r| r(file)).map(|url| {
                    format!(
                        "@font-face{{font-family:'{}';src:url('{}');font-display:swap;}}",
                        font_family(f).replace('\'', "\\'"),
                        url.replace('\'', "%27")
                    )
                }),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let image_scene = selected_scene;
    let image = move || {
        let t = theme.get();
        t.background
            .image
            .as_ref()
            .and_then(|n| resolve.with_value(|r| r(n)))
            .map(|url| {
                format!(
                    "background-image:url(\"{}\");opacity:{};filter:blur({}px)",
                    url.replace('"', "%22"),
                    t.background.image_opacity
                        * if image_scene.is_some_and(|scene| matches!(
                            scene.get().as_str(),
                            "question" | "answer" | "leaderboard"
                        )) {
                            0.5
                        } else {
                            1.0
                        },
                    t.background.image_blur
                )
            })
            .unwrap_or_default()
    };
    let t = use_t();
    let defaults = quiz_core::quiz::intro::DEFAULT_RULE_KEYS.map(|key| t.get(key));
    let intro = quiz
        .as_ref()
        .map(|q| quiz_core::show::Intro::new(q, &defaults));
    let has_round = quiz.as_ref().is_some_and(|q| {
        quiz_core::quiz::parse::parse_questions_csv(&q.questions_csv)
            .iter()
            .any(|r| !quiz_core::quiz::parse::is_round_hidden(r) && !r.questions.is_empty())
    });
    let scene = selected_scene.unwrap_or_else(|| {
        RwSignal::new(if compact {
            if intro.is_some_and(|intro| intro.has_rules) {
                "rules".into()
            } else {
                "dashboard".into()
            }
        } else {
            "welcome".into()
        })
    });
    let wall = selected_wall.unwrap_or_else(|| RwSignal::new(false));
    let replay = RwSignal::new(0_u64);
    let frame = NodeRef::<leptos::html::Div>::new();
    let scale = RwSignal::new(0.);
    Effect::new(move |_| {
        if let Some(element) = frame.get() {
            let update = move || {
                if let Some(element) = frame.get() {
                    scale.set(element.get_bounding_client_rect().width() / 2400.);
                }
            };
            update();
            let callback =
                wasm_bindgen::closure::Closure::<dyn FnMut(js_sys::Array)>::new(move |_| update());
            if let Ok(observer) = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) {
                observer.observe(&element);
                let observer = StoredValue::new_local((observer, callback));
                on_cleanup(move || observer.with_value(|(observer, _)| observer.disconnect()));
            }
        }
    });
    let quiz = StoredValue::new(quiz);
    let resolver = Callback::new(move |name: String| {
        let key = quiz_core::quiz::parse::media_key(&name);
        assets
            .get()
            .iter()
            .find(|(n, _)| quiz_core::quiz::parse::media_key(n) == key)
            .map(|(_, url)| url.clone())
            .or_else(|| {
                if name.starts_with("assets/") {
                    resolve.with_value(|r| r(&name))
                } else if name.starts_with("https://") || name.starts_with("http://") {
                    Some(name.clone())
                } else if quiz.with_value(|q| q.as_ref().is_some_and(|q| q.builtin)) {
                    Some(format!("source/{name}"))
                } else {
                    None
                }
            })
    });
    view! {
        <style>{font_css}</style>
        {(!compact).then(|| view! {
            <nav class="flex flex-wrap gap-2 mb-3" aria-label=t.get("te_title")>
                {[("welcome", "mp_welcome"), ("rules", "mp_rules"), ("prizes", "mp_prizes"),
                  ("dashboard", "te_s_dashboard"), ("round", "te_s_round"),
                  ("question", "te_s_question"), ("answer", "te_s_answer"),
                  ("leaderboard", "te_s_leaderboard"), ("pause", "mp_pause")]
                    .into_iter().filter(|(key, _)| match *key {
                        "rules" => intro.is_some_and(|intro| intro.has_rules),
                        "prizes" => intro.is_some_and(|intro| intro.has_prizes),
                        "round" | "question" | "answer" => has_round,
                        _ => true,
                    }).map(|(key, label)| view! {
                        <button type="button" class=move || if scene.get() == key { "px-4 py-2 rounded-xl bg-yellow-500 text-black font-bold text-sm" } else { BUTTON } aria-pressed=move || scene.get() == key
                            on:click=move |_| scene.set(key.into())>{t.get(label)}</button>
                    }).collect_view()}
                <button type="button" class=BUTTON on:click=move |_| { wall.update(|v| *v = !*v); scene.set("welcome".into()); }>{t.get("te_wl_intro")}</button>
                <button type="button" class=BUTTON on:click=move |_| replay.update(|v| *v += 1)>"↻"</button>
            </nav>
        })}
        <div node_ref=frame class="theme-preview relative overflow-hidden rounded-xl ring-1 ring-white/15">
            <div class="theme-preview-canvas" style=move || format!("transform:scale({})", scale.get())>
                <crate::theme::ThemeScope theme=theme class="bg-ground font-show relative w-full h-full overflow-hidden">
                    {move || { replay.get(); assets.get(); quiz.get_value().map(|quiz| view! { <preview::Scenes quiz=quiz scene=scene resolve=resolver wall=wall.get()/> }) }}
                    <div class="theme-preview-image absolute inset-0 bg-cover bg-center pointer-events-none" style=image></div>
                    {move || {
                        let quiet = matches!(scene.get().as_str(), "question" | "answer" | "leaderboard");
                        view! { <crate::scenes::backdrop::ThemeDecorations quiet=quiet/> }
                    }}
                    {celebrate.map(|trigger| view! { <preview::CelebrationPreview trigger=trigger/> })}
                </crate::theme::ThemeScope>
            </div>
        </div>
    }
}

#[derive(Clone, Copy)]
struct Draft {
    raw: RwSignal<Value>,
    changed: RwSignal<u64>,
    scene: RwSignal<String>,
    celebrate: RwSignal<u64>,
    wall: RwSignal<bool>,
}
impl Draft {
    fn set(self, path: &str, value: Value) {
        if path == "scenes.welcome.intro" {
            self.wall.set(true);
        }
        if path.starts_with("scenes.") {
            if let Some(scene) = path.split('.').nth(1) {
                self.scene.set(scene.into());
            }
        }
        self.raw.update(|v| set_path(v, path, value));
        self.changed.update(|n| *n += 1);
    }
    fn replace(self, theme: Theme) {
        self.raw.set(serde_json::to_value(theme).unwrap());
        self.changed.update(|n| *n += 1);
    }
}
#[component]
fn Panel(title: String, children: Children) -> impl IntoView {
    view! {
        <section class="rounded-2xl bg-white/[0.04] border border-white/10 p-4 space-y-3">
            <h2 class="font-['League_Spartan'] font-bold uppercase tracking-widest text-sm text-gray-300">{title}</h2>
            {children()}
        </section>
    }
}

#[component]
fn TextField(
    draft: Draft,
    path: String,
    label: String,
    #[prop(default = 80)] max: usize,
    #[prop(optional)] localized: bool,
) -> impl IntoView {
    let t = use_t();
    let path = StoredValue::new(path);
    let read = move || {
        let value = path.with_value(|p| value_at(&draft.raw.get(), p));
        if localized {
            value[t.lang().code()]
                .as_str()
                .or(value["en"].as_str())
                .unwrap_or("")
                .to_owned()
        } else {
            value.as_str().unwrap_or("").into()
        }
    };
    let write = Callback::new(move |text: String| {
        let value = if localized {
            LANGS
                .iter()
                .map(|l| ((*l).into(), json!(text)))
                .collect::<serde_json::Map<_, _>>()
                .into()
        } else {
            json!(text)
        };
        path.with_value(|p| draft.set(p, value));
    });
    view! {
        <label class="block space-y-1">
            <span class="text-sm text-gray-400">{label}</span>
            {if max == 300 {
                view! { <textarea class=INPUT maxlength=max prop:value=read on:input=move |ev| write.run(event_target_value(&ev))/> }.into_any()
            } else {
                view! { <input class=INPUT maxlength=max prop:value=read on:input=move |ev| write.run(event_target_value(&ev))/> }.into_any()
            }}
        </label>
    }
}

#[component]
fn Tags(draft: Draft) -> impl IntoView {
    let t = use_t();
    let text = RwSignal::new(String::new());
    let tags = Signal::derive(move || {
        draft.raw.get()["tags"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect::<Vec<_>>()
    });
    let add = move || {
        let tag = slug(&json!(text.get_untracked().trim_end_matches(',')));
        let mut items = tags.get_untracked();
        if !tag.is_empty() && items.len() < 8 && !items.contains(&tag) {
            items.push(tag);
            draft.set("tags", json!(items));
        }
        text.set(String::new());
    };
    view! {
        <div class="space-y-1">
            <label for="theme-tags" class="text-sm text-gray-400">{t.get("te_tags")}</label>
            <div class="flex flex-wrap items-center gap-1.5 bg-black/40 border border-white/10 rounded-xl px-3 py-2">
                {move || tags.get().into_iter().map(|tag| {
                    let remove = tag.clone();
                    view! {
                        <span class="flex items-center gap-1 pl-2 pr-1 py-0.5 rounded-full bg-white/10 text-sm">
                            {format!("#{tag}")}
                            <button type="button" class="p-0.5 rounded-full hover:bg-white/20" aria-label=tag on:click=move |_| {
                                let items = tags.get_untracked().into_iter().filter(|tag| tag != &remove).collect::<Vec<_>>();
                                draft.set("tags", json!(items));
                            }>"×"</button>
                        </span>
                    }
                }).collect_view()}
                <input id="theme-tags" class="flex-1 min-w-24 bg-transparent py-1 text-white focus:outline-none" placeholder=t.get("te_tags_hint")
                    prop:value=move || text.get() on:input=move |ev| { let value = event_target_value(&ev); let commit = value.ends_with(','); text.set(value); if commit { add(); } }
                    on:blur=move |_| add() on:keydown=move |ev| {
                        if ev.key() == "Enter" { ev.prevent_default(); add(); }
                        if ev.key() == "Backspace" && text.get_untracked().is_empty() { let mut items = tags.get_untracked(); items.pop(); draft.set("tags", json!(items)); }
                    }/>
            </div>
        </div>
    }
}

#[component]
fn Choice(
    draft: Draft,
    path: String,
    label: String,
    options: Vec<String>,
    prefix: String,
    #[prop(optional)] nullable: bool,
    #[prop(optional)] default: String,
) -> impl IntoView {
    let t = use_t();
    let path = StoredValue::new(path);
    let selected = Signal::derive(move || {
        path.with_value(|p| {
            value_at(&draft.raw.get(), p)
                .as_str()
                .unwrap_or(&default)
                .to_owned()
        })
    });
    let mut choices = options;
    if nullable {
        choices.insert(0, String::new());
    }
    view! {
        <div class="space-y-1.5">
            <span class="text-sm text-gray-400">{label}</span>
            <div class="flex flex-wrap gap-2">
                {choices.into_iter().map(|value| {
                    let label = if prefix.is_empty() { value.clone() } else { t.get(&format!("{prefix}{}", if value.is_empty() { "none" } else { &value })) };
                    let active = value.clone();
                    let pressed = active.clone();
                    view! {
                        <button type="button" aria-pressed=move || selected.get() == pressed class=move || {
                            if selected.get() == active { "flex-1 px-3 py-2 rounded-lg border text-sm font-bold bg-yellow-500 text-black border-yellow-500" }
                            else { "flex-1 px-3 py-2 rounded-lg border text-sm font-bold bg-black/20 text-gray-300 border-white/10 hover:bg-white/5" }
                        } on:click=move |_| path.with_value(|p| {
                            draft.set(p, if value.is_empty() { Value::Null } else { json!(value) });
                            if p == "sounds.pack" { if let Ok(pack) = serde_json::from_value::<SoundPack>(json!(value)) { crate::sounds::preview(pack, quiz_core::sounds::Sound::Answer, 1.); } }
                        })>{label}</button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn Slider(
    draft: Draft,
    path: String,
    label: String,
    min: f64,
    max: f64,
    step: f64,
) -> impl IntoView {
    let path = StoredValue::new(path);
    let value = move || path.with_value(|p| value_at(&draft.raw.get(), p).as_f64().unwrap_or(min));
    view! {
        <label class="block text-xs text-gray-400">
            {label}
            <output class="float-right">{move || {
                let value = value();
                if max == 1. { format!("{}%", (value * 100.).round()) } else { value.to_string() }
            }}</output>
            <input class="w-full accent-yellow-500 cursor-pointer" type="range" min=min max=max step=step
                prop:value=value on:input=move |ev| {
                    if let Ok(number) = event_target_value(&ev).parse::<f64>() { path.with_value(|p| draft.set(p, json!(number))); }
                }/>
        </label>
    }
}
#[component]
fn ColorField(
    draft: Draft,
    path: String,
    label: String,
    fallback: &'static str,
    #[prop(optional)] dark: bool,
) -> impl IntoView {
    let t = use_t();
    let path = StoredValue::new(path);
    let value = Signal::derive(move || {
        path.with_value(|p| {
            value_at(
                &serde_json::to_value(normalized(&draft.raw.get())).unwrap(),
                p,
            )
            .as_str()
            .unwrap_or(fallback)
            .to_owned()
        })
    });
    let own =
        move || path.with_value(|p| value_at(&draft.raw.get(), p).as_str().map(str::to_owned));
    let colors = Signal::derive(move || {
        let theme = normalized(&draft.raw.get());
        theme
            .colors
            .values()
            .cloned()
            .chain(theme.background.glows.into_iter().map(|g| g.color))
            .collect::<Vec<_>>()
    });
    view! { <div class="flex items-center gap-3">
        <super::color_picker::ColorPicker value=value on_change=Callback::new(move |c| path.with_value(|p| draft.set(p,json!(c)))) label=label.clone() theme_colors=colors dark=dark/>
        <span class="text-sm flex-1">{label}</span><span class="text-xs font-mono text-gray-400 uppercase">{move || own().unwrap_or_else(||t.get("te_classic_color"))}</span>
        <button type="button" class=BUTTON disabled=move ||own().is_none() title=t.get("te_reset_color") aria-label=t.get("te_reset_color") on:click=move |_| path.with_value(|p| draft.set(p,Value::Null))>"↺"</button>
    </div> }
}
#[component]
fn Upload(
    draft: Draft,
    id: String,
    path: String,
    files: RwSignal<Vec<Media>>,
    notice: RwSignal<String>,
    #[prop(optional)] font: bool,
    #[prop(optional)] pack: bool,
) -> impl IntoView {
    let t = use_t();
    let store = store::use_store();
    let busy = RwSignal::new(false);
    let picker = NodeRef::<leptos::html::Input>::new();
    let image_path = StoredValue::new(path.clone());
    let image = move || {
        image_path.with_value(|p| value_at(&draft.raw.get(), p).as_str().map(str::to_owned))
    };
    let image_url = Memo::new(move |_| {
        let name = image()?;
        let file = files
            .get()
            .into_iter()
            .find(|f| f.name.eq_ignore_ascii_case(&name))?;
        let blob = store::blob(&file.bytes, &file.content_type).ok()?;
        web_sys::Url::create_object_url_with_blob(&blob).ok()
    });
    Effect::new(move |_| {
        if let Some(url) = image_url.get() {
            on_cleanup(move || {
                let _ = web_sys::Url::revoke_object_url(&url);
            });
        }
    });
    let upload = move |ev| {
        let input = event_target::<web_sys::HtmlInputElement>(&ev);
        let Some(file) = input.files().and_then(|files| files.get(0)) else {
            return;
        };
        input.set_value("");
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        notice.set(String::new());
        let store = store.clone();
        let id = id.clone();
        let path = path.clone();
        leptos::task::spawn_local(async move {
            let result: Result<(), String> = async {
                let name = if pack {
                    String::new()
                } else {
                    upload_name(&file.name(), font, file.size() as usize).map_err(|code| {
                        t.fmt(
                            &format!("te_err_{code}"),
                            &[
                                (
                                    "types",
                                    if font {
                                        "woff2, woff, ttf, otf"
                                    } else {
                                        "jpg, jpeg, png, webp, avif"
                                    },
                                ),
                                ("size", if font { "2" } else { "8" }),
                            ],
                        )
                    })?
                };
                let buffer = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
                    .await
                    .map_err(|e| format!("{e:?}"))?;
                let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
                if pack {
                    let imported = read_theme_pack(&bytes).map_err(|e| {
                        let code = e.to_string();
                        if matches!(code.as_str(), "bad_json" | "no_theme") {
                            t.get(&format!("thm_err_{code}"))
                        } else {
                            code
                        }
                    })?;
                    let (theme, assets) = theme_for_quiz_media(&imported);
                    store
                        .put_media(&id, &media_from_assets(assets))
                        .await
                        .map_err(|e| e.to_string())?;
                    files.set(store.list_media(&id).await.map_err(|e| e.to_string())?);
                    draft.replace(theme);
                    notice.set(
                        imported
                            .problems
                            .iter()
                            .map(|p| t.fmt(&format!("thm_p_{}", p.code), &[("detail", &p.detail)]))
                            .collect::<Vec<_>>()
                            .join("; "),
                    );
                } else {
                    let media = Media {
                        name: name.clone(),
                        bytes,
                        content_type: mime(&name).into(),
                        ..Default::default()
                    };
                    store
                        .put_media(&id, &[media])
                        .await
                        .map_err(|e| e.to_string())?;
                    files.set(store.list_media(&id).await.map_err(|e| e.to_string())?);
                    draft.set(
                        &path,
                        if font {
                            json!({"file": name})
                        } else {
                            json!(name)
                        },
                    );
                }
                Ok(())
            }
            .await;
            if let Err(error) = result {
                notice.set(error);
            }
            busy.set(false);
        });
    };
    view! {
        <div class="space-y-2">
            <span class="text-sm text-gray-400">{t.get(if pack {"thm_import"} else if font {"te_f_upload"} else {"te_bg_image"})}</span>
            {move || (!pack && !font).then(||image_url.get()).flatten().map(|url|view! {<img src=url alt="" class="w-full aspect-video object-cover rounded-lg border border-white/10"/>})}
            <div class="flex gap-2">
                <button type="button" class=BUTTON disabled=move ||busy.get() on:click=move |_|if let Some(i)=picker.get(){i.click();}>
                    <Show when=move ||busy.get()><span class="animate-spin" aria-hidden="true">"◌"</span></Show>
                    {move || t.get(if pack {"thm_import"} else if font {"te_f_upload"} else if image().is_some(){"te_bg_replace"}else{"te_bg_upload"})}
                </button>
                <Show when=move || !pack && !font && image().is_some()><button type="button" class=BUTTON disabled=move ||busy.get() on:click=move |_|image_path.with_value(|p|draft.set(p,Value::Null))>{t.get("te_remove")}</button></Show>
            </div>
            <input node_ref=picker type="file" hidden disabled=move ||busy.get() accept=if pack {".zip,.json"}else if font{".woff2,.woff,.ttf,.otf"}else{".jpg,.jpeg,.png,.webp,.avif"} on:change=upload/>
        </div>
    }
}
#[component]
pub fn Page(id: String, fresh: bool) -> impl IntoView {
    let store = store::use_store();
    let t = use_t();
    let id = StoredValue::new(id);
    let loaded = LocalResource::new(move || {
        let store = store.clone();
        let id = id.get_value();
        async move {
            let quiz = store
                .get_quiz(&id)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| t.get("ed_not_found"))?;
            let files = store.list_media(&id).await.map_err(|e| e.to_string())?;
            Ok::<_, String>((quiz, files))
        }
    });
    view! {
        <main class="min-h-screen bg-[#050505] text-white font-sans">
            {move || loaded.get().map(|result| match result.as_ref() {
                Ok((quiz, files)) => view! { <Editor quiz=quiz.clone() initial_files=files.clone() fresh=fresh/> }.into_any(),
                Err(error) => view! {
                    <div class="p-12"><a href="#/" class=BUTTON>{t.get("settings_exit")}</a><p role="alert">{error.clone()}</p></div>
                }.into_any(),
            }).unwrap_or_else(|| view! { <p role="status" class="p-12">{t.get("loading")}</p> }.into_any())}
        </main>
    }
}
#[component]
fn Editor(
    quiz: quiz_core::quiz::model::Quiz,
    initial_files: Vec<Media>,
    fresh: bool,
) -> impl IntoView {
    let t = use_t();
    let store = store::use_store();
    let id = StoredValue::new(quiz.id.clone());
    let preview_quiz = quiz.clone();
    let custom = quiz.fields.get("customTheme").map(normalized);
    let selected = quiz
        .settings
        .fields
        .get("theme")
        .and_then(Value::as_str)
        .unwrap_or("default");
    let initial = if !fresh { custom.clone() } else { None }
        .unwrap_or_else(|| own_copy(theme_for_quiz(selected, custom.as_ref()).clone(), t));
    let draft = Draft {
        raw: RwSignal::new(serde_json::to_value(initial).unwrap()),
        changed: RwSignal::new(0),
        scene: RwSignal::new("welcome".into()),
        celebrate: RwSignal::new(0),
        wall: RwSignal::new(false),
    };
    let theme = Signal::derive(move || normalized(&draft.raw.get()));
    let files = RwSignal::new(initial_files);
    let notice = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    let save_failed = RwSignal::new(false);
    let library_id = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let sharing = RwSignal::new(false);
    let save_store = store.clone();
    Effect::new(move |_| {
        let rev = draft.changed.get();
        if rev == 0 {
            return;
        }
        let theme = normalized(&draft.raw.get_untracked());
        let store = save_store.clone();
        let id = id.get_value();
        saving.set(true);
        save_failed.set(false);
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(600).await;
            if draft.changed.try_get_untracked() != Some(rev) {
                return;
            }
            let result = save_quiz_theme(&store, &id, theme).await;
            if let Err(e) = result {
                let _ = notice.try_set(e);
                let _ = save_failed.try_set(true);
            }
            if draft.changed.try_get_untracked() == Some(rev) {
                let _ = saving.try_set(false);
            }
        });
    });
    let cleanup_store = StoredValue::new_local(store.clone());
    let cleanup_id = id.get_value();
    let snapshot = StoredValue::new(None::<Theme>);
    Effect::new(move |_| {
        if draft.changed.get() > 0 {
            snapshot.set_value(Some(normalized(&draft.raw.get())));
        }
    });
    on_cleanup(move || {
        if let Some(theme) = snapshot.get_value() {
            let store = cleanup_store.get_value();
            leptos::task::spawn_local(async move {
                let _ = save_quiz_theme(&store, &cleanup_id, theme).await;
            });
        }
    });
    let keep = move |_| {
        let store = store.clone();
        busy.set(true);
        let theme = theme.get_untracked();
        let files = files.get_untracked();
        leptos::task::spawn_local(async move {
            let used = theme_files(&theme);
            let entry = LibraryTheme {
                id: library_id.get_untracked().unwrap_or_else(store::new_id),
                theme,
                files: files
                    .into_iter()
                    .filter(|m| used.iter().any(|n| n.eq_ignore_ascii_case(&m.name)))
                    .collect(),
                saved_at: 0,
            };
            match store.put_theme(&entry).await {
                Ok(saved) => {
                    library_id.set(Some(saved.id));
                    notice.set(t.fmt(
                        "te_lib_saved",
                        &[("name", theme_name(&saved.theme, t.lang().code()))],
                    ));
                }
                Err(e) => notice.set(e.to_string()),
            }
            busy.set(false);
        });
    };
    let fonts = ["title", "body"].map(|role| {
        view! {
            <FontField draft=draft role=role id=id.get_value() files=files notice=notice/>
        }
    });
    let colors = COLOR_KEYS.iter().zip(["#3b82f6", "#eab308", "#111111", "#050505", "#000000", "#0a0a0a"])
        .map(|(&key, fallback)| view! {
            <ColorField draft=draft path=format!("colors.{key}") label=t.get(&format!("te_c_{key}"))
                fallback=fallback dark=!["primary", "accent"].contains(&key)/>
        }).collect_view();
    let styles = STYLE_OPTIONS.iter().map(|(key, options)| view! {
        <Choice draft=draft path=format!("style.{key}") label=t.get(&format!("te_st_{key}"))
            options=strings(options) prefix=format!("te_st_{key}_") default=options[0].into()/>
    }).collect_view();
    let scenes = [
        (
            "scenes.welcome.intro",
            "te_s_welcome",
            "te_wl_intro",
            "te_wl_",
            WELCOME_INTROS,
        ),
        (
            "scenes.round.layout",
            "te_s_round",
            "te_rd_layout",
            "te_rd_",
            ROUND_LAYOUTS,
        ),
        (
            "scenes.dashboard.layout",
            "te_s_dashboard",
            "te_lb_layout",
            "te_db_",
            DASHBOARD_LAYOUTS,
        ),
        (
            "scenes.leaderboard.layout",
            "te_leaderboard",
            "te_lb_layout",
            "te_lb_",
            LEADERBOARD_LAYOUTS,
        ),
    ]
    .map(|(path, title, label, prefix, options)| {
        view! {
            <Panel title=t.get(title)>
                <Choice draft=draft path=path.into() label=t.get(label) options=strings(options)
                    prefix=prefix.into() default=options[0].into()/>
                <p class="text-xs text-gray-500">{t.get(match path {"scenes.welcome.intro"=>"te_wl_help","scenes.leaderboard.layout"=>"te_lb_help",_=>""})}</p>
            </Panel>
        }
    });
    let sounds = quiz_core::sounds::Sound::ALL.map(|sound| {
        let key = serde_json::to_value(sound)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned();
        view! {
            <button type="button" class=BUTTON disabled=move || theme.get().sounds.pack.is_none()
                on:click=move |_| {
                    if let Some(pack) = theme.get_untracked().sounds.pack {
                        crate::sounds::preview(pack, sound, 1.);
                    }
                }>{t.get(&format!("te_sd_ev_{key}"))}</button>
        }
    });
    let back_store = StoredValue::new_local(store::use_store());
    let back = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        let store = back_store.get_value();
        let id = id.get_value();
        let theme = theme.get_untracked();
        leptos::task::spawn_local(async move {
            if draft.changed.get_untracked() > 0 {
                if let Err(error) = save_quiz_theme(&store, &id, theme).await {
                    notice.set(error);
                    return;
                }
            }
            crate::router::navigate(Route::Edit(id));
        });
    };
    view! {
        <header class="sticky top-0 z-30 bg-[#050505]/95 backdrop-blur border-b border-white/10">
            <div class="max-w-[1700px] mx-auto px-6 py-4 flex flex-wrap items-center gap-3">
                <a href=Route::Edit(quiz.id).hash() class="text-gray-400 hover:text-white" on:click=back>{t.get("te_back")}</a>
                <h1 class="font-['League_Spartan'] text-xl font-bold uppercase tracking-widest mr-auto">{t.get("te_title")}</h1>
                <span class="text-sm text-gray-500" role="status">{move || if save_failed.get() { notice.get() } else { t.get(if saving.get() { "te_saving" } else { "te_saved" }) }}</span>
                <button type="button" class=BUTTON disabled=move || busy.get() on:click=keep>
                    {move || t.get(if library_id.get().is_some() { "te_lib_update" } else { "te_lib_save" })}
                </button>
                <button type="button" class=BUTTON on:click=move |_| {
                    if let Err(error) = download(&theme.get_untracked(), &files.get_untracked()) { notice.set(error); }
                }>{t.get("te_export")}</button>
                <button type="button" class=BUTTON on:click=move |_| sharing.set(true)>{t.get("te_share")}</button>
            </div>
        </header>
        <div class="max-w-[1700px] mx-auto px-6 py-6 grid gap-6 lg:grid-cols-[420px_1fr]">
            <aside class="space-y-4">
                <p role="status" class="text-sm text-yellow-300">{move || notice.get()}</p>
                <Panel title=t.get("te_about")>
                    <TextField draft=draft path="name".into() label=t.get("te_name") localized=true/>
                    <TextField draft=draft path="author".into() label=t.get("te_author")/>
                    <TextField draft=draft path="description".into() label=t.get("te_description") max=300 localized=true/>
                    <Tags draft=draft/>
                    <Upload draft=draft id=id.get_value() path="".into() files=files notice=notice pack=true/>
                </Panel>
                <Panel title=t.get("te_colors")>{colors}</Panel>
                <Panel title=t.get("te_fonts")>{fonts}</Panel>
                <Panel title=t.get("te_background")>
                    <Upload draft=draft id=id.get_value() path="background.image".into() files=files notice=notice/>
                    <Slider draft=draft path="background.imageOpacity".into() label=t.get("te_bg_opacity") min=0. max=1. step=0.05/>
                    <Slider draft=draft path="background.imageBlur".into() label=t.get("te_bg_blur") min=0. max=20. step=1./>
                    <Choice draft=draft path="background.ornament".into() label=t.get("te_scenery") options=strings(ORNAMENTS) prefix="te_sc_".into() nullable=true/>
                </Panel>
                <Panel title=t.get("te_glows")><p class="text-xs text-gray-500">{t.get("te_glows_help")}</p><Glows draft=draft/></Panel>
                <Panel title=t.get("te_style")><p class="text-xs text-gray-500">{t.get("te_style_help")}</p>{styles}</Panel>
                {scenes}
                <Panel title=t.get("te_celebration")>
                    <Choice draft=draft path="celebration.type".into() label=t.get("te_celebration") options=strings(CELEBRATIONS) prefix="te_cb_".into() nullable=true/>
                    <p class="text-xs text-gray-500">{t.get("te_celebration_help")}</p>
                    <button type="button" class=BUTTON disabled=move || theme.get().celebration.r#type.is_none()
                        on:click=move |_| { draft.scene.set("leaderboard".into()); draft.celebrate.update(|n| *n += 1); }>{t.get("te_cb_play")}</button>
                </Panel>
                <Panel title=t.get("te_sounds")>
                    <p class="text-xs text-gray-500">{t.get("te_sounds_help")}</p>
                    <Choice draft=draft path="sounds.pack".into() label=t.get("te_sounds") options=strings(SOUND_PACKS) prefix="te_sd_".into() nullable=true/>
                    <div class="flex flex-wrap gap-2">{sounds}</div>
                </Panel>
                <Panel title=t.get("te_decoration")>
                    <Choice draft=draft path="decoration.type".into() label=t.get("te_decoration") options=strings(DECORATIONS) prefix="te_dc_".into() nullable=true/>
                    <p class="text-xs text-gray-500">{t.get("te_decoration_help")}</p>
                </Panel>
            </aside>
            <div class="space-y-3 lg:sticky lg:top-24 self-start">
                <Preview theme=theme files=Signal::from(files) quiz=preview_quiz.clone() selected_scene=draft.scene selected_wall=draft.wall celebrate=draft.celebrate/>
                <div class="rounded-xl border border-yellow-500/40 p-3 text-sm" role="status">
                    {move || {
                        let issues = readability_issues(&theme.get());
                        if issues.is_empty() {
                            vec![view! { <p class="text-green-400">{t.get("te_r_ok")}</p> }.into_any()]
                        } else {
                            issues.iter().map(|code| view! { <p class="text-yellow-200">{t.get(&format!("te_r_{code}"))}</p> }.into_any()).collect()
                        }
                    }}
                </div>
            </div>
        </div>
        <ShareDialog theme=theme files=files notice=notice open=sharing/>
    }
}

#[component]
fn FontField(
    draft: Draft,
    role: &'static str,
    id: String,
    files: RwSignal<Vec<Media>>,
    notice: RwSignal<String>,
) -> impl IntoView {
    let t = use_t();
    let path = StoredValue::new(format!("fonts.{role}"));
    let read = move || path.with_value(|p| value_at(&draft.raw.get(), p));
    view! {
        <label class="block text-sm text-gray-400">
            {t.get(&format!("te_f_{role}"))}
            <select class=INPUT prop:value=move || {
                let value = read();
                if value["file"].is_string() { "__file".into() } else { value["family"].as_str().unwrap_or("").to_owned() }
            } on:change=move |ev| {
                let family = event_target_value(&ev);
                if family != "__file" { path.with_value(|p| draft.set(p, if family.is_empty() { Value::Null } else { json!({"family": family}) })); }
            }>
                <option value="">{t.get("te_f_default")}</option>
                <option value="__file" disabled=true>{move || t.fmt("te_f_file", &[("name", read()["file"].as_str().unwrap_or(""))])}</option>
                {FONT_GROUPS.iter().map(|(group, fonts)| view! {
                    <optgroup label=t.get(&format!("te_f_group_{group}"))>
                        {fonts.iter().map(|font| view! { <option value=*font>{*font}</option> }).collect_view()}
                    </optgroup>
                }).collect_view()}
            </select>
        </label>
        <Upload draft=draft id=id path=path.get_value() files=files notice=notice font=true/>
    }
}

#[component]
fn ShareDialog(
    theme: Signal<Theme>,
    files: RwSignal<Vec<Media>>,
    notice: RwSignal<String>,
    open: RwSignal<bool>,
) -> impl IntoView {
    let t = use_t();
    let key = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            open.set(false);
        }
    });
    on_cleanup(move || key.remove());
    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4" on:click=move |_| open.set(false)>
                <section role="dialog" aria-modal="true" aria-label=t.get("te_share_title")
                    class="theme-dialog w-full max-w-lg rounded-2xl bg-[#0d0d10] border border-white/15 p-6 space-y-4" on:click=|ev| ev.stop_propagation()>
                    <button type="button" class=BUTTON on:click=move |_| open.set(false)>{t.get("chk_close")}</button>
                    <h2 class="text-xl font-bold">{t.get("te_share_title")}</h2>
                    <ol class="list-decimal pl-5 space-y-2">{["te_share_step1", "te_share_step2", "te_share_step3"].map(|key| view! { <li>{t.get(key)}</li> })}</ol>
                    <p class="text-gray-400 text-sm">{t.get("te_share_github")}</p>
                    <p class="text-yellow-300">{move || {
                        if theme.get().author.trim().is_empty() { t.get("te_share_missing") }
                        else if !readability_issues(&theme.get()).is_empty() { t.get("te_share_readability") }
                        else { String::new() }
                    }}</p>
                    <button type="button" class=BUTTON disabled=move || theme.get().author.trim().is_empty() on:click=move |_| {
                        let theme = theme.get_untracked();
                        match download(&theme, &files.get_untracked()) {
                            Ok(()) => { let _ = window().open_with_url_and_target_and_features(&share_url(&theme, t.lang().code()), "_blank", "noopener"); },
                            Err(error) => notice.set(error),
                        }
                    }>{t.get("te_share_go")}</button>
                </section>
            </div>
        </Show>
    }
}
fn strings(options: &[&str]) -> Vec<String> {
    options.iter().map(|s| s.to_string()).collect()
}
fn share_url(theme: &Theme, lang: &str) -> String {
    let encode = |s: &str| {
        s.bytes()
            .map(|b| {
                if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                    (b as char).to_string()
                } else {
                    format!("%{b:02X}")
                }
            })
            .collect::<String>()
    };
    let tags = theme.tags.join(", ");
    let title = format!("Theme: {}", theme_name(theme, lang));
    let description = theme
        .description
        .get(lang)
        .or_else(|| theme.description.get("en"))
        .or_else(|| theme.description.values().next())
        .map(String::as_str)
        .unwrap_or("");
    let params = [
        ("template", "theme.yml"),
        ("title", &title),
        ("name", theme_name(theme, lang)),
        ("author", &theme.author),
        ("description", description),
        ("tags", &tags),
    ]
    .iter()
    .map(|(k, v)| format!("{k}={}", encode(v)))
    .collect::<Vec<_>>()
    .join("&");
    format!("https://github.com/Jakub-Kos/QuizNightWebApp/issues/new?{params}")
}
#[component]
fn Glows(draft: Draft) -> impl IntoView {
    let t = use_t();
    let count = move || {
        draft.raw.get()["background"]["glows"]
            .as_array()
            .map_or(0, Vec::len)
    };
    view! {
        <For each={move || (0..count()).collect::<Vec<_>>()} key=|i| *i let:i>
            <div class="rounded-xl border border-white/10 p-3 space-y-2">
                <p class="text-sm text-gray-400">{t.fmt("te_glow_n", &[("n", &(i + 1).to_string())])}</p>
                <ColorField draft=draft path=format!("background.glows.{i}.color") label=t.get("te_glow_color") fallback="#3b82f6"/>
                {[("x", 0., 100., 1.), ("y", 0., 100., 1.), ("size", 5., 150., 1.), ("opacity", 0., 1., 0.01)]
                    .map(|(key, min, max, step)| view! {
                        <Slider draft=draft path=format!("background.glows.{i}.{key}") label=t.get(&format!("te_glow_{key}")) min=min max=max step=step/>
                    })}
                <button type="button" class=BUTTON on:click=move |_| {
                    draft.raw.update(|raw| { if let Some(glows) = raw["background"]["glows"].as_array_mut() { if i < glows.len() { glows.remove(i); } } });
                    draft.changed.update(|n| *n += 1);
                }>{t.get("te_remove")}</button>
            </div>
        </For>
        <button type="button" class=BUTTON disabled={move || count() >= 6} on:click=move |_| {
            draft.raw.update(|raw| {
                let color = raw["colors"]["primary"].as_str().unwrap_or("#3b82f6").to_owned();
                if let Some(glows) = raw["background"]["glows"].as_array_mut() { glows.push(json!({"color": color, "x": 50, "y": 100, "size": 60, "opacity": 0.2})); }
            });
            draft.changed.update(|n| *n += 1);
        }>{t.get("te_glow_add")}</button>
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_edits_reset_and_normalize() {
        let mut raw = serde_json::to_value(theme_by_id("default")).unwrap();
        set_path(&mut raw, "scenes.round.layout", json!("slam"));
        set_path(&mut raw, "colors.primary", json!("#abc"));
        assert_eq!(normalized(&raw).colors["primary"], "#aabbcc");
        assert_eq!(
            normalized(&raw).scenes.round.unwrap().layout,
            RoundLayout::Slam
        );
        set_path(
            &mut raw,
            "background.glows",
            json!([{"color":"#aabbcc","x":50,"y":100,"size":60,"opacity":0.2}]),
        );
        set_path(&mut raw, "background.glows.0.opacity", json!(0.35));
        assert_eq!(value_at(&raw, "background.glows.0.opacity"), json!(0.35));
        assert_eq!(normalized(&raw).background.glows[0].opacity, 0.35);
        set_path(&mut raw, "colors.primary", Value::Null);
        assert!(!normalized(&raw).colors.contains_key("primary"));
    }
    #[test]
    fn uploads_match_react() {
        assert_eq!(
            upload_name("Český Font.WOFF2", true, 5).unwrap(),
            "theme-cesky-font.woff2"
        );
        assert_eq!(upload_name("x.svg", false, 5).unwrap_err(), "file_type");
        assert_eq!(
            upload_name("x.png", false, 8 * 1024 * 1024 + 1).unwrap_err(),
            "file_size"
        );
        assert!(upload_name("x.otf", true, 2 * 1024 * 1024).is_ok());
    }
    #[test]
    fn packs_and_sharing() {
        let mut theme = theme_by_id("default").clone();
        theme.background.image = Some("theme-a.png".into());
        theme.author = "A & B".into();
        let bytes = export_theme_pack(
            &theme,
            &[PackAsset {
                name: "theme-a.png".into(),
                bytes: vec![1, 2],
            }],
        )
        .unwrap();
        let pack = read_theme_pack(&bytes).unwrap();
        let (theme, assets) = theme_for_quiz_media(&pack);
        assert_eq!(theme.background.image.as_deref(), Some("theme-a.png"));
        assert_eq!(assets[0].bytes, vec![1, 2]);
        assert!(share_url(&theme, "en").contains("author=A%20%26%20B"));
    }
}
