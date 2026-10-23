// Unfold a setup page section by its id and scroll to it (the setup checklist's steps)
export const OPEN_SECTION = "quiznight:open-section";
export const openSection = (id) => window.dispatchEvent(new CustomEvent(OPEN_SECTION, { detail: id }));
