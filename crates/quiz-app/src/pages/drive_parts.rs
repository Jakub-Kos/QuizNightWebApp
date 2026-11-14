//! Shared Google account, sync status and media progress UI.
use crate::{
    google::{self, auth, drive, sync},
    i18n::{T, use_t},
};
use leptos::{prelude::*, task::spawn_local};

pub const BUTTON: &str = "inline-flex items-center justify-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm disabled:opacity-50";
pub const INPUT: &str = "w-full bg-black/40 border border-white/10 rounded-xl px-4 py-3 text-white";
pub fn translations(t: T) -> serde_json::Value {
    serde_json::from_str(match t.lang() {
        crate::i18n::Lang::En => include_str!("../../i18n/en.json"),
        crate::i18n::Lang::Cs => include_str!("../../i18n/cs.json"),
        crate::i18n::Lang::Sk => include_str!("../../i18n/sk.json"),
    })
    .unwrap()
}
pub fn error_text(error: &google::Error, t: T) -> Option<String> {
    let email = auth::get_session().map(|s| s.user.email);
    drive::drive_error_text(error, &translations(t), email.as_deref())
}
#[component]
pub fn DriveIcon() -> impl IntoView {
    view! { <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="M3 6h18v14H3ZM7 3h10v3M6 16h.01M10 16h8"/></svg> }
}
#[component]
fn StateIcon(#[prop(default = "check")] name: &'static str) -> impl IntoView {
    let path = match name {
        "busy" => "M22 12a10 10 0 1 1-7-9.5",
        "warning" => "m12 3 10 18H2Zm0 6v5m0 3v1",
        "signin" => "M9 5V3h12v18H9v-2M3 12h12m-5-5 5 5-5 5",
        "signout" => "M9 5V3H3v18h6v-2M9 12h12m-5-5 5 5-5 5",
        _ => "m5 12 4 4L19 6",
    };
    view! { <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" class=if name == "busy" { "animate-spin shrink-0" } else { "shrink-0" } aria-hidden="true"><path d=path/></svg> }
}
#[component]
pub fn BetaBadge() -> impl IntoView {
    let t = use_t();
    view! { <span class="text-[10px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded bg-sky-500/15 text-sky-300">{move || t.get("gd_beta")}</span> }
}
#[component]
pub fn GoogleAccountBar(on_error: Callback<Option<String>>) -> impl IntoView {
    let t = use_t();
    let account = auth::use_google_account();
    let session = account.session;
    let signed = account.signed_in;
    let busy = RwSignal::new(false);
    view! { <div class="flex flex-wrap items-center gap-3 text-sm">
        {move || session.get().map(|s| view! { <span class="flex items-center gap-2 min-w-0">{s.user.photo.map(|photo| view! { <img src=photo alt="" referrerpolicy="no-referrer" class="w-7 h-7 rounded-full"/> })}<span>{t.fmt(if signed.get() { "gd_signed_in_as" } else { "gd_expired_as" }, &[("name", &s.user.name)])}<span class="text-gray-500">" · "{s.user.email}</span></span></span> })}
        <Show when=move || !signed.get()><button class=BUTTON disabled=move || busy.get() on:click=move |_| { busy.set(true); spawn_local(async move { if let Err(e) = auth::sign_in(false).await { on_error.run(error_text(&e,t)); } busy.try_set(false); }); }>{move || view!{<StateIcon name=if busy.get(){"busy"}else{"signin"}/>}}{move || t.get(if session.get().is_some() { "gd_sign_in_again" } else { "gd_sign_in" })}</button></Show>
        <Show when=move || session.get().is_some()><button class="ml-auto text-gray-500 hover:text-white" on:click=move |_| auth::sign_out()><StateIcon name="signout"/>{move || t.get("gd_sign_out")}</button></Show>
    </div> }
}
pub fn progress_share(p: &drive::Progress) -> f64 {
    if p.total_bytes != 0 {
        (p.bytes as f64 / p.total_bytes as f64).clamp(0., 1.)
    } else if p.total != 0 {
        (p.done as f64 / p.total as f64).clamp(0., 1.)
    } else {
        0.
    }
}
fn size(n: u64) -> String {
    if n < 1024 {
        format!("{n} B")
    } else if n < 1024 * 1024 {
        format!("{:.1} KB", n as f64 / 1024.)
    } else {
        format!("{:.1} MB", n as f64 / 1048576.)
    }
}
#[component]
pub fn DriveProgress(
    progress: RwSignal<Option<drive::Progress>>,
    #[prop(default = false)] uploading: bool,
) -> impl IntoView {
    let t = use_t();
    view! { {move || progress.get().map(|p| view! { <div class="space-y-2" role="status"><div class="h-2 rounded-full bg-white/10 overflow-hidden"><div class="h-full bg-yellow-500" style:width=format!("{}%",progress_share(&p)*100.)/></div><p class="text-xs text-gray-400">{t.fmt(if uploading { "ed_drive_uploading" } else { "gd_downloading" }, &[("done",&p.done.to_string()),("total",&p.total.to_string()),("size",&format!("{} / {}",size(p.bytes),size(p.total_bytes)))])}</p></div> })} }
}
#[component]
pub fn SaveStatus(
    saving: Signal<bool>,
    drive: Signal<bool>,
    state: RwSignal<sync::SyncState>,
    #[prop(optional)] on_refresh: Option<Callback<()>>,
) -> impl IntoView {
    let t = use_t();
    let signed = auth::use_google_account().signed_in;
    view! { <span class=move ||{let s=state.get();if drive.get() && (saving.get() || matches!(s.phase,sync::Phase::Pending|sync::Phase::Syncing)){"flex items-center gap-2 text-xs text-yellow-300"}else if drive.get() && s.phase==sync::Phase::Error{"flex items-center gap-2 text-xs text-red-300"}else if drive.get() && s.at.is_some(){"flex items-center gap-2 text-xs text-green-300"}else{"flex items-center gap-2 text-xs text-gray-400"}} aria-live="polite">{move ||{let s=state.get();view!{<StateIcon name=if saving.get() || (drive.get() && matches!(s.phase,sync::Phase::Pending|sync::Phase::Syncing)){"busy"}else if drive.get() && s.phase==sync::Phase::Error{"warning"}else{"check"}/>}}}{move || {
        let s=state.get();
        if drive.get() && (saving.get() || matches!(s.phase,sync::Phase::Pending|sync::Phase::Syncing)) { t.get("ed_saving_drive") }
        else if drive.get() && s.phase == sync::Phase::Error { t.get("ed_save_drive_failed") }
        else if drive.get() && s.at.is_some() { let date=js_sys::Date::new(&s.at.unwrap().into()); t.fmt("ed_saved_drive",&[("time",&date.to_locale_time_string("default").as_string().unwrap_or_default())]) }
        else { t.get(if saving.get() { "ed_saving" } else { "ed_saved" }) }
    }}{on_refresh.map(|refresh| view! { <Show when=move ||drive.get() && signed.get()><button class=BUTTON title=move || t.get("ed_refresh_drive") aria-label=move || t.get("ed_refresh_drive") disabled=move || state.get().phase == sync::Phase::Syncing on:click=move |_| refresh.run(())>"↻"</button></Show> })}</span> }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn progress_handles_unknown_sizes_and_clamps() {
        let mut p = drive::Progress::default();
        assert_eq!(progress_share(&p), 0.);
        p.done = 1;
        p.total = 4;
        assert_eq!(progress_share(&p), 0.25);
        p.bytes = 75;
        p.total_bytes = 100;
        assert_eq!(progress_share(&p), 0.75);
        p.bytes = 150;
        assert_eq!(progress_share(&p), 1.);
    }
}
