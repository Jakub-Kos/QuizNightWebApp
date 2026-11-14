use leptos::prelude::*;
/// The Lucide icons of the question screen, as lucide-react draws them: `size` px, `stroke_width`,
/// an optional (reactive) class on the `<svg>`.
#[component]
pub fn GameIcon(
    name: &'static str,
    size: u32,
    #[prop(default = 2)] stroke_width: u32,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    let shapes = match name {
        "maximize" => view! { <g><path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"/></g> }.into_any(),
        "minimize" => view! { <g><path d="M20 10h-6V4M4 14h6v6M14 10l7-7M3 21l7-7"/></g> }.into_any(),
        "arrow_down_up" => view! { <g>
            <path d="m3 16 4 4 4-4" />
            <path d="M7 20V4" />
            <path d="m21 8-4-4-4 4" />
            <path d="M17 4v16" />
        </g> }.into_any(),
        "arrow_left" => view! { <g>
            <path d="m12 19-7-7 7-7" />
            <path d="M19 12H5" />
        </g> }.into_any(),
        "arrow_right" => view! { <g>
            <path d="M5 12h14" />
            <path d="m12 5 7 7-7 7" />
        </g> }.into_any(),
        "check" => view! { <g>
            <path d="M20 6 9 17l-5-5" />
        </g> }.into_any(),
        "clock" => view! { <g>
            <path d="M12 6v6l4 2" />
            <circle cx="12" cy="12" r="10" />
        </g> }.into_any(),
        "hash" => view! { <g>
            <line x1="4" x2="20" y1="9" y2="9" />
            <line x1="4" x2="20" y1="15" y2="15" />
            <line x1="10" x2="8" y1="3" y2="21" />
            <line x1="16" x2="14" y1="3" y2="21" />
        </g> }.into_any(),
        "list_ordered" => view! { <g>
            <path d="M11 5h10" />
            <path d="M11 12h10" />
            <path d="M11 19h10" />
            <path d="M4 4h1v5" />
            <path d="M4 9h2" />
            <path d="M6.5 20H3.4c0-1 2.6-1.925 2.6-3.5a1.5 1.5 0 0 0-2.6-1.02" />
        </g> }.into_any(),
        // Edit3 in lucide-react
        "pen_line" => view! { <g>
            <path d="M13 21h8" />
            <path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" />
        </g> }.into_any(),
        "x" => view! { <g>
            <path d="M18 6 6 18" />
            <path d="m6 6 12 12" />
        </g> }.into_any(),
        _ => view! { <g/> }.into_any(),
    };
    view! { <svg width=size height=size class=move || class.get() viewBox="0 0 24 24" fill="none"
    stroke="currentColor" stroke-width=stroke_width stroke-linecap="round" stroke-linejoin="round">{shapes}</svg> }
}

#[component]
pub fn IntroIcon(
    name: &'static str,
    size: u32,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let paths = match name {
"shield" => view! { <g>
            <path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" />
        </g> }.into_any(),
"gavel" => view! { <g>
            <path d="m14 13-8.381 8.38a1 1 0 0 1-3.001-3l8.384-8.381" />
            <path d="m16 16 6-6" />
            <path d="m21.5 10.5-8-8" />
            <path d="m8 8 6-6" />
            <path d="m8.5 7.5 8 8" />
        </g> }.into_any(),
"triangle_alert" => view! { <g>
            <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3" />
            <path d="M12 9v4" />
            <path d="M12 17h.01" />
        </g> }.into_any(),
"check" => view! { <g>
            <path d="M20 6 9 17l-5-5" />
        </g> }.into_any(),
"clock" => view! { <g>
            <circle cx="12" cy="12" r="10" />
            <path d="M12 6v6l4 2" />
        </g> }.into_any(),
"users" => view! { <g>
            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
            <path d="M16 3.128a4 4 0 0 1 0 7.744" />
            <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
            <circle cx="9" cy="7" r="4" />
        </g> }.into_any(),
"star" => view! { <g>
            <path d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z" />
        </g> }.into_any(),
"info" => view! { <g>
            <circle cx="12" cy="12" r="10" />
            <path d="M12 16v-4" />
            <path d="M12 8h.01" />
        </g> }.into_any(),
"arrow_right" => view! { <g>
            <path d="M5 12h14" />
            <path d="m12 5 7 7-7 7" />
        </g> }.into_any(),
"gift" => view! { <g>
            <path d="M12 7v14" />
            <path d="M20 11v8a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-8" />
            <path d="M7.5 7a1 1 0 0 1 0-5A4.8 8 0 0 1 12 7a4.8 8 0 0 1 4.5-5 1 1 0 0 1 0 5" />
            <rect x="3" y="7" width="18" height="4" rx="1" />
        </g> }.into_any(),
"trophy" => view! { <g>
            <path d="M10 14.66V17a1 1 0 0 1-1 1 2 2 0 0 0-2 2v2" />
            <path d="M14 14.66V17a1 1 0 0 0 1 1 2 2 0 0 1 2 2v2" />
            <path d="M17.916 10H19.5A2.5 2.5 0 0 0 22 7.5V5a1 1 0 0 0-1-1h-3" />
            <path d="M4 22h16" />
            <path d="M6 9a6 6 0 0 0 12 0V3a1 1 0 0 0-1-1H7a1 1 0 0 0-1 1z" />
            <path d="M6.084 10H4.5A2.5 2.5 0 0 1 2 7.5V5a1 1 0 0 1 1-1h3" />
        </g> }.into_any(),
"medal" => view! { <g>
            <path d="M7.21 15 2.66 7.14a2 2 0 0 1 .13-2.2L4.4 2.8A2 2 0 0 1 6 2h12a2 2 0 0 1 1.6.8l1.6 2.14a2 2 0 0 1 .14 2.2L16.79 15" />
            <path d="M11 12 5.12 2.2" />
            <path d="m13 12 5.88-9.8" />
            <path d="M8 7h8" />
            <circle cx="12" cy="17" r="5" />
            <path d="M12 18v-2h-.5" />
        </g> }.into_any(),
_ => view! { <g/> }.into_any(),
};
    view! { <svg width=size height=size class=class viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{paths}</svg> }
}

#[component]
pub fn ArrowLeft() -> impl IntoView {
    view! { <svg width="24" height="24" viewBox="0 0 24 24" fill="none"
        stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="m12 19-7-7 7-7" />
            <path d="M19 12H5" />
    </svg> }
}

#[component]
pub fn Settings() -> impl IntoView {
    view! { <svg width="24" height="24" viewBox="0 0 24 24" fill="none"
        stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d=concat!(
                "M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 ",
                "4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 ",
                "2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 ",
                "2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915",
            ) />
            <circle cx="12" cy="12" r="3" />
    </svg> }
}
