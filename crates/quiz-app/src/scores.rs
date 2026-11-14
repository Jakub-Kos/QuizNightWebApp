//! One live scoreboard for the lifetime of a show, shared by all its scenes.
use crate::quiz_context::{QuizContext, query_param};
use leptos::prelude::*;
use quiz_core::quiz::{
    model::{Quiz, ResolvedTeam, resolve_teams},
    parse::{ParsedScores, parse_scores_csv},
    scores::manual_leaderboard,
};
use quiz_store::QuizStore;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

const DEMO_SCORES: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vTc7xeSY_i7arNqlYwJKIukSjZS-cpBVAjsol62xtGE_bxJLHCrdpAi_pMfpijJ3y7zTJzmYnl7CWiL/pub?gid=728177247&single=true&output=csv";

#[derive(Clone, Copy)]
pub struct ScoresContext {
    pub scores: ReadSignal<ParsedScores>,
    /// Fresh attendance metadata; media leases live in the provider.
    pub teams: ReadSignal<Arc<Vec<ResolvedTeam>>>,
    pub venue_arc: ReadSignal<f64>,
    pub loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub last_loaded: ReadSignal<Option<f64>>,
    pub auth_needed: ReadSignal<bool>,
    pub sign_in_ending: Memo<Option<u32>>,
    reload: Callback<()>,
}

impl ScoresContext {
    pub fn reload(self) {
        self.reload.run(());
    }
}

pub fn use_scores() -> ScoresContext {
    expect_context()
}

pub fn empty_scores() -> ParsedScores {
    ParsedScores {
        teams: vec![],
        rounds: vec![],
    }
}

fn scores_url(quiz: &Quiz, override_url: Option<String>) -> Option<String> {
    override_url.or_else(|| {
        if !quiz.scores_sheet_url.is_empty() {
            Some(quiz.scores_sheet_url.clone())
        } else if quiz.builtin {
            Some(DEMO_SCORES.into())
        } else {
            None
        }
    })
}

async fn fetch_scores(url: &str, t: crate::i18n::T) -> Result<ParsedScores, String> {
    let csv = crate::google::sheets::fetch_sheet_csv(url)
        .await
        .map_err(|e| {
            crate::google::sheets::sheet_error_text(&e, &crate::pages::drive_parts::translations(t))
        })?;
    parse_scores_csv(&csv).ok_or_else(|| t.get("ed_err_format"))
}

/// React's usesGoogle excludes published Sheets and anonymous public links.
fn uses_google(url: Option<&str>, private: bool, enabled: bool, known_user: bool) -> bool {
    enabled
        && (private || known_user)
        && url.is_some_and(|u| {
            u.split("docs.google.com/spreadsheets/d/")
                .nth(1)
                .is_some_and(|id| !id.starts_with("e/"))
        })
}
fn ending_minutes(expires: f64, now: f64, valid: bool) -> Option<u32> {
    let minutes = ((expires - now) / 60_000.).ceil();
    (valid && minutes <= 10.).then_some(minutes.max(1.) as u32)
}

pub fn renew_sign_in() {
    leptos::task::spawn_local(async {
        let _ = crate::google::auth::sign_in(false).await;
    });
}

fn venue_arc(quiz: &Quiz) -> f64 {
    quiz.settings
        .fields
        .get("venueArc")
        .and_then(|v| v.as_f64())
        .filter(|a| a.is_finite() && *a > 0.)
        .unwrap_or(180.)
}

fn manual_scores(quiz: &Quiz, word: &str) -> ParsedScores {
    let rounds = quiz_core::quiz::parse::parse_questions_csv(&quiz.questions_csv);
    manual_leaderboard(&quiz.teams, &rounds, &quiz.manual_scores, Some(word))
}

pub fn provide_scores(quiz: QuizContext) {
    let (teams, set_teams) = signal(quiz.teams.clone());
    let (arc, set_arc) = signal(venue_arc(&quiz.quiz));
    let (scores, set_scores) = signal(empty_scores());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (last_loaded, set_last_loaded) = signal(None::<f64>);
    let url = scores_url(&quiz.quiz, query_param("scores"));
    let store = crate::store::use_store();
    let revision = store.revision();
    let store = StoredValue::new_local(store);
    let quiz = StoredValue::new(quiz);
    let t = crate::i18n::use_t();
    let manual = url.is_none();
    let private = quiz.with_value(|q| q.quiz.drive.is_some());
    let account = crate::google::auth::use_google_account();
    let google_url = StoredValue::new(url.clone());
    let (auth_needed, set_auth_needed) = signal(false);
    let now = RwSignal::new(js_sys::Date::now());
    let google = Memo::new(move |_| {
        google_url.with_value(|u| {
            uses_google(
                u.as_deref(),
                private,
                crate::google::auth::google_enabled(),
                account.session.get().is_some(),
            )
        })
    });
    let sign_in_ending = Memo::new(move |_| {
        if !google.get() {
            return None;
        }
        account
            .session
            .get()
            .and_then(|s| ending_minutes(s.expires, now.get(), account.signed_in.get()))
    });
    let clock = StoredValue::new_local(Some(gloo_timers::callback::Interval::new(
        30_000,
        move || now.set(js_sys::Date::now()),
    )));
    on_cleanup(move || {
        clock.update_value(|c| {
            c.take();
        });
    });
    let url = Arc::new(url);
    let cancelled = Arc::new(AtomicBool::new(false));
    let in_flight = Arc::new(AtomicBool::new(false));
    let cancel_on_cleanup = cancelled.clone();
    let pending = Arc::new(AtomicBool::new(false));
    let reload_tick = RwSignal::new(0_u64);
    let load = Callback::new(move |()| {
        if cancelled.load(Ordering::Relaxed) {
            return;
        }
        if in_flight.swap(true, Ordering::Relaxed) {
            pending.store(true, Ordering::Relaxed);
            return;
        }
        set_loading.set(true);
        let (url, cancelled, in_flight) = (url.clone(), cancelled.clone(), in_flight.clone());
        let pending = pending.clone();
        // Capture translations outside the async task so language changes remain reactive.
        let round_word = t.get("round");
        leptos::task::spawn_local(async move {
            // Read attendance even for a Sheet-backed show. The store revision is broadcast
            // across tabs; polling also recovers changes when a notification is missed.
            let mut context = quiz.get_value();
            let fresh = match store.get_value().get_quiz(&context.quiz.id).await {
                Ok(Some(fresh)) => Ok(fresh),
                Ok(None) if context.quiz.builtin => Ok((*context.quiz).clone()),
                Ok(None) => Err(t.get("ed_not_found")),
                Err(e) => Err(e.to_string()),
            };
            if let Ok(fresh) = &fresh {
                if let Ok(urls) = crate::store::media_urls(&store.get_value(), &fresh.id).await {
                    context.media_urls = Arc::new(urls);
                }
                context.quiz = Arc::new(fresh.clone());
                context.teams = Arc::new(resolve_teams(&fresh.teams, |name| {
                    context
                        .media_urls
                        .0
                        .get(&quiz_core::quiz::parse::media_key(name))
                        .cloned()
                        .or_else(|| context.resolve_media(name))
                }));
                if !cancelled.load(Ordering::Relaxed) {
                    set_arc.set(venue_arc(fresh));
                    set_teams.set(context.teams.clone());
                    // Retain the object URLs for as long as their resolved teams are used.
                    quiz.set_value(context);
                }
            }
            let result = if let Some(url) = url.as_ref() {
                fetch_scores(url, t).await
            } else {
                fresh.map(|fresh| manual_scores(&fresh, &round_word))
            };
            in_flight.store(false, Ordering::Relaxed);
            if cancelled.load(Ordering::Relaxed) {
                return;
            }
            match result {
                Ok(next) => {
                    set_auth_needed.set(false);
                    set_scores.set(next);
                    set_error.set(None);
                    set_last_loaded.set(Some(js_sys::Date::now()));
                }
                // Retain the last successful scores on network or parse failure.
                Err(error) => {
                    if google.get_untracked() && crate::google::auth::current_token().is_none() {
                        set_auth_needed.set(true);
                    }
                    set_error.set(Some(error));
                }
            }
            if pending.swap(false, Ordering::Relaxed) {
                // A forced reload during an in-flight request must finish its queued
                // read before presenter controls adopt the result and reset the reveal.
                reload_tick.update(|n| *n += 1);
            } else {
                set_loading.set(false);
            }
        });
    });
    provide_context(ScoresContext {
        scores,
        teams,
        venue_arc: arc,
        loading,
        error,
        last_loaded,
        auth_needed,
        sign_in_ending,
        reload: load,
    });
    Effect::new(move |_| {
        reload_tick.get();
        account.session.get();
        account.signed_in.get();
        revision.get();
        if manual {
            t.lang();
        }
        load.run(());
    });
    let poll = StoredValue::new_local(Some(gloo_timers::callback::Interval::new(
        10_000,
        move || load.run(()),
    )));
    on_cleanup(move || {
        cancel_on_cleanup.store(true, Ordering::Relaxed);
        poll.update_value(|poll| {
            poll.take();
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reload_uses_fresh_scores_attendance_and_round_numbers() {
        let mut quiz = Quiz {
            questions_csv: "Round 2 - Two,1,Q,Written,,,,,A\nRound 1 - One,1,Q,Written,,,,,A\n"
                .into(),
            teams: vec![
                quiz_core::quiz::teams::new_team("a"),
                quiz_core::quiz::teams::new_team("b"),
            ],
            ..Quiz::default()
        };
        quiz.teams[0].name = " A ".into();
        quiz.teams[1].name = "B".into();
        quiz.manual_scores
            .insert("a".into(), [("1".into(), 3.5), ("2".into(), -1.)].into());
        let before = manual_scores(&quiz, "ROUND");
        assert_eq!(before.teams.len(), 2);
        assert_eq!(before.teams[0].scores["Round 1"], 3.5);
        quiz.teams[0].present = Some(false);
        quiz.teams[1].name = "Renamed".into();
        quiz.manual_scores
            .insert("b".into(), [("2".into(), 7.)].into());
        let after = manual_scores(&quiz, "kolo");
        assert_eq!(after.teams.len(), 1);
        assert_eq!(after.teams[0].name, "Renamed");
        assert_eq!(after.teams[0].scores["Kolo 2"], 7.);
        assert_eq!(venue_arc(&quiz), 180.);
        quiz.settings
            .fields
            .insert("venueArc".into(), serde_json::json!(270));
        assert_eq!(venue_arc(&quiz), 270.);
    }

    #[test]
    fn source_selection_preserves_override_demo_sheet_and_manual() {
        let mut quiz = Quiz::default();
        assert_eq!(scores_url(&quiz, None), None);
        quiz.builtin = true;
        assert_eq!(scores_url(&quiz, None).as_deref(), Some(DEMO_SCORES));
        quiz.scores_sheet_url = "custom.csv".into();
        assert_eq!(scores_url(&quiz, None).as_deref(), Some("custom.csv"));
        assert_eq!(
            scores_url(&quiz, Some("dev.csv".into())).as_deref(),
            Some("dev.csv")
        );
    }

    #[test]
    fn google_notices_preserve_public_and_expiry_conditions() {
        let url = Some("https://docs.google.com/spreadsheets/d/private/edit");
        assert!(uses_google(url, true, true, false));
        assert!(uses_google(url, false, true, true));
        assert!(!uses_google(url, false, true, false));
        assert!(!uses_google(
            Some("https://docs.google.com/spreadsheets/d/e/published/pub"),
            true,
            true,
            true
        ));
        assert!(!uses_google(url, true, false, true));
        assert_eq!(ending_minutes(660_001., 0., true), None);
        assert_eq!(ending_minutes(600_000., 0., true), Some(10));
        assert_eq!(ending_minutes(60_001., 0., true), Some(2));
        assert_eq!(ending_minutes(10_000., 0., false), None);
    }
}
