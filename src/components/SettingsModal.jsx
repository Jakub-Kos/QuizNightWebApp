import { motion } from "framer-motion";
import { X, Save, Clock, Zap, Maximize, Globe, ChevronUp, ChevronDown } from "lucide-react";

export default function SettingsModal({ onClose, config, onUpdate, t }) {
  const handleChange = (key, value) => {
    onUpdate({ ...config, [key]: value });
  };

  // --- CUSTOM TIME PICKER LOGIC ---
  const [hours, minutes] = (config.startTime || "20:00").split(":").map(Number);

  // Handle Arrow Clicks
  const updateTime = (type, change) => {
    let newH = hours;
    let newM = minutes;

    if (type === "h") {
      newH = (hours + change + 24) % 24;
    } else {
      newM = (minutes + change + 60) % 60;
    }
    saveTime(newH, newM);
  };

  // Handle Direct Typing
  const handleInput = (type, value) => {
    let val = parseInt(value);
    if (isNaN(val)) return; // Ignore empty/invalid

    if (type === "h") {
      val = Math.max(0, Math.min(23, val)); // Clamp 0-23
      saveTime(val, minutes);
    } else {
      val = Math.max(0, Math.min(59, val)); // Clamp 0-59
      saveTime(hours, val);
    }
  };

  // Helper to save back to config string
  const saveTime = (h, m) => {
    const timeString = `${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}`;
    handleChange("startTime", timeString);
  };

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <motion.div
        className="bg-[#053049] w-full max-w-lg rounded-2xl border border-gray-700 shadow-2xl overflow-hidden flex flex-col max-h-[90vh]"
        initial={{ scale: 0.9, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0.9, opacity: 0 }}
      >
        {/* --- HEADER --- */}
        <div className="p-6 border-b border-gray-700 flex justify-between items-center bg-black/20">
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <Zap size={20} className="text-yellow-400"/> {t.settings_title}
          </h2>
          <button onClick={onClose} className="text-gray-400 hover:text-white transition-colors bg-white/5 hover:bg-white/10 p-2 rounded-full"><X size={20} /></button>
        </div>

        {/* --- SCROLLABLE BODY --- */}
        <div className="p-6 space-y-8 overflow-y-auto custom-scrollbar">

          {/* LANGUAGE SWITCHER */}
          <div className="space-y-4">
             <h3 className="text-xs font-bold text-gray-400 uppercase tracking-widest flex items-center gap-2">
              <Globe size={14} /> {t.language}
            </h3>
            <div className="flex gap-2">
                <button
                    onClick={() => handleChange("language", "cs")}
                    className={`flex-1 py-3 rounded-xl border font-bold transition-all ${config.language === "cs" ? "bg-yellow-500 text-black border-yellow-500" : "bg-black/20 text-gray-400 border-white/5 hover:bg-white/5"}`}
                >
                    Čeština 🇨🇿
                </button>
                <button
                    onClick={() => handleChange("language", "sk")}
                    className={`flex-1 py-3 rounded-xl border font-bold transition-all ${config.language === "sk" ? "bg-red-600 text-white border-red-500" : "bg-black/20 text-gray-400 border-white/5 hover:bg-white/5"}`}
                >
                    Slovenčina 🇸🇰
                </button>
                <button
                    onClick={() => handleChange("language", "en")}
                    className={`flex-1 py-3 rounded-xl border font-bold transition-all ${config.language === "en" ? "bg-blue-600 text-white border-blue-500" : "bg-black/20 text-gray-400 border-white/5 hover:bg-white/5"}`}
                >
                    English 🇺🇸
                </button>
            </div>
          </div>

          {/* DISPLAY & TIME SECTION */}
          <div className="space-y-4 pt-4 border-t border-gray-700">
            <h3 className="text-xs font-bold text-gray-400 uppercase tracking-widest flex items-center gap-2">
              <Clock size={14} /> {t.display_options}
            </h3>

            {/* Toggle: Show Clock */}
            <div className="flex items-center justify-between bg-black/20 p-4 rounded-xl border border-white/5">
              <span className="text-gray-200 font-medium">{t.show_clock}</span>
              <button
                onClick={() => handleChange("showTime", !config.showTime)}
                className={`w-14 h-8 rounded-full p-1 transition-all duration-300 ${config.showTime ? "bg-green-500 shadow-[0_0_10px_rgba(34,197,94,0.4)]" : "bg-gray-700"}`}
              >
                <div className={`w-6 h-6 bg-white rounded-full shadow-md transition-transform duration-300 ${config.showTime ? "translate-x-6" : "translate-x-0"}`} />
              </button>
            </div>

            {/* 👇 IMPROVED INTERACTIVE CLOCK WIDGET 👇 */}
            <div className="bg-black/20 p-4 rounded-xl border border-white/5 flex flex-col items-center">
               <label className="text-xs font-bold text-gray-400 uppercase tracking-widest mb-4 self-start">{t.start_time}</label>

               <div className="flex items-center gap-4 bg-black/40 px-6 py-4 rounded-2xl border border-white/10 shadow-inner">

                   {/* HOURS */}
                   <div className="flex flex-col items-center gap-1">
                       <button onClick={() => updateTime('h', 1)} className="p-2 text-yellow-500/50 hover:text-yellow-400 hover:bg-white/5 rounded-full transition-all"><ChevronUp size={24} /></button>
                       <input
                          type="number"
                          min="0" max="23"
                          value={hours.toString().padStart(2, '0')}
                          onChange={(e) => handleInput('h', e.target.value)}
                          onBlur={(e) => saveTime(hours, minutes)} // Force re-format on blur
                          className="w-20 bg-transparent text-center font-mono text-5xl font-bold text-white focus:outline-none focus:text-yellow-400 appearance-none"
                          style={{ MozAppearance: "textfield" }} // Hides spinner in Firefox
                       />
                       <button onClick={() => updateTime('h', -1)} className="p-2 text-yellow-500/50 hover:text-yellow-400 hover:bg-white/5 rounded-full transition-all"><ChevronDown size={24} /></button>
                   </div>

                   {/* SEPARATOR */}
                   <span className="text-4xl font-bold text-yellow-500/50 pb-2 animate-pulse">:</span>

                   {/* MINUTES */}
                   <div className="flex flex-col items-center gap-1">
                       <button onClick={() => updateTime('m', 1)} className="p-2 text-yellow-500/50 hover:text-yellow-400 hover:bg-white/5 rounded-full transition-all"><ChevronUp size={24} /></button>
                       <input
                          type="number"
                          min="0" max="59"
                          value={minutes.toString().padStart(2, '0')}
                          onChange={(e) => handleInput('m', e.target.value)}
                          onBlur={(e) => saveTime(hours, minutes)} // Force re-format on blur
                          className="w-20 bg-transparent text-center font-mono text-5xl font-bold text-white focus:outline-none focus:text-yellow-400 appearance-none"
                          style={{ MozAppearance: "textfield" }}
                       />
                       <button onClick={() => updateTime('m', -1)} className="p-2 text-yellow-500/50 hover:text-yellow-400 hover:bg-white/5 rounded-full transition-all"><ChevronDown size={24} /></button>
                   </div>
               </div>
            </div>

          </div>

          {/* ANIMATION TIMING */}
          <div className="space-y-4 pt-4 border-t border-gray-700">
            <h3 className="text-xs font-bold text-gray-400 uppercase tracking-widest flex items-center gap-2">
              <Zap size={14} /> {t.anim_speed}
            </h3>

            {/* Header Rotation Slider */}
            <div className="bg-black/20 p-4 rounded-xl border border-white/5">
              <div className="flex justify-between text-gray-300 text-sm mb-3">
                <span>{t.header_rot}</span>
                <span className="font-mono text-pink-400">{config.headerInterval ? config.headerInterval / 1000 : 5}s</span>
              </div>
              <input
                type="range" min="3000" max="10000" step="1000"
                value={config.headerInterval || 5000}
                onChange={(e) => handleChange("headerInterval", parseInt(e.target.value))}
                className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-pink-500"
              />
            </div>

            {/* Split Delay Slider */}
            <div className="bg-black/20 p-4 rounded-xl border border-white/5">
              <div className="flex justify-between text-gray-300 text-sm mb-3">
                <span>{t.read_time}</span>
                <span className="font-mono text-yellow-400">{config.splitDelay / 1000}s</span>
              </div>
              <input
                type="range" min="1000" max="8000" step="500"
                value={config.splitDelay}
                onChange={(e) => handleChange("splitDelay", parseInt(e.target.value))}
                className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-yellow-500"
              />
            </div>

            {/* Cycle Duration Slider */}
            <div className="bg-black/20 p-4 rounded-xl border border-white/5">
              <div className="flex justify-between text-gray-300 text-sm mb-3">
                <span>{t.cycle_time}</span>
                <span className="font-mono text-blue-400">{config.cycleDuration / 1000}s</span>
              </div>
              <input
                type="range" min="4000" max="15000" step="1000"
                value={config.cycleDuration}
                onChange={(e) => handleChange("cycleDuration", parseInt(e.target.value))}
                className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-500"
              />
            </div>
          </div>
        </div>

        {/* --- FOOTER --- */}
        <div className="p-4 bg-black/40 text-right border-t border-gray-700">
          <button onClick={onClose} className="px-6 py-3 bg-yellow-500 hover:bg-yellow-400 text-black font-bold rounded-xl flex items-center gap-2 ml-auto hover:scale-105 transition-transform">
            <Save size={18} /> {t.save_close}
          </button>
        </div>
      </motion.div>
    </div>
  );
}