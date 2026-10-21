import { Plus, X, Shuffle, AlertTriangle } from "lucide-react";
import { TYPES, withLetters } from "../quiz/sheetModel";
import { MAX_SORT_ITEMS } from "../quiz/sort";
import { isUrl } from "../quiz/model";
import { inputCls } from "./EditorParts";

const smallBtn = "flex items-center gap-1.5 px-3 py-2 rounded-lg bg-white/10 hover:bg-white/20 text-xs disabled:opacity-40";
const MEDIA_TYPES = ["PImage", "PVideo", "Audio", "Image", "Video"];
const NO_TEXT_NEEDED = ["PImage", "PVideo", "Handout"];

const Label = ({ children, hint }) => (
  <span className="block text-xs font-bold uppercase tracking-wider text-gray-400 mb-1.5">
    {children}{hint && <span className="normal-case tracking-normal font-normal text-gray-500"> · {hint}</span>}
  </span>
);

// Zdroj path with suggestions from the synced media, and a note when the file is not there
function MediaInput({ value, onChange, hasMedia, placeholder, t }) {
  const missing = value.trim() && !isUrl(value) && !hasMedia(value.trim());
  return (
    <div className="space-y-1">
      <input value={value} onChange={(e) => onChange(e.target.value)} list="qe-media" placeholder={placeholder || t.qe_media_hint} className={inputCls} />
      {missing && <p className="flex items-start gap-1.5 text-xs text-yellow-300/90"><AlertTriangle size={14} className="shrink-0 mt-px" /> {t.qe_media_missing}</p>}
    </div>
  );
}

// The fields of one question, depending on its type. typeHelp: { [type]: "what it looks like" }
export default function QuestionForm({ draft, onChange, mediaNames, hasMedia, typeHelp, disabled, t }) {
  const { type } = draft;
  const set = (fields) => onChange({ ...draft, ...fields });

  const setAnswerAt = (i, value) => set({ answers: draft.answers.map((a, j) => (j === i ? value : a)) });
  const setItem = (i, fields) => set({ items: draft.items.map((it, j) => (j === i ? { ...it, ...fields } : it)) });
  // Adding or removing an item changes how many letters there are, so they are mixed again
  const resizeItems = (items) => set({ items: withLetters(items) });

  return (
    <fieldset disabled={disabled} className="space-y-5 min-w-0">
      <datalist id="qe-media">{mediaNames.map((name) => <option key={name} value={name} />)}</datalist>

      <div>
        <Label>{t.qe_type}</Label>
        <div className="flex flex-wrap gap-1.5">
          {TYPES.map((name) => (
            <button key={name} type="button" onClick={() => set({ type: name })}
              className={`px-3 py-1.5 rounded-lg text-sm ${type === name ? "bg-yellow-500 text-black font-bold" : "bg-white/5 text-gray-300 hover:bg-white/10"}`}>
              {name}
            </button>
          ))}
        </div>
        {typeHelp[type] && <p className="text-xs text-gray-500 mt-2">{typeHelp[type]}</p>}
      </div>

      <label className="block">
        <Label hint={NO_TEXT_NEEDED.includes(type) ? t.qe_optional : null}>{t.qe_text}</Label>
        <textarea value={draft.text} onChange={(e) => set({ text: e.target.value })} rows={3} className={`${inputCls} resize-y`} />
      </label>

      {type === "ABCD" && (
        <div className="space-y-2">
          <Label>{t.qe_options}</Label>
          {["A", "B", "C", "D"].map((o) => (
            <div key={o} className="flex items-center gap-2">
              <button type="button" onClick={() => set({ answer: o })} title={t.qe_correct}
                className={`w-10 h-10 shrink-0 rounded-lg font-black ${draft.answer.trim().toUpperCase() === o ? "bg-green-500 text-black" : "bg-white/10 text-gray-300 hover:bg-white/20"}`}>
                {o}
              </button>
              <input value={draft.options[o]} onChange={(e) => set({ options: { ...draft.options, [o]: e.target.value } })} className={inputCls} />
            </div>
          ))}
          <p className="text-xs text-gray-500">{t.qe_correct_hint}</p>
        </div>
      )}

      {type === "Yes/No" && (
        <div>
          <Label>{t.qe_answer}</Label>
          <div className="flex gap-2">
            {["Yes", "No"].map((v) => (
              <button key={v} type="button" onClick={() => set({ answer: v })}
                className={`px-5 py-2.5 rounded-lg font-bold ${draft.answer.trim().toLowerCase() === v.toLowerCase() ? "bg-green-500 text-black" : "bg-white/10 text-gray-300 hover:bg-white/20"}`}>
                {v === "Yes" ? t.qe_yes : t.qe_no}
              </button>
            ))}
          </div>
        </div>
      )}

      {type === "Top5" && (
        <div className="space-y-2">
          <Label hint={t.qe_answers_hint}>{t.qe_answers}</Label>
          {draft.answers.map((answer, i) => (
            <div key={i} className="flex items-center gap-2">
              <span className="w-6 text-right text-sm text-gray-500">{i + 1}.</span>
              <input value={answer} onChange={(e) => setAnswerAt(i, e.target.value)} className={inputCls} />
              <button type="button" onClick={() => set({ answers: draft.answers.filter((_, j) => j !== i) })} disabled={draft.answers.length <= 1}
                className="p-2 rounded-lg text-gray-500 hover:text-red-300 hover:bg-red-500/20 disabled:opacity-30" aria-label={t.qe_remove}><X size={16} /></button>
            </div>
          ))}
          <button type="button" onClick={() => set({ answers: [...draft.answers, ""] })} disabled={draft.answers.length >= 10} className={smallBtn}>
            <Plus size={14} /> {t.qe_add_answer}
          </button>
        </div>
      )}

      {type === "Sort" && (
        <div className="space-y-2">
          <Label hint={t.qe_items_hint}>{t.qe_items}</Label>
          {draft.items.map((item, i) => (
            <div key={i} className="flex items-start gap-2">
              <span className="w-6 pt-3 text-right text-sm text-gray-500">{i + 1}.</span>
              <span className="w-10 h-[50px] shrink-0 flex items-center justify-center rounded-lg bg-white/10 font-black text-yellow-400" title={t.qe_letter}>{item.letter}</span>
              <div className="flex-1 min-w-0 grid sm:grid-cols-[3fr_2fr] gap-2">
                <input value={item.text} onChange={(e) => setItem(i, { text: e.target.value })} placeholder={t.qe_item} className={inputCls} />
                <MediaInput value={item.media} onChange={(media) => setItem(i, { media })} hasMedia={hasMedia} placeholder={t.qe_item_media} t={t} />
              </div>
              <button type="button" onClick={() => resizeItems(draft.items.filter((_, j) => j !== i))} disabled={draft.items.length <= 2}
                className="mt-1.5 p-2 rounded-lg text-gray-500 hover:text-red-300 hover:bg-red-500/20 disabled:opacity-30" aria-label={t.qe_remove}><X size={16} /></button>
            </div>
          ))}
          <div className="flex flex-wrap gap-2">
            <button type="button" onClick={() => resizeItems([...draft.items, { text: "", media: "" }])} disabled={draft.items.length >= MAX_SORT_ITEMS} className={smallBtn}>
              <Plus size={14} /> {t.qe_add_item}
            </button>
            <button type="button" onClick={() => resizeItems(draft.items)} className={smallBtn}>
              <Shuffle size={14} /> {t.qe_shuffle}
            </button>
          </div>
          <p className="text-xs text-gray-500">{t.qe_letters_hint}</p>
        </div>
      )}

      {type === "Handout" && (
        <>
          <div><Label>{t.qe_handout_picture}</Label><MediaInput value={draft.media} onChange={(media) => set({ media })} hasMedia={hasMedia} t={t} /></div>
          <div><Label>{t.qe_handout_answer}</Label><MediaInput value={draft.answer} onChange={(answer) => set({ answer })} hasMedia={hasMedia} t={t} /></div>
        </>
      )}

      {!["ABCD", "Yes/No", "Top5", "Sort", "Handout"].includes(type) && (
        <label className="block">
          <Label>{t.qe_answer}</Label>
          <input value={draft.answer} onChange={(e) => set({ answer: e.target.value })} className={inputCls} />
        </label>
      )}

      {!["Sort", "Handout"].includes(type) && (
        <div>
          <Label hint={MEDIA_TYPES.includes(type) ? null : t.qe_optional}>{t.qe_media}</Label>
          <MediaInput value={draft.media} onChange={(media) => set({ media })} hasMedia={hasMedia} t={t} />
        </div>
      )}
    </fieldset>
  );
}
