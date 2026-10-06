import { useState, useEffect, useRef } from "react";
import { Check, X, Users, MapPin, Camera, Plus, CircleHelp } from "lucide-react";
import { getQuiz, updateQuiz } from "../quiz/storage";
import { TEAM_COLORS, TEAM_ICONS } from "../quiz/model";
import { newTeam, saveTeamImage, isPresent, directionText } from "../quiz/teams";
import { fmt } from "../quiz/files";
import { useMediaUrls } from "../hooks/useMediaUrls";
import VenueMap from "./VenueMap";
import BackButton from "./BackButton";

const fieldCls = "bg-black/40 border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-gray-600 focus:outline-none focus:border-yellow-500/60";

// Moderator's check-in at the start of the quiz: who came, how many players, last-minute photo or motto
// changes, and where each team sits (for facing them during the results). Usually opened from the presenter.
// Teams fill the top half of the window, the seating map the full-width bottom half.
export default function Attendance({ id, t }) {
  const [quiz, setQuiz] = useState(undefined);
  const [saveState, setSaveState] = useState("saved");
  const [placing, setPlacing] = useState(null);
  const dirty = useRef(false);
  const photoInput = useRef(null);
  const photoFor = useRef(null);
  const { urlFor, reload } = useMediaUrls(id);

  useEffect(() => { getQuiz(id).then((q) => setQuiz(q ?? null)); }, [id]);

  // Only teams and the field of view are written, so the setup page and score entry keep their changes
  useEffect(() => {
    if (!dirty.current) return;
    const { teams, settings } = quiz;
    const timer = setTimeout(() => updateQuiz(id, (q) => ({ ...q, teams, settings: { ...q.settings, venueArc: settings.venueArc } })).then(() => {
      dirty.current = false;
      setSaveState("saved");
    }), 400);
    return () => clearTimeout(timer);
  }, [id, quiz]);

  const change = (fn) => {
    dirty.current = true;
    setSaveState("saving");
    setQuiz(fn);
  };
  const updateTeam = (teamId, fields) => change((q) => ({ ...q, teams: q.teams.map((team) => (team.id === teamId ? { ...team, ...fields } : team)) }));
  // A team that has a seat is in the room
  const setSeat = (teamId, seat) => { updateTeam(teamId, { seat, present: true }); setPlacing(null); };
  const addTeam = () => {
    const team = newTeam({ present: true, color: TEAM_COLORS[quiz.teams.length % TEAM_COLORS.length] });
    change((q) => ({ ...q, teams: [...q.teams, team] }));
  };

  const onPhoto = async (e) => {
    const file = e.target.files[0];
    e.target.value = "";
    const team = photoFor.current;
    if (!file || !team) return;
    const image = await saveTeamImage(id, team, file);
    await reload();
    updateTeam(team.id, { image });
  };

  useEffect(() => {
    if (!placing) return;
    const onKey = (e) => e.key === "Escape" && setPlacing(null);
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [placing]);

  if (quiz === undefined) return <div className="min-h-screen bg-[#050505]" />;
  if (quiz === null) {
    return (
      <div className="min-h-screen bg-[#050505] text-white flex flex-col items-center justify-center gap-6">
        <p className="text-xl text-gray-300">{t.ed_not_found}</p>
        <a href="#/" className="px-6 py-3 rounded-xl bg-yellow-500 text-black font-bold">{t.settings_exit}</a>
      </div>
    );
  }

  const arc = quiz.settings.venueArc || 180;
  const present = quiz.teams.filter((team) => team.present === true);
  const players = present.reduce((n, team) => n + (Number(team.players) || 0), 0);
  const placingTeam = quiz.teams.find((team) => team.id === placing);

  return (
    <div className="h-screen flex flex-col bg-[#050505] text-white font-sans">
      <header className="shrink-0 flex flex-wrap items-center gap-x-6 gap-y-2 px-6 py-4 border-b border-white/10">
        <BackButton quizId={quiz.id} t={t} />
        <h1 className="font-['League_Spartan'] text-2xl font-black">{t.att_title}: {quiz.title}</h1>
        <span className="text-lg font-bold text-yellow-400">{fmt(t.att_summary, { present: present.length, total: quiz.teams.length, players })}</span>
        <span className="ml-auto text-xs text-gray-500">{saveState === "saving" ? t.ed_saving : t.ed_saved}</span>
      </header>

      {/* Teams: top half, scrolls */}
      <div className="flex-1 min-h-0 overflow-y-auto px-6 py-4 space-y-3">
        <p className="text-sm text-gray-400">{t.att_help}</p>
        <input ref={photoInput} type="file" accept="image/*" hidden onChange={onPhoto} />
        <ul className="grid xl:grid-cols-2 gap-2">
          {quiz.teams.map((team, i) => {
            const Icon = TEAM_ICONS[team.icon] || TEAM_ICONS.Brain;
            const color = team.color || TEAM_COLORS[i % TEAM_COLORS.length];
            const image = urlFor(team.image);
            const direction = directionText(team.seat, t);
            const state = team.present === true ? "present" : team.present === false ? "absent" : "unchecked";
            return (
              <li key={team.id} className={`flex flex-wrap items-center gap-3 p-3 rounded-xl border transition-colors ${
                state === "present" ? "bg-green-500/[0.07] border-green-500/30" : state === "absent" ? "bg-black/30 border-white/5 opacity-50" : "bg-black/30 border-white/10"} ${
                placing === team.id ? "ring-2 ring-yellow-400" : ""}`}>
                <div className="flex flex-col gap-1">
                  <button onClick={() => updateTeam(team.id, { present: true })} title={t.att_present}
                    className={`p-2 rounded-lg ${state === "present" ? "bg-green-600 text-white" : "bg-white/5 text-gray-500 hover:text-green-300"}`}><Check size={18} /></button>
                  <button onClick={() => updateTeam(team.id, { present: false })} title={t.att_absent}
                    className={`p-2 rounded-lg ${state === "absent" ? "bg-red-700 text-white" : "bg-white/5 text-gray-500 hover:text-red-300"}`}><X size={18} /></button>
                </div>
                <button onClick={() => { photoFor.current = team; photoInput.current.click(); }} title={t.tm_photo}
                  className={`group relative w-16 h-16 shrink-0 rounded-xl overflow-hidden bg-gradient-to-br ${color} flex items-center justify-center`}>
                  {image ? <img src={image} alt="" className="w-full h-full object-cover" /> : <Icon size={26} className="text-white/80" />}
                  <span className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 flex items-center justify-center transition-opacity"><Camera size={18} /></span>
                </button>
                <div className="flex-1 min-w-[180px] space-y-2">
                  <div className="flex gap-2">
                    <input value={team.name} onChange={(e) => updateTeam(team.id, { name: e.target.value })} placeholder={t.tm_name} className={`${fieldCls} flex-1 min-w-0 font-bold`} />
                    <label className="flex items-center gap-1 text-xs text-gray-400" title={t.tm_players}>
                      <Users size={14} />
                      <input type="number" min="0" value={team.players || ""} onChange={(e) => updateTeam(team.id, { players: parseInt(e.target.value) || 0 })} className={`${fieldCls} w-16`} />
                    </label>
                  </div>
                  <input value={team.quote || ""} onChange={(e) => updateTeam(team.id, { quote: e.target.value })} placeholder={t.tm_quote} className={`${fieldCls} w-full italic`} />
                </div>
                <button onClick={() => setPlacing(placing === team.id ? null : team.id)}
                  className={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm whitespace-nowrap ${placing === team.id ? "bg-yellow-500 text-black font-bold" : direction ? "bg-white/10 hover:bg-white/20" : "bg-white/5 text-gray-400 hover:bg-white/10"}`}>
                  <MapPin size={16} /> {direction || t.att_place}
                </button>
              </li>
            );
          })}
          <li>
            <button onClick={addTeam} className="w-full h-full min-h-14 flex items-center justify-center gap-2 px-4 py-3 rounded-xl border border-dashed border-white/15 hover:bg-white/5 text-sm">
              <Plus size={16} /> {t.tm_add}
            </button>
          </li>
        </ul>
      </div>

      {/* Seating map: full-width bottom half */}
      <div className="h-[50vh] shrink-0 flex flex-col border-t border-white/10 bg-white/[0.03]">
        <div className="shrink-0 flex flex-wrap items-center gap-3 px-6 py-2">
          <p className={`flex items-center gap-2 text-sm mr-auto ${placingTeam ? "text-yellow-300 font-bold" : "text-gray-400"}`}>
            <CircleHelp size={16} className="shrink-0" />
            {placingTeam ? fmt(t.att_place_hint, { team: placingTeam.name || "?" }) : t.att_map_help}
          </p>
          <span className="text-sm text-gray-400">{t.att_arc}</span>
          {[180, 270].map((a) => (
            <button key={a} onClick={() => change((q) => ({ ...q, settings: { ...q.settings, venueArc: a } }))}
              className={`px-3 py-1.5 rounded-lg text-sm ${arc === a ? "bg-yellow-500 text-black font-bold" : "bg-white/10 hover:bg-white/20"}`}>{a}°</button>
          ))}
        </div>
        <div className="flex-1 min-h-0 px-6 pb-3">
          <VenueMap arc={arc} teams={quiz.teams.map((team) => (isPresent(team) ? team : { ...team, seat: null }))}
            onSeat={setSeat} placing={placing} youLabel={t.att_you} fill className="h-full" />
        </div>
      </div>
    </div>
  );
}
