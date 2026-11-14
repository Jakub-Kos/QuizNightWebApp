//! Phase 1 spike of the Rust rewrite (docs/RUST_REWRITE.md): proves the show's animations without
//! Framer Motion. The modules are meant to be kept; `demos` is throwaway.
//!
//! The default URL opens the library; dev query parameters retain the built-in demo show.
//! Open a spike demo with `?demo=stage|leaderboard|presence|confetti`.

mod calibration;
mod demos;
mod i18n;
mod pages;
mod particles;
mod presence;
mod quiz_context;
mod reorder;
mod router;
mod scenes;
mod scores;
mod settings;
mod show;
mod sounds;
mod spring;
mod stage;
mod store;
mod theme;
mod tour;

use leptos::prelude::*;
use stage::Stage;

/// The spike demo asked for with `?demo=`, if any.
fn demo_param() -> Option<String> {
    let search = window().location().search().unwrap_or_default();
    web_sys::UrlSearchParams::new_with_str(&search)
        .ok()
        .and_then(|params| params.get("demo"))
}

fn demo_name() -> String {
    demo_param().unwrap_or_else(|| "leaderboard".to_string())
}

/// The chosen demo. Built inside `Stage`, because the demos read the contexts it provides.
#[component]
fn Demo() -> impl IntoView {
    match demo_name().as_str() {
        "stage" => view! { <demos::stage::StageDemo/> }.into_any(),
        "presence" => view! { <demos::presence::PresenceDemo/> }.into_any(),
        "confetti" => view! { <demos::confetti::ConfettiDemo/> }.into_any(),
        _ => view! { <demos::leaderboard::LeaderboardDemo/> }.into_any(),
    }
}

/// The four spike demos with a switcher between them.
#[component]
fn Spike() -> impl IntoView {
    let current = demo_name();
    let links = ["leaderboard", "presence", "confetti", "stage"].map(|name| {
        let class = if name == current {
            "demo-link demo-link-current"
        } else {
            "demo-link"
        };
        view! { <a class=class href=format!("?demo={name}")>{name}</a> }
    });
    view! {
        <Stage><Demo/></Stage>
        <nav class="demo-nav">{links.to_vec()}</nav>
    }
}

#[component]
fn Shell() -> impl IntoView {
    let route = router::current_route();
    let lang = RwSignal::new(i18n::Lang::from_code(
        &window()
            .local_storage()
            .ok()
            .flatten()
            .and_then(|s| s.get_item("quiznight.lang").ok().flatten())
            .unwrap_or_else(|| "cs".into()),
    ));
    provide_context(lang);
    i18n::provide_translations(Signal::derive(move || lang.get()));
    view! { {move || {
        let route = route.get();
        let no_hash = window().location().hash().unwrap_or_default().is_empty();
        if no_hash && demo_param().is_some() { return view! { <Spike/> }.into_any(); }
        if no_hash && (["round", "mode", "theme", "csv", "scores", "dash", "welcome", "role"].iter()
            .any(|key| quiz_context::query_param(key).is_some())
            || quiz_context::query_param("show").as_deref() == Some("demo")) {
            return view! { <show::ShowApp/> }.into_any();
        }
        match route {
            router::Route::Show(id) => view! { <show::ShowApp id=id/> }.into_any(),
            router::Route::Presenter(id) => view! { <show::ShowApp id=id presenter=true/> }.into_any(),
            router::Route::Library | router::Route::Drive(_) => view! { <pages::library::Page/> }.into_any(),
            router::Route::Help(section) => view! { <pages::help::Page section=section/> }.into_any(),
            router::Route::Themes => view! { <pages::workshop::Page/> }.into_any(),
            router::Route::Edit(id) => view! { <pages::quiz_editor::Page id=id/> }.into_any(),
            router::Route::Questions(id) => view! { <pages::questions_editor::Page id=id/> }.into_any(),
            router::Route::Theme { id, fresh } => view! { <pages::theme_editor::Page id=id fresh=fresh/> }.into_any(),
            router::Route::Check(id) => view! { <pages::quiz_check::Page id=id/> }.into_any(),
            router::Route::Map(id) => view! { <pages::quiz_map::Page id=id/> }.into_any(),
            router::Route::Attendance(id) => view! { <pages::attendance::Page id=id/> }.into_any(),
            router::Route::Scores(id) => view! { <pages::scores_entry::Page id=id/> }.into_any(),
        }
    }} }
}
#[component]
fn App() -> impl IntoView {
    let store = LocalResource::new(store::open);
    view! { {move || store.get().map(|store| {
        store::provide_store(store.clone());
        view! { <Shell/> }
    })} }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
