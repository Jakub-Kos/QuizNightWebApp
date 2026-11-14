//! Reviewed, data-only community themes, fetched relative to this build.
use super::theme_editor::{BUTTON, Preview, download, media_from_assets, save_quiz_theme};
use crate::{i18n::use_t, router::Route, store};
use leptos::prelude::*;
use quiz_core::theme::{
    builtin::theme_name,
    format::theme_files,
    pack::{PackAsset, ThemePack, theme_for_quiz_media},
    workshop::{IndexEntry, parse_index, workshop_file_url},
};
use quiz_store::{LibraryTheme, MediaStore, QuizStore, ThemeStore};

async fn fetch(url: &str) -> Result<gloo_net::http::Response, String> {
    let response = gloo_net::http::Request::get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(format!("HTTP {}", response.status()));
    }
    Ok(response)
}
async fn fetch_pack(entry: &IndexEntry) -> Result<ThemePack, String> {
    let mut assets = Vec::new();
    for name in theme_files(&entry.theme) {
        let bytes = fetch(&workshop_file_url("", entry, &name))
            .await?
            .binary()
            .await
            .map_err(|e| e.to_string())?;
        assets.push(PackAsset { name, bytes });
    }
    Ok(ThemePack {
        theme: entry.theme.clone(),
        assets,
        problems: Vec::new(),
    })
}
fn matches(entry: &IndexEntry, query: &str, tag: &str, lang: &str) -> bool {
    let theme = &entry.theme;
    let text = format!(
        "{} {} {} {}",
        theme_name(theme, lang),
        theme.author,
        theme
            .description
            .values()
            .cloned()
            .collect::<Vec<_>>()
            .join(" "),
        theme.tags.join(" ")
    )
    .to_lowercase();
    (tag.is_empty() || theme.tags.iter().any(|t| t == tag))
        && text.contains(&query.trim().to_lowercase())
}
fn description(entry: &IndexEntry, lang: &str) -> String {
    entry
        .theme
        .description
        .get(lang)
        .or_else(|| entry.theme.description.get("en"))
        .cloned()
        .unwrap_or_default()
}
#[component]
pub fn Page() -> impl IntoView {
    let t = use_t();
    let store = StoredValue::new_local(store::use_store());
    let query = RwSignal::new(String::new());
    let tag = RwSignal::new(String::new());
    let open = RwSignal::new(None::<IndexEntry>);
    let loaded = LocalResource::new(move || async move {
        let raw = fetch("workshop/index.json")
            .await?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| e.to_string())?;
        let entries = parse_index(&raw);
        let demo = store
            .get_value()
            .get_quiz("demo")
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| t.get("ed_not_found"))?;
        Ok::<_, String>((entries, demo))
    });
    view! {
        <div class="min-h-screen bg-[#050505] text-white font-sans">
            <header class="sticky top-0 z-30 bg-[#050505]/95 backdrop-blur border-b border-white/10">
                <div class="max-w-6xl mx-auto px-6 py-4 flex flex-wrap items-center gap-4">
                    <a href="#/" class="text-gray-400 hover:text-white">{t.get("ws_back")}</a>
                    <h1 class="text-2xl font-bold uppercase tracking-widest mr-auto">{t.get("ws_title")}</h1>
                    <input type="search" class="w-full sm:w-72 bg-black/40 border border-white/10 rounded-xl px-3 py-2" placeholder=t.get("ws_search") aria-label=t.get("ws_search") prop:value=move || query.get() on:input=move |ev| query.set(event_target_value(&ev))/>
                </div>
            </header>
            <main class="max-w-6xl mx-auto px-6 py-6 space-y-6">
                <p class="text-gray-400 max-w-3xl">{t.get("ws_intro")}</p>
                {move || loaded.get().map(|result| match result {
                    Err(_) => view! { <p role="alert" class="text-red-400">{t.get("ws_error")}</p> }.into_any(),
                    Ok((entries, demo)) => {
                        let mut tags = entries.iter().flat_map(|e| e.theme.tags.clone()).collect::<Vec<_>>();
                        tags.sort(); tags.dedup(); tags.insert(0, String::new());
                        let entries = StoredValue::new(entries.clone());
                        let demo = StoredValue::new(demo.clone());
                        view! {
                            <div class="flex flex-wrap gap-2">{tags.into_iter().map(|value| {
                                let selected = value.clone();
                                let pressed = value.clone(); let label = if value.is_empty() { t.get("ws_all") } else { format!("#{value}") };
                                view! { <button type="button" class=move || if tag.get() == selected { "px-3 py-1.5 rounded-full text-sm bg-yellow-500 text-black font-bold" } else { "px-3 py-1.5 rounded-full text-sm bg-white/5 text-gray-300 hover:bg-white/10" } aria-pressed=move || tag.get() == pressed on:click=move |_| tag.set(value.clone())>{label}</button> }
                            }).collect_view()}</div>
                            <div class="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
                                {move || {
                                    let shown = entries.get_value().into_iter().filter(|e| matches(e, &query.get(), &tag.get(), t.lang().code())).collect::<Vec<_>>();
                                    if shown.is_empty() { return view! { <p class="text-gray-500">{t.get("ws_none")}</p> }.into_any(); }
                                    shown.into_iter().map(|entry| view! { <Card entry=entry demo=demo.get_value() open=open/> }).collect_view().into_any()
                                }}
                            </div>
                            {move || open.get().map(|entry| view! { <Detail entry=entry demo=demo.get_value() close=Callback::new(move |_| open.set(None))/> })}
                        }.into_any()
                    }
                }).unwrap_or_else(|| view! { <p role="status">{t.get("loading")}</p> }.into_any())}
                <section class="rounded-2xl bg-white/[0.04] border border-white/10 p-5">
                    <h2 class="font-bold">{t.get("ws_share_title")}</h2>
                    <p class="text-sm text-gray-400">{t.get("ws_share_text")}</p>
                </section>
            </main>
        </div>
    }
}
#[component]
fn Card(
    entry: IndexEntry,
    demo: quiz_core::quiz::model::Quiz,
    open: RwSignal<Option<IndexEntry>>,
) -> impl IntoView {
    let t = use_t();
    let theme = Signal::stored(entry.theme.clone());
    let base = format!("workshop/{}", entry.path);
    let entry = StoredValue::new(entry);
    view! {
        <article role="button" tabindex="0" class="cursor-pointer rounded-2xl border border-white/10 hover:border-white/30 bg-white/[0.03] p-2 space-y-2"
            on:click=move |_| open.set(Some(entry.get_value()))
            on:keydown=move |ev| { if ev.key() == "Enter" || ev.key() == " " { ev.prevent_default(); open.set(Some(entry.get_value())); } }>
            <Preview theme=theme files=Signal::stored(Vec::new()) base=base quiz=demo compact=true/>
            <div class="px-2 pb-1">
                <h3 class="font-bold truncate">{entry.with_value(|e| theme_name(&e.theme, t.lang().code()).to_owned())}</h3>
                <p class="text-xs text-gray-500">{entry.with_value(|e| t.fmt("thm_by", &[("author", &e.theme.author)]))}</p>
                <p class="text-sm text-gray-400 line-clamp-2">{entry.with_value(|e| description(e, t.lang().code()))}</p>
                <p class="text-xs text-gray-500">{entry.with_value(|e| e.theme.tags.iter().map(|tag| format!("#{tag}")).collect::<Vec<_>>().join(" "))}</p>
            </div>
        </article>
    }
}
#[component]
fn Detail(
    entry: IndexEntry,
    demo: quiz_core::quiz::model::Quiz,
    close: Callback<()>,
) -> impl IntoView {
    let t = use_t();
    let store = StoredValue::new_local(store::use_store());
    let entry = StoredValue::new(entry);
    let busy = RwSignal::new(false);
    let notice = RwSignal::new(String::new());
    let applied = RwSignal::new(None::<String>);
    let target = RwSignal::new(String::new());
    let quizzes = LocalResource::new(move || async move {
        let mut list = store
            .get_value()
            .list_quizzes()
            .await
            .map_err(|e| e.to_string())?;
        list.retain(|q| !q.builtin);
        list.sort_by_key(|q| std::cmp::Reverse(q.updated_at));
        if let Some(first) = list.first() {
            target.set(first.id.clone());
        }
        Ok::<_, String>(list)
    });
    let key = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            close.run(());
        }
    });
    on_cleanup(move || key.remove());
    let act = Callback::new(move |kind: &'static str| {
        if busy.get_untracked() {
            return;
        }
        let target = target.get_untracked();
        if kind == "quiz" {
            let chosen = quizzes
                .get_untracked()
                .and_then(|r| r.ok())
                .and_then(|list| list.iter().find(|q| q.id == target).cloned());
            let Some(chosen) = chosen else {
                return;
            };
            if let Some(custom) = chosen.fields.get("customTheme") {
                let name = quiz_core::theme::format::normalize_theme(custom)
                    .theme
                    .map(|theme| theme_name(&theme, t.lang().code()).to_owned())
                    .unwrap_or_default();
                if !window()
                    .confirm_with_message(&t.fmt("thm_new_confirm", &[("name", &name)]))
                    .unwrap_or(false)
                {
                    return;
                }
            }
        }
        busy.set(true);
        notice.set(String::new());
        applied.set(None);
        let store = store.get_value();
        let entry = entry.get_value();
        leptos::task::spawn_local(async move {
            let result: Result<String, String> = async {
                let pack = fetch_pack(&entry).await?;
                if kind == "download" {
                    download(&pack.theme, &media_from_assets(pack.assets))?;
                    return Ok(String::new());
                }
                let (theme, assets) = theme_for_quiz_media(&pack);
                let files = media_from_assets(assets);
                if kind == "library" {
                    store
                        .put_theme(&LibraryTheme {
                            id: store::new_id(),
                            theme,
                            files,
                            saved_at: 0,
                        })
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(t.fmt(
                        "te_lib_saved",
                        &[("name", theme_name(&entry.theme, t.lang().code()))],
                    ))
                } else {
                    store
                        .put_media(&target, &files)
                        .await
                        .map_err(|e| e.to_string())?;
                    save_quiz_theme(&store, &target, theme).await?;
                    let title = store
                        .get_quiz(&target)
                        .await
                        .map_err(|e| e.to_string())?
                        .map(|q| q.title)
                        .unwrap_or_default();
                    applied.set(Some(target));
                    Ok(t.fmt(
                        "ws_used",
                        &[
                            ("name", theme_name(&entry.theme, t.lang().code())),
                            ("quiz", &title),
                        ],
                    ))
                }
            }
            .await;
            notice
                .set(result.unwrap_or_else(|message| t.fmt("ws_failed", &[("message", &message)])));
            busy.set(false);
        });
    });
    view! {
        <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-start justify-center overflow-y-auto p-4 sm:p-8" on:click=move |_| close.run(())>
            <section role="dialog" aria-modal="true" aria-label=entry.with_value(|e| theme_name(&e.theme, t.lang().code()).to_owned()) class="theme-dialog w-full max-w-5xl rounded-2xl bg-[#0d0d10] border border-white/15 p-5 space-y-4" on:click=|ev| ev.stop_propagation()>
                <button type="button" class=BUTTON on:click=move |_| close.run(())>{t.get("chk_close")}</button>
                <h2 class="text-2xl font-bold">{entry.with_value(|e| theme_name(&e.theme, t.lang().code()).to_owned())}</h2>
                <p class="text-sm text-gray-500">{entry.with_value(|e| t.fmt("thm_by", &[("author", &e.theme.author)]))}</p>
                <p class="text-gray-300">{entry.with_value(|e| description(e, t.lang().code()))}</p>
                <Preview theme=Signal::stored(entry.get_value().theme) files=Signal::stored(Vec::new()) base=entry.with_value(|e| format!("workshop/{}", e.path)) quiz=demo/>
                <div class="flex flex-wrap items-center gap-2">
                    {move || quizzes.get().map(|result| match result {
                        Err(e) => view! { <p role="alert">{e}</p> }.into_any(),
                        Ok(list) if list.is_empty() => view! { <p class="text-gray-500">{t.get("ws_no_quizzes")}</p> }.into_any(),
                        Ok(list) => view! {
                            <select class="bg-black/40 border border-white/10 rounded-xl px-3 py-2 max-w-xs" aria-label=t.get("ws_pick_quiz") prop:value=move || target.get() on:change=move |ev| target.set(event_target_value(&ev))>
                                {list.iter().map(|q| view! { <option value=q.id.clone()>{if q.title.is_empty() { t.get("ed_untitled") } else { q.title.clone() }}</option> }).collect_view()}
                            </select>
                            <button type="button" class=BUTTON disabled=move || busy.get() on:click=move |_| act.run("quiz")>{t.get("ws_use")}</button>
                        }.into_any(),
                    })}
                    <button type="button" class=BUTTON disabled=move || busy.get() on:click=move |_| act.run("library")>{t.get("te_lib_save")}</button>
                    <button type="button" class=BUTTON disabled=move || busy.get() on:click=move |_| act.run("download")>{t.get("ws_download")}</button>
                </div>
                <p role="status" class="text-sm text-yellow-300">{move || if busy.get() { t.get("loading") } else { notice.get() }}</p>
                {move || applied.get().map(|id| view! { <a href=Route::Edit(id).hash() class="underline">{t.get("ws_open_setup")}</a> })}
            </section>
        </div>
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn search_includes_metadata_and_intersects_tag() {
        let entry = IndexEntry {
            path: "themes/test/".into(),
            added_at: String::new(),
            theme: quiz_core::theme::builtin::theme_by_id("halloween").clone(),
        };
        assert!(matches(&entry, "", "", "cs"));
        assert!(matches(&entry, " HALLOWEEN ", "", "en"));
        assert!(!matches(&entry, "", "missing-tag", "en"));
        assert!(!matches(&entry, "no matching phrase", "", "en"));
    }
}
