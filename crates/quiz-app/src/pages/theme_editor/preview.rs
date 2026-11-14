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
    let t = crate::i18n::use_t();
    let mut scores = quiz_core::quiz::scores::manual_leaderboard(
        &context.quiz.teams,
        &context.rounds,
        &context.quiz.manual_scores,
        Some(&t.get("round")),
    );
    if !quiz_core::quiz::scores::has_points(&scores) {
        let fallback = context
            .teams
            .iter()
            .map(|team| team.team.name.clone())
            .collect::<Vec<_>>();
        scores = quiz_core::quiz::scores::sample_leaderboard(
            &context.quiz.teams,
            &scores.rounds,
            Some(&t.get("round")),
            &fallback,
        );
    }
    let scores = StoredValue::new(scores);
    let teams = StoredValue::new(context.teams.as_ref().clone());
    let theme = crate::theme::use_theme();
    let rounds = StoredValue::new(rounds);
    view! { <div class="h-full w-full pointer-events-none" inert=true>
        {move || match scene.get().as_str() {
            "rules" => view! { <crate::scenes::rules::Rules/> }.into_any(),
            "prizes" => view! { <crate::scenes::prizes::Prizes/> }.into_any(),
            "dashboard" => view! { <crate::scenes::dashboard::Dashboard/> }.into_any(),
            "leaderboard" => view! { <crate::scenes::leaderboard::LeaderboardPreview scores=scores.get_value() teams=teams.get_value() step=usize::MAX layout=theme.get().scenes.leaderboard.as_ref().map_or(quiz_core::theme::format::LeaderboardLayout::Table, |s| s.layout)/> }.into_any(),
            "pause" => view! { <crate::scenes::pause::PausePreview scores=scores.get_value() teams=teams.get_value()/> }.into_any(),
            "round" | "question" | "answer" => {
                let round = rounds.with_value(|rounds| rounds.iter().find(|r| !is_round_hidden(r) && !r.questions.is_empty()).cloned());
                round.map(|round| {
                    let scene = scene.get_untracked();
                    let step = quiz_core::quiz::validate::preview_step(serde_json::to_value(&round.questions[0]).unwrap()["type"].as_str().unwrap_or(""), if scene == "answer" { "answer" } else { "question" });
                    view! { <crate::scenes::game::QuestionPreview round=round question_index=if scene == "round" { None } else { Some(0) } step=step resolve_media=resolve/> }.into_any()
                }).unwrap_or_else(|| view! { <crate::scenes::dashboard::Dashboard/> }.into_any())
            },
            _ => view! { <crate::scenes::welcome::WelcomePreview wall/> }.into_any(),
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
