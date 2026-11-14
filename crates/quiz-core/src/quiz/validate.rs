use super::parse::{Round, is_round_hidden, is_url};
use super::sort::{MAX_SORT_ITEMS, SheetLetters, sheet_letters, sort_layout};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationInput {
    pub teams: Vec<ValidationTeam>,
    pub scores_sheet_url: Option<String>,
    pub builtin: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationTeam {
    pub name: String,
    pub image: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warning,
    Info,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub level: Level,
    pub code: String,
    pub params: Value,
    pub round_index: isize,
    pub q_index: isize,
}
fn kind(file: &str) -> &str {
    let e = file.rsplit('.').next().unwrap_or("").to_lowercase();
    if ["jpg", "jpeg", "png", "gif", "webp", "avif", "svg", "bmp"].contains(&e.as_str()) {
        "image"
    } else if ["mp3", "wav", "ogg", "m4a", "aac", "flac"].contains(&e.as_str()) {
        "audio"
    } else if ["mp4", "webm", "mov", "m4v"].contains(&e.as_str()) {
        "video"
    } else {
        "other"
    }
}
fn blank(v: &Value) -> bool {
    v.is_null() || v.as_str().is_some_and(|s| s.trim().is_empty())
}
pub fn preview_step(kind: &str, view: &str) -> usize {
    1 + usize::from(matches!(kind, "PImage" | "PVideo")) + usize::from(view == "answer")
}
pub fn validate_quiz(
    quiz: &ValidationInput,
    rounds: &[Round],
    has_media: impl Fn(&str) -> bool,
) -> Vec<Issue> {
    use Level::*;
    let mut out = vec![];
    let mut add = |level, code: &str, ri, qi, params| {
        out.push(Issue {
            level,
            code: code.into(),
            round_index: ri,
            q_index: qi,
            params,
        })
    };
    if rounds.is_empty() {
        add(Error, "no_rounds", -1, -1, json!({}));
    }
    if quiz.teams.is_empty() {
        add(Info, "no_teams", -1, -1, json!({}));
    }
    if quiz.scores_sheet_url.as_deref().unwrap_or("").is_empty() && !quiz.builtin {
        add(Info, "no_scores", -1, -1, json!({}));
    }
    let mut names = std::collections::HashSet::new();
    for t in &quiz.teams {
        let name = t.name.trim().to_lowercase();
        if name.is_empty() {
            add(Warning, "team_no_name", -1, -1, json!({}));
        } else if !names.insert(name) {
            add(Warning, "team_duplicate", -1, -1, json!({"team":t.name}));
        }
    }
    for (ri, r) in rounds.iter().enumerate() {
        let ri = ri as isize;
        if is_round_hidden(r) {
            add(Warning, "round_hidden", ri, -1, json!({}));
        }
        let mut seen = std::collections::HashSet::new();
        for (qi, q) in r.questions.iter().enumerate() {
            let qi = qi as isize;
            let v = serde_json::to_value(q).unwrap();
            let t = v["type"].as_str().unwrap();
            let a = &v["answer"];
            let mut issue = |l, c, p| add(l, c, ri, qi, p);
            if !seen.insert(&q.id) {
                issue(Warning, "duplicate_number", json!({"number":q.id}));
            }
            if ![
                "Written", "Numeric", "ABCD", "Yes/No", "Top5", "Sort", "Handout", "PImage",
                "PVideo", "Audio", "Image", "Video",
            ]
            .contains(&t)
            {
                issue(Warning, "type_unknown", json!({"type":t}));
            }
            if q.text.trim().is_empty() && !["PImage", "PVideo", "Handout"].contains(&t) {
                issue(Warning, "no_text", json!({}));
            }
            let length = q.text.encode_utf16().count();
            if length > 250 {
                issue(Warning, "long_text", json!({"length":length}));
            }
            if t == "Top5" {
                let count = a.as_array().unwrap().iter().filter(|a| !blank(a)).count();
                if !(3..=15).contains(&count) {
                    issue(Warning, "top5_count", json!({"count":count}));
                } else if v["declared"]
                    .as_u64()
                    .is_some_and(|n| n != 0 && n != count as u64)
                {
                    issue(
                        Warning,
                        "top5_declared",
                        json!({"declared":v["declared"],"count":count}),
                    );
                }
            } else if t == "Handout" {
                let answer = a.as_str().unwrap_or("").trim();
                if answer.is_empty() {
                    issue(Warning, "handout_no_answer", json!({}));
                } else if !is_url(answer) && !has_media(answer) {
                    issue(Error, "media_missing", json!({"file":answer}));
                } else if !["image", "other"].contains(&kind(answer)) {
                    issue(Warning, "media_kind", json!({"file":answer,"type":t}));
                }
            } else if t == "Sort" {
                if let super::parse::QuestionData::Known(super::parse::KnownQuestionData::Sort {
                    items,
                    ..
                }) = &q.data
                {
                    let count = items.len();
                    if count < 2 {
                        issue(Error, "sort_count_low", json!({"count":count}));
                    }
                    if count > MAX_SORT_ITEMS {
                        issue(
                            Warning,
                            "sort_count_high",
                            json!({"count":count,"max":MAX_SORT_ITEMS}),
                        );
                    }
                    let letters = sheet_letters(items);
                    if count >= 2 && letters == SheetLetters::None {
                        issue(
                            Warning,
                            "sort_no_letters",
                            json!({"letters":sort_layout(q).correct.iter().map(|i|i.item.letter.as_deref().unwrap()).collect::<Vec<_>>().join(", ")}),
                        );
                    }
                    if letters == SheetLetters::Invalid {
                        issue(
                            Error,
                            "sort_bad_letters",
                            json!({"letters":items.iter().map(|i|i.letter.as_deref().filter(|s|!s.is_empty()).unwrap_or("–")).collect::<Vec<_>>().join(", "),"max":char::from_u32(64+count as u32).unwrap().to_string()}),
                        );
                    }
                    if letters == SheetLetters::Ok
                        && count > 1
                        && items.iter().enumerate().all(|(i, it)| {
                            it.letter.as_deref()
                                == Some(char::from_u32(65 + i as u32).unwrap().to_string().as_str())
                        })
                    {
                        issue(Warning, "sort_in_order", json!({}));
                    }
                    let pictures: Vec<_> = items
                        .iter()
                        .filter_map(|i| i.media.as_deref().filter(|s| !s.is_empty()))
                        .collect();
                    if !pictures.is_empty() && pictures.len() < count {
                        issue(
                            Warning,
                            "sort_some_pictures",
                            json!({"count":pictures.len(),"total":count}),
                        );
                    }
                    for media in pictures {
                        if !is_url(media) && !has_media(media) {
                            issue(Error, "media_missing", json!({"file":media}));
                        } else if !["image", "other"].contains(&kind(media)) {
                            issue(Warning, "media_kind", json!({"file":media,"type":t}));
                        }
                    }
                }
            } else if blank(a) {
                issue(Warning, "no_answer", json!({}));
            }
            if t == "ABCD" {
                let missing: Vec<_> = ["A", "B", "C", "D"]
                    .into_iter()
                    .filter(|k| blank(&v["options"][k]))
                    .collect();
                if !missing.is_empty() {
                    issue(Error, "abcd_options", json!({"missing":missing.join(", ")}));
                }
                if !blank(a)
                    && !["A", "B", "C", "D"]
                        .contains(&a.as_str().unwrap_or("").trim().to_uppercase().as_str())
                {
                    issue(Error, "abcd_answer", json!({"answer":a}));
                }
            }
            if t == "Yes/No"
                && !blank(a)
                && !["yes", "no"].contains(&a.as_str().unwrap_or("").trim().to_lowercase().as_str())
            {
                issue(Error, "yesno_answer", json!({"answer":a}));
            }
            if t == "Numeric"
                && !blank(a)
                && !a.as_str().unwrap_or("").chars().any(|c| c.is_ascii_digit())
            {
                issue(Warning, "numeric_answer", json!({"answer":a}));
            }
            if let Some(media) = q.media.as_deref().filter(|s| !s.is_empty()) {
                if !is_url(media) && !has_media(media) {
                    issue(Error, "media_missing", json!({"file":media}));
                }
                let expected = if t == "Audio" {
                    "audio"
                } else if t.contains("Video") {
                    "video"
                } else {
                    "image"
                };
                if kind(media) != "other" && kind(media) != expected {
                    issue(Warning, "media_kind", json!({"file":media,"type":t}));
                }
            } else if ["PImage", "PVideo", "Audio", "Image", "Video", "Handout"].contains(&t) {
                issue(Error, "media_required", json!({"type":t}));
            }
        }
    }
    for t in &quiz.teams {
        if let Some(file) = t
            .image
            .as_deref()
            .filter(|s| !s.is_empty() && !is_url(s) && !has_media(s))
        {
            add(
                Error,
                "team_image_missing",
                -1,
                -1,
                json!({"team":t.name,"file":file}),
            );
        }
    }
    out
}
