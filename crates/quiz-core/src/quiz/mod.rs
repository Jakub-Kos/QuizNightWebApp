//! Port of the pure logic in src/quiz/*.js. One submodule per JS file.

pub mod demo_teams;
pub mod examples;
pub mod initials;
pub mod intro;
pub mod model;
pub mod parse;
pub mod scores;
pub mod sheet_model;
pub mod sheets;
pub mod sort;
pub mod teams;
pub mod validate;

pub use parse::*;
