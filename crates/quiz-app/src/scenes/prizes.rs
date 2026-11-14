use super::icons::IntroIcon;
use crate::{i18n::use_t, quiz_context::use_quiz, show::use_show};
use leptos::prelude::*;
use quiz_core::show::Action;
#[component]
pub fn Prizes() -> impl IntoView {
    let s = use_show();
    let t = use_t();
    let p = quiz_core::quiz::intro::prizes_for(&use_quiz().quiz).unwrap_or_default();
    let extra = p.extra;
    let prizes = [
        (
            2,
            "prize_silver",
            p.second,
            "from-slate-400 to-slate-600",
            "shadow-slate-400/30",
            "h-[500px]",
            0.2,
        ),
        (
            1,
            "prize_gold",
            p.first,
            "from-yellow-300 to-yellow-600",
            "shadow-yellow-500/50",
            "h-[580px]",
            0.0,
        ),
        (
            3,
            "prize_bronze",
            p.third,
            "from-orange-400 to-orange-700",
            "shadow-orange-500/30",
            "h-[450px]",
            0.4,
        ),
    ];
    view! {
    <div class="h-full w-full bg-ground relative overflow-hidden flex flex-col items-center justify-center font-show">
        <crate::scenes::backdrop::SceneBackdrop/>



          <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full h-[500px] bg-accent-600/10 \
        blur-[130px] rounded-full pointer-events-none" />


          <div


             class="intro-prizes-0 relative z-10 text-center mb-12"
          >
              <h1 class="font-title text-8xl font-black text-white uppercase tracking-[0.1em] drop-shadow-2xl \
        mb-2">{move || t.get("prizes_title")}</h1>
          </div>


          <div class="flex items-end justify-center gap-8 relative z-10 px-10 pb-8">
              {prizes.into_iter().map(|(rank, key, text, color, glow, height, delay)| { view! {
                  <div




                    style=format!("--delay:{delay}s;--intro-ease:{}", crate::spring::Spring::from_bounce(0.3, 0.8).to_css_linear())
    class={format!("intro-prizes-1 relative w-[320px] {} rounded-[3rem] p-[2px] bg-gradient-to-b {} {} shadow-2xl \
        flex-shrink-0 group hover:-translate-y-4 transition-transform duration-500", height, color, glow)}
                  >

                      <div class="w-full h-full bg-panel rounded-[2.9rem] flex flex-col items-center p-8 relative \
        overflow-hidden">


                          <div class={format!("absolute inset-0 bg-gradient-to-b {} opacity-10 group-hover:opacity-20 \
        transition-opacity duration-500", color)} />


                          <div class={format!("w-20 h-20 rounded-full bg-gradient-to-br {} flex items-center \
        justify-center text-black font-black text-4xl mb-10 shadow-lg z-10 ring-4 ring-black/50", color)}>
                              "#"{rank}
                          </div>


                          <div class={format!("text-white mb-8 drop-shadow-[0_0_25px_rgba(255,255,255,0.4)] transform \
        group-hover:scale-110 group-hover:rotate-3 transition-all duration-500", )}>
                              <IntroIcon name=if rank == 1 {"trophy"} else {"medal"} size=if rank == 1 {80} else if rank == 2 {64} else {56}/>
                          </div>


                          <h2 class={format!("text-4xl text-center font-black text-transparent bg-clip-text \
        bg-gradient-to-br {} uppercase tracking-wider mb-4", color)}>
                              {move || t.get(key)}
                          </h2>

                          <div class="w-12 h-1 bg-white/10 rounded-full my-6" />


                          {(!text.is_empty()).then(|| view! {
                              <p class="relative z-10 text-3xl font-bold text-white text-center leading-snug \
        line-clamp-4 break-words">{text}</p>
                          })}


                          {(rank == 1).then(|| view! {
                             <div class="absolute inset-0 bg-gradient-to-tr from-transparent via-white/20 \
        to-transparent translate-y-full group-hover:animate-[shimmer_2s_infinite]" />
                          })}
                      </div>
                  </div>
              }}).collect_view()}
          </div>


          {(!extra.is_empty()).then(|| view! {
            <div



               class="intro-prizes-2 relative z-10 flex items-center gap-4 px-8 py-4 rounded-2xl bg-white/5 border \
        border-white/10 text-2xl"
            >
                <IntroIcon name="gift" size=32 class="text-pink-400 shrink-0"/>
                <span class="text-white/50 uppercase tracking-widest font-bold">{move || t.get("prize_last")}</span>
                <span class="text-white font-bold">{extra}</span>
            </div>
          })}


          <div



             class="intro-prizes-3 mt-8 relative z-10"
          >
             <button
                data-tour="next"
                on:click=move |_| s.dispatch.run(Action::NextPrizes)
                class="px-12 py-4 bg-white/10 hover:bg-accent-500 text-white hover:text-black border border-white/20 \
        hover:border-accent-500 font-bold text-lg uppercase tracking-[0.2em] rounded-xl transition-all hover:scale-105 backdrop-blur-md"
            >
                {move || t.get("continue_dash")}
            </button>
          </div>

        </div>
    }
}
