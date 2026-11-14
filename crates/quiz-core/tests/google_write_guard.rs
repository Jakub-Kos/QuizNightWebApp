use quiz_core::quiz::write_guard::*;
use serde_json::json;
use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};
const D: &str = "https://www.googleapis.com/drive/v3/files";
const U: &str = "https://www.googleapis.com/upload/drive/v3/files";
const S: &str = "https://sheets.googleapis.com/v4/spreadsheets";
fn run<F: Future>(f: F) -> F::Output {
    match pin!(f)
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(v) => v,
        _ => panic!("pure mock must be ready"),
    }
}
fn target(m: &str, u: &str, b: &str, k: Kind, ids: &[&str]) {
    assert_eq!(
        write_targets(m, u, b),
        Ok(Some(Target {
            kind: k,
            ids: ids.iter().map(|s| s.to_string()).collect()
        }))
    );
}
#[test]
fn reads() {
    assert_eq!(
        write_targets("GET", &format!("{D}/abc?fields=id"), ""),
        Ok(None)
    );
    assert_eq!(write_targets("HEAD", S, ""), Ok(None));
}
#[test]
fn sheet_edits() {
    for suffix in [":batchUpdate", "/values:batchUpdate"] {
        target(
            "POST",
            &format!("{S}/s1{suffix}"),
            "{}",
            Kind::Edit,
            &["s1"],
        );
    }
}
#[test]
fn metadata_content_sharing() {
    for (m, url) in [
        ("PATCH", format!("{D}/s1?fields=id&supportsAllDrives=true")),
        ("PATCH", format!("{U}/s1?uploadType=media")),
        (
            "POST",
            format!("{D}/s1/permissions?sendNotificationEmail=true"),
        ),
        (
            "DELETE",
            format!("{D}/s1/permissions/p9?supportsAllDrives=true"),
        ),
    ] {
        target(m, &url, "{}", Kind::Edit, &["s1"]);
    }
}
#[test]
fn new_files_multipart_and_copy() {
    for(url,b)in [(format!("{U}?uploadType=resumable"),json!({"name":"a.jpg","parents":["m1"]}).to_string()),(format!("{U}?uploadType=multipart"),"--b\r\nContent-Type: application/json\r\n\r\n{\"name\":\"quiznight.json\",\"parents\":[\"m1\"]}\r\n--b\r\n\r\n{\"x\":1}\r\n--b--".into()),(format!("{D}/template/copy?fields=id"),json!({"parents":["m1"]}).to_string())]{target("POST",&url,&b,Kind::Create,&["m1"]);}
}
#[test]
fn empty_folder_anywhere() {
    target(
        "POST",
        &format!("{D}?fields=id,name"),
        &json!({"name":"Quiz","mimeType":"application/vnd.google-apps.folder","parents":["root"]})
            .to_string(),
        Kind::Folder,
        &["root"],
    );
}
#[test]
fn refused_requests() {
    for (m, u, b, reason) in [
        ("DELETE", format!("{D}/s1"), "{}", "delete"),
        (
            "POST",
            format!("{U}?uploadType=resumable"),
            "{}",
            "no_parent",
        ),
        (
            "POST",
            "https://www.googleapis.com/drive/v3/changes/watch".into(),
            "{}",
            "unknown_url",
        ),
        ("DELETE", format!("{S}/s1"), "{}", "sheet_method"),
        ("PUT", format!("{D}/s1"), "{}", "unknown_method"),
    ] {
        assert_eq!(write_targets(m, &u, b), Err(reason.into()));
    }
}
#[test]
fn trash_classification() {
    target(
        "PATCH",
        &format!("{D}/f1"),
        "{\"trashed\": true}",
        Kind::Trash,
        &["f1"],
    );
}
fn parents(id: String) -> impl Future<Output = Vec<String>> {
    async move {
        match id.as_str() {
            "other" | "q" | "loose" => vec!["root".into()],
            "private" => vec!["other".into()],
            "sheet" | "m" => vec!["q".into()],
            "k3" => vec!["m".into()],
            "cat" => vec!["k3".into()],
            _ => vec![],
        }
    }
}
fn roots() -> Vec<String> {
    vec!["q".into(), "sheet".into(), "m".into()]
}
fn check(g: &mut WriteGuard, id: &str) -> Result<Option<Target>, String> {
    run(g.check(
        "PATCH",
        &format!("{D}/{id}"),
        "{}",
        || async { roots() },
        parents,
        || async { Some("root".into()) },
    ))
}
#[test]
fn own_files() {
    let mut g = WriteGuard::default();
    assert!(check(&mut g, "sheet").is_ok());
    assert!(check(&mut g, "q").is_ok());
    assert_eq!(
        quiz_file_ids(
            &json!({"drive":{"folderId":"q","spreadsheetId":"sheet","mediaFolderId":"m","configFileId":null}})
        ),
        roots()
    );
}
#[test]
fn nested_files_cached() {
    let mut g = WriteGuard::default();
    let calls = Cell::new(0);
    for _ in 0..2 {
        let before = calls.get();
        assert!(
            run(g.check(
                "PATCH",
                &format!("{D}/cat"),
                "{}",
                || async { roots() },
                |id| {
                    calls.set(calls.get() + 1);
                    parents(id)
                },
                || async { Some("root".into()) }
            ))
            .is_ok()
        );
        if before > 0 {
            assert_eq!(calls.get(), before)
        }
    }
    assert!(check(&mut g, "k3").is_ok());
}
#[test]
fn outside_quizzes() {
    let mut g = WriteGuard::default();
    assert!(check(&mut g, "private").is_err());
    assert!(
        run(g.check(
            "POST",
            &format!("{S}/elsewhere/values:batchUpdate"),
            "{}",
            || async { roots() },
            parents,
            || async { Some("root".into()) }
        ))
        .is_err()
    );
    assert!(
        run(g.check(
            "POST",
            U,
            "{\"parents\":[\"other\"]}",
            || async { roots() },
            parents,
            || async { Some("root".into()) }
        ))
        .is_err()
    );
}
#[test]
fn my_drive_is_never_a_quiz() {
    let mut g = WriteGuard::default();
    for (id, allowed) in [("loose", true), ("private", false), ("root", false)] {
        assert_eq!(
            run(g.check(
                "PATCH",
                &format!("{D}/{id}"),
                "{}",
                || async { vec!["root".into(), "loose".into()] },
                parents,
                || async { Some("root".into()) }
            ))
            .is_ok(),
            allowed
        );
    }
}
#[test]
fn no_delete_even_owned() {
    let mut g = WriteGuard::default();
    assert!(
        run(g.check(
            "DELETE",
            &format!("{D}/sheet"),
            "",
            || async { roots() },
            parents,
            || async { Some("root".into()) }
        ))
        .is_err()
    );
}
#[test]
fn created_only_trash_and_write() {
    let mut g = WriteGuard::default();
    assert!(
        run(g.check(
            "PATCH",
            &format!("{D}/q"),
            "{\"trashed\":true}",
            || async { vec![] },
            parents,
            || async { None }
        ))
        .is_err()
    );
    g.created("newQuiz");
    for (m, u, b) in [
        (
            "POST",
            format!("{D}/template/copy"),
            "{\"parents\":[\"newQuiz\"]}",
        ),
        ("PATCH", format!("{D}/newQuiz"), "{\"trashed\":true}"),
    ] {
        assert!(run(g.check(m, &u, b, || async { vec![] }, parents, || async { None })).is_ok());
    }
}
#[test]
fn quizzes_opened_later() {
    let mut g = WriteGuard::default();
    let roots = RefCell::new(vec![]);
    assert!(
        run(g.check(
            "PATCH",
            &format!("{D}/sheet"),
            "{}",
            || async { roots.borrow().clone() },
            parents,
            || async { Some("root".into()) }
        ))
        .is_err()
    );
    roots.borrow_mut().push("q".into());
    assert!(
        run(g.check(
            "PATCH",
            &format!("{D}/sheet"),
            "{}",
            || async { roots.borrow().clone() },
            parents,
            || async { Some("root".into()) }
        ))
        .is_ok()
    );
}
#[test]
fn cycles_and_eight_level_limit() {
    let mut g = WriteGuard::default();
    assert!(
        run(g.check(
            "PATCH",
            &format!("{D}/cycle"),
            "{}",
            || async { roots() },
            |id| async move { vec![id] },
            || async { None }
        ))
        .is_err()
    );
    let mut g = WriteGuard::default();
    assert!(
        run(g.check(
            "PATCH",
            &format!("{D}/0"),
            "{}",
            || async { vec!["9".into()] },
            |id| async move { vec![(id.parse::<u8>().unwrap() + 1).to_string()] },
            || async { None }
        ))
        .is_err()
    );
}
