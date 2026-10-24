import { describe, it, expect } from "vitest";
import { playsHere, notePresenter, playPack } from "./sounds";

describe("which window plays the sounds", () => {
  it("is the presenter window when it is open, else the TV window", () => {
    notePresenter(false);
    expect(playsHere(false)).toBe(true);
    expect(playsHere(true)).toBe(true);
    notePresenter();
    expect(playsHere(false)).toBe(false);
    expect(playsHere(true)).toBe(true);
    notePresenter(false);
    expect(playsHere(false)).toBe(true);
  });

  it("does nothing where there is no audio", () => {
    expect(playPack("chimes", "answer")).toBe(false);
  });
});
