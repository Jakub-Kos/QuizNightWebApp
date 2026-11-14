//! Dashboard's tiles and timeline, sharing completion state with the columns.
use crate::{i18n::use_t, spring::Spring};
use leptos::prelude::*;
use quiz_core::{quiz::parse::Round, show::PlayMode};

type Toggle = Callback<(web_sys::MouseEvent, String)>;
type Select = Callback<(String, PlayMode)>;

fn tile_grid(count: usize) -> (usize, usize, usize) {
    if count == 0 {
        return (0, 0, 0);
    }
    let columns = match count {
        1..=4 => count,
        5..=6 => 3,
        _ => 4,
    };
    let rows = count.div_ceil(columns);
    (columns, rows, (rows * 34).min(70))
}
fn next_index(rounds: &[Round], completed: &[String]) -> Option<usize> {
    rounds.iter().position(|r| !completed.contains(&r.id))
}
fn done_share(count: usize, next: Option<usize>) -> f64 {
    if count <= 1 {
        0.0
    } else {
        next.unwrap_or(count - 1) as f64 / (count - 1) as f64
    }
}
fn title_of(round: &Round, index: usize, done: bool, label: &str) -> String {
    if done && !round.title.is_empty() {
        round.title.clone()
    } else {
        format!("{label} {}", index + 1)
    }
}

#[component]
fn LayoutIcon(name: &'static str, #[prop(default = 20)] size: u32) -> impl IntoView {
    view! { <svg width=size height=size viewBox="0 0 24 24" fill=if name == "play" { "currentColor" } else { "none" } stroke="currentColor" stroke-width=if name == "check" { "3" } else { "2" } stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        {match name {
            "eye" => view! { <g><path d="M2 12s3-8 10-8 10 8 10 8-3 8-10 8S2 12 2 12Z"/><circle cx="12" cy="12" r="3"/></g> }.into_any(),
            "play" => view! { <path d="m6 3 15 9-15 9Z"/> }.into_any(),
            "check" => view! { <path d="m5 12 4 4L19 6"/> }.into_any(),
            "check_circle" => view! { <g><circle cx="12" cy="12" r="10"/><path d="m8 12 3 3 5-6"/></g> }.into_any(),
            _ => view! { <circle cx="12" cy="12" r="10"/> }.into_any(),
        }}
    </svg> }
}

#[component]
fn Buttons(
    id: String,
    index: usize,
    on_select_round: Select,
    #[prop(into)] compact: Signal<bool>,
) -> impl IntoView {
    let t = use_t();
    let id = StoredValue::new(id);
    let size = move || {
        if compact.get() {
            "py-[1.2cqh] text-[1.3cqh]"
        } else {
            "py-[1.8cqh] text-[1.5cqh]"
        }
    };
    view! {
        <div class="grid grid-cols-2 gap-[1cqh] w-full">
            <button data-tour=(index == 0).then_some("questions") on:click=move |_| on_select_round.run((id.get_value(), PlayMode::QuestionsOnly))
                class=move || format!("{} rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 font-bold tracking-widest uppercase flex items-center justify-center gap-2 transition-colors", size())>
                <span class="text-gray-400">{move || view! { <LayoutIcon name="eye" size=if compact.get() {16} else {20}/> }}</span>{move || t.get("btn_questions")}
            </button>
            <button data-tour=(index == 0).then_some("answers") on:click=move |_| on_select_round.run((id.get_value(), PlayMode::WithAnswers))
                class=move || format!("{} rounded-xl bg-gradient-to-br from-accent-400 to-accent-600 hover:from-accent-300 hover:to-accent-500 text-black font-black tracking-widest uppercase flex items-center justify-center gap-2 shadow-[0_0_20px_rgb(var(--glow-accent)/0.3)] transition-colors", size())>
                {move || view! { <LayoutIcon name="play" size=if compact.get() {16} else {20}/> }}{move || t.get("btn_answers")}
            </button>
        </div>
    }
}

#[component]
fn FinishToggle(id: String, #[prop(into)] done: Signal<bool>, on_toggle: Toggle) -> impl IntoView {
    let t = use_t();
    let id = StoredValue::new(id);
    let label = move || {
        t.get(if done.get() {
            "mark_unfinished"
        } else {
            "mark_finished"
        })
    };
    view! {
        <button on:click=move |e| on_toggle.run((e, id.get_value())) title=label aria-label=label
            class=move || format!("p-[0.8cqh] rounded-full border transition-colors {}", if done.get() { "border-green-500/40 text-green-400 bg-green-500/10" } else { "border-white/15 text-white/40 hover:text-white hover:border-white/40" })>
            {move || view! { <LayoutIcon name=if done.get() {"check_circle"} else {"circle"}/> }}
        </button>
    }
}

#[component]
pub fn TilesView(
    rounds: Vec<Round>,
    #[prop(into)] completed: Signal<Vec<String>>,
    on_toggle: Toggle,
    on_select_round: Select,
) -> impl IntoView {
    let t = use_t();
    let (columns, rows, height) = tile_grid(rounds.len());
    let rounds = StoredValue::new(rounds);
    let next = Memo::new(move |_| rounds.with_value(|r| next_index(r, &completed.get())));
    view! {
        <div class="grid w-full max-w-[1700px] gap-[2cqh]" style=format!("grid-template-columns:repeat({}, minmax(0, 1fr));grid-template-rows:repeat({}, minmax(0, 1fr));height:{height}cqh", columns.max(1), rows.max(1))>
            {rounds.get_value().into_iter().enumerate().map(|(index, round)| {
                let round = StoredValue::new(round);
                let done = Signal::derive(move || completed.with(|c| round.with_value(|r| c.contains(&r.id))));
                let is_next = move || next.get() == Some(index);
                view! {
                    <div data-tour=(index == 0).then_some("round") style=format!("--i:{index}")
                        class=move || format!("layout-tile relative min-h-0 rounded-[2rem] border overflow-hidden flex flex-col p-[2.4cqh] gap-[1.2cqh] {}", if done.get() { "border-green-500/30 bg-[#051005]" } else if is_next() { "border-accent-400/70 bg-panel shadow-[0_0_40px_rgb(var(--glow-accent)/0.25)]" } else { "border-white/10 bg-panel" })>
                        <span class=move || format!("font-title absolute -right-[1cqh] -top-[3cqh] text-[18cqh] font-black leading-none pointer-events-none {}", if done.get() { "text-green-500/10" } else { "text-white/[0.06]" })>{index + 1}</span>
                        <div class="flex items-center gap-2 relative">
                            <span class="px-3 py-1 rounded-full bg-primary-500/10 border border-primary-500/20 text-primary-400 text-[1.2cqh] font-black tracking-widest uppercase">{round.with_value(|r| r.questions.len())}" "{move || t.get("questions_count")}</span>
                            <Show when=is_next><span class="px-3 py-1 rounded-full bg-accent-500 text-black text-[1.2cqh] font-black tracking-widest uppercase">{move || t.get("db_next")}</span></Show>
                            <span class="ml-auto"><FinishToggle id=round.with_value(|r| r.id.clone()) done=done on_toggle=on_toggle/></span>
                        </div>
                        <h2 class="font-title relative flex-1 min-h-0 flex items-center text-[3.6cqh] font-black uppercase leading-tight text-white line-clamp-3">{move || round.with_value(|r| title_of(r, index, done.get(), &t.get("round")))}</h2>
                        <div class="relative"><Buttons id=round.with_value(|r| r.id.clone()) index=index on_select_round=on_select_round compact=Signal::stored(rows > 1)/></div>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

#[component]
pub fn TimelineView(
    rounds: Vec<Round>,
    #[prop(into)] completed: Signal<Vec<String>>,
    on_toggle: Toggle,
    on_select_round: Select,
) -> impl IntoView {
    let t = use_t();
    let count = rounds.len();
    let rounds = StoredValue::new(rounds);
    let next = Memo::new(move |_| rounds.with_value(|r| next_index(r, &completed.get())));
    let spring = Spring::new(100.0, 10.0, 1.0).to_css_linear();
    view! {
        <div class="relative w-full max-w-[1800px] h-[64cqh] flex items-start" style=format!("--layout-spring:{spring}")>
            <div class="absolute left-[5%] right-[5%] top-[14cqh] h-[0.8cqh] rounded-full bg-white/10"/>
            <div class="layout-timeline-progress absolute left-[5%] top-[14cqh] h-[0.8cqh] rounded-full bg-gradient-to-r from-green-500 to-accent-400" style=move || format!("width:{}%", done_share(count,next.get()) * 90.0)/>
            <div class="relative w-full grid" style=format!("grid-template-columns:repeat({}, minmax(0, 1fr))",count.max(1))>
                {rounds.get_value().into_iter().enumerate().map(|(index, round)| {
                    let round = StoredValue::new(round);
                    let done = Signal::derive(move || completed.with(|c| round.with_value(|r| c.contains(&r.id))));
                    let is_next = move || next.get() == Some(index);
                    view! {
                        <div data-tour=(index == 0).then_some("round") class="flex flex-col items-center text-center px-[0.8cqw] gap-[1.4cqh]">
                            <div class="h-[28cqh] flex items-center justify-center">
                                <div class=move || format!("rounded-full flex items-center justify-center font-title font-black border-4 {}", if is_next() { "layout-next-node w-[16cqh] h-[16cqh] text-[7cqh] bg-accent-500 text-black border-accent-200 shadow-[0_0_60px_rgb(var(--glow-accent)/0.6)]" } else if done.get() { "w-[9cqh] h-[9cqh] text-[4cqh] bg-green-600 text-white border-green-300/50" } else { "w-[9cqh] h-[9cqh] text-[4cqh] bg-panel text-white/70 border-white/15" })>
                                    {move || if done.get() { view! { <LayoutIcon name="check" size=40/> }.into_any() } else { (index+1).into_any() }}
                                </div>
                            </div>
                            <h2 class=move || format!("font-title font-black uppercase leading-tight line-clamp-2 {}", if is_next() { "text-[3cqh] text-white" } else { "text-[2cqh] text-white/70" })>{move || round.with_value(|r| title_of(r,index,done.get(),&t.get("round")))}</h2>
                            <span class="text-[1.3cqh] font-bold tracking-widest uppercase text-primary-400">{round.with_value(|r| r.questions.len())}" "{move || t.get("questions_count")}</span>
                            <div class=move || if is_next() { "w-[125%] max-w-[20cqw]" } else { "w-full" }><Buttons id=round.with_value(|r| r.id.clone()) index=index on_select_round=on_select_round compact=Signal::derive(move || !is_next())/></div>
                            <FinishToggle id=round.with_value(|r| r.id.clone()) done=done on_toggle=on_toggle/>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rounds() -> Vec<Round> {
        (0..3)
            .map(|i| Round {
                id: i.to_string(),
                number: i.to_string(),
                title: format!("Secret {i}"),
                rows: vec![],
                questions: vec![],
            })
            .collect()
    }
    #[test]
    fn tile_positions() {
        for (n, expected) in [
            (0, (0, 0, 0)),
            (1, (1, 1, 34)),
            (4, (4, 1, 34)),
            (5, (3, 2, 68)),
            (6, (3, 2, 68)),
            (7, (4, 2, 68)),
            (9, (4, 3, 70)),
        ] {
            assert_eq!(tile_grid(n), expected);
        }
    }
    #[test]
    fn timeline_progress() {
        let rounds = rounds();
        assert_eq!(next_index(&rounds, &[]), Some(0));
        assert_eq!(
            next_index(&rounds, &["0".into(), "2".into(), "hidden".into()]),
            Some(1)
        );
        assert_eq!(
            next_index(&rounds, &["0".into(), "1".into(), "2".into()]),
            None
        );
        assert_eq!(done_share(0, None), 0.0);
        assert_eq!(done_share(1, None), 0.0);
        assert_eq!(done_share(3, Some(1)), 0.5);
        assert_eq!(done_share(3, None), 1.0);
    }
    #[test]
    fn titles_stay_hidden_until_done() {
        let mut round = rounds().remove(0);
        assert_eq!(title_of(&round, 0, false, "Round"), "Round 1");
        assert_eq!(title_of(&round, 0, true, "Round"), "Secret 0");
        round.title.clear();
        assert_eq!(title_of(&round, 2, true, "Kolo"), "Kolo 3");
    }
}
