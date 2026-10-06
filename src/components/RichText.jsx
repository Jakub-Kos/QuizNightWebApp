import { Fragment } from "react";

// "Use `Shift+P`" -> text with <code> for the parts in backticks
export default function RichText({ text }) {
  return String(text).split("`").map((part, i) => (i % 2 ? (
    <code key={i} className="px-1.5 py-0.5 rounded bg-white/10 text-yellow-200 font-mono text-[0.9em] whitespace-nowrap">{part}</code>
  ) : <Fragment key={i}>{part}</Fragment>));
}
