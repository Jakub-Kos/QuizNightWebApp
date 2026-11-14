use super::model::{Podium, Rank, TeamId};
use regex::Regex;
use std::{collections::HashMap, sync::LazyLock};

static NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)název týmu|team name|^tým$|^team$|^název$|^name$").expect("valid pattern")
});
static HISTORY_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)master_team|název týmu|team name|^tým$|^team$|^název$|^name$|tým|team")
        .expect("valid pattern")
});
static PLAYERS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)počet hráčů|players|hráč").expect("valid pattern"));
static QUOTE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)motto|moudro|citát|quote").expect("valid pattern"));
static IMAGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)profilovk|fotk|photo|image|obrázek|logo").expect("valid pattern")
});
static RANK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)rank|pořadí|umístění").expect("valid pattern"));
static ACCURACY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)accuracy|úspěšnost|přesnost").expect("valid pattern"));
static INTEGER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\s]*([+-]?\d+)").expect("valid pattern"));
static FLOAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*([+-]?(?:\d+\.?\d*|\.\d+))").expect("valid pattern"));
use super::model::{Seat, Team};
use super::parse::{ParsedScores, ScoreTeam};
use serde::Serialize;
#[derive(Debug, thiserror::Error)]
/// Describes ImportError.
pub enum ImportError {
    #[error("no_name_column")]
    /// Describes NoNameColumn.
    NoNameColumn,
    #[error("no_history_columns")]
    /// Describes NoHistoryColumns.
    NoHistoryColumns,
    #[error("{0}")]
    /// Describes Csv.
    Csv(String),
}
impl ImportError {
    /// Describes key.
    pub fn key(&self) -> &'static str {
        match self {
            Self::NoNameColumn => "no_name_column",
            Self::NoHistoryColumns => "no_history_columns",
            Self::Csv(_) => "csv",
        }
    }
}
impl From<String> for ImportError {
    fn from(s: String) -> Self {
        match s.as_str() {
            "no_name_column" => Self::NoNameColumn,
            "no_history_columns" => Self::NoHistoryColumns,
            _ => Self::Csv(s),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
/// Describes TeamsImport.
pub struct TeamsImport {
    /// Describes Vec<Team>.
    pub teams: Vec<Team>,
    /// Describes usize.
    pub added: usize,
    /// Describes usize.
    pub updated: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
/// Describes HistoryImport.
pub struct HistoryImport {
    /// Describes Vec<Team>.
    pub teams: Vec<Team>,
    /// Describes usize.
    pub matched: usize,
    /// Describes usize.
    pub total: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
/// Describes ScoresImport.
pub struct ScoresImport {
    /// Describes Vec<Team>.
    pub teams: Vec<Team>,
    /// Describes usize.
    pub added: usize,
    /// Describes usize.
    pub existing: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
/// Describes Side.
pub enum Side {
    /// Describes Front.
    Front,
    /// Describes Left.
    Left,
    /// Describes Right.
    Right,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
/// Describes Direction.
pub struct Direction {
    /// Describes u32.
    pub deg: u32,
    /// Describes Side.
    pub side: Side,
}
impl Direction {
    /// Describes message_key.
    pub fn message_key(&self) -> &'static str {
        match self.side {
            Side::Front => "att_dir_front",
            Side::Left => "att_dir_left",
            Side::Right => "att_dir_right",
        }
    }
}
/// Describes seat_direction.
pub fn seat_direction(seat: &Seat) -> Direction {
    let deg = ((seat.angle.abs() / 5.0).round() * 5.0) as u32;
    Direction {
        deg,
        side: if deg == 0 {
            Side::Front
        } else if seat.angle < 0.0 {
            Side::Left
        } else {
            Side::Right
        },
    }
}

type Row = HashMap<String, String>;
fn table(text: &str) -> Result<(Vec<String>, Vec<Row>), ImportError> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(text.trim_start_matches('\u{feff}').as_bytes());
    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| ImportError::Csv(e.to_string()))?
        .iter()
        .map(str::to_owned)
        .collect();
    let rows = reader
        .records()
        .map(|record| {
            let record = record.map_err(|e| ImportError::Csv(e.to_string()))?;
            Ok(headers
                .iter()
                .cloned()
                .zip(record.iter().map(str::to_owned))
                .collect())
        })
        .collect::<Result<_, ImportError>>()?;
    Ok((headers, rows))
}
fn column<'a>(headers: &'a [String], pattern: &Regex) -> Option<&'a str> {
    headers
        .iter()
        .find(|s| pattern.is_match(s))
        .map(String::as_str)
}
fn cell<'a>(row: &'a Row, column: Option<&str>) -> &'a str {
    column
        .and_then(|c| row.get(c))
        .map(String::as_str)
        .unwrap_or("")
}
fn same(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}
fn number(s: &str) -> Option<i64> {
    INTEGER.captures(s).and_then(|c| c[1].parse().ok())
}
/// Parses a rank cell, retaining an unknown marker for unreadable ranks.
fn rank_cell(s: &str) -> Rank {
    number(s).map_or_else(|| Rank::Unknown("?".into()), Rank::Number)
}
/// Parses the numeric prefix of a percentage cell.
fn accuracy_cell(s: &str) -> Option<f64> {
    let s = s.replacen('%', "", 1).replacen(',', ".", 1);
    let n = FLOAT
        .captures(&s)
        .and_then(|c| c[1].parse::<f64>().ok())
        .unwrap_or(0.0);
    n.is_finite().then_some(n)
}
/// Counts podium finishes among numeric ranks.
fn podium(ranks: &[i64]) -> Podium {
    Podium {
        first: ranks.iter().filter(|&&n| n == 1).count(),
        second: ranks.iter().filter(|&&n| n == 2).count(),
        third: ranks.iter().filter(|&&n| n == 3).count(),
    }
}
/// Creates a team with the standard defaults.
pub fn new_team(id: impl Into<String>) -> Team {
    Team {
        id: Some(TeamId::Text(id.into())),
        quote: Some(String::new()),
        players: Some(0),
        icon: Some("Brain".into()),
        is_new: Some(true),
        ..Team::default()
    }
}
/// Imports registrations and updates teams with matching names.
pub fn import_teams_csv(
    text: &str,
    existing: &[Team],
    has_media: impl Fn(&str) -> bool,
    mut new_id: impl FnMut() -> String,
) -> Result<TeamsImport, ImportError> {
    let (headers, rows) = table(text)?;
    let name = column(&headers, &NAME).ok_or(ImportError::NoNameColumn)?;
    let players = column(&headers, &PLAYERS);
    let quote = column(&headers, &QUOTE);
    let image = column(&headers, &IMAGE);
    let mut result = TeamsImport {
        teams: existing.to_vec(),
        added: 0,
        updated: 0,
    };
    for row in rows {
        let name = cell(&row, Some(name)).trim();
        if name.is_empty() {
            continue;
        }
        let team = if let Some(i) = result.teams.iter().position(|t| same(&t.name, name)) {
            result.updated += 1;
            &mut result.teams[i]
        } else {
            result.added += 1;
            result.teams.push(new_team(new_id()));
            let i = result.teams.len() - 1;
            &mut result.teams[i]
        };
        team.name = name.into();
        let value = cell(&row, players);
        if !value.is_empty() {
            team.players = Some(number(value).unwrap_or(0));
        }
        let value = cell(&row, quote);
        if !value.is_empty() {
            team.quote = Some(value.trim().into());
        }
        let value = cell(&row, image).trim();
        let lower = value.to_lowercase();
        if !value.is_empty()
            && !lower.starts_with("http:")
            && !lower.starts_with("https:")
            && has_media(value)
        {
            team.image = Some(value.into());
        }
    }
    Ok(result)
}
/// Imports historical ranks and accuracy for matching teams.
pub fn import_history_csv(text: &str, existing: &[Team]) -> Result<HistoryImport, ImportError> {
    let (headers, rows) = table(text)?;
    let name = column(&headers, &HISTORY_NAME).ok_or(ImportError::NoHistoryColumns)?;
    let columns: Vec<_> = headers.iter().filter(|s| RANK.is_match(s)).collect();
    if columns.is_empty() {
        return Err(ImportError::NoHistoryColumns);
    }
    let accuracy = column(&headers, &ACCURACY);
    let mut result = HistoryImport {
        teams: existing.to_vec(),
        matched: 0,
        total: rows.len(),
    };
    for team in &mut result.teams {
        let Some(row) = rows.iter().find(|r| same(cell(r, Some(name)), &team.name)) else {
            continue;
        };
        result.matched += 1;
        let history: Vec<_> = columns
            .iter()
            .map(|c| rank_cell(cell(row, Some(c))))
            .collect();
        let ranks: Vec<_> = history
            .iter()
            .filter_map(|r| match r {
                Rank::Number(n) => Some(*n),
                Rank::Unknown(_) => None,
            })
            .collect();
        team.is_new = Some(ranks.is_empty());
        team.best_rank = Some(ranks.iter().min().copied().unwrap_or(0));
        team.podium = Some(podium(&ranks));
        team.rank_history = Some(history);
        team.correct_rate = if accuracy.is_some() {
            accuracy_cell(cell(row, accuracy))
        } else {
            Some(team.correct_rate.unwrap_or(0.0))
        };
    }
    Ok(result)
}
/// Adds teams whose names are absent from the existing list.
pub fn add_teams_from_scores(
    scores: &ParsedScores,
    existing: &[Team],
    mut new_id: impl FnMut() -> String,
) -> ScoresImport {
    let mut teams = existing.to_vec();
    let mut added = 0;
    for row in &scores.teams {
        if teams.iter().any(|t| same(&t.name, &row.name)) {
            continue;
        }
        let mut team = new_team(new_id());
        team.name = row.name.trim().into();
        teams.push(team);
        added += 1;
    }
    ScoresImport {
        teams,
        added,
        existing: scores.teams.len() - added,
    }
}
/// Returns the media filename for a team's image.
pub fn team_image_name(team: &Team, file_name: &str) -> String {
    let id = match &team.id {
        Some(TeamId::Text(s)) => s.clone(),
        Some(TeamId::Number(n)) => n.to_string(),
        None => "null".into(),
    };
    format!(
        "team-{}.{}",
        id,
        file_name.rsplit('.').next().unwrap_or("").to_lowercase()
    )
}
/// Finds a score row's team by ID, then by name.
pub fn team_for_scores<'a>(teams: &'a [Team], row: &ScoreTeam) -> Option<&'a Team> {
    teams
        .iter()
        .find(|t| {
            row.id.as_ref().is_some_and(|id| {
                !id.is_empty() && t.id.as_ref() == Some(&TeamId::Text(id.clone()))
            })
        })
        .or_else(|| teams.iter().find(|t| same(&t.name, &row.name)))
}
/// Reports whether a team counts as present.
pub fn is_present(team: &Team) -> bool {
    team.present != Some(false)
}
