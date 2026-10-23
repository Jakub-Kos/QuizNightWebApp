import { useState, useEffect, useMemo, useRef, useDeferredValue } from "react";
import { MotionConfig } from "framer-motion";
import { ArrowLeft, Upload, Trash2, Plus, Download, BookmarkPlus, CheckCircle2, AlertTriangle, Loader2, RotateCcw, Share2, X, ExternalLink } from "lucide-react";
import { TRANSLATIONS } from "../data/translations";
import { getQuiz, updateQuiz, listMedia } from "../quiz/storage";
import { QuizContext } from "../quiz/context";
import { parseQuestionsCsv, isRoundHidden } from "../quiz/parse";
import { resolveTeams, mediaKey } from "../quiz/model";
import { rulesFor, prizesFor } from "../quiz/intro";
import { previewStep } from "../quiz/validate";
import { fmt } from "../quiz/files";
import { downloadBlob } from "../quiz/package";
import { themeForQuiz, themeName, THEMES, CUSTOM_THEME_ID } from "../quiz/themes";
import { normalizeTheme, readabilityIssues, FONT_GROUPS, ORNAMENTS, DECORATIONS, COLOR_KEYS, LANGS, LIMITS, slug } from "../quiz/themeFormat";
import { exportThemePack, themePackName } from "../quiz/themePack";
import { storeThemeFile, pruneThemeFiles, saveToLibrary, FileError } from "../quiz/themeStore";
import { driveErrorText } from "../quiz/drive";
import { shareUrl } from "../quiz/workshop";
import { quizHash } from "../hooks/useHashRoute";
import { useDisplaySettings } from "../hooks/useDisplay";
import { useScores } from "../hooks/useScores";
import { sampleLeaderboard, hasPoints } from "../quiz/scores";
import { DEMO_TEAMS } from "../quiz/demoTeams";
import { useUiLanguage } from "../hooks/useUiLanguage";
import { inputCls, btnCls } from "./EditorParts";
import { StageFrame } from "./Stage";
import ColorPicker from "./ColorPicker";
import { ThemeDecorations } from "./Theme";
import WelcomeScreen from "./WelcomeScreen";
import RulesScreen from "./RulesScreen";
import PrizesScreen from "./PrizesScreen";
import Dashboard from "./Dashboard";
import QuestionScreen from "./QuestionScreen";
import Leaderboard from "./Leaderboard";
import PauseScreen from "./PauseScreen";

const noop = () => {};
// The colours Classic uses, shown when a theme leaves one out
const CLASSIC_COLORS = { primary: "#3b82f6", accent: "#eab308", groundTop: "#111111", ground: "#050505", groundBottom: "#000000", panel: "#0a0a0a" };
const QUIET_SCENES = ["question", "answer", "leaderboard"];
const DARK_KEYS = ["groundTop", "ground", "groundBottom", "panel"];

// #/quiz/<id>/theme: the quiz's own theme, edited with a live preview of every screen. fresh (the
// …/theme/new link) starts a new own theme from the theme the quiz shows now instead of editing it.
export default function ThemeEditor({ id, fresh, t }) {
  const [quiz, setQuiz] = useState(undefined);
  const [media, setMedia] = useState([]);
  const urls = useRef([]);

  // Object URLs for the preview; the ref lets them be revoked on reload and unmount
  const fetchMedia = async () => {
    const list = await listMedia(id);
    urls.current.forEach((url) => URL.revokeObjectURL(url));
    const next = list.map((m) => ({ ...m, url: URL.createObjectURL(m.blob) }));
    urls.current = next.map((m) => m.url);
    return next;
  };
  const loadMedia = async () => setMedia(await fetchMedia());

  useEffect(() => {
    getQuiz(id).then((q) => setQuiz(q ?? null));
    fetchMedia().then(setMedia);
    return () => urls.current.forEach((url) => URL.revokeObjectURL(url));
  }, [id]); // eslint-disable-line react-hooks/exhaustive-deps

  if (quiz === undefined) return <div className="min-h-screen bg-[#050505]" />;
  if (quiz === null) {
    return (
      <div className="min-h-screen bg-[#050505] text-white flex flex-col items-center justify-center gap-6">
        <p className="text-xl text-gray-300">{t.ed_not_found}</p>
        <a href="#/" className="px-6 py-3 rounded-xl bg-yellow-500 text-black font-bold">{t.settings_exit}</a>
      </div>
    );
  }
  return <Editor quiz={quiz} media={media} reloadMedia={loadMedia} fresh={fresh} t={t} />;
}

// A copy of a theme to edit as the quiz's own one
function copyOf(theme, t) {
  if (!THEMES.includes(theme)) return theme;
  const name = Object.fromEntries(LANGS.map((l) => [l, fmt(t.te_copy_name, { name: theme.name[l] || theme.name.en })]));
  return { ...theme, id: slug(`${theme.id}-own`), name, author: "" };
}

function Editor({ quiz, media, reloadMedia, fresh, t }) {
  const [lang] = useUiLanguage();
  const [display] = useDisplaySettings("main");
  const [draft, setDraft] = useState(() => (!fresh && quiz.customTheme ? quiz.customTheme : copyOf(themeForQuiz(quiz), t)));
  const [saveState, setSaveState] = useState("saved");
  const [scene, setScene] = useState("welcome");
  const [notice, setNotice] = useState(null); // { ok, text }
  const [busy, setBusy] = useState(null);
  const [libraryId, setLibraryId] = useState(null);
  const [sharing, setSharing] = useState(false);
  const changed = useRef(false);

  // Every change is saved shortly after, as the quiz's own theme, chosen for the show
  useEffect(() => {
    if (!changed.current) return;
    setSaveState("saving");
    const timer = setTimeout(async () => {
      const theme = normalizeTheme(draft, { stored: true }).theme;
      await updateQuiz(quiz.id, (q) => ({ ...q, customTheme: theme, settings: { ...q.settings, theme: CUSTOM_THEME_ID } }));
      await pruneThemeFiles(quiz.id, theme);
      setSaveState("saved");
    }, 600);
    return () => clearTimeout(timer);
  }, [draft, quiz.id]);

  const change = (fn) => {
    changed.current = true;
    setDraft((d) => fn(d));
  };
  const set = (path, value) => change((d) => {
    const [group, key] = path.split(".");
    return key ? { ...d, [group]: { ...d[group], [key]: value } } : { ...d, [group]: value };
  });

  const showT = TRANSLATIONS[quiz.settings?.language] || TRANSLATIONS.en;
  const byKey = useMemo(() => new Map(media.map((m) => [mediaKey(m.name), m])), [media]);
  const resolveMedia = useMemo(() => (name) => (name ? byKey.get(mediaKey(name))?.url ?? null : null), [byKey]);
  const rounds = useMemo(() => parseQuestionsCsv(quiz.questionsCsv), [quiz.questionsCsv]);
  const teams = useMemo(() => resolveTeams(quiz.teams, resolveMedia), [quiz.teams, resolveMedia]);
  const context = useMemo(() => ({ quiz, rounds, teams, resolveMedia }), [quiz, rounds, teams, resolveMedia]);
  const liveScores = useScores(quiz, rounds, showT.round);
  // Without points yet, the leaderboard and half-time screens get sample points, so they can be judged
  const scores = useMemo(() => (hasPoints(liveScores) ? liveScores
    : sampleLeaderboard(quiz.teams, liveScores.rounds, showT.round, DEMO_TEAMS.map((team) => team.name))), [liveScores, quiz.teams, showT.round]);
  // The preview follows a moment behind while a colour is being dragged, so the controls stay smooth
  const previewTheme = useDeferredValue(draft);
  const issues = readabilityIssues(draft);
  const themeColors = [...COLOR_KEYS.map((key) => draft.colors[key] || CLASSIC_COLORS[key]), ...draft.background.glows.map((g) => g.color)];

  const upload = async (file, kind, apply) => {
    if (!file) return;
    setBusy(kind);
    setNotice(null);
    try {
      const name = await storeThemeFile(quiz.id, file, kind);
      await reloadMedia();
      apply(name);
    } catch (err) {
      setNotice({ ok: false, text: err instanceof FileError ? fmt(t[`te_err_${err.message}`], err.params) : driveErrorText(err, t) });
    }
    setBusy(null);
  };

  const exportTheme = async () => downloadBlob(await exportThemePack(draft, (name) => byKey.get(mediaKey(name))?.blob ?? null), themePackName(draft));
  const keep = async () => {
    setBusy("library");
    const saved = await saveToLibrary(quiz.id, normalizeTheme(draft, { stored: true }).theme, libraryId ?? undefined);
    setLibraryId(saved);
    setNotice({ ok: true, text: fmt(t.te_lib_saved, { name: themeName(draft, lang) }) });
    setBusy(null);
  };

  // The preview's screens, with this quiz's content
  const firstRound = rounds.find((r) => !isRoundHidden(r) && r.questions.length) || null;
  const rules = rulesFor(quiz, showT);
  const prizes = prizesFor(quiz);
  const screens = [
    ["welcome", t.mp_welcome, <WelcomeScreen onStart={noop} startTime={quiz.settings.startTime} showTime={quiz.settings.showTime} config={quiz.settings} t={showT} />],
    rules && ["rules", t.mp_rules, <RulesScreen rules={rules} onNext={noop} t={showT} />],
    prizes && ["prizes", t.mp_prizes, <PrizesScreen prizes={prizes} onNext={noop} t={showT} />],
    ["dashboard", t.te_s_dashboard, <Dashboard rounds={rounds} onSelectRound={noop} onOpenLeaderboard={noop} onOpenPause={noop} t={showT} />],
    firstRound && ["round", t.te_s_round, <QuestionScreen roundData={firstRound} mode="with_answers" onBack={noop} isPresenter={false} t={showT} preview={{ qIndex: -1, step: 0 }} />],
    firstRound && ["question", t.te_s_question, <QuestionScreen roundData={firstRound} mode="with_answers" onBack={noop} isPresenter={false} t={showT} preview={{ qIndex: 0, step: previewStep(firstRound.questions[0].type, "question") }} />],
    firstRound && ["answer", t.te_s_answer, <QuestionScreen roundData={firstRound} mode="with_answers" onBack={noop} isPresenter={false} t={showT} preview={{ qIndex: 0, step: previewStep(firstRound.questions[0].type, "answer") }} />],
    ["leaderboard", t.te_s_leaderboard, <Leaderboard data={scores.teams} availableRounds={scores.rounds} onClose={noop} isPresenter={false} t={showT} preview={{ step: 0 }} />],
    ["pause", t.mp_pause, <PauseScreen isPresenter={false} leaderboardData={scores.teams} availableRounds={scores.rounds} onBack={noop} t={showT} />],
  ].filter(Boolean);
  const shown = screens.find(([key]) => key === scene) || screens[0];

  return (
    <QuizContext.Provider value={context}>
      <div className="min-h-screen bg-[#050505] text-white font-sans">
        <header className="sticky top-0 z-30 bg-[#050505]/95 backdrop-blur border-b border-white/10">
          <div className="max-w-[1700px] mx-auto px-6 py-4 flex flex-wrap items-center gap-3">
            <a href={quizHash(quiz.id, "edit")} className="flex items-center gap-2 text-gray-400 hover:text-white">
              <ArrowLeft size={18} /> {t.te_back}
            </a>
            <h1 className="font-['League_Spartan'] text-xl font-bold uppercase tracking-widest mr-auto">{t.te_title}</h1>
            <span className="text-sm text-gray-500">{saveState === "saving" ? t.te_saving : t.te_saved}</span>
            <button type="button" className={btnCls} onClick={keep} disabled={busy === "library"}>
              {busy === "library" ? <Loader2 size={16} className="animate-spin" /> : <BookmarkPlus size={16} />} {libraryId ? t.te_lib_update : t.te_lib_save}
            </button>
            <button type="button" className={btnCls} onClick={exportTheme}><Download size={16} /> {t.te_export}</button>
            <button type="button" className={`${btnCls} bg-yellow-500 hover:bg-yellow-400 text-black font-bold`} onClick={() => setSharing(true)}><Share2 size={16} /> {t.te_share}</button>
          </div>
        </header>

        <div className="max-w-[1700px] mx-auto px-6 py-6 grid gap-6 lg:grid-cols-[420px_1fr]">
          <aside className="space-y-4">
            {notice && <p className={`text-sm ${notice.ok ? "text-green-400" : "text-red-400"}`}>{notice.text}</p>}

            <Panel title={t.te_about}>
              <Field label={t.te_name}>
                <input className={inputCls} value={themeName(draft, lang)} maxLength={LIMITS.text}
                  onChange={(e) => set("name", Object.fromEntries(LANGS.map((l) => [l, e.target.value])))} />
              </Field>
              <Field label={t.te_author}>
                <input className={inputCls} value={draft.author} maxLength={LIMITS.text} onChange={(e) => set("author", e.target.value)} />
              </Field>
              <Field label={t.te_description}>
                <textarea className={`${inputCls} min-h-20`} value={draft.description[lang] || draft.description.en || ""} maxLength={LIMITS.description}
                  onChange={(e) => set("description", Object.fromEntries(LANGS.map((l) => [l, e.target.value])))} />
              </Field>
              <Field label={t.te_tags}>
                <TagsInput tags={draft.tags} onChange={(tags) => set("tags", tags)} placeholder={t.te_tags_hint} />
              </Field>
            </Panel>

            <Panel title={t.te_colors}>
              {COLOR_KEYS.map((key) => (
                <ColorField key={key} label={t[`te_c_${key}`]} help={t[`te_c_${key}_help`]} value={draft.colors[key]} fallback={CLASSIC_COLORS[key]}
                  dark={DARK_KEYS.includes(key)} themeColors={themeColors}
                  onChange={(value) => set(`colors.${key}`, value)}
                  onReset={() => change((d) => { const { [key]: _, ...colors } = d.colors; return { ...d, colors }; })} t={t} />
              ))}
            </Panel>

            <Panel title={t.te_fonts}>
              {["title", "body"].map((role) => (
                <FontField key={role} label={t[`te_f_${role}`]} font={draft.fonts[role]} busy={busy === `font-${role}`} t={t}
                  onFamily={(family) => change((d) => { const { [role]: _, ...fonts } = d.fonts; return { ...d, fonts: family ? { ...fonts, [role]: { family } } : fonts }; })}
                  onFile={(file) => upload(file, "font", (name) => change((d) => ({ ...d, fonts: { ...d.fonts, [role]: { file: name } } })))} />
              ))}
            </Panel>

            <Panel title={t.te_background}>
              <PictureField image={draft.background.image} url={resolveMedia(draft.background.image)} busy={busy === "image"} t={t}
                onFile={(file) => upload(file, "image", (name) => set("background.image", name))}
                onRemove={() => set("background.image", null)} />
              {draft.background.image && (
                <>
                  <Slider label={t.te_bg_opacity} value={draft.background.imageOpacity} min={0} max={1} step={0.05} percent onChange={(v) => set("background.imageOpacity", v)} />
                  <Slider label={t.te_bg_blur} value={draft.background.imageBlur} min={0} max={20} step={1} onChange={(v) => set("background.imageBlur", v)} />
                </>
              )}
              <Choice label={t.te_scenery} value={draft.background.ornament} options={[null, ...ORNAMENTS]} name={(o) => t[`te_sc_${o ?? "none"}`]} onChange={(v) => set("background.ornament", v)} />
            </Panel>

            <Panel title={t.te_glows} help={t.te_glows_help}>
              {draft.background.glows.map((g, i) => {
                const setGlow = (fields) => set("background.glows", draft.background.glows.map((x, j) => (j === i ? { ...x, ...fields } : x)));
                return (
                  <div key={i} className="rounded-xl border border-white/10 p-3 space-y-2">
                    <div className="flex items-center gap-2">
                      <ColorPicker value={g.color} onChange={(color) => setGlow({ color })} label={t.te_glow_color} themeColors={themeColors} t={t} />
                      <span className="text-sm text-gray-400 mr-auto">{fmt(t.te_glow_n, { n: i + 1 })}</span>
                      <button type="button" onClick={() => set("background.glows", draft.background.glows.filter((_, j) => j !== i))} className="p-1.5 rounded-lg text-gray-500 hover:text-red-400 hover:bg-white/5" aria-label={t.te_remove}><Trash2 size={16} /></button>
                    </div>
                    <Slider label={t.te_glow_x} value={g.x} min={0} max={100} step={1} onChange={(v) => setGlow({ x: v })} />
                    <Slider label={t.te_glow_y} value={g.y} min={0} max={100} step={1} onChange={(v) => setGlow({ y: v })} />
                    <Slider label={t.te_glow_size} value={g.size} min={5} max={150} step={1} onChange={(v) => setGlow({ size: v })} />
                    <Slider label={t.te_glow_opacity} value={g.opacity} min={0} max={1} step={0.01} percent onChange={(v) => setGlow({ opacity: v })} />
                  </div>
                );
              })}
              {draft.background.glows.length < LIMITS.glows && (
                <button type="button" className={btnCls} onClick={() => set("background.glows", [...draft.background.glows, { color: draft.colors.primary || CLASSIC_COLORS.primary, x: 50, y: 100, size: 60, opacity: 0.2 }])}>
                  <Plus size={16} /> {t.te_glow_add}
                </button>
              )}
            </Panel>

            <Panel title={t.te_decoration}>
              <Choice value={draft.decoration.type} options={[null, ...DECORATIONS]} name={(o) => t[`te_dc_${o ?? "none"}`]} onChange={(v) => set("decoration.type", v)} />
              <p className="text-xs text-gray-500">{t.te_decoration_help}</p>
            </Panel>
          </aside>

          <main className="space-y-3 lg:sticky lg:top-24 self-start">
            <div className="flex flex-wrap gap-1.5">
              {screens.map(([key, label]) => (
                <button key={key} type="button" onClick={() => setScene(key)}
                  className={`px-3 py-1.5 rounded-lg text-sm ${shown[0] === key ? "bg-yellow-500 text-black font-bold" : "bg-white/5 text-gray-300 hover:bg-white/10"}`}>
                  {label}
                </button>
              ))}
            </div>
            <MotionConfig reducedMotion="never">
              <StageFrame display={display} theme={previewTheme} className="rounded-xl ring-1 ring-white/15">
                <div key={shown[0]} className="contents">{shown[2]}</div>
                <ThemeDecorations quiet={QUIET_SCENES.includes(shown[0])} />
              </StageFrame>
            </MotionConfig>
            <div className={`rounded-xl border p-3 text-sm ${issues.length ? "border-yellow-500/40 bg-yellow-500/[0.05]" : "border-green-500/20"}`}>
              {issues.length ? (
                <ul className="space-y-1">
                  {issues.map((code) => <li key={code} className="flex gap-2 text-yellow-200"><AlertTriangle size={16} className="shrink-0 mt-0.5" /> {t[`te_r_${code}`]}</li>)}
                </ul>
              ) : (
                <p className="flex gap-2 text-green-400"><CheckCircle2 size={16} className="shrink-0 mt-0.5" /> {t.te_r_ok}</p>
              )}
            </div>
          </main>
        </div>
      </div>
      {sharing && <ShareDialog theme={draft} issues={issues} lang={lang} onExport={exportTheme} onClose={() => setSharing(false)} t={t} />}
    </QuizContext.Provider>
  );
}

function Panel({ title, help, children }) {
  return (
    <section className="rounded-2xl bg-white/[0.04] border border-white/10 p-4 space-y-3">
      <h2 className="font-['League_Spartan'] font-bold uppercase tracking-widest text-sm text-gray-300">{title}</h2>
      {help && <p className="text-xs text-gray-500 -mt-1">{help}</p>}
      {children}
    </section>
  );
}

function Field({ label, children }) {
  return (
    <label className="block space-y-1">
      <span className="text-sm text-gray-400">{label}</span>
      {children}
    </label>
  );
}

function ColorField({ label, help, value, fallback, dark, themeColors, onChange, onReset, t }) {
  return (
    <div className="flex items-center gap-3">
      <ColorPicker value={value || fallback} onChange={onChange} label={label} dark={dark} themeColors={themeColors} t={t} />
      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium">{label}</div>
        <div className="text-xs text-gray-500 truncate">{help}</div>
      </div>
      <span className="text-xs font-mono text-gray-400 uppercase">{value || t.te_classic_color}</span>
      <button type="button" onClick={onReset} disabled={!value} title={t.te_reset_color} aria-label={t.te_reset_color}
        className="p-1.5 rounded-lg text-gray-500 hover:text-white hover:bg-white/5 disabled:opacity-20"><RotateCcw size={14} /></button>
    </div>
  );
}

function FontField({ label, font, busy, onFamily, onFile, t }) {
  const input = useRef(null);
  return (
    <div className="space-y-1.5">
      <span className="text-sm text-gray-400">{label}</span>
      <div className="flex gap-2">
        <select className={`${inputCls} flex-1`} value={font?.family ?? (font?.file ? "__file" : "")}
          onChange={(e) => (e.target.value === "__upload" ? input.current?.click() : e.target.value !== "__file" && onFamily(e.target.value || null))}>
          <option value="">{t.te_f_default}</option>
          {Object.entries(FONT_GROUPS).map(([group, fonts]) => (
            <optgroup key={group} label={t[`te_f_group_${group}`]}>
              {fonts.map((f) => <option key={f} value={f}>{f}</option>)}
            </optgroup>
          ))}
          {font?.file && <option value="__file">{fmt(t.te_f_file, { name: font.file.replace(/^theme-/, "") })}</option>}
          <option value="__upload">{t.te_f_upload}</option>
        </select>
        {busy && <Loader2 size={18} className="animate-spin self-center" />}
      </div>
      <input ref={input} type="file" accept=".woff2,.woff,.ttf,.otf" className="hidden" onChange={(e) => { onFile(e.target.files[0]); e.target.value = ""; }} />
    </div>
  );
}

function PictureField({ image, url, busy, onFile, onRemove, t }) {
  const input = useRef(null);
  return (
    <div className="space-y-2">
      <span className="text-sm text-gray-400">{t.te_bg_image}</span>
      {url && <img src={url} alt="" className="w-full aspect-video object-cover rounded-lg border border-white/10" />}
      <div className="flex gap-2">
        <button type="button" className={btnCls} onClick={() => input.current?.click()} disabled={busy}>
          {busy ? <Loader2 size={16} className="animate-spin" /> : <Upload size={16} />} {image ? t.te_bg_replace : t.te_bg_upload}
        </button>
        {image && <button type="button" className={btnCls} onClick={onRemove}><Trash2 size={16} /> {t.te_remove}</button>}
      </div>
      <input ref={input} type="file" accept=".jpg,.jpeg,.png,.webp,.avif" className="hidden" onChange={(e) => { onFile(e.target.files[0]); e.target.value = ""; }} />
    </div>
  );
}

function Slider({ label, value, min, max, step, percent = false, onChange }) {
  return (
    <label className="block">
      <span className="flex justify-between text-xs text-gray-400 mb-1">
        <span>{label}</span>
        <span className="font-mono">{percent ? `${Math.round(value * 100)} %` : value}</span>
      </span>
      <input type="range" min={min} max={max} step={step} value={value} onChange={(e) => onChange(Number(e.target.value))} className="w-full accent-yellow-500 cursor-pointer" />
    </label>
  );
}

function Choice({ label, value, options, name, onChange }) {
  return (
    <div className="space-y-1.5">
      {label && <span className="text-sm text-gray-400">{label}</span>}
      <div className="flex flex-wrap gap-2">
        {options.map((o) => (
          <button key={o ?? "none"} type="button" onClick={() => onChange(o)}
            className={`flex-1 px-3 py-2 rounded-lg border text-sm font-bold ${value === o ? "bg-yellow-500 text-black border-yellow-500" : "bg-black/20 text-gray-300 border-white/10 hover:bg-white/5"}`}>
            {name(o)}
          </button>
        ))}
      </div>
    </div>
  );
}

// Tags as chips: typing a comma or Enter adds one (made into a-z, 0-9 and dashes, like the file keeps them)
function TagsInput({ tags, onChange, placeholder }) {
  const [text, setText] = useState("");
  const add = (value) => {
    const tag = slug(value);
    if (tag && !tags.includes(tag) && tags.length < LIMITS.tags) onChange([...tags, tag]);
    setText("");
  };
  return (
    <div className="flex flex-wrap items-center gap-1.5 bg-black/40 border border-white/10 rounded-xl px-3 py-2">
      {tags.map((tag) => (
        <span key={tag} className="flex items-center gap-1 pl-2 pr-1 py-0.5 rounded-full bg-white/10 text-sm">
          #{tag}
          <button type="button" onClick={() => onChange(tags.filter((x) => x !== tag))} className="p-0.5 rounded-full hover:bg-white/20" aria-label={tag}><X size={12} /></button>
        </span>
      ))}
      <input value={text} placeholder={tags.length ? "" : placeholder} className="flex-1 min-w-24 bg-transparent py-1 text-white placeholder-gray-600 focus:outline-none"
        onChange={(e) => (e.target.value.endsWith(",") ? add(e.target.value.slice(0, -1)) : setText(e.target.value))}
        onKeyDown={(e) => { if (e.key === "Enter") { e.preventDefault(); add(text); } if (e.key === "Backspace" && !text && tags.length) onChange(tags.slice(0, -1)); }}
        onBlur={() => text && add(text)} />
    </div>
  );
}

// Sharing to the workshop: the theme file is downloaded, and the GitHub form opens filled in, for the
// file to be attached there; the theme appears in the gallery once it has been reviewed
function ShareDialog({ theme, issues, lang, onExport, onClose, t }) {
  const missing = !themeName(theme, lang).trim() || !theme.author.trim();
  const share = async () => {
    await onExport();
    window.open(shareUrl(theme, lang), "_blank", "noopener");
  };
  return (
    <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4" onClick={onClose}>
      <div className="w-full max-w-lg rounded-2xl bg-[#0d0d10] border border-white/15 p-6 space-y-4" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-start gap-3">
          <h2 className="flex-1 text-xl font-bold">{t.te_share_title}</h2>
          <button type="button" onClick={onClose} aria-label={t.chk_close} className="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-white/10"><X size={18} /></button>
        </div>
        <ol className="list-decimal pl-5 space-y-2 text-gray-300">
          <li>{t.te_share_step1}</li>
          <li>{t.te_share_step2}</li>
          <li>{t.te_share_step3}</li>
        </ol>
        <p className="text-sm text-gray-500">{t.te_share_github}</p>
        {missing && <p className="flex gap-2 text-sm text-yellow-300"><AlertTriangle size={16} className="shrink-0 mt-0.5" /> {t.te_share_missing}</p>}
        {issues.length > 0 && <p className="flex gap-2 text-sm text-yellow-300"><AlertTriangle size={16} className="shrink-0 mt-0.5" /> {t.te_share_readability}</p>}
        <button type="button" onClick={share} disabled={missing} className={`${btnCls} w-full bg-yellow-500 hover:bg-yellow-400 text-black font-bold`}>
          <ExternalLink size={16} /> {t.te_share_go}
        </button>
      </div>
    </div>
  );
}
