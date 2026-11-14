//! Manual score entry; only the edited score cell is patched in storage.
use crate::{i18n::use_t, router::Route, store};
use leptos::prelude::*;
use quiz_core::quiz::{
    model::{Quiz, TeamId},
    parse::parse_questions_csv,
    scores::{manual_leaderboard, scored_rounds},
};
use quiz_store::QuizStore;
use std::collections::BTreeMap;
use wasm_bindgen::JsCast;

type Drafts = BTreeMap<(String, String), String>;

fn team_key(id: &Option<TeamId>) -> String {
    match id {
        Some(TeamId::Text(s)) => s.clone(),
        Some(TeamId::Number(n)) => n.to_string(),
        None => "null".into(),
    }
}
/// Accept intermediate decimal input, including a minus sign and trailing separator.
fn score_input(value: &str) -> Option<String> {
    let value = value.replace(',', ".");
    let digits = value.strip_prefix('-').unwrap_or(&value);
    (digits.chars().all(|c| c.is_ascii_digit() || c == '.') && digits.matches('.').count() <= 1)
        .then_some(value)
}
fn score_value(value: &str) -> Option<f64> {
    if value.is_empty() {
        None
    } else {
        Some(
            value
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .unwrap_or(0.),
        )
    }
}
fn patch_score(q: &mut Quiz, team: &str, round: &str, value: &str) {
    let points = q.manual_scores.entry(team.into()).or_default();
    match score_value(value) {
        Some(n) => {
            points.insert(round.into(), n);
        }
        None => {
            points.remove(round);
        }
    }
}
fn adopt_scores(q: &mut Quiz, scores: &quiz_core::quiz::parse::ParsedScores) {
    for team in &q.teams {
        let id = team_key(&team.id);
        let rows = scores
            .teams
            .iter()
            .filter(|row| {
                row.id.as_deref() == Some(&id)
                    || (row.id.is_none() && row.name.trim() == team.name.trim())
            })
            .collect::<Vec<_>>();
        if rows.len() != 1 {
            continue;
        }
        let points = rows[0]
            .scores
            .iter()
            .filter_map(|(heading, score)| {
                let round = heading
                    .split(|c: char| !c.is_ascii_digit() && c != '.')
                    .find(|s| !s.is_empty())?;
                Some((round.to_owned(), *score))
            })
            .collect();
        q.manual_scores.insert(id, points);
    }
}
fn totals(q: &Quiz) -> BTreeMap<String, f64> {
    let rounds = parse_questions_csv(&q.questions_csv);
    let board = manual_leaderboard(&q.teams, &rounds, &q.manual_scores, None);
    q.teams
        .iter()
        .filter(|t| !t.name.trim().is_empty() && quiz_core::quiz::teams::is_present(t))
        .zip(board.teams)
        .map(|(team, row)| (team_key(&team.id), row.scores.values().sum()))
        .collect()
}
fn round_heading(word: &str, number: &str) -> String {
    let mut chars = word.chars();
    let first = chars
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();
    format!("{first}{} {number}", chars.as_str().to_lowercase())
}
fn rank(points: &BTreeMap<String, f64>, team: &str) -> usize {
    let total = points.get(team).copied().unwrap_or(0.);
    1 + points.values().filter(|v| **v > total).count()
}

fn is_popup() -> bool {
    window()
        .opener()
        .ok()
        .is_some_and(|o| !o.is_null() && !o.is_undefined())
        && !window().name().unwrap_or_default().is_empty()
}
fn back(id: &str) {
    if is_popup() {
        let _ = window().close();
    } else if let Ok(history) = window().history() {
        if history.length().unwrap_or(0) > 1 {
            let _ = history.back();
        } else {
            crate::router::navigate(Route::Edit(id.into()));
        }
    } else {
        crate::router::navigate(Route::Edit(id.into()));
    }
}

#[component]
pub fn Page(id: String) -> impl IntoView {
    let t = use_t();
    let store = store::use_store();
    let account = crate::google::auth::use_google_account();
    let quiz = LocalResource::new(move || {
        let signed = account.signed_in.get();
        let store = store.clone();
        let id = id.clone();
        async move {
            let mut q = store.get_quiz(&id).await.map_err(|e| e.to_string())?;
            if signed && q.as_ref().is_some_and(|q| q.drive.is_some()) {
                if let Ok(Some(fresh)) = crate::google::sync::sync_drive_config(&id).await {
                    q = Some(fresh);
                }
                if let Some(q) = q.as_mut() {
                    if let Ok(csv) =
                        crate::google::sheets::fetch_sheet_csv(&q.scores_sheet_url).await
                    {
                        if let Some(scores) = quiz_core::quiz::parse::parse_scores_csv(&csv) {
                            adopt_scores(q, &scores);
                        }
                    }
                }
            }
            Ok::<_, String>(q)
        }
    });
    view! { <main class="min-h-screen bg-[#050505] text-white font-sans">{move || quiz.get().map(|result| match result {
        Ok(Some(q)) => view! { <Scores quiz=q/> }.into_any(),
        Ok(None) => view! { <div class="p-12"><p>{t.get("ed_not_found")}</p><a href="#/" class="text-yellow-400">{t.get("settings_exit")}</a></div> }.into_any(),
        Err(e) => view! { <p role="alert" class="p-12 text-red-300">{t.get("lib_action_failed")}" "{e}</p> }.into_any(),
    })}</main> }
}

#[component]
fn Scores(quiz: Quiz) -> impl IntoView {
    let t = use_t();
    let id = StoredValue::new(quiz.id.clone());
    let rounds = parse_questions_csv(&quiz.questions_csv);
    let rounds: Vec<_> = scored_rounds(&rounds).into_iter().cloned().collect();
    let teams: Vec<_> = quiz
        .teams
        .iter()
        .filter(|t| !t.name.trim().is_empty() && quiz_core::quiz::teams::is_present(t))
        .cloned()
        .collect();
    let drafts = RwSignal::new(
        quiz.manual_scores
            .iter()
            .flat_map(|(id, points)| {
                points
                    .iter()
                    .map(move |(r, n)| ((id.clone(), r.clone()), n.to_string()))
            })
            .collect::<Drafts>(),
    );
    let linked = !quiz.scores_sheet_url.is_empty();
    let title = quiz.title.clone();
    let quiz = RwSignal::new(quiz);
    let points = Memo::new(move |_| quiz.with(totals));
    let pending = RwSignal::new(0_usize);
    let error = RwSignal::new(None::<String>);
    let store = StoredValue::new_local(store::use_store());
    let drive = quiz.get_untracked().drive.is_some() && linked;
    let account = crate::google::auth::use_google_account();
    // Keep writes ordered and coalesce cells changed while an earlier request is in flight.
    let queue = std::rc::Rc::new(std::cell::RefCell::new(Drafts::new()));
    let running = std::rc::Rc::new(std::cell::Cell::new(false));
    let writer = StoredValue::new_local((queue, running));
    let write_remote = Callback::new(move |(team, round, value): (String, String, String)| {
        if !drive {
            return;
        }
        let (queue, running) = writer.get_value();
        queue.borrow_mut().insert((team, round), value);
        if running.replace(true) {
            return;
        }
        let queue = queue.clone();
        let running = running.clone();
        let data = store.get_value();
        let id = id.get_value();
        pending.update(|n| *n += 1);
        leptos::task::spawn_local(async move {
            loop {
                let next = queue.borrow_mut().pop_first();
                let Some(((team, round), value)) = next else {
                    break;
                };
                let result = async {
                    let q = data
                        .get_quiz(&id)
                        .await?
                        .ok_or_else(|| crate::google::Error::Other("Quiz not found".into()))?;
                    crate::google::sync::write_score_cell(&q, &team, &round, &value).await
                }
                .await;
                if let Err(e) = result {
                    error.try_set(Some(
                        super::drive_parts::error_text(&e, t).unwrap_or_default(),
                    ));
                }
            }
            running.set(false);
            pending.try_update(|n| *n -= 1);
        });
    });
    let change = Callback::new(move |(team, round, value): (String, String, String)| {
        drafts.update(|d| {
            d.insert((team.clone(), round.clone()), value.clone());
        });
        quiz.update(|q| patch_score(q, &team, &round, &value));
        pending.update(|n| *n += 1);
        leptos::task::spawn_local(async move {
            let remote = (team.clone(), round.clone(), value.clone());
            let result = store
                .get_value()
                .update_quiz(&id.get_value(), move |q| {
                    patch_score(q, &team, &round, &value)
                })
                .await;
            error.set(match result {
                Ok(Some(_)) => {
                    write_remote.run(remote);
                    None
                }
                Ok(None) => Some(t.get("ed_not_found")),
                Err(e) => Some(e.to_string()),
            });
            pending.update(|n| *n -= 1);
        });
    });
    let empty = teams.is_empty() || rounds.is_empty();
    let no_teams = teams.is_empty();
    view! { <div class="max-w-6xl mx-auto px-6 py-8 space-y-6">
        <header class="flex flex-wrap items-center gap-4">
            <button on:click=move |_| back(&id.get_value()) class="text-gray-400 hover:text-white">{if is_popup() {"× "} else {"← "}}{move || t.get(if is_popup() {"chk_close"} else {"nav_back"})}</button>
            <h1 class="font-['League_Spartan'] text-3xl font-black">{move || t.get("sc_title")}": "{title}</h1>
            <span class="ml-auto text-xs text-gray-500" role="status">{move || t.get(if pending.get()>0 {"ed_saving"} else if error.get().is_some() {"lib_action_failed"} else {"ed_saved"})}</span>
        </header>
        <Show when=move || error.get().is_some()><p role="alert" class="text-red-300">{move || error.get()}</p></Show>
        <Show when=move || drive && !account.signed_in.get()><super::drive_parts::GoogleAccountBar on_error=Callback::new(move |e|error.set(e))/></Show>
        <Show when=move || linked && !drive><p class="flex items-start gap-2 p-4 rounded-xl bg-yellow-500/10 border border-yellow-500/30 text-yellow-200 text-sm"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="shrink-0 mt-0.5" aria-hidden="true"><path d="m12 3 10 18H2z M12 9v4 M12 17h.01"/></svg>{move || t.get("sc_sheet_linked")}</p></Show>
        <p class="text-sm text-gray-400">{move || t.get("sc_help")}</p>
        {if empty {view! { <p class="text-gray-400">{move || t.get(if no_teams {"sc_no_teams"} else {"ed_no_questions"})}" "<a class="text-yellow-400 underline" href=Route::Edit(id.get_value()).hash()>{move || t.get("lib_edit")}</a></p> }.into_any()} else {
            view! { <div class="overflow-x-auto rounded-2xl border border-white/10"><table class="w-full text-sm">
                <thead class="bg-white/5 text-gray-400"><tr><th class="text-left px-4 py-3 font-bold">{move || t.get("sc_team")}</th>
                    {rounds.iter().map(|r| {let number=r.number.clone(); view! {<th class="px-2 py-3 font-bold whitespace-nowrap" title=r.title.clone()>{move || round_heading(&t.get("round"), &number)}</th>}}).collect_view()}
                    <th class="px-4 py-3 font-bold text-right">{move || t.get("sc_total")}</th><th class="px-4 py-3 text-right">"#"</th></tr></thead>
                <tbody class="divide-y divide-white/5">{teams.into_iter().enumerate().map(|(row,team)| {
                    let key=StoredValue::new(team_key(&team.id));
                    view! { <tr class="hover:bg-white/[0.03]"><th scope="row" class="px-4 py-2 font-bold whitespace-nowrap text-left">{team.name.clone()}</th>
                        {rounds.iter().enumerate().map(|(col,r)| {let round=StoredValue::new(r.number.clone()); let label=format!("{} · {} {}",team.name,t.get("round"),r.number);
                            view! {<td class="px-1 py-1"><input type="text" inputmode="decimal" aria-label=label data-cell=format!("{row}:{col}")
                                prop:value=move || drafts.with(|d| d.get(&(key.get_value(),round.get_value())).cloned().unwrap_or_default())
                                on:input=move |e| { let input=event_target::<web_sys::HtmlInputElement>(&e); if let Some(value)=score_input(&input.value()) {change.run((key.get_value(),round.get_value(),value));} else {input.set_value(&drafts.with_untracked(|d| d.get(&(key.get_value(),round.get_value())).cloned().unwrap_or_default()));} }
                                on:focus=move |e| { event_target::<web_sys::HtmlInputElement>(&e).select(); }
                                on:keydown=move |e: web_sys::KeyboardEvent| {if e.key()=="Enter" {e.prevent_default(); let next=row as isize+if e.shift_key() {-1} else {1}; if let Ok(Some(node))=document().query_selector(&format!("[data-cell='{next}:{col}']")) && let Ok(input)=node.dyn_into::<web_sys::HtmlInputElement>() {let _=input.focus();input.select();}}}
                                class="w-16 mx-auto block bg-black/40 border border-white/10 rounded-lg px-2 py-2 text-center font-mono focus:outline-none focus:border-yellow-500/60"/></td>}
                        }).collect_view()}
                        <td class="px-4 py-2 text-right font-mono font-bold text-yellow-400">{move || points.with(|p| p.get(&key.get_value()).copied().unwrap_or(0.))}</td>
                        <td class="px-4 py-2 text-right font-mono text-gray-400">{move || points.with(|p| rank(p,&key.get_value()))}</td>
                    </tr> }
                }).collect_view()}</tbody>
            </table></div> }.into_any()
        }}
    </div> }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drive_scores_follow_ids_after_rename_and_use_round_numbers() {
        let mut q = Quiz::default();
        let mut team = quiz_core::quiz::teams::new_team("t1");
        team.name = "New name".into();
        q.teams.push(team);
        let scores = quiz_core::quiz::parse::parse_scores_csv(
            "Team name,Round 1,Kolo 2,quiznight-id\nOld name,3.5,-2,t1",
        )
        .unwrap();
        adopt_scores(&mut q, &scores);
        assert_eq!(q.manual_scores["t1"]["1"], 3.5);
        assert_eq!(q.manual_scores["t1"]["2"], -2.);
    }
    #[test]
    fn decimal_drafts_and_invalid_input() {
        assert_eq!(round_heading("ROUND", "3"), "Round 3");
        assert_eq!(round_heading("kolo", "2"), "Kolo 2");
        for s in ["", "-", ".", "-.", "3.", "-2.5"] {
            assert_eq!(score_input(s).as_deref(), Some(s));
        }
        assert_eq!(score_input("3,5").as_deref(), Some("3.5"));
        for s in ["1..2", "2-3", "NaN", "1e2", " 2"] {
            assert_eq!(score_input(s), None);
        }
    }
    #[test]
    fn patches_preserve_other_cells_and_match_core() {
        let mut q = Quiz::default();
        q.teams.push(quiz_core::quiz::teams::new_team("a"));
        q.teams[0].name = "A".into();
        patch_score(&mut q, "a", "1", "3,5".replace(',', ".").as_str());
        patch_score(&mut q, "a", "2", "-2");
        patch_score(&mut q, "a", "3", "5");
        patch_score(&mut q, "a", "3", "");
        q.manual_scores
            .insert("other".into(), [("1".into(), 8.)].into());
        patch_score(&mut q, "a", "1", "3.5");
        assert_eq!(q.manual_scores["other"]["1"], 8.);
        assert!(!q.manual_scores["a"].contains_key("3"));
        q.questions_csv =
            "Round 1 - One,1,Q,Written,,,,,A\nRound 2 - Two,1,Q,Written,,,,,A\n".into();
        let rounds = parse_questions_csv(&q.questions_csv);
        let board = manual_leaderboard(&q.teams, &rounds[..1], &q.manual_scores, Some("Round"));
        assert_eq!(totals(&q).get("a"), Some(&1.5));
        assert_eq!(board.teams[0].scores.values().sum::<f64>(), 3.5);
    }
    #[test]
    fn tied_totals_share_competition_rank() {
        let points = BTreeMap::from([("a".into(), 10.), ("b".into(), 10.), ("c".into(), 5.)]);
        assert_eq!(rank(&points, "a"), 1);
        assert_eq!(rank(&points, "b"), 1);
        assert_eq!(rank(&points, "c"), 3);
    }
}
