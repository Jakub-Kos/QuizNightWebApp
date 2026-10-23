import { CheckCircle2, AlertTriangle, AlertCircle, Play } from "lucide-react";
import { fmt } from "../quiz/files";

const STATES = {
  done: { icon: CheckCircle2, color: "text-green-400", box: "border-green-500/20 hover:border-green-500/40" },
  todo: { icon: AlertTriangle, color: "text-yellow-400", box: "border-yellow-500/40 bg-yellow-500/[0.05] hover:border-yellow-500/70" },
  error: { icon: AlertCircle, color: "text-red-400", box: "border-red-500/50 bg-red-500/[0.06] hover:border-red-500/80" },
};

// "Ready for the show" at the top of the setup page: one step per thing the show needs, each going to
// where it is done. steps: [{ key, label, status, state: "done" | "todo" | "error", onClick }]
export default function ReadyChecklist({ steps, onStart, t }) {
  const done = steps.filter((s) => s.state === "done").length;
  const allDone = done === steps.length;

  return (
    <section className="rounded-2xl bg-white/[0.04] border border-white/10 p-5 space-y-4">
      <div className="flex flex-wrap items-center gap-x-4 gap-y-3">
        <div className="flex-1 min-w-[200px] space-y-2">
          <h2 className="font-['League_Spartan'] text-xl font-bold uppercase tracking-widest">{t.rd_title}</h2>
          <div className="flex items-center gap-3">
            <div className="h-1.5 flex-1 max-w-56 rounded-full bg-white/10 overflow-hidden">
              <div className={`h-full rounded-full transition-all ${allDone ? "bg-green-500" : "bg-yellow-500"}`} style={{ width: `${(done / steps.length) * 100}%` }} />
            </div>
            <span className="text-sm text-gray-400">{allDone ? t.rd_all_done : fmt(t.rd_progress, { done, total: steps.length })}</span>
          </div>
        </div>
        <button onClick={onStart}
          className={`flex items-center gap-2 px-5 py-3 rounded-xl font-bold ${allDone ? "bg-yellow-500 hover:bg-yellow-400 text-black" : "bg-white/10 hover:bg-white/20"}`}>
          <Play size={18} /> {t.sl_title}
        </button>
      </div>
      <ol className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-2">
        {steps.map((s) => {
          const { icon: Icon, color, box } = STATES[s.state];
          return (
            <li key={s.key}>
              <button onClick={s.onClick} className={`w-full h-full text-left rounded-xl border p-3 transition-colors ${box}`}>
                <span className="flex items-center gap-2 font-bold text-sm"><Icon size={16} className={`shrink-0 ${color}`} /> {s.label}</span>
                <span className="block text-xs text-gray-400 mt-1 pl-6">{s.status}</span>
              </button>
            </li>
          );
        })}
      </ol>
    </section>
  );
}
