//! Local question editor. All question mutations are quiz-core sheet plans.
use crate::{router::Route, store};
use leptos::prelude::*;
use quiz_core::quiz::{
    examples,
    model::Quiz,
    parse::{Round, is_url, media_key},
    sheet_model::*,
    validate::{ValidationInput, validate_quiz},
};
use quiz_store::{Media, MediaStore, QuizStore};
use wasm_bindgen::{JsCast, closure::Closure};

mod import;
use import::CsvImport;

const BUTTON: &str =
    "px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm disabled:opacity-40";
const INPUT: &str = "w-full bg-black/40 border border-white/15 rounded-xl px-4 py-3 text-white focus:border-yellow-500 outline-none";
const TYPES: [(&str, &str); 12] = [
    ("Written", "written"),
    ("Numeric", "numeric"),
    ("ABCD", "abcd"),
    ("Yes/No", "yesno"),
    ("Top5", "top5"),
    ("Sort", "sort"),
    ("Image", "image"),
    ("PImage", "pimage"),
    ("Audio", "audio"),
    ("Video", "video"),
    ("PVideo", "pvideo"),
    ("Handout", "handout"),
];

// CSV is kept losslessly as cells, including columns outside the editor model.
fn csv_rows(csv: &str) -> Vec<Vec<String>> {
    let mut rows = vec![];
    let mut row = vec![];
    let mut cell = String::new();
    let mut chars = csv.trim_start_matches('\u{feff}').chars().peekable();
    let mut quoted = false;
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                chars.next();
                cell.push('"');
            }
            '"' => quoted = !quoted,
            ',' if !quoted => row.push(std::mem::take(&mut cell)),
            '\r' | '\n' if !quoted => {
                if c == '\r' && chars.peek() == Some(&'\n') {
                    chars.next();
                }
                row.push(std::mem::take(&mut cell));
                rows.push(std::mem::take(&mut row));
            }
            _ => cell.push(c),
        }
    }
    if !cell.is_empty() || !row.is_empty() || (!csv.is_empty() && !csv.ends_with(['\r', '\n'])) {
        row.push(cell);
        rows.push(row);
    }
    rows
}
fn csv_text(rows: &[Vec<String>]) -> String {
    rows.iter()
        .map(|r| {
            r.iter()
                .map(|s| {
                    if s.contains([',', '"', '\n', '\r']) || s.starts_with(' ') || s.ends_with(' ')
                    {
                        format!("\"{}\"", s.replace('"', "\"\""))
                    } else {
                        s.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join("\r\n")
}
fn sheet(csv: &str) -> Sheet {
    let rows = csv_rows(csv);
    read_sheet(&to_model_rows(&rows, &stamp_columns(&rows)))
}
fn apply(csv: &str, plan: &Plan) -> String {
    let mut raw = csv_rows(csv);
    let cols = stamp_columns(&raw);
    let mapped = |i| match i {
        10 => cols.author,
        11 => cols.changed,
        _ => i,
    };
    let insert = plan
        .insert
        .iter()
        .enumerate()
        .map(|(index, r)| {
            let mut out = if index < plan.delete_count {
                raw[plan.start + index].clone()
            } else {
                Vec::new()
            };
            out.resize(
                out.len().max((cols.author.max(cols.changed) + 1).max(12)),
                String::new(),
            );
            for (i, v) in r.iter().enumerate().take(12) {
                out[mapped(i)] = v.clone();
            }
            out
        })
        .collect::<Vec<_>>();
    raw.splice(plan.start..plan.start + plan.delete_count, insert);
    for f in &plan.fixups {
        let c = mapped(f.col);
        let size = raw[f.row].len().max(c + 1);
        raw[f.row].resize(size, String::new());
        raw[f.row][c] = f.value.clone();
    }
    csv_text(&raw)
}
#[derive(Clone)]
enum Selection {
    Edit {
        ri: usize,
        qi: usize,
        rows: Vec<Vec<String>>,
    },
    Add(String),
    Round,
}
#[derive(Clone)]
enum Action {
    Save,
    Delete,
    Rename(String, String),
    Move(Vec<Vec<String>>, usize, usize, MoveDirection),
    Examples,
    Import(String),
}
fn locate(s: &Sheet, ri: usize, qi: usize, rows: &[Vec<String>]) -> Option<(usize, usize)> {
    find_block(
        s,
        rows,
        Some(&BlockLocation {
            ri: ri as isize,
            qi: qi as isize,
        }),
    )
    .map(|p| (p.ri as usize, p.qi as usize))
}
fn make_plan(
    s: &Sheet,
    sel: &Selection,
    d: &Draft,
    title: &str,
    meta: &Meta,
    action: &Action,
) -> Option<Plan> {
    match action {
        Action::Rename(marker, title) => plan_rename(s, find_round(s, marker)?, title, meta),
        Action::Move(rows, ri, qi, dir) => {
            let (ri, qi) = locate(s, *ri, *qi, rows)?;
            plan_move(s, ri, qi, dir.clone(), meta)
        }
        Action::Delete => {
            if let Selection::Edit { ri, qi, rows } = sel {
                let (ri, qi) = locate(s, *ri, *qi, rows)?;
                plan_delete(s, ri, qi, meta)
            } else {
                None
            }
        }
        Action::Save => match sel {
            Selection::Edit { ri, qi, rows } => {
                let (ri, qi) = locate(s, *ri, *qi, rows)?;
                plan_save(s, ri, qi, d, meta)
            }
            Selection::Add(marker) => plan_add(s, find_round(s, marker)?, d, meta),
            Selection::Round => Some(plan_add_round(s, title, d, meta)),
        },
        Action::Examples | Action::Import(_) => None,
    }
}
fn draft_round(s: &Sheet, selection: Option<&Selection>, draft: &Draft, title: &str) -> Round {
    let selected = match selection {
        Some(Selection::Edit { ri, .. }) => s.rounds.get(*ri),
        Some(Selection::Add(marker)) => find_round(s, marker).and_then(|ri| s.rounds.get(ri)),
        _ => None,
    };
    let mut question = preview_question(draft);
    if let Some(round) = selected {
        if let Some(Selection::Edit { qi, .. }) = selection {
            if let Some(block) = round.questions.get(*qi) {
                question.id = block.question.id.clone();
            }
        } else if let Some(number) = round
            .questions
            .last()
            .and_then(|b| b.rows.last())
            .and_then(|row| row.get(1))
            .and_then(|n| n.parse::<f64>().ok())
        {
            question.id = (number + 1.0).to_string();
        }
    }
    Round {
        id: "editor".into(),
        number: selected.map(|r| r.number.clone()).unwrap_or_default(),
        title: selected
            .map(|r| r.title.clone())
            .unwrap_or_else(|| title.into()),
        rows: vec![],
        questions: vec![question],
    }
}

fn upload_path(typed: &str, file: &str) -> String {
    typed
        .trim()
        .rsplit_once('/')
        .map_or_else(|| file.into(), |(folder, _)| format!("{folder}/{file}"))
}
fn shuffled(items: &[DraftItem]) -> Vec<DraftItem> {
    with_letters(items, &mut js_sys::Math::random)
}
fn confirm(message: &str) -> bool {
    window().confirm_with_message(message).unwrap_or(false)
}
fn stamp() -> String {
    let d = js_sys::Date::new_0();
    stamp_now(
        d.get_full_year() as i32,
        d.get_month() + 1,
        d.get_date(),
        d.get_hours(),
        d.get_minutes(),
    )
}

#[component]
pub fn Page(id: String) -> impl IntoView {
    let store = StoredValue::new_local(store::use_store());
    let id = StoredValue::new(id);
    let t = crate::i18n::use_t();
    let loaded = LocalResource::new(move || {
        let store = store.get_value();
        let id = id.get_value();
        async move { store.get_quiz(&id).await.map_err(|e| e.to_string()) }
    });
    view! { <main class="min-h-screen bg-[#050505] text-white font-sans questions-enter">
        {move || loaded.get().map(|result| match result {
            Ok(Some(q))=>view! {<Editor quiz=q/>}.into_any(),
            Ok(None)=>view! {<p class="p-12">{t.get("ed_not_found")} <a href="#/">{t.get("settings_exit")}</a></p>}.into_any(),
            Err(e)=>view! {<p role="alert" class="p-12">{e}</p>}.into_any(),
        }).unwrap_or_else(|| view! {<p class="p-12">{t.get("loading")}</p>}.into_any())}
    </main> }
}

#[component]
fn Editor(quiz: Quiz) -> impl IntoView {
    let t = crate::i18n::use_t();
    let store = StoredValue::new_local(store::use_store());
    let quiz = RwSignal::new(quiz);
    let data = Memo::new(move |_| sheet(&quiz.get().questions_csv));
    let selection = RwSignal::new(None::<Selection>);
    let draft = RwSignal::new(empty_draft("Written"));
    let dirty = RwSignal::new(false);
    let title = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let notice = RwSignal::new(String::new());
    let names = RwSignal::new(Vec::<String>::new());
    let renaming = RwSignal::new(None::<(String, String)>);
    let conflict = RwSignal::new(None::<Block>);
    let view_step = RwSignal::new("question");
    let media = LocalResource::new(move || {
        store.with_value(|s| {
            s.revision().get();
        });
        let store = store.get_value();
        let id = quiz.get_untracked().id;
        async move { store.list_media(&id).await.map_err(|e| e.to_string()) }
    });
    Effect::new(move |_| {
        if let Some(Ok(files)) = media.get() {
            let mut available = files.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
            if quiz.get_untracked().builtin {
                available.extend(quiz_store::package::referenced_media(&quiz.get_untracked()));
            }
            available.sort();
            available.dedup();
            names.set(available);
        }
    });
    let preview_urls = StoredValue::new_local(store::MediaUrls::default());
    let preview_version = RwSignal::new(0_u64);
    Effect::new(move |_| {
        if let Some(Ok(files)) = media.get() {
            let mut urls = store::MediaUrls::default();
            for file in files {
                if let Ok(blob) = store::blob(&file.bytes, &file.content_type)
                    && let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob)
                {
                    if let Some(old) = urls.0.insert(media_key(&file.name), url) {
                        let _ = web_sys::Url::revoke_object_url(&old);
                    }
                }
            }
            preview_urls.set_value(urls);
            preview_version.update(|v| *v += 1);
        }
    });
    let choose = Callback::new(move |sel: Option<Selection>| {
        if busy.get_untracked() || (dirty.get_untracked() && !confirm(&t.get("qe_discard"))) {
            return;
        }
        if let Some(Selection::Edit { ri, qi, .. }) = &sel {
            draft.set(draft_from(
                &data.get_untracked().rounds[*ri].questions[*qi].question,
            ));
        } else {
            draft.set(empty_draft("Written"));
            title.set(String::new());
        }
        view_step.set("question");
        conflict.set(None);
        selection.set(sel);
        dirty.set(false);
        notice.set(String::new());
    });
    let action = Callback::new(move |action: Action| {
        if busy.get_untracked() {
            return;
        }
        // TODO(auth): authenticated Google Sheet write-back and collaborative presence.
        if matches!(action, Action::Delete) && !confirm(&format!("{}?", t.get("qe_delete"))) {
            return;
        }
        if matches!(action, Action::Import(_) | Action::Examples)
            && dirty.get_untracked()
            && !confirm(&t.get("qe_discard"))
        {
            return;
        }
        let baseline = quiz.get_untracked().questions_csv;
        busy.set(true);
        notice.set(String::new());
        let store = store.get_value();
        let id = quiz.get_untracked().id;
        let sel = selection.get_untracked().unwrap_or(Selection::Round);
        let d = draft.get_untracked();
        let round_title = title.get_untracked();
        let meta = Meta {
            author: String::new(),
            changed: stamp(),
            headers: Some(StampHeaders {
                author: t.get("qe_col_author"),
                changed: t.get("qe_col_changed"),
            }),
        };
        leptos::task::spawn_local(async move {
            let mut saved_target = None;
            let result: Result<Quiz, String> = async {
                let current = store
                    .get_quiz(&id)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| t.get("ed_not_found"))?;
                let fresh = sheet(&current.questions_csv);
                if matches!(action, Action::Save) {
                    saved_target = match &sel {
                        Selection::Edit { ri, qi, rows } => locate(&fresh, *ri, *qi, rows),
                        Selection::Add(marker) => find_round(&fresh, marker)
                            .map(|ri| (ri, fresh.rounds[ri].questions.len())),
                        Selection::Round => Some((fresh.rounds.len(), 0)),
                    };
                }
                let csv = if let Action::Import(csv) = &action {
                    if current.questions_csv != baseline {
                        return Err(t.get("qe_conflict_round"));
                    }
                    csv.clone()
                } else if matches!(action, Action::Examples) {
                    if !fresh.rounds.is_empty() {
                        return Err(t.get("qe_conflict_round"));
                    }
                    let example = examples::example_questions(
                        &current.settings.language,
                        &t.get("ed_example_round"),
                    );
                    let mut files = vec![];
                    for name in &example.media {
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
                            return Err(format!("Missing example media: {name}"));
                        }
                        files.push(Media {
                            name: name.clone(),
                            content_type: response
                                .headers()
                                .get("content-type")
                                .unwrap_or_default(),
                            bytes: response.binary().await.map_err(|e| e.to_string())?,
                            ..Default::default()
                        });
                    }
                    store
                        .put_media(&id, &files)
                        .await
                        .map_err(|e| e.to_string())?;
                    names.update(|n| {
                        n.extend(files.iter().map(|f| f.name.clone()));
                        n.sort();
                        n.dedup();
                    });
                    example.csv
                } else {
                    if matches!(action, Action::Move(..))
                        && current.questions_csv != quiz.get_untracked().questions_csv
                    {
                        quiz.set(current);
                        return Err(t.get("qe_conflict_round"));
                    }
                    let Some(plan) = make_plan(&fresh, &sel, &d, &round_title, &meta, &action)
                    else {
                        if let Selection::Edit { ri, qi, .. } = sel.clone() {
                            if let Some(b) = fresh.rounds.get(ri).and_then(|r| r.questions.get(qi))
                            {
                                conflict.set(Some(b.clone()));
                                selection.set(Some(Selection::Edit {
                                    ri,
                                    qi,
                                    rows: b.rows.clone(),
                                }));
                            }
                        }
                        quiz.set(current);
                        return Err(t.get("qe_conflict_round"));
                    };
                    apply(&current.questions_csv, &plan)
                };
                // Re-check inside the atomic local store update: never overwrite another tab's edit.
                let expected = current.questions_csv;
                let mut changed = false;
                let saved = store
                    .update_quiz(&id, |q| {
                        if q.questions_csv == expected {
                            q.questions_csv = csv;
                            // Local edits become authoritative instead of re-importing an older Sheet at show start.
                            q.questions_sheet_url.clear();
                            changed = true;
                        }
                    })
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| t.get("ed_not_found"))?;
                if !changed {
                    quiz.set(saved);
                    return Err(t.get("qe_conflict_round"));
                }
                Ok(saved)
            }
            .await;
            match result {
                Ok(saved) => {
                    let next = sheet(&saved.questions_csv);
                    if matches!(action, Action::Save) {
                        let target = saved_target;
                        if let Some((ri, qi)) = target
                            && let Some(b) = next.rounds.get(ri).and_then(|r| r.questions.get(qi))
                        {
                            draft.set(draft_from(&b.question));
                            selection.set(Some(Selection::Edit {
                                ri,
                                qi,
                                rows: b.rows.clone(),
                            }));
                        }
                    } else if matches!(action, Action::Move(..)) {
                        if let Some(Selection::Edit { ri, qi, rows }) = selection.get_untracked() {
                            if let Some((ri, qi)) = locate(&next, ri, qi, &rows) {
                                selection.set(Some(Selection::Edit {
                                    ri,
                                    qi,
                                    rows: next.rounds[ri].questions[qi].rows.clone(),
                                }));
                            }
                        }
                    } else if !matches!(action, Action::Rename(..)) {
                        selection.set(None);
                    }
                    if let Action::Rename(old, _) = &action
                        && let Some(Selection::Add(marker)) = selection.get_untracked()
                        && &marker == old
                        && let Some(ri) = find_round(&data.get_untracked(), old)
                    {
                        selection.set(Some(Selection::Add(next.rounds[ri].marker.clone())));
                    }
                    quiz.set(saved);
                    if matches!(
                        action,
                        Action::Save | Action::Delete | Action::Examples | Action::Import(_)
                    ) {
                        dirty.set(false);
                    }
                    renaming.set(None);
                    conflict.set(None);
                    notice.set(t.get("ed_saved"));
                }
                Err(e) => notice.set(e),
            }
            busy.set(false);
        });
    });
    let key =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
            if (e.ctrl_key() || e.meta_key()) && e.key().eq_ignore_ascii_case("s") {
                e.prevent_default();
                if selection.get_untracked().is_some() && dirty.get_untracked() {
                    action.run(Action::Save);
                }
            }
        });
    let _ = window().add_event_listener_with_callback("keydown", key.as_ref().unchecked_ref());
    let leave = Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| {
        if dirty.get_untracked() {
            e.prevent_default();
            let _ = js_sys::Reflect::set(&e, &"returnValue".into(), &"".into());
        }
    });
    let _ =
        window().add_event_listener_with_callback("beforeunload", leave.as_ref().unchecked_ref());
    let handlers = StoredValue::new_local((key, leave));
    on_cleanup(move || {
        handlers.with_value(|(key, leave)| {
            let _ = window()
                .remove_event_listener_with_callback("keydown", key.as_ref().unchecked_ref());
            let _ = window().remove_event_listener_with_callback(
                "beforeunload",
                leave.as_ref().unchecked_ref(),
            );
        })
    });
    let preview = Memo::new(move |_| {
        draft_round(
            &data.get(),
            selection.get().as_ref(),
            &draft.get(),
            &title.get(),
        )
    });
    let issues = Memo::new(move |_| {
        let round = preview.get();
        validate_quiz(
            &ValidationInput {
                scores_sheet_url: Some("-".into()),
                ..Default::default()
            },
            &[round],
            |name| names.get().iter().any(|n| media_key(n) == media_key(name)),
        )
        .into_iter()
        .filter(|i| i.q_index == 0 && i.code != "duplicate_number")
        .collect::<Vec<_>>()
    });
    view! {
        <header class="sticky top-0 z-30 bg-[#050505]/95 backdrop-blur border-b border-white/10"><div class="max-w-[1800px] mx-auto px-6 py-4 flex flex-wrap items-center gap-4">
            <a class="text-gray-400 hover:text-white" href=Route::Edit(quiz.get_untracked().id).hash() on:click=move |e| {if busy.get_untracked() || (dirty.get_untracked() && !confirm(&t.get("qe_discard"))) {e.prevent_default();}}>{t.get("qe_back")}</a>
            <h1 class="text-2xl font-black">{t.get("qe_title")}": "{move ||quiz.get().title}</h1>
        </div></header>
        <div class="max-w-[1800px] mx-auto px-6 py-6 space-y-6">
            <p role="status" class="text-sm text-yellow-200">{move ||notice.get()}</p>
            <CsvImport action=action busy=busy/>
            <Show when=move ||data.get().rounds.is_empty()>
                <section class="rounded-2xl border border-white/10 p-6 space-y-4"><h2 class="text-xl font-bold">{t.get("qs_title")}</h2>
                    <div class="grid xl:grid-cols-3 gap-3">
                        <div class="rounded-xl border border-white/10 bg-black/30 p-4 space-y-3 flex flex-col"><h3 class="font-bold text-sm">{t.get("qs_sheet_title")}</h3><p class="text-sm text-gray-400">{t.get("qs_sheet_text")}</p><a class=BUTTON href="https://docs.google.com/spreadsheets/d/1p1-bxa2EBWcRvRE_bWu3AbHaSHkgjD7jgBcvu1tWNvw/copy" target="_blank" rel="noreferrer">{t.get("qs_sheet_copy")}</a></div>
                        <div class="rounded-xl border border-white/10 bg-black/30 p-4 space-y-3 flex flex-col"><h3 class="font-bold text-sm">{t.get("qs_examples_title")}</h3><p class="text-sm text-gray-400">{t.get("qs_examples_text")}</p><button class=BUTTON disabled=move ||busy.get() on:click=move |_|action.run(Action::Examples)>{t.get("qs_examples_load")}</button></div>
                        <div class="rounded-xl border border-white/10 bg-black/30 p-4 space-y-3 flex flex-col"><h3 class="font-bold text-sm">{t.get("qs_csv_title")}</h3><p class="text-sm text-gray-400">{t.get("qs_csv_text")}</p><a class=BUTTON href="templates/questions-template.csv" download="questions-template.csv">{t.get("qs_csv_download")}</a></div>
                    </div>
                    <a class="text-sm text-gray-400 hover:text-yellow-400" href="#/help/questions">{t.get("qs_help")}</a>
                    <p class="text-sm text-gray-400">{t.get("qs_preview")}</p>
                    <div class="overflow-x-auto"><table class="text-xs text-gray-400"><tbody>{examples::example_questions(&quiz.get_untracked().settings.language,&t.get("ed_example_round")).rows.into_iter().take(4).map(|r|view!{<tr>{r.into_iter().take(10).map(|s|view!{<td class="border border-white/10 p-2">{s}</td>}).collect_view()}</tr>}).collect_view()}</tbody></table></div>
                </section>
            </Show>
            <div class="grid lg:grid-cols-[320px_minmax(0,1fr)] gap-6 items-start">
                <nav class="space-y-4" aria-label=t.get("qe_title")>
                    <details class="rounded-xl bg-white/5 p-3" open><summary class="text-sm text-gray-400">{t.get("qe_recent")}</summary>
                    {move || { let mut recent=data.get().rounds.iter().enumerate().flat_map(|(ri,r)|r.questions.iter().enumerate().map(move |(qi,b)|(ri,qi,b.clone()))).filter(|(_,_,b)|parse_stamp(&b.changed).is_some()).collect::<Vec<_>>(); recent.sort_by(|a,b|b.2.changed.cmp(&a.2.changed));recent.into_iter().take(6).map(|(ri,qi,b)|view!{<button class="block w-full text-left text-xs p-2 text-gray-400 hover:text-white" on:click=move |_|choose.run(Some(Selection::Edit{ri,qi,rows:b.rows.clone()}))>{format!("{} · {}",b.changed,b.question.text)}</button>}).collect_view() }}
                    </details>
                    {move ||data.get().rounds.into_iter().enumerate().map(|(ri,r)| {let marker=r.marker.clone();let rename_marker=marker.clone();let round_name=r.title.clone();let count=r.questions.len();view!{
                        <section class="rounded-2xl bg-white/[0.04] border border-white/10 overflow-hidden">
                            <div class="px-4 py-3 bg-white/5 flex items-center gap-2"><h2 class="font-bold flex-1">{format!("{}. {}",r.number,r.title)}</h2><button class=BUTTON disabled=move ||busy.get() aria-label=t.get("qe_rename") on:click=move |_|renaming.set(Some((rename_marker.clone(),round_name.clone())))><Icon name="edit"/></button></div>
                            <ul>{r.questions.into_iter().enumerate().map(|(qi,b)| {let rows=b.rows.clone();let up=rows.clone();let down=rows.clone();let kind=draft_from(&b.question).kind; let label=if b.question.text.trim().is_empty(){b.question.media.clone().unwrap_or_else(||"—".into())}else{b.question.text.clone()};view!{
                                <li class="flex items-center border-t border-white/5"><button class=move ||if matches!(selection.get(),Some(Selection::Edit{ri:a,qi:c,..}) if a==ri && c==qi){"text-left flex-1 min-w-0 p-3 bg-yellow-500/10 border-l-2 border-yellow-500"}else{"text-left flex-1 min-w-0 p-3 hover:bg-white/5 border-l-2 border-transparent"} on:click=move |_|choose.run(Some(Selection::Edit{ri,qi,rows:rows.clone()}))><span class="block truncate text-sm">{format!("{} · {}",b.question.id,label)}</span><span class="block text-xs text-gray-500">{format!("{} · {} {}",kind,b.author,b.changed)}</span></button>
                                <div class="flex flex-col"><button class="px-2 text-gray-400 disabled:opacity-20" aria-label=t.get("qe_move_up") disabled=move ||qi==0 ||busy.get() on:click=move |_|action.run(Action::Move(up.clone(),ri,qi,MoveDirection::Up))><Icon name="up"/></button><button class="px-2 text-gray-400 disabled:opacity-20" aria-label=t.get("qe_move_down") disabled=move ||qi+1==count ||busy.get() on:click=move |_|action.run(Action::Move(down.clone(),ri,qi,MoveDirection::Down))><Icon name="down"/></button></div>
                                </li>
                            }}).collect_view()}</ul>
                            <button class="w-full px-4 py-3 text-left text-sm text-gray-400 hover:text-white border-t border-white/5" on:click=move |_|choose.run(Some(Selection::Add(marker.clone())))>{t.get("qe_add_question")}</button>
                        </section>
                    }}).collect_view()}
                    <button class=BUTTON disabled=move ||busy.get() on:click=move |_|choose.run(Some(Selection::Round))>{t.get("qe_add_round")}</button>
                </nav>
                <section class="min-w-0">
                    <Show when=move ||selection.get().is_some() fallback=move ||view!{<p class="rounded-2xl border-2 border-dashed border-white/10 p-12 text-center text-gray-500">{t.get("qe_pick")}</p>}>
                        <div class="rounded-2xl bg-white/[0.04] border border-white/10 p-6 space-y-5">
                            <div class="flex justify-between"><h2 class="text-xl font-bold">{move ||{
                                if matches!(selection.get(),Some(Selection::Round)) { t.get("qe_new_round") } else {
                                    let round = preview.get();
                                    let label = if matches!(selection.get(),Some(Selection::Edit{..})) { t.fmt("qe_number", &[("n", &round.questions[0].id)]) } else { t.get("qe_new_question") };
                                    format!("{}. {} · {}",round.number,round.title,label)
                                }
                            }}</h2><button class=BUTTON on:click=move |_|choose.run(None)>{t.get("qe_cancel")}</button></div>
                            <Show when=move ||matches!(selection.get(),Some(Selection::Round))><label class="block">{t.get("qe_round_title")}<input class=INPUT prop:value=move ||title.get() on:input=move |e| {title.set(event_target_value(&e));dirty.set(true);}/></label></Show>
                            {move || conflict.get().map(|b|view!{<div class="rounded-xl bg-yellow-500/10 border border-yellow-500/30 p-4 space-y-2 text-sm"><p>{t.get("qe_conflict")}</p><p>{b.question.text.clone()}</p><button class=BUTTON on:click=move |_|{draft.set(draft_from(&b.question));dirty.set(false);conflict.set(None);}>{t.get("qe_load_theirs")}</button><span>{t.get("qe_keep_mine")}</span></div>})}
                            <div class="grid 2xl:grid-cols-2 gap-6 items-start">
                                <QuestionForm draft=draft dirty=dirty busy=busy names=names quiz_id=quiz.get_untracked().id view_step=view_step is_new=Signal::derive(move ||!matches!(selection.get(),Some(Selection::Edit{..})))/>
                                <div class="space-y-3"><p class="text-xs font-bold uppercase text-gray-400">{t.get("qe_preview")}</p>
                                    {move || {
                                        preview_version.get();
                                        let d = draft.get();
                                        let step = if view_step.get() == "media" { 1 } else { quiz_core::quiz::validate::preview_step(&d.kind, view_step.get()) };
                                        let round = preview.get();
                                        let builtin = quiz.get().builtin;
                                        let resolve_media = Callback::new(move |name: String| {
                                            if is_url(&name) { Some(name) } else {
                                                preview_urls.with_value(|urls| urls.0.get(&media_key(&name)).cloned())
                                                    .or_else(|| builtin.then(|| format!("source/{name}")))
                                            }
                                        });
                                        view! { <DraftPreview round=round step=step resolve_media=resolve_media quiz=quiz.get()/> }
                                    }}
                                    <ul class="space-y-2 text-sm">{move ||issues.get().into_iter().map(|i| {let params=i.params.as_object().map(|p|p.iter().map(|(k,v)|(k.clone(),v.as_str().map(str::to_string).unwrap_or_else(||v.to_string()))).collect::<Vec<_>>()).unwrap_or_default(); let refs=params.iter().map(|(k,v)|(k.as_str(),v.as_str())).collect::<Vec<_>>();view!{<li class=if i.level==quiz_core::quiz::validate::Level::Error {"text-red-300"}else{"text-yellow-300"}><Rich text=t.fmt(&format!("chk_{}",i.code),&refs)/></li>}}).collect_view()}</ul>
                                </div>
                            </div>
                            <div class="flex flex-wrap gap-3 border-t border-white/5 pt-4"><button class="px-4 py-3 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold disabled:opacity-40" disabled=move ||busy.get() || (!dirty.get() && matches!(selection.get(),Some(Selection::Edit{..}))) on:click=move |_|action.run(Action::Save)>{move ||if busy.get(){t.get("qe_saving")}else{t.get("save_close")}}</button><span class="text-xs text-gray-500 self-center">{t.get("qe_keys")}</span><Show when=move ||matches!(selection.get(),Some(Selection::Edit{..}))><button class="ml-auto px-4 py-3 rounded-xl text-red-300 hover:bg-red-500/20" disabled=move ||busy.get() on:click=move |_|action.run(Action::Delete)>{t.get("qe_delete")}</button></Show></div>
                        </div>
                    </Show>
                </section>
            </div>
            {move ||renaming.get().map(|(marker,name)| {let value=RwSignal::new(name);view!{<div class="fixed inset-0 z-50 bg-black/85 flex items-center justify-center p-6" on:keydown=move |e:web_sys::KeyboardEvent|{if e.key()=="Escape" && !busy.get_untracked(){renaming.set(None);}}><section role="dialog" aria-modal="true" aria-label=t.get("qe_rename") class="rounded-2xl bg-[#151515] border border-white/10 p-6 space-y-4 w-full max-w-lg"><label>{t.get("qe_round_title")}<input class=INPUT prop:value=move ||value.get() on:input=move |e|value.set(event_target_value(&e))/></label><button class=BUTTON disabled=move ||busy.get() on:click=move |_|action.run(Action::Rename(marker.clone(),value.get_untracked()))>{t.get("save_close")}</button><button class=BUTTON on:click=move |_|renaming.set(None)>{t.get("qe_cancel")}</button></section></div>}})}
        </div>
    }
}

#[component]
fn DraftPreview(
    round: Round,
    step: usize,
    resolve_media: Callback<String, Option<String>>,
    quiz: Quiz,
) -> impl IntoView {
    let lang = crate::i18n::Lang::from_code(&quiz.settings.language);
    crate::i18n::provide_translations(Signal::stored(lang));
    let custom = quiz
        .fields
        .get("customTheme")
        .and_then(|v| serde_json::from_value::<quiz_core::theme::Theme>(v.clone()).ok());
    let theme = quiz_core::theme::builtin::theme_for_quiz(
        quiz.settings
            .fields
            .get("theme")
            .and_then(|v| v.as_str())
            .unwrap_or("default"),
        custom.as_ref(),
    )
    .clone();
    view! { <crate::theme::ThemeScope theme=Signal::stored(theme) class="questions-preview rounded-xl overflow-hidden border border-white/15">
        <crate::scenes::game::QuestionPreview round=round question_index=Some(0) step=step resolve_media=resolve_media/>
    </crate::theme::ThemeScope> }
}

#[component]
fn Icon(name: &'static str) -> impl IntoView {
    let path = match name {
        "up" => "m6 15 6-6 6 6",
        "down" => "m6 9 6 6 6-6",
        _ => "m16 3 5 5-12 12-6 1 1-6L16 3Z M14 5l5 5",
    };
    view! { <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d=path/></svg> }
}

#[component]
fn Rich(text: String) -> impl IntoView {
    text.split('`').enumerate().map(|(i,s)|if i%2==1 {view!{<code class="px-1.5 py-0.5 rounded bg-white/10 text-yellow-200 font-mono whitespace-nowrap">{s.to_owned()}</code>}.into_any()}else{s.to_owned().into_any()}).collect_view()
}

#[derive(Clone, Copy)]
enum Field {
    Answer,
    Media,
    Option(&'static str),
    Top(usize),
    Item(usize),
    Picture(usize),
}
fn value(d: &Draft, f: Field) -> String {
    match f {
        Field::Answer => d.answer.clone(),
        Field::Media => d.media.clone(),
        Field::Option(k) => d.options.get(k).cloned().unwrap_or_default(),
        Field::Top(i) => d.answers.get(i).cloned().unwrap_or_default(),
        Field::Item(i) => d.items.get(i).map(|i| i.text.clone()).unwrap_or_default(),
        Field::Picture(i) => d.items.get(i).map(|i| i.media.clone()).unwrap_or_default(),
    }
}
fn set(d: &mut Draft, f: Field, v: String) {
    match f {
        Field::Answer => d.answer = v,
        Field::Media => d.media = v,
        Field::Option(k) => {
            d.options.insert(k.into(), v);
        }
        Field::Top(i) => {
            if let Some(a) = d.answers.get_mut(i) {
                *a = v
            }
        }
        Field::Item(i) => {
            if let Some(a) = d.items.get_mut(i) {
                a.text = v
            }
        }
        Field::Picture(i) => {
            if let Some(a) = d.items.get_mut(i) {
                a.media = v
            }
        }
    }
}
#[component]
fn FieldInput(
    draft: RwSignal<Draft>,
    dirty: RwSignal<bool>,
    field: Field,
    busy: RwSignal<bool>,
    label: String,
    #[prop(default = false)] media: bool,
    names: RwSignal<Vec<String>>,
    id: String,
) -> impl IntoView {
    let t = crate::i18n::use_t();
    let store = StoredValue::new_local(store::use_store());
    let id = StoredValue::new(id);
    let uploading = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    let upload = Callback::new(move |file: web_sys::File| {
        if uploading.get_untracked() || busy.get_untracked() {
            return;
        }
        let requested = upload_path(&value(&draft.get_untracked(), field), &file.name());
        let existing = names
            .get_untracked()
            .into_iter()
            .find(|n| media_key(n) == media_key(&requested));
        if existing.is_some() && !confirm(&t.fmt("qe_upload_replace", &[("file", &requested)])) {
            return;
        }
        // Storage upserts by exact name, while quiz references compare normalized names.
        let path = existing.unwrap_or(requested);
        uploading.set(true);
        busy.set(true);
        error.set(String::new());
        let store = store.get_value();
        let id = id.get_value();
        leptos::task::spawn_local(async move {
            let result: Result<(), String> = async {
                let bytes = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
                    .await
                    .map_err(|e| format!("{e:?}"))?;
                store
                    .put_media(
                        &id,
                        &[Media {
                            name: path.clone(),
                            bytes: js_sys::Uint8Array::new(&bytes).to_vec(),
                            content_type: file.type_(),
                            ..Default::default()
                        }],
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                names.update(|n| {
                    n.push(path.clone());
                    n.sort();
                    n.dedup();
                });
                draft.update(|d| set(d, field, path));
                dirty.set(true);
                Ok(())
            }
            .await;
            if let Err(e) = result {
                error.set(e);
            }
            uploading.set(false);
            busy.set(false);
        });
    });
    view! {<div class="space-y-1" on:dragover=move |e:web_sys::DragEvent|{if media{e.prevent_default();}} on:drop=move |e:web_sys::DragEvent|{if media{e.prevent_default();if let Some(f)=e.data_transfer().and_then(|d|d.files()).and_then(|f|f.item(0)){upload.run(f);}}}>
        <label class="block text-xs font-bold uppercase text-gray-400">{label}<input class=INPUT list=if media{Some("qe-media")}else{None} prop:value=move ||value(&draft.get(),field) on:input=move |e|{draft.update(|d|set(d,field,event_target_value(&e)));dirty.set(true);}/></label>
        <Show when=move ||media><label class="block text-xs text-gray-400">{t.get("qe_upload")}<input type="file" disabled=move ||uploading.get() accept=move ||match draft.get().kind.as_str(){"Audio"=>"audio/*","Video"|"PVideo"=>"video/*",_=>"image/*"} on:change=move |e|{let input=event_target::<web_sys::HtmlInputElement>(&e);if let Some(f)=input.files().and_then(|f|f.item(0)){upload.run(f);}input.set_value("");}/></label>
        <Show when=move ||{let v=value(&draft.get(),field);!v.trim().is_empty() && !is_url(&v) && !names.get().iter().any(|n|media_key(n)==media_key(&v))}><p class="text-xs text-yellow-300">{t.get("qe_media_missing_upload")}</p></Show></Show>
        <Show when=move ||uploading.get()><p role="status" class="text-xs text-gray-400">{t.get("lib_importing")}</p></Show><p role="alert" class="text-xs text-red-300">{move ||error.get()}</p>
    </div>}
}
#[component]
fn QuestionForm(
    draft: RwSignal<Draft>,
    dirty: RwSignal<bool>,
    busy: RwSignal<bool>,
    names: RwSignal<Vec<String>>,
    quiz_id: String,
    view_step: RwSignal<&'static str>,
    is_new: Signal<bool>,
) -> impl IntoView {
    let t = crate::i18n::use_t();
    let id = StoredValue::new(quiz_id);
    let expanded = RwSignal::new(is_new.get_untracked());
    Effect::new(move |_| {
        expanded.set(is_new.get());
    });
    view! {<fieldset disabled=move ||busy.get() class="space-y-5 min-w-0">
        <datalist id="qe-media">{move ||names.get().into_iter().map(|n|view!{<option value=n/>}).collect_view()}</datalist>
        <button type="button" class=BUTTON on:click=move |_|expanded.update(|v|*v=!*v)>{move ||format!("{} · {}",draft.get().kind,t.get("qt_change"))}</button>
        <Show when=move ||expanded.get()><div class="grid sm:grid-cols-2 gap-2">{TYPES.into_iter().map(|(kind,code)|view!{<button type="button" class=move ||if draft.get().kind==kind{"text-left p-3 rounded-xl border border-yellow-500 bg-yellow-500/10"}else{"text-left p-3 rounded-xl border border-white/10 bg-white/[0.03] hover:bg-white/[0.07]"} aria-pressed=move ||draft.get().kind==kind on:click=move |_|{draft.update(|d|d.kind=kind.into());dirty.set(true);expanded.set(false);view_step.set("question");}><strong class="block text-sm">{t.get(&format!("qt_{code}_name"))}</strong><code class="text-xs text-gray-500">{kind}</code><span class="block text-xs text-gray-400">{t.get(&format!("qt_{code}_desc"))}</span></button>}).collect_view()}</div></Show>
        <div class="flex gap-2 text-xs"><span>{t.get("qt_how")}</span>{["media","question","answer"].into_iter().map(|s|view!{<Show when=move ||s!="media" ||matches!(draft.get().kind.as_str(),"PImage"|"PVideo")><button type="button" class=BUTTON aria-pressed=move ||view_step.get()==s on:click=move |_|view_step.set(s)>{t.get(&format!("qt_step_{s}"))}</button></Show>}).collect_view()}</div>
        <label class="block text-xs font-bold uppercase text-gray-400">{move ||format!("{}{}", t.get("qe_text"), if matches!(draft.get().kind.as_str(),"PImage"|"PVideo"|"Handout") { format!(" · {}",t.get("qe_optional")) } else { String::new() })}<textarea rows="3" class=INPUT prop:value=move ||draft.get().text on:input=move |e|{draft.update(|d|d.text=event_target_value(&e));dirty.set(true);}/></label>
        <Show when=move ||draft.get().kind=="ABCD"><div class="space-y-2">{["A","B","C","D"].into_iter().map(|k|view!{<div class="flex gap-2 items-end"><button type="button" class=move ||if draft.get().answer.trim().eq_ignore_ascii_case(k){"p-3 bg-green-500 text-black rounded-lg"}else{"p-3 bg-white/10 rounded-lg"} title=t.get("qe_correct") on:click=move |_|{draft.update(|d|d.answer=k.into());dirty.set(true);}>{k}</button><FieldInput draft=draft dirty=dirty busy=busy field=Field::Option(k) label=k.to_string() names=names id=id.get_value()/></div>}).collect_view()}<p class="text-xs text-gray-500">{t.get("qe_correct_hint")}</p></div></Show>
        <Show when=move ||draft.get().kind=="Yes/No"><div class="flex gap-2">{["Yes","No"].into_iter().map(|v|view!{<button type="button" class=move ||if draft.get().answer.trim().eq_ignore_ascii_case(v){"px-5 py-2 rounded-lg bg-green-500 text-black"}else{"px-5 py-2 rounded-lg bg-white/10"} on:click=move |_|{draft.update(|d|d.answer=v.into());dirty.set(true);}>{t.get(if v=="Yes"{"qe_yes"}else{"qe_no"})}</button>}).collect_view()}</div></Show>
        <Show when=move ||draft.get().kind=="Top5"><div class="space-y-2"><p>{t.get("qe_answers_hint")}</p><For each=move ||0..draft.get().answers.len() key=|i|*i children=move |i|view!{<div class="flex gap-2 items-end"><FieldInput draft=draft dirty=dirty busy=busy field=Field::Top(i) label=format!("{}.",i+1) names=names id=id.get_value()/><button type="button" class=BUTTON disabled=move ||draft.get().answers.len()<=1 on:click=move |_|{draft.update(|d|{d.answers.remove(i);});dirty.set(true);}>{t.get("qe_remove")}</button></div>}/><button type="button" class=BUTTON disabled=move ||draft.get().answers.len()>=15 on:click=move |_|{draft.update(|d|d.answers.push(String::new()));dirty.set(true);}>{t.get("qe_add_answer")}</button></div></Show>
        <Show when=move ||draft.get().kind=="Sort"><div class="space-y-2"><p>{t.get("qe_items")}</p><p class="text-xs text-gray-500">{t.get("qe_items_hint")}</p><For each=move ||0..draft.get().items.len() key=|i|*i children=move |i|view!{<div class="rounded-xl border border-white/10 p-3 space-y-2"><p class="text-yellow-400">{format!("{}. ",i+1)}{move ||draft.get().items.get(i).map(|v|v.letter.clone())}</p><FieldInput draft=draft dirty=dirty busy=busy field=Field::Item(i) label=t.get("qe_item") names=names id=id.get_value()/><FieldInput draft=draft dirty=dirty busy=busy field=Field::Picture(i) label=t.get("qe_item_media") media=true names=names id=id.get_value()/><button type="button" class=BUTTON disabled=move ||draft.get().items.len()<=2 on:click=move |_|{draft.update(|d|{d.items.remove(i);d.items=shuffled(&d.items);});dirty.set(true);}>{t.get("qe_remove")}</button></div>}/><button type="button" class=BUTTON disabled=move ||draft.get().items.len()>=quiz_core::quiz::sort::MAX_SORT_ITEMS on:click=move |_|{draft.update(|d|{d.items.push(DraftItem{text:String::new(),media:String::new(),letter:String::new()});d.items=shuffled(&d.items);});dirty.set(true);}>{t.get("qe_add_item")}</button><button type="button" class=BUTTON on:click=move |_|{draft.update(|d|d.items=shuffled(&d.items));dirty.set(true);}>{t.get("qe_shuffle")}</button><p class="text-xs text-gray-500">{t.get("qe_letters_hint")}</p></div></Show>
        <Show when=move ||draft.get().kind=="Handout"><div class="space-y-3"><FieldInput draft=draft dirty=dirty busy=busy field=Field::Media label=t.get("qe_handout_picture") media=true names=names id=id.get_value()/><FieldInput draft=draft dirty=dirty busy=busy field=Field::Answer label=t.get("qe_handout_answer") media=true names=names id=id.get_value()/></div></Show>
        <Show when=move ||!matches!(draft.get().kind.as_str(),"ABCD"|"Yes/No"|"Top5"|"Sort"|"Handout")><FieldInput draft=draft dirty=dirty busy=busy field=Field::Answer label=t.get("qe_answer") names=names id=id.get_value()/></Show>
        <Show when=move ||!matches!(draft.get().kind.as_str(),"Sort"|"Handout")><FieldInput draft=draft dirty=dirty busy=busy field=Field::Media label=t.get("qe_media") media=true names=names id=id.get_value()/></Show>
    </fieldset>}
}
#[cfg(test)]
mod tests {
    use super::*;
    fn meta() -> Meta {
        Meta {
            author: "A".into(),
            changed: "2026-11-13 12:00".into(),
            headers: None,
        }
    }
    #[test]
    fn csv_round_trip() {
        let rows = vec![vec![
            " a ".into(),
            "quote\"comma,new\nline".into(),
            "".into(),
        ]];
        assert_eq!(csv_rows(&csv_text(&rows)), rows);
        assert_eq!(
            csv_rows("a,b\r\nc,d\n"),
            vec![vec!["a", "b"], vec!["c", "d"]]
        );
    }
    #[test]
    fn stale_selection() {
        let csv = examples::example_questions("en", "Examples").csv;
        let s = sheet(&csv);
        let sel = Selection::Edit {
            ri: 0,
            qi: 0,
            rows: s.rounds[0].questions[0].rows.clone(),
        };
        let mut d = empty_draft("Written");
        d.text = "Replacement".into();
        let p = make_plan(&s, &sel, &d, "", &meta(), &Action::Save).unwrap();
        assert!(
            make_plan(
                &sheet(&apply(&csv, &p)),
                &sel,
                &d,
                "",
                &meta(),
                &Action::Save
            )
            .is_none()
        );
    }
    #[test]
    fn folders() {
        assert_eq!(upload_path("round/cat.png", "dog.png"), "round/dog.png");
        assert_eq!(upload_path("cat.png", "dog.png"), "dog.png");
    }
    #[test]
    fn edits_preserve_unmodeled_columns_and_relocated_stamps() {
        let csv = "Round,Q#,Question,Type,A,B,C,D,Answer,Media,Notes,Author,Changed,Extra\r\nKolo 1: Test,1,Old,Written,,,,,Answer,,Keep,Someone,2026-01-01,Untouched";
        let original = sheet(csv);
        let mut draft = draft_from(&original.rounds[0].questions[0].question);
        draft.text = "New".into();
        let plan = plan_save(&original, 0, 0, &draft, &meta()).unwrap();
        let saved = apply(csv, &plan);
        let rows = csv_rows(&saved);
        assert_eq!(rows[1][10], "Keep");
        assert_eq!(rows[1][11], "A");
        assert_eq!(rows[1][12], "2026-11-13 12:00");
        assert_eq!(rows[1][13], "Untouched");
        assert_eq!(sheet(&saved).rounds[0].questions[0].question.text, "New");
    }
    #[test]
    fn plans_apply_multirow_questions_moves_and_deletes() {
        let csv = examples::example_questions("en", "Examples").csv;
        let original = sheet(&csv);
        let mut draft = empty_draft("Top5");
        draft.text = "Three answers".into();
        draft.answers = vec!["One".into(), "Two".into(), "Three".into()];
        let added = apply(&csv, &plan_add(&original, 0, &draft, &meta()).unwrap());
        let data = sheet(&added);
        let last = data.rounds[0].questions.len() - 1;
        assert_eq!(data.rounds[0].questions[last].rows.len(), 3);
        let moved = apply(
            &added,
            &plan_move(&data, 0, last, MoveDirection::Up, &meta()).unwrap(),
        );
        let data = sheet(&moved);
        assert_eq!(
            data.rounds[0].questions[last - 1].question.text,
            "Three answers"
        );
        let deleted = apply(&moved, &plan_delete(&data, 0, last - 1, &meta()).unwrap());
        assert_eq!(
            sheet(&deleted).rounds[0].questions.len(),
            original.rounds[0].questions.len()
        );
        assert_eq!(
            sheet(&deleted).rounds[0]
                .questions
                .iter()
                .map(|b| b.question.text.clone())
                .collect::<Vec<_>>(),
            original.rounds[0]
                .questions
                .iter()
                .map(|b| b.question.text.clone())
                .collect::<Vec<_>>()
        );
    }
    #[test]
    fn new_round_from_blank_and_rename_keep_first_question() {
        let mut draft = empty_draft("Written");
        draft.text = "First question".into();
        let csv = apply("", &plan_add_round(&sheet(""), "Test", &draft, &meta()));
        let data = sheet(&csv);
        assert_eq!(data.rounds[0].title, "Test");
        let csv = apply(&csv, &plan_rename(&data, 0, "Renamed", &meta()).unwrap());
        assert_eq!(sheet(&csv).rounds[0].title, "Renamed");
        assert_eq!(
            sheet(&csv).rounds[0].questions[0].question.text,
            "First question"
        );
    }
    #[test]
    fn csv_accepts_bom_escaped_quotes_and_empty_final_cell() {
        assert_eq!(
            csv_rows("\u{feff}a,\"two\"\"quotes\",\r\n"),
            vec![vec!["a", "two\"quotes", ""]]
        );
        assert_eq!(csv_rows(""), Vec::<Vec<String>>::new());
        assert_eq!(csv_rows("a,"), vec![vec!["a", ""]]);
    }
    #[test]
    fn preview_uses_selected_round_and_continuation_row_number() {
        let s = sheet(&examples::example_questions("en", "Examples").csv);
        let block = &s.rounds[0].questions[2];
        let selection = Selection::Edit {
            ri: 0,
            qi: 2,
            rows: block.rows.clone(),
        };
        let d = draft_from(&block.question);
        let preview = draft_round(&s, Some(&selection), &d, "Ignored");
        assert_eq!(preview.title, s.rounds[0].title);
        assert_eq!(preview.questions[0].id, block.question.id);
        let selection = Selection::Add(s.rounds[0].marker.clone());
        let preview = draft_round(&s, Some(&selection), &d, "Ignored");
        let expected = s.rounds[0].questions.last().unwrap().rows.last().unwrap()[1]
            .parse::<f64>()
            .unwrap()
            + 1.0;
        assert_eq!(preview.questions[0].id, expected.to_string());
        assert_eq!(
            draft_round(&s, Some(&Selection::Round), &d, "New round").title,
            "New round"
        );
    }
}
