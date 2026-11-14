use quiz_core::quiz::parse_questions_csv;
#[test]
fn javascript_csv_fixtures() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/quiz/questions.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let actual = parse_questions_csv(case["input"].as_str().unwrap());
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            case["output"],
            "{}",
            case["name"]
        );
    }
}
#[test]
fn other_fixtures() {
    use quiz_core::quiz::{examples::*, parse::*, sort::*};
    for c in serde_json::from_str::<serde_json::Value>(include_str!("fixtures/quiz/markers.json"))
        .unwrap()
        .as_array()
        .unwrap()
    {
        assert_eq!(
            serde_json::to_value(split_marker(c["input"].as_str().unwrap())).unwrap(),
            c["output"]
        );
    }
    for c in serde_json::from_str::<serde_json::Value>(include_str!("fixtures/quiz/scores.json"))
        .unwrap()
        .as_array()
        .unwrap()
    {
        assert_eq!(
            serde_json::to_value(parse_scores_csv(c["input"].as_str().unwrap())).unwrap(),
            c["output"]
        );
    }
    for c in serde_json::from_str::<serde_json::Value>(include_str!("fixtures/quiz/sort.json"))
        .unwrap()
        .as_array()
        .unwrap()
    {
        let q: Question = serde_json::from_value(c["input"].clone()).unwrap();
        assert_eq!(serde_json::to_value(sort_layout(&q)).unwrap(), c["output"]);
    }
    for c in serde_json::from_str::<serde_json::Value>(include_str!("fixtures/quiz/examples.json"))
        .unwrap()
        .as_array()
        .unwrap()
    {
        assert_eq!(
            serde_json::to_value(example_questions(
                c["lang"].as_str().unwrap(),
                c["title"].as_str().unwrap()
            ))
            .unwrap(),
            c["output"]
        );
    }
}
