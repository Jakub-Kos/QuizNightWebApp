import { buildQuestions, isRoundMarker, isQuestionRow, splitMarker, normalizeType } from "./parse";
import { sortLayout, sheetLetters } from "./sort";

// The questions tab as rows, for the question editor. Columns A–J are the questions.csv columns
// (see parse.js); the editor adds who last changed the question and when. Here those two are always
// columns 10 and 11 of a row; in the Sheet they are wherever their headers are (stampColumns), so a
// Sheet that already uses K and L for something else keeps it.
// Every edit is a plan: replace `deleteCount` rows at `start` with `insert`, then write single cells
// (`fixups`, in row numbers after the replacement). Rows are 0-based here, 1-based in the Sheet.

export const COLUMNS = 12;
export const AUTHOR_COL = 10;
export const CHANGED_COL = 11;
const QUESTION_COLUMNS = 10;

// The headers the editor writes (qe_col_author / qe_col_changed), in every UI language
const AUTHOR_HEADER = /^(author|autor|szerző)$/i;
const CHANGED_HEADER = /^(changed|upraveno|módosítva)$/i;

// Where the author and time columns are in the Sheet's rows: the columns with those headers, else the
// first columns right of A–J that are empty in every row. { author, changed }, 0-based.
export function stampColumns(rows) {
  const headerRow = rows.find((row) => cell(row, 1).toLowerCase() === "q#") || [];
  const find = (pattern) => headerRow.findIndex((c, i) => i >= QUESTION_COLUMNS && pattern.test(String(c ?? "").trim()));
  let author = find(AUTHOR_HEADER);
  let changed = find(CHANGED_HEADER);
  const taken = (i) => i === author || i === changed || rows.some((row) => cell(row, i));
  const nextFree = () => {
    let i = QUESTION_COLUMNS;
    while (taken(i)) i++;
    return i;
  };
  if (author < 0) author = nextFree();
  if (changed < 0) changed = nextFree();
  return { author, changed };
}

// Sheet rows -> rows of this module (A–J, author, time)
export const toModelRows = (rows, cols) =>
  rows.map((row) => [...Array.from({ length: QUESTION_COLUMNS }, (_, i) => row?.[i] ?? ""), row?.[cols.author] ?? "", row?.[cols.changed] ?? ""]);

const cell = (row, i) => String(row?.[i] ?? "").trim();
const pad = (row) => Array.from({ length: COLUMNS }, (_, i) => String(row?.[i] ?? ""));

// Rows compared the way the Sheet API returns them (trailing empty cells are left out). The round marker
// and Q# are left out: renumbering or a moved marker does not change the question itself.
const sameRow = (a, b) => pad(a).every((v, i) => i < 2 || v.trim() === cell(b, i));
export const sameRows = (a, b) => a.length === b.length && a.every((row, i) => sameRow(row, b[i]));

// rows -> { rows, headerRow, rounds: [{ marker, markerRow, word, number, title, questions: [block] }] }
// block: { start, end, rows, question, author, changed }, the rows one question takes (several for Top5
// and Sort), grouped exactly like parseQuestionsCsv groups them
export function readSheet(rows) {
  const rounds = [];
  let current = null;
  let headerRow = -1;

  rows.forEach((row, index) => {
    const col0 = cell(row, 0);
    const col1 = cell(row, 1);
    if (headerRow < 0 && col1.toLowerCase() === "q#") headerRow = index;
    if (isRoundMarker(col0, col1)) {
      current = { marker: col0, markerRow: index, ...splitMarker(col0), questions: [] };
      rounds.push(current);
    }
    if (!current || !isQuestionRow(row)) return;

    const last = current.questions.at(-1);
    const lastType = last && normalizeType(cell(last.rows[0], 3));
    if (last && (lastType === "Top5" || lastType === "Sort") && !cell(row, 2) && !cell(row, 3)) {
      last.end = index;
      last.rows.push(row);
    } else {
      current.questions.push({ start: index, end: index, rows: [row] });
    }
  });

  rounds.forEach((round) => round.questions.forEach((block) => {
    block.question = buildQuestions(block.rows)[0];
    block.author = cell(block.rows[0], AUTHOR_COL);
    block.changed = cell(block.rows[0], CHANGED_COL);
  }));
  return { rows, headerRow, rounds };
}

// Editable form of a question
export function draftFrom(question) {
  const q = question;
  const sortItems = () => {
    // Items without letters get the ones the show would use, so the Sheet then holds them
    const layout = q.items.some((it) => !it.letter) ? sortLayout(q).correct : q.items;
    return layout.map((it) => ({ text: it.text || "", letter: it.letter || "", media: it.media || "" }));
  };
  return {
    type: q.type,
    text: String(q.text ?? ""),
    options: { A: String(q.options?.A ?? ""), B: String(q.options?.B ?? ""), C: String(q.options?.C ?? ""), D: String(q.options?.D ?? "") },
    answer: q.type === "Top5" ? "" : String(q.answer ?? ""),
    media: q.media || "",
    answers: q.type === "Top5" ? q.answer.map((a) => String(a ?? "")) : ["", "", "", "", ""],
    items: q.type === "Sort" && q.items?.length ? sortItems() : withLetters([{ text: "", media: "" }, { text: "", media: "" }, { text: "", media: "" }]),
  };
}

export const emptyDraft = (type = "Written") => draftFrom({ type, text: "", options: {}, answer: type === "Top5" ? ["", "", "", "", ""] : "", media: null, items: [] });

// Letters A, B, C… in a random order that is not A, B, C… itself (the check warns about that)
export function withLetters(items) {
  const letters = items.map((_, i) => String.fromCharCode(65 + i));
  if (items.length > 1) {
    do {
      for (let i = letters.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [letters[i], letters[j]] = [letters[j], letters[i]];
      }
    } while (letters.every((l, i) => l === String.fromCharCode(65 + i)));
  }
  return items.map((it, i) => ({ ...it, letter: letters[i] }));
}

// Draft -> sheet rows (12 columns, A and K/L empty), numbered from `number`
export function rowsFor(draft, number) {
  const n = Number(number) || 1;
  const row = (i, text, type, options, answer, media) => pad(["", String(n + i), text, type, ...options, answer, media]);
  const none = ["", "", "", ""];

  if (draft.type === "Top5") {
    const answers = draft.answers.map((a) => a.trim()).filter(Boolean);
    // "Top5" for five answers, else "Top10" and so on, so the Sheet says how long the list is
    const type = answers.length === 5 || !answers.length ? "Top5" : `Top${answers.length}`;
    return (answers.length ? answers : [""]).map((answer, i) => row(i, i ? "" : draft.text, i ? "" : type, none, answer, i ? "" : draft.media.trim()));
  }
  if (draft.type === "Sort") {
    let items = draft.items.filter((it) => it.text.trim() || it.media.trim());
    // Empty items left out: the letters must still be A, B, C… in some order
    if (sheetLetters(items) !== "ok") items = withLetters(items);
    return (items.length ? items : [{ text: "", letter: "", media: "" }]).map((it, i) =>
      row(i, i ? "" : draft.text, i ? "" : "Sort", [it.text.trim(), "", "", ""], it.letter, it.media.trim()));
  }
  const options = draft.type === "ABCD" ? ["A", "B", "C", "D"].map((o) => draft.options[o]) : none;
  return [row(0, draft.text, draft.type, options, draft.answer.trim(), draft.media.trim())];
}

// The question as the show will see it
export const previewQuestion = (draft) => buildQuestions(rowsFor(draft, 1))[0];

export const roundMarker = (word, number, title) => `${word || "Kolo"} ${number}${title.trim() ? ` - ${title.trim()}` : ""}`;

// "2026-10-21 19:30" in local time: readable in the Sheet and by Date
export function stampNow(date = new Date()) {
  const p = (v) => String(v).padStart(2, "0");
  return `${date.getFullYear()}-${p(date.getMonth() + 1)}-${p(date.getDate())} ${p(date.getHours())}:${p(date.getMinutes())}`;
}
export const parseStamp = (stamp) => {
  const d = new Date(String(stamp).replace(" ", "T"));
  return isNaN(d) ? null : d;
};

const stamped = (rows, author, changed) => rows.map((row, i) => {
  const next = pad(row);
  next[AUTHOR_COL] = i ? "" : author;
  next[CHANGED_COL] = i ? "" : changed;
  return next;
});

const questionNumbers = (round) => round.questions.flatMap((b) => b.rows.map((r) => Number(cell(r, 1))));
// Rounds numbered 1, 2, 3… row by row (the usual way: Q# is the answer line) are kept that way
const isConsecutive = (round) => questionNumbers(round).every((n, i) => n === i + 1);

// Apply a replacement to the rows and add the fixups that keep the Sheet tidy: renumbering the round
// when it was numbered consecutively, and the K/L headers
function finish(sheet, roundIndex, { start, deleteCount, insert, fixups = [] }, { renumber = true, headers }) {
  const next = [...sheet.rows.slice(0, start), ...insert, ...sheet.rows.slice(start + deleteCount)];
  const all = [...fixups];
  const set = (row, col, value) => {
    const existing = all.find((f) => f.row === row && f.col === col);
    if (existing) existing.value = value;
    else all.push({ row, col, value });
    next[row] = pad(next[row]);
    next[row][col] = value;
  };
  all.forEach((f) => { next[f.row] = pad(next[f.row]); next[f.row][f.col] = f.value; });

  const oldRound = sheet.rounds[roundIndex];
  if (renumber && oldRound && (oldRound.questions.length === 0 || isConsecutive(oldRound))) {
    const round = readSheet(next).rounds[roundIndex];
    let n = 1;
    round?.questions.forEach((b) => {
      for (let r = b.start; r <= b.end; r++) {
        if (cell(next[r], 1) !== String(n)) set(r, 1, String(n));
        n++;
      }
    });
  }
  if (headers && sheet.headerRow >= 0) {
    if (!cell(next[sheet.headerRow], AUTHOR_COL)) set(sheet.headerRow, AUTHOR_COL, headers.author);
    if (!cell(next[sheet.headerRow], CHANGED_COL)) set(sheet.headerRow, CHANGED_COL, headers.changed);
  }
  return { start, deleteCount, insert, fixups: all, next };
}

// meta: { author, changed, headers: { author, changed } } for K/L

export function planSave(sheet, ri, qi, draft, meta) {
  const block = sheet.rounds[ri].questions[qi];
  const rows = stamped(rowsFor(draft, cell(block.rows[0], 1) || 1), meta.author, meta.changed);
  rows[0][0] = String(block.rows[0][0] ?? ""); // the round marker, when the question carries it
  return finish(sheet, ri, { start: block.start, deleteCount: block.end - block.start + 1, insert: rows }, meta);
}

// A row the template prepared for a question: nothing in it but a Q# (and the round marker on a round's
// first row)
const isPrepared = (row, isMarkerRow) =>
  [2, 3, 4, 5, 6, 7, 8, 9, AUTHOR_COL, CHANGED_COL].every((i) => !cell(row, i)) && (isMarkerRow || !cell(row, 0));

// A new question goes into the first prepared rows after the round's last question (the round's own
// first row when it is still empty); rows are inserted only when the round has no prepared rows left
export function planAdd(sheet, ri, draft, meta) {
  const round = sheet.rounds[ri];
  const last = round.questions.at(-1);
  const rows = stamped(rowsFor(draft, last ? Number(cell(sheet.rows[last.end], 1)) + 1 : 1), meta.author, meta.changed);
  const after = last ? last.end + 1 : round.markerRow;
  const end = sheet.rounds[ri + 1]?.markerRow ?? sheet.rows.length;
  const prepared = (r) => r < end && isPrepared(sheet.rows[r], r === round.markerRow);

  let start = after;
  while (start < end && !prepared(start)) start++;
  if (start >= end) start = after;
  let free = 0;
  while (free < rows.length && prepared(start + free)) free++;
  if (start === round.markerRow) rows[0][0] = round.marker;
  return finish(sheet, ri, { start, deleteCount: free, insert: rows }, meta);
}

// Deleting keeps the round's rows: the questions below move up, and the freed rows at the end of the
// round's questions are left prepared (nothing but their Q#), ready for the next question. Columns A
// (round marker) and B (Q#) stay where they are; only the question cells move.
export function planDelete(sheet, ri, qi, meta) {
  const round = sheet.rounds[ri];
  const block = round.questions[qi];
  const lastRow = round.questions.at(-1).end;
  const range = sheet.rows.slice(block.start, lastRow + 1).map(pad);
  const content = [
    ...sheet.rows.slice(block.end + 1, lastRow + 1).map(pad),
    ...Array.from({ length: block.end - block.start + 1 }, () => pad([])),
  ];
  const rows = range.map((row, i) => [row[0], row[1], ...content[i].slice(2)]);
  return finish(sheet, ri, { start: block.start, deleteCount: rows.length, insert: rows }, meta);
}

export function planRename(sheet, ri, title, meta) {
  const round = sheet.rounds[ri];
  const marker = roundMarker(round.word, round.number, title);
  return finish(sheet, ri, { start: round.markerRow, deleteCount: 0, insert: [], fixups: [{ row: round.markerRow, col: 0, value: marker }] }, { ...meta, renumber: false });
}

// A new round after the last one, with its first question
export function planAddRound(sheet, title, draft, meta) {
  const lastRound = sheet.rounds.at(-1);
  const start = lastRound
    ? (lastRound.questions.at(-1)?.end ?? lastRound.markerRow) + 1
    : Math.max(sheet.headerRow + 1, 0);
  const number = Math.max(0, ...sheet.rounds.map((r) => Number(r.number) || 0)) + 1;
  const rows = stamped(rowsFor(draft, 1), meta.author, meta.changed);
  rows[0][0] = roundMarker(lastRound?.word, number, title);
  return finish(sheet, -1, { start, deleteCount: 0, insert: rows }, meta);
}

// The same question in a newer copy of the Sheet: same contents, nearest to where it was
export function findBlock(sheet, rows, near = { ri: -1, qi: -1 }) {
  let best = null;
  sheet.rounds.forEach((round, ri) => round.questions.forEach((block, qi) => {
    if (!sameRows(rows, block.rows)) return;
    const distance = Math.abs(ri - near.ri) * 1000 + Math.abs(qi - near.qi);
    if (!best || distance < best.distance) best = { ri, qi, distance };
  }));
  return best && { ri: best.ri, qi: best.qi };
}

export const findRound = (sheet, marker) => sheet.rounds.findIndex((r) => r.marker === marker);

// Swap a question with its neighbour in the round (dir -1 up, +1 down). Both are rewritten in one
// replacement; the round marker stays on the first row, and the round is renumbered as usual.
export function planMove(sheet, ri, qi, dir, meta) {
  const round = sheet.rounds[ri];
  const [first, second] = dir < 0 ? [round.questions[qi - 1], round.questions[qi]] : [round.questions[qi], round.questions[qi + 1]];
  const between = sheet.rows.slice(first.end + 1, second.start); // skipped (empty) rows stay between them
  const rows = [...second.rows, ...between, ...first.rows].map(pad);
  const marker = String(first.rows[0][0] ?? "");
  rows.forEach((row) => { if (row[0] === marker) row[0] = ""; });
  rows[0][0] = marker;
  return finish(sheet, ri, { start: first.start, deleteCount: second.end - first.start + 1, insert: rows }, { ...meta, headers: null });
}

const column = (i) => (i < 26 ? String.fromCharCode(65 + i) : String.fromCharCode(64 + Math.floor(i / 26)) + String.fromCharCode(65 + (i % 26)));

// A plan as Sheets API requests: { structure } for :batchUpdate (rows inserted or deleted, sent first),
// then { raw, entered } value ranges. Text goes in exactly as typed (RAW: no formulas, dates or number
// formats), Q# as a number like the rest of the column (USER_ENTERED). A–J are written as one block,
// author and time into their own columns (tab.stamps); other columns of the tab are left alone.
// name: the tab's name quoted for a range.
export function sheetWrites(tab, name, { start, deleteCount, insert, fixups }) {
  const structure = [];
  const diff = insert.length - deleteCount;
  if (diff > 0) {
    const at = start + deleteCount;
    structure.push({ insertDimension: { range: { sheetId: tab.sheetId, dimension: "ROWS", startIndex: at, endIndex: at + diff }, inheritFromBefore: at > 0 } });
  }
  if (diff < 0) {
    structure.push({ deleteDimension: { range: { sheetId: tab.sheetId, dimension: "ROWS", startIndex: start + insert.length, endIndex: start + deleteCount } } });
  }

  const raw = [];
  const entered = [];
  const sheetCol = (col) => (col === AUTHOR_COL ? tab.stamps.author : col === CHANGED_COL ? tab.stamps.changed : col);
  const end = start + insert.length;
  if (insert.length) {
    raw.push({ range: `${name}!A${start + 1}:${column(QUESTION_COLUMNS - 1)}${end}`, values: insert.map((row) => pad(row).slice(0, QUESTION_COLUMNS)) });
    [AUTHOR_COL, CHANGED_COL].forEach((col) => {
      const at = column(sheetCol(col));
      raw.push({ range: `${name}!${at}${start + 1}:${at}${end}`, values: insert.map((row) => [pad(row)[col]]) });
    });
  }
  insert.forEach((row, i) => entered.push({ range: `${name}!B${start + i + 1}`, values: [[pad(row)[1]]] }));
  fixups.forEach((f) => (f.col === 1 ? entered : raw).push({ range: `${name}!${column(sheetCol(f.col))}${f.row + 1}`, values: [[f.value]] }));
  return { structure, raw, entered };
}
