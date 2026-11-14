use quiz_core::quiz::{model::Team, parse::Round, scores};
use serde_json::{Value, json};

#[test]
fn javascript_scores() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/quiz/records/scores.json")).unwrap();
    for c in cases {
        let a = &c["args"];
        let rounds = |i: usize| serde_json::from_value::<Vec<Round>>(a[i].clone()).unwrap();
        let teams = || serde_json::from_value::<Vec<Team>>(a[0].clone()).unwrap();
        let word = || a[2].as_str();
        let output = match c["fn"].as_str().unwrap() {
            "scoredRounds" => json!(scores::scored_rounds(&rounds(0))),
            "manualLeaderboard" => json!(scores::manual_leaderboard(
                &teams(),
                &rounds(1),
                &serde_json::from_value(a[2].clone()).unwrap(),
                a[3].as_str()
            )),
            "sampleLeaderboard" => json!(scores::sample_leaderboard(
                &teams(),
                &serde_json::from_value::<Vec<String>>(a[1].clone()).unwrap(),
                word(),
                &serde_json::from_value::<Vec<String>>(a[3].clone()).unwrap()
            )),
            "hasPoints" => json!(scores::has_points(
                &serde_json::from_value(a[0].clone()).unwrap()
            )),
            "roundLabel" => {
                let round: Round = serde_json::from_value(a[0].clone()).unwrap();
                // The typed API supplies the number; the UI capitalizes the supplied word.
                let word = a[1].as_str().unwrap();
                let mut chars = word.chars();
                json!(format!(
                    "{}{} {}",
                    chars
                        .next()
                        .map(|c| c.to_uppercase().to_string())
                        .unwrap_or_default(),
                    chars.as_str().to_lowercase(),
                    scores::round_label(&round)
                ))
            }
            _ => panic!("unknown fixture"),
        };
        assert_eq!(
            without_nulls(&output),
            without_nulls(&c["output"]),
            "{} {}",
            c["fn"],
            c["name"]
        );
    }
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
