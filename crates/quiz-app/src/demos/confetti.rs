use crate::particles::ParticleCanvas;
use leptos::prelude::*;
#[component]
pub fn ConfettiDemo() -> impl IntoView {
    let active = RwSignal::new(false);
    let trigger = RwSignal::new(0u64);
    let fireworks = RwSignal::new(false);
    let multiplier = RwSignal::new(1usize);
    let count = RwSignal::new(0usize);
    let frame_ms = RwSignal::new(0f64);
    let fire = move |kind| {
        fireworks.set(kind);
        trigger.update(|n| *n += 1);
    };
    let listener = window_event_listener(leptos::ev::keydown, move |e| {
        if e.repeat() {
            return;
        }
        match e.key().to_lowercase().as_str() {
            "c" => fire(false),
            "f" => fire(true),
            _ => (),
        }
    });
    on_cleanup(move || listener.remove());
    view! {
        <style>{include_str!("../../styles/confetti.css")}</style>
        <section class="celebration-demo">
            <p>"THE GRAND FINALE"</p><h1>"Final standings"</h1>
            <p>"1 · Quiz royalty     2 · The wild guesses     3 · Brainstorm"</p>
            <div class="demo-controls">
                <button on:click=move |_| fire(false)>"Confetti · C"</button>
                <button on:click=move |_| fire(true)>"Fireworks · F"</button>
                <button on:click=move |_| multiplier.set(1)>"×1"</button>
                <button on:click=move |_| multiplier.set(5)>"×5"</button>
                <button on:click=move |_| multiplier.set(20)>"×20"</button>
                <p>{move || format!("×{} · {} particles · {:.2} ms simulation + draw",multiplier.get(),count.get(),frame_ms.get())}</p>
            </div>
        </section>
        <ParticleCanvas active trigger fireworks multiplier count frame_ms/>
    }
}
