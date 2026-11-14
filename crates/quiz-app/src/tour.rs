//! TourGuide.jsx and quiz/tour.js. Mount outside Stage, in viewport coordinates.
use leptos::prelude::*;
use quiz_core::{quiz::parse::Round, show::Scene};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, closure::Closure};
pub const TOUR_KEY: &str = "quiznight.tour";
pub const TOUR_EVENT: &str = "quiznight:tour";
pub const TOUR_STEPS: usize = 6;
/// No separate seen flag or automatic offer: explicit starts reset to intro;
/// ending removes the record. Extra fields survive phase updates like JS spread.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tour {
    pub quiz_id: String,
    pub phase: String,
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}
impl Tour {
    pub fn start(quiz_id: String) -> Self {
        Self {
            quiz_id,
            phase: "intro".into(),
            extra: Default::default(),
        }
    }
    pub fn active_for(&self, id: &str) -> bool {
        self.quiz_id == id
    }
}
pub fn decode_tour(raw: Option<&str>) -> Option<Tour> {
    serde_json::from_str(raw?).ok()
}
pub fn encode_tour(tour: Option<&Tour>) -> Option<String> {
    tour.and_then(|t| serde_json::to_string(t).ok())
}
pub fn next_phase<'a>(phase: &'a str, scene: Scene, presenter: bool, visited: &[Scene]) -> &'a str {
    let back = |from| scene == Scene::Dashboard && visited.contains(&from);
    match phase {
        "intro" if scene == Scene::Dashboard => "questions",
        "questions" if scene == Scene::Game => "questions_back",
        "questions_back" if back(Scene::Game) => {
            if presenter {
                "answers"
            } else {
                "presenter"
            }
        }
        "presenter" if presenter => "answers",
        "answers" if scene == Scene::Game => "answers_back",
        "answers_back" if back(Scene::Game) => "leaderboard",
        "leaderboard" if scene == Scene::Leaderboard => "leaderboard_back",
        "leaderboard_back" if back(Scene::Leaderboard) => "done",
        _ => phase,
    }
}
pub fn tour_step(phase: &str) -> usize {
    match phase {
        "questions" | "questions_back" => 2,
        "presenter" => 3,
        "answers" | "answers_back" => 4,
        "leaderboard" | "leaderboard_back" => 5,
        "done" => 6,
        _ => 1,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuestionProgress {
    pub q_index: i64,
    pub step: usize,
}
impl Default for QuestionProgress {
    fn default() -> Self {
        Self {
            q_index: -1,
            step: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Actions {
    None,
    Presenter,
    Done,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message {
    pub text: &'static str,
    pub targets: &'static [&'static str],
    pub actions: Actions,
}
pub fn message(
    phase: &str,
    scene: Scene,
    question: QuestionProgress,
    lb_step: usize,
    kind: Option<&str>,
) -> Message {
    let mut m = Message {
        text: "tr_back_to_dashboard",
        targets: &[],
        actions: Actions::None,
    };
    match phase {
        "done" => {
            m.text = "tr_done";
            m.actions = Actions::Done;
        }
        "intro" => match scene {
            Scene::Welcome => {
                m.text = "tr_welcome";
                m.targets = &["start"];
            }
            Scene::Rules => {
                m.text = "tr_rules";
                m.targets = &["next"];
            }
            Scene::Prizes => {
                m.text = "tr_prizes";
                m.targets = &["next"];
            }
            _ => {}
        },
        "presenter" if scene == Scene::Dashboard => {
            m.text = "tr_presenter";
            m.actions = Actions::Presenter;
        }
        "questions" | "answers" | "leaderboard" if scene == Scene::Dashboard => {
            (m.text, m.targets) = match phase {
                "questions" => ("tr_dashboard", &["questions", "round"][..]),
                "answers" => ("tr_answers", &["answers", "round"][..]),
                _ => ("tr_leaderboard", &["leaderboard"][..]),
            };
        }
        "questions_back" | "answers_back" if scene == Scene::Game => {
            m.text = if question.q_index < 0 {
                "tr_round_card"
            } else if phase == "questions_back" {
                "tr_questions"
            } else if kind.is_some_and(|k| {
                question.step >= quiz_core::quiz::validate::preview_step(k, "answer")
            }) {
                "tr_answer_shown"
            } else {
                "tr_answer_reveal"
            };
        }
        "leaderboard_back" if scene == Scene::Leaderboard => {
            m.text = if lb_step == 0 {
                "tr_lb_start"
            } else if lb_step % 2 == 1 {
                "tr_lb_card"
            } else {
                "tr_lb_points"
            };
        }
        _ => {}
    }
    m
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}
pub fn highlight(r: Rect) -> Rect {
    Rect {
        left: r.left - 8.0,
        top: r.top - 8.0,
        width: r.width + 16.0,
        height: r.height + 16.0,
    }
}
/// React uses right-4 bottom-14, not target-relative placement or collision avoidance.
pub fn bubble_rect(viewport: (f64, f64), height: f64) -> Rect {
    let width = 380.0_f64.min((viewport.0 - 32.0).max(0.0));
    Rect {
        left: viewport.0 - 16.0 - width,
        top: viewport.1 - 56.0 - height,
        width,
        height,
    }
}
pub fn read_tour() -> Option<Tour> {
    let raw = window()
        .local_storage()
        .ok()
        .flatten()?
        .get_item(TOUR_KEY)
        .ok()
        .flatten();
    decode_tour(raw.as_deref())
}
pub fn write_tour(tour: Option<&Tour>) {
    if let Ok(Some(storage)) = window().local_storage() {
        if let Some(json) = encode_tour(tour) {
            let _ = storage.set_item(TOUR_KEY, &json);
        } else {
            let _ = storage.remove_item(TOUR_KEY);
        }
    }
    if let Ok(event) = web_sys::Event::new(TOUR_EVENT) {
        let _ = window().dispatch_event(&event);
    }
}
pub fn start_tour(quiz_id: &str) {
    write_tour(Some(&Tour::start(quiz_id.into())));
    let encoded = js_sys::encode_uri_component(quiz_id)
        .as_string()
        .unwrap_or_default();
    let _ = window().location().set_hash(&format!("/quiz/{encoded}"));
}
pub fn end_tour() {
    write_tour(None);
}
fn target_rect(targets: &[&str]) -> Option<Rect> {
    // First existing element, then width check: a zero-width button does not fall
    // through to its card, matching JS's find(Boolean).
    let el = targets.iter().find_map(|name| {
        document()
            .query_selector(&format!("[data-tour=\"{name}\"]"))
            .ok()
            .flatten()
    })?;
    let r = el.get_bounding_client_rect();
    (r.width() != 0.0).then(|| {
        highlight(Rect {
            left: r.left(),
            top: r.top(),
            width: r.width(),
            height: r.height(),
        })
    })
}
fn event_data(event: &web_sys::MessageEvent) -> Option<serde_json::Value> {
    serde_json::from_str(&js_sys::JSON::stringify(&event.data()).ok()?.as_string()?).ok()
}
/// Same inputs as React. Translations default to use_t(). Scene keyboard shortcuts
/// remain with the scenes; native buttons support keyboard activation.
#[component]
pub fn TourGuide(
    #[prop(into)] quiz_id: Signal<String>,
    #[prop(into)] scene: Signal<Scene>,
    #[prop(into)] active_round: Signal<Option<Round>>,
    #[prop(into)] is_presenter: Signal<bool>,
    #[prop(optional)] t: Option<crate::i18n::T>,
) -> impl IntoView {
    let t = t.unwrap_or_else(crate::i18n::use_t);
    let tour = RwSignal::new(read_tour());
    let active = Memo::new(move |_| tour.get().is_some_and(|v| v.active_for(&quiz_id.get())));
    let question = RwSignal::new(QuestionProgress::default());
    let lb_step = RwSignal::new(0_usize);
    let presenter_seen = RwSignal::new(false);
    let ring = RwSignal::new(None::<Rect>);
    let sync = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| tour.set(read_tour()));
    for event in ["storage", TOUR_EVENT] {
        let _ = window().add_event_listener_with_callback(event, sync.as_ref().unchecked_ref());
    }
    let sync = StoredValue::new_local(Some(sync));
    on_cleanup(move || {
        sync.update_value(|s| {
            if let Some(s) = s.take() {
                for event in ["storage", TOUR_EVENT] {
                    let _ = window()
                        .remove_event_listener_with_callback(event, s.as_ref().unchecked_ref());
                }
            }
        });
    });
    Effect::new(move |_| {
        let _ = scene.get();
        question.set(QuestionProgress::default());
        lb_step.set(0);
    });
    // Separate channel objects hear scene messages posted in this window, too.
    Effect::new(move |_| {
        if !active.get() {
            return;
        }
        let mut subscriptions = Vec::new();
        // React uses the global app channel; the Rust shell now scopes it by quiz.
        // Listen to both so presenter discovery works across either implementation.
        for (name, channel_name) in [
            ("quiz-question-sync", "quiz-question-sync".to_owned()),
            ("quiz-leaderboard-sync", "quiz-leaderboard-sync".to_owned()),
            ("quiz-app-sync", "quiz-app-sync".to_owned()),
            ("quiz-app-sync", format!("quiz-app-sync-{}", quiz_id.get())),
        ] {
            if let Ok(channel) = web_sys::BroadcastChannel::new(&channel_name) {
                let listener = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(
                    move |e: web_sys::MessageEvent| {
                        let Some(v) = event_data(&e) else {
                            return;
                        };
                        let payload = &v["payload"];
                        match (name, v["type"].as_str()) {
                            ("quiz-question-sync", Some("Q_UPDATE")) => {
                                question.set(QuestionProgress {
                                    q_index: payload["qIndex"].as_i64().unwrap_or(-1),
                                    step: payload["step"].as_u64().unwrap_or(0) as usize,
                                })
                            }
                            ("quiz-leaderboard-sync", Some("LB_UPDATE" | "LB_STATE")) => {
                                lb_step.set(payload["revealStep"].as_u64().unwrap_or(0) as usize)
                            }
                            _ => {}
                        }
                        if name == "quiz-app-sync" && v["sender"] == "presenter" {
                            presenter_seen.set(true);
                        }
                    },
                );
                channel.set_onmessage(Some(listener.as_ref().unchecked_ref()));
                subscriptions.push((channel, listener));
            }
        }
        let subscriptions = StoredValue::new_local(Some(subscriptions));
        on_cleanup(move || {
            subscriptions.update_value(|s| {
                if let Some(s) = s.take() {
                    for (channel, _) in s {
                        channel.set_onmessage(None);
                        channel.close();
                    }
                }
            })
        });
    });
    let visited = StoredValue::new((String::new(), Vec::<Scene>::new()));
    Effect::new(move |_| {
        let current_scene = scene.get();
        let presenter = is_presenter.get() || presenter_seen.get();
        if !active.get() {
            return;
        }
        let Some(mut current) = tour.get() else {
            return;
        };
        visited.update_value(|(phase, scenes)| {
            if *phase != current.phase {
                *phase = current.phase.clone();
                scenes.clear();
            }
            if !scenes.contains(&current_scene) {
                scenes.push(current_scene);
            }
            let next = next_phase(&current.phase, current_scene, presenter, scenes).to_owned();
            if next != current.phase {
                current.phase = next;
                write_tour(Some(&current));
            }
        });
    });
    let msg = Memo::new(move |_| {
        let phase = tour.get().map(|t| t.phase).unwrap_or_default();
        let progress = question.get();
        let round = active_round.get();
        let kind = round
            .as_ref()
            .and_then(|r| {
                usize::try_from(progress.q_index)
                    .ok()
                    .and_then(|i| r.questions.get(i))
            })
            .map(|q| {
                serde_json::to_value(q)
                    .ok()
                    .and_then(|v| v["type"].as_str().map(str::to_owned))
                    .unwrap_or_default()
            });
        message(
            &phase,
            scene.get(),
            progress,
            lb_step.get(),
            kind.as_deref(),
        )
    });
    Effect::new(move |_| {
        let targets = if active.get() { msg.get().targets } else { &[] };
        ring.set(target_rect(targets));
        if targets.is_empty() {
            return;
        }
        let measure = move || ring.set(target_rect(targets));
        let first = gloo_timers::callback::Timeout::new(0, measure);
        let timer = gloo_timers::callback::Interval::new(300, measure);
        let resize = window_event_listener(leptos::ev::resize, move |_| measure());
        let resources = StoredValue::new_local(Some((first, timer, resize)));
        on_cleanup(move || {
            resources.update_value(|r| {
                if let Some((first, timer, resize)) = r.take() {
                    drop((first, timer));
                    resize.remove();
                }
            })
        });
    });
    let skip = move |_| {
        if let Some(mut current) = tour.get_untracked() {
            current.phase = "answers".into();
            write_tour(Some(&current));
        }
    };
    view! {
        {move || ring.get().filter(|_| active.get()).map(|r| view! {
            <div aria-hidden="true" class="tour-highlight fixed z-[290] pointer-events-none rounded-2xl ring-4 ring-yellow-400 animate-pulse shadow-[0_0_40px_rgba(250,204,21,0.6)]" style=format!("left:{}px;top:{}px;width:{}px;height:{}px", r.left, r.top, r.width, r.height)/>
        })}
        <crate::presence::Presence when=Signal::derive(move || active.get()) enter="tour-enter" exit="tour-exit" class="fixed inset-0 z-[300] pointer-events-none">
            <div role="region" aria-label=move || t.get("tr_title") class="pointer-events-auto fixed z-[300] right-4 bottom-14 w-[min(380px,calc(100vw-32px))] rounded-2xl bg-[#0f0f0f]/95 backdrop-blur border border-yellow-500/50 shadow-2xl p-5 space-y-3 font-sans text-white">
                <div class="flex items-center gap-2">
                    <TourIcon kind="compass"/>
                    <span class="text-xs font-bold uppercase tracking-widest text-yellow-400">{move || t.get("tr_title")}</span>
                    <span class="text-xs text-gray-400">{move || t.fmt("tr_step", &[("n", &tour_step(&tour.get().map(|v| v.phase).unwrap_or_default()).to_string()), ("total", "6")])}</span>
                    <button on:click=move |_| end_tour() class="ml-auto p-1.5 rounded-full text-gray-400 hover:text-white hover:bg-white/10" aria-label=move || t.get("tr_end") title=move || t.get("tr_end")><TourIcon kind="x"/></button>
                </div>
                <div class="flex gap-1">{(0..TOUR_STEPS).map(move |i| view! {
                    <span class=move || if i < tour_step(&tour.get().map(|v| v.phase).unwrap_or_default()) { "h-1 flex-1 rounded-full bg-yellow-400" } else { "h-1 flex-1 rounded-full bg-white/10" }/>
                }).collect_view()}</div>
                <p class="text-sm leading-relaxed text-gray-200" aria-live="polite">{move || rich_keys(t.get(msg.get().text))}</p>
                <Show when=move || msg.get().actions == Actions::Presenter>
                    <button on:click=skip class="text-sm text-gray-400 hover:text-white underline underline-offset-4">{move || t.get("tr_presenter_skip")}</button>
                </Show>
                <Show when=move || msg.get().actions == Actions::Done>
                    <div class="flex flex-wrap gap-2">
                        <button on:click=move |_| { end_tour(); let _ = window().location().set_hash("/"); } class="flex items-center gap-2 px-4 py-2 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold text-sm"><TourIcon kind="plus"/>{move || t.get("tr_done_new")}</button>
                        <button on:click=move |_| end_tour() class="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">{move || t.get("tr_end")}</button>
                    </div>
                </Show>
            </div>
        </crate::presence::Presence>
    }
}
fn rich_keys(text: String) -> impl IntoView {
    text.split('`').enumerate().map(|(i, part)| {
        if i % 2 == 1 { view! { <code class="px-1.5 py-0.5 rounded bg-white/10 text-yellow-200 font-mono text-[0.9em] whitespace-nowrap">{part.to_owned()}</code> }.into_any() }
        else { part.to_owned().into_any() }
    }).collect_view()
}
#[component]
fn TourIcon(kind: &'static str) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" width=if kind == "compass" {18} else {16} height=if kind == "compass" {18} else {16} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" class=if kind == "compass" {"text-yellow-400"} else {""}>
        {match kind {
            "compass" => view! { <circle cx="12" cy="12" r="10"/><path d="m16.24 7.76-2.12 6.36-6.36 2.12 2.12-6.36z"/> }.into_any(),
            "plus" => view! { <path d="M12 5v14M5 12h14"/> }.into_any(),
            _ => view! { <path d="m18 6-12 12M6 6l12 12"/> }.into_any(),
        }}
    </svg> }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Scene::*;
    const PHASES: &[&str] = &[
        "intro",
        "questions",
        "questions_back",
        "presenter",
        "answers",
        "answers_back",
        "leaderboard",
        "leaderboard_back",
        "done",
        "unknown",
    ];
    #[test]
    fn every_phase_and_scene_transition() {
        // Explicit transition table, including the presenter shortcut.
        let transitions = [
            ("intro", Dashboard, false, vec![], "questions"),
            ("questions", Game, false, vec![], "questions_back"),
            ("questions_back", Dashboard, false, vec![Game], "presenter"),
            ("questions_back", Dashboard, true, vec![Game], "answers"),
            ("presenter", Dashboard, true, vec![], "answers"),
            ("presenter", Game, true, vec![], "answers"),
            ("answers", Game, false, vec![], "answers_back"),
            ("answers_back", Dashboard, false, vec![Game], "leaderboard"),
            (
                "leaderboard",
                Leaderboard,
                false,
                vec![],
                "leaderboard_back",
            ),
            (
                "leaderboard_back",
                Dashboard,
                false,
                vec![Leaderboard],
                "done",
            ),
        ];
        for (phase, scene, presenter, visited, expected) in transitions {
            assert_eq!(next_phase(phase, scene, presenter, &visited), expected);
        }
        // A dashboard left behind in the other window cannot advance a back phase.
        for phase in ["questions_back", "answers_back", "leaderboard_back"] {
            for presenter in [false, true] {
                assert_eq!(next_phase(phase, Dashboard, presenter, &[Dashboard]), phase);
                assert_eq!(
                    next_phase(phase, Pause, presenter, &[Game, Leaderboard]),
                    phase
                );
            }
        }
        for phase in PHASES {
            for scene in [Welcome, Rules, Prizes, Pause] {
                assert_eq!(next_phase(phase, scene, false, &[]), *phase);
            }
        }
    }
    #[test]
    fn numbered_steps() {
        for (phase, expected) in PHASES.iter().zip([1, 2, 2, 3, 4, 4, 5, 5, 6, 1]) {
            assert_eq!(tour_step(phase), expected);
        }
        assert_eq!(TOUR_STEPS, 6);
    }
    #[test]
    fn messages_and_ordered_targets() {
        let q = QuestionProgress::default();
        for (phase, scene, text, targets, actions) in [
            (
                "intro",
                Welcome,
                "tr_welcome",
                &["start"][..],
                Actions::None,
            ),
            ("intro", Rules, "tr_rules", &["next"][..], Actions::None),
            ("intro", Prizes, "tr_prizes", &["next"][..], Actions::None),
            (
                "questions",
                Dashboard,
                "tr_dashboard",
                &["questions", "round"][..],
                Actions::None,
            ),
            (
                "answers",
                Dashboard,
                "tr_answers",
                &["answers", "round"][..],
                Actions::None,
            ),
            (
                "leaderboard",
                Dashboard,
                "tr_leaderboard",
                &["leaderboard"][..],
                Actions::None,
            ),
            (
                "presenter",
                Dashboard,
                "tr_presenter",
                &[][..],
                Actions::Presenter,
            ),
            (
                "questions_back",
                Game,
                "tr_round_card",
                &[][..],
                Actions::None,
            ),
            (
                "answers_back",
                Game,
                "tr_round_card",
                &[][..],
                Actions::None,
            ),
        ] {
            assert_eq!(
                message(phase, scene, q, 0, None),
                Message {
                    text,
                    targets,
                    actions
                }
            );
        }
        for scene in [Welcome, Rules, Prizes, Dashboard, Game, Leaderboard, Pause] {
            assert_eq!(message("done", scene, q, 0, None).actions, Actions::Done);
            assert_eq!(message("done", scene, q, 0, None).text, "tr_done");
            assert_eq!(
                message("unknown", scene, q, 0, None).text,
                "tr_back_to_dashboard"
            );
        }
        for phase in PHASES.iter().filter(|p| **p != "done") {
            assert_eq!(
                message(phase, Pause, q, 0, None).text,
                "tr_back_to_dashboard"
            );
        }
        for phase in ["questions", "answers", "leaderboard", "presenter"] {
            assert_eq!(
                message(phase, Game, q, 0, None).text,
                "tr_back_to_dashboard"
            );
        }
        for phase in [
            "questions_back",
            "answers_back",
            "leaderboard_back",
            "intro",
        ] {
            assert_eq!(
                message(phase, Dashboard, q, 0, None).text,
                "tr_back_to_dashboard"
            );
        }
    }
    #[test]
    fn question_and_leaderboard_progress() {
        for (kind, threshold) in [
            ("Written", 2),
            ("PImage", 3),
            ("PVideo", 3),
            ("Top5", 2),
            ("Sort", 2),
        ] {
            for step in 0..5 {
                let q = QuestionProgress { q_index: 0, step };
                assert_eq!(
                    message("answers_back", Game, q, 0, Some(kind)).text,
                    if step >= threshold {
                        "tr_answer_shown"
                    } else {
                        "tr_answer_reveal"
                    }
                );
                assert_eq!(
                    message("questions_back", Game, q, 0, Some(kind)).text,
                    "tr_questions"
                );
                assert_eq!(
                    message("answers_back", Game, q, 0, None).text,
                    "tr_answer_reveal"
                );
            }
        }
        for (step, text) in [
            "tr_lb_start",
            "tr_lb_card",
            "tr_lb_points",
            "tr_lb_card",
            "tr_lb_points",
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(
                message(
                    "leaderboard_back",
                    Leaderboard,
                    QuestionProgress::default(),
                    step,
                    None
                )
                .text,
                *text
            );
        }
    }
    #[test]
    fn shared_storage_format_and_explicit_restart() {
        assert_eq!(TOUR_KEY, "quiznight.tour");
        assert_eq!(TOUR_EVENT, "quiznight:tour");
        for raw in [
            None,
            Some("null"),
            Some(""),
            Some("bad JSON"),
            Some("false"),
        ] {
            assert_eq!(decode_tour(raw), None);
        }
        let mut tour =
            decode_tour(Some(r#"{"quizId":"demo","phase":"answers","future":7}"#)).unwrap();
        assert!(tour.active_for("demo"));
        assert!(!tour.active_for("other"));
        tour.phase = "done".into();
        let encoded = encode_tour(Some(&tour)).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&encoded).unwrap(),
            serde_json::json!({"quizId":"demo","phase":"done","future":7})
        );
        assert_eq!(encode_tour(None), None);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                &encode_tour(Some(&Tour::start("demo".into()))).unwrap()
            )
            .unwrap(),
            serde_json::json!({"quizId":"demo","phase":"intro"})
        );
    }
    #[test]
    fn viewport_placement_and_scaled_target_padding() {
        assert_eq!(
            highlight(Rect {
                left: 100.,
                top: 20.,
                width: 80.,
                height: 40.
            }),
            Rect {
                left: 92.,
                top: 12.,
                width: 96.,
                height: 56.
            }
        );
        assert_eq!(
            bubble_rect((1920., 1080.), 200.),
            Rect {
                left: 1524.,
                top: 824.,
                width: 380.,
                height: 200.
            }
        );
        assert_eq!(
            bubble_rect((320., 600.), 200.),
            Rect {
                left: 16.,
                top: 344.,
                width: 288.,
                height: 200.
            }
        );
        assert_eq!(bubble_rect((320., 100.), 200.).top, -156.); // JS has no vertical clamp.
    }
}
