mod completed_rounds;
pub mod dashboard;
pub mod game;
mod game_steps;
pub mod leaderboard;
pub mod pause;
pub mod prizes;
pub mod rules;
pub mod welcome;
pub const DEFAULT_RULES: [&str; 4] = [
    "Work together as a team.",
    "Write down your answers.",
    "Do not use phones to look up answers.",
    "The host has the final decision.",
];
pub fn default_rules() -> Vec<String> {
    DEFAULT_RULES.iter().map(|s| (*s).into()).collect()
}

mod icons;

mod team_avatar;
mod team_stats;

mod sort_items;

pub mod backdrop;
mod celebration;
mod dashboard_layouts;
mod team_wall;
mod round_intro;
pub mod venue_map;
