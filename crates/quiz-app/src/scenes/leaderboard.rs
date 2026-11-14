//! Audience standings and the two-step score-group reveal from Leaderboard.jsx.
use super::backdrop::SceneBackdrop;
use super::{
    celebration::Celebration,
    icons::IntroIcon,
    team_avatar::{TeamAvatar, team_color},
};
use crate::{
    i18n::use_t,
    presence::Presence,
    quiz_context::use_quiz,
    reorder,
    scores::{empty_scores, use_scores},
    show::use_show,
    stage::StageLayout,
    theme::use_theme,
};
use leptos::{ev, prelude::*};
use quiz_core::{
    quiz::{
        model::{ResolvedTeam, Team, resolve_teams},
        parse::ParsedScores,
        teams::team_for_scores,
    },
    show::{Action, Role},
    theme::format::LeaderboardLayout,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, closure::Closure};

fn default_round(board: &ParsedScores) -> String {
    board
        .rounds
        .iter()
        .rev()
        .find(|r| {
            board
                .teams
                .iter()
                .any(|t| t.scores.get(*r).copied().unwrap_or(0.) > 0.)
        })
        .or_else(|| board.rounds.first())
        .cloned()
        .unwrap_or_default()
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Reveal {
    snapshot: ParsedScores,
    selected_round: String,
    reveal_step: usize,
}
impl Reveal {
    fn adopt(&mut self, board: ParsedScores) {
        if !board.rounds.contains(&self.selected_round) {
            self.selected_round = default_round(&board);
        }
        self.snapshot = board;
    }
}
#[derive(Clone, Debug, PartialEq)]
struct Standing {
    key: usize,
    team: ResolvedTeam,
    total: f64,
    previous: f64,
    points: f64,
    group: usize,
    previous_rank: usize,
    rank: usize,
    change: i32,
    score: f64,
    revealed: bool,
    highest: bool,
    fire: bool,
    delta_leader: f64,
    delta_next: f64,
}
#[derive(Clone, Debug, PartialEq)]
struct Board {
    teams: Vec<Standing>,
    groups: Vec<f64>,
}
fn number(n: f64) -> String {
    if n.fract() == 0. {
        format!("{n:.0}")
    } else {
        format!("{n:.1}")
    }
}
fn competition_ranks(scores: &[f64]) -> Vec<usize> {
    let mut rank = 1;
    scores
        .iter()
        .enumerate()
        .map(|(i, score)| {
            if i > 0 && *score < scores[i - 1] {
                rank = i + 1;
            }
            rank
        })
        .collect()
}
fn build_board(snapshot: &ParsedScores, round: &str, metadata: &[ResolvedTeam]) -> Board {
    let Some(index) = snapshot.rounds.iter().position(|r| r == round) else {
        return Board {
            teams: vec![],
            groups: vec![],
        };
    };
    let raw: Vec<Team> = metadata.iter().map(|t| t.team.clone()).collect();
    let mut teams: Vec<_> = snapshot
        .teams
        .iter()
        .enumerate()
        .map(|(key, row)| {
            let matched = team_for_scores(&raw, row)
                .and_then(|t| raw.iter().position(|r| std::ptr::eq(t, r)));
            let mut team = matched.map(|i| metadata[i].clone()).unwrap_or_else(|| {
                resolve_teams(
                    &[Team {
                        name: row.name.clone(),
                        ..Team::default()
                    }],
                    |_| None,
                )
                .remove(0)
            });
            team.team.name = row.name.clone();
            let points = |r: &String| {
                row.scores
                    .get(r)
                    .copied()
                    .filter(|n| n.is_finite())
                    .unwrap_or(0.)
            };
            let previous = snapshot.rounds[..index].iter().map(points).sum::<f64>();
            let current = points(&snapshot.rounds[index]);
            let previous_points = if index > 0 {
                points(&snapshot.rounds[index - 1])
            } else {
                0.
            };
            (
                Standing {
                    key,
                    team,
                    total: previous + current,
                    previous,
                    points: current,
                    group: 0,
                    previous_rank: 1,
                    rank: 1,
                    change: 0,
                    score: previous,
                    revealed: false,
                    highest: false,
                    fire: false,
                    delta_leader: 0.,
                    delta_next: 0.,
                },
                previous_points,
            )
        })
        .collect();
    let max = teams.iter().map(|(t, _)| t.points).fold(0., f64::max);
    let prev_max = teams.iter().map(|(_, p)| *p).fold(0., f64::max);
    teams.sort_by(|(a, _), (b, _)| b.previous.total_cmp(&a.previous));
    let ranks = competition_ranks(&teams.iter().map(|(t, _)| t.previous).collect::<Vec<_>>());
    let mut groups: Vec<_> = teams.iter().map(|(t, _)| t.points).collect();
    groups.sort_by(f64::total_cmp);
    groups.dedup();
    let teams = teams
        .into_iter()
        .zip(ranks)
        .map(|((mut t, p), rank)| {
            t.previous_rank = rank;
            t.rank = rank;
            t.highest = max > 0. && t.points == max;
            t.fire = t.highest && prev_max > 0. && p == prev_max;
            t.group = groups.iter().position(|p| *p == t.points).unwrap_or(0);
            t
        })
        .collect();
    Board { teams, groups }
}
fn max_step(board: &Board) -> usize {
    board.groups.len() * 2 + 1
}
// An empty board has a nominal final step, but must remain ready to adopt late data.
fn next_step(board: &Board, current: usize) -> usize {
    if board.teams.is_empty() {
        current
    } else {
        (current + 1).min(max_step(board))
    }
}
fn active_group(step: usize, max: usize) -> Option<usize> {
    (step % 2 == 1 && step < max).then_some(step / 2)
}
/// Rows only change places when entering or leaving the final step (the sort). A view
/// transition on any other step would lift the named rows above the points overlay.
fn reorders(board: &Board, from: usize, to: usize) -> bool {
    let last = max_step(board);
    from != to && (from == last || to == last)
}
fn active_teams(board: &Board, step: usize) -> Vec<Standing> {
    let final_step = step == max_step(board);
    let mut teams = board.teams.clone();
    for t in &mut teams {
        t.revealed = step >= t.group * 2 + 2;
        t.score = if t.revealed { t.total } else { t.previous };
    }
    if final_step {
        teams.sort_by(|a, b| b.score.total_cmp(&a.score));
    }
    let ranks = competition_ranks(
        &teams
            .iter()
            .map(|t| if final_step { t.score } else { t.previous })
            .collect::<Vec<_>>(),
    );
    let leader = teams.first().map_or(0., |t| t.score);
    for i in 0..teams.len() {
        let next = if i == 0 {
            0.
        } else {
            teams[i - 1].score - teams[i].score
        };
        let t = &mut teams[i];
        t.rank = ranks[i];
        t.change = t.previous_rank as i32 - t.rank as i32;
        t.delta_leader = leader - t.score;
        t.delta_next = if t.rank == 1 { 0. } else { next };
    }
    teams
}
fn shown_layout(layout: LeaderboardLayout, final_step: bool) -> LeaderboardLayout {
    if final_step || layout == LeaderboardLayout::Race {
        layout
    } else {
        LeaderboardLayout::Table
    }
}
fn race_geometry(teams: &[Standing]) -> (usize, f64, f64, f64) {
    let top = teams.iter().map(|t| t.total).fold(1., f64::max);
    let low = teams.iter().map(|t| t.previous).fold(top, f64::min);
    let base = (low - (top - low) * 0.25).max(0.);
    let columns = if teams.len() > 12 { 2 } else { 1 };
    let rows = teams.len().div_ceil(columns).max(1);
    (
        columns,
        (68. / rows as f64 - 0.8).min(7.5),
        base,
        (top - base).max(1.),
    )
}
#[component]
pub fn Leaderboard() -> impl IntoView {
    let show = use_show();
    let t = use_t();
    let quiz = use_quiz();
    let metadata = use_scores().teams;
    let theme = use_theme();
    let presenter = show.role == Role::Presenter;
    let round_chosen = RwSignal::new(false);
    let reveal = RwSignal::new(Reveal {
        snapshot: empty_scores(),
        selected_round: String::new(),
        reveal_step: 0,
    });
    let live = use_scores().scores;
    let board =
        Memo::new(move |_| reveal.with(|r| build_board(&r.snapshot, &r.selected_round, &metadata.get())));
    let step = Signal::derive(move || reveal.with(|r| r.reveal_step.min(max_step(&board.get()))));
    let final_step = Signal::derive(move || step.get() == max_step(&board.get()));
    // Leaderboard.jsx: a cue when stepping forwards, the winner's at the final standings.
    let heard = StoredValue::new(0usize);
    Effect::new(move |_| {
        let (now, last) = (step.get(), max_step(&board.get()));
        let before = heard.get_value();
        heard.set_value(now);
        if now <= before || board.with(|b| max_step(b) == 0) {
            return;
        }
        if now == last {
            crate::sounds::play(quiz_core::sounds::Sound::Winner);
        } else if now % 2 == 1 {
            crate::sounds::play(quiz_core::sounds::Sound::Reveal);
        }
    });
    let rows = Memo::new(move |_| active_teams(&board.get(), step.get()));
    let layout = Signal::derive(move || {
        shown_layout(
            theme
                .get()
                .scenes
                .leaderboard
                .as_ref()
                .map_or(LeaderboardLayout::Table, |l| l.layout),
            final_step.get(),
        )
    });
    let celebrating = RwSignal::new(false);
    let celebration_enabled = quiz
        .quiz
        .settings
        .fields
        .get("celebration")
        .and_then(|v| v.as_bool())
        != Some(false);
    let seen = StoredValue::new(0_usize);
    Effect::new(move |_| {
        let current = step.get();
        let previous = seen.get_value();
        seen.set_value(current);
        if current < max_step(&board.get()) {
            celebrating.set(false);
        } else if current > previous
            && celebration_enabled
            && !presenter
            && !board.get().teams.is_empty()
        {
            celebrating.set(true);
        }
    });
    // Keep a frozen snapshot once revealing. The wire shape matches React's LB_* protocol.
    let presenter_open = RwSignal::new(false);
    Effect::new(move |_| {
        let next = live.get();
        if (presenter || !presenter_open.get())
            && reveal.with_untracked(|r| r.reveal_step == 0 && r.snapshot != next)
        {
            reveal.update(|r| {
                if !round_chosen.get_untracked() {
                    r.selected_round.clear();
                }
                r.adopt(next);
            });
        }
    });
    let channel =
        StoredValue::new_local(web_sys::BroadcastChannel::new("quiz-leaderboard-sync").ok());
    let role = if presenter { "presenter" } else { "main" };
    let last_synced = StoredValue::new(reveal.get_untracked());
    let post = move |kind: &str, payload: Option<Reveal>| {
        channel.with_value(|c| {
            if let Some(c) = c {
                let json = serde_json::json!({"type":kind,"sender":role,"payload":payload});
                if let Ok(value) = js_sys::JSON::parse(&json.to_string()) {
                    let _ = c.post_message(&value);
                }
            }
        });
    };
    let stage = use_context::<Signal<StageLayout>>();
    let message =
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
            let Some(json) = js_sys::JSON::stringify(&event.data())
                .ok()
                .and_then(|s| s.as_string())
            else {
                return;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) else {
                return;
            };
            let Some(sender) = value["sender"].as_str() else {
                return;
            };
            if sender == role {
                return;
            }
            let kind = value["type"].as_str().unwrap_or("");
            if sender == "presenter" {
                presenter_open.set(kind != "LB_LEAVE");
            }
            match kind {
                "LB_REQUEST" => post("LB_STATE", Some(reveal.get_untracked())),
                "LB_UPDATE" | "LB_STATE" => {
                    if let Ok(next) = serde_json::from_value::<Reveal>(value["payload"].clone()) {
                        if kind == "LB_UPDATE" || !presenter || next.reveal_step > 0 {
                            last_synced.set_value(next.clone());
                            let scale = stage.map_or(1., |l| l.get_untracked().scale);
                            let (from, to) = (reveal.get_untracked().reveal_step, next.reveal_step);
                            if reorders(&board.get_untracked(), from, to) {
                                reorder::reorder(reorder::Mode::ViewTransition, scale, move || {
                                    reveal.set(next)
                                });
                            } else {
                                reveal.set(next);
                            }
                        } else {
                            post("LB_UPDATE", Some(reveal.get_untracked()));
                        }
                    }
                }
                _ => {}
            }
        });
    channel.with_value(|c| {
        if let Some(c) = c {
            c.set_onmessage(Some(message.as_ref().unchecked_ref()));
        }
    });
    post("LB_REQUEST", None);
    let leave = window_event_listener(ev::pagehide, move |_| {
        if presenter {
            post("LB_LEAVE", None);
        }
    });
    on_cleanup(move || leave.remove());
    Effect::new(move |_| {
        let next = reveal.get();
        if next != last_synced.get_value() {
            last_synced.set_value(next.clone());
            post("LB_UPDATE", Some(next));
        }
    });
    let message = StoredValue::new_local(Some(message));
    on_cleanup(move || {
        if presenter {
            post("LB_LEAVE", None);
        }
        channel.with_value(|c| {
            if let Some(c) = c {
                c.set_onmessage(None);
                c.close();
            }
        });
        message.update_value(|message| {
            message.take();
        });
    });
    let ui = crate::show::use_show_ui();
    let keys = window_event_listener(ev::keydown, move |event| {
        // Do not steal input while the show settings are open or a form is focused.
        if ui.settings_open.get_untracked() {
            return;
        }
        let tag = event
            .target()
            .and_then(|e| e.dyn_into::<web_sys::Element>().ok())
            .map(|e| e.tag_name())
            .unwrap_or_default();
        if matches!(tag.as_str(), "INPUT" | "SELECT" | "TEXTAREA") {
            return;
        }
        let current = step.get_untracked();
        let next = match event.key().as_str() {
            "ArrowRight" => next_step(&board.get_untracked(), current),
            "ArrowLeft" => current.saturating_sub(1),
            "Escape" => {
                show.dispatch.run(Action::BackToDashboard);
                return;
            }
            _ => return,
        };
        event.prevent_default();
        if next != current {
            let scale = stage.map_or(1., |l| l.get_untracked().scale);
            let change = move || reveal.update(|r| r.reveal_step = next);
            if reorders(&board.get_untracked(), current, next) {
                reorder::reorder(reorder::Mode::ViewTransition, scale, change);
            } else {
                change();
            }
        }
    });
    on_cleanup(move || keys.remove());
    if presenter {
        return view! {<PresenterLeaderboard reveal=reveal board=board step=step round_chosen=round_chosen/>}.into_any();
    }
    let overlay =
        Signal::derive(move || active_group(step.get(), max_step(&board.get())).is_some());
    let last_group = RwSignal::new(0_usize);
    Effect::new(move |_| {
        if let Some(i) = active_group(step.get(), max_step(&board.get())) {
            last_group.set(i);
        }
    });
    view! {
        <div class="h-full w-full bg-ground relative overflow-hidden flex flex-col font-show selection:bg-primary-500/30 standings" style=format!("--standings-spring:{}", crate::spring::Spring::from_bounce(0.4, 0.8).to_css_linear())>
            <SceneBackdrop quiet=true/>
            <Show when=move || celebrating.get()><Celebration on_done=Callback::new(move |_| celebrating.set(false))/></Show>
            <div class="absolute top-0 right-0 w-[1200px] h-[1200px] bg-primary-900/10 blur-[250px] rounded-full pointer-events-none"/>
            <div class="absolute bottom-0 left-0 w-[800px] h-[800px] bg-accent-900/5 blur-[200px] rounded-full pointer-events-none"/>
            <div class="relative z-10 flex flex-col h-full w-full max-w-[1920px] mx-auto pt-[6cqh] pb-[6cqh] px-12">
                <div class="flex items-center justify-between border-b border-white/10 pb-6 mb-8 shrink-0">
                    <div class="flex items-center gap-6"><IntroIcon name="trophy" size=48 class="text-yellow-500"/>
                        <div><h1 class="font-title text-5xl font-black uppercase tracking-[0.2em] text-white leading-none">{move || t.get("lb_title")}</h1>
                        <h2 class="text-primary-400 font-bold tracking-[0.4em] text-sm uppercase mt-2">{move || if final_step.get() {t.get("lb_final")} else {t.fmt("lb_round_reveal", &[("round", &reveal.get().selected_round)])}}</h2></div>
                    </div>
                    <div class="flex flex-col items-end"><span class="text-white/40 font-bold tracking-widest uppercase text-xs mb-2">{move || t.get("lb_progress")}</span>
                        <div class="flex gap-1.5">{move || board.get().groups.iter().enumerate().map(|(i,_)| view! {
                            <div class=if step.get().ge(&(i*2+2)) {"w-6 h-3 rounded-full transition-colors duration-500 bg-primary-500 shadow-[0_0_10px_rgb(var(--glow-primary)/0.5)]"} else {"w-6 h-3 rounded-full transition-colors duration-500 bg-white/10"}/>
                        }).collect_view()}<div class=move || if final_step.get() {"w-8 h-3 rounded-full ml-2 transition-colors duration-500 bg-green-500 shadow-[0_0_15px_rgba(74,222,128,0.8)]"} else {"w-8 h-3 rounded-full ml-2 transition-colors duration-500 bg-white/10"}/></div>
                    </div>
                </div>
                <div class="flex-1 w-full overflow-hidden relative">
                    <Show when=move || board.get().teams.is_empty()>
                        <div class="absolute inset-0 flex flex-col items-center justify-center gap-6 text-center"><IntroIcon name="trophy" size=96 class="text-white/10"/><h2 class="text-5xl font-black uppercase tracking-[0.2em] text-white/60">{move || t.get("lb_empty_title")}</h2></div>
                    </Show>
                    <Show when=move || layout.get() == LeaderboardLayout::Table>
                        <div class="grid grid-cols-2 grid-flow-row gap-x-16 gap-y-5 w-full content-start">
                            <For each=move || rows.get() key=|r| r.key children=move |r| { let key=r.key; view! {<TableRow rows=rows key=key final_step=final_step/>} }/>
                        </div>
                    </Show>
                    <Show when=move || layout.get() == LeaderboardLayout::Race><RaceView rows=rows final_step=final_step/></Show>
                    <Show when=move || matches!(layout.get(), LeaderboardLayout::Podium | LeaderboardLayout::Spotlight | LeaderboardLayout::Medals)>
                        {move || view! {<FinalView teams=rows.get() layout=layout.get()/>}}
                    </Show>
                </div>
                <Presence when=Signal::derive(move || !presenter && !presenter_open.get() && step.get() == 0 && !board.get().teams.is_empty()) enter="standings-fade-in" exit="standings-fade-out" class="standings-hint absolute bottom-[3.5cqh] inset-x-0 pointer-events-none">
                    <p class="text-center text-white/40 font-sans text-xl tracking-wide">{move || t.get("lb_keys_hint")}</p>
                </Presence>
                <Presence when=overlay enter="standings-overlay-in" exit="standings-overlay-out" class="absolute inset-0 z-50">
                    // The dim layer fades on its own so the cards are never see-through over the list.
                    <div class="standings-overlay-bg absolute inset-0 bg-black/80 backdrop-blur-md"/>
                    <div class="relative h-full flex flex-col items-center justify-center p-12">
                        <h2 class="standings-group-title font-title text-7xl font-black uppercase tracking-[0.2em] text-primary-400 drop-shadow-[0_0_40px_rgb(var(--glow-primary)/0.5)] mb-16">{move || t.fmt("lb_teams_with", &[("points", &board.get().groups.get(last_group.get()).copied().map(number).unwrap_or_default())])}</h2>
                        <div class="flex flex-wrap justify-center gap-8 max-w-[1600px]">
                            {move || board.get().teams.into_iter().filter(|r| r.group == last_group.get()).enumerate().map(|(i,r)| view! {
                                <div style=format!("--i:{i}") class="standings-group-card bg-black border border-white/20 rounded-[3rem] p-10 flex flex-col items-center justify-center shadow-[0_0_50px_rgba(0,0,0,0.8)] relative w-[350px]">
                                    <div class=format!("absolute inset-0 bg-gradient-to-b {} opacity-20 rounded-[3rem] pointer-events-none", team_color(r.team.color))/>
                                    {r.fire.then(|| view! {<div class="absolute -inset-8 bg-orange-600/30 blur-3xl z-0 animate-pulse rounded-full pointer-events-none"/>})}
                                    <div class="absolute -top-8 flex gap-10 z-20">{r.highest.then(|| view! {<StandingsIcon name="crown" size=48 class="text-yellow-500"/>})}{r.fire.then(|| view! {<StandingsIcon name="flame" size=56 class="text-orange-500 animate-bounce"/>})}</div>
                                    <TeamAvatar team=r.team.clone() class="w-32 h-32 rounded-full border-4 border-white/20 mb-6 relative z-10" text_class="text-5xl"/>
                                    <h3 class="text-2xl font-black text-white uppercase tracking-wider text-center relative z-10 leading-tight mb-2">{r.team.team.name}</h3>
                                    {r.team.team.quote.filter(|q| !q.is_empty()).map(|q| view! {<p class="text-sm text-white/50 italic text-center relative z-10">"\""{q}"\""</p>})}
                                </div>
                            }).collect_view()}
                        </div>
                    </div>
                </Presence>
            </div>
        </div>
    }.into_any()
}

fn focus_rows(board: &Board, step: usize) -> Vec<Standing> {
    if step >= max_step(board).saturating_sub(1) {
        let top = board.teams.iter().map(|r| r.total).fold(0., f64::max);
        board
            .teams
            .iter()
            .filter(|r| r.total == top)
            .cloned()
            .collect()
    } else {
        board
            .teams
            .iter()
            .filter(|r| r.group == step / 2)
            .cloned()
            .collect()
    }
}
fn next_action(board: &Board, step: usize) -> (&'static str, String) {
    if step == max_step(board) {
        ("lb_done", String::new())
    } else if step + 1 == max_step(board) {
        ("lb_shuffle", String::new())
    } else {
        (
            if step % 2 == 1 {
                "lb_apply"
            } else {
                "lb_reveal_group"
            },
            board
                .groups
                .get(step / 2)
                .copied()
                .map(number)
                .unwrap_or_default(),
        )
    }
}
fn seat_direction_text(team: &Team, t: crate::i18n::T) -> String {
    team.seat
        .as_ref()
        .map(|s| {
            let direction = quiz_core::quiz::teams::seat_direction(s);
            t.fmt(
                direction.message_key(),
                &[("deg", &direction.deg.to_string())],
            )
        })
        .unwrap_or_else(|| "—".into())
}

#[component]
fn PresenterLeaderboard(
    reveal: RwSignal<Reveal>,
    board: Memo<Board>,
    step: Signal<usize>,
    round_chosen: RwSignal<bool>,
) -> impl IntoView {
    let t = use_t();
    let show = use_show();
    let quiz = use_quiz();
    let scores = use_scores();
    let live = scores.scores;
    let reloading = scores.loading;
    let reload_failed = Signal::derive(move || scores.error.get().is_some());
    let last_sync = Signal::derive(move || scores.last_loaded.get().map(|time| {
        let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(time));
        format!("{:02}:{:02}:{:02}", date.get_hours(), date.get_minutes(), date.get_seconds())
    }).unwrap_or_else(|| "—".into()));
    let has_sheet = crate::quiz_context::query_param("scores").is_some()
        || !quiz.quiz.scores_sheet_url.is_empty() || quiz.quiz.builtin;
    let reload_requested = RwSignal::new(false);
    let reload = move |_| {
        reload_requested.set(true);
        scores.reload();
    };
    Effect::new(move |_| {
        if !reloading.get() && reload_requested.get() {
            reload_requested.set(false);
            if scores.error.get_untracked().is_none() {
                reveal.update(|r| {
                    if !round_chosen.get_untracked() { r.selected_round.clear(); }
                    r.adopt(live.get_untracked());
                    r.reveal_step = 0;
                });
            }
        }
    });
    let has_scores = move || reveal.with(|r| !r.snapshot.teams.is_empty());
    let scored_count = move || {
        reveal.with(|r| {
            r.snapshot
                .teams
                .iter()
                .filter(|row| row.scores.get(&r.selected_round).copied().unwrap_or(0.) > 0.)
                .count()
        })
    };
    let changed = move || {
        step.get() > 0 && !live.get().teams.is_empty() && live.get() != reveal.get().snapshot
    };
    let focus = Memo::new(move |_| focus_rows(&board.get(), step.get()));
    let map_teams = Signal::derive(move || scores.teams.with(|teams| teams.iter()
        .filter(|r| r.team.seat.is_some() && quiz_core::quiz::teams::is_present(&r.team))
        .map(|r| r.team.clone()).collect::<Vec<_>>()));
    let has_seated = move || map_teams.with(|teams| !teams.is_empty());
    let highlight = Signal::derive(move || {
        let focus = focus.get();
        map_teams.with(|teams| {
            teams
                .iter()
                .enumerate()
                .filter(|(_, team)| {
                    focus.iter().any(|r| match (&team.id, &r.team.team.id) {
                        (Some(a), Some(b)) => a == b,
                        _ => team.name == r.team.team.name,
                    })
                })
                .map(|(i, _)| i)
                .collect::<Vec<_>>()
        })
    });
    // TODO(auth): private Google Sheets sign-in and authentication notices.
    view! {
        <div class="h-full w-full bg-panel text-white p-10 font-sans flex flex-col relative">
            <div class="flex justify-between items-start border-b border-white/10 pb-6 mb-8">
                <div><h1 class="text-3xl font-black uppercase tracking-widest text-yellow-500 mb-2">{move || t.get("lb_controls")}</h1>
                    <div class="text-white/40 font-mono mb-4">{move || t.get("lb_step")}": "<span class="text-white">{move || format!("{} / {}",step.get(),max_step(&board.get()))}</span>
                        <Show when=move || has_scores() && step.get()<max_step(&board.get())><span class="ml-3 font-sans">{move || t.fmt("lb_steps_left",&[("n",&(max_step(&board.get())-step.get()).to_string())])}</span></Show>
                    </div>
                    <div class="flex items-center gap-4 bg-primary-900/20 border border-primary-500/30 px-4 py-3 rounded-xl w-fit">
                        <PresenterIcon name="database" size=24 class="text-primary-500"/>
                        <div class="flex flex-col"><label for="leaderboard-round" class="text-primary-400 font-bold uppercase text-[10px] tracking-widest">{move || t.get("lb_select_round")}</label>
                            <select id="leaderboard-round" prop:value=move || reveal.get().selected_round on:change=move |e| {round_chosen.set(true); reveal.update(|r| {r.selected_round=event_target_value(&e);r.reveal_step=0;});} class="bg-transparent text-white font-black text-lg outline-none cursor-pointer">
                                {move || reveal.get().snapshot.rounds.into_iter().map(|r| view! {<option value=r.clone() selected={let r=r.clone(); move || reveal.get().selected_round==r} class="bg-gray-900">{r.clone()}</option>}).collect_view()}
                            </select>
                        </div>
                        <div class="h-8 w-px bg-white/10 mx-2"/>
                        <div class="flex flex-col"><span class="text-primary-400 font-bold uppercase text-[10px] tracking-widest">{move || t.get("lb_last_sync")}</span><span class="font-mono text-sm text-white/80">{move || last_sync.get()}</span></div>
                    </div>
                </div>
                <div class="flex gap-3">
                    <button on:click=reload disabled=move || reloading.get() || scores.loading.get() title=move || t.get("lb_reload_note") class="flex items-center gap-2 px-5 py-3 bg-white/10 hover:bg-white/20 disabled:opacity-50 rounded-xl transition-all font-bold"><span class=move || if reloading.get() || scores.loading.get() {"animate-spin"} else {""}><PresenterIcon name="reload" size=18/></span>{move || t.get("lb_reload")}</button>
                    <button on:click=move |_| show.dispatch.run(Action::BackToDashboard) class="px-6 py-3 bg-red-900/50 hover:bg-red-800/80 rounded-xl transition-all font-bold">{move || t.get("lb_exit")}</button>
                </div>
            </div>
            <div class="space-y-2 mb-6 -mt-2" role="status">
                <Show when=move || reload_failed.get() || scores.error.get().is_some()><p class="flex items-start gap-2 text-red-300"><PresenterIcon name="alert" size=18 class="shrink-0 mt-0.5"/>{move || t.get("lb_reload_failed")}</p></Show>
                <Show when=changed><p class="flex items-start gap-2 text-amber-300"><PresenterIcon name="alert" size=18 class="shrink-0 mt-0.5"/>{move || t.get("lb_scores_changed")}</p></Show>
                <Show when=move || has_scores() && scored_count()<reveal.get().snapshot.teams.len()><p class="flex items-start gap-2 text-yellow-300"><PresenterIcon name="alert" size=18 class="shrink-0 mt-0.5"/>{move || t.fmt("lb_scored_count",&[("n",&scored_count().to_string()),("total",&reveal.get().snapshot.teams.len().to_string()),("round",&reveal.get().selected_round)])}</p></Show>
            </div>
            <Show when=has_scores fallback=move || view! {
                <div class="flex-1 flex flex-col items-center justify-center gap-4 text-center bg-white/5 rounded-2xl border border-white/10 p-10"><IntroIcon name="trophy" size=48 class="text-white/20"/><h2 class="text-2xl font-black">{move || t.get("lb_empty_title")}</h2><p class="text-white/60 max-w-xl">{move || t.get(if has_sheet {"lb_empty_sheet"} else {"lb_empty_manual"})}</p></div>
            }>
                <div class="flex gap-6 mb-3">
                    <button on:click=move |_| reveal.update(|r| r.reveal_step=r.reveal_step.saturating_sub(1)) disabled=move || step.get()==0 class="px-8 py-6 bg-white/5 hover:bg-white/10 disabled:opacity-20 rounded-2xl font-bold flex flex-col items-center justify-center transition-all border border-white/10"><PresenterIcon name="back" size=32 class="mb-2"/>{move || t.get("lb_prev_step")}</button>
                    <button on:click=move |_| {let next=next_step(&board.get_untracked(),step.get_untracked()); reveal.update(|r| r.reveal_step=next);} disabled=move || step.get()==max_step(&board.get()) class=move || format!("flex-1 flex flex-col items-center justify-center py-6 rounded-2xl font-black text-2xl tracking-widest uppercase transition-all shadow-xl {}",if step.get()==max_step(&board.get()) {"bg-white/5 text-white/20"} else if step.get()+1==max_step(&board.get()) {"bg-green-600 hover:bg-green-500 text-white shadow-[0_0_40px_rgba(74,222,128,0.4)]"} else {"bg-primary-600 hover:bg-primary-500 text-white shadow-[0_0_30px_rgb(var(--glow-primary)/0.3)]"})>
                        <Show when=move || step.get()+1==max_step(&board.get())><PresenterIcon name="shuffle" size=28 class="mb-2"/></Show>
                        {move || {let (key,points)=next_action(&board.get(),step.get());t.fmt(key,&[("points",&points)])}}
                    </button>
                </div>
                <p class="flex items-start gap-2 text-sm text-white/40 mb-8"><PresenterIcon name="info" size=16 class="shrink-0 mt-0.5"/>{move || t.get(if step.get()==max_step(&board.get()) {"lb_done_note"} else {"lb_order_note"})}</p>
                <div class="flex-1 min-h-0 flex flex-col gap-6">
                    <Show when=has_seated>
                        <div class="h-[24vh] min-h-[170px] shrink-0 bg-white/5 rounded-2xl border border-white/10 p-4 flex gap-4">
                            <div class="flex-1 min-w-0 flex flex-col"><h3 class="text-sm font-bold text-white/30 uppercase tracking-widest">{move || t.get("att_look")}</h3>
                                {move || view! {<super::venue_map::VenueMap arc=scores.venue_arc.get() teams=map_teams highlight=highlight you_label=t.get("att_you") fill=true marker_scale=1.8 class="flex-1 min-h-0"/>}}
                            </div>
                            <ul class="w-72 shrink-0 space-y-2 self-center overflow-y-auto max-h-full">{move || focus.get().into_iter().map(|r| view! {<li class="flex justify-between items-baseline gap-3"><span class="text-lg font-black truncate">{r.team.team.name.clone()}</span><span class="text-lg font-bold text-yellow-300 whitespace-nowrap">{seat_direction_text(&r.team.team,t)}</span></li>}).collect_view()}</ul>
                        </div>
                    </Show>
                    <div class="flex-1 min-h-0 bg-white/5 rounded-2xl border border-white/10 overflow-y-auto p-6 custom-scrollbar">
                        <h3 class="text-sm font-bold text-white/30 uppercase tracking-widest mb-6">{move || t.get("lb_order")}</h3>
                        {move || board.get().groups.into_iter().enumerate().map(|(i,points)| view! {
                            <div class=move || if step.get().ge(&(i*2+2)) {"mb-6 p-4 rounded-xl border bg-primary-900/30 border-primary-500/30"} else {"mb-6 p-4 rounded-xl border bg-black/40 border-white/10"}>
                                <h4 class="text-lg font-black text-primary-400 mb-3 flex items-center gap-2"><Show when=move || step.get().ge(&(i*2+2))><PresenterIcon name="check" size=18 class="text-primary-500"/></Show>{move || t.fmt("lb_group",&[("points",&number(points))])}</h4>
                                <div class="flex flex-wrap gap-3">{board.get().teams.into_iter().filter(|r| r.group==i).map(|r| view! {
                                    <div class="flex items-center gap-1 bg-white/10 px-3 py-1.5 rounded text-sm font-bold border border-white/5">{r.team.team.name.clone()}
                                        {r.team.team.seat.as_ref().map(|_| view! {<span class="ml-1 text-xs font-normal text-yellow-300/80">{seat_direction_text(&r.team.team,t)}</span>})}
                                        {r.highest.then(|| view! {<StandingsIcon name="crown" size=14 class="text-yellow-500 ml-1"/>})}
                                        {r.fire.then(|| view! {<StandingsIcon name="flame" size=14 class="text-orange-500"/>})}
                                    </div>
                                }).collect_view()}</div>
                            </div>
                        }).collect_view()}
                        <div class=move || if step.get()==max_step(&board.get()) {"p-4 rounded-xl border bg-green-900/30 border-green-500/50"} else {"p-4 rounded-xl border bg-black/40 border-white/10"}>
                            <h4 class=move || if step.get()==max_step(&board.get()) {"text-lg font-black flex items-center gap-2 text-green-400"} else {"text-lg font-black flex items-center gap-2 text-white/40"}><PresenterIcon name="shuffle" size=18/>{move || t.get("lb_final_step")}</h4>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn PresenterIcon(
    name: &'static str,
    size: u32,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let paths = match name {
        "database" => view! {<g><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14a9 3 0 0 0 18 0V5M3 12a9 3 0 0 0 18 0"/></g>}.into_any(),
        "back" => view! {<path d="m12 19-7-7 7-7M19 12H5"/>}.into_any(),
        "reload" => view! {<g><path d="M20 7v5h-5M4 17v-5h5"/><path d="M6 7a7 7 0 0 1 12-2l2 7M4 12l2 7a7 7 0 0 0 12-2"/></g>}.into_any(),
        "shuffle" => view! {<path d="m18 14 4 4-4 4M18 2l4 4-4 4M2 18h3c5 0 9-12 14-12h3M2 6h3c2 0 4 2 6 6s5 6 8 6h3"/>}.into_any(),
        "check" => view! {<g><circle cx="12" cy="12" r="10"/><path d="m8 12 3 3 5-6"/></g>}.into_any(),
        "alert" => view! {<g><path d="m12 3 10 18H2Z"/><path d="M12 9v4m0 4h.01"/></g>}.into_any(),
        _ => view! {<g><circle cx="12" cy="12" r="10"/><path d="M12 11v6m0-10h.01"/></g>}.into_any(),
    };
    view! {<svg width=size height=size class=class viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{paths}</svg>}
}

#[component]
fn StandingsIcon(
    name: &'static str,
    size: u32,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let path = match name {
        "crown" => "m2 8 5 4 5-9 5 9 5-4-3 13H5L2 8Z M5 21h14",
        "flame" => "M12 22c6 0 9-4 8-9-1-4-5-8-7-11 0 5-4 6-4 10-2-1-3-3-3-4-4 6-4 14 6 14Z",
        "up" => "m5 12 7-7 7 7 M12 5v14",
        "down" => "m5 12 7 7 7-7 M12 5v14",
        "minus" => "M5 12h14",
        _ => "M9 9a3 3 0 0 1 6 0c0 2-3 2-3 4 M12 17h.01",
    };
    view! {<svg width=size height=size class=class viewBox="0 0 24 24" fill=if name=="flame" {"currentColor"} else {"none"} stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d=path/></svg>}
}
#[component]
fn AnimatedScore(value: Signal<f64>) -> impl IntoView {
    let displayed = RwSignal::new(value.get_untracked());
    let timer = StoredValue::new_local(None::<gloo_timers::callback::Interval>);
    Effect::new(move |_| {
        let to = value.get();
        timer.update_value(|timer| {
            timer.take();
        });
        let from = displayed.get_untracked();
        if reorder::reduced_motion() || from == to {
            displayed.set(to);
            return;
        }
        let start = window().performance().map_or(0., |p| p.now());
        let interval = gloo_timers::callback::Interval::new(16, move || {
            let elapsed = window().performance().map_or(600., |p| p.now() - start);
            let progress = (elapsed / 600.).clamp(0., 1.);
            displayed.set(from + (to - from) * (1. - (1. - progress).powi(3)));
            if progress >= 1. {
                timer.update_value(|timer| {
                    timer.take();
                });
            }
        });
        timer.set_value(Some(interval));
    });
    on_cleanup(move || {
        timer.update_value(|timer| {
            timer.take();
        })
    });
    view! {<span>{move || number((displayed.get()*10.).round()/10.)}</span>}
}
#[component]
fn TableRow(rows: Memo<Vec<Standing>>, key: usize, final_step: Signal<bool>) -> impl IntoView {
    let t = use_t();
    let row = Signal::derive(move || {
        rows.get()
            .into_iter()
            .find(|r| r.key == key)
            .expect("mounted row")
    });
    let initial = row.get_untracked();
    let winner = move || final_step.get() && row.get().rank == 1;
    let fire = move || row.get().fire && row.get().revealed;
    view! {
        <div data-reorder="" style=format!("view-transition-name:standings-team-{key}") class=move || format!("relative flex items-stretch h-[80px] rounded-r-2xl overflow-hidden shadow-lg transition-colors duration-700 {}", if winner() {"bg-black/80 border border-yellow-500/80 shadow-[0_0_30px_rgba(234,179,8,0.2)]"} else if row.get().revealed {"bg-primary-950/20 border border-primary-500/20"} else {"bg-black/40 border border-white/10"})>
            <Show when=winner><div class="absolute inset-0 border-2 border-yellow-500/80 rounded-r-xl pointer-events-none z-20"/></Show>
            <Show when=fire><div class="absolute inset-0 standings-fire border-b-2 border-orange-500 rounded-r-xl pointer-events-none z-10"/></Show>
            <div class=move || format!("w-[60px] flex items-center justify-center font-mono font-black text-2xl border-r border-white/5 shrink-0 transition-colors relative z-20 {}", if winner() {"bg-yellow-500 text-black"} else {"bg-white/10 text-white/80"})>{move || row.get().rank}</div>
            <div class="w-[50px] flex items-center justify-center bg-white/[0.02] border-r border-white/5 shrink-0 relative z-20">
                <Show when=move || final_step.get() fallback=|| view! {<div class="w-1.5 h-1.5 rounded-full bg-white/10 animate-pulse"/>}>
                    {move || {let change = row.get().change; view! {<div class=if change.is_positive() {"standings-pop flex flex-col items-center text-green-500"} else if change.is_negative() {"standings-pop flex flex-col items-center text-red-500"} else {"standings-pop flex flex-col items-center text-white/20"}><StandingsIcon name=if change.is_positive() {"up"} else if change.is_negative() {"down"} else {"minus"} size=16/>{(change!=0).then(|| view! {<span class="text-[10px] font-black leading-none">{change.abs()}</span>})}</div>}}}
                </Show>
            </div>
            <div class=format!("w-[6px] shrink-0 bg-gradient-to-b {} relative z-20", team_color(initial.team.color)) style:opacity=move || if row.get().revealed {"1"} else {"0.3"}/>
            <div class="flex-1 flex items-center px-4 overflow-hidden gap-4 relative z-20">
                <TeamAvatar team=initial.team.clone() class="w-10 h-10 rounded-full border border-white/20" text_class="text-sm"/>
                <span class=move || format!("font-black text-xl tracking-widest uppercase truncate {} {}", if winner() {"text-yellow-400"} else if fire() {"text-orange-400 drop-shadow-[0_0_8px_rgba(249,115,22,0.8)]"} else {"text-white"}, if row.get().revealed {""} else {"opacity-50"})>{initial.team.team.name}</span>
                <Show when=fire><StandingsIcon name="flame" size=20 class="text-orange-500 animate-pulse shrink-0"/></Show>
            </div>
            <div class=move || format!("w-[110px] flex items-center justify-center px-2 border-l border-white/5 shrink-0 relative z-20 {}", if row.get().revealed {"bg-green-500/10 text-green-400"} else {"bg-white/[0.02] text-white/20"})>
                <Show when=move || row.get().revealed fallback=|| view! {<StandingsIcon name="help" size=16 class="opacity-30"/>}>
                    <span class="standings-pop text-base font-bold tracking-widest relative"><Show when=move || row.get().highest><span class="absolute -top-1 -right-4 text-yellow-500 rotate-12"><StandingsIcon name="crown" size=12/></span></Show>"+"{move || number(row.get().points)}" "<span class="text-[10px] text-green-500/50">{move || t.get("lb_round_short")}</span></span>
                </Show>
            </div>
            <div class=move || format!("w-[100px] flex items-center justify-end px-4 border-l border-white/5 shrink-0 relative z-20 {}", if row.get().revealed && !final_step.get() {"bg-primary-900/20"} else {"bg-white/[0.02]"})>
                <span class=move || format!("font-mono font-black text-3xl transition-colors duration-500 {}", if winner() {"text-yellow-500"} else if row.get().revealed {"text-primary-400"} else {"text-white/40"})><AnimatedScore value=Signal::derive(move || row.get().score)/></span>
            </div>
            <div class="w-[120px] flex items-center justify-end px-4 bg-white/5 border-l border-white/10 shrink-0 relative z-20">
                <Show when=move || final_step.get() fallback=|| view! {<div class="w-8 h-1 bg-white/10 rounded-full"/>}>
                    <Show when=move || row.get().rank==1 fallback=move || view! {<div class="standings-pop flex flex-col items-end justify-center"><span class="font-mono font-bold text-white/80 text-base leading-none tracking-tighter">"-"{move || number(row.get().delta_leader)}</span><span class="font-mono font-bold text-white/40 text-[10px] leading-none mt-1 tracking-tighter">"(-"{move || number(row.get().delta_next)}")"</span></div>}>
                        <span class="standings-pop font-black text-yellow-500 text-sm tracking-widest uppercase">{move || t.get("lb_leader")}</span>
                    </Show>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn RaceView(rows: Memo<Vec<Standing>>, final_step: Signal<bool>) -> impl IntoView {
    let t = use_t();
    let geometry = Memo::new(move |_| race_geometry(&rows.get()));
    view! {
        <div class="grid gap-x-12 gap-y-[0.8cqh] content-start" style=move || format!("grid-template-columns:repeat({},minmax(0,1fr))",geometry.get().0)>
            <For each=move || rows.get() key=|r| r.key children=move |initial| {
                let key = initial.key;
                let row = Signal::derive(move || rows.get().into_iter().find(|r| r.key==key).expect("mounted row"));
                let winner = move || final_step.get() && row.get().rank==1;
                let bar = move || view! {
                    <div class=move || if geometry.get().0.gt(&1) {"relative rounded-full bg-white/5 overflow-hidden h-[1.7cqh]"} else {"relative rounded-full bg-white/5 overflow-hidden flex-1 h-[62%]"}>
                        <div class=move || format!("absolute inset-y-0 left-0 rounded-full transition-[width] duration-1000 ease-out {}", if winner() {"bg-gradient-to-r from-yellow-600 to-yellow-300"} else if row.get().revealed {"bg-gradient-to-r from-primary-700 to-primary-400"} else {"bg-white/15"}) style:width=move || {let (_,_,base,range)=geometry.get(); format!("{}%",((row.get().score-base)/range*100.).max(3.))}/>
                        <Show when=move || row.get().revealed && !final_step.get() && geometry.get().0==1><span class="absolute right-[1cqw] inset-y-0 flex items-center font-bold text-[1.6cqh] text-green-400">"+"{move || number(row.get().points)}" "{move || t.get("lb_round_short")}</span></Show>
                    </div>
                }.into_any();
                let name = move || view! {<span class=move || format!("truncate font-black uppercase tracking-wide {} {}",if winner() {"text-yellow-400"} else {"text-white"},if row.get().revealed {""} else {"opacity-60"})>{move || row.get().team.team.name}</span>};
                let score = move || view! {<span class=move || format!("shrink-0 text-right font-mono font-black {}", if winner() {"text-yellow-400"} else if row.get().revealed {"text-primary-300"} else {"text-white/40"})><AnimatedScore value=Signal::derive(move || row.get().score)/></span>};
                view! {
                    <div data-reorder="" class="flex items-center gap-[1.2cqw]" style=move || format!("view-transition-name:standings-team-{key};height:{}cqh",geometry.get().1)>
                        <span class=move || format!("w-[3cqw] shrink-0 text-center font-mono font-black text-[2.4cqh] {}",if winner() {"text-yellow-400"} else {"text-white/70"})>{move || row.get().rank}</span>
                        <TeamAvatar team=initial.team class="h-[80%] aspect-square rounded-full border border-white/20 shrink-0" text_class="text-[1.4cqh]"/>
                        <Show when=move || geometry.get().0.gt(&1) fallback=move || view! {<div class="w-[26%] shrink-0 text-[2.2cqh]">{name}</div>{bar()}<div class="w-[6cqw] text-[2.6cqh]">{score}</div>}>
                            <div class="flex-1 min-w-0 flex flex-col justify-center gap-[0.5cqh]"><div class="flex items-baseline gap-3"><div class="flex-1 min-w-0 text-[1.9cqh]">{name}</div><Show when=move || row.get().revealed && !final_step.get()><span class="shrink-0 font-bold text-[1.5cqh] text-green-400">"+"{move || number(row.get().points)}</span></Show><div class="text-[2.3cqh]">{score}</div></div>{bar()}</div>
                        </Show>
                    </div>
                }
            }/>
        </div>
    }
}

#[component]
fn FinalView(teams: Vec<Standing>, layout: LeaderboardLayout) -> impl IntoView {
    let rest = teams.iter().skip(3).cloned().collect::<Vec<_>>();
    let rest_delay = match layout {
        LeaderboardLayout::Spotlight => 2.4,
        LeaderboardLayout::Medals => 1.8,
        _ => 1.6,
    };
    let cards = [1,0,2].into_iter().filter_map(|i| teams.get(i).cloned().map(|t| (i,t))).map(|(i,team)| {
        let big = i==0;
        let delay = match (layout,i) {(LeaderboardLayout::Spotlight,0)=>1.5,(LeaderboardLayout::Spotlight,1)=>0.7,(LeaderboardLayout::Medals,0)=>1.,(LeaderboardLayout::Medals,1)=>0.5,(_,0)=>1.2,(_,1)=>0.6,_=>0.};
        match layout {
            LeaderboardLayout::Podium => {
                let (height,color) = match i {0=>("25cqh","bg-gradient-to-b from-yellow-300 to-yellow-500"),1=>("18cqh","bg-gradient-to-b from-slate-200 to-slate-400"),_=>("12cqh","bg-gradient-to-b from-orange-300 to-orange-600")};
                view! {<div class="standings-podium flex flex-col items-center justify-end w-[22%] min-w-0" style=format!("--delay:{delay}s")>
                    <TeamAvatar team=team.team.clone() class="w-[11cqh] h-[11cqh] rounded-full border-4 border-white/30 mb-[1.2cqh] shadow-2xl" text_class="text-[3.5cqh]"/>
                    <div class="w-full text-center truncate font-black uppercase tracking-wider text-[2.6cqh] text-white">{team.team.team.name}</div>
                    <div class="font-mono font-black text-[3.4cqh] text-primary-300 mb-[1cqh]">{number(team.score)}</div>
                    <div class=format!("w-full rounded-t-2xl flex items-start justify-center pt-[1.2cqh] shadow-[0_0_40px_rgba(0,0,0,0.5)] {color}") style:height=height><span class="font-title font-black text-[6cqh] leading-none text-black/60">{team.rank}</span></div>
                </div>}.into_any()
            },
            LeaderboardLayout::Spotlight => {
                let x = match i {0=>50,1=>20,_=>80};
                let beam = if big {32} else {20};
                view! {<div class="absolute bottom-0 flex flex-col items-center" style=format!("left:{x}%;transform:translateX(-50%);width:{}%;--delay:{delay}s",if big {30} else {22})>
                    <div class="standings-beam absolute bottom-0 left-1/2 -translate-x-1/2 pointer-events-none" style=format!("width:{beam}cqw;height:70cqh;clip-path:polygon(42% 0,58% 0,100% 100%,0 100%);background:linear-gradient(to bottom,rgba(255,244,214,{}),rgba(255,244,214,0.02))",if big {0.32} else {0.16})/>
                    <div class="standings-beam absolute bottom-[1cqh] left-1/2 -translate-x-1/2 rounded-[50%] pointer-events-none" style=format!("width:{}cqw;height:5cqh;background:radial-gradient(ellipse,rgba(255,244,214,{}),transparent 70%)",beam as f64*0.9,if big {0.35} else {0.18})/>
                    <div class="standings-spot relative flex flex-col items-center pb-[3cqh] w-full">
                        {big.then(|| view! {<StandingsIcon name="crown" size=64 class="text-yellow-400 drop-shadow-[0_0_20px_rgba(234,179,8,0.8)] mb-[0.5cqh]"/>})}
                        <TeamAvatar team=team.team.clone() class=if big {"w-[22cqh] h-[22cqh] rounded-full border-4 border-yellow-300/80 shadow-2xl mb-[1.2cqh]"} else {"w-[13cqh] h-[13cqh] rounded-full border-4 border-white/30 shadow-2xl mb-[1.2cqh]"} text_class=if big {"text-[6cqh]"} else {"text-[3.5cqh]"}/>
                        <div class=if big {"w-full text-center truncate font-title font-black uppercase text-[4.4cqh] text-yellow-300"} else {"w-full text-center truncate font-title font-black uppercase text-[2.6cqh] text-white"}>{team.team.team.name}</div>
                        <div class=if big {"font-mono font-black text-[4cqh] text-white"} else {"font-mono font-black text-[2.8cqh] text-white/70"}>{number(team.score)}</div>
                        <div class=if big {"mt-[0.6cqh] px-4 py-1 rounded-full font-black text-[1.6cqh] bg-yellow-400 text-black"} else {"mt-[0.6cqh] px-4 py-1 rounded-full font-black text-[1.6cqh] bg-white/10 text-white/80"}>"#"{team.rank}</div>
                    </div>
                </div>}.into_any()
            },
            _ => {
                let (card,medal,ribbon) = match i {0=>("from-yellow-300/25 to-yellow-700/10 border-yellow-300/70","from-yellow-200 to-yellow-500","bg-primary-600"),1=>("from-slate-200/20 to-slate-600/10 border-slate-200/60","from-slate-100 to-slate-400","bg-primary-700"),_=>("from-orange-300/20 to-orange-800/10 border-orange-300/60","from-orange-200 to-orange-600","bg-primary-800")};
                view! {<div class=format!("standings-medal relative flex flex-col items-center w-[24%] rounded-[2rem] border-2 bg-gradient-to-b {card} px-[1.5cqw] pb-[2.5cqh]") style=format!("--delay:{delay}s;height:{}",if big {"100%"} else {"86%"})>
                    <div class="absolute top-0 left-1/2 -translate-x-1/2 flex gap-[0.6cqw]"><div class=format!("w-[2.2cqw] h-[7cqh] {ribbon} origin-top -rotate-12")/><div class=format!("w-[2.2cqw] h-[7cqh] {ribbon} origin-top rotate-12")/></div>
                    <div class=format!("relative mt-[4.5cqh] shrink-0 {} rounded-full bg-gradient-to-br {medal} border-4 border-white/50 shadow-[0_0_40px_rgba(0,0,0,0.5)] flex items-center justify-center",if big {"w-[11cqh] h-[11cqh]"} else {"w-[9cqh] h-[9cqh]"})><span class=if big {"font-title font-black text-black/70 text-[5.5cqh]"} else {"font-title font-black text-black/70 text-[4.5cqh]"}>{team.rank}</span></div>
                    <div class="mt-auto flex flex-col items-center w-full min-h-0">
                        <TeamAvatar team=team.team.clone() class=if big {"w-[14cqh] h-[14cqh] rounded-full border-4 border-white/30 mb-[1.2cqh] shadow-2xl"} else {"w-[10cqh] h-[10cqh] rounded-full border-4 border-white/30 mb-[1.2cqh] shadow-2xl"} text_class="text-[3.5cqh]"/>
                        <div class=if big {"w-full text-center truncate font-title font-black uppercase text-white text-[3.6cqh]"} else {"w-full text-center truncate font-title font-black uppercase text-white text-[2.8cqh]"}>{team.team.team.name}</div>
                        <div class=if big {"font-mono font-black text-primary-300 text-[4cqh]"} else {"font-mono font-black text-primary-300 text-[3.2cqh]"}>{number(team.score)}</div>
                    </div>
                </div>}.into_any()
            }
        }
    }).collect_view();
    view! {<div class="h-full flex flex-col">
        <div class=if layout==LeaderboardLayout::Spotlight {"relative flex-1 min-h-0"} else if layout==LeaderboardLayout::Medals {"flex-1 min-h-0 flex items-end justify-center gap-[2.5cqw] pt-[2cqh]"} else {"flex-1 min-h-0 flex items-end justify-center gap-[2cqw]"} style:perspective="1200px">{cards}</div>
        {(!rest.is_empty()).then(|| view! {<div class="grid grid-cols-3 gap-x-6 gap-y-[0.8cqh] mt-[2.5cqh] shrink-0">
            {rest.into_iter().enumerate().map(|(i,r)| view! {<div class="standings-rest flex items-center gap-3 h-[4cqh] px-3 rounded-lg bg-black/40 border border-white/10" style=format!("--i:{i};--delay:{rest_delay}s")>
                <span class="w-[2.5cqw] font-mono font-black text-[1.8cqh] text-white/60">{r.rank}</span><TeamAvatar team=r.team.clone() class="h-[70%] aspect-square rounded-full border border-white/20 shrink-0" text_class="text-[1.1cqh]"/>
                <span class="flex-1 truncate font-bold uppercase tracking-wider text-[1.7cqh] text-white">{r.team.team.name}</span><span class="font-mono font-black text-[2cqh] text-primary-300">{number(r.score)}</span>
            </div>}).collect_view()}
        </div>})}
    </div>}
}

#[cfg(test)]
mod tests {
    use super::*;
    use quiz_core::quiz::parse::ScoreTeam;
    fn fixture() -> ParsedScores {
        ParsedScores {
            rounds: vec!["R1".into(), "R2".into()],
            teams: [("A", 10., 2.), ("B", 8., 5.), ("C", 8., 5.), ("D", 0., 0.)]
                .into_iter()
                .map(|(name, a, b)| ScoreTeam {
                    name: name.into(),
                    scores: [("R1".into(), a), ("R2".into(), b)].into(),
                    id: None,
                })
                .collect(),
        }
    }
    #[test]
    fn early_navigation_waits_for_scores_then_adopts_late_data() {
        let mut reveal = Reveal {
            snapshot: empty_scores(),
            selected_round: String::new(),
            reveal_step: 0,
        };
        for _ in 0..3 {
            let board = build_board(&reveal.snapshot, &reveal.selected_round, &[]);
            reveal.reveal_step = next_step(&board, reveal.reveal_step);
        }
        assert_eq!(reveal.reveal_step, 0);
        reveal.adopt(fixture());
        let board = build_board(&reveal.snapshot, &reveal.selected_round, &[]);
        assert_eq!(reveal.selected_round, "R2");
        assert_eq!(active_teams(&board, reveal.reveal_step).len(), 4);
        assert_eq!(next_step(&board, reveal.reveal_step), 1);
        assert_eq!(next_step(&board, max_step(&board)), max_step(&board));
    }
    #[test]
    fn group_reveal_and_final_ties() {
        let board = build_board(&fixture(), "R2", &[]);
        assert_eq!(board.groups, vec![0., 2., 5.]);
        assert_eq!(max_step(&board), 7);
        assert_eq!(
            board
                .teams
                .iter()
                .map(|t| t.previous_rank)
                .collect::<Vec<_>>(),
            vec![1, 2, 2, 4]
        );
        for step in 0..=7 {
            let rows = active_teams(&board, step);
            assert_eq!(
                active_group(step, 7),
                if step % 2 == 1 && step < 7 {
                    Some(step / 2)
                } else {
                    None
                }
            );
            for r in &rows {
                assert_eq!(r.revealed, step >= r.group * 2 + 2);
            }
            if step < 7 {
                assert_eq!(
                    rows.iter().map(|r| r.key).collect::<Vec<_>>(),
                    vec![0, 1, 2, 3]
                );
            }
        }
        let rows = active_teams(&board, 7);
        assert_eq!(
            rows.iter().map(|r| r.key).collect::<Vec<_>>(),
            vec![1, 2, 0, 3]
        );
        assert_eq!(
            rows.iter().map(|r| r.rank).collect::<Vec<_>>(),
            vec![1, 1, 3, 4]
        );
        assert_eq!(
            rows.iter().map(|r| r.change).collect::<Vec<_>>(),
            vec![1, 1, -2, 0]
        );
        assert_eq!((rows[2].delta_leader, rows[2].delta_next), (1., 1.));
    }
    #[test]
    fn defaults_empty_invalid_zero_and_negative() {
        let mut scores = fixture();
        assert_eq!(default_round(&scores), "R2");
        for row in &mut scores.teams {
            row.scores.insert("R2".into(), 0.);
        }
        assert_eq!(default_round(&scores), "R1");
        assert!(build_board(&scores, "missing", &[]).teams.is_empty());
        assert_eq!(default_round(&empty_scores()), "");
        let board = build_board(&scores, "R2", &[]);
        assert_eq!(board.groups, vec![0.]);
        assert!(board.teams.iter().all(|t| !t.highest && !t.fire));
        scores.teams[0].scores.insert("R2".into(), -0.5);
        let board = build_board(&scores, "R2", &[]);
        assert_eq!(board.groups, vec![-0.5, 0.]);
        assert_eq!(active_teams(&board, 2)[0].score, 9.5);
    }
    #[test]
    fn cumulative_scores_fire_and_metadata() {
        let mut scores = fixture();
        scores.teams[0].scores.insert("R2".into(), 5.);
        let meta = resolve_teams(
            &[Team {
                name: "A".into(),
                quote: Some("hello".into()),
                ..Team::default()
            }],
            |_| None,
        );
        let board = build_board(&scores, "R2", &meta);
        assert!(board.teams[0].fire);
        assert!(board.teams[0].highest);
        assert_eq!(board.teams[0].total, 15.);
        assert_eq!(board.teams[0].team.team.quote.as_deref(), Some("hello"));
        assert!(!board.teams[1].fire);
        let first = build_board(&scores, "R1", &[]);
        assert!(
            first
                .teams
                .iter()
                .all(|t| t.previous == 0. && !t.fire && t.previous_rank == 1)
        );
    }
    #[test]
    fn layouts_and_race_baseline() {
        for l in [
            LeaderboardLayout::Table,
            LeaderboardLayout::Race,
            LeaderboardLayout::Podium,
            LeaderboardLayout::Spotlight,
            LeaderboardLayout::Medals,
        ] {
            assert_eq!(shown_layout(l, true), l);
            assert_eq!(
                shown_layout(l, false),
                if l == LeaderboardLayout::Race {
                    l
                } else {
                    LeaderboardLayout::Table
                }
            );
        }
        let board = build_board(&fixture(), "R2", &[]);
        assert_eq!(race_geometry(&board.teams).0, 1);
        let crowded = vec![board.teams[0].clone(); 13];
        assert_eq!(race_geometry(&crowded).0, 2);
        assert_eq!(number(1.5), "1.5");
        assert_eq!(number(2.), "2");
    }
    #[test]
    fn presenter_actions_focus_and_round_selection() {
        let board = build_board(&fixture(), "R2", &[]);
        assert_eq!(next_action(&board, 0), ("lb_reveal_group", "0".into()));
        assert_eq!(next_action(&board, 1), ("lb_apply", "0".into()));
        assert_eq!(next_action(&board, 6).0, "lb_shuffle");
        assert_eq!(next_action(&board, 7).0, "lb_done");
        assert_eq!(focus_rows(&board, 0)[0].team.team.name, "D");
        assert_eq!(focus_rows(&board, 2)[0].team.team.name, "A");
        for step in [6, 7] {
            assert_eq!(
                focus_rows(&board, step)
                    .iter()
                    .map(|r| r.team.team.name.as_str())
                    .collect::<Vec<_>>(),
                vec!["B", "C"]
            );
        }
        let mut reveal = Reveal {
            snapshot: fixture(),
            selected_round: "R1".into(),
            reveal_step: 0,
        };
        reveal.adopt(fixture());
        assert_eq!(reveal.selected_round, "R1");
        reveal.selected_round = "removed".into();
        reveal.adopt(fixture());
        assert_eq!(reveal.selected_round, "R2");
        assert!(focus_rows(&build_board(&empty_scores(), "", &[]), 0).is_empty());
    }
    #[test]
    fn wire_matches_react() {
        let scores = fixture();
        let reveal = Reveal {
            selected_round: default_round(&scores),
            snapshot: scores,
            reveal_step: 3,
        };
        let wire = serde_json::to_value(&reveal).unwrap();
        assert_eq!(wire["revealStep"], 3);
        assert_eq!(wire["selectedRound"], "R2");
        assert_eq!(serde_json::from_value::<Reveal>(wire).unwrap(), reveal);
    }
}
