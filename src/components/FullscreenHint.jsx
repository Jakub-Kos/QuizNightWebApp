import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Maximize, X } from "lucide-react";
import { useIsFullscreen, enterFullscreen, FULLSCREEN_KEY } from "../hooks/useFullscreen";

// Rendered outside the Stage so it stays readable regardless of scaling
export default function FullscreenHint({ t }) {
  const isFullscreen = useIsFullscreen();
  const [dismissed, setDismissed] = useState(false);
  const visible = !isFullscreen && !dismissed;

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          initial={{ opacity: 0, y: -20, x: "-50%" }}
          animate={{ opacity: 1, y: 0, x: "-50%" }}
          exit={{ opacity: 0, y: -20, x: "-50%" }}
          className="fixed top-4 left-1/2 z-[250] flex items-center gap-4 pl-5 pr-2 py-2 rounded-full bg-[#053049]/95 backdrop-blur-md border border-yellow-500/40 shadow-2xl text-white font-sans text-sm"
        >
          <span>
            {t.fs_hint_before} <kbd className="mx-1 px-2 py-0.5 rounded bg-white/15 border border-white/20 font-mono font-bold text-yellow-400">{FULLSCREEN_KEY}</kbd> {t.fs_hint_after}
          </span>
          <button onClick={enterFullscreen} className="flex items-center gap-2 px-4 py-2 rounded-full bg-yellow-500 hover:bg-yellow-400 text-black font-bold transition-colors">
            <Maximize size={16} /> {t.calib_fullscreen}
          </button>
          <button onClick={() => setDismissed(true)} className="p-2 rounded-full text-gray-400 hover:text-white hover:bg-white/10 transition-colors" aria-label="Close">
            <X size={16} />
          </button>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
