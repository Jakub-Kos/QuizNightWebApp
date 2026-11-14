use super::*;

#[component]
pub(super) fn Start(editor: Editor, on_examples: Callback<()>) -> impl IntoView {
    view! { <div class="space-y-4">
        <h3 class="font-bold">{move || editor.t.get("qs_title")}</h3>
        <div class="grid sm:grid-cols-3 gap-3">
            <div class="rounded-xl border border-white/10 bg-black/30 p-4 space-y-3 flex flex-col">
                <h4 class="font-bold text-sm">{move || editor.t.get("qs_sheet_title")}</h4>
                <p class="text-sm text-gray-400">{move || editor.t.get("qs_sheet_text")}</p>
                <a class=PRIMARY href="https://docs.google.com/spreadsheets/d/1p1-bxa2EBWcRvRE_bWu3AbHaSHkgjD7jgBcvu1tWNvw/copy" target="_blank" rel="noreferrer">{move || editor.t.get("qs_sheet_copy")}</a>
            </div>
            <div class="rounded-xl border border-white/10 bg-black/30 p-4 space-y-3 flex flex-col">
                <h4 class="font-bold text-sm">{move || editor.t.get("qs_examples_title")}</h4>
                <p class="text-sm text-gray-400">{move || editor.t.get("qs_examples_text")}</p>
                <button class=BUTTON disabled=move || editor.busy.get() on:click=move |_| on_examples.run(())>{move || editor.t.get("qs_examples_load")}</button>
            </div>
            <div class="rounded-xl border border-white/10 bg-black/30 p-4 space-y-3 flex flex-col">
                <h4 class="font-bold text-sm">{move || editor.t.get("qs_csv_title")}</h4>
                <p class="text-sm text-gray-400">{move || editor.t.get("qs_csv_text")}</p>
                <a class=BUTTON href="templates/questions-template.csv" download="questions-template.csv">{move || editor.t.get("qs_csv_download")}</a>
            </div>
        </div>
        <figure class="space-y-2"><figcaption class="text-xs text-gray-500">{move || editor.t.get("qs_preview")}</figcaption>
            <div class="overflow-x-auto rounded-xl border border-white/10"><table class="w-full text-xs text-left"><thead class="bg-white/5"><tr>{quiz_core::quiz::examples::SHEET_HEAD.into_iter().map(|head| view! { <th class="p-2 whitespace-nowrap">{head}</th> }).collect_view()}</tr></thead>
                <tbody>{move || {
                    let language = editor.quiz.with(|q| q.settings.language.clone());
                    quiz_core::quiz::examples::example_questions(&language, &editor.t.get("ed_example_round")).rows.into_iter().take(4).map(|row| view! { <tr class="border-t border-white/10">{row.into_iter().map(|cell| view! { <td class="p-2">{cell}</td> }).collect_view()}</tr> }).collect_view()
                }}</tbody>
            </table></div>
        </figure>
        <a class="inline-flex items-center gap-1.5 text-sm text-gray-400 hover:text-yellow-400" href="#/help/questions"><Icon name="help"/>{move || editor.t.get("qs_help")}</a>
    </div> }
}
