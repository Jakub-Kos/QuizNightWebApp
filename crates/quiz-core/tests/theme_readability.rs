use quiz_core::theme::{NormalizeOptions, contrast, normalize_theme_with_options};
fn readability_issues(v: &serde_json::Value) -> Vec<&'static str> {
    let mut v = v.clone();
    v["format"] = json!("quiznight-theme");
    quiz_core::theme::readability_issues(
        &normalize_theme_with_options(
            &v,
            &NormalizeOptions {
                stored: true,
                ..Default::default()
            },
        )
        .theme
        .unwrap(),
    )
}
use serde_json::json;

#[test]
fn editor_readability_cases() {
    assert_eq!(contrast("#ffffff", "#000000"), 21.0);
    assert!(readability_issues(&json!({})).is_empty());
    assert_eq!(
        readability_issues(
            &json!({"colors":{"ground":"#888888","panel":"#999999","accent":"#333333","primary":"#ffff66"}})
        ),
        vec!["text_ground", "text_panel", "accent_text", "primary_text"]
    );
    assert_eq!(
        readability_issues(
            &json!({"background":{"image":"x.jpg","imageOpacity":0.9,"imageBlur":0}})
        ),
        vec!["image"]
    );
    assert!(
        readability_issues(
            &json!({"background":{"image":"x.jpg","imageOpacity":0.9,"imageBlur":8}})
        )
        .is_empty()
    );
}
