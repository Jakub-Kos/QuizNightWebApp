use quiz_cli::{find_pack_url, md, read_pack, report};
use serde_json::Value;
fn decode(s: &str) -> Vec<u8> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bits = 0u32;
    let mut count = 0;
    let mut out = Vec::new();
    for c in s.bytes().take_while(|c| *c != b'=') {
        bits = (bits << 6) | alphabet.iter().position(|b| *b == c).unwrap() as u32;
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8);
        }
    }
    out
}
#[test]
fn javascript_parity() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/report.json")).unwrap();
    for case in fixture["packs"].as_array().unwrap() {
        let pack = read_pack(&decode(case["zipBase64"].as_str().unwrap())).unwrap();
        assert_eq!(
            report(case["source"].as_str().unwrap(), &pack, false),
            case["report"],
            "{}",
            case["name"]
        );
    }
    for case in fixture["errors"].as_array().unwrap() {
        assert_eq!(
            read_pack(&decode(case["zipBase64"].as_str().unwrap()))
                .unwrap_err()
                .to_string(),
            case["error"],
            "{}",
            case["name"]
        );
    }
    for case in fixture["md"].as_array().unwrap() {
        assert_eq!(
            md(case["input"].as_str().unwrap()),
            case["output"],
            "{case}"
        );
    }
    for case in fixture["urls"].as_array().unwrap() {
        assert_eq!(
            find_pack_url(case["input"].as_str().unwrap()),
            case["output"].as_str().map(str::to_owned),
            "{case}"
        );
    }
}
#[test]
fn rejects_large_deflated_entry() {
    use std::io::{Cursor, Write};
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file(
        "large",
        zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();
    zip.write_all(&vec![0; 31 * 1024 * 1024]).unwrap();
    assert_eq!(
        read_pack(&zip.finish().unwrap().into_inner())
            .unwrap_err()
            .to_string(),
        "the pack unpacks to too much data"
    );
}
