import { AlertTriangle, CheckCircle2 } from "lucide-react";

export const Section = ({ title, help, actions, children }) => (
  <section className="rounded-2xl bg-white/[0.04] border border-white/10 p-6 space-y-4">
    <div className="flex flex-wrap items-start gap-3">
      <div className="flex-1 min-w-[240px]">
        <h2 className="font-['League_Spartan'] text-xl font-bold uppercase tracking-widest">{title}</h2>
        {help && <p className="text-sm text-gray-400 mt-1">{help}</p>}
      </div>
      {actions && <div className="flex flex-wrap gap-2">{actions}</div>}
    </div>
    {children}
  </section>
);

// status: { ok: boolean, text } or null
export const Status = ({ status }) => status && (
  <p className={`flex items-start gap-2 text-sm ${status.ok ? "text-green-300" : "text-red-300"}`}>
    {status.ok ? <CheckCircle2 size={16} className="shrink-0 mt-0.5" /> : <AlertTriangle size={16} className="shrink-0 mt-0.5" />}
    {status.text}
  </p>
);

export const inputCls = "w-full bg-black/40 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-gray-600 focus:outline-none focus:border-yellow-500/60";
export const btnCls = "flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-white/10 hover:bg-white/20 text-sm whitespace-nowrap disabled:opacity-50";
