use quiz_core::{
    quiz::model::{Quiz, TeamColor},
    theme::format::normalize_theme,
};
use quiz_store::{package::*, *};
use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};

// MemoryStore futures never suspend; no runtime dependency is necessary.
fn run<T>(future: impl Future<Output = T>) -> T {
    match pin!(future).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("MemoryStore unexpectedly suspended"),
    }
}
fn media(name: &str, bytes: &[u8]) -> Media {
    Media {
        name: name.into(),
        bytes: bytes.into(),
        content_type: "image/png".into(),
        ..Default::default()
    }
}
#[test]
fn quiz_and_media_contract() {
    run(async {
        let store = MemoryStore::new();
        assert!(store.list_quizzes().await.unwrap().is_empty());
        assert!(store.get_quiz("missing").await.unwrap().is_none());
        assert!(
            store
                .update_quiz("missing", |_| panic!())
                .await
                .unwrap()
                .is_none()
        );
        let quiz = Quiz::new("one", 1);
        let saved = store.put_quiz(&quiz).await.unwrap();
        assert!(saved.updated_at > 1);
        assert_eq!(saved.created_at, 1);
        store
            .put_media("one", &[media("A.png", &[1]), media("b.png", &[2])])
            .await
            .unwrap();
        store
            .put_media("two", &[media("A.png", &[3])])
            .await
            .unwrap();
        store
            .put_media("one", &[media("A.png", &[4])])
            .await
            .unwrap();
        assert_eq!(store.list_media("one").await.unwrap().len(), 2);
        assert_eq!(
            store
                .get_media("one", "A.png")
                .await
                .unwrap()
                .unwrap()
                .bytes,
            [4]
        );
        assert!(store.get_media("one", "a.png").await.unwrap().is_none());
        let changed = store
            .update_quiz("one", |q| {
                q.id = "bad".into();
                q.title = "Changed".into();
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(changed.id, "one");
        assert_eq!(changed.title, "Changed");
        assert_eq!(store.list_quizzes().await.unwrap().len(), 1);
        store.delete_media("one", "b.png").await.unwrap();
        store.delete_media("one", "b.png").await.unwrap();
        store.delete_quiz("one").await.unwrap();
        store.delete_quiz("one").await.unwrap();
        assert!(store.get_quiz("one").await.unwrap().is_none());
        assert!(store.list_media("one").await.unwrap().is_empty());
        assert_eq!(store.list_media("two").await.unwrap().len(), 1);
    });
}
#[test]
fn theme_contract() {
    run(async {
        let store = MemoryStore::new();
        let theme =
            normalize_theme(&serde_json::json!({"format": "quiznight-theme", "version": 1, "id": "test", "name": "Test"}))
                .theme
                .unwrap();
        let entry = LibraryTheme {
            id: "theme".into(),
            theme,
            saved_at: 1,
            files: vec![media("font.woff2", &[0, 1])],
        };
        let saved = store.put_theme(&entry).await.unwrap();
        assert!(saved.saved_at > 1);
        let loaded = store.get_theme("theme").await.unwrap().unwrap();
        assert_eq!(loaded.files, saved.files);
        assert_eq!(
            serde_json::to_value(loaded.theme).unwrap(),
            serde_json::to_value(saved.theme).unwrap()
        );
        assert_eq!(store.list_themes().await.unwrap().len(), 1);
        store.delete_theme("theme").await.unwrap();
        store.delete_theme("theme").await.unwrap();
        assert!(store.get_theme("theme").await.unwrap().is_none());
    });
}
#[test]
fn react_fixture_and_memory_round_trip() {
    run(async {
        let import = read_zip(include_bytes!("react-package.zip"), "fresh", 123).unwrap();
        assert_eq!(import.quiz.title, "React fixture");
        assert_eq!(import.quiz.settings.language, "en");
        assert_eq!(import.quiz.settings.start_time, "20:00");
        assert_eq!(import.quiz.manual_scores["17"]["1"], 9.);
        assert_eq!(import.quiz.teams[0].color, Some(TeamColor::Palette(0)));
        assert_eq!(import.media[0].content_type, "image/png");
        let store = MemoryStore::new();
        save_import(&store, &import).await.unwrap();
        assert!(matches!(
            save_import(&store, &import).await,
            Err(Error::Conflict(_))
        ));
        let bytes = export_zip(&store, &import.quiz).await.unwrap();
        let copy = read_zip(&bytes, "copy", 456).unwrap();
        let mut expected = import.quiz;
        expected.id = "copy".into();
        expected.created_at = 456;
        expected.updated_at = 456;
        assert_eq!(copy.quiz, expected);
        assert_eq!(copy.media, import.media);
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let meta: serde_json::Value =
            serde_json::from_reader(zip.by_name("quiz.json").unwrap()).unwrap();
        for key in ["id", "createdAt", "updatedAt", "builtin", "questionsCsv"] {
            assert!(meta.get(key).is_none());
        }
    });
}
#[test]
fn import_identity_and_collision_rules() {
    let imported = read_entries(vec![
        ("folder/quiz.json".into(), br#"{"id":"old","createdAt":1,"builtin":true,"drive":{"folderId":"x"},"customTheme":{"version":999},"title":""}"#.to_vec()),
        ("folder/Named.CSV".into(), b"csv".to_vec()),
        ("first/PHOTO.png".into(), vec![1]), ("last/photo.PNG".into(), vec![2]),
        ("__MACOSX/junk.jpg".into(), vec![3]), ("media/.hidden.png".into(), vec![4]),
    ], "new", 42).unwrap();
    assert_eq!(imported.quiz.id, "new");
    assert_eq!(imported.quiz.title, "Named");
    assert_eq!(imported.quiz.created_at, 42);
    assert!(!imported.quiz.builtin);
    assert!(imported.quiz.drive.is_none());
    assert!(!imported.quiz.fields.contains_key("customTheme"));
    assert_eq!(imported.media.len(), 1);
    assert_eq!(imported.media[0].name, "photo.PNG");
    assert_eq!(imported.media[0].bytes, [2]);
}
#[test]
fn reject_unsafe_paths_and_bad_json() {
    for path in [
        "../evil.png",
        "/evil.png",
        "C:/evil.png",
        "a\\evil.png",
        "a/../evil.png",
    ] {
        assert!(read_entries(vec![(path.into(), vec![])], "id", 0).is_err());
    }
    assert!(read_entries(vec![("quiz.json".into(), b"{".to_vec())], "id", 0).is_err());
    assert!(read_zip(b"not zip", "id", 0).is_err());
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    zip.start_file("../escape.png", zip::write::SimpleFileOptions::default())
        .unwrap();
    assert!(read_zip(&zip.finish().unwrap().into_inner(), "id", 0).is_err());
}

#[test]
fn duplication_and_referenced_media() {
    run(async {
        let store = MemoryStore::new();
        let mut import = read_zip(include_bytes!("react-package.zip"), "original", 10).unwrap();
        import.quiz.teams.push(quiz_core::quiz::model::Team {
            name: "Remote".into(),
            image: Some("https://example.com/image.png".into()),
            ..Default::default()
        });
        assert_eq!(referenced_media(&import.quiz), ["PHOTO.PNG"]);
        save_import(&store, &import).await.unwrap();
        duplicate_quiz(&store, &import.quiz, "Copy title", "duplicate", 20)
            .await
            .unwrap();
        let copy = store.get_quiz("duplicate").await.unwrap().unwrap();
        assert_eq!(copy.title, "Copy title");
        assert_eq!(copy.created_at, 20);
        assert_eq!(copy.teams, import.quiz.teams);
        assert_eq!(store.list_media("duplicate").await.unwrap(), import.media);
    });
}

#[test]
fn limits_and_deflate() {
    assert!(read_entries(vec![("x".into(), vec![]); MAX_ENTRIES + 1], "id", 0).is_err());
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    zip.start_file(
        "questions.csv",
        zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();
    std::io::Write::write_all(&mut zip, b"hello").unwrap();
    let import = read_zip(&zip.finish().unwrap().into_inner(), "id", 0).unwrap();
    assert_eq!(import.quiz.questions_csv, "hello");
}
