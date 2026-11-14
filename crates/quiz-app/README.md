# quiz-app

The Leptos (client-side rendered) front end: quiz library, show, host window, editors and help.
Progress and open work are tracked in `docs/RUST_REWRITE.md`.

```sh
trunk serve --port 8097     # http://localhost:8097/
cargo test -p quiz-app      # host-side unit tests
```

## URLs

Hash routes: `#/` (library), `#/quiz/<id>/edit`, `/questions`, `/theme`, `/check`,
`/map`, `/attendance`, `/scores`, `/show`, `/presenter`, plus `#/themes` and `#/help[/section]`.

Without a hash, these development parameters open the demo show:
`?show=demo`, `?round=<n>`, `?mode=questions`, `?theme=<id>`, `?csv=<url>`, `?scores=<url>`,
`?dash=tiles|timeline`, `?welcome=wall|cards`, `?role=presenter`.

`?demo=stage|leaderboard|presence|confetti` opens the animation spikes from phase 1.

## Layout

- `src/scenes/`: the show screens. `src/pages/`: library, editors, help. Each page exports
  `#[component] pub fn Page(...)` and is mounted by the router in `src/main.rs`.
- `styles/`: one CSS file per screen, imported by `style.css`. Tailwind only generates class names that
  appear literally in the source, so never build class names at run time.
- `i18n/`: `cs.json`, `en.json`, `sk.json`, the interface texts (`sk.json` is deliberately Hungarian).
- Storage goes through `quiz-store`. If IndexedDB cannot be opened the app falls back to memory and
  shows a warning.
