# Rust rewrite

Quiz Night OS rewritten in Rust (mostly as a joke). This file tracks what is done and what is left.

## Status (2026-11-14)

The repository is Rust only. The React app was removed before the Rust app reached parity; it is in
the git history (last on `main` at the time) and is the visual reference for anything still to port.

Everything is on the `rust-rewrite` branch and nothing has been deployed: `main` and GitHub Pages still
serve the React app.

| Area                             | State                                                           |
| -------------------------------- | --------------------------------------------------------------- |
| Logic (`quiz-core`)              | Ported, with tests against output dumped from the React code.   |
| Show (TV and host windows)       | Ported. Every screen seen working in headless Chromium.         |
| Library and storage              | Ported. Demo seeding, saving and reopening a quiz seen working. |
| Editors and other pages          | Ported. Each page seen loading; none clicked through.           |
| Theme workshop tool (`quiz-cli`) | Ported, with tests.                                             |
| GitHub workflows                 | Switched to Rust, never run.                                    |
| Backend (SQL, media, sync, auth) | Not started.                                                    |

All checks so far were done in headless Chromium by taking screenshots. Nobody has run a whole quiz
night with it, watched it on a real TV, or heard the sounds.

### Seen working in the browser

- Welcome (cards and team wall), rules, prizes, rounds dashboard (columns, tiles, timeline).
- Question screen: all twelve types, media, zoom, the host view, and the TV following the host window.
- Leaderboard: reveal steps, points overlay, final sort, host view. Pause screen. Settings. Calibration.
- Theme backdrop and decorations (Classic, Halloween, Christmas).
- Library: demo quiz seeded into IndexedDB, new quiz saved, stored quiz opened as a show.
- First screen of: quiz editor, questions editor, theme editor, theme workshop, quiz check, quiz map,
  attendance, scores entry, help.

### Ported but never tried

- Editing in any editor: typing, saving, reloading, uploads, folder drops, CSV and Sheet imports.
- Quiz zip export and import, delete and duplicate.
- The show launcher (two windows on two screens) and fullscreen.
- Scores typed in one tab reaching a running show in another.
- The guided tour.
- Sounds.
- Opening quizzes saved by the React app. The origin is the same on GitHub Pages, so existing users'
  data will be read by the Rust app on the first deploy.
- Both GitHub workflows (`deploy.yml`, `theme-check.yml`).

### Google (sign-in, Drive, Sheets)

Ported the way the React app did it on GitHub Pages: browser-only sign-in with the same client ID,
storage keys and Drive folder layout. The code is in `crates/quiz-app/src/google/` (its README lists
the API, the differences from React and a manual test script) and in the library, quiz editor,
questions editor, attendance, scores and show.

None of it has run against Google: it needs a real sign-in from an origin registered for the client
ID. Seen in the browser only: signed out, the app looks as before; `#/drive` sets the invite flag and
the library then shows the Google Drive block. Most likely to be wrong: the sign-in popup, large
media uploads (cross-origin rules), picking the tab of a private Sheet, presence cleanup when a tab
closes, and score writes at the same time as team-row changes. A failed score write to the Sheet is
reported but not retried.

### Known gaps against the React app

- The questions editor edits local quizzes too; React only edited Drive quizzes there.
- The colour picker, map page and theme editor were brought closer to React in the last round, but
  nobody has compared them side by side.

### Other open items

- The JSON test fixtures were dumped from the React code. With the React code gone they cannot be
  regenerated; a change in expected behaviour means editing them by hand.
- The theme review report prints a command containing the submitted URL
  (`cargo run -p quiz-cli -- add <url>`). A crafted URL could inject shell text if someone pastes it.
- One dead-code warning in `crates/quiz-app/src/tour.rs`.

### Next

1. Click through every editor and the show launcher in a real browser, fix what breaks.
2. Run one full quiz night end to end on a TV, with sound.
3. Decide the two open decisions below, then start the backend.

## How it is built

- **Frontend:** Leptos, client-side rendered, compiled to WebAssembly, built with Trunk.
- **Styling:** Tailwind v4 through the standalone CLI (no Node). One CSS file per screen in
  `crates/quiz-app/styles/`.
- **Storage:** the browser's IndexedDB, behind traits in `quiz-store`.
- **Show sync:** `BroadcastChannel` between the TV and host windows.

| Crate        | Contents                                                                                                                                    |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `quiz-core`  | Pure Rust, no web dependencies: model, CSV parsing, validation, scores, sorting, theme format, zip packs, show state machine, spring solver |
| `quiz-store` | `QuizStore`, `MediaStore` and `ThemeStore` traits; IndexedDB and in-memory implementations; quiz zip import and export                      |
| `quiz-app`   | Leptos UI: stage, scenes, editors, theme engine                                                                                             |
| `quiz-cli`   | Theme workshop check, add and index                                                                                                         |

### Rules to keep

- The UI only talks to the store traits, so another store (HTTP, SQL) can be swapped in. Futures are
  not `Send`: WASM is single-threaded in the browser and on Workers.
- The show is a pure reducer in `quiz-core`, and sync messages are one serde enum (`SyncMessage`).
- Question types and scene layouts are enums, so adding one makes the compiler list every place to update.
- Themes are data only (never CSS or code). The theme JSON format is version 1.
- A team's colour is an index into a palette, not a class string.
- `quiz-cli` and `quiz-app` share one `normalize_theme`, so the workshop check cannot drift from the app.
- The show must run with no network (pub wifi). A server is a sync target, never the only copy.

### Animation

| Effect                            | How                                                                           |
| --------------------------------- | ----------------------------------------------------------------------------- |
| Enter animations, staggers        | CSS keyframes with an `--i` custom property for the delay                     |
| Exit animations                   | `Presence` keeps the node mounted until `animationend`                        |
| Reorders (final leaderboard sort) | View Transitions API with a `view-transition-name` per team; FLIP as fallback |
| Springs                           | CSS `linear()` easings generated by the spring solver in `quiz-core`          |
| Confetti, fireworks               | A Rust particle system on a canvas                                            |

Run a view transition only when rows really change places: while it runs, named rows are drawn above
everything else, including overlays.

## Planned backend

Not started. A Rust Cloudflare Worker (`workers-rs`) in a new `quiz-api` crate: D1 for SQL, R2 for
media, a Durable Object for live show sync over WebSocket (a phone as host remote). In SQL the
questions, rounds, teams and scores become tables, and a Google Sheet is one way to fill them.

Moving from GitHub Pages to Cloudflare changes the origin, so users would migrate by zip export.

## Open decisions

- **Google Drive sync:** ported as it was for GitHub Pages. Whether to keep it once there is a backend, and what to do for local use, is undecided.
- **Auth:** proposed Google sign-in, with anonymous local-only use still working.
