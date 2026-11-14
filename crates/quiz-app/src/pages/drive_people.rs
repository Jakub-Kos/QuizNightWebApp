use super::drive_parts::{BUTTON, INPUT, error_text};
use crate::{
    google::{auth, drive},
    i18n::use_t,
};
use leptos::{prelude::*, task::spawn_local};
use serde_json::Value;
fn text(v: &Value, k: &str) -> String {
    v[k].as_str().unwrap_or_default().into()
}
fn role_key(role: &str) -> &str {
    match role {
        "owner" | "organizer" => "gd_role_owner",
        "writer" | "fileOrganizer" => "gd_role_editor",
        "reader" | "commenter" => "gd_role_viewer",
        _ => role,
    }
}
fn removable(p: &Value, me: &str, can_share: bool) -> bool {
    can_share
        && !matches!(p["role"].as_str(), Some("owner" | "organizer"))
        && (me.is_empty() || p["emailAddress"] != me)
}
#[component]
pub fn DrivePeople(file_id: String) -> impl IntoView {
    let t = use_t();
    let account = auth::use_google_account();
    let signed = account.signed_in;
    let session = account.session;
    let id = StoredValue::new(file_id);
    let refresh = RwSignal::new(0u32);
    let open = RwSignal::new(false);
    let email = RwSignal::new(String::new());
    let role = RwSignal::new("writer".to_string());
    let busy = RwSignal::new(false);
    let status = RwSignal::new(None::<(bool, String)>);
    let state = LocalResource::new(move || {
        refresh.get();
        let signed = signed.get();
        let id = id.get_value();
        async move {
            if signed {
                Some(drive::list_people(&id).await)
            } else {
                None
            }
        }
    });
    let invite = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        busy.set(true);
        status.set(None);
        let address = email.get_untracked();
        let role = role.get_untracked();
        spawn_local(async move {
            match drive::invite_person(
                &id.get_value(),
                &address,
                &role,
                &t.get("gd_invite_message"),
            )
            .await
            {
                Ok(_) => {
                    status.try_set(Some((
                        true,
                        t.fmt("gd_invited", &[("email", address.trim())]),
                    )));
                    email.try_set(String::new());
                    refresh.try_update(|n| *n += 1);
                }
                Err(e) => {
                    status.try_set(error_text(&e, t).map(|s| (false, s)));
                }
            }
            busy.try_set(false);
        });
    };
    view! { <Show when=move || signed.get()>{move || state.get().flatten().map(|result| {
        let (data,error)=match result { Ok(v)=>(v,None),Err(e)=>(Value::Null,error_text(&e,t)) };
        let can_share=data["canShare"]==true;let people=data["people"].as_array().cloned().unwrap_or_default();let count=people.len();
        view! { <div class="rounded-xl bg-black/30 border border-white/5"><button class="w-full flex gap-2 px-4 py-3 text-sm text-left" on:click=move |_| open.update(|v|*v=!*v)><b>{t.get("gd_people")}</b>{error.is_none().then(||format!(" · {count}"))}<span class="ml-auto text-gray-500">{move || t.get(if open.get(){"gd_hide"}else{"gd_show"})}</span></button>
        <div hidden=move || !open.get() class="px-4 pb-4 space-y-4">{error.map(|e|view!{<p class="text-red-300" role="alert">{e}</p>})}<ul>{people.into_iter().map(|p| {
            let me=session.get().map(|s|s.user.email).unwrap_or_default();let name=text(&p,"displayName");let address=text(&p,"emailAddress");let label=match p["type"].as_str(){Some("anyone")=>t.get("gd_anyone"),Some("domain")=>t.fmt("gd_domain",&[("domain",&text(&p,"domain"))]),_=>if name.is_empty(){address.clone()}else{name.clone()}};
            let remove=removable(&p,&me,can_share);let permission=text(&p,"id");let confirm=t.fmt("gd_remove_confirm",&[("name",&label)]);
            view!{<li class="flex items-center gap-3 py-2 text-sm">{p["photoLink"].as_str().map(|s|view!{<img src=s.to_string() alt="" referrerpolicy="no-referrer" class="w-8 h-8 rounded-full"/>})}<span class="flex-1 min-w-0"><span class="block truncate">{label}{(!me.is_empty()&&address==me).then(||format!(" ({})",t.get("gd_you")))}</span>{(!name.is_empty()&&p["type"]=="user").then(||view!{<span class="text-xs text-gray-500">{address}</span>})}</span><span class="text-xs text-gray-400">{t.get(role_key(p["role"].as_str().unwrap_or_default()))}</span>{remove.then(||view!{<button class=BUTTON aria-label=t.get("gd_remove") title=t.get("gd_remove") disabled=move || busy.get() on:click=move |_| { if !window().confirm_with_message(&confirm).unwrap_or(false){return;}let permission=permission.clone();busy.set(true);status.set(None);spawn_local(async move{if let Err(e)=drive::remove_person(&id.get_value(),&permission).await{status.try_set(error_text(&e,t).map(|s|(false,s)));}else{refresh.try_update(|n|*n+=1);}busy.try_set(false);});}>"×"</button>})}</li>}
        }).collect_view()}</ul>
        {can_share.then(||view!{<form on:submit=invite class="flex flex-col sm:flex-row gap-2"><input class=INPUT type="email" required placeholder=t.get("gd_invite_email") prop:value=move || email.get() on:input=move |e|email.set(event_target_value(&e))/><select class=INPUT prop:value=move ||role.get() on:change=move |e|role.set(event_target_value(&e))><option value="writer">{t.get("gd_role_editor")}</option><option value="reader">{t.get("gd_role_viewer")}</option></select><button class=BUTTON type="submit" disabled=move ||busy.get()||email.get().trim().is_empty()>{t.get("gd_invite")}</button></form>})}
        {move ||status.get().map(|(ok,s)|view!{<p role="status" class=if ok{"text-green-300 text-sm"}else{"text-red-300 text-sm"}>{s}</p>})}<p class="text-xs text-gray-500">{t.get(if can_share{"gd_people_note"}else{"gd_people_readonly"})}</p></div></div>}
    })}</Show> }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn permissions_protect_owner_and_self() {
        assert!(!removable(&json!({"role":"owner"}), "me", true));
        assert!(!removable(
            &json!({"role":"writer","emailAddress":"me"}),
            "me",
            true
        ));
        assert!(!removable(&json!({"role":"reader"}), "me", false));
        assert!(removable(&json!({"role":"reader"}), "me", true));
        assert_eq!(role_key("fileOrganizer"), "gd_role_editor");
    }
}
