//! Leaderboard construction shared by manual scoring and previews.
use std::collections::BTreeMap;

use super::model::Team;
use super::parse::{ParsedScores, Round, ScoreTeam, is_round_hidden};

pub const SCORES_CHANNEL: &str = "quiz-scores";
pub type ManualScores = BTreeMap<String, BTreeMap<String, f64>>;

pub fn scored_rounds(rounds: &[Round]) -> Vec<&Round> {
    rounds
        .iter()
        .filter(|round| !is_round_hidden(round))
        .collect()
}

/// The UI translates the round word and combines it with this stable number.
pub fn round_label(round: &Round) -> &str {
    &round.number
}

fn label(number: &str, word: &str) -> String {
    let mut chars = word.chars();
    let first = chars
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();
    format!("{}{} {}", first, chars.as_str().to_lowercase(), number)
}

pub fn manual_leaderboard(
    teams: &[Team],
    rounds: &[Round],
    manual_scores: &ManualScores,
    round_word: Option<&str>,
) -> ParsedScores {
    let scored = scored_rounds(rounds);
    let labels: Vec<String> = scored
        .iter()
        .map(|r| {
            label(
                &r.number,
                round_word.filter(|word| !word.is_empty()).unwrap_or("Kolo"),
            )
        })
        .collect();
    let teams = teams
        .iter()
        .filter_map(|team| {
            let name = team.name.trim();
            if name.is_empty() || team.present == Some(false) {
                return None;
            }
            let id = team.id.as_ref().map(|id| match id {
                super::model::TeamId::Text(s) => s.clone(),
                super::model::TeamId::Number(n) => n.to_string(),
            });
            let points = id.as_ref().and_then(|id| manual_scores.get(id));
            let scores = scored
                .iter()
                .zip(&labels)
                .map(|(round, label)| {
                    let value = points
                        .and_then(|p| p.get(&round.number))
                        .copied()
                        .unwrap_or(0.0);
                    (label.clone(), if value.is_finite() { value } else { 0.0 })
                })
                .collect();
            Some(ScoreTeam {
                name: name.to_owned(),
                scores,
                id: None,
            })
        })
        .collect();
    ParsedScores {
        teams,
        rounds: labels,
    }
}

pub fn sample_leaderboard(
    teams: &[Team],
    rounds: &[String],
    round_word: Option<&str>,
    fallback_names: &[String],
) -> ParsedScores {
    let names: Vec<String> = teams
        .iter()
        .filter_map(|team| {
            let name = team.name.trim();
            (!name.is_empty()).then(|| name.to_owned())
        })
        .collect();
    let names = if names.is_empty() {
        fallback_names
    } else {
        &names
    };
    let labels = if rounds.is_empty() {
        (1..=4)
            .map(|n| {
                format!(
                    "{} {n}",
                    round_word.filter(|word| !word.is_empty()).unwrap_or("Kolo")
                )
            })
            .collect()
    } else {
        rounds.to_vec()
    };
    let mut seed = 11_u64;
    let teams = names
        .iter()
        .map(|name| {
            let scores = labels
                .iter()
                .map(|label| {
                    seed = (seed * 16807) % 2147483647;
                    (
                        label.clone(),
                        (seed as f64 / 2147483647.0 * 20.0).round() / 2.0,
                    )
                })
                .collect();
            ScoreTeam {
                name: name.clone(),
                scores,
                id: None,
            }
        })
        .collect();
    ParsedScores {
        teams,
        rounds: labels,
    }
}

pub fn has_points(board: &ParsedScores) -> bool {
    board
        .teams
        .iter()
        .any(|team| team.scores.values().any(|points| *points > 0.0))
}
