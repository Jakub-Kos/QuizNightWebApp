//! Quiz validation and fixed TV frames, shared with the show map.
use crate::{
    i18n::{Lang, T, provide_translations, use_t},
    quiz_context::{QuizContext, load_stored},
    router::Route,
    scenes::game::QuestionPreview,
    theme::ThemeScope,
};
use leptos::prelude::*;
use quiz_core::quiz::{
    parse::is_round_hidden,
    validate::{Issue, Level, ValidationInput, ValidationTeam, preview_step, validate_quiz},
};

#[component]
pub(crate) fn Icon(name: &'static str) -> impl IntoView {
    let path = match name {
        "back" => "m12 19-7-7 7-7m-7 7h14",
        "next" => "m12 5 7 7-7 7M5 12h14",
        "close" => "m6 6 12 12M6 18 18 6",
        "play" => "m6 3 14 9-14 9z",
        "map" => "M10 3h4v4h-4zM3 17h4v4H3zM17 17h4v4h-4zM12 7v6M5 17v-4h14v4",
        "check" => "m9 12 2 2 4-4M22 12a10 10 0 1 1-5-8.66",
        "error" => "M12 8v4m0 4h.01M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0",
        "warning" => {
            "m10.3 3.9-8.2 14.2A2 2 0 0 0 3.8 21h16.4a2 2 0 0 0 1.7-2.9L13.7 3.9a2 2 0 0 0-3.4 0ZM12 9v4m0 4h.01"
        }
        _ => "M12 8h.01M12 12v4M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0",
    };
    view! { <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" class="inline-block shrink-0"><path d=path/></svg> }
}

#[component]
pub fn Page(id: String) -> impl IntoView {
    let store = crate::store::use_store();
    let loaded = LocalResource::new(move || load_stored(store.clone(), id.clone()));
    view! { {move || loaded.get().map(|result| match result {
        Ok(context) => view! { <Check context/> }.into_any(),
        Err(message) => view! { <LoadError message/> }.into_any(),
    })} }
}

#[component]
pub(crate) fn LoadError(message: String) -> impl IntoView {
    let t = use_t();
    view! { <main class="min-h-screen bg-[#050505] text-white flex flex-col items-center justify-center gap-6">
        <p>{if message == "Quiz not found" {t.get("ed_not_found")} else {message}}</p><a href=Route::Library.hash() class="px-6 py-3 rounded-xl bg-yellow-500 text-black font-bold">{t.get("settings_exit")}</a>
    </main> }
}

/// Owns the media lease and confines the quiz's language/theme to the TV frame.
#[component]
pub(crate) fn Frame(
    context: QuizContext,
    ri: usize,
    qi: Option<usize>,
    step: usize,
) -> impl IntoView {
    view! { <PreviewScope context=context.clone()>
        <QuestionFrame context ri qi step/>
    </PreviewScope> }
}

#[component]
fn QuestionFrame(context: QuizContext, ri: usize, qi: Option<usize>, step: usize) -> impl IntoView {
    let round = context.rounds[ri].clone();
    let context = StoredValue::new(context);
    let resolve_media =
        Callback::new(move |name: String| context.with_value(|c| c.resolve_media(&name)));
    view! { <QuestionPreview round question_index=qi step resolve_media/> }
}

/// Isolate each frame's quiz, language, theme and media lease from the surrounding UI.
#[component]
pub(crate) fn PreviewScope(context: QuizContext, children: Children) -> impl IntoView {
    view! { <leptos::context::Provider value=context.clone()>
        <ScopeContents context>{children()}</ScopeContents>
    </leptos::context::Provider> }
}

#[component]
fn ScopeContents(context: QuizContext, children: Children) -> impl IntoView {
    provide_translations(Signal::stored(Lang::from_code(
        &context.quiz.settings.language,
    )));
    let custom = context
        .quiz
        .fields
        .get("customTheme")
        .and_then(|value| serde_json::from_value(value.clone()).ok());
    let theme = crate::settings::selected_theme(&context.quiz.settings, custom.as_ref());
    let context = StoredValue::new(context);
    let resolve_media =
        Callback::new(move |name: String| context.with_value(|c| c.resolve_media(&name)));
    // File fonts use the same stable family names as ThemeScope's CSS variables.
    let fonts = theme
        .fonts
        .values()
        .filter_map(|font| {
            if let quiz_core::theme::format::Font::File { file } = font {
                let family = quiz_core::theme::builtin::font_family(font);
                resolve_media.run(file.clone()).map(|url| {
                    format!(
                        "@font-face{{font-family:{family};src:url({});font-display:swap;}}",
                        serde_json::to_string(&url).unwrap_or_default()
                    )
                })
            } else {
                None
            }
        })
        .collect::<String>();
    view! { <ThemeScope theme=Signal::stored(theme)>
        <style>{fonts}</style>
        {children()}
    </ThemeScope> }
}

pub(crate) fn issue_message(t: T, issue: &Issue) -> String {
    let values: Vec<(String, String)> = issue
        .params
        .as_object()
        .into_iter()
        .flatten()
        .map(|(key, value)| {
            (
                key.clone(),
                value
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| value.to_string()),
            )
        })
        .collect();
    let vars: Vec<_> = values
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    t.fmt(&format!("chk_{}", issue.code), &vars)
}
fn level_rank(level: &Level) -> usize {
    match level {
        Level::Error => 0,
        Level::Warning => 1,
        Level::Info => 2,
    }
}
fn level_class(level: &Level) -> &'static str {
    match level {
        Level::Error => "text-red-300",
        Level::Warning => "text-yellow-300",
        Level::Info => "text-blue-300",
    }
}
fn location(context: &QuizContext, t: T, ri: isize, qi: isize) -> String {
    if ri < 0 {
        return t.get("chk_quiz");
    }
    let round = &context.rounds[ri as usize];
    let label = format!("{} {}", t.get("chk_round"), round.number);
    if qi < 0 {
        label
    } else {
        format!(
            "{label} · {} {}",
            t.get("chk_question"),
            round.questions[qi as usize].id
        )
    }
}
fn tiles(context: &QuizContext) -> Vec<(usize, Option<usize>)> {
    context
        .rounds
        .iter()
        .enumerate()
        .flat_map(|(ri, r)| {
            std::iter::once((ri, None)).chain((0..r.questions.len()).map(move |qi| (ri, Some(qi))))
        })
        .collect()
}

#[component]
fn ViewSwitch(answer: RwSignal<bool>) -> impl IntoView {
    let t = use_t();
    view! { <div class="flex gap-1 bg-white/5 rounded-xl p-1 text-sm">
        {[false,true].into_iter().map(|value| view! {
            <button on:click=move |_| answer.set(value) class=move || if answer.get() == value {
                "px-3 py-1.5 rounded-lg bg-yellow-500 text-black font-bold"
            } else { "px-3 py-1.5 rounded-lg text-gray-400 hover:text-white" }>
                {t.get(if value { "chk_view_answer" } else { "chk_view_question" })}
            </button>
        }).collect_view()}
    </div> }
}

#[component]
fn Check(context: QuizContext) -> impl IntoView {
    let t = use_t();
    let input = ValidationInput {
        teams: context
            .quiz
            .teams
            .iter()
            .map(|team| ValidationTeam {
                name: team.name.clone(),
                image: team.image.clone(),
            })
            .collect(),
        scores_sheet_url: Some(context.quiz.scores_sheet_url.clone()),
        builtin: context.quiz.builtin,
    };
    let checking = RwSignal::new(context.quiz.builtin);
    let issues = RwSignal::new(validate_quiz(&input, &context.rounds, |name| {
        context.resolve_media(name).is_some()
    }));
    // Demo media live on the server, rather than in IndexedDB. Verify them without downloading.
    if context.quiz.builtin {
        let context = context.clone();
        leptos::task::spawn_local(async move {
            let names = quiz_store::package::referenced_media(&context.quiz);
            let mut present = std::collections::HashSet::new();
            for name in names {
                if let Some(url) = context.resolve_media(&name)
                    && let Ok(response) = gloo_net::http::RequestBuilder::new(&url)
                        .method(gloo_net::http::Method::HEAD)
                        .send()
                        .await
                    && response.ok()
                    && !response
                        .headers()
                        .get("content-type")
                        .unwrap_or_default()
                        .contains("text/html")
                {
                    present.insert(name);
                }
            }
            let _ = issues.try_set(validate_quiz(&input, &context.rounds, |name| {
                present.contains(name)
            }));
            let _ = checking.try_set(false);
        });
    }
    let frames = StoredValue::new(tiles(&context));
    let context = StoredValue::new(context);
    let answer = RwSignal::new(false);
    let columns = RwSignal::new(4_usize);
    let open = RwSignal::new(None::<usize>);
    let go = move |delta: isize| {
        open.update(|at| {
            if let Some(index) = at {
                *index = index
                    .saturating_add_signed(delta)
                    .min(frames.with_value(Vec::len).saturating_sub(1));
            }
        })
    };
    let keys = window_event_listener(leptos::ev::keydown, move |event| {
        if open.get_untracked().is_none() {
            return;
        }
        match event.key().as_str() {
            "Escape" => open.set(None),
            "ArrowRight" => {
                event.prevent_default();
                go(1);
            }
            "ArrowLeft" => {
                event.prevent_default();
                go(-1);
            }
            " " => {
                event.prevent_default();
                answer.update(|v| *v = !*v);
            }
            _ => (),
        }
    });
    on_cleanup(move || keys.remove());
    let open_tile = move |ri: isize, qi: isize| {
        if ri >= 0 {
            open.set(frames.with_value(|all| {
                all.iter()
                    .position(|&(r, q)| r == ri as usize && q == usize::try_from(qi).ok())
            }));
        }
    };
    view! {
        <div class="check-page min-h-screen bg-[#050505] text-white font-sans">
            <header class="sticky top-0 z-30 bg-[#050505]/95 backdrop-blur border-b border-white/10">
                <div class="max-w-[1800px] mx-auto px-6 py-4 flex flex-wrap items-center gap-4">
                    <a href=context.with_value(|c| if c.quiz.builtin { Route::Library } else { Route::Edit(c.quiz.id.clone()) }.hash()) class="flex items-center gap-2 text-gray-400 hover:text-white"><Icon name="back"/> {context.with_value(|c| t.get(if c.quiz.builtin {"settings_exit"} else {"lib_edit"}))}</a>
                    <h1 class="font-['League_Spartan'] text-2xl font-black truncate">{t.get("chk_title")}": " {context.with_value(|c| if c.quiz.title.is_empty() {t.get("ed_untitled")} else {c.quiz.title.clone()})}</h1>
                    <div class="ml-auto flex flex-wrap items-center gap-3">
                        <ViewSwitch answer/>
                        <label class="flex items-center gap-2 text-sm text-gray-400">{t.get("chk_size")}
                            <input type="range" min="2" max="8" prop:value=move || 10-columns.get()
                                on:input=move |event| { if let Ok(value) = event_target_value(&event).parse::<usize>() { columns.set(10-value.clamp(2,8)); } }
                                class="w-24 accent-yellow-500"/>
                        </label>
                        <a href=context.with_value(|c| Route::Map(c.quiz.id.clone()).hash()) class="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 text-sm"><Icon name="map"/>{t.get("mp_open")}</a>
                        <a href=context.with_value(|c| Route::Show(c.quiz.id.clone()).hash()) class="flex items-center gap-2 px-4 py-2 rounded-xl bg-yellow-500 text-black font-bold text-sm"><Icon name="play"/>{t.get("lib_open")}</a>
                    </div>
                </div>
            </header>
            <main class="max-w-[1800px] mx-auto px-6 py-8 space-y-10">
                <section class="rounded-2xl bg-white/[0.04] border border-white/10 p-6 space-y-4">
                    <div class="flex flex-wrap items-center gap-4">
                        <h2 class="font-['League_Spartan'] text-xl font-bold uppercase tracking-widest">{t.get("chk_problems")}</h2>
                        {[(Level::Error,"chk_errors"),(Level::Warning,"chk_warnings"),(Level::Info,"chk_notes")].into_iter().map(|(level,key)| { let class = level_class(&level); view! {
                            <span class=class>{move || issues.with(|all| all.iter().filter(|i| i.level == level).count())}" " {t.get(key)}</span>
                        } }).collect_view()}
                    </div>
                    <Show when=move || checking.get()><p class="text-xs text-gray-500">{t.get("chk_checking_media")}</p></Show>
                    <Show when=move || issues.with(|all| all.iter().all(|i| i.level == Level::Info))>
                        <p class="flex items-center gap-2 text-green-300"><Icon name="check"/>{t.get("chk_all_good")}</p>
                    </Show>
                    <ul class="divide-y divide-white/5">{move || {
                        let mut sorted = issues.get();
                        sorted.sort_by_key(|i| level_rank(&i.level));
                        sorted.into_iter().map(move |issue| {
                            let text = issue_message(t,&issue);
                            view! { <li><button disabled=issue.round_index.lt(&0)
                                on:click=move |_| open_tile(issue.round_index,issue.q_index)
                                class="w-full flex items-start gap-3 py-2 text-left text-sm hover:bg-white/5">
                                <span class=level_class(&issue.level)><Icon name=match issue.level { Level::Error => "error", Level::Warning => "warning", Level::Info => "info" }/></span>
                                <span class="text-gray-500 w-40 shrink-0">{context.with_value(|c| location(c,t,issue.round_index,issue.q_index))}</span>
                                <span class="text-gray-200">{text}</span>
                            </button></li> }
                        }).collect_view()
                    }}</ul>
                </section>
                {context.with_value(|c| c.rounds.iter().enumerate().map(|(ri,round)| {
                    let round = round.clone();
                    view! { <section class="space-y-4">
                        <h2 class="flex flex-wrap items-center gap-3 text-2xl font-bold">
                            <span class="text-gray-500">{t.get("chk_round")}" "{round.number.clone()}</span>{round.title.clone()}
                            {is_round_hidden(&round).then(|| view! { <span class="text-xs px-2 py-1 rounded bg-white/10 text-gray-400">{t.get("chk_hidden_badge")}</span> })}
                        </h2>
                        <div class="grid gap-4" style=move || format!("grid-template-columns:repeat({},minmax(0,1fr))",columns.get())>
                            {frames.with_value(|all| all.iter().enumerate().filter(|(_, (r,_))| *r == ri).map(|(index,&(_,qi))| {
                                let caption = qi.map_or_else(|| t.get("chk_intro"), |q| format!("{} {} · {}{}",t.get("chk_question"),round.questions[q].id,crate::scenes::game::kind(&round.questions[q]),round.questions[q].media.as_ref().map(|m|format!(" · {m}")).unwrap_or_default()));
                                let kind = qi.map(|q| crate::scenes::game::kind(&round.questions[q])).unwrap_or_default();
                                view! { <div role="button" tabindex="0" class="group text-left space-y-1.5 cursor-pointer focus-visible:outline-yellow-400 rounded-lg" on:click=move |_| open.set(Some(index)) aria-haspopup="dialog"
                                    on:keydown=move |event| { if matches!(event.key().as_str(), "Enter" | " ") { event.prevent_default(); open.set(Some(index)); } }>
                                    <div class="check-thumbnail relative rounded-lg overflow-hidden" class:check-error=move || issues.with(|all| all.iter().any(|i| i.round_index == ri as isize && usize::try_from(i.q_index).ok() == qi && i.level == Level::Error))
                                        class:check-warning=move || issues.with(|all| all.iter().any(|i| i.round_index == ri as isize && usize::try_from(i.q_index).ok() == qi && i.level != Level::Error))>
                                        {move || view! { <Frame context=context.get_value() ri qi step=if qi.is_some() {preview_step(&kind,if answer.get() {"answer"} else {"question"})} else {0}/> }}
                                        <span class="check-badge absolute top-2 right-2 min-w-6 h-6 px-1.5 rounded-full text-xs font-bold flex items-center justify-center" class:hidden=move || issues.with(|all| !all.iter().any(|i| i.round_index == ri as isize && usize::try_from(i.q_index).ok() == qi))>{move || {
                                            let n = issues.with(|all| all.iter().filter(|i| i.round_index == ri as isize && usize::try_from(i.q_index).ok() == qi).count());
                                            (n > 0).then(|| n.to_string())
                                        }}</span>
                                    </div>
                                    <p class="text-xs text-gray-400 truncate">{caption}</p>
                                </div> }
                            }).collect_view())}
                        </div>
                    </section> }
                }).collect_view())}
            </main>
            {move || open.get().map(|index| {
                let (ri,qi) = frames.with_value(|all| all[index]);
                let kind = context.with_value(|c| qi.map(|q| crate::scenes::game::kind(&c.rounds[ri].questions[q])).unwrap_or_default());
                view! { <div role="dialog" aria-modal="true" aria-label=t.get("chk_title") class="fixed inset-0 z-50 bg-black/90 backdrop-blur-sm text-white flex flex-col items-center justify-center gap-4 p-4" on:click=move |_| open.set(None)>
                    <div class="check-modal w-full flex flex-col gap-3" on:click=|event| event.stop_propagation()>
                        <div class="flex flex-wrap items-center gap-3 text-sm">
                            <span>{context.with_value(|c| location(c,t,ri as isize,qi.map_or(-1,|i| i as isize)))}</span>
                            <span class="text-gray-500">{index+1}" / "{frames.with_value(Vec::len)}</span>
                            <div class="ml-auto"><ViewSwitch answer/></div>
                            <button autofocus aria-label=t.get("chk_close") on:click=move |_| open.set(None) class="p-2 rounded-full hover:bg-white/10"><Icon name="close"/></button>
                        </div>
                        <div class="relative">
                            {move || view! { <Frame context=context.get_value() ri qi step=if qi.is_some() {preview_step(&kind,if answer.get() {"answer"} else {"question"})} else {0}/> }}
                            <button aria-label=t.get("btn_prev") disabled=index.eq(&0) on:click=move |_| go(-1) class="absolute left-2 top-1/2 p-3 rounded-full bg-black/60 disabled:opacity-20"><Icon name="back"/></button>
                            <button aria-label=t.get("btn_next") disabled={index+1 == frames.with_value(Vec::len)} on:click=move |_| go(1) class="absolute right-2 top-1/2 p-3 rounded-full bg-black/60 disabled:opacity-20"><Icon name="next"/></button>
                        </div>
                        <ul class="space-y-1 text-sm">{move || issues.get().into_iter().filter(|i| i.round_index == ri as isize && usize::try_from(i.q_index).ok() == qi).map(|issue| view! { <li class=level_class(&issue.level)><Icon name=match issue.level { Level::Error => "error", Level::Warning => "warning", Level::Info => "info" }/>" "{issue_message(t,&issue)}</li> }).collect_view()}</ul>
                        <p class="text-xs text-gray-500">{t.get("chk_keys")}</p>
                    </div>
                </div> }
            })}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn severity_orders_errors_before_warnings_and_notes() {
        assert!(level_rank(&Level::Error) < level_rank(&Level::Warning));
        assert!(level_rank(&Level::Warning) < level_rank(&Level::Info));
    }
}
