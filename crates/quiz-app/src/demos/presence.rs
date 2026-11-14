use crate::presence::{Presence, PresenceSwitch};
use leptos::{ev, prelude::*};

#[component]
pub fn PresenceDemo() -> impl IntoView {
    let scene = RwSignal::new(0_u64);
    let wait = RwSignal::new(false);
    let modal = RwSignal::new(false);
    let listener = window_event_listener(ev::keydown, move |e| {
        if e.key() == "ArrowRight" {
            scene.update(|s| *s = 1 - *s);
        }
        if e.key() == "Escape" {
            modal.set(false);
        }
    });
    on_cleanup(move || listener.remove());
    view! {
        <style>{include_str!("../../styles/presence.css")}</style>
        <div class="presence-demo">
            <PresenceSwitch scene=scene wait=wait enter="scene-enter" exit="scene-exit" render=move |key| view! {
                <section class="presence-scene" data-scene=key>
                    {if key == 0 { view! {
                        <div class="classic-card">
                            <h2 style="--i:0">"Round 3"</h2>
                            <div class="round-rule" style="--i:1"></div>
                            <h1 class="font-title" style="--i:2">"Music & Movies"</h1>
                        </div>
                    }.into_any() } else { view! {
                        <div class="second-panel"><h1>"Ready to play?"</h1><p>"Your next question awaits."</p></div>
                    }.into_any() }}
                </section>
            }/>
            <nav class="presence-controls">
                <button on:click=move |_| scene.update(|s| *s = 1 - *s)>"Next scene →"</button>
                <label><input type="checkbox" prop:checked=move || wait.get() on:change=move |e| wait.set(event_target_checked(&e))/>" Wait for exit"</label>
                <button on:click=move |_| modal.update(|v| *v = !*v)>"Toggle modal"</button>
            </nav>
            <Presence when=modal enter="modal-enter" exit="modal-exit">
                <div class="modal-backdrop" on:click=move |_| modal.set(false)>
                    <div class="modal-panel" role="dialog" aria-modal="true" aria-label="Pause" on:click=move |e| e.stop_propagation()>
                        <h2>"Take a breath"</h2><p>"The next round is coming."</p>
                        <button on:click=move |_| modal.set(false)>"Continue"</button>
                    </div>
                </div>
            </Presence>
        </div>
    }
}
