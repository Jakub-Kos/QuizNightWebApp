//! Team check-in and seating on the presenter's perspective map.
use crate::{i18n::use_t, router::Route, scenes::venue_map::VenueMap, store};
use leptos::prelude::*;
use quiz_core::quiz::{
    model::{Quiz, Seat, Team, TeamColor, TeamId, resolve_teams},
    teams::{is_present, new_team, seat_direction, team_image_name},
};
use quiz_store::{Media, MediaStore, QuizStore};
use std::sync::Arc;

const FIELD: &str = "bg-black/40 border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-gray-600 focus:outline-none focus:border-yellow-500/60";
#[derive(Clone)]
enum Edit {
    Present(usize, bool),
    Name(usize, String),
    Players(usize, i64),
    Quote(usize, String),
    Seat(usize, Seat),
    Image(usize, String),
    Add(Team),
    Arc(f64),
}
impl Edit {
    fn index(&self) -> Option<usize> {
        match self {
            Self::Present(i, _)
            | Self::Name(i, _)
            | Self::Players(i, _)
            | Self::Quote(i, _)
            | Self::Seat(i, _)
            | Self::Image(i, _) => Some(*i),
            _ => None,
        }
    }
}
fn apply(q: &mut Quiz, edit: &Edit, index: Option<usize>) {
    match edit {
        Edit::Add(team) => q.teams.push(team.clone()),
        Edit::Arc(arc) => {
            q.settings
                .fields
                .insert("venueArc".into(), serde_json::json!(arc));
        }
        edit => {
            if let Some(team) = index.and_then(|i| q.teams.get_mut(i)) {
                match edit {
                    Edit::Present(_, v) => team.present = Some(*v),
                    Edit::Name(_, v) => team.name = v.clone(),
                    Edit::Players(_, v) => team.players = Some((*v).max(0)),
                    Edit::Quote(_, v) => team.quote = Some(v.clone()),
                    Edit::Seat(_, v) => {
                        team.seat = Some(v.clone());
                        team.present = Some(true);
                    }
                    Edit::Image(_, v) => team.image = Some(v.clone()),
                    _ => {}
                }
            }
        }
    }
}
fn apply_stored(q: &mut Quiz, edit: &Edit, target: Option<TeamId>) {
    let index = if let Some(id) = target {
        q.teams.iter().position(|t| t.id.as_ref() == Some(&id))
    } else {
        edit.index()
    };
    apply(q, edit, index);
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
fn Icon(#[prop(default = "pin")] name: &'static str) -> impl IntoView {
    let d = match name {
        "check" => "M20 6 9 17l-5-5",
        "x" => "m6 6 12 12M6 18 18 6",
        "plus" => "M12 5v14M5 12h14",
        "players" => {
            "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2 M16 3a4 4 0 0 1 0 8 M22 21v-2a4 4 0 0 0-3-3.87 M13 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0"
        }
        "camera" => "M14 4h-4L8 7H3v13h18V7h-5z M15 13a3 3 0 1 1-6 0 3 3 0 0 1 6 0",
        _ => "M20 10c0 6-8 12-8 12S4 16 4 10a8 8 0 1 1 16 0 M15 10a3 3 0 1 1-6 0 3 3 0 0 1 6 0",
    };
    view! {<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d=d/></svg>}
}

#[component]
pub fn Page(id: String) -> impl IntoView {
    let t = use_t();
    let store = store::use_store();
    let quiz = LocalResource::new(move || {
        let store = store.clone();
        let id = id.clone();
        async move { store.get_quiz(&id).await.map_err(|e| e.to_string()) }
    });
    view! {<main class="min-h-screen bg-[#050505] text-white font-sans">{move || quiz.get().map(|q| match q {
        Ok(Some(quiz))=>view!{<Attendance quiz=quiz/>}.into_any(),
        Ok(None)=>view!{<div class="p-12"><p>{t.get("ed_not_found")}</p><a href="#/" class="text-yellow-400">{t.get("settings_exit")}</a></div>}.into_any(),
        Err(e)=>view!{<p class="p-12 text-red-300" role="alert">{t.get("lib_action_failed")}" "{e}</p>}.into_any(),
    })}</main>}
}

#[component]
fn Attendance(quiz: Quiz) -> impl IntoView {
    // TODO(auth): Google sign-in and Drive team synchronization/write-back.
    let t = use_t();
    let id = StoredValue::new(quiz.id.clone());
    let title = quiz.title.clone();
    let quiz = RwSignal::new(quiz);
    let store = StoredValue::new_local(store::use_store());
    let media = RwSignal::new(Arc::new(store::MediaUrls::default()));
    let error = RwSignal::new(None::<String>);
    let pending = RwSignal::new(0_usize);
    let placing = RwSignal::new(None::<usize>);
    leptos::task::spawn_local(async move {
        match store::media_urls(&store.get_value(), &id.get_value()).await {
            Ok(urls) => media.set(Arc::new(urls)),
            Err(e) => error.set(Some(e)),
        }
    });
    let change = Callback::new(move |edit: Edit| {
        let target = edit
            .index()
            .and_then(|i| quiz.with_untracked(|q| q.teams.get(i).and_then(|t| t.id.clone())));
        quiz.update(|q| apply(q, &edit, edit.index()));
        pending.update(|n| *n += 1);
        leptos::task::spawn_local(async move {
            let result = store
                .get_value()
                .update_quiz(&id.get_value(), move |q| apply_stored(q, &edit, target))
                .await;
            error.set(match result {
                Ok(Some(_)) => None,
                Ok(None) => Some(t.get("ed_not_found")),
                Err(e) => Some(e.to_string()),
            });
            pending.update(|n| *n -= 1);
        });
    });
    let photo_for = RwSignal::new(None::<usize>);
    let photo = NodeRef::<leptos::html::Input>::new();
    let on_photo = move |e| {
        let input = event_target::<web_sys::HtmlInputElement>(&e);
        let file = input.files().and_then(|f| f.get(0));
        input.set_value("");
        let Some((file, i)) = file.zip(photo_for.get_untracked()) else {
            return;
        };
        let Some(team) = quiz.with_untracked(|q| q.teams.get(i).cloned()) else {
            return;
        };
        pending.update(|n| *n += 1);
        leptos::task::spawn_local(async move {
            let result: Result<(), String> = async {
                let bytes = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
                    .await
                    .map_err(|e| format!("{e:?}"))?;
                let name = team_image_name(&team, &file.name());
                store
                    .get_value()
                    .put_media(
                        &id.get_value(),
                        &[Media {
                            name: name.clone(),
                            bytes: js_sys::Uint8Array::new(&bytes).to_vec(),
                            content_type: file.type_(),
                            fields: Default::default(),
                        }],
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                let urls = store::media_urls(&store.get_value(), &id.get_value()).await?;
                media.set(Arc::new(urls));
                change.run(Edit::Image(i, name));
                Ok(())
            }
            .await;
            if let Err(e) = result {
                error.set(Some(e));
            }
            pending.update(|n| *n -= 1);
        });
    };
    let key = window_event_listener(leptos::ev::keydown, move |e| {
        if e.key() == "Escape" {
            placing.set(None);
        }
    });
    on_cleanup(move || key.remove());
    let map_teams = Signal::derive(move || {
        quiz.with(|q| {
            q.teams
                .iter()
                .map(|t| {
                    let mut t = t.clone();
                    if !is_present(&t) {
                        t.seat = None;
                    }
                    t
                })
                .collect::<Vec<_>>()
        })
    });
    let arc = Memo::new(move |_| {
        quiz.with(|q| {
            q.settings
                .fields
                .get("venueArc")
                .and_then(|v| v.as_f64())
                .filter(|a| *a > 0.)
                .unwrap_or(180.)
        })
    });
    let on_seat = Callback::new(move |(i, seat)| {
        change.run(Edit::Seat(i, seat));
        placing.set(None);
    });
    view! {<div class="h-screen flex flex-col bg-[#050505] text-white font-sans">
        <header class="shrink-0 flex flex-wrap items-center gap-x-6 gap-y-2 px-6 py-4 border-b border-white/10">
            <button on:click=move |_| back(&id.get_value()) class="text-gray-400 hover:text-white">{if is_popup() {"× "} else {"← "}}{move ||t.get(if is_popup() {"chk_close"} else {"nav_back"})}</button>
            <h1 class="font-['League_Spartan'] text-2xl font-black">{move ||t.get("att_title")}": "{title}</h1>
            <span class="text-lg font-bold text-yellow-400">{move ||quiz.with(|q|{let present:Vec<_>=q.teams.iter().filter(|t|t.present==Some(true)).collect();let players:i64=present.iter().map(|t|t.players.unwrap_or(0)).sum();t.fmt("att_summary",&[("present",&present.len().to_string()),("total",&q.teams.len().to_string()),("players",&players.to_string())])})}</span>
            <span class="ml-auto text-xs text-gray-500" role="status">{move ||t.get(if pending.get()>0 {"ed_saving"} else if error.get().is_some() {"lib_action_failed"} else {"ed_saved"})}</span>
        </header>
        <Show when=move ||error.get().is_some()><p role="alert" class="px-6 text-red-300">{move ||error.get()}</p></Show>
        <div class="flex-1 min-h-0 overflow-y-auto px-6 py-4 space-y-3"><p class="text-sm text-gray-400">{move ||t.get("att_help")}</p>
            <input node_ref=photo type="file" accept="image/*" hidden on:change=on_photo/>
            <ul class="grid xl:grid-cols-2 gap-2"><For each=move ||0..quiz.with(|q|q.teams.len()) key=|i|*i children=move |i| {
                let team=Memo::new(move |_|quiz.with(|q|q.teams[i].clone()));
                let choose_photo=Callback::new(move |()|{photo_for.set(Some(i));if let Some(input)=photo.get(){input.click();}});
                view! {<AttendanceTeam index=i team=team quiz=quiz media=media placing=placing change=change photo=choose_photo/>}
            }/>
                <li><button on:click=move |_|{let mut team=new_team(store::new_id());team.present=Some(true);team.color=Some(TeamColor::Palette(quiz.with_untracked(|q|q.teams.len())%8));change.run(Edit::Add(team));} class="w-full h-full min-h-14 flex items-center justify-center gap-2 px-4 py-3 rounded-xl border border-dashed border-white/15 hover:bg-white/5 text-sm"><Icon name="plus"/>{move ||t.get("tm_add")}</button></li>
            </ul>
        </div>
        <div class="h-[50vh] shrink-0 flex flex-col border-t border-white/10 bg-white/[0.03]">
            <div class="shrink-0 flex flex-wrap items-center gap-3 px-6 py-2">
                <p class=move ||if placing.get().is_some(){"text-sm mr-auto text-yellow-300 font-bold"}else{"text-sm mr-auto text-gray-400"}>{move ||placing.get().map(|i|t.fmt("att_place_hint",&[("team",&quiz.with(|q|q.teams[i].name.clone()))])).unwrap_or_else(||t.get("att_map_help"))}</p>
                <span class="text-sm text-gray-400">{move ||t.get("att_arc")}</span>
                {[180.,270.].into_iter().map(|a|view!{<button class=move ||if arc.get()==a {"px-3 py-1.5 rounded-lg text-sm bg-yellow-500 text-black font-bold"}else{"px-3 py-1.5 rounded-lg text-sm bg-white/10 hover:bg-white/20"} on:click=move |_|change.run(Edit::Arc(a))>{format!("{a}°")}</button>}).collect_view()}
            </div>
            <div class="flex-1 min-h-0 px-6 pb-3">{move ||view!{<VenueMap teams=map_teams arc=arc.get() on_seat=on_seat placing=Signal::from(placing) you_label=t.get("att_you") fill=true class="h-full"/>}}</div>
        </div>
    </div>}
}

// Attendance uses the configured team icon, matching the check-in form in React.
#[component]
fn AttendanceAvatar(
    team: quiz_core::quiz::model::ResolvedTeam,
    class: &'static str,
) -> impl IntoView {
    // Literal palette classes so Tailwind includes every gradient.
    let color = match team.color % 8 {
        0 => "from-purple-600 to-indigo-900",
        1 => "from-red-600 to-pink-900",
        2 => "from-blue-600 to-cyan-800",
        3 => "from-emerald-600 to-teal-900",
        4 => "from-orange-500 to-red-600",
        5 => "from-yellow-500 to-orange-700",
        6 => "from-sky-500 to-blue-700",
        _ => "from-stone-700 to-stone-900",
    };
    // Lucide SVG paths, as shipped by lucide-react (ISC license).
    let icon: &[&str] = match team.team.icon.as_deref().unwrap_or("Brain") {
        "Rocket" => &[
            "M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z",
            "m12 15-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z",
            "M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0",
            "M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5",
        ],
        "Zap" => &[
            "M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z",
        ],
        "Crown" => &[
            "M11.562 3.266a.5.5 0 0 1 .876 0L15.39 8.87a1 1 0 0 0 1.516.294L21.183 5.5a.5.5 0 0 1 .798.519l-2.834 10.246a1 1 0 0 1-.956.734H5.81a1 1 0 0 1-.957-.734L2.02 6.02a.5.5 0 0 1 .798-.519l4.276 3.664a1 1 0 0 0 1.516-.294z",
            "M5 21h14",
        ],
        "Flame" => &[
            "M12 3q1 4 4 6.5t3 5.5a1 1 0 0 1-14 0 5 5 0 0 1 1-3 1 1 0 0 0 5 0c0-2-1.5-3-1.5-5q0-2 2.5-4",
        ],
        "Shield" => &[
            "M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z",
        ],
        "Beer" => &[
            "M17 11h1a3 3 0 0 1 0 6h-1",
            "M9 12v6",
            "M13 12v6",
            "M14 7.5c-1 0-1.44.5-3 .5s-2-.5-3-.5-1.72.5-2.5.5a2.5 2.5 0 0 1 0-5c.78 0 1.57.5 2.5.5S9.44 2 11 2s2 1.5 3 1.5 1.72-.5 2.5-.5a2.5 2.5 0 0 1 0 5c-.78 0-1.5-.5-2.5-.5Z",
            "M5 8v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V8",
        ],
        "Glasses" => &[
            "M14 15a2 2 0 0 0-2-2 2 2 0 0 0-2 2",
            "M2.5 13 5 7c.7-1.3 1.4-2 3-2",
            "M21.5 13 19 7c-.7-1.3-1.5-2-3-2",
            "M10 15a4 4 0 1 0-8 0a4 4 0 1 0 8 0 M22 15a4 4 0 1 0-8 0a4 4 0 1 0 8 0",
        ],
        "Gavel" => &[
            "m14 13-8.381 8.38a1 1 0 0 1-3.001-3l8.384-8.381",
            "m16 16 6-6",
            "m21.5 10.5-8-8",
            "m8 8 6-6",
            "m8.5 7.5 8 8",
        ],
        "Microscope" => &[
            "M6 18h8",
            "M3 22h18",
            "M14 22a7 7 0 1 0 0-14h-1",
            "M9 14h2",
            "M9 12a2 2 0 0 1-2-2V6h6v4a2 2 0 0 1-2 2Z",
            "M12 6V3a1 1 0 0 0-1-1H9a1 1 0 0 0-1 1v3",
        ],
        _ => &[
            "M12 18V5",
            "M15 13a4.17 4.17 0 0 1-3-4 4.17 4.17 0 0 1-3 4",
            "M17.598 6.5A3 3 0 1 0 12 5a3 3 0 1 0-5.598 1.5",
            "M17.997 5.125a4 4 0 0 1 2.526 5.77",
            "M18 18a4 4 0 0 0 2-7.464",
            "M19.967 17.483A4 4 0 1 1 12 18a4 4 0 1 1-7.967-.517",
            "M6 18a4 4 0 0 1-2-7.464",
            "M6.003 5.125a4 4 0 0 0-2.526 5.77",
        ],
    };
    view! { <div class=format!("overflow-hidden flex items-center justify-center bg-gradient-to-br {color} {class}")>
        {match team.image {
            Some(url) => view! { <img src=url alt=team.team.name class="w-full h-full object-cover"/> }.into_any(),
            None => view! { <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-white/80" aria-hidden="true">{icon.iter().map(|d| view! { <path d=*d/> }).collect_view()}</svg> }.into_any(),
        }}
    </div> }
}

#[component]
fn AttendanceTeam(
    index: usize,
    team: Memo<Team>,
    quiz: RwSignal<Quiz>,
    media: RwSignal<Arc<store::MediaUrls>>,
    placing: RwSignal<Option<usize>>,
    change: Callback<Edit>,
    photo: Callback<()>,
) -> impl IntoView {
    let t = use_t();
    view! {<li class=move ||format!("flex flex-wrap items-center gap-3 p-3 rounded-xl border transition-colors {} {}",match team.get().present {Some(true)=>"bg-green-500/[0.07] border-green-500/30",Some(false)=>"bg-black/30 border-white/5 opacity-50",None=>"bg-black/30 border-white/10"},if placing.get()==Some(index){"ring-2 ring-yellow-400"}else{""})>
        <div class="flex flex-col gap-1">
            <button title=move ||t.get("att_present") aria-label=move ||t.get("att_present") aria-pressed=move ||team.get().present==Some(true) class=move ||if team.get().present==Some(true){"p-2 rounded-lg bg-green-600 text-white"}else{"p-2 rounded-lg bg-white/5 text-gray-500 hover:text-green-300"} on:click=move |_|change.run(Edit::Present(index,true))><Icon name="check"/></button>
            <button title=move ||t.get("att_absent") aria-label=move ||t.get("att_absent") aria-pressed=move ||team.get().present==Some(false) class=move ||if team.get().present==Some(false){"p-2 rounded-lg bg-red-700 text-white"}else{"p-2 rounded-lg bg-white/5 text-gray-500 hover:text-red-300"} on:click=move |_|change.run(Edit::Present(index,false))><Icon name="x"/></button>
        </div>
        <button class="group relative w-16 h-16 shrink-0 rounded-xl overflow-hidden" title=move ||t.get("tm_photo") aria-label=move ||t.get("tm_photo") on:click=move |_|photo.run(())>
            {move ||{let urls=media.get();let mut source=quiz.get().teams;for t in &mut source{if t.name.trim().is_empty(){t.name="?".into();}}
                let builtin=quiz.with(|q|q.builtin);
                let resolved=resolve_teams(&source,|name|if name.starts_with("https://")||name.starts_with("http://"){Some(name.into())}else{urls.0.get(&quiz_core::quiz::parse::media_key(name)).cloned().or_else(||builtin.then(||format!("source/{name}")))});
                resolved.into_iter().nth(index).map(|team|view!{<AttendanceAvatar team=team class="w-16 h-16 rounded-xl"/>})}}
            <span class="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 group-focus:opacity-100 flex items-center justify-center transition-opacity"><Icon name="camera"/></span>
        </button>
        <div class="flex-1 min-w-[180px] space-y-2"><div class="flex gap-2">
            <input aria-label=move ||t.get("tm_name") placeholder=move ||t.get("tm_name") prop:value=move ||team.get().name class=format!("{FIELD} flex-1 min-w-0 font-bold") on:input=move |e|change.run(Edit::Name(index,event_target_value(&e)))/>
            <label class="flex items-center gap-1 text-xs text-gray-400" title=move ||t.get("tm_players")><Icon name="players"/><input aria-label=move ||t.get("tm_players") type="number" min="0" prop:value=move ||team.get().players.filter(|n|*n!=0).map(|n|n.to_string()).unwrap_or_default() class=format!("{FIELD} w-16") on:input=move |e|change.run(Edit::Players(index,event_target_value(&e).parse().unwrap_or(0)))/></label>
        </div><input aria-label=move ||t.get("tm_quote") placeholder=move ||t.get("tm_quote") prop:value=move ||team.get().quote.unwrap_or_default() class=format!("{FIELD} w-full italic") on:input=move |e|change.run(Edit::Quote(index,event_target_value(&e)))/></div>
        <button class=move ||if placing.get()==Some(index){"flex items-center gap-2 px-3 py-2 rounded-lg text-sm whitespace-nowrap bg-yellow-500 text-black font-bold"}else if team.get().seat.is_some(){"flex items-center gap-2 px-3 py-2 rounded-lg text-sm whitespace-nowrap bg-white/10 hover:bg-white/20"}else{"flex items-center gap-2 px-3 py-2 rounded-lg text-sm whitespace-nowrap bg-white/5 text-gray-400 hover:bg-white/10"} on:click=move |_|placing.update(|p|*p=if *p==Some(index){None}else{Some(index)})><Icon/>{move ||team.get().seat.as_ref().map(|s|{let d=seat_direction(s);t.fmt(d.message_key(),&[("deg",&d.deg.to_string())])}).unwrap_or_else(||t.get("att_place"))}</button>
    </li>}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atomic_edits_preserve_scores_settings_and_deleted_teams() {
        let mut q = Quiz {
            teams: vec![new_team("a")],
            ..Quiz::default()
        };
        q.manual_scores
            .insert("a".into(), [("1".into(), 5.)].into());
        q.settings
            .fields
            .insert("unrelated".into(), serde_json::json!(true));
        apply_stored(
            &mut q,
            &Edit::Quote(0, "Motto".into()),
            Some(TeamId::Text("a".into())),
        );
        apply_stored(
            &mut q,
            &Edit::Name(0, "Wrong team".into()),
            Some(TeamId::Text("deleted".into())),
        );
        apply(&mut q, &Edit::Arc(270.), None);
        assert!(q.teams[0].name.is_empty());
        assert_eq!(q.teams[0].quote.as_deref(), Some("Motto"));
        assert_eq!(q.manual_scores["a"]["1"], 5.);
        assert_eq!(q.settings.fields["unrelated"], true);
        assert_eq!(q.settings.fields["venueArc"], 270.);
    }

    #[test]
    fn seat_checks_in_and_targets_stable_id() {
        let mut q = Quiz {
            title: "Unchanged".into(),
            teams: vec![new_team("a"), new_team("b")],
            ..Quiz::default()
        };
        let edit = Edit::Seat(
            1,
            Seat {
                angle: 45.,
                dist: 0.5,
            },
        );
        q.teams[1].present = Some(false);
        apply(&mut q, &edit, edit.index());
        assert_eq!(q.teams[1].present, Some(true));
        assert!(q.teams[0].seat.is_none());
        assert_eq!(q.title, "Unchanged");
        apply(&mut q, &Edit::Players(1, -4), Some(1));
        assert_eq!(q.teams[1].players, Some(0));
        q.teams.swap(0, 1);
        apply_stored(
            &mut q,
            &Edit::Name(1, "B".into()),
            Some(TeamId::Text("b".into())),
        );
        assert_eq!(q.teams[0].name, "B");
    }
}
