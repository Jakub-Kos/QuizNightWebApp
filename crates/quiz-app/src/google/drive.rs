use super::fetch::{Options, google_fetch, google_json, post};
use super::*;
pub use quiz_core::quiz::drive::{is_media_folder_name, parse_drive_link, sheet_url};
use quiz_core::quiz::model::Quiz;
use quiz_core::quiz::parse::{parse_questions_csv, parse_scores_csv};
use quiz_store::{Media, MediaStore, QuizStore};
use serde_json::json;
use std::{
    collections::{BTreeMap, HashSet},
    rc::Rc,
};
use wasm_bindgen::{JsCast, closure::Closure};
use wasm_bindgen_futures::JsFuture;
fn err(code: &str) -> Error {
    Error::Drive(code.into(), code.into())
}
pub fn to_drive_error(e: Error) -> Error {
    match e {
        e @ Error::Drive(..) => e,
        Error::Auth(_) => err("auth"),
        Error::Blocked(s) => Error::Drive("blocked".into(), s),
        Error::Api(403 | 404, _) => err("access"),
        e => Error::Drive("api".into(), e.to_string()),
    }
}
fn upload_error(e: Error) -> Error {
    if matches!(e, Error::Api(403, _)) {
        err("upload_denied")
    } else {
        to_drive_error(e)
    }
}
pub fn drive_error_text(e: &Error, t: &Value, email: Option<&str>) -> Option<String> {
    if let Error::Auth(s) = e {
        if s == "popup_closed" || s == "superseded" {
            return None;
        }
        return Some(string(
            t,
            if s == "scope" {
                "gd_err_scope"
            } else {
                "gd_err_signin"
            },
        ));
    }
    let code = match e {
        Error::Drive(c, _) => c.as_str(),
        Error::Blocked(_) => "blocked",
        _ => "api",
    };
    let key = match code {
        "link" => "link",
        "access" => "access",
        "no_sheet" => "no_sheet",
        "auth" => "auth",
        "readonly" => "readonly",
        "upload_denied" => "upload",
        "no_folder" => "no_folder",
        "bad_email" => "bad_email",
        "template" => "template",
        "blocked" => "blocked",
        _ => "api",
    };
    Some(
        string(t, &format!("gd_err_{key}"))
            .replace("{email}", email.unwrap_or("?"))
            .replace("{message}", &e.to_string()),
    )
}
fn query(pairs: &[(&str, String)]) -> String {
    config::form_query(pairs)
}
fn quoted(s: &str) -> String {
    format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'"))
}
async fn get_file(id: &str) -> Result<Value> {
    google_json(
        &format!(
            "{}/files/{id}?fields=id,name,mimeType,parents&supportsAllDrives=true",
            config::DRIVE_API
        ),
        Options::default(),
    )
    .await
}
async fn list_children(id: &str) -> Result<Vec<Value>> {
    let mut files = vec![];
    let mut page = String::new();
    loop {
        let mut pairs = vec![
            ("q", format!("'{id}' in parents and trashed = false")),
            (
                "fields",
                "nextPageToken,files(id,name,mimeType,modifiedTime,size)".into(),
            ),
            ("pageSize", "1000".into()),
            ("supportsAllDrives", "true".into()),
            ("includeItemsFromAllDrives", "true".into()),
        ];
        if !page.is_empty() {
            pairs.push(("pageToken", page));
        }
        let data = google_json(
            &format!("{}/files?{}", config::DRIVE_API, query(&pairs)),
            Options::default(),
        )
        .await?;
        files.extend(data["files"].as_array().cloned().unwrap_or_default());
        page = string(&data, "nextPageToken");
        if page.is_empty() {
            return Ok(files);
        }
    }
}
async fn find_quiz(link: &str) -> Result<Value> {
    let id = parse_drive_link(link).ok_or(err("link"))?;
    let item = get_file(&id).await?;
    let mut sheet = None;
    let folder = if item["mimeType"] == config::FOLDER {
        Some(item)
    } else if item["mimeType"] == config::SPREADSHEET {
        sheet = Some(item.clone());
        if let Some(p) = item["parents"][0].as_str() {
            get_file(p).await.ok()
        } else {
            None
        }
    } else {
        return Err(err("link"));
    };
    let children = if let Some(f) = &folder {
        list_children(&string(f, "id")).await?
    } else {
        vec![]
    };
    if sheet.is_none() {
        sheet = children
            .iter()
            .filter(|f| f["mimeType"] == config::SPREADSHEET)
            .max_by_key(|f| string(f, "modifiedTime"))
            .cloned();
    }
    let sheet = sheet.ok_or(err("no_sheet"))?;
    let media = children
        .iter()
        .find(|f| f["mimeType"] == config::FOLDER && is_media_folder_name(&string(f, "name")));
    Ok(
        json!({"folderId":folder.as_ref().map(|f|f["id"].clone()),"folderName":folder.as_ref().map(|f|f["name"].clone()).unwrap_or(sheet["name"].clone()),"spreadsheetId":sheet["id"],"spreadsheetName":sheet["name"],"mediaFolderId":media.map(|f|f["id"].clone()),"mediaFolderName":media.map(|f|f["name"].clone())}),
    )
}
pub async fn read_drive_quiz(link: &str) -> Result<Value> {
    async{let mut found=find_quiz(link).await?;let id=string(&found,"spreadsheetId");let tabs=sheets::sheet_tabs(&id).await?;let ranges=tabs.iter().map(|t|format!("ranges={}",enc(&sheets::tab_range(&t.title)))).collect::<Vec<_>>().join("&");let data=google_json(&format!("{}/{id}/values:batchGet?{ranges}",config::SHEETS_API),Options::default()).await?;let csv:Vec<_>=tabs.iter().enumerate().map(|(i,_)|sheets::rows_to_csv(&sheets::rows(&data["valueRanges"][i]["values"]))).collect();let scores=csv.iter().position(|c|parse_scores_csv(c).is_some());let questions=(0..tabs.len()).find(|&i|Some(i)!=scores&&!parse_questions_csv(&csv[i]).is_empty()).or_else(||(0..tabs.len()).find(|&i|Some(i)!=scores)).or_else(||if tabs.is_empty(){None}else{Some(0)}).ok_or(err("no_sheet"))?;found["questionsTab"]=json!(tabs[questions].title);found["scoresTab"]=scores.map(|i|json!(tabs[i].title)).unwrap_or(Value::Null);Ok(json!({"drive":found,"questionsCsv":csv[questions],"questionsSheetUrl":sheet_url(&id,Some(tabs[questions].sheet_id)),"scoresSheetUrl":scores.map(|i|sheet_url(&id,Some(tabs[i].sheet_id))).unwrap_or_default()}))}.await.map_err(to_drive_error)
}
async fn find_child(parent: &str, name: &str, folder: bool) -> Result<Option<Value>> {
    let q = format!(
        "{} in parents and name = {} and trashed = false and mimeType {} '{}'",
        quoted(parent),
        quoted(name),
        if folder { "=" } else { "!=" },
        config::FOLDER
    );
    let data = google_json(
        &format!(
            "{}/files?{}",
            config::DRIVE_API,
            query(&[
                ("q", q),
                ("fields", "files(id,name)".into()),
                ("supportsAllDrives", "true".into()),
                ("includeItemsFromAllDrives", "true".into())
            ])
        ),
        Options::default(),
    )
    .await?;
    Ok(data["files"][0]
        .as_object()
        .map(|_| data["files"][0].clone()))
}
async fn create_folder(parent: &str, name: &str) -> Result<Value> {
    post(
        &format!(
            "{}/files?supportsAllDrives=true&fields=id,name",
            config::DRIVE_API
        ),
        &json!({"name":name,"mimeType":config::FOLDER,"parents":[parent]}),
    )
    .await
}
pub async fn ensure_media_folder(drive: &Value) -> Result<Value> {
    async {
        if drive["mediaFolderId"].is_string() {
            return Ok(json!({"id":drive["mediaFolderId"],"name":drive["mediaFolderName"]}));
        }
        let parent = drive["folderId"].as_str().ok_or(err("no_folder"))?;
        match find_child(parent, "Zdroje", true).await? {
            Some(f) => Ok(f),
            None => create_folder(parent, "Zdroje").await,
        }
    }
    .await
    .map_err(upload_error)
}
pub async fn create_drive_quiz(
    title: &str,
    parent_link: Option<&str>,
    template_url: &str,
) -> Result<String> {
    let mut folder = None;
    let result = async {
        let parent = match parent_link.filter(|s| !s.trim().is_empty()) {
            Some(s) => parse_drive_link(s).ok_or(err("link"))?,
            None => "root".into(),
        };
        let template = parse_drive_link(template_url)
            .filter(|_| template_url.contains("/d/"))
            .ok_or(err("template"))?;
        let f = create_folder(&parent, title.trim()).await?;
        folder = Some(f.clone());
        let id = string(&f, "id");
        post(
            &format!(
                "{}/files/{template}/copy?supportsAllDrives=true&fields=id",
                config::DRIVE_API
            ),
            &json!({"name":title.trim(),"parents":[id]}),
        )
        .await
        .map_err(|e| {
            if matches!(e, Error::Auth(_) | Error::Blocked(_)) {
                e
            } else {
                Error::Drive("template".into(), e.to_string())
            }
        })?;
        create_folder(&id, "Zdroje").await?;
        Ok(id)
    }
    .await;
    if result.is_err() {
        if let Some(f) = folder {
            let _ = google_fetch(
                &format!(
                    "{}/files/{}?supportsAllDrives=true",
                    config::DRIVE_API,
                    string(&f, "id")
                ),
                Options::json("PATCH", &json!({"trashed":true})),
            )
            .await;
        }
    }
    result.map_err(upload_error)
}
pub async fn list_people(id: &str) -> Result<Value> {
    async{let list=google_json(&format!("{}/files/{id}/permissions?supportsAllDrives=true&fields=permissions(id,type,role,emailAddress,displayName,photoLink,domain,deleted)",config::DRIVE_API),Options::default()).await?;let file=google_json(&format!("{}/files/{id}?supportsAllDrives=true&fields=capabilities(canShare)",config::DRIVE_API),Options::default()).await?;let mut people=list["permissions"].as_array().cloned().unwrap_or_default();people.retain(|p|p["deleted"]!=true);people.sort_by_key(|p|(["owner","organizer","fileOrganizer","writer","commenter","reader"].iter().position(|r|p["role"]==*r).unwrap_or(9),p["displayName"].as_str().or(p["emailAddress"].as_str()).unwrap_or("").to_lowercase()));Ok(json!({"people":people,"canShare":file["capabilities"]["canShare"]==true}))}.await.map_err(to_drive_error)
}
pub async fn invite_person(id: &str, email: &str, role: &str, message: &str) -> Result<Value> {
    let mut params = vec![
        ("supportsAllDrives", "true".into()),
        ("sendNotificationEmail", "true".into()),
    ];
    if !message.is_empty() {
        params.push(("emailMessage", message.into()));
    }
    post(
        &format!(
            "{}/files/{id}/permissions?{}",
            config::DRIVE_API,
            query(&params)
        ),
        &json!({"type":"user","role":role,"emailAddress":email.trim()}),
    )
    .await
    .map_err(|e| {
        if matches!(e, Error::Api(400, _)) {
            err("bad_email")
        } else {
            to_drive_error(e)
        }
    })
}
pub async fn remove_person(id: &str, permission: &str) -> Result<()> {
    google_fetch(
        &format!(
            "{}/files/{id}/permissions/{permission}?supportsAllDrives=true",
            config::DRIVE_API
        ),
        Options {
            method: "DELETE".into(),
            ..Default::default()
        },
    )
    .await
    .map(|_| ())
    .map_err(to_drive_error)
}
/// XHR progress is a fraction in 0..1. Caller owns the callbacks until this future completes.
async fn send_file(
    url: &str,
    file: &web_sys::Blob,
    mime: &str,
    progress: Option<Rc<dyn Fn(f64)>>,
) -> Result<Value> {
    let xhr = web_sys::XmlHttpRequest::new()?;
    xhr.open("PUT", url)?;
    xhr.set_request_header("Content-Type", mime)?;
    let x = xhr.clone();
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let x = x.clone();
        let reject_http = reject.clone();
        let load = Closure::<dyn FnMut()>::once(move || {
            let status = x.status().unwrap_or(0);
            if (200..300).contains(&status) {
                match js_sys::JSON::parse(&x.response_text().ok().flatten().unwrap_or_default()) {
                    Ok(v) => {
                        let _ = resolve.call1(&JsValue::NULL, &v);
                    }
                    Err(e) => {
                        let _ = reject_http.call1(&JsValue::NULL, &e);
                    }
                }
            } else {
                let _ = reject_http.call1(&JsValue::NULL, &js(&json!({"status":status})));
            }
        });
        let error = Closure::<dyn FnMut()>::once(move || {
            let _ = reject.call1(&JsValue::NULL, &js(&json!({"status":0})));
        });
        xhr.set_onload(Some(load.as_ref().unchecked_ref()));
        xhr.set_onerror(Some(error.as_ref().unchecked_ref()));
        load.forget();
        error.forget();
    });
    let size = file.size();
    let cb = Closure::<dyn FnMut(web_sys::ProgressEvent)>::new(move |e: web_sys::ProgressEvent| {
        if let Some(p) = &progress {
            p(e.loaded()
                / if e.total() > 0. {
                    e.total()
                } else {
                    size.max(1.)
                });
        }
    });
    xhr.upload()?
        .set_onprogress(Some(cb.as_ref().unchecked_ref()));
    xhr.send_with_opt_blob(Some(file))?;
    let result = JsFuture::from(promise).await;
    xhr.set_onload(None);
    xhr.set_onerror(None);
    xhr.upload()?.set_onprogress(None);
    match result {
        Ok(v) => value(v),
        Err(e) => {
            let v = value(e).unwrap_or_default();
            let status = v["status"].as_u64().unwrap_or(0) as u16;
            Err(Error::Api(
                status,
                if status == 0 {
                    "network".into()
                } else {
                    format!("HTTP {status}")
                },
            ))
        }
    }
}
pub async fn upload_media(
    quiz_id: &str,
    media_folder: &str,
    path: &str,
    file: &web_sys::Blob,
    on_progress: Option<Rc<dyn Fn(f64)>>,
    confirm_replace: Option<Rc<dyn Fn(String) -> js_sys::Promise>>,
) -> Result<Option<String>> {
    async {
        let clean = path
            .split('/')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("/");
        let mut segments = clean.split('/').collect::<Vec<_>>();
        let name = segments.pop().unwrap_or_default();
        let mut parent = media_folder.to_owned();
        for s in segments {
            let f = match find_child(&parent, s, true).await? {
                Some(f) => f,
                None => create_folder(&parent, s).await?,
            };
            parent = string(&f, "id");
        }
        let existing = find_child(&parent, name, false).await?;
        if existing.is_some() {
            let yes = if let Some(confirm) = confirm_replace {
                JsFuture::from(confirm(clean.clone()))
                    .await?
                    .as_bool()
                    .unwrap_or(false)
            } else {
                false
            };
            if !yes {
                return Ok(None);
            }
        }
        let mime = if file.type_().is_empty() {
            "application/octet-stream".into()
        } else {
            file.type_()
        };
        let url = format!(
            "{}{}?uploadType=resumable&supportsAllDrives=true&fields=id,modifiedTime",
            config::UPLOAD_API,
            existing
                .as_ref()
                .map(|f| format!("/{}", string(f, "id")))
                .unwrap_or_default()
        );
        let body = if existing.is_some() {
            json!({})
        } else {
            json!({"name":name,"parents":[parent]})
        };
        let mut options = Options::json(if existing.is_some() { "PATCH" } else { "POST" }, &body);
        options.headers = vec![
            (
                "Content-Type".into(),
                "application/json; charset=UTF-8".into(),
            ),
            ("X-Upload-Content-Type".into(), mime.clone()),
            (
                "X-Upload-Content-Length".into(),
                (file.size() as u64).to_string(),
            ),
        ];
        let init = google_fetch(&url, options).await?;
        let session = init
            .headers()
            .get("Location")?
            .ok_or(Error::Drive("api".into(), "no upload session".into()))?;
        let uploaded = send_file(&session, file, &mime, on_progress).await?;
        let bytes = js_sys::Uint8Array::new(&JsFuture::from(file.array_buffer()).await?).to_vec();
        let fields = BTreeMap::from([
            ("driveId".into(), uploaded["id"].clone()),
            ("driveModified".into(), uploaded["modifiedTime"].clone()),
        ]);
        fetch::store()?
            .put_media(
                quiz_id,
                &[Media {
                    name: clean.clone(),
                    bytes,
                    content_type: mime,
                    fields,
                }],
            )
            .await?;
        Ok(Some(clean))
    }
    .await
    .map_err(upload_error)
}
#[derive(Debug, Clone, Default)]
pub struct Progress {
    pub done: usize,
    pub total: usize,
    pub bytes: u64,
    pub total_bytes: u64,
}
#[derive(Debug, Clone)]
pub struct MediaSync {
    pub downloaded: usize,
    pub kept: usize,
    pub removed: usize,
    pub duplicates: Vec<String>,
}
async fn media_tree(folder: &str) -> Result<Vec<Value>> {
    let mut queue = vec![(folder.to_owned(), String::new())];
    let mut out = vec![];
    while let Some((folder, prefix)) = queue.pop() {
        for f in list_children(&folder).await? {
            let path = format!("{prefix}{}", string(&f, "name"));
            if f["mimeType"] == config::FOLDER {
                queue.push((string(&f, "id"), format!("{path}/")));
            } else if quiz_store::package::media_type(&path).is_some() {
                out.push(json!({"id":f["id"],"path":path,"modified":f["modifiedTime"],"size":f["size"].as_str().and_then(|s|s.parse::<u64>().ok()).unwrap_or(0)}));
            }
        }
    }
    Ok(out)
}
pub async fn sync_drive_media(
    quiz_id: &str,
    folder: Option<&str>,
    on_progress: Option<Rc<dyn Fn(Progress)>>,
) -> Result<MediaSync> {
    async {
        let remote = if let Some(f) = folder {
            media_tree(f).await?
        } else {
            vec![]
        };
        let store = fetch::store()?;
        let local = store.list_media(quiz_id).await?;
        let paths: HashSet<_> = remote.iter().map(|f| string(f, "path")).collect();
        let keys: HashSet<_> = paths
            .iter()
            .map(|s| quiz_core::quiz::parse::media_key(s))
            .collect();
        let mut removed = 0;
        for m in &local {
            if !paths.contains(&m.name)
                && (m.fields.contains_key("driveId")
                    || keys.contains(&quiz_core::quiz::parse::media_key(&m.name)))
            {
                store.delete_media(quiz_id, &m.name).await?;
                removed += 1;
            }
        }
        let todo = remote
            .iter()
            .filter(|f| {
                !local.iter().any(|m| {
                    m.name == string(f, "path")
                        && m.fields.get("driveModified") == Some(&f["modified"])
                })
            })
            .collect::<Vec<_>>();
        let progress = Rc::new(std::cell::RefCell::new(Progress {
            total: todo.len(),
            total_bytes: todo.iter().map(|f| f["size"].as_u64().unwrap_or(0)).sum(),
            ..Default::default()
        }));
        if let Some(p) = &on_progress {
            p(progress.borrow().clone());
        }
        let queue = Rc::new(std::cell::RefCell::new(
            todo.iter()
                .map(|f| (*f).clone())
                .collect::<std::collections::VecDeque<_>>(),
        ));
        let failed = Rc::new(std::cell::RefCell::new(None::<Error>));
        let workers = js_sys::Array::new();
        for _ in 0..3 {
            let (queue, progress, failed, callback, store, quiz_id) = (
                queue.clone(),
                progress.clone(),
                failed.clone(),
                on_progress.clone(),
                store.clone(),
                quiz_id.to_owned(),
            );
            workers.push(&wasm_bindgen_futures::future_to_promise(async move {
                loop {
                    if failed.borrow().is_some() {
                        break;
                    }
                    let file = queue.borrow_mut().pop_front();
                    let Some(file) = file else {
                        break;
                    };
                    let result = async {
                        let res = google_fetch(
                            &format!(
                                "{}/files/{}?alt=media&supportsAllDrives=true",
                                config::DRIVE_API,
                                string(&file, "id")
                            ),
                            Options::default(),
                        )
                        .await?;
                        let mime = res.headers().get("content-type")?.unwrap_or_default();
                        let progress_bytes = progress.clone();
                        let byte_callback = callback.clone();
                        let bytes = download_bytes(res, move |n| {
                            progress_bytes.borrow_mut().bytes += n;
                            if let Some(p) = &byte_callback {
                                p(progress_bytes.borrow().clone());
                            }
                        })
                        .await?;
                        store
                            .put_media(
                                &quiz_id,
                                &[Media {
                                    name: string(&file, "path"),
                                    bytes,
                                    content_type: mime,
                                    fields: BTreeMap::from([
                                        ("driveId".into(), file["id"].clone()),
                                        ("driveModified".into(), file["modified"].clone()),
                                    ]),
                                }],
                            )
                            .await?;
                        Ok::<_, Error>(())
                    }
                    .await;
                    if let Err(e) = result {
                        if failed.borrow().is_none() {
                            *failed.borrow_mut() = Some(e);
                        }
                        break;
                    }
                    progress.borrow_mut().done += 1;
                    if let Some(p) = &callback {
                        p(progress.borrow().clone());
                    }
                }
                Ok(JsValue::UNDEFINED)
            }));
        }
        JsFuture::from(js_sys::Promise::all(&workers)).await?;
        if let Some(error) = failed.borrow_mut().take() {
            return Err(error);
        }
        let mut counts = BTreeMap::new();
        for f in &remote {
            *counts
                .entry(quiz_core::quiz::parse::media_key(&string(f, "path")))
                .or_insert(0) += 1;
        }
        Ok(MediaSync {
            downloaded: todo.len(),
            kept: remote.len() - todo.len(),
            removed,
            duplicates: counts
                .into_iter()
                .filter(|(_, n)| *n > 1)
                .map(|(k, _)| k)
                .collect(),
        })
    }
    .await
    .map_err(to_drive_error)
}

async fn download_bytes(res: web_sys::Response, on_bytes: impl Fn(u64)) -> Result<Vec<u8>> {
    let Some(body) = res.body() else {
        return Ok(js_sys::Uint8Array::new(&JsFuture::from(res.array_buffer()?).await?).to_vec());
    };
    let reader = js_sys::Reflect::get(&body, &"getReader".into())?
        .dyn_into::<js_sys::Function>()?
        .call0(&body)?;
    let read = js_sys::Reflect::get(&reader, &"read".into())?.dyn_into::<js_sys::Function>()?;
    let mut bytes = Vec::new();
    loop {
        let chunk = JsFuture::from(read.call0(&reader)?.dyn_into::<js_sys::Promise>()?).await?;
        if js_sys::Reflect::get(&chunk, &"done".into())?
            .as_bool()
            .unwrap_or(false)
        {
            break;
        }
        let data =
            js_sys::Uint8Array::new(&js_sys::Reflect::get(&chunk, &"value".into())?).to_vec();
        on_bytes(data.len() as u64);
        bytes.extend(data);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_mapping_and_translations() {
        assert_eq!(to_drive_error(Error::Auth("expired".into())), err("auth"));
        assert_eq!(
            to_drive_error(Error::Api(404, "missing".into())),
            err("access")
        );
        assert_eq!(
            to_drive_error(Error::Api(403, "denied".into())),
            err("access")
        );
        assert_eq!(
            to_drive_error(Error::Blocked("outside:x".into())),
            Error::Drive("blocked".into(), "outside:x".into())
        );
        assert_eq!(
            upload_error(Error::Api(403, "denied".into())),
            err("upload_denied")
        );
        assert_eq!(
            to_drive_error(Error::Api(500, "message".into())),
            Error::Drive("api".into(), "message".into())
        );
        let t = json!({"gd_err_scope":"scope","gd_err_signin":"signin","gd_err_access":"share with {email}","gd_err_api":"error {message}","gd_err_blocked":"blocked"});
        for code in ["popup_closed", "superseded"] {
            assert_eq!(drive_error_text(&Error::Auth(code.into()), &t, None), None);
        }
        assert_eq!(
            drive_error_text(&Error::Auth("scope".into()), &t, None),
            Some("scope".into())
        );
        assert_eq!(
            drive_error_text(&err("access"), &t, Some("a@b")),
            Some("share with a@b".into())
        );
        assert_eq!(
            drive_error_text(&Error::Other("oops".into()), &t, None),
            Some("error oops".into())
        );
        assert_eq!(
            drive_error_text(&Error::Blocked("outside:x".into()), &t, None),
            Some("blocked".into())
        );
    }
}

/// Ensure and persist the folder before uploading, so the write guard can authorize its files.
pub async fn upload_quiz_media(
    quiz: &Quiz,
    file: &web_sys::Blob,
    path: &str,
    progress: Option<Rc<dyn Fn(f64)>>,
    confirm: Option<Rc<dyn Fn(String) -> js_sys::Promise>>,
) -> Result<Option<String>> {
    let store = super::fetch::store()?;
    let mut drive = quiz
        .drive
        .clone()
        .ok_or_else(|| Error::Drive("link".into(), "link".into()))?;
    let folder = ensure_media_folder(&drive).await?;
    drive["mediaFolderId"] = folder["id"].clone();
    drive["mediaFolderName"] = folder["name"].clone();
    let folder_id = string(&folder, "id");
    store
        .update_quiz(&quiz.id, |q| {
            if let Some(d) = q.drive.as_mut() {
                d["mediaFolderId"] = drive["mediaFolderId"].clone();
                d["mediaFolderName"] = drive["mediaFolderName"].clone();
            }
        })
        .await?;
    upload_media(&quiz.id, &folder_id, path, file, progress, confirm).await
}
