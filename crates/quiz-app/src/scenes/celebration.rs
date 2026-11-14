//! Theme-selected, eight-second final standings celebration.
use super::backdrop::ThemeDecorations;
use crate::{particles::ParticleCanvas, theme::use_theme};
use leptos::prelude::*;
use quiz_core::theme::format::{Celebration as Kind, Decoration};
use std::time::Duration;

fn effective_kind(kind: Option<Kind>, decoration: Option<Decoration>) -> Option<Kind> {
    match (kind, decoration) {
        (Some(Kind::Decoration), None) => Some(Kind::Confetti),
        _ => kind,
    }
}
#[component]
pub fn Celebration(on_done: Callback<()>) -> impl IntoView {
    let theme = use_theme().get_untracked();
    let kind = effective_kind(theme.celebration.r#type, theme.decoration.r#type);
    let active = RwSignal::new(false);
    let trigger = RwSignal::new(0_u64);
    let fireworks = RwSignal::new(kind == Some(Kind::Fireworks));
    let multiplier = RwSignal::new(1_usize);
    let count = RwSignal::new(0_usize);
    let frame_ms = RwSignal::new(0.);
    let fading = RwSignal::new(false);
    let fade = set_timeout_with_handle(move || fading.set(true), Duration::from_millis(6800)).ok();
    let done = set_timeout_with_handle(move || on_done.run(()), Duration::from_millis(8000)).ok();
    on_cleanup(move || {
        if let Some(h) = fade {
            h.clear();
        }
        if let Some(h) = done {
            h.clear();
        }
    });
    Effect::new(move |_| {
        if matches!(kind, Some(Kind::Confetti | Kind::Fireworks)) {
            trigger.set(1);
        }
    });
    let palette = Signal::stored(crate::particles::theme_palette(&theme));
    view! {
        <div class="absolute inset-0 z-[45] pointer-events-none transition-opacity duration-1000 standings-celebration" style:opacity=move || if fading.get() {"0"} else {"1"} aria-hidden="true">
            {matches!(kind,Some(Kind::Confetti|Kind::Fireworks)).then(|| view! {
                <ParticleCanvas active=active trigger=trigger fireworks=fireworks multiplier=multiplier count=count frame_ms=frame_ms palette=palette/>
            })}
            {(kind==Some(Kind::Decoration)).then(|| view! {<ThemeDecorations density=3.0/>})}
        </div>
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disabled_and_fallback() {
        assert_eq!(effective_kind(None, Some(Decoration::Stars)), None);
        assert_eq!(
            effective_kind(Some(Kind::Decoration), None),
            Some(Kind::Confetti)
        );
        assert_eq!(
            effective_kind(Some(Kind::Decoration), Some(Decoration::Snow)),
            Some(Kind::Decoration)
        );
        assert_eq!(
            effective_kind(Some(Kind::Fireworks), None),
            Some(Kind::Fireworks)
        );
    }
}
