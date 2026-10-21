import { useState } from "react";
import { PenLine, Hash, LayoutGrid, ThumbsUp, ListOrdered, ArrowUpDown, Image as ImageIcon, Maximize, Film, Clapperboard, Music, Printer, ChevronRight } from "lucide-react";

// The question types in the order and groups the editor offers them, with what the show does in each
// step of the preview (QuestionScreen; PImage and PVideo show their media alone first). code: the
// part of the translation keys qt_<code>_name / _desc and the optional step labels qt_<code>_<step>.
const TYPE_GROUPS = [
  ["qt_group_answer", [
    { type: "Written", code: "written", icon: PenLine },
    { type: "Numeric", code: "numeric", icon: Hash },
    { type: "ABCD", code: "abcd", icon: LayoutGrid },
    { type: "Yes/No", code: "yesno", icon: ThumbsUp },
    { type: "Top5", code: "top5", icon: ListOrdered },
    { type: "Sort", code: "sort", icon: ArrowUpDown },
  ]],
  ["qt_group_media", [
    { type: "Image", code: "image", icon: ImageIcon },
    { type: "PImage", code: "pimage", icon: Maximize, steps: ["media", "question", "answer"] },
    { type: "Audio", code: "audio", icon: Music },
    { type: "Video", code: "video", icon: Film },
    { type: "PVideo", code: "pvideo", icon: Clapperboard, steps: ["media", "question", "answer"] },
    { type: "Handout", code: "handout", icon: Printer },
  ]],
];
const ALL = TYPE_GROUPS.flatMap(([, types]) => types);
const typeInfo = (type) => ALL.find((info) => info.type === type) ?? { type, code: "", icon: PenLine };

function TypeCard({ info, selected, onClick, t }) {
  const Icon = info.icon;
  return (
    <button type="button" onClick={onClick} aria-pressed={selected}
      className={`text-left flex gap-3 p-3 rounded-xl border transition-colors ${selected ? "border-yellow-500 bg-yellow-500/10" : "border-white/10 bg-white/[0.03] hover:bg-white/[0.07]"}`}>
      <Icon size={20} className={`shrink-0 mt-0.5 ${selected ? "text-yellow-400" : "text-gray-400"}`} />
      <span className="min-w-0">
        <span className="flex flex-wrap items-baseline gap-x-2">
          <span className="font-bold text-sm text-white">{t[`qt_${info.code}_name`] || info.type}</span>
          <code className="text-[10px] text-gray-500">{info.type}</code>
        </span>
        <span className="block text-xs text-gray-400 mt-0.5">{t[`qt_${info.code}_desc`]}</span>
      </span>
    </button>
  );
}

// Type cards in groups; once a type is chosen they fold into that card (open: start unfolded, for new
// questions). Below, how the chosen type plays, step by step; a step shows that moment in the preview.
export default function QuestionTypePicker({ type, onChange, view, onView, open: startOpen = false, t }) {
  const [open, setOpen] = useState(startOpen);
  const info = typeInfo(type);
  const steps = info.steps ?? ["question", "answer"];
  const stepLabel = (step) => t[`qt_${info.code}_${step}`] || t[`qt_step_${step}`];

  return (
    <div className="space-y-3">
      {open ? (
        <div className="space-y-4">
          {TYPE_GROUPS.map(([group, types]) => (
            <div key={group} className="space-y-2">
              <p className="text-xs text-gray-500">{t[group]}</p>
              <div className="grid sm:grid-cols-2 gap-2">
                {types.map((item) => (
                  <TypeCard key={item.type} info={item} selected={item.type === type} t={t}
                    onClick={() => { onChange(item.type); setOpen(false); }} />
                ))}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="grid sm:grid-cols-[1fr_auto] gap-2 items-stretch">
          <TypeCard info={info} selected t={t} onClick={() => setOpen(true)} />
          <button type="button" onClick={() => setOpen(true)} className="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">{t.qt_change}</button>
        </div>
      )}

      <div className="flex flex-wrap items-center gap-1.5 text-xs">
        <span className="text-gray-500 mr-1">{t.qt_how}</span>
        {steps.map((step, i) => (
          <span key={step} className="flex items-center gap-1.5">
            {i > 0 && <ChevronRight size={12} className="text-gray-600" />}
            <button type="button" onClick={() => onView?.(step)}
              className={`flex items-center gap-1.5 px-2.5 py-1 rounded-full ${view === step ? "bg-yellow-500 text-black font-bold" : "bg-white/5 text-gray-300 hover:bg-white/10"}`}>
              <span className="font-mono">{i + 1}</span> {stepLabel(step)}
            </button>
          </span>
        ))}
      </div>
    </div>
  );
}
