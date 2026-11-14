# Google API and Drive UI integration

This module implements the React browser-only GIS token model. No backend, API key, refresh token,
redirect OAuth flow, or new dependencies are required. It is compiled by `mod google;`; the UI must
wire it explicitly. All asynchronous operations are local WASM futures, without `Send` bounds.

## Integration contract

1. After opening the existing app store, call `fetch::configure_store(store.clone())` exactly once.
   Use that same store for the library, quiz editing and media. The guard rereads its quiz records;
   never register arbitrary Google IDs as trusted roots. Save the result of opening a Drive quiz
   before attempting writes to its existing files.
2. Inside a Leptos owner, call `auth::provide_google_account()` once; consumers call
   `auth::use_google_account()`. This subscribes to storage events, preloads GIS, and schedules token
   expiry. Preserve that owner for the lifetime of the app. `auth::load_google()` may also be preloaded
   on an invitation page; await `auth::sign_in(false)` from a click. `true` selects another account.
3. `Store::put_quiz` and `Store::update_quiz` now call `sync::notify_quiz_saved` after successful
   local saves. Do not notify a second time in consumers. Use
   `Store::update_quiz_source(id, change, "drive")` for imported/synced changes to avoid a loop;
   `sync_drive_config` already does this. This replaces React's module-level `onQuizSaved`.
4. On opening/refreshing a Drive quiz, call `read_drive_quiz`, merge with core `merge_drive`, persist,
   sync its media, then call `sync_drive_config`. Supply `syncedAt`/`syncedBy` through `merge_drive`'s
   extra object when applicable. Deduplicate library entries using `same_drive_quiz`.
5. Replace public-only CSV calls in question loading/score polling with `sheets::fetch_sheet_csv`.
   Setup question loading and score previews now do this. The show loaders remain for step three.
6. For team images: ensure its media folder, persist returned `mediaFolderId`/`mediaFolderName` on the
   quiz, then upload. Replace confirmation returns a JS Promise of boolean. Upload cancellation returns
   `Ok(None)`; do not assign an image path in that case. Local-only uploads continue using the store.
7. The question editor supplies a planner comparing its original question with the freshly loaded
   sheet. Return `None` on conflict. Show the fresh `QuestionSheet` from `EditError::Conflict` and let
   the user resolve it. `commit_edit` cannot infer which question the user originally opened.
8. Refresh editor presence periodically, read other participants, and call `write_presence(..., None,
   true)` on pagehide. Keepalive still requires an unexpired token and guard authorization.
9. Use invitation and account state to reveal Drive UI, and `drive_error_text`/`sheet_error_text` with
   the existing locale JSON. The library and setup editor now do this; shared locale JSON is unchanged.

## Public browser API (React equivalents)

Paths below are relative to `crate::google`. JSON `Value` records retain the original camelCase schema.
All functions returning `Result` use the error enum below unless otherwise stated.

| API | Purpose / React replacement |
| --- | --- |
| `Error::{Auth(String), Api(u16,String), Blocked(String), Drive(String,String), Sheet(String), Other(String)}` | Typed GoogleAuthError, GoogleApiError, WriteBlockedError, DriveError, SheetError and other failures. |
| `Result<T>` | Shared result alias; errors implement Display/Error. |
| `config::{GOOGLE_CLIENT_ID, GOOGLE_SHEET_TEMPLATE, google_copy_url}` | Single config source, from config.js; `googleCopyUrl`. |
| `auth::User { name, email, photo }` | Saved Google user; photo is optional. |
| `auth::Session { token, expires, user }` | Same `quiznight.google` JSON; expiry is epoch milliseconds, token may be null. |
| `auth::{STORAGE_KEY, INVITE_KEY, SCOPE}` | Original storage keys and full Drive scope. |
| `auth::google_enabled()` | `googleEnabled`. |
| `auth::has_drive_invite()` | `hasDriveInvite`. |
| `auth::accept_drive_invite()` | `acceptDriveInvite`; writes `quiznight.driveInvite = "1"`. |
| `auth::get_session()` | `getSession`; retains known user when token expires. |
| `auth::current_token()` | `currentToken`; requires over 60 seconds left. |
| `auth::subscribe_google(callback) -> Subscription` | `subscribeGoogle`; dropping subscription removes it. |
| `auth::load_google().await` | `loadGoogle`; cached/retryable dynamic GIS script loading. |
| `auth::sign_in(choose_account).await` | `signIn({chooseAccount})`; supersedes earlier pending sign-in. |
| `auth::sign_out()` | `signOut`; revokes when GIS is loaded, clears session immediately. |
| `auth::GoogleAccount { session, signed_in }` | Reactive account signals; replaces useGoogleAccount. |
| `auth::provide_google_account()` | Create/provide the account context under the root owner. |
| `auth::use_google_account()` | Read provided account context. |
| `fetch::configure_store(store)` | Hook the write guard and sync/media APIs to the existing store. |
| `fetch::Options { method, headers, body, keepalive, no_store }` | Fetch options; empty method means GET, body is an optional string. |
| `fetch::Options::json(method, &value)` | Build a JSON request with Content-Type. |
| `fetch::google_fetch(url, options).await -> Response` | `googleFetch`, including guard, Authorization and 401 session invalidation. |
| `fetch::google_json(url, options).await -> Value` | `googleJson`. |
| `fetch::post(url, &body).await -> Value` | Shared JSON POST helper from driveConfig/sheetEdit. |
| `drive::read_drive_quiz(link).await -> Value` | `readDriveQuiz`; returns `{drive, questionsCsv, questionsSheetUrl, scoresSheetUrl}`. |
| `drive::ensure_media_folder(&drive).await -> Value` | `ensureMediaFolder`; returns `{id,name}`, does not persist quiz fields. |
| `drive::upload_media(quiz_id, folder_id, path, &Blob, progress, confirm).await` | `uploadMedia`; XHR resumable PUT, optional progress callback, optional asynchronous replacement confirmation, and local media persistence. |
| `drive::create_drive_quiz(title, parent_link, template_url).await -> String` | `createDriveQuiz`; returns folder ID, trashes its own incomplete folder on failure. |
| `drive::list_people(file_id).await -> Value` | `listPeople`; `{people,canShare}` with role ordering. |
| `drive::invite_person(file_id, email, role, message).await -> Value` | `invitePerson`; notification email and optional message. |
| `drive::remove_person(file_id, permission_id).await` | `removePerson`; DELETE permission only. |
| `drive::sync_drive_media(quiz_id, folder_id, progress).await -> MediaSync` | `syncDriveMedia`; three streamed downloads, change detection, removal and duplicate names. |
| `drive::Progress { done, total, bytes, total_bytes }` | Download progress callback payload. |
| `drive::MediaSync { downloaded, kept, removed, duplicates }` | Sync result. |
| `drive::to_drive_error(error)` | `toDriveError`; auth/blocked/access/API conversion. |
| `drive::drive_error_text(&error, &translations, email) -> Option<String>` | `driveErrorText`; None for closed/superseded popup, existing gd_err_* translations. |
| `drive::{parse_drive_link, sheet_url, is_media_folder_name}` | Convenience core reexports of parseDriveLink/sheetUrl/isMediaFolderName. |
| `sheets::Tab { sheet_id, title, index }` | Sheet tab properties; serialized fields use camelCase. |
| `sheets::sheet_tabs(id).await -> Vec<Tab>` | `sheetTabs`; five-minute cache, sorted by index. |
| `sheets::tab_values(id, title).await` | Authenticated values reader shared by sync/editor. |
| `sheets::rows(&Value)` | Sheets scalar values to string rows. |
| `sheets::gid(url)` | Extract optional numeric tab ID. |
| `sheets::fetch_sheet_csv(url).await` | `fetchSheetCsv`; private API first, public CSV fallback with no-store/cache buster. |
| `sheets::sheet_error_text(&error, &translations)` | `sheetErrorText`; existing ed_err_http/ed_err_network translations. |
| `sheets::{rows_to_csv, tab_range, to_csv_url}` | Core reexports of rowsToCsv/tabRange/toCsvUrl. |
| `sheet_edit::QuestionSheet { spreadsheet_id, title, tab, sheet }` | `loadQuestionSheet` payload, using the existing core sheet model. |
| `sheet_edit::ConflictError { data }` | ConflictError with fresh QuestionSheet. |
| `sheet_edit::EditError::{Conflict(Box<ConflictError>), Google(Error)}` | Typed planner/save failure. |
| `sheet_edit::load_question_sheet(&quiz).await` | `loadQuestionSheet`; resolves by gid, then saved tab name, then first tab. |
| `sheet_edit::can_edit_sheet(id).await -> bool` | `canEditSheet`. |
| `sheet_edit::commit_edit(&quiz, planner).await` | `commitEdit`; fresh read, plan, structure/RAW/USER_ENTERED requests, fresh read. |
| `sheet_edit::Presence { key, at, question, typing, name }` | Editor presence; at is seconds. |
| `sheet_edit::presence_key(email)` | `presenceKey`; compatible FNV hash in base 36. |
| `sheet_edit::read_presence(id).await` | `readPresence`; excludes entries at least 60 seconds old. |
| `sheet_edit::write_presence(id, key, entry, keepalive).await` | `writePresence`; clears stale entries older than ten minutes. |
| `sync::SyncState { phase, at, error }` | `getSyncState` payload; at retains last successful time. |
| `sync::Phase::{Idle,Pending,Syncing,Done,Error}` | Original sync phases. |
| `sync::get_sync_state(quiz_id)` | `getSyncState`. |
| `sync::on_sync_state(callback) -> SyncSubscription` | `onSyncState`; callback receives quiz ID and state; drop unsubscribes. |
| `sync::use_drive_sync(quiz_id) -> RwSignal<SyncState>` | `useDriveSync`; automatically unsubscribes with owner. Recreate under an owner when quiz ID changes. |
| `sync::sync_drive_config(quiz_id).await -> Option<Quiz>` | `syncDriveConfig`; serializes calls per quiz, merges, reconciles, saves with atomic store update. |
| `sync::notify_quiz_saved(&quiz, source)` | React onQuizSaved hook with 1500 ms debounce. |
| `sync::write_scores_tab(id, &tab, &table, &ops).await` | `writeScoresTab`; structure before RAW cell writes. |
| `sync::new_team_id()` | `newTeamId`; t plus up to eight random base-36 characters. |

## Pure core API

Paths below are under `quiz_core::quiz`; these contain no browser code.

| API | Purpose / React replacement |
| --- | --- |
| `drive::{GOOGLE_CLIENT_ID, GOOGLE_SHEET_TEMPLATE, DRIVE_API, UPLOAD_API, SHEETS_API, FOLDER, SPREADSHEET}` | Shared configuration and Google endpoint/MIME constants. |
| `drive::{google_copy_url, parse_drive_link, folder_url, sheet_url, is_media_folder_name}` | Config/link helpers. |
| `drive::same_drive_quiz(&a, &b)` | `sameDriveQuiz`, folder-first/Sheet fallback identity. |
| `drive::merge_drive(&mut quiz, &fields, &extra)` | `mergeDrive`; preserves manually set scores link when remote has none. |
| `drive::{encode_component, form_query}` | JavaScript-compatible range and URLSearchParams encoding. |
| `write_guard::{Kind, Target}` | Write classification kind and affected IDs. |
| `write_guard::write_targets(method, url, body)` | `writeTargets`; Result<Option<Target>,String>, error is refusal reason. |
| `write_guard::body_parents(body)` | `bodyParents`; JSON or multipart metadata. |
| `write_guard::quiz_file_ids(&quiz_json)` | `quizFileIds`. |
| `write_guard::WriteGuard::{default, check, created}` | `createWriteGuard`; async injected roots/parents/root readers, cache, eight-level walk and created-only trash. Reader failures are represented as empty parents/None. |
| `drive_config::CONFIG_NAME` | `quiznight.json`. |
| `drive_config::merge3(base,local,remote)` | `merge3`; Option<&Value> distinguishes missing from JSON null. |
| `drive_config::overlay(&local,&remote)` | First-sync overlay. |
| `drive_config::upgrade(config)` | Version-one team-name config to version two placeholders. |
| `drive_config::config_from_quiz(&quiz)` | `configFromQuiz`; archives, confirmed teams, settings and stored custom theme. |
| `drive_config::apply_config(&quiz,&config,order)` | `applyConfig`; preserves drafts, validates theme and archives hidden IDs. |
| `drive_config::ScoreRow { row,name,id }` | Named scores-tab row; row is zero-based. |
| `drive_config::ScoresTable` | Header/name/id columns, width/end, named/empty rows. |
| `drive_config::scores_table(&rows)` | `scoresTable`; None without team-name column. |
| `drive_config::Ops { ids,names,clears,appends }` | Reconciliation operations, retaining original JSON shapes. |
| `drive_config::Reconciled { entries,order,ops }` | Reconciliation result. |
| `drive_config::reconcile(rows,entries,base,local,can_write,new_id)` | `reconcile`; ID generator is injected for deterministic tests. |
| `drive_config::scores_writes(table,sheet_id,quoted_name,ops)` | `scoresWrites`; `{structure,cells}` JSON request bodies. |
| `drive_config::column(index)` | Zero-based column to A1 letters. |
| `sheets::{tab_range,to_csv_url,rows_to_csv}` | Existing helpers extended with Papa-compatible CSV writing. |
| `sheet_presence::{Presence,presence_key,read_presence,presence_properties}` | Pure presence hash, parser, expiry, cleanup and payload building. |

## Fidelity, differences and verification limits

- Model/storage/team fields already round-trip through Quiz.drive and flattened extension fields.
  Media driveId/driveModified live in Media.fields. ZIP import already drops Drive identity; this
  behavior was retained. No storage migrations were introduced.
- Rust types use snake_case fields, with original camelCase JSON for Google/stored data. Error classes
  become enum variants. The planner signals conflict using Option instead of throwing.
- Access capability and config/tab reads are sequential rather than Promise.all; results and failure
  policy match, but latency can differ. Concurrent config calls serialize through in-window promise chains,
  including after an awaiting UI owner is disposed. This is not a cross-window lock, as in React.
- Permission tie ordering uses lowercase Unicode string order rather than browser localeCompare.
- JSON objects use serde_json's sorted map order. A config without a scores tab therefore defaults
  to ID-sorted teams instead of JavaScript insertion order; scores-tab order is preserved explicitly.
- Presence values preserve React's UTF-16 length limit, except Rust cannot retain a trailing unpaired
  surrogate; a non-BMP character at the boundary is omitted whole. The original 124-character
  budget is retained, including its limitations for UTF-8 byte limits on Drive.
- Media downloads buffer the complete response bytes to use the existing MediaStore API; chunk
  progress still streams, with three workers. Large files may use more memory than React's Blobs.
- Browser event callbacks for GIS/script loading live for the window lifetime. Subscription handles
  and Leptos timers are cleaned up on owner disposal.
- The Drive guard's created-file cache belongs to the window; storage roots are re-read as needed.
  Guard requests cannot be sent before configuration except for safe empty-folder creation/reads.
- No real Google sign-in, popup, upload, shared-drive permission, CORS, cross-tab storage event or
  keepalive request can be exercised in this sandbox. Highest live-test risks are GIS activation from
  async click handlers, resumable upload Location/CORS handling, browser lifecycle timing for presence,
  private-sheet/tab selection, and concurrent editors. Run these on the configured GitHub Pages origin.

All original writeGuard.test.js and driveConfig.test.js scenarios are ported to core integration
 tests, with additional links, encoding, merging, CSV, presence and browser-module error/request tests.

## Step two implementation and handoff

- `main.rs` configures the shared Store once after open. Shell provides one Google account context.
  The existing router's `Drive(None)` / `Drive(Some(link))` now reach the library's Drive panel /
  invitation overlay, accepting the invite flag only when Google is enabled.
- `pages/drive_parts.rs` provides the account chip, beta badge, save status and download/upload
  progress. `pages/drive_people.rs` lists permissions, invites with writer/reader role and removes
  permissions with confirmation; owner/organizer/self are protected and `canShare` gates changes.
- Library opening deduplicates with `same_drive_quiz`, merges and persists the record before entering
  setup. Invitation auto-opening preserves the existing-local-folder shortcut without sign-in.
- `pages/quiz_editor/drive_ui.rs` refreshes questions/tabs, merges config, downloads media and records
  `syncedAt` / `syncedBy`; setup auto-runs it once on a signed-in visit. Config saves use the Store
  hook and show sync status. Setup rereads storage revisions only while its patch queue is clean.
  Uploads preserve nested paths, strip a dropped media-root directory, persist a newly created media
  folder before uploading, and ask once per drop before replacing. Team photo cancellation does not
  assign an image. Permissions/share links, offline/media warnings, Google checklist and score-tab
  previews use existing translated texts.

Step-two requests (historical; see the step-three handoff below for their final status):

1. `pages/questions_editor.rs`: use `sheet_edit::load_question_sheet` / `commit_edit` with an
   original-vs-fresh conflict planner, readonly/sign-in notices and participant presence lifecycle.
2. Setup `Teams` still needs the React scores-tab draft/Enter-confirm/remove-confirm lifecycle,
   plus attendance/scores-entry write-back integration. The general Store hook already schedules
   config sync; use source `"drive"` for remote imports. Avoid double notifications.
3. `quiz_context.rs::load_stored` still fetches public CSV directly. Replace that read with
   `google::sheets::fetch_sheet_csv(&quiz.questions_sheet_url)` while retaining the stored CSV on
   failure (offline fallback). This is the show/bundle path and was outside step-two file ownership.
4. `scores.rs::fetch_scores` (and its loader/polling callers) must use `google::sheets::fetch_sheet_csv`
   instead of public-only HTTP. Preserve polling cancellation and fallback behavior; use
   `sheet_error_text` for translated failures. The show also needs the Google session/sign-in notices
   from React `useLiveScores`/show. No changes were made to `scores.rs`, show or scenes in step two.
5. Live-test on the configured origin: GIS sign-in/expiry/other-account/retry, opening private Sheets,
   view-only sharing, upload replacements/cancellation and resumable XHR/CORS, dropped directories,
   and concurrent setup/config edits. None can be exercised without a real Google session here.

The port preserves texts and workflow conditions, but does not reproduce every Lucide glyph or
pixel-level React layout. Permissions use inline SVG/photos or text controls. The highest untested
UI risks are GIS activation timing, owner disposal while requests are in flight, and setup edits
racing a remote merge; the patch queue and source flag avoid routine local overwrite/sync loops.

## Step three implementation and handoff

- Questions editing keeps the existing local/CSV workflow. Drive mutations use `commit_edit` with
  original block matching and a neighbour check for moves. Conflicts adopt the remote Sheet, retain
  the draft and offer the remote version. Cached CSV updates use source `drive` and retain the link.
  Reloads and 15-second polling refind the open question; polling checks the mutation generation and
  stored CSV before adoption. Read-only users can browse and preview but cannot mutate or upload.
- Presence refreshes on selection/typing changes and every 20 seconds, reads other editors on the
  question poll, and removes the entry with keepalive on pagehide and editor cleanup.
- Question and attendance uploads share `drive::upload_quiz_media`: ensure/persist the media folder
  before upload, return cancellation without assigning a path, and use resumable upload progress.
- Setup scores-tab teams use `draft` until Enter/blur and confirm removal of confirmed rows. Attendance
  uses the Store config hook, refreshes remote config on sign-in/every 15 seconds, retains drafts and
  confirms their names on Enter/blur. No duplicate save notifications were added.
- Drive score entry imports scores and reconciles teams on signed-in load. Edits save locally and use
  ordered, coalesced, single-cell RAW writes to the linked scores tab. Targets require an unambiguous
  team ID (or a unique name on legacy rows without IDs) and round header; missing targets refuse writes.
  This extends React, whose manual score table only wrote local scores and warned for linked Sheets.
  A failed remote score write is reported; it is not durably queued for automatic retry after navigation.
- `scores.rs` reads via `fetch_sheet_csv`, retains successful scores after failures, and reloads on
  account changes. Auth-needed is set on failed reads for non-published Google Sheet links when the
  quiz is from Drive or a Google user is known and no valid token remains. The host offers renewal
  during the final ten minutes; the leaderboard shows the host warning and TV empty-state sign-in.
  The existing auth provider shares `quiznight.google` through localStorage and storage events.
- `show.rs::load_stored_authenticated` supplies authenticated question loading with stored CSV fallback
  within step-three ownership. The general `quiz_context::load_stored` remains public-only; its quiz
  check/map callers need the main session to make the same transport change, then deduplicate the show
  wrapper. No reveal/overlay/view-transition behavior was changed.

Still requires real Google/browser verification: concurrent editors and row moves/deletions, lifecycle
keepalive, GIS popup activation, view-only sharing, legacy score headers, score writes concurrent with
team reconciliation, upload replacement/CORS, and host/TV token expiry/renewal. Local Rust checks do
not establish live Google feature parity. `qe_not_drive` is intentionally not used to reject local
quizzes: the task requires retaining the Rust local editor even though React rejected those quizzes.

Step-three validation: WASM `cargo check --offline -p quiz-app --target wasm32-unknown-unknown`
passes with zero warnings. `cargo test --offline --workspace` fails only at the two expected
quiz-cli localhost-binding tests (`Operation not permitted`). Running the complete workspace with
only `binary_checks_and_adds_from_a_url` and `download_follows_redirects_and_stops_at_the_limit`
skipped passes all 235 runnable tests, with zero warnings. New coverage checks presence selection
keys, Google notice/expiry conditions, stable score targets/ambiguity, and remote score adoption.
`rg 'TODO\(auth\)' crates` finds no markers. No browser, ports, Trunk, commit, or push was used.

### Manual account test

1. On the configured origin, sign in, create/open a Drive quiz with private question and score tabs,
   refresh it, and confirm stored questions still open offline. Also open an existing public-link
   quiz and a local-only quiz signed out; their prior editing/show behavior must still work.
2. Save, add, rename, move and delete questions. Inspect Sheet cells, author/time stamps and the
   preserved Sheet link. Open two editors as different users, edit the same question, save in the
   first and then the second; check the conflict draft, remote answer and both resolution paths.
   Observe here/open/typing notices, then close one tab and check its presence disappears.
3. Repeat with a viewer-only Google account: browse/preview/reload, verify mutation/upload controls
   cannot write. Sign out and back in; verify the sign-in state and successful reload.
4. Upload question and attendance images, including a nested path and a quiz without a media folder.
   Verify folder creation, progress, preview, replacement acceptance and cancellation without changing
   the draft reference. Confirm the actual files are in Drive.
5. Add a team and type a name: it must have no scores row until Enter/blur. Rename it, cancel and then
   accept removal. Change attendance, players, quote, image and seat in another window; check config
   and other windows after sync. Edit a team directly in the scores tab and refresh setup/attendance.
6. Open the quiz's scores route, edit/clear decimal and negative scores, and inspect the exact Sheet
   cells and live leaderboard. Rename/reorder teams between edits to verify stable ID matching.
   Disconnect the network for a write: an error must appear; reconnect and re-enter the failed cell
   (failed writes are not durably retried after leaving the page).
7. Launch host and TV against private Sheets. Verify questions and score polling, cached scores on
   failed reads, host renewal within ten minutes of token expiry, and sign-in-needed after an expired
   token fails to read. Renew in one window and confirm both reload via shared localStorage. With an
   empty leaderboard, confirm the TV's sign-in button; a populated TV should keep its last scores.

### Needs from the main session

- Replace the public fetch in `quiz_context.rs::load_stored` with `fetch_sheet_csv`, retaining cached
  questions on failure, for quiz check/map. Afterwards consolidate `show.rs`'s owned wrapper with that
  loader. This file was outside the implementation agent's allowed edits.
- Execute the real-account script above. The implementation has no browser/sign-in verification.
- The new Drive manual-score writer deliberately refuses missing/ambiguous team or round cells;
  verify the live template's headers and concurrent scores-row reconciliation. Failed writes are
  visible but require re-entering the cell; a durable offline score queue is not implemented.
