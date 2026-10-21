import { useState, useEffect, useCallback } from "react";
import { Users, UserPlus, Loader2, X, Globe, Info } from "lucide-react";
import { listPeople, invitePerson, removePerson, driveErrorText } from "../quiz/drive";
import { fmt } from "../quiz/files";
import { Status, inputCls, btnCls } from "./EditorParts";

const ROLE_KEY = { owner: "gd_role_owner", organizer: "gd_role_owner", fileOrganizer: "gd_role_editor", writer: "gd_role_editor", commenter: "gd_role_viewer", reader: "gd_role_viewer" };

// Who can open the quiz folder, with invite and remove for people allowed to share it.
// fileId: the folder (or the Sheet when the quiz has no folder); account from useGoogleAccount.
export default function DrivePeople({ fileId, account, t }) {
  const [state, setState] = useState(null); // { people, canShare } | { error }
  const [open, setOpen] = useState(false);
  const [email, setEmail] = useState("");
  const [role, setRole] = useState("writer");
  const [busy, setBusy] = useState(null); // "invite" or a permission id
  const [status, setStatus] = useState(null);
  const errorText = useCallback((err) => driveErrorText(err, t, account.user?.email), [t, account.user]);

  const load = useCallback(() => listPeople(fileId).then(setState, (err) => setState({ error: errorText(err) })), [fileId, errorText]);
  useEffect(() => {
    if (!account.signedIn) return;
    let cancelled = false;
    listPeople(fileId).then((s) => !cancelled && setState(s), (err) => !cancelled && setState({ error: errorText(err) }));
    return () => { cancelled = true; };
  }, [account.signedIn, fileId, errorText]);

  const invite = async (e) => {
    e.preventDefault();
    setBusy("invite");
    setStatus(null);
    try {
      await invitePerson(fileId, email, role, t.gd_invite_message);
      setStatus({ ok: true, text: fmt(t.gd_invited, { email: email.trim() }) });
      setEmail("");
      await load();
    } catch (err) {
      setStatus({ ok: false, text: errorText(err) });
    }
    setBusy(null);
  };

  const remove = async (person) => {
    if (!window.confirm(fmt(t.gd_remove_confirm, { name: person.displayName || person.emailAddress }))) return;
    setBusy(person.id);
    setStatus(null);
    try {
      await removePerson(fileId, person.id);
      await load();
    } catch (err) {
      setStatus({ ok: false, text: errorText(err) });
    }
    setBusy(null);
  };

  if (!account.signedIn || !state) return null;
  const people = state.people || [];
  const isMe = (p) => p.emailAddress && p.emailAddress === account.user?.email;

  return (
    <div className="rounded-xl bg-black/30 border border-white/5">
      <button onClick={() => setOpen((v) => !v)} className="w-full flex items-center gap-2 px-4 py-3 text-sm text-left">
        <Users size={16} className="text-gray-500" />
        <span className="font-bold">{t.gd_people}</span>
        {!state.error && <span className="text-gray-500">· {people.length}</span>}
        <span className="ml-auto text-gray-500 text-xs">{open ? t.gd_hide : t.gd_show}</span>
      </button>

      {open && (
        <div className="px-4 pb-4 space-y-4">
          {state.error ? <Status status={{ ok: false, text: state.error }} /> : (
            <ul className="divide-y divide-white/5">
              {people.map((p) => (
                <li key={p.id} className="flex items-center gap-3 py-2 text-sm">
                  {p.type === "anyone" || p.type === "domain" ? (
                    <span className="w-8 h-8 rounded-full bg-white/10 flex items-center justify-center text-gray-400"><Globe size={16} /></span>
                  ) : p.photoLink ? (
                    <img src={p.photoLink} alt="" referrerPolicy="no-referrer" className="w-8 h-8 rounded-full" />
                  ) : (
                    <span className="w-8 h-8 rounded-full bg-white/10 flex items-center justify-center text-gray-300 font-bold">{(p.displayName || p.emailAddress || "?")[0].toUpperCase()}</span>
                  )}
                  <span className="min-w-0 flex-1">
                    <span className="block truncate">
                      {p.type === "anyone" ? t.gd_anyone : p.type === "domain" ? fmt(t.gd_domain, { domain: p.domain }) : p.displayName || p.emailAddress}
                      {isMe(p) && <span className="text-gray-500"> ({t.gd_you})</span>}
                    </span>
                    {p.type === "user" && p.displayName && <span className="block text-xs text-gray-500 truncate">{p.emailAddress}</span>}
                  </span>
                  <span className="text-xs text-gray-400 whitespace-nowrap">{t[ROLE_KEY[p.role]] || p.role}</span>
                  {state.canShare && p.role !== "owner" && p.role !== "organizer" && !isMe(p) && (
                    <button onClick={() => remove(p)} disabled={Boolean(busy)} aria-label={t.gd_remove} title={t.gd_remove}
                      className="p-1.5 rounded-lg text-gray-500 hover:text-red-300 hover:bg-red-500/20">
                      {busy === p.id ? <Loader2 size={14} className="animate-spin" /> : <X size={14} />}
                    </button>
                  )}
                </li>
              ))}
            </ul>
          )}

          {state.canShare && (
            <form onSubmit={invite} className="flex flex-col sm:flex-row gap-2">
              <input type="email" required value={email} onChange={(e) => setEmail(e.target.value)} placeholder={t.gd_invite_email} className={`${inputCls} py-2.5`} />
              <select value={role} onChange={(e) => setRole(e.target.value)} className={`${inputCls} py-2.5 sm:w-auto`}>
                <option value="writer">{t.gd_role_editor}</option>
                <option value="reader">{t.gd_role_viewer}</option>
              </select>
              <button type="submit" disabled={busy === "invite" || !email.trim()} className={btnCls}>
                {busy === "invite" ? <Loader2 size={16} className="animate-spin" /> : <UserPlus size={16} />} {t.gd_invite}
              </button>
            </form>
          )}
          <Status status={status} />
          <p className="flex items-start gap-2 text-xs text-gray-500"><Info size={14} className="shrink-0 mt-px" /> {state.canShare ? t.gd_people_note : t.gd_people_readonly}</p>
        </div>
      )}
    </div>
  );
}
