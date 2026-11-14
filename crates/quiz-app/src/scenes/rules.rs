use super::icons::IntroIcon;
use crate::{i18n::use_t, quiz_context::use_quiz, show::use_show};
use leptos::prelude::*;
use quiz_core::show::Action;
#[component]
pub fn Rules() -> impl IntoView {
    let s = use_show();
    let t = use_t();
    let defaults = quiz_core::quiz::intro::DEFAULT_RULE_KEYS.map(|key| t.get(key));
    let rules = quiz_core::quiz::intro::rules_for(&use_quiz().quiz, &defaults).unwrap_or_default();
    let columns = if rules.len() <= 2 {
        "grid-cols-1 max-w-3xl mx-auto"
    } else {
        "grid-cols-2"
    };
    let text_size = if rules.len() <= 4 {
        "text-2xl"
    } else if rules.len() <= 6 {
        "text-xl"
    } else {
        "text-lg"
    };
    const ICONS: [&str; 8] = [
        "shield",
        "gavel",
        "triangle_alert",
        "check",
        "clock",
        "users",
        "star",
        "info",
    ];
    const COLORS: [&str; 8] = [
        "text-primary-400",
        "text-red-400",
        "text-yellow-400",
        "text-green-400",
        "text-purple-400",
        "text-cyan-400",
        "text-orange-400",
        "text-pink-400",
    ];
    view! {
    <div class="h-full w-full bg-ground relative overflow-hidden flex flex-col items-center justify-center font-show">
        <crate::scenes::backdrop::SceneBackdrop/>



          <div class="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[500px] bg-primary-600/10 blur-[120px] \
        rounded-full opacity-50 pointer-events-none" />
          <div class="absolute bottom-0 right-0 w-[600px] h-[600px] bg-purple-900/10 blur-[150px] rounded-full \
        opacity-30 pointer-events-none" />


          <div



             class="intro-rules-0 relative z-10 w-full max-w-5xl"
          >

            <div class="text-center mb-16 relative">
                 <h1



                     style=format!("--intro-ease:{}", crate::spring::Spring::new(100.0, 10.0, 1.0).to_css_linear()) class="intro-rules-1 font-title text-8xl font-black text-white uppercase tracking-[0.1em] drop-shadow-2xl"
                 >
                    {move || t.get("rules_title")}
                 </h1>
                 <div



                     class="intro-rules-2 h-1 bg-primary-500 mx-auto mt-4 rounded-full \
        shadow-[0_0_15px_rgb(var(--glow-primary)/0.5)]"
                 />
            </div>


            <div class={format!("grid {} gap-6 px-8", columns)}>
                {rules.into_iter().enumerate().map(|(i, text)| { view! {
                    <div




                        style=format!("--x:{}px;--delay:{}s;--intro-ease:{}", if i % 2 == 0 {-30} else {30}, 0.4 + i as f64 * 0.1, crate::spring::Spring::from_bounce(0.2, 0.8).to_css_linear()) class="intro-rules-3 group relative overflow-hidden bg-white/5 border border-white/10 rounded-3xl p-8 hover:bg-white/10 transition-colors duration-300"
                    >

                        <div class="absolute inset-0 bg-gradient-to-r from-primary-500/0 via-white/5 to-primary-500/0 \
        translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000" />

                        <div class="flex items-center gap-6 relative z-10">
                            <div class="flex-shrink-0 w-16 h-16 rounded-2xl bg-black/40 border border-white/10 flex \
        items-center justify-center shadow-inner group-hover:scale-110 transition-transform duration-300">
                                <IntroIcon name=ICONS[i % 8] size=32 class=COLORS[i % 8]/>
                            </div>
                            <span class={format!("{} text-gray-200 font-bold leading-tight group-hover:text-white \
        transition-colors", text_size)}>
                                {text}
                            </span>
                        </div>
                    </div>
                }}).collect_view()}
            </div>


            <div



                 class="intro-rules-4 mt-16 flex justify-center"
            >
                <button
                    data-tour="next"
                    on:click=move |_| s.dispatch.run(Action::NextRules)
                    class="group relative px-12 py-5 bg-white text-black font-black text-xl uppercase tracking-widest \
        rounded-full overflow-hidden transition-all hover:scale-105 hover:shadow-[0_0_40px_rgba(255,255,255,0.3)]"
                >
                    <div class="absolute inset-0 bg-gradient-to-r from-primary-400 to-cyan-300 opacity-0 \
        group-hover:opacity-100 transition-opacity duration-300" />
                    <span class="relative z-10 flex items-center gap-3">
                        {move || t.get("continue")} <IntroIcon name="arrow_right" size=24 class="group-hover:translate-x-1 transition-transform"/>
                    </span>
                </button>
            </div>

          </div>
        </div>
    }
}
