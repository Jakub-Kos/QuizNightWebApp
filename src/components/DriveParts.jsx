import { useState } from "react";
import { LogIn, LogOut, Loader2, RefreshCw, CheckCircle2, AlertTriangle } from "lucide-react";
import { driveErrorText } from "../quiz/drive";
import { fmt, formatBytes } from "../quiz/files";
import { btnCls } from "./EditorParts";

// Who is signed in, with sign-in / sign-out; account from useGoogleAccount
export function GoogleAccountBar({ account, t, onError }) {
  const [busy, setBusy] = useState(false);
  const { user, signedIn } = account;

  const handleSignIn = async () => {
    setBusy(true);
    try { await account.signIn(); } catch (err) { onError?.(driveErrorText(err, t, user?.email)); }
    setBusy(false);
  };

  return (
    <div className="flex flex-wrap items-center gap-3 text-sm">
      {user && (
        <span className="flex items-center gap-2 min-w-0">
          {user.photo && <img src={user.photo} alt="" referrerPolicy="no-referrer" className="w-7 h-7 rounded-full" />}
          <span className="truncate">
            {signedIn ? fmt(t.gd_signed_in_as, { name: user.name }) : fmt(t.gd_expired_as, { name: user.name })}
            <span className="text-gray-500"> · {user.email}</span>
          </span>
        </span>
      )}
      {!signedIn && (
        <button onClick={handleSignIn} disabled={busy} className={`${btnCls} bg-white text-black hover:bg-gray-200 font-bold`}>
          {busy ? <Loader2 size={16} className="animate-spin" /> : <LogIn size={16} />}
          {user ? t.gd_sign_in_again : t.gd_sign_in}
        </button>
      )}
      {user && (
        <button onClick={account.signOut} className="flex items-center gap-1.5 text-gray-500 hover:text-white ml-auto">
          <LogOut size={14} /> {t.gd_sign_out}
        </button>
      )}
    </div>
  );
}

// progress from syncDriveMedia
export function DriveProgress({ progress, t }) {
  const share = progress.totalBytes ? progress.bytes / progress.totalBytes : progress.total ? progress.done / progress.total : 0;
  return (
    <div className="space-y-1.5">
      <div className="h-2 rounded-full bg-white/10 overflow-hidden">
        <div className="h-full bg-yellow-500 transition-[width]" style={{ width: `${Math.min(100, share * 100)}%` }} />
      </div>
      <p className="text-xs text-gray-400">
        {fmt(t.gd_downloading, { done: progress.done, total: progress.total, size: `${formatBytes(progress.bytes)} / ${formatBytes(progress.totalBytes)}` })}
      </p>
    </div>
  );
}

// The Drive features are a private beta for invited accounts
export const BetaBadge = ({ t }) => (
  <span className="text-[10px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded bg-sky-500/15 text-sky-300 font-sans">
    {t.gd_beta}
  </span>
);

// Whether a section's changes are saved: in this browser, and for Drive quizzes also in the folder
// (sync from useDriveSync), with a button that loads the latest version from Drive
export function SaveStatus({ saving, drive, sync, onRefresh, t }) {
  const time = (ms) => new Date(ms).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  let state;
  if (!drive) state = saving ? [Loader2, t.ed_saving, "text-gray-400", true] : [CheckCircle2, t.ed_saved, "text-gray-500"];
  else if (saving || sync.phase === "pending" || sync.phase === "syncing") state = [Loader2, t.ed_saving_drive, "text-yellow-300", true];
  else if (sync.phase === "error") state = [AlertTriangle, t.ed_save_drive_failed, "text-red-300"];
  else if (sync.at) state = [CheckCircle2, fmt(t.ed_saved_drive, { time: time(sync.at) }), "text-green-300/80"];
  else state = [CheckCircle2, t.ed_saved, "text-gray-500"];
  const [Icon, text, color, spin] = state;
  return (
    <span className="flex items-center gap-2 text-xs font-sans normal-case tracking-normal">
      <span className={`flex items-center gap-1.5 ${color}`}><Icon size={14} className={spin ? "animate-spin" : ""} /> {text}</span>
      {drive && onRefresh && (
        <button type="button" onClick={onRefresh} disabled={sync.phase === "syncing"} title={t.ed_refresh_drive} aria-label={t.ed_refresh_drive}
          className="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-white/10 disabled:opacity-40">
          <RefreshCw size={14} className={sync.phase === "syncing" ? "animate-spin" : ""} />
        </button>
      )}
    </span>
  );
}
