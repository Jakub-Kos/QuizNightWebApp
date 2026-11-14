//! Two-window show launch, with optional placement on an extended display.
use super::*;
use wasm_bindgen::JsValue;

fn property(object: &JsValue, name: &str) -> JsValue {
    js_sys::Reflect::get(object, &name.into()).unwrap_or_default()
}
fn rich(text: String) -> impl IntoView {
    text.split('`').enumerate().map(|(i, part)| {
        if i % 2 == 1 {
            view! { <code class="px-1.5 py-0.5 rounded bg-white/10 text-yellow-200 font-mono text-[0.9em] whitespace-nowrap">{part.to_owned()}</code> }.into_any()
        } else { part.to_owned().into_any() }
    }).collect_view()
}
fn absolute_url(route: Route) -> String {
    let location = window().location();
    format!(
        "{}{}{}{}",
        location.origin().unwrap_or_default(),
        location.pathname().unwrap_or_default(),
        location.search().unwrap_or_default(),
        route.hash()
    )
}
async fn open_tv(url: String) -> &'static str {
    let browser = window();
    let mut features = "popup,width=1280,height=720".to_owned();
    let mut placed = false;
    // Window Management is optional. Denied permission falls back to a regular popup.
    if property(&property(browser.as_ref(), "screen"), "isExtended").as_bool() == Some(true)
        && let Ok(get_details) =
            property(browser.as_ref(), "getScreenDetails").dyn_into::<js_sys::Function>()
        && let Ok(promise) = get_details.call0(browser.as_ref())
        && let Ok(details) =
            wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&promise)).await
    {
        let current = property(&details, "currentScreen");
        for screen in js_sys::Array::from(&property(&details, "screens")).iter() {
            if screen == current {
                continue;
            }
            let coordinate = |key| property(&screen, key).as_f64().unwrap_or(0.) as i32;
            let width = coordinate("availWidth");
            let height = coordinate("availHeight");
            if width > 0 && height > 0 {
                features = format!(
                    "popup,left={},top={},width={width},height={height}",
                    coordinate("availLeft"),
                    coordinate("availTop")
                );
                placed = true;
                break;
            }
        }
    }
    match browser.open_with_url_and_target_and_features(&url, "QuizNightTV", &features) {
        Ok(Some(_)) if placed => "screen",
        Ok(Some(_)) => "opened",
        _ => "blocked",
    }
}

#[component]
pub(super) fn Launcher(editor: Editor) -> impl IntoView {
    let tv = RwSignal::new("");
    let presenter = RwSignal::new(false);
    let presenter_blocked = RwSignal::new(false);
    let opening = RwSignal::new(false);
    let dialog = NodeRef::<leptos::html::Div>::new();
    let previous_focus = document().active_element();
    Effect::new(move |_| {
        if let Some(dialog) = dialog.get() {
            let _ = dialog.focus();
        }
    });
    on_cleanup(move || {
        if let Some(element) =
            previous_focus.and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = element.focus();
        }
    });
    let open_tv = move |_| {
        if opening.get_untracked() {
            return;
        }
        opening.set(true);
        let url = absolute_url(Route::Show(editor.id()));
        spawn_local(async move {
            let state = open_tv(url).await;
            tv.try_set(state);
            opening.try_set(false);
        });
    };
    let open_presenter = move |_| {
        let opened = matches!(
            window().open_with_url_and_target_and_features(
                &absolute_url(Route::Presenter(editor.id())),
                "PresenterWindow",
                "width=1200,height=800"
            ),
            Ok(Some(_))
        );
        presenter.set(opened);
        presenter_blocked.set(!opened);
    };
    view! {
        <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-sm flex items-center justify-center p-4 font-sans text-white" on:click=move |_| editor.launcher.set(false)>
            <div node_ref=dialog tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="editor-launcher-title"
                class="editor-launcher w-full max-w-xl max-h-full overflow-y-auto rounded-3xl bg-[#0f0f0f] border border-white/10 shadow-2xl p-6 sm:p-8 space-y-5"
                on:click=|ev| ev.stop_propagation()
                on:keydown=move |ev| {
                    if ev.key() == "Escape" { ev.prevent_default(); editor.launcher.set(false); }
                    if ev.key() == "Tab" && let Some(dialog) = dialog.get()
                        && let Ok(nodes) = dialog.query_selector_all("button:not(:disabled), a[href]")
                    {
                        let first = nodes.item(0).and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());
                        let last = nodes.item(nodes.length().saturating_sub(1)).and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());
                        let active = document().active_element();
                        if ev.shift_key() && active.as_ref() == first.as_ref().map(|n| n.as_ref()) {
                            ev.prevent_default(); if let Some(last) = last { let _ = last.focus(); }
                        } else if !ev.shift_key() && (active.as_ref() == last.as_ref().map(|n| n.as_ref()) || active.as_ref() == Some(dialog.as_ref())) {
                            ev.prevent_default(); if let Some(first) = first { let _ = first.focus(); }
                        }
                    }
                }>
                <div class="flex items-start gap-4"><div class="flex-1 space-y-2">
                    <h2 id="editor-launcher-title" class="font-['League_Spartan'] text-3xl font-black uppercase tracking-wide text-yellow-400">{move || editor.t.get("sl_title")}</h2>
                    <p class="text-gray-300">{move || editor.t.get("sl_intro")}</p>
                </div><button class="p-2 rounded-full text-gray-400 hover:text-white hover:bg-white/10" aria-label=move || editor.t.get("chk_close") on:click=move |_| editor.launcher.set(false)>"×"</button></div>
                <div class=move || if matches!(tv.get(), "opened" | "screen") { "rounded-2xl border p-5 space-y-3 border-green-500/30 bg-green-500/[0.04]" } else { "rounded-2xl border p-5 space-y-3 border-white/10 bg-white/[0.03]" }>
                    <h3 class="font-bold text-lg">"1. "{move || editor.t.get("sl_tv_title")}</h3>
                    <p role="status" class=move || if tv.get() == "blocked" { "text-sm text-yellow-200" } else { "text-sm text-gray-300" }>{move || editor.t.get(match tv.get() { "opened" => "sl_tv_opened", "screen" => "sl_tv_opened_screen", "blocked" => "sl_tv_blocked", _ => "sl_tv_text" })}</p>
                    <button class=PRIMARY disabled=move || opening.get() on:click=open_tv><Icon name="play"/>{move || editor.t.get(if matches!(tv.get(), "opened" | "screen") { "sl_tv_again" } else { "sl_tv_open" })}</button>
                </div>
                <div class=move || if presenter.get() { "rounded-2xl border p-5 space-y-3 border-green-500/30 bg-green-500/[0.04]" } else { "rounded-2xl border p-5 space-y-3 border-white/10 bg-white/[0.03]" }>
                    <h3 class="font-bold text-lg">"2. "{move || editor.t.get("sl_presenter_title")}</h3>
                    <p class="text-sm text-gray-300">{move || editor.t.get("sl_presenter_text")}</p>
                    <p role="status" class="text-sm text-yellow-200" hidden=move || !presenter_blocked.get()>{move || editor.t.get("sl_tv_blocked")}</p>
                    <button class=PRIMARY on:click=open_presenter>{move || editor.t.get("sl_presenter_open")}</button>
                </div>
                <p class="text-sm text-gray-400">{move || rich(editor.t.get("sl_keys"))}</p>
                <div class="pt-4 border-t border-white/5 flex flex-wrap gap-4"><a class="flex items-center gap-2 text-sm text-gray-400 hover:text-white" href=editor.link(Route::Show)><Icon name="play"/>{move || editor.t.get("sl_one_screen")}</a><Show when=move || editor.id() == quiz_core::quiz::model::DEMO_ID><button class="text-sm text-yellow-300 hover:text-yellow-200" on:click=move |_| crate::tour::start_tour(&editor.id())>{move || editor.t.get("tr_start")}</button></Show></div>
            </div>
        </div>
    }
}
