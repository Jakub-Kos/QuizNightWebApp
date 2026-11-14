//! Port of TeamAvatar.jsx: a team's photo, or its initials on the team colour when it has none
//! (`initials` from `resolve_teams` are distinct within the quiz). Size and rounding come from
//! `class`; `text_class` sets the size of the initials.
use leptos::prelude::*;
use quiz_core::quiz::model::ResolvedTeam;

/// quiz_core's TEAM_COLORS, repeated literally so that Tailwind generates the team gradients.
const TEAM_COLORS: &[&str] = &[
    "from-purple-600 to-indigo-900",
    "from-red-600 to-pink-900",
    "from-blue-600 to-cyan-800",
    "from-emerald-600 to-teal-900",
    "from-orange-500 to-red-600",
    "from-yellow-500 to-orange-700",
    "from-sky-500 to-blue-700",
    "from-stone-700 to-stone-900",
];

#[component]
pub fn TeamAvatar(
    team: ResolvedTeam,
    #[prop(default = "")] class: &'static str,
    #[prop(default = "text-2xl")] text_class: &'static str,
) -> impl IntoView {
    let background = if team.image.is_some() {
        "bg-white/5".to_string()
    } else {
        format!("bg-gradient-to-br {}", team_color(team.color))
    };
    view! {
        <div class=format!("overflow-hidden shrink-0 flex items-center justify-center {background} {class}")>
            {match team.image {
                Some(image) => view! { <img src=image alt=team.team.name class="w-full h-full object-cover"/> }.into_any(),
                None => view! {
                    <span class=format!("font-black text-white/90 tracking-tight leading-none drop-shadow-[0_2px_8px_rgba(0,0,0,0.4)] {text_class}")>
                        {team.initials}
                    </span>
                }.into_any(),
            }}
        </div>
    }
}

/// The Tailwind gradient stops of a team colour (an index into TEAM_COLORS).
pub fn team_color(index: usize) -> &'static str {
    TEAM_COLORS[index % TEAM_COLORS.len()]
}

#[cfg(test)]
mod tests {
    #[test]
    fn colours_match_core() {
        assert_eq!(super::TEAM_COLORS, quiz_core::quiz::model::TEAM_COLORS);
        assert_eq!(super::team_color(9), "from-red-600 to-pink-900");
    }
}
