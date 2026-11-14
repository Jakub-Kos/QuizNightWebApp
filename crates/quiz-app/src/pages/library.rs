//! Local quiz library. No Drive integration.
use crate::{
    router::{Route, navigate},
    store,
};
use leptos::prelude::*;
use quiz_core::quiz::{model::Quiz, parse::parse_questions_csv};
use quiz_store::{QuizStore, package};
use wasm_bindgen::JsCast;

const BUTTON: &str = "inline-flex items-center justify-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm";
const PRIMARY: &str = "inline-flex items-center justify-center gap-2 px-4 py-2 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold text-sm";

#[component]
fn Icon() -> impl IntoView {
    view! { <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M12 5v14M5 12h14"/></svg> }
}
fn safe_filename(title: &str) -> String {
    let s: String = title
        .chars()
        .filter(|c| !"\\/:*?\"<>|".contains(*c))
        .collect();
    if s.trim().is_empty() {
        "quiz".into()
    } else {
        s.trim().into()
    }
}
fn download(bytes: &[u8], title: &str) -> Result<(), String> {
    let blob = store::blob(bytes, "application/zip")?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(|e| format!("{e:?}"))?;
    let a = document()
        .create_element("a")
        .map_err(|e| format!("{e:?}"))?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|e| format!("{e:?}"))?;
    a.set_href(&url);
    a.set_download(&format!("{}.zip", safe_filename(title)));
    a.click();
    gloo_timers::callback::Timeout::new(1000, move || {
        let _ = web_sys::Url::revoke_object_url(&url);
    })
    .forget();
    Ok(())
}
async fn import_files(files: Vec<web_sys::File>, store: store::Store) -> Result<String, String> {
    let mut entries = Vec::new();
    for file in files {
        let bytes = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
            .await
            .map_err(|e| format!("{e:?}"))?;
        let bytes = js_sys::Uint8Array::new(&bytes).to_vec();
        if file.name().to_ascii_lowercase().ends_with(".zip") {
            let imported = package::read_zip(&bytes, store::new_id(), store::now_ms())
                .map_err(|e| e.to_string())?;
            entries.push((
                "quiz.json".into(),
                serde_json::to_vec(&imported.quiz).map_err(|e| e.to_string())?,
            ));
            entries.push((
                "questions.csv".into(),
                imported.quiz.questions_csv.into_bytes(),
            ));
            entries.extend(
                imported
                    .media
                    .into_iter()
                    .map(|m| (format!("media/{}", m.name), m.bytes)),
            );
            continue;
        }
        let path = js_sys::Reflect::get(&file, &"relativePath".into())
            .ok()
            .filter(|v| !v.is_undefined())
            .map(Ok)
            .unwrap_or_else(|| js_sys::Reflect::get(&file, &"webkitRelativePath".into()))
            .ok()
            .and_then(|v| v.as_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| file.name());
        entries.push((path, bytes));
    }
    let imported = package::read_entries(entries, store::new_id(), store::now_ms())
        .map_err(|e| e.to_string())?;
    package::save_import(&store, &imported)
        .await
        .map_err(|e| e.to_string())?;
    Ok(imported.quiz.id)
}

#[component]
pub fn Page() -> impl IntoView {
    let store = store::use_store();
    let persistent = store.is_persistent();
    let revision = store.revision();
    let store = StoredValue::new_local(store);
    let lang = expect_context::<RwSignal<crate::i18n::Lang>>();
    let t = crate::i18n::use_t();
    let dismissed = RwSignal::new(
        window()
            .local_storage()
            .ok()
            .flatten()
            .and_then(|s| s.get_item("quiznight.welcomeDismissed").ok().flatten())
            .as_deref()
            == Some("1"),
    );
    let error = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let launching = RwSignal::new(None::<String>);
    let quizzes = LocalResource::new(move || {
        revision.get();
        let store = store.get_value();
        async move {
            let mut list = store.list_quizzes().await.map_err(|e| e.to_string())?;
            list.sort_by_key(|q| (!q.builtin, std::cmp::Reverse(q.updated_at)));
            Ok::<_, String>(list)
        }
    });
    let create = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(String::new());
        let store = store.get_value();
        let mut quiz = Quiz::new(store::new_id(), store::now_ms());
        quiz.title = t.get("ed_untitled");
        quiz.settings.language = lang.get_untracked().code().into();
        leptos::task::spawn_local(async move {
            match store.put_quiz(&quiz).await {
                Ok(_) => navigate(Route::Edit(quiz.id)),
                Err(e) => error.set(e.to_string()),
            }
            busy.set(false);
        });
    };
    let import = Callback::new(move |files: Vec<web_sys::File>| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(String::new());
        let store = store.get_value();
        leptos::task::spawn_local(async move {
            match import_files(files, store).await {
                Ok(id) => navigate(Route::Edit(id)),
                Err(e) => error.set(e),
            }
            busy.set(false);
        });
    });
    view! {
        <main class="min-h-screen bg-[#050505] text-white font-sans">
        <div class="max-w-5xl mx-auto px-6 py-12 space-y-10 library-enter">
            <header class="flex flex-wrap items-end justify-between gap-6">
                <div><h1 class="font-['League_Spartan'] text-5xl font-black uppercase tracking-widest text-yellow-400">"Quiz Night"</h1>
                    <p class="text-gray-400 mt-3 max-w-xl">{move || t.get("lib_subtitle")}</p></div>
                <nav class="flex flex-wrap items-center gap-3">
                    <a href="#/themes" class=BUTTON>{move || t.get("ws_title")}</a>
                    <a href="#/help" class=BUTTON>{move || t.get("lib_help")}</a>
                    {[(crate::i18n::Lang::Cs, "Čeština"), (crate::i18n::Lang::Sk, "Slovenčina"), (crate::i18n::Lang::En, "English")].into_iter().map(|(code,name)| view! {
                        <button class=move || if lang.get() == code { PRIMARY } else { BUTTON } on:click=move |_| {
                            lang.set(code);
                            if let Ok(Some(storage)) = window().local_storage() { let _ = storage.set_item("quiznight.lang", code.code()); }
                        }>{name}</button>
                    }).collect_view()}
                </nav>
            </header>
            <Show when=move || !persistent><p role="status" class="p-4 rounded-xl bg-yellow-500/10 border border-yellow-500/30 text-yellow-200">"Temporary storage: changes will be lost when this window closes. Export your quizzes to keep them."</p></Show>
            <Show when=move || !error.get().is_empty()><p role="alert" class="p-4 rounded-xl bg-red-500/10 border border-red-500/30 text-red-200">{move || error.get()}</p></Show>
            <Show when=move || !dismissed.get() && quizzes.get().is_some_and(|r| r.is_ok_and(|list| list.iter().all(|q| q.builtin)))>
                <section class="rounded-2xl bg-gradient-to-br from-yellow-500/10 to-transparent border border-yellow-500/30 p-6 space-y-5">
                    <button class=BUTTON on:click=move |_| { dismissed.set(true); if let Ok(Some(s)) = window().local_storage() { let _ = s.set_item("quiznight.welcomeDismissed", "1"); } }>{move || t.get("chk_close")}</button>
                    <h2 class="text-2xl font-bold">{move || t.get("wl_title")}</h2><p>{move || t.get("wl_text")}</p>
                    <button class=PRIMARY on:click=move |_| launching.set(Some("demo".into()))>{move || t.get("wl_demo_button")}</button>
                    <button class=PRIMARY on:click=move |_| crate::tour::start_tour("demo")>{move || t.get("tr_start")}</button>
                    <a class=BUTTON href="#/help/start">{move || t.get("wl_help")}</a>
                </section>
            </Show>
            <section class="grid md:grid-cols-[1fr_2fr] gap-4">
                <button disabled=move || busy.get() on:click=create class="flex items-center justify-center gap-3 p-8 rounded-2xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold text-lg"><Icon/>{move || t.get("lib_new")}</button>
                <label class="rounded-2xl border-2 border-dashed border-white/15 flex flex-col items-center justify-center gap-3 text-gray-300 p-10 cursor-pointer"
                    on:dragover=move |e: web_sys::DragEvent| e.prevent_default()
                    on:drop=move |e: web_sys::DragEvent| { e.prevent_default(); if let Some(data) = e.data_transfer() {
                        match dropped_files(data) {
                            Ok(promise) => leptos::task::spawn_local(async move {
                                match wasm_bindgen_futures::JsFuture::from(promise).await {
                                    Ok(files) => import.run(js_sys::Array::from(&files).iter().filter_map(|f| f.dyn_into().ok()).collect()),
                                    Err(e) => error.set(format!("{e:?}")),
                                }
                            }),
                            Err(e) => error.set(e),
                        }
                    } }>
                    {move || if busy.get() { t.get("lib_importing") } else { t.get("lib_import_hint") }}
                    <input type="file" multiple accept=".zip,.csv,.json,image/*,audio/*,video/*,.woff2,.woff,.ttf,.otf" disabled=move || busy.get()
                        on:change=move |e| { let input = event_target::<web_sys::HtmlInputElement>(&e); if let Some(files) = input.files() { import.run((0..files.length()).filter_map(|i| files.item(i)).collect()); } input.set_value(""); }/>
                </label>
            </section>
            <button class=BUTTON disabled=move || busy.get() on:click=move |_| {
                busy.set(true); let store = store.get_value();
                leptos::task::spawn_local(async move {
                    let result = async {
                        let quiz = store.get_quiz("demo").await.map_err(|e| e.to_string())?.ok_or("Demo not available")?;
                        download(&store::export_quiz(&store, &quiz).await?, &quiz.title)
                    }.await;
                    if let Err(e) = result { error.set(e); } busy.set(false);
                });
            }>{move || t.get("lib_template")}</button>
            <section class="space-y-4">
                <h2 class="font-['League_Spartan'] text-2xl font-bold uppercase tracking-widest text-gray-400">{move || t.get("lib_title")}</h2>
                {move || quizzes.get().map(|result| match result {
                    Err(e) => view! { <p role="alert">{e.clone()}</p> }.into_any(),
                    Ok(list) => view! { <div class="grid md:grid-cols-2 gap-4">
                        {list.iter().cloned().map(|quiz| view! { <QuizCard quiz=quiz busy=busy error=error launching=launching/> }).collect_view()}
                    </div> }.into_any(),
                }).unwrap_or_else(|| view! { <p>{t.get("loading")}</p> }.into_any())}
            </section>
            {move || launching.get().map(|id| view! { <Launcher id=id close=Callback::new(move |_| launching.set(None))/> })}
        </div></main>
    }
}
#[component]
fn QuizCard(
    quiz: Quiz,
    busy: RwSignal<bool>,
    error: RwSignal<String>,
    launching: RwSignal<Option<String>>,
) -> impl IntoView {
    let t = crate::i18n::use_t();
    let store = StoredValue::new_local(store::use_store());
    let quiz = StoredValue::new(quiz);
    let confirm = RwSignal::new(false);
    let rounds = parse_questions_csv(&quiz.get_value().questions_csv);
    let summary = format!(
        "{} {} · {} {} · {} {}",
        rounds.len(),
        t.get("lib_rounds"),
        rounds.iter().map(|r| r.questions.len()).sum::<usize>(),
        t.get("lib_questions"),
        quiz.get_value().teams.len(),
        t.get("lib_teams")
    );
    let action = Callback::new(move |kind: &'static str| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(String::new());
        let store = store.get_value();
        let quiz = quiz.get_value();
        leptos::task::spawn_local(async move {
            let result: Result<(), String> = async {
                if kind == "delete" {
                    return store.delete_quiz(&quiz.id).await.map_err(|e| e.to_string());
                }
                let bytes = store::export_quiz(&store, &quiz).await?;
                if kind == "export" {
                    return download(&bytes, &quiz.title);
                }
                let mut imported = package::read_zip(&bytes, store::new_id(), store::now_ms())
                    .map_err(|e| e.to_string())?;
                imported.quiz.title = format!("{} ({})", quiz.title, t.get("lib_copy_suffix"));
                package::save_import(&store, &imported)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(())
            }
            .await;
            if let Err(e) = result {
                error.set(e);
            }
            confirm.set(false);
            busy.set(false);
        });
    });
    view! { <article class="rounded-2xl bg-white/[0.04] border border-white/10 p-6 flex flex-col gap-5">
        <div><h3 class="text-xl font-bold truncate">{if quiz.get_value().title.is_empty() { t.get("ed_untitled") } else { quiz.get_value().title }}</h3>
            <p class="text-sm text-gray-400 mt-1">{summary}</p>
            {(!quiz.get_value().builtin).then(|| view! { <p class="text-xs text-gray-500 mt-1">{format!("{} {}", t.get("lib_updated"), js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(quiz.get_value().updated_at as f64)).to_locale_string("default", &js_sys::Object::new()).as_string().unwrap_or_default())}</p> })}
            {quiz.get_value().builtin.then(|| view! { <span class="text-xs text-blue-300">{t.get("lib_demo_badge")}</span> })}
        </div>
        <div class="flex flex-wrap gap-2">
            <button class=PRIMARY on:click=move |_| launching.set(Some(quiz.get_value().id))>{move || t.get("lib_open")}</button>
            {[("edit", "lib_edit"), ("questions", "ed_questions"), ("theme", "ed_theme"), ("check", "chk_open"), ("map", "mp_title"), ("attendance", "att_title"), ("scores", "sc_title")].into_iter().map(|(suffix,key)| {
                let hash = format!("{}/{suffix}", Route::Show(quiz.get_value().id).hash());
                view! { <a class=BUTTON href=hash>{move || t.get(key)}</a> }
            }).collect_view()}
            <button class=BUTTON disabled=move || busy.get() on:click=move |_| action.run("copy")>{move || t.get("lib_copy")}</button>
            <button class=BUTTON disabled=move || busy.get() on:click=move |_| action.run("export")>{move || t.get("lib_export")}</button>
            <Show when=move || !quiz.get_value().builtin>
                <button class=BUTTON disabled=move || busy.get() on:click=move |_| confirm.set(true)>{move || t.get("lib_delete")}</button>
            </Show>
        </div>
        <Show when=move || confirm.get()><div class="flex flex-wrap items-center gap-2">
            <span class="text-red-300 text-sm">{move || t.get("lib_delete_confirm")}</span>
            <button class=BUTTON on:click=move |_| confirm.set(false)>{move || t.get("lib_cancel")}</button>
            <button class="px-4 py-2 rounded-xl bg-red-600 hover:bg-red-500 text-sm font-bold" disabled=move || busy.get() on:click=move |_| action.run("delete")>{move || t.get("lib_delete")}</button>
        </div></Show>
    </article> }
}
#[component]
fn Launcher(id: String, close: Callback<()>) -> impl IntoView {
    let id = StoredValue::new(id);
    let key = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
        move |e: web_sys::KeyboardEvent| {
            if e.key() == "Escape" {
                close.run(());
            }
        },
    );
    let _ = window().add_event_listener_with_callback("keydown", key.as_ref().unchecked_ref());
    let key = StoredValue::new_local(Some(key));
    on_cleanup(move || {
        if let Some(Some(key)) = key.try_update_value(Option::take) {
            let _ = window()
                .remove_event_listener_with_callback("keydown", key.as_ref().unchecked_ref());
        }
    });
    let t = crate::i18n::use_t();
    let tv = RwSignal::new(None::<bool>);
    let host = RwSignal::new(None::<bool>);
    let open = move |presenter| {
        let route = if presenter {
            Route::Presenter(id.get_value())
        } else {
            Route::Show(id.get_value())
        };
        let location = window().location();
        let url = format!(
            "{}{}{}",
            location.origin().unwrap_or_default(),
            location.pathname().unwrap_or_default(),
            route.hash()
        );
        if !presenter {
            leptos::task::spawn_local(async move {
                let open = js_sys::Function::new_with_args(
                    "url",
                    r#"
                    return (async () => {
                        let features = "popup,width=1280,height=720";
                        try {
                            if (screen.isExtended && window.getScreenDetails) {
                                const details = await window.getScreenDetails();
                                const other = details.screens.find(s => s !== details.currentScreen);
                                if (other) features = `popup,left=${other.availLeft},top=${other.availTop},width=${other.availWidth},height=${other.availHeight}`;
                            }
                        } catch {}
                        return !!window.open(url, "QuizNightTV", features);
                    })();
                "#,
                );
                let result = open.call1(&wasm_bindgen::JsValue::NULL, &url.into()).ok();
                let opened = if let Some(result) = result {
                    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&result))
                        .await
                        .ok()
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                } else {
                    false
                };
                tv.set(Some(opened));
            });
            return;
        }
        let opened = window()
            .open_with_url_and_target_and_features(
                &url,
                if presenter {
                    "PresenterWindow"
                } else {
                    "QuizNightTV"
                },
                if presenter {
                    "width=1200,height=800"
                } else {
                    "popup,width=1280,height=720"
                },
            )
            .ok()
            .flatten()
            .is_some();
        if presenter {
            host.set(Some(opened));
        } else {
            tv.set(Some(opened));
        }
    };
    view! { <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-sm flex items-center justify-center p-4" on:click=move |_| close.run(())>
        <section role="dialog" aria-modal="true" aria-label=t.get("sl_title") class="w-full max-w-xl max-h-full overflow-y-auto rounded-3xl bg-[#0f0f0f] border border-white/10 shadow-2xl p-8 space-y-5" on:click=|e| e.stop_propagation()>
            <button class=BUTTON on:click=move |_| close.run(())>{move || t.get("chk_close")}</button>
            <h2 class="text-3xl font-black uppercase text-yellow-400">{move || t.get("sl_title")}</h2>
            <p class="text-gray-300">{move || t.get("sl_intro")}</p>
            <h3 class="font-bold text-lg">{move || t.get("sl_tv_title")}</h3>
            <p>{move || t.get(if tv.get() == Some(false) { "sl_tv_blocked" } else if tv.get() == Some(true) { "sl_tv_opened" } else { "sl_tv_text" })}</p>
            <button class=PRIMARY on:click=move |_| open(false)>{move || t.get("sl_tv_open")}</button>
            <h3 class="font-bold text-lg">{move || t.get("sl_presenter_title")}</h3><p>{move || t.get("sl_presenter_text")}</p>
            <button class=PRIMARY on:click=move |_| open(true)>{move || t.get("sl_presenter_open")}</button>
            <Show when=move || host.get() == Some(false)><p role="alert">{move || t.get("sl_tv_blocked")}</p></Show>
            <Show when=move || id.get_value() == "demo"><button class=BUTTON on:click=move |_| crate::tour::start_tour("demo")>{move || t.get("tr_start")}</button></Show>
            <a class=BUTTON href=Route::Show(id.get_value()).hash()>{move || t.get("sl_one_screen")}</a>
        </section>
    </div> }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filenames() {
        assert_eq!(safe_filename(" a/b:quiz? "), "abquiz");
        assert_eq!(safe_filename(" /:*? "), "quiz");
    }
}

fn dropped_files(data: web_sys::DataTransfer) -> Result<js_sys::Promise, String> {
    let collect = js_sys::Function::new_with_args(
        "data",
        r#"
        const roots = Array.from(data.items || []).map(i => i.webkitGetAsEntry?.()).filter(Boolean);
        const fallback = Array.from(data.files);
        return (async () => {
            if (!roots.length) return fallback;
            const files = [];
            async function walk(entry, path) {
                if (entry.isFile) {
                    const f = await new Promise((ok, fail) => entry.file(ok, fail));
                    f.relativePath = path + f.name; files.push(f);
                } else if (entry.isDirectory) {
                    const reader = entry.createReader();
                    for (;;) {
                        const batch = await new Promise((ok, fail) => reader.readEntries(ok, fail));
                        if (!batch.length) break;
                        for (const child of batch) await walk(child, path + entry.name + "/");
                    }
                }
            }
            for (const entry of roots) await walk(entry, "");
            return files;
        })();
    "#,
    );
    let result = collect
        .call1(&wasm_bindgen::JsValue::NULL, &data)
        .map_err(|e| format!("{e:?}"))?;
    Ok(js_sys::Promise::resolve(&result))
}
