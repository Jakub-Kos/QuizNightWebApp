//! Shared colour popover with saturation/value, hue, hex entry and swatches.
use leptos::prelude::*;
use wasm_bindgen::JsCast;
const BRIGHTS: &[&str] = &[
    "#3b82f6", "#00b4d8", "#14b8a6", "#22c55e", "#84cc16", "#eab308", "#f5c542", "#ff7a1a",
    "#ef4444", "#e0303a", "#ec4899", "#a855f7", "#8b3dff", "#6366f1", "#94a3b8", "#ffffff",
];
const DARKS: &[&str] = &[
    "#050505", "#111111", "#0b0614", "#1d0c30", "#06150e", "#0f2a1d", "#06101d", "#0b1d33",
    "#1a0a0a", "#2a1410",
];
fn parse_hex(value: &str) -> Option<[f64; 3]> {
    let hex = value.trim().trim_start_matches('#');
    if !matches!(hex.len(), 3 | 6) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let hex = if hex.len() == 3 {
        hex.chars().flat_map(|c| [c, c]).collect()
    } else {
        hex.to_owned()
    };
    Some([0, 2, 4].map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap() as f64 / 255.))
}
fn hsv(hex: &str) -> (f64, f64, f64) {
    let [r, g, b] = parse_hex(hex).unwrap_or([0., 0., 0.]);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d == 0. {
        0.
    } else if max == r {
        ((g - b) / d).rem_euclid(6.)
    } else if max == g {
        (b - r) / d + 2.
    } else {
        (r - g) / d + 4.
    };
    (h * 60., if max == 0. { 0. } else { d / max }, max)
}
fn hex(h: f64, s: f64, v: f64) -> String {
    let c = v * s;
    let x = c * (1. - ((h / 60.).rem_euclid(2.) - 1.).abs());
    let m = v - c;
    let rgb = match (h.rem_euclid(360.) / 60.) as u8 {
        0 => [c, x, 0.],
        1 => [x, c, 0.],
        2 => [0., c, x],
        3 => [0., x, c],
        4 => [x, 0., c],
        _ => [c, 0., x],
    };
    let [r, g, b] = rgb.map(|n| ((n + m) * 255.).round() as u8);
    format!("#{r:02x}{g:02x}{b:02x}")
}
#[component]
pub fn ColorPicker(
    value: Signal<String>,
    on_change: Callback<String>,
    label: String,
    #[prop(optional)] theme_colors: Option<Signal<Vec<String>>>,
    #[prop(optional)] dark: bool,
) -> impl IntoView {
    let t = crate::i18n::use_t();
    let open = RwSignal::new(false);
    let root = NodeRef::<leptos::html::Div>::new();
    let color = RwSignal::new(hsv(&value.get_untracked()));
    Effect::new(move |_| {
        let next = hsv(&value.get());
        color.update(|old| {
            if next.1 > 0. {
                old.0 = next.0;
            }
            old.1 = next.1;
            old.2 = next.2;
        });
    });
    let pointer = window_event_listener(leptos::ev::pointerdown, move |ev| {
        if let Some(root) = root.get()
            && !ev
                .target()
                .and_then(|v| v.dyn_into::<web_sys::Node>().ok())
                .is_some_and(|n| root.contains(Some(&n)))
        {
            open.set(false);
        }
    });
    let key = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            open.set(false);
        }
    });
    on_cleanup(move || {
        pointer.remove();
        key.remove();
    });
    let set = Callback::new(move |v: String| {
        if let Some([r, g, b]) = parse_hex(&v) {
            on_change.run(format!(
                "#{:02x}{:02x}{:02x}",
                (r * 255.).round() as u8,
                (g * 255.).round() as u8,
                (b * 255.).round() as u8
            ));
        }
    });
    let field = NodeRef::<leptos::html::Div>::new();
    let pick = move |ev: web_sys::PointerEvent| {
        if let Some(el) = field.get() {
            let rect = el.get_bounding_client_rect();
            let (h, _, _) = color.get_untracked();
            let s = ((ev.client_x() as f64 - rect.left()) / rect.width()).clamp(0., 1.);
            let v = (1. - (ev.client_y() as f64 - rect.top()) / rect.height()).clamp(0., 1.);
            color.set((h, s, v));
            on_change.run(hex(h, s, v));
        }
    };
    let eyedropper = js_sys::Reflect::has(&window(), &"EyeDropper".into()).unwrap_or(false);
    view! { <div node_ref=root class="relative shrink-0">
        <button type="button" class="w-11 h-11 rounded-lg ring-1 ring-white/20 hover:ring-white/60 shadow-inner" style=move ||format!("background:{}",value.get()) aria-label=label.clone() aria-expanded=move ||open.get() on:click=move |_|open.update(|v|*v=!*v)/>
        <Show when=move ||open.get()><div class="absolute left-0 top-full mt-2 z-50 w-64 p-3 rounded-2xl bg-[#16161a] border border-white/15 shadow-2xl space-y-3">
            <div class="flex justify-between items-center"><strong class="text-sm">{label.clone()}</strong><button type="button" aria-label=t.get("chk_close") on:click=move |_|open.set(false)>"×"</button></div>
            <div node_ref=field class="color-saturation" style=move ||format!("background-color:{}",hex(color.get().0,1.,1.)) on:pointerdown=move |e|{e.prevent_default();if let Some(el)=field.get(){let _=el.set_pointer_capture(e.pointer_id());}pick(e);} on:pointermove=move |e|if e.buttons()!=0{pick(e);}>
                <span class="color-cursor" style=move ||format!("left:{}%;top:{}%",color.get().1*100.,(1.-color.get().2)*100.)/>
            </div>
            <input type="range" min="0" max="359" class="color-hue w-full" aria-label=label.clone() prop:value=move ||color.get().0 on:input=move |e|{let h=event_target_value(&e).parse().unwrap_or(0.);let (_,s,v)=color.get_untracked();color.set((h,s,v));on_change.run(hex(h,s,v));}/>
            <div class="flex items-center gap-2"><span>"#"</span><input class="min-w-0 w-full bg-black/40 rounded-lg p-2 font-mono uppercase text-sm" aria-label=label.clone() prop:value=move ||value.get().trim_start_matches('#').to_owned() on:input=move |e|set.run(event_target_value(&e))/>
                <Show when=move ||eyedropper><button type="button" title=t.get("cp_eyedropper") aria-label=t.get("cp_eyedropper") on:click=move |_|{leptos::task::spawn_local(async move{let f=js_sys::Function::new_no_args("return new EyeDropper().open().then(r=>r.sRGBHex)");if let Ok(v)=f.call0(&wasm_bindgen::JsValue::NULL) && let Ok(v)=wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&v)).await && let Some(v)=v.as_string(){set.run(v);}});}>
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m16 3 5 5-4 4-2-2-9 9H3v-3l9-9-2-2 4-4Z"/></svg>
                </button></Show>
            </div>
            {move || {let mut others=Vec::new();for c in theme_colors.map(|s|s.get()).unwrap_or_default(){if !c.is_empty() && c!=value.get() && !others.contains(&c){others.push(c);}} (!others.is_empty()).then(||view!{<span class="text-xs text-gray-500">{t.get("cp_theme")}</span><div class="theme-color-swatches">{others.into_iter().take(8).map(|c|view!{<Swatch color=c value=value set=set/>}).collect_view()}</div>})}}
            <span class="text-xs text-gray-500">{t.get("cp_suggested")}</span><div class="theme-color-swatches">{(if dark{DARKS}else{BRIGHTS}).iter().map(|c|view!{<Swatch color=(*c).to_owned() value=value set=set/>}).collect_view()}</div>
        </div></Show>
    </div> }
}
#[component]
fn Swatch(color: String, value: Signal<String>, set: Callback<String>) -> impl IntoView {
    let c = StoredValue::new(color.clone());
    view! {<button type="button" style=format!("background:{color}") title=color.clone() aria-label=color aria-pressed=move ||value.get()==c.get_value() on:click=move |_|set.run(c.get_value())/>}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn colours() {
        for c in BRIGHTS.iter().chain(DARKS) {
            let (h, s, v) = hsv(c);
            assert_eq!(hex(h, s, v), *c);
        }
        assert_eq!(parse_hex("abc"), parse_hex("#aabbcc"));
        assert!(parse_hex("#zzzzzz").is_none());
        assert!(parse_hex("é0000").is_none());
    }
}
