use quiz_core::theme::*;
use serde_json::Value;
#[test]
fn javascript_parity() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/theme/normalize.json")).unwrap();
    for c in cases {
        let options = NormalizeOptions {
            assets: serde_json::from_value(
                c["options"]["assets"]
                    .as_array()
                    .map(|a| Value::Array(a.clone()))
                    .unwrap_or(serde_json::json!([])),
            )
            .unwrap(),
            stored: c["options"]["stored"].as_bool().unwrap_or(false),
        };
        let result = normalize_theme_with_options(&c["input"], &options);
        assert_eq!(
            serde_json::to_value(&result).unwrap(),
            c["output"],
            "{}",
            c["name"]
        );
        if let Some(t) = result.theme {
            assert_eq!(
                serde_json::to_value(theme_files(&t)).unwrap(),
                c["files"],
                "{}",
                c["name"]
            );
            assert_eq!(
                serde_json::to_value(readability_issues(&t)).unwrap(),
                c["readability"],
                "{}",
                c["name"]
            );
            let renamed = rename_theme_files(&t, |_| None);
            assert!(theme_files(&renamed).is_empty());
        }
        assert_eq!(
            contrast("#ffffff", "#000000"),
            c["contrast"].as_f64().unwrap()
        );
    }
}
