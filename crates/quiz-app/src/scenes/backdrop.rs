//! Scene backgrounds and ambient decorations, ported from Theme.jsx.
//! Geometry and animation parameters are pure and use the reference's Park–Miller seeds.

use leptos::prelude::*;
use quiz_core::theme::format::{Decoration, Ornament, Theme};
use std::{
    fmt::Write,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Seeded(u64);
impl Seeded {
    fn next(&mut self) -> f64 {
        self.0 = self.0 * 16807 % 2147483647;
        (self.0 - 1) as f64 / 2147483646.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Snow,
    Bat,
    Ember,
    Leaf,
    Petal,
    Confetto,
    Heart,
    Bubble,
    Star,
    ShootingStar,
}

#[derive(Clone, Debug, PartialEq)]
struct Piece {
    kind: Kind,
    index: usize,
    left: f64,
    top: f64,
    size: f64,
    duration: f64,
    delay: f64,
    drift: f64,
    secondary: f64,
    opacity: f64,
    blur: f64,
    tint: f64,
    color: &'static str,
}
impl Piece {
    fn new(kind: Kind, index: usize) -> Self {
        Self {
            kind,
            index,
            left: 0.,
            top: 0.,
            size: 0.,
            duration: 0.,
            delay: 0.,
            drift: 0.,
            secondary: 0.,
            opacity: 1.,
            blur: 0.,
            tint: 0.,
            color: "#ffffff",
        }
    }
}

const LEAF_COLORS: &[&str] = &["#d9541e", "#e8a33d", "#b5361c", "#c97b2b", "#8c5a2b"];
const PETAL_COLORS: &[&str] = &["#f9c6d8", "#fbe3ec", "#f5a9c4", "#ffffff"];
const CONFETTI_COLORS: &[&str] = &[
    "var(--color-accent-400)",
    "var(--color-primary-400)",
    "#ffffff",
    "#ff5d8f",
    "#4ade80",
    "var(--color-accent-200)",
];
const BULBS: &[&str] = &["#ff3b3b", "#ffd447", "#39d353", "#4aa8ff"];
const FLOWERS: &[&str] = &["#f9a8d4", "#fde68a", "#ffffff", "#c4b5fd", "#fca5a5"];
const BAT: &str = "M50 14 L52.5 8.5 L53.6 15 C56 14 57 16 58 18 C64 9 76 7 92 11 C83 15 81 21 83 27 C77 23 70 25 68 31 C64 27 58 27 56 31 C54 29 52 31 50 35 C48 31 46 29 44 31 C42 27 36 27 32 31 C30 25 23 23 17 27 C19 21 17 15 8 11 C24 7 36 9 42 18 C43 16 44 14 46.4 15 L47.5 8.5 Z";
const LEAF: &str = "M50 0 C70 20 95 35 90 60 C85 85 60 95 50 100 C40 95 15 85 10 60 C5 35 30 20 50 0 Z M50 10 V100";
const HEART: &str = "M50 88 C20 65 4 48 4 30 C4 16 15 6 28 6 C38 6 46 12 50 20 C54 12 62 6 72 6 C85 6 96 16 96 30 C96 48 80 65 50 88 Z";

fn piece_count(base: usize, density: f64, round: bool) -> usize {
    // Public density props must not allocate unbounded DOM or accept NaN/infinity.
    if !density.is_finite() || density <= 0. {
        return 0;
    }
    let n = (base as f64 * density).min(10_000.);
    if round {
        n.round() as usize
    } else {
        n as usize
    }
}

fn decoration_pieces(theme: &Theme, density: f64) -> Vec<Piece> {
    let Some(decoration) = theme.decoration.r#type else {
        return vec![];
    };
    let mut pieces = vec![];
    match decoration {
        Decoration::Snow => {
            let mut rng = Seeded(7);
            let storm = density > 1.;
            for i in 0..piece_count(70, density, true) {
                let depth = if storm {
                    0.4 + rng.next() * 0.6
                } else {
                    rng.next()
                };
                let mut p = Piece::new(Kind::Snow, i);
                p.left = rng.next() * 100.;
                p.size = (0.35 + depth * 0.9) * if storm { 1.45 } else { 1. };
                p.duration = (22. - depth * 12.) * if storm { 0.55 } else { 1. };
                p.delay = -rng.next() * 22.;
                p.drift = (rng.next() - 0.5) * 4.;
                p.secondary = 3. + rng.next() * 4.;
                p.opacity = 0.35 + depth * 0.55;
                p.blur = if depth < 0.3 { 0.15 } else { 0. };
                pieces.push(p);
            }
        }
        Decoration::Bats => {
            let mut rng = Seeded(29);
            for i in 0..piece_count(16, density, true) {
                let mut p = Piece::new(Kind::Ember, i);
                p.left = rng.next() * 100.;
                p.size = 0.4 + rng.next() * 0.5;
                p.duration = 9. + rng.next() * 9.;
                p.delay = -rng.next() * 18.;
                p.drift = (rng.next() - 0.5) * 8.;
                p.color = if rng.next() < 0.2 {
                    "#7dffb8"
                } else {
                    "#ffab45"
                };
                pieces.push(p);
            }
            let mut rng = Seeded(13);
            for i in 0..piece_count(7, density, true) {
                let mut p = Piece::new(Kind::Bat, i);
                p.top = 6. + rng.next() * 40.;
                p.size = 0.55 + rng.next() * 0.7;
                p.duration = 16. + rng.next() * 14.;
                p.delay = -rng.next() * 30.;
                p.secondary = 0.22 + rng.next() * 0.12;
                pieces.push(p);
            }
        }
        Decoration::Leaves | Decoration::Petals | Decoration::Confetti => {
            let (kind, count, seed, size, duration, colors) = match decoration {
                Decoration::Leaves => (Kind::Leaf, 30, 71, [2.4, 4.2], [14., 24.], LEAF_COLORS),
                Decoration::Petals => (Kind::Petal, 34, 73, [0.8, 1.6], [12., 22.], PETAL_COLORS),
                _ => (
                    Kind::Confetto,
                    90,
                    79,
                    [1.1, 1.9],
                    [8., 16.],
                    CONFETTI_COLORS,
                ),
            };
            let mut rng = Seeded(seed);
            for i in 0..piece_count(count, density, false) {
                let mut p = Piece::new(kind, i);
                p.left = rng.next() * 100.;
                p.size = (size[0] + rng.next() * (size[1] - size[0]))
                    * if density > 1. { 1.4 } else { 1. };
                p.duration = duration[0] + rng.next() * (duration[1] - duration[0]);
                p.delay = -rng.next() * duration[1];
                p.drift = (rng.next() - 0.5) * 10.;
                p.secondary = 3. + rng.next() * 5.;
                p.color = colors[i % colors.len()];
                pieces.push(p);
            }
        }
        Decoration::Hearts | Decoration::Bubbles => {
            let (kind, count, seed, size, duration) = if decoration == Decoration::Hearts {
                (Kind::Heart, 16, 83, [1.6, 3.4], [12., 22.])
            } else {
                (Kind::Bubble, 22, 89, [0.8, 3.2], [10., 20.])
            };
            let mut rng = Seeded(seed);
            for i in 0..piece_count(count, density, false) {
                let mut p = Piece::new(kind, i);
                p.left = rng.next() * 100.;
                p.size = size[0] + rng.next() * (size[1] - size[0]);
                p.duration = duration[0] + rng.next() * (duration[1] - duration[0]);
                p.delay = -rng.next() * duration[1];
                p.drift = (rng.next() - 0.5) * 4.;
                p.tint = rng.next();
                p.secondary = 3. + p.tint * 3.;
                pieces.push(p);
            }
        }
        Decoration::Stars => {
            let mut rng = Seeded(67);
            for i in 0..piece_count(80, density, true) {
                let mut p = Piece::new(Kind::Star, i);
                p.left = rng.next() * 100.;
                p.top = rng.next() * 70.;
                p.size = 0.15 + rng.next() * 0.35;
                p.delay = -rng.next() * 4.;
                p.duration = 2. + rng.next() * 4.;
                pieces.push(p);
            }
            // The two shooting stars do not scale with density in the reference.
            for (i, top, delay) in [(0, 8., 0.), (1, 22., -7.)] {
                let mut p = Piece::new(Kind::ShootingStar, i);
                p.top = top;
                p.delay = delay;
                p.duration = 14.;
                pieces.push(p);
            }
        }
    }
    pieces
}

fn piece_view(p: Piece) -> AnyView {
    let Piece {
        kind,
        index,
        left,
        top,
        size,
        duration,
        delay,
        drift,
        secondary,
        opacity,
        blur,
        tint,
        color,
    } = p;
    match kind {
        Kind::Snow => view! {
            <span class="theme-anim absolute top-0 will-change-transform" style=format!("left:{left}%;--drift:{drift}cqw;animation:theme-fall {duration}s linear {delay}s infinite")>
                <span class="theme-anim block rounded-full bg-white" style=format!("width:{size}cqh;height:{size}cqh;opacity:{opacity};filter:blur({blur}cqh);--sway:{}cqw;animation:theme-sway {secondary}s ease-in-out {delay}s infinite", drift / 2.)/>
            </span>
        }.into_any(),
        Kind::Bat => view! {
            <span class="theme-anim absolute left-0 will-change-transform" style=format!("top:{top}cqh;--scale:{size};animation:theme-fly {duration}s ease-in-out {delay}s infinite")>
                <svg viewBox="0 0 100 40" class="theme-anim block w-[6cqw]" style=format!("animation:theme-flap {secondary}s ease-in-out infinite;filter:drop-shadow(0 0 0.3cqh rgba(255,140,60,0.8)) drop-shadow(0 0 1.2cqh rgba(160,80,255,0.35))")><path d=BAT fill="#0c0512"/></svg>
            </span>
        }.into_any(),
        Kind::Ember => view! {
            <span class="theme-anim absolute bottom-0 rounded-full will-change-transform" style=format!("left:{left}%;width:{size}cqh;height:{size}cqh;background:{color};box-shadow:0 0 1.2cqh {color};--drift:{drift}cqw;animation:theme-rise {duration}s ease-out {delay}s infinite")/>
        }.into_any(),
        Kind::Leaf | Kind::Petal | Kind::Confetto => {
            let shape = match kind {
                Kind::Leaf => view! { <svg viewBox="0 0 100 100" class="w-full h-full"><path d=LEAF fill=color stroke="rgba(0,0,0,0.25)" stroke-width="3"/></svg> }.into_any(),
                Kind::Petal => view! { <span class="block w-full h-full" style=format!("background:{color};border-radius:80% 0 80% 0;opacity:0.85")/> }.into_any(),
                _ => view! { <span class="block w-full" style=format!("background:{color};height:{};border-radius:{}", if index.is_multiple_of(3) { "100%" } else { "45%" }, if index.is_multiple_of(3) { "50%" } else { "1px" })/> }.into_any(),
            };
            view! { <span class="theme-anim absolute top-0 will-change-transform" style=format!("left:{left}%;--drift:{drift}cqw;animation:theme-fall {duration}s linear {delay}s infinite")>
                <span class="theme-anim block" style=format!("width:{size}cqh;height:{size}cqh;animation:theme-tumble {secondary}s linear {delay}s infinite")>{shape}</span>
            </span> }.into_any()
        }
        Kind::Heart | Kind::Bubble => {
            let shape = if kind == Kind::Heart {
                view! { <svg viewBox="0 0 100 100" class="w-full h-full"><path d=HEART fill=if tint < 0.5 { "#ff4d6d" } else { "#ff8fab" } style="filter:drop-shadow(0 0 6px rgba(255,77,109,0.6))"/></svg> }.into_any()
            } else {
                view! { <span class="block w-full h-full rounded-full" style="border:1px solid rgba(255,255,255,0.35);background:radial-gradient(circle at 30% 30%, rgba(255,255,255,0.45), rgba(255,255,255,0.05) 45%, transparent 70%)"/> }.into_any()
            };
            view! { <span class="theme-anim absolute bottom-0 will-change-transform" style=format!("left:{left}%;animation:theme-float {duration}s ease-in {delay}s infinite")>
                <span class="theme-anim block" style=format!("width:{size}cqh;height:{size}cqh;--sway:{drift}cqw;animation:theme-sway {secondary}s ease-in-out {delay}s infinite")>{shape}</span>
            </span> }.into_any()
        }
        Kind::Star => view! {
            <span class="theme-anim absolute rounded-full bg-white" style=format!("left:{left}%;top:{top}%;width:{size}cqh;height:{size}cqh;box-shadow:0 0 {}cqh rgba(255,255,255,0.8);animation:theme-twinkle {duration}s ease-in-out {delay}s infinite", size * 3.)/>
        }.into_any(),
        Kind::ShootingStar => view! {
            <span class="theme-anim absolute left-0 h-[0.25cqh] w-[12cqw] rounded-full" style=format!("top:{top}%;background:linear-gradient(90deg, transparent, rgba(255,255,255,0.9));animation:theme-shoot 14s ease-in {delay}s infinite")/>
        }.into_any(),
    }
}

/// URLs will come from the theme pack's media resolver, never from a pack filename directly.
fn resolved_image_url(_theme: &Theme) -> Option<String> {
    // TODO(theme-assets): resolve theme.background.image through the quiz's media store.
    None
}

fn image_style(url: &str, theme: &Theme, quiet: bool) -> String {
    // CSS string escaping keeps quotes/control characters in asset URLs inside url(...).
    let escaped: String = url
        .chars()
        .map(|c| match c {
            '"' => "\\\"".into(),
            '\\' => "\\\\".into(),
            c if c.is_control() => format!("\\{:x} ", c as u32),
            c => c.to_string(),
        })
        .collect();
    let opacity = theme.background.image_opacity * if quiet { 0.5 } else { 1. };
    let blur = theme.background.image_blur;
    let effect = if blur > 0. {
        format!(";filter:blur({blur}px);transform:scale(1.05)")
    } else {
        String::new()
    };
    format!("background-image:url(\"{escaped}\");opacity:{opacity}{effect}")
}

#[component]
pub fn SceneBackdrop(#[prop(optional)] quiet: bool) -> impl IntoView {
    let theme = crate::theme::use_theme();
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
    let glow_id = format!(
        "theme-pumpkin-glow-{}",
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    );
    view! {
        <div class="absolute inset-0 z-0 pointer-events-none overflow-hidden" aria-hidden="true">
            <div class="absolute inset-0 bg-gradient-to-b from-ground-top to-ground-bottom"/>
            {move || {
                let t = theme.get();
                resolved_image_url(&t).map(|url| view! { <div class="absolute inset-0 bg-cover bg-center" style=image_style(&url, &t, quiet)/> })
            }}
            {move || theme.get().background.glows.into_iter().map(|g| view! {
                <div class="absolute rounded-full -translate-x-1/2 -translate-y-1/2" style=format!("left:{}%;top:{}%;width:{}cqw;height:{}cqw;opacity:{};background:radial-gradient(circle, {} 0%, transparent 70%)", g.x, g.y, g.size, g.size, g.opacity, g.color)/>
            }).collect_view()}
            // Only internally generated SVG/HTML is inserted here, never pack text or asset URLs.
            <div class="absolute inset-0" inner_html=move || ornament_markup(&theme.get(), quiet, &glow_id)/>
        </div>
    }
}

#[component]
pub fn ThemeDecorations(
    #[prop(optional)] quiet: bool,
    #[prop(default = 1.0)] density: f64,
) -> impl IntoView {
    let theme = crate::theme::use_theme();
    view! {
        {move || {
            let t = theme.get();
            t.decoration.r#type.map(|_| view! {
                <div class="absolute inset-0 z-30 pointer-events-none overflow-hidden transition-opacity duration-1000" aria-hidden="true" style=if quiet { "opacity:0.15" } else { "opacity:1" }>
                    {decoration_pieces(&t, density).into_iter().map(piece_view).collect_view()}
                </div>
            })
        }}
    }
}

fn ornament_markup(theme: &Theme, quiet: bool, glow_id: &str) -> String {
    if quiet {
        return String::new();
    }
    match theme.background.ornament {
        None => String::new(),
        Some(Ornament::Graveyard) => graveyard(glow_id),
        Some(Ornament::Lights) => lights(),
        Some(Ornament::Skyline) => skyline(),
        Some(Ornament::Waves) => waves(),
        Some(Ornament::Meadow) => meadow(),
    }
}

const FOG_BLOBS: &str = "radial-gradient(ellipse 18% 60% at 10% 80%, rgba(150,110,200,0.5), transparent 70%), radial-gradient(ellipse 22% 50% at 35% 90%, rgba(130,90,180,0.45), transparent 70%), radial-gradient(ellipse 16% 55% at 62% 85%, rgba(160,120,210,0.5), transparent 70%), radial-gradient(ellipse 20% 60% at 85% 88%, rgba(140,100,190,0.45), transparent 70%)";
const WAVE: &str =
    "M0 40 Q100 0 200 40 T400 40 T600 40 T800 40 T1000 40 T1200 40 T1400 40 T1600 40 V120 H0 Z";

fn graveyard(glow_id: &str) -> String {
    let mut out = String::from(
        r##"
<div class="absolute right-[5cqw] top-[8cqh] w-[15cqh] h-[15cqh] rounded-full" style="background:radial-gradient(circle at 38% 36%, #fff6dc 0%, #ffd889 45%, #e6a14a 100%);box-shadow:0 0 10cqh rgba(255,190,100,0.35),0 0 30cqh rgba(255,140,40,0.15);opacity:0.85">
  <div class="absolute left-[22%] top-[52%] w-[18%] h-[18%] rounded-full bg-[#d8913f]/40"></div>
  <div class="absolute left-[58%] top-[30%] w-[12%] h-[12%] rounded-full bg-[#d8913f]/35"></div>
  <div class="absolute left-[55%] top-[64%] w-[9%] h-[9%] rounded-full bg-[#d8913f]/30"></div>
</div>
"##,
    );
    out.push_str(&cobweb());
    out.push_str(r#"<div class="absolute inset-x-0 bottom-0 h-[30cqh]" style="background:linear-gradient(to top, rgba(120,80,170,0.22), transparent)"></div>"#);
    for (time, opacity, delay) in [(70, 0.35, 0), (110, 0.25, -40)] {
        let _ = write!(
            out,
            r#"<div class="theme-anim absolute bottom-0 left-0 w-[200%] h-[28cqh]" style="background-image:{FOG_BLOBS};background-size:50% 100%;opacity:{opacity};filter:blur(2cqh);animation:theme-drift {time}s linear {delay}s infinite"></div>"#
        );
    }
    let _ = write!(
        out,
        r##"<svg class="absolute inset-x-0 bottom-0 w-full h-[30cqh]" viewBox="0 0 1600 300" preserveAspectRatio="xMidYMax slice"><defs><radialGradient id="{glow_id}"><stop offset="0%" stop-color="#ffb347" stop-opacity="0.55"/><stop offset="100%" stop-color="#ff7a1a" stop-opacity="0"/></radialGradient></defs>
<path d="M0 300 L0 222 Q180 182 400 210 Q620 238 820 205 Q1050 170 1260 208 Q1430 236 1600 196 L1600 300 Z" fill="#12091e"/>"##
    );
    for (x, y, w, h, tilt, cross) in [
        (290., 214., 34., 50., -6., false),
        (352., 220., 30., 58., 4., true),
        (420., 216., 26., 38., 8., false),
        (1130., 196., 36., 54., 3., false),
        (1196., 204., 28., 62., -5., true),
        (1300., 218., 30., 42., -10., false),
    ] {
        let d = if cross {
            format!(
                "M{} {} h{} v{} h{} v{} h{} v{} h{} v{} h{} v{} h{} Z",
                -w / 6.,
                -h,
                w / 3.,
                h / 4.,
                w / 3.,
                w / 3.,
                -w / 3.,
                h * 0.75 - w / 3.,
                -w / 3.,
                -(h * 0.75 - w / 3.),
                -w / 3.,
                -w / 3.,
                w / 3.
            )
        } else {
            format!(
                "M{} 0 V{} A{} {} 0 0 1 {} {} V0 Z",
                -w / 2.,
                -h + w / 2.,
                w / 2.,
                w / 2.,
                w / 2.,
                -h + w / 2.
            )
        };
        let _ = write!(
            out,
            r##"<g transform="translate({x} {y}) rotate({tilt})" fill="#0e0718"><path d="{d}"/></g>"##
        );
    }
    out.push_str(r##"
<path d="M0 300 L0 240 Q260 212 520 236 Q800 262 1080 230 Q1340 204 1600 242 L1600 300 Z" fill="#06020b"/>
<g stroke="#06020b" stroke-linecap="round" fill="none">
<path d="M1462 250 C1458 214 1450 180 1438 120" stroke-width="18"/>
<path d="M1446 170 C1420 150 1392 146 1366 120 M1442 140 C1470 118 1500 112 1526 86 M1438 124 C1430 104 1432 92 1420 78" stroke-width="8"/>
<path d="M1366 120 L1344 112 M1380 134 L1366 150 M1526 86 L1548 84 M1500 108 L1512 126 M1420 78 L1408 70 M1420 78 L1428 66" stroke-width="4"/></g>"##);
    for (x, y, scale, delay) in [
        (205, 232, 1., 0.),
        (640, 244, 0.7, -1.3),
        (1010, 230, 0.9, -2.1),
    ] {
        let _ = write!(
            out,
            r##"<g transform="translate({x} {y}) scale({scale})">
<circle r="70" fill="url(#{glow_id})" class="theme-anim" style="animation:theme-flicker 3.1s ease-in-out {delay}s infinite"/>
<rect x="-4" y="-38" width="8" height="14" rx="3" fill="#3f5a1f" transform="rotate(12)"/>
<ellipse cx="-15" cy="0" rx="20" ry="25" fill="#c4560a"/><ellipse cx="15" cy="0" rx="20" ry="25" fill="#c4560a"/><ellipse cx="0" cy="0" rx="21" ry="27" fill="#e0680f"/>
<path d="M-8 -24 Q-12 0 -8 25 M8 -24 Q12 0 8 25" stroke="#a9470a" stroke-width="2" fill="none"/>
<g fill="#ffd257" class="theme-anim" style="animation:theme-flicker 3.1s ease-in-out {delay}s infinite">
<path d="M-17 -8 L-7 -8 L-12 -17 Z M7 -8 L17 -8 L12 -17 Z M-3 -1 L3 -1 L0 -6 Z"/>
<path d="M-19 6 L-13 13 L-8 8 L-3 14 L3 8 L8 14 L13 8 L19 6 Q0 24 -19 6 Z"/></g></g>"##
        );
    }
    out.push_str("</svg>");
    out
}

fn cobweb() -> String {
    let spokes: Vec<_> = [0., 18., 36., 54., 72., 90.].map(f64::to_radians).into();
    let at = |r: f64, a: f64| (r * a.cos(), r * a.sin());
    let mut out = String::from(
        r#"<svg class="absolute left-0 top-0 w-[24cqh] h-[24cqh]" viewBox="0 0 200 200"><g stroke="rgba(230,220,255,0.22)" stroke-width="1.2" fill="none">"#,
    );
    for &a in &spokes {
        let (x, y) = at(200., a);
        let _ = write!(out, r#"<line x1="0" y1="0" x2="{x}" y2="{y}"/>"#);
    }
    for r in [45., 90., 135., 180.] {
        let mut d = format!("M{r} 0 ");
        for pair in spokes.windows(2) {
            let (x, y) = at(r, pair[1]);
            let (cx, cy) = at(r * 0.82, (pair[0] + pair[1]) / 2.);
            let _ = write!(d, "Q{cx} {cy} {x} {y} ");
        }
        let _ = write!(out, r#"<path d="{d}"/>"#);
    }
    out.push_str(r##"</g><g class="theme-anim" style="animation:theme-bob 4s ease-in-out infinite">
<line x1="120" y1="40" x2="120" y2="118" stroke="rgba(230,220,255,0.3)" stroke-width="1"/>
<g transform="translate(120 124)" stroke="#05020a" stroke-width="2" fill="none">
<path d="M-4 -2 L-14 -10 M-4 0 L-16 -2 M-4 2 L-15 7 M-3 4 L-12 13 M4 -2 L14 -10 M4 0 L16 -2 M4 2 L15 7 M3 4 L12 13"/>
<ellipse rx="6" ry="7" fill="#05020a"/><circle cy="-8" r="4" fill="#05020a"/>
<circle cx="-1.5" cy="-9" r="0.9" fill="#ff7a1a" stroke="none"/><circle cx="1.5" cy="-9" r="0.9" fill="#ff7a1a" stroke="none"/></g></g></svg>"##);
    out
}

#[derive(Debug, PartialEq)]
struct Bulb {
    x: f64,
    y: f64,
    color: &'static str,
    delay: f64,
}
fn light_bulbs() -> Vec<Bulb> {
    (0..9)
        .flat_map(|s| {
            (1..=3).map(move |k| {
                let t = k as f64 / 4.;
                let i = s * 3 + k - 1;
                Bulb {
                    x: (s as f64 + t) / 9. * 100.,
                    y: 0.6 + 3.2 * (std::f64::consts::PI * t).sin(),
                    color: BULBS[i % BULBS.len()],
                    delay: (i as f64 * 0.37) % 2.4,
                }
            })
        })
        .collect()
}
fn lights() -> String {
    let mut wire = String::from("M0,6 ");
    for s in 0..9 {
        let x0 = s as f64 / 9. * 1000.;
        let x1 = (s + 1) as f64 / 9. * 1000.;
        let _ = write!(wire, "Q{},70 {x1},6 ", (x0 + x1) / 2.);
    }
    let mut out = format!(
        r##"<div class="absolute inset-x-0 top-0 h-[7cqh]"><svg class="absolute inset-0 w-full h-full" viewBox="0 0 1000 70" preserveAspectRatio="none"><path d="{wire}" fill="none" stroke="#0d3322" stroke-width="2" vector-effect="non-scaling-stroke"/></svg>"##
    );
    for Bulb { x, y, color, delay } in light_bulbs() {
        let _ = write!(
            out,
            r#"<span class="theme-anim absolute w-[0.9cqh] h-[1.4cqh] rounded-full -translate-x-1/2" style="left:{x}%;top:{y}cqh;background:{color};box-shadow:0 0 1.6cqh {color};animation:theme-twinkle 2.4s ease-in-out {delay}s infinite"></span>"#
        );
    }
    out.push_str("</div>");
    out
}

#[derive(Debug, PartialEq)]
struct Building {
    x: f64,
    w: f64,
    h: f64,
    spire: bool,
}
#[derive(Debug, PartialEq)]
struct Window {
    x: f64,
    y: f64,
    flicker: bool,
    delay: f64,
}
#[derive(Debug, PartialEq)]
struct City {
    back: Vec<Building>,
    front: Vec<Building>,
    windows: Vec<Window>,
}
fn city() -> City {
    let mut rng = Seeded(41);
    let mut row = |min: f64, max: f64| {
        let mut list = vec![];
        let mut x = 0.;
        while x < 1600. {
            let w = 40. + rng.next() * 70.;
            let h = min + rng.next() * (max - min);
            let spire = rng.next() < 0.15;
            list.push(Building { x, w, h, spire });
            x += w + rng.next() * 6.;
        }
        list
    };
    let back = row(90., 210.);
    let front = row(50., 150.);
    let mut windows = vec![];
    for b in &front {
        for c in 0..((b.w - 10.) / 14.).floor() as usize {
            for r in 0..((b.h - 16.) / 18.).floor() as usize {
                if rng.next() < 0.28 {
                    windows.push(Window {
                        x: b.x + 8. + c as f64 * 14.,
                        y: 300. - b.h + 10. + r as f64 * 18.,
                        flicker: rng.next() < 0.12,
                        delay: -rng.next() * 6.,
                    });
                }
            }
        }
    }
    City {
        back,
        front,
        windows,
    }
}
fn skyline() -> String {
    let city = city();
    let mut out = String::from(
        r#"<svg class="absolute inset-x-0 bottom-0 w-full h-[34cqh]" viewBox="0 0 1600 300" preserveAspectRatio="xMidYMax slice">"#,
    );
    for (row, color) in [(&city.back, "#0e0b1a"), (&city.front, "#05030a")] {
        let mut d = String::new();
        for b in row {
            let _ = write!(d, "M{} 300 V{} h{} V300 Z ", b.x, 300. - b.h, b.w);
            if b.spire {
                let _ = write!(d, "M{} {} v-30 h4 v30 Z ", b.x + b.w / 2. - 2., 300. - b.h);
            }
        }
        let _ = write!(out, r#"<path d="{d}" fill="{color}"/>"#);
    }
    for Window {
        x,
        y,
        flicker,
        delay,
    } in city.windows
    {
        let anim = if flicker {
            format!(
                r#"class="theme-anim" style="animation:theme-twinkle 3.5s ease-in-out {delay}s infinite""#
            )
        } else {
            String::new()
        };
        let _ = write!(
            out,
            r##"<rect x="{x}" y="{y}" width="6" height="9" fill="#ffd27a" opacity="0.75" {anim}/>"##
        );
    }
    out.push_str("</svg>");
    out
}
fn waves() -> String {
    let mut out =
        String::from(r#"<div class="absolute inset-x-0 bottom-0 h-[24cqh] overflow-hidden">"#);
    for (y, time, opacity, shade) in [(0, 26, 0.25, 400), (30, 18, 0.35, 600), (58, 12, 0.55, 800)]
    {
        let _ = write!(
            out,
            r#"<svg viewBox="0 0 1600 120" preserveAspectRatio="none" class="theme-anim absolute left-0 w-[200%] h-full" style="top:{y}%;opacity:{opacity};animation:theme-drift {time}s linear infinite"><path d="{WAVE}" style="fill:var(--color-primary-{shade})" transform="scale(0.5 1)"/><path d="{WAVE}" style="fill:var(--color-primary-{shade})" transform="translate(800 0) scale(0.5 1)"/></svg>"#
        );
    }
    out.push_str("</div>");
    out
}

#[derive(Debug, PartialEq)]
struct Blade {
    x: f64,
    h: f64,
    lean: f64,
    back: bool,
}
#[derive(Debug, PartialEq)]
struct Flower {
    x: f64,
    y: f64,
    r: f64,
    color: &'static str,
}
fn meadow_geometry() -> (Vec<Blade>, Vec<Flower>) {
    let mut rng = Seeded(53);
    let blades = (0..160)
        .map(|i| Blade {
            x: i as f64 / 160. * 1600. + rng.next() * 10.,
            h: 30. + rng.next() * 60.,
            lean: (rng.next() - 0.5) * 30.,
            back: rng.next() < 0.5,
        })
        .collect();
    let flowers = (0..26)
        .map(|_| Flower {
            x: 20. + rng.next() * 1560.,
            y: 238. + rng.next() * 40.,
            r: 5. + rng.next() * 5.,
            color: FLOWERS[(rng.next() * FLOWERS.len() as f64).floor() as usize],
        })
        .collect();
    (blades, flowers)
}
fn grass(blades: &[Blade], back: bool) -> String {
    let mut d = String::new();
    for b in blades.iter().filter(|b| b.back == back) {
        let _ = write!(
            d,
            "M{} 300 Q{} {} {} {} Q{} {} {} 300 Z ",
            b.x - 4.,
            b.x + b.lean / 2.,
            300. - b.h / 2.,
            b.x + b.lean,
            300. - b.h - if back { 20. } else { 0. },
            b.x + 2.,
            300. - b.h / 2.,
            b.x + 4.
        );
    }
    d
}
fn meadow() -> String {
    let (blades, flowers) = meadow_geometry();
    let mut out = format!(
        r##"<svg class="absolute inset-x-0 bottom-0 w-full h-[26cqh]" viewBox="0 0 1600 300" preserveAspectRatio="xMidYMax slice"><path d="M0 300 L0 250 Q400 228 800 248 Q1200 266 1600 244 L1600 300 Z" fill="#071a0d"/><g class="theme-anim" style="transform-origin:50% 100%;animation:theme-grass 7s ease-in-out infinite"><path d="{}" fill="#0c2a15"/></g>"##,
        grass(&blades, true)
    );
    for Flower { x, y, r, color } in flowers {
        let _ = write!(out, r#"<g transform="translate({x} {y})">"#);
        for a in [0, 72, 144, 216, 288] {
            let _ = write!(
                out,
                r#"<ellipse cx="0" cy="{}" rx="{}" ry="{r}" fill="{color}" opacity="0.85" transform="rotate({a})"/>"#,
                -r,
                r * 0.6
            );
        }
        let _ = write!(out, r##"<circle r="{}" fill="#facc15"/></g>"##, r * 0.45);
    }
    let _ = write!(
        out,
        r##"<g class="theme-anim" style="transform-origin:50% 100%;animation:theme-grass 5s ease-in-out -2s infinite"><path d="{}" fill="#123d1f"/></g></svg>"##,
        grass(&blades, false)
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use quiz_core::theme::builtin::{theme_by_id, themes};

    fn with_decoration(decoration: Decoration) -> Theme {
        let mut theme = theme_by_id("default").clone();
        theme.decoration.r#type = Some(decoration);
        theme
    }

    #[test]
    fn every_builtin_theme_has_stable_finite_pieces_and_quiet_backdrops() {
        for theme in themes() {
            for density in [0., 0.5, 1., 2., 3.] {
                let pieces = decoration_pieces(theme, density);
                assert_eq!(pieces, decoration_pieces(theme, density), "{}", theme.id);
                for p in pieces {
                    assert!(p.left.is_finite() && (0. ..=100.).contains(&p.left));
                    assert!(p.size > 0. && p.duration > 0. && p.delay <= 0.);
                    assert!(p.drift.is_finite() && p.secondary.is_finite());
                }
            }
            assert_eq!(
                ornament_markup(theme, false, "test"),
                ornament_markup(theme, false, "test")
            );
            assert!(ornament_markup(theme, true, "test").is_empty());
            assert!(resolved_image_url(theme).is_none());
        }
        assert!(decoration_pieces(theme_by_id("default"), 1.).is_empty());
    }

    #[test]
    fn counts_follow_reference_rounding_and_fractional_array_lengths() {
        for (decoration, normal, half, double) in [
            (Decoration::Snow, 70, 35, 140),
            (Decoration::Bats, 23, 12, 46), // 8 embers + round(3.5) bats
            (Decoration::Leaves, 30, 15, 60),
            (Decoration::Petals, 34, 17, 68),
            (Decoration::Confetti, 90, 45, 180),
            (Decoration::Hearts, 16, 8, 32),
            (Decoration::Bubbles, 22, 11, 44),
            (Decoration::Stars, 82, 42, 162), // two shooting stars at every density
        ] {
            let theme = with_decoration(decoration);
            assert_eq!(decoration_pieces(&theme, 1.).len(), normal);
            assert_eq!(decoration_pieces(&theme, 0.5).len(), half);
            assert_eq!(decoration_pieces(&theme, 2.).len(), double);
            assert_eq!(decoration_pieces(&theme, 1.), decoration_pieces(&theme, 1.));
        }
        let leaves = with_decoration(Decoration::Leaves);
        assert_eq!(decoration_pieces(&leaves, 0.25).len(), 7);
        assert_eq!(
            decoration_pieces(&with_decoration(Decoration::Snow), 0.25).len(),
            18
        );
    }

    #[test]
    fn seeded_values_match_the_react_generator() {
        let mut rng = Seeded(7);
        assert!((rng.next() - 0.000054784119180202595).abs() < 1e-15);
        let snow = decoration_pieces(&with_decoration(Decoration::Snow), 1.);
        assert!((snow[0].left - 92.07645169652669).abs() < 1e-10);
        assert!((snow[0].size - 0.35004930570726217).abs() < 1e-12);
        assert_eq!(snow[0].blur, 0.15);
        let bats = decoration_pieces(&with_decoration(Decoration::Bats), 1.);
        assert_eq!(bats[0].kind, Kind::Ember);
        assert_eq!(bats[16].kind, Kind::Bat);
        assert!((bats[16].top - 6.004069693390345).abs() < 1e-10);
    }

    #[test]
    fn storm_snow_is_larger_brighter_and_faster_and_falling_shapes_enlarge() {
        let theme = with_decoration(Decoration::Snow);
        let calm = decoration_pieces(&theme, 1.);
        let storm = decoration_pieces(&theme, 2.);
        for (a, b) in calm.iter().zip(&storm) {
            assert!(b.size > a.size && b.opacity >= a.opacity && b.duration < a.duration);
            assert_eq!(a.left, b.left);
            assert_eq!(a.delay, b.delay);
            assert_eq!(b.blur, 0.);
        }
        for decoration in [Decoration::Leaves, Decoration::Petals, Decoration::Confetti] {
            let t = with_decoration(decoration);
            for (a, b) in decoration_pieces(&t, 1.)
                .iter()
                .zip(decoration_pieces(&t, 2.))
            {
                assert!((b.size - a.size * 1.4).abs() < 1e-12);
                assert_eq!(a.left, b.left);
                assert_eq!(a.duration, b.duration);
                assert_eq!(a.delay, b.delay);
            }
        }
    }

    #[test]
    fn ornament_geometry_is_deterministic_and_has_reference_counts() {
        let bulbs = light_bulbs();
        assert_eq!(bulbs.len(), 27);
        assert_eq!(bulbs[0].color, "#ff3b3b");
        assert!((bulbs[1].y - 3.8).abs() < 1e-12);
        assert_eq!(bulbs, light_bulbs());
        let city = city();
        assert_eq!(city, super::city());
        for row in [&city.back, &city.front] {
            assert_eq!(row[0].x, 0.);
            assert!(row.last().unwrap().x < 1600.);
            assert!(row.last().unwrap().x + row.last().unwrap().w >= 1594.);
            assert!(row.windows(2).all(|p| p[1].x >= p[0].x + p[0].w));
        }
        assert!(!city.windows.is_empty());
        assert!(city.windows.iter().all(|w| w.x >= 0.
            && w.y >= 100.
            && w.y < 300.
            && w.delay >= -6.
            && w.delay <= 0.));
        let (blades, flowers) = meadow_geometry();
        assert_eq!(blades.len(), 160);
        assert_eq!(flowers.len(), 26);
        assert_eq!((blades, flowers), meadow_geometry());
        let mut theme = theme_by_id("default").clone();
        for ornament in [
            Ornament::Graveyard,
            Ornament::Lights,
            Ornament::Skyline,
            Ornament::Waves,
            Ornament::Meadow,
        ] {
            theme.background.ornament = Some(ornament);
            let html = ornament_markup(&theme, false, "test-glow");
            assert!(!html.is_empty());
            assert!(!html.contains("NaN") && !html.contains("Infinity"));
            assert!(ornament_markup(&theme, true, "test-glow").is_empty());
        }
        let a = graveyard("scene-a");
        let b = graveyard("scene-b");
        assert!(a.contains("id=\"scene-a\"") && a.contains("url(#scene-a)"));
        assert!(b.contains("id=\"scene-b\"") && !b.contains("scene-a"));
    }

    #[test]
    fn invalid_densities_are_bounded() {
        for density in [f64::NAN, f64::INFINITY, -1., 0.] {
            assert_eq!(piece_count(70, density, true), 0);
        }
        assert_eq!(piece_count(90, f64::MAX, false), 10_000);
    }

    #[test]
    fn background_image_style_dims_quiet_scenes_and_escapes_urls() {
        let mut theme = theme_by_id("default").clone();
        assert_eq!(
            image_style("blob:example", &theme, true),
            "background-image:url(\"blob:example\");opacity:0.25"
        );
        theme.background.image_blur = 4.;
        let style = image_style("a\"b\\c\n", &theme, false);
        assert!(style.contains("a\\\"b\\\\c\\a "));
        assert!(style.contains("opacity:0.5;filter:blur(4px);transform:scale(1.05)"));
    }
}
