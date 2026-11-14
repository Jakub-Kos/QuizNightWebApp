use quiz_core::quiz::{
    drive::*,
    drive_config::*,
    model::{Quiz, Team},
    sheets::*,
};
use serde_json::{Value, json};
fn values(teams: Vec<Vec<&str>>, prepared: usize) -> Vec<Vec<String>> {
    let mut v = vec![
        vec!["Výsledky".into()],
        vec![
            "#".into(),
            "Název týmu".into(),
            "Kolo 1".into(),
            "Kolo 2".into(),
            "Celkem".into(),
        ],
    ];
    for (i, t) in teams.iter().enumerate() {
        v.push(
            std::iter::once((i + 1).to_string())
                .chain(t.iter().map(|s| s.to_string()))
                .collect(),
        );
    }
    for i in 0..prepared {
        v.push(vec![
            (teams.len() + i + 1).to_string(),
            "".into(),
            "".into(),
            "".into(),
            "0".into(),
        ]);
    }
    v
}
fn row(r: usize, n: &str, id: Option<&str>) -> ScoreRow {
    ScoreRow {
        row: r,
        name: n.into(),
        id: id.map(str::to_owned),
    }
}
fn entry(n: &str) -> Value {
    json!({"name":n,"motto":format!("{n}!")})
}
fn rec(
    rows: &[ScoreRow],
    entries: Value,
    base: Option<Value>,
    local: Value,
    w: bool,
) -> Reconciled {
    let mut i = 0;
    reconcile(rows, &entries, base.as_ref(), &local, w, &mut || {
        i += 1;
        format!("generated{i}")
    })
}
#[test]
fn scores_table_header_rows_and_empty() {
    let t = scores_table(&values(
        vec![vec!["Sovy", "1", "2"], vec!["Lišky", "3", "1"]],
        2,
    ))
    .unwrap();
    assert_eq!((t.header, t.name_col, t.id_col, t.end), (1, 1, -1, 6));
    assert_eq!(t.rows, vec![row(2, "Sovy", None), row(3, "Lišky", None)]);
    assert_eq!(t.empty, vec![4, 5]);
}
#[test]
fn scores_table_no_names() {
    assert!(scores_table(&[vec!["Kolo".into(), "Q#".into()]]).is_none());
}
#[test]
fn hidden_id_column() {
    let s = scores_table(&values(vec![vec!["Sovy"]], 2)).unwrap();
    let w = scores_writes(
        &s,
        3,
        "'Body'",
        &Ops {
            ids: vec![json!({"row":2,"id":"t1"})],
            ..Default::default()
        },
    );
    assert!(
        w["cells"]
            .as_array()
            .unwrap()
            .contains(&json!({"range":"'Body'!F2","values":[["quiznight-id"]]}))
    );
    assert!(
        w["cells"]
            .as_array()
            .unwrap()
            .contains(&json!({"range":"'Body'!F3","values":[["t1"]]}))
    );
    assert_eq!(
        w["structure"][0]["updateDimensionProperties"]["range"]["startIndex"],
        5
    );
    assert_eq!(
        w["structure"][0]["updateDimensionProperties"]["properties"]["hiddenByUser"],
        true
    );
}
#[test]
fn new_team_uses_prepared_row() {
    let s = scores_table(&values(vec![vec!["Sovy"]], 2)).unwrap();
    let w = scores_writes(
        &s,
        3,
        "'Body'",
        &Ops {
            appends: vec![json!({"name":"Nový","id":"t9"})],
            ..Default::default()
        },
    );
    assert!(
        w["structure"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r.get("insertDimension").is_none())
    );
    assert!(
        w["cells"]
            .as_array()
            .unwrap()
            .contains(&json!({"range":"'Body'!B4","values":[["Nový"]]}))
    );
}
#[test]
fn inserts_below_table() {
    let s = scores_table(&values(vec![vec!["Sovy"]], 1)).unwrap();
    let w = scores_writes(
        &s,
        3,
        "'Body'",
        &Ops {
            appends: vec![
                json!({"name":"A","id":"a"}),
                json!({"name":"B","id":"b"}),
                json!({"name":"C","id":"c"}),
            ],
            ..Default::default()
        },
    );
    let r = &w["structure"][1]["insertDimension"]["range"];
    assert_eq!(r["startIndex"], 4);
    assert_eq!(r["endIndex"], 6);
    let names: Vec<_> = w["cells"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["range"].as_str().unwrap().contains("!B"))
        .map(|c| c["range"].clone())
        .collect();
    assert_eq!(
        names,
        vec![json!("'Body'!B4"), json!("'Body'!B5"), json!("'Body'!B6")]
    );
}
#[test]
fn removal_keeps_points() {
    let mut v = values(vec![vec!["Sovy", "1"], vec!["Lišky", "2"]], 2);
    v[1].push("quiznight-id".into());
    let s = scores_table(&v).unwrap();
    let w = scores_writes(
        &s,
        3,
        "'Body'",
        &Ops {
            clears: vec![2],
            ..Default::default()
        },
    );
    assert_eq!(
        w,
        json!({"structure":[],"cells":[{"range":"'Body'!B3","values":[[""]]},{"range":"'Body'!F3","values":[[""]]}]})
    );
}
#[test]
fn sheet_rename_keeps_details() {
    let e = json!({"t1":entry("Sovy")});
    let r = rec(
        &[row(2, "Sovy noci", Some("t1"))],
        e.clone(),
        Some(e.clone()),
        e,
        true,
    );
    assert_eq!(r.entries["t1"], json!({"name":"Sovy noci","motto":"Sovy!"}));
    assert_eq!(r.ops, Ops::default());
}
#[test]
fn archive_and_reconnect() {
    let e = json!({"t1":entry("Sovy"),"t2":entry("Lišky")});
    let gone = rec(
        &[row(2, "Lišky", Some("t2"))],
        e.clone(),
        Some(e.clone()),
        e,
        true,
    );
    assert_eq!(gone.entries["t1"]["removed"], true);
    assert_eq!(gone.order, vec!["t2"]);
    let back = rec(
        &[row(2, "Lišky", Some("t2")), row(3, "sovy", None)],
        gone.entries,
        None,
        json!({}),
        true,
    );
    assert_eq!(back.order, vec!["t2", "t1"]);
    assert_eq!(back.entries["t1"], json!({"name":"sovy","motto":"Sovy!"}));
    assert_eq!(back.ops.ids, vec![json!({"row":3,"id":"t1"})]);
}
#[test]
fn local_renames_removals_and_additions() {
    let base = json!({"t1":entry("Sovy"),"t2":entry("Lišky")});
    let local = json!({"t1":entry("Noční sovy"),"t3":entry("Nový")});
    let r = rec(
        &[row(2, "Sovy", Some("t1")), row(3, "Lišky", Some("t2"))],
        json!({"t1":local["t1"],"t2":base["t2"],"t3":local["t3"]}),
        Some(base),
        local,
        true,
    );
    assert_eq!(r.ops.names, vec![json!({"row":2,"name":"Noční sovy"})]);
    assert_eq!(r.ops.clears, vec![3]);
    assert_eq!(r.ops.appends, vec![json!({"name":"Nový","id":"t3"})]);
    assert_eq!(r.entries["t2"]["removed"], true);
    assert_eq!(r.order, vec!["t1", "t3"]);
}
#[test]
fn viewer_never_writes_placeholder() {
    let r = rec(
        &[row(2, "Sovy", None)],
        json!({"name:sovy":entry("Sovy")}),
        Some(json!({"t1":entry("Sovy")})),
        json!({"t1":entry("Sovy noci")}),
        false,
    );
    assert_eq!(r.ops, Ops::default());
    assert_eq!(r.order, vec!["name:sovy"]);
    let r = rec(
        &[row(2, "Sovy", None)],
        json!({"name:sovy":entry("Sovy")}),
        None,
        json!({}),
        true,
    );
    assert_eq!(r.ops.ids.len(), 1);
    let id = r.ops.ids[0]["id"].as_str().unwrap();
    assert!(!id.starts_with("name:"));
    assert_eq!(r.entries[id]["motto"], "Sovy!");
}
#[test]
fn merge_missing_null_arrays_and_conflicts() {
    let b = json!({"a":1,"nested":{"x":1,"y":1},"gone":1,"array":[1]});
    let l = json!({"a":2,"nested":{"x":1,"y":2},"array":[2],"null":null});
    let r = json!({"a":3,"nested":{"x":2,"y":1},"gone":1,"array":[3]});
    assert_eq!(
        merge3(Some(&b), Some(&l), Some(&r)),
        Some(json!({"a":2,"nested":{"x":2,"y":2},"array":[2],"null":null}))
    );
}
#[test]
fn config_roundtrip_archive_drafts_upgrade() {
    let mut q = Quiz::new("q", 1);
    q.title = "Local".into();
    q.teams = vec![
        serde_json::from_value::<Team>(json!({"id":"t1","name":" Sovy ","draft":false})).unwrap(),
        serde_json::from_value::<Team>(json!({"id":"draft","name":"Draft","draft":true})).unwrap(),
    ];
    q.fields
        .insert("teamArchive".into(), json!({"old":{"name":"Old"}}));
    let c = config_from_quiz(&q);
    assert_eq!(c["teams"]["t1"]["name"], "Sovy");
    assert_eq!(c["teams"]["old"]["removed"], true);
    assert!(c["teams"].get("draft").is_none());
    let applied = apply_config(&q, &c, None);
    assert_eq!(applied.teams.len(), 2);
    assert_eq!(applied.fields["teamArchive"]["old"]["name"], "Old");
    assert_eq!(
        upgrade(json!({"version":1,"teams":{" Sovy ":{"motto":"x"}}}))["teams"]["name:sovy"]["name"],
        " Sovy "
    );
}
#[test]
fn links_and_config() {
    let id = "abcdefghijklmnopqrst";
    for u in [
        id.to_owned(),
        format!("https://drive.google.com/drive/folders/{id}"),
        format!("https://docs.google.com/spreadsheets/d/{id}/edit"),
        format!("https://drive.google.com/open?id={id}"),
    ] {
        assert_eq!(parse_drive_link(&u), Some(id.into()));
    }
    for u in [
        "",
        "bad",
        "https://docs.google.com/spreadsheets/d/e/published/pub",
    ] {
        assert_eq!(parse_drive_link(u), None);
    }
    assert_eq!(
        google_copy_url(&sheet_url(id, Some(7))),
        format!("https://docs.google.com/spreadsheets/d/{id}/copy")
    );
    assert_eq!(google_copy_url("unchanged"), "unchanged");
    assert_eq!(
        folder_url(id),
        format!("https://drive.google.com/drive/folders/{id}")
    );
    for n in [" Zdroje ", "Zdroj", "SOURCE", "sources", "Media"] {
        assert!(is_media_folder_name(n));
    }
    assert!(!is_media_folder_name("assets"));
    assert!(same_drive_quiz(
        &json!({"folderId":null,"spreadsheetId":"s"}),
        &json!({"spreadsheetId":"s"})
    ));
    assert_eq!(column(26), "AA");
    assert_eq!(column(701), "ZZ");
}
#[test]
fn csv_request_escaping() {
    assert_eq!(
        rows_to_csv(&[
            vec![" a ".into(), "x,y".into(), "\"".into()],
            vec!["n\nr".into(), "".into()]
        ]),
        "\" a \",\"x,y\",\"\"\"\"\r\n\"n\nr\","
    );
    assert_eq!(tab_range("It's"), "'It''s'");
}
#[test]
fn google_request_encoding() {
    assert_eq!(
        encode_component("'It's a tab'!A1:B2"),
        "'It's%20a%20tab'!A1%3AB2"
    );
    assert_eq!(encode_component("č"), "%C4%8D");
    assert_eq!(
        form_query(&[
            ("q", "'folder' in parents".into()),
            ("fields", "files(id,name)".into())
        ]),
        "q=%27folder%27+in+parents&fields=files%28id%2Cname%29"
    );
}
#[test]
fn drive_model_and_extension_fields_roundtrip() {
    let raw = json!({"id":"q","drive":{"folderId":"f","spreadsheetId":"s","mediaFolderId":"m","configFileId":"c","syncedAt":123,"syncedBy":"a@b","configBase":{"version":2}},"teams":[{"id":"t1","name":"Sovy","draft":false,"motto":"x"}],"teamArchive":{"old":{"name":"Old"}}});
    let q: Quiz = serde_json::from_value(raw.clone()).unwrap();
    let stored = serde_json::to_value(q).unwrap();
    for k in ["drive", "teamArchive"] {
        assert_eq!(stored[k], raw[k]);
    }
    assert_eq!(stored["teams"][0]["id"], "t1");
    assert_eq!(stored["teams"][0]["draft"], false);
    assert_eq!(stored["teams"][0]["motto"], "x");
}
