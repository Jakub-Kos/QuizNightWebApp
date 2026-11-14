use crate::stage::StageLayout;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

const COLORS: [&str; 6] = [
    "#eab308", "#3b82f6", "#ffffff", "#facc15", "#ff5d8f", "#4ade80",
];
fn default_palette() -> Vec<String> {
    COLORS.iter().map(|color| (*color).into()).collect()
}

pub fn theme_palette(theme: &quiz_core::theme::Theme) -> Vec<String> {
    let mut colors = default_palette();
    for (index, key) in [(0, "accent"), (1, "primary")] {
        if let Some(color) = theme.colors.get(key).filter(|c| !c.is_empty()) {
            colors[index] = color.clone();
        }
    }
    colors
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Rectangle,
    Circle,
    Rocket,
    Spark,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub rotation: f64,
    pub spin: f64,
    pub size: f64,
    pub colour: String,
    pub shape: Shape,
    pub life: f64,
}
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / ((1u64 << 53) as f64)
    }
}
pub struct System {
    pub particles: Vec<Particle>,
    pub width: f64,
    pub height: f64,
    rng: Rng,
    elapsed: f64,
    next_launch: f64,
    fireworks: bool,
    multiplier: usize,
    palette: Vec<String>,
}
impl System {
    pub fn new(width: f64, height: f64, seed: u64) -> Self {
        Self {
            particles: vec![],
            width,
            height,
            rng: Rng::new(seed),
            elapsed: 0.,
            next_launch: 0.,
            fireworks: false,
            multiplier: 1,
            palette: default_palette(),
        }
    }
    pub fn set_palette(&mut self, palette: Vec<String>) {
        self.palette = if palette.is_empty() {
            default_palette()
        } else {
            palette
        };
    }
    pub fn confetti(&mut self, multiplier: usize) {
        for _ in 0..(self.width / 4.).round() as usize * multiplier {
            let x = self.rng.next() * self.width;
            let y = -self.rng.next() * self.height;
            let vx = (self.rng.next() - 0.5) * 180.;
            let colour =
                self.palette[(self.rng.next() * self.palette.len() as f64) as usize].clone();
            self.particles.push(Particle {
                x,
                y,
                vx,
                vy: 60.,
                rotation: self.rng.next() * std::f64::consts::TAU,
                spin: (self.rng.next() - 0.5) * 8.,
                size: 8. + self.rng.next() * 10.,
                colour,
                shape: if self.rng.next() < 0.2 {
                    Shape::Circle
                } else {
                    Shape::Rectangle
                },
                life: 8.,
            });
        }
    }
    pub fn fireworks(&mut self, multiplier: usize) {
        self.fireworks = true;
        self.elapsed = 0.;
        self.next_launch = 0.;
        self.multiplier = multiplier;
    }
    pub fn alive(&self) -> bool {
        !self.particles.is_empty() || self.fireworks
    }
    pub fn step(&mut self, dt: f64) {
        if !dt.is_finite() || dt <= 0. {
            return;
        }
        // After a stall (slow frame, background tab) the show continues where it was instead of
        // fast-forwarding to its end.
        let mut remaining = dt.min(0.1);
        while remaining > 0. {
            let dt = remaining.min(1. / 60.);
            remaining -= dt;
            self.elapsed += dt;
            if self.fireworks && self.elapsed < 6.5 && self.elapsed >= self.next_launch {
                self.next_launch = self.elapsed + 0.25 + self.rng.next() * 0.45;
                for _ in 0..self.multiplier {
                    self.particles.push(Particle {
                        x: self.width * (0.12 + self.rng.next() * 0.76),
                        y: self.height,
                        vx: 0.,
                        vy: -self.height,
                        rotation: 0.,
                        spin: 0.,
                        size: 6.,
                        colour: self.palette
                            [(self.rng.next() * self.palette.len() as f64) as usize]
                            .clone(),
                        shape: Shape::Rocket,
                        life: 0.6 + self.rng.next() * 0.25,
                    });
                }
            }
            if self.elapsed >= 6.5 {
                self.fireworks = false;
            }
            let mut bursts = vec![];
            for p in &mut self.particles {
                p.life -= dt;
                if p.shape == Shape::Rocket && p.life <= 0. {
                    bursts.push((p.x, p.y, p.colour.clone()));
                }
                if p.shape != Shape::Rocket {
                    p.vx *= (-0.9 * dt).exp();
                    p.vy += if p.shape == Shape::Spark {
                        self.height * 0.35 * dt
                    } else {
                        648. * dt
                    };
                }
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                p.rotation += p.spin * dt;
                if p.shape == Shape::Rectangle {
                    p.x += (p.rotation * 2.).sin() * 45. * dt;
                }
            }
            self.particles.retain(|p| {
                p.life > 0. && p.y < self.height + 100. && p.x > -200. && p.x < self.width + 200.
            });
            for (x, y, colour) in bursts {
                let count = 80 + (self.rng.next() * 50.) as usize;
                for k in 0..count {
                    let angle = k as f64 / count as f64 * std::f64::consts::TAU;
                    let speed = self.height * (0.15 + self.rng.next() * 0.3);
                    self.particles.push(Particle {
                        x,
                        y,
                        vx: angle.cos() * speed,
                        vy: angle.sin() * speed,
                        rotation: 0.,
                        spin: 0.,
                        size: 3.5,
                        colour: if self.rng.next() < 0.8 {
                            colour.clone()
                        } else {
                            "#ffffff".into()
                        },
                        shape: Shape::Spark,
                        life: 1.6,
                    });
                }
            }
        }
    }
}

#[component]
pub fn ParticleCanvas(
    active: RwSignal<bool>,
    trigger: RwSignal<u64>,
    fireworks: RwSignal<bool>,
    multiplier: RwSignal<usize>,
    count: RwSignal<usize>,
    frame_ms: RwSignal<f64>,
    #[prop(optional, into)] palette: Option<Signal<Vec<String>>>,
) -> impl IntoView {
    let layout = expect_context::<Signal<StageLayout>>();
    let theme = crate::theme::use_theme();
    let palette = palette.unwrap_or_else(|| Signal::derive(move || theme_palette(&theme.get())));
    let canvas = NodeRef::<leptos::html::Canvas>::new();
    let system = StoredValue::new_local(System::new(2400., 1350., 1));
    let last = StoredValue::new_local(None::<f64>);
    let reduced = window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten();
    let reduced = StoredValue::new_local(reduced);
    Effect::new(move |_| {
        if trigger.get() == 0 {
            return;
        }
        if reduced.with_value(|m| m.as_ref().is_some_and(|m| m.matches())) {
            return;
        }
        let l = layout.get_untracked();
        system.update_value(|s| {
            s.set_palette(palette.get_untracked());
            s.width = l.w;
            s.height = l.h;
            if fireworks.get_untracked() {
                s.fireworks(multiplier.get_untracked());
            } else {
                s.confetti(multiplier.get_untracked());
            }
        });
        last.set_value(None);
        active.set(true);
    });
    // Leptos owns the recursive RAF closure and cancels its handle on owner cleanup.
    let (start, stop) = raf_loop(move |now| {
        let Some(canvas) = canvas.get() else {
            return;
        };
        let Ok(Some(ctx)) = canvas.get_context("2d") else {
            return;
        };
        let Ok(ctx) = ctx.dyn_into::<web_sys::CanvasRenderingContext2d>() else {
            return;
        };
        let l = layout.get_untracked();
        if canvas.width() != l.w as u32 {
            canvas.set_width(l.w as u32);
        }
        if canvas.height() != l.h as u32 {
            canvas.set_height(l.h as u32);
        }
        ctx.clear_rect(0., 0., l.w, l.h);
        let dt = last.with_value(|v| v.map(|v| (now - v) / 1000.).unwrap_or(0.));
        last.set_value(Some(now));
        let begin = window().performance().map(|p| p.now()).unwrap_or(now);
        system.update_value(|s| {
            s.step(dt);
            for p in &s.particles {
                ctx.save();
                ctx.set_global_alpha(p.life.min(1.));
                ctx.set_fill_style_str(&p.colour);
                let _ = ctx.translate(p.x, p.y);
                let _ = ctx.rotate(p.rotation);
                if p.shape == Shape::Spark {
                    let _ = ctx.set_global_composite_operation("lighter");
                    // A soft halo. Canvas shadow blur costs over a second per frame for a few
                    // hundred sparks, which stalls the show.
                    ctx.set_global_alpha(p.life.min(1.) * 0.25);
                    ctx.begin_path();
                    let _ = ctx.arc(0., 0., p.size * 3., 0., std::f64::consts::TAU);
                    ctx.fill();
                    ctx.set_global_alpha(p.life.min(1.));
                    ctx.set_stroke_style_str(&p.colour);
                    ctx.set_line_width(4.);
                    ctx.begin_path();
                    ctx.move_to(-p.vx * 0.025, -p.vy * 0.025);
                    ctx.line_to(0., 0.);
                    ctx.stroke();
                }
                if matches!(p.shape, Shape::Circle | Shape::Spark) {
                    ctx.begin_path();
                    let _ = ctx.arc(0., 0., p.size, 0., std::f64::consts::TAU);
                    ctx.fill();
                } else {
                    ctx.fill_rect(
                        -p.size / 2.,
                        0.,
                        p.size,
                        if p.shape == Shape::Rocket {
                            26.
                        } else {
                            p.size * 0.5
                        },
                    );
                }
                ctx.restore();
            }
            count.set(s.particles.len());
            if !s.alive() {
                active.set(false);
            }
        });
        frame_ms.set(
            window()
                .performance()
                .map(|p| p.now() - begin)
                .unwrap_or(0.),
        );
    });
    Effect::new(move |_| {
        if active.get() {
            start();
        } else {
            stop();
            if let Some(c) = canvas.get() {
                if let Ok(Some(ctx)) = c.get_context("2d") {
                    if let Ok(ctx) = ctx.dyn_into::<web_sys::CanvasRenderingContext2d>() {
                        ctx.clear_rect(0., 0., c.width() as f64, c.height() as f64);
                    }
                }
            }
        }
    });
    view! { <canvas class="particle-canvas" node_ref=canvas width="2400" height="1350"/> }
}
#[cfg(test)]
mod tests {
    use super::*;

    /// Steps frame by frame, as the canvas does: one call advances at most 0.1 s.
    fn run(system: &mut System, seconds: f64) {
        for _ in 0..(seconds * 60.) as usize {
            system.step(1. / 60.);
        }
    }

    #[test]
    fn theme_colors_and_classic_fallback() {
        let mut theme = quiz_core::theme::builtin::theme_by_id("default").clone();
        theme.colors.clear();
        assert_eq!(theme_palette(&theme), default_palette());
        theme.colors.insert("accent".into(), "#123456".into());
        theme.colors.insert("primary".into(), "#abcdef".into());
        let palette = theme_palette(&theme);
        assert_eq!(&palette[..2], &["#123456", "#abcdef"]);
        let mut system = System::new(2400., 1350., 42);
        system.set_palette(vec!["#123456".into()]);
        system.confetti(1);
        assert!(system.particles.iter().all(|p| p.colour == "#123456"));
        system.particles.clear();
        system.fireworks(1);
        system.step(0.01);
        assert!(system.particles.iter().all(|p| p.colour == "#123456"));
        run(&mut system, 1.);
        assert!(
            system
                .particles
                .iter()
                .filter(|p| p.shape == Shape::Spark)
                .all(|p| p.colour == "#123456" || p.colour == "#ffffff")
        );
        system.set_palette(vec![]);
        assert_eq!(system.palette, default_palette());
    }

    #[test]
    fn deterministic_and_finite() {
        let mut a = System::new(2400., 1350., 42);
        let mut b = System::new(2400., 1350., 42);
        a.confetti(1);
        b.confetti(1);
        assert_eq!(a.particles.len(), 600);
        run(&mut a, 0.5);
        run(&mut b, 0.5);
        assert_eq!(a.particles, b.particles);
        assert!(a.particles.iter().all(|p| p.vy > 60.));
        run(&mut a, 30.);
        assert!(!a.alive());
    }
    #[test]
    fn rockets_burst_and_finish() {
        let mut s = System::new(2400., 1350., 7);
        s.fireworks(1);
        run(&mut s, 1.);
        assert!(s.particles.iter().any(|p| p.shape == Shape::Spark));
        run(&mut s, 12.);
        assert!(!s.alive());
    }
    #[test]
    fn a_stall_does_not_fast_forward_the_show() {
        let mut s = System::new(2400., 1350., 7);
        s.fireworks(1);
        s.step(60.);
        assert!(s.alive());
    }
}

fn raf_loop(callback: impl Fn(f64) + 'static) -> (impl Fn() + Copy, impl Fn() + Copy) {
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::closure::Closure;
    type Frame = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;
    let frame: Frame = Rc::new(RefCell::new(None));
    let handle = Rc::new(RefCell::new(None::<i32>));
    let running = Rc::new(RefCell::new(false));
    let weak = Rc::downgrade(&frame);
    let h = handle.clone();
    let r = running.clone();
    *frame.borrow_mut() = Some(Closure::new(move |now| {
        callback(now);
        if *r.borrow() {
            if let Some(f) = weak.upgrade() {
                if let Some(cb) = f.borrow().as_ref() {
                    *h.borrow_mut() = window()
                        .request_animation_frame(cb.as_ref().unchecked_ref())
                        .ok();
                }
            }
        }
    }));
    let state = StoredValue::new_local((frame, handle, running));
    let stop = move || {
        state.with_value(|(_, h, r)| {
            *r.borrow_mut() = false;
            if let Some(id) = h.borrow_mut().take() {
                let _ = window().cancel_animation_frame(id);
            }
        })
    };
    on_cleanup(move || {
        stop();
        state.with_value(|(f, _, _)| {
            f.borrow_mut().take();
        });
    });
    let start = move || {
        state.with_value(|(f, h, r)| {
            if *r.borrow() {
                return;
            }
            *r.borrow_mut() = true;
            if let Some(cb) = f.borrow().as_ref() {
                *h.borrow_mut() = window()
                    .request_animation_frame(cb.as_ref().unchecked_ref())
                    .ok();
            }
        })
    };
    (start, stop)
}

#[cfg(test)]
mod fireworks_show {
    use super::*;

    /// The show launches rockets for its whole length instead of dying after the first bursts.
    #[test]
    fn keeps_launching_until_the_end() {
        let mut system = System::new(2400., 1350., 1);
        system.fireworks(1);
        let mut empty_frames = 0;
        for frame in 0..(6 * 60) {
            system.step(1. / 60.);
            if frame > 30 && system.particles.is_empty() {
                empty_frames += 1;
            }
            assert!(system.alive(), "show ended early at frame {frame}");
        }
        assert!(
            empty_frames < 30,
            "{empty_frames} frames with nothing on screen"
        );
    }
}
