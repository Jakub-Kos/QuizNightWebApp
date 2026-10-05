import { useState, useEffect, useMemo } from "react";
import { MotionConfig } from "framer-motion";
import { X } from "lucide-react";
import Papa from "papaparse";
import { TRANSLATIONS } from "../data/translations";
import { QuizContext } from "../quiz/context";
import { parseQuestionsCsv } from "../quiz/parse";
import { useDisplaySettings } from "../hooks/useDisplay";
import { StageFrame } from "./Stage";
import QuestionScreen from "./QuestionScreen";
import Rich from "./RichText";

// Sample files shipped with the demo in public/source/
const MEDIA = { Image: "pad_BM2.webp", PImage: "1944paris.jpg", Audio: "audio.m4a", Video: "video.mp4", PVideo: "video.mp4" };
const resolveMedia = (name) => (name ? `${import.meta.env.BASE_URL}source/${name}` : null);

// Header of the questions sheet template (public/templates), with the column letters of Google Sheets
const SHEET_HEAD = ["Kolo", "Q#", "Otázka", "Typ Odpovědi", "A", "B", "C", "D", "Správná odpověď", "Zdroj"];
const COLUMN_LETTERS = "ABCDEFGHIJ".split("");

// The sheet rows that make up a type's example, written the way the help table tells users to
function exampleRows(type, example) {
  const options = ["A", "B", "C", "D"].map((o) => example.options?.[o] ?? "");
  const row = (number, text, typeCell, opts, answer, media) => ["", String(number), text, typeCell, ...opts, answer, media];
  const none = ["", "", "", ""];
  if (type === "Top5") {
    const [first, ...rest] = example.answer;
    return [row(1, example.text, type, none, first, ""), ...rest.map((answer, i) => row(i + 2, "", "", none, answer, ""))];
  }
  if (type === "Sort") {
    return example.items.map((item, i) => row(i + 1, i ? "" : example.text, i ? "" : type, [item, "", "", ""], example.letters?.[i] ?? "", ""));
  }
  return [row(1, example.text, type, options, example.answer ?? "", MEDIA[type] || "")];
}

// Steps of QuestionScreen worth showing per type: PImage/PVideo show their media alone first
function framesFor(type, labels) {
  return type === "PImage" || type === "PVideo"
    ? [[labels.media, 1], [labels.question, 2], [labels.answer, 3]]
    : [[labels.question, 1], [labels.answer, 2]];
}

// The question-type table of the help page, with live previews rendered by the real question screen.
// section: the "types" help section (table rows: [type, on screen, fill in] + examples per type).
export default function HelpTypes({ section, lang, t }) {
  const [display] = useDisplaySettings("main");
  const showT = TRANSLATIONS[lang] || TRANSLATIONS.en;
  const labels = { media: section.mediaStep, question: t.chk_view_question, answer: t.chk_view_answer };
  const [open, setOpen] = useState(null); // { qIndex, step }

  // Sheet rows of every type's example, and one round holding the questions the real parser makes of
  // them, in table order: the previews are exactly what those rows produce
  const sheets = useMemo(() => section.table.rows.map(([type]) => {
    type = type.replace(/`/g, "");
    return exampleRows(type, section.examples[type] || {});
  }), [section]);
  const round = useMemo(() => ({
    id: "help-types",
    number: "1",
    title: "",
    questions: sheets.flatMap((rows) => {
      const marked = rows.map((r, i) => (i ? r : ["Kolo 1 - Help", ...r.slice(1)]));
      return parseQuestionsCsv(Papa.unparse([SHEET_HEAD, ...marked]))[0]?.questions ?? [];
    }),
  }), [sheets]);
  const context = useMemo(() => ({ quiz: {}, rounds: [round], teams: [], resolveMedia }), [round]);

  useEffect(() => {
    if (!open) return;
    const onKey = (e) => e.key === "Escape" && setOpen(null);
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  const frame = (qIndex, step) => (
    <QuestionScreen roundData={round} mode="with_answers" onBack={() => {}} isPresenter={false} t={showT} preview={{ qIndex, step }} />
  );

  return (
    <QuizContext.Provider value={context}>
      {/* Previews show the end state of each step instead of animating */}
      <MotionConfig reducedMotion="always">
        <p className="text-sm text-gray-500">{section.previewHint}</p>
        <div className="space-y-4">
          {section.table.rows.map(([type, onScreen, fillIn], qIndex) => (
            <article key={type} className="rounded-xl border border-white/10 bg-white/[0.02] p-4 space-y-3">
              <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1 text-sm">
                <h3 className="text-base"><Rich text={type} /></h3>
                <p className="text-gray-300"><Rich text={onScreen} /></p>
                <p className="text-gray-400 basis-full"><span className="text-gray-500">{section.table.head[2]}:</span> <Rich text={fillIn} /></p>
              </div>
              <SheetRows rows={sheets[qIndex]} label={section.sheetLabel} />
              {/* Frames share the full width: two per row for most types, three when the media comes first */}
              <div className={`grid gap-3 items-start ${framesFor(round.questions[qIndex].type, labels).length === 3 ? "sm:grid-cols-3" : "sm:grid-cols-2"}`}>
                {framesFor(round.questions[qIndex].type, labels).map(([label, step]) => (
                  // A div with button semantics: the screen inside renders its own <button>s
                  <div key={step} role="button" tabIndex={0} onClick={() => setOpen({ qIndex, step })}
                    onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), setOpen({ qIndex, step }))}
                    className="group text-left space-y-1 cursor-pointer focus:outline-none">
                    <StageFrame display={display} className="pointer-events-none rounded-md ring-1 ring-white/10 group-hover:ring-2 group-hover:ring-white/60 group-focus-visible:ring-2 group-focus-visible:ring-white">
                      {frame(qIndex, step)}
                    </StageFrame>
                    <span className="text-xs text-gray-500">{label}</span>
                  </div>
                ))}
              </div>
            </article>
          ))}
        </div>

        {open && (
          <div className="fixed inset-0 z-50 bg-black/90 backdrop-blur-sm text-white flex items-center justify-center p-4" onClick={() => setOpen(null)}>
            <div className="w-full flex flex-col gap-3" style={{ maxWidth: "min(1600px, calc((100vh - 120px) * 16 / 9))" }} onClick={(e) => e.stopPropagation()}>
              <div className="flex justify-end">
                <button onClick={() => setOpen(null)} className="p-2 rounded-full hover:bg-white/10" aria-label={t.chk_close}><X size={20} /></button>
              </div>
              <StageFrame display={display} className="rounded-xl ring-1 ring-white/20">{frame(open.qIndex, open.step)}</StageFrame>
            </div>
          </div>
        )}
      </MotionConfig>
    </QuizContext.Provider>
  );
}

// The example's rows as they look in Google Sheets: column letters, the template header, then the rows
function SheetRows({ rows, label }) {
  const cell = "border border-white/10 px-1.5 py-1 whitespace-nowrap";
  return (
    <div className="space-y-1">
      <span className="text-xs text-gray-500">{label}</span>
      <div className="overflow-x-auto [color-scheme:dark]">
        <table className="text-xs font-mono border-collapse">
          <thead>
            <tr className="bg-white/[0.06] text-gray-500 text-center">
              <th className={`${cell} w-8`} />
              {COLUMN_LETTERS.map((letter) => <th key={letter} className={`${cell} font-normal`}>{letter}</th>)}
            </tr>
            <tr className="text-gray-400">
              <td className={`${cell} bg-white/[0.06] text-gray-500 text-center`}>1</td>
              {SHEET_HEAD.map((name) => <td key={name} className={`${cell} font-bold`}>{name}</td>)}
            </tr>
          </thead>
          <tbody>
            {rows.map((row, r) => (
              <tr key={r}>
                <td className={`${cell} bg-white/[0.06] text-gray-500 text-center`}>{r + 2}</td>
                {row.map((value, c) => <td key={c} className={`${cell} ${value ? "text-yellow-200" : ""}`}>{value}</td>)}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
