//! Isolated audience scenes: no show reducer, channel, or navigation side effects.
use leptos::prelude::*;
use quiz_core::{
    quiz::{
        model::{Quiz, resolve_teams},
        parse::{is_round_hidden, parse_questions_csv},
    },
    show::{PlayMode, Role, Scene, ShowState},
};
use std::sync::Arc;

#[component]
pub(super) fn Scenes(
    mut quiz: Quiz,
    #[prop(optional)] wall: bool,
    scene: RwSignal<String>,
    resolve: Callback<String, Option<String>>,
) -> impl IntoView {
    if wall {
        quiz.settings.welcome_layout = "wall".into();
        quiz.settings.intro_pause = 1200;
    }
    let rounds = Arc::new(parse_questions_csv(&quiz.questions_csv));
    let teams = if quiz.teams.iter().any(|t| !t.name.trim().is_empty()) {
        quiz.teams.clone()
    } else {
        quiz_core::quiz::demo_teams::demo_teams()
    };
    let context = crate::quiz_context::QuizContext {
        teams: Arc::new(resolve_teams(&teams, |name| resolve.run(name.into()))),
        rounds: rounds.clone(),
        quiz: Arc::new(quiz),
        media_urls: Default::default(),
    };
    crate::i18n::provide_translations(Signal::stored(crate::i18n::Lang::from_code(
        &context.quiz.settings.language,
    )));
    provide_context(context.clone());
    // Scenes read the live settings signal; the preview never changes them.
    let settings = RwSignal::new(context.quiz.settings.clone());
    provide_context(settings);
    provide_context(crate::settings::SettingsContext {
        settings,
        update: Callback::new(|_| ()),
    });

    provide_context(crate::show::ShowContext {
        role: Role::Main,
        state: RwSignal::new(ShowState {
            scene: Scene::Welcome,
            active_round_id: None,
            play_mode: PlayMode::WithAnswers,
        }),
        dispatch: Callback::new(|_| ()),
    });
    provide_context(crate::show::ShowUi {
        settings_open: RwSignal::new(false),
        calibrating: RwSignal::new(false),
    });
    provide_context(Signal::stored(crate::stage::StageLayout {
        w: 2400.,
        h: 1350.,
        scale: 1.,
    }));
    let rounds = StoredValue::new(rounds);
    view! { <div class="h-full w-full pointer-events-none" inert=true>
        {move || match scene.get().as_str() {
            "rules" => view! { <crate::scenes::rules::Rules/> }.into_any(),
            "prizes" => view! { <crate::scenes::prizes::Prizes/> }.into_any(),
            "dashboard" => view! { <crate::scenes::dashboard::Dashboard/> }.into_any(),
            "leaderboard" => view! { <Standings/> }.into_any(),
            "pause" => view! { <Standings pause=true/> }.into_any(),
            "round" | "question" | "answer" => {
                let round = rounds.with_value(|rounds| rounds.iter().find(|r| !is_round_hidden(r) && !r.questions.is_empty()).cloned());
                round.map(|round| {
                    let scene = scene.get_untracked();
                    let step = quiz_core::quiz::validate::preview_step(serde_json::to_value(&round.questions[0]).unwrap()["type"].as_str().unwrap_or(""), if scene == "answer" { "answer" } else { "question" });
                    view! { <crate::scenes::game::QuestionPreview round=round question_index=if scene == "round" { None } else { Some(0) } step=step resolve_media=resolve/> }.into_any()
                }).unwrap_or_else(|| view! { <crate::scenes::dashboard::Dashboard/> }.into_any())
            },
            _ => view! { <crate::scenes::welcome::Welcome/> }.into_any(),
        }}
    </div> }
}

#[component]
pub(super) fn CelebrationPreview(trigger: RwSignal<u64>) -> impl IntoView {
    let theme = crate::theme::use_theme();
    // The particle canvas reads the stage size; the preview has no `Stage` above it.
    provide_context(Signal::stored(crate::stage::StageLayout {
        w: 2400.,
        h: 1350.,
        scale: 1.,
    }));
    let active = RwSignal::new(false);
    let fire = RwSignal::new(0_u64);
    let fireworks = RwSignal::new(false);
    let decoration = RwSignal::new(false);
    let generation = RwSignal::new(0_u64);
    Effect::new(move |_| {
        let rev = trigger.get();
        if rev == 0 {
            return;
        }
        generation.set(rev);
        let theme = theme.get_untracked();
        let kind = theme.celebration.r#type;
        let storm = kind == Some(quiz_core::theme::format::Celebration::Decoration)
            && theme.decoration.r#type.is_some();
        decoration.set(storm);
        fireworks.set(kind == Some(quiz_core::theme::format::Celebration::Fireworks));
        if !storm {
            fire.update(|n| *n += 1);
        }
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(8000).await;
            if generation.try_get_untracked() == Some(rev) {
                decoration.set(false);
            }
        });
    });
    view! {
        <div class="absolute inset-0 z-40 pointer-events-none">
            <crate::particles::ParticleCanvas active=active trigger=fire fireworks=fireworks multiplier=RwSignal::new(1_usize) count=RwSignal::new(0_usize) frame_ms=RwSignal::new(0.)/>
            <Show when=move || decoration.get()><crate::scenes::backdrop::ThemeDecorations density=3./></Show>
        </div>
    }
}

/// A static final result, deliberately separate from the live reveal's BroadcastChannel.
#[component]
fn Standings(#[prop(optional)] pause: bool) -> impl IntoView {
    let t = crate::i18n::use_t();
    let quiz = crate::quiz_context::use_quiz();
    let theme = crate::theme::use_theme();
    let mut board = quiz_core::quiz::scores::manual_leaderboard(
        &quiz.quiz.teams,
        &quiz.rounds,
        &quiz.quiz.manual_scores,
        Some(&t.get("round")),
    );
    if !quiz_core::quiz::scores::has_points(&board) {
        let fallback = quiz
            .teams
            .iter()
            .map(|team| team.team.name.clone())
            .collect::<Vec<_>>();
        board = quiz_core::quiz::scores::sample_leaderboard(
            &quiz.quiz.teams,
            &board.rounds,
            Some(&t.get("round")),
            &fallback,
        );
    }
    let mut rows = board
        .teams
        .into_iter()
        .map(|team| {
            let total = team.scores.values().sum::<f64>();
            (team.name, total)
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut previous = None;
    let mut rank = 0;
    let rows = rows
        .into_iter()
        .enumerate()
        .map(|(index, (name, total))| {
            if previous != Some(total) {
                rank = index + 1;
                previous = Some(total);
            }
            super::standings::Row { name, total, rank }
        })
        .collect::<Vec<_>>();
    let rows = StoredValue::new(rows);

    view! {
        <div class="relative h-full w-full bg-ground font-show flex flex-col items-center justify-center p-24">
            <crate::scenes::backdrop::SceneBackdrop quiet=!pause/>
            <h1 class="relative z-10 font-title text-8xl font-black uppercase text-white mb-12">{t.get(if pause { "pause_title" } else { "lb_title" })}</h1>
            {pause.then(|| view! { <p class="relative z-10 text-accent-400 text-8xl font-black mb-10">"10:00"</p> })}
            {move || {
                let layout = theme.get().scenes.leaderboard.map(|s| s.layout).unwrap_or(quiz_core::theme::format::LeaderboardLayout::Table);
                view! {
                    <div class="relative z-10 w-full flex-1 min-h-0">
                        <super::standings::Board rows=rows.get_value() layout=if pause { quiz_core::theme::format::LeaderboardLayout::Table } else { layout }/>
                    </div>
                }

            }}
        </div>
    }
}
