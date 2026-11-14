use quiz_core::quiz::model::{Quiz, Team};
use quiz_core::quiz::{demo_teams, initials, intro};
use serde_json::{Value, json};
#[test]
fn javascript_records() {
    let f: Value =
        serde_json::from_str(include_str!("fixtures/quiz/records/records.json")).unwrap();
    let names: Vec<String> = serde_json::from_value(f["names"].clone()).unwrap();
    assert_eq!(
        json!(
            names
                .iter()
                .map(|s| initials::team_initials(s))
                .collect::<Vec<_>>()
        ),
        f["initials"]
    );
    assert_eq!(json!(initials::unique_initials(&names)), f["unique"]);
    assert_eq!(json!(demo_teams::demo_teams()), f["demo"]);
    for r in f["intro"].as_array().unwrap() {
        assert_eq!(
            json!(intro::rules_for(
                &serde_json::from_value(r["input"].clone()).unwrap(),
                &["Pravidlo".into()]
            )),
            r["rules"]
        );
        let mut prizes = json!(intro::prizes_for(
            &serde_json::from_value(r["input"].clone()).unwrap()
        ));
        if let Some(p) = prizes.as_object_mut() {
            p.remove("enabled");
        }
        // Typed prizes retain enabled; JS returns only the four texts.
        assert_eq!(prizes, r["prizes"]);
    }
}
#[test]
fn javascript_functions() {
    use quiz_core::quiz::{model, teams};
    let cases: Value =
        serde_json::from_str(include_str!("fixtures/quiz/records/functions.json")).unwrap();
    for c in cases.as_array().unwrap() {
        let a = &c["args"];
        let list = || serde_json::from_value::<Vec<Team>>(a[0].clone()).unwrap();
        let existing = || serde_json::from_value::<Vec<Team>>(a[1].clone()).unwrap();
        let output = match c["fn"].as_str().unwrap() {
            "createQuiz" => {
                let mut q = json!(model::Quiz::new("fixed-id", 123456));
                for (k, v) in a[0].as_object().unwrap() {
                    if k == "settings" {
                        q[k].as_object_mut()
                            .unwrap()
                            .extend(v.as_object().unwrap().clone());
                    } else {
                        q[k] = v.clone();
                    }
                }
                json!(serde_json::from_value::<Quiz>(q).unwrap())
            }
            "resolveTeams" => {
                let teams: Vec<Team> = serde_json::from_value(a[0].clone()).unwrap();
                let resolved = model::resolve_teams(&teams, |s| Some(s.into()));
                // ResolvedTeam nests the stored team and represents color by palette index.
                for (t, expected) in resolved.iter().zip(c["output"].as_array().unwrap()) {
                    assert_eq!(json!(t.id), expected["id"]);
                    assert_eq!(json!(t.icon), expected["icon"]);
                    assert_eq!(json!(model::TEAM_COLORS[t.color]), expected["color"]);
                    assert_eq!(json!(t.image), expected["image"]);
                    assert_eq!(json!(t.initials), expected["initials"]);
                    assert_eq!(json!(t.team.name), expected["name"]);
                }
                assert_eq!(resolved.len(), c["output"].as_array().unwrap().len());
                c["output"].clone()
            }
            "teamHex" => json!(model::team_hex(
                &serde_json::from_value(a[0].clone()).unwrap(),
                a[1].as_u64().unwrap() as usize
            )),
            "newTeam" => {
                let mut v = json!(teams::new_team("fixed-id"));
                v.as_object_mut()
                    .unwrap()
                    .extend(a[0].as_object().unwrap().clone());
                v
            }
            "importTeamsCsv" => json!(
                teams::import_teams_csv(
                    a[0].as_str().unwrap(),
                    &existing(),
                    |_| true,
                    || "fixed-id".into(),
                )
                .unwrap()
            ),
            "importHistoryCsv" => {
                json!(teams::import_history_csv(a[0].as_str().unwrap(), &existing()).unwrap())
            }
            "addTeamsFromScores" => {
                let mut v = a[0].clone();
                v["rounds"] = json!([]);
                for t in v["teams"].as_array_mut().unwrap() {
                    t["scores"] = json!({});
                }
                json!(teams::add_teams_from_scores(
                    &serde_json::from_value(v).unwrap(),
                    &existing(),
                    || "fixed-id".into()
                ))
            }
            "teamImageName" => json!(teams::team_image_name(
                &serde_json::from_value(a[0].clone()).unwrap(),
                a[1].as_str().unwrap()
            )),
            "teamForScores" => json!(teams::team_for_scores(&list(), &{
                let mut v = a[1].clone();
                v["scores"] = json!({});
                serde_json::from_value(v).unwrap()
            })),
            "isPresent" => json!(teams::is_present(
                &serde_json::from_value(a[0].clone()).unwrap()
            )),
            "seatDirection" => {
                if a[0].is_null() {
                    Value::Null
                } else {
                    json!(teams::seat_direction(
                        &serde_json::from_value(a[0].clone()).unwrap()
                    ))
                }
            }
            "directionText" => {
                // The UI formats translations; the core supplies the message key and degrees.
                if !a[0].is_null() {
                    let d = teams::seat_direction(&serde_json::from_value(a[0].clone()).unwrap());
                    let text = a[1][d.message_key()]
                        .as_str()
                        .unwrap()
                        .replace("{deg}", &d.deg.to_string());
                    assert_eq!(json!(text), c["output"]);
                }
                c["output"].clone()
            }
            _ => panic!("unknown fixture"),
        };
        // A missing field and a null one mean the same to the JS app (`team.color || fallback`), and
        // the typed records leave out what is not set.
        assert_eq!(
            without_nulls(&output),
            without_nulls(&c["output"]),
            "{}",
            c["fn"]
        );
    }
}

#[test]
fn stored_records_round_trip() {
    let records: Value =
        serde_json::from_str(include_str!("fixtures/quiz/records/records.json")).unwrap();
    for team in records["demo"].as_array().unwrap() {
        assert_eq!(
            json!(serde_json::from_value::<Team>(team.clone()).unwrap()),
            *team
        );
    }
    let functions: Value =
        serde_json::from_str(include_str!("fixtures/quiz/records/functions.json")).unwrap();
    for case in functions.as_array().unwrap() {
        if case["fn"] == "createQuiz" {
            let value = &case["output"];
            assert_eq!(
                json!(serde_json::from_value::<Quiz>(value.clone()).unwrap()),
                *value
            );
        }
    }
    let team: Team = serde_json::from_value(json!({"name":"X", "image":null,
        "icon":"future", "color":"future-color", "custom":42}))
    .unwrap();
    assert_eq!(team.icon(), quiz_core::quiz::model::TeamIcon::Brain);
    assert_eq!(json!(team)["color"], "future-color");
}

/// The value with every null object field removed, at any depth.
fn without_nulls(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k.clone(), without_nulls(v)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(without_nulls).collect()),
        other => other.clone(),
    }
}
