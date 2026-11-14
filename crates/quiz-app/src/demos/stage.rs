use crate::stage::{Display, StageLayout};
use leptos::prelude::*;
#[component]
pub fn StageDemo() -> impl IntoView {
    let layout = expect_context::<Signal<StageLayout>>();
    let display = expect_context::<RwSignal<Display>>();
    view! {
        <style>{include_str!("../../styles/confetti.css")}</style>
        <section class="calibration">
            <i class="corner tl"/><i class="corner tr"/><i class="corner bl"/><i class="corner br"/>
            <div class="safe-area"/>
            <h1>"Stage calibration"</h1>
            <p>"All four corners should be visible. The circle should look round."</p>
            <div class="calibration-circle"/>
            <div class="demo-controls">
                <p>{move || { let l=layout.get(); format!("{:.0} × {:.0} px · scale {:.4}",l.w,l.h,l.scale) }}</p>
                <label>"Content size "
                    <input type="range" min="0.7" max="1.3" step="0.01"
                        prop:value=move || display.get().size
                        on:input=move |e| { if let Ok(v)=event_target_value(&e).parse::<f64>() { display.update(|d| d.size=v.clamp(0.7,1.3)); } }/>
                    {move || format!("{:.0}%",display.get().size*100.)}
                </label>
                <label>"Margin "
                    <input type="range" min="0" max="10" step="0.5"
                        prop:value=move || display.get().margin
                        on:input=move |e| { if let Ok(v)=event_target_value(&e).parse::<f64>() { display.update(|d| d.margin=v.clamp(0.,10.)); } }/>
                    {move || format!("{:.1}%",display.get().margin)}
                </label>
                <button on:click=move |_| display.set(Display::default())>"Reset"</button>
            </div>
        </section>
    }
}
