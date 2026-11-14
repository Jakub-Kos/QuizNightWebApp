//! Port of WelcomeScreen.jsx (shared audience/presenter welcome): the show's title, start time and
//! countdown take turns in the header while big team cards cycle, each unfolding its stats.
//! Framer Motion animations are CSS animations in styles/welcome.css; springs become `linear()`
//! easings passed in custom properties (see `spring_vars`).
use super::team_avatar::{TeamAvatar, team_color};
use super::team_stats::{TeamDetails, WelcomeIcon};
use super::team_wall::TeamWall;
use crate::{
    i18n::use_t, presence::PresenceSwitch, quiz_context::use_quiz, show::use_show, spring::Spring,
};
use leptos::leptos_dom::helpers::{
    IntervalHandle, TimeoutHandle, set_interval_with_handle, set_timeout_with_handle,
};
use leptos::prelude::*;
use quiz_core::{quiz::model::ResolvedTeam, show::Action};
use std::{sync::LazyLock, time::Duration};
use wasm_bindgen::{JsCast, closure::Closure};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Cards,
    Wall,
}

/// Cards suit teams with photos, mottos or past results; teams known only by name get the team
/// wall. `setting` is settings.welcomeLayout: "auto" (default), "cards" or "wall".
pub fn choose_layout(teams: &[ResolvedTeam], setting: &str) -> Layout {
    match setting {
        "cards" => Layout::Cards,
        "wall" => Layout::Wall,
        _ => {
            let rich = teams
                .iter()
                .filter(|t| {
                    t.image.is_some()
                        || t.team.quote.as_ref().is_some_and(|q| !q.is_empty())
                        || !t.team.is_new.unwrap_or(false)
                })
                .count();
            // rich < teams.length / 2
            if rich * 2 < teams.len() {
                Layout::Wall
            } else {
                Layout::Cards
            }
        }
    }
}

/// `config?.x || default`: a missing or zero setting takes the default.
fn or_default(value: u64, default: u64) -> u64 {
    if value == 0 { default } else { value }
}

/// One of the two faded cards stacked behind the current one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StackCard {
    pub offset: usize,
    pub team: usize,
    /// Tells apart two stack cards of the same team (a quiz with a single team)
    pub dup: usize,
}

/// The cards behind team `active`: the one after next (offset 2) first, then the next one.
pub fn stack_cards(active: usize, count: usize) -> Vec<StackCard> {
    if count == 0 {
        return Vec::new();
    }
    let mut cards: Vec<StackCard> = Vec::new();
    for offset in [2, 1] {
        let team = (active + offset) % count;
        let dup = cards.iter().filter(|c| c.team == team).count();
        cards.push(StackCard { offset, team, dup });
    }
    cards
}

/// The pose a stack card animates to (Framer's animate prop), as CSS.
pub fn stack_style(offset: usize) -> String {
    let o = offset as f64;
    format!(
        "translate: {}px {}px; scale: {}; z-index: {}; opacity: {}",
        o * 20.0,
        o * -30.0,
        1.0 - o * 0.05,
        10 - offset as i32,
        ((0.4 - o * 0.1) * 100.0).round() / 100.0
    )
}

/// JavaScript string length (UTF-16 code units), which the size rules count.
fn js_len(s: &str) -> usize {
    s.encode_utf16().count()
}

pub fn name_size(name: &str) -> &'static str {
    if js_len(name) > 15 {
        "text-5xl"
    } else {
        "text-7xl"
    }
}

/// One Tailwind size smaller than the name, smaller still for a long quote.
pub fn quote_size(name: &str, quote: &str) -> &'static str {
    let len = js_len(quote);
    if len > 120 {
        "text-3xl"
    } else if len > 50 || js_len(name) > 15 {
        "text-4xl"
    } else {
        "text-5xl"
    }
}

/// What the countdown header says.
#[derive(Clone, Debug, PartialEq)]
pub enum Countdown {
    /// No start time: the text stays empty
    Unset,
    Now,
    Hours(i64, i64),
    Minutes(i64, i64),
    /// A start time JavaScript cannot read (`Number()` gives NaN)
    Invalid,
}

/// JavaScript's `Number(part)` for a part of "HH:MM".
fn js_number(part: &str) -> Option<f64> {
    let p = part.trim();
    if p.is_empty() {
        return Some(0.0);
    }
    p.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Time left until `start` ("HH:MM", today) at `now_ms` milliseconds after local midnight.
pub fn countdown(start: &str, now_ms: f64) -> Countdown {
    if start.is_empty() {
        return Countdown::Unset;
    }
    let mut parts = start.split(':');
    let hours = parts.next().and_then(js_number);
    let minutes = parts.next().and_then(js_number);
    let (Some(h), Some(m)) = (hours, minutes) else {
        return Countdown::Invalid;
    };
    let target = h.trunc() * 3_600_000.0 + m.trunc() * 60_000.0;
    let diff = target - now_ms;
    if diff <= 0.0 {
        return Countdown::Now;
    }
    let h = (diff / 3_600_000.0).floor() as i64;
    let m = ((diff % 3_600_000.0) / 60_000.0).floor() as i64;
    let s = ((diff % 60_000.0) / 1000.0).floor() as i64;
    if h > 0 {
        Countdown::Hours(h, m)
    } else {
        Countdown::Minutes(m, s)
    }
}

/// Which header item shows: 0 title, 1 start time, 2 countdown.
pub fn header_item(state: u64, show_time: bool) -> u64 {
    if show_time { state % 3 } else { 0 }
}

/// The springs of WelcomeScreen.jsx as CSS custom properties (Framer's resolution: a spring with
/// `duration` and `bounce` lasts `duration`; physics springs last until they settle).
pub fn spring_vars() -> String {
    let physics = |name: &str, spring: Spring| {
        format!(
            "--wl-{name}: {}; --wl-{name}-d: {:.3}s;",
            spring.to_css_linear(),
            spring.settle_time()
        )
    };
    [
        // type: "spring", bounce: 0.2, duration: 0.8 (card)
        format!(
            "--wl-card: {};",
            Spring::from_bounce(0.2, 0.8).to_css_linear()
        ),
        // type: "spring", bounce: 0.3, duration: 0.8 (header items, details panel)
        format!(
            "--wl-panel: {};",
            Spring::from_bounce(0.3, 0.8).to_css_linear()
        ),
        // Framer's default for x/y
        physics("snap", Spring::new(500.0, 25.0, 1.0)),
        // Framer's default for scale
        physics("scale", Spring::new(550.0, 30.0, 1.0)),
        // type: "spring" without options
        physics("pop", Spring::new(100.0, 10.0, 1.0)),
    ]
    .concat()
}
static SPRING_VARS: LazyLock<String> = LazyLock::new(spring_vars);

/// Milliseconds since local midnight.
fn now_of_day() -> f64 {
    let d = js_sys::Date::new_0();
    ((d.get_hours() as f64 * 60.0 + d.get_minutes() as f64) * 60.0 + d.get_seconds() as f64)
        * 1000.0
        + d.get_milliseconds() as f64
}

const NO_TEAM_COLOR: &str = "from-purple-600 to-indigo-900";

#[component]
pub fn Welcome() -> impl IntoView {
    let q = use_quiz();
    let s = use_show();
    let t = use_t();
    let settings = crate::settings::use_settings().settings;
    // QuizShow.jsx renders the same WelcomeScreen for both roles, on the scaled stage.
    let presenter = s.role == quiz_core::show::Role::Presenter;
    // Teams marked absent at the attendance check are not introduced
    let teams: Vec<ResolvedTeam> = q
        .teams
        .iter()
        .filter(|t| t.team.present != Some(false))
        .cloned()
        .collect();
    let split_delay = Memo::new(move |_| settings.with(|s| or_default(s.split_delay, 3000)));
    let cycle_duration = Memo::new(move |_| settings.with(|s| or_default(s.cycle_duration, 8000)));
    let header_interval =
        Memo::new(move |_| settings.with(|s| or_default(s.header_interval, 5000)));
    let intro_pause = Memo::new(move |_| settings.with(|s| s.intro_pause));
    let show_time = Memo::new(move |_| settings.with(|s| s.show_time));
    let start_time = Memo::new(move |_| settings.with(|s| s.start_time.clone()));
    // Dev only: only the two supported values override the live setting.
    let layout_override = crate::quiz_context::query_param("welcome")
        .filter(|value| matches!(value.as_str(), "wall" | "cards"));
    let layout_teams = teams.clone();
    let layout = Memo::new(move |_| {
        settings.with(|s| {
            choose_layout(
                &layout_teams,
                layout_override.as_deref().unwrap_or(&s.welcome_layout),
            )
        })
    });
    let count = teams.len();
    let has_teams = count > 0;
    let teams = StoredValue::new(teams);

    let active = RwSignal::new(0usize);
    // Index of the card whose details are unfolded; any other card is shown centred
    let detail_for = RwSignal::new(None::<usize>);
    let exiting = RwSignal::new(false);
    let paused = RwSignal::new(false);
    let header_state = RwSignal::new(0u64);
    let countdown_now = RwSignal::new(Countdown::Unset);

    let timeouts = StoredValue::new(Vec::<TimeoutHandle>::new());
    let header_timer = StoredValue::new(None::<IntervalHandle>);
    let countdown_timer = StoredValue::new(None::<IntervalHandle>);
    let start_timer = StoredValue::new(None::<TimeoutHandle>);
    let clear_timeouts = move || {
        timeouts.update_value(|v| v.drain(..).for_each(|h| h.clear()));
    };

    // --- HEADER CYCLE ---
    Effect::new(move |_| {
        let enabled = show_time.get();
        let interval = header_interval.get();
        if let Some(h) = header_timer.get_value() {
            h.clear();
        }
        header_timer.set_value(if enabled {
            set_interval_with_handle(
                move || header_state.update(|n| *n = (*n + 1) % 3),
                Duration::from_millis(interval),
            )
            .ok()
        } else {
            None
        });
    });
    // --- COUNTDOWN ---
    Effect::new(move |_| {
        let start = start_time.get();
        let update = move || countdown_now.set(countdown(&start, now_of_day()));
        if let Some(h) = countdown_timer.get_value() {
            h.clear();
        }
        update();
        countdown_timer.set_value(set_interval_with_handle(update, Duration::from_secs(1)).ok());
    });
    // --- MAIN ANIMATION ---
    Effect::new(move |_| {
        let index = active.get();
        let leaving = exiting.get();
        let stopped = paused.get();
        let layout = layout.get();
        let split_delay = split_delay.get();
        let cycle_duration = cycle_duration.get();
        clear_timeouts();
        if leaving || layout == Layout::Wall || !has_teams {
            return;
        }
        let mut handles = Vec::new();
        // A new card starts centred (detail_for still points at the previous one) and always
        // unfolds its details after a delay, even if paused
        handles.extend(
            set_timeout_with_handle(
                move || detail_for.set(Some(index)),
                Duration::from_millis(split_delay),
            )
            .ok(),
        );
        // Only auto-cycle if not paused
        if !stopped && count > 1 {
            handles.extend(
                set_timeout_with_handle(
                    move || detail_for.set(None),
                    Duration::from_millis(cycle_duration.saturating_sub(600)),
                )
                .ok(),
            );
            handles.extend(
                set_timeout_with_handle(
                    move || active.update(|n| *n = (*n + 1) % count),
                    Duration::from_millis(cycle_duration),
                )
                .ok(),
            );
        }
        timeouts.set_value(handles);
    });
    on_cleanup(move || {
        if let Some(v) = timeouts.try_update_value(std::mem::take) {
            v.into_iter().for_each(|h| h.clear());
        }
        for timer in [header_timer, countdown_timer] {
            if let Some(Some(h)) = timer.try_get_value() {
                h.clear();
            }
        }
        if let Some(Some(h)) = start_timer.try_get_value() {
            h.clear();
        }
    });

    // --- HANDLERS ---
    let dispatch = s.dispatch;
    let start = move || {
        // A double click or held key must never schedule another scene transition.
        if exiting.get_untracked() {
            return;
        }
        exiting.set(true);
        start_timer.set_value(
            set_timeout_with_handle(
                move || dispatch.run(Action::Start),
                Duration::from_millis(800),
            )
            .ok(),
        );
    };
    let handle_start = move |_| start();
    let manual_select = move |index: usize| {
        active.set(index);
        paused.set(true); // Stop auto-rotation
    };

    // Keyboard shortcuts are scoped to the scene and leave focused controls/overlays alone.
    let ui = use_context::<crate::show::ShowUi>();
    let keys =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            if event.repeat()
                || event.alt_key()
                || event.ctrl_key()
                || event.meta_key()
                || event.shift_key()
                || exiting.get_untracked()
                || ui.is_some_and(|ui| {
                    ui.settings_open.get_untracked() || ui.calibrating.get_untracked()
                })
            {
                return;
            }
            if event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .is_some_and(|target| {
                    target
                        .closest(
                            "button, a, input, select, textarea, [contenteditable], [role=dialog]",
                        )
                        .ok()
                        .flatten()
                        .is_some()
                })
            {
                return;
            }
            match keyboard_action(&event.key(), layout.get_untracked(), count) {
                Some(WelcomeKey::Start) => start(),
                Some(WelcomeKey::TogglePause) => paused.update(|value| *value = !*value),
                Some(WelcomeKey::Select(delta)) => {
                    manual_select(relative_team(active.get_untracked(), count, delta))
                }
                None => return,
            }
            event.prevent_default();
        });
    let _ = window().add_event_listener_with_callback("keydown", keys.as_ref().unchecked_ref());
    let keys = StoredValue::new_local(Some(keys));
    on_cleanup(move || {
        if let Some(keys) = keys.try_update_value(Option::take).flatten() {
            let _ = window()
                .remove_event_listener_with_callback("keydown", keys.as_ref().unchecked_ref());
        }
    });

    let active_color = move || {
        if layout.get() == Layout::Wall {
            return NO_TEAM_COLOR;
        }
        teams.with_value(|v| {
            v.get(active.get())
                .map_or(NO_TEAM_COLOR, |team| team_color(team.color))
        })
    };
    let stack = Memo::new(move |_| stack_cards(active.get(), count));
    let details_key = move || match detail_for.get() {
        Some(i) if i == active.get() => teams.with_value(|v| {
            v.get(i)
                .filter(|team| !team.team.is_new.unwrap_or(false))
                .map_or(0, |_| i as u64 + 1)
        }),
        _ => 0,
    };
    let title = q.quiz.title.clone();

    view! {
        <div class=move || format!(
                "welcome-screen h-full w-full relative overflow-hidden flex flex-col bg-ground font-show{}",
                if exiting.get() { " welcome-leave" } else { "" })
            style=SPRING_VARS.as_str() data-role=if presenter { "presenter" } else { "main" }>

            <crate::scenes::backdrop::SceneBackdrop/>
            <div class=move || format!("absolute inset-0 opacity-20 bg-gradient-to-r {} blur-[150px] transition-colors duration-1000", active_color()) />

            <div class="relative w-full h-full flex flex-col isolate">

                // 1. ROTATING HEADER
                <div class="h-[20cqh] w-full relative z-50 perspective-1000 welcome-header">
                    <PresenceSwitch
                        scene=Signal::derive(move || header_item(header_state.get(), show_time.get()))
                        wait=Signal::derive(|| true)
                        enter="welcome-flip-enter" exit="welcome-flip-exit"
                        render=move |item| {
                            view! {
                                <div class="absolute inset-0 flex flex-col items-center justify-center">
                                    {match item {
                                        // text-shadow, not a drop-shadow filter: filters inside the 3D rotation flicker
                                        0 => view! {
                                            <h1 class="font-title text-8xl font-black text-white uppercase tracking-[0.1em]"
                                                style="text-shadow: 0 18px 40px rgba(0,0,0,0.45)">{move || t.get("title")}</h1>
                                        }.into_any(),
                                        1 => view! {
                                            <div class="flex flex-col items-center">
                                                <span class="text-3xl text-accent-500 font-bold tracking-[0.3em] mb-2 uppercase">{move || t.get("event_schedule")}</span>
                                                <div class="flex items-center gap-4 text-white font-mono text-6xl font-bold bg-white/[0.07] px-8 py-2 rounded-2xl border border-white/10">
                                                    <WelcomeIcon name="clock" size=48 class="text-accent-500"/>
                                                    <span>{move || start_time.get()}</span>
                                                </div>
                                            </div>
                                        }.into_any(),
                                        _ => view! {
                                            <div class="flex flex-col items-center">
                                                <span class="text-4xl text-green-500 font-bold tracking-[0.3em] mb-2 uppercase">{move || t.get("get_ready")}</span>
                                                <div class="flex items-center gap-4 text-white font-mono text-6xl font-bold bg-white/[0.07] px-8 py-2 rounded-2xl border border-white/10">
                                                    <WelcomeIcon name="timer" size=48 class="text-green-500"/>
                                                    <span>{move || match countdown_now.get() {
                                                        Countdown::Unset => String::new(),
                                                        Countdown::Now => t.get("happening_now"),
                                                        Countdown::Hours(h, m) => format!("{} {h} {} {m} {}", t.get("starts_in"), t.get("hr"), t.get("min")),
                                                        Countdown::Minutes(m, s) => format!("{} {m} {} {s} {}", t.get("starts_in"), t.get("min"), t.get("sec")),
                                                        Countdown::Invalid => format!("{} NaN {} NaN {}", t.get("starts_in"), t.get("min"), t.get("sec")),
                                                    }}</span>
                                                </div>
                                            </div>
                                        }.into_any(),
                                    }}
                                </div>
                            }
                        }/>
                </div>

                // 2. MAIN CONTENT
                <div class="h-[65cqh] w-full relative flex items-center pl-32 perspective-1000">
                    {(!has_teams).then(|| view! {
                        <h1 class="font-title w-full pr-32 text-center text-8xl font-black uppercase tracking-widest text-white/90">{title}</h1>
                    })}
                    <Show when=move || has_teams && layout.get() == Layout::Wall>
                        <div class="absolute inset-0"><TeamWall teams=teams.get_value()
                            intro_ms=Signal::derive(move || wall_intro_duration(cycle_duration.get()))
                            pause_ms=Signal::derive(move || intro_pause.get())/></div>
                    </Show>
                    <Show when=move || has_teams && layout.get() == Layout::Cards>
                        <div class="relative w-[600px] h-[800px] z-20 welcome-stack">
                            <For each=move || stack.get() key=|c| (c.team, c.dup) children=move |card| {
                                let key = (card.team, card.dup);
                                let offset = move || stack.with(|v| v.iter()
                                    .find(|c| (c.team, c.dup) == key).map_or(card.offset, |c| c.offset));
                                let color = teams.with_value(|v| team_color(v[card.team].color));
                                view! {
                                    <div class=format!("absolute inset-0 rounded-[3rem] bg-gradient-to-br {color} border border-white/10 shadow-2xl welcome-stack-card")
                                        style=move || stack_style(offset())/>
                                }
                            }/>

                            <PresenceSwitch scene=Signal::derive(move || active.get() as u64)
                                wait=Signal::derive(|| false) enter="welcome-card-enter" exit="welcome-card-exit"
                                render=move |index| {
                                    let team = teams.with_value(|v| v[index as usize].clone());
                                    view! { <TeamCard team=team/> }
                                }/>

                            <PresenceSwitch scene=Signal::derive(details_key)
                                wait=Signal::derive(|| false) enter="welcome-details-enter" exit="welcome-details-exit"
                                render=move |key| {
                                    (key > 0).then(|| {
                                        let team = teams.with_value(|v| v[key as usize - 1].clone());
                                        view! { <TeamDetails team=team/> }
                                    })
                                }/>
                        </div>

                        <div class="absolute right-24 top-10 bottom-32 w-[550px] z-10 flex flex-col justify-start">
                            <div class="flex justify-between items-center mb-4 border-b border-white/10 pb-2">
                                <h3 class="text-gray-500 text-3xl uppercase tracking-widest">{move || t.get("roster")}" ("{count}")"</h3>
                                <Show when=move || paused.get()>
                                    <button on:click=move |_| paused.set(false) class="flex items-center gap-2 text-1xl font-bold text-green-400 bg-green-400/10 px-3 py-1 rounded-full animate-pulse hover:bg-green-400/20">
                                        <WelcomeIcon name="rotate_cw" size=12/>" "{move || t.get("resume_autoplay")}
                                    </button>
                                </Show>
                            </div>

                            <div class="grid grid-cols-4 gap-3">
                                {teams.get_value().into_iter().enumerate().map(|(i, team)| {
                                    let label = team.team.name.clone();
                                    let is_active = move || active.get() == i;
                                    let is_past = move || i < active.get();
                                    view! {
                                        <button type="button" aria-label=label aria-pressed=is_active on:click=move |_| manual_select(i)
                                            class=move || format!(
                                                "aspect-square rounded-xl border relative overflow-hidden transition-all duration-300 group cursor-pointer {} {}",
                                                if is_active() { "border-accent-500 scale-110 shadow-[0_0_15px_rgb(var(--glow-accent)/0.6)] z-20 bg-white/20" } else { "border-white/10 bg-white/5" },
                                                if is_past() { "opacity-30 grayscale border-transparent hover:opacity-100 hover:grayscale-0" } else { "opacity-70 hover:opacity-100 hover:border-white/30" })>
                                            <TeamAvatar team=team class="w-full h-full" text_class="text-3xl"/>
                                            <div class=move || format!("absolute top-1 left-1 text-[15px] font-bold px-1.5 rounded {}",
                                                if is_active() { "bg-accent-500 text-black" } else { "bg-black/50 text-white/50" })>
                                                {i + 1}
                                            </div>
                                            <Show when=is_active>
                                                <div class="absolute inset-0 border-2 border-accent-500 rounded-xl welcome-ring"/>
                                            </Show>
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    </Show>
                </div>

                // 3. FOOTER
                <div class="h-[15cqh] w-full flex items-center justify-center z-[100] relative">
                    <Show when=move || !exiting.get()>
                        <button data-tour="start" aria-keyshortcuts="Enter" on:click=handle_start
                            class="welcome-start group relative flex flex-col items-center justify-center p-10 outline-none">
                            // The main text: starts ghosted, glows on hover
                            <span class="text-4xl font-black uppercase tracking-[0.2em] text-white/20 transition-all duration-500 group-hover:text-accent-400 group-hover:tracking-[0.4em] group-hover:scale-110 group-hover:drop-shadow-[0_0_15px_rgb(var(--glow-accent)/0.8)]">
                                {move || t.get("start_show")}
                            </span>
                            // The "power line": expands from the centre
                            <div class="h-[2px] bg-gradient-to-r from-transparent via-accent-500 to-transparent w-0 group-hover:w-full transition-all duration-700 ease-out mt-4 opacity-0 group-hover:opacity-100" />
                            // A subtle play icon that appears when hovered
                            <div class="absolute bottom-2 text-accent-500 welcome-play">
                                <WelcomeIcon name="play" size=16 fill="currentColor"/>
                            </div>
                        </button>
                    </Show>
                </div>
            </div>
        </div>
    }
}

#[derive(Debug, PartialEq, Eq)]
enum WelcomeKey {
    Start,
    TogglePause,
    Select(isize),
}

fn keyboard_action(key: &str, layout: Layout, count: usize) -> Option<WelcomeKey> {
    match key {
        "Enter" => Some(WelcomeKey::Start),
        " " if layout == Layout::Cards && count > 1 => Some(WelcomeKey::TogglePause),
        "ArrowRight" if layout == Layout::Cards && count > 0 => Some(WelcomeKey::Select(1)),
        "ArrowLeft" if layout == Layout::Cards && count > 0 => Some(WelcomeKey::Select(-1)),
        _ => None,
    }
}
fn relative_team(active: usize, count: usize, delta: isize) -> usize {
    if count == 0 {
        0
    } else {
        (active as isize + delta).rem_euclid(count as isize) as usize
    }
}
fn wall_intro_duration(cycle_ms: u64) -> u64 {
    ((cycle_ms as f64 * 0.6) as u64).max(3000)
}

/// The big card of the current team.
#[component]
fn TeamCard(team: ResolvedTeam) -> impl IntoView {
    let t = use_t();
    let color = team_color(team.color);
    let name = team.team.name.clone();
    let quote = team.team.quote.clone().unwrap_or_default();
    let has_quote = !quote.is_empty();
    let players = team.team.players.unwrap_or(0);
    let name_class = name_size(&name);
    let quote_class = if has_quote {
        quote_size(&name, &quote)
    } else {
        "hidden"
    };
    view! {
        <div class="absolute inset-0 z-30 welcome-main-card">
            <div class=format!("w-full h-full rounded-[3rem] bg-gradient-to-br {color} p-3 shadow-[0_0_80px_rgba(0,0,0,0.6)]")>
                <div class=format!("h-full w-full bg-black/85 backdrop-blur-md rounded-[2.5rem] p-10 flex flex-col items-center text-white relative overflow-hidden border border-white/5 {}",
                    if has_quote { "" } else { "justify-center" })>
                    <div class="absolute -bottom-24 -right-16 opacity-[0.07] rotate-12 text-white font-black text-[320px] leading-none select-none">
                        {team.initials.clone()}
                    </div>
                    <TeamAvatar team=team.clone() class="w-75 h-75 rounded-3xl mb-8 shadow-inner border border-white/10 relative z-10" text_class="text-[140px]"/>
                    <h2 class=format!("font-black text-center leading-none mb-6 z-10 drop-shadow-lg tracking-tight {name_class}")>
                        {name}
                    </h2>
                    <p class=format!("text-gray-400 italic text-center z-10 px-4 leading-relaxed {quote_class}")>
                        "\""{quote}"\""
                    </p>
                    {(players > 0).then(|| view! {
                        <div class=format!("{} flex gap-4 z-10", if has_quote { "mt-auto" } else { "mt-4" })>
                            <span class="px-6 py-3 bg-white/10 rounded-full flex items-center gap-3 text-4xl font-bold uppercase tracking-wider border border-white/10">
                                <WelcomeIcon name="users" size=48/>" "{players}" "{move || t.get("players")}
                            </span>
                        </div>
                    })}
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quiz_core::quiz::model::{Team, resolve_teams};

    fn teams(list: &[(bool, Option<&str>, Option<&str>)]) -> Vec<ResolvedTeam> {
        let raw: Vec<Team> = list
            .iter()
            .enumerate()
            .map(|(i, (is_new, quote, image))| Team {
                name: format!("Team {i}"),
                is_new: Some(*is_new),
                quote: quote.map(Into::into),
                image: image.map(Into::into),
                ..Team::default()
            })
            .collect();
        resolve_teams(&raw, |s| (!s.is_empty()).then(|| s.to_string()))
    }

    #[test]
    fn keyboard_navigation_is_bounded_and_layout_specific() {
        assert_eq!(
            keyboard_action("Enter", Layout::Wall, 0),
            Some(WelcomeKey::Start)
        );
        assert_eq!(
            keyboard_action("ArrowRight", Layout::Cards, 3),
            Some(WelcomeKey::Select(1))
        );
        assert_eq!(
            keyboard_action("ArrowLeft", Layout::Cards, 3),
            Some(WelcomeKey::Select(-1))
        );
        assert_eq!(keyboard_action("ArrowRight", Layout::Cards, 0), None);
        assert_eq!(keyboard_action("ArrowLeft", Layout::Wall, 3), None);
        assert_eq!(
            keyboard_action(" ", Layout::Cards, 3),
            Some(WelcomeKey::TogglePause)
        );
        assert_eq!(keyboard_action(" ", Layout::Cards, 1), None);
        assert_eq!(keyboard_action(" ", Layout::Wall, 3), None);
        assert_eq!(relative_team(0, 3, -1), 2);
        assert_eq!(relative_team(2, 3, 1), 0);
        assert_eq!(relative_team(0, 1, -1), 0);
        assert_eq!(relative_team(0, 0, 1), 0);
    }

    #[test]
    fn wall_duration_has_reacts_minimum() {
        assert_eq!(wall_intro_duration(8000), 4800);
        assert_eq!(wall_intro_duration(1000), 3000);
        assert_eq!(wall_intro_duration(10_000), 6000);
    }

    #[test]
    fn layout_choice() {
        assert_eq!(choose_layout(&[], "auto"), Layout::Cards);
        assert_eq!(choose_layout(&[], "wall"), Layout::Wall);
        let bare = teams(&[
            (true, None, None),
            (true, Some(""), None),
            (false, None, None),
        ]);
        assert_eq!(choose_layout(&bare, "auto"), Layout::Wall);
        assert_eq!(choose_layout(&bare, "cards"), Layout::Cards);
        assert_eq!(choose_layout(&bare, "nonsense"), Layout::Wall);
        let half = teams(&[(true, None, Some("a.jpg")), (true, None, None)]);
        assert_eq!(choose_layout(&half, "auto"), Layout::Cards);
        let rich = teams(&[
            (true, Some("Hi"), None),
            (false, None, None),
            (true, None, None),
        ]);
        assert_eq!(choose_layout(&rich, "auto"), Layout::Cards);
    }

    #[test]
    fn stack_behind_the_card() {
        let c = stack_cards(3, 5);
        assert_eq!(
            c.iter().map(|c| (c.offset, c.team)).collect::<Vec<_>>(),
            vec![(2, 0), (1, 4)]
        );
        let single = stack_cards(0, 1);
        assert_eq!(single[0].team, 0);
        assert_eq!((single[0].dup, single[1].dup), (0, 1));
        assert!(stack_cards(0, 0).is_empty());
        assert_eq!(
            stack_style(1),
            "translate: 20px -30px; scale: 0.95; z-index: 9; opacity: 0.3"
        );
        assert_eq!(
            stack_style(2),
            "translate: 40px -60px; scale: 0.9; z-index: 8; opacity: 0.2"
        );
    }

    #[test]
    fn text_sizes() {
        assert_eq!(name_size("The Quizzards"), "text-7xl");
        assert_eq!(name_size("Sixteen letters!"), "text-5xl");
        // 15 two-byte letters are 15 JavaScript characters
        assert_eq!(name_size("ěščřžýáíéůúťďňó"), "text-7xl");
        assert_eq!(quote_size("Short", "Hi"), "text-5xl");
        assert_eq!(quote_size("A rather long team name", "Hi"), "text-4xl");
        assert_eq!(quote_size("Short", &"x".repeat(51)), "text-4xl");
        assert_eq!(quote_size("Short", &"x".repeat(81)), "text-4xl");
        assert_eq!(quote_size("Short", &"x".repeat(121)), "text-3xl");
        assert_eq!(
            quote_size("A rather long team name", &"x".repeat(121)),
            "text-3xl"
        );
    }

    #[test]
    fn countdown_text() {
        let at = |h: f64, m: f64, s: f64| ((h * 60.0 + m) * 60.0 + s) * 1000.0;
        assert_eq!(countdown("", at(10.0, 0.0, 0.0)), Countdown::Unset);
        assert_eq!(
            countdown("20:00", at(18.0, 29.0, 30.0)),
            Countdown::Hours(1, 30)
        );
        assert_eq!(
            countdown("20:00", at(19.0, 58.0, 0.5)),
            Countdown::Minutes(1, 59)
        );
        assert_eq!(countdown("20:00", at(20.0, 0.0, 0.0)), Countdown::Now);
        assert_eq!(countdown("20:00", at(21.0, 0.0, 0.0)), Countdown::Now);
        assert_eq!(countdown("20", at(1.0, 0.0, 0.0)), Countdown::Invalid);
        assert_eq!(countdown("ab:00", at(1.0, 0.0, 0.0)), Countdown::Invalid);
        // Hours past midnight roll over into tomorrow, as Date.setHours does
        assert_eq!(
            countdown("25:00", at(23.0, 0.0, 0.0)),
            Countdown::Hours(2, 0)
        );
    }

    #[test]
    fn header_and_settings() {
        assert_eq!(header_item(4, true), 1);
        assert_eq!(header_item(2, false), 0);
        assert_eq!(or_default(0, 3000), 3000);
        assert_eq!(or_default(1200, 3000), 1200);
        let vars = spring_vars();
        for name in [
            "--wl-card:",
            "--wl-panel:",
            "--wl-snap-d:",
            "--wl-scale:",
            "--wl-pop-d:",
        ] {
            assert!(vars.contains(name), "{name}");
        }
    }
}
