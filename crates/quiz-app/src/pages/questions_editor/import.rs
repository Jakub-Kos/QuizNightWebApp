use super::*;

fn importable(csv: &str) -> bool {
    !csv.trim_start().starts_with('<') && sheet(csv).rounds.iter().any(|r| !r.questions.is_empty())
}

#[component]
pub(super) fn CsvImport(action: Callback<Action>, busy: RwSignal<bool>) -> impl IntoView {
    let t = crate::i18n::use_t();
    let csv = RwSignal::new(String::new());
    let url = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());
    let over = RwSignal::new(false);
    let accept = Callback::new(move |text: String| {
        if importable(&text) {
            error.set(String::new());
            action.run(Action::Import(text));
        } else {
            error.set(t.questions_editor("qe_csv_invalid"));
        }
    });
    let file = Callback::new(move |file: web_sys::File| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        leptos::task::spawn_local(async move {
            let result = wasm_bindgen_futures::JsFuture::from(file.text()).await;
            busy.set(false);
            match result {
                Ok(text) => accept.run(text.as_string().unwrap_or_default()),
                Err(e) => error.set(format!("{e:?}")),
            }
        });
    });
    view! {
        <details class="rounded-2xl border border-white/10 bg-white/[0.03] p-5">
            <summary class="font-bold cursor-pointer">{t.questions_editor("qe_csv_import")}</summary>
            <fieldset disabled=move ||busy.get() class="space-y-4 mt-4">
                <p class="text-sm text-gray-400">{t.questions_editor("qe_csv_replace")}</p>
                <label class="block space-y-2"><span class="text-sm">{t.questions_editor("qe_csv_url")}</span>
                    <input type="url" class=INPUT prop:value=move ||url.get() on:input=move |e|url.set(event_target_value(&e))/>
                </label>
                <button class=BUTTON disabled=move ||busy.get() ||url.get().trim().is_empty() on:click=move |_| {
                    let address = quiz_core::quiz::sheets::to_csv_url(&url.get_untracked());
                    if !is_url(&address) { error.set(t.questions_editor("qe_csv_invalid")); return; }
                    busy.set(true);
                    error.set(String::new());
                    leptos::task::spawn_local(async move {
                        let result: Result<String,String> = async {
                            let response = gloo_net::http::Request::get(&address).send().await.map_err(|e|e.to_string())?;
                            if !response.ok() { return Err(format!("HTTP {}",response.status())); }
                            response.text().await.map_err(|e|e.to_string())
                        }.await;
                        busy.set(false);
                        match result { Ok(text)=>accept.run(text), Err(e)=>error.set(e) }
                    });
                }>{t.questions_editor("qe_csv_load")}</button>
                <label class="block space-y-2"><span class="text-sm">{t.questions_editor("qe_csv_paste")}</span>
                    <textarea class=INPUT rows="5" prop:value=move ||csv.get() on:input=move |e|csv.set(event_target_value(&e))/>
                </label>
                <button class=BUTTON disabled=move ||busy.get() ||csv.get().trim().is_empty() on:click=move |_|accept.run(csv.get_untracked())>{t.questions_editor("qe_csv_import")}</button>
                <div class="questions-file-drop rounded-xl border-2 border-dashed border-white/15 p-4" class:questions-file-over=move ||over.get()
                    on:dragover=move |e:web_sys::DragEvent|{e.prevent_default();over.set(true);}
                    on:dragleave=move |_|over.set(false)
                    on:drop=move |e:web_sys::DragEvent|{e.prevent_default();over.set(false);if let Some(f)=e.data_transfer().and_then(|d|d.files()).and_then(|f|f.item(0)){file.run(f);}}>
                    <label class="text-sm">{t.get("ed_upload_csv")}<input class="block mt-2 text-sm" type="file" accept=".csv,text/csv" on:change=move |e|{let input=event_target::<web_sys::HtmlInputElement>(&e);if let Some(f)=input.files().and_then(|f|f.item(0)){file.run(f);}input.set_value("");}/></label>
                </div>
                <p role="alert" class="text-sm text-red-300">{move ||error.get()}</p>
            </fieldset>
        </details>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn import_requires_questions_and_rejects_html() {
        assert!(!importable(""));
        assert!(!importable("<html>Sign in</html>"));
        assert!(!importable("Kolo,Q#,Question,Type"));
        assert!(importable(
            &examples::example_questions("en", "Examples").csv
        ));
    }
}
