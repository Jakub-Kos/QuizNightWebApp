//! Pure navigation matching QuestionScreen's intro and per-question frames.
use quiz_core::{quiz::parse::Question, show::PlayMode};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Steps {
    pub question: Option<usize>,
    pub step: usize,
}
impl Steps {
    pub fn count(kind: &str, mode: PlayMode) -> usize {
        let base = if matches!(kind, "PImage" | "PVideo") {
            4
        } else {
            3
        };
        base - usize::from(mode == PlayMode::QuestionsOnly)
    }
    /// True means the caller should finish the round. The last frame is retained.
    pub fn next(&mut self, questions: &[Question], mode: PlayMode) -> bool {
        let Some(i) = self.question else {
            if questions.is_empty() {
                return true;
            }
            self.question = Some(0);
            self.step = 0;
            return false;
        };
        if self.step + 1 < Self::count(&kind(&questions[i]), mode) {
            self.step += 1;
        } else if i + 1 < questions.len() {
            self.question = Some(i + 1);
            self.step = 0;
        } else {
            return true;
        }
        false
    }
    pub fn previous(&mut self, questions: &[Question], mode: PlayMode) {
        if self.step > 0 {
            self.step -= 1;
        } else if let Some(i) = self.question.filter(|i| *i > 0) {
            self.question = Some(i - 1);
            self.step = Self::count(&kind(&questions[i - 1]), mode) - 1;
        } else {
            self.question = None;
        }
    }
    pub fn revealed(self, kind: &str, mode: PlayMode) -> bool {
        mode == PlayMode::WithAnswers && self.step == Self::count(kind, mode) - 1
    }
}

pub fn kind(q: &Question) -> String {
    serde_json::to_value(q).unwrap()["type"]
        .as_str()
        .unwrap_or("")
        .to_owned()
}

/// Wire format shared with QuestionScreen.jsx (the introductory frame is qIndex -1).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionFrame {
    pub q_index: i64,
    pub step: usize,
    #[serde(default)]
    pub zoom: bool,
}
impl QuestionFrame {
    pub fn new(position: Steps, zoom: bool) -> Self {
        Self {
            q_index: position.question.map_or(-1, |i| i as i64),
            step: position.step,
            zoom,
        }
    }
    /// Reject stale/malformed indices before they can index the current round's questions.
    pub fn position(&self, questions: &[Question], mode: PlayMode) -> Option<Steps> {
        if self.q_index == -1 && self.step == 0 {
            return Some(Steps::default());
        }
        let i = usize::try_from(self.q_index).ok()?;
        let question = questions.get(i)?;
        (self.step < Steps::count(&kind(question), mode)).then_some(Steps {
            question: Some(i),
            step: self.step,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum QuestionMessage {
    #[serde(rename = "Q_REQUEST")]
    Request { sender: String },
    #[serde(rename = "Q_UPDATE")]
    Update {
        sender: String,
        payload: QuestionFrame,
    },
}
#[cfg(test)]
mod sync_tests {
    use super::*;
    #[test]
    fn javascript_wire_format_and_optional_zoom() {
        assert_eq!(
            serde_json::to_value(QuestionMessage::Request {
                sender: "main".into()
            })
            .unwrap(),
            serde_json::json!({"type":"Q_REQUEST","sender":"main"})
        );
        let msg = QuestionMessage::Update {
            sender: "presenter".into(),
            payload: QuestionFrame::new(
                Steps {
                    question: Some(2),
                    step: 1,
                },
                true,
            ),
        };
        assert_eq!(
            serde_json::to_value(&msg).unwrap(),
            serde_json::json!({"type":"Q_UPDATE","sender":"presenter","payload":{"qIndex":2,"step":1,"zoom":true}})
        );
        assert_eq!(
            serde_json::from_value::<QuestionMessage>(serde_json::to_value(&msg).unwrap()).unwrap(),
            msg
        );
        let frame: QuestionFrame =
            serde_json::from_value(serde_json::json!({"qIndex":-1,"step":0})).unwrap();
        assert!(!frame.zoom);
        assert_eq!(
            frame.position(&[], PlayMode::WithAnswers),
            Some(Steps::default())
        );
    }
    #[test]
    fn invalid_frames_cannot_index_a_round() {
        let questions = quiz_core::quiz::parse::parse_questions_csv(include_str!(
            "../../assets/dev-questions.csv"
        ))[0]
            .questions
            .clone();
        for (q_index, step) in [(-2, 0), (-1, 1), (999, 0), (0, 99)] {
            assert!(
                QuestionFrame {
                    q_index,
                    step,
                    zoom: false
                }
                .position(&questions, PlayMode::WithAnswers)
                .is_none()
            );
        }
        assert!(
            QuestionFrame {
                q_index: 0,
                step: 2,
                zoom: false
            }
            .position(&questions, PlayMode::QuestionsOnly)
            .is_none()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mixed_round_both_modes() {
        let questions: Vec<Question> = ["Written", "PImage", "Top5", "Numeric", "ABCD", "PVideo",
            "Sort", "Yes/No", "Audio", "Video", "Image", "Handout"].iter().map(|kind| {
            serde_json::from_value(serde_json::json!({"id":"1", "text":"?", "type":kind,
                "answer": if *kind == "Top5" { serde_json::json!(["a"]) } else { serde_json::json!("a") },
                "items":[], "options":{}, "media":null})).unwrap()
        }).collect();
        for mode in [PlayMode::WithAnswers, PlayMode::QuestionsOnly] {
            let mut p = Steps::default();
            p.previous(&questions, mode);
            assert_eq!(p, Steps::default());
            assert!(!p.next(&questions, mode));
            for (i, q) in questions.iter().enumerate() {
                for step in 0..Steps::count(&kind(q), mode) {
                    assert_eq!(
                        p,
                        Steps {
                            question: Some(i),
                            step
                        }
                    );
                    assert_eq!(
                        p.revealed(&kind(q), mode),
                        mode == PlayMode::WithAnswers && step == Steps::count(&kind(q), mode) - 1
                    );
                    let before = p;
                    let finished = p.next(&questions, mode);
                    if !finished {
                        p.previous(&questions, mode);
                        assert_eq!(p, before);
                        p.next(&questions, mode);
                    } else {
                        assert_eq!(i, questions.len() - 1);
                    }
                }
            }
        }
    }
}
