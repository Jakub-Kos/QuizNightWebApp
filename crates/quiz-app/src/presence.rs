//! Animation-event driven mounting and keyed scene overlap.
use leptos::prelude::*;
use std::time::Duration;

/// Pure lifecycle; reversing an exit preserves the mounted subtree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Entering,
    Present,
    Exiting,
    Absent,
}
impl Phase {
    pub fn visibility(self, shown: bool) -> Self {
        match (self, shown) {
            (Self::Absent | Self::Exiting, true) => Self::Entering,
            (Self::Entering | Self::Present, false) => Self::Exiting,
            _ => self,
        }
    }
    pub fn finished(self) -> Self {
        match self {
            Self::Entering => Self::Present,
            Self::Exiting => Self::Absent,
            _ => self,
        }
    }
}

/// Mounts children inside a div. `enter`/`exit` animate that div. Descendant
/// animation events are ignored. CSS duration and delay determine a safety deadline;
/// missing or disabled animations finish promptly. Reversals retain child state.
#[component]
pub fn Presence(
    #[prop(into)] when: Signal<bool>,
    #[prop(into)] enter: String,
    #[prop(into)] exit: String,
    children: ChildrenFn,
    #[prop(optional)] on_exit: Option<Callback<()>>,
    /// Extra classes for the wrapper, e.g. to position it (`absolute inset-0 z-50`).
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    let phase = RwSignal::new(Phase::Absent);
    let node = NodeRef::<leptos::html::Div>::new();
    let timer = StoredValue::new(None::<leptos::leptos_dom::helpers::TimeoutHandle>);
    let enter = StoredValue::new(enter);
    let exit = StoredValue::new(exit);
    let class = StoredValue::new(class);
    let finish = move || {
        let old = phase.get_untracked();
        phase.set(old.finished());
        if old == Phase::Exiting {
            if let Some(cb) = on_exit {
                cb.run(());
            }
        }
    };
    Effect::new(move |_| {
        let shown = when.get();
        phase.update(|p| *p = p.visibility(shown));
    });
    Effect::new(move |_| {
        let current = phase.get();
        let element = node.get();
        if let Some(h) = timer.get_value() {
            h.clear();
        }
        if !matches!(current, Phase::Entering | Phase::Exiting) {
            return;
        }
        if let Some(element) = element {
            let style = window().get_computed_style(&element).ok().flatten();
            let seconds = |name: &str| -> Vec<f64> {
                style
                    .as_ref()
                    .and_then(|s| s.get_property_value(name).ok())
                    .unwrap_or_default()
                    .split(',')
                    .map(|s| {
                        let s = s.trim();
                        if let Some(v) = s.strip_suffix("ms") {
                            v.parse::<f64>().unwrap_or(0.0) / 1000.0
                        } else {
                            s.trim_end_matches('s').parse().unwrap_or(0.0)
                        }
                    })
                    .collect()
            };
            let durations = seconds("animation-duration");
            let delays = seconds("animation-delay");
            let deadline = durations
                .iter()
                .enumerate()
                .map(|(i, d)| d + delays[i % delays.len()])
                .fold(0.0_f64, f64::max);
            let h = leptos::leptos_dom::helpers::set_timeout_with_handle(
                finish,
                Duration::from_secs_f64(deadline.max(0.0) + 0.08),
            )
            .ok();
            timer.set_value(h);
        }
    });
    on_cleanup(move || {
        if let Some(Some(h)) = timer.try_get_value() {
            h.clear();
        }
    });
    view! {
        <Show when=move || phase.get() != Phase::Absent>
            <div node_ref=node class=move || {
                    // "presence" makes the wrapper fill its parent, so a scene's `h-full` still works.
                    let animation = match phase.get() {
                        Phase::Entering => enter.get_value(),
                        Phase::Exiting => exit.get_value(),
                        _ => String::new(),
                    };
                    format!("presence {} {animation}", class.get_value())
                }
                on:animationend=move |event: web_sys::AnimationEvent| {
                    if event.target() == event.current_target() { finish(); }
                }>{children()}</div>
        </Show>
    }
}

/// Keyed scenes (u64 keys), rendered once per mounted instance. In overlap mode
/// old scenes exit alongside the latest scene. `wait` defers the latest key until
/// every outgoing scene has left. Rapid changes coalesce to the latest key.
#[component]
pub fn PresenceSwitch<V: IntoView + 'static>(
    #[prop(into)] scene: Signal<u64>,
    #[prop(into)] wait: Signal<bool>,
    enter: &'static str,
    exit: &'static str,
    render: impl Fn(u64) -> V + Send + Sync + 'static,
) -> impl IntoView {
    let entries = RwSignal::new(Vec::<(u64, u64)>::new());
    let serial = StoredValue::new(0_u64);
    let render = StoredValue::new(render);
    Effect::new(move |_| {
        let key = scene.get();
        let waiting = wait.get();
        let mounted = entries.get();
        if mounted.iter().any(|(_, k)| *k == key) || (waiting && !mounted.is_empty()) {
            return;
        }
        serial.update_value(|s| *s += 1);
        entries.update(|v| v.push((serial.get_value(), key)));
    });
    view! {
        <For each=move || entries.get() key=|(id, _)| *id children=move |(id, key)| {
            view! {
                <Presence when=Signal::derive(move || scene.get() == key)
                    enter=enter exit=exit
                    on_exit=Callback::new(move |_| entries.update(|v| v.retain(|(i,_)| *i != id)))>
                    {render.with_value(|f| f(key)).into_any()}
                </Presence>
            }
        }/>
    }
}

#[cfg(test)]
mod tests {
    use super::Phase::*;
    #[test]
    fn lifecycle_and_reversals() {
        assert_eq!(Absent.visibility(true), Entering);
        assert_eq!(Entering.finished(), Present);
        assert_eq!(Present.visibility(false), Exiting);
        assert_eq!(Entering.visibility(false), Exiting);
        assert_eq!(Exiting.visibility(true), Entering);
        assert_eq!(Exiting.finished(), Absent);
        assert_eq!(Absent.visibility(false), Absent);
    }
}
