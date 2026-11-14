#[allow(dead_code)]
#[path = "scenes/icons.rs"]
mod frame_icons;
use leptos::prelude::*;
use quiz_core::show::{Action, Intro, PlayMode, Role, ShowState, SyncMessage};
use wasm_bindgen::{JsCast, closure::Closure};

#[derive(Clone, Copy)]
pub struct ShowContext {
    pub role: Role,
    pub state: RwSignal<ShowState>,
    pub dispatch: Callback<Action>,
}
pub fn use_show() -> ShowContext {
    expect_context()
}
#[derive(Clone, Copy)]
pub struct ShowUi {
    pub settings_open: RwSignal<bool>,
    pub calibrating: RwSignal<bool>,
}
pub fn use_show_ui() -> ShowUi {
    expect_context()
}
fn post(channel: &web_sys::BroadcastChannel, messages: Vec<SyncMessage>) {
    for message in messages {
        if let Ok(json) = serde_json::to_string(&message) {
            if let Ok(value) = js_sys::JSON::parse(&json) {
                let _ = channel.post_message(&value);
            }
        }
    }
}
#[component]
pub fn ShowApp(
    #[prop(optional)] id: Option<String>,
    #[prop(default = false)] presenter: bool,
) -> impl IntoView {
    let store = crate::store::use_store();
    let loaded = LocalResource::new(move || {
        let id = id.clone();
        let store = store.clone();
        async move {
            match id {
                Some(id) => crate::quiz_context::load_stored(store, id).await,
                None => crate::quiz_context::load_demo().await,
            }
        }
    });
    view! { {move || loaded.get().map(|result| match result {
        Ok(quiz) => view! { <LoadedShow quiz=quiz.clone() presenter=presenter/> }.into_any(),
        Err(error) => view! { <div class="min-h-screen bg-[#050505] text-white p-12"><p role="alert">{error.clone()}</p><a href="#/" class="text-yellow-400">"← Library"</a></div> }.into_any(),
    }).unwrap_or_else(|| view! { <p>"Loading quiz…"</p> }.into_any())} }
}
// Keep the bundle's offline fallback while routing Sheet reads through the Google session.
/// What QuizShow.jsx puts on its ThemeScope.
const SCOPE_CLASS: &str = concat!(
    "bg-ground h-full w-full font-show overflow-hidden relative ",
    "selection:bg-accent-500/30 text-white",
);

#[component]
fn LoadedShow(quiz: crate::quiz_context::QuizContext, presenter: bool) -> impl IntoView {
    let quiz_id = StoredValue::new(quiz.quiz.id.clone());
    let tour_rounds = StoredValue::new(quiz.rounds.clone());
    let builtin = quiz.quiz.builtin;
    let store = StoredValue::new_local(crate::store::use_store());
    let role =
        web_sys::UrlSearchParams::new_with_str(&window().location().search().unwrap_or_default())
            .ok()
            .and_then(|p| p.get("role"))
            .filter(|v| v == "presenter")
            .map_or(Role::Main, |_| Role::Presenter);
    let role = if presenter { Role::Presenter } else { role };
    let reducer = StoredValue::new(quiz_core::show::Show::new(
        role,
        quiz.quiz.settings.clone(),
        Intro::new(&quiz.quiz, &crate::scenes::default_rules()),
    ));
    let settings = RwSignal::new(quiz.quiz.settings.clone());
    crate::i18n::provide_translations(Signal::derive(move || {
        crate::i18n::Lang::from_code(&settings.get().language)
    }));
    let custom = StoredValue::new(
        quiz.quiz
            .fields
            .get("customTheme")
            .and_then(|v| serde_json::from_value::<quiz_core::theme::Theme>(v.clone()).ok()),
    );
    // ThemeScope already derives the same family name for Font::File entries.
    // Install the rules while this loaded quiz owns its object URLs.
    let mut font_css = String::new();
    custom.with_value(|theme| {
        if let Some(theme) = theme {
            for font in theme.fonts.values() {
                if let quiz_core::theme::format::Font::File { file } = font
                    && let Some(url) = quiz.resolve_media(file) {
                        let family = quiz_core::theme::builtin::font_family(font);
                        font_css.push_str(&format!("@font-face {{ font-family: '{family}'; src: url('{url}'); font-display: swap; }}"));
                    }
            }
        }
    });
    let font_style = document().create_element("style").ok();
    if let Some(style) = &font_style {
        style.set_text_content(Some(&font_css));
        if let Ok(Some(head)) = document().query_selector("head") {
            let _ = head.append_child(style);
        }
    }
    let font_style = StoredValue::new_local(font_style);
    on_cleanup(move || {
        if let Some(style) = font_style.get_value() {
            style.remove();
        }
    });
    let theme_override = StoredValue::new(crate::quiz_context::query_param("theme"));
    let theme = Signal::derive(move || {
        if let Some(id) = theme_override.get_value() {
            quiz_core::theme::builtin::theme_by_id(&id).clone()
        } else {
            custom.with_value(|custom| {
                crate::settings::selected_theme(&settings.get(), custom.as_ref())
            })
        }
    });
    provide_context(theme);
    let t = crate::i18n::use_t();
    let ui = ShowUi {
        settings_open: RwSignal::new(false),
        calibrating: RwSignal::new(false),
    };
    provide_context(ui);
    let ui = use_show_ui();
    crate::scores::provide_scores(quiz.clone());
    let live_scores = crate::scores::use_scores();
    let asset_quiz = quiz.clone();
    provide_context(crate::theme::ThemeAssets(Callback::new(
        move |name: String| asset_quiz.resolve_media(&name),
    )));
    provide_context(quiz);
    let state = RwSignal::new(reducer.with_value(|r| r.state().clone()));
    let channel = StoredValue::new_local(
        web_sys::BroadcastChannel::new(&format!("quiz-app-sync-{}", quiz_id.get_value())).ok(),
    );
    let send = move |messages| {
        channel.with_value(|c| {
            if let Some(c) = c {
                post(c, messages);
            }
        })
    };
    let dispatch = Callback::new(move |action| {
        reducer.update_value(|r| {
            send(r.dispatch(action));
            state.set(r.state().clone());
        });
    });
    provide_context(ShowContext {
        role,
        state,
        dispatch,
    });
    let update = Callback::new(move |next: quiz_core::quiz::model::Settings| {
        reducer.update_value(|r| send(r.set_settings(next.clone())));
        settings.set(next.clone());
        if !builtin {
            let store = store.get_value();
            let id = quiz_id.get_value();
            leptos::task::spawn_local(async move {
                use quiz_store::QuizStore;
                if let Err(e) = store.update_quiz(&id, |q| q.settings = next).await {
                    web_sys::console::error_1(&e.to_string().into());
                }
            });
        }
    });
    // Scene settings consumers must read SettingsContext for live CONFIG changes.
    provide_context(settings);
    provide_context(crate::settings::SettingsContext { settings, update });
    let incoming =
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
            if let Ok(json) = js_sys::JSON::stringify(&event.data()) {
                if let Some(json) = json.as_string() {
                    if let Ok(message) = serde_json::from_str::<SyncMessage>(&json) {
                        reducer.update_value(|r| {
                            send(r.receive(&message));
                            if settings.get_untracked() != *r.settings() {
                                settings.set(r.settings().clone());
                            }
                            state.set(r.state().clone());
                        });
                    }
                }
            }
        });
    channel.with_value(|c| {
        if let Some(c) = c {
            c.set_onmessage(Some(incoming.as_ref().unchecked_ref()));
        }
    });
    send(reducer.with_value(|r| r.on_open()));
    // Dev only: `?round=<n>` starts the sheet's n-th round (hidden ones too), `&mode=questions`
    // without answers (crate README)
    if let Some(n) = crate::quiz_context::query_param("round").and_then(|n| n.parse::<usize>().ok())
    {
        if let Some(round) = crate::quiz_context::use_quiz()
            .rounds
            .get(n.wrapping_sub(1))
        {
            let questions =
                crate::quiz_context::query_param("mode").as_deref() == Some("questions");
            dispatch.run(Action::StartRound {
                round_id: round.id.clone(),
                mode: if questions {
                    PlayMode::QuestionsOnly
                } else {
                    PlayMode::WithAnswers
                },
            });
        }
    }
    let heartbeat = (role == Role::Presenter).then(|| {
        gloo_timers::callback::Interval::new(10_000, move || {
            send(vec![SyncMessage::PresenterHere { sender: role }])
        })
    });
    let hide = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
        if role == Role::Presenter {
            send(vec![SyncMessage::PresenterGone { sender: role }]);
        }
    });
    let keys =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            if !event.shift_key() {
                return;
            }
            match event.key().to_ascii_lowercase().as_str() {
                "c" => ui.calibrating.update(|value| *value = !*value),
                "d" => dispatch.run(Action::BackToDashboard),
                "p" => {
                    let location = window().location();
                    let url = format!(
                        "{}{}{}",
                        location.origin().unwrap_or_default(),
                        location.pathname().unwrap_or_default(),
                        if location.hash().unwrap_or_default().is_empty() {
                            let search = location.search().unwrap_or_default();
                            let params = web_sys::UrlSearchParams::new_with_str(&search).unwrap();
                            params.set("role", "presenter");
                            format!("?{}", params.to_string())
                        } else {
                            crate::router::Route::Presenter(quiz_id.get_value()).hash()
                        }
                    );
                    let _ = window().open_with_url_and_target_and_features(
                        &url,
                        "PresenterWindow",
                        "width=1200,height=800",
                    );
                }
                _ => {}
            }
        });
    let _ = window().add_event_listener_with_callback("pagehide", hide.as_ref().unchecked_ref());
    let _ = window().add_event_listener_with_callback("keydown", keys.as_ref().unchecked_ref());
    let resources = StoredValue::new_local(Some((incoming, hide, keys, heartbeat)));
    on_cleanup(move || {
        let Some((incoming, hide, keys, heartbeat)) =
            resources.try_update_value(|r| r.take()).flatten()
        else {
            return;
        };
        let _ =
            window().remove_event_listener_with_callback("pagehide", hide.as_ref().unchecked_ref());
        let _ =
            window().remove_event_listener_with_callback("keydown", keys.as_ref().unchecked_ref());
        channel.with_value(|c| {
            if let Some(c) = c {
                c.set_onmessage(None);
                c.close();
            }
        });
        drop((incoming, hide, keys, heartbeat));
    });
    let display = crate::calibration::display_settings(role);
    let bypass = Signal::derive(move || {
        // Read navigation to react to both local and remote scene changes.
        let _ = state.get();
        reducer.with_value(|r| r.uses_full_window(ui.calibrating.get()))
    });
    let quiet_scene = Memo::new(move |_| matches!(state.get().scene as u64, 4 | 5));
    view! { <crate::stage::Stage display=display bypass=bypass>
        <crate::theme::ThemeScope theme=theme class=SCOPE_CLASS>
        <Show when=move || role == Role::Main>
            <div class="absolute top-0 w-full p-6 flex justify-between z-50 pointer-events-none">
                <div class="pointer-events-auto flex gap-2">
                    <Show when=move || matches!(state.get().scene as u64, 4..=6)>
                        <button on:click=move |_| dispatch.run(Action::BackToDashboard)
                            class=BACK_CLASS><frame_icons::ArrowLeft/></button>
                    </Show>
                </div>
                <div class="pointer-events-auto flex gap-4">
                    <button on:click=move |_| ui.settings_open.update(|value| *value = !*value)
                        class=SETTINGS_CLASS><frame_icons::Settings/></button>
                </div>
            </div>
        </Show>
        <crate::presence::PresenceSwitch scene=Signal::derive(move || state.get().scene as u64)
            wait=Signal::derive(|| true) enter="show-enter" exit="show-exit" render=move |key| {
                use crate::scenes::*;
                match key {
                    0 => view! { <welcome::Welcome/> }.into_any(),
                    1 => view! { <rules::Rules/> }.into_any(),
                    2 => view! { <prizes::Prizes/> }.into_any(),
                    3 => view! { <dashboard::Dashboard/> }.into_any(),
                    4 => view! { <game::Game/> }.into_any(),
                    5 => view! { <leaderboard::Leaderboard/> }.into_any(),
                    _ => view! { <pause::Pause/> }.into_any(),
                }
            }/>
        <SoundOwner owns=Signal::derive(move || reducer.with_value(|r| r.plays_sounds()))/>
        <Show when=move || role == Role::Main>
            {move || { let quiet = quiet_scene.get(); view! { <crate::scenes::backdrop::ThemeDecorations quiet=quiet/> } }}
        </Show>
        <Show when=move || role == Role::Main>
            <div class=FOOTER_CLASS>
                <span class="tracking-widest">"Quiz Night OS v2.0"</span>
                <Show when=move || settings.get().show_time>
                    <span class="font-mono text-accent-500/50">
                        {move || t.get("start_show")}{": "}{move || settings.get().start_time}
                    </span>
                </Show>
            </div>
        </Show>
        <Show when=move || ui.calibrating.get()><crate::calibration::CalibrationPattern/></Show>
        </crate::theme::ThemeScope>
    </crate::stage::Stage>
    <Show when=move || role == Role::Presenter && !builtin>
        <div class="fixed bottom-4 left-4 z-[150] flex gap-2 font-sans">
            <Show when=move || live_scores.auth_needed.get()><button title=move ||t.get("lb_signin_needed") on:click=move |_|crate::scores::renew_sign_in() class="px-4 py-3 rounded-xl bg-amber-500 text-black font-bold">{move ||t.get("lb_signin")}</button></Show>
            <Show when=move || !live_scores.auth_needed.get() && live_scores.sign_in_ending.get().is_some()><button on:click=move |_|crate::scores::renew_sign_in() class="px-4 py-3 rounded-xl bg-amber-500/15 border border-amber-500/50 text-amber-200 font-bold">{move ||t.fmt("lb_signin_ending",&[("n", &live_scores.sign_in_ending.get().unwrap_or(1).to_string())])}</button></Show>
        </div>
    </Show>
    <crate::settings::SettingsModal/>
    <Show when=move || ui.calibrating.get()><crate::calibration::CalibrationControls display=display/></Show>
    <Show when=move || role == Role::Main><crate::calibration::FullscreenHint/></Show>
    <Show when=move || !ui.calibrating.get()>
        <crate::tour::TourGuide
            quiz_id=Signal::derive(move || quiz_id.get_value())
            scene=Signal::derive(move || state.get().scene)
            active_round=Signal::derive(move || {
                let id = state.get().active_round_id;
                tour_rounds.with_value(|rounds| rounds.iter().find(|r| Some(&r.id) == id.as_ref()).cloned())
            })
            is_presenter=Signal::derive(move || role == Role::Presenter)
            t=t
        />
    </Show>
    }
}

const BACK_CLASS: &str = concat!(
    "p-3 bg-white/10 border border-white/10 text-white rounded-full hover:bg-white ",
    "hover:text-black transition-all",
);

const SETTINGS_CLASS: &str = concat!(
    "p-3 bg-white/10 border border-white/10 text-white rounded-full hover:bg-white ",
    "hover:text-black hover:rotate-90 transition-all",
);

const FOOTER_CLASS: &str = concat!(
    "absolute bottom-0 w-full h-10 bg-black/50 border-t border-white/5 flex items-center ",
    "justify-between px-6 text-gray-500 text-xs z-40 pointer-events-none backdrop-blur-sm",
);

/// Installs the show's sound player; it has to sit inside the ThemeScope to see the theme.
#[component]
fn SoundOwner(owns: Signal<bool>) -> impl IntoView {
    crate::sounds::provide_player(owns);
}
