//! Read-only dashboard cards with an isolated show context and scoped theme.
use super::*;
use std::sync::Arc;

#[component]
pub(super) fn Preview(editor: Editor, theme: Theme) -> impl IntoView {
    let theme = StoredValue::new(theme);
    view! { <div class="editor-theme-frame relative aspect-video overflow-hidden rounded-lg pointer-events-none" inert=true aria-hidden="true">
        {move || {
            let quiz = editor.quiz.get();
            let media = editor.media.get();
            view! { <DashboardPreview quiz media theme=theme.get_value()/> }
        }}
    </div> }
}
#[component]
fn DashboardPreview(quiz: Quiz, media: Vec<Media>, theme: Theme) -> impl IntoView {
    let mut urls = store::MediaUrls::default();
    for name in quiz_core::theme::format::theme_files(&theme) {
        if let Some(media) = media
            .iter()
            .find(|m| media_key(&m.name) == media_key(&name))
            && let Ok(blob) = store::blob(&media.bytes, &media.content_type)
            && let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob)
        {
            urls.0.insert(media_key(&name), url);
        }
    }
    let urls = Arc::new(urls);
    let image = theme
        .background
        .image
        .as_ref()
        .and_then(|name| {
            urls.0
                .get(&media_key(name))
                .cloned()
                .or_else(|| is_url(name).then(|| name.clone()))
        })
        .map(|url| {
            format!(
                "background-image:url(\"{}\");opacity:{};filter:blur({}px)",
                url.replace('"', "%22"),
                theme.background.image_opacity * 0.5,
                theme.background.image_blur
            )
        })
        .unwrap_or_default();
    let fonts = theme
        .fonts
        .values()
        .filter_map(|font| {
            if let quiz_core::theme::format::Font::File { file } = font {
                urls.0.get(&media_key(file)).map(|url| {
                    format!(
                        "@font-face{{font-family:'{}';src:url('{}');font-display:swap;}}",
                        builtin::font_family(font).replace('\'', "\\'"),
                        url
                    )
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    crate::i18n::provide_translations(Signal::stored(crate::i18n::Lang::from_code(
        &quiz.settings.language,
    )));
    provide_context(crate::quiz_context::QuizContext {
        rounds: Arc::new(parse_questions_csv(&quiz.questions_csv)),
        quiz: Arc::new(quiz),
        teams: Arc::new(vec![]),
        media_urls: urls,
    });
    provide_context(crate::show::ShowContext {
        role: quiz_core::show::Role::Main,
        state: RwSignal::new(quiz_core::show::ShowState {
            scene: quiz_core::show::Scene::Dashboard,
            active_round_id: None,
            play_mode: quiz_core::show::PlayMode::WithAnswers,
        }),
        dispatch: Callback::new(|_| ()),
    });
    provide_context(Signal::stored(crate::stage::StageLayout {
        w: 2400.,
        h: 1350.,
        scale: 1.,
    }));
    let frame = NodeRef::<leptos::html::Div>::new();
    let scale = RwSignal::new(0.1);
    Effect::new(move |_| {
        if let Some(element) = frame.get() {
            let update = move || {
                if let Some(element) = frame.get() {
                    scale.set(element.get_bounding_client_rect().width() / 2400.);
                }
            };
            update();
            let callback =
                wasm_bindgen::closure::Closure::<dyn FnMut(js_sys::Array)>::new(move |_| update());
            if let Ok(observer) = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) {
                observer.observe(&element);
                let observer = StoredValue::new_local((observer, callback));
                on_cleanup(move || observer.with_value(|(observer, _)| observer.disconnect()));
            }
        }
    });
    view! { <div node_ref=frame class="w-full h-full">
        <style>{fonts}</style>
        <div class="editor-theme-canvas" style:transform=move || format!("scale({})", scale.get())>
            <crate::theme::ThemeScope theme=Signal::stored(theme) class="bg-ground font-show relative w-full h-full overflow-hidden">
                <crate::scenes::dashboard::Dashboard/>
                <div class="absolute inset-0 z-0 bg-cover bg-center pointer-events-none" style=image/>
                <crate::scenes::backdrop::ThemeDecorations/>
            </crate::theme::ThemeScope>
        </div>
    </div> }
}
