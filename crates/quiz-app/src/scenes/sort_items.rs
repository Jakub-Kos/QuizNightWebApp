//! Sort cards remount on reveal and stagger in React's mixed or correct order.
use super::icons::GameIcon;
use crate::i18n::use_t;
use leptos::prelude::*;
use quiz_core::quiz::sort::{PlacedItem, SortLayout};

/// The items in the order they stand on screen: mixed up, or in the correct order once revealed.
fn ordered(layout: &SortLayout, revealed: bool) -> &[PlacedItem] {
    if revealed {
        &layout.correct
    } else {
        &layout.shown
    }
}

/// PictureGrid: the number of columns, and a card's height in cqh (leaves room for the question and
/// the answer line).
fn picture_grid(n: usize) -> (usize, u32) {
    let cols = match n {
        0..=4 => n,
        5 | 6 => 3,
        _ => n.div_ceil(2),
    };
    let one_row = cols > 0 && n.div_ceil(cols) == 1;
    (cols, if one_row { 44 } else { 23 })
}

/// TextList: more than five items take two columns, filled top to bottom, so the order reads down
/// the left column first. Returns (two columns, rows).
fn text_rows(n: usize) -> (bool, usize) {
    let two = n > 5;
    (two, if two { n.div_ceil(2) } else { n })
}

#[component]
pub fn SortItems(
    layout: SortLayout,
    #[prop(into)] revealed: Signal<bool>,
    resolve_media: Callback<String, Option<String>>,
) -> impl IntoView {
    let t = use_t();
    let n = layout.shown.len();
    let source = move |item: &PlacedItem| {
        item.item
            .media
            .as_deref()
            .and_then(|m| resolve_media.run(m.to_string()))
    };
    let pictures = layout.shown.iter().any(|item| source(item).is_some());
    let (cols, height) = picture_grid(n);
    let (two_columns, rows) = text_rows(n);
    let answer: Vec<String> = layout
        .correct
        .iter()
        .map(|it| it.item.letter.clone().unwrap_or_default())
        .collect();
    let layout = StoredValue::new(layout);
    let shown = revealed;

    let item_view = move |(at_mount, item): (bool, PlacedItem)| {
        let letter = item.item.letter.clone().unwrap_or_default();
        let place = item.place;
        // Fade in one after another when mounted: from below on the question, from the left when
        // the answer is already up. Played once (a moved node would replay it).
        let index = layout.with_value(|l| {
            ordered(l, at_mount)
                .iter()
                .position(|it| it.item.letter == item.item.letter)
                .unwrap_or(0)
        });
        let appear = if at_mount {
            "sort-appear sort-appear-revealed"
        } else {
            "sort-appear"
        };
        let played = RwSignal::new(false);
        let on_end = move |event: web_sys::AnimationEvent| {
            if event.target() == event.current_target() && event.animation_name() == "game-from" {
                played.set(true);
            }
        };
        let badge = {
            let letter = letter.clone();
            move |extra: &'static str| {
                view! {
                    <div class=format!("flex items-center gap-2 shrink-0{extra}")>
                        <Show when=move || shown.get()>
                            <span class="w-14 h-14 rounded-xl bg-green-500 text-black flex items-center justify-center font-black text-3xl">
                                {format!("{place}.")}
                            </span>
                        </Show>
                        <span class=move || format!(
                            "w-14 h-14 rounded-full flex items-center justify-center font-black text-2xl border {}",
                            if shown.get() { "bg-black/60 border-green-400/60 text-green-300" }
                            else { "bg-black/40 border-white/15 text-white" })>
                            {letter.clone()}
                        </span>
                    </div>
                }
            }
        };
        if pictures {
            let src = source(&item);
            let text = item.item.text.clone();
            view! {
                <div on:animationend=on_end
                    style=format!("--i:{index};width:calc((100% - {} * 1.25rem) / {cols});height:{height}cqh", cols.saturating_sub(1))
                    class=move || format!(
                        "relative rounded-3xl overflow-hidden border flex flex-col bg-black/50 {}{}",
                        if shown.get() { "border-green-500/70 shadow-[0_0_30px_rgba(34,197,94,0.25)] " }
                        else { "border-white/10 " },
                        if played.get() { "" } else { appear })>
                    <div class="flex-1 min-h-0 flex items-center justify-center bg-black/40">
                        {match src {
                            Some(src) => view! { <img src=src alt="" class="w-full h-full object-contain"/> }.into_any(),
                            None => view! { <span class="text-8xl font-black text-white/10">{letter.clone()}</span> }.into_any(),
                        }}
                    </div>
                    {(!text.is_empty()).then(|| view! {
                        <div class=move || format!(
                            "shrink-0 px-4 py-3 text-center font-bold text-3xl leading-tight line-clamp-2 {}",
                            if shown.get() { "bg-green-900/60 text-white" } else { "bg-white/5 text-white" })>
                            {text.clone()}
                        </div>
                    })}
                    {badge(" absolute top-3 left-3 drop-shadow-xl")}
                </div>
            }
            .into_any()
        } else {
            view! {
                <div on:animationend=on_end style=format!("--i:{index}")
                    class=move || format!(
                        "flex items-center gap-6 rounded-2xl border {} {}{}",
                        if two_columns { "px-6 py-3" } else { "px-8 py-4" },
                        if shown.get() { "bg-gradient-to-r from-green-900/50 to-green-800/20 border-green-500/60 " }
                        else { "bg-white/5 border-white/10 " },
                        if played.get() { "" } else { appear })>
                    {badge("")}
                    <span class=format!("font-bold text-white text-left leading-tight {}",
                        if two_columns { "text-4xl" } else { "text-5xl" })>
                        {item.item.text.clone()}
                    </span>
                </div>
            }
            .into_any()
        }
    };

    view! {
        <div class="w-full flex flex-col items-center gap-[2.5cqh]">
            {if pictures {
                view! {
                    <div class="w-full flex flex-wrap justify-center gap-5 max-w-[2100px]">
                        <For each=move || layout.with_value(|l| ordered(l, shown.get()).iter().cloned().map(|item| (shown.get(), item)).collect::<Vec<_>>())
                            key=|(reveal, item)| (*reveal, item.item.letter.clone()) children=item_view/>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class=format!("grid gap-x-8 {}", if two_columns {
                            "grid-cols-2 w-full max-w-[1800px] gap-y-3"
                        } else { "w-full max-w-5xl gap-y-4" })
                        style=format!("grid-template-rows:repeat({rows}, auto);grid-auto-flow:column")>
                        <For each=move || layout.with_value(|l| ordered(l, shown.get()).iter().cloned().map(|item| (shown.get(), item)).collect::<Vec<_>>())
                            key=|(reveal, item)| (*reveal, item.item.letter.clone()) children=item_view/>
                    </div>
                }.into_any()
            }}

            <Show when=move || shown.get() fallback=move || view! {
                <div class="flex items-center gap-5 px-12 py-4 rounded-full bg-white/5 border border-white/10 text-white/50 animate-pulse">
                    <GameIcon name="arrow_down_up" size=32/>
                    <span class="font-black text-3xl tracking-[0.15em] uppercase">{t.get("sort_hint")}</span>
                </div>
            }>
                <div style=format!("--delay:{}s", 0.25 * n as f64 + 0.2)
                    class="sort-answer-in flex items-center gap-5 px-10 py-4 rounded-full bg-gradient-to-br from-green-500 to-green-600 text-black shadow-[0_0_60px_rgba(34,197,94,0.5)]">
                    <span class="text-2xl font-bold uppercase tracking-widest">{t.get("sort_correct_order")}</span>
                    <span class="flex items-center gap-3 text-5xl font-black">
                        {answer.iter().enumerate().map(|(i, letter)| view! {
                            <span class="flex items-center gap-3">
                                {(i > 0).then(|| view! {
                                    <GameIcon name="arrow_right" size=28 stroke_width=3
                                        class=Signal::stored("opacity-60".to_string())/>
                                })}
                                {letter.clone()}
                            </span>
                        }).collect_view()}
                    </span>
                </div>
            </Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quiz_core::quiz::{
        parse::{Question, parse_questions_csv},
        sort::sort_layout,
    };

    /// The Sort questions of the dev sheet (assets/dev-questions.csv), parsed as the app parses them.
    fn dev_questions() -> Vec<Question> {
        let rounds = parse_questions_csv(include_str!("../../assets/dev-questions.csv"));
        rounds.into_iter().flat_map(|r| r.questions).collect()
    }
    fn letters(items: &[PlacedItem]) -> Vec<&str> {
        items
            .iter()
            .map(|it| it.item.letter.as_deref().unwrap())
            .collect()
    }
    fn texts(items: &[PlacedItem]) -> Vec<&str> {
        items.iter().map(|it| it.item.text.as_str()).collect()
    }

    #[test]
    fn reveal_and_reverse_keep_letters_and_places() {
        let questions = dev_questions();
        // The picture question: the sheet's letters C A E B D for the items in the correct order
        let layout = sort_layout(&questions[1]);
        assert!(layout.from_sheet);
        // Before the reveal: A, B, C… with each item's correct place
        let before = ordered(&layout, false);
        assert_eq!(letters(before), ["A", "B", "C", "D", "E"]);
        assert_eq!(
            texts(before),
            ["Beijing", "Kyiv", "Paris", "Lockdown", "Moscow"]
        );
        assert_eq!(
            before.iter().map(|it| it.place).collect::<Vec<_>>(),
            [2, 4, 1, 5, 3]
        );
        // The reveal moves the same items (same letters and places) into the correct order
        let after = ordered(&layout, true);
        assert_eq!(letters(after), ["C", "A", "E", "B", "D"]);
        assert_eq!(
            texts(after),
            ["Paris", "Beijing", "Moscow", "Kyiv", "Lockdown"]
        );
        assert_eq!(
            after.iter().map(|it| it.place).collect::<Vec<_>>(),
            [1, 2, 3, 4, 5]
        );
        let mut same = after.to_vec();
        same.sort_by_key(|it| it.item.letter.clone());
        assert_eq!(same, before);
        // Going back shows the mixed-up order again
        assert_eq!(ordered(&layout, false), layout.shown.as_slice());
        assert!(after.iter().all(|it| it.item.media.is_some()));
    }

    #[test]
    fn sheet_without_letters_is_mixed_but_never_in_order() {
        let questions = dev_questions();
        let layout = sort_layout(&questions[2]);
        assert!(!layout.from_sheet);
        let before = ordered(&layout, false);
        assert_eq!(letters(before), ["A", "B", "C", "D", "E", "F", "G"]);
        assert_ne!(texts(before), texts(&layout.correct));
        assert_eq!(
            texts(ordered(&layout, true)),
            [
                "Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus"
            ]
        );
        // The text question with letters from the sheet
        let layout = sort_layout(&questions[0]);
        assert_eq!(layout.answer, "C A D B");
    }

    #[test]
    fn grid_and_list_shapes() {
        assert_eq!(picture_grid(3), (3, 44));
        assert_eq!(picture_grid(4), (4, 44));
        assert_eq!(picture_grid(5), (3, 23));
        assert_eq!(picture_grid(6), (3, 23));
        assert_eq!(picture_grid(7), (4, 23));
        assert_eq!(picture_grid(10), (5, 23));
        assert_eq!(text_rows(5), (false, 5));
        assert_eq!(text_rows(7), (true, 4));
    }
}
