import { useState, useEffect } from "react";
import { Tv, Laptop, X, CheckCircle2, AlertTriangle, Play, Compass } from "lucide-react";
import { DEMO_ID } from "../quiz/model";
import { startTour } from "../quiz/tour";
import { quizHash, navigate } from "../hooks/useHashRoute";
import Rich from "./RichText";

const showUrl = (quizId, view) => window.location.origin + window.location.pathname + quizHash(quizId, view);

// The TV window goes straight onto the second screen when the browser can tell where it is (Chrome's
// window management, asked for once); otherwise it opens here and is dragged to the TV
async function openTvWindow(url) {
  let features = "popup,width=1280,height=720";
  let placed = false;
  try {
    if (window.screen.isExtended && "getScreenDetails" in window) {
      const details = await window.getScreenDetails();
      const other = details.screens.find((s) => s !== details.currentScreen);
      if (other) {
        features = `popup,left=${other.availLeft},top=${other.availTop},width=${other.availWidth},height=${other.availHeight}`;
        placed = true;
      }
    }
  } catch { /* not allowed: opened on this screen */ }
  const win = window.open(url, "QuizNightTV", features);
  return win ? (placed ? "screen" : "opened") : "blocked";
}

// "Start the show": opens the TV window for the audience and the presenter window that controls it,
// so nothing meant for the host ever has to appear on the TV
export default function ShowLauncher({ quizId, onClose, t }) {
  const [tv, setTv] = useState(null); // null | "opened" | "screen" | "blocked"
  const [presenter, setPresenter] = useState(false);

  useEffect(() => {
    const onKey = (e) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const openTv = async () => setTv(await openTvWindow(showUrl(quizId)));
  const openPresenter = () => {
    // The same window name as Shift+P, so the two never open two presenter windows
    window.open(showUrl(quizId, "presenter"), "PresenterWindow", "width=1200,height=800");
    setPresenter(true);
  };

  const tvText = { opened: t.sl_tv_opened, screen: t.sl_tv_opened_screen, blocked: t.sl_tv_blocked }[tv] || t.sl_tv_text;
  const step = (n, done, icon, title, text, button, onClick, warn) => (
    <div className={`rounded-2xl border p-5 space-y-3 ${done ? "border-green-500/30 bg-green-500/[0.04]" : "border-white/10 bg-white/[0.03]"}`}>
      <div className="flex items-center gap-3">
        <span className={`w-8 h-8 shrink-0 rounded-full flex items-center justify-center font-bold ${done ? "bg-green-500 text-black" : "bg-white/10"}`}>
          {done ? <CheckCircle2 size={18} /> : n}
        </span>
        <h3 className="font-bold text-lg">{title}</h3>
      </div>
      <p className={`text-sm flex items-start gap-2 ${warn ? "text-yellow-200" : "text-gray-300"}`}>
        {warn && <AlertTriangle size={16} className="shrink-0 mt-0.5 text-yellow-400" />} {text}
      </p>
      <button onClick={onClick}
        className={`flex items-center gap-2 px-5 py-3 rounded-xl font-bold ${done ? "bg-white/10 hover:bg-white/20" : "bg-yellow-500 hover:bg-yellow-400 text-black"}`}>
        {icon} {button}
      </button>
    </div>
  );

  return (
    <div className="fixed inset-0 z-50 bg-black/85 backdrop-blur-sm flex items-center justify-center p-4 font-sans text-white" onClick={onClose}>
      <div className="w-full max-w-xl max-h-full overflow-y-auto rounded-3xl bg-[#0f0f0f] border border-white/10 shadow-2xl p-6 sm:p-8 space-y-5" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-start gap-4">
          <div className="flex-1 space-y-2">
            <h2 className="font-['League_Spartan'] text-3xl font-black uppercase tracking-wide text-yellow-400">{t.sl_title}</h2>
            <p className="text-gray-300">{t.sl_intro}</p>
          </div>
          <button onClick={onClose} className="p-2 rounded-full text-gray-400 hover:text-white hover:bg-white/10" aria-label={t.chk_close}><X size={20} /></button>
        </div>

        {step(1, tv === "opened" || tv === "screen", <Tv size={18} />, t.sl_tv_title, tvText, tv && tv !== "blocked" ? t.sl_tv_again : t.sl_tv_open, openTv, tv === "blocked")}
        {step(2, presenter, <Laptop size={18} />, t.sl_presenter_title, t.sl_presenter_text, t.sl_presenter_open, openPresenter)}

        <p className="text-sm text-gray-400"><Rich text={t.sl_keys} /></p>
        <div className="pt-4 border-t border-white/5 flex flex-wrap gap-x-6 gap-y-2">
          <button onClick={() => navigate(quizHash(quizId))} className="flex items-center gap-2 text-sm text-gray-400 hover:text-white">
            <Play size={16} /> {t.sl_one_screen}
          </button>
          {quizId === DEMO_ID && (
            <button onClick={() => startTour(quizId)} className="flex items-center gap-2 text-sm text-yellow-300 hover:text-yellow-200">
              <Compass size={16} /> {t.tr_start}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
