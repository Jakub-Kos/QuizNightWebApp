use serde::{Deserialize, Serialize};
use serde_json::Value;
pub const EXAMPLE_MEDIA: [(&str, &str); 6] = [
    ("Image", "pad_BM2.webp"),
    ("PImage", "1944paris.jpg"),
    ("Audio", "audio.m4a"),
    ("Video", "video.mp4"),
    ("PVideo", "video.mp4"),
    ("Handout", "handout.jpg"),
];
pub const SHEET_HEAD: [&str; 10] = [
    "Kolo",
    "Q#",
    "Otázka",
    "Typ Odpovědi",
    "A",
    "B",
    "C",
    "D",
    "Správná odpověď",
    "Zdroj",
];
pub fn example_rows(kind: &str, example: &Value) -> Vec<Vec<String>> {
    let text = example["text"].as_str().unwrap_or("");
    let row = |i: usize, text: &str, t: &str, opts: Vec<String>, answer: String, media: &str| {
        let mut r = vec![String::new(), i.to_string(), text.into(), t.into()];
        r.extend(opts);
        r.extend([answer, media.into()]);
        r
    };
    let none = || vec![String::new(); 4];
    if kind == "Top5" {
        return example["answer"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, a)| {
                row(
                    i + 1,
                    if i == 0 { text } else { "" },
                    if i == 0 { kind } else { "" },
                    none(),
                    a.as_str().unwrap_or("").into(),
                    "",
                )
            })
            .collect();
    }
    if kind == "Sort" {
        return example["items"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let mut opts = none();
                opts[0] = item.as_str().unwrap_or("").into();
                row(
                    i + 1,
                    if i == 0 { text } else { "" },
                    if i == 0 { kind } else { "" },
                    opts,
                    example["letters"][i].as_str().unwrap_or("").into(),
                    "",
                )
            })
            .collect();
    }
    vec![row(
        1,
        text,
        kind,
        ["A", "B", "C", "D"]
            .iter()
            .map(|k| example["options"][k].as_str().unwrap_or("").into())
            .collect(),
        example["answer"].as_str().unwrap_or("").into(),
        EXAMPLE_MEDIA
            .iter()
            .find(|(k, _)| *k == kind)
            .map_or("", |(_, v)| v),
    )]
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExampleQuestions {
    pub csv: String,
    pub rows: Vec<Vec<String>>,
    pub media: Vec<String>,
}
pub fn example_questions(lang: &str, title: &str) -> ExampleQuestions {
    let data: Value =
        serde_json::from_str(include_str!("../../tests/fixtures/quiz/example-data.json")).unwrap();
    let section = data.get(lang).unwrap_or(&data["en"]);
    let mut rows = vec![];
    for t in section["types"].as_array().unwrap() {
        let t = t.as_str().unwrap();
        rows.extend(example_rows(t, &section["examples"][t]));
    }
    let mut media = vec![];
    for (i, r) in rows.iter_mut().enumerate() {
        r[0] = if i == 0 {
            format!(
                "{} 1 - {}",
                if lang == "cs" { "Kolo" } else { "Round" },
                title
            )
        } else {
            String::new()
        };
        r[1] = (i + 1).to_string();
        for m in [r[9].as_str(), if r[3] == "Handout" { &r[8] } else { "" }] {
            if !m.is_empty() && !media.iter().any(|v| v == m) {
                media.push(m.to_owned());
            }
        }
    }
    let quote = |s: &str| {
        if s.contains([',', '"', '\r', '\n']) || s.starts_with(' ') || s.ends_with(' ') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.into()
        }
    };
    let csv = std::iter::once(SHEET_HEAD.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        .chain(rows.clone())
        .map(|r| r.iter().map(|s| quote(s)).collect::<Vec<_>>().join(","))
        .collect::<Vec<_>>()
        .join("\r\n");
    ExampleQuestions { csv, rows, media }
}
