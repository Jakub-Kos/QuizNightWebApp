use quiz_core::quiz::{parse::Round, validate::*};
#[test]
fn validation_parity() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/quiz/validate.json")).unwrap();
    for c in cases.as_array().unwrap() {
        let quiz: ValidationInput = serde_json::from_value(c["quiz"].clone()).unwrap();
        let rounds: Vec<Round> = c["rounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                let mut r = r.clone();
                if r.get("rows").is_none() {
                    r["rows"] = serde_json::json!([]);
                }
                serde_json::from_value(r).unwrap()
            })
            .collect();
        assert_eq!(
            serde_json::to_value(validate_quiz(&quiz, &rounds, |f| c["files"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_str() == Some(f))))
            .unwrap(),
            c["output"]
        );
    }
}
