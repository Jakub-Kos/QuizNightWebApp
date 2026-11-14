//! Question editor sheet model.
use super::parse::{
    Question, build_questions, is_question_row, is_round_marker, normalize_type, split_marker,
};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub const COLUMNS: usize = 12;
pub const AUTHOR_COL: usize = 10;
pub const CHANGED_COL: usize = 11;
type Rows = Vec<Vec<String>>;
fn cell(row: &[String], i: usize) -> &str {
    row.get(i).map_or("", |s| s.trim())
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StampColumns {
    pub author: usize,
    pub changed: usize,
}
static AUTHOR: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new("(?i)^(author|autor|szerző)$").unwrap());
static CHANGED: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new("(?i)^(changed|upraveno|módosítva)$").unwrap());
pub fn stamp_columns(rows: &[Vec<String>]) -> StampColumns {
    let header = rows.iter().find(|r| cell(r, 1).eq_ignore_ascii_case("q#"));
    let find = |re: &regex::Regex| {
        header.and_then(|r| {
            r.iter()
                .enumerate()
                .find(|(i, s)| *i >= 10 && re.is_match(s.trim()))
                .map(|(i, _)| i)
        })
    };
    let mut author = find(&AUTHOR);
    let mut changed = find(&CHANGED);
    let free = |a: Option<usize>, c: Option<usize>| {
        (10..)
            .find(|&i| Some(i) != a && Some(i) != c && rows.iter().all(|r| cell(r, i).is_empty()))
            .unwrap()
    };
    if author.is_none() {
        author = Some(free(author, changed));
    }
    if changed.is_none() {
        changed = Some(free(author, changed));
    }
    StampColumns {
        author: author.unwrap(),
        changed: changed.unwrap(),
    }
}
pub fn to_model_rows(rows: &[Vec<String>], cols: &StampColumns) -> Rows {
    rows.iter()
        .map(|r| {
            (0..12)
                .map(|i| {
                    r.get(match i {
                        10 => cols.author,
                        11 => cols.changed,
                        _ => i,
                    })
                    .cloned()
                    .unwrap_or_default()
                })
                .collect()
        })
        .collect()
}
pub fn same_rows(a: &[Vec<String>], b: &[Vec<String>]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(a, b)| (2..12).all(|i| cell(a, i) == cell(b, i)))
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub start: usize,
    pub end: usize,
    pub rows: Rows,
    pub question: Question,
    pub author: String,
    pub changed: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetRound {
    pub marker: String,
    pub marker_row: usize,
    pub word: String,
    pub number: String,
    pub title: String,
    pub questions: Vec<Block>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sheet {
    pub rows: Rows,
    pub header_row: isize,
    pub rounds: Vec<SheetRound>,
}
pub fn read_sheet(rows: &[Vec<String>]) -> Sheet {
    let mut sheet = Sheet {
        rows: rows.to_vec(),
        header_row: -1,
        rounds: vec![],
    };
    for (index, row) in rows.iter().enumerate() {
        let a = cell(row, 0);
        let b = cell(row, 1);
        if sheet.header_row < 0 && b.eq_ignore_ascii_case("q#") {
            sheet.header_row = index as isize;
        }
        if is_round_marker(a, b) {
            let m = split_marker(a);
            sheet.rounds.push(SheetRound {
                marker: a.into(),
                marker_row: index,
                word: m.word,
                number: m.number,
                title: m.title,
                questions: vec![],
            });
        }
        let Some(round) = sheet.rounds.last_mut() else {
            continue;
        };
        if !is_question_row(row) {
            continue;
        }
        if let Some(last) = round.questions.last_mut()
            && matches!(
                normalize_type(cell(&last.rows[0], 3)).as_str(),
                "Top5" | "Sort"
            )
            && cell(row, 2).is_empty()
            && cell(row, 3).is_empty()
        {
            last.end = index;
            last.rows.push(row.clone());
            last.question = build_questions(&last.rows).remove(0);
            continue;
        }
        round.questions.push(Block {
            start: index,
            end: index,
            rows: vec![row.clone()],
            question: build_questions(std::slice::from_ref(row)).remove(0),
            author: cell(row, 10).into(),
            changed: cell(row, 11).into(),
        });
    }
    sheet
}
pub fn round_marker(word: &str, number: &str, title: &str) -> String {
    format!(
        "{} {}{}",
        if word.is_empty() { "Kolo" } else { word },
        number,
        if title.trim().is_empty() {
            String::new()
        } else {
            format!(" - {}", title.trim())
        }
    )
}
pub fn stamp_now(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> String {
    format!("{year}-{month:02}-{day:02} {hour:02}:{minute:02}")
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftItem {
    pub text: String,
    pub media: String,
    pub letter: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
    pub options: std::collections::BTreeMap<String, String>,
    pub answer: String,
    pub media: String,
    pub answers: Vec<String>,
    pub items: Vec<DraftItem>,
}
pub fn with_letters(items: &[DraftItem], random: &mut impl FnMut() -> f64) -> Vec<DraftItem> {
    let mut letters: Vec<usize> = (0..items.len()).collect();
    if items.len() > 1 {
        loop {
            for i in (1..letters.len()).rev() {
                let j = (random() * (i + 1) as f64).floor() as usize;
                letters.swap(i, j);
            }
            if letters.iter().enumerate().any(|(i, l)| i != *l) {
                break;
            }
        }
    }
    items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            let mut it = it.clone();
            it.letter = char::from_u32(65 + letters[i] as u32).unwrap().to_string();
            it
        })
        .collect()
}
pub fn draft_from(q: &Question) -> Draft {
    use super::parse::{KnownQuestionData as K, QuestionData as D};
    let v = serde_json::to_value(&q.data).unwrap();
    let kind = v["type"].as_str().unwrap().to_string();
    let mut d = Draft {
        kind,
        text: q.text.clone(),
        options: ["A", "B", "C", "D"]
            .into_iter()
            .map(|k| (k.into(), q.options.get(k).cloned().unwrap_or_default()))
            .collect(),
        answer: v["answer"].as_str().unwrap_or("").into(),
        media: q.media.clone().unwrap_or_default(),
        answers: vec![String::new(); 5],
        items: with_letters(
            &vec![
                DraftItem {
                    text: String::new(),
                    media: String::new(),
                    letter: String::new()
                };
                3
            ],
            &mut || 0.0,
        ),
    };
    if let D::Known(K::Top5 { answer, .. }) = &q.data {
        d.answers = answer.clone();
    }
    if let D::Known(K::Sort { items, .. }) = &q.data
        && !items.is_empty()
    {
        let source = if items
            .iter()
            .any(|i| i.letter.as_deref().unwrap_or("").is_empty())
        {
            super::sort::sort_layout(q)
                .correct
                .into_iter()
                .map(|i| i.item)
                .collect()
        } else {
            items.clone()
        };
        d.items = source
            .into_iter()
            .map(|i| DraftItem {
                text: i.text,
                letter: i.letter.unwrap_or_default(),
                media: i.media.unwrap_or_default(),
            })
            .collect();
    }
    d
}
pub fn empty_draft(kind: &str) -> Draft {
    let mut row = vec![String::new(); 12];
    row[3] = kind.into();
    let mut d = draft_from(&build_questions(&[row]).remove(0));
    if kind == "Top5" {
        d.answers = vec![String::new(); 5];
    }
    d
}
pub fn rows_for(d: &Draft, number: f64) -> Rows {
    let n = if number == 0.0 || number.is_nan() {
        1.0
    } else {
        number
    };
    let row =
        |i: usize, text: &str, kind: &str, options: Vec<String>, answer: &str, media: &str| {
            let mut r = vec![
                String::new(),
                (n + i as f64).to_string(),
                text.into(),
                kind.into(),
            ];
            r.extend(options);
            r.push(answer.into());
            r.push(media.into());
            r.resize(12, String::new());
            r
        };
    let none = || vec![String::new(); 4];
    if d.kind == "Top5" {
        let mut answers: Vec<_> = d
            .answers
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        let kind = if answers.len() == 5 || answers.is_empty() {
            "Top5".into()
        } else {
            format!("Top{}", answers.len())
        };
        if answers.is_empty() {
            answers.push("");
        }
        return answers
            .iter()
            .enumerate()
            .map(|(i, a)| {
                row(
                    i,
                    if i == 0 { &d.text } else { "" },
                    if i == 0 { &kind } else { "" },
                    none(),
                    a,
                    if i == 0 { d.media.trim() } else { "" },
                )
            })
            .collect();
    }
    if d.kind == "Sort" {
        let mut items: Vec<_> = d
            .items
            .iter()
            .filter(|i| !i.text.trim().is_empty() || !i.media.trim().is_empty())
            .cloned()
            .collect();
        let mut letters: Vec<_> = items.iter().map(|i| i.letter.clone()).collect();
        letters.sort();
        if letters
            != (0..items.len())
                .map(|i| char::from_u32(65 + i as u32).unwrap().to_string())
                .collect::<Vec<_>>()
        {
            items = with_letters(&items, &mut || 0.0);
        }
        if items.is_empty() {
            items.push(DraftItem {
                text: String::new(),
                media: String::new(),
                letter: String::new(),
            });
        }
        return items
            .iter()
            .enumerate()
            .map(|(i, it)| {
                row(
                    i,
                    if i == 0 { &d.text } else { "" },
                    if i == 0 { "Sort" } else { "" },
                    vec![
                        it.text.trim().into(),
                        String::new(),
                        String::new(),
                        String::new(),
                    ],
                    &it.letter,
                    it.media.trim(),
                )
            })
            .collect();
    }
    vec![row(
        0,
        &d.text,
        &d.kind,
        if d.kind == "ABCD" {
            ["A", "B", "C", "D"]
                .iter()
                .map(|k| d.options.get(*k).cloned().unwrap_or_default())
                .collect()
        } else {
            none()
        },
        d.answer.trim(),
        d.media.trim(),
    )]
}
pub fn preview_question(d: &Draft) -> Question {
    build_questions(&rows_for(d, 1.0)).remove(0)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedStamp {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
}
pub fn parse_stamp(stamp: &str) -> Option<ParsedStamp> {
    let re = regex::Regex::new(r"^(\d{4})-(\d{2})-(\d{2})(?:[ T](\d{2}):(\d{2}))?$").ok()?;
    let c = re.captures(stamp)?;
    let year = c[1].parse().ok()?;
    let month = c[2].parse().ok()?;
    let day = c[3].parse().ok()?;
    let hour = c.get(4).map_or(Some(0), |v| v.as_str().parse().ok())?;
    let minute = c.get(5).map_or(Some(0), |v| v.as_str().parse().ok())?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 {
        return None;
    }
    Some(ParsedStamp {
        year,
        month,
        day,
        hour,
        minute,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub author: String,
    pub changed: String,
    pub headers: Option<StampHeaders>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StampHeaders {
    pub author: String,
    pub changed: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fixup {
    pub row: usize,
    pub col: usize,
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub start: usize,
    pub delete_count: usize,
    pub insert: Rows,
    pub fixups: Vec<Fixup>,
    pub next: Rows,
}
fn pad(row: &[String]) -> Vec<String> {
    (0..12)
        .map(|i| row.get(i).cloned().unwrap_or_default())
        .collect()
}
fn stamped(rows: Rows, meta: &Meta) -> Rows {
    rows.iter()
        .enumerate()
        .map(|(i, r)| {
            let mut r = pad(r);
            r[10] = if i == 0 {
                meta.author.clone()
            } else {
                String::new()
            };
            r[11] = if i == 0 {
                meta.changed.clone()
            } else {
                String::new()
            };
            r
        })
        .collect()
}
fn finish(sheet: &Sheet, ri: Option<usize>, mut p: Plan, meta: &Meta, renumber: bool) -> Plan {
    p.next = sheet.rows[..p.start]
        .iter()
        .chain(&p.insert)
        .chain(&sheet.rows[p.start + p.delete_count..])
        .cloned()
        .collect();
    for f in &p.fixups {
        p.next[f.row] = pad(&p.next[f.row]);
        p.next[f.row][f.col] = f.value.clone();
    }
    let initial_fixups = p.fixups.clone();
    let mut set = |row: usize, col: usize, value: String| {
        if let Some(f) = p.fixups.iter_mut().find(|f| f.row == row && f.col == col) {
            f.value = value.clone();
        } else {
            p.fixups.push(Fixup {
                row,
                col,
                value: value.clone(),
            });
        }
        p.next[row] = pad(&p.next[row]);
        p.next[row][col] = value;
    };
    if renumber && let Some(ri) = ri {
        let old = &sheet.rounds[ri];
        let consecutive = old
            .questions
            .iter()
            .flat_map(|b| &b.rows)
            .enumerate()
            .all(|(i, r)| cell(r, 1).parse::<f64>().unwrap_or(0.0) == (i + 1) as f64);
        if consecutive {
            let mut next = sheet.rows[..p.start]
                .iter()
                .chain(&p.insert)
                .chain(&sheet.rows[p.start + p.delete_count..])
                .cloned()
                .collect::<Rows>();
            for f in &initial_fixups {
                next[f.row] = pad(&next[f.row]);
                next[f.row][f.col] = f.value.clone();
            }
            if let Some(round) = read_sheet(&next).rounds.get(ri) {
                let mut n = 1;
                for b in &round.questions {
                    for (r, row) in next.iter().enumerate().take(b.end + 1).skip(b.start) {
                        if cell(row, 1) != n.to_string() {
                            set(r, 1, n.to_string());
                        }
                        n += 1;
                    }
                }
            }
        }
    }
    if let Some(h) = &meta.headers
        && sheet.header_row >= 0
    {
        let r = sheet.header_row as usize;
        for (c, v) in [(10, &h.author), (11, &h.changed)] {
            if cell(&sheet.rows[r], c).is_empty() {
                set(r, c, v.clone());
            }
        }
    }
    p
}
fn replacement(start: usize, delete_count: usize, insert: Rows) -> Plan {
    Plan {
        start,
        delete_count,
        insert,
        fixups: vec![],
        next: vec![],
    }
}
pub fn plan_save(s: &Sheet, ri: usize, qi: usize, d: &Draft, m: &Meta) -> Option<Plan> {
    let b = s.rounds.get(ri)?.questions.get(qi)?;
    let mut rows = stamped(rows_for(d, cell(&b.rows[0], 1).parse().unwrap_or(1.0)), m);
    rows[0][0] = b.rows[0].first().cloned().unwrap_or_default();
    Some(finish(
        s,
        Some(ri),
        replacement(b.start, b.end - b.start + 1, rows),
        m,
        true,
    ))
}
pub fn plan_add(s: &Sheet, ri: usize, d: &Draft, m: &Meta) -> Option<Plan> {
    let r = s.rounds.get(ri)?;
    let last = r.questions.last();
    let n = last.map_or(1.0, |b| {
        cell(&s.rows[b.end], 1).parse::<f64>().unwrap_or(f64::NAN) + 1.0
    });
    let mut rows = stamped(rows_for(d, n), m);
    let after = last.map_or(r.marker_row, |b| b.end + 1);
    let end = s.rounds.get(ri + 1).map_or(s.rows.len(), |r| r.marker_row);
    let prepared = |i: usize| {
        i < end
            && (2..12).all(|c| cell(&s.rows[i], c).is_empty())
            && (i == r.marker_row || cell(&s.rows[i], 0).is_empty())
    };
    let start = (after..end).find(|&i| prepared(i)).unwrap_or(after);
    let free = (0..rows.len()).take_while(|i| prepared(start + i)).count();
    if start == r.marker_row {
        rows[0][0] = r.marker.clone();
    }
    Some(finish(s, Some(ri), replacement(start, free, rows), m, true))
}
pub fn plan_delete(s: &Sheet, ri: usize, qi: usize, m: &Meta) -> Option<Plan> {
    let r = s.rounds.get(ri)?;
    let b = r.questions.get(qi)?;
    let end = r.questions.last()?.end;
    let mut content = s.rows[b.end + 1..=end]
        .iter()
        .map(|r| pad(r))
        .collect::<Rows>();
    content.extend(vec![vec![String::new(); 12]; b.end - b.start + 1]);
    let rows = s.rows[b.start..=end]
        .iter()
        .zip(content)
        .map(|(r, mut c)| {
            c[0] = r.first().cloned().unwrap_or_default();
            c[1] = cell(r, 1).into();
            c
        })
        .collect();
    Some(finish(
        s,
        Some(ri),
        replacement(b.start, end - b.start + 1, rows),
        m,
        true,
    ))
}
pub fn plan_rename(s: &Sheet, ri: usize, title: &str, m: &Meta) -> Option<Plan> {
    let r = s.rounds.get(ri)?;
    let mut p = replacement(r.marker_row, 0, vec![]);
    p.fixups.push(Fixup {
        row: r.marker_row,
        col: 0,
        value: round_marker(&r.word, &r.number, title),
    });
    Some(finish(s, Some(ri), p, m, false))
}
pub fn plan_add_round(s: &Sheet, title: &str, d: &Draft, m: &Meta) -> Plan {
    let last = s.rounds.last();
    let start = last.map_or((s.header_row + 1).max(0) as usize, |r| {
        r.questions.last().map_or(r.marker_row, |b| b.end) + 1
    });
    let n = s
        .rounds
        .iter()
        .map(|r| r.number.parse::<f64>().unwrap_or(0.0))
        .fold(0.0, f64::max)
        + 1.0;
    let mut rows = stamped(rows_for(d, 1.0), m);
    rows[0][0] = round_marker(last.map_or("Kolo", |r| &r.word), &n.to_string(), title);
    finish(s, None, replacement(start, 0, rows), m, true)
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockLocation {
    pub ri: isize,
    pub qi: isize,
}
pub fn find_block(
    s: &Sheet,
    rows: &[Vec<String>],
    near: Option<&BlockLocation>,
) -> Option<BlockLocation> {
    let default = BlockLocation { ri: -1, qi: -1 };
    let near = near.unwrap_or(&default);
    s.rounds
        .iter()
        .enumerate()
        .flat_map(|(ri, r)| {
            r.questions
                .iter()
                .enumerate()
                .filter(|(_, b)| same_rows(rows, &b.rows))
                .map(move |(qi, _)| BlockLocation {
                    ri: ri as isize,
                    qi: qi as isize,
                })
        })
        .min_by_key(|b| (b.ri - near.ri).abs() * 1000 + (b.qi - near.qi).abs())
}
pub fn find_round(s: &Sheet, marker: &str) -> Option<usize> {
    s.rounds.iter().position(|r| r.marker == marker)
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MoveDirection {
    Up,
    Down,
}
pub fn plan_move(s: &Sheet, ri: usize, qi: usize, dir: MoveDirection, m: &Meta) -> Option<Plan> {
    let r = s.rounds.get(ri)?;
    let i = match dir {
        MoveDirection::Up => qi.checked_sub(1)?,
        MoveDirection::Down => qi,
    };
    let a = r.questions.get(i)?;
    let b = r.questions.get(i + 1)?;
    let mut rows = b
        .rows
        .iter()
        .chain(&s.rows[a.end + 1..b.start])
        .chain(&a.rows)
        .map(|r| pad(r))
        .collect::<Rows>();
    let marker = a.rows[0].first().cloned().unwrap_or_default();
    for r in &mut rows {
        if r[0] == marker {
            r[0].clear();
        }
    }
    rows[0][0] = marker;
    let mut meta = m.clone();
    meta.headers = None;
    Some(finish(
        s,
        Some(ri),
        replacement(a.start, b.end - a.start + 1, rows),
        &meta,
        true,
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteTab {
    pub sheet_id: i64,
    pub stamps: StampColumns,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionRange {
    pub sheet_id: i64,
    pub dimension: String,
    pub start_index: usize,
    pub end_index: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionWrite {
    pub range: DimensionRange,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit_from_before: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StructureWrite {
    InsertDimension(DimensionWrite),
    DeleteDimension(DimensionWrite),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueRange {
    pub range: String,
    pub values: Vec<Vec<String>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetWrites {
    pub structure: Vec<StructureWrite>,
    pub raw: Vec<ValueRange>,
    pub entered: Vec<ValueRange>,
}
pub fn sheet_writes(tab: &WriteTab, name: &str, p: &Plan) -> SheetWrites {
    let mut w = SheetWrites {
        structure: vec![],
        raw: vec![],
        entered: vec![],
    };
    let column = |i: usize| {
        if i < 26 {
            ((65 + i) as u8 as char).to_string()
        } else {
            format!(
                "{}{}",
                (64 + i / 26) as u8 as char,
                (65 + i % 26) as u8 as char
            )
        }
    };
    let mapped = |i| match i {
        10 => tab.stamps.author,
        11 => tab.stamps.changed,
        _ => i,
    };
    if p.insert.len() != p.delete_count {
        let growing = p.insert.len() > p.delete_count;
        let at = p.start + p.delete_count.min(p.insert.len());
        let d = DimensionWrite {
            range: DimensionRange {
                sheet_id: tab.sheet_id,
                dimension: "ROWS".into(),
                start_index: at,
                end_index: p.start + p.delete_count.max(p.insert.len()),
            },
            inherit_from_before: growing.then_some(at > 0),
        };
        w.structure.push(if growing {
            StructureWrite::InsertDimension(d)
        } else {
            StructureWrite::DeleteDimension(d)
        });
    }
    if !p.insert.is_empty() {
        w.raw.push(ValueRange {
            range: format!("{name}!A{}:J{}", p.start + 1, p.start + p.insert.len()),
            values: p.insert.iter().map(|r| pad(r)[..10].to_vec()).collect(),
        });
        for c in [10, 11] {
            let col = column(mapped(c));
            w.raw.push(ValueRange {
                range: format!(
                    "{name}!{col}{}:{col}{}",
                    p.start + 1,
                    p.start + p.insert.len()
                ),
                values: p.insert.iter().map(|r| vec![pad(r)[c].clone()]).collect(),
            });
        }
    }
    for (i, r) in p.insert.iter().enumerate() {
        w.entered.push(ValueRange {
            range: format!("{name}!B{}", p.start + i + 1),
            values: vec![vec![pad(r)[1].clone()]],
        });
    }
    for f in &p.fixups {
        let v = ValueRange {
            range: format!("{name}!{}{}", column(mapped(f.col)), f.row + 1),
            values: vec![vec![f.value.clone()]],
        };
        if f.col == 1 {
            w.entered.push(v);
        } else {
            w.raw.push(v);
        }
    }
    w
}
