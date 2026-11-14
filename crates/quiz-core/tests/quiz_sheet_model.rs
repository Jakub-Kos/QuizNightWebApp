use quiz_core::quiz::sheet_model::*;
use serde_json::{Value, from_value, to_value};
#[test]
fn javascript_parity() {
    let f: Value = serde_json::from_str(include_str!("fixtures/quiz/sheet-model.json")).unwrap();
    for (i, s) in f["sheets"].as_array().unwrap().iter().enumerate() {
        let rows: Vec<Vec<String>> = from_value(s["rows"].clone()).unwrap();
        let cols = stamp_columns(&rows);
        assert_eq!(to_value(&cols).unwrap(), s["cols"], "sheet {i} columns");
        let model = to_model_rows(&rows, &cols);
        assert_eq!(to_value(&model).unwrap(), s["model"], "sheet {i} rows");
        assert_eq!(
            to_value(read_sheet(&model)).unwrap(),
            s["sheet"],
            "sheet {i}"
        );
        for (j, q) in s["questions"].as_array().unwrap().iter().enumerate() {
            let d = draft_from(&from_value(q["question"].clone()).unwrap());
            assert_eq!(
                to_value(&d).unwrap(),
                q["draft"],
                "sheet {i} question {j} draft"
            );
            assert_eq!(
                to_value(rows_for(&d, 1.0)).unwrap(),
                q["rows"],
                "sheet {i} question {j} rows"
            );
            assert_eq!(
                to_value(preview_question(&d)).unwrap(),
                q["preview"],
                "sheet {i} question {j} preview"
            );
        }
    }
    for c in f["letters"].as_array().unwrap() {
        let items: Vec<DraftItem> = from_value(c["items"].clone()).unwrap();
        assert_eq!(
            to_value(with_letters(&items, &mut || 0.0)).unwrap(),
            c["output"]
        );
    }
    for c in f["empty"].as_array().unwrap() {
        assert_eq!(
            to_value(empty_draft(c["type"].as_str().unwrap())).unwrap(),
            c["output"]
        );
    }
    for c in f["same"].as_array().unwrap() {
        assert_eq!(
            same_rows(
                &from_value::<Vec<Vec<String>>>(c["a"].clone()).unwrap(),
                &from_value::<Vec<Vec<String>>>(c["b"].clone()).unwrap()
            ),
            c["output"].as_bool().unwrap()
        );
    }
}
