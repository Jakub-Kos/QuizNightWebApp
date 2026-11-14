//! Quiz Night logic with no web dependencies. See docs/RUST_REWRITE.md.

/// Quiz model, question parsing, validation, teams and scores (port of src/quiz/*.js).
pub mod quiz;
/// The show as a pure state machine and the messages its windows exchange.
pub mod show;
/// Theme format, built-in themes, theme packs and the workshop index.
pub mod theme;
pub mod sounds;
