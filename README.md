<div align="center">

# Quiz Night OS

**Turn your pub quiz into a TV game show.**

Write the questions, drop in your pictures and teams, and run the whole night from your laptop:<br>
the TV shows the show, a second window gives the host the answers and the controls.

### [▶ Open the app](https://jakub-kos.github.io/QuizNightWebApp/)

Free, nothing to install, no account. Your quizzes stay in your own browser.

</div>

## Documentation

- **How to use the app:** [English](docs/USAGE.md) · [Čeština](docs/USAGE.cs.md) · [Slovenčina](docs/USAGE.sk.md)
- **[Rust rewrite notes](docs/RUST_REWRITE.md)**: what is done, what is left.
- The app's own **Help** page (linked from the quiz library) shows every question type with a live preview.

## What it does

- Twelve question types: multiple choice, yes or no, number, written answer, top list, put in order,
  printed handout, and questions with a picture, sound or video.
- Team introductions, rules and prizes screens, a rounds dashboard, a half-time screen.
- A leaderboard that reveals the round's points group by group, from the lowest score up.
- A host window with the answers, the next step and a seating map of the room.
- Themes: Classic, Halloween, Christmas, your own in the theme editor, or one from the theme workshop.
- English, Czech and Slovak. Works best in Chrome or Edge.

## For developers

Written in Rust: [Leptos](https://leptos.dev/) compiled to WebAssembly, built with
[Trunk](https://trunkrs.dev/), styled with Tailwind CSS v4.

There is no backend: quizzes and media live in the browser's IndexedDB, the TV and host windows talk
over `BroadcastChannel`, and Google Sheets are read as CSV.

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk

cargo test --workspace          # logic, store, CLI and app tests

cd crates/quiz-app
trunk serve --port 8097         # http://localhost:8097/
trunk build --release           # production build in crates/quiz-app/dist/
```

Every push to `main` tests, builds and deploys to GitHub Pages.

| Path | Contents |
| --- | --- |
| `crates/quiz-core` | Model, CSV parsing, validation, scores, themes, show state machine. No web dependencies. |
| `crates/quiz-store` | Storage traits, IndexedDB and in-memory stores, quiz zip import and export. |
| `crates/quiz-app` | The user interface: show, host window, editors, help. |
| `crates/quiz-cli` | Theme workshop command: check, add, index. |
| `public/` | Demo quiz media, Sheet templates and the workshop themes, copied into the build. |

### Reviewing a workshop theme

Submissions arrive as issues labelled `theme`, with the theme's `.zip` attached. A workflow
(`.github/workflows/theme-check.yml`) checks the file and comments the result on the issue.

1. Look at it: download the zip, import it in any quiz's setup (Look → Import theme) and click through the show.
2. Add it: `cargo run -p quiz-cli -- add <URL or path of the .zip>`. It refuses a theme that would lose
   anything; `--replace` updates a theme with the same id.
3. Commit `public/workshop/` and push. Close the issue.

Themes are plain data (colours, fonts, pictures and built-in effects, never CSS or code).

## License

MIT
