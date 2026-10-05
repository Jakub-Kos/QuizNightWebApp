import { useRef, useState } from "react";
import { Upload, FolderOpen, Files } from "lucide-react";
import { filesFromDrop } from "../quiz/files";

// Drop zone that accepts files or whole folders, plus buttons for both
export default function FileDrop({ onFiles, accept, label, t, compact = false }) {
  const filesInput = useRef(null);
  const folderInput = useRef(null);
  const [over, setOver] = useState(false);

  const handleDrop = async (e) => {
    e.preventDefault();
    setOver(false);
    const files = await filesFromDrop(e.dataTransfer);
    if (files.length) onFiles(files);
  };
  const handleInput = (e) => {
    const files = [...e.target.files];
    e.target.value = "";
    if (files.length) onFiles(files);
  };

  return (
    <div
      onDragOver={(e) => { e.preventDefault(); setOver(true); }}
      onDragLeave={() => setOver(false)}
      onDrop={handleDrop}
      className={`rounded-2xl border-2 border-dashed transition-colors flex flex-col items-center justify-center gap-4 text-center ${compact ? "p-5" : "p-10"} ${over ? "border-yellow-400 bg-yellow-400/10" : "border-white/15 bg-white/[0.02]"}`}
    >
      <Upload className="text-gray-500" size={compact ? 24 : 36} />
      <p className="text-gray-300">{label}</p>
      <div className="flex flex-wrap justify-center gap-2">
        <button onClick={() => filesInput.current.click()} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">
          <Files size={16} /> {t.lib_import_files}
        </button>
        <button onClick={() => folderInput.current.click()} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">
          <FolderOpen size={16} /> {t.lib_import_folder}
        </button>
      </div>
      <input ref={filesInput} type="file" multiple accept={accept} hidden onChange={handleInput} />
      <input ref={folderInput} type="file" webkitdirectory="" hidden onChange={handleInput} />
    </div>
  );
}
