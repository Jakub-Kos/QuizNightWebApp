//! All present teams, with a themed rotating introduction.
use super::{
    team_avatar::{TeamAvatar, team_color},
    team_stats::WelcomeIcon,
};
use crate::{i18n::use_t, presence::Presence, spring::Spring, theme::use_theme};
use leptos::{
    leptos_dom::helpers::{TimeoutHandle, set_timeout_with_handle},
    prelude::*,
};
use quiz_core::{
    quiz::model::{ResolvedTeam, TEAM_HEX},
    theme::WelcomeIntro,
};
use std::time::Duration;

fn grid(count: usize) -> (usize, usize) {
    let rows = ((count as f64 / 1.8).sqrt().round() as usize).max(1);
    (rows, count.div_ceil(rows).max(1))
}
fn entrance_delay(index: usize) -> u64 {
    150 + index as u64 * 70
}
fn initial_delay(count: usize) -> u64 {
    1500 + count as u64 * 70
}
fn next_team(index: usize, count: usize) -> Option<usize> {
    (count > 0).then(|| index % count)
}
fn name_font(name: &str, rows: usize) -> String {
    let text = name.trim();
    let length = text.chars().count().max(4);
    let longest = text
        .split_whitespace()
        .map(|w| w.chars().count())
        .max()
        .unwrap_or(4)
        .max(4);
    let lines = if length > 12 && text.chars().any(char::is_whitespace) {
        2
    } else {
        1
    };
    let per_line = if lines == 2 {
        longest.max(length.div_ceil(2) + 2)
    } else {
        length
    };
    let max = match rows {
        0..=2 => 3.0,
        3 => 2.25,
        4 => 1.875,
        _ => 1.5,
    };
    format!(
        "min({max}rem, {:.1}cqi, {:.1}cqh)",
        150.0 / per_line as f64,
        28.0 / lines as f64
    )
}
fn intro_font(name: &str, max: &str, width: f64) -> String {
    format!(
        "min({max}, {:.1}cqw)",
        width / name.chars().count().clamp(6, 18) as f64
    )
}
fn tilt(name: &str) -> i32 {
    // Array.from(name).map(c => c.charCodeAt(0)): astral characters contribute
    // their leading UTF-16 surrogate, matching the React photo's stable tilt.
    (name
        .chars()
        .map(|c| u32::from(c.encode_utf16(&mut [0; 2])[0]))
        .sum::<u32>()
        % 7) as i32
        - 3
}
fn schedule(timer: StoredValue<Option<TimeoutHandle>>, ms: u64, f: impl FnOnce() + 'static) {
    if let Some(h) = timer.get_value() {
        h.clear();
    }
    timer.set_value(set_timeout_with_handle(f, Duration::from_millis(ms)).ok());
}

/// Pause starts after the outgoing intro's Presence has finished, matching useTakeover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IntroPhase {
    Initial,
    Showing(usize),
    Exiting(usize),
    Pause(usize),
}
fn phase_timer(
    phase: IntroPhase,
    count: usize,
    intro_ms: u64,
    pause_ms: u64,
) -> Option<(u64, IntroPhase)> {
    if count == 0 {
        return None;
    }
    match phase {
        IntroPhase::Initial => Some((initial_delay(count), IntroPhase::Showing(0))),
        IntroPhase::Showing(i) => Some((intro_ms, IntroPhase::Exiting(i))),
        IntroPhase::Pause(i) => Some((
            pause_ms,
            IntroPhase::Showing(next_team(i, count).expect("nonempty wall")),
        )),
        IntroPhase::Exiting(_) => None,
    }
}

#[component]
pub fn TeamWall(
    teams: Vec<ResolvedTeam>,
    #[prop(into, default = Signal::stored(4700))] intro_ms: Signal<u64>,
    #[prop(into, default = Signal::stored(2600))] pause_ms: Signal<u64>,
) -> impl IntoView {
    let count = teams.len();
    let phase = RwSignal::new(IntroPhase::Initial);
    let featured = Memo::new(move |_| match phase.get() {
        IntroPhase::Showing(i) => Some(i),
        _ => None,
    });
    let timer = StoredValue::new(None::<TimeoutHandle>);
    Effect::new(move |_| {
        let current = phase.get();
        // Track only the setting relevant to this phase; a pause edit should not
        // restart an intro already on screen, nor should a language/theme change.
        let intro = if matches!(current, IntroPhase::Showing(_)) {
            intro_ms.get()
        } else {
            0
        };
        let pause = if matches!(current, IntroPhase::Pause(_)) {
            pause_ms.get()
        } else {
            0
        };
        if let Some(h) = timer.get_value() {
            h.clear();
        }
        timer.set_value(None);
        if let Some((ms, next)) = phase_timer(current, count, intro, pause) {
            schedule(timer, ms, move || phase.set(next));
        }
    });
    on_cleanup(move || {
        if let Some(Some(h)) = timer.try_get_value() {
            h.clear();
        }
    });
    view! { <TeamWallView teams phase featured/> }
}

/// Fixed team wall, optionally showing one themed team introduction. No rotation or sound.
#[component]
pub fn TeamWallPreview(
    teams: Vec<ResolvedTeam>,
    #[prop(default = None)] team_index: Option<usize>,
) -> impl IntoView {
    let selected = team_index.filter(|i| *i < teams.len());
    let phase = RwSignal::new(selected.map_or(IntroPhase::Initial, IntroPhase::Showing));
    let featured = Memo::new(move |_| selected);
    view! { <TeamWallView teams phase featured/> }
}

#[component]
fn TeamWallView(
    teams: Vec<ResolvedTeam>,
    phase: RwSignal<IntroPhase>,
    featured: Memo<Option<usize>>,
) -> impl IntoView {
    let t = use_t();
    let count = teams.len();
    let players: i64 = teams.iter().map(|t| t.team.players.unwrap_or(0)).sum();
    let (rows, cols) = grid(count);
    let spring = Spring::from_duration(0.3, 0.9).to_css_linear();
    let flip_spring = Spring::from_bounce(0.25, 0.9).to_css_linear();
    view! {
        <div class="w-full h-full flex flex-col px-24 pb-2 team-wall" style=format!("--wall-spring:{spring};--wall-flip-spring:{flip_spring}")>
            <p class="relative text-center text-3xl font-bold uppercase tracking-[0.3em] text-white/40 mb-8">{move || t.fmt(if players != 0 { "wall_summary" } else { "wall_summary_teams" }, &[("teams", &count.to_string()), ("players", &players.to_string())])}</p>
            <div class="relative flex-1 min-h-0">
                <div class="wall-fullscreen">{teams.iter().take(4).enumerate().map(|(i, team)| view! {
                    <div class=format!("wall-aurora wall-aurora-{i} absolute left-0 top-0 w-[28%] h-[32%] rounded-full blur-[140px] opacity-25") style=format!("background:{}; animation-duration:{}s", TEAM_HEX[team.color % 8], 22 + i * 4)/>
                }).collect_view()}</div>
                <div class="absolute inset-0 flex flex-wrap justify-center content-center gap-6">
                    {teams.iter().cloned().enumerate().map(|(i, team)| {
                        let font = name_font(&team.team.name, rows);
                        let (avatar, initials, gap, icon) = match rows {
                            0..=2 => ("w-36 h-36 rounded-2xl border border-white/20 relative shadow-xl", "text-6xl", "gap-4", 32),
                            3 => ("w-20 h-20 rounded-2xl border border-white/20 relative shadow-xl", "text-3xl", "gap-2", 22),
                            4 => ("w-14 h-14 rounded-2xl border border-white/20 relative shadow-xl", "text-2xl", "gap-1.5", 18),
                            _ => ("w-12 h-12 rounded-2xl border border-white/20 relative shadow-xl", "text-xl", "gap-1", 16),
                        };
                        let players = team.team.players.unwrap_or(0);
                        view! {
                            <div class="wall-tile-enter" style=format!("width:calc((100% - {} * 1.5rem) / {cols});height:calc((100% - {} * 1.5rem) / {rows});container-type:size;--i:{i};--delay:{}ms", cols-1, rows-1, entrance_delay(i))>
                                <div class="w-full h-full wall-tile-focus" class:wall-featured=move || featured.get() == Some(i)>
                                    <div class="w-full h-full wall-float" style=format!("animation-duration:{}s;animation-delay:{}s", 6.0 + (i % 5) as f64 * 0.8, i as f64 * 0.37)>
                                        <div class=format!("relative w-full h-full rounded-3xl border border-white/10 overflow-hidden flex flex-col items-center justify-center {gap} p-4")>
                                            <div class=format!("absolute inset-0 bg-gradient-to-br {} opacity-[0.18]", team_color(team.color))/><div class="absolute inset-0 bg-black/40"/>
                                            <TeamAvatar team=team.clone() class=avatar text_class=initials/>
                                            <h3 class="relative font-black text-white text-center leading-tight line-clamp-2 break-words max-w-full px-2" style=format!("font-size:{font}")>{team.team.name}</h3>
                                            <div class="relative flex items-center gap-1 text-white/60">
                                                {(players > 0).then(|| if players <= 8 {
                                                    (0..players).map(|_| view! { <svg width=icon height=icon viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="7" r="4"/><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/></svg> }).collect_view().into_any()
                                                } else { view! { <WelcomeIcon name="users" size=icon/><span class="font-bold" style=format!("font-size:{icon}px")>{players}</span> }.into_any() })}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
                <div class="wall-fullscreen overflow-hidden z-10"><div class="wall-shimmer absolute inset-y-0 w-[10%] -skew-x-[20deg] bg-gradient-to-r from-transparent via-white/[0.07] to-transparent"/></div>
                {teams.into_iter().enumerate().map(|(i, team)| {
                    let team = StoredValue::new(team);
                    view! {
                        <Presence when=Signal::derive(move || featured.get() == Some(i)) enter="wall-intro-enter" exit="wall-intro-exit" class="absolute inset-0 pointer-events-none"
                            on_exit=Callback::new(move |_| {
                                if phase.get_untracked() == IntroPhase::Exiting(i) {
                                    phase.set(IntroPhase::Pause(i + 1));
                                }
                            })>
                            <WallIntro team=team.get_value()/>
                        </Presence>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn Players(team: ResolvedTeam) -> impl IntoView {
    let t = use_t();
    let players = team.team.players.unwrap_or(0);
    (players > 0).then(|| view! { <span class="wall-players px-6 py-3 bg-white/10 rounded-full flex items-center gap-3 text-4xl font-bold uppercase tracking-wider border border-white/10"><WelcomeIcon name="users" size=44/>{players}" "{move || t.get("players")}</span> })
}

#[component]
fn WallIntro(team: ResolvedTeam) -> impl IntoView {
    let theme = use_theme();
    let intro = move || {
        theme.with(|t| {
            t.scenes
                .welcome
                .as_ref()
                .map(|w| w.intro)
                .unwrap_or(WelcomeIntro::Flip)
        })
    };
    let team = StoredValue::new(team);
    move || {
        let team = team.get_value();
        let name = team.team.name.clone();
        let quote = team.team.quote.clone().unwrap_or_default();
        let color = TEAM_HEX[team.color % 8];
        let has_meta = !quote.is_empty() || team.team.players.unwrap_or(0) > 0;
        let style = match intro() {
            WelcomeIntro::Broadcast => "broadcast",
            WelcomeIntro::Spotlight => "spotlight",
            WelcomeIntro::Polaroid => "polaroid",
            _ => "flip",
        };
        view! {
            <div class=format!("wall-intro wall-{style}") style=format!("--team-color:{color};--tilt:{}deg", tilt(&name))>
                <div class="wall-fullscreen wall-dim z-20"/>
                {match intro() {
                    WelcomeIntro::Broadcast => view! {
                        <div class="wall-broadcast-card absolute left-[-6%] right-[4%] top-[30%] z-30" style="container-type:inline-size">
                            <div class="wall-broadcast-stripe absolute inset-x-0 -top-[1.2cqw] h-[0.8cqw] origin-left bg-accent-500"/>
                            <div class="wall-broadcast-band relative flex items-center -skew-x-[8deg] bg-panel/95 border-l-[2cqw] shadow-[0_2cqw_6cqw_rgba(0,0,0,0.6)]" style=format!("border-color:{color}")>
                                <div class="w-[24cqw] shrink-0"/><div class="skew-x-[8deg] flex-1 min-w-0 py-[2.2cqw] pr-[8cqw]"><h2 class="wall-broadcast-name font-title font-black text-white uppercase leading-[1.02] break-words" style=format!("font-size:{}", intro_font(&name, "7cqw", 80.0))>{name.clone()}</h2></div>
                            </div>
                            <div class="wall-broadcast-meta relative ml-[24cqw] mr-[16cqw] mt-[1cqw] -skew-x-[8deg] px-[2cqw] py-[1cqw] flex items-center gap-[2cqw]" style=format!("background:{color};display:{}", if has_meta { "flex" } else { "none" })><div class="skew-x-[8deg] flex items-center gap-[2cqw] min-w-0 text-white text-[2.2cqw] font-bold drop-shadow"><Players team=team.clone()/>{(!quote.is_empty()).then(|| view! { <span class="italic truncate opacity-90">"\""{quote}"\""</span> })}</div></div>
                            <div class="wall-broadcast-avatar absolute left-[7cqw] top-1/2 -translate-y-1/2" style=if has_meta { "margin-top:-2.6cqw" } else { "margin-top:0" }><TeamAvatar team=team.clone() class="w-[15cqw] h-[15cqw] rounded-[2cqw] border-[0.5cqw] border-white shadow-2xl" text_class="text-[6cqw]"/></div>
                        </div>
                    }.into_any(),
                    WelcomeIntro::Spotlight => view! {
                        <div class="absolute inset-0 z-30 flex flex-col items-center justify-end pb-[3%]" style="container-type:size">
                            <div class="wall-beam absolute -top-[30%] left-1/2 -translate-x-1/2 w-[70cqw] h-[125%] origin-top"/><div class="wall-floor absolute bottom-[2%] w-[60cqw] h-[16cqh] rounded-[50%]"/>
                            <div class="wall-spotlight-content relative flex flex-col items-center gap-[2.5cqh] max-w-[80cqw]">
                                <TeamAvatar team=team.clone() class="w-[36cqh] h-[36cqh] rounded-full border-[0.8cqh] border-white/80 shadow-[0_0_8cqh_rgba(255,250,220,0.45)]" text_class="text-[14cqh]"/>
                                <h2 class="wall-spotlight-name font-title font-black text-white text-center uppercase leading-none break-words [text-shadow:0_0_4cqh_rgba(255,250,220,0.6)]" style=format!("font-size:{}", intro_font(&name, "15cqh", 120.0))>{name.clone()}</h2>
                                {(!quote.is_empty()).then(|| view! { <p class="text-[4.5cqh] text-white/70 italic text-center leading-snug line-clamp-2">"\""{quote}"\""</p> })}<Players team=team.clone()/>
                            </div>
                        </div>
                    }.into_any(),
                    WelcomeIntro::Polaroid => view! {
                        <div class="absolute inset-0 z-30 flex items-center justify-center gap-[4%]" style="container-type:size">
                            <div class="wall-photo relative bg-[#f7f3ea] p-[2.2cqh] pb-0 shadow-[0_3cqh_8cqh_rgba(0,0,0,0.7)] w-[min(84cqh,40cqw)]">
                                <div class="absolute -top-[2.6cqh] left-1/2 -translate-x-1/2 rotate-[-3deg] w-[38%] h-[5cqh] bg-white/45 border border-white/40 backdrop-blur-sm"/>
                                <div class="wall-develop"><TeamAvatar team=team.clone() class="w-full aspect-square rounded-none" text_class="text-[min(22cqh,14cqw)]"/></div>
                                <div class="h-[16cqh] flex items-center justify-center px-[1cqh]" style="container-type:inline-size"><h2 class="font-title font-black text-[#1d1d1f] text-center leading-[1.05] line-clamp-2 break-words" style=format!("font-size:min(8cqh, {:.1}cqi)", 140.0 / name.chars().count().clamp(6,20) as f64)>{name.clone()}</h2></div>
                            </div>
                            <div class="wall-photo-meta max-w-[34cqw] flex flex-col gap-[3cqh]" style=if has_meta { "display:flex" } else { "display:none" }>{(!quote.is_empty()).then(|| view! { <p class="text-[7cqh] text-white/90 italic leading-snug line-clamp-4">"\""{quote}"\""</p> })}<Players team=team.clone()/></div>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <div class="absolute inset-0 flex items-center justify-center z-30" style="perspective:2000px">
                            <div class=format!("wall-flip-card w-[58%] h-[86%] rounded-[3rem] bg-gradient-to-br {} p-3 shadow-[0_0_120px_rgba(0,0,0,0.7)]", team_color(team.color))>
                                <div class="w-full h-full rounded-[2.5rem] bg-black/80 backdrop-blur-md flex items-center gap-12 px-14 relative overflow-hidden" style="container-type:size">
                                    <TeamAvatar team=team.clone() class="w-[min(34cqw,80cqh)] h-[min(34cqw,80cqh)] rounded-[2rem] border border-white/20 shadow-2xl" text_class="text-[min(12cqw,28cqh)]"/>
                                    <div class="flex-1 min-w-0 flex flex-col gap-6"><h2 class="font-black text-white leading-[1.05] break-words" style=format!("font-size:{}", intro_font(&name, "6rem",110.0))>{name.clone()}</h2>{(!quote.is_empty()).then(|| view! { <p class="text-4xl text-white/60 italic leading-snug line-clamp-3">"\""{quote}"\""</p> })}<Players team=team.clone()/></div>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                }}
            </div>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn intro_waits_for_exit_before_starting_configured_pause() {
        assert_eq!(phase_timer(IntroPhase::Initial, 0, 4800, 2600), None);
        assert_eq!(
            phase_timer(IntroPhase::Initial, 3, 4800, 2600),
            Some((1710, IntroPhase::Showing(0)))
        );
        assert_eq!(
            phase_timer(IntroPhase::Showing(2), 3, 4800, 2600),
            Some((4800, IntroPhase::Exiting(2)))
        );
        assert_eq!(phase_timer(IntroPhase::Exiting(2), 3, 4800, 2600), None);
        assert_eq!(
            phase_timer(IntroPhase::Pause(3), 3, 4800, 2600),
            Some((2600, IntroPhase::Showing(0)))
        );
        assert_eq!(
            phase_timer(IntroPhase::Pause(1), 3, 4800, 0),
            Some((0, IntroPhase::Showing(1)))
        );
        // Changing a duration updates the current phase without losing the team index.
        assert_eq!(
            phase_timer(IntroPhase::Showing(2), 3, 6000, 900),
            Some((6000, IntroPhase::Exiting(2)))
        );
        assert_eq!(
            phase_timer(IntroPhase::Pause(2), 3, 6000, 900),
            Some((900, IntroPhase::Showing(2)))
        );
    }

    #[test]
    fn balanced_grids() {
        for (n, expected) in [
            (0, (1, 1)),
            (1, (1, 1)),
            (6, (2, 3)),
            (15, (3, 5)),
            (25, (4, 7)),
        ] {
            assert_eq!(grid(n), expected);
        }
    }
    #[test]
    fn cycle_and_delays() {
        assert_eq!(next_team(5, 3), Some(2));
        assert_eq!(next_team(1, 0), None);
        assert_eq!(initial_delay(6), 1920);
        assert_eq!(entrance_delay(2), 290);
    }
    #[test]
    fn fitting_and_tilt() {
        assert_eq!(name_font("Quiz", 2), "min(3rem, 37.5cqi, 28.0cqh)");
        assert_eq!(
            name_font("Long team name", 3),
            "min(2.25rem, 16.7cqi, 14.0cqh)"
        );
        assert_eq!(name_font("ěščř", 5), "min(1.5rem, 37.5cqi, 28.0cqh)");
        assert!((-3..=3).contains(&tilt("Quiz")));
        assert_eq!(tilt("😀"), (0xd83d_u32 % 7) as i32 - 3);
    }
}
