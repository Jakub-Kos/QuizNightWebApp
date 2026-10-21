import { useState } from "react";
import { AlertTriangle, CheckCircle2, HelpCircle, ChevronDown } from "lucide-react";

// Which sections this viewer folded away, kept per browser
const FOLDED_KEY = "quiznight.foldedSections";
const readFolded = () => {
  try { return JSON.parse(localStorage.getItem(FOLDED_KEY)) || {}; } catch { return {}; }
};

// A setup page section that folds away by its title. id: remembers the folding; helpId: section of
// the help page (#/help/<helpId>) linked from the "?" next to the title; warning: what still needs
// doing, outlined in yellow; status: shown at the right of the title even when folded; busy: a
// running bar along the top (work being saved).
export function Section({ id, title, help, helpId, actions, warning, status, busy, children }) {
  const [open, setOpen] = useState(() => !id || !readFolded()[id]);
  const toggle = () => {
    const next = !open;
    setOpen(next);
    if (!id) return;
    try { localStorage.setItem(FOLDED_KEY, JSON.stringify({ ...readFolded(), [id]: !next })); } catch { /* not remembered */ }
  };

  return (
    <section className={`relative overflow-hidden rounded-2xl border p-6 space-y-4 ${warning ? "bg-yellow-500/[0.04] border-yellow-500/40" : "bg-white/[0.04] border-white/10"}`}>
      {busy && <div className="absolute inset-x-0 top-0 h-0.5 overflow-hidden"><div className="h-full w-2/5 bg-yellow-400 progress-indeterminate" /></div>}
      <div className="flex flex-wrap items-start gap-3">
        <div className="flex-1 min-w-[240px]">
          <h2 className="flex flex-wrap items-center gap-2 font-['League_Spartan'] text-xl font-bold uppercase tracking-widest">
            <button type="button" onClick={toggle} aria-expanded={open} className="flex items-center gap-2 text-left uppercase hover:text-yellow-400">
              <ChevronDown size={18} className={`shrink-0 text-gray-500 transition-transform ${open ? "" : "-rotate-90"}`} />
              {title}
            </button>
            {helpId && <a href={`#/help/${helpId}`} className="text-gray-500 hover:text-yellow-400" aria-label="?"><HelpCircle size={18} /></a>}
            {warning && !open && <AlertTriangle size={16} className="text-yellow-400" />}
          </h2>
          {open && help && <p className="text-sm text-gray-400 mt-1">{help}</p>}
        </div>
        {status && <div className="flex items-center gap-2">{status}</div>}
        {open && actions && <div className="flex flex-wrap gap-2">{actions}</div>}
      </div>
      {open && warning && (
        <p className="flex items-start gap-2 text-sm text-yellow-200"><AlertTriangle size={16} className="shrink-0 mt-0.5 text-yellow-400" /> {warning}</p>
      )}
      {open && children}
    </section>
  );
}

// status: { ok: boolean, text } or null
export const Status = ({ status }) => status && (
  <p className={`flex items-start gap-2 text-sm ${status.ok ? "text-green-300" : "text-red-300"}`}>
    {status.ok ? <CheckCircle2 size={16} className="shrink-0 mt-0.5" /> : <AlertTriangle size={16} className="shrink-0 mt-0.5" />}
    {status.text}
  </p>
);

export const inputCls = "w-full bg-black/40 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-gray-600 focus:outline-none focus:border-yellow-500/60";
export const btnCls = "flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-white/10 hover:bg-white/20 text-sm whitespace-nowrap disabled:opacity-50";
