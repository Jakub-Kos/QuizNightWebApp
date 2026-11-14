//! Keyed DOM reordering with native snapshots or interruptible FLIP.
use crate::spring::Spring;
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    ViewTransition,
    Flip,
}
pub fn reduced_motion() -> bool {
    window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|q| q.matches())
}
fn items() -> Vec<web_sys::Element> {
    let nodes = document().query_selector_all("[data-reorder]").unwrap();
    (0..nodes.length())
        .filter_map(|i| nodes.item(i)?.dyn_into().ok())
        .collect()
}
fn cancel(el: &web_sys::Element) {
    if let Ok(f) = js_sys::Reflect::get(el, &"getAnimations".into()) {
        if let Some(f) = f.dyn_ref::<js_sys::Function>() {
            if let Ok(a) = f.call0(el) {
                for a in js_sys::Array::from(&a) {
                    if let Ok(a) = a.dyn_into::<web_sys::Animation>() {
                        a.cancel();
                    }
                }
            }
        }
    }
}
/// The callback may update any keyed Leptos list. The returned transition callback
/// promise waits for Leptos's reactive DOM effects before snapshotting.
pub fn reorder(mode: Mode, scale: f64, change: impl FnOnce() + 'static) {
    reorder_with(mode, scale, Spring::from_bounce(0.25, 1.2), change);
}
/// `reorder` with the given spring for the glide.
pub fn reorder_with(mode: Mode, scale: f64, spring: Spring, change: impl FnOnce() + 'static) {
    if reduced_motion() {
        change();
        return;
    }
    let root = document()
        .document_element()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>();
    let _ = root
        .style()
        .set_property("--reorder-duration", &format!("{}s", spring.settle_time()));
    let _ = root
        .style()
        .set_property("--reorder-easing", &spring.to_css_linear());
    if mode == Mode::ViewTransition {
        if let Ok(method) = js_sys::Reflect::get(&document(), &"startViewTransition".into()) {
            if let Some(method) = method.dyn_ref::<js_sys::Function>() {
                let cb = Closure::once_into_js(move || {
                    change();
                    wasm_bindgen_futures::future_to_promise(async {
                        leptos::task::tick().await;
                        Ok(JsValue::UNDEFINED)
                    })
                });
                let _ = method.call1(&document(), &cb);
                return;
            }
        }
    }
    let before = items()
        .into_iter()
        .map(|el| {
            let rect = el.get_bounding_client_rect();
            cancel(&el);
            (el, rect)
        })
        .collect::<Vec<_>>();
    change();
    leptos::task::spawn_local(async move {
        leptos::task::tick().await;
        for (el, old) in before {
            let new = el.get_bounding_client_rect();
            let frames = js_sys::Array::new();
            for transform in [
                format!(
                    "translate({}px,{}px)",
                    (old.x() - new.x()) / scale.max(0.001),
                    (old.y() - new.y()) / scale.max(0.001)
                ),
                "translate(0px,0px)".into(),
            ] {
                let frame = js_sys::Object::new();
                let _ = js_sys::Reflect::set(&frame, &"transform".into(), &transform.into());
                frames.push(&frame);
            }
            let options = js_sys::Object::new();
            let _ = js_sys::Reflect::set(
                &options,
                &"duration".into(),
                &(spring.settle_time() * 1000.0).into(),
            );
            let _ =
                js_sys::Reflect::set(&options, &"easing".into(), &spring.to_css_linear().into());
            if let Ok(method) = js_sys::Reflect::get(&el, &"animate".into()) {
                if let Some(method) = method.dyn_ref::<js_sys::Function>() {
                    let _ = method.call2(&el, &frames, &options);
                }
            }
        }
    });
}
