//! Intermission timer, presenter controls and rotating team spotlight.
use super::{
    icons::IntroIcon,
    team_avatar::{TeamAvatar, team_color},
};
use crate::{
    i18n::use_t, presence::PresenceSwitch, quiz_context::use_quiz, scores::use_scores,
    show::use_show,
};
use leptos::prelude::*;
use quiz_core::{
    quiz::{
        model::{ResolvedTeam, Team, resolve_teams},
        parse::ParsedScores,
        teams::team_for_scores,
    },
    show::{Action, Role},
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, closure::Closure};

fn format_time(seconds: u32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}
fn adjust_time(seconds: u32, minutes: i32) -> u32 {
    (i64::from(seconds) + i64::from(minutes) * 60).clamp(0, i64::from(u32::MAX)) as u32
}
fn tick(seconds: u32, active: bool) -> u32 {
    if active {
        seconds.saturating_sub(1)
    } else {
        seconds
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SpotlightTeam {
    avatar: ResolvedTeam,
    total: f64,
    rank: usize,
    history: Vec<usize>,
    has_metadata: bool,
}
impl SpotlightTeam {
    fn color(&self) -> &'static str {
        if self.has_metadata {
            team_color(self.avatar.color)
        } else {
            "from-primary-900 to-primary-950"
        }
    }
}

/// Stable standard competition ranking: ties share a rank, then skip places (1, 1, 3).
fn ranks(totals: &[f64]) -> Vec<usize> {
    let mut order: Vec<_> = (0..totals.len()).collect();
    order.sort_by(|&a, &b| totals[b].total_cmp(&totals[a]));
    let mut ranks = vec![1; totals.len()];
    let mut rank = 1;
    for (i, &index) in order.iter().enumerate() {
        if i > 0 && totals[index] < totals[order[i - 1]] {
            rank = i + 1;
        }
        ranks[index] = rank;
    }
    ranks
}
fn spotlight_teams(board: &ParsedScores, metadata: &[ResolvedTeam]) -> Vec<SpotlightTeam> {
    let mut totals = vec![0.0; board.teams.len()];
    let mut history = vec![Vec::new(); board.teams.len()];
    for round in &board.rounds {
        // The React graph only includes rounds with at least one positive score.
        if !board
            .teams
            .iter()
            .any(|team| team.scores.get(round).is_some_and(|p| *p > 0.0))
        {
            continue;
        }
        for (total, team) in totals.iter_mut().zip(&board.teams) {
            *total += team.scores.get(round).copied().unwrap_or_default();
        }
        for (history, rank) in history.iter_mut().zip(ranks(&totals)) {
            history.push(rank);
        }
    }
    let final_ranks = ranks(&totals);
    let raw: Vec<Team> = metadata.iter().map(|t| t.team.clone()).collect();
    let mut teams: Vec<_> = board
        .teams
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let meta = team_for_scores(&raw, row)
                .and_then(|team| metadata.iter().find(|m| &m.team == team));
            let mut avatar = meta.cloned().unwrap_or_else(|| {
                resolve_teams(
                    &[Team {
                        name: row.name.clone(),
                        ..Team::default()
                    }],
                    |_| None,
                )
                .remove(0)
            });
            avatar.team.name = row.name.clone();
            SpotlightTeam {
                avatar,
                total: totals[i],
                rank: final_ranks[i],
                history: history[i].clone(),
                has_metadata: meta.is_some(),
            }
        })
        .collect();
    teams.sort_by(|a, b| b.total.total_cmp(&a.total));
    teams
}
fn graph_point(index: usize, count: usize, rank: usize, max_rank: usize) -> (f64, f64) {
    (
        20.0 + index as f64 / count.saturating_sub(1).max(1) as f64 * 350.0,
        20.0 + rank.saturating_sub(1) as f64 / max_rank.saturating_sub(1).max(1) as f64 * 70.0,
    )
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
enum TimerMessage {
    #[serde(rename_all = "camelCase")]
    TimerSync { time_left: u32, is_active: bool },
}

#[component]
pub fn Pause() -> impl IntoView {
    let show = use_show();
    let quiz = use_quiz();
    let t = use_t();
    let presenter = show.role == Role::Presenter;
    let time_left = RwSignal::new(30 * 60_u32);
    let active = RwSignal::new(false);
    let spotlight = RwSignal::new(0_u64);
    let live = use_scores().scores;
    let teams = Memo::new(move |_| spotlight_teams(&live.get(), &quiz.teams));
    let count = Signal::derive(move || teams.with(Vec::len));
    let channel = StoredValue::new_local(web_sys::BroadcastChannel::new("quiz-pause-sync").ok());
    let incoming =
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
            if presenter {
                return;
            }
            if let Ok(json) = js_sys::JSON::stringify(&event.data()) {
                if let Some(json) = json.as_string() {
                    if let Ok(TimerMessage::TimerSync {
                        time_left: seconds,
                        is_active,
                    }) = serde_json::from_str(&json)
                    {
                        time_left.set(seconds);
                        active.set(is_active);
                    }
                }
            }
        });
    channel.with_value(|c| {
        if let Some(c) = c {
            c.set_onmessage(Some(incoming.as_ref().unchecked_ref()));
        }
    });
    Effect::new(move |_| {
        let message = TimerMessage::TimerSync {
            time_left: time_left.get(),
            is_active: active.get(),
        };
        if presenter {
            channel.with_value(|c| {
                if let Some(c) = c {
                    if let Ok(json) = serde_json::to_string(&message) {
                        if let Ok(value) = js_sys::JSON::parse(&json) {
                            let _ = c.post_message(&value);
                        }
                    }
                }
            });
        }
    });
    let timer = StoredValue::new_local(None::<gloo_timers::callback::Interval>);
    Effect::new(move |_| {
        let running = active.get();
        let seconds = time_left.get();
        timer.update_value(|timer| {
            *timer = None;
            if presenter && running && seconds > 0 {
                *timer = Some(gloo_timers::callback::Interval::new(1000, move || {
                    time_left.update(|seconds| *seconds = tick(*seconds, true));
                }));
            }
        });
    });
    let rotation = (!presenter).then(|| {
        gloo_timers::callback::Interval::new(8000, move || {
            let count = count.get_untracked();
            if count > 0 {
                spotlight.update(|i| *i = (*i + 1) % count as u64);
            }
        })
    });
    let resources = StoredValue::new_local(Some((incoming, rotation)));
    on_cleanup(move || {
        timer.update_value(|timer| *timer = None);
        channel.with_value(|c| {
            if let Some(c) = c {
                c.set_onmessage(None);
                c.close();
            }
        });
        resources.update_value(|r| {
            r.take();
        });
    });
    // TODO(sounds)
    if presenter {
        // TODO(pause-presenter-layout): outer Stage must honor Show::uses_full_window.
        return view! {
            <div class="h-full w-full bg-panel text-white p-10 font-sans flex flex-col items-center justify-center relative">
                <button on:click=move |_| show.dispatch.run(Action::BackToDashboard) class="absolute top-10 left-10 flex items-center gap-3 px-6 py-3 bg-white/10 hover:bg-white/20 rounded-xl transition-all font-bold">
                    <PauseIcon name="back" size=20/>{move || t.get("pause_back")}
                </button>
                <PauseIcon name="coffee" size=80 class="text-primary-500 mb-6 opacity-50"/>
                <h1 class="text-4xl font-black uppercase tracking-widest text-primary-400 mb-12">{move || t.get("pause_controls")}</h1>
                <div class="text-[120px] font-mono font-bold leading-none mb-12 drop-shadow-[0_0_30px_rgb(var(--glow-primary)/0.5)]">{move || format_time(time_left.get())}</div>
                <div class="flex gap-6 items-center bg-white/5 p-8 rounded-3xl border border-white/10">
                    <button aria-label="Subtract one minute" on:click=move |_| time_left.update(|s| *s = adjust_time(*s, -1)) class="p-5 bg-white/5 hover:bg-white/10 rounded-2xl transition-all"><PauseIcon name="minus" size=32/></button>
                    <button on:click=move |_| active.update(|v| *v = !*v) class=move || format!("flex items-center gap-4 px-12 py-6 rounded-2xl font-black text-2xl tracking-widest uppercase transition-all shadow-xl {}", if active.get() { "bg-yellow-500 hover:bg-yellow-400 text-black shadow-yellow-500/20" } else { "bg-green-600 hover:bg-green-500 text-white shadow-green-500/20" })>
                        {move || if active.get() { view! { <PauseIcon name="pause" size=32/> }.into_any() } else { view! { <PauseIcon name="play" size=32/> }.into_any() }}
                        {move || t.get(if active.get() { "timer_pause" } else { "timer_start" })}
                    </button>
                    <button aria-label="Add one minute" on:click=move |_| time_left.update(|s| *s = adjust_time(*s, 1)) class="p-5 bg-white/5 hover:bg-white/10 rounded-2xl transition-all"><PauseIcon name="plus" size=32/></button>
                </div>
            </div>
        }.into_any();
    }
    view! {
        <div class="pause-screen h-full w-full bg-ground relative overflow-hidden flex flex-col font-show selection:bg-primary-500/30">
            <div class="absolute inset-0 bg-ground-bottom z-0 pointer-events-none"/>
            <div class="pause-background absolute inset-0 pointer-events-none">
                <PresenceSwitch scene=Signal::derive(move || spotlight.get()) wait=Signal::stored(false) enter="pause-bg-enter" exit="pause-bg-exit" render=move |index| {
                    move || teams.with(|teams| teams.get(index as usize % teams.len().max(1)).map(|team| view! {
                        <div class=format!("absolute inset-0 bg-gradient-to-br {} z-0 pointer-events-none blur-3xl", team.color())/>
                    }))
                }/>
            </div>
            <div class="relative z-10 flex flex-col h-full w-full">
                <div class="flex-none h-[22cqh] flex flex-col items-center justify-center pt-8">
                    <div class="flex items-center gap-4 text-primary-500 mb-2 animate-pulse">
                        <PauseIcon name="coffee" size=28/><h2 class="font-title text-2xl font-black uppercase tracking-[0.4em]">{move || t.get("pause_title")}</h2><PauseIcon name="coffee" size=28/>
                    </div>
                    <div class=move || format!("text-[120px] font-mono font-black leading-none drop-shadow-[0_0_50px_rgb(var(--glow-primary)/0.3)] transition-colors duration-1000 {}", if time_left.get() <= 60 { "text-red-500 drop-shadow-[0_0_50px_rgba(239,68,68,0.5)]" } else { "text-white" })>{move || format_time(time_left.get())}</div>
                    <div class="text-white/40 font-bold tracking-[0.4em] text-xl uppercase mt-4">{move || t.get(if time_left.get() == 0 { "pause_soon" } else { "pause_until" })}</div>
                </div>
                <div class="pause-spotlight flex-1 flex items-center justify-center p-12 w-full max-w-[1700px] mx-auto">
                    <PresenceSwitch scene=Signal::derive(move || spotlight.get()) wait=Signal::stored(true) enter="pause-card-enter" exit="pause-card-exit" render=move |index| {
                        move || teams.with(|teams| teams.get(index as usize % teams.len().max(1)).cloned().map(|team| view! { <SpotlightCard team=team max_rank=count.get()/> }))
                    }/>
                </div>
                <div class="flex-none h-[10cqh] bg-primary-950/20 border-t border-primary-500/20 flex items-center overflow-hidden backdrop-blur-md relative z-20">
                    <div class="h-full bg-primary-600 px-8 flex items-center justify-center relative z-30 shadow-[10px_0_20px_rgba(0,0,0,0.5)]">
                        <span class="font-black text-white uppercase tracking-[0.3em] text-lg whitespace-nowrap">{move || t.get("pause_live")}</span>
                        <div class="absolute right-[-14px] top-1/2 -translate-y-1/2 w-0 h-0 border-t-[15px] border-t-transparent border-l-[15px] border-l-primary-600 border-b-[15px] border-b-transparent"/>
                    </div>
                    <div class="pause-mask flex-1 overflow-hidden h-full flex items-center relative">
                        <div class="pause-ticker flex whitespace-nowrap items-center h-full" style=move || format!("--pause-ticker-duration:{}s", (count.get() as f64 * 3.5).max(0.1))>
                            {move || teams.with(|teams| teams.iter().chain(teams.iter()).map(|team| view! {
                                <div class="flex items-center gap-6 px-12 border-r border-white/10 h-[50%]">
                                    <span class=if team.rank == 1 { "text-2xl font-black text-yellow-500" } else { "text-2xl font-black text-white/30" }>{team.rank}"."</span>
                                    <span class="text-3xl font-bold text-white uppercase tracking-wider">{team.avatar.team.name.clone()}</span>
                                    <span class="text-3xl font-mono font-black text-primary-400">"["{team.total}" "{move || t.get("pts")}"]"</span>
                                </div>
                            }).collect_view())}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }.into_any()
}

#[component]
fn SpotlightCard(team: SpotlightTeam, max_rank: usize) -> impl IntoView {
    let t = use_t();
    let color = team.color();
    let rank = team.rank;
    let quote = team.avatar.team.quote.clone().unwrap_or_default();
    view! {
        <div class="w-full bg-black/40 backdrop-blur-2xl border border-white/10 rounded-[4rem] p-12 flex items-center gap-16 shadow-[0_0_100px_rgba(0,0,0,0.5)] relative overflow-hidden">
            <div class=format!("absolute top-0 left-0 w-[500px] h-[500px] bg-gradient-to-br {color} opacity-20 blur-[100px] pointer-events-none rounded-full -translate-x-1/2 -translate-y-1/2")/>
            <TeamAvatar team=team.avatar.clone() class="w-[400px] h-[400px] rounded-3xl border border-white/10 shadow-2xl relative z-10" text_class="text-[150px]"/>
            <div class="flex-1 flex flex-col justify-center relative z-10 h-full py-2">
                <div class="flex items-center gap-3 mb-4">
                    <IntroIcon name=if rank == 1 { "trophy" } else { "medal" } size=32 class=if rank == 1 { "text-yellow-500" } else { "text-white/30" }/>
                    <span class=if rank == 1 { "text-xl font-bold tracking-[0.3em] uppercase text-yellow-500" } else { "text-xl font-bold tracking-[0.3em] uppercase text-white/40" }>{move || t.fmt("pause_rank", &[("rank", &rank.to_string())])}</span>
                </div>
                <h1 class="text-[70px] font-black text-white uppercase tracking-tight leading-none mb-6 drop-shadow-lg line-clamp-2">{team.avatar.team.name}</h1>
                <div class="flex justify-between items-start gap-12 w-full pr-10">
                    <div class="flex-1">
                        {(!quote.is_empty()).then(|| view! {
                            <div class="relative pl-6"><div class="absolute left-0 top-0 bottom-0 w-1.5 bg-white/20 rounded-full"/>
                                <p class="text-2xl text-white/60 font-medium italic leading-relaxed line-clamp-2">"\""{quote}"\""</p>
                            </div>
                        })}
                        <RankGraph history=team.history max_rank=max_rank/>
                    </div>
                    <div class="flex flex-col items-end shrink-0 mt-auto">
                        <span class="text-[110px] font-mono font-black text-primary-400 leading-none drop-shadow-[0_0_30px_rgb(var(--glow-primary)/0.3)]">{team.total}</span>
                        <span class="text-2xl text-white/30 font-bold tracking-widest font-sans mt-2">{move || t.get("total_pts")}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn RankGraph(history: Vec<usize>, max_rank: usize) -> impl IntoView {
    let t = use_t();
    if history.len() < 2 {
        return view! { <div class="mt-8 text-white/30 text-xs tracking-widest uppercase italic flex items-center gap-2"><PauseIcon name="trend" size=16/>{move || t.get("pause_trend_more")}</div> }.into_any();
    }
    let count = history.len();
    let points = history
        .iter()
        .enumerate()
        .map(|(i, &rank)| {
            let (x, y) = graph_point(i, count, rank, max_rank);
            format!("{x},{y}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    view! {
        <div class="mt-8 flex flex-col items-start relative z-10">
            <span class="text-[20px] font-bold text-white/30 uppercase tracking-[0.3em] mb-6">{move || t.get("pause_trend")}</span>
            <svg width="390" height="110" class="overflow-visible -ml-[20px]">
                <polyline points=points fill="none" stroke="rgba(255,255,255,0.2)" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"/>
                {history.into_iter().enumerate().map(|(i, rank)| {
                    let (x, y) = graph_point(i, count, rank, max_rank);
                    let last = i == count - 1;
                    view! { <g>
                        <circle cx=x cy=y r=if last {6} else {4} fill=if last {"#60A5FA"} else {"#fff"}/>
                        <text x=x y=y-14.0 fill=if last {"#60A5FA"} else {"rgba(255,255,255,0.6)"} font-size="30" font-weight="900" text-anchor="middle" class="drop-shadow-md">{rank}"."</text>
                    </g> }
                }).collect_view()}
            </svg>
            <div class="flex justify-between w-[350px] mt-2 text-[20px] text-white/30 font-bold tracking-widest uppercase"><span>{move || t.get("pause_start")}</span><span>{move || t.get("pause_now")}</span></div>
        </div>
    }.into_any()
}

/// Scene-local Lucide paths; no additional Cargo icon features required.
#[component]
fn PauseIcon(
    name: &'static str,
    size: u32,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let paths = match name {
        "coffee" => view! { <g><path d="M18 8h1a4 4 0 0 1 0 8h-1"/><path d="M3 8h15v9a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4z"/><path d="M6 1v3M10 1v3M14 1v3"/></g> }.into_any(),
        "back" => view! { <g><path d="m12 19-7-7 7-7M19 12H5"/></g> }.into_any(),
        "minus" => view! { <path d="M5 12h14"/> }.into_any(),
        "plus" => view! { <path d="M5 12h14M12 5v14"/> }.into_any(),
        "pause" => view! { <g fill="currentColor"><rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/></g> }.into_any(),
        "play" => view! { <polygon points="6,3 20,12 6,21" fill="currentColor"/> }.into_any(),
        _ => view! { <g><path d="m22 7-8.5 8.5-5-5L2 17"/><path d="M16 7h6v6"/></g> }.into_any(),
    };
    view! { <svg width=size height=size class=class viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{paths}</svg> }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quiz_core::quiz::parse::ScoreTeam;
    use std::collections::BTreeMap;
    #[test]
    fn timer_format_adjust_and_floor() {
        assert_eq!(format_time(1800), "30:00");
        assert_eq!(format_time(61), "1:01");
        assert_eq!(format_time(0), "0:00");
        assert_eq!(format_time(3600), "60:00");
        assert_eq!(adjust_time(30, -1), 0);
        assert_eq!(adjust_time(1800, 1), 1860);
        assert_eq!(adjust_time(u32::MAX, 1), u32::MAX);
        assert_eq!(tick(1, true), 0);
        assert_eq!(tick(0, true), 0);
        assert_eq!(tick(60, false), 60);
    }
    #[test]
    fn competition_ties_and_active_round_history() {
        let row = |name: &str, scores: [f64; 3]| ScoreTeam {
            name: name.into(),
            id: None,
            scores: ["one", "two", "inactive"]
                .into_iter()
                .zip(scores)
                .map(|(r, p)| (r.into(), p))
                .collect::<BTreeMap<_, _>>(),
        };
        let board = ParsedScores {
            rounds: vec!["one".into(), "two".into(), "inactive".into()],
            teams: vec![
                row("A", [5., 0., -10.]),
                row("B", [5., 3., 0.]),
                row("C", [1., 4., 0.]),
            ],
        };
        let teams = spotlight_teams(&board, &[]);
        assert_eq!(
            teams
                .iter()
                .map(|t| (t.avatar.team.name.as_str(), t.rank, t.total))
                .collect::<Vec<_>>(),
            vec![("B", 1, 8.), ("A", 2, 5.), ("C", 2, 5.)]
        );
        assert_eq!(teams[0].history, vec![1, 1]);
        assert_eq!(teams[1].history, vec![1, 2]);
        assert_eq!(teams[2].history, vec![3, 2]);
        assert_eq!(ranks(&[5., 5., 1.]), vec![1, 1, 3]);
        assert!(
            spotlight_teams(
                &ParsedScores {
                    teams: vec![],
                    rounds: vec![]
                },
                &[]
            )
            .is_empty()
        );
    }
    #[test]
    fn graph_extents_and_single_team_are_finite() {
        assert_eq!(graph_point(0, 2, 1, 3), (20., 20.));
        assert_eq!(graph_point(1, 2, 3, 3), (370., 90.));
        assert_eq!(graph_point(1, 2, 1, 1), (370., 20.));
    }
    #[test]
    fn score_identity_preserves_avatar_and_quote() {
        use quiz_core::quiz::model::TeamId;
        let metadata = resolve_teams(
            &[Team {
                id: Some(TeamId::Text("known".into())),
                name: "Original name".into(),
                quote: Some("Our motto".into()),
                image: Some("portrait.png".into()),
                ..Team::default()
            }],
            |path| Some(format!("source/{path}")),
        );
        let board = ParsedScores {
            teams: vec![ScoreTeam {
                id: Some("known".into()),
                name: "Scoreboard name".into(),
                scores: BTreeMap::new(),
            }],
            rounds: vec![],
        };
        let teams = spotlight_teams(&board, &metadata);
        assert_eq!(teams[0].avatar.team.name, "Scoreboard name");
        assert_eq!(teams[0].avatar.team.quote.as_deref(), Some("Our motto"));
        assert_eq!(
            teams[0].avatar.image.as_deref(),
            Some("source/portrait.png")
        );
        assert_eq!(teams[0].rank, 1);
        assert!(teams[0].history.is_empty());
        assert!(teams[0].has_metadata);
    }
    #[test]
    fn timer_wire_format_matches_react() {
        let message = TimerMessage::TimerSync {
            time_left: 42,
            is_active: true,
        };
        let value = serde_json::to_value(message).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"type":"TIMER_SYNC", "timeLeft":42, "isActive":true})
        );
        assert!(serde_json::from_value::<TimerMessage>(value).is_ok());
    }
}
