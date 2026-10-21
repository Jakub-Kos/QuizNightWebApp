import { useState } from "react";
import { LogIn, LogOut, Loader2 } from "lucide-react";
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
