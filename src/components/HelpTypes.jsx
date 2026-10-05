import { useState, useEffect, useMemo } from "react";
import { MotionConfig } from "framer-motion";
import { X } from "lucide-react";
import { TRANSLATIONS } from "../data/translations";
import { QuizContext } from "../quiz/context";
import { useDisplaySettings } from "../hooks/useDisplay";
import { StageFrame } from "./Stage";
import QuestionScreen from "./QuestionScreen";
import Rich from "./RichText";

// Sample files shipped with the demo in public/source/
const MEDIA = { Image: "pad_BM2.webp", PImage: "1944paris.jpg", Audio: "audio.m4a", Video: "video.mp4", PVideo: "video.mp4" };
const resolveMedia = (name) => (name ? `${import.meta.env.BASE_URL}source/${name}` : null);

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

  // One round holding an example question for every type, in table order
  const round = useMemo(() => ({
    id: "help-types",
    number: "1",
    title: "",
    questions: section.table.rows.map((row, i) => {
      const type = row[0].replace(/`/g, "");
      const example = section.examples[type] || {};
      return { id: String(i + 1), type, text: example.text, options: example.options || {}, answer: example.answer ?? "", media: MEDIA[type] || null };
    }),
  }), [section]);
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
