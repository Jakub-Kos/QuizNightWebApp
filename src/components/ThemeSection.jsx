import { useEffect, useMemo, useRef, useState } from "react";
import { Check, Upload, Download, Loader2, Pencil, Sparkles, X, Palette } from "lucide-react";
import { TRANSLATIONS } from "../data/translations";
import { QuizContext } from "../quiz/context";
import { parseQuestionsCsv } from "../quiz/parse";
import { themeChoices, themeForQuiz, themeName, CUSTOM_THEME_ID } from "../quiz/themes";
import { useDisplaySettings } from "../hooks/useDisplay";
import { useUiLanguage } from "../hooks/useUiLanguage";
import { fmt } from "../quiz/files";
import { listLibraryThemes, deleteLibraryTheme } from "../quiz/storage";
import { quizHash, navigate } from "../hooks/useHashRoute";
import { Section, btnCls } from "./EditorParts";
import { StageFrame } from "./Stage";
import { ThemeDecorations } from "./Theme";
import Dashboard from "./Dashboard";

const noop = () => {};

// The show's look (quiz/themes.js): a card per theme with the round overview as it would appear, the
// quiz's own theme among them once one is imported or made in the theme editor, and My themes (saved in
// this browser) to use here. onImport(files) and onUseLibrary(entry) resolve with { ok, text, problems }.
// resolveMedia and onExport come from the setup page, which holds the quiz's media.
export default function ThemeSection({ quiz, onChange, onImport, onUseLibrary, onExport, resolveMedia, warning, t }) {
  const [display] = useDisplaySettings("main");
  const [lang] = useUiLanguage();
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState(null);
  const input = useRef(null);
  const showT = TRANSLATIONS[quiz.settings?.language] || TRANSLATIONS.en;
  const rounds = useMemo(() => parseQuestionsCsv(quiz.questionsCsv), [quiz.questionsCsv]);
  const context = useMemo(() => ({ quiz, rounds, teams: [], resolveMedia }), [quiz, rounds, resolveMedia]);
  const current = quiz.settings?.theme ?? null; // none yet: nothing is marked as chosen
  const shown = themeForQuiz(quiz);
  const [library, setLibrary] = useState([]);
  useEffect(() => { listLibraryThemes().then(setLibrary).catch(() => {}); }, []);

  // A new own theme starts from the one shown; it replaces an existing own theme, so that is asked first
  const newTheme = () => {
    if (quiz.customTheme && !window.confirm(fmt(t.thm_new_confirm, { name: themeName(quiz.customTheme, lang) }))) return;
    navigate(quizHash(quiz.id, "theme/new"));
  };
  const applyEntry = async (entry) => {
    if (quiz.customTheme && !window.confirm(fmt(t.thm_new_confirm, { name: themeName(quiz.customTheme, lang) }))) return;
    setBusy(true);
    setStatus(await onUseLibrary(entry));
    setBusy(false);
  };
  const forget = async (entry) => {
    if (!window.confirm(fmt(t.thm_lib_delete_confirm, { name: themeName(entry.theme, lang) }))) return;
    await deleteLibraryTheme(entry.id);
    setLibrary(await listLibraryThemes());
  };

  const importFiles = async (files) => {
    if (!files.length) return;
    setBusy(true);
    setStatus(await onImport(files));
    setBusy(false);
  };

  return (
    <Section id="theme" warning={warning} title={t.ed_theme} help={t.ed_theme_help}>
      <QuizContext.Provider value={context}>
        <div className={`grid gap-4 ${quiz.customTheme ? "sm:grid-cols-2 lg:grid-cols-4" : "sm:grid-cols-3"}`}>
          {themeChoices(quiz).map(({ key, theme }) => {
            const chosen = key === current;
            return (
              // Not a <button>: the preview inside has buttons of its own
              <div key={key} role="button" tabIndex={0} aria-pressed={chosen} onClick={() => onChange(key)}
                onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onChange(key); } }}
                className={`group text-left rounded-xl p-2 border cursor-pointer transition-all ${chosen ? "border-yellow-500 bg-yellow-500/10" : "border-white/10 hover:border-white/30 bg-black/20"}`}>
                <StageFrame display={display} theme={theme} className="rounded-lg pointer-events-none">
                  <Dashboard rounds={rounds} onSelectRound={noop} onOpenLeaderboard={noop} onOpenPause={noop} t={showT} />
                  <ThemeDecorations />
                </StageFrame>
                <span className="flex items-center gap-2 px-1 pt-2 font-bold text-white min-w-0">
                  {chosen && <Check size={16} className="shrink-0 text-yellow-400" />}
                  <span className="truncate">{themeName(theme, lang)}</span>
                  {key === CUSTOM_THEME_ID && <span className="shrink-0 text-[10px] uppercase tracking-wider px-1.5 py-0.5 rounded bg-white/10 text-gray-300">{t.thm_own}</span>}
                </span>
                {key === CUSTOM_THEME_ID && theme.author && <span className="block px-1 text-xs text-gray-500 truncate">{fmt(t.thm_by, { author: theme.author })}</span>}
              </div>
            );
          })}
        </div>
      </QuizContext.Provider>

      <div className="flex flex-wrap items-center gap-2 mt-4">
        {quiz.customTheme && (
          <a href={quizHash(quiz.id, "theme")} className={btnCls}><Pencil size={16} /> {t.thm_edit}</a>
        )}
        {shown !== quiz.customTheme && (
          <button type="button" className={btnCls} onClick={newTheme}><Sparkles size={16} /> {fmt(t.thm_new, { name: themeName(shown, lang) })}</button>
        )}
        <button type="button" className={btnCls} disabled={busy} onClick={() => input.current?.click()}>
          {busy ? <Loader2 size={16} className="animate-spin" /> : <Upload size={16} />} {t.thm_import}
        </button>
        <button type="button" className={btnCls} disabled={busy} onClick={() => onExport(shown)}>
          <Download size={16} /> {fmt(t.thm_export, { name: themeName(shown, lang) })}
        </button>
        <a href="#/themes" className={btnCls}><Palette size={16} /> {t.thm_browse}</a>
        <input ref={input} type="file" accept=".zip,.json,application/zip,application/json" className="hidden"
          onChange={(e) => { importFiles([...e.target.files]); e.target.value = ""; }} />
      </div>
      {library.length > 0 && (
        <div className="mt-5 space-y-2">
          <h3 className="text-sm font-bold text-gray-300">{t.thm_library}</h3>
          <p className="text-xs text-gray-500">{t.thm_library_help}</p>
          <ul className="flex flex-wrap gap-2">
            {library.map((entry) => (
              <li key={entry.id} className="flex items-center rounded-xl border border-white/10 bg-black/20">
                <button type="button" disabled={busy} onClick={() => applyEntry(entry)} className="flex items-center gap-2 pl-3 pr-2 py-2 text-sm hover:text-yellow-300">
                  <span className="flex -space-x-1">
                    {["primary", "accent", "ground"].map((k) => <span key={k} className="w-3.5 h-3.5 rounded-full ring-1 ring-black/60" style={{ background: entry.theme.colors?.[k] || "#333" }} />)}
                  </span>
                  {themeName(entry.theme, lang)}
                </button>
                <button type="button" onClick={() => forget(entry)} aria-label={t.thm_lib_delete} title={t.thm_lib_delete}
                  className="p-2 text-gray-500 hover:text-red-400"><X size={14} /></button>
              </li>
            ))}
          </ul>
        </div>
      )}
      {status && (
        <div className={`mt-3 text-sm ${status.ok ? "text-green-400" : "text-red-400"}`}>
          <p>{status.text}</p>
          {status.problems?.length > 0 && (
            <div className="mt-2 text-yellow-300/90">
              <p>{t.thm_problems}</p>
              <ul className="list-disc pl-5 mt-1 space-y-0.5">
                {status.problems.map((p, i) => <li key={i}>{fmt(t[`thm_p_${p.code}`] || p.code, { detail: p.detail })}</li>)}
              </ul>
            </div>
          )}
        </div>
      )}
    </Section>
  );
}
