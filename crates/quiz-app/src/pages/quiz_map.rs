//! The show graph: intro, dashboard hub, round passes, standings and half-time.
use super::quiz_check::{Icon, LoadError, PreviewScope};
use crate::scenes::game::{PreviewStage, QuestionPreviewMode};
use crate::{
    i18n::use_t,
    quiz_context::{QuizContext, load_stored},
    router::Route,
};
use leptos::prelude::*;
use quiz_core::quiz::{
    intro::{prizes_for, rules_for},
    parse::is_round_hidden,
    validate::preview_step,
};
use quiz_core::show::{PlayMode, Role, Scene, ShowState};
use wasm_bindgen::JsCast;

#[derive(Clone, Debug, PartialEq)]
struct Tile {
    label: String,
    frame: Option<(usize, Option<usize>, usize, bool)>,
    scene: Option<&'static str>,
}

/// The first two questions and the last one, without duplicating short rounds.
fn shown_questions(count: usize) -> Vec<usize> {
    let mut out: Vec<_> = (0..count.min(2)).collect();
    if count > 2 {
        out.push(count - 1);
    }
    out
}

#[component]
pub fn Page(id: String) -> impl IntoView {
    let store = crate::store::use_store();
    let loaded = LocalResource::new(move || load_stored(store.clone(), id.clone()));
    view! { {move || loaded.get().map(|result| match result {
        Ok(context) => view! { <Map context/> }.into_any(),
        Err(message) => view! { <LoadError message/> }.into_any(),
    })} }
}

#[component]
fn TileFrame(tile: Tile, context: QuizContext) -> impl IntoView {
    view! { <PreviewScope context=context.clone()>
        {if let Some((ri, qi, step, answers)) = tile.frame {
            let round = context.rounds[ri].clone();
            let context = StoredValue::new(context.clone());
            view! { <QuestionPreviewMode round question_index=qi step
                mode=if answers { PlayMode::WithAnswers } else { PlayMode::QuestionsOnly }
                resolve_media=Callback::new(move |name: String| context.with_value(|c| c.resolve_media(&name)))/> }.into_any()
        } else {
            view! { <PreviewStage><StaticScene scene=tile.scene.unwrap_or("mp_dashboard")/></PreviewStage> }.into_any()
        }}
    </PreviewScope> }
}

/// Mounts audience previews without the live scenes’ keys, score polling or channels.
#[component]
fn StaticScene(scene: &'static str) -> impl IntoView {
    provide_context(crate::show::ShowContext {
        role: Role::Main,
        state: RwSignal::new(ShowState {
            scene: Scene::Dashboard,
            active_round_id: None,
            play_mode: PlayMode::WithAnswers,
        }),
        dispatch: Callback::new(|_| ()),
    });
    let context = crate::quiz_context::use_quiz();
    let scores = crate::scores::use_scores().scores;
    let theme = crate::theme::use_theme();
    let teams = context.teams.as_ref().clone();
    match scene {
        "mp_rules" => view! { <crate::scenes::rules::Rules/> }.into_any(),
        "mp_prizes" => view! { <crate::scenes::prizes::Prizes/> }.into_any(),
        "mp_dashboard" => view! { <crate::scenes::dashboard::Dashboard/> }.into_any(),
        "mp_welcome" => view! { <crate::scenes::welcome::WelcomePreview/> }.into_any(),
        "mp_pause" => view! { {move || view! { <crate::scenes::pause::PausePreview scores=scores.get() teams=teams.clone()/> }} }.into_any(),
        _ => {
            let step = match scene {
                "mp_lb_card" => 1,
                "mp_lb_added" => 2,
                "mp_lb_final" => usize::MAX,
                _ => 0,
            };
            view! { {move || view! { <crate::scenes::leaderboard::LeaderboardPreview
                scores=scores.get() teams=teams.clone() step layout=theme.get().scenes.leaderboard.as_ref().map_or(quiz_core::theme::format::LeaderboardLayout::Table, |s| s.layout)/> }} }.into_any()
        }
    }
}

#[component]
fn Thumb(
    tile: Tile,
    context: StoredValue<QuizContext>,
    open: RwSignal<Option<Tile>>,
) -> impl IntoView {
    let selected = tile.clone();
    let keyboard_tile = tile.clone();
    view! { <div role="button" tabindex="0" on:keydown=move |event| { if matches!(event.key().as_str(), "Enter" | " ") { event.prevent_default(); open.set(Some(keyboard_tile.clone())); } } class="group shrink-0 w-56 text-left cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-yellow-400 rounded-lg"
        aria-haspopup="dialog" on:click=move |_| open.set(Some(selected.clone()))>
        <div class="rounded-lg ring-1 ring-white/15 group-hover:ring-2 group-hover:ring-yellow-400 overflow-hidden">
            <TileFrame tile=tile.clone() context=context.get_value()/>
        </div>
        <span class="block text-xs text-gray-300 mt-1.5 truncate group-hover:text-yellow-300">{tile.label}</span>
    </div> }
}

fn rich_keys(text: String) -> impl IntoView {
    text.split('`').enumerate().map(|(index, part)| {
        if index % 2 == 1 {
            view! { <code class="rounded bg-white/10 px-1 font-mono text-white">{part.to_owned()}</code> }.into_any()
        } else { part.to_owned().into_any() }
    }).collect_view()
}

#[component]
fn Edge(text: String) -> impl IntoView {
    view! { <div class="shrink-0 h-[126px] flex flex-col justify-center w-32 px-2">
        <span class="text-[11px] leading-tight text-gray-300 text-center mb-1.5">{rich_keys(text)}</span>
        <div class="relative h-px bg-white/30"><span class="absolute -right-1.5 -top-3 text-white/50">"→"</span></div>
    </div> }
}

#[component]
fn ReturnToHub() -> impl IntoView {
    let t = use_t();
    view! { <div class="shrink-0 h-[126px] flex items-center">
        <span class="px-3 py-2 rounded-full border border-yellow-500/40 bg-yellow-500/10 text-yellow-200 text-xs font-bold">"↵ "{t.get("mp_dashboard")}</span>
    </div> }
}

#[component]
fn Pass(
    ri: usize,
    answers: bool,
    context: StoredValue<QuizContext>,
    open: RwSignal<Option<Tile>>,
) -> impl IntoView {
    let t = use_t();
    let round = context.with_value(|c| c.rounds[ri].clone());
    let count = round.questions.len();
    let title = Tile {
        label: format!("{} · {}", round.title, t.get("mp_round_card")),
        frame: Some((ri, None, 0, answers)),
        scene: None,
    };
    view! {
        <Thumb tile=title context open/>
        {shown_questions(count).into_iter().map(|qi| {
            let q = &round.questions[qi];
            let kind = crate::scenes::game::kind(q);
            let question = Tile { label: format!("{} · {}", round.title,t.fmt("mp_question",&[("n",&q.id)])), scene: None, frame: Some((ri,Some(qi),preview_step(&kind,"question"),answers)) };
            let answer = Tile { label: format!("{} · {}", round.title,t.fmt("mp_answer",&[("n",&q.id)])), scene: None, frame: Some((ri,Some(qi),preview_step(&kind,"answer"),answers)) };
            view! {
                {(qi > 1 && count > 3).then(|| view! { <div class="shrink-0 h-[126px] flex items-center px-3 text-xs text-gray-500">{t.fmt("mp_more",&[("n",&(count-3).to_string())])}</div> })}
                <Edge text=t.get("mp_edge_next")/><Thumb tile=question context open/>
                {answers.then(|| view! { <Edge text=t.get("mp_edge_answer")/><Thumb tile=answer context open/> })}
            }
        }).collect_view()}
        <Edge text=t.get("mp_edge_last")/><ReturnToHub/>
    }
}

#[component]
fn Map(context: QuizContext) -> impl IntoView {
    let t = use_t();
    crate::scores::provide_scores(context.clone());
    let defaults = (1..=4)
        .map(|n| t.get(&format!("rule_{n}")))
        .collect::<Vec<_>>();
    let rules = rules_for(&context.quiz, &defaults).is_some();
    let prizes = prizes_for(&context.quiz).is_some();
    let context = StoredValue::new(context);
    let open = RwSignal::new(None::<Tile>);
    let launching = RwSignal::new(false);
    let key = window_event_listener(leptos::ev::keydown, move |event| {
        if event.key() == "Escape" {
            open.set(None);
            launching.set(false);
        }
    });
    on_cleanup(move || key.remove());
    let tile = move |key: &'static str| Tile {
        label: t.get(key),
        frame: None,
        scene: Some(key),
    };
    let click = move |key: &str| t.fmt("mp_click", &[("button", &t.get(key))]);
    view! {
        <div class="map-page min-h-screen bg-[#050505] text-white font-sans">
            <header class="sticky top-0 z-30 bg-[#050505]/95 backdrop-blur border-b border-white/10">
                <div class="max-w-[1800px] mx-auto px-6 py-4 flex flex-wrap items-center gap-4">
                    <a href=context.with_value(|c| if c.quiz.builtin {Route::Library} else {Route::Edit(c.quiz.id.clone())}.hash()) class="flex items-center gap-2 text-gray-400 hover:text-white"><Icon name="back"/> {context.with_value(|c| t.get(if c.quiz.builtin {"settings_exit"} else {"lib_edit"}))}</a>
                    <h1 class="font-['League_Spartan'] text-2xl font-black truncate">{t.get("mp_title")}": "{context.with_value(|c| if c.quiz.title.is_empty() {t.get("ed_untitled")} else {c.quiz.title.clone()})}</h1>
                    <div class="ml-auto flex flex-wrap items-center gap-3">
                        <a href=context.with_value(|c| Route::Check(c.quiz.id.clone()).hash()) class="px-4 py-2 rounded-xl bg-white/10 text-sm">{t.get("chk_open")}</a>
                        <button on:click=move |_| launching.set(true) class="flex items-center gap-2 px-4 py-2 rounded-xl bg-yellow-500 text-black font-bold text-sm"><Icon name="play"/>{t.get("lib_open")}</button>
                    </div>
                </div>
            </header>
            <main class="max-w-[1800px] mx-auto px-6 py-8 space-y-10">
                <p class="text-gray-300 max-w-3xl">{t.get("mp_intro")}</p>
                <section class="space-y-3">
                    <h2 class="text-xl font-bold uppercase tracking-widest text-gray-400">{t.get("mp_before")}</h2>
                    <div class="flex items-start overflow-x-auto pb-3">
                        <Thumb tile=tile("mp_welcome") context open/><Edge text=click("start_show")/>
                        {rules.then(|| view! { <Thumb tile=tile("mp_rules") context open/><Edge text=click("continue")/> })}
                        {prizes.then(|| view! { <Thumb tile=tile("mp_prizes") context open/><Edge text=click("continue_dash")/> })}
                        <Thumb tile=tile("mp_dashboard") context open/>
                    </div>
                </section>
                <section class="space-y-4">
                    <h2 class="text-xl font-bold uppercase tracking-widest text-gray-400">{t.get("mp_hub")}</h2>
                    <p class="text-sm text-gray-400">{t.get("mp_hub_text")}</p>
                    <div class="grid lg:grid-cols-[260px_1fr] gap-6">
                        <aside class="space-y-4 lg:sticky lg:top-24 lg:self-start">
                            <Thumb tile=tile("mp_dashboard") context open/>
                            <div class="rounded-xl border border-white/10 bg-white/[0.03] p-4 space-y-2 text-sm text-gray-300">
                                <h3 class="font-bold text-white">{t.get("mp_keys_title")}</h3>
                                {["mp_key_d","mp_key_esc","mp_key_z","mp_key_p","mp_key_c"].into_iter().map(|key| view! {<p>{rich_keys(t.get(key))}</p>}).collect_view()}
                            </div>
                        </aside>
                        <div class="min-w-0 map-trunk">
                            {context.with_value(|c| c.rounds.iter().enumerate().filter(|(_,r)| !is_round_hidden(r)).map(|(ri,r)| {
                                view! { <section class="map-branch space-y-3 pb-8">
                                    <h3 class="text-sm font-bold text-white/80 pl-4">{t.get("round")}" "{r.number.clone()}" · "{r.title.clone()}</h3>
                                    {[false,true].into_iter().map(|answers| view! {
                                        <div><p class="text-xs text-gray-500 pl-4 mb-2">{t.get(if answers {"mp_round_a"} else {"mp_round_q"})}</p>
                                            <div class="flex items-start overflow-x-auto pb-3">
                                                <Edge text=click(if answers {"btn_answers"} else {"btn_questions"})/>
                                                <Pass ri answers context open/>
                                            </div>
                                        </div>
                                    }).collect_view()}
                                </section> }
                            }).collect_view())}
                            <section class="map-branch space-y-3 pb-8">
                                <h3 class="text-sm font-bold text-white/80 pl-4">{t.get("mp_leaderboard")}</h3>
                                <div class="flex items-start overflow-x-auto pb-3">
                                    <Edge text=click("leaderboard_btn")/>
                                    <Thumb tile=tile("mp_lb_table") context open/><Edge text=t.get("mp_edge_group")/>
                                    <Thumb tile=tile("mp_lb_card") context open/><Edge text=t.get("mp_edge_points")/>
                                    <Thumb tile=tile("mp_lb_added") context open/>
                                    <span class="shrink-0 h-[126px] flex items-center px-3 text-gray-500">"…"</span>
                                    <Edge text=t.get("mp_edge_shuffle")/><Thumb tile=tile("mp_lb_final") context open/>
                                    <Edge text=t.get("mp_back_lb")/><ReturnToHub/>
                                </div>
                            </section>
                            <section class="map-branch space-y-3">
                                <h3 class="text-sm font-bold text-white/80 pl-4">{t.get("mp_pause")}</h3>
                                <div class="flex items-start overflow-x-auto pb-3">
                                    <Edge text=click("half_time")/><Thumb tile=tile("mp_pause") context open/>
                                    <Edge text=t.get("mp_back_pause")/><ReturnToHub/>
                                </div>
                            </section>
                        </div>
                    </div>
                </section>
            </main>
            {move || open.get().map(|selected| view! {
                <div role="dialog" aria-modal="true" aria-label=selected.label.clone() class="fixed inset-0 z-50 bg-black/90 backdrop-blur-sm text-white flex items-center justify-center p-4" on:click=move |_| open.set(None)>
                    <div class="check-modal w-full space-y-3" on:click=|event| event.stop_propagation()>
                        <div class="flex items-center justify-between"><h2>{selected.label.clone()}</h2>
                            <button autofocus aria-label=t.get("chk_close") class="p-2 rounded-full hover:bg-white/10" on:click=move |_| open.set(None)><Icon name="close"/></button>
                        </div>
                        <TileFrame tile=selected.clone() context=context.get_value()/>
                    </div>
                </div>
            })}
        </div>
        <Show when=move || launching.get()><Launcher id=context.with_value(|c| c.quiz.id.clone()) close=Callback::new(move |_| launching.set(false))/></Show>
    }
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
            <button class="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm" on:click=move |_| close.run(())>{move || t.get("chk_close")}</button>
            <h2 class="text-3xl font-black uppercase text-yellow-400">{move || t.get("sl_title")}</h2>
            <p class="text-gray-300">{move || t.get("sl_intro")}</p>
            <h3 class="font-bold text-lg">{move || t.get("sl_tv_title")}</h3>
            <p>{move || t.get(if tv.get() == Some(false) { "sl_tv_blocked" } else if tv.get() == Some(true) { "sl_tv_opened" } else { "sl_tv_text" })}</p>
            <button class="px-5 py-3 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold" on:click=move |_| open(false)>{move || t.get("sl_tv_open")}</button>
            <h3 class="font-bold text-lg">{move || t.get("sl_presenter_title")}</h3><p>{move || t.get("sl_presenter_text")}</p>
            <button class="px-5 py-3 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold" on:click=move |_| open(true)>{move || t.get("sl_presenter_open")}</button>
            <Show when=move || host.get() == Some(false)><p role="alert">{move || t.get("sl_tv_blocked")}</p></Show>
            <Show when=move || id.get_value() == "demo"><button class="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm" on:click=move |_| crate::tour::start_tour("demo")>{move || t.get("tr_start")}</button></Show>
            <a class="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm" href=Route::Show(id.get_value()).hash()>{move || t.get("sl_one_screen")}</a>
        </section>
    </div> }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn short_passes_never_duplicate_questions() {
        assert!(shown_questions(0).is_empty());
        assert_eq!(shown_questions(1), [0]);
        assert_eq!(shown_questions(2), [0, 1]);
        assert_eq!(shown_questions(3), [0, 1, 2]);
        assert_eq!(shown_questions(10), [0, 1, 9]);
    }
}
