//! The details panel that unfolds beside a team card on the welcome screen (the `details-…` block
//! of WelcomeScreen.jsx) and its sub-components RecentFormBadges and SlotMachineRank.
//! RankGraph and AchievementSlots in WelcomeScreen.jsx are not used by the React app and are not ported.
use crate::i18n::use_t;
use leptos::prelude::*;
use quiz_core::quiz::model::{Rank, ResolvedTeam};

/// Lucide icons of the welcome screen (paths from lucide-leptos), drawn like lucide-react draws them.
#[component]
pub fn WelcomeIcon(
    name: &'static str,
    size: u32,
    #[prop(into, optional)] class: String,
    #[prop(default = 2)] stroke_width: u32,
    #[prop(default = "none")] fill: &'static str,
) -> impl IntoView {
    let paths = match name {
        "clock" => view! { <g>
            <circle cx="12" cy="12" r="10" />
            <path d="M12 6v6l4 2" />
        </g> }
        .into_any(),
        "timer" => view! { <g>
            <line x1="10" x2="14" y1="2" y2="2" />
            <line x1="12" x2="15" y1="14" y2="11" />
            <circle cx="12" cy="14" r="8" />
        </g> }
        .into_any(),
        "users" => view! { <g>
            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
            <path d="M16 3.128a4 4 0 0 1 0 7.744" />
            <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
            <circle cx="9" cy="7" r="4" />
        </g> }
        .into_any(),
        "crown" => view! { <g>
            <path d="M11.562 3.266a.5.5 0 0 1 .876 0L15.39 8.87a1 1 0 0 0 1.516.294L21.183 5.5a.5.5 0 0 1 .798.519l-2.834 10.246a1 1 0 0 1-.956.734H5.81a1 1 0 0 1-.957-.734L2.02 6.02a.5.5 0 0 1 .798-.519l4.276 3.664a1 1 0 0 0 1.516-.294z" />
            <path d="M5 21h14" />
        </g> }
        .into_any(),
        "target" => view! { <g>
            <circle cx="12" cy="12" r="10" />
            <circle cx="12" cy="12" r="6" />
            <circle cx="12" cy="12" r="2" />
        </g> }
        .into_any(),
        "medal" => view! { <g>
            <path d="M7.21 15 2.66 7.14a2 2 0 0 1 .13-2.2L4.4 2.8A2 2 0 0 1 6 2h12a2 2 0 0 1 1.6.8l1.6 2.14a2 2 0 0 1 .14 2.2L16.79 15" />
            <path d="M11 12 5.12 2.2" />
            <path d="m13 12 5.88-9.8" />
            <path d="M8 7h8" />
            <circle cx="12" cy="17" r="5" />
            <path d="M12 18v-2h-.5" />
        </g> }
        .into_any(),
        "trending_up" => view! { <g>
            <path d="M16 7h6v6" />
            <path d="m22 7-8.5 8.5-5-5L2 17" />
        </g> }
        .into_any(),
        "play" => view! { <g>
            <path d="M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z" />
        </g> }
        .into_any(),
        "rotate_cw" => view! { <g>
            <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
            <path d="M21 3v5h-5" />
        </g> }
        .into_any(),
        _ => view! { <g/> }.into_any(),
    };
    view! {
        <svg width=size height=size class=class viewBox="0 0 24 24" fill=fill stroke="currentColor"
            stroke-width=stroke_width stroke-linecap="round" stroke-linejoin="round">{paths}</svg>
    }
}

/// How many results fit on the card (a TV cannot scroll).
pub const MAX_FORM_BADGES: usize = 5;

/// The latest results with their volume index (for the "Vol." label).
pub fn form_window(history: &[Rank]) -> Vec<(usize, &Rank)> {
    let first = history.len().saturating_sub(MAX_FORM_BADGES);
    history.iter().enumerate().skip(first).collect()
}

/// A volume the team did not play ("?" in the scores sheet).
pub fn is_empty_slot(rank: &Rank) -> bool {
    matches!(rank, Rank::Unknown(s) if s == "?")
}

/// The badge colours of a result (RecentFormBadges' badgeClass).
pub fn badge_class(rank: &Rank) -> &'static str {
    match rank {
        Rank::Number(1) => {
            "bg-gradient-to-br from-yellow-400 to-yellow-600 border-yellow-300 text-yellow-950 shadow-[0_0_20px_rgba(234,179,8,0.5)] scale-110 z-10"
        }
        Rank::Number(2) => {
            "bg-gradient-to-br from-slate-300 to-slate-400 border-slate-200 text-slate-900 shadow-[0_0_15px_rgba(203,213,225,0.4)] scale-105 z-10"
        }
        Rank::Number(3) => {
            "bg-gradient-to-br from-orange-400 to-orange-600 border-orange-300 text-orange-950 shadow-[0_0_15px_rgba(249,115,22,0.4)] scale-105 z-10"
        }
        _ => {
            "bg-primary-950/80 border-primary-800 text-primary-300 shadow-[0_2px_10px_rgb(var(--glow-primary-deep)/0.2)]"
        }
    }
}

fn rank_text(rank: &Rank) -> String {
    match rank {
        Rank::Number(n) => n.to_string(),
        Rank::Unknown(s) => s.clone(),
    }
}

/// SlotMachineRank's formatRank: the final number reads 1st/2nd/3rd (in English, as in React).
pub fn format_rank(rank: &Rank, is_final: bool) -> String {
    match (is_final, rank) {
        (true, Rank::Number(1)) => "1st".into(),
        (true, Rank::Number(2)) => "2nd".into(),
        (true, Rank::Number(3)) => "3rd".into(),
        _ => format!("#{}", rank_text(rank)),
    }
}

/// The numbers the slot machine spins through before it stops at the real rank.
const FAKE_SPINS: [i64; 12] = [12, 5, 16, 8, 2, 14, 7, 11, 4, 9, 15, 3];
const SLOT_ITEM_HEIGHT: usize = 40;

pub fn slot_strip(rank: &Rank) -> Vec<(String, &'static str)> {
    let mut strip: Vec<_> = FAKE_SPINS
        .iter()
        .map(|n| (format_rank(&Rank::Number(*n), false), "text-3xl"))
        .collect();
    let size = if *rank == Rank::Number(1) {
        "text-2xl"
    } else {
        "text-3xl"
    };
    strip.push((format_rank(rank, true), size));
    strip
}

/// Colours of the three podium tiles: (top bar, medal).
pub fn podium_colors(place: usize) -> (&'static str, &'static str) {
    match place {
        0 => ("bg-yellow-500", "text-yellow-400"),
        1 => ("bg-gray-400", "text-gray-400"),
        _ => ("bg-orange-700", "text-orange-600"),
    }
}

#[component]
fn SlotMachineRank(rank: Rank, delay: f64) -> impl IntoView {
    let strip = slot_strip(&rank);
    let end = (strip.len() - 1) * SLOT_ITEM_HEIGHT;
    view! {
        <div class="h-[40px] w-[60px] overflow-hidden flex justify-center items-start leading-none">
            <div class="flex flex-col items-center welcome-slot"
                style=format!("transform: translateY(-{end}px); --delay: {delay}s")>
                {strip.into_iter().map(|(text, size)| view! {
                    <span class=format!("h-[40px] flex items-center justify-center font-black {size}")>{text}</span>
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn RecentFormBadges(history: Vec<Rank>) -> impl IntoView {
    let t = use_t();
    if history.is_empty() {
        return view! {
            <div class="flex-1 flex items-center justify-center border border-white/5 bg-white/5 rounded-xl text-gray-500 italic text-sm">
                {move || t.get("no_history")}
            </div>
        }
        .into_any();
    }
    let badges: Vec<(usize, Rank)> = form_window(&history)
        .into_iter()
        .map(|(i, r)| (i, r.clone()))
        .collect();
    view! {
        <div class="w-full h-full flex items-center justify-center gap-4 pt-4 pb-2 px-2 overflow-hidden">
            {badges.into_iter().enumerate().map(|(k, (i, rank))| {
                if is_empty_slot(&rank) {
                    return view! {
                        <div class="flex-shrink-0 w-28 h-28 rounded-2xl border-2 border-dashed border-white/10 bg-black/20 flex flex-col items-center justify-center relative">
                            <span class="font-mono text-[10px] absolute top-1.5 uppercase tracking-widest text-white/30 font-bold">
                                "Vol."{i + 1}
                            </span>
                            <div class="mt-3 w-3 h-1 rounded-full bg-white/10"></div>
                        </div>
                    }.into_any();
                }
                let delay = 0.2 + k as f64 * 0.15;
                let gold = rank == Rank::Number(1);
                let content = if gold {
                    view! {
                        <div class="flex flex-col items-center">
                            // The crown pops in right as the spin finishes
                            <div class="welcome-crown" style=format!("--delay: {}s", 2.2 + k as f64 * 0.15)>
                                <WelcomeIcon name="crown" size=18 class="mb-0.5" stroke_width=3/>
                            </div>
                            <SlotMachineRank rank=rank.clone() delay=delay/>
                        </div>
                    }.into_any()
                } else {
                    view! { <SlotMachineRank rank=rank.clone() delay=delay/> }.into_any()
                };
                view! {
                    <div class=format!("flex-shrink-0 w-28 h-28 rounded-2xl border-2 flex flex-col items-center justify-center relative overflow-hidden welcome-badge {}", badge_class(&rank))>
                        {gold.then(|| view! {
                            <div class="absolute inset-0 bg-white/40 -translate-x-full animate-[shimmer_2s_infinite]" />
                        })}
                        <span class="font-mono text-[10px] absolute top-1.5 uppercase tracking-widest opacity-60 font-bold">
                            "Vol."{i + 1}
                        </span>
                        <div class="mt-3">{content}</div>
                    </div>
                }.into_any()
            }).collect_view()}
        </div>
    }
    .into_any()
}

/// The details panel: all-time best, accuracy, podium counts and the latest results.
#[component]
pub fn TeamDetails(team: ResolvedTeam) -> impl IntoView {
    let t = use_t();
    // React prints `undefined` for a missing best rank or accuracy; empty here.
    let best = team
        .team
        .best_rank
        .map(|r| r.to_string())
        .unwrap_or_default();
    let rate = team
        .team
        .correct_rate
        .map(|r| r.to_string())
        .unwrap_or_default();
    let podium = team.team.podium.clone().unwrap_or_default();
    let counts = [podium.first, podium.second, podium.third];
    let history = team.team.rank_history.clone().unwrap_or_default();
    view! {
        <div class="welcome-team-details absolute top-0 bottom-0 w-[850px] bg-panel border border-white/20 rounded-r-[3rem] rounded-l-[2rem] p-10 flex flex-col shadow-2xl"
            style="transform: translateX(620px); z-index: 10">
            <div class="flex justify-between items-start border-b border-white/10 pb-6 mb-6">
                <div>
                    <h3 class="text-gray-400 uppercase tracking-[0.2em] text-4xl mb-1">{move || t.get("all_time_best")}</h3>
                    <span class="text-white font-black text-6xl flex items-center gap-3">
                        <WelcomeIcon name="crown" size=65 class="text-yellow-500"/>" "{move || t.get("rank")}" #"{best}
                    </span>
                </div>
                <div class="text-right">
                    <h3 class="text-gray-400 uppercase tracking-[0.2em] text-4xl mb-1">{move || t.get("accuracy")}</h3>
                    <span class="text-white font-black text-6xl flex items-center justify-end gap-3">
                        {rate}"% "<WelcomeIcon name="target" size=65 class="text-green-500"/>
                    </span>
                </div>
            </div>

            <div class="grid grid-cols-3 gap-4 mb-6">
                {counts.into_iter().enumerate().map(|(i, count)| {
                    let (bar, medal) = podium_colors(i);
                    let label = ["place_1", "place_2", "place_3"][i];
                    view! {
                        <div class="p-5 rounded-2xl flex flex-col items-center bg-white/5 border border-white/5 relative overflow-hidden">
                            <div class=format!("absolute top-0 left-0 w-full h-1 {bar}")/>
                            <WelcomeIcon name="medal" size=64 class=format!("mb-2 {medal}")/>
                            <span class="text-6xl font-black text-white">{count}</span>
                            <span class="text-3xl uppercase tracking-widest text-gray-500 mt-1">{move || t.get(label)}</span>
                        </div>
                    }
                }).collect_view()}
            </div>

            <div class="mt-auto flex-1 flex flex-col">
                <h3 class="text-gray-400 uppercase tracking-widest text-2xl mb-3 flex items-center gap-2">
                    <WelcomeIcon name="trending_up" size=32/>" "{move || t.get("season_history")}
                </h3>
                <div class="flex-1">
                    <RecentFormBadges history=history/>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ranks(v: &[i64]) -> Vec<Rank> {
        v.iter().map(|n| Rank::Number(*n)).collect()
    }
    #[test]
    fn only_the_latest_five_results_show() {
        let history = ranks(&[1, 2, 3, 4, 5, 6, 7]);
        let shown: Vec<_> = form_window(&history).into_iter().map(|(i, _)| i).collect();
        assert_eq!(shown, vec![2, 3, 4, 5, 6]);
        assert_eq!(form_window(&ranks(&[4, 1])).len(), 2);
        assert!(form_window(&[]).is_empty());
    }
    #[test]
    fn slots_and_labels() {
        assert!(is_empty_slot(&Rank::Unknown("?".into())));
        assert!(!is_empty_slot(&Rank::Unknown("x".into())));
        assert_eq!(format_rank(&Rank::Number(1), true), "1st");
        assert_eq!(format_rank(&Rank::Number(2), true), "2nd");
        assert_eq!(format_rank(&Rank::Number(3), true), "3rd");
        assert_eq!(format_rank(&Rank::Number(4), true), "#4");
        assert_eq!(format_rank(&Rank::Number(1), false), "#1");
        let strip = slot_strip(&Rank::Number(1));
        assert_eq!(strip.len(), 13);
        assert_eq!(strip[0], ("#12".into(), "text-3xl"));
        assert_eq!(strip[12], ("1st".into(), "text-2xl"));
        assert_eq!(slot_strip(&Rank::Number(7))[12], ("#7".into(), "text-3xl"));
    }
    #[test]
    fn colours() {
        assert!(badge_class(&Rank::Number(1)).contains("from-yellow-400"));
        assert!(badge_class(&Rank::Number(2)).contains("from-slate-300"));
        assert!(badge_class(&Rank::Number(3)).contains("from-orange-400"));
        assert!(badge_class(&Rank::Number(9)).starts_with("bg-primary-950/80"));
        assert!(badge_class(&Rank::Unknown("1".into())).starts_with("bg-primary-950/80"));
        assert_eq!(podium_colors(0), ("bg-yellow-500", "text-yellow-400"));
        assert_eq!(podium_colors(2), ("bg-orange-700", "text-orange-600"));
    }
}
