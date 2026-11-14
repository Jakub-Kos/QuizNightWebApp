//! Static final layouts. No reveal protocol or live-show event listeners.
use leptos::prelude::*;
use quiz_core::theme::format::LeaderboardLayout;
#[derive(Clone)]
pub(super) struct Row {
    pub name: String,
    pub total: f64,
    pub rank: usize,
}
#[component]
fn Avatar(name: String, #[prop(optional)] large: bool) -> impl IntoView {
    let quiz = crate::quiz_context::use_quiz();
    let team = quiz.teams.iter().find(|team| team.team.name == name);
    let image = team.and_then(|team| team.image.clone());
    let initials = team.map(|team| team.initials.clone()).unwrap_or_else(|| {
        name.split_whitespace()
            .take(2)
            .filter_map(|part| part.chars().next())
            .collect()
    });
    view! {
        <div class=if large { "w-[15cqh] h-[15cqh] rounded-full border-4 border-white/30 bg-primary-800 text-[4cqh] font-black overflow-hidden flex items-center justify-center shrink-0" } else { "h-[5cqh] w-[5cqh] rounded-full border border-white/20 bg-primary-800 text-[1.5cqh] font-black overflow-hidden flex items-center justify-center shrink-0" }>
            {image.map(|image| view! { <img src=image alt="" class="w-full h-full object-cover"/> }.into_any()).unwrap_or_else(|| initials.into_any())}
        </div>
    }
}
#[component]
pub(super) fn Board(rows: Vec<Row>, layout: LeaderboardLayout) -> impl IntoView {
    match layout {
        LeaderboardLayout::Table | LeaderboardLayout::Race => {
            let race = layout == LeaderboardLayout::Race;
            let top = rows.iter().map(|row| row.total).fold(1., f64::max);
            let low = rows.iter().map(|row| row.total).fold(top, f64::min);
            let base = (low - (top - low) * 0.25).max(0.);
            view! {
                <div class="w-full grid gap-3 content-start">
                    {rows.into_iter().map(|row| {
                        let width = ((row.total - base) / (top - base).max(1.) * 100.).max(3.);
                        view! {
                            <div class="relative theme-card flex items-center gap-[1.2cqw] rounded-2xl px-6 py-3 bg-panel border border-white/10">
                                <span class="w-[3cqw] text-[2.4cqh] font-black text-accent-400">{row.rank}</span>
                                <Avatar name=row.name.clone()/>
                                <span class="w-[26%] font-black uppercase text-[2.2cqh] truncate">{row.name}</span>
                                {race.then(|| view! {
                                    <div class="flex-1 h-[3cqh] rounded-full bg-white/5 overflow-hidden">
                                        <div class="h-full rounded-full bg-gradient-to-r from-primary-700 to-primary-400" style=format!("width:{width}%")/>
                                    </div>
                                })}
                                <span class="ml-auto text-[2.6cqh] font-mono font-black text-primary-300">{row.total}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            }.into_any()
        }
        _ => {
            let top = rows.iter().take(3).cloned().collect::<Vec<_>>();
            let rest = rows.into_iter().skip(3).collect::<Vec<_>>();
            view! {
                <div class="h-full w-full flex flex-col gap-[2.5cqh]">
                    <div class="flex-1 min-h-0 flex items-end justify-center gap-[2.5cqw]">
                        {top.into_iter().enumerate().map(|(place, row)| {
                            let first = place == 0;
                            let order = if first { 1 } else if place == 1 { 0 } else { 2 };
                            let medal = match place { 0 => "bg-gradient-to-b from-yellow-300 to-yellow-500", 1 => "bg-gradient-to-b from-slate-200 to-slate-400", _ => "bg-gradient-to-b from-orange-300 to-orange-600" };
                            let height = if first { 25 } else if place == 1 { 18 } else { 12 };
                            view! {
                                <div class="relative w-[24%] min-w-0 flex flex-col items-center justify-end text-center" style=format!("order:{order};height:{}%", if first { 100 } else { 86 })>
                                    {(layout == LeaderboardLayout::Spotlight).then(|| view! {
                                        <div class="absolute bottom-0 left-1/2 -translate-x-1/2 pointer-events-none h-[70cqh] w-[28cqw]" style="clip-path:polygon(42% 0,58% 0,100% 100%,0 100%);background:linear-gradient(to bottom,rgba(255,244,214,.28),rgba(255,244,214,.02))"/>
                                    })}
                                    {(layout == LeaderboardLayout::Medals).then(|| view! {
                                        <div class="absolute inset-0 rounded-[2rem] border-2 border-accent-300/60 bg-gradient-to-b from-accent-300/20 to-accent-700/10"/>
                                        <div class=format!("relative mt-[4.5cqh] mb-auto rounded-full w-[10cqh] h-[10cqh] flex items-center justify-center text-black/70 font-black text-[5cqh] {medal}")>{row.rank}</div>
                                    })}
                                    <div class="relative flex flex-col items-center w-full gap-[1.2cqh] pb-[2cqh]">
                                        <Avatar name=row.name.clone() large=true/>
                                        <div class="w-full truncate font-title font-black uppercase text-[3cqh]">{row.name}</div>
                                        <div class="font-mono font-black text-[3.4cqh] text-primary-300">{row.total}</div>
                                        {(layout == LeaderboardLayout::Spotlight).then(|| view! { <span class="rounded-full px-4 py-1 bg-accent-400 text-black font-black text-[1.8cqh]">{format!("#{}", row.rank)}</span> })}
                                    </div>
                                    {(layout == LeaderboardLayout::Podium).then(|| view! {
                                        <div class=format!("w-full rounded-t-2xl pt-[1.2cqh] text-[6cqh] font-title font-black text-black/60 {medal}") style=format!("height:{height}cqh")>{row.rank}</div>
                                    })}
                                </div>
                            }
                        }).collect_view()}
                    </div>
                    <div class="grid grid-cols-3 gap-x-6 gap-y-[0.8cqh]">
                        {rest.into_iter().map(|row| view! {
                            <div class="flex items-center gap-3 h-[4cqh] px-3 rounded-lg bg-black/40 border border-white/10">
                                <span class="font-black text-[1.8cqh] text-white/60">{row.rank}</span>
                                <span class="flex-1 truncate font-bold uppercase text-[1.7cqh]">{row.name}</span>
                                <span class="font-mono font-black text-[2cqh] text-primary-300">{row.total}</span>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
            }.into_any()
        }
    }
}
