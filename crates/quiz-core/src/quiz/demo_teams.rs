use super::model::Team;
use std::sync::LazyLock;
static TEAMS: LazyLock<Vec<Team>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("demo_teams.json")).expect("static demo teams")
});
pub fn demo_teams() -> Vec<Team> {
    TEAMS.clone()
}
