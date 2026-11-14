use super::*;
use crate::{
    google::{auth, config, drive, sync},
    pages::{drive_parts, drive_people::DrivePeople},
};

fn string(v: &Value, k: &str) -> String {
    v[k].as_str().unwrap_or_default().into()
}
fn report(editor: Editor, e: crate::google::Error) -> String {
    drive_parts::error_text(&e, editor.t).unwrap_or_default()
}
async fn refresh(editor: Editor) -> Result<(), String> {
    let data = editor.data.get_value();
    let q = editor.quiz.get_untracked();
    let d = q.drive.as_ref().unwrap();
    let link = d["folderId"]
        .as_str()
        .or(d["spreadsheetId"].as_str())
        .unwrap_or_default();
    let fields = drive::read_drive_quiz(link)
        .await
        .map_err(|e| report(editor, e))?;
    let folder = fields["drive"]["mediaFolderId"].as_str().map(str::to_owned);
    data.store
        .update_quiz_source(
            &data.id,
            move |q| config::merge_drive(q, &fields, &json!({})),
            "drive",
        )
        .await
        .map_err(|e| e.to_string())?;
    sync::sync_drive_config(&data.id)
        .await
        .map_err(|e| report(editor, e))?;
    let media = drive::sync_drive_media(
        &data.id,
        folder.as_deref(),
        Some(Rc::new(move |p| {
            editor.drive_progress.try_set(Some(p));
        })),
    )
    .await
    .map_err(|e| report(editor, e))?;
    let name = auth::get_session().map(|s| s.user.name).unwrap_or_default();
    let saved = data
        .store
        .update_quiz_source(
            &data.id,
            move |q| {
                if let Some(d) = q.drive.as_mut() {
                    d["syncedAt"] = json!(store::now_ms());
                    d["syncedBy"] = json!(name);
                }
            },
            "drive",
        )
        .await
        .map_err(|e| e.to_string())?;
    if let Some(q) = saved
        && !data.queue.borrow().running
        && data.queue.borrow().pending.is_empty()
    {
        editor.quiz.try_set(q);
    }
    editor.reload_media().await?;
    let mut text = editor.t.fmt(
        "gd_result",
        &[
            ("downloaded", &media.downloaded.to_string()),
            ("kept", &media.kept.to_string()),
            ("removed", &media.removed.to_string()),
        ],
    );
    if !media.duplicates.is_empty() {
        text.push(' ');
        text.push_str(
            &editor
                .t
                .fmt("gd_duplicates", &[("names", &media.duplicates.join(", "))]),
        );
    }
    editor
        .status
        .try_set(Some((media.duplicates.is_empty(), text)));
    Ok(())
}
#[component]
pub(super) fn DriveSection(editor: Editor) -> impl IntoView {
    let signed = editor.signed_in;
    let started = RwSignal::new(false);
    let syncing = RwSignal::new(false);
    let run = Callback::new(move |()| {
        if editor.busy.get_untracked() {
            return;
        }
        editor.busy.set(true);
        syncing.set(true);
        editor.status.set(None);
        spawn_local(async move {
            if let Err(e) = refresh(editor).await {
                editor.message(Err(e));
            }
            editor.drive_progress.try_set(None);
            editor.busy.try_set(false);
            syncing.try_set(false);
        });
    });
    Effect::new(move |_| {
        if editor.quiz.with(|q| q.drive.is_some())
            && signed.get()
            && !started.get_untracked()
            && !editor.busy.get()
        {
            started.set(true);
            run.run(());
        }
    });
    view! {<Show when=move ||editor.quiz.with(|q|q.drive.is_some())><Section editor name="drive" title="gd_title" help="gd_section_help"><drive_parts::BetaBadge/><drive_parts::GoogleAccountBar on_error=Callback::new(move |s: Option<String>|editor.status.set(s.map(|s|(false,s))))/>
        {move ||editor.quiz.with(|q|q.drive.clone()).map(|d|{
            let folder=string(&d,"folderId");let sheet=string(&d,"spreadsheetId");let media=string(&d,"mediaFolderId");
            let file_id=if folder.is_empty(){sheet.clone()}else{folder.clone()};
            view!{<ul class="space-y-2 text-sm">{(!folder.is_empty()).then(||view!{<li><a href=config::folder_url(&folder) target="_blank" rel="noreferrer" class="hover:text-yellow-400">{string(&d,"folderName")}</a></li>})}<li><a href=config::sheet_url(&sheet,None) target="_blank" rel="noreferrer" class="hover:text-yellow-400">{string(&d,"spreadsheetName")}</a><span class="text-gray-500">" · "{editor.t.fmt("gd_questions_tab",&[("tab",&string(&d,"questionsTab"))])}{d["scoresTab"].as_str().map(|s|format!(" · {}",editor.t.fmt("gd_scores_tab",&[("tab",s)])))}</span></li><li>{if media.is_empty(){view!{<span class="text-yellow-300">{editor.t.get("gd_no_media_folder")}</span>}.into_any()}else{view!{<a href=config::folder_url(&media) target="_blank" rel="noreferrer">{string(&d,"mediaFolderName")}</a>}.into_any()}}</li></ul>{(!folder.is_empty()).then(||view!{<ShareLink folder/>})}<DrivePeople file_id/>
            <p class="text-xs text-gray-500">{if let Some(at)=d["syncedAt"].as_f64(){editor.t.fmt("gd_synced",&[("time",&js_sys::Date::new(&at.into()).to_locale_string("default",&js_sys::Object::new()).as_string().unwrap_or_default()),("name",d["syncedBy"].as_str().filter(|s|!s.is_empty()).unwrap_or("?"))])}else{editor.t.get("gd_never")}}</p>}
        })}
        <div class="flex flex-wrap gap-3"><a class=PRIMARY href=editor.link(Route::Questions)>{move ||editor.t.get("qe_open")}</a><button class=BUTTON disabled=move ||!signed.get()||editor.busy.get() on:click=move |_|run.run(())>{move ||editor.t.get(if syncing.get(){"gd_syncing"}else{"gd_sync"})}</button></div>
        <drive_parts::DriveProgress progress=editor.drive_progress/><Status editor/><p class="text-xs text-gray-500">{move ||editor.t.get("gd_offline_note")}</p>
    </Section></Show>}
}
#[component]
fn ShareLink(folder: String) -> impl IntoView {
    let t = use_t();
    let copied = RwSignal::new(false);
    let link = format!(
        "{}{}{}",
        window().location().origin().unwrap_or_default(),
        window().location().pathname().unwrap_or_default(),
        Route::Drive(Some(folder)).hash()
    );
    let copy = link.clone();
    view! {<div class="rounded-xl bg-black/30 p-4 space-y-2"><div class="flex gap-2"><input class=INPUT readonly prop:value=link/><button class=BUTTON on:click=move |_|{let text=copy.clone();spawn_local(async move{let navigator=window().navigator();if let Ok(clipboard)=js_sys::Reflect::get(&navigator,&"clipboard".into()) && let Ok(write)=js_sys::Reflect::get(&clipboard,&"writeText".into()).and_then(|v|v.dyn_into::<js_sys::Function>()) && let Ok(promise)=write.call1(&clipboard,&text.into()){if wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&promise)).await.is_ok(){copied.try_set(true);gloo_timers::callback::Timeout::new(2000,move ||{copied.try_set(false);}).forget();}}});}>{move ||t.get(if copied.get(){"gd_copied"}else{"gd_copy_link"})}</button></div><p class="text-xs text-gray-500">{move ||t.get("gd_share_help")}</p></div>}
}
/// Keep subfolders, but do not repeat the dropped media root in Drive.
fn upload_path(path: &str) -> String {
    let mut parts = path
        .split('/')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    if parts.len() > 1 && config::is_media_folder_name(parts[0]) {
        parts.remove(0);
    }
    parts.join("/")
}
pub(super) async fn upload(editor: Editor, files: Vec<web_sys::File>) -> Result<String, String> {
    editor.drive_uploading.set(true);
    let result = upload_inner(editor, files).await;
    editor.drive_progress.try_set(None);
    editor.drive_uploading.try_set(false);
    // Completed files must still be visible if a later upload failed.
    let loaded = editor.reload_media().await;
    match result {
        Ok(s) => {
            loaded?;
            Ok(s)
        }
        Err(e) => Err(e),
    }
}
async fn upload_folder(editor: Editor) -> Result<String, String> {
    let data = editor.data.get_value();
    let d = editor.quiz.get_untracked().drive.unwrap();
    let folder = if let Some(id) = d["mediaFolderId"].as_str() {
        id.to_string()
    } else {
        let folder = drive::ensure_media_folder(&d)
            .await
            .map_err(|e| report(editor, e))?;
        let id = string(&folder, "id");
        let saved = data
            .store
            .update_quiz_source(
                &data.id,
                move |q| {
                    if let Some(d) = q.drive.as_mut() {
                        d["mediaFolderId"] = folder["id"].clone();
                        d["mediaFolderName"] = folder["name"].clone();
                    }
                },
                "drive",
            )
            .await
            .map_err(|e| e.to_string())?;
        if let Some(q) = saved {
            editor.quiz.try_set(q);
        }
        id
    };
    Ok(folder)
}
pub(super) async fn upload_team_image(
    editor: Editor,
    name: &str,
    file: &web_sys::File,
) -> Result<Option<String>, String> {
    let folder = upload_folder(editor).await?;
    drive::upload_media(
        &editor.id(),
        &folder,
        name,
        file.as_ref(),
        None,
        Some(Rc::new(move |_| {
            js_sys::Promise::resolve(&wasm_bindgen::JsValue::from_bool(
                window()
                    .confirm_with_message(&editor.t.get("ed_drive_replace"))
                    .unwrap_or(false),
            ))
        })),
    )
    .await
    .map_err(|e| report(editor, e))
}
async fn upload_inner(editor: Editor, files: Vec<web_sys::File>) -> Result<String, String> {
    let files = files
        .into_iter()
        .filter(|f| quiz_store::package::media_type(&f.name()).is_some())
        .collect::<Vec<_>>();
    let data = editor.data.get_value();
    let folder = upload_folder(editor).await?;
    let mut p = drive::Progress {
        total: files.len(),
        total_bytes: files.iter().map(|f| f.size() as u64).sum(),
        ..Default::default()
    };
    editor.drive_progress.try_set(Some(p.clone()));
    let replace = Rc::new(RefCell::new(None));
    for file in files {
        let path = ["relativePath", "webkitRelativePath"]
            .into_iter()
            .find_map(|key| {
                js_sys::Reflect::get(&file, &key.into())
                    .ok()
                    .and_then(|v| v.as_string())
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_else(|| file.name());
        let before = p.bytes;
        let size = file.size() as u64;
        let progress = p.clone();
        let replace = replace.clone();
        drive::upload_media(
            &data.id,
            &folder,
            &upload_path(&path),
            file.as_ref(),
            Some(Rc::new(move |share| {
                let mut p = progress.clone();
                p.bytes = before + (size as f64 * share) as u64;
                editor.drive_progress.try_set(Some(p));
            })),
            Some(Rc::new(move |_| {
                let answer = *replace.borrow();
                let answer = answer.unwrap_or_else(|| {
                    window()
                        .confirm_with_message(&editor.t.get("ed_drive_replace"))
                        .unwrap_or(false)
                });
                *replace.borrow_mut() = Some(answer);
                js_sys::Promise::resolve(&wasm_bindgen::JsValue::from_bool(answer))
            })),
        )
        .await
        .map_err(|e| report(editor, e))?;
        p.done += 1;
        p.bytes += size;
        editor.drive_progress.try_set(Some(p.clone()));
    }
    Ok(editor
        .t
        .fmt("ed_drive_uploaded", &[("count", &p.done.to_string())]))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upload_paths_keep_nested_directories() {
        assert_eq!(upload_path("/Zdroje/round/a.png"), "round/a.png");
        assert_eq!(upload_path("source/a.png"), "a.png");
        assert_eq!(upload_path("round/a.png"), "round/a.png");
        assert_eq!(upload_path("a.png"), "a.png");
    }
}
