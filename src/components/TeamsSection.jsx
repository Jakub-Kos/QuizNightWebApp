import { useRef, useState } from "react";
import { Plus, Users, History, Trash2, Camera, Crown, Target, Sheet, Loader2 } from "lucide-react";
import { TEAM_COLORS, TEAM_ICONS } from "../quiz/model";
import { newTeam, importTeamsCsv, importHistoryCsv, addTeamsFromScores } from "../quiz/teams";
import { fetchSheetCsv, sheetErrorText } from "../quiz/sheets";
import { parseScoresCsv } from "../quiz/parse";
import { fmt } from "../quiz/files";
import { Section, Status, btnCls } from "./EditorParts";

const fieldCls = "bg-black/40 border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-gray-600 focus:outline-none focus:border-yellow-500/60";

// teams: stored team records; mediaUrl(name) -> thumbnail URL; onUploadImage(team, file) -> stored file name;
// scoresSheetUrl: the quiz's live scores Sheet, whose team names can be loaded as teams
export default function TeamsSection({ teams, onChange, mediaUrl, hasMedia, scoresSheetUrl, onUploadImage, t }) {
  const [status, setStatus] = useState(null);
  const [loadingScores, setLoadingScores] = useState(false);
  const teamsCsv = useRef(null);
  const historyCsv = useRef(null);
  const photoInput = useRef(null);
  const photoFor = useRef(null);

  const updateTeam = (id, fields) => onChange(teams.map((team) => (team.id === id ? { ...team, ...fields } : team)));
  const removeTeam = (id) => onChange(teams.filter((team) => team.id !== id));
  const addTeam = () => onChange([...teams, newTeam({ color: TEAM_COLORS[teams.length % TEAM_COLORS.length] })]);
  const cycleColor = (team, i) => {
    const current = TEAM_COLORS.indexOf(team.color || TEAM_COLORS[i % TEAM_COLORS.length]);
    updateTeam(team.id, { color: TEAM_COLORS[(current + 1) % TEAM_COLORS.length] });
  };

  const readCsv = async (e, handler) => {
    const file = e.target.files[0];
    e.target.value = "";
    if (!file) return;
    try {
      handler(await file.text());
    } catch (err) {
      setStatus({ ok: false, text: t[`tm_err_${err.message}`] || String(err.message) });
    }
  };

  const onTeamsCsv = (e) => readCsv(e, (text) => {
    const result = importTeamsCsv(text, teams, hasMedia);
    onChange(result.teams);
    setStatus({ ok: true, text: fmt(t.tm_imported, result) });
  });

  const onHistoryCsv = (e) => readCsv(e, (text) => {
    const result = importHistoryCsv(text, teams);
    onChange(result.teams);
    setStatus({ ok: result.matched > 0, text: fmt(t.tm_history_imported, result) });
  });

  const loadFromScores = async () => {
    setLoadingScores(true);
    try {
      const scores = parseScoresCsv(await fetchSheetCsv(scoresSheetUrl));
      if (!scores) setStatus({ ok: false, text: t.ed_err_format });
      else if (scores.teams.length === 0) setStatus({ ok: false, text: t.tm_from_scores_none });
      else {
        const result = addTeamsFromScores(scores, teams);
        onChange(result.teams);
        setStatus({ ok: true, text: fmt(t.tm_from_scores_done, result) });
      }
    } catch (err) {
      setStatus({ ok: false, text: sheetErrorText(err, t) });
    }
    setLoadingScores(false);
  };

  const onPhoto = async (e) => {
    const file = e.target.files[0];
    e.target.value = "";
    const team = photoFor.current;
    if (!file || !team) return;
    updateTeam(team.id, { image: await onUploadImage(team, file) });
  };

  const names = teams.map((team) => team.name.trim().toLowerCase()).filter(Boolean);
  const duplicates = new Set(names.filter((n, i) => names.indexOf(n) !== i));

  return (
    <Section
      title={t.tm_title}
      help={t.tm_help}
      helpId="teams"
      actions={(
        <>
          <button onClick={loadFromScores} disabled={!scoresSheetUrl || loadingScores} title={scoresSheetUrl ? "" : t.tm_from_scores_need} className={btnCls}>
            {loadingScores ? <Loader2 size={16} className="animate-spin" /> : <Sheet size={16} />} {t.tm_from_scores}
          </button>
          <button onClick={() => teamsCsv.current.click()} className={btnCls}><Users size={16} /> {t.tm_import}</button>
          <button onClick={() => historyCsv.current.click()} disabled={teams.length === 0} className={btnCls}><History size={16} /> {t.tm_import_history}</button>
        </>
      )}
    >
      <input ref={teamsCsv} type="file" accept=".csv,text/csv" hidden onChange={onTeamsCsv} />
      <input ref={historyCsv} type="file" accept=".csv,text/csv" hidden onChange={onHistoryCsv} />
      <input ref={photoInput} type="file" accept="image/*" hidden onChange={onPhoto} />
      <Status status={status} />

      {teams.length === 0 && <p className="text-sm text-gray-500">{t.tm_empty}</p>}

      <ul className="space-y-2">
        {teams.map((team, i) => {
          const Icon = TEAM_ICONS[team.icon] || TEAM_ICONS.Brain;
          const color = team.color || TEAM_COLORS[i % TEAM_COLORS.length];
          const image = team.image ? mediaUrl(team.image) : null;
          const duplicate = duplicates.has(team.name.trim().toLowerCase());
          return (
            <li key={team.id} className="flex flex-wrap items-center gap-3 p-3 rounded-xl bg-black/30 border border-white/5">
              <button onClick={() => { photoFor.current = team; photoInput.current.click(); }} title={t.tm_photo}
                className={`group relative w-14 h-14 shrink-0 rounded-xl overflow-hidden bg-gradient-to-br ${color} flex items-center justify-center`}>
                {image ? <img src={image} alt="" className="w-full h-full object-cover" /> : <Icon size={24} className="text-white/80" />}
                <span className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 flex items-center justify-center transition-opacity"><Camera size={18} /></span>
              </button>
              <div className="flex-1 min-w-[200px] space-y-2">
                <div className="flex gap-2">
                  <input value={team.name} onChange={(e) => updateTeam(team.id, { name: e.target.value })} placeholder={t.tm_name}
                    className={`${fieldCls} flex-1 font-bold ${duplicate || !team.name.trim() ? "border-red-500/60" : ""}`} />
                  <label className="flex items-center gap-1 text-xs text-gray-400" title={t.tm_players}>
                    <Users size={14} />
                    <input type="number" min="0" value={team.players || ""} onChange={(e) => updateTeam(team.id, { players: parseInt(e.target.value) || 0 })}
                      className={`${fieldCls} w-16`} />
                  </label>
                </div>
                <input value={team.quote || ""} onChange={(e) => updateTeam(team.id, { quote: e.target.value })} placeholder={t.tm_quote}
                  className={`${fieldCls} w-full italic`} />
                {duplicate && <p className="text-xs text-red-300">{t.tm_duplicate}</p>}
                {!team.isNew && team.rankHistory?.length > 0 && (
                  <p className="flex items-center gap-3 text-xs text-gray-500">
                    <span className="flex items-center gap-1"><Crown size={12} /> #{team.bestRank}</span>
                    <span className="flex items-center gap-1"><Target size={12} /> {team.correctRate}%</span>
                    <span>{team.rankHistory.join(" · ")}</span>
                  </p>
                )}
              </div>
              <button onClick={() => cycleColor(team, i)} title={t.tm_color} className={`w-8 h-8 rounded-full bg-gradient-to-br ${color} ring-2 ring-white/10 hover:ring-white/50`} />
              <button onClick={() => removeTeam(team.id)} title={t.lib_delete} className="p-2 rounded-lg text-gray-500 hover:text-red-300 hover:bg-red-500/20">
                <Trash2 size={16} />
              </button>
            </li>
          );
        })}
      </ul>

      <button onClick={addTeam} className={`${btnCls} w-full border border-dashed border-white/15 bg-transparent`}>
        <Plus size={16} /> {t.tm_add}
      </button>
    </Section>
  );
}
