// Sort questions: items are stored in the correct order and shown mixed up, labelled A, B, C…
// The sheet normally gives each item its letter (the answer column), so whoever marks the answer sheets
// reads the answer straight from the sheet. Without letters the items are mixed automatically, derived
// from the question itself, so every window shows the same order.

export const MAX_SORT_ITEMS = 10;

// "ok" when the items carry the letters A.. in some order, "none" when no item has a letter, else "invalid"
export function sheetLetters(items) {
  if (items.every((it) => !it.letter)) return "none";
  const expected = items.map((_, i) => String.fromCharCode(65 + i));
  const letters = items.map((it) => it.letter);
  return [...letters].sort().join() === expected.join() ? "ok" : "invalid";
}

function seededRandom(seedText) {
  let h = 2166136261;
  for (const ch of seedText) h = Math.imul(h ^ ch.codePointAt(0), 16777619);
  return () => {
    h = Math.imul(h ^ (h >>> 15), 2246822507);
    h = Math.imul(h ^ (h >>> 13), 3266489909);
    return ((h ^= h >>> 16) >>> 0) / 4294967296;
  };
}

function automaticOrder(question, items) {
  const random = seededRandom(`${question.id}|${question.text}|${items.map((it) => it.text).join("|")}`);
  const order = items.map((_, i) => i);
  for (let i = order.length - 1; i > 0; i--) {
    const j = Math.floor(random() * (i + 1));
    [order[i], order[j]] = [order[j], order[i]];
  }
  // Never show the items already in the right order
  if (order.length > 1 && order.every((v, i) => v === i)) order.push(order.shift());
  return order;
}

// { shown: [{ ...item, letter, place }] in display order, correct: same objects in the correct order,
//   answer: "C A D B", fromSheet } — place is the 1-based correct position, letter the label on screen
export function sortLayout(question) {
  const items = question.items || [];
  const fromSheet = sheetLetters(items) === "ok";
  const order = fromSheet
    ? items.map((it, i) => ({ i, letter: it.letter })).sort((a, b) => a.letter.localeCompare(b.letter)).map((x) => x.i)
    : automaticOrder(question, items);

  const shown = order.map((index, k) => ({ ...items[index], letter: String.fromCharCode(65 + k), place: index + 1 }));
  const correct = [...shown].sort((a, b) => a.place - b.place);
  return { shown, correct, answer: correct.map((it) => it.letter).join(" "), fromSheet };
}
