use crate::{i18n::use_t, quiz_context::use_quiz, show::use_show};
use leptos::prelude::*;
use super::dashboard_layouts::{TilesView, TimelineView};
use quiz_core::theme::DashboardLayout;

use quiz_core::{
    quiz::parse::is_round_hidden,
    show::{Action, PlayMode},
};
fn title_size(text: &str, expanded: bool) -> &'static str {
    let n = text.encode_utf16().count();
    let i = if n < 12 {
        0
    } else if n < 18 {
        1
    } else if n < 24 {
        2
    } else if n < 30 {
        3
    } else {
        4
    };
    if expanded {
        [
            "text-6xl",
            "text-[54px]",
            "text-5xl",
            "text-[42px]",
            "text-4xl",
        ][i]
    } else {
        [
            "text-5xl",
            "text-[42px]",
            "text-4xl",
            "text-[32px]",
            "text-3xl",
        ][i]
    }
}
#[component]
pub fn Dashboard() -> impl IntoView {
    let s = use_show();
    let q = use_quiz();
    let t = use_t();
    let completed = super::completed_rounds::use_completed(&q.quiz.id);
    let rounds = q
        .rounds
        .iter()
        .filter(|r| !is_round_hidden(r))
        .cloned()
        .collect::<Vec<_>>();
    let total = rounds.len();
    let ids = StoredValue::new(rounds.iter().map(|r| r.id.clone()).collect::<Vec<_>>());
    let count =
        move || ids.with_value(|ids| ids.iter().filter(|id| completed.get().contains(id)).count());
    let theme = crate::theme::use_theme();
    // Dev only: `?dash=tiles|timeline` shows a layout no built-in theme uses.
    let forced = match crate::quiz_context::query_param("dash").as_deref() {
        Some("tiles") => Some(DashboardLayout::Tiles),
        Some("timeline") => Some(DashboardLayout::Timeline),
        _ => None,
    };
    let layout = move || {
        forced.unwrap_or_else(|| {
            theme.with(|t| t.scenes.dashboard.as_ref().map_or(DashboardLayout::Columns, |d| d.layout))
        })
    };
    let layout_rounds = StoredValue::new(rounds.clone());
    let on_toggle = Callback::new(move |(event, id): (web_sys::MouseEvent, String)| {
        event.stop_propagation();
        completed.update(|v| super::completed_rounds::toggle(v, &id));
    });
    let on_select_round = Callback::new(move |(round_id, mode)| s.dispatch.run(Action::StartRound { round_id, mode }));
    let cards=rounds.into_iter().enumerate().map(|(index, round)| {
let id=StoredValue::new(round.id);
let name=StoredValue::new(round.title);
let questions=round.questions.len();
let done=move || completed.get().contains(&id.get_value());
let title=move || if done() && !name.get_value().is_empty() {name.get_value()}
else {format!("{} {}",t.get("round"),index+1)};
view!{<div class=move || format!("dash-card relative rounded-[2.5rem] overflow-hidden border cursor-pointer group flex flex-col \
    items-center justify-end shadow-2xl transition-colors duration-500 {}", if done() {"border-green-500/30 bg-[#051005]"} else {"border-white/10 bg-panel"})
style=format!("--i:{index};--dash-spring:{}", crate::spring::Spring::from_bounce(0.2, 0.7).to_css_linear()) data-tour=(index == 0).then_some("round")>
<div class="dash-shade absolute inset-0 transition-all duration-700 ease-in-out pointer-events-none"/>
<div class=move || format!("dash-number font-title absolute text-[150px] font-black pointer-events-none leading-none origin-top \
    {}", if done() {"text-green-500"} else {"text-white"})>{index+1}</div>
<div class="dash-collapsed absolute inset-x-0 bottom-12 flex justify-center pointer-events-none">
<div class="w-10 h-10 flex items-end justify-center relative">
<h2 style="transform-origin:left bottom;transform:rotate(-90deg) translateX(0) translateY(50%)"
class=move || format!("font-title absolute left-1/2 bottom-0 w-[55cqh] text-left font-bold tracking-[0.2em] uppercase \
    transition-colors leading-tight {} {}", title_size(&title(), false), if done() {"text-green-500/60 group-hover:text-green-400"} else {"text-white/40 group-hover:text-white/80"})>{title}</h2>
</div>
</div>
<div class="dash-expanded relative z-10 flex flex-col items-center text-center w-full max-w-[450px] mb-8 px-6">
<div class="flex gap-3 mb-6">
<span class="px-5 py-2 bg-primary-500/10 text-primary-400 text-[11px] font-black tracking-widest uppercase \
    rounded-full border border-primary-500/20">{questions}" "{t.get("questions_count")}</span>
<Show when=done>
<span class="px-5 py-2 bg-green-500/20 text-green-400 text-[11px] font-black tracking-widest uppercase \
    rounded-full border border-green-500/30 flex items-center gap-2">
<CircleCheck size=12/>{t.get("finished")}</span>
</Show>
</div>
<h2 class=move || format!("font-title {} font-black uppercase tracking-tight mb-8 drop-shadow-xl leading-tight text-white", title_size(&title(), true))>{title}</h2>
<div class="grid grid-cols-2 gap-4 w-full mb-6" >
<button class="py-5 bg-white/5 hover:bg-white/10 text-white rounded-2xl font-bold tracking-widest text-xs flex \
    flex-col items-center justify-center gap-2 transition-all border border-white/10 \
    hover:border-white/20" data-tour=(index == 0).then_some("questions") on:click=move |_| s.dispatch.run(Action::StartRound {round_id:id.get_value(), mode:PlayMode::QuestionsOnly})>
<Eye size=24 attr:class="text-gray-400"/>
<span>{t.get("btn_questions")}</span>
</button>
<button class="group relative flex items-center gap-3 px-8 py-5 bg-gradient-to-br from-primary-900/50 \
    to-primary-800/30 hover:from-primary-600 hover:to-primary-500 text-white rounded-full \
    transition-all duration-300 shadow-[0_0_20px_rgb(var(--glow-primary)/0.1)] \
    hover:shadow-[0_0_40px_rgb(var(--glow-primary)/0.4)] border border-primary-500/20" data-tour=(index == 0).then_some("answers") on:click=move |_| s.dispatch.run(Action::StartRound {round_id:id.get_value(), mode:PlayMode::WithAnswers})>
<Play size=24 fill="currentColor"/>
<span>{t.get("btn_answers")}</span>
</button>
</div>
<button class=move || format!("px-6 py-3 rounded-full border text-[10px] sm:text-xs font-bold tracking-widest flex items-center \
    gap-2 transition-all {}", if done() {"bg-transparent text-white/30 border-white/10 hover:text-white/80 hover:border-white/30"} else {"bg-white/5 text-white/50 border-white/10 hover:bg-white/10 hover:text-white"})
on:click=move |e| {e.stop_propagation(); completed.update(|v| super::completed_rounds::toggle(v, &id.get_value()));}>
{move || if done() {view!{<Circle size=16/>}.into_any()} else {view!{<CircleCheck size=16/>}.into_any()}}
{move || t.get(if done() {"mark_unfinished"} else {"mark_finished"})}</button>
</div>
</div>}}).collect_view();
    view! {<div class="h-full w-full bg-ground relative overflow-hidden flex flex-col font-show selection:bg-accent-500/30" >
    <crate::scenes::backdrop::SceneBackdrop quiet=true/>
    <div class="absolute top-0 right-0 w-[800px] h-[800px] bg-primary-900/10 blur-[150px] rounded-full \
        pointer-events-none" >
    </div>
    <div class="absolute bottom-0 left-0 w-[600px] h-[600px] bg-accent-600/5 blur-[120px] rounded-full \
        pointer-events-none" >
    </div>
    <div class="h-[15cqh] flex items-center justify-between px-16 relative z-20 border-b border-white/5 bg-black/40 \
        backdrop-blur-md shrink-0" >
    <div class="flex flex-col gap-3 w-[400px]" >
    <div class="flex justify-between items-end" >
    <h1 class="text-2xl font-black text-white/50 uppercase tracking-[0.3em]" >{t.get("progress")}</h1>
    <span class="text-primary-400 font-mono font-bold tracking-widest text-xl" >{count} <span class="text-white/30">"/ "{total}</span>
    <button class:hidden=move || count() == 0 title=t.get("calib_reset") class="ml-3 align-middle text-white/30 hover:text-white transition-colors" on:click=move |_| completed.set(Vec::new())>
    <RotateCcw size=16 attr:class="inline"/>
    </button>
    </span>
    </div>
    <div class="h-3 w-full bg-white/5 rounded-full overflow-hidden border border-white/10" >
    <div class="h-full bg-gradient-to-r from-primary-600 to-primary-400 \
        shadow-[0_0_15px_rgb(var(--glow-primary)/0.6)] rounded-full" style=move || format!("width:{}%", count() as f64 / total.max(1) as f64 * 100.0)>
    </div>
    </div>
    </div>
    <div class="flex gap-6" >
    <button class="group relative flex items-center gap-3 px-8 py-5 bg-gradient-to-br from-primary-900/50 \
        to-primary-800/30 hover:from-primary-600 hover:to-primary-500 text-white rounded-full \
        transition-all duration-300 shadow-[0_0_20px_rgb(var(--glow-primary)/0.1)] \
        hover:shadow-[0_0_40px_rgb(var(--glow-primary)/0.4)] border border-primary-500/20" data-tour="pause" on:click=move |_| s.dispatch.run(Action::OpenPause)>
    <Coffee size=28 attr:class="relative z-10"/>
    <span class="font-black text-xl uppercase tracking-[0.15em] relative z-10" >{t.get("half_time")}</span>
    </button>
    <button class="group relative flex items-center gap-4 px-10 py-5 bg-gradient-to-br from-accent-600 to-accent-400 \
        hover:from-accent-400 hover:to-accent-300 text-black rounded-full transition-all duration-300 \
        hover:scale-[1.03] shadow-[0_0_30px_rgb(var(--glow-accent)/0.2)] \
        hover:shadow-[0_0_50px_rgb(var(--glow-accent)/0.5)] overflow-hidden" data-tour="leaderboard" on:click=move |_| s.dispatch.run(Action::OpenLeaderboard)>
    <Trophy size=32 attr:class="relative z-10"/>
    <span class="font-black text-2xl uppercase tracking-[0.15em] relative z-10" >{t.get("leaderboard_btn")}</span>
    <div class="absolute inset-0 bg-white/40 translate-x-[-150%] skew-x-12 \
        group-hover:animate-[shimmer_1.5s_infinite]" >
    </div>
    </button>
    </div>
    </div>
    <div class="flex-1 flex flex-col items-center justify-center p-10 relative z-10 w-full" >
    <Show when=move || total == 0>
    <div class="flex flex-col items-center justify-center h-full text-white/30 space-y-4" >
    <p class="font-mono tracking-widest uppercase" >{t.get("ed_no_questions")}</p>
    </div>
    </Show>
    {move || match (total > 0, layout()) {
        (true, DashboardLayout::Tiles) => view! { <TilesView rounds=layout_rounds.get_value() completed=completed on_toggle=on_toggle on_select_round=on_select_round/> }.into_any(),
        (true, DashboardLayout::Timeline) => view! { <TimelineView rounds=layout_rounds.get_value() completed=completed on_toggle=on_toggle on_select_round=on_select_round/> }.into_any(),
        _ => ().into_any(),
    }}
    <div class="flex w-full max-w-[1600px] h-[70cqh] gap-4" class:hidden=move || total == 0 || layout() != DashboardLayout::Columns>{cards}</div>
    </div>
    </div>}
}

mod circle_icon {
    use leptos::{prelude::*, svg::Svg};
    #[component]
    pub fn Circle(
        #[prop(default = 24.into(), into)] size: Signal<usize>,
        #[prop(default = "currentColor".into(), into)] color: Signal<String>,
        #[prop(default = "none".into(), into)] fill: Signal<String>,
        #[prop(default = 2.into(), into)] stroke_width: Signal<usize>,
        #[prop(default = false.into(), into)] absolute_stroke_width: Signal<bool>,
        #[prop(optional)] node_ref: NodeRef<Svg>,
    ) -> impl IntoView {
        let stroke_width = Signal::derive(move || {
            if absolute_stroke_width.get() {
                stroke_width.get() * 24 / size.get()
            } else {
                stroke_width.get()
            }
        });
        view! {
            <svg
                node_ref=node_ref
                class:lucide=true
                xmlns="http://www.w3.org/2000/svg"
                width=size
                height=size
                viewBox="0 0 24 24"
                fill=fill
                stroke=color
                stroke-width=stroke_width
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <circle cx="12" cy="12" r="10" />
            </svg>
        }
    }
}
use circle_icon::Circle;

mod circle_check_icon {
    use leptos::{prelude::*, svg::Svg};
    #[component]
    pub fn CircleCheck(
        #[prop(default = 24.into(), into)] size: Signal<usize>,
        #[prop(default = "currentColor".into(), into)] color: Signal<String>,
        #[prop(default = "none".into(), into)] fill: Signal<String>,
        #[prop(default = 2.into(), into)] stroke_width: Signal<usize>,
        #[prop(default = false.into(), into)] absolute_stroke_width: Signal<bool>,
        #[prop(optional)] node_ref: NodeRef<Svg>,
    ) -> impl IntoView {
        let stroke_width = Signal::derive(move || {
            if absolute_stroke_width.get() {
                stroke_width.get() * 24 / size.get()
            } else {
                stroke_width.get()
            }
        });
        view! {
            <svg
                node_ref=node_ref
                class:lucide=true
                xmlns="http://www.w3.org/2000/svg"
                width=size
                height=size
                viewBox="0 0 24 24"
                fill=fill
                stroke=color
                stroke-width=stroke_width
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <circle cx="12" cy="12" r="10" />
                <path d="m16 9-5.5 5.5L8 12" />
            </svg>
        }
    }
}
use circle_check_icon::CircleCheck;

mod coffee_icon {
    use leptos::{prelude::*, svg::Svg};
    #[component]
    pub fn Coffee(
        #[prop(default = 24.into(), into)] size: Signal<usize>,
        #[prop(default = "currentColor".into(), into)] color: Signal<String>,
        #[prop(default = "none".into(), into)] fill: Signal<String>,
        #[prop(default = 2.into(), into)] stroke_width: Signal<usize>,
        #[prop(default = false.into(), into)] absolute_stroke_width: Signal<bool>,
        #[prop(optional)] node_ref: NodeRef<Svg>,
    ) -> impl IntoView {
        let stroke_width = Signal::derive(move || {
            if absolute_stroke_width.get() {
                stroke_width.get() * 24 / size.get()
            } else {
                stroke_width.get()
            }
        });
        view! {
            <svg
                node_ref=node_ref
                class:lucide=true
                xmlns="http://www.w3.org/2000/svg"
                width=size
                height=size
                viewBox="0 0 24 24"
                fill=fill
                stroke=color
                stroke-width=stroke_width
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="M10 2v2" />
                <path d="M14 2v2" />
                <path d="M16 8a1 1 0 0 1 1 1v8a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4V9a1 1 0 0 1 1-1h14a4 4 0 1 1 0 8h-1" />
                <path d="M6 2v2" />
            </svg>
        }
    }
}
use coffee_icon::Coffee;

mod eye_icon {
    use leptos::{prelude::*, svg::Svg};
    #[component]
    pub fn Eye(
        #[prop(default = 24.into(), into)] size: Signal<usize>,
        #[prop(default = "currentColor".into(), into)] color: Signal<String>,
        #[prop(default = "none".into(), into)] fill: Signal<String>,
        #[prop(default = 2.into(), into)] stroke_width: Signal<usize>,
        #[prop(default = false.into(), into)] absolute_stroke_width: Signal<bool>,
        #[prop(optional)] node_ref: NodeRef<Svg>,
    ) -> impl IntoView {
        let stroke_width = Signal::derive(move || {
            if absolute_stroke_width.get() {
                stroke_width.get() * 24 / size.get()
            } else {
                stroke_width.get()
            }
        });
        view! {
            <svg
                node_ref=node_ref
                class:lucide=true
                xmlns="http://www.w3.org/2000/svg"
                width=size
                height=size
                viewBox="0 0 24 24"
                fill=fill
                stroke=color
                stroke-width=stroke_width
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 \
        0" />
                <circle cx="12" cy="12" r="3" />
            </svg>
        }
    }
}
use eye_icon::Eye;

mod play_icon {
    use leptos::{prelude::*, svg::Svg};
    #[component]
    pub fn Play(
        #[prop(default = 24.into(), into)] size: Signal<usize>,
        #[prop(default = "currentColor".into(), into)] color: Signal<String>,
        #[prop(default = "none".into(), into)] fill: Signal<String>,
        #[prop(default = 2.into(), into)] stroke_width: Signal<usize>,
        #[prop(default = false.into(), into)] absolute_stroke_width: Signal<bool>,
        #[prop(optional)] node_ref: NodeRef<Svg>,
    ) -> impl IntoView {
        let stroke_width = Signal::derive(move || {
            if absolute_stroke_width.get() {
                stroke_width.get() * 24 / size.get()
            } else {
                stroke_width.get()
            }
        });
        view! {
            <svg
                node_ref=node_ref
                class:lucide=true
                xmlns="http://www.w3.org/2000/svg"
                width=size
                height=size
                viewBox="0 0 24 24"
                fill=fill
                stroke=color
                stroke-width=stroke_width
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z" />
            </svg>
        }
    }
}
use play_icon::Play;

mod rotate_ccw_icon {
    use leptos::{prelude::*, svg::Svg};
    #[component]
    pub fn RotateCcw(
        #[prop(default = 24.into(), into)] size: Signal<usize>,
        #[prop(default = "currentColor".into(), into)] color: Signal<String>,
        #[prop(default = "none".into(), into)] fill: Signal<String>,
        #[prop(default = 2.into(), into)] stroke_width: Signal<usize>,
        #[prop(default = false.into(), into)] absolute_stroke_width: Signal<bool>,
        #[prop(optional)] node_ref: NodeRef<Svg>,
    ) -> impl IntoView {
        let stroke_width = Signal::derive(move || {
            if absolute_stroke_width.get() {
                stroke_width.get() * 24 / size.get()
            } else {
                stroke_width.get()
            }
        });
        view! {
            <svg
                node_ref=node_ref
                class:lucide=true
                xmlns="http://www.w3.org/2000/svg"
                width=size
                height=size
                viewBox="0 0 24 24"
                fill=fill
                stroke=color
                stroke-width=stroke_width
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
                <path d="M3 3v5h5" />
            </svg>
        }
    }
}
use rotate_ccw_icon::RotateCcw;

mod trophy_icon {
    use leptos::{prelude::*, svg::Svg};
    #[component]
    pub fn Trophy(
        #[prop(default = 24.into(), into)] size: Signal<usize>,
        #[prop(default = "currentColor".into(), into)] color: Signal<String>,
        #[prop(default = "none".into(), into)] fill: Signal<String>,
        #[prop(default = 2.into(), into)] stroke_width: Signal<usize>,
        #[prop(default = false.into(), into)] absolute_stroke_width: Signal<bool>,
        #[prop(optional)] node_ref: NodeRef<Svg>,
    ) -> impl IntoView {
        let stroke_width = Signal::derive(move || {
            if absolute_stroke_width.get() {
                stroke_width.get() * 24 / size.get()
            } else {
                stroke_width.get()
            }
        });
        view! {
            <svg
                node_ref=node_ref
                class:lucide=true
                xmlns="http://www.w3.org/2000/svg"
                width=size
                height=size
                viewBox="0 0 24 24"
                fill=fill
                stroke=color
                stroke-width=stroke_width
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="M10 14.66V17a1 1 0 0 1-1 1 2 2 0 0 0-2 2v2" />
                <path d="M14 14.66V17a1 1 0 0 0 1 1 2 2 0 0 1 2 2v2" />
                <path d="M17.916 10H19.5A2.5 2.5 0 0 0 22 7.5V5a1 1 0 0 0-1-1h-3" />
                <path d="M4 22h16" />
                <path d="M6 9a6 6 0 0 0 12 0V3a1 1 0 0 0-1-1H7a1 1 0 0 0-1 1z" />
                <path d="M6.084 10H4.5A2.5 2.5 0 0 1 2 7.5V5a1 1 0 0 1 1-1h3" />
            </svg>
        }
    }
}
use trophy_icon::Trophy;
