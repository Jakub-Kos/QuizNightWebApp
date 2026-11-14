//! Pure question CSV parsing. No clocks, random sources, or browser dependencies.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SortItem {
    pub text: String,
    pub letter: Option<String>,
    pub media: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum KnownQuestionData {
    Top5 {
        answer: Vec<String>,
        declared: Option<usize>,
    },
    Sort {
        items: Vec<SortItem>,
        answer: String,
    },
    Written {
        answer: String,
    },
    Numeric {
        answer: String,
    },
    ABCD {
        answer: String,
    },
    #[serde(rename = "Yes/No")]
    YesNo {
        answer: String,
    },
    Handout {
        answer: String,
    },
    PImage {
        answer: String,
    },
    PVideo {
        answer: String,
    },
    Audio {
        answer: String,
    },
    Image {
        answer: String,
    },
    Video {
        answer: String,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QuestionData {
    Known(KnownQuestionData),
    Unknown {
        #[serde(rename = "type")]
        kind: String,
        answer: String,
    },
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Question {
    #[serde(skip)]
    pub missing_fields: Vec<String>,
    pub id: String,
    pub text: String,
    pub options: BTreeMap<String, String>,
    pub media: Option<String>,
    #[serde(flatten)]
    pub data: QuestionData,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Round {
    pub id: String,
    pub number: String,
    pub title: String,
    pub rows: Vec<Vec<String>>,
    pub questions: Vec<Question>,
}
pub fn media_key(name: &str) -> String {
    name.trim()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("")
        .to_lowercase()
}
pub fn is_url(name: &str) -> bool {
    let s = name.trim().to_lowercase();
    s.starts_with("https://") || s.starts_with("http://")
}
fn cell(row: &[String], i: usize) -> &str {
    row.get(i).map(String::as_str).unwrap_or("")
}
fn media(row: &[String]) -> Option<String> {
    let s = cell(row, 9);
    (!s.is_empty()).then(|| s.trim().to_owned())
}
pub fn is_round_marker(a: &str, b: &str) -> bool {
    regex::Regex::new(r"(?i)^(kolo|round)(?-u:\b)")
        .unwrap()
        .is_match(a)
        && b.to_lowercase() != "q#"
}
pub fn is_question_row(row: &[String]) -> bool {
    let n = cell(row, 1).trim();
    !n.is_empty() && js_numeric(n) && [2, 9, 8].iter().any(|&i| !cell(row, i).trim().is_empty())
}
pub fn normalize_type(s: &str) -> String {
    let s = s.trim();
    if regex::Regex::new(r"(?i)^top\s*\d+$").unwrap().is_match(s) {
        "Top5".into()
    } else if s.is_empty() {
        "Written".into()
    } else {
        s.into()
    }
}
pub fn build_questions(rows: &[Vec<String>]) -> Vec<Question> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < rows.len() {
        let r = &rows[i];
        let t = normalize_type(cell(r, 3));
        let mut end = i + 1;
        if t == "Sort" || t == "Top5" {
            while end < rows.len()
                && cell(&rows[end], 2).is_empty()
                && cell(&rows[end], 3).is_empty()
            {
                end += 1;
            }
        }
        let answer = cell(r, 8).to_owned();
        let mut options = BTreeMap::new();
        let mut m = media(r);
        let data = if !matches!(
            t.as_str(),
            "Top5"
                | "Sort"
                | "Written"
                | "Numeric"
                | "ABCD"
                | "Yes/No"
                | "Handout"
                | "PImage"
                | "PVideo"
                | "Audio"
                | "Image"
                | "Video"
        ) {
            QuestionData::Unknown {
                kind: t.clone(),
                answer: answer.clone(),
            }
        } else {
            QuestionData::Known(match t.as_str() {
                "Top5" => KnownQuestionData::Top5 {
                    answer: rows[i..end].iter().map(|r| cell(r, 8).into()).collect(),
                    declared: if cell(r, 3).trim() == "Top5" {
                        None
                    } else {
                        cell(r, 3)
                            .trim()
                            .get(3..)
                            .and_then(|s| s.trim().parse().ok())
                            .filter(|n| *n != 0)
                    },
                },
                "Sort" => {
                    m = None;
                    KnownQuestionData::Sort {
                        answer: String::new(),
                        items: rows[i..end]
                            .iter()
                            .map(|r| {
                                let o = cell(r, 4).trim();
                                let a = cell(r, 8).trim();
                                let letter = a.len() == 1 && a.as_bytes()[0].is_ascii_alphabetic();
                                SortItem {
                                    text: if !o.is_empty() {
                                        o.into()
                                    } else if letter {
                                        String::new()
                                    } else {
                                        a.into()
                                    },
                                    letter: if !o.is_empty() || letter {
                                        (!a.is_empty()).then(|| a.to_uppercase())
                                    } else {
                                        None
                                    },
                                    media: media(r),
                                }
                            })
                            .filter(|it| !it.text.is_empty() || it.media.is_some())
                            .collect(),
                    }
                }
                "Written" => KnownQuestionData::Written { answer },
                "Numeric" => KnownQuestionData::Numeric { answer },
                "ABCD" => KnownQuestionData::ABCD { answer },
                "Yes/No" => KnownQuestionData::YesNo { answer },
                "Handout" => KnownQuestionData::Handout { answer },
                "PImage" => KnownQuestionData::PImage { answer },
                "PVideo" => KnownQuestionData::PVideo { answer },
                "Audio" => KnownQuestionData::Audio { answer },
                "Image" => KnownQuestionData::Image { answer },
                "Video" => KnownQuestionData::Video { answer },
                _ => unreachable!(),
            })
        };
        if t != "Sort" && t != "Top5" {
            for (j, k) in ["A", "B", "C", "D"].iter().enumerate() {
                if r.len() > j + 4 {
                    options.insert((*k).into(), cell(r, j + 4).into());
                }
            }
        }
        out.push(Question {
            missing_fields: [(1, "id"), (2, "text"), (8, "answer")]
                .into_iter()
                .filter(|(n, k)| r.len() <= *n && (*k != "answer" || (t != "Sort" && t != "Top5")))
                .map(|(_, k)| k.to_owned())
                .collect(),
            id: cell(r, 1).into(),
            text: cell(r, 2).into(),
            options,
            media: m,
            data,
        });
        i = end;
    }
    out
}
pub fn parse_questions_csv(text: &str) -> Vec<Round> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());
    let mut rounds: Vec<Round> = Vec::new();
    for record in reader.records() {
        let row: Vec<String> = record
            .unwrap_or_default()
            .iter()
            .map(str::to_owned)
            .collect();
        let a = cell(&row, 0).trim();
        let b = cell(&row, 1).trim();
        if is_round_marker(a, b) {
            let marker = split_marker(a);
            rounds.push(Round {
                id: format!("round-{}", rounds.len() + 1),
                number: marker.number,
                title: marker.title,
                rows: vec![],
                questions: vec![],
            });
        }
        if let Some(r) = rounds.last_mut().filter(|_| is_question_row(&row)) {
            r.rows.push(row);
        }
    }
    for r in &mut rounds {
        r.questions = build_questions(&r.rows);
    }
    rounds
}
pub fn is_round_hidden(r: &Round) -> bool {
    let t = r.title.to_uppercase();
    t == "KOLO" || t.contains("TEST") || t.contains("KOLO X") || r.questions.is_empty()
}

pub const ROUND_WORD: &str = r"(?i)^(kolo|round)(?-u:\b)";
pub const TEAM_COLUMN: &str = "(?i)název týmu|team name";
pub const TEAM_ID_HEADER: &str = "quiznight-id";
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub word: String,
    pub number: String,
    pub title: String,
}
pub fn split_marker(marker: &str) -> Marker {
    let re = regex::Regex::new(ROUND_WORD).unwrap();
    let (head, title) = marker.split_once('-').unwrap_or((marker, ""));
    Marker {
        word: re.find(marker).map_or("", |m| m.as_str()).into(),
        number: re.replace(head, "").trim().into(),
        title: title.trim().into(),
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreTeam {
    pub name: String,
    #[serde(serialize_with = "serialize_scores")]
    pub scores: BTreeMap<String, f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedScores {
    pub teams: Vec<ScoreTeam>,
    pub rounds: Vec<String>,
}
pub fn parse_scores_csv(text: &str) -> Option<ParsedScores> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());
    let rows: Vec<Vec<String>> = reader
        .records()
        .map(|r| {
            r.unwrap_or_default()
                .iter()
                .map(|v| v.trim().to_owned())
                .collect()
        })
        .collect();
    let team = regex::Regex::new(TEAM_COLUMN).unwrap();
    let hi = rows
        .iter()
        .position(|r| r.iter().any(|v| team.is_match(v)))?;
    let headers = &rows[hi];
    let ni = headers.iter().position(|v| team.is_match(v)).unwrap();
    let id = headers.iter().position(|v| v == TEAM_ID_HEADER);
    let round = regex::Regex::new("(?i)kolo|round").unwrap();
    let cols: Vec<_> = headers
        .iter()
        .enumerate()
        .filter(|(_, v)| round.is_match(v))
        .collect();
    let float =
        regex::Regex::new(r"^[+-]?(?:Infinity|(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][+-]?[0-9]+)?)")
            .unwrap();
    let teams = rows[hi + 1..]
        .iter()
        .filter(|r| !cell(r, ni).is_empty())
        .map(|r| ScoreTeam {
            name: cell(r, ni).into(),
            id: id.and_then(|i| (!cell(r, i).is_empty()).then(|| cell(r, i).into())),
            scores: cols
                .iter()
                .map(|(i, n)| {
                    let v = cell(r, *i).replacen(',', ".", 1);
                    let score = float
                        .find(&v)
                        .and_then(|m| m.as_str().parse::<f64>().ok())
                        .unwrap_or(0.0);
                    ((*n).clone(), score)
                })
                .collect(),
        })
        .collect();
    Some(ParsedScores {
        teams,
        rounds: cols.iter().map(|(_, n)| (*n).clone()).collect(),
    })
}

fn serialize_scores<S: serde::Serializer>(
    scores: &BTreeMap<String, f64>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let values: BTreeMap<_, serde_json::Value> = scores
        .iter()
        .map(|(k, &v)| {
            (
                k,
                if v.is_finite() && v.fract() == 0.0 && v.abs() < i64::MAX as f64 {
                    serde_json::json!(v as i64)
                } else {
                    serde_json::json!(v)
                },
            )
        })
        .collect();
    values.serialize(serializer)
}

fn js_numeric(s: &str) -> bool {
    if matches!(s, "Infinity" | "+Infinity" | "-Infinity") {
        return true;
    }
    if let Some((prefix, digits)) = s.get(..2).zip(s.get(2..)) {
        let radix = match prefix {
            "0x" | "0X" => 16,
            "0b" | "0B" => 2,
            "0o" | "0O" => 8,
            _ => 0,
        };
        if radix != 0 {
            return !digits.is_empty() && digits.chars().all(|c| c.is_digit(radix));
        }
    }
    s.parse::<f64>().is_ok_and(|n| !n.is_nan())
}

impl Serialize for Question {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = serde_json::to_value(&self.data).map_err(serde::ser::Error::custom)?;
        let obj = value.as_object_mut().unwrap();
        obj.insert("id".into(), serde_json::json!(self.id));
        obj.insert("text".into(), serde_json::json!(self.text));
        obj.insert("options".into(), serde_json::json!(self.options));
        obj.insert("media".into(), serde_json::json!(self.media));
        for key in &self.missing_fields {
            obj.remove(key);
        }
        value.serialize(serializer)
    }
}
