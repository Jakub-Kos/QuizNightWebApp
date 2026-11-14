use super::parse::{KnownQuestionData, Question, QuestionData, SortItem};
use serde::{Deserialize, Serialize};
pub const MAX_SORT_ITEMS: usize = 10;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SheetLetters {
    Ok,
    None,
    Invalid,
}
pub fn sheet_letters(items: &[SortItem]) -> SheetLetters {
    if items
        .iter()
        .all(|i| i.letter.as_deref().unwrap_or("").is_empty())
    {
        return SheetLetters::None;
    }
    let mut letters: Vec<_> = items
        .iter()
        .map(|i| i.letter.clone().unwrap_or_default())
        .collect();
    letters.sort();
    if letters
        == (0..items.len())
            .map(|i| char::from_u32(65 + i as u32).unwrap().to_string())
            .collect::<Vec<_>>()
    {
        SheetLetters::Ok
    } else {
        SheetLetters::Invalid
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedItem {
    #[serde(flatten)]
    pub item: SortItem,
    pub place: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortLayout {
    pub shown: Vec<PlacedItem>,
    pub correct: Vec<PlacedItem>,
    pub answer: String,
    pub from_sheet: bool,
}
pub fn sort_layout(question: &Question) -> SortLayout {
    let items = match &question.data {
        QuestionData::Known(KnownQuestionData::Sort { items, .. }) => items.as_slice(),
        _ => &[],
    };
    let from_sheet = sheet_letters(items) == SheetLetters::Ok;
    let mut order: Vec<usize> = (0..items.len()).collect();
    if from_sheet {
        order.sort_by_key(|&i| items[i].letter.clone());
    } else {
        let seed = format!(
            "{}|{}|{}",
            question.id,
            question.text,
            items
                .iter()
                .map(|i| i.text.as_str())
                .collect::<Vec<_>>()
                .join("|")
        );
        let mut h = 2166136261u32;
        for ch in seed.chars() {
            h = (h ^ ch as u32).wrapping_mul(16777619);
        }
        for i in (1..order.len()).rev() {
            h = (h ^ (h >> 15)).wrapping_mul(2246822507);
            h = (h ^ (h >> 13)).wrapping_mul(3266489909);
            h ^= h >> 16;
            let j = ((h as f64 / 4294967296.0) * (i + 1) as f64).floor() as usize;
            order.swap(i, j);
        }
        if order.len() > 1 && order.iter().enumerate().all(|(i, &v)| i == v) {
            order.rotate_left(1);
        }
    }
    let shown: Vec<_> = order
        .iter()
        .enumerate()
        .map(|(k, &i)| {
            let mut item = items[i].clone();
            item.letter = Some(char::from_u32(65 + k as u32).unwrap().to_string());
            PlacedItem { item, place: i + 1 }
        })
        .collect();
    let mut correct = shown.clone();
    correct.sort_by_key(|i| i.place);
    let answer = correct
        .iter()
        .map(|i| i.item.letter.as_deref().unwrap())
        .collect::<Vec<_>>()
        .join(" ");
    SortLayout {
        shown,
        correct,
        answer,
        from_sheet,
    }
}
