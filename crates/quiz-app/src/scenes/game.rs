//! QuestionScreen.jsx: audience screen and synchronized presenter dashboard.
//! The steps through a round are `game_steps::Steps`; Sort questions are `sort_items::SortItems`.
//! Framer Motion's animations are CSS keyframes and transitions in styles/game.css, driven by the
//! springs in `SPRINGS`.
pub(crate) use super::game_steps::kind;
use super::game_steps::{QuestionFrame, QuestionMessage, Steps};
use super::icons::GameIcon;
use super::round_intro::RoundIntro;
use super::sort_items::SortItems;
use crate::{
    i18n::{T, use_t},
    presence::Presence,
    quiz_context::use_quiz,
    show::use_show,
    spring::Spring,
};
use leptos::prelude::*;
use quiz_core::{
    quiz::{parse::Question, sort::sort_layout},
    show::{Action, PlayMode},
};
use std::{sync::LazyLock, time::Duration};
use wasm_bindgen::{JsCast, closure::Closure};

/// Framer Motion's springs as CSS custom properties for styles/game.css: `move` and `scale` are the
/// springs it gives x/y and scale when a transition names none (stiffness 500, damping 25 and 550,
/// 30), `number` is the big number's `{ type: "spring", duration: 0.8, bounce: 0.2 }`, `answer` the
/// answer card's `{ type: "spring", bounce: 0.4 }`.
static SPRINGS: LazyLock<String> = LazyLock::new(|| {
    [
        ("move", Spring::new(500.0, 25.0, 1.0)),
        ("scale", Spring::new(550.0, 30.0, 1.0)),
        ("number", Spring::from_duration(0.2, 0.8)),
        ("answer", Spring::from_duration(0.4, 0.8)),
    ]
    .iter()
    .map(|(name, spring)| {
        format!(
            "--spring-{name}:{};--spring-{name}-time:{:.3}s;",
            spring.to_css_linear(),
            spring.settle_time()
        )
    })
    .collect()
});

/// A question with its media resolved (processedQuestions in QuestionScreen.jsx).
#[derive(Clone)]
struct Prepared {
    question: Question,
    kind: String,
    source: Option<String>,
    /// Handout: the answer column holds the answer picture.
    answer_source: Option<String>,
}
impl Prepared {
    /// A picture that Z enlarges (hasImage): any media but video and audio.
    fn has_image(&self) -> bool {
        self.source.is_some() && !self.kind.contains("Video") && self.kind != "Audio"
    }
    fn is_pre_media(&self) -> bool {
        matches!(self.kind.as_str(), "PImage" | "PVideo")
    }
}

/// `String(question.answer ?? "")` for the types with one answer.
fn answer_text(q: &Question) -> String {
    match &serde_json::to_value(q).unwrap_or_default()["answer"] {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// getFontSize: the question's size by its length (in UTF-16 units, as JS counts).
fn font_size(text: &str) -> &'static str {
    match text.encode_utf16().count() {
        0 => "text-5xl",
        n if n < 30 => "text-7xl",
        n if n < 80 => "text-6xl",
        _ => "text-5xl",
    }
}

/// keepTogether: spaces that must not break a line, inside numbers ("2 166 086"), between a number
/// and a short unit ("217 291 km²", "1,5 l", "40 %") and after an ordinal ("20. století").
fn keep_together(text: &str) -> String {
    let c: Vec<char> = text.chars().collect();
    let digit = |i: usize| c.get(i).is_some_and(|ch| ch.is_ascii_digit());
    let letter = |i: usize| c.get(i).is_some_and(|ch| ch.is_alphabetic());
    (0..c.len())
        .map(|i| {
            if c[i] != ' ' || i == 0 {
                return c[i];
            }
            let after_digit = digit(i - 1);
            let thousands = after_digit && (1..=3).all(|k| digit(i + k)) && !digit(i + 4);
            let sign = matches!(c.get(i + 1), Some('%' | '‰' | '°')) && !letter(i + 2);
            let word = (1..=3).any(|len| (1..=len).all(|k| letter(i + k)) && !letter(i + len + 1));
            let ordinal = i >= 2 && c[i - 1] == '.' && digit(i - 2) && letter(i + 1);
            if thousands || (after_digit && (sign || word)) || ordinal {
                '\u{a0}'
            } else {
                ' '
            }
        })
        .collect()
}

/// splitAnswer: "Viktoriin ostrov (217 291 km²)" -> the name, and the bracketed detail shown on a
/// smaller line below.
fn split_answer(answer: &str) -> (String, Option<String>) {
    let text = answer.trim();
    if let Some((main, detail)) = text.strip_suffix(')').and_then(|s| s.rsplit_once('(')) {
        let main = main.trim_end();
        if !main.is_empty() && !detail.is_empty() && !detail.contains(')') {
            return (keep_together(main), Some(keep_together(detail)));
        }
    }
    (keep_together(text), None)
}

/// A top list of more than five answers is split into two columns, of more than ten into three.
fn top_columns(count: usize) -> usize {
    match count {
        0..=5 => 1,
        6..=10 => 2,
        _ => 3,
    }
}

#[component]
pub fn Game() -> impl IntoView {
    let show = use_show();
    let quiz = use_quiz();
    let t = use_t();
    let presenter = show.role == quiz_core::show::Role::Presenter;
    let state = show.state.get_untracked();
    let mode = state.play_mode;
    let Some(round) = quiz
        .rounds
        .iter()
        .find(|r| Some(&r.id) == state.active_round_id.as_ref())
        .cloned()
    else {
        return view! { <p>"No active round"</p> }.into_any();
    };
    let resolver_quiz = quiz.clone();
    let prepared = prepare(&round, move |name| resolver_quiz.resolve_media(name));
    let label = format!("{} {}", t.get("round"), round.number);
    let title = round.title.clone();
    let round = StoredValue::new(round);
    let prepared = StoredValue::new(prepared);
    let position = RwSignal::new(Steps::default());
    // QuestionScreen.jsx: the round cue with the title card, the answer cue at the reveal.
    let heard = StoredValue::new(None::<Steps>);
    Effect::new(move |_| {
        let now = position.get();
        let before = heard.get_value();
        heard.set_value(Some(now));
        if before == Some(now) {
            return;
        }
        let Some(index) = now.question else {
            if before.is_none() {
                crate::sounds::play(quiz_core::sounds::Sound::Round);
            }
            return;
        };
        let reveal_at = prepared.with_value(|p| {
            p.get(index).map(|q| {
                if matches!(q.kind.as_str(), "PImage" | "PVideo") {
                    3
                } else {
                    2
                }
            })
        });
        if before.is_some() && mode != PlayMode::QuestionsOnly && Some(now.step) == reveal_at {
            crate::sounds::play(quiz_core::sounds::Sound::Answer);
        }
    });
    // zoom: the question's picture enlarged to fill the screen
    let zoom = RwSignal::new(false);
    let timer = RwSignal::new(0_u64);
    question_sync(position, zoom, timer, round, mode, presenter);
    let with_current = move |f: &dyn Fn(&Prepared) -> bool| {
        position
            .get_untracked()
            .question
            .is_some_and(|i| prepared.with_value(|p| f(&p[i])))
    };

    let advance = move |forward: bool| {
        zoom.set(false);
        let before = position.get_untracked();
        let mut next = before;
        let finished = round.with_value(|r| {
            if forward {
                next.next(&r.questions, mode)
            } else {
                next.previous(&r.questions, mode);
                false
            }
        });
        if finished {
            show.dispatch.run(Action::BackToDashboard);
            return;
        }
        // Seconds on the current question: a new one starts at 0
        if next.question != before.question {
            timer.set(0);
        }
        position.set(next);
    };
    let keys = window_event_listener(leptos::ev::keydown, move |event| {
        let key = event.key();
        match key.as_str() {
            "ArrowRight" => advance(true),
            "ArrowLeft" => advance(false),
            "Escape" if zoom.get_untracked() => zoom.set(false),
            "Escape" => show.dispatch.run(Action::BackToDashboard),
            _ if key.eq_ignore_ascii_case("z") && with_current(&Prepared::has_image) => {
                zoom.update(|z| *z = !*z)
            }
            _ => (),
        }
    });
    on_cleanup(move || keys.remove());

    // The timer runs in questions-only play once a question is shown, restarted on every step as
    // the JSX's interval is
    let ticking = StoredValue::new(None::<IntervalHandle>);
    let stop = move || {
        if let Some(Some(handle)) = ticking.try_update_value(Option::take) {
            handle.clear();
        }
    };
    Effect::new(move |_| {
        let p = position.get();
        stop();
        if mode == PlayMode::QuestionsOnly && p.question.is_some() && p.step > 0 {
            let handle =
                set_interval_with_handle(move || timer.update(|s| *s += 1), Duration::from_secs(1));
            ticking.set_value(handle.ok());
        }
    });
    on_cleanup(stop);

    if presenter {
        return view! { <Presenter prepared position zoom timer mode title=title.clone()
        on_previous=Callback::new(move |_| advance(false))
        on_next=Callback::new(move |_| advance(true))
        on_back=Callback::new(move |_| show.dispatch.run(Action::BackToDashboard))/> }
        .into_any();
    }

    let resolve_media = Callback::new(move |name: String| quiz.resolve_media(&name));
    view! { <Audience prepared position zoom timer mode round label title
    resolve_media preview=false
    on_previous=Callback::new(move |_| advance(false))
    on_next=Callback::new(move |_| advance(true))/> }
    .into_any()
}

/// A fixed audience frame on a 2400×1350 canvas scaled to its parent's width (16:9).
/// `None` selects the round title card; `Some(index)` is zero based. Step 0 is the
/// question number, 1 the question (pre-media for PImage/PVideo), 2 the answer
/// (question for PImage/PVideo), and 3 the PImage/PVideo answer. Invalid indices
/// select the title card; oversized steps clamp to the final answer.
///
/// Inherits ThemeScope and translations. The caller owns the media URLs for the
/// mounted lifetime. Props are snapshots: remount this component to change the frame.
/// No show context, synchronization, hotkeys, running timer, sounds or media autoplay.
///
/// ```ignore
/// <QuestionPreview round question_index=Some(0) step=2
///     resolve_media=Callback::new(|name: String| Some(format!("source/{name}")))/>
/// ```
#[component]
pub fn QuestionPreview(
    round: quiz_core::quiz::parse::Round,
    question_index: Option<usize>,
    step: usize,
    resolve_media: Callback<String, Option<String>>,
) -> impl IntoView {
    view! { <QuestionPreviewMode round question_index step resolve_media mode=PlayMode::WithAnswers/> }
}

/// Internal map renderer: also shows the questions-only footer and stopped timer.
#[component]
pub(crate) fn QuestionPreviewMode(
    round: quiz_core::quiz::parse::Round,
    question_index: Option<usize>,
    step: usize,
    resolve_media: Callback<String, Option<String>>,
    mode: PlayMode,
) -> impl IntoView {
    let t = use_t();
    let fixed = preview_position(&round.questions, question_index, step);
    let prepared = prepare(&round, |name| resolve_media.run(name.to_string()));
    let label = format!("{} {}", t.get("round"), round.number);
    let title = round.title.clone();
    let position = RwSignal::new(fixed);
    view! {
        <PreviewStage>
            <Audience prepared=StoredValue::new(prepared) position zoom=RwSignal::new(false)
                timer=RwSignal::new(0) mode round=StoredValue::new(round)
                label title resolve_media preview=true
                on_previous=Callback::new(|_| ()) on_next=Callback::new(|_| ())/>
        </PreviewStage>
    }
}

/// Shared inert, fixed-size stage for the check/map's standalone scene renderers.
#[component]
pub(crate) fn PreviewStage(children: Children) -> impl IntoView {
    let frame = NodeRef::<leptos::html::Div>::new();
    let scale = RwSignal::new(0.0);
    Effect::new(move |_| {
        if let Some(element) = frame.get() {
            let resize = Closure::<dyn FnMut(js_sys::Array)>::new(move |_| {
                if let Some(element) = frame.get() {
                    scale.set(element.get_bounding_client_rect().width() / crate::stage::BASE_W);
                }
            });
            if let Ok(observer) = web_sys::ResizeObserver::new(resize.as_ref().unchecked_ref()) {
                scale.set(element.get_bounding_client_rect().width() / crate::stage::BASE_W);
                observer.observe(&element);
                let resize = StoredValue::new_local(Some((observer, resize)));
                on_cleanup(move || {
                    resize.update_value(|value| {
                        if let Some((observer, _)) = value.take() {
                            observer.disconnect();
                        }
                    });
                });
            }
        }
    });
    let layout = Signal::derive(move || crate::stage::StageLayout {
        w: crate::stage::BASE_W,
        h: crate::stage::BASE_H,
        scale: scale.get(),
    });
    view! {
        <leptos::context::Provider value=layout>
        <div node_ref=frame class="question-preview" inert=true>
            <div class="question-preview-canvas font-show"
                style=move || format!("transform:scale({})", scale.get())>
                {children()}
            </div>
        </div>
        </leptos::context::Provider>
    }
}

/// Keep malformed editor selections from indexing a round or hiding its answer.
fn preview_position(questions: &[Question], question: Option<usize>, step: usize) -> Steps {
    match question.and_then(|index| questions.get(index).map(|q| (index, q))) {
        Some((index, q)) => Steps {
            question: Some(index),
            step: step.min(Steps::count(&kind(q), PlayMode::WithAnswers) - 1),
        },
        None => Steps::default(),
    }
}

fn prepare(
    round: &quiz_core::quiz::parse::Round,
    resolve: impl Fn(&str) -> Option<String>,
) -> Vec<Prepared> {
    round
        .questions
        .iter()
        .map(|q| {
            let kind = kind(q);
            Prepared {
                source: q.media.as_deref().and_then(&resolve),
                answer_source: (kind == "Handout")
                    .then(|| resolve(&answer_text(q)))
                    .flatten(),
                question: q.clone(),
                kind,
            }
        })
        .collect()
}

#[component]
fn Audience(
    prepared: StoredValue<Vec<Prepared>>,
    position: RwSignal<Steps>,
    zoom: RwSignal<bool>,
    timer: RwSignal<u64>,
    mode: PlayMode,
    round: StoredValue<quiz_core::quiz::parse::Round>,
    label: String,
    title: String,
    resolve_media: Callback<String, Option<String>>,
    preview: bool,
    on_previous: Callback<()>,
    on_next: Callback<()>,
) -> impl IntoView {
    let t = use_t();
    let current = Memo::new(move |_| position.get().question);
    let total = prepared.with_value(Vec::len);
    let zoom_source = move || {
        let p = position.get();
        p.question.and_then(|i| {
            prepared.with_value(|all| {
                let q = &all[i];
                picture_source(q, p, mode)
            })
        })
    };
    let has_image = move || {
        current
            .get()
            .is_some_and(|i| prepared.with_value(|p| p[i].has_image()))
    };
    // Follow source changes while open (including Handout's answer), retain the last picture
    // while Presence runs its exit animation.
    let zoom_display = Memo::new(move |previous: Option<&Option<String>>| {
        if zoom.get() && has_image() {
            zoom_source()
        } else {
            previous.cloned().flatten()
        }
    });
    let big = move || position.get().step == 0;
    let questions_only = mode == PlayMode::QuestionsOnly;

    view! {
        <Show when=move || current.get().is_some()
            fallback=move || view! { <RoundIntro label=label.clone() title=title.clone()
                on_click=on_next/> }>
            <div class="game-screen h-full w-full bg-ground relative overflow-hidden flex flex-col font-show"
                style=SPRINGS.as_str()>
                <crate::scenes::backdrop::SceneBackdrop quiet=true/>

                // BIG NUMBER
                <div class="relative h-[25cqh] w-full flex items-center justify-center z-20 pointer-events-none">
                    <div class="game-number flex flex-col items-center" class:game-number-big=big>
                        <span class=move || format!(
                            "font-black text-white text-5xl uppercase tracking-widest transition-colors duration-500 {}",
                            if big() { "opacity-100 text-primary-500" } else { "opacity-50 text-gray-500" })>
                            {t.get("question")}
                        </span>
                        <span class="font-title font-black text-[250px] text-white leading-none drop-shadow-[0_0_50px_rgba(255,255,255,0.2)]">
                            {move || current.get().map(|i| prepared.with_value(|p| p[i].question.id.clone()))}
                        </span>
                    </div>
                </div>

                // CONTENT and HANDOUT, rendered anew for each question
                {move || current.get().map(|index| view! {
                    <QuestionView index prepared=prepared.with_value(|p| p[index].clone())
                        position mode zoom resolve_media preview/>
                })}

                // ENLARGED IMAGE
                <Presence when=Signal::derive(move || zoom.get() && has_image())
                    class="absolute inset-0 z-50" enter="game-zoom-enter" exit="game-zoom-exit">
                    <div on:click=move |_| zoom.set(false)
                        class="absolute inset-0 z-50 bg-black/95 flex items-center justify-center p-[2cqh] cursor-zoom-out">
                        // Kept while it fades out, as AnimatePresence keeps the last frame
                        <img src=move || zoom_display.get() alt="Visual" class="w-full h-full object-contain"/>
                    </div>
                </Presence>

                // FOOTER
                <div class="h-[10cqh] border-t border-white/5 bg-black/50 backdrop-blur-md flex items-center justify-between px-10 relative z-30">
                    <div class="flex items-center gap-4 text-white/40 font-mono text-xs uppercase tracking-widest font-bold">
                        <span class="bg-white/10 px-3 py-1.5 rounded-md text-white">
                            {t.get("round")} " " {round.with_value(|r| r.number.clone())}
                        </span>
                        <span>{move || t.fmt("question_of", &[
                            ("n", &current.get().map_or(0, |i| i + 1).to_string()),
                            ("total", &total.to_string()),
                        ])}</span>
                        <span class="text-primary-500">
                            {t.get(if questions_only { "q_mode_questions" } else { "q_mode_answers" })}
                        </span>
                    </div>

                    {questions_only.then(|| view! {
                        <div class="absolute left-1/2 -translate-x-1/2 flex items-center gap-3 text-white font-mono text-3xl font-bold bg-white/5 px-8 py-2.5 rounded-full border border-white/10 shadow-lg transition-colors">
                            <GameIcon name="clock" size=24 class=Signal::derive(move || {
                                if timer.get() > 0 { "text-accent-500 animate-pulse" } else { "text-gray-500" }.to_string()
                            })/>
                            <span>{move || format!("{}:{:02}", timer.get() / 60, timer.get() % 60)}</span>
                        </div>
                    })}

                    <div class="flex items-center gap-3 text-white/30 text-xs uppercase font-bold tracking-wider">
                        <button on:click=move |_| on_previous.run(())
                            class="p-4 bg-white/5 rounded-full hover:bg-white/10 hover:text-white transition-all hover:scale-110">
                            <GameIcon name="arrow_left" size=20/>
                        </button>
                        <button on:click=move |_| on_next.run(())
                            class="p-4 bg-white/10 border border-white/20 rounded-full hover:bg-primary-500 hover:border-primary-500 hover:shadow-[0_0_15px_rgb(var(--glow-primary)/0.5)] transition-all hover:scale-110 text-white">
                            <GameIcon name="arrow_right" size=20/>
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
    .into_any()
}

/// A handout's reveal swaps in the answer picture (handoutRevealed).
fn handout_revealed(q: &Prepared, p: Steps, mode: PlayMode) -> bool {
    q.kind == "Handout" && p.revealed(&q.kind, mode) && q.answer_source.is_some()
}

fn picture_source(q: &Prepared, p: Steps, mode: PlayMode) -> Option<String> {
    if handout_revealed(q, p, mode) {
        q.answer_source.clone()
    } else {
        q.source.clone()
    }
}

/// One question's content (pre-media, question, options or answer) and handout sheet.
#[component]
fn QuestionView(
    index: usize,
    prepared: Prepared,
    position: RwSignal<Steps>,
    mode: PlayMode,
    zoom: RwSignal<bool>,
    resolve_media: Callback<String, Option<String>>,
    preview: bool,
) -> impl IntoView {
    let t = use_t();
    let kind = prepared.kind.clone();
    let pre = prepared.is_pre_media();
    let handout = kind == "Handout";
    // This question's step; 0 once another question is current, until this view is gone
    let step = Memo::new(move |_| {
        let p = position.get();
        if p.question == Some(index) { p.step } else { 0 }
    });
    let at = move || Steps {
        question: Some(index),
        step: step.get(),
    };
    let revealed = Signal::derive({
        let kind = kind.clone();
        move || at().revealed(&kind, mode)
    });
    let show_big = move || step.get() == 0;
    let show_pre = move || pre && step.get() == 1;
    let show_question = move || !show_big() && !show_pre();
    // AnimatePresence mode="wait": the question waits until the pre-media has faded out
    let pre_mounted = RwSignal::new(untrack(show_pre));
    Effect::new(move |_| {
        if show_pre() {
            pre_mounted.set(true);
        }
    });
    let prepared = StoredValue::new(prepared);
    let sheet_source = move || prepared.with_value(|p| p.source.clone());
    let answer_source = move || prepared.with_value(|p| p.answer_source.clone());
    let is_video = kind == "PVideo";
    let is_revealed_handout = move || prepared.with_value(|p| handout_revealed(p, at(), mode));
    let id = prepared.with_value(|p| p.question.id.clone());
    let text = prepared.with_value(|p| p.question.text.clone());

    view! {
        <div class="flex-1 flex flex-col items-center justify-start relative z-10 px-16 pt-4">
            <Presence when=Signal::derive(show_pre) enter="game-pre-enter" exit="game-pre-exit"
                on_exit=Callback::new(move |_| pre_mounted.set(false))>
                <div class="w-full h-[65cqh] flex items-center justify-center">
                    {if is_video {
                        view! { <GameVideo source=sheet_source() autoplay=!preview
                            class="max-h-full max-w-full rounded-2xl shadow-2xl border border-white/10"/> }.into_any()
                    } else {
                        view! { <img src=sheet_source() alt="Visual" on:click=move |_| zoom.set(true)
                            class="max-h-full max-w-full rounded-2xl shadow-2xl border border-white/10 cursor-zoom-in"/> }.into_any()
                    }}
                </div>
            </Presence>
            <Show when=move || show_question() && !handout && !pre_mounted.get()>
                <QuestionContent prepared=prepared.get_value() revealed zoom resolve_media/>
            </Show>
        </div>

        // HANDOUT: the printed sheet as large as the screen allows, then the answer sheet
        <Show when=move || handout && !show_big()>
            <div class="absolute inset-x-0 top-0 bottom-[10cqh] z-40 bg-ground flex flex-col px-[3cqw] pt-[2.5cqh] pb-[2cqh] gap-[1.5cqh]">
                // clear of the back and settings buttons
                <div class="flex items-center gap-6 min-h-[5cqh] px-[2.5cqw]">
                    <span class="shrink-0 font-black text-4xl text-white/50 uppercase tracking-widest">
                        {t.get("question")} " " {id.clone()}
                    </span>
                    {(!text.is_empty()).then(|| view! {
                        <h2 class="flex-1 min-w-0 font-bold text-5xl text-white leading-tight truncate">{text.clone()}</h2>
                    })}
                    <Show when=is_revealed_handout>
                        <span class="game-pop ml-auto shrink-0 px-8 py-3 rounded-full bg-green-600 text-white font-black text-3xl uppercase tracking-wider shadow-[0_0_40px_rgba(22,163,74,0.5)]">
                            {t.get("correct_answer")}
                        </span>
                    </Show>
                </div>
                <div class="relative flex-1 min-h-0">
                    // The sheet and the answer cross-fade (AnimatePresence initial={false}, 0.6 s)
                    <img src=sheet_source() alt="" on:click=move |_| zoom.set(true)
                        class="game-handout absolute inset-0 w-full h-full object-contain cursor-zoom-in"
                        class:game-handout-hidden=is_revealed_handout/>
                    {answer_source().map(|src| view! {
                        <img src=src alt="" on:click=move |_| zoom.set(true)
                            class="game-handout absolute inset-0 w-full h-full object-contain cursor-zoom-in"
                            class:game-handout-hidden=move || !is_revealed_handout()/>
                    })}
                </div>
            </div>
        </Show>
    }
}

/// The question, its media and the options or the answer (the "question-content" motion.div).
#[component]
fn QuestionContent(
    prepared: Prepared,
    revealed: Signal<bool>,
    zoom: RwSignal<bool>,
    resolve_media: Callback<String, Option<String>>,
) -> impl IntoView {
    let t = use_t();
    let Prepared {
        question: q,
        kind,
        source,
        ..
    } = prepared.clone();
    let media = (source.is_some() && !prepared.is_pre_media()).then(|| {
        let media = if kind.contains("Video") {
            view! { <GameVideo source=source.clone() autoplay=false
                class="max-h-[40cqh] rounded-2xl border border-white/20"/> }
            .into_any()
        } else if kind == "Audio" {
            view! {
                <div class="bg-white/10 p-6 rounded-full border border-white/10 flex items-center justify-center w-[500px]">
                    <GameAudio source=source.clone()/>
                </div>
            }
            .into_any()
        } else {
            view! { <img src=source.clone() on:click=move |_| zoom.set(true)
                class="max-h-[40cqh] rounded-2xl border border-white/20 cursor-zoom-in"/> }
            .into_any()
        };
        view! { <div class="mb-8 max-h-[40cqh] relative">{media}</div> }
    });
    let show_options = matches!(kind.as_str(), "ABCD" | "Yes/No" | "Sort" | "Top5");
    let answers = match kind.as_str() {
        "ABCD" => abcd(&q, revealed).into_any(),
        "Yes/No" => yes_no(&q, revealed, t).into_any(),
        "Sort" => view! { <SortItems layout=sort_layout(&q) revealed resolve_media/> }.into_any(),
        "Top5" => top_list(&q, revealed).into_any(),
        _ => ().into_any(),
    };
    let written = (!show_options).then(|| {
        let answer = answer_text(&q);
        let (icon, expected) = match kind.as_str() {
            "Numeric" => ("hash", t.get("type_numeric")),
            "Top5" => {
                let n = top_answers(&q).len();
                let n = if n == 0 { 5 } else { n }.to_string();
                ("pen_line", t.fmt("type_top5", &[("n", &n)]))
            }
            _ => ("pen_line", t.get("type_written")),
        };
        view! {
            <div class="flex flex-col items-center">
                <Show when=move || revealed.get() fallback=move || view! {
                    <div class="flex items-center gap-6 px-16 py-8 rounded-[3rem] bg-white/5 border border-white/10 text-white/50 animate-pulse shadow-inner">
                        <GameIcon name=icon size=40/>
                        <span class="font-black text-4xl tracking-[0.2em] uppercase">{expected.clone()}</span>
                    </div>
                }>
                    <div class="game-answer px-20 py-10 bg-gradient-to-br from-green-500 to-green-600 rounded-[3rem] text-black font-black text-7xl shadow-[0_0_80px_rgba(34,197,94,0.6)] uppercase tracking-tight">
                        {answer.clone()}
                    </div>
                </Show>
            </div>
        }
    });
    view! {
        <div class="game-content w-full max-w-[1600px] flex flex-col items-center text-center">
            <h2 class=format!("{} font-bold text-white leading-tight mb-8 drop-shadow-xl", font_size(&q.text))>
                {q.text.clone()}
            </h2>
            {media}
            <div class="w-full flex-1 flex items-center justify-center min-h-[200px]">
                {answers}
                {written}
            </div>
        </div>
    }
}

fn abcd(q: &Question, revealed: Signal<bool>) -> impl IntoView + use<> {
    let answer = answer_text(q).trim().to_uppercase();
    let options = q.options.clone();
    view! {
        <div class="grid grid-cols-2 gap-6 w-full max-w-[1400px]">
            {["A", "B", "C", "D"].into_iter().map(|letter| {
                let correct = answer == letter;
                view! {
                    <div class=move || format!(
                        "p-6 rounded-3xl border flex items-center gap-6 transition-all duration-700 {}",
                        if !revealed.get() { "bg-white/5 border-white/10 text-gray-300" }
                        else if correct { "bg-green-600 border-green-400 shadow-[0_0_40px_rgba(22,163,74,0.6)] scale-105 z-10 text-white" }
                        else { "opacity-20 grayscale border-transparent" })>
                        <div class="w-16 h-16 rounded-full flex items-center justify-center font-black text-2xl bg-black/30 border border-white/10 shrink-0">{letter}</div>
                        <span class="font-bold text-4xl text-left leading-tight">{options.get(letter).cloned()}</span>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

fn yes_no(q: &Question, revealed: Signal<bool>, t: T) -> impl IntoView + use<> {
    let answer = answer_text(q).trim().to_lowercase();
    view! {
        <div class="grid grid-cols-2 gap-12 w-full max-w-3xl">
            {["Yes", "No"].into_iter().map(|value| {
                let correct = answer == value.to_lowercase();
                view! {
                    <div class=move || format!(
                        "h-48 rounded-[3rem] border flex flex-col items-center justify-center gap-4 transition-all duration-500 {}",
                        if !revealed.get() { "bg-white/5 border-white/10" }
                        else if correct { "bg-green-600 border-green-400 scale-110 shadow-[0_0_40px_rgba(22,163,74,0.6)]" }
                        else { "opacity-20" })>
                        <GameIcon name=if value == "Yes" { "check" } else { "x" } size=64/>
                        <span class="text-5xl font-black uppercase">
                            {t.get(if value == "Yes" { "answer_yes" } else { "answer_no" })}
                        </span>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

fn top_list(q: &Question, revealed: Signal<bool>) -> impl IntoView + use<> {
    let answers = top_answers(q);
    let count = answers.len();
    let columns = top_columns(count);
    // A column never has more than five rows, so two columns keep the full size; three columns are
    // narrower and get slightly smaller text
    let (row, rank, text, detail) = if columns < 3 {
        (
            "px-8 py-5 gap-6",
            "w-16 h-16 text-2xl",
            "text-5xl",
            "text-3xl",
        )
    } else {
        (
            "px-6 py-4 gap-4",
            "w-14 h-14 text-xl",
            "text-4xl",
            "text-2xl",
        )
    };
    // Rows of equal height (the tallest answer's), filled down each column
    view! {
        <div class=format!("grid gap-x-6 {} w-full",
                if columns == 1 { "gap-y-4 max-w-4xl" } else { "gap-y-3 max-w-[92cqw]" })
            style=format!(
                "grid-template-columns:repeat({columns}, minmax(0, 1fr));grid-template-rows:repeat({}, 1fr);grid-auto-flow:column",
                count.div_ceil(columns))>
            {answers.into_iter().enumerate().map(|(i, answer)| {
                let (main, extra) = split_answer(&answer);
                view! {
                    <div class=move || format!(
                        "{row} min-w-0 rounded-2xl border flex items-center transition-all duration-700 {}",
                        if revealed.get() {
                            "bg-gradient-to-r from-primary-900/60 to-primary-800/40 border-primary-500 shadow-[0_0_20px_rgb(var(--glow-primary)/0.4)]"
                        } else { "bg-white/5 border-white/10" })>
                        <div class=format!("{rank} rounded-xl bg-black/50 border border-white/10 flex items-center justify-center font-black text-white/50 shrink-0 shadow-inner")>
                            {format!("#{}", i + 1)}
                        </div>
                        <Show when=move || revealed.get() fallback=move || view! {
                            <span class=format!("{text} min-w-0 font-bold leading-tight text-transparent bg-white/10 rounded-lg select-none blur-sm truncate")>
                                "???????????????"
                            </span>
                        }>
                            <span class="min-w-0 flex flex-col text-left leading-tight">
                                <span class=format!("{text} font-bold tracking-tight text-white [text-wrap:balance]")>{main.clone()}</span>
                                {extra.clone().map(|extra| view! {
                                    <span class=format!("{detail} font-semibold text-primary-200/70 mt-1 [text-wrap:balance]")>{extra}</span>
                                })}
                            </span>
                        </Show>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

/// A video that stops when it leaves the screen.
#[component]
fn GameVideo(source: Option<String>, autoplay: bool, class: &'static str) -> impl IntoView {
    let node = NodeRef::<leptos::html::Video>::new();
    Effect::new(move |_| {
        if let Some(element) = node.get() {
            on_cleanup(move || {
                let _ = element.pause();
            });
        }
    });
    view! { <video node_ref=node src=source controls autoplay=autoplay preload="auto" class=class/> }
}

/// An audio player that stops when it leaves the screen.
#[component]
fn GameAudio(source: Option<String>) -> impl IntoView {
    let node = NodeRef::<leptos::html::Audio>::new();
    Effect::new(move |_| {
        if let Some(element) = node.get() {
            on_cleanup(move || {
                let _ = element.pause();
            });
        }
    });
    view! { <audio node_ref=node controls src=source preload="auto" class="w-full"/> }
}

/// One channel per mounted round. Mark incoming frames before updating signals to prevent echoes.
fn question_sync(
    position: RwSignal<Steps>,
    zoom: RwSignal<bool>,
    timer: RwSignal<u64>,
    round: StoredValue<quiz_core::quiz::parse::Round>,
    mode: PlayMode,
    presenter: bool,
) {
    let sender = if presenter { "presenter" } else { "main" };
    let channel = StoredValue::new_local(web_sys::BroadcastChannel::new("quiz-question-sync").ok());
    let last = StoredValue::new(QuestionFrame::new(
        position.get_untracked(),
        zoom.get_untracked(),
    ));
    let send = move |message: QuestionMessage| {
        channel.with_value(|c| {
            if let Some(c) = c
                && let Ok(json) = serde_json::to_string(&message)
                && let Ok(value) = js_sys::JSON::parse(&json)
            {
                let _ = c.post_message(&value);
            }
        });
    };
    let incoming =
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
            let message = js_sys::JSON::stringify(&event.data())
                .ok()
                .and_then(|j| j.as_string())
                .and_then(|j| serde_json::from_str::<QuestionMessage>(&j).ok());
            match message {
                Some(QuestionMessage::Request { sender: other }) if other != sender => {
                    send(QuestionMessage::Update {
                        sender: sender.into(),
                        payload: QuestionFrame::new(position.get_untracked(), zoom.get_untracked()),
                    });
                }
                Some(QuestionMessage::Update {
                    sender: other,
                    payload,
                }) if other != sender => {
                    if let Some(next) = round.with_value(|r| payload.position(&r.questions, mode)) {
                        last.set_value(payload.clone());
                        if next.question != position.get_untracked().question {
                            timer.set(0);
                        }
                        batch(move || {
                            if position.get_untracked() != next {
                                position.set(next);
                            }
                            if zoom.get_untracked() != payload.zoom {
                                zoom.set(payload.zoom);
                            }
                        });
                    }
                }
                _ => (),
            }
        });
    channel.with_value(|c| {
        if let Some(c) = c {
            c.set_onmessage(Some(incoming.as_ref().unchecked_ref()));
        }
    });
    send(QuestionMessage::Request {
        sender: sender.into(),
    });
    Effect::new(move |_| {
        let frame = QuestionFrame::new(position.get(), zoom.get());
        if frame != last.get_value() {
            last.set_value(frame.clone());
            send(QuestionMessage::Update {
                sender: sender.into(),
                payload: frame,
            });
        }
    });
    let incoming = StoredValue::new_local(Some(incoming));
    on_cleanup(move || {
        channel.with_value(|c| {
            if let Some(c) = c {
                c.set_onmessage(None);
                c.close();
            }
        });
        incoming.update_value(|c| {
            c.take();
        });
    });
}

#[component]
fn Presenter(
    prepared: StoredValue<Vec<Prepared>>,
    position: RwSignal<Steps>,
    zoom: RwSignal<bool>,
    timer: RwSignal<u64>,
    mode: PlayMode,
    title: String,
    on_previous: Callback<()>,
    on_next: Callback<()>,
    on_back: Callback<()>,
) -> impl IntoView {
    let t = use_t();
    let current = Memo::new(move |_| position.get().question);
    let total = prepared.with_value(Vec::len);
    let has_image = move || {
        current
            .get()
            .is_some_and(|i| prepared.with_value(|p| p[i].has_image()))
    };
    view! {
        <div class="game-screen h-full w-full bg-panel text-white p-10 font-sans flex flex-col">
            <div class="flex flex-wrap justify-between items-center gap-4 mb-8 border-b border-white/10 pb-6">
                <div>
                    <h1 class="text-3xl font-black text-yellow-500 tracking-widest uppercase">{t.get("host_dashboard")}</h1>
                    <div class="flex items-center gap-3 mt-2">
                        <span class="text-white bg-white/10 px-3 py-1 rounded text-sm font-bold">{title}</span>
                        <span class="text-primary-400 font-mono text-sm">{t.get(if mode == PlayMode::QuestionsOnly { "q_mode_questions" } else { "q_mode_answers" })}</span>
                    </div>
                </div>
                <div class="flex flex-wrap gap-4">
                    <Show when=has_image>
                        <button on:click=move |_| zoom.update(|z| *z = !*z)
                            class=move || format!("px-6 py-4 font-bold rounded-xl transition-all flex items-center gap-2 {}", if zoom.get() { "bg-yellow-500 text-black" } else { "bg-white/10 hover:bg-white/20 text-white" })>
                            {move || view! { <GameIcon name=if zoom.get() { "minimize" } else { "maximize" } size=20/> }}
                            {move || t.get(if zoom.get() { "zoom_close" } else { "zoom_open" })}
                        </button>
                    </Show>
                    <button on:click=move |_| on_previous.run(()) class="px-8 py-4 bg-white/10 hover:bg-white/20 text-white font-bold rounded-xl transition-all">"< " {t.get("btn_prev")}</button>
                    <button on:click=move |_| on_next.run(()) class="px-12 py-4 bg-primary-600 hover:bg-primary-500 text-white font-black rounded-xl transition-all shadow-[0_0_20px_rgb(var(--glow-primary)/0.4)]">{t.get("btn_next")} " >"</button>
                    <button on:click=move |_| on_back.run(()) class="px-6 py-4 bg-red-900/50 hover:bg-red-800/80 text-white font-bold rounded-xl transition-all ml-8">{t.get("exit_round")}</button>
                </div>
            </div>
            <div class="grid grid-cols-2 gap-10 flex-1 min-h-0">
                <div class="bg-black/50 p-8 rounded-[2rem] border border-white/10 flex flex-col overflow-auto">
                    <span class="text-primary-500 font-bold tracking-widest uppercase mb-6 text-sm flex items-center gap-2">
                        <span class="w-2 h-2 rounded-full bg-primary-500 animate-pulse"/> {t.get("live_on_screen")}
                    </span>
                    <Show when=move || current.get().is_some() fallback=move || view! {
                        <div class="flex-1 flex items-center justify-center text-4xl font-black text-white/30 uppercase text-center">{t.get("round_intro")}</div>
                    }>
                        {move || current.get().map(|i| {
                            let p = prepared.with_value(|p| p[i].clone());
                            let steps = Steps::count(&p.kind, mode) - 1;
                            view! {
                                <div class="flex flex-col h-full">
                                    <div class="flex justify-between items-end mb-6 border-b border-white/10 pb-4">
                                        <h2 class="text-4xl font-black text-white">{t.fmt("question_of", &[("n", &(i+1).to_string()), ("total", &total.to_string())])}</h2>
                                        <span class="text-white/40 font-mono">{move || t.fmt("step_of", &[("n", &position.get().step.to_string()), ("total", &steps.to_string())])}</span>
                                    </div>
                                    <p class="text-2xl text-white/90 leading-relaxed font-medium mb-8">{p.question.text.clone()}</p>
                                    {presenter_options(&p, t)}
                                    {(p.kind == "Handout").then(|| p.source.clone().map(|source| view! {
                                        <div class="mb-6 flex flex-col gap-2">
                                            <span class="text-white/40 font-bold uppercase tracking-widest text-xs">{t.get("handout_sheet")}</span>
                                            <img src=source alt="" class="max-h-[17vh] self-start rounded-xl border border-white/10"/>
                                        </div>
                                    }))}
                                    <div class="mt-auto bg-green-900/30 border border-green-500/50 p-6 rounded-2xl flex flex-col gap-2">
                                        <span class="text-green-500/80 font-bold uppercase tracking-widest text-xs">{t.get("correct_answer")}</span>
                                        {presenter_answer(&p)}
                                    </div>
                                    {(mode == PlayMode::QuestionsOnly).then(|| view! {
                                        <div class="mt-6 flex items-center gap-3 text-yellow-500 font-mono text-xl bg-yellow-500/10 p-4 rounded-xl border border-yellow-500/20 w-fit shadow-inner">
                                            <GameIcon name="clock" size=24/>{move || format!("{}:{:02}", timer.get()/60, timer.get()%60)}
                                        </div>
                                    })}
                                </div>
                            }
                        })}
                    </Show>
                </div>
                <div class="bg-white/5 p-8 rounded-[2rem] border border-white/5 flex flex-col relative overflow-hidden">
                    <span class="text-gray-400 font-bold tracking-widest uppercase mb-6 text-sm">{t.get("next_title")}</span>
                    <div class="flex-1 flex flex-col justify-center items-center text-center px-10">
                        <GameIcon name="arrow_right" size=80 class=Signal::stored("text-white/5 mb-8".to_string())/>
                        <h3 class="text-3xl font-bold text-white/60">{move || {
                            let p = position.get();
                            match p.question {
                                None => t.get("next_show_number"),
                                Some(i) if p.step < prepared.with_value(|q| Steps::count(&q[i].kind, mode))-1 => t.get("next_step"),
                                Some(i) if i+1 < total => t.fmt("next_question", &[("n", &(i+2).to_string())]),
                                _ => t.get("next_finish"),
                            }
                        }}</h3>
                    </div>
                </div>
            </div>
        </div>
    }
}
fn presenter_options(p: &Prepared, t: T) -> AnyView {
    let q = &p.question;
    let options = match p.kind.as_str() {
        "ABCD" => ["A", "B", "C", "D"]
            .into_iter()
            .map(|letter| {
                (
                    format!(
                        "{letter}: {}",
                        q.options.get(letter).cloned().unwrap_or_default()
                    ),
                    answer_text(q).trim().eq_ignore_ascii_case(letter),
                )
            })
            .collect::<Vec<_>>(),
        "Yes/No" => ["Yes", "No"]
            .into_iter()
            .map(|value| {
                (
                    t.get(if value == "Yes" {
                        "answer_yes"
                    } else {
                        "answer_no"
                    }),
                    answer_text(q).trim().eq_ignore_ascii_case(value),
                )
            })
            .collect(),
        _ => return ().into_any(),
    };
    view! { <div class="grid grid-cols-2 gap-4 mb-8">
        {options.into_iter().map(|(label, correct)| view! {
            <div class=format!("p-4 rounded-xl border font-bold {}", if correct { "bg-green-600/20 border-green-500 text-green-400" } else { "bg-white/5 border-white/10 text-white/50" })>{label}</div>
        }).collect_view()}
    </div> }.into_any()
}
fn presenter_answer(p: &Prepared) -> AnyView {
    let q = &p.question;
    match p.kind.as_str() {
        "Handout" => match p.answer_source.clone() {
            Some(source) => view! { <img src=source alt="" class="max-h-[17vh] self-start rounded-xl border border-green-500/40 mt-1"/> }.into_any(),
            None => view! { <span class="text-xl font-bold text-green-400/60">"–"</span> }.into_any(),
        },
        "Sort" => {
            let layout = sort_layout(q);
            let rows = layout.correct.len().div_ceil(2);
            view! {
                <div class="flex flex-col gap-3 mt-1">
                    <span class="text-4xl font-black text-green-400 tracking-wider">{layout.answer}</span>
                    <div class="grid grid-cols-2 gap-x-6 gap-y-1" style=format!("grid-template-rows:repeat({},auto);grid-auto-flow:column", rows)>
                        {layout.correct.into_iter().map(|it| view! {
                            <div class="flex gap-3 items-center min-w-0">
                                <span class="text-green-500/50 font-bold font-mono text-sm w-8">{it.place} "."</span>
                                <span class="text-xl font-black text-green-400">{it.item.letter}</span>
                                <span class="text-xl font-bold text-green-200 truncate">{if it.item.text.is_empty() { it.item.media.unwrap_or_default() } else { it.item.text }}</span>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
            }.into_any()
        },
        "Top5" => {
            let answers = top_answers(q);
            let count = answers.len();
            view! {
                <div class="grid gap-x-8 gap-y-2 mt-2" style=format!("grid-template-columns:repeat({},minmax(0,1fr));grid-template-rows:repeat({},1fr);grid-auto-flow:column", top_columns(count), count.div_ceil(top_columns(count)))>
                    {answers.into_iter().enumerate().map(|(i, answer)| {
                        let (main, detail) = split_answer(&answer);
                        view! {
                            <div class="flex gap-3 items-baseline min-w-0">
                                <span class="text-green-500/50 font-bold font-mono text-sm">"#" {i+1}</span>
                                <span class=format!("{} font-black text-green-400 [text-wrap:balance]", if count > 5 { "text-xl" } else { "text-2xl" })>
                                    {main}{detail.map(|d| view! { <span class="ml-2 text-base font-bold text-green-400/60">{d}</span> })}
                                </span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            }.into_any()
        },
        _ => view! { <span class="text-4xl font-black text-green-400">{answer_text(q)}</span> }.into_any(),
    }
}
fn top_answers(q: &Question) -> Vec<String> {
    serde_json::to_value(q).unwrap_or_default()["answer"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| v.as_str().unwrap_or_default().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_frames_cover_every_type_and_reject_stale_selections() {
        let rounds = quiz_core::quiz::parse::parse_questions_csv(include_str!(
            "../../assets/dev-questions.csv"
        ));
        for round in rounds {
            assert_eq!(
                preview_position(&round.questions, None, 99),
                Steps::default()
            );
            assert_eq!(
                preview_position(&round.questions, Some(round.questions.len()), 0),
                Steps::default()
            );
            for (index, q) in round.questions.iter().enumerate() {
                let last = Steps::count(&kind(q), PlayMode::WithAnswers) - 1;
                for step in 0..=last {
                    assert_eq!(
                        preview_position(&round.questions, Some(index), step),
                        Steps {
                            question: Some(index),
                            step
                        }
                    );
                }
                assert_eq!(
                    preview_position(&round.questions, Some(index), usize::MAX).step,
                    last
                );
            }
        }
    }

    #[test]
    fn preparation_resolves_handout_answer_without_treating_text_answers_as_media() {
        let questions = vec![
            serde_json::from_value(serde_json::json!({"id":"1","text":"Sheet","type":"Handout","answer":"answer.jpg","media":"sheet.jpg","options":{}})).unwrap(),
            serde_json::from_value(serde_json::json!({"id":"2","text":"Who?","type":"Written","answer":"A person","media":null,"options":{}})).unwrap(),
        ];
        let round = quiz_core::quiz::parse::Round {
            id: "r".into(),
            number: "1".into(),
            title: "Round".into(),
            rows: vec![],
            questions,
        };
        let prepared = prepare(&round, |name| Some(format!("resolved/{name}")));
        assert_eq!(prepared[0].source.as_deref(), Some("resolved/sheet.jpg"));
        assert_eq!(
            prepared[0].answer_source.as_deref(),
            Some("resolved/answer.jpg")
        );
        assert!(prepared[1].source.is_none());
        assert!(prepared[1].answer_source.is_none());
        let missing = prepare(&round, |_| None);
        assert!(missing[0].source.is_none());
        assert!(missing[0].answer_source.is_none());
    }

    #[test]
    fn keeps_numbers_units_and_ordinals_together() {
        let nb = |s: &str| s.replace('_', "\u{a0}");
        assert_eq!(keep_together("2 166 086"), nb("2_166_086"));
        assert_eq!(keep_together("217 291 km²"), nb("217_291_km²"));
        assert_eq!(keep_together("1,5 l a 40 %"), nb("1,5_l a 40_%"));
        assert_eq!(keep_together("20. století"), nb("20._století"));
        assert_eq!(
            keep_together("5 kilometrů, 12 3456"),
            "5 kilometrů, 12 3456"
        );
        assert_eq!(keep_together("Rok 1989 bylo"), "Rok 1989 bylo");
        assert_eq!(keep_together("Rok 1989 byl"), nb("Rok 1989_byl"));
    }

    #[test]
    fn splits_the_bracketed_detail() {
        assert_eq!(
            split_answer(" Viktoriin ostrov (217 291 km²) "),
            (
                "Viktoriin ostrov".into(),
                Some("217\u{a0}291\u{a0}km²".into())
            )
        );
        assert_eq!(split_answer("(only)"), ("(only)".into(), None));
        assert_eq!(split_answer("a (b) c"), ("a (b) c".into(), None));
        assert_eq!(split_answer("a ()"), ("a ()".into(), None));
        assert_eq!(split_answer("x (a (b))"), ("x (a (b))".into(), None));
    }

    #[test]
    fn handout_zoom_follows_the_reveal_and_falls_back_to_the_sheet() {
        let q: Question = serde_json::from_value(serde_json::json!({
            "id":"1", "text":"Sheet", "type":"Handout", "answer":"answer.jpg", "media":"sheet.jpg", "options":{}
        })).unwrap();
        let mut p = Prepared {
            question: q,
            kind: "Handout".into(),
            source: Some("sheet.jpg".into()),
            answer_source: Some("answer.jpg".into()),
        };
        let at = |step| Steps {
            question: Some(0),
            step,
        };
        assert_eq!(
            picture_source(&p, at(1), PlayMode::WithAnswers).as_deref(),
            Some("sheet.jpg")
        );
        assert_eq!(
            picture_source(&p, at(2), PlayMode::WithAnswers).as_deref(),
            Some("answer.jpg")
        );
        assert_eq!(
            picture_source(&p, at(1), PlayMode::QuestionsOnly).as_deref(),
            Some("sheet.jpg")
        );
        p.answer_source = None;
        assert_eq!(
            picture_source(&p, at(2), PlayMode::WithAnswers).as_deref(),
            Some("sheet.jpg")
        );
        for kind in ["Audio", "Video", "PVideo"] {
            p.kind = kind.into();
            assert!(!p.has_image());
        }
        for kind in ["Written", "Image", "PImage", "Handout"] {
            p.kind = kind.into();
            assert!(p.has_image());
        }
    }

    #[test]
    fn font_size_counts_utf16_like_js() {
        // 29 characters but 36 bytes
        assert_eq!(font_size("Kdy a kde? (město. ± 1 rok)ěš"), "text-7xl");
        assert_eq!(font_size(""), "text-5xl");
        assert_eq!(font_size(&"a".repeat(30)), "text-6xl");
        assert_eq!(font_size(&"a".repeat(80)), "text-5xl");
    }
}
