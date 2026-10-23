import { useMemo } from "react";
import { Check } from "lucide-react";
import { TRANSLATIONS } from "../data/translations";
import { QuizContext } from "../quiz/context";
import { parseQuestionsCsv } from "../quiz/parse";
import { THEMES } from "../quiz/themes";
import { useDisplaySettings } from "../hooks/useDisplay";
import { useUiLanguage } from "../hooks/useUiLanguage";
import { Section } from "./EditorParts";
import { StageFrame } from "./Stage";
import { ThemeDecorations } from "./Theme";
import Dashboard from "./Dashboard";

const noop = () => {};

// The show's look (quiz/themes.js): a card per theme with the round overview as it would appear
export default function ThemeSection({ quiz, onChange, warning, t }) {
  const [display] = useDisplaySettings("main");
  const [lang] = useUiLanguage();
  const showT = TRANSLATIONS[quiz.settings?.language] || TRANSLATIONS.en;
  const rounds = useMemo(() => parseQuestionsCsv(quiz.questionsCsv), [quiz.questionsCsv]);
  const context = useMemo(() => ({ quiz, rounds, teams: [], resolveMedia: () => null }), [quiz, rounds]);
  const current = quiz.settings?.theme ?? null; // none yet: nothing is marked as chosen

  return (
    <Section id="theme" warning={warning} title={t.ed_theme} help={t.ed_theme_help}>
      <QuizContext.Provider value={context}>
        <div className="grid gap-4 sm:grid-cols-3">
          {THEMES.map((theme) => {
            const chosen = theme.id === current;
            return (
              <button key={theme.id} type="button" onClick={() => onChange(theme.id)} aria-pressed={chosen}
                className={`group text-left rounded-xl p-2 border transition-all ${chosen ? "border-yellow-500 bg-yellow-500/10" : "border-white/10 hover:border-white/30 bg-black/20"}`}>
                <StageFrame display={display} themeId={theme.id} className="rounded-lg pointer-events-none">
                  <Dashboard rounds={rounds} onSelectRound={noop} onOpenLeaderboard={noop} onOpenPause={noop} t={showT} />
                  <ThemeDecorations />
                </StageFrame>
                <span className="flex items-center gap-2 px-1 pt-2 font-bold text-white">
                  {chosen && <Check size={16} className="text-yellow-400" />}
                  {theme.name[lang] || theme.name.en}
                </span>
              </button>
            );
          })}
        </div>
      </QuizContext.Provider>
    </Section>
  );
}
