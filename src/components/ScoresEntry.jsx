import { useState, useEffect, useMemo, useRef } from "react";
import { AlertTriangle } from "lucide-react";
import BackButton from "./BackButton";
import { getQuiz, updateQuiz } from "../quiz/storage";
import { parseQuestionsCsv } from "../quiz/parse";
import { scoredRounds, roundLabel, announceScores } from "../quiz/scores";
import { isPresent } from "../quiz/teams";
import { quizHash } from "../hooks/useHashRoute";

// Score table for quizzes without a scores Sheet; usually opened from the presenter window.
// Every change is saved and announced, and the show's leaderboard picks it up.
export default function ScoresEntry({ id, t }) {
  const [quiz, setQuiz] = useState(undefined);
  const [scores, setScores] = useState({});
  const [saveState, setSaveState] = useState("saved");
  const dirty = useRef(false);

  useEffect(() => {
    getQuiz(id).then((q) => {
      setQuiz(q ?? null);
      setScores(q?.manualScores || {});
    });
  }, [id]);

  useEffect(() => {
    if (!dirty.current) return;
    const timer = setTimeout(async () => {
      await updateQuiz(id, (q) => ({ ...q, manualScores: scores }));
      announceScores(id);
      dirty.current = false;
      setSaveState("saved");
    }, 300);
    return () => clearTimeout(timer);
  }, [id, scores]);

  const rounds = useMemo(() => (quiz ? scoredRounds(parseQuestionsCsv(quiz.questionsCsv)) : []), [quiz]);
  // Teams marked absent at the attendance check get no row
  const teams = quiz ? quiz.teams.filter((team) => team.name.trim() && isPresent(team)) : [];

  const setScore = (teamId, roundNumber, value) => {
    dirty.current = true;
    setSaveState("saving");
    setScores((s) => {
      const teamScores = { ...s[teamId] };
      if (value === "") delete teamScores[roundNumber];
      // Kept as typed (comma -> dot) so "3." can become "3.5"; readers convert with Number()
      else teamScores[roundNumber] = value.replace(",", ".");
      return { ...s, [teamId]: teamScores };
    });
  };

  // Enter moves down the column, so one round can be typed in for every team in a row
  const onKeyDown = (e, row, col) => {
    if (e.key !== "Enter") return;
    e.preventDefault();
    const next = document.querySelector(`[data-cell="${row + (e.shiftKey ? -1 : 1)}:${col}"]`);
    next?.focus();
    next?.select();
  };

  if (quiz === undefined) return <div className="min-h-screen bg-[#050505]" />;
  if (quiz === null) {
    return (
      <div className="min-h-screen bg-[#050505] text-white flex flex-col items-center justify-center gap-6">
        <p className="text-xl text-gray-300">{t.ed_not_found}</p>
        <a href="#/" className="px-6 py-3 rounded-xl bg-yellow-500 text-black font-bold">{t.settings_exit}</a>
      </div>
    );
  }

  const total = (teamId) => rounds.reduce((sum, r) => sum + (Number(scores[teamId]?.[r.number]) || 0), 0);
  const ranked = [...teams].sort((a, b) => total(b.id) - total(a.id));
  const rankOf = (teamId) => 1 + ranked.findIndex((team) => total(team.id) === total(teamId));

  return (
    <div className="min-h-screen bg-[#050505] text-white font-sans">
      <div className="max-w-6xl mx-auto px-6 py-8 space-y-6">
        <header className="flex flex-wrap items-center gap-4">
          <BackButton quizId={quiz.id} t={t} />
          <h1 className="font-['League_Spartan'] text-3xl font-black">{t.sc_title}: {quiz.title}</h1>
          <span className="ml-auto text-xs text-gray-500">{saveState === "saving" ? t.ed_saving : t.ed_saved}</span>
        </header>

        {quiz.scoresSheetUrl && (
          <p className="flex items-start gap-2 p-4 rounded-xl bg-yellow-500/10 border border-yellow-500/30 text-yellow-200 text-sm">
            <AlertTriangle size={16} className="shrink-0 mt-0.5" /> {t.sc_sheet_linked}
          </p>
        )}
        <p className="text-sm text-gray-400">{t.sc_help}</p>

        {teams.length === 0 || rounds.length === 0 ? (
          <p className="text-gray-400">{teams.length === 0 ? t.sc_no_teams : t.ed_no_questions} <a href={quizHash(quiz.id, "edit")} className="text-yellow-400 underline">{t.lib_edit}</a></p>
        ) : (
          <div className="overflow-x-auto rounded-2xl border border-white/10">
            <table className="w-full text-sm">
              <thead className="bg-white/5 text-gray-400">
                <tr>
                  <th className="text-left px-4 py-3 font-bold">{t.sc_team}</th>
                  {rounds.map((r) => (
                    <th key={r.number} className="px-2 py-3 font-bold whitespace-nowrap" title={r.title}>{roundLabel(r)}</th>
                  ))}
                  <th className="px-4 py-3 font-bold text-right">{t.sc_total}</th>
                  <th className="px-4 py-3 font-bold text-right">#</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/5">
                {teams.map((team, row) => (
                  <tr key={team.id} className="hover:bg-white/[0.03]">
                    <td className="px-4 py-2 font-bold whitespace-nowrap">{team.name}</td>
                    {rounds.map((r, col) => (
                      <td key={r.number} className="px-1 py-1">
                        <input
                          data-cell={`${row}:${col}`}
                          type="text" inputMode="decimal"
                          value={scores[team.id]?.[r.number] ?? ""}
                          onChange={(e) => /^-?\d*([.,]\d*)?$/.test(e.target.value) && setScore(team.id, r.number, e.target.value)}
                          onKeyDown={(e) => onKeyDown(e, row, col)}
                          onFocus={(e) => e.target.select()}
                          className="w-16 mx-auto block bg-black/40 border border-white/10 rounded-lg px-2 py-2 text-center font-mono focus:outline-none focus:border-yellow-500/60"
                        />
                      </td>
                    ))}
                    <td className="px-4 py-2 text-right font-mono font-bold text-yellow-400">{total(team.id)}</td>
                    <td className="px-4 py-2 text-right font-mono text-gray-400">{rankOf(team.id)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
