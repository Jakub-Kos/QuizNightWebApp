use quiz_core::quiz::sheet_model::*;
use serde::de::DeserializeOwned;
use serde_json::{Value, from_value, to_value};
fn arg<T: DeserializeOwned>(a: &Value, i: usize) -> T {
    from_value(a[i].clone()).unwrap()
}
#[test]
fn javascript_parity() {
    let cases: Value =
        serde_json::from_str(include_str!("fixtures/quiz/sheet-plans.json")).unwrap();
    let sheets = cases["sheets"].clone();
    for c in cases["cases"].as_array().unwrap() {
        // The sheet a case starts from is stored once, as `{ "$sheet": index }`.
        let mut args = c["args"].clone();
        if let Some(index) = args[0]["$sheet"].as_u64() {
            args[0] = sheets[index as usize].clone();
        }
        let a = &args;
        let f = c["fn"].as_str().unwrap();
        let output = match f {
            "planSave" => to_value(plan_save(
                &arg(a, 0),
                arg(a, 1),
                arg(a, 2),
                &arg(a, 3),
                &arg(a, 4),
            )),
            "planAdd" => to_value(plan_add(&arg(a, 0), arg(a, 1), &arg(a, 2), &arg(a, 3))),
            "planDelete" => to_value(plan_delete(&arg(a, 0), arg(a, 1), arg(a, 2), &arg(a, 3))),
            "planRename" => to_value(plan_rename(
                &arg(a, 0),
                arg(a, 1),
                &arg::<String>(a, 2),
                &arg(a, 3),
            )),
            "planAddRound" => to_value(plan_add_round(
                &arg(a, 0),
                &arg::<String>(a, 1),
                &arg(a, 2),
                &arg(a, 3),
            )),
            "planMove" => to_value(plan_move(
                &arg(a, 0),
                arg(a, 1),
                arg(a, 2),
                if arg::<i32>(a, 3) < 0 {
                    MoveDirection::Up
                } else {
                    MoveDirection::Down
                },
                &arg(a, 4),
            )),
            "findBlock" => to_value(find_block(
                &arg(a, 0),
                &arg::<Vec<Vec<String>>>(a, 1),
                Some(&arg(a, 2)),
            )),
            "findRound" => {
                to_value(find_round(&arg(a, 0), &arg::<String>(a, 1)).map_or(-1, |i| i as i64))
            }
            "sheetWrites" => to_value(sheet_writes(&arg(a, 0), &arg::<String>(a, 1), &arg(a, 2))),
            _ => panic!("unknown function {f}"),
        }
        .unwrap();
        assert_eq!(output, c["output"], "{} {}", f, c["name"]);
    }
}

fn strings(s: &[&str]) -> Vec<String> {
    s.iter().map(|s| (*s).into()).collect()
}
fn tab() -> Vec<Vec<String>> {
    let mut rows = vec![
        strings(&["Kvíz 2026"]),
        strings(&[
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
        ]),
        strings(&[
            "Kolo 1 - Zeměpis",
            "1",
            "Hlavní město Francie?",
            "Written",
            "",
            "",
            "",
            "",
            "Paříž",
        ]),
        strings(&[
            "",
            "2",
            "Kolik nohou má pavouk?",
            "Numeric",
            "",
            "",
            "",
            "",
            "8",
        ]),
        strings(&[
            "",
            "3",
            "Největší planeta?",
            "ABCD",
            "Mars",
            "Jupiter",
            "Venuše",
            "Merkur",
            "B",
        ]),
    ];
    for n in 4..=10 {
        rows.push(vec!["".into(), n.to_string()]);
    }
    rows.push(strings(&[
        "Kolo 2 - Hudba",
        "1",
        "Kdo zpívá?",
        "Audio",
        "",
        "",
        "",
        "",
        "Queen",
        "pisen.mp3",
    ]));
    rows.push(strings(&[
        "",
        "2",
        "Pět největších zemí",
        "Top5",
        "",
        "",
        "",
        "",
        "Rusko",
    ]));
    for (i, a) in ["Kanada", "Čína", "USA", "Brazílie"].iter().enumerate() {
        rows.push(vec![
            "".into(),
            (3 + i).to_string(),
            "".into(),
            "".into(),
            "".into(),
            "".into(),
            "".into(),
            "".into(),
            (*a).into(),
        ]);
    }
    rows
}
fn sheet_of(rows: &[Vec<String>]) -> Sheet {
    read_sheet(&to_model_rows(rows, &stamp_columns(rows)))
}
fn meta() -> Meta {
    Meta {
        author: "Tester".into(),
        changed: "2026-10-23 12:00".into(),
        headers: Some(StampHeaders {
            author: "Autor".into(),
            changed: "Upraveno".into(),
        }),
    }
}
fn draft(text: &str, answer: &str) -> Draft {
    let mut d = empty_draft("Written");
    d.text = text.into();
    d.answer = answer.into();
    d
}
fn writes(p: &Plan) -> SheetWrites {
    sheet_writes(
        &WriteTab {
            sheet_id: 7,
            stamps: StampColumns {
                author: 10,
                changed: 11,
            },
        },
        "'Q'",
        p,
    )
}
fn texts(s: &Sheet, ri: usize) -> Vec<&str> {
    s.rounds[ri]
        .questions
        .iter()
        .map(|b| b.question.text.as_str())
        .collect()
}
#[test]
fn read_sheet_groups_like_parser() {
    let rows = tab();
    let s = sheet_of(&rows);
    let csv = rows
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n");
    let parsed = quiz_core::quiz::parse::parse_questions_csv(&csv);
    assert_eq!(s.header_row, 1);
    assert_eq!(
        s.rounds
            .iter()
            .map(|r| r
                .questions
                .iter()
                .map(|b| &b.question.text)
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        parsed
            .iter()
            .map(|r| r.questions.iter().map(|q| &q.text).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
    assert_eq!(s.rounds[1].questions[1].rows.len(), 5);
}
#[test]
fn add_fills_prepared_row() {
    let s = sheet_of(&tab());
    let p = plan_add(&s, 0, &draft("Nová otázka", "x"), &meta()).unwrap();
    assert_eq!(p.start, 5);
    assert_eq!(p.delete_count, 1);
    assert_eq!(p.next.len(), s.rows.len());
    assert!(writes(&p).structure.is_empty());
    assert_eq!(p.next[5][1..3], strings(&["4", "Nová otázka"]));
    assert_eq!(p.next[5][10], "Tester");
}
#[test]
fn add_inserts_only_overflow() {
    let rows = tab()
        .into_iter()
        .filter(|r| !(r.len() == 2 && (4..=9).contains(&r[1].parse::<i32>().unwrap_or(0))))
        .collect::<Vec<_>>();
    let s = sheet_of(&rows);
    let mut d = empty_draft("Top5");
    d.text = "Top list".into();
    d.answers = strings(&["a", "b", "c"]);
    let p = plan_add(&s, 0, &d, &meta()).unwrap();
    assert_eq!(p.delete_count, 1);
    assert_eq!(p.insert.len(), 3);
    let w = writes(&p);
    assert_eq!(w.structure.len(), 1);
    let StructureWrite::InsertDimension(d) = &w.structure[0] else {
        panic!("expected insertion")
    };
    assert_eq!(d.range.start_index, p.start + 1);
    assert_eq!(d.range.end_index, p.start + 3);
    assert_eq!(texts(&read_sheet(&p.next), 1), texts(&s, 1));
}
#[test]
fn add_without_prepared_rows() {
    let s = sheet_of(&tab());
    let p = plan_add(&s, 1, &draft("Další", "x"), &meta()).unwrap();
    assert_eq!(p.start, s.rounds[1].questions.last().unwrap().end + 1);
    assert_eq!(p.delete_count, 0);
    assert_eq!(texts(&read_sheet(&p.next), 1).last(), Some(&"Další"));
}
#[test]
fn delete_leaves_prepared_rows() {
    let s = sheet_of(&tab());
    let p = plan_delete(&s, 0, 0, &meta()).unwrap();
    assert_eq!(p.insert.len(), p.delete_count);
    assert!(writes(&p).structure.is_empty());
    assert_eq!(p.next.len(), s.rows.len());
    let after = read_sheet(&p.next);
    assert_eq!(after.rounds[0].marker, "Kolo 1 - Zeměpis");
    assert_eq!(
        texts(&after, 0),
        vec!["Kolik nohou má pavouk?", "Největší planeta?"]
    );
    assert_eq!(p.next[2][0], "Kolo 1 - Zeměpis");
    assert_eq!(
        p.next[2..5]
            .iter()
            .map(|r| r[1].as_str())
            .collect::<Vec<_>>(),
        vec!["1", "2", "3"]
    );
    assert!(p.next[4][2..].iter().all(String::is_empty));
}
#[test]
fn delete_top5() {
    let s = sheet_of(&tab());
    let p = plan_delete(&s, 1, 1, &meta()).unwrap();
    assert_eq!(p.insert.len(), p.delete_count);
    assert_eq!(p.next.len(), s.rows.len());
    assert_eq!(read_sheet(&p.next).rounds[1].questions.len(), 1);
}
#[test]
fn save_preserves_marker() {
    let p = plan_save(&sheet_of(&tab()), 0, 0, &draft("Změněná", "Lyon"), &meta()).unwrap();
    assert_eq!(
        p.next[2][..3],
        strings(&["Kolo 1 - Zeměpis", "1", "Změněná"])
    );
    assert_eq!(p.next[2][8], "Lyon");
}
#[test]
fn move_preserves_marker() {
    let p = plan_move(&sheet_of(&tab()), 0, 0, MoveDirection::Down, &meta()).unwrap();
    assert_eq!(
        &texts(&read_sheet(&p.next), 0)[..2],
        &["Kolik nohou má pavouk?", "Hlavní město Francie?"]
    );
    assert_eq!(p.next[2][0], "Kolo 1 - Zeměpis");
    assert_eq!(p.next[3][0], "");
}
#[test]
fn find_after_renumber_and_marker_move() {
    let s = sheet_of(&tab());
    let mut rows = tab();
    rows[3][1] = "7".into();
    rows[3][0] = "something".into();
    assert_eq!(
        find_block(
            &sheet_of(&rows),
            &s.rounds[0].questions[1].rows,
            Some(&BlockLocation { ri: 0, qi: 1 })
        ),
        Some(BlockLocation { ri: 0, qi: 1 })
    );
}
#[test]
fn find_changed_question() {
    let s = sheet_of(&tab());
    let mut rows = tab();
    rows[3][2] = "Edited in the Sheet".into();
    assert_eq!(
        find_block(&sheet_of(&rows), &s.rounds[0].questions[1].rows, None),
        None
    );
}
#[test]
fn default_stamp_columns() {
    assert_eq!(
        stamp_columns(&tab()),
        StampColumns {
            author: 10,
            changed: 11
        }
    );
}
#[test]
fn localized_stamp_headers() {
    let mut rows = tab();
    rows[1].resize(14, String::new());
    rows[1][10] = "Poznámky".into();
    rows[1][12] = "Szerző".into();
    rows[1][13] = "Módosítva".into();
    assert_eq!(
        stamp_columns(&rows),
        StampColumns {
            author: 12,
            changed: 13
        }
    );
}
#[test]
fn stamps_skip_used_columns() {
    let mut rows = tab();
    rows[3].resize(11, String::new());
    rows[3][10] = "check this one".into();
    assert_eq!(
        stamp_columns(&rows),
        StampColumns {
            author: 11,
            changed: 12
        }
    );
}
#[test]
fn save_only_writes_stamp_columns_past_j() {
    let mut rows = tab();
    rows[1].resize(11, String::new());
    rows[2].resize(11, String::new());
    rows[1][10] = "Notes".into();
    rows[2][10] = "moderator note".into();
    let stamps = stamp_columns(&rows);
    let s = read_sheet(&to_model_rows(&rows, &stamps));
    assert_eq!(s.rounds[0].questions[0].author, "");
    let p = plan_save(&s, 0, 0, &draft("Změněná", "x"), &meta()).unwrap();
    let w = sheet_writes(
        &WriteTab {
            sheet_id: 7,
            stamps,
        },
        "'Q'",
        &p,
    );
    assert!(
        w.raw
            .iter()
            .chain(&w.entered)
            .all(|v| !v.range.starts_with("'Q'!K"))
    );
    for (range, values) in [
        ("'Q'!L3:L3", vec![strings(&["Tester"])]),
        ("'Q'!M3:M3", vec![strings(&["2026-10-23 12:00"])]),
        ("'Q'!L2", vec![strings(&["Autor"])]),
        ("'Q'!A3:J3", vec![p.insert[0][..10].to_vec()]),
    ] {
        assert!(
            w.raw.iter().any(|v| v.range == range && v.values == values),
            "{range}"
        );
    }
    assert!(
        w.entered
            .iter()
            .any(|v| v.range == "'Q'!B3" && v.values == vec![strings(&["1"])])
    );
}
