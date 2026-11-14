//! Perspective floor map shared by the presenter and attendance editor.
use leptos::{prelude::*, svg};
use quiz_core::quiz::model::{Seat, Team, team_hex};
use wasm_bindgen::{JsCast, closure::Closure};

const C: f64 = 2.5;
const WD: f64 = 400.;
const HD: f64 = 210.;
const HD_BACK: f64 = 95.;
const Y0: f64 = 34. + HD * C / (1. + C);
fn scale_at(z: f64) -> f64 {
    C / (z + C)
}
fn project(x: f64, z: f64) -> (f64, f64) {
    (
        x * WD * scale_at(z),
        Y0 - z * if z < 0. { HD_BACK } else { HD } * scale_at(z),
    )
}
fn seat_point(angle: f64, dist: f64) -> (f64, f64) {
    project(
        dist * angle.to_radians().sin(),
        dist * angle.to_radians().cos(),
    )
}
fn seat_at(sx: f64, sy: f64, half: f64) -> Seat {
    let zp = ((Y0 - sy) / if sy > Y0 { HD_BACK } else { HD }).min(C * 0.9);
    let z = zp * C / (C - zp);
    let x = sx / (WD * scale_at(z));
    Seat {
        angle: x.atan2(z).to_degrees().clamp(-half, half).round(),
        dist: (x.hypot(z).clamp(0.18, 1.) * 100.).round() / 100.,
    }
}
/// Invert SVG's default xMidYMid/meet transform, including any letterboxing.
fn viewport_seat(
    x: f64,
    y: f64,
    box_size: (f64, f64),
    view_size: (f64, f64),
    half: f64,
) -> Option<Seat> {
    let (w, h) = box_size;
    let (vw, vh) = view_size;
    let scale = (w / vw).min(h / vh);
    if !scale.is_finite() || scale <= 0. {
        return None;
    }
    Some(seat_at(
        (x - w / 2.) / scale,
        (y - (h - vh * scale) / 2.) / scale,
        half,
    ))
}

fn arc_path(half: f64, dist: f64) -> String {
    let mut points = vec![];
    let mut a = -half;
    while a <= half + 0.001 {
        let (x, y) = seat_point(a, dist);
        points.push(format!(
            "{} {x:.1} {y:.1}",
            if points.is_empty() { "M" } else { "L" }
        ));
        a += 3.;
    }
    points.join(" ")
}

/// Indices in `teams` identify markers, highlights, the team being placed, and seat callbacks.
/// Keep the vector order stable while dragging; absent teams can retain their index with seat=None.
#[component]
pub fn VenueMap(
    teams: Signal<Vec<Team>>,
    #[prop(default = 180.)] arc: f64,
    #[prop(optional)] on_seat: Option<Callback<(usize, Seat)>>,
    #[prop(optional)] placing: Option<Signal<Option<usize>>>,
    #[prop(optional)] highlight: Option<Signal<Vec<usize>>>,
    #[prop(default = String::new())] you_label: String,
    #[prop(default = false)] fill: bool,
    #[prop(default = 1.)] marker_scale: f64,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let node = NodeRef::<svg::Svg>::new();
    let dragging = RwSignal::new(None::<usize>);
    let box_size = RwSignal::new((0_f64, 0_f64));
    let half = arc / 2.;
    let back = half.to_radians().cos();
    let height = if back < 0. { project(0., back).1 } else { Y0 } + 40. * marker_scale;
    let width = Signal::derive(move || {
        let (w, h) = box_size.get();
        if fill && h > 0. {
            (height * w / h).max(880.)
        } else {
            880.
        }
    });
    Effect::new(move |_| {
        if !fill {
            return;
        }
        let Some(svg) = node.get() else {
            return;
        };
        let callback = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
            if let Ok(entry) = entries.get(0).dyn_into::<web_sys::ResizeObserverEntry>() {
                let rect = entry.content_rect();
                box_size.set((rect.width(), rect.height()));
            }
        });
        if let Ok(observer) = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) {
            observer.observe(svg.unchecked_ref());
            let resources = StoredValue::new_local(Some((observer, callback)));
            on_cleanup(move || {
                resources.update_value(|r| {
                    if let Some((observer, _)) = r.take() {
                        observer.disconnect();
                    }
                })
            });
        }
    });
    let at_pointer = move |event: &web_sys::PointerEvent| {
        let svg = node.get()?;
        let rect = svg
            .unchecked_ref::<web_sys::Element>()
            .get_bounding_client_rect();
        viewport_seat(
            event.client_x() as f64 - rect.left(),
            event.client_y() as f64 - rect.top(),
            (rect.width(), rect.height()),
            (width.get_untracked(), height),
            half,
        )
    };
    let (lx, ly) = seat_point(-half, 1.);
    let (rx, ry) = seat_point(half, 1.);
    let edge = arc_path(half, 1.).replacen('M', "L", 1);
    view! {
        <svg node_ref=node viewBox=move || format!("{} 0 {} {height}",-width.get()/2.,width.get()) class=move || format!("w-full select-none touch-none {} {class}", if placing.is_some_and(|p| p.get().is_some()) {"cursor-crosshair"} else {""})
            on:pointermove=move |e| {if let (Some(cb),Some(index),Some(seat)) = (on_seat,dragging.get_untracked(),at_pointer(&e)) {cb.run((index,seat));}}
            on:pointerup=move |_| dragging.set(None)
            on:pointercancel=move |_| dragging.set(None)
            on:pointerleave=move |_| dragging.set(None)
            on:pointerdown=move |e| {if let (Some(cb),Some(index),Some(seat)) = (on_seat,placing.and_then(|p| p.get_untracked()),at_pointer(&e)) {cb.run((index,seat));}}>
            <path d=format!("M 0 {Y0} L {lx} {ly} {edge} L {rx} {ry} Z") fill="rgba(255,255,255,0.04)" stroke="rgba(255,255,255,0.18)"/>
            <path d=arc_path(half,0.55) fill="none" stroke="rgba(255,255,255,0.07)"/>
            {(-((half/45.).floor() as i32)..=((half/45.).floor() as i32)).map(|i| {let (x,y)=seat_point(i as f64*45.,1.); view! {<line x1="0" y1=Y0 x2=x y2=y stroke="rgba(255,255,255,0.09)" stroke-dasharray=if i==0 {"none"} else {"6 6"}/>}}).collect_view()}
            <circle cx="0" cy=Y0 r="10" fill="#eab308"/>
            <path d=format!("M -7 {} L 0 {} L 7 {} Z",Y0-13.,Y0-24.,Y0-13.) fill="#eab308"/>
            <text x="18" y=Y0+5. font-size="14" fill="#eab308" font-weight="bold">{you_label}</text>
            {move || {
                let mut seated: Vec<_> = teams.get().into_iter().enumerate().filter(|(_,t)| t.seat.is_some()).collect();
                seated.sort_by(|(_,a),(_,b)| {let z=|t: &Team| {let s=t.seat.as_ref().unwrap(); s.dist*s.angle.to_radians().cos()}; z(b).total_cmp(&z(a))});
                seated.into_iter().map(|(i,team)| {
                    let seat=team.seat.as_ref().unwrap();
                    let depth=scale_at(seat.dist*seat.angle.to_radians().cos()).min(1.3)*marker_scale;
                    let (x,y)=seat_point(seat.angle,seat.dist);
                    let focused=highlight.is_some_and(|h| h.get().contains(&i));
                    let r=if focused {17.} else {13.}*depth;
                    let label=if team.name.chars().count()>16 {format!("{}…",team.name.chars().take(15).collect::<String>())} else {team.name.clone()};
                    view! {<g transform=format!("translate({x} {y})") opacity=if highlight.is_some() && !focused {0.25} else {1.}
                        class=if on_seat.is_some() {"cursor-grab"} else {""}
                        on:pointerdown=move |e| {if on_seat.is_some() {e.stop_propagation(); if let Some(svg)=node.get() {let _=svg.unchecked_ref::<web_sys::Element>().set_pointer_capture(e.pointer_id());} dragging.set(Some(i));}}>
                        <ellipse cy=r*0.35 rx=r ry=r*0.35 fill="rgba(0,0,0,0.5)"/>
                        <circle r=r fill=team_hex(&team,i) stroke=if focused || placing.is_some_and(|p| p.get()==Some(i)) {"#facc15"} else {"rgba(255,255,255,0.6)"} stroke-width=if focused {3.} else {1.5}/>
                        <text y=r+15.*depth text-anchor="middle" font-size=if focused {15.} else {13.}*depth font-weight="bold" fill="white" stroke="#050505" stroke-width="3" paint-order="stroke">{label}</text>
                    </g>}
                }).collect_view()
            }}
        </svg>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pointer_coordinates_account_for_letterboxing_and_empty_boxes() {
        let height = Y0 + 40.;
        let (x, y) = seat_point(45., 0.55);
        for (w, h) in [
            (880_f64, height),
            (1760_f64, height),
            (440_f64, height * 2.),
        ] {
            let scale = (w / 880.).min(h / height);
            let seat = viewport_seat(
                w / 2. + x * scale,
                (h - height * scale) / 2. + y * scale,
                (w, h),
                (880., height),
                90.,
            );
            assert_eq!(
                seat,
                Some(Seat {
                    angle: 45.,
                    dist: 0.55
                })
            );
        }
        assert!(viewport_seat(0., 0., (0., 0.), (880., height), 90.).is_none());
    }

    #[test]
    fn projection_round_trips_front_and_back() {
        for angle in [-135., -90., -45., 0., 45., 90., 135.] {
            for dist in [0.18, 0.55, 1.] {
                let (x, y) = seat_point(angle, dist);
                let seat = seat_at(x, y, 135.);
                assert_eq!(seat, Seat { angle, dist });
            }
        }
    }
    #[test]
    fn placement_clamps_floor_and_angle() {
        assert_eq!(seat_at(0., Y0, 90.).dist, 0.18);
        let seat = seat_at(10000., Y0 + 10000., 90.);
        assert_eq!(seat.dist, 1.);
        assert_eq!(seat.angle, 90.);
        assert!(arc_path(90., 1.).starts_with("M -400.0"));
    }
}
