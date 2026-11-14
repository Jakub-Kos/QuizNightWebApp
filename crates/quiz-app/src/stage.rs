//! The stage: scenes are laid out on a virtual canvas that is scaled to fit the window, so they use
//! cqh/cqw (percent of the canvas) where they would use vh/vw. Port of Stage.jsx and useDisplay.js.

use leptos::ev;
use leptos::prelude::*;

/// Virtual canvas at 100% content size: a 1920x1080 screen at 80% browser zoom.
pub const BASE_W: f64 = 2400.0;
pub const BASE_H: f64 = 1350.0;

/// Per-device calibration. `size`: content size multiplier (bigger = larger elements, smaller
/// canvas). `margin`: edge inset in percent of the window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Display {
    pub size: f64,
    pub margin: f64,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            size: 1.0,
            margin: 0.0,
        }
    }
}

/// Canvas size in px and the factor that fits it into the window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageLayout {
    pub w: f64,
    pub h: f64,
    pub scale: f64,
}

pub fn stage_layout(viewport: (f64, f64), display: Display) -> StageLayout {
    let w = (BASE_W / display.size).round();
    let h = (BASE_H / display.size).round();
    let free = 1.0 - 2.0 * display.margin / 100.0;
    let scale = (viewport.0 * free / w).min(viewport.1 * free / h);
    StageLayout { w, h, scale }
}

fn viewport_size() -> (f64, f64) {
    let size = |value: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>| {
        value.ok().and_then(|v| v.as_f64()).unwrap_or(0.0)
    };
    (size(window().inner_width()), size(window().inner_height()))
}

/// Scales its children to the window. Provides `RwSignal<Display>` (the calibration) and
/// `Signal<StageLayout>` (canvas size in px, for animations that cannot use cqh units) as context.
#[component]
pub fn Stage(
    children: Children,
    #[prop(optional)] display: Option<RwSignal<Display>>,
    #[prop(optional, into)] bypass: Signal<bool>,
) -> impl IntoView {
    let viewport = RwSignal::new(viewport_size());
    // The stage lives as long as the page, so the listener is never removed.
    let _ = window_event_listener(ev::resize, move |_| viewport.set(viewport_size()));

    let display = display.unwrap_or_else(|| RwSignal::new(Display::default()));
    let layout = Signal::derive(move || {
        if bypass.get() {
            let (w, h) = viewport.get();
            StageLayout { w, h, scale: 1.0 }
        } else {
            stage_layout(viewport.get(), display.get())
        }
    });
    provide_context(display);
    provide_context(layout);

    let style = move || {
        let StageLayout { w, h, scale } = layout.get();
        if bypass.get() {
            "position:absolute;inset:0;left:0;top:0;width:100%;height:100%;transform:none"
                .to_string()
        } else {
            format!("width:{w}px;height:{h}px;transform:translate(-50%,-50%) scale({scale})")
        }
    };
    view! {
        <div class="stage-viewport">
            <div class="stage font-show" style=style>{children()}</div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fits_a_full_hd_window() {
        let layout = stage_layout((1920.0, 1080.0), Display::default());
        assert_eq!((layout.w, layout.h), (2400.0, 1350.0));
        assert!((layout.scale - 0.8).abs() < 1e-9);
    }

    #[test]
    fn size_and_margin_shrink_the_picture() {
        let layout = stage_layout(
            (1920.0, 1080.0),
            Display {
                size: 1.25,
                margin: 5.0,
            },
        );
        assert_eq!((layout.w, layout.h), (1920.0, 1080.0));
        assert!((layout.scale - 0.9).abs() < 1e-9);
    }
}
