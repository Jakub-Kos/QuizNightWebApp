//! Zip packs from strangers: archives that lie about their sizes, entry names that leave the theme
//! folder, duplicate names, many entries, symlinks. The archives are built by hand where the zip
//! crate would not write them.
use quiz_cli::{MAX_PACK, read_pack};
use std::io::{Cursor, Write};
use std::path::Path;

const THEME: &str = r#"{"format":"quiznight-theme","version":1,"id":"calm","name":"Calm"}"#;

fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// One entry of a hand-built archive. `data` is what is stored in the file; the sizes and CRC are
/// what the headers declare, which need not be true.
#[derive(Clone)]
struct Entry {
    name: Vec<u8>,
    data: Vec<u8>,
    method: u16,
    crc: u32,
    compressed: u32,
    uncompressed: u32,
    flags: u16,
    made_by: u16,
    external: u32,
}

fn stored(name: &str, data: &[u8]) -> Entry {
    Entry {
        name: name.as_bytes().to_vec(),
        data: data.to_vec(),
        method: 0,
        crc: crc32(data),
        compressed: data.len() as u32,
        uncompressed: data.len() as u32,
        flags: 0x800,
        made_by: 20,
        external: 0,
    }
}

/// A deflated entry, compressed by the zip crate.
fn deflated(name: &str, data: &[u8]) -> Entry {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file(
        "x",
        zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();
    zip.write_all(data).unwrap();
    let bytes = zip.finish().unwrap().into_inner();
    let archive = zip::ZipArchive::new(Cursor::new(&bytes[..])).unwrap();
    let file = archive.by_index_data(0).unwrap();
    let start = 30 + 1 + u16::from_le_bytes([bytes[28], bytes[29]]) as usize;
    let compressed = file.compressed_size() as usize;
    Entry {
        data: bytes[start..start + compressed].to_vec(),
        method: 8,
        compressed: compressed as u32,
        ..stored(name, data)
    }
}

/// Local headers, data, central directory and its end, as a zip writer lays them out. `count`
/// overrides the number of entries the end record claims.
fn build(entries: &[Entry], count: Option<u16>) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for e in entries {
        let offset = out.len() as u32;
        out.extend(0x0403_4b50u32.to_le_bytes());
        out.extend(20u16.to_le_bytes());
        out.extend(e.flags.to_le_bytes());
        out.extend(e.method.to_le_bytes());
        out.extend([0, 0, 0x21, 0]); // time, date
        out.extend(e.crc.to_le_bytes());
        out.extend(e.compressed.to_le_bytes());
        out.extend(e.uncompressed.to_le_bytes());
        out.extend((e.name.len() as u16).to_le_bytes());
        out.extend(0u16.to_le_bytes());
        out.extend(&e.name);
        out.extend(&e.data);
        central.extend(0x0201_4b50u32.to_le_bytes());
        central.extend(e.made_by.to_le_bytes());
        central.extend(20u16.to_le_bytes());
        central.extend(e.flags.to_le_bytes());
        central.extend(e.method.to_le_bytes());
        central.extend([0, 0, 0x21, 0]);
        central.extend(e.crc.to_le_bytes());
        central.extend(e.compressed.to_le_bytes());
        central.extend(e.uncompressed.to_le_bytes());
        central.extend((e.name.len() as u16).to_le_bytes());
        central.extend([0; 8]); // extra, comment, disk, internal attributes
        central.extend(e.external.to_le_bytes());
        central.extend(offset.to_le_bytes());
        central.extend(&e.name);
    }
    let start = out.len() as u32;
    let n = count.unwrap_or(entries.len() as u16);
    out.extend(&central);
    out.extend(0x0605_4b50u32.to_le_bytes());
    out.extend([0; 4]);
    out.extend(n.to_le_bytes());
    out.extend(n.to_le_bytes());
    out.extend((central.len() as u32).to_le_bytes());
    out.extend(start.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out
}

fn error(bytes: &[u8]) -> String {
    read_pack(bytes).unwrap_err().0
}

#[test]
fn hand_built_archives_are_read() {
    let png = b"\x89PNG not really".to_vec();
    let theme = r#"{"format":"quiznight-theme","version":1,"id":"calm","name":"Calm","background":{"image":"bg.png"}}"#;
    for bg in [stored("bg.png", &png), deflated("bg.png", &png)] {
        let pack = read_pack(&build(&[stored("theme.json", theme.as_bytes()), bg], None)).unwrap();
        assert!(pack.problems.is_empty(), "{:?}", pack.problems);
        assert_eq!(pack.assets[0].bytes, png);
    }
}

/// A deflated entry that declares a few bytes and inflates to more than the limit. The zip reader
/// stops at the declared size and fails (fflate instead truncates the data to the declared size,
/// after inflating all of it); `read_pack` also caps what it reads at the limit.
#[test]
fn central_directory_understating_sizes() {
    let mut bomb = deflated("big.jpg", &vec![0; MAX_PACK as usize + 1024]);
    bomb.uncompressed = 10;
    let bytes = build(&[stored("theme.json", THEME.as_bytes()), bomb], None);
    assert!(bytes.len() < 100 * 1024);
    assert_eq!(error(&bytes), "this is not a .zip file");
    // Spread over several entries, each one under the limit
    let mut part = deflated("part", &vec![0; 8 * 1024 * 1024]);
    part.uncompressed = 1;
    let parts: Vec<_> = (0..5)
        .map(|i| Entry {
            name: format!("part{i}.jpg").into_bytes(),
            ..part.clone()
        })
        .collect();
    let mut entries = vec![stored("theme.json", THEME.as_bytes())];
    entries.extend(parts);
    assert_eq!(error(&build(&entries, None)), "this is not a .zip file");
}

/// A stored entry whose declared uncompressed size is 0 while its data is larger than the limit
/// (fflate takes the stored data whole and counts 0 bytes of it towards the limit).
#[test]
fn stored_entry_understating_size() {
    let mut big = stored("big.jpg", &vec![0; MAX_PACK as usize + 1]);
    big.uncompressed = 0;
    let bytes = build(&[stored("theme.json", THEME.as_bytes()), big], None);
    assert_eq!(error(&bytes), "this is not a .zip file");
}

/// A small lie (an entry that inflates to more than it declares, under the limit) is refused too.
#[test]
fn central_directory_understating_a_small_entry() {
    let mut lie = deflated("bg.png", &[7; 5000]);
    lie.uncompressed = 10;
    let theme = r#"{"format":"quiznight-theme","version":1,"id":"calm","name":"Calm","background":{"image":"bg.png"}}"#;
    assert_eq!(
        error(&build(&[stored("theme.json", theme.as_bytes()), lie], None)),
        "this is not a .zip file"
    );
}

#[test]
fn overstated_sizes_are_refused_before_unpacking() {
    let mut claim = stored("a.jpg", b"tiny");
    claim.uncompressed = MAX_PACK as u32 + 1;
    let bytes = build(&[stored("theme.json", THEME.as_bytes()), claim], None);
    assert_eq!(error(&bytes), "the pack unpacks to too much data");
}

#[test]
fn traversal_and_absolute_names_are_read_but_flagged() {
    for name in ["../../escape.png", "/etc/escape.png", "a/../../escape.png"] {
        let theme = format!(
            r#"{{"format":"quiznight-theme","version":1,"id":"calm","name":"Calm","background":{{"image":"{name}"}}}}"#
        );
        let pack = read_pack(&build(
            &[stored("theme.json", theme.as_bytes()), stored(name, b"png")],
            None,
        ))
        .unwrap();
        // the theme library accepts the name, as the JavaScript one does
        assert_eq!(pack.theme.background.image.as_deref(), Some(name));
        assert!(!quiz_cli::stays_inside(name));
    }
}

/// fflate returns entries as a JavaScript object: a repeated name keeps its first place and its
/// last contents.
#[test]
fn duplicate_names_behave_as_in_javascript() {
    let first = r#"{"format":"quiznight-theme","version":1,"id":"first","name":"First"}"#;
    let second = r#"{"format":"quiznight-theme","version":1,"id":"second","name":"Second"}"#;
    let pack = read_pack(&build(
        &[
            stored("theme.json", first.as_bytes()),
            stored("x.txt", b"x"),
            stored("theme.json", second.as_bytes()),
        ],
        None,
    ))
    .unwrap();
    assert_eq!(pack.theme.id, "second");
    let theme = r#"{"format":"quiznight-theme","version":1,"id":"calm","name":"Calm","background":{"image":"bg.png"}}"#;
    let pack = read_pack(&build(
        &[
            stored("bg.png", b"one"),
            stored("theme.json", theme.as_bytes()),
            stored("bg.png", b"two"),
        ],
        None,
    ))
    .unwrap();
    assert_eq!(pack.assets.len(), 1);
    assert_eq!(pack.assets[0].bytes, b"two");
}

#[test]
fn many_entries() {
    let start = std::time::Instant::now();
    let mut entries = vec![stored("theme.json", THEME.as_bytes())];
    entries.extend((0..20_000).map(|i| stored(&format!("f{i}.txt"), b"")));
    let pack = read_pack(&build(&entries, None)).unwrap();
    assert_eq!(pack.problems.len(), 20_000);
    assert!(start.elapsed().as_secs() < 30, "{:?}", start.elapsed());
    // An end record that claims more entries than there are
    let bytes = build(&[stored("theme.json", THEME.as_bytes())], Some(u16::MAX));
    assert_eq!(error(&bytes), "this is not a .zip file");
}

/// A symlink entry is read as a file holding the link's target, and `add` writes it as a plain file.
#[test]
fn symlink_entries_are_plain_files() {
    let theme = r#"{"format":"quiznight-theme","version":1,"id":"linked","name":"Linked","background":{"image":"bg.png"}}"#;
    let link = Entry {
        made_by: 3 << 8 | 20,
        external: 0o120777 << 16,
        ..stored("bg.png", b"/etc/passwd")
    };
    let bytes = build(&[stored("theme.json", theme.as_bytes()), link], None);
    let pack = read_pack(&bytes).unwrap();
    assert_eq!(pack.assets[0].bytes, b"/etc/passwd");

    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("linked.zip");
    std::fs::write(&file, &bytes).unwrap();
    let workshop = temp.path().join("workshop");
    let (code, _, err) = run(&workshop, &["add", file.to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    let written = workshop.join("themes/linked/bg.png");
    let meta = std::fs::symlink_metadata(&written).unwrap();
    assert!(meta.file_type().is_file());
    assert_eq!(std::fs::read(written).unwrap(), b"/etc/passwd");
}

/// `add` refuses a theme whose files would land outside its folder, and writes nothing.
#[test]
fn add_never_writes_outside_the_theme_folder() {
    for name in [
        "../../../escape.png",
        "../escape.png",
        "/tmp/escape.png",
        "assets/../../escape.png",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let theme = format!(
            r#"{{"format":"quiznight-theme","version":1,"id":"evil","name":"Evil","background":{{"image":"{name}"}}}}"#
        );
        let bytes = build(
            &[stored("theme.json", theme.as_bytes()), stored(name, b"png")],
            None,
        );
        let file = temp.path().join("evil.zip");
        std::fs::write(&file, &bytes).unwrap();
        let workshop = temp.path().join("a/b/workshop");
        let (code, out, err) = run(&workshop, &["add", file.to_str().unwrap()]);
        assert_eq!(code, 1, "{name}");
        assert_eq!(out, "");
        assert_eq!(
            err,
            format!(
                "✗ the theme refers to a file outside its folder ({name}); fix the pack before adding it\n"
            )
        );
        let mut written = Vec::new();
        list(temp.path(), &mut written);
        assert_eq!(written, vec![file.clone()], "{name}");
    }
}

fn list(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            list(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn run(workshop: &Path, args: &[&str]) -> (i32, String, String) {
    let mut args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    args.extend(["--dir".into(), workshop.to_string_lossy().into_owned()]);
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = quiz_cli::run(
        &args,
        None,
        "2026-01-01",
        &|_| panic!("no download"),
        &mut out,
        &mut err,
    );
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

/// A wrong CRC is refused (fflate does not check CRCs).
#[test]
fn wrong_crc() {
    let mut bad = stored("theme.json", THEME.as_bytes());
    bad.crc ^= 1;
    assert_eq!(error(&build(&[bad], None)), "this is not a .zip file");
}
