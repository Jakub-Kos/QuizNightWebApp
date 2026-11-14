//! The workshop commands, compared with what scripts/workshop.mjs prints and writes, and the five
//! cases of src/quiz/workshopScript.test.js.
use quiz_cli::{find_pack_url, md, read_pack, report};
use std::io::{BufRead, BufReader, Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, thread};

fn committed() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../public/workshop")
}

fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let target = to.join(e.file_name());
        if e.path().is_dir() {
            copy(&e.path(), &target);
        } else {
            fs::copy(e.path(), target).unwrap();
        }
    }
}

/// A temporary copy of public/workshop.
fn workshop() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("workshop");
    copy(&committed(), &dir);
    (temp, dir)
}

/// Every file below a folder with its contents, for comparing trees.
fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, base, out);
            } else {
                let name = p.strip_prefix(base).unwrap().to_string_lossy().into_owned();
                out.push((name, fs::read(&p).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// A theme pack as the theme editor makes one (fflate's zipSync): theme.json and the given files.
fn pack(theme: &str, files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let theme = format!(r#"{{"format":"quiznight-theme","version":1,{theme}}}"#);
    zip.start_file("theme.json", options).unwrap();
    zip.write_all(theme.as_bytes()).unwrap();
    for (name, data) in files {
        zip.start_file(*name, options).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

struct Run {
    code: i32,
    out: String,
    err: String,
}

fn run(dir: &Path, args: &[&str], issue_body: Option<&str>, fetch: quiz_cli::Fetch<'_>) -> Run {
    let mut args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    args.extend(["--dir".into(), dir.to_string_lossy().into_owned()]);
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = quiz_cli::run(&args, issue_body, "2026-11-01", fetch, &mut out, &mut err);
    Run {
        code,
        out: String::from_utf8(out).unwrap(),
        err: String::from_utf8(err).unwrap(),
    }
}

fn no_fetch(url: &str) -> Result<Vec<u8>, String> {
    panic!("unexpected download of {url}")
}

// The five cases of src/quiz/workshopScript.test.js

#[test]
fn finds_the_attached_zip_in_an_issue_body_and_nothing_else() {
    let body = "### Theme file\n\n[theme-neon.zip](https://github.com/user-attachments/files/123456/theme-neon.zip)\n\nsee https://evil.example.com/x.zip";
    assert_eq!(
        find_pack_url(body).as_deref(),
        Some("https://github.com/user-attachments/files/123456/theme-neon.zip")
    );
    assert_eq!(find_pack_url("https://evil.example.com/theme.zip"), None);
    assert_eq!(find_pack_url(""), None);
}

#[test]
fn refuses_what_is_not_a_theme_pack_and_packs_that_unpack_to_too_much() {
    assert!(read_pack(&[1, 2, 3]).unwrap_err().0.contains("not a .zip"));
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("a.txt", zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"x").unwrap();
    let bytes = zip.finish().unwrap().into_inner();
    assert!(read_pack(&bytes).unwrap_err().0.contains("no theme.json"));
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("theme.json", options).unwrap();
    zip.write_all(b"{}").unwrap();
    zip.start_file("big.jpg", options).unwrap();
    zip.write_all(&vec![0; 31 * 1024 * 1024]).unwrap();
    let bytes = zip.finish().unwrap().into_inner();
    assert!(read_pack(&bytes).unwrap_err().0.contains("too much"));
}

#[test]
fn reports_a_passing_theme_with_the_command_to_add_it() {
    let pack = read_pack(&pack(r#""id":"calm","name":"Calm","author":"Ann""#, &[])).unwrap();
    let text = report(
        "https://github.com/user-attachments/files/1/t.zip",
        &pack,
        false,
    );
    assert!(text.contains("<!-- workshop-check -->"));
    assert!(text.contains('✅'));
    assert!(
        text.contains("cargo run -p quiz-cli -- add https://github.com/user-attachments/files/1/t.zip")
    );
}

#[test]
fn lists_what_would_be_dropped_and_gives_no_command() {
    let pack = read_pack(&pack(
        r#""id":"x","name":"X","colors":{"accent":"red"}"#,
        &[],
    ))
    .unwrap();
    let text = report("x.zip", &pack, false);
    assert!(text.contains('❌'));
    assert!(text.contains("`color`: accent"));
    assert!(!text.contains("workshop:add"));
}

#[test]
fn keeps_text_from_the_submission_from_formatting_the_comment_or_mentioning_people() {
    assert_eq!(
        md("@owner <img src=x> | **bold** [link](u)"),
        "@\u{200b}owner \\<img src=x\\> \\| \\*\\*bold\\*\\* \\[link\\](u)"
    );
    let pack = read_pack(&pack(
        r#""id":"x","name":"Hi @everyone","author":"<b>me</b>""#,
        &[],
    ))
    .unwrap();
    let text = report("x.zip", &pack, false);
    assert!(!text.contains("@everyone"));
    assert!(!text.contains("<b>"));
}

// index

/// The binary rewrites the committed index byte for byte, from the folders and the old dates.
#[test]
fn index_matches_the_committed_one() {
    let (_temp, dir) = workshop();
    let result = Command::new(env!("CARGO_BIN_EXE_quiz-cli"))
        .args(["index", "--dir"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&result.stdout),
        "✓ 3 workshop themes checked, index written\n"
    );
    assert!(result.stderr.is_empty());
    assert_eq!(
        fs::read(dir.join("index.json")).unwrap(),
        fs::read(committed().join("index.json")).unwrap()
    );
}

/// Without an index, every theme is dated today; the committed themes were all added on one day.
#[test]
fn index_without_a_previous_one_dates_themes_today() {
    let (_temp, dir) = workshop();
    fs::remove_file(dir.join("index.json")).unwrap();
    let mut out = Vec::new();
    quiz_cli::write_index(&dir, "2026-10-23", &mut out, &mut Vec::new()).unwrap();
    assert_eq!(
        fs::read(dir.join("index.json")).unwrap(),
        fs::read(committed().join("index.json")).unwrap()
    );
}

#[test]
fn index_refuses_a_theme_that_does_not_pass() {
    let (_temp, dir) = workshop();
    let before = fs::read(dir.join("index.json")).unwrap();
    let bad = dir.join("themes/bad");
    fs::create_dir(&bad).unwrap();
    fs::write(
        bad.join("theme.json"),
        r#"{"format":"quiznight-theme","version":1,"id":"bad","name":"Bad","colors":{"accent":"red"},"background":{"image":"x.png"}}"#,
    )
    .unwrap();
    fs::write(bad.join("zz.txt"), "x").unwrap();
    fs::write(bad.join("aa.txt"), "x").unwrap();
    let r = run(&dir, &["index"], None, &no_fetch);
    assert_eq!(r.code, 1);
    assert_eq!(r.out, "");
    assert_eq!(
        r.err,
        "✗ bad: the theme does not pass the check:\n  - color: accent\n  - missing_file: x.png\n  - unused_file: aa.txt\n  - unused_file: zz.txt\n"
    );
    assert_eq!(fs::read(dir.join("index.json")).unwrap(), before);

    fs::write(
        bad.join("theme.json"),
        r#"{"format":"quiznight-theme","version":1,"id":"calm","name":"Calm"}"#,
    )
    .unwrap();
    fs::remove_file(bad.join("zz.txt")).unwrap();
    fs::remove_file(bad.join("aa.txt")).unwrap();
    let r = run(&dir, &["index"], None, &no_fetch);
    assert_eq!(
        (r.code, r.err.as_str()),
        (
            1,
            "✗ bad: the folder must be named after the theme's id (calm)\n"
        )
    );
    fs::remove_file(bad.join("theme.json")).unwrap();
    let r = run(&dir, &["index"], None, &no_fetch);
    assert_eq!(r.code, 1);
    let path = std::path::absolute(bad.join("theme.json")).unwrap();
    assert_eq!(
        r.err,
        format!(
            "✗ bad: theme.json cannot be read (ENOENT: no such file or directory, open '{}')\n",
            path.display()
        )
    );
}

#[test]
fn index_warns_about_hard_to_read_themes() {
    let (_temp, dir) = workshop();
    let white = dir.join("themes/white");
    fs::create_dir(&white).unwrap();
    fs::write(
        white.join("theme.json"),
        r##"{"format":"quiznight-theme","version":1,"id":"white","name":"White","colors":{"ground":"#ffffff","groundTop":"#ffffff","groundBottom":"#ffffff","panel":"#ffffff"}}"##,
    )
    .unwrap();
    let r = run(&dir, &["index"], None, &no_fetch);
    assert_eq!(r.code, 0);
    assert_eq!(r.out, "✓ 4 workshop themes checked, index written\n");
    assert_eq!(r.err, "! white: hard to read: text_ground, text_panel\n");
    let index = fs::read_to_string(dir.join("index.json")).unwrap();
    // the new theme comes first, dated today
    assert!(index.contains(
        r#""path": "themes/white/",
      "addedAt": "2026-11-01","#
    ));
}

// add

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n not a real picture";

fn sunset() -> Vec<u8> {
    pack(
        r##""id":"sunset-bar","name":{"en":"Sunset bar","cs":"Západ"},"author":"Ann","colors":{"panel":"#202020","primary":"#c2410c"},"fonts":{"body":{"family":"Nunito"},"title":{"family":"Montserrat"}},"background":{"image":"assets/bg.png","imageOpacity":0.35}"##,
        &[("assets/bg.png", PNG), ("notes.txt", b"hello")],
    )
}

#[test]
fn add_writes_the_theme_its_files_and_the_index() {
    let (temp, dir) = workshop();
    let file = temp.path().join("sunset.zip");
    fs::write(&file, sunset()).unwrap();
    let r = run(&dir, &["add", file.to_str().unwrap()], None, &no_fetch);
    assert_eq!(r.err, "");
    assert_eq!(r.code, 0);
    assert_eq!(
        r.out,
        "✓ added \"Sunset bar\" by Ann as public/workshop/themes/sunset-bar/\n✓ 4 workshop themes checked, index written\n"
    );
    let theme = dir.join("themes/sunset-bar");
    let files: Vec<_> = tree(&theme).into_iter().map(|(n, _)| n).collect();
    assert_eq!(files, ["assets/bg.png", "theme.json"]);
    assert_eq!(fs::read(theme.join("assets/bg.png")).unwrap(), PNG);
    // keys in the order the Node script writes them
    let json = fs::read_to_string(theme.join("theme.json")).unwrap();
    assert!(json.starts_with(
        "{\n  \"format\": \"quiznight-theme\",\n  \"version\": 1,\n  \"id\": \"sunset-bar\",\n  \"name\": {\n    \"en\": \"Sunset bar\",\n    \"cs\": \"Západ\"\n  },"
    ));
    assert!(json.contains("\"colors\": {\n    \"primary\": \"#c2410c\",\n    \"panel\": \"#202020\"\n  },\n  \"fonts\": {\n    \"title\": {"));
    assert!(json.ends_with("}\n"));
    // index again changes nothing
    let index = fs::read(dir.join("index.json")).unwrap();
    assert_eq!(run(&dir, &["index"], None, &no_fetch).code, 0);
    assert_eq!(fs::read(dir.join("index.json")).unwrap(), index);
}

#[test]
fn add_refuses_a_theme_that_would_drop_something() {
    let (temp, dir) = workshop();
    let before = tree(&dir);
    let file = temp.path().join("drop.zip");
    fs::write(
        &file,
        pack(
            r#""id":"drop","name":"Drop","colors":{"accent":"red"}"#,
            &[],
        ),
    )
    .unwrap();
    let r = run(&dir, &["add", file.to_str().unwrap()], None, &no_fetch);
    assert_eq!(r.code, 1);
    assert_eq!(r.out, "");
    assert_eq!(
        r.err,
        "✗ the theme does not pass the check (ask the author to fix it, or fix theme.json yourself):\n  - color: accent\n"
    );
    assert_eq!(tree(&dir), before);
}

#[test]
fn add_replaces_only_when_asked() {
    let (temp, dir) = workshop();
    let before = tree(&dir);
    let file = temp.path().join("ocean.zip");
    fs::write(
        &file,
        pack(
            r#""id":"deep-ocean","name":"Deep ocean v2","author":"@someone""#,
            &[],
        ),
    )
    .unwrap();
    let file = file.to_str().unwrap();
    let r = run(&dir, &["add", file], None, &no_fetch);
    assert_eq!(r.code, 1);
    assert_eq!(
        r.err,
        "✗ a theme with the id \"deep-ocean\" is already in the workshop (add --replace to overwrite it)\n"
    );
    assert_eq!(tree(&dir), before);
    for args in [["add", file, "--replace"], ["add", "--replace", file]] {
        let r = run(&dir, &args, None, &no_fetch);
        assert_eq!(r.code, 0, "{}", r.err);
        assert_eq!(
            r.out,
            "✓ added \"Deep ocean v2\" by @someone as public/workshop/themes/deep-ocean/\n✓ 3 workshop themes checked, index written\n"
        );
    }
    let index = fs::read_to_string(dir.join("index.json")).unwrap();
    assert!(index.contains("Deep ocean v2"));
    // the replaced theme keeps the date it was first added
    assert!(!index.contains("2026-11-01"));
}

#[test]
fn add_needs_a_pack() {
    let (temp, dir) = workshop();
    let r = run(&dir, &["add"], None, &no_fetch);
    assert_eq!(
        (r.code, r.err.as_str()),
        (1, "✗ give the theme pack: a .zip file or its URL\n")
    );
    let missing = temp.path().join("missing.zip");
    let r = run(&dir, &["add", missing.to_str().unwrap()], None, &no_fetch);
    assert_eq!(r.code, 1);
    assert_eq!(
        r.err,
        format!(
            "✗ Error: ENOENT: no such file or directory, open '{}'\n",
            missing.display()
        )
    );
    let junk = temp.path().join("junk.zip");
    fs::write(&junk, "junk").unwrap();
    let r = run(&dir, &["add", junk.to_str().unwrap()], None, &no_fetch);
    assert_eq!((r.code, r.err.as_str()), (1, "✗ this is not a .zip file\n"));
}

#[test]
fn usage() {
    let (_temp, dir) = workshop();
    for args in [&[][..], &["frobnicate"][..]] {
        let r = run(&dir, args, None, &no_fetch);
        assert_eq!(r.code, 1);
        assert_eq!(
            r.err,
            "✗ usage: quiz-cli index | add <zip or URL> [--replace] | check <zip or URL> | check --from-issue\n"
        );
    }
}

// check

#[test]
fn check_reports_and_always_succeeds() {
    let (temp, dir) = workshop();
    let before = tree(&dir);
    let file = temp.path().join("sunset.zip");
    fs::write(&file, sunset()).unwrap();
    let source = file.to_str().unwrap();
    let r = run(&dir, &["check", source], None, &no_fetch);
    assert_eq!((r.code, r.err.as_str()), (0, ""));
    assert_eq!(
        r.out,
        format!(
            "<!-- workshop-check -->
### Theme check

✅ **Passes the check.**

| | |
| --- | --- |
| Name | Sunset bar (en), Západ (cs) |
| Author | Ann |
| Id | `sunset-bar` |
| Tags | — |
| Fonts | title: Montserrat, body: Nunito |
| Scenery, decoration | —, — |
| Files | assets/bg.png (1 KB) |

Not used by the theme (left out when added): notes.txt.

To review it, download the file and import it in a quiz (Look → Import theme). To add it:

```
cargo run -p quiz-cli -- add {source}
```
"
        )
    );
    // a theme already in the workshop
    fs::write(&file, pack(r#""id":"deep-ocean","name":"Deep""#, &[])).unwrap();
    let r = run(&dir, &["check", source], None, &no_fetch);
    assert!(r.out.contains(
        "| Id | `deep-ocean` ⚠️ already in the workshop: adding it replaces that theme |"
    ));
    assert!(
        r.out
            .contains(&format!("cargo run -p quiz-cli -- add {source} --replace\n```"))
    );
    // refused
    fs::write(
        &file,
        pack(r#""id":"x","name":"X","colors":{"accent":"red"}"#, &[]),
    )
    .unwrap();
    let r = run(&dir, &["check", source], None, &no_fetch);
    assert_eq!(r.code, 0);
    assert!(r.out.contains(
        "❌ **Does not pass.** These parts would be left out:\n\n- `color`: accent\n\n| | |"
    ));
    assert!(!r.out.contains("workshop:add"));
    // not a pack
    fs::write(&file, "junk").unwrap();
    let r = run(&dir, &["check", source], None, &no_fetch);
    assert_eq!(
        r.out,
        "<!-- workshop-check -->\n### Theme check\n\n❌ The attached file could not be checked: this is not a .zip file.\n\nDownload the theme again from the theme editor (Share to the workshop) and attach the new file.\n"
    );
    assert_eq!(r.code, 0);
    assert_eq!(tree(&dir), before);
}

const NO_FILE: &str = "<!-- workshop-check -->\n### Theme check\n\n❌ No theme file found. Drag the downloaded `theme-….zip` into the issue (edit it), and the check runs again.\n";

#[test]
fn check_from_issue() {
    let (_temp, dir) = workshop();
    // no attachment, or a link that is not one: nothing is downloaded
    for body in [
        None,
        Some(""),
        Some("no file, see https://evil.example.com/x.zip and http://127.0.0.1/x.zip"),
    ] {
        let r = run(&dir, &["check", "--from-issue"], body, &no_fetch);
        assert_eq!((r.code, r.out.as_str(), r.err.as_str()), (0, NO_FILE, ""));
    }
    let r = run(&dir, &["check"], None, &no_fetch);
    assert_eq!((r.code, r.out.as_str()), (0, NO_FILE));
    // the attachment is downloaded and reported with the command to add it
    let url = "https://github.com/user-attachments/files/42/theme-sunset-bar.zip";
    let body = format!("### Theme file\n\n[theme-sunset-bar.zip]({url})\n");
    let fetch = |u: &str| {
        assert_eq!(u, url);
        Ok(sunset())
    };
    let r = run(
        &dir,
        &["check", "--from-issue", "ignored.zip"],
        Some(&body),
        &fetch,
    );
    assert_eq!(r.code, 0);
    assert!(r.out.contains("✅ **Passes the check.**"));
    assert!(
        r.out
            .ends_with(&format!("cargo run -p quiz-cli -- add {url}\n```\n"))
    );
    // a failed download
    let fetch = |_: &str| Err("the download failed (HTTP 404)".to_owned());
    let r = run(&dir, &["check", "--from-issue"], Some(&body), &fetch);
    assert!(
        r.out
            .contains("❌ The attached file could not be checked: the download failed (HTTP 404).")
    );
}

// download, against a server on localhost

/// Serves a few responses on 127.0.0.1 for as long as the test runs.
fn serve() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            thread::spawn(move || {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let path = line.split(' ').nth(1).unwrap_or("").to_owned();
                loop {
                    let mut header = String::new();
                    if reader.read_line(&mut header).unwrap_or(0) == 0 || header == "\r\n" {
                        break;
                    }
                }
                let head = |status: &str, headers: &str| {
                    format!("HTTP/1.1 {status}\r\nConnection: close\r\n{headers}\r\n")
                };
                let big = 31 * 1024 * 1024;
                let _ = match path.as_str() {
                    "/good.zip" => {
                        let body = sunset();
                        stream
                            .write_all(
                                head("200 OK", &format!("Content-Length: {}\r\n", body.len()))
                                    .as_bytes(),
                            )
                            .and_then(|_| stream.write_all(&body))
                    }
                    "/redirect" => stream.write_all(
                        head("302 Found", "Location: /redirect2\r\nContent-Length: 0\r\n")
                            .as_bytes(),
                    ),
                    "/redirect2" => stream.write_all(
                        head("301 Moved", "Location: /good.zip\r\nContent-Length: 0\r\n")
                            .as_bytes(),
                    ),
                    "/loop" => stream.write_all(
                        head("302 Found", "Location: /loop\r\nContent-Length: 0\r\n").as_bytes(),
                    ),
                    "/declared-big" => stream
                        .write_all(head("200 OK", &format!("Content-Length: {big}\r\n")).as_bytes())
                        .and_then(|_| stream.write_all(b"PK")),
                    "/chunked-big" => {
                        let chunk = vec![0u8; 1024 * 1024];
                        let mut sent = stream
                            .write_all(head("200 OK", "Transfer-Encoding: chunked\r\n").as_bytes());
                        for _ in 0..31 {
                            sent = sent
                                .and_then(|_| {
                                    stream.write_all(format!("{:x}\r\n", chunk.len()).as_bytes())
                                })
                                .and_then(|_| stream.write_all(&chunk))
                                .and_then(|_| stream.write_all(b"\r\n"));
                        }
                        sent.and_then(|_| stream.write_all(b"0\r\n\r\n"))
                    }
                    _ => {
                        stream.write_all(head("404 Not Found", "Content-Length: 0\r\n").as_bytes())
                    }
                };
            });
        }
    });
    base
}

#[test]
fn download_follows_redirects_and_stops_at_the_limit() {
    let base = serve();
    assert_eq!(
        quiz_cli::download(&format!("{base}/good.zip")).unwrap(),
        sunset()
    );
    assert_eq!(
        quiz_cli::download(&format!("{base}/redirect")).unwrap(),
        sunset()
    );
    assert_eq!(
        quiz_cli::download(&format!("{base}/missing")).unwrap_err(),
        "the download failed (HTTP 404)"
    );
    assert_eq!(
        quiz_cli::download(&format!("{base}/loop")).unwrap_err(),
        "TypeError: fetch failed"
    );
    assert_eq!(
        quiz_cli::download(&format!("{base}/declared-big")).unwrap_err(),
        "the file is too big"
    );
    assert_eq!(
        quiz_cli::download(&format!("{base}/chunked-big")).unwrap_err(),
        "the file is too big"
    );
    assert_eq!(
        quiz_cli::download("http://127.0.0.1:9/x.zip").unwrap_err(),
        "TypeError: fetch failed"
    );
    assert_eq!(
        quiz_cli::download("http://").unwrap_err(),
        "TypeError: Failed to parse URL from http://"
    );
}

/// The binary downloads what `check` and `add` are given as a URL.
#[test]
fn binary_checks_and_adds_from_a_url() {
    let base = serve();
    let (_temp, dir) = workshop();
    let url = format!("{base}/redirect");
    let check = Command::new(env!("CARGO_BIN_EXE_quiz-cli"))
        .args(["check", &url, "--dir"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(check.status.code(), Some(0));
    let out = String::from_utf8(check.stdout).unwrap();
    assert!(out.contains("✅ **Passes the check.**"), "{out}");
    assert!(out.ends_with(&format!("cargo run -p quiz-cli -- add {url}\n```\n")));
    let big = Command::new(env!("CARGO_BIN_EXE_quiz-cli"))
        .args(["check", &format!("{base}/chunked-big"), "--dir"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        String::from_utf8(big.stdout)
            .unwrap()
            .contains("could not be checked: the file is too big.")
    );
    let add = Command::new(env!("CARGO_BIN_EXE_quiz-cli"))
        .args(["add", &url, "--dir"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(
        add.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    assert_eq!(
        fs::read(dir.join("themes/sunset-bar/assets/bg.png")).unwrap(),
        PNG
    );
    // from an issue, a URL that is not a GitHub attachment is never fetched
    let issue = Command::new(env!("CARGO_BIN_EXE_quiz-cli"))
        .args(["check", "--from-issue", "--dir"])
        .arg(&dir)
        .env("ISSUE_BODY", format!("[x]({base}/good.zip)"))
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(issue.stdout).unwrap(), NO_FILE);
}
