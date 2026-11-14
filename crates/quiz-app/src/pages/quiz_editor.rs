//! Local quiz setup, including media and theme packs.
// Drive teams are reconciled with the scores tab through the shared config sync.
mod drive_ui;
mod dropped_files;
mod launcher;
mod preview;
mod questions_start;
use crate::{
    i18n::{T, use_t},
    router::Route,
    store,
};
use launcher::Launcher;
use leptos::{prelude::*, task::spawn_local};
use quiz_core::{
    quiz::{
        intro,
        model::{Quiz, TEAM_HEX, Team, TeamColor},
        parse::{ParsedScores, is_url, media_key, parse_questions_csv, parse_scores_csv},
        teams,
        validate::{self, Level, ValidationInput, ValidationTeam},
    },
    theme::{builtin, format::Theme, pack},
};
use quiz_store::{Media, MediaStore, QuizStore, ThemeStore};
use serde_json::{Value, json};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use wasm_bindgen::JsCast;

const INPUT: &str = "w-full bg-black/40 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-gray-600 focus:outline-none focus:border-yellow-500/60";
const BUTTON: &str = "flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-white/10 hover:bg-white/20 text-sm whitespace-nowrap disabled:opacity-50";
const FIELD: &str = "w-full bg-black/40 border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-gray-600 focus:outline-none focus:border-yellow-500/60";
const PRIMARY: &str = "flex items-center justify-center gap-2 px-5 py-3 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold";
type Patch = BTreeMap<String, Value>;

/// Settings are patched one key at a time: the show also writes this object.
fn changes(before: &Quiz, after: &Quiz) -> Patch {
    let before = serde_json::to_value(before).unwrap();
    let after = serde_json::to_value(after).unwrap();
    let mut patch = Patch::new();
    for key in [
        "title",
        "questionsCsv",
        "questionsSheetUrl",
        "scoresSheetUrl",
        "teams",
        "rules",
        "prizes",
        "customTheme",
    ] {
        if before[key] != after[key] {
            patch.insert(key.into(), after[key].clone());
        }
    }
    for (key, value) in after["settings"].as_object().unwrap() {
        if &before["settings"][key] != value {
            patch.insert(format!("settings.{key}"), value.clone());
        }
    }
    patch
}
fn apply_patch(quiz: &mut Quiz, patch: &Patch) {
    let mut value = serde_json::to_value(&*quiz).unwrap();
    for (key, next) in patch {
        if let Some(key) = key.strip_prefix("settings.") {
            value["settings"][key] = next.clone();
        } else if next.is_null() {
            value.as_object_mut().unwrap().remove(key);
        } else {
            value[key] = next.clone();
        }
    }
    if let Ok(next) = serde_json::from_value(value) {
        *quiz = next;
    }
}
#[derive(Default)]
struct SaveQueue {
    pending: Patch,
    running: bool,
    generation: u64,
}
#[derive(Clone)]
struct Data {
    store: store::Store,
    id: String,
    queue: Rc<RefCell<SaveQueue>>,
}
#[derive(Clone, Copy)]
struct Editor {
    data: StoredValue<Data, LocalStorage>,
    quiz: RwSignal<Quiz>,
    media: RwSignal<Vec<Media>>,
    urls: RwSignal<BTreeMap<String, String>>,
    lease: StoredValue<Option<store::MediaUrls>, LocalStorage>,
    save: RwSignal<String>,
    status: RwSignal<Option<(bool, String)>>,
    busy: RwSignal<bool>,
    opened: RwSignal<String>,
    drive_progress: RwSignal<Option<crate::google::drive::Progress>>,
    drive_uploading: RwSignal<bool>,
    launcher: RwSignal<bool>,
    scores_sheet_mode: RwSignal<bool>,
    signed_in: RwSignal<bool>,
    t: T,
}
impl Editor {
    fn edit(self, change: impl FnOnce(&mut Quiz)) {
        let before = self.quiz.get_untracked();
        let mut after = before.clone();
        change(&mut after);
        let patch = changes(&before, &after);
        if patch.is_empty() {
            return;
        }
        self.quiz.set(after);
        let data = self.data.get_value();
        {
            let mut queue = data.queue.borrow_mut();
            queue.pending.extend(patch);
            queue.generation += 1;
        }
        self.start_save();
    }
    fn start_save(self) {
        let data = self.data.get_value();
        if data.queue.borrow().running || data.queue.borrow().pending.is_empty() {
            return;
        }
        data.queue.borrow_mut().running = true;
        self.save.set(self.t.get("ed_saving"));
        let saved = self.t.get("ed_saved");
        let not_found = self.t.get("ed_not_found");
        spawn_local(async move {
            loop {
                let generation = data.queue.borrow().generation;
                gloo_timers::future::TimeoutFuture::new(400).await;
                if generation != data.queue.borrow().generation {
                    continue;
                }
                let patch = std::mem::take(&mut data.queue.borrow_mut().pending);
                let to_save = patch.clone();
                let result = data
                    .store
                    .update_quiz(&data.id, move |q| apply_patch(q, &to_save))
                    .await;
                if let Err(e) = result {
                    let mut queue = data.queue.borrow_mut();
                    for (k, v) in patch {
                        queue.pending.entry(k).or_insert(v);
                    }
                    queue.running = false;
                    self.save.try_set(e.to_string());
                    return;
                }
                if matches!(result, Ok(None)) {
                    let mut queue = data.queue.borrow_mut();
                    for (k, v) in patch {
                        queue.pending.entry(k).or_insert(v);
                    }
                    queue.running = false;
                    self.save.try_set(not_found.clone());
                    return;
                }
                if data.queue.borrow().pending.is_empty() {
                    data.queue.borrow_mut().running = false;
                    self.save.try_set(saved);
                    return;
                }
            }
        });
    }
    fn id(self) -> String {
        self.data.with_value(|d| d.id.clone())
    }
    fn link(self, route: impl FnOnce(String) -> Route) -> String {
        route(self.id()).hash()
    }
    fn message(self, result: Result<String, String>) {
        self.status.try_set(Some(match result {
            Ok(s) => (true, s),
            Err(s) => (false, s),
        }));
    }
    async fn reload_media(self) -> Result<(), String> {
        let data = self.data.get_value();
        let mut media = data
            .store
            .list_media(&data.id)
            .await
            .map_err(|e| e.to_string())?;
        media.sort_by(|a, b| a.name.cmp(&b.name));
        let urls = store::media_urls(&data.store, &data.id).await?;
        self.urls.try_set(urls.0.clone());
        self.media.try_set(media);
        self.lease.try_update_value(|lease| *lease = Some(urls));
        Ok(())
    }
    fn has_media(self, name: &str) -> bool {
        is_url(name)
            || self
                .media
                .with(|m| m.iter().any(|m| media_key(&m.name) == media_key(name)))
    }
    fn url(self, name: &str) -> String {
        if is_url(name) {
            name.into()
        } else {
            self.urls
                .with(|u| u.get(&media_key(name)).cloned().unwrap_or_default())
        }
    }
    fn missing(self) -> Vec<String> {
        quiz_store::package::referenced_media(&self.quiz.get())
            .into_iter()
            .filter(|name| !self.has_media(name))
            .collect()
    }
    fn issues(self) -> Vec<validate::Issue> {
        let q = self.quiz.get();
        validate::validate_quiz(
            &ValidationInput {
                teams: q
                    .teams
                    .iter()
                    .map(|t| ValidationTeam {
                        name: t.name.clone(),
                        image: t.image.clone(),
                    })
                    .collect(),
                scores_sheet_url: Some(q.scores_sheet_url.clone()),
                builtin: q.builtin,
            },
            &parse_questions_csv(&q.questions_csv),
            |n| self.has_media(n),
        )
    }
    fn warning(self, key: &str) -> String {
        let q = self.quiz.get();
        match key {
            "drive" if q.drive.is_some() && !self.signed_in.get() => self.t.get("ed_warn_signin"),
            "questions" if parse_questions_csv(&q.questions_csv).is_empty() => {
                self.t.get("ed_warn_questions")
            }
            "media" if !self.missing().is_empty() => self.t.fmt(
                "ed_warn_media",
                &[("count", &self.missing().len().to_string())],
            ),
            "scores" if self.scores_sheet_mode.get() && q.scores_sheet_url.trim().is_empty() => {
                self.t.get("ed_warn_scores")
            }
            "teams" if !q.teams.iter().any(|t| !t.name.trim().is_empty()) => {
                self.t.get("ed_warn_teams")
            }
            "intro"
                if q.prizes.enabled
                    && [
                        &q.prizes.first,
                        &q.prizes.second,
                        &q.prizes.third,
                        &q.prizes.extra,
                    ]
                    .iter()
                    .all(|s| s.trim().is_empty()) =>
            {
                self.t.get("ed_warn_prizes")
            }
            "theme"
                if !q
                    .settings
                    .fields
                    .get("theme")
                    .is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty())) =>
            {
                self.t.get("ed_warn_theme")
            }
            _ => String::new(),
        }
    }
    fn team(self, key: &str) -> Team {
        self.quiz.with(|q| {
            q.teams
                .iter()
                .enumerate()
                .find(|(i, t)| team_key(t, *i) == key)
                .map(|(_, t)| t.clone())
                .unwrap_or_default()
        })
    }
    fn edit_team(self, key: &str, change: impl FnOnce(&mut Team)) {
        self.edit(|q| {
            if let Some((_, team)) = q
                .teams
                .iter_mut()
                .enumerate()
                .find(|(i, t)| team_key(t, *i) == key)
            {
                change(team);
            }
        });
    }
}
fn team_background(index: usize) -> String {
    let end = [
        "#312e81", "#831843", "#155e75", "#134e4a", "#dc2626", "#c2410c", "#1d4ed8", "#1c1917",
    ];
    format!(
        "linear-gradient(to bottom right, {}, {})",
        TEAM_HEX[index % TEAM_HEX.len()],
        end[index % end.len()]
    )
}
fn team_key(team: &Team, index: usize) -> String {
    team.id
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap())
        .unwrap_or_else(|| format!("row-{index}"))
}
fn files(input: &web_sys::HtmlInputElement) -> Vec<web_sys::File> {
    let result = input
        .files()
        .map(|f| (0..f.length()).filter_map(|i| f.get(i)).collect())
        .unwrap_or_default();
    input.set_value("");
    result
}
async fn bytes(file: &web_sys::File) -> Result<Vec<u8>, String> {
    wasm_bindgen_futures::JsFuture::from(file.array_buffer())
        .await
        .map(|v| js_sys::Uint8Array::new(&v).to_vec())
        .map_err(|e| format!("{e:?}"))
}
async fn text(file: &web_sys::File) -> Result<String, String> {
    String::from_utf8(bytes(file).await?).map_err(|e| e.to_string())
}
async fn sheet(url: &str, t: T) -> Result<String, String> {
    crate::google::sheets::fetch_sheet_csv(url)
        .await
        .map_err(|e| {
            crate::google::sheets::sheet_error_text(&e, &super::drive_parts::translations(t))
        })
}
fn download(bytes: &[u8], name: &str, mime: &str) -> Result<(), String> {
    let url = web_sys::Url::create_object_url_with_blob(&store::blob(bytes, mime)?)
        .map_err(|e| format!("{e:?}"))?;
    let anchor = document()
        .create_element("a")
        .map_err(|e| format!("{e:?}"))?
        .unchecked_into::<web_sys::HtmlAnchorElement>();
    anchor.set_href(&url);
    anchor.set_download(name);
    anchor.click();
    gloo_timers::callback::Timeout::new(1000, move || {
        let _ = web_sys::Url::revoke_object_url(&url);
    })
    .forget();
    Ok(())
}
fn size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024. * 1024.))
    }
}

#[component]
fn Icon(#[prop(default = "plus")] name: &'static str) -> impl IntoView {
    let path = match name {
        "back" => "M19 12H5m7 7-7-7 7-7",
        "play" => "m8 5 12 7-12 7Z",
        "trash" => "M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7m4-7v7",
        "upload" => "M12 16V3m-5 5 5-5 5 5M3 16v5h18v-5",
        "check" => "m5 12 4 4L19 6",
        "warning" => "m12 3 10 18H2Zm0 6v5m0 3v1",
        "help" => "M9 9a3 3 0 1 1 5 2c-2 1-2 2-2 3m0 3v1M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0",
        "down" => "m6 9 6 6 6-6",
        "audio" => "M9 18V5l12-2v13M9 8l12-2M9 18a3 3 0 1 1-3-3h3m12 1a3 3 0 1 1-3-3h3",
        "video" => "M3 5h14v14H3Zm14 5 5-3v10l-5-3",
        "Brain" => "M12 5c-4-5-10 2-7 5-5 5 2 12 7 8m0-13c4-5 10 2 7 5 5 5-2 12-7 8V5",
        "Rocket" => "M8 16C8 6 15 3 21 3c0 6-3 13-13 13Zm0 0-5 5m5-11H3l-2 5h6m7 2v5l5-2v-5",
        "Zap" => "m13 2-9 12h7l-1 8 10-13h-8Z",
        "Crown" => "m3 5 4 5 5-7 5 7 4-5-2 14H5Z",
        "Flame" => "M12 2c0 8-7 6-7 13a7 7 0 0 0 14 0c0-4-2-7-4-9 0 5-3 6-3-4Z",
        "Shield" => "m12 2 9 4v7c0 6-9 9-9 9S3 19 3 13V6Z",
        "Beer" => "M4 6h13v15H4Zm13 3h5v8h-5M7 2v2m4-2v2m4-2v2M8 9v9m5-9v9",
        "Glasses" => {
            "M1 13a5 5 0 1 0 10 0 5 5 0 1 0-10 0Zm12 0a5 5 0 1 0 10 0 5 5 0 1 0-10 0Zm-2 0h2M1 12l2-7m20 7-2-7"
        }
        "Gavel" => "m14 3 7 7-4 4-7-7Zm-2 8L3 20m-1 2h10",
        "Microscope" => "m9 3 6 6-4 4-6-6Zm5 10c8-2 10 8 3 8H4m4-5v5",
        _ => "M12 5v14M5 12h14",
    };
    view! { <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d=path/></svg> }
}
#[component]
fn Status(editor: Editor) -> impl IntoView {
    view! { <div role="status" aria-live="polite" class="whitespace-pre-line">{move || editor.status.get().map(|(ok,text)| view! { <p class=if ok { "text-sm text-green-300" } else { "text-sm text-red-300" }>{text}</p> })}</div> }
}

fn remember_section(name: &str, folded: bool) {
    if let Ok(Some(storage)) = window().local_storage() {
        let mut saved = storage
            .get_item("quiznight.foldedSections")
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .filter(Value::is_object)
            .unwrap_or(json!({}));
        saved[name] = json!(folded);
        let _ = storage.set_item("quiznight.foldedSections", &saved.to_string());
    }
}
#[component]
fn Section(
    editor: Editor,
    name: &'static str,
    title: &'static str,
    #[prop(default = "")] help: &'static str,
    children: Children,
) -> impl IntoView {
    let open = RwSignal::new(
        window()
            .local_storage()
            .ok()
            .flatten()
            .and_then(|s| s.get_item("quiznight.foldedSections").ok().flatten())
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .is_none_or(|v| v[name] != true),
    );
    let node = NodeRef::<leptos::html::Section>::new();
    Effect::new(move |_| {
        if editor.opened.get() == name {
            open.set(true);
            remember_section(name, false);
            if let Some(node) = node.get() {
                node.scroll_into_view();
            }
        }
    });
    let toggle = move |_| {
        let next = !open.get_untracked();
        open.set(next);
        remember_section(name, !next);
    };
    view! { <section node_ref=node id=format!("editor-{name}") class=move || if editor.warning(name).is_empty() { "editor-section scroll-mt-6 relative rounded-2xl border p-6 space-y-4 bg-white/[0.04] border-white/10" } else { "editor-section scroll-mt-6 relative rounded-2xl border p-6 space-y-4 bg-yellow-500/[0.04] border-yellow-500/40" }>
        <div class="flex items-center gap-2"><h2 class="font-['League_Spartan'] text-xl font-bold uppercase tracking-widest"><button type="button" on:click=toggle aria-expanded=move || open.get() class="flex items-center gap-2 text-left uppercase hover:text-yellow-400"><span class:editor-folded=move || !open.get()><Icon name="down"/></span>{move || editor.t.get(title)}</button></h2><Show when=move || matches!(name, "questions" | "media" | "scores" | "teams")><a href=Route::Help(Some(name.into())).hash() class="text-gray-500 hover:text-yellow-400" aria-label=move || editor.t.get("lib_help")><Icon name="help"/></a></Show><Show when=move || !open.get() && !editor.warning(name).is_empty()><span class="text-yellow-400"><Icon name="warning"/></span></Show></div>
        <div hidden=move || !open.get() class="editor-section-body space-y-4"><p class="text-sm text-gray-400" hidden=help.is_empty()>{move || editor.t.get(help)}</p><p class="text-sm text-yellow-200" hidden=move || editor.warning(name).is_empty()>{move || editor.warning(name)}</p>{children()}</div>
    </section> }
}

#[component]
pub fn Page(id: String) -> impl IntoView {
    let t = use_t();
    let store = StoredValue::new_local(store::use_store());
    let id = StoredValue::new(id);
    let loaded = LocalResource::new(move || {
        let store = store.get_value();
        let id = id.get_value();
        async move { store.get_quiz(&id).await.map_err(|e| e.to_string()) }
    });
    view! { <main class="quiz-editor min-h-screen bg-[#050505] text-white font-sans"><Suspense fallback=move || view! { <p class="p-10 text-gray-400" role="status">{t.get("lib_importing")}</p> }>{move || loaded.get().map(|result| match result {
        Ok(Some(quiz)) => view! { <Loaded quiz=quiz store=store.get_value()/> }.into_any(),
        Ok(None) => view! { <div class="min-h-screen flex flex-col items-center justify-center gap-6"><p>{t.get("ed_not_found")}</p><a href="#/" class=PRIMARY>{t.get("settings_exit")}</a></div> }.into_any(),
        Err(error) => view! { <div class="p-10 space-y-4"><p role="alert">{error}</p><button class=BUTTON on:click=move |_| loaded.refetch()>{t.get("ed_refresh")}</button><a href="#/">{t.get("settings_exit")}</a></div> }.into_any(),
    })}</Suspense></main> }
}
#[component]
fn Loaded(quiz: Quiz, store: store::Store) -> impl IntoView {
    let scores_sheet_mode = RwSignal::new(!quiz.scores_sheet_url.trim().is_empty());
    let editor = Editor {
        launcher: RwSignal::new(false),
        scores_sheet_mode,
        signed_in: crate::google::auth::use_google_account().signed_in,
        data: StoredValue::new_local(Data {
            id: quiz.id.clone(),
            store,
            queue: Rc::new(RefCell::new(SaveQueue::default())),
        }),
        quiz: RwSignal::new(quiz),
        media: RwSignal::new(vec![]),
        urls: RwSignal::new(BTreeMap::new()),
        lease: StoredValue::new_local(None),
        save: RwSignal::new(String::new()),
        status: RwSignal::new(None),
        busy: RwSignal::new(false),
        opened: RwSignal::new(String::new()),
        drive_progress: RwSignal::new(None),
        drive_uploading: RwSignal::new(false),
        t: use_t(),
    };
    editor.save.set(editor.t.get("ed_saved"));
    spawn_local(async move {
        if let Err(e) = editor.reload_media().await {
            editor.message(Err(e));
        }
    });
    on_cleanup(move || {
        editor.lease.try_update_value(|v| *v = None);
    });
    let sync = crate::google::sync::use_drive_sync(&editor.id());
    let google = crate::google::auth::use_google_account();
    let signed = google.signed_in;
    // Pull completed config merges into the editor only while no local edits are pending.
    let revision = editor.data.with_value(|d| d.store.revision());
    Effect::new(move |_| {
        revision.get();
        editor.save.get();
        let data = editor.data.get_value();
        if data.queue.borrow().running || !data.queue.borrow().pending.is_empty() {
            return;
        }
        spawn_local(async move {
            if let Ok(Some(q)) = data.store.get_quiz(&data.id).await
                && !data.queue.borrow().running
                && data.queue.borrow().pending.is_empty()
            {
                editor.quiz.try_set(q);
            }
        });
    });
    view! { <div class="max-w-4xl mx-auto px-6 py-10 space-y-6" aria-busy=move || editor.busy.get()>
        <header class="flex flex-wrap items-center gap-4"><a href="#/" class="flex items-center gap-2 text-gray-400 hover:text-white"><Icon name="back"/>{move || editor.t.get("settings_exit")}</a><a href="#/help" class="flex items-center gap-2 text-gray-400 text-sm"><Icon name="help"/>{move || editor.t.get("lib_help")}</a><Show when=move ||editor.quiz.with(|q|q.drive.is_none())><button class="ml-auto text-xs text-gray-400" on:click=move |_| editor.start_save() title=move || editor.t.get("ed_refresh") aria-live="polite">{move || editor.save.get()}</button></Show><Show when=move ||editor.quiz.with(|q|q.drive.is_some())><super::drive_parts::SaveStatus saving=Signal::derive(move ||editor.save.get()==editor.t.get("ed_saving")) drive=Signal::derive(move ||editor.quiz.with(|q|q.drive.is_some())) state=sync on_refresh=Callback::new(move |_| { if signed.get_untracked() { spawn_local(async move { if let Err(e)=crate::google::sync::sync_drive_config(&editor.id()).await { editor.message(Err(super::drive_parts::error_text(&e,editor.t).unwrap_or_default())); } }); } })/></Show><a class=BUTTON href=editor.link(Route::Map)>{move || editor.t.get("mp_open")}</a><a class=BUTTON href=editor.link(Route::Check)>{move || editor.t.get("chk_open")}</a><button class=PRIMARY on:click=move |_| editor.launcher.set(true)><Icon name="play"/>{move || editor.t.get("sl_title")}</button></header>
        <input aria-label=move || editor.t.get("ed_untitled") prop:value=move || editor.quiz.with(|q| q.title.clone()) on:input=move |ev| editor.edit(|q| q.title = event_target_value(&ev)) placeholder=move || editor.t.get("ed_untitled") class="w-full bg-transparent font-['League_Spartan'] text-4xl font-black tracking-wide border-b border-white/10 pb-3 focus:outline-none focus:border-yellow-500/60"/>
        <Show when=move || editor.launcher.get()><Launcher editor/></Show>
        <Status editor/>
        <Checklist editor/>
        <drive_ui::DriveSection editor/>
        <Themes editor/>
        <Questions editor/>
        <MediaSection editor/>
        <Scores editor/>
        <Teams editor/>
        <Intro editor/>
        <Settings editor/>
        <a href=editor.link(Route::Presenter) class=BUTTON>{move || editor.t.get("sl_presenter_open")}</a>
    </div> }
}

#[derive(Clone, PartialEq)]
struct Step {
    key: &'static str,
    label: &'static str,
    status: String,
    todo: bool,
    error: bool,
}
fn checklist(editor: Editor) -> Vec<Step> {
    let q = editor.quiz.get();
    let rounds = parse_questions_csv(&q.questions_csv);
    let count: usize = rounds.iter().map(|r| r.questions.len()).sum();
    let named = q.teams.iter().filter(|t| !t.name.trim().is_empty()).count();
    let issues = editor.issues();
    let errors = issues.iter().filter(|i| i.level == Level::Error).count();
    let warnings = issues.iter().filter(|i| i.level == Level::Warning).count();
    let t = editor.t;
    let theme = selected_theme(&q);
    let theme_name = builtin::theme_name(&theme, t.lang().code()).to_owned();
    let keys = [
        ("theme", "ed_theme"),
        ("questions", "ed_questions"),
        ("media", "ed_media"),
        ("teams", "tm_title"),
        ("scores", "ed_scores"),
        ("intro", "ed_intro"),
        ("check", "chk_open"),
    ];
    keys.into_iter()
        .chain(q.drive.is_some().then_some(("drive", "rd_google")))
        .map(|(key, label)| {
            let todo = if key == "check" {
                errors > 0
            } else {
                !editor.warning(key).is_empty()
            };
            let status = match key {
                "drive" => t.get(if todo {
                    "rd_google_todo"
                } else {
                    "rd_google_ok"
                }),
                "theme" => {
                    if todo {
                        t.get("rd_theme_todo")
                    } else {
                        theme_name.clone()
                    }
                }
                "questions" => {
                    if todo {
                        t.get("ed_warn_questions")
                    } else {
                        t.fmt(
                            "rd_questions_ok",
                            &[
                                ("rounds", &rounds.len().to_string()),
                                ("questions", &count.to_string()),
                            ],
                        )
                    }
                }
                "media" => {
                    if todo {
                        t.fmt(
                            "rd_media_todo",
                            &[("count", &editor.missing().len().to_string())],
                        )
                    } else {
                        let n = quiz_store::package::referenced_media(&q).len();
                        if n == 0 {
                            t.get("rd_media_none")
                        } else {
                            t.fmt("rd_media_ok", &[("count", &n.to_string())])
                        }
                    }
                }
                "teams" => {
                    if todo {
                        t.get("ed_warn_teams")
                    } else {
                        t.fmt("rd_teams_ok", &[("count", &named.to_string())])
                    }
                }
                "scores" => t.get(if todo {
                    "rd_scores_todo"
                } else if q.scores_sheet_url.is_empty() {
                    "rd_scores_manual"
                } else {
                    "rd_scores_sheet"
                }),
                "intro" => t.get(if todo { "rd_intro_todo" } else { "rd_intro_ok" }),
                _ => {
                    if errors > 0 {
                        t.fmt("rd_check_err", &[("count", &errors.to_string())])
                    } else if warnings > 0 {
                        t.fmt("rd_check_warn", &[("count", &warnings.to_string())])
                    } else {
                        t.get("rd_check_ok")
                    }
                }
            };
            Step {
                key,
                label,
                status,
                todo,
                error: key == "check" && errors > 0,
            }
        })
        .collect()
}
#[component]
fn Checklist(editor: Editor) -> impl IntoView {
    let steps = Memo::new(move |_| checklist(editor));
    view! { <section class="rounded-2xl bg-white/[0.04] border border-white/10 p-5 space-y-4"><div class="flex flex-wrap items-center gap-4"><div class="flex-1 min-w-[200px] space-y-2"><h2 class="font-['League_Spartan'] text-xl font-bold uppercase tracking-widest">{move || editor.t.get("rd_title")}</h2><div class="flex items-center gap-3"><div class="h-1.5 flex-1 max-w-56 rounded-full bg-white/10 overflow-hidden"><div class=move || if steps.with(|s| s.iter().all(|s| !s.todo)) { "h-full rounded-full bg-green-500 transition-all" } else { "h-full rounded-full bg-yellow-500 transition-all" } style:width=move || format!("{}%", steps.with(|s| s.iter().filter(|s| !s.todo).count()*100/s.len()))/></div><span class="text-sm text-gray-400">{move || steps.with(|s| { let done = s.iter().filter(|s| !s.todo).count(); if done == s.len() { editor.t.get("rd_all_done") } else { editor.t.fmt("rd_progress", &[("done", &done.to_string()), ("total", &s.len().to_string())]) } })}</span></div></div><button class=PRIMARY on:click=move |_| editor.launcher.set(true)><Icon name="play"/>{move || editor.t.get("sl_title")}</button></div>
        <ol class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-2">{move || steps.get().into_iter().map(|s| view! { <li><button class=if s.error { "w-full h-full text-left rounded-xl border p-3 border-red-500/50 bg-red-500/[0.06]" } else if s.todo { "w-full h-full text-left rounded-xl border p-3 border-yellow-500/40 bg-yellow-500/[0.05]" } else { "w-full h-full text-left rounded-xl border p-3 border-green-500/20 hover:border-green-500/40" } on:click=move |_| { if s.key == "check" { crate::router::navigate(Route::Check(editor.id())); } else { editor.opened.set(String::new()); editor.opened.set(s.key.into()); } }><span class="flex items-center gap-2 font-bold text-sm"><span class=if s.error { "text-red-400" } else if s.todo { "text-yellow-400" } else { "text-green-400" }><Icon name=if s.todo { "warning" } else { "check" }/></span>{editor.t.get(s.label)}</span><span class="block text-xs text-gray-400 mt-1 pl-6">{s.status}</span></button></li> }).collect_view()}</ol>
    </section> }
}
fn selected_theme(q: &Quiz) -> Theme {
    let custom: Option<Theme> = q
        .fields
        .get("customTheme")
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    builtin::theme_for_quiz(
        q.settings
            .fields
            .get("theme")
            .and_then(Value::as_str)
            .unwrap_or("default"),
        custom.as_ref(),
    )
    .clone()
}
fn set_questions(editor: Editor, csv: String, url: String) {
    let rounds = parse_questions_csv(&csv);
    let msg = if rounds.is_empty() {
        editor.t.get("ed_no_rounds")
    } else {
        format!(
            "{}: {} {}",
            editor.t.get("ed_loaded"),
            rounds.len(),
            editor.t.get("lib_rounds")
        )
    };
    editor.edit(|q| {
        q.questions_csv = csv;
        q.questions_sheet_url = url;
    });
    editor.status.set(Some((!rounds.is_empty(), msg)));
}
#[component]
fn Questions(editor: Editor) -> impl IntoView {
    let url = RwSignal::new(editor.quiz.get_untracked().questions_sheet_url);
    let input = NodeRef::<leptos::html::Input>::new();
    let load = move |_| {
        let url = url.get_untracked().trim().to_owned();
        editor.busy.set(true);
        spawn_local(async move {
            match sheet(&url, editor.t).await {
                Ok(csv) => set_questions(editor, csv, url),
                Err(e) => editor.message(Err(e)),
            }
            editor.busy.try_set(false);
        });
    };
    let upload = move |_| {
        if let Some(file) = input.get().and_then(|i| {
            let file = files(&i).into_iter().next();
            i.set_value("");
            file
        }) {
            spawn_local(async move {
                match text(&file).await {
                    Ok(csv) => {
                        url.set(String::new());
                        set_questions(editor, csv, String::new());
                    }
                    Err(e) => editor.message(Err(e)),
                }
            });
        }
    };
    let examples = move |_| {
        editor.busy.set(true);
        spawn_local(async move {
            let example = quiz_core::quiz::examples::example_questions(
                &editor.quiz.get_untracked().settings.language,
                &editor.t.get("ed_example_round"),
            );
            let result = async {
                let mut media = vec![];
                for name in example.media {
                    let response = gloo_net::http::Request::get(&format!("source/{name}"))
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;
                    if !response.ok()
                        || response
                            .headers()
                            .get("content-type")
                            .is_some_and(|s| s.contains("text/html"))
                    {
                        return Err(format!("{name}: HTTP {}", response.status()));
                    }
                    media.push(Media {
                        name,
                        content_type: response.headers().get("content-type").unwrap_or_default(),
                        bytes: response.binary().await.map_err(|e| e.to_string())?,
                        ..Default::default()
                    });
                }
                let data = editor.data.get_value();
                data.store
                    .put_media(&data.id, &media)
                    .await
                    .map_err(|e| e.to_string())?;
                editor.reload_media().await?;
                url.set(String::new());
                set_questions(editor, example.csv, String::new());
                Ok(editor.t.get("qs_examples_done"))
            }
            .await;
            editor.message(result);
            editor.busy.try_set(false);
        });
    };
    view! { <Section editor name="questions" title="ed_questions" help=if editor.quiz.get_untracked().drive.is_some() { "gd_questions_note" } else { "ed_questions_help" }><div hidden=move ||editor.quiz.with(|q|q.drive.is_some()) class="flex flex-col sm:flex-row gap-2"><input type="url" class=INPUT aria-label=move || editor.t.get("ed_sheet_link") placeholder=move || editor.t.get("ed_sheet_link") prop:value=move || url.get() on:input=move |ev| url.set(event_target_value(&ev))/><button class=BUTTON on:click=load disabled=move || editor.busy.get() || url.get().trim().is_empty()>{move || editor.t.get(if editor.quiz.with(|q| q.questions_sheet_url == url.get() && !q.questions_sheet_url.is_empty()) { "ed_refresh" } else { "ed_load" })}</button><button class=BUTTON on:click=move |_| { if let Some(i) = input.get() { i.click(); } }>{move || editor.t.get("ed_upload_csv")}</button><input node_ref=input type="file" accept=".csv,text/csv" hidden on:change=upload/></div>
        <p class="text-xs text-gray-500" hidden=move || editor.quiz.with(|q| q.drive.is_some() || q.questions_sheet_url.is_empty())>{move || editor.t.get("ed_sheet_auto")}</p>
        <a class=BUTTON href=editor.link(Route::Questions)>{move || editor.t.get("qe_open")}</a>
        <details hidden=move ||editor.quiz.with(|q|q.drive.is_some())><summary class="text-sm text-gray-400 cursor-pointer">{move || editor.t.get("ed_upload_csv")}" / CSV"</summary><textarea class=INPUT rows="8" aria-label="CSV" prop:value=move || editor.quiz.with(|q| q.questions_csv.clone()) on:input=move |ev| { url.set(String::new()); editor.edit(|q| { q.questions_csv = event_target_value(&ev); q.questions_sheet_url.clear(); }); }/></details>
        {move || { let rounds = parse_questions_csv(&editor.quiz.get().questions_csv); let count: usize = rounds.iter().map(|r| r.questions.len()).sum(); if rounds.is_empty() && editor.quiz.with(|q|q.drive.is_some()) { view! { <p class="text-sm text-gray-500">{editor.t.get("ed_no_questions")}</p> }.into_any() } else if rounds.is_empty() { view! { <questions_start::Start editor on_examples=Callback::new(examples)/> }.into_any() } else { view! { <ul class="divide-y divide-white/5 rounded-xl bg-black/30">{rounds.iter().map(|r| view! { <li class="flex justify-between gap-4 px-4 py-2 text-sm"><span><span class="text-gray-500">{r.number.clone()}". "</span>{r.title.clone()}</span><span class="text-gray-500">{r.questions.len()}" "{editor.t.get("lib_questions")}</span></li> }).collect_view()}<li class="flex justify-between px-4 py-2 text-sm font-bold"><span>{rounds.len()}" "{editor.t.get("lib_rounds")}</span><span>{count}" "{editor.t.get("lib_questions")}</span></li></ul> }.into_any() } }}
    </Section> }
}
async fn add_media(editor: Editor, files: Vec<web_sys::File>) -> Result<String, String> {
    if editor.quiz.get_untracked().drive.is_some() && crate::google::auth::current_token().is_some()
    {
        return drive_ui::upload(editor, files).await;
    }
    let data = editor.data.get_value();
    let old = data
        .store
        .list_media(&data.id)
        .await
        .map_err(|e| e.to_string())?;
    let mut accepted: Vec<Media> = vec![];
    for file in files {
        if let Some(mime) = quiz_store::package::media_type(&file.name()) {
            let path = file.name();
            let media = Media {
                name: path,
                bytes: bytes(&file).await?,
                content_type: mime.into(),
                ..Default::default()
            };
            accepted.retain(|m| media_key(&m.name) != media_key(&media.name));
            accepted.push(media);
        }
    }
    // Upsert first so a failed upload cannot destroy the old file.
    data.store
        .put_media(&data.id, &accepted)
        .await
        .map_err(|e| e.to_string())?;
    for old in old {
        if accepted
            .iter()
            .any(|m| m.name != old.name && media_key(&m.name) == media_key(&old.name))
        {
            data.store
                .delete_media(&data.id, &old.name)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    editor.reload_media().await?;
    Ok(format!("{}: {}", editor.t.get("ed_loaded"), accepted.len()))
}
#[component]
fn MediaSection(editor: Editor) -> impl IntoView {
    let input = NodeRef::<leptos::html::Input>::new();
    let folder = NodeRef::<leptos::html::Input>::new();
    Effect::new(move |_| {
        if let Some(input) = folder.get() {
            let _ = input.set_attribute("webkitdirectory", "");
        }
    });
    let over = RwSignal::new(false);
    let upload = move |files| {
        editor.busy.set(true);
        spawn_local(async move {
            editor.message(add_media(editor, files).await);
            editor.busy.try_set(false);
        });
    };
    view! { <Section editor name="media" title="ed_media" help="ed_media_help">
        {move || { let missing = editor.missing(); (!missing.is_empty()).then(|| view! { <div class="p-4 rounded-xl bg-red-500/10 border border-red-500/30 space-y-2"><p class="text-red-200 font-bold text-sm">{editor.t.get("ed_missing")}" ("{missing.len()}")"</p><div class="flex flex-wrap gap-2">{missing.into_iter().map(|name| view! { <code class="px-2 py-1 rounded bg-black/40 text-red-200 text-xs">{name}</code> }).collect_view()}</div></div> }) }}
        <p class="text-green-300 text-sm" hidden=move || !editor.missing().is_empty() || quiz_store::package::referenced_media(&editor.quiz.get()).is_empty()>{move || editor.t.get("ed_all_media")}</p>
        <div class="editor-drop rounded-2xl border-2 border-dashed border-white/15 bg-white/[0.02] p-5 text-center space-y-4" class:editor-over=move || over.get() on:dragover=move |ev| { ev.prevent_default(); over.set(true); } on:dragleave=move |_| over.set(false) on:drop=move |ev| { ev.prevent_default(); over.set(false); if editor.busy.get_untracked() { return; } if let Some(transfer) = ev.data_transfer() { let entries = dropped_files::capture(&transfer); editor.busy.set(true); spawn_local(async move { let result = async { let files = dropped_files::collect(entries).await?; add_media(editor, files).await }.await; editor.message(result); editor.busy.try_set(false); }); } }><div class="flex justify-center text-gray-500"><Icon name="upload"/></div><p class="text-gray-300">{move || editor.t.get(if editor.quiz.with(|q|q.drive.is_some()) && editor.signed_in.get() { "ed_media_drop_drive" } else { "ed_media_drop" })}</p><div class="flex flex-wrap justify-center gap-2"><button class=BUTTON disabled=move || editor.busy.get() on:click=move |_| { if let Some(i) = input.get() { i.click(); } }>{move || editor.t.get("lib_import_files")}</button><button class=BUTTON disabled=move || editor.busy.get() on:click=move |_| { if let Some(i) = folder.get() { i.click(); } }>{move || editor.t.get("lib_import_folder")}</button></div><input node_ref=input type="file" multiple accept="image/*,audio/*,video/*" hidden on:change=move |_| { if let Some(i) = input.get() { upload(files(&i)); } }/><input node_ref=folder type="file" multiple hidden on:change=move |_| { if let Some(i) = folder.get() { upload(files(&i)); } }/></div>
        <Show when=move ||editor.drive_uploading.get()><super::drive_parts::DriveProgress progress=editor.drive_progress uploading=true/></Show>
        <p class="text-xs text-gray-500">{move || editor.media.with(|m| format!("{} · {}",m.len(),size(m.iter().map(|m| m.bytes.len()).sum())))}</p>
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3"><For each=move || editor.media.get() key=|m| (m.name.clone(), m.bytes.len(), m.content_type.clone()) children=move |m| {
            let name = m.name.clone(); let remove_name = name.clone(); let image_name = name.clone(); let used_name = name.clone();
            view! { <div class="rounded-xl bg-black/40 border border-white/10 overflow-hidden"><div class="aspect-video bg-black flex items-center justify-center text-gray-500">{if m.content_type.starts_with("audio") { view! { <Icon name="audio"/> }.into_any() } else if m.content_type.starts_with("video") { view! { <Icon name="video"/> }.into_any() } else { view! { <img src=move || editor.url(&image_name) alt=name.clone() class="w-full h-full object-contain"/> }.into_any() }}</div><div class="p-2 flex items-start gap-2"><div class="min-w-0 flex-1"><p class="text-xs truncate" title=name.clone()>{name.clone()}</p><p class="text-[10px] text-gray-500">{size(m.bytes.len())}<span class="text-yellow-500/80" hidden=move || quiz_store::package::referenced_media(&editor.quiz.get()).iter().any(|n| media_key(n) == media_key(&used_name))>" · "{editor.t.get("ed_unused")}</span></p></div><button class="p-1 rounded text-gray-500 hover:text-red-300" aria-label=move || editor.t.get("lib_delete") on:click=move |_| { let name = remove_name.clone(); spawn_local(async move { let data = editor.data.get_value(); let result = async { data.store.delete_media(&data.id,&name).await.map_err(|e| e.to_string())?; editor.reload_media().await?; Ok(editor.t.get("ed_saved")) }.await; editor.message(result); }); }><Icon name="trash"/></button></div></div> }
        }/></div>
    </Section> }
}

#[component]
fn Scores(editor: Editor) -> impl IntoView {
    let sheet_mode = editor.scores_sheet_mode;
    let signed = crate::google::auth::use_google_account().signed_in;
    let drive_tab = Memo::new(move |_| {
        editor.quiz.with(|q| {
            q.drive
                .as_ref()
                .and_then(|d| d["scoresTab"].as_str())
                .unwrap_or_default()
                .to_string()
        })
    });
    let last_url = RwSignal::new(String::new());
    let preview = RwSignal::new(None::<ParsedScores>);
    let status = RwSignal::new(None::<(bool, String)>);
    let test = move |_| {
        editor.busy.set(true);
        spawn_local(async move {
            let result = sheet(&editor.quiz.get_untracked().scores_sheet_url, editor.t).await;
            let result = result.and_then(|csv| {
                parse_scores_csv(&csv).ok_or_else(|| editor.t.get("ed_err_format"))
            });
            match result {
                Ok(scores) => {
                    let q = editor.quiz.get_untracked();
                    let unmatched: Vec<_> = scores
                        .teams
                        .iter()
                        .filter(|row| teams::team_for_scores(&q.teams, row).is_none())
                        .map(|row| row.name.clone())
                        .collect();
                    let mut text = editor.t.fmt(
                        "ed_scores_ok",
                        &[
                            ("teams", &scores.teams.len().to_string()),
                            ("rounds", &scores.rounds.len().to_string()),
                        ],
                    );
                    let ok = unmatched.is_empty() || q.teams.is_empty();
                    if !ok {
                        text.push(' ');
                        text.push_str(
                            &editor
                                .t
                                .fmt("ed_scores_unmatched", &[("names", &unmatched.join(", "))]),
                        );
                    }
                    status.try_set(Some((ok, text)));
                    preview.try_set(Some(scores));
                }
                Err(e) => {
                    status.try_set(Some((false, e)));
                    preview.try_set(None);
                }
            }
            editor.busy.try_set(false);
        });
    };
    let loaded = RwSignal::new(false);
    Effect::new(move |_| {
        if signed.get() && !drive_tab.get().is_empty() && !loaded.get_untracked() {
            loaded.set(true);
            test(());
        }
    });
    view! { <Section editor name="scores" title="ed_scores"><Show when=move ||!drive_tab.get().is_empty()><p class="text-sm text-gray-300">{move ||editor.t.fmt("ed_scores_drive",&[("tab",&drive_tab.get())])}</p><button class=BUTTON disabled=move ||!signed.get()||editor.busy.get() on:click=move |_|test(())>{move ||editor.t.get("ed_refresh")}</button></Show><div hidden=move ||!drive_tab.get().is_empty() class="flex gap-1 bg-white/5 rounded-xl p-1 w-fit text-sm">{[false,true].into_iter().map(|mode| view! { <button class=move || if sheet_mode.get() == mode { "px-3 py-1.5 rounded-lg bg-yellow-500 text-black font-bold" } else { "px-3 py-1.5 rounded-lg text-gray-400 hover:text-white" } on:click=move |_| { if sheet_mode.get_untracked() == mode { return; } sheet_mode.set(mode); status.set(None); preview.set(None); if mode { let url = last_url.get_untracked(); editor.edit(|q| q.scores_sheet_url = url); } else { last_url.set(editor.quiz.get_untracked().scores_sheet_url); editor.edit(|q| q.scores_sheet_url.clear()); } }>{move || editor.t.get(if mode { "ed_scores_mode_sheet" } else { "ed_scores_mode_manual" })}</button> }).collect_view()}</div>
        <div hidden=move || !sheet_mode.get() || !drive_tab.get().is_empty() class="space-y-3"><p class="text-sm text-gray-400">{move || editor.t.get("ed_scores_help")}</p><div class="flex flex-col sm:flex-row gap-2"><input class=INPUT type="url" aria-label=move || editor.t.get("ed_sheet_link") placeholder=move || editor.t.get("ed_sheet_link") prop:value=move || editor.quiz.with(|q| q.scores_sheet_url.clone()) on:input=move |ev| { status.set(None); preview.set(None); editor.edit(|q| q.scores_sheet_url = event_target_value(&ev).trim().into()); }/><button class=BUTTON on:click=move |_|test(()) disabled=move || editor.busy.get() || editor.quiz.with(|q| q.scores_sheet_url.is_empty())>{move || editor.t.get("ed_test")}</button></div><p class="text-sm text-yellow-200" hidden=move || !editor.quiz.with(|q| q.scores_sheet_url.is_empty())>{move || editor.t.get("ed_warn_scores")}</p></div>
        <div hidden=move || sheet_mode.get() || !drive_tab.get().is_empty() class="space-y-3"><p class="text-sm text-gray-400">{move || editor.t.get("ed_scores_manual_help")}</p><a class=BUTTON href=editor.link(Route::Scores)>{move || editor.t.get("sc_open")}</a></div>
        <div role="status">{move || status.get().map(|(ok,text)| view! { <p class=if ok { "text-sm text-green-300" } else { "text-sm text-red-300" }>{text}</p> })}</div>
        {move || preview.get().map(|scores| { let mut rows = scores.teams; rows.sort_by(|a,b| b.scores.values().sum::<f64>().total_cmp(&a.scores.values().sum::<f64>())); view! { <div class="overflow-x-auto"><table class="w-full text-sm text-left"><thead><tr><th class="p-2">{editor.t.get("tm_name")}</th>{scores.rounds.iter().map(|r| view! { <th class="p-2">{r.clone()}</th> }).collect_view()}<th class="p-2">{editor.t.get("ed_scores_total")}</th></tr></thead><tbody>{rows.iter().map(|row| { let matched = editor.quiz.with(|q| teams::team_for_scores(&q.teams,row).is_some()); view! { <tr class="border-t border-white/10"><td class=if matched { "p-2" } else { "p-2 text-yellow-300" } title=if matched { String::new() } else { editor.t.get("ed_scores_unmatched_one") }>{row.name.clone()}</td>{scores.rounds.iter().map(|r| view! { <td class="p-2 tabular-nums">{row.scores.get(r).map(|n| n.to_string()).unwrap_or_else(|| "–".into())}</td> }).collect_view()}<td class="p-2 font-bold">{row.scores.values().sum::<f64>()}</td></tr> } }).collect_view()}</tbody></table></div> } })}
    </Section> }
}
fn import_error(error: teams::ImportError, t: T) -> String {
    match error {
        teams::ImportError::NoNameColumn => t.get("tm_err_no_name_column"),
        teams::ImportError::NoHistoryColumns => t.get("tm_err_no_history_columns"),
        teams::ImportError::Csv(message) => message,
    }
}
#[component]
fn Teams(editor: Editor) -> impl IntoView {
    let registrations = NodeRef::<leptos::html::Input>::new();
    let history = NodeRef::<leptos::html::Input>::new();
    let read = move |file: web_sys::File, history: bool| {
        spawn_local(async move {
            let result = text(&file).await.and_then(|csv| {
                let q = editor.quiz.get_untracked();
                if history {
                    let result = teams::import_history_csv(&csv, &q.teams)
                        .map_err(|e| import_error(e, editor.t))?;
                    editor.edit(|q| q.teams = result.teams);
                    Ok((
                        result.matched > 0,
                        editor.t.fmt(
                            "tm_history_imported",
                            &[
                                ("matched", &result.matched.to_string()),
                                ("total", &result.total.to_string()),
                            ],
                        ),
                    ))
                } else {
                    let result = teams::import_teams_csv(
                        &csv,
                        &q.teams,
                        |n| editor.has_media(n),
                        store::new_id,
                    )
                    .map_err(|e| import_error(e, editor.t))?;
                    editor.edit(|q| q.teams = result.teams);
                    Ok((
                        true,
                        editor.t.fmt(
                            "tm_imported",
                            &[
                                ("added", &result.added.to_string()),
                                ("updated", &result.updated.to_string()),
                            ],
                        ),
                    ))
                }
            });
            match result {
                Ok(status) => {
                    editor.status.try_set(Some(status));
                }
                Err(error) => editor.message(Err(error)),
            }
        });
    };
    let from_scores = move |_| {
        editor.busy.set(true);
        spawn_local(async move {
            let result = async {
                let csv = sheet(&editor.quiz.get_untracked().scores_sheet_url, editor.t).await?;
                let scores = parse_scores_csv(&csv).ok_or_else(|| editor.t.get("ed_err_format"))?;
                if scores.teams.is_empty() {
                    return Err(editor.t.get("tm_from_scores_none"));
                }
                let result = teams::add_teams_from_scores(
                    &scores,
                    &editor.quiz.get_untracked().teams,
                    store::new_id,
                );
                editor.edit(|q| q.teams = result.teams);
                Ok(editor.t.fmt(
                    "tm_from_scores_done",
                    &[
                        ("added", &result.added.to_string()),
                        ("existing", &result.existing.to_string()),
                    ],
                ))
            }
            .await;
            editor.message(result);
            editor.busy.try_set(false);
        });
    };
    view! { <Section editor name="teams" title="tm_title" help="tm_help">{move ||editor.quiz.with(|q|q.drive.as_ref().and_then(|d|d["scoresTab"].as_str()).map(str::to_owned)).map(|tab|view!{<p class="text-sm text-gray-400">{editor.t.fmt("tm_from_sheet",&[("tab",&tab)])}</p>})}<div class="flex flex-wrap gap-2"><button hidden=move ||editor.quiz.with(|q|q.drive.as_ref().is_some_and(|d|d["scoresTab"].is_string())) class=BUTTON on:click=from_scores disabled=move || editor.busy.get() || editor.quiz.with(|q| q.scores_sheet_url.is_empty()) title=move || editor.t.get("tm_from_scores_need")>{move || editor.t.get("tm_from_scores")}</button><button class=BUTTON on:click=move |_| { if let Some(i) = registrations.get() { i.click(); } }>{move || editor.t.get("tm_import")}</button><button class=BUTTON disabled=move || editor.quiz.with(|q| q.teams.is_empty()) on:click=move |_| { if let Some(i) = history.get() { i.click(); } }>{move || editor.t.get("tm_import_history")}</button></div><input node_ref=registrations type="file" accept=".csv,text/csv" hidden on:change=move |_| { if let Some(file) = registrations.get().and_then(|i| files(&i).into_iter().next()) { read(file,false); } }/><input node_ref=history type="file" accept=".csv,text/csv" hidden on:change=move |_| { if let Some(file) = history.get().and_then(|i| files(&i).into_iter().next()) { read(file,true); } }/>
        <p class="text-sm text-gray-500" hidden=move || editor.quiz.with(|q| !q.teams.is_empty())>{move || editor.t.get("tm_empty")}</p>
        <ul class="space-y-2"><For each={move || editor.quiz.with(|q| q.teams.iter().enumerate().map(|(i,t)| team_key(t,i)).collect::<Vec<_>>())} key=|key| key.clone() children=move |key| view! { <TeamRow editor row_key=key/> }/></ul>
        <div class="flex flex-col sm:flex-row gap-2"><button class=BUTTON on:click=move |_| editor.edit(|q| { let mut team = teams::new_team(store::new_id()); team.color = Some(TeamColor::Palette(q.teams.len()%8)); if q.drive.as_ref().is_some_and(|d|d["scoresTab"].is_string()) {team.fields.insert("draft".into(),serde_json::json!(true));} q.teams.push(team); })><Icon/>{move || editor.t.get("tm_add")}</button><a class=BUTTON href=editor.link(Route::Attendance)>{move || editor.t.get("att_title")}</a></div>
    </Section> }
}
#[component]
fn TeamRow(editor: Editor, row_key: String) -> impl IntoView {
    let key = StoredValue::new(row_key);
    let team = move || editor.team(&key.get_value());
    let photo = NodeRef::<leptos::html::Input>::new();
    let upload = move |_| {
        if let Some(file) = photo.get().and_then(|i| files(&i).into_iter().next()) {
            let team = team();
            let key = key.get_value();
            spawn_local(async move {
                let result = async {
                    let name = teams::team_image_name(&team, &file.name());
                    let data = editor.data.get_value();
                    if editor.quiz.get_untracked().drive.is_some()
                        && editor.signed_in.get_untracked()
                    {
                        if let Some(path) =
                            drive_ui::upload_team_image(editor, &name, &file).await?
                        {
                            editor.edit_team(&key, |t| t.image = Some(path));
                            editor.reload_media().await?;
                        }
                        return Ok(editor.t.get("ed_saved"));
                    }
                    let media = Media {
                        name: name.clone(),
                        content_type: file.type_(),
                        bytes: bytes(&file).await?,
                        ..Default::default()
                    };
                    data.store
                        .put_media(&data.id, &[media])
                        .await
                        .map_err(|e| e.to_string())?;
                    editor.edit_team(&key, |t| t.image = Some(name.clone()));
                    if let Some(old) = team
                        .image
                        .filter(|old| old != &name && old.starts_with("team-"))
                    {
                        data.store
                            .delete_media(&data.id, &old)
                            .await
                            .map_err(|e| e.to_string())?;
                    }
                    editor.reload_media().await?;
                    Ok(editor.t.get("ed_saved"))
                }
                .await;
                editor.message(result);
            });
        }
    };
    let confirm_name = move || {
        if team().fields.get("draft") == Some(&serde_json::json!(true)) && !team().name.trim().is_empty() {
            editor.edit_team(&key.get_value(), |t| { t.fields.remove("draft"); });
        }
    };
    let duplicate = move || {
        let name = team().name.trim().to_lowercase();
        !name.is_empty()
            && editor.quiz.with(|q| {
                q.teams
                    .iter()
                    .filter(|t| t.name.trim().to_lowercase() == name)
                    .count()
                    > 1
            })
    };
    view! { <li class="flex flex-wrap items-center gap-3 p-3 rounded-xl bg-black/30 border border-white/5">
        <button class="relative w-14 h-14 shrink-0 rounded-xl overflow-hidden flex items-center justify-center" style:background=move || team_background(team().color.as_ref().map(|c| c.index(0)).unwrap_or(0)) title=move || editor.t.get("tm_photo") on:click=move |_| { if let Some(i) = photo.get() { i.click(); } }>{move || { let t = team(); if let Some(image) = t.image { view! { <img src=editor.url(&image) alt="" class="w-full h-full object-cover"/> }.into_any() } else { let icon = match t.icon.as_deref() { Some("Rocket") => "Rocket", Some("Zap") => "Zap", Some("Crown") => "Crown", Some("Flame") => "Flame", Some("Shield") => "Shield", Some("Beer") => "Beer", Some("Glasses") => "Glasses", Some("Gavel") => "Gavel", Some("Microscope") => "Microscope", _ => "Brain" }; view! { <Icon name=icon/> }.into_any() } }}</button><input node_ref=photo type="file" accept="image/*" hidden on:change=upload/>
        <div class="flex-1 min-w-[200px] space-y-2"><div class="flex gap-2"><input class=FIELD aria-label=move || editor.t.get("tm_name") prop:value=move || team().name placeholder=move || editor.t.get("tm_name") class:editor-invalid=move || duplicate() || team().name.trim().is_empty() on:blur=move |_|confirm_name() on:keydown=move |e: web_sys::KeyboardEvent| {if e.key()=="Enter" {confirm_name();}} autofocus=team().fields.get("draft")==Some(&serde_json::json!(true)) on:input=move |ev| editor.edit_team(&key.get_value(),|t| t.name = event_target_value(&ev))/><input class="w-20 bg-black/40 border border-white/10 rounded-lg px-3 py-2 text-sm" type="number" min="0" aria-label=move || editor.t.get("tm_players") title=move || editor.t.get("tm_players") prop:value=move || team().players.unwrap_or(0) on:input=move |ev| editor.edit_team(&key.get_value(),|t| t.players = Some(event_target_value(&ev).parse::<i64>().unwrap_or(0).max(0)))/></div><input class=FIELD aria-label=move || editor.t.get("tm_quote") prop:value=move || team().quote.unwrap_or_default() placeholder=move || editor.t.get("tm_quote") on:input=move |ev| editor.edit_team(&key.get_value(),|t| t.quote = Some(event_target_value(&ev)))/><p class="text-xs text-red-300" hidden=move || !duplicate()>{move || editor.t.get("tm_duplicate")}</p><p class="text-xs text-gray-500">{move || { let t = team(); if t.is_new != Some(true) && t.rank_history.as_ref().is_some_and(|r| !r.is_empty()) { format!("#{} · {}% · {}",t.best_rank.unwrap_or(0),t.correct_rate.unwrap_or(0.),t.rank_history.unwrap_or_default().iter().map(|r| match r { quiz_core::quiz::model::Rank::Number(n) => n.to_string(), quiz_core::quiz::model::Rank::Unknown(s) => s.clone() }).collect::<Vec<_>>().join(" · ")) } else { String::new() } }}</p></div>
        <select class="bg-black border border-white/10 rounded-lg text-sm p-2" aria-label="Icon" prop:value=move || team().icon.unwrap_or_else(|| "Brain".into()) on:change=move |ev| editor.edit_team(&key.get_value(),|t| t.icon = Some(event_target_value(&ev)))>{["Brain","Rocket","Zap","Crown","Flame","Shield","Beer","Glasses","Gavel","Microscope"].into_iter().map(|icon| view! { <option value=icon>{icon}</option> }).collect_view()}</select>
        <button type="button" class="w-8 h-8 rounded-full ring-2 ring-white/10 hover:ring-white/50" title=move ||editor.t.get("tm_color") aria-label=move ||editor.t.get("tm_color") style=move ||format!("background:{}",TEAM_HEX[team().color.as_ref().map(|c|c.index(0)).unwrap_or(0)%TEAM_HEX.len()]) on:click=move |_|editor.edit_team(&key.get_value(),|team| {let i=team.color.as_ref().map(|c|c.index(0)).unwrap_or(0);team.color=Some(TeamColor::Palette((i+1)%TEAM_HEX.len()));})/>

        <button class="p-2 rounded-lg text-gray-500 hover:text-red-300" title=move || editor.t.get("lib_delete") aria-label=move || editor.t.get("lib_delete") on:click=move |_| { let current=team(); if let Some(tab)=editor.quiz.get_untracked().drive.as_ref().and_then(|d|d["scoresTab"].as_str()) { if current.fields.get("draft") != Some(&serde_json::json!(true)) && !window().confirm_with_message(&editor.t.fmt("tm_remove_confirm", &[("name", &current.name),("tab",tab)])).unwrap_or(false) {return;} } editor.edit(|q| { let key = key.get_value(); let index = q.teams.iter().enumerate().position(|(i,t)| team_key(t,i) == key); if let Some(i) = index { q.teams.remove(i); } }); }><Icon name="trash"/></button>
        <button class="text-xs text-gray-400 hover:text-red-300" hidden=move || team().image.is_none() on:click=move |_| editor.edit_team(&key.get_value(),|t| t.image = None)>{move || editor.t.get("lib_delete")}" "{move || editor.t.get("tm_photo")}</button>
    </li> }
}

fn default_rules(language: &str) -> Vec<String> {
    let source = match language {
        "cs" => include_str!("../../i18n/cs.json"),
        "sk" => include_str!("../../i18n/sk.json"),
        _ => include_str!("../../i18n/en.json"),
    };
    let translations: BTreeMap<String, String> = serde_json::from_str(source).unwrap();
    intro::DEFAULT_RULE_KEYS
        .iter()
        .map(|key| translations.get(*key).cloned().unwrap_or_default())
        .collect()
}
#[component]
fn Intro(editor: Editor) -> impl IntoView {
    let items = move || {
        editor.quiz.with(|q| {
            q.rules
                .items
                .clone()
                .unwrap_or_else(|| default_rules(&q.settings.language))
        })
    };
    view! { <Section editor name="intro" title="ed_intro" help="ed_intro_help"><div class="grid lg:grid-cols-2 gap-6"><div class="space-y-3 rounded-xl bg-black/30 border border-white/5 p-4"><label class="flex items-center justify-between gap-4 text-gray-200 font-medium">{move || editor.t.get("ed_rules_show")}<input type="checkbox" class="editor-toggle" prop:checked=move || editor.quiz.with(|q| q.rules.enabled) on:change=move |ev| editor.edit(|q| q.rules.enabled = event_target_checked(&ev))/></label>
    <div class="space-y-3" hidden=move || !editor.quiz.with(|q| q.rules.enabled)><ol class="space-y-2"><For each={move || (0..items().len()).collect::<Vec<_>>()} key=|i| *i children=move |i| view! { <li class="flex items-center gap-2"><span class="text-xs text-gray-500">{i+1}"."</span><input class=FIELD aria-label=move || editor.t.get("ed_rule_placeholder") prop:value=move || items().get(i).cloned().unwrap_or_default() on:input=move |ev| { let mut next = items(); if let Some(rule) = next.get_mut(i) { *rule = event_target_value(&ev); editor.edit(|q| q.rules.items = Some(next)); } }/><button class="p-2 text-gray-500 hover:text-red-300" aria-label=move || editor.t.get("lib_delete") on:click=move |_| { let mut next = items(); if i < next.len() { next.remove(i); editor.edit(|q| q.rules.items = Some(next)); } }><Icon name="trash"/></button></li> }/></ol><div class="flex flex-wrap gap-2"><button class=BUTTON disabled={move || items().len() >= intro::MAX_RULES} on:click=move |_| { let mut next = items(); if next.len() < intro::MAX_RULES { next.push(String::new()); editor.edit(|q| q.rules.items = Some(next)); } }><Icon/>{move || editor.t.get("ed_rule_add")}</button><button class=BUTTON hidden=move || editor.quiz.with(|q| q.rules.items.is_none()) on:click=move |_| editor.edit(|q| q.rules.items = None)>{move || editor.t.get("ed_rules_default")}</button></div></div></div>
    <div class="space-y-3 rounded-xl bg-black/30 border border-white/5 p-4"><label class="flex items-center justify-between gap-4 text-gray-200 font-medium">{move || editor.t.get("ed_prizes_show")}<input type="checkbox" class="editor-toggle" prop:checked=move || editor.quiz.with(|q| q.prizes.enabled) on:change=move |ev| editor.edit(|q| q.prizes.enabled = event_target_checked(&ev))/></label><div class="space-y-3" hidden=move || !editor.quiz.with(|q| q.prizes.enabled)>{[("first","ed_prize_1"),("second","ed_prize_2"),("third","ed_prize_3"),("extra","ed_prize_extra")].into_iter().map(|(key,label)| view! { <label class="block space-y-1"><span class="text-xs text-gray-400">{move || editor.t.get(label)}</span><input class=FIELD prop:value=move || editor.quiz.with(|q| match key { "first" => q.prizes.first.clone(), "second" => q.prizes.second.clone(), "third" => q.prizes.third.clone(), _ => q.prizes.extra.clone() }) placeholder=move || editor.t.get("ed_prize_placeholder") on:input=move |ev| editor.edit(|q| { let field = match key { "first" => &mut q.prizes.first, "second" => &mut q.prizes.second, "third" => &mut q.prizes.third, _ => &mut q.prizes.extra }; *field = event_target_value(&ev); })/></label> }).collect_view()}</div></div></div></Section> }
}
#[component]
fn Settings(editor: Editor) -> impl IntoView {
    view! { <Section editor name="settings" title="settings_title"><div class="grid sm:grid-cols-2 gap-4">
    <label class="space-y-1"><span class="text-sm text-gray-400">{move || editor.t.get("language")}</span><select class=INPUT prop:value=move || editor.quiz.with(|q| q.settings.language.clone()) on:change=move |ev| editor.edit(|q| q.settings.language = event_target_value(&ev))><option value="en">"English"</option><option value="cs">"Čeština"</option><option value="sk">"Slovenčina"</option></select></label>
    <label class="space-y-1"><span class="text-sm text-gray-400">{move || editor.t.get("start_time")}</span><input class=INPUT type="time" prop:value=move || editor.quiz.with(|q| q.settings.start_time.clone()) on:input=move |ev| editor.edit(|q| q.settings.start_time = event_target_value(&ev))/></label>
    <label class="flex items-center justify-between gap-3"><span>{move || editor.t.get("show_clock")}</span><input class="editor-toggle" type="checkbox" prop:checked=move || editor.quiz.with(|q| q.settings.show_time) on:change=move |ev| editor.edit(|q| q.settings.show_time = event_target_checked(&ev))/></label>
    <label class="space-y-1"><span class="text-sm text-gray-400">{move || editor.t.get("set_layout")}</span><select class=INPUT prop:value=move || editor.quiz.with(|q| q.settings.welcome_layout.clone()) on:change=move |ev| editor.edit(|q| q.settings.welcome_layout = event_target_value(&ev))>{[("auto","set_layout_auto"),("cards","set_layout_cards"),("wall","set_layout_wall")].into_iter().map(|(value,label)| view! { <option value=value>{move || editor.t.get(label)}</option> }).collect_view()}</select></label>
    {[("splitDelay","anim_speed"),("cycleDuration","cycle_time"),("headerInterval","header_rot"),("introPause","set_intro_pause")].into_iter().map(|(key,label)| view! { <label class="space-y-1"><span class="text-sm text-gray-400">{move || editor.t.get(label)}" (s)"</span><input class=INPUT type="number" min="0" max="60" step="0.1" prop:value=move || editor.quiz.with(|q| (match key { "splitDelay" => q.settings.split_delay, "cycleDuration" => q.settings.cycle_duration, "headerInterval" => q.settings.header_interval, _ => q.settings.intro_pause }) as f64/1000.) on:input=move |ev| { if let Ok(n) = event_target_value(&ev).parse::<f64>() { let n = (n.clamp(0.,60.)*1000.).round() as u64; editor.edit(|q| match key { "splitDelay" => q.settings.split_delay = n, "cycleDuration" => q.settings.cycle_duration = n, "headerInterval" => q.settings.header_interval = n, _ => q.settings.intro_pause = n }); } }/></label> }).collect_view()}
    </div><p class="text-xs text-gray-500">{move || editor.t.get("set_layout_help")}</p></Section> }
}
async fn apply_theme(editor: Editor, pack: pack::ThemePack) -> Result<String, String> {
    let (theme, assets) = pack::theme_for_quiz_media(&pack);
    let data = editor.data.get_value();
    let media: Vec<_> = assets
        .into_iter()
        .map(|a| Media {
            content_type: quiz_store::package::media_type(&a.name)
                .unwrap_or("application/octet-stream")
                .into(),
            name: a.name,
            bytes: a.bytes,
            ..Default::default()
        })
        .collect();
    data.store
        .put_media(&data.id, &media)
        .await
        .map_err(|e| e.to_string())?;
    editor.edit(|q| {
        q.fields
            .insert("customTheme".into(), serde_json::to_value(&theme).unwrap());
        q.settings.fields.insert("theme".into(), json!("custom"));
    });
    editor.reload_media().await?;
    let mut message = editor.t.fmt(
        "thm_applied",
        &[("name", builtin::theme_name(&theme, editor.t.lang().code()))],
    );
    if !pack.problems.is_empty() {
        message.push('\n');
        message.push_str(&editor.t.get("thm_problems"));
    }
    for problem in pack.problems {
        message.push('\n');
        message.push_str(&editor.t.fmt(
            &format!("thm_p_{}", problem.code),
            &[("detail", &problem.detail)],
        ));
    }
    Ok(message)
}
fn confirm_theme(editor: Editor) -> bool {
    editor.quiz.with_untracked(|q| {
        q.fields
            .get("customTheme")
            .and_then(|v| serde_json::from_value::<Theme>(v.clone()).ok())
            .is_none_or(|theme| {
                window()
                    .confirm_with_message(&editor.t.fmt(
                        "thm_new_confirm",
                        &[("name", builtin::theme_name(&theme, editor.t.lang().code()))],
                    ))
                    .unwrap_or(false)
            })
    })
}
#[component]
fn ThemeCard(editor: Editor, theme: Theme, theme_key: String) -> impl IntoView {
    let key = StoredValue::new(theme_key);
    let theme = StoredValue::new(theme);
    let choose = move || {
        editor.edit(|q| {
            q.settings
                .fields
                .insert("theme".into(), json!(key.get_value()));
        })
    };
    view! { <div role="button" tabindex="0" class=move || if editor.quiz.with(|q| q.settings.fields.get("theme").and_then(Value::as_str) == Some(key.get_value().as_str())) { "text-left rounded-xl p-2 border border-yellow-500 bg-yellow-500/10 cursor-pointer" } else { "text-left rounded-xl p-2 border border-white/10 hover:border-white/30 bg-black/20 cursor-pointer" }
        aria-pressed=move || editor.quiz.with(|q| q.settings.fields.get("theme").and_then(Value::as_str) == Some(key.get_value().as_str()))
        on:click=move |_| choose()
        on:keydown=move |ev| { if matches!(ev.key().as_str(), "Enter" | " ") { ev.prevent_default(); choose(); } }>
        <preview::Preview editor theme=theme.get_value()/>
        <span class="block px-1 pt-2 font-bold text-white">{move || theme.with_value(|theme| builtin::theme_name(theme, editor.t.lang().code()).to_owned())}
            <span class="ml-2 text-[10px] uppercase tracking-wider px-1.5 py-0.5 rounded bg-white/10 text-gray-300" hidden=key.get_value() != "custom">{move || editor.t.get("thm_own")}</span>
        </span>
        <span class="block px-1 text-xs text-gray-500 truncate" hidden=theme.with_value(|t| t.author.is_empty())>{move || theme.with_value(|theme| editor.t.fmt("thm_by", &[("author", &theme.author)]))}</span>
    </div> }
}
#[component]
fn Themes(editor: Editor) -> impl IntoView {
    let input = NodeRef::<leptos::html::Input>::new();
    let refresh = RwSignal::new(0u32);
    let library = LocalResource::new(move || {
        refresh.get();
        let store = editor.data.get_value().store;
        async move { store.list_themes().await.map_err(|e| e.to_string()) }
    });
    let import = move |_| {
        if let Some(file) = input.get().and_then(|i| {
            let file = files(&i).into_iter().next();
            i.set_value("");
            file
        }) {
            if !confirm_theme(editor) {
                return;
            }
            editor.busy.set(true);
            spawn_local(async move {
                let result = async {
                    let bytes = bytes(&file).await?;
                    let pack = pack::read_theme_pack(&bytes).map_err(|e| match e {
                        pack::ThemePackError::NoTheme => editor.t.get("thm_err_no_theme"),
                        pack::ThemePackError::BadJson => editor.t.get("thm_err_bad_json"),
                        error => error.to_string(),
                    })?;
                    let name = builtin::theme_name(&pack.theme, editor.t.lang().code()).to_owned();
                    let message = apply_theme(editor, pack).await?;
                    Ok(message.replacen(
                        &editor.t.fmt("thm_applied", &[("name", &name)]),
                        &editor.t.fmt("thm_imported", &[("name", &name)]),
                        1,
                    ))
                }
                .await;
                editor.message(result);
                editor.busy.try_set(false);
            });
        }
    };
    let export = move |_| {
        let q = editor.quiz.get_untracked();
        let theme = selected_theme(&q);
        let result = (|| {
            let assets = editor.media.with_untracked(|media| {
                quiz_core::theme::format::theme_files(&theme)
                    .into_iter()
                    .filter_map(|name| {
                        media
                            .iter()
                            .find(|m| media_key(&m.name) == media_key(&name))
                            .map(|m| pack::PackAsset {
                                name,
                                bytes: m.bytes.clone(),
                            })
                    })
                    .collect::<Vec<_>>()
            });
            let bytes = pack::export_theme_pack(&theme, &assets).map_err(|e| e.to_string())?;
            download(&bytes, &pack::theme_pack_name(&theme), "application/zip")?;
            Ok(editor.t.get("ed_saved"))
        })();
        editor.message(result);
    };
    view! { <Section editor name="theme" title="ed_theme" help="ed_theme_help"><div class="grid gap-4 sm:grid-cols-3">{builtin::themes().iter().map(|theme| view! { <ThemeCard editor theme=theme.clone() theme_key=theme.id.clone()/> }).collect_view()}{move || editor.quiz.with(|q| q.fields.get("customTheme").and_then(|v| serde_json::from_value::<Theme>(v.clone()).ok())).map(|theme| view! { <ThemeCard editor theme theme_key="custom".into()/> })}</div>
        <div class="flex flex-wrap gap-2"><a class=BUTTON hidden=move || !editor.quiz.with(|q| q.fields.contains_key("customTheme")) href=editor.link(|id| Route::Theme { id,fresh:false })>{move || editor.t.get("thm_edit")}</a><button class=BUTTON on:click=move |_| { if confirm_theme(editor) { crate::router::navigate(Route::Theme { id:editor.id(),fresh:true }); } }>{move || editor.t.fmt("thm_new", &[("name",builtin::theme_name(&selected_theme(&editor.quiz.get()),editor.t.lang().code()))])}</button><button class=BUTTON disabled=move || editor.busy.get() on:click=move |_| { if let Some(i) = input.get() { i.click(); } }><Icon name="upload"/>{move || editor.t.get("thm_import")}</button><input node_ref=input type="file" accept=".zip,.json,application/zip,application/json" hidden on:change=import/><button class=BUTTON disabled=move || editor.busy.get() on:click=export>{move || editor.t.fmt("thm_export", &[("name",builtin::theme_name(&selected_theme(&editor.quiz.get()),editor.t.lang().code()))])}</button><a class=BUTTON href="#/themes">{move || editor.t.get("thm_browse")}</a></div>
        <Suspense>{move || library.get().map(|result| match result { Err(e) => view! { <p class="text-red-300 text-sm">{e}</p> }.into_any(), Ok(entries) => { let empty = entries.is_empty(); view! { <div class="space-y-2" hidden=empty><h3 class="text-sm font-bold text-gray-300">{editor.t.get("thm_library")}</h3><p class="text-xs text-gray-500">{editor.t.get("thm_library_help")}</p><div class="flex flex-wrap gap-2">{entries.into_iter().map(|entry| { let theme_name = builtin::theme_name(&entry.theme,editor.t.lang().code()).to_owned(); let delete_name = theme_name.clone(); let id = entry.id.clone(); view! { <div class="flex rounded-xl border border-white/10 bg-black/20"><button class=BUTTON disabled=move || editor.busy.get() on:click=move |_| { if !confirm_theme(editor) { return; } let pack = pack::ThemePack { theme:entry.theme.clone(), assets:entry.files.iter().map(|m| pack::PackAsset { name:m.name.clone(),bytes:m.bytes.clone() }).collect(),problems:vec![] }; editor.busy.set(true); spawn_local(async move { editor.message(apply_theme(editor,pack).await); editor.busy.try_set(false); }); }>{theme_name}</button><button class="p-2 text-gray-500 hover:text-red-300" aria-label=move || editor.t.get("thm_lib_delete") on:click=move |_| { if !window().confirm_with_message(&editor.t.fmt("thm_lib_delete_confirm", &[("name",&delete_name)])).unwrap_or(false) { return; } let id = id.clone(); spawn_local(async move { let data = editor.data.get_value(); match data.store.delete_theme(&id).await { Ok(()) => refresh.update(|n| *n += 1), Err(e) => editor.message(Err(e.to_string())) } }); }><Icon name="trash"/></button></div> } }).collect_view()}</div></div> }.into_any() } })}</Suspense>
    </Section> }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn editor_patch_preserves_concurrent_scores_and_settings() {
        let before = Quiz::new("test", 0);
        let mut after = before.clone();
        after.title = "Changed".into();
        after
            .settings
            .fields
            .insert("theme".into(), json!("halloween"));
        let mut stored = before.clone();
        stored.settings.start_time = "21:00".into();
        stored
            .manual_scores
            .entry("team".into())
            .or_default()
            .insert("round".into(), 7.);
        apply_patch(&mut stored, &changes(&before, &after));
        assert_eq!(stored.title, "Changed");
        assert_eq!(stored.settings.start_time, "21:00");
        assert_eq!(stored.settings.fields["theme"], "halloween");
        assert_eq!(stored.manual_scores["team"]["round"], 7.);
    }
    #[test]
    fn theme_patch_keeps_external_fields_and_only_changes_selected_settings() {
        let before = Quiz::new("quiz", 0);
        let mut edited = before.clone();
        edited
            .fields
            .insert("customTheme".into(), json!({"id": "own"}));
        edited
            .settings
            .fields
            .insert("theme".into(), json!("custom"));
        let mut stored = before.clone();
        stored.drive = Some(json!({"folderId": "external"}));
        stored.settings.language = "cs".into();
        apply_patch(&mut stored, &changes(&before, &edited));
        assert_eq!(stored.fields["customTheme"]["id"], "own");
        assert_eq!(stored.drive.unwrap()["folderId"], "external");
        assert_eq!(stored.settings.fields["theme"], "custom");
        assert_eq!(stored.settings.language, "cs");
    }
    #[test]
    fn new_edits_override_failed_patch_without_losing_other_fields() {
        let before = Quiz::default();
        let mut first = before.clone();
        first.title = "One".into();
        first.questions_csv = "CSV".into();
        let mut second = first.clone();
        second.title = "Two".into();
        let mut pending = changes(&first, &second);
        for (k, v) in changes(&before, &first) {
            pending.entry(k).or_insert(v);
        }
        let mut saved = before;
        apply_patch(&mut saved, &pending);
        assert_eq!(saved.title, "Two");
        assert_eq!(saved.questions_csv, "CSV");
    }
    #[test]
    fn rules_follow_show_language_and_keep_all_defaults() {
        assert_eq!(default_rules("cs").len(), 4);
        assert_ne!(default_rules("cs"), default_rules("en"));
        assert_eq!(default_rules("unknown"), default_rules("en"));
    }
}
