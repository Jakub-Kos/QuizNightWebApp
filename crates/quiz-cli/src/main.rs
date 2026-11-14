//! `quiz-cli`: the theme workshop command, run from the repository root.
//!
//!   quiz-cli [--dir <workshop folder>] index
//!   quiz-cli [--dir <workshop folder>] add <zip or URL> [--replace]
//!   quiz-cli [--dir <workshop folder>] check <zip or URL>
//!   quiz-cli [--dir <workshop folder>] check --from-issue      (reads $ISSUE_BODY)
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let body = std::env::var("ISSUE_BODY").ok();
    let code = quiz_cli::run(
        &args,
        body.as_deref(),
        &quiz_cli::utc_today(),
        &quiz_cli::download,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );
    std::process::exit(code);
}
