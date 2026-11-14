//! RoundIntro.jsx's four theme-selected entrances.
use crate::{spring::Spring, theme::use_theme};
use leptos::prelude::*;
use quiz_core::theme::RoundLayout;

fn title_size(title: &str, max: f64) -> String {
    format!(
        "{:.1}cqh",
        (480.0 / title.chars().count().max(8) as f64)
            .max(5.0)
            .min(max)
    )
}
fn bulbs() -> Vec<(f64, f64)> {
    let mut spots = Vec::new();
    for i in 0..22 {
        let x = (i as f64 + 0.5) / 22.0 * 100.0;
        spots.extend([(x, 0.0), (x, 100.0)]);
    }
    for i in 0..8 {
        let y = (i as f64 + 0.5) / 8.0 * 100.0;
        spots.extend([(0.0, y), (100.0, y)]);
    }
    spots
}
#[component]
pub fn RoundIntro(label: String, title: String, on_click: Callback<()>) -> impl IntoView {
    let theme = use_theme();
    let label = StoredValue::new(label);
    let title = StoredValue::new(title);
    let style = [
        ("curtain", 0.3, 0.9),
        ("label", 0.5, 0.8),
        ("marquee", 0.35, 1.0),
    ]
    .into_iter()
    .map(|(name, bounce, duration)| {
        format!(
            "--round-{name}:{};",
            Spring::from_duration(bounce, duration).to_css_linear()
        )
    })
    .collect::<String>();
    let scale = Spring::new(550.0, 30.0, 1.0);
    let style = format!(
        "{style}--round-scale:{};--round-scale-time:{}s;",
        scale.to_css_linear(),
        scale.settle_time()
    );
    view! {
        <div on:click=move |_| on_click.run(()) style=style
            class="round-intro h-full w-full bg-ground flex flex-col items-center justify-center font-show cursor-pointer relative overflow-hidden">
            <crate::scenes::backdrop::SceneBackdrop/>
            {move || {
                let layout = theme.get().scenes.round.as_ref().map(|r| r.layout).unwrap_or(RoundLayout::Classic);
                let label = label.get_value();
                let title = title.get_value();
                match layout {
                    RoundLayout::Classic => view! {
                        <div class="round-classic text-center z-10 flex flex-col items-center max-w-6xl px-8">
                            <h2 class="text-primary-500 font-bold tracking-[0.5em] text-4xl mb-6 uppercase drop-shadow-lg">{label}</h2>
                            <div class="h-2 w-40 bg-primary-500 mx-auto mb-12 rounded-full shadow-[0_0_20px_rgb(var(--glow-primary)/0.8)]"/>
                            <h1 class="font-title text-[120px] font-black text-white uppercase tracking-tight drop-shadow-2xl leading-none text-center">{title.clone()}</h1>
                        </div>
                    }.into_any(),
                    RoundLayout::Curtain => view! {
                        <div class="round-light absolute inset-0 pointer-events-none"/>
                        <div class="round-curtain-title relative z-10 text-center flex flex-col items-center max-w-[70cqw] px-8">
                            <h2 class="text-accent-400 font-bold tracking-[0.5em] text-[3cqh] mb-[3cqh] uppercase">{label}</h2>
                            <h1 style=format!("font-size:{}", title_size(&title, 10.0)) class="font-title font-black text-white uppercase leading-none drop-shadow-2xl">{title.clone()}</h1>
                            <div class="mt-[6cqh] h-[1.2cqh] w-[60cqw] rounded-full bg-[radial-gradient(ellipse,rgb(var(--glow-accent)/0.5),transparent_70%)]"/>
                        </div>
                        <div class="round-cloth round-curtain-left absolute top-0 bottom-0 left-0 w-[51%] z-20 shadow-[0_0_80px_rgba(0,0,0,0.8)]"/>
                        <div class="round-cloth round-curtain-right absolute top-0 bottom-0 right-0 w-[51%] z-20 shadow-[0_0_80px_rgba(0,0,0,0.8)]"/>
                        <div class="absolute top-0 inset-x-0 z-30">
                            <div class="round-cloth round-valance h-[6cqh] border-b-[0.6cqh] border-accent-500/70"/>
                            <div class="round-scallop h-[3cqh]"/>
                        </div>
                    }.into_any(),
                    RoundLayout::Slam => {
                        let number: String = label.chars().filter(char::is_ascii_digit).collect();
                        view! {
                            <div class="round-shake absolute inset-0 flex items-center justify-center">
                                <span class="round-hollow absolute font-title font-black leading-none select-none text-transparent text-[80cqh]">{if number.is_empty() { "?".into() } else { number }}</span>
                                {(0..3).map(|i| view! { <div style=format!("--i:{i};--ring-scale:{}", 5.0 + i as f64 * 1.5) class="round-ring absolute rounded-full border-accent-400"/> }).collect_view()}
                                {(0..16).map(|i| view! { <div style=format!("--angle:{}deg", i as f64 * 22.5) class="round-streak absolute h-[0.6cqh] w-[14cqw] origin-left bg-gradient-to-r from-white to-transparent"/> }).collect_view()}
                                <div class="round-flash absolute inset-0 bg-white pointer-events-none"/>
                                <div class="relative z-10 text-center flex flex-col items-center max-w-[80cqw] px-8">
                                    <h2 class="round-slam-label mb-[3cqh] px-[2cqh] py-[0.8cqh] -skew-x-12 bg-accent-500 text-black font-black tracking-[0.3em] text-[3.4cqh] uppercase shadow-[0_0_30px_rgb(var(--glow-accent)/0.5)]">{label}</h2>
                                    <h1 style=format!("font-size:{}", title_size(&title, 13.0)) class="round-slam-title font-title font-black text-white uppercase leading-[0.95] drop-shadow-[0_1cqh_0_rgb(var(--glow-primary-deep)/0.9)]">{title.clone()}</h1>
                                </div>
                            </div>
                        }.into_any()
                    },
                    RoundLayout::Marquee => view! {
                        <div class="round-marquee relative z-10 w-[72cqw] flex flex-col items-center">
                            <div class="relative -mb-[1cqh] px-[5cqw] pt-[2.4cqh] pb-[3cqh] bg-accent-500 text-black [clip-path:polygon(8%_100%,0_0,100%_0,92%_100%)]">
                                <span class="font-title font-black uppercase tracking-[0.35em] text-[4cqh]">{label}</span>
                            </div>
                            <div class="relative w-full rounded-[3cqh] bg-panel border-[1.4cqh] border-accent-600 shadow-[0_0_80px_rgb(var(--glow-accent)/0.35)] px-[6cqw] py-[9cqh]">
                                {bulbs().into_iter().enumerate().map(|(i,(x,y))| view! { <span style=format!("left:{x}%;top:{y}%;--i:{}", i % 3) class="round-bulb absolute w-[1.8cqh] h-[1.8cqh] -ml-[0.9cqh] -mt-[0.9cqh] rounded-full bg-[#fff6d5]"/> }).collect_view()}
                                <h1 style=format!("font-size:{}", title_size(&title, 10.0)) class="round-sign font-title font-black text-white uppercase leading-none text-center [text-shadow:0_0_2cqh_rgb(var(--glow-primary)/0.9),0_0_6cqh_rgb(var(--glow-primary)/0.6)]">{title.clone()}</h1>
                            </div>
                            <div class="flex justify-between w-[50%]">
                                <div class="w-[1.6cqw] h-[10cqh] bg-gradient-to-b from-accent-700 to-transparent"/>
                                <div class="w-[1.6cqw] h-[10cqh] bg-gradient-to-b from-accent-700 to-transparent"/>
                            </div>
                        </div>
                    }.into_any(),
                }
            }}
        </div>
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn title_sizes_and_bulb_positions_match_react() {
        assert_eq!(title_size("short", 10.0), "10.0cqh");
        assert_eq!(title_size(&"😀".repeat(48), 13.0), "10.0cqh");
        assert_eq!(title_size(&"a".repeat(200), 10.0), "5.0cqh");
        assert_eq!(bulbs().len(), 60);
        assert!(
            bulbs()
                .iter()
                .all(|(x, y)| *x >= 0.0 && *x <= 100.0 && *y >= 0.0 && *y <= 100.0)
        );
    }
}
