use crate::{
    reorder::{self, Mode},
    stage::StageLayout,
};
use leptos::{ev, prelude::*};
const NAMES: [&str; 10] = [
    "Quiz Wizards",
    "The Brainstorm",
    "Midnight Owls",
    "Trivia Titans",
    "Lucky Guesses",
    "Quantum Cats",
    "Beer Pressure",
    "Fact Hunters",
    "The Underdogs",
    "Last Orders",
];
const COLORS: [&str; 10] = [
    "#eab308", "#38bdf8", "#a78bfa", "#fb7185", "#34d399", "#f97316", "#818cf8", "#2dd4bf",
    "#e879f9", "#facc15",
];
fn score(id: usize, step: usize) -> i32 {
    100 - id as i32 * 5
        + if id >= 10 - step.min(5) * 2 {
            [8, 24, 12, 33, 18, 40, 22, 45, 32, 54][id]
        } else {
            0
        }
}
#[component]
pub fn LeaderboardDemo() -> impl IntoView {
    let step = RwSignal::new(0usize);
    let native = RwSignal::new(true);
    let race = RwSignal::new(false);
    let layout = expect_context::<Signal<StageLayout>>();
    let advance = move || {
        if step.get_untracked() < 5 {
            reorder::reorder(
                if native.get_untracked() {
                    Mode::ViewTransition
                } else {
                    Mode::Flip
                },
                layout.get_untracked().scale,
                move || step.update(|s| *s += 1),
            );
        }
    };
    let listener = window_event_listener(ev::keydown, move |e| {
        if e.key() == "ArrowRight" {
            e.prevent_default();
            advance();
        }
    });
    on_cleanup(move || listener.remove());
    view! {<section class="leaderboard-demo text-white"><header><p class="text-primary-400 tracking-widest uppercase">"Quiz night · Round 4"</p><h1 class="text-accent-400 font-black uppercase">"Leaderboard"</h1><p>"Reveal the round from the lowest scoring teams upwards"</p></header>
    <nav><button on:click=move |_|advance() disabled=move ||step.get()==5>"Reveal next group →"</button><button on:click=move |_|native.update(|v|*v=!*v)>{move ||if native.get(){"Mode: View Transitions"}else{"Mode: FLIP"}}</button><button on:click=move |_|race.update(|v|*v=!*v)>"Table / race"</button><button on:click=move |_|reorder::reorder(Mode::Flip,layout.get_untracked().scale,move ||step.set(0))>"Reset"</button><span>{move ||format!("{} / 5 groups",step.get())}</span></nav>
    <div class=move ||if race.get(){"leaderboard-rows race"}else{"leaderboard-rows"}>
    <For each=move ||{let mut ids=(0..10).collect::<Vec<_>>();ids.sort_by_key(|&id|(-score(id,step.get()),id));ids} key=|id|*id children=move |id|{
    view!{<article data-reorder="" style=format!("view-transition-name:team-{id};--team-color:{}",COLORS[id]) class="leaderboard-row bg-black/40 border border-white/10 shadow-lg">
    <b class="rank">{move ||{let s=step.get();1+(0..10).filter(|&j|(score(j,s),std::cmp::Reverse(j))>(score(id,s),std::cmp::Reverse(id))).count()}}</b><span class="badge">{id+1}</span><strong>{NAMES[id]}</strong><div class="race-track"><div style=move ||format!("width:{}%",score(id,step.get()) as f64/160.0*100.0)></div></div><span class="round text-green-400">{move ||if id>=10-step.get()*2{format!("+{}",score(id,step.get())-(100-id as i32*5))}else{"?".into()}}</span><Score value=Signal::derive(move ||score(id,step.get()))/></article>}
    }/>
    </div></section>}
}
#[component]
fn Score(value: Signal<i32>) -> impl IntoView {
    let displayed = RwSignal::new(value.get_untracked());
    let generation = RwSignal::new(0u32);
    Effect::new(move |_| {
        let target = value.get();
        let start = displayed.get_untracked();
        generation.update(|g| *g += 1);
        let token = generation.get_untracked();
        if reorder::reduced_motion() {
            displayed.set(target);
            return;
        }
        leptos::task::spawn_local(async move {
            for i in 1..=40 {
                gloo_timers::future::TimeoutFuture::new(20).await;
                if generation.try_get_untracked() != Some(token) {
                    return;
                }
                displayed.set(
                    start
                        + ((target - start) as f64 * (1.0 - (1.0 - i as f64 / 40.0).powi(3)))
                            .round() as i32,
                );
            }
        });
    });
    view! {<b class="score text-primary-400">{move ||displayed.get()}</b>}
}
