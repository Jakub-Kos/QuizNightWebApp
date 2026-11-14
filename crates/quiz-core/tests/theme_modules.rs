use quiz_core::theme::*;
use quiz_core::theme::{builtin::*, pack::*, workshop::*};
use serde_json::{Value, json};
fn decode(s: &str) -> Vec<u8> {
    let mut bits = 0u32;
    let mut count = 0;
    let mut out = Vec::new();
    for c in s.bytes().filter(|c| *c != b'=') {
        let n = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
            .iter()
            .position(|x| *x == c)
            .unwrap();
        bits = (bits << 6) | n as u32;
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8);
        }
    }
    out
}
#[test]
fn module_parity() {
    let f: Value = serde_json::from_str(include_str!("fixtures/theme/modules.json")).unwrap();
    for (i, c) in f["builtin"].as_array().unwrap().iter().enumerate() {
        let t: Theme = serde_json::from_value(c["theme"].clone()).unwrap();
        if i < themes().len() {
            assert_eq!(
                serde_json::to_value(&themes()[i]).unwrap(),
                c["theme"],
                "builtin {i}"
            );
        }
        assert_eq!(
            serde_json::to_value(theme_style(&t)).unwrap(),
            c["style"],
            "style {i}"
        );
        assert_eq!(json!(style_classes(&t)), c["classes"], "classes {i}");
        for l in ["en", "cs", "sk"] {
            assert_eq!(json!(theme_name(&t, l)), c["names"][l], "name {i} {l}");
        }
    }
    for c in f["packs"].as_array().unwrap() {
        let p = read_theme_pack(&decode(c["base64"].as_str().unwrap())).unwrap();
        assert_eq!(
            serde_json::to_value(&p.theme).unwrap(),
            c["theme"],
            "pack {}",
            c["name"]
        );
        assert_eq!(
            serde_json::to_value(&p.problems).unwrap(),
            c["problems"],
            "problems {}",
            c["name"]
        );
        assert_eq!(
            json!(
                p.assets
                    .iter()
                    .map(|a| json!({"name":a.name,"bytes":a.bytes}))
                    .collect::<Vec<_>>()
            ),
            c["assets"],
            "assets {}",
            c["name"]
        );
        let again = read_theme_pack(&export_theme_pack(&p.theme, &p.assets).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(again.theme).unwrap(),
            c["theme"],
            "roundtrip {}",
            c["name"]
        );
    }
    for c in f["errors"].as_array().unwrap() {
        assert_eq!(
            read_theme_pack(&decode(c["base64"].as_str().unwrap()))
                .unwrap_err()
                .to_string(),
            c["error"].as_str().unwrap(),
            "error {}",
            c["name"]
        );
    }
    for c in f["packName"].as_array().unwrap() {
        let t = serde_json::from_value(c["theme"].clone()).unwrap();
        assert_eq!(json!(theme_pack_name(&t)), c["name"]);
    }
    let entries = parse_index(&f["workshop"]);
    assert_eq!(
        serde_json::to_value(&entries).unwrap(),
        f["workshop"]["themes"]
    );
    for e in entries {
        assert_eq!(
            serde_json::to_value(index_entry(&e.theme, &e.added_at)).unwrap(),
            serde_json::to_value(&e).unwrap()
        );
    }
}
