import { ArrowLeft, X } from "lucide-react";
import { quizHash } from "../hooks/useHashRoute";

// Host tools (attendance, scores) usually open in their own window from the presenter: there the button
// closes that window. Opened inside the app it goes back to the previous page, or to the quiz setup.
export default function BackButton({ quizId, t }) {
  const isPopup = Boolean(window.opener) && window.name !== "";
  const goBack = () => {
    if (isPopup) window.close();
    else if (window.history.length > 1) window.history.back();
    else window.location.hash = quizHash(quizId, "edit");
  };
  return (
    <button onClick={goBack} className="flex items-center gap-2 text-gray-400 hover:text-white">
      {isPopup ? <X size={18} /> : <ArrowLeft size={18} />} {isPopup ? t.chk_close : t.nav_back}
    </button>
  );
}
