//! Themes: the file format and its normalization, built-in themes, zip packs and the workshop index.

/// Built-in themes (port of src/quiz/themes.js).
pub mod builtin;
/// The theme.json format and `normalize_theme` (port of src/quiz/themeFormat.js).
pub mod format;
/// Theme .zip packs (port of src/quiz/themePack.js).
pub mod pack;
/// The workshop index (port of src/quiz/workshop.js).
pub mod workshop;

pub use format::*;
