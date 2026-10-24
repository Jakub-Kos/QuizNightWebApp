import { createContext, useContext } from "react";

// The theme's sounds (theme.sounds.pack), made by the browser itself with the Web Audio API: no sound
// files, so nothing to license or download. Events: "round" (a round's title card), "answer" (an answer
// is revealed), "reveal" (a group of teams on the leaderboard) and "winner" (the final standings).
// Both show windows see every event, but only one plays it: the presenter window when it is open (it is
// the one the host clicks in, so the browser lets it play sound), otherwise the TV window.

export const SOUND_EVENTS = ["round", "answer", "reveal", "winner"];

let ctx = null;
const audio = () => {
  if (!ctx && typeof window !== "undefined" && window.AudioContext) ctx = new window.AudioContext();
  return ctx;
};

// Browsers start audio only after the page was clicked or a key was pressed
if (typeof window !== "undefined") {
  const unlock = () => { if (audio()?.state === "suspended") ctx.resume().catch(() => {}); };
  window.addEventListener("pointerdown", unlock);
  window.addEventListener("keydown", unlock);
}

// The TV window plays only while no presenter window has said it is open in the last 30 s
let presenterSeen = 0;
export const notePresenter = (here = true) => { presenterSeen = here ? Date.now() : 0; };
export const playsHere = (isPresenter) => isPresenter || Date.now() - presenterSeen > 30_000;

// One note: frequency in Hz (or [from, to] for a glide), start and length in seconds from now
function tone(out, freq, start, length, { type = "sine", gain = 0.3, attack = 0.01, vibrato = 0, filter = null } = {}) {
  const c = out.context;
  const t0 = c.currentTime + start;
  const osc = c.createOscillator();
  osc.type = type;
  const [from, to] = Array.isArray(freq) ? freq : [freq, freq];
  osc.frequency.setValueAtTime(from, t0);
  if (to !== from) osc.frequency.exponentialRampToValueAtTime(to, t0 + length * 0.8);
  if (vibrato) {
    const lfo = c.createOscillator();
    const depth = c.createGain();
    lfo.frequency.value = 6;
    depth.gain.value = vibrato;
    lfo.connect(depth).connect(osc.frequency);
    lfo.start(t0);
    lfo.stop(t0 + length + 0.1);
  }
  const env = c.createGain();
  env.gain.setValueAtTime(0, t0);
  env.gain.linearRampToValueAtTime(gain, t0 + attack);
  env.gain.exponentialRampToValueAtTime(0.0001, t0 + length);
  let node = osc;
  if (filter) {
    const f = c.createBiquadFilter();
    f.type = "lowpass";
    f.frequency.value = filter;
    node = node.connect(f);
  }
  node.connect(env).connect(out);
  osc.start(t0);
  osc.stop(t0 + length + 0.05);
}

// A short burst of filtered noise (a jingle, a whoosh)
function noise(out, start, length, { gain = 0.15, highpass = 6000 } = {}) {
  const c = out.context;
  const t0 = c.currentTime + start;
  const buffer = c.createBuffer(1, Math.ceil(c.sampleRate * length), c.sampleRate);
  const data = buffer.getChannelData(0);
  for (let i = 0; i < data.length; i++) data[i] = (Math.random() * 2 - 1) * (1 - i / data.length);
  const src = c.createBufferSource();
  src.buffer = buffer;
  const f = c.createBiquadFilter();
  f.type = "highpass";
  f.frequency.value = highpass;
  const env = c.createGain();
  env.gain.value = gain;
  src.connect(f).connect(env).connect(out);
  src.start(t0);
}

// A bell: a tone with an inharmonic overtone
const bell = (out, freq, start, length, gain = 0.25) => {
  tone(out, freq, start, length, { gain });
  tone(out, freq * 2.76, start, length * 0.5, { gain: gain * 0.3 });
};
const note = (n) => 440 * 2 ** ((n - 69) / 12); // MIDI note number -> Hz

const PACKS = {
  chimes: {
    round: (o) => { bell(o, note(72), 0, 1.2); bell(o, note(79), 0.18, 1.4); },
    answer: (o) => { bell(o, note(84), 0, 1.6, 0.3); bell(o, note(88), 0.08, 1.4, 0.18); },
    reveal: (o) => { noise(o, 0, 0.35, { gain: 0.06, highpass: 3000 }); bell(o, note(76), 0.25, 1.2); },
    winner: (o) => {
      [72, 76, 79, 84].forEach((n, i) => bell(o, note(n), i * 0.14, 1.6));
      [84, 88, 91].forEach((n, i) => bell(o, note(n), 0.7 + i * 0.05, 2.2, 0.18));
    },
  },
  spooky: {
    round: (o) => { tone(o, [note(45), note(57)], 0, 1.6, { type: "sawtooth", gain: 0.16, attack: 0.3, filter: 700 }); tone(o, note(76), 0.4, 1.8, { gain: 0.12, vibrato: 12, attack: 0.3 }); },
    answer: (o) => { tone(o, [note(69), note(81)], 0, 1.2, { gain: 0.18, vibrato: 14, attack: 0.08 }); },
    reveal: (o) => { [57, 60, 63].forEach((n) => tone(o, note(n), 0, 1.4, { type: "sawtooth", gain: 0.08, attack: 0.15, filter: 900 })); },
    winner: (o) => {
      [45, 48, 52, 57].forEach((n) => tone(o, note(n), 0, 2.6, { type: "sawtooth", gain: 0.07, attack: 0.4, filter: 1100 }));
      tone(o, [note(69), note(93)], 0.3, 2.2, { gain: 0.16, vibrato: 18, attack: 0.2 });
      noise(o, 1.6, 0.8, { gain: 0.05, highpass: 1500 });
    },
  },
  festive: {
    round: (o) => { noise(o, 0, 0.12); noise(o, 0.15, 0.12); bell(o, note(79), 0.05, 1.2); bell(o, note(84), 0.25, 1.4); },
    answer: (o) => { noise(o, 0, 0.1); bell(o, note(88), 0, 1.4, 0.28); },
    reveal: (o) => { [0, 0.12, 0.24].forEach((s) => noise(o, s, 0.1, { gain: 0.1 })); bell(o, note(81), 0.3, 1.2); },
    winner: (o) => {
      [76, 76, 76, 76, 76, 76, 76, 79, 72, 74, 76].forEach((n, i) => bell(o, note(n + 12), [0, 0.18, 0.36, 0.72, 0.9, 1.08, 1.44, 1.62, 1.8, 1.98, 2.16][i], 0.6, 0.18));
      [0, 0.36, 0.72, 1.08, 1.44].forEach((s) => noise(o, s, 0.12, { gain: 0.09 }));
    },
  },
  arcade: {
    round: (o) => { [60, 64, 67, 72].forEach((n, i) => tone(o, note(n), i * 0.07, 0.12, { type: "square", gain: 0.08 })); },
    answer: (o) => { tone(o, note(84), 0, 0.08, { type: "square", gain: 0.09 }); tone(o, note(91), 0.08, 0.22, { type: "square", gain: 0.09 }); },
    reveal: (o) => { tone(o, [note(48), note(84)], 0, 0.35, { type: "square", gain: 0.07 }); },
    winner: (o) => {
      [[72, 0], [72, 0.12], [72, 0.24], [72, 0.4], [68, 0.56], [70, 0.72], [72, 0.88], [70, 1.04], [72, 1.16]].forEach(([n, s]) => tone(o, note(n), s, 0.16, { type: "square", gain: 0.08 }));
      tone(o, note(48), 0, 1.4, { type: "triangle", gain: 0.12 });
    },
  },
};

// Evens out the packs' loudness (square waves sound louder than their level suggests)
const PACK_LEVEL = { chimes: 1, spooky: 1.7, festive: 1, arcade: 1.4 };

// Plays a pack's sound now (volume 0–1); returns false when the browser has no audio or it is not
// unlocked yet
export function playPack(pack, event, volume = 0.6) {
  const c = audio();
  if (!c || !PACKS[pack]?.[event] || volume <= 0) return false;
  if (c.state === "suspended") c.resume().catch(() => {});
  const out = c.createGain();
  out.gain.value = volume * (PACK_LEVEL[pack] ?? 1);
  out.connect(c.destination);
  PACKS[pack][event](out);
  return true;
}

// For a show window: play(event) plays the theme's sound if this window is the one that plays
export function soundPlayer({ pack, volume, enabled, isPresenter }) {
  return (event) => {
    if (enabled && pack && playsHere(isPresenter)) playPack(pack, event, volume);
  };
}

export const SoundContext = createContext(() => {});
export const useSound = () => useContext(SoundContext);
