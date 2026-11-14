use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
pub const DEMO_ID: &str = "demo";
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TeamIcon {
    #[default]
    Brain,
    Rocket,
    Zap,
    Crown,
    Flame,
    Shield,
    Beer,
    Glasses,
    Gavel,
    Microscope,
}
pub const TEAM_COLORS: [&str; 8] = [
    "from-purple-600 to-indigo-900",
    "from-red-600 to-pink-900",
    "from-blue-600 to-cyan-800",
    "from-emerald-600 to-teal-900",
    "from-orange-500 to-red-600",
    "from-yellow-500 to-orange-700",
    "from-sky-500 to-blue-700",
    "from-stone-700 to-stone-900",
];
pub const TEAM_HEX: [&str; 8] = [
    "#9333ea", "#dc2626", "#2563eb", "#059669", "#f97316", "#eab308", "#0ea5e9", "#57534e",
];
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeamColor {
    Palette(usize),
    Other(String),
}
impl Serialize for TeamColor {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(match self {
            Self::Palette(i) => TEAM_COLORS[i % TEAM_COLORS.len()],
            Self::Other(v) => v,
        })
    }
}
impl<'de> Deserialize<'de> for TeamColor {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = String::deserialize(d)?;
        Ok(TEAM_COLORS
            .iter()
            .position(|s| *s == v)
            .map(Self::Palette)
            .unwrap_or(Self::Other(v)))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TeamId {
    Number(i64),
    Text(String),
}
pub fn team_hex(team: &Team, index: usize) -> &'static str {
    TEAM_HEX[team
        .color
        .as_ref()
        .map_or(index % TEAM_COLORS.len(), |c| c.index(index))]
}
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedTeam {
    pub id: TeamId,
    pub icon: TeamIcon,
    pub color: usize,
    pub initials: String,
    pub image: Option<String>,
    pub team: Team,
}
pub fn resolve_teams(
    teams: &[Team],
    resolve_media: impl Fn(&str) -> Option<String>,
) -> Vec<ResolvedTeam> {
    let mut out: Vec<_> = teams
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.name.trim().is_empty())
        .map(|(i, t)| ResolvedTeam {
            id: t.id.clone().unwrap_or(TeamId::Number((i + 1) as i64)),
            icon: t.icon(),
            color: t
                .color
                .as_ref()
                .map_or(i % TEAM_COLORS.len(), |c| c.index(i)),
            initials: String::new(),
            image: t.image.as_deref().and_then(&resolve_media),
            team: t.clone(),
        })
        .collect();
    let names = out.iter().map(|t| t.team.name.clone()).collect::<Vec<_>>();
    for (t, s) in out.iter_mut().zip(super::initials::unique_initials(&names)) {
        t.initials = s;
    }
    out
}
impl Team {
    pub fn icon(&self) -> TeamIcon {
        self.icon
            .as_ref()
            .and_then(|s| serde_json::from_value(json!(s)).ok())
            .unwrap_or_default()
    }
}
/// Stored fields retain their original JSON representation, including extension fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Team {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<TeamId>,
    pub name: String,
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<TeamColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_new: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub present: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_rank: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_rate")]
    pub correct_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank_history: Option<Vec<Rank>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub podium: Option<Podium>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seat: Option<Seat>,
    #[serde(flatten)]
    pub fields: std::collections::BTreeMap<String, Value>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub show_time: bool,
    pub start_time: String,
    pub split_delay: u64,
    pub cycle_duration: u64,
    pub header_interval: u64,
    pub language: String,
    pub welcome_layout: String,
    pub intro_pause: u64,
    #[serde(flatten)]
    pub fields: std::collections::BTreeMap<String, Value>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            show_time: true,
            start_time: "20:00".into(),
            split_delay: 3000,
            cycle_duration: 8000,
            header_interval: 5000,
            language: "cs".into(),
            welcome_layout: "auto".into(),
            intro_pause: 2600,
            fields: Default::default(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Quiz {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub questions_csv: String,
    pub questions_sheet_url: String,
    pub scores_sheet_url: String,
    pub teams: Vec<Team>,
    pub settings: Settings,
    pub rules: Rules,
    pub prizes: Prizes,
    pub manual_scores: std::collections::BTreeMap<String, std::collections::BTreeMap<String, f64>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub builtin: bool,
    pub drive: Option<Value>,
    #[serde(flatten)]
    pub fields: std::collections::BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Rank {
    Number(i64),
    Unknown(String),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Podium {
    pub first: usize,
    pub second: usize,
    pub third: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Seat {
    pub angle: f64,
    pub dist: f64,
}
pub const TEAM_ICONS: [TeamIcon; 10] = [
    TeamIcon::Brain,
    TeamIcon::Rocket,
    TeamIcon::Zap,
    TeamIcon::Crown,
    TeamIcon::Flame,
    TeamIcon::Shield,
    TeamIcon::Beer,
    TeamIcon::Glasses,
    TeamIcon::Gavel,
    TeamIcon::Microscope,
];
impl TeamColor {
    pub fn index(&self, fallback: usize) -> usize {
        match self {
            Self::Palette(i) => i % TEAM_COLORS.len(),
            Self::Other(_) => fallback % TEAM_COLORS.len(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Rules {
    pub enabled: bool,
    pub items: Option<Vec<String>>,
}
impl Default for Rules {
    fn default() -> Self {
        Self {
            enabled: true,
            items: None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Prizes {
    pub enabled: bool,
    pub first: String,
    pub second: String,
    pub third: String,
    pub extra: String,
}
impl Default for Prizes {
    fn default() -> Self {
        Self {
            enabled: true,
            first: String::new(),
            second: String::new(),
            third: String::new(),
            extra: String::new(),
        }
    }
}
impl Quiz {
    pub fn new(id: impl Into<String>, now_ms: i64) -> Self {
        Self {
            id: id.into(),
            created_at: now_ms,
            updated_at: now_ms,
            ..Self::default()
        }
    }
}

fn serialize_rate<S: serde::Serializer>(v: &Option<f64>, s: S) -> Result<S::Ok, S::Error> {
    match v {
        Some(n) if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n < i64::MAX as f64 => {
            s.serialize_i64(*n as i64)
        }
        Some(n) => s.serialize_f64(*n),
        None => s.serialize_none(),
    }
}
