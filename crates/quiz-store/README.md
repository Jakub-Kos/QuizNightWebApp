# quiz-store

Single-threaded async `QuizStore`, `MediaStore`, and `ThemeStore` traits use core's typed quizzes/themes and portable `Media { name, bytes, content_type, fields }`. No `Send` bound, browser type, or runtime dependency is exposed. Generic implementations can be injected into the UI; the traits are not dyn compatible. An HTTP backend can implement these interfaces later (atomic updates need server-side concurrency control).

`MemoryStore::new()` implements every operation on every target. Keep one instance for the app lifetime. It can be selected explicitly if `IndexedDbStore::open().await` fails; fallback is not automatic, so a browser error never silently hides saved quizzes. Saves stamp `updatedAt`/`savedAt`, updates preserve ids, and quiz deletion also removes its media. Missing gets/updates return `None`; deletes are idempotent. Media batch puts upsert exact names atomically. Media reads return bytes plus MIME type and preserve optional Drive metadata.

`IndexedDbStore` is wasm32-only. It uses raw web-sys with request and transaction completion futures. Quiz updates run in one read/write transaction. Blob reads happen after transaction completion. Connections close on drop.

## Existing React schema (unchanged)

- `quiznight`, version **1**: `quizzes`, key path `id`, camelCase quiz records; `media`, key path `key`, nonunique index `quizId` on `quizId`. Media records are `{ key: quizId + '/' + name, quizId, name, blob: Blob, ...extra }`; extra includes `driveId` and `driveModified`.
- `quiznight-library`, version **1**: `themes`, key path `id`, records `{ id, theme, files: [{ name, blob: Blob }], savedAt }`. Theme listings are newest first.

## Quiz zip files

`package::export_zip(store, quiz)` writes stored (uncompressed) zip entries: `quiz.json`, `questions.csv`, and `media/<name>`. JSON excludes `id`, `createdAt`, `updatedAt`, `builtin`, and `questionsCsv`, matching package.js; other metadata is retained on export. This is wire compatibility, not identical zip headers or JSON whitespace.

`read_zip(bytes, fresh_id, now_ms)` and `read_entries(entries, fresh_id, now_ms)` return an `Import`. The caller supplies a new UUID and current timestamp for each import, including duplicates. Import selects the same known metadata fields as JS, merges settings defaults, retains team ids/manual scores, validates custom themes through core with `stored: true`, drops Drive identity, and uses the CSV filename for an empty title. Metadata/CSV can be in a folder. Junk is ignored. Media uses extension-derived MIME types and lowercase basenames for clashes; last file wins. `save_import` saves via both traits and guards accidental id reuse. Like JS, saving quiz plus media is two transactions; the uniqueness precheck is not atomic across clients.

Archive paths reject absolute paths, traversal, backslashes, NUL and colon. Limits: 10,000 entries, 256 MiB per file, 512 MiB aggregate decompressed bytes and input zip bytes. Bounded reads enforce limits even if headers lie. Core's theme pack currently has no archive path/cap helpers to reuse; core is unchanged. Theme validation and its asset format rules are reused directly.

## Verification and remaining work

Offline native tests exercise the MemoryStore contract, a zip fixture generated with the React app's fflate dependency, round trips, identity/name-clash behavior, and invalid input. Regenerate with `node tests/dump-package.mjs` from this crate (module resolution finds root node_modules).

Browser execution is still needed for opening fresh/existing databases, cross-window atomic updates, transaction abort/quota failures, Blob/File conversion, and React↔Rust live reads/writes. No IndexedDB runtime tests run in the native sandbox. HTTP/Cloudflare storage, UI event subscriptions/save source tags, browser persistence requests, object URL lifetimes, file pickers/download UI, automatic fallback selection, demo media fetching, and Drive synchronization are not implemented. For demo export, populate a media store with fetched assets before calling export. Import of loose files accepts portable bytes; browser File/drop traversal belongs to the UI.
